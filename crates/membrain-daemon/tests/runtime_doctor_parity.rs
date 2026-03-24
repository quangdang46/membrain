use membrain_daemon::daemon::{DaemonRuntime, DaemonRuntimeConfig};
use membrain_daemon::rpc::{RuntimeMetrics, RuntimePosture};
use serde_json::{json, Value};
use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::UnixStream;
use tokio::time::timeout;

fn unique_socket_path(label: &str) -> PathBuf {
    static NEXT_SOCKET_ID: AtomicU64 = AtomicU64::new(1);

    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("clock should be after epoch")
        .as_nanos();
    let unique_id = NEXT_SOCKET_ID.fetch_add(1, Ordering::Relaxed);
    std::env::temp_dir().join(format!(
        "membrain-daemon-{label}-{}-{nanos}-{unique_id}.sock",
        std::process::id()
    ))
}

async fn send_request(socket_path: &std::path::Path, request: Value) -> Value {
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

async fn spawn_runtime() -> (PathBuf, tokio::task::JoinHandle<anyhow::Result<()>>) {
    let socket_path = unique_socket_path("doctor-parity");
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

async fn shutdown_runtime(
    socket_path: &std::path::Path,
    handle: tokio::task::JoinHandle<anyhow::Result<()>>,
) {
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
async fn runtime_doctor_rejects_unknown_params_like_other_zero_arg_methods() {
    let (socket_path, handle) = spawn_runtime().await;

    let doctor_response = send_request(
        &socket_path,
        json!({
            "jsonrpc":"2.0",
            "method":"doctor",
            "params":{"unexpected":true},
            "id":"doctor-invalid"
        }),
    )
    .await;

    assert_eq!(doctor_response["error"]["code"], json!(-32602));
    assert_eq!(
        doctor_response["error"]["message"],
        json!("unknown field unexpected")
    );
    assert_eq!(doctor_response["error"]["data"], json!(null));

    shutdown_runtime(&socket_path, handle).await;
}

#[tokio::test]
async fn runtime_doctor_jsonrpc_response_matches_shared_doctor_contract_fields() {
    let (socket_path, handle) = spawn_runtime().await;

    let posture_response = send_request(
        &socket_path,
        json!({
            "jsonrpc":"2.0",
            "method":"set_posture",
            "params":{"posture":"degraded","reasons":["repair_in_flight"]},
            "id":"posture"
        }),
    )
    .await;
    assert_eq!(posture_response["result"]["posture"], json!("degraded"));

    let doctor_response = send_request(
        &socket_path,
        json!({"jsonrpc":"2.0","method":"doctor","params":{},"id":"doctor"}),
    )
    .await;

    let result = &doctor_response["result"];
    assert_eq!(result["status"], json!("warn"));
    assert_eq!(result["action"], json!("doctor"));
    assert_eq!(result["posture"], json!(RuntimePosture::Degraded.as_str()));
    assert_eq!(result["degraded_reasons"], json!(["repair_in_flight"]));
    let metrics = serde_json::from_value::<RuntimeMetrics>(result["metrics"].clone())
        .expect("doctor metrics should match runtime metrics schema");
    assert_eq!(metrics.queue_depth, 0);
    assert!(metrics.active_requests >= 1);
    assert_eq!(metrics.background_jobs, 0);
    assert_eq!(metrics.cancelled_requests, 0);
    assert_eq!(metrics.maintenance_runs, 0);
    assert_eq!(
        result["warnings"]
            .as_array()
            .expect("warnings should be an array"),
        &vec![json!("operator_review_recommended")]
    );
    let indexes = result["indexes"]
        .as_array()
        .expect("indexes should be an array");
    assert_eq!(indexes.len(), 4);
    assert_eq!(indexes[0]["family"], json!("schema"));
    assert_eq!(indexes[3]["family"], json!("cache"));
    assert_eq!(indexes[3]["health"], json!("warn"));
    assert_eq!(indexes[3]["usable"], json!(true));

    assert_eq!(result["action"], json!("doctor"));
    assert_eq!(result["posture"], json!("degraded"));
    assert_eq!(result["degraded_reasons"], json!(["repair_in_flight"]));
    assert!(result["metrics"].is_object());
    assert!(result["indexes"].is_array());
    assert!(result["warnings"].is_array());
    assert!(result.get("health").is_none());

    shutdown_runtime(&socket_path, handle).await;
}

#[tokio::test]
async fn runtime_doctor_resource_read_matches_runtime_doctor_payload_shape() {
    let (socket_path, handle) = spawn_runtime().await;

    let doctor_resource = send_request(
        &socket_path,
        json!({
            "jsonrpc":"2.0",
            "method":"resource.read",
            "params":{"uri":"membrain://daemon/runtime/doctor"},
            "id":"resource-read-doctor"
        }),
    )
    .await;

    let result = &doctor_resource["result"];
    assert_eq!(result["status"], json!("ok"));
    assert_eq!(
        result["payload"]["uri"],
        json!("membrain://daemon/runtime/doctor")
    );
    assert_eq!(result["payload"]["resource_kind"], json!("runtime_doctor"));
    assert_eq!(result["payload"]["payload"]["action"], json!("doctor"));
    assert_eq!(result["payload"]["payload"]["posture"], json!("full"));
    assert!(result["payload"]["payload"]["metrics"].is_object());
    assert!(result["payload"]["payload"]["indexes"].is_array());
    assert!(result["payload"]["payload"]["warnings"].is_array());
    assert!(result["payload"]["payload"].get("health").is_none());

    let metrics =
        serde_json::from_value::<RuntimeMetrics>(result["payload"]["payload"]["metrics"].clone())
            .expect("resource doctor metrics should match runtime metrics schema");
    assert_eq!(metrics.queue_depth, 0);
    assert!(metrics.active_requests >= 1);
    assert_eq!(metrics.background_jobs, 0);
    assert_eq!(metrics.cancelled_requests, 0);
    assert_eq!(metrics.maintenance_runs, 0);

    let missing_resource = send_request(
        &socket_path,
        json!({
            "jsonrpc":"2.0",
            "method":"resource.read",
            "params":{"uri":"membrain://daemon/runtime/missing"},
            "id":"resource-read-missing"
        }),
    )
    .await;
    assert_eq!(missing_resource["error"]["code"], json!(-32602));
    assert_eq!(
        missing_resource["error"]["message"],
        json!("unknown resource uri 'membrain://daemon/runtime/missing'")
    );
    assert_eq!(
        missing_resource["error"]["data"]["error_kind"],
        json!("validation_failure")
    );

    shutdown_runtime(&socket_path, handle).await;
}

#[tokio::test]
async fn runtime_reuses_error_model_for_invalid_jsonrpc_and_unknown_methods() {
    let (socket_path, handle) = spawn_runtime().await;

    let invalid_jsonrpc = send_request(
        &socket_path,
        json!({
            "jsonrpc":"1.0",
            "method":"status",
            "params":{},
            "id":"invalid-jsonrpc"
        }),
    )
    .await;
    assert_eq!(invalid_jsonrpc["error"]["code"], json!(-32600));
    assert_eq!(
        invalid_jsonrpc["error"]["message"],
        json!("unsupported jsonrpc version")
    );
    assert_eq!(
        invalid_jsonrpc["error"]["data"],
        json!({"expected":"2.0","actual":"1.0"})
    );

    let unknown_method = send_request(
        &socket_path,
        json!({
            "jsonrpc":"2.0",
            "method":"not_a_real_method",
            "params":{},
            "id":"unknown-method"
        }),
    )
    .await;
    assert_eq!(unknown_method["error"]["code"], json!(-32601));
    assert_eq!(
        unknown_method["error"]["message"],
        json!("unknown method 'not_a_real_method'")
    );
    assert_eq!(unknown_method["error"]["data"], json!(null));

    shutdown_runtime(&socket_path, handle).await;
}
