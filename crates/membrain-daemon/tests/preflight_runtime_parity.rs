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
async fn preflight_explain_and_allow_keep_blocked_scope_parity() {
    let (socket_path, handle) = spawn_runtime("preflight-scope-parity").await;

    let explain_response = send_request(
        &socket_path,
        json!({
            "jsonrpc":"2.0",
            "method":"preflight.explain",
            "params":{
                "namespace":"team.alpha",
                "original_query":"delete prior audit events across all namespaces",
                "proposed_action":"purge namespace audit history"
            },
            "id":"preflight-explain-scope"
        }),
    )
    .await;
    let allow_response = send_request(
        &socket_path,
        json!({
            "jsonrpc":"2.0",
            "method":"preflight.allow",
            "params":{
                "namespace":"team.alpha",
                "original_query":"delete prior audit events across all namespaces",
                "proposed_action":"purge namespace audit history",
                "authorization_token":"allow-123",
                "bypass_flags":["manual_override"]
            },
            "id":"preflight-allow-scope"
        }),
    )
    .await;

    assert_eq!(explain_response["result"]["allowed"], json!(false));
    assert_eq!(
        explain_response["result"]["preflight_state"],
        json!("blocked")
    );
    assert_eq!(
        explain_response["result"]["preflight_outcome"],
        json!("preview_only")
    );
    assert_eq!(
        explain_response["result"]["blocked_reasons"],
        json!(["scope_ambiguous", "confirmation_required"])
    );
    assert_eq!(
        explain_response["result"]["blocked_reason"],
        json!("requested scope is ambiguous")
    );
    let explain_checks = explain_response["result"]["check_results"]
        .as_array()
        .expect("check results should be an array");
    assert!(explain_checks.iter().any(|check| {
        check["check_name"] == json!("required_input")
            && check["checked_scope"] == json!("request_scope")
    }));
    assert_eq!(
        explain_response["result"]["policy_summary"]["decision"],
        json!("allow")
    );
    assert_eq!(
        explain_response["result"]["policy_summary"]["outcome_class"],
        json!("accepted")
    );
    assert_eq!(
        explain_response["result"]["confirmation"]["confirmed"],
        json!(false)
    );
    assert!(explain_response["result"]["request_id"]
        .as_str()
        .expect("request id should be present")
        .starts_with("daemon-preflight-explain-"));
    assert!(explain_response["result"]["preflight_id"]
        .as_str()
        .expect("preflight id should be present")
        .starts_with("preflight-"));

    assert_eq!(allow_response["result"]["success"], json!(false));
    assert_eq!(
        allow_response["result"]["preflight_state"],
        json!("missing_data")
    );
    assert_eq!(
        allow_response["result"]["preflight_outcome"],
        json!("blocked")
    );
    assert_eq!(allow_response["result"]["outcome_class"], json!("blocked"));
    assert_eq!(
        allow_response["result"]["blocked_reasons"],
        json!(["scope_ambiguous"])
    );
    assert_eq!(
        allow_response["result"]["policy_summary"]["decision"],
        json!("allow")
    );
    assert_eq!(
        allow_response["result"]["policy_summary"]["outcome_class"],
        json!("accepted")
    );
    assert_eq!(
        allow_response["result"]["confirmation"]["confirmed"],
        json!(true)
    );
    assert!(allow_response["result"]
        .get("confirmation_reason")
        .is_none());
    assert!(allow_response["result"].get("execution_id").is_none());

    shutdown_runtime(&socket_path, handle).await;
}

#[tokio::test]
async fn preflight_run_and_explain_keep_degraded_serialization_parity() {
    let (socket_path, handle) = spawn_runtime("preflight-degraded-parity").await;

    let run_response = send_request(
        &socket_path,
        json!({
            "jsonrpc":"2.0",
            "method":"preflight.run",
            "params":{
                "namespace":"team.alpha",
                "original_query":"show degraded fallback status because input unreadable",
                "proposed_action":"inspect maintenance fallback"
            },
            "id":"preflight-run-degraded"
        }),
    )
    .await;
    let explain_response = send_request(
        &socket_path,
        json!({
            "jsonrpc":"2.0",
            "method":"preflight.explain",
            "params":{
                "namespace":"team.alpha",
                "original_query":"show degraded fallback status because input unreadable",
                "proposed_action":"inspect maintenance fallback"
            },
            "id":"preflight-explain-degraded"
        }),
    )
    .await;

    assert_eq!(run_response["result"]["success"], json!(true));
    assert_eq!(run_response["result"]["preflight_state"], json!("ready"));
    assert_eq!(
        run_response["result"]["preflight_outcome"],
        json!("degraded")
    );
    assert_eq!(run_response["result"]["outcome_class"], json!("degraded"));
    assert_eq!(run_response["result"]["blocked_reasons"], json!([]));
    assert_eq!(run_response["result"]["degraded"], json!(true));
    let run_checks = run_response["result"]["check_results"]
        .as_array()
        .expect("check results should be an array");
    assert!(run_checks.iter().any(|check| {
        check["check_name"] == json!("authoritative_input")
            && check["status"] == json!("degraded")
            && check["reason_codes"] == json!(["authoritative_input_unreadable"])
    }));
    assert_eq!(
        run_response["result"]["policy_summary"]["decision"],
        json!("allow")
    );
    assert_eq!(
        run_response["result"]["policy_summary"]["outcome_class"],
        json!("accepted")
    );
    assert!(run_response["result"]["execution_id"]
        .as_str()
        .expect("execution id should be present")
        .starts_with("exec-"));
    assert!(run_response["result"].get("confirmation_reason").is_none());

    assert_eq!(explain_response["result"]["allowed"], json!(false));
    assert_eq!(
        explain_response["result"]["preflight_state"],
        json!("ready")
    );
    assert_eq!(
        explain_response["result"]["preflight_outcome"],
        json!("preview_only")
    );
    assert_eq!(explain_response["result"]["blocked_reasons"], json!([]));
    assert_eq!(explain_response["result"]["allowed"], json!(false));
    assert!(explain_response["result"].get("blocked_reason").is_none());
    let explain_checks = explain_response["result"]["check_results"]
        .as_array()
        .expect("check results should be an array");
    assert!(explain_checks.iter().any(|check| {
        check["check_name"] == json!("authoritative_input")
            && check["status"] == json!("degraded")
            && check["reason_codes"] == json!(["authoritative_input_unreadable"])
    }));
    assert_eq!(
        explain_response["result"]["policy_summary"]["decision"],
        json!("allow")
    );
    assert_eq!(
        explain_response["result"]["policy_summary"]["outcome_class"],
        json!("accepted")
    );
    assert!(explain_response["result"]["request_id"]
        .as_str()
        .expect("request id should be present")
        .starts_with("daemon-preflight-explain-"));
    assert!(explain_response["result"]["preflight_id"]
        .as_str()
        .expect("preflight id should be present")
        .starts_with("preflight-"));

    shutdown_runtime(&socket_path, handle).await;
}

#[tokio::test]
async fn preflight_methods_reuse_validation_error_model_for_unknown_fields() {
    let (socket_path, handle) = spawn_runtime("preflight-validation-errors").await;

    let run_response = send_request(
        &socket_path,
        json!({
            "jsonrpc":"2.0",
            "method":"preflight.run",
            "params":{
                "namespace":"team.alpha",
                "original_query":"inspect authorization state",
                "proposed_action":"preview policy gate",
                "unexpected":true
            },
            "id":"preflight-run-unknown-field"
        }),
    )
    .await;
    assert_eq!(run_response["error"]["code"], json!(-32602));
    assert_eq!(
        run_response["error"]["message"],
        json!("unknown field unexpected")
    );
    assert_eq!(run_response["error"]["data"], json!(null));

    let explain_response = send_request(
        &socket_path,
        json!({
            "jsonrpc":"2.0",
            "method":"preflight.explain",
            "params":{
                "namespace":"team.alpha",
                "original_query":"inspect authorization state",
                "proposed_action":"preview policy gate",
                "unexpected":true
            },
            "id":"preflight-explain-unknown-field"
        }),
    )
    .await;
    assert_eq!(explain_response["error"]["code"], json!(-32602));
    assert_eq!(
        explain_response["error"]["message"],
        json!("unknown field unexpected")
    );
    assert_eq!(explain_response["error"]["data"], json!(null));

    let allow_response = send_request(
        &socket_path,
        json!({
            "jsonrpc":"2.0",
            "method":"preflight.allow",
            "params":{
                "namespace":"team.alpha",
                "original_query":"inspect authorization state",
                "proposed_action":"preview policy gate",
                "authorization_token":"allow-123",
                "bypass_flags":["manual_override"],
                "unexpected":true
            },
            "id":"preflight-allow-unknown-field"
        }),
    )
    .await;
    assert_eq!(allow_response["error"]["code"], json!(-32602));
    assert_eq!(
        allow_response["error"]["message"],
        json!("unknown field unexpected")
    );
    assert_eq!(allow_response["error"]["data"], json!(null));

    shutdown_runtime(&socket_path, handle).await;
}

#[tokio::test]
async fn preflight_allow_rejects_non_string_bypass_flags_with_shared_validation_error() {
    let (socket_path, handle) = spawn_runtime("preflight-allow-invalid-bypass").await;

    let allow_response = send_request(
        &socket_path,
        json!({
            "jsonrpc":"2.0",
            "method":"preflight.allow",
            "params":{
                "namespace":"team.alpha",
                "original_query":"inspect authorization state",
                "proposed_action":"preview policy gate",
                "authorization_token":"allow-123",
                "bypass_flags":["manual_override", 7]
            },
            "id":"preflight-allow-invalid-bypass"
        }),
    )
    .await;

    assert_eq!(allow_response["error"]["code"], json!(-32602));
    assert_eq!(
        allow_response["error"]["message"],
        json!("bypass_flags must be an array of strings")
    );
    assert_eq!(allow_response["error"]["data"], json!(null));

    shutdown_runtime(&socket_path, handle).await;
}
