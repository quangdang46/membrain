use membrain_daemon::daemon::{DaemonRuntime, DaemonRuntimeConfig};
use serde_json::{json, Value};
use std::path::{Path, PathBuf};
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::UnixStream;
use tokio::time::timeout;

fn unique_socket_path(label: &str) -> PathBuf {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("clock should be after epoch")
        .as_nanos();
    std::env::temp_dir().join(format!(
        "membrain-daemon-{label}-{}-{nanos}.sock",
        std::process::id()
    ))
}

async fn send_request(socket_path: &Path, request: Value) -> Value {
    let stream = UnixStream::connect(socket_path)
        .await
        .expect("daemon socket should accept connections");
    let (reader, mut writer) = stream.into_split();
    writer
        .write_all(request.to_string().as_bytes())
        .await
        .expect("request should write");
    writer.write_all(b"\n").await.expect("newline should write");
    writer.flush().await.expect("request should flush");

    let mut line = String::new();
    let mut reader = BufReader::new(reader);
    reader
        .read_line(&mut line)
        .await
        .expect("response should read");
    serde_json::from_str(&line).expect("daemon response should be valid json")
}

async fn spawn_runtime(label: &str) -> (PathBuf, tokio::task::JoinHandle<anyhow::Result<()>>) {
    let socket_path = unique_socket_path(label);
    let mut config = DaemonRuntimeConfig::new(&socket_path);
    config.maintenance_interval = Duration::from_secs(3600);
    let runtime = DaemonRuntime::with_config(config);
    let handle = tokio::spawn(async move { runtime.run_until_stopped().await });

    timeout(Duration::from_secs(2), async {
        while tokio::fs::metadata(&socket_path).await.is_err() {
            tokio::task::yield_now().await;
        }
    })
    .await
    .expect("daemon socket should appear");

    (socket_path, handle)
}

async fn shutdown_runtime(socket_path: &Path, handle: tokio::task::JoinHandle<anyhow::Result<()>>) {
    let shutdown_response = send_request(
        socket_path,
        json!({"jsonrpc":"2.0","method":"shutdown","params":{},"id":"shutdown"}),
    )
    .await;
    assert_eq!(shutdown_response["result"]["shutting_down"], json!(true));
    timeout(Duration::from_secs(2), handle)
        .await
        .expect("daemon task should finish")
        .expect("join should succeed")
        .expect("runtime should stop cleanly");
}

#[tokio::test]
async fn runtime_share_and_unshare_preserve_sharing_scope_contract() {
    let (socket_path, handle) = spawn_runtime("share-unshare-parity").await;

    let share_response = send_request(
        &socket_path,
        json!({
            "jsonrpc":"2.0",
            "method":"share",
            "params":{"id":42,"namespace_id":"team.beta"},
            "id":"share"
        }),
    )
    .await;
    assert_eq!(share_response["result"]["status"], json!("accepted"));
    assert_eq!(share_response["result"]["visibility"], json!("shared"));
    assert_eq!(
        share_response["result"]["policy_summary"]["sharing_scope"],
        json!("shared")
    );
    assert_eq!(
        share_response["result"]["policy_filters_applied"][0]["sharing_scope"],
        json!("shared")
    );
    assert_eq!(
        share_response["result"]["policy_summary"]["redaction_fields"],
        json!([])
    );
    assert_eq!(
        share_response["result"]["audit"]["request_id"],
        json!("req-share-42")
    );
    assert_eq!(share_response["result"]["audit"]["redacted"], json!(false));

    let unshare_response = send_request(
        &socket_path,
        json!({
            "jsonrpc":"2.0",
            "method":"unshare",
            "params":{"id":42,"namespace":"team.alpha"},
            "id":"unshare"
        }),
    )
    .await;
    assert_eq!(unshare_response["result"]["status"], json!("accepted"));
    assert_eq!(unshare_response["result"]["visibility"], json!("private"));
    assert_eq!(
        unshare_response["result"]["policy_summary"]["sharing_scope"],
        json!("private")
    );
    assert_eq!(
        unshare_response["result"]["policy_filters_applied"][0]["sharing_scope"],
        json!("private")
    );
    assert_eq!(
        unshare_response["result"]["policy_summary"]["redaction_fields"],
        json!(["sharing_scope"])
    );
    assert_eq!(
        unshare_response["result"]["policy_filters_applied"][0]["redaction_fields"],
        json!(["sharing_scope"])
    );
    assert_eq!(
        unshare_response["result"]["audit"]["request_id"],
        json!("req-unshare-42")
    );
    assert_eq!(unshare_response["result"]["audit"]["redacted"], json!(true));

    let invalid_unshare = send_request(
        &socket_path,
        json!({
            "jsonrpc":"2.0",
            "method":"unshare",
            "params":{"id":42,"namespace":"bad namespace"},
            "id":"unshare-invalid"
        }),
    )
    .await;
    assert_eq!(invalid_unshare["error"]["code"], json!(-32602));
    assert_eq!(
        invalid_unshare["error"]["message"],
        json!("malformed namespace")
    );
    assert_eq!(
        invalid_unshare["error"]["data"]["error_kind"],
        json!("validation_failure")
    );

    shutdown_runtime(&socket_path, handle).await;
}
