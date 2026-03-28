use membrain_core::engine::confidence::{ConfidenceInputs, ConfidenceOutput};
use membrain_core::engine::contradiction::ResolutionState;
use membrain_core::engine::lease::LeaseMetadata;
use membrain_core::engine::result::FreshnessMarkers;
use membrain_core::persistence::{
    open_hot_db, save_runtime_records, PersistedDaemonMemoryRecord, PersistedTier2Layout,
};
use membrain_core::types::{CanonicalMemoryType, FastPathRouteFamily};
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

fn unique_db_root(label: &str) -> PathBuf {
    static NEXT_DB_ID: AtomicU64 = AtomicU64::new(1);

    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("clock should be after epoch")
        .as_nanos();
    let unique_id = NEXT_DB_ID.fetch_add(1, Ordering::Relaxed);
    std::env::temp_dir().join(format!(
        "membrain-daemon-{label}-db-{}-{nanos}-{unique_id}",
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

async fn spawn_runtime_with_db(
    label: &str,
) -> (PathBuf, tokio::task::JoinHandle<anyhow::Result<()>>) {
    let socket_path = unique_socket_path(label);
    let db_root = unique_db_root(label);
    let hot_db_path = db_root.join("hot.db");
    let cold_db_path = db_root.join("cold.db");
    tokio::fs::create_dir_all(&db_root)
        .await
        .expect("db root should exist");

    let mut config =
        DaemonRuntimeConfig::new(&socket_path).with_db_paths(&hot_db_path, &cold_db_path);
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

async fn spawn_runtime_with_seeded_db(
    label: &str,
    records: &[PersistedDaemonMemoryRecord],
) -> (PathBuf, tokio::task::JoinHandle<anyhow::Result<()>>) {
    let socket_path = unique_socket_path(label);
    let db_root = unique_db_root(label);
    let hot_db_path = db_root.join("hot.db");
    let cold_db_path = db_root.join("cold.db");
    tokio::fs::create_dir_all(&db_root)
        .await
        .expect("db root should exist");

    let mut conn = open_hot_db(&hot_db_path).expect("hot db should open");
    save_runtime_records(&mut conn, records).expect("runtime records should save");

    let mut config =
        DaemonRuntimeConfig::new(&socket_path).with_db_paths(&hot_db_path, &cold_db_path);
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

fn seeded_runtime_record(
    memory_id: u64,
    namespace: &str,
    compact_text: &str,
    projected_freshness_markers: Option<FreshnessMarkers>,
) -> PersistedDaemonMemoryRecord {
    PersistedDaemonMemoryRecord {
        layout: PersistedTier2Layout {
            namespace: namespace.to_string(),
            memory_id,
            session_id: 1,
            memory_type: CanonicalMemoryType::Event,
            route_family: FastPathRouteFamily::Event,
            compact_text: compact_text.to_string(),
            fingerprint: memory_id.saturating_mul(97),
            payload_size_bytes: compact_text.len(),
            affect: None,
            is_landmark: false,
            landmark_label: None,
            era_id: None,
            visibility: "private".to_string(),
            lease: LeaseMetadata::new(membrain_core::engine::lease::LeasePolicy::Durable, 0),
            raw_text: compact_text.to_string(),
        },
        passive_observation: None,
        causal_parents: Vec::new(),
        causal_link_type: None,
        confidence_inputs: ConfidenceInputs {
            corroboration_count: 1,
            reconsolidation_count: 0,
            ticks_since_last_access: 0,
            age_ticks: 0,
            resolution_state: ResolutionState::None,
            conflict_score: 0,
            causal_parent_count: 0,
            authoritativeness: 900,
            recall_count: 1,
        },
        confidence_output: ConfidenceOutput {
            uncertainty_score: 50,
            corroboration_uncertainty: 50,
            reconsolidation_uncertainty: 50,
            freshness_uncertainty: 50,
            contradiction_uncertainty: 0,
            missing_evidence_uncertainty: 25,
            confidence: 950,
            confidence_interval: None,
        },
        projected_freshness_markers,
    }
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
async fn runtime_goal_working_state_methods_surface_blackboard_projection_parity() {
    let (socket_path, handle) = spawn_runtime().await;

    let goal_pin_response = send_request(
        &socket_path,
        json!({
            "jsonrpc":"2.0",
            "method":"goal_pin",
            "params":{
                "namespace":"team.alpha",
                "task_id":"task-42",
                "memory_id":7,
                "request_id":"req-goal-pin"
            },
            "id":"goal-pin"
        }),
    )
    .await;
    assert_eq!(
        goal_pin_response["result"]["authoritative_truth"],
        json!("durable_memory")
    );
    assert_eq!(goal_pin_response["result"]["status"], json!("active"));
    assert_eq!(
        goal_pin_response["result"]["blackboard_state"]["Present"]["projection_kind"],
        json!("working_state_projection")
    );
    assert!(
        goal_pin_response["result"]["blackboard_state"]["Present"]["active_evidence"]
            .as_array()
            .expect("active evidence array")
            .iter()
            .any(|handle| handle["memory_id"] == json!(7) && handle["pinned"] == json!(true))
    );

    let goal_dismiss_response = send_request(
        &socket_path,
        json!({
            "jsonrpc":"2.0",
            "method":"goal_dismiss",
            "params":{
                "namespace":"team.alpha",
                "task_id":"task-42",
                "memory_id":2,
                "agent_id":"agent-12"
            },
            "id":"goal-dismiss"
        }),
    )
    .await;
    assert_eq!(
        goal_dismiss_response["result"]["authoritative_truth"],
        json!("durable_memory")
    );
    assert!(
        goal_dismiss_response["result"]["blackboard_state"]["Present"]["active_evidence"]
            .as_array()
            .expect("active evidence array")
            .iter()
            .all(|handle| handle["memory_id"] != json!(2))
    );

    let goal_snapshot_response = send_request(
        &socket_path,
        json!({
            "jsonrpc":"2.0",
            "method":"goal_snapshot",
            "params":{
                "namespace":"team.alpha",
                "task_id":"task-42",
                "note":"handoff snapshot"
            },
            "id":"goal-snapshot"
        }),
    )
    .await;
    assert_eq!(
        goal_snapshot_response["result"]["authoritative_truth"],
        json!("durable_memory")
    );
    assert_eq!(
        goal_snapshot_response["result"]["snapshot"]["artifact_kind"],
        json!("blackboard_snapshot")
    );
    assert_eq!(
        goal_snapshot_response["result"]["snapshot"]["note"],
        json!("handoff snapshot")
    );

    shutdown_runtime(&socket_path, handle).await;
}

#[tokio::test]
async fn runtime_fork_and_merge_methods_surface_conflict_and_audit_parity() {
    let (socket_path, handle) = spawn_runtime().await;

    let fork_response = send_request(
        &socket_path,
        json!({
            "jsonrpc":"2.0",
            "method":"fork",
            "params":{
                "name":"agent-specialist",
                "namespace":"team.alpha",
                "inherit":"shared",
                "note":"runtime parity"
            },
            "id":"fork-runtime"
        }),
    )
    .await;
    assert_eq!(fork_response["result"]["name"], json!("agent-specialist"));
    assert_eq!(
        fork_response["result"]["parent_namespace"],
        json!("team.alpha")
    );
    assert_eq!(
        fork_response["result"]["fork_working_state_count"],
        json!(0)
    );
    assert_eq!(
        fork_response["result"]["isolation_semantics"],
        json!("inherit_by_reference_until_explicit_merge")
    );

    let goal_pin_response = send_request(
        &socket_path,
        json!({
            "jsonrpc":"2.0",
            "method":"goal_pin",
            "params":{
                "namespace":"agent-specialist",
                "task_id":"fork-task",
                "memory_id":7
            },
            "id":"goal-pin-fork"
        }),
    )
    .await;
    assert_eq!(goal_pin_response["result"]["status"], json!("active"));

    let merge_preview = send_request(
        &socket_path,
        json!({
            "jsonrpc":"2.0",
            "method":"merge_fork",
            "params":{
                "fork_name":"agent-specialist",
                "target_namespace":"team.alpha",
                "conflict_strategy":"manual",
                "dry_run":true
            },
            "id":"merge-preview"
        }),
    )
    .await;
    assert_eq!(
        merge_preview["result"]["fork_name"],
        json!("agent-specialist")
    );
    assert_eq!(
        merge_preview["result"]["target_namespace"],
        json!("team.alpha")
    );
    assert_eq!(merge_preview["result"]["dry_run"], json!(true));
    assert_eq!(merge_preview["result"]["conflicts_found"], json!(1));
    assert_eq!(merge_preview["result"]["conflicts_pending"], json!(1));
    assert_eq!(
        merge_preview["result"]["fork_working_state_count"],
        json!(1)
    );
    assert_eq!(merge_preview["result"]["divergence_detected"], json!(true));
    assert_eq!(merge_preview["result"]["audit_sequences"], json!([]));
    assert_eq!(
        merge_preview["result"]["conflict_items"][0]["item_kind"],
        json!("working_state")
    );
    assert_eq!(
        merge_preview["result"]["conflict_items"][0]["preferred_side"],
        json!("manual")
    );

    let merge_apply = send_request(
        &socket_path,
        json!({
            "jsonrpc":"2.0",
            "method":"merge_fork",
            "params":{
                "fork_name":"agent-specialist",
                "target_namespace":"team.alpha",
                "conflict_strategy":"fork-wins",
                "dry_run":false
            },
            "id":"merge-apply"
        }),
    )
    .await;
    assert_eq!(merge_apply["result"]["fork_status"], json!("merged"));
    assert_eq!(merge_apply["result"]["conflicts_found"], json!(0));
    assert_eq!(merge_apply["result"]["conflicts_pending"], json!(0));
    assert_eq!(merge_apply["result"]["memories_merged"], json!(1));
    assert_eq!(
        merge_apply["result"]["audit_sequences"]
            .as_array()
            .map(|v| v.len()),
        Some(1)
    );

    shutdown_runtime(&socket_path, handle).await;
}

#[tokio::test]
async fn runtime_zero_arg_methods_accept_common_envelope_fields() {
    let (socket_path, handle) = spawn_runtime().await;

    let doctor_response = send_request(
        &socket_path,
        json!({
            "jsonrpc":"2.0",
            "method":"doctor",
            "params":{
                "request_id":"req-doctor-ctx",
                "workspace_id":"ws-7",
                "agent_id":"agent-3",
                "session_id":"session-9",
                "task_id":"task-2",
                "time_budget_ms":75,
                "policy_context":{"include_public":true,"sharing_visibility":"public"}
            },
            "id":"doctor-common"
        }),
    )
    .await;
    assert_eq!(doctor_response["result"]["status"], json!("ok"));
    assert!(doctor_response["error"].is_null());

    let health_response = send_request(
        &socket_path,
        json!({
            "jsonrpc":"2.0",
            "method":"health",
            "params":{
                "request_id":"req-health-ctx",
                "workspace_id":"ws-7",
                "agent_id":"agent-3",
                "session_id":"session-9",
                "task_id":"task-2",
                "time_budget_ms":75,
                "policy_context":{"include_public":true,"sharing_visibility":"public"}
            },
            "id":"health-common"
        }),
    )
    .await;
    let hot_memories = health_response["result"]["hot_memories"]
        .as_u64()
        .expect("health hot_memories should be numeric");
    assert_eq!(hot_memories, 0);
    assert!(health_response["error"].is_null());

    let resources_list_response = send_request(
        &socket_path,
        json!({
            "jsonrpc":"2.0",
            "method":"resources.list",
            "params":{"request_id":"req-resources-ctx"},
            "id":"resources-list-common"
        }),
    )
    .await;
    assert_eq!(resources_list_response["result"]["status"], json!("ok"));
    assert!(resources_list_response["error"].is_null());

    let resource_read_response = send_request(
        &socket_path,
        json!({
            "jsonrpc":"2.0",
            "method":"resource.read",
            "params":{
                "uri":"membrain://daemon/runtime/status",
                "request_id":"req-resource-read-ctx"
            },
            "id":"resource-read-common"
        }),
    )
    .await;
    assert_eq!(resource_read_response["result"]["status"], json!("ok"));
    assert!(resource_read_response["error"].is_null());

    let streams_list_response = send_request(
        &socket_path,
        json!({
            "jsonrpc":"2.0",
            "method":"streams.list",
            "params":{"request_id":"req-streams-ctx"},
            "id":"streams-list-common"
        }),
    )
    .await;
    assert_eq!(streams_list_response["result"]["status"], json!("ok"));
    assert!(streams_list_response["error"].is_null());

    shutdown_runtime(&socket_path, handle).await;
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

    let health_response = send_request(
        &socket_path,
        json!({
            "jsonrpc":"2.0",
            "method":"health",
            "params":{"unexpected":true},
            "id":"health-invalid"
        }),
    )
    .await;

    assert_eq!(health_response["error"]["code"], json!(-32602));
    assert_eq!(
        health_response["error"]["message"],
        json!("unknown field unexpected")
    );
    assert_eq!(health_response["error"]["data"], json!(null));

    let resources_list_response = send_request(
        &socket_path,
        json!({
            "jsonrpc":"2.0",
            "method":"resources.list",
            "params":{"unexpected":true},
            "id":"resources-list-invalid"
        }),
    )
    .await;
    assert_eq!(resources_list_response["error"]["code"], json!(-32602));
    assert_eq!(
        resources_list_response["error"]["message"],
        json!("unknown field unexpected")
    );
    assert_eq!(resources_list_response["error"]["data"], json!(null));

    let streams_list_response = send_request(
        &socket_path,
        json!({
            "jsonrpc":"2.0",
            "method":"streams.list",
            "params":{"unexpected":true},
            "id":"streams-list-invalid"
        }),
    )
    .await;
    assert_eq!(streams_list_response["error"]["code"], json!(-32602));
    assert_eq!(
        streams_list_response["error"]["message"],
        json!("unknown field unexpected")
    );
    assert_eq!(streams_list_response["error"]["data"], json!(null));

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
    assert_eq!(result["error_kind"], json!(null));
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
        &vec![
            json!("operator_review_recommended"),
            json!("stale_action_critical_recheck_required")
        ]
    );
    let indexes = result["indexes"]
        .as_array()
        .expect("indexes should be an array");
    assert_eq!(indexes.len(), 4);
    assert_eq!(indexes[0]["family"], json!("schema"));
    assert_eq!(indexes[3]["family"], json!("cache"));
    assert_eq!(indexes[3]["health"], json!("ok"));
    assert_eq!(indexes[3]["usable"], json!(true));

    assert_eq!(result["action"], json!("doctor"));
    assert_eq!(result["posture"], json!("degraded"));
    assert_eq!(result["degraded_reasons"], json!(["repair_in_flight"]));
    assert!(result["metrics"].is_object());
    assert_eq!(result["summary"]["ok_checks"], json!(4));
    assert_eq!(result["summary"]["warn_checks"], json!(3));
    assert_eq!(result["summary"]["fail_checks"], json!(0));
    assert_eq!(result["repair_engine_component"], json!("engine.repair"));
    assert!(result["checks"].is_array());
    assert_eq!(result["checks"][3]["name"], json!("serving_posture"));
    assert_eq!(result["checks"][3]["status"], json!("warn"));
    assert_eq!(result["checks"][4]["name"], json!("runtime_authority"));
    assert_eq!(result["checks"][4]["status"], json!("ok"));
    assert_eq!(result["checks"][5]["name"], json!("lease_freshness"));
    assert_eq!(result["checks"][5]["status"], json!("warn"));
    assert!(result["runbook_hints"].is_array());
    assert_eq!(
        result["runbook_hints"]
            .as_array()
            .expect("runbook hints should be an array")
            .iter()
            .map(|hint| hint["runbook_id"].clone())
            .collect::<Vec<_>>(),
        vec![json!("repair_backlog_growth"), json!("incident_response")]
    );
    assert_eq!(result["availability"]["posture"], json!("degraded"));
    assert_eq!(
        result["availability"]["degraded_reasons"],
        json!(["repair_in_flight"])
    );
    assert_eq!(
        result["remediation"]["next_steps"],
        json!(["check_health", "run_repair"])
    );
    assert!(result["indexes"].is_array());
    assert!(result["repair_reports"].is_array());
    assert_eq!(
        result["repair_reports"][0]["target"],
        json!("lexical_index")
    );
    assert_eq!(
        result["repair_reports"][1]["target"],
        json!("metadata_index")
    );
    assert_eq!(
        result["repair_reports"][0]["rebuild_entrypoint"],
        json!(null)
    );
    assert_eq!(
        result["repair_reports"][0]["verification_artifact_name"],
        json!("fts5_lexical_parity")
    );
    assert_eq!(
        result["repair_reports"][0]["parity_check"],
        json!("fts5_projection_matches_durable_truth")
    );
    assert_eq!(
        result["repair_reports"][0]["authoritative_rows"],
        json!(128)
    );
    assert_eq!(result["repair_reports"][0]["derived_rows"], json!(128));
    assert_eq!(result["repair_reports"][0]["durable_sources"], json!([]));
    assert_eq!(
        result["repair_reports"][0]["affected_item_count"],
        json!(128)
    );
    assert_eq!(result["repair_reports"][0]["error_count"], json!(0));
    assert_eq!(result["repair_reports"][0]["queue_depth_before"], json!(4));
    assert_eq!(result["repair_reports"][0]["queue_depth_after"], json!(0));
    assert_eq!(
        result["repair_reports"][3]["target"],
        json!("semantic_cold_index")
    );
    assert!(result["health"].is_object());
    assert_eq!(result["health"]["availability_posture"], json!("Degraded"));
    assert_eq!(result["health"]["repair_queue_depth"], json!(0));

    shutdown_runtime(&socket_path, handle).await;
}

#[tokio::test]
async fn runtime_health_jsonrpc_response_matches_shared_health_contract_fields() {
    let (socket_path, handle) = spawn_runtime().await;

    let posture_response = send_request(
        &socket_path,
        json!({
            "jsonrpc":"2.0",
            "method":"set_posture",
            "params":{"posture":"degraded","reasons":["repair_in_flight"]},
            "id":"posture-health"
        }),
    )
    .await;
    assert_eq!(posture_response["result"]["posture"], json!("degraded"));

    let encode_response = send_request(
        &socket_path,
        json!({
            "jsonrpc":"2.0",
            "method":"encode",
            "params":{
                "content":"operator attention note",
                "namespace":"team.alpha"
            },
            "id":"encode-health"
        }),
    )
    .await;
    assert_eq!(encode_response["result"]["status"], json!("accepted"));

    let recall_response = send_request(
        &socket_path,
        json!({
            "jsonrpc":"2.0",
            "method":"recall",
            "params":{
                "query_text":"operator attention note",
                "namespace":"team.alpha",
                "session_id":"session-health",
                "task_id":"task-health"
            },
            "id":"recall-health"
        }),
    )
    .await;
    assert!(recall_response["result"].is_object());

    let health_response = send_request(
        &socket_path,
        json!({"jsonrpc":"2.0","method":"health","params":{},"id":"health"}),
    )
    .await;

    let result = &health_response["result"];
    let hot_memories = result["hot_memories"]
        .as_u64()
        .expect("health hot_memories should be numeric");
    let hot_capacity = result["hot_capacity"]
        .as_u64()
        .expect("health hot_capacity should be numeric");
    let hot_utilization_pct = result["hot_utilization_pct"]
        .as_f64()
        .expect("health hot_utilization_pct should be numeric");
    assert_eq!(hot_memories, 1);
    assert_eq!(hot_capacity, 100);
    assert!((hot_utilization_pct - 1.0).abs() < 1e-6);
    let avg_confidence = result["avg_confidence"]
        .as_f64()
        .expect("avg_confidence should be numeric");
    assert!(avg_confidence > 0.0);
    assert!(avg_confidence <= 1.0);
    assert_eq!(result["unresolved_conflicts"], json!(0));
    assert_eq!(result["availability_posture"], json!("Degraded"));
    assert_eq!(result["repair_queue_depth"], json!(0));
    assert!(result["cache"].is_object());
    assert!(result["indexes"].is_object());
    assert!(result["subsystem_status"].is_array());
    assert!(result["trends"].is_array());
    assert!(result["trend_summary"].is_array());
    assert_eq!(
        result["feature_availability"][0]["feature"],
        json!("health")
    );
    assert_eq!(result["feature_availability"][0]["posture"], json!("Full"));
    assert_eq!(
        result["feature_availability"][0]["note"],
        json!("daemon_doctor_embeds_brain_health_report")
    );
    assert_eq!(result["degraded_status"]["posture"], json!("Degraded"));
    assert_eq!(
        result["degraded_status"]["surviving_query_capabilities"],
        json!(["doctor", "health", "audit"])
    );
    assert_eq!(
        result["degraded_status"]["recommended_runbooks"],
        json!(["repair_backlog_growth"])
    );
    assert_eq!(result["total_encodes"], json!(1));
    assert_eq!(result["total_recalls"], json!(0));
    assert_eq!(result["attention"]["total_encode_count"], json!(1));
    assert_eq!(result["attention"]["total_recall_count"], json!(0));
    assert_eq!(result["attention"]["highest_namespace_pressure"], json!(0));
    assert_eq!(
        result["attention"]["hotspots"][0]["namespace"],
        json!("team.alpha")
    );
    assert_eq!(
        result["attention"]["hotspots"][0]["contributing_signals"]["encode_count"],
        json!(1)
    );
    assert_eq!(
        result["attention"]["hotspots"][0]["contributing_signals"]["recall_count"],
        json!(0)
    );
    assert!(result["attention"]["hotspots"][0]["sample_log"]
        .as_str()
        .unwrap()
        .contains("namespace=team.alpha"));
    assert_eq!(result["cache"]["hints_submitted"], json!(0));
    assert_eq!(result["cache"]["prefetch_queue_depth"], json!(0));

    shutdown_runtime(&socket_path, handle).await;
}

#[tokio::test]
async fn runtime_health_jsonrpc_surfaces_lifecycle_transition_after_background_maintenance() {
    let (socket_path, handle) = spawn_runtime().await;

    let health_before = send_request(
        &socket_path,
        json!({"jsonrpc":"2.0","method":"health","params":{},"id":"health-before-maintenance"}),
    )
    .await;
    let lifecycle_before = &health_before["result"]["lifecycle"];
    assert_eq!(lifecycle_before["background_maintenance_runs"], json!(0));
    assert!(lifecycle_before["background_maintenance_log"]
        .as_array()
        .expect("background maintenance log should be an array")
        .iter()
        .any(|entry| entry
            .as_str()
            .is_some_and(|value| value.contains("no_background_lifecycle_activity_recorded"))));

    let maintenance_response = send_request(
        &socket_path,
        json!({
            "jsonrpc":"2.0",
            "method":"run_maintenance",
            "params":{"polls_budget":2,"step_delay_ms":10},
            "id":"run-maintenance"
        }),
    )
    .await;
    assert!(maintenance_response.get("result").is_some());

    tokio::time::sleep(Duration::from_millis(40)).await;

    let health_after = send_request(
        &socket_path,
        json!({"jsonrpc":"2.0","method":"health","params":{},"id":"health-after-maintenance"}),
    )
    .await;
    let lifecycle_after = &health_after["result"]["lifecycle"];
    assert_eq!(lifecycle_after["background_maintenance_runs"], json!(1));
    assert_ne!(lifecycle_after, lifecycle_before);
    let background_log = lifecycle_after["background_maintenance_log"]
        .as_array()
        .expect("background maintenance log should be an array");
    assert!(background_log.iter().any(|entry| {
        entry.as_str().is_some_and(|value| {
            value.contains("maintenance_consolidation_completed")
                && value.contains("maintenance_id=1")
        })
    }));
    assert!(background_log.iter().any(|entry| {
        entry.as_str().is_some_and(|value| {
            value.contains("maintenance_reconsolidation_applied")
                && value.contains("maintenance_id=1")
        })
    }));
    assert!(background_log.iter().any(|entry| {
        entry.as_str().is_some_and(|value| {
            value.contains("maintenance_forgetting_evaluated") && value.contains("maintenance_id=1")
        })
    }));

    shutdown_runtime(&socket_path, handle).await;
}

#[tokio::test]
async fn runtime_recall_and_why_keep_hydrated_evidence_visible_after_maintenance() {
    let (socket_path, handle) = spawn_runtime().await;

    let observe_response = send_request(
        &socket_path,
        json!({
            "jsonrpc":"2.0",
            "method":"observe",
            "params":{
                "content":"Dang prefers concise answers after launch review",
                "namespace":"team.alpha",
                "source_label":"stdin:test"
            },
            "id":"observe-lifecycle-wrapper"
        }),
    )
    .await;
    assert_eq!(observe_response["result"]["status"], json!("accepted"));

    let maintenance_response = send_request(
        &socket_path,
        json!({
            "jsonrpc":"2.0",
            "method":"run_maintenance",
            "params":{"polls_budget":2,"step_delay_ms":10},
            "id":"run-maintenance-lifecycle-wrapper"
        }),
    )
    .await;
    assert!(maintenance_response.get("result").is_some());

    tokio::time::sleep(Duration::from_millis(40)).await;

    let recall_response = send_request(
        &socket_path,
        json!({
            "jsonrpc":"2.0",
            "method":"recall",
            "params":{
                "query_text":"Dang prefers concise answers",
                "namespace":"team.alpha",
                "session_id":"session-lifecycle",
                "task_id":"task-lifecycle"
            },
            "id":"recall-lifecycle-wrapper"
        }),
    )
    .await;
    assert_eq!(recall_response["result"]["status"], json!("ok"));
    assert_eq!(
        recall_response["result"]["retrieval"]["result"]["packaging_metadata"]["degraded_summary"],
        json!(null)
    );
    let memory_id = recall_response["result"]["retrieval"]["result"]["evidence_pack"][0]["result"]
        ["memory_id"]
        .as_u64()
        .expect("maintenance recall should expose the observed memory id");
    assert_eq!(
        recall_response["result"]["retrieval"]["result"]["evidence_pack"][0]["result"]["memory_id"],
        json!(memory_id)
    );
    let recall_result_reasons = recall_response["result"]["retrieval"]["explain_trace"]
        ["result_reasons"]
        .as_array()
        .expect("recall result reasons should be an array");
    assert!(recall_result_reasons.iter().any(|reason| {
        reason["memory_id"] == json!(memory_id) && reason["reason_code"] == json!("score_kept")
    }));

    let why_response = send_request(
        &socket_path,
        json!({
            "jsonrpc":"2.0",
            "method":"why",
            "params":{"id":memory_id,"namespace":"team.alpha"},
            "id":"why-lifecycle-wrapper"
        }),
    )
    .await;
    assert_eq!(why_response["result"]["status"], json!("ok"));
    assert_eq!(
        why_response["result"]["retrieval"]["result"]["packaging_metadata"]["degraded_summary"],
        json!(null)
    );
    assert_eq!(
        why_response["result"]["retrieval"]["result"]["evidence_pack"][0]["result"]["memory_id"],
        json!(memory_id)
    );
    let why_result_reasons = why_response["result"]["retrieval"]["explain_trace"]["result_reasons"]
        .as_array()
        .expect("why result reasons should be an array");
    assert!(why_result_reasons.iter().any(|reason| {
        reason["memory_id"] == json!(memory_id)
            && reason["reason_code"] == json!("query_by_example_seed_materialized")
    }));
    let why_freshness_markers = why_response["result"]["retrieval"]["explain_trace"]
        ["freshness_markers"]
        .as_array()
        .expect("why freshness markers should be an array");
    assert!(!why_freshness_markers.is_empty());

    shutdown_runtime(&socket_path, handle).await;
}

#[tokio::test]
async fn runtime_maintenance_projects_cold_and_reconsolidating_lifecycle_reasons_on_real_daemon_path(
) {
    let (socket_path, handle) = spawn_runtime_with_db("maintenance-real-lifecycle").await;

    for (id, content) in [
        (
            "observe-maintenance-oldest",
            "release deploy rollback checklist from last quarter",
        ),
        (
            "observe-maintenance-newest",
            "current deploy remediation plan for production rollout",
        ),
    ] {
        let observe_response = send_request(
            &socket_path,
            json!({
                "jsonrpc":"2.0",
                "method":"observe",
                "params":{
                    "content":content,
                    "namespace":"team.alpha",
                    "source_label":"stdin:test"
                },
                "id":id
            }),
        )
        .await;
        assert_eq!(observe_response["result"]["status"], json!("accepted"));
        assert_eq!(observe_response["result"]["memories_created"], json!(1));
    }

    let maintenance_response = send_request(
        &socket_path,
        json!({
            "jsonrpc":"2.0",
            "method":"run_maintenance",
            "params":{"polls_budget":2,"step_delay_ms":10},
            "id":"run-maintenance-real-lifecycle"
        }),
    )
    .await;
    assert!(maintenance_response.get("result").is_some());

    tokio::time::sleep(Duration::from_millis(40)).await;

    let recall_response = send_request(
        &socket_path,
        json!({
            "jsonrpc":"2.0",
            "method":"recall",
            "params":{
                "query_text":"deploy remediation rollout plan",
                "namespace":"team.alpha",
                "result_budget":4
            },
            "id":"recall-maintenance-real-lifecycle"
        }),
    )
    .await;
    assert_eq!(recall_response["result"]["status"], json!("ok"));
    assert_eq!(
        recall_response["result"]["retrieval"]["result"]["packaging_metadata"]["degraded_summary"],
        json!(null)
    );
    let evidence_pack = recall_response["result"]["retrieval"]["result"]["evidence_pack"]
        .as_array()
        .expect("recall evidence pack should be an array");
    let cold_memory = evidence_pack
        .iter()
        .find(|item| {
            item["result"]["answered_from"] == json!("tier3_cold")
                && item["result"]["entry_lane"] == json!("cold_fallback")
        })
        .expect("maintenance should project one cold durable result");
    let reconsolidating_memory = evidence_pack
        .iter()
        .find(|item| item["result"]["memory_id"] != cold_memory["result"]["memory_id"])
        .expect("maintenance should leave one non-cold result");
    let reconsolidating_memory_id = reconsolidating_memory["result"]["memory_id"]
        .as_u64()
        .expect("reconsolidating memory id should be present");

    let recall_result_reasons = recall_response["result"]["retrieval"]["explain_trace"]
        ["result_reasons"]
        .as_array()
        .expect("recall result reasons should be an array");
    assert!(recall_result_reasons
        .iter()
        .any(|reason| reason["reason_code"] == json!("cold_consolidated")));
    assert!(recall_result_reasons
        .iter()
        .any(|reason| reason["reason_code"] == json!("reconsolidation_window_open")));
    let recall_freshness_markers = recall_response["result"]["retrieval"]["explain_trace"]
        ["freshness_markers"]
        .as_array()
        .expect("recall freshness markers should be an array");
    assert!(recall_freshness_markers
        .iter()
        .any(|marker| marker["code"] == json!("lifecycle_projection")));

    let why_response = send_request(
        &socket_path,
        json!({
            "jsonrpc":"2.0",
            "method":"why",
            "params":{"id":reconsolidating_memory_id,"namespace":"team.alpha"},
            "id":"why-maintenance-real-lifecycle"
        }),
    )
    .await;
    assert_eq!(why_response["result"]["status"], json!("ok"));
    let why_result_reasons = why_response["result"]["retrieval"]["explain_trace"]["result_reasons"]
        .as_array()
        .expect("why result reasons should be an array");
    assert!(why_result_reasons
        .iter()
        .any(|reason| { reason["reason_code"] == json!("reconsolidation_window_open") }));
    assert!(why_result_reasons.iter().any(|reason| {
        reason["memory_id"] == json!(reconsolidating_memory_id)
            && reason["reason_code"] == json!("query_by_example_seed_materialized")
    }));
    let why_freshness_markers = why_response["result"]["retrieval"]["explain_trace"]
        ["freshness_markers"]
        .as_array()
        .expect("why freshness markers should be an array");
    assert!(why_freshness_markers
        .iter()
        .any(|marker| marker["code"] == json!("lifecycle_projection")));

    shutdown_runtime(&socket_path, handle).await;
}

#[tokio::test]
async fn runtime_health_resource_read_matches_runtime_health_payload_shape() {
    let (socket_path, handle) = spawn_runtime().await;

    let health_resource = send_request(
        &socket_path,
        json!({
            "jsonrpc":"2.0",
            "method":"resource.read",
            "params":{"uri":"membrain://daemon/runtime/health"},
            "id":"resource-read-health"
        }),
    )
    .await;

    let result = &health_resource["result"];
    assert_eq!(result["status"], json!("ok"));
    assert_eq!(
        result["payload"]["uri"],
        json!("membrain://daemon/runtime/health")
    );
    assert_eq!(result["payload"]["resource_kind"], json!("runtime_health"));
    assert_eq!(result["payload"]["payload"]["hot_memories"], json!(0));
    assert_eq!(result["payload"]["payload"]["hot_capacity"], json!(100));
    assert!(result["payload"]["payload"]["cache"].is_object());
    assert!(result["payload"]["payload"]["indexes"].is_object());
    assert!(result["payload"]["payload"]["trend_summary"].is_array());
    assert_eq!(
        result["payload"]["payload"]["feature_availability"][0]["feature"],
        json!("health")
    );
    assert_eq!(
        result["payload"]["payload"]["feature_availability"][0]["posture"],
        json!("Full")
    );
    assert_eq!(
        result["payload"]["payload"]["feature_availability"][1]["feature"],
        json!("runtime_authority")
    );
    assert_eq!(
        result["payload"]["payload"]["feature_availability"][1]["posture"],
        json!("Full")
    );
    assert!(
        result["payload"]["payload"]["feature_availability"][1]["note"]
            .as_str()
            .is_some_and(|note| {
                note.contains("daemon_authority:mode=unix_socket_daemon")
                    && note.contains("authoritative_runtime=true")
                    && note.contains("warm_runtime_guarantees=")
            })
    );
    assert_eq!(
        result["payload"]["payload"]["feature_availability"][2]["feature"],
        json!("embedder_runtime")
    );
    assert_eq!(
        result["payload"]["payload"]["feature_availability"][2]["posture"],
        json!("Degraded")
    );
    assert!(
        result["payload"]["payload"]["feature_availability"][2]["note"]
            .as_str()
            .is_some_and(|note| {
                note.contains("state=not_loaded")
                    && note.contains("backend=local_fastembed")
                    && note.contains("generation=all-minilm-l6-v2@default")
            })
    );
    assert!(result["payload"]["payload"]["degraded_status"].is_null());

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
    assert_eq!(
        result["payload"]["payload"]["summary"]["ok_checks"],
        json!(6)
    );
    assert_eq!(
        result["payload"]["payload"]["summary"]["warn_checks"],
        json!(1)
    );
    assert_eq!(
        result["payload"]["payload"]["summary"]["fail_checks"],
        json!(0)
    );
    assert_eq!(
        result["payload"]["payload"]["repair_engine_component"],
        json!("engine.repair")
    );
    assert!(result["payload"]["payload"]["metrics"].is_object());
    assert!(result["payload"]["payload"]["checks"].is_array());
    assert_eq!(
        result["payload"]["payload"]["checks"][4]["status"],
        json!("ok")
    );
    assert_eq!(result["payload"]["payload"]["runbook_hints"], json!([]));
    assert_eq!(result["payload"]["payload"]["availability"], json!(null));
    assert_eq!(result["payload"]["payload"]["error_kind"], json!(null));
    assert_eq!(result["payload"]["payload"]["remediation"], json!(null));
    assert!(result["payload"]["payload"]["indexes"].is_array());
    assert!(result["payload"]["payload"]["repair_reports"].is_array());
    assert!(result["payload"]["payload"]["warnings"].is_array());
    assert!(result["payload"]["payload"]["health"].is_object());
    assert_eq!(
        result["payload"]["payload"]["repair_reports"][0]["target"],
        json!("lexical_index")
    );
    assert_eq!(
        result["payload"]["payload"]["repair_reports"][0]["rebuild_entrypoint"],
        json!(null)
    );
    assert_eq!(
        result["payload"]["payload"]["repair_reports"][0]["verification_artifact_name"],
        json!("fts5_lexical_parity")
    );
    assert_eq!(
        result["payload"]["payload"]["repair_reports"][0]["parity_check"],
        json!("fts5_projection_matches_durable_truth")
    );
    assert_eq!(
        result["payload"]["payload"]["repair_reports"][0]["authoritative_rows"],
        json!(128)
    );
    assert_eq!(
        result["payload"]["payload"]["repair_reports"][0]["durable_sources"],
        json!([])
    );
    assert_eq!(
        result["payload"]["payload"]["repair_reports"][0]["affected_item_count"],
        json!(128)
    );
    assert_eq!(result["payload"]["payload"]["warnings"], json!([]));
    assert_eq!(
        result["payload"]["payload"]["health"]["availability_posture"],
        json!(null)
    );
    assert_eq!(
        result["payload"]["payload"]["health"]["feature_availability"][1]["feature"],
        json!("runtime_authority")
    );
    assert_eq!(
        result["payload"]["payload"]["health"]["feature_availability"][1]["posture"],
        json!("Full")
    );
    assert!(
        result["payload"]["payload"]["health"]["feature_availability"][1]["note"]
            .as_str()
            .is_some_and(|note| {
                note.contains("daemon_authority:mode=unix_socket_daemon")
                    && note.contains("authoritative_runtime=true")
                    && note.contains("warm_runtime_guarantees=")
            })
    );
    assert_eq!(
        result["payload"]["payload"]["health"]["feature_availability"][2]["feature"],
        json!("embedder_runtime")
    );
    assert_eq!(
        result["payload"]["payload"]["health"]["feature_availability"][2]["posture"],
        json!("Degraded")
    );
    assert!(
        result["payload"]["payload"]["health"]["feature_availability"][2]["note"]
            .as_str()
            .is_some_and(|note| {
                note.contains("state=not_loaded")
                    && note.contains("backend=local_fastembed")
                    && note.contains("generation=all-minilm-l6-v2@default")
            })
    );
    assert_eq!(
        result["payload"]["payload"]["checks"][4]["name"],
        json!("runtime_authority")
    );
    assert_eq!(
        result["payload"]["payload"]["checks"][4]["status"],
        json!("ok")
    );

    let metrics =
        serde_json::from_value::<RuntimeMetrics>(result["payload"]["payload"]["metrics"].clone())
            .expect("resource doctor metrics should match runtime metrics schema");
    assert_eq!(metrics.queue_depth, 0);
    assert!(metrics.active_requests >= 1);
    assert_eq!(metrics.background_jobs, 0);
    assert_eq!(metrics.cancelled_requests, 0);
    assert_eq!(metrics.maintenance_runs, 0);

    let status_resource = send_request(
        &socket_path,
        json!({
            "jsonrpc":"2.0",
            "method":"resource.read",
            "params":{"uri":"membrain://daemon/runtime/status"},
            "id":"resource-read-status"
        }),
    )
    .await;
    assert_eq!(status_resource["result"]["status"], json!("ok"));
    assert_eq!(
        status_resource["result"]["payload"]["resource_kind"],
        json!("runtime_status")
    );
    assert_eq!(
        status_resource["result"]["payload"]["payload"]["posture"],
        json!("full")
    );
    assert_eq!(
        status_resource["result"]["payload"]["payload"]["authority_mode"],
        json!("unix_socket_daemon")
    );
    assert_eq!(
        status_resource["result"]["payload"]["payload"]["authoritative_runtime"],
        json!(true)
    );
    assert_eq!(
        status_resource["result"]["payload"]["payload"]["maintenance_active"],
        json!(true)
    );
    assert_eq!(
        status_resource["result"]["payload"]["payload"]["warm_runtime_guarantees"],
        json!([
            "daemon_owned_runtime_state",
            "shared_process_state",
            "unix_socket_authority",
            "repeated_request_warmth",
            "background_maintenance_loop"
        ])
    );
    assert!(status_resource["result"]["payload"]["payload"]["metrics"].is_object());

    let streams_resource = send_request(
        &socket_path,
        json!({
            "jsonrpc":"2.0",
            "method":"resource.read",
            "params":{"uri":"membrain://daemon/runtime/streams"},
            "id":"resource-read-streams"
        }),
    )
    .await;
    assert_eq!(streams_resource["result"]["status"], json!("ok"));
    assert_eq!(
        streams_resource["result"]["payload"]["resource_kind"],
        json!("stream_listing")
    );
    assert_eq!(
        streams_resource["result"]["payload"]["payload"]["streams"][0]["method"],
        json!("maintenance.status")
    );
    assert_eq!(
        streams_resource["result"]["payload"]["payload"]["streams"][0]["delivery"],
        json!("jsonrpc_notification")
    );

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
async fn runtime_resource_and_stream_listings_match_shared_mcp_payload_shape() {
    let (socket_path, handle) = spawn_runtime().await;

    let resources_list = send_request(
        &socket_path,
        json!({
            "jsonrpc":"2.0",
            "method":"resources.list",
            "params":{},
            "id":"resources-list"
        }),
    )
    .await;
    assert_eq!(resources_list["result"]["status"], json!("ok"));
    assert_eq!(
        resources_list["result"]["payload"]["namespace"],
        json!("daemon.runtime")
    );
    assert_eq!(
        resources_list["result"]["payload"]["resources"][0]["resource_kind"],
        json!("runtime_status")
    );
    assert_eq!(
        resources_list["result"]["payload"]["resources"][1]["resource_kind"],
        json!("runtime_health")
    );
    assert_eq!(
        resources_list["result"]["payload"]["resources"][2]["resource_kind"],
        json!("runtime_doctor")
    );
    assert_eq!(
        resources_list["result"]["payload"]["resources"][3]["resource_kind"],
        json!("stream_listing")
    );
    assert_eq!(
        resources_list["result"]["payload"]["resources"][4]["uri_template"],
        json!("membrain://{namespace}/memories/{memory_id}")
    );

    let streams_list = send_request(
        &socket_path,
        json!({
            "jsonrpc":"2.0",
            "method":"streams.list",
            "params":{},
            "id":"streams-list"
        }),
    )
    .await;
    assert_eq!(streams_list["result"]["status"], json!("ok"));
    assert_eq!(
        streams_list["result"]["payload"]["namespace"],
        json!("daemon.runtime")
    );
    assert_eq!(
        streams_list["result"]["payload"]["streams"][0]["name"],
        json!("maintenance-status")
    );
    assert_eq!(
        streams_list["result"]["payload"]["streams"][0]["method"],
        json!("maintenance.status")
    );
    assert_eq!(
        streams_list["result"]["payload"]["streams"][0]["example_subscriptions"][0],
        json!("maintenance.status")
    );

    let tools_list = send_request(
        &socket_path,
        json!({
            "jsonrpc":"2.0",
            "method":"tools.list",
            "params":{},
            "id":"tools-list"
        }),
    )
    .await;
    assert_eq!(tools_list["error"]["code"], json!(-32601));
    assert_eq!(
        tools_list["error"]["message"],
        json!("unknown method 'tools.list'")
    );

    shutdown_runtime(&socket_path, handle).await;
}

#[tokio::test]
async fn runtime_partial_archival_recovery_marker_survives_unix_socket_recall_and_why() {
    let namespace = "team.alpha";
    let memory_id = 41;
    let record = seeded_runtime_record(
        memory_id,
        namespace,
        "partial archival recovery for rollout remediation proof",
        Some(FreshnessMarkers::archival_recovery_partial(None)),
    );
    let (socket_path, handle) =
        spawn_runtime_with_seeded_db("partial-archival-recovery", &[record]).await;

    let recall_response = send_request(
        &socket_path,
        json!({
            "jsonrpc":"2.0",
            "method":"recall",
            "params":{"query_text":"partial archival recovery rollout remediation","namespace":namespace},
            "id":"recall-partial-archival"
        }),
    )
    .await;
    assert_eq!(recall_response["result"]["status"], json!("ok"));
    assert_eq!(
        recall_response["result"]["retrieval"]["result"]["evidence_pack"][0]["result"]["memory_id"],
        json!(memory_id)
    );
    let recall_markers = recall_response["result"]["retrieval"]["explain_trace"]
        ["freshness_markers"]
        .as_array()
        .expect("recall freshness markers should be an array");
    assert!(recall_markers
        .iter()
        .any(|marker| marker["code"] == json!("archival_recovery_partial")));
    assert_eq!(
        recall_response["result"]["retrieval"]["result"]["evidence_pack"][0]["freshness_markers"]
            ["durable_lifecycle_state"],
        json!("partially_restored")
    );
    assert_eq!(
        recall_response["result"]["retrieval"]["result"]["evidence_pack"][0]["freshness_markers"]
            ["routing_lifecycle_state"],
        json!("archival_recovery_partial")
    );

    let why_response = send_request(
        &socket_path,
        json!({
            "jsonrpc":"2.0",
            "method":"why",
            "params":{"id":memory_id,"namespace":namespace},
            "id":"why-partial-archival"
        }),
    )
    .await;
    assert_eq!(why_response["result"]["status"], json!("ok"));
    let why_markers = why_response["result"]["retrieval"]["explain_trace"]["freshness_markers"]
        .as_array()
        .expect("why freshness markers should be an array");
    assert!(why_markers
        .iter()
        .any(|marker| marker["code"] == json!("archival_recovery_partial")));
    assert_eq!(
        why_response["result"]["retrieval"]["result"]["evidence_pack"][0]["freshness_markers"]
            ["durable_lifecycle_state"],
        json!("partially_restored")
    );
    assert_eq!(
        why_response["result"]["retrieval"]["result"]["evidence_pack"][0]["freshness_markers"]
            ["routing_lifecycle_state"],
        json!("archival_recovery_partial")
    );

    shutdown_runtime(&socket_path, handle).await;
}

#[tokio::test]
async fn runtime_cold_recall_without_partial_restore_marker_stays_cold_consolidated() {
    let namespace = "team.alpha";
    let memory_id = 42;
    let record = seeded_runtime_record(
        memory_id,
        namespace,
        "cold retained evidence without partial archival recovery marker",
        None,
    );
    let (socket_path, handle) =
        spawn_runtime_with_seeded_db("cold-without-partial-restore", &[record]).await;

    let recall_response = send_request(
        &socket_path,
        json!({
            "jsonrpc":"2.0",
            "method":"recall",
            "params":{"query_text":"cold retained evidence partial archival","namespace":namespace},
            "id":"recall-cold-consolidated"
        }),
    )
    .await;
    assert_eq!(recall_response["result"]["status"], json!("ok"));
    let recall_markers = recall_response["result"]["retrieval"]["explain_trace"]
        ["freshness_markers"]
        .as_array()
        .expect("recall freshness markers should be an array");
    assert!(!recall_markers
        .iter()
        .any(|marker| marker["code"] == json!("archival_recovery_partial")));
    let recall_reasons = recall_response["result"]["retrieval"]["explain_trace"]["result_reasons"]
        .as_array()
        .expect("recall result reasons should be an array");
    assert!(recall_reasons
        .iter()
        .any(|reason| reason["reason_code"] == json!("cold_consolidated")));

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

    let invalid_set_posture = send_request(
        &socket_path,
        json!({
            "jsonrpc":"2.0",
            "method":"set_posture",
            "params":{"posture":"degraded","unexpected":true},
            "id":"set-posture-invalid"
        }),
    )
    .await;
    assert_eq!(invalid_set_posture["error"]["code"], json!(-32602));
    assert_eq!(
        invalid_set_posture["error"]["message"],
        json!("unknown field unexpected")
    );
    assert_eq!(invalid_set_posture["error"]["data"], json!(null));

    let invalid_sleep = send_request(
        &socket_path,
        json!({
            "jsonrpc":"2.0",
            "method":"sleep",
            "params":{"millis":1,"unexpected":true},
            "id":"sleep-invalid"
        }),
    )
    .await;
    assert_eq!(invalid_sleep["error"]["code"], json!(-32602));
    assert_eq!(
        invalid_sleep["error"]["message"],
        json!("unknown field unexpected")
    );
    assert_eq!(invalid_sleep["error"]["data"], json!(null));

    shutdown_runtime(&socket_path, handle).await;
}
