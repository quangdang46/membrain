#![cfg(not(windows))]

#[cfg(unix)]
use membrain_core::engine::confidence::{ConfidenceInputs, ConfidenceOutput};
#[cfg(unix)]
use membrain_core::engine::contradiction::ResolutionState;
#[cfg(unix)]
use membrain_core::engine::lease::LeaseMetadata;
#[cfg(unix)]
use membrain_core::engine::result::FreshnessMarkers;
#[cfg(unix)]
use membrain_core::persistence::{
    open_hot_db, save_runtime_records, PersistedDaemonMemoryRecord, PersistedTier2Layout,
};
#[cfg(unix)]
use membrain_core::types::{CanonicalMemoryType, FastPathRouteFamily};
use serde_json::{json, Value};
use std::io::{BufRead, BufReader, Write};
#[cfg(unix)]
use std::os::unix::net::UnixStream;
use std::process::{Child, Command, Stdio};
#[cfg(unix)]
use std::thread::sleep;
#[cfg(unix)]
use std::time::Duration;
use std::time::{SystemTime, UNIX_EPOCH};

fn test_db_root() -> std::path::PathBuf {
    let unique = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system time should be after unix epoch")
        .as_nanos();
    std::env::temp_dir().join(format!("membrain-cli-e2e-{unique}"))
}

fn run_membrain(args: &[&str]) -> (bool, String, String) {
    let db_root = test_db_root();
    run_membrain_with_db(&db_root, args)
}

fn run_membrain_with_db(db_root: &std::path::Path, args: &[&str]) -> (bool, String, String) {
    let output = Command::new(env!("CARGO_BIN_EXE_membrain"))
        .arg("--db-path")
        .arg(db_root)
        .args(args)
        .output()
        .expect("membrain binary should run");

    (
        output.status.success(),
        String::from_utf8(output.stdout).expect("stdout should be utf8"),
        String::from_utf8(output.stderr).expect("stderr should be utf8"),
    )
}

fn parse_json(stdout: &str) -> Value {
    serde_json::from_str(stdout).expect("command should emit valid json")
}

const RELEASE_RECALL_QUERY: &str =
    "Which release deploy fix should we roll out after the pipeline outage?";
const RELEASE_RECALL_TARGET: &str =
    "production deploy pipeline remediation rollout for incident fix";
const RELEASE_RECALL_LEXICAL_DISTRACTOR: &str = "release rollback checklist after outage";
const RELEASE_RECALL_UNRELATED_DISTRACTOR: &str =
    "checkout payment capture retry fix for shopping cart failure";

#[cfg(unix)]
fn seed_runtime_records(db_root: &std::path::Path, records: &[PersistedDaemonMemoryRecord]) {
    std::fs::create_dir_all(db_root).expect("db root should exist");
    let hot_db_path = db_root.join("hot.db");
    let mut conn = open_hot_db(&hot_db_path).expect("hot db should open");
    save_runtime_records(&mut conn, records).expect("runtime records should save");
}

#[cfg(unix)]
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

fn spawn_membrain_mcp(db_root: &std::path::Path) -> Child {
    Command::new(env!("CARGO_BIN_EXE_membrain"))
        .arg("--db-path")
        .arg(db_root)
        .arg("mcp")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("membrain mcp should spawn")
}

#[cfg(unix)]
fn spawn_membrain_daemon(db_root: &std::path::Path, socket_path: &std::path::Path) -> Child {
    Command::new(env!("CARGO_BIN_EXE_membrain"))
        .arg("--db-path")
        .arg(db_root)
        .arg("daemon")
        .arg("--socket-path")
        .arg(socket_path)
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::piped())
        .spawn()
        .expect("membrain daemon should spawn")
}

#[cfg(unix)]
fn unique_socket_path(label: &str) -> std::path::PathBuf {
    let unique = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system time should be after unix epoch")
        .as_nanos();
    let compact_label = label.chars().take(8).collect::<String>();
    std::env::temp_dir().join(format!(
        "mb-{compact_label}-{}-{:x}.sock",
        std::process::id(),
        unique & 0xffff_ffff
    ))
}

#[cfg(unix)]
fn wait_for_socket(socket_path: &std::path::Path) -> bool {
    for _ in 0..500 {
        if socket_path.exists() && UnixStream::connect(socket_path).is_ok() {
            return true;
        }
        sleep(Duration::from_millis(10));
    }
    false
}

#[cfg(unix)]
fn send_daemon_request(socket_path: &std::path::Path, request: Value) -> Value {
    let mut stream =
        UnixStream::connect(socket_path).expect("daemon socket should accept connections");
    let payload = serde_json::to_string(&request).expect("request should serialize");
    stream
        .write_all(payload.as_bytes())
        .expect("request should write");
    stream.write_all(b"\n").expect("newline should write");
    stream.flush().expect("request should flush");

    let mut reader = BufReader::new(stream);
    let mut line = String::new();
    reader.read_line(&mut line).expect("response should read");
    serde_json::from_str(&line).expect("daemon response should be valid json")
}

#[cfg(unix)]
fn shutdown_membrain_daemon(child: &mut Child, socket_path: &std::path::Path) {
    let shutdown = send_daemon_request(
        socket_path,
        json!({"jsonrpc":"2.0","method":"shutdown","params":{},"id":"shutdown"}),
    );
    assert_eq!(shutdown["result"]["shutting_down"], json!(true));
    let status = child.wait().expect("daemon process should exit");
    assert!(status.success(), "daemon exit status: {status}");
}

fn send_mcp_request(child: &mut Child, request: Value) -> Value {
    let stdin = child.stdin.as_mut().expect("child stdin should be piped");
    let payload = serde_json::to_string(&request).expect("request should serialize");
    stdin
        .write_all(payload.as_bytes())
        .expect("request should write");
    stdin.write_all(b"\n").expect("newline should write");
    stdin.flush().expect("stdin should flush");

    let stdout = child.stdout.as_mut().expect("child stdout should be piped");
    let mut reader = BufReader::new(stdout);
    let mut line = String::new();
    reader.read_line(&mut line).expect("response should read");
    serde_json::from_str(&line).expect("response should be valid json")
}

#[test]
fn cli_encode_json_emits_expected_machine_readable_fields() {
    let (ok, stdout, stderr) = run_membrain(&[
        "remember",
        "Paris is the capital of France",
        "--namespace",
        "test_ns",
        "--kind",
        "semantic",
        "--source",
        "cli-e2e",
        "--json",
    ]);

    assert!(ok, "stderr: {stderr}");
    let json = parse_json(&stdout);
    assert_eq!(json["ok"], true);
    assert_eq!(json["namespace"], "test_ns");
    assert_eq!(json["outcome_class"], "accepted");
    assert_eq!(json["result"]["outcome"], "accepted");
    assert_eq!(json["result"]["memory_id"], 1);
    assert_eq!(json["result"]["namespace"], "test_ns");
    assert_eq!(json["result"]["memory_type"], "observation");
    assert_eq!(
        json["result"]["compact_text"],
        "Paris is the capital of France"
    );
    assert_eq!(json["result"]["source"], "cli-e2e");
}

#[test]
fn cli_remember_accepts_space_separated_negative_valence() {
    let db_root = test_db_root();
    let (ok, stdout, stderr) = run_membrain_with_db(
        &db_root,
        &[
            "remember",
            "Deploy caused 30-min downtime",
            "--namespace",
            "test_ns",
            "--valence",
            "-0.8",
            "--arousal",
            "0.8",
            "--json",
        ],
    );

    assert!(ok, "stderr: {stderr}");
    let json = parse_json(&stdout);
    assert_eq!(json["ok"], true);
    assert_eq!(json["namespace"], "test_ns");
    assert_eq!(json["result"]["outcome"], "accepted");
    assert_eq!(json["result"]["memory_id"], 1);
}

#[test]
fn cli_restart_rehydrates_persisted_memory_for_inspect_recall_and_explain() {
    let db_root = test_db_root();

    let (encode_ok, encode_stdout, encode_stderr) = run_membrain_with_db(
        &db_root,
        &[
            "remember",
            RELEASE_RECALL_TARGET,
            "--namespace",
            "test_ns",
            "--kind",
            "semantic",
            "--json",
        ],
    );
    assert!(encode_ok, "stderr: {encode_stderr}");
    let encode_json = parse_json(&encode_stdout);
    let memory_id = encode_json["result"]["memory_id"]
        .as_u64()
        .expect("encode should return memory id");

    for distractor in [
        RELEASE_RECALL_LEXICAL_DISTRACTOR,
        RELEASE_RECALL_UNRELATED_DISTRACTOR,
    ] {
        let (distractor_ok, _distractor_stdout, distractor_stderr) = run_membrain_with_db(
            &db_root,
            &[
                "remember",
                distractor,
                "--namespace",
                "test_ns",
                "--kind",
                "semantic",
                "--json",
            ],
        );
        assert!(distractor_ok, "stderr: {distractor_stderr}");
    }

    let (inspect_ok, inspect_stdout, inspect_stderr) = run_membrain_with_db(
        &db_root,
        &[
            "inspect",
            "--id",
            &memory_id.to_string(),
            "--namespace",
            "test_ns",
            "--json",
        ],
    );
    assert!(inspect_ok, "stderr: {inspect_stderr}");
    let inspect_json = parse_json(&inspect_stdout);
    assert_eq!(inspect_json["result"]["memory_id"], json!(memory_id));

    let (recall_ok, recall_stdout, recall_stderr) = run_membrain_with_db(
        &db_root,
        &[
            "recall",
            RELEASE_RECALL_QUERY,
            "--namespace",
            "test_ns",
            "--top",
            "2",
            "--explain",
            "full",
            "--json",
        ],
    );
    assert!(recall_ok, "stderr: {recall_stderr}");
    let recall_json = parse_json(&recall_stdout);
    assert_eq!(
        recall_json["result"]["evidence_pack"][0]["result"]["memory_id"],
        json!(memory_id)
    );
    assert_eq!(
        recall_json["result"]["evidence_pack"][0]["result"]["compact_text"],
        json!(RELEASE_RECALL_TARGET)
    );
    assert!(recall_json["result"]["explain"]["result_reasons"]
        .as_array()
        .expect("recall result_reasons should be an array")
        .iter()
        .any(|reason| reason["reason_code"] == json!("semantic_recall_trace")));

    let (explain_ok, explain_stdout, explain_stderr) = run_membrain_with_db(
        &db_root,
        &[
            "why",
            &memory_id.to_string(),
            "--namespace",
            "test_ns",
            "--json",
        ],
    );
    assert!(explain_ok, "stderr: {explain_stderr}");
    let explain_json = parse_json(&explain_stdout);
    assert_eq!(
        explain_json["result"]["evidence_pack"][0]["result"]["memory_id"],
        json!(memory_id)
    );
    assert!(explain_json["result"]["explain"]["result_reasons"]
        .as_array()
        .expect("why result_reasons should be an array")
        .iter()
        .any(|reason| reason["reason_code"] == json!("query_by_example_seed_materialized")));
}

#[test]
#[cfg(unix)]
fn cross_surface_restart_rehydrates_persisted_memory_for_cli_daemon_and_mcp() {
    let db_root = test_db_root();
    let namespace = "test_ns";

    let (encode_ok, encode_stdout, encode_stderr) = run_membrain_with_db(
        &db_root,
        &[
            "remember",
            RELEASE_RECALL_TARGET,
            "--namespace",
            namespace,
            "--kind",
            "semantic",
            "--json",
        ],
    );
    assert!(encode_ok, "stderr: {encode_stderr}");
    let encode_json = parse_json(&encode_stdout);
    let memory_id = encode_json["result"]["memory_id"]
        .as_u64()
        .expect("remember should return persisted memory id");

    for distractor in [
        RELEASE_RECALL_LEXICAL_DISTRACTOR,
        RELEASE_RECALL_UNRELATED_DISTRACTOR,
    ] {
        let (distractor_ok, _distractor_stdout, distractor_stderr) = run_membrain_with_db(
            &db_root,
            &[
                "remember",
                distractor,
                "--namespace",
                namespace,
                "--kind",
                "semantic",
                "--json",
            ],
        );
        assert!(distractor_ok, "stderr: {distractor_stderr}");
    }

    let (inspect_ok, inspect_stdout, inspect_stderr) = run_membrain_with_db(
        &db_root,
        &[
            "inspect",
            "--id",
            &memory_id.to_string(),
            "--namespace",
            namespace,
            "--json",
        ],
    );
    assert!(inspect_ok, "stderr: {inspect_stderr}");
    let inspect_json = parse_json(&inspect_stdout);
    assert_eq!(inspect_json["result"]["memory_id"], json!(memory_id));

    let (semantic_recall_ok, semantic_recall_stdout, semantic_recall_stderr) = run_membrain_with_db(
        &db_root,
        &[
            "recall",
            RELEASE_RECALL_QUERY,
            "--namespace",
            namespace,
            "--top",
            "2",
            "--explain",
            "full",
            "--json",
        ],
    );
    assert!(semantic_recall_ok, "stderr: {semantic_recall_stderr}");
    let semantic_recall_json = parse_json(&semantic_recall_stdout);
    assert_eq!(
        semantic_recall_json["result"]["evidence_pack"][0]["result"]["memory_id"],
        json!(memory_id)
    );
    assert_eq!(
        semantic_recall_json["result"]["evidence_pack"][0]["result"]["compact_text"],
        json!(RELEASE_RECALL_TARGET)
    );
    assert_eq!(
        semantic_recall_json["result"]["packaging_metadata"]["degraded_summary"],
        json!(null)
    );
    assert!(semantic_recall_json["result"]["explain"]["result_reasons"]
        .as_array()
        .expect("recall result_reasons should be an array")
        .iter()
        .any(|reason| reason["reason_code"] == json!("semantic_recall_trace")));

    let (why_ok, why_stdout, why_stderr) = run_membrain_with_db(
        &db_root,
        &[
            "why",
            &memory_id.to_string(),
            "--namespace",
            namespace,
            "--json",
        ],
    );
    assert!(why_ok, "stderr: {why_stderr}");
    let why_json = parse_json(&why_stdout);
    assert_eq!(
        why_json["result"]["evidence_pack"][0]["result"]["memory_id"],
        json!(memory_id)
    );
    assert!(why_json["result"]["explain"]["result_reasons"]
        .as_array()
        .expect("why result_reasons should be an array")
        .iter()
        .any(|reason| reason["reason_code"] == json!("query_by_example_seed_materialized")));

    let (health_ok, health_stdout, health_stderr) =
        run_membrain_with_db(&db_root, &["health", "--json"]);
    assert!(health_ok, "stderr: {health_stderr}");
    let health_json = parse_json(&health_stdout);
    assert_eq!(health_json["hot_memories"], json!(3));
    assert_eq!(health_json["total_encodes"], json!(3));
    assert_eq!(health_json["attention"]["total_encode_count"], json!(3));
    assert_eq!(
        health_json["attention"]["hotspots"][0]["namespace"],
        json!(namespace)
    );
    assert_eq!(
        health_json["feature_availability"][0]["feature"],
        json!("health")
    );

    let (doctor_ok, doctor_stdout, doctor_stderr) =
        run_membrain_with_db(&db_root, &["doctor", "--json"]);
    assert!(doctor_ok, "stderr: {doctor_stderr}");
    let doctor_json = parse_json(&doctor_stdout);
    assert_eq!(doctor_json["action"], json!("doctor"));
    assert_eq!(doctor_json["error_kind"], json!(null));
    assert_eq!(doctor_json["health"]["hot_memories"], json!(3));
    assert_eq!(doctor_json["health"]["total_encodes"], json!(3));
    assert_eq!(
        doctor_json["health"]["attention"]["total_encode_count"],
        json!(3)
    );

    let socket_path = unique_socket_path("cross-surface-parity");
    let mut daemon = spawn_membrain_daemon(&db_root, &socket_path);
    assert!(
        wait_for_socket(&socket_path),
        "daemon socket did not become ready: {}",
        socket_path.display()
    );

    let daemon_inspect = send_daemon_request(
        &socket_path,
        json!({
            "jsonrpc":"2.0",
            "method":"inspect",
            "params":{"id":memory_id,"namespace":namespace},
            "id":"daemon-inspect"
        }),
    );
    assert_eq!(daemon_inspect["result"]["status"], json!("ok"));
    assert_eq!(
        daemon_inspect["result"]["payload"]["memory_id"],
        json!(memory_id)
    );

    let daemon_recall = send_daemon_request(
        &socket_path,
        json!({
            "jsonrpc":"2.0",
            "method":"recall",
            "params":{"query_text":RELEASE_RECALL_QUERY,"namespace":namespace,"result_budget":2},
            "id":"daemon-recall"
        }),
    );
    assert_eq!(
        daemon_recall["result"]["retrieval"]["result"]["evidence_pack"][0]["result"]["memory_id"],
        json!(memory_id)
    );
    assert_eq!(
        daemon_recall["result"]["retrieval"]["result"]["evidence_pack"][0]["result"]
            ["compact_text"],
        json!(RELEASE_RECALL_TARGET)
    );
    assert_eq!(
        daemon_recall["result"]["retrieval"]["result"]["packaging_metadata"]["degraded_summary"],
        json!(null)
    );

    let daemon_why = send_daemon_request(
        &socket_path,
        json!({
            "jsonrpc":"2.0",
            "method":"why",
            "params":{"id":memory_id,"namespace":namespace},
            "id":"daemon-why"
        }),
    );
    assert_eq!(daemon_why["result"]["status"], json!("ok"));
    assert_eq!(
        daemon_why["result"]["retrieval"]["result"]["evidence_pack"][0]["result"]["memory_id"],
        json!(memory_id)
    );
    assert_eq!(
        daemon_why["result"]["retrieval"]["result"]["explain"]["query_by_example"]
            ["materialized_seed_descriptors"][0],
        json!(format!("memory:{memory_id}"))
    );

    let daemon_health = send_daemon_request(
        &socket_path,
        json!({"jsonrpc":"2.0","method":"health","params":{},"id":"daemon-health"}),
    );
    assert_eq!(daemon_health["result"]["hot_memories"], json!(3));
    assert_eq!(daemon_health["result"]["total_encodes"], json!(3));
    assert_eq!(
        daemon_health["result"]["attention"]["total_encode_count"],
        json!(3)
    );
    assert_eq!(
        daemon_health["result"]["attention"]["hotspots"][0]["namespace"],
        json!(namespace)
    );
    assert_eq!(
        daemon_health["result"]["feature_availability"][0]["feature"],
        json!("health")
    );

    let daemon_doctor = send_daemon_request(
        &socket_path,
        json!({"jsonrpc":"2.0","method":"doctor","params":{},"id":"daemon-doctor"}),
    );
    assert_eq!(daemon_doctor["result"]["action"], json!("doctor"));
    assert_eq!(daemon_doctor["result"]["error_kind"], json!(null));
    assert_eq!(daemon_doctor["result"]["health"]["hot_memories"], json!(3));
    assert_eq!(daemon_doctor["result"]["health"]["total_encodes"], json!(3));
    assert_eq!(
        daemon_doctor["result"]["health"]["attention"]["total_encode_count"],
        json!(3)
    );

    shutdown_membrain_daemon(&mut daemon, &socket_path);

    let mut mcp = spawn_membrain_mcp(&db_root);

    let mcp_recall = send_mcp_request(
        &mut mcp,
        json!({
            "jsonrpc": "2.0",
            "method": "recall",
            "params": {"query_text": RELEASE_RECALL_QUERY, "namespace": namespace, "result_budget": 2},
            "id": "mcp-recall"
        }),
    );
    assert_eq!(mcp_recall["result"]["status"], json!("ok"));
    assert_eq!(
        mcp_recall["result"]["retrieval"]["result"]["evidence_pack"][0]["result"]["memory_id"],
        json!(memory_id)
    );
    assert_eq!(
        mcp_recall["result"]["retrieval"]["result"]["evidence_pack"][0]["result"]["compact_text"],
        json!(RELEASE_RECALL_TARGET)
    );
    assert_eq!(
        mcp_recall["result"]["retrieval"]["result"]["packaging_metadata"]["degraded_summary"],
        json!(null)
    );

    let mcp_inspect = send_mcp_request(
        &mut mcp,
        json!({
            "jsonrpc": "2.0",
            "method": "inspect",
            "params": {"id": memory_id, "namespace": namespace},
            "id": "mcp-inspect"
        }),
    );
    assert_eq!(mcp_inspect["result"]["status"], json!("ok"));
    assert_eq!(
        mcp_inspect["result"]["payload"]["memory_id"],
        json!(memory_id)
    );

    let mcp_why = send_mcp_request(
        &mut mcp,
        json!({
            "jsonrpc": "2.0",
            "method": "why",
            "params": {"id": memory_id, "namespace": namespace},
            "id": "mcp-why"
        }),
    );
    assert_eq!(mcp_why["result"]["status"], json!("ok"));
    assert_eq!(
        mcp_why["result"]["retrieval"]["result"]["evidence_pack"][0]["result"]["memory_id"],
        json!(memory_id)
    );
    assert_eq!(
        mcp_why["result"]["retrieval"]["result"]["explain"]["query_by_example"]
            ["materialized_seed_descriptors"][0],
        json!(format!("memory:{memory_id}"))
    );

    let mcp_health = send_mcp_request(
        &mut mcp,
        json!({"jsonrpc":"2.0","method":"health","params":{},"id":"mcp-health"}),
    );
    assert_eq!(mcp_health["result"]["hot_memories"], json!(3));
    assert_eq!(mcp_health["result"]["total_encodes"], json!(3));
    assert_eq!(
        mcp_health["result"]["attention"]["total_encode_count"],
        json!(3)
    );
    assert_eq!(
        mcp_health["result"]["attention"]["hotspots"][0]["namespace"],
        json!(namespace)
    );
    assert_eq!(
        mcp_health["result"]["feature_availability"][0]["feature"],
        json!("health")
    );

    let mcp_doctor = send_mcp_request(
        &mut mcp,
        json!({"jsonrpc":"2.0","method":"doctor","params":{},"id":"mcp-doctor"}),
    );
    assert_eq!(mcp_doctor["result"]["action"], json!("doctor"));
    assert_eq!(mcp_doctor["result"]["error_kind"], json!(null));
    assert_eq!(mcp_doctor["result"]["health"]["hot_memories"], json!(3));
    assert_eq!(mcp_doctor["result"]["health"]["total_encodes"], json!(3));
    assert_eq!(
        mcp_doctor["result"]["health"]["attention"]["total_encode_count"],
        json!(3)
    );

    let shutdown_json = send_mcp_request(
        &mut mcp,
        json!({
            "jsonrpc": "2.0",
            "method": "shutdown",
            "params": {},
            "id": "mcp-shutdown"
        }),
    );
    assert_eq!(shutdown_json["result"]["shutting_down"], json!(true));
    let status = mcp.wait().expect("mcp process should exit");
    assert!(status.success(), "mcp exit status: {status}");
}

#[test]
fn mcp_protocol_discovers_bounded_live_tools_and_placeholder_prompts() {
    let db_root = test_db_root();
    let mut mcp = spawn_membrain_mcp(&db_root);

    let initialize = send_mcp_request(
        &mut mcp,
        json!({
            "jsonrpc":"2.0",
            "method":"initialize",
            "params":{
                "protocolVersion":"2024-11-05",
                "capabilities":{"tools":{}},
                "clientInfo":{"name":"cli-e2e","version":"1.0"}
            },
            "id":"mcp-initialize"
        }),
    );
    assert_eq!(initialize["result"]["protocolVersion"], json!("2024-11-05"));
    assert_eq!(
        initialize["result"]["serverInfo"]["name"],
        json!("membrain")
    );
    assert_eq!(
        initialize["result"]["capabilities"]["tools"]["listChanged"],
        json!(false)
    );
    assert_eq!(
        initialize["result"]["capabilities"]["prompts"]["listChanged"],
        json!(false)
    );

    let initialized = send_mcp_request(
        &mut mcp,
        json!({
            "jsonrpc":"2.0",
            "method":"notifications/initialized",
            "params":{},
            "id":"mcp-initialized"
        }),
    );
    assert_eq!(initialized["result"], json!({}));

    let tools = send_mcp_request(
        &mut mcp,
        json!({
            "jsonrpc":"2.0",
            "method":"tools/list",
            "params":{},
            "id":"mcp-tools-list"
        }),
    );
    let tool_names = tools["result"]["tools"]
        .as_array()
        .expect("tools array")
        .iter()
        .map(|tool| tool["name"].as_str().expect("tool name"))
        .collect::<Vec<_>>();
    assert_eq!(
        tool_names,
        vec!["encode", "recall", "inspect", "why", "health", "doctor"]
    );
    assert!(tools["result"]["tools"]
        .as_array()
        .expect("tools array")
        .iter()
        .any(|tool| {
            tool["name"] == json!("recall")
                && tool["description"]
                    .as_str()
                    .expect("recall description")
                    .contains("hydrated evidence")
        }));

    let tool_call = send_mcp_request(
        &mut mcp,
        json!({
            "jsonrpc":"2.0",
            "method":"tools/call",
            "params":{"name":"health","arguments":{}},
            "id":"mcp-tool-call-health"
        }),
    );
    assert!(tool_call["result"]["feature_availability"].is_array());
    assert_eq!(
        tool_call["result"]["feature_availability"][0]["feature"],
        json!("health")
    );

    let resources = send_mcp_request(
        &mut mcp,
        json!({
            "jsonrpc":"2.0",
            "method":"resources/list",
            "params":{},
            "id":"mcp-resources-list"
        }),
    );
    let resources = resources["result"]["resources"]
        .as_array()
        .expect("resources array");
    assert_eq!(resources.len(), 1);
    assert_eq!(resources[0]["uri"], json!("membrain://default/status"));

    let resource_read = send_mcp_request(
        &mut mcp,
        json!({
            "jsonrpc":"2.0",
            "method":"resources/read",
            "params":{"uri":"membrain://default/status"},
            "id":"mcp-resources-read"
        }),
    );
    assert_eq!(
        resource_read["result"]["contents"][0]["uri"],
        json!("membrain://default/status")
    );
    let status_payload = serde_json::from_str::<Value>(
        resource_read["result"]["contents"][0]["text"]
            .as_str()
            .expect("status payload text"),
    )
    .expect("status payload json");
    assert_eq!(status_payload["authority_mode"], json!("stdio_facade"));
    assert_eq!(status_payload["authoritative_runtime"], json!(false));

    let streams = send_mcp_request(
        &mut mcp,
        json!({
            "jsonrpc":"2.0",
            "method":"streams/list",
            "params":{},
            "id":"mcp-streams-list"
        }),
    );
    assert_eq!(streams["error"]["code"], json!(-32601));
    assert_eq!(
        streams["error"]["message"],
        json!("unknown method 'streams/list'")
    );

    let prompts = send_mcp_request(
        &mut mcp,
        json!({
            "jsonrpc":"2.0",
            "method":"prompts/list",
            "params":{},
            "id":"mcp-prompts-list"
        }),
    );
    assert_eq!(prompts["result"]["prompts"], json!([]));

    let prompt_get = send_mcp_request(
        &mut mcp,
        json!({
            "jsonrpc":"2.0",
            "method":"prompts/get",
            "params":{"name":"missing","arguments":{}},
            "id":"mcp-prompts-get"
        }),
    );
    assert_eq!(prompt_get["error"]["code"], json!(-32601));
    assert_eq!(
        prompt_get["error"]["message"],
        json!("unknown prompt: missing")
    );

    let shutdown_json = send_mcp_request(
        &mut mcp,
        json!({
            "jsonrpc":"2.0",
            "method":"shutdown",
            "params":{},
            "id":"mcp-shutdown"
        }),
    );
    assert_eq!(shutdown_json["result"]["shutting_down"], json!(true));
    let status = mcp.wait().expect("mcp process should exit");
    assert!(status.success(), "mcp exit status: {status}");
}

#[test]
#[cfg(unix)]
fn partial_archival_recovery_marker_survives_daemon_and_stdio_mcp_transport() {
    let db_root = test_db_root();
    let namespace = "test_ns";
    let memory_id = 41;
    seed_runtime_records(
        &db_root,
        &[seeded_runtime_record(
            memory_id,
            namespace,
            "partial archival recovery for rollout remediation proof",
            Some(FreshnessMarkers::archival_recovery_partial(None)),
        )],
    );

    let socket_path = unique_socket_path("partial-archival-marker");
    let mut daemon = spawn_membrain_daemon(&db_root, &socket_path);
    assert!(
        wait_for_socket(&socket_path),
        "daemon socket did not become ready: {}",
        socket_path.display()
    );

    let daemon_recall = send_daemon_request(
        &socket_path,
        json!({
            "jsonrpc":"2.0",
            "method":"recall",
            "params":{"query_text":"partial archival recovery rollout remediation","namespace":namespace},
            "id":"daemon-partial-archival-recall"
        }),
    );
    let daemon_markers = daemon_recall["result"]["retrieval"]["explain_trace"]["freshness_markers"]
        .as_array()
        .expect("daemon freshness markers array");
    assert!(daemon_markers
        .iter()
        .any(|marker| marker["code"] == json!("archival_recovery_partial")));
    assert_eq!(
        daemon_recall["result"]["retrieval"]["result"]["evidence_pack"][0]["freshness_markers"]
            ["durable_lifecycle_state"],
        json!("partially_restored")
    );
    assert_eq!(
        daemon_recall["result"]["retrieval"]["result"]["evidence_pack"][0]["freshness_markers"]
            ["routing_lifecycle_state"],
        json!("archival_recovery_partial")
    );

    shutdown_membrain_daemon(&mut daemon, &socket_path);

    let mut mcp = spawn_membrain_mcp(&db_root);
    let _ = send_mcp_request(
        &mut mcp,
        json!({
            "jsonrpc":"2.0",
            "method":"initialize",
            "params":{
                "protocolVersion":"2024-11-05",
                "capabilities":{"tools":{}},
                "clientInfo":{"name":"cli-e2e","version":"1.0"}
            },
            "id":"mcp-initialize"
        }),
    );

    let mcp_recall = send_mcp_request(
        &mut mcp,
        json!({
            "jsonrpc":"2.0",
            "method":"tools/call",
            "params":{
                "name":"recall",
                "arguments":{"query_text":"partial archival recovery rollout remediation","namespace":namespace}
            },
            "id":"mcp-partial-archival-recall"
        }),
    );
    let mcp_markers = mcp_recall["result"]["retrieval"]["explain_trace"]["freshness_markers"]
        .as_array()
        .expect("mcp freshness markers array");
    assert!(mcp_markers
        .iter()
        .any(|marker| marker["code"] == json!("archival_recovery_partial")));
    assert_eq!(
        mcp_recall["result"]["retrieval"]["result"]["evidence_pack"][0]["freshness_markers"]
            ["durable_lifecycle_state"],
        json!("partially_restored")
    );
    assert_eq!(
        mcp_recall["result"]["retrieval"]["result"]["evidence_pack"][0]["freshness_markers"]
            ["routing_lifecycle_state"],
        json!("archival_recovery_partial")
    );

    let mcp_why = send_mcp_request(
        &mut mcp,
        json!({
            "jsonrpc":"2.0",
            "method":"tools/call",
            "params":{
                "name":"why",
                "arguments":{"query":"partial archival recovery rollout remediation","namespace":namespace}
            },
            "id":"mcp-partial-archival-why"
        }),
    );
    let mcp_why_markers = mcp_why["result"]["retrieval"]["explain_trace"]["freshness_markers"]
        .as_array()
        .expect("mcp why freshness markers array");
    assert!(mcp_why_markers
        .iter()
        .any(|marker| marker["code"] == json!("archival_recovery_partial")));
    assert_eq!(
        mcp_why["result"]["retrieval"]["result"]["evidence_pack"][0]["freshness_markers"]
            ["durable_lifecycle_state"],
        json!("partially_restored")
    );
    assert_eq!(
        mcp_why["result"]["retrieval"]["result"]["evidence_pack"][0]["freshness_markers"]
            ["routing_lifecycle_state"],
        json!("archival_recovery_partial")
    );

    let shutdown_json = send_mcp_request(
        &mut mcp,
        json!({
            "jsonrpc":"2.0",
            "method":"shutdown",
            "params":{},
            "id":"mcp-shutdown"
        }),
    );
    assert_eq!(shutdown_json["result"]["shutting_down"], json!(true));
    let status = mcp.wait().expect("mcp process should exit");
    assert!(status.success(), "mcp exit status: {status}");
}

#[test]
fn cli_recall_json_preserves_route_and_result_fields() {
    let db_root = test_db_root();
    let (remember_ok, _remember_stdout, remember_stderr) = run_membrain_with_db(
        &db_root,
        &[
            "remember",
            "Paris is the capital of France",
            "--namespace",
            "test_ns",
            "--kind",
            "semantic",
            "--json",
        ],
    );
    assert!(remember_ok, "stderr: {remember_stderr}");

    let (ok, stdout, stderr) = run_membrain_with_db(
        &db_root,
        &[
            "recall",
            "capital of France",
            "--namespace",
            "test_ns",
            "--top",
            "3",
            "--explain",
            "full",
            "--json",
        ],
    );

    assert!(ok, "stderr: {stderr}");
    let json = parse_json(&stdout);
    assert_eq!(json["ok"], true);
    assert_eq!(json["namespace"], "test_ns");
    assert_eq!(json["outcome_class"], "accepted");
    assert!(json["route_summary"]["route_reason"].as_str().is_some());
    assert!(json["route_summary"]["tier1_consulted_first"]
        .as_bool()
        .is_some());
    assert!(json["result"]["evidence_pack"].is_array());
    assert_eq!(
        json["result"]["evidence_pack"][0]["result"]["compact_text"],
        "Paris is the capital of France"
    );
    assert_eq!(json["result"]["output_mode"], "balanced");
    assert!(json["result"]["action_pack"].is_array());
    assert_eq!(json["result"]["packaging_metadata"]["result_budget"], 3);
    assert_eq!(
        json["result"]["packaging_metadata"]["packaging_mode"],
        "evidence_plus_action"
    );
    assert_eq!(
        json["result"]["packaging_metadata"]["degraded_summary"],
        json!(null)
    );
    assert!(json["result"]["explain"]["result_reasons"]
        .as_array()
        .expect("result_reasons should be an array")
        .iter()
        .any(|value| value["reason_code"] == "semantic_recall_trace"));
}

#[test]
fn cli_recall_high_mode_can_suppress_action_pack_when_policy_caveat_exists() {
    let (ok, stdout, stderr) = run_membrain(&[
        "recall",
        "capital of France",
        "--namespace",
        "test_ns",
        "--top",
        "3",
        "--confidence",
        "high",
        "--include-public",
        "--json",
    ]);

    assert!(ok, "stderr: {stderr}");
    let json = parse_json(&stdout);
    assert_eq!(json["result"]["output_mode"], "strict");
    assert!(json["result"]["evidence_pack"].is_array());
    assert!(json["result"]["action_pack"].is_null());
    assert_eq!(
        json["result"]["packaging_metadata"]["packaging_mode"],
        "evidence_only"
    );
}

#[test]
fn cli_recall_semantic_query_prefers_semantic_winner_over_lexical_distractor() {
    let db_root = test_db_root();
    let namespace = "semantic_cli";

    let (lexical_ok, _lexical_stdout, lexical_stderr) = run_membrain_with_db(
        &db_root,
        &[
            "remember",
            "release rollback checklist after outage",
            "--namespace",
            namespace,
            "--kind",
            "semantic",
            "--json",
        ],
    );
    assert!(lexical_ok, "stderr: {lexical_stderr}");

    let (semantic_ok, _semantic_stdout, semantic_stderr) = run_membrain_with_db(
        &db_root,
        &[
            "remember",
            "production deploy pipeline remediation rollout for incident fix",
            "--namespace",
            namespace,
            "--kind",
            "semantic",
            "--json",
        ],
    );
    assert!(semantic_ok, "stderr: {semantic_stderr}");

    let (ok, stdout, stderr) = run_membrain_with_db(
        &db_root,
        &[
            "recall",
            "Which release deploy fix should we roll out after the pipeline outage?",
            "--namespace",
            namespace,
            "--top",
            "2",
            "--explain",
            "full",
            "--json",
        ],
    );

    assert!(ok, "stderr: {stderr}");
    let json = parse_json(&stdout);
    assert_eq!(json["ok"], true);
    assert_eq!(
        json["result"]["packaging_metadata"]["degraded_summary"],
        json!(null)
    );
    assert_eq!(
        json["result"]["evidence_pack"][0]["result"]["compact_text"],
        "production deploy pipeline remediation rollout for incident fix"
    );
    assert_eq!(
        json["result"]["evidence_pack"][0]["result"]["entry_lane"],
        "exact"
    );
    assert!(json["result"]["explain"]["result_reasons"]
        .as_array()
        .expect("result_reasons should be an array")
        .iter()
        .any(|value| value["reason_code"] == "semantic_recall_trace"));
}

#[test]
fn cli_explain_json_preserves_trace_stages_and_patterns() {
    let db_root = test_db_root();
    let (remember_ok, _remember_stdout, remember_stderr) = run_membrain_with_db(
        &db_root,
        &[
            "remember",
            "deploy the service after the last incident by restoring the healthy release and verifying the rollback checklist",
            "--namespace",
            "test_ns",
            "--kind",
            "semantic",
            "--json",
        ],
    );
    assert!(remember_ok, "stderr: {remember_stderr}");

    let (ok, stdout, stderr) = run_membrain_with_db(
        &db_root,
        &[
            "why",
            "how to deploy the service after the last incident?",
            "--namespace",
            "test_ns",
            "--json",
        ],
    );

    assert!(ok, "stderr: {stderr}");
    let json = parse_json(&stdout);
    assert_eq!(json["ok"], true);
    assert_eq!(json["namespace"], "test_ns");
    assert_eq!(json["outcome_class"], "accepted");
    assert!(json["trace_stages"].is_array());
    assert_eq!(json["result"]["explain"]["ranking_profile"], "balanced");
    assert_eq!(json["result"]["evidence_pack"][0]["result"]["memory_id"], 1);
    assert!(json["result"]["explain"]["result_reasons"]
        .as_array()
        .expect("result_reasons should be an array")
        .iter()
        .any(|value| value["detail"]
            .as_str()
            .is_some_and(|detail| detail.contains("matched_patterns=how to"))));
    assert!(json["result"]["explain"]["result_reasons"]
        .as_array()
        .expect("result_reasons should be an array")
        .iter()
        .any(|value| value["reason_code"] == "semantic_explain_trace"));
}

#[test]
fn cli_inspect_missing_memory_json_preserves_validation_failure_taxonomy() {
    let (ok, stdout, _stderr) =
        run_membrain(&["inspect", "--id", "123", "--namespace", "test_ns", "--json"]);

    assert!(
        ok,
        "inspect --json should return a failure envelope with exit 0"
    );
    let json = parse_json(&stdout);
    assert_eq!(json["ok"], false);
    assert_eq!(json["error_kind"], "validation_failure");
    assert_eq!(json["outcome_class"], "rejected");
    assert_eq!(json["request_id"], "inspect-not-found");
    assert_eq!(json["remediation"]["summary"], "validation_failure");
    assert_eq!(json["remediation"]["next_steps"][0], "fix_request");
}

#[test]
fn cli_observe_json_surfaces_passive_observation_retention_and_provenance() {
    let (ok, stdout, stderr) = run_membrain(&[
        "observe",
        "watcher noticed deploy drift\n\nwatcher saw cache warmup finish",
        "--namespace",
        "test_ns",
        "--source-label",
        "stdin:e2e",
        "--json",
    ]);

    assert!(ok, "stderr: {stderr}");
    let json = parse_json(&stdout);
    assert_eq!(json["ok"], true);
    assert_eq!(json["namespace"], "test_ns");
    assert_eq!(json["outcome_class"], "accepted");
    assert_eq!(json["passive_observation"]["source_kind"], "observation");
    assert_eq!(json["passive_observation"]["write_decision"], "capture");
    assert_eq!(json["passive_observation"]["captured_as_observation"], true);
    assert_eq!(
        json["passive_observation"]["observation_source"],
        json!({"Present": "stdin:e2e"})
    );
    assert_eq!(
        json["passive_observation"]["observation_chunk_id"]
            .as_object()
            .and_then(|value| value.get("Present"))
            .and_then(|value| value.as_str())
            .map(|value| value.starts_with("obs-")),
        Some(true)
    );
    assert_eq!(
        json["passive_observation"]["retention_marker"],
        json!({"Present": "volatile_observation"})
    );
}

#[test]
fn cli_schemas_json_lists_bounded_schema_artifacts() {
    let (ok, stdout, stderr) = run_membrain(&[
        "schemas",
        "--namespace",
        "test_ns",
        "--top",
        "2",
        "--min-episode-count",
        "3",
        "--json",
    ]);

    assert!(ok, "stderr: {stderr}");
    let json = parse_json(&stdout);
    assert_eq!(json["ok"], true);
    assert_eq!(json["namespace"], "test_ns");
    assert_eq!(json["result"]["outcome"], "accepted");
    assert_eq!(json["result"]["namespace"], "test_ns");
    assert_eq!(json["result"]["top"], 2);
    assert!(json["result"]["schemas"].is_array());
    assert_eq!(json["result"]["schemas"].as_array().map(Vec::len), Some(1));
    assert_eq!(json["result"]["schemas"][0]["source_count"], 3);
    assert_eq!(json["result"]["schemas"][0]["confidence"], 740);
    assert!(json["result"]["schemas"][0]["content"]
        .as_str()
        .is_some_and(|content| content.contains("schema pattern")));
    assert_eq!(
        json["result"]["schemas"][0]["keywords"],
        json!(["deploy", "rollback", "canary"])
    );
    assert_eq!(
        json["result"]["schemas"][0]["compressed_member_ids"],
        json!([1, 2, 3])
    );
}

#[test]
fn cli_goal_show_json_surfaces_blackboard_projection_and_authoritative_truth() {
    let (ok, stdout, stderr) =
        run_membrain(&["goal", "show", "--task", "deploy-incident", "--json"]);

    assert!(ok, "stderr: {stderr}");
    let json = parse_json(&stdout);
    assert_eq!(json["ok"], true);
    assert_eq!(json["namespace"], "default");
    assert_eq!(json["result"]["status"], "active");
    assert_eq!(json["result"]["authoritative_truth"], "durable_memory");
    assert_eq!(
        json["result"]["blackboard_state"]["Present"]["projection_kind"],
        "working_state_projection"
    );
    assert_eq!(
        json["result"]["blackboard_state"]["Present"]["authoritative_truth"],
        "durable_memory"
    );
    assert_eq!(json["warnings"][0]["code"], "goal_get");
}

#[test]
fn cli_goal_pause_resume_snapshot_and_abandon_json_emit_checkpointed_surfaces() {
    let (ok_pause, stdout_pause, stderr_pause) = run_membrain(&[
        "goal",
        "pause",
        "--task",
        "deploy-incident",
        "--note",
        "waiting for approval",
        "--json",
    ]);
    assert!(ok_pause, "stderr: {stderr_pause}");
    let pause = parse_json(&stdout_pause);
    assert_eq!(pause["result"]["status"], "dormant");
    assert_eq!(
        pause["result"]["checkpoint"]["authoritative_truth"],
        "durable_memory"
    );
    assert_eq!(pause["warnings"][0]["code"], "goal_pause");

    let (ok_resume, stdout_resume, stderr_resume) =
        run_membrain(&["goal", "resume", "--task", "deploy-incident", "--json"]);
    assert!(ok_resume, "stderr: {stderr_resume}");
    let resume = parse_json(&stdout_resume);
    assert_eq!(resume["result"]["status"], "active");
    assert_eq!(resume["result"]["restored_evidence_handles"], json!([1, 2]));
    assert_eq!(resume["warnings"][0]["code"], "goal_resume");

    let (ok_snapshot, stdout_snapshot, stderr_snapshot) = run_membrain(&[
        "goal",
        "snapshot",
        "--task",
        "deploy-incident",
        "--note",
        "handoff",
        "--json",
    ]);
    assert!(ok_snapshot, "stderr: {stderr_snapshot}");
    let snapshot = parse_json(&stdout_snapshot);
    assert_eq!(
        snapshot["result"]["snapshot"]["artifact_kind"],
        "blackboard_snapshot"
    );
    assert_eq!(snapshot["result"]["authoritative_truth"], "durable_memory");
    assert_eq!(snapshot["warnings"][0]["code"], "goal_snapshot");

    let (ok_abandon, stdout_abandon, stderr_abandon) = run_membrain(&[
        "goal",
        "abandon",
        "--task",
        "deploy-incident",
        "--reason",
        "superseded by rollback",
        "--json",
    ]);
    assert!(ok_abandon, "stderr: {stderr_abandon}");
    let abandon = parse_json(&stdout_abandon);
    assert_eq!(abandon["result"]["status"], "abandoned");
    assert_eq!(abandon["result"]["authoritative_truth"], "durable_memory");
    assert_eq!(abandon["warnings"][0]["code"], "goal_abandon");
}

#[test]
fn cli_goal_resume_without_checkpoint_reports_stale_explicitly() {
    let (ok, stdout, stderr) = run_membrain(&["goal", "resume", "--json"]);

    assert!(ok, "stderr: {stderr}");
    let json = parse_json(&stdout);
    assert_eq!(json["result"]["status"], "stale");
    assert_eq!(json["result"]["checkpoint"]["stale"], true);
    assert_eq!(json["result"]["warnings"][0]["code"], "not_dormant");
    assert_eq!(json["warnings"][0]["code"], "goal_resume");
}

#[test]
fn cli_maintenance_json_reports_rebuild_outputs() {
    let (ok, stdout, stderr) = run_membrain(&[
        "maintenance",
        "--action",
        "repair",
        "--namespace",
        "test_ns",
        "--json",
    ]);

    assert!(ok, "stderr: {stderr}");
    let json = parse_json(&stdout);
    assert_eq!(json["outcome"], "accepted");
    assert_eq!(json["action"], "repair");
    assert_eq!(json["namespace"], "test_ns");
    assert_eq!(json["targets_checked"], 7);
    assert_eq!(json["rebuilt"], 7);
    assert_eq!(json["affected_item_count"], 458);
    assert_eq!(json["error_count"], 0);
    assert_eq!(json["queue_depth_before"], 7);
    assert_eq!(json["queue_depth_after"], 0);
    assert_eq!(json["results"].as_array().map(Vec::len), Some(7));
    assert_eq!(
        json["results"][0]["verification_artifact_name"],
        "fts5_lexical_parity"
    );
    assert_eq!(
        json["results"][0]["parity_check"],
        "fts5_projection_matches_durable_truth"
    );
    assert_eq!(json["results"][0]["authoritative_rows"], 128);
    assert_eq!(json["results"][0]["derived_rows"], 128);
    assert_eq!(
        json["results"][0]["durable_sources"],
        json!([
            "durable_memory_records",
            "namespace_policy_metadata",
            "canonical_content_handles"
        ])
    );
    assert_eq!(
        json["results"][2]["durable_sources"],
        json!([
            "durable_memory_records",
            "canonical_embeddings",
            "namespace_policy_metadata"
        ])
    );
}

#[test]
fn cli_audit_json_preserves_request_and_run_correlation_fields() {
    let (ok, stdout, stderr) = run_membrain(&[
        "audit",
        "--namespace",
        "team.alpha",
        "--id",
        "21",
        "--recent",
        "1",
        "--json",
    ]);

    assert!(ok, "stderr: {stderr}");
    let json = parse_json(&stdout);
    assert_eq!(json["total_matches"], 3);
    assert_eq!(json["returned_rows"], 1);
    assert_eq!(json["truncated"], true);
    let rows = json["rows"]
        .as_array()
        .expect("audit output should include rows");
    assert_eq!(rows.len(), 1);
    assert_eq!(rows[0]["request_id"], "req-migration-21");
    assert_eq!(rows[0]["related_run"], "migration-0042");
    assert_eq!(rows[0]["kind"], "maintenance_migration_applied");
}

#[test]
fn cli_audit_json_can_filter_by_session_id() {
    let (ok, stdout, stderr) = run_membrain(&[
        "audit",
        "--namespace",
        "team.alpha",
        "--session",
        "5",
        "--json",
    ]);

    assert!(ok, "stderr: {stderr}");
    let json = parse_json(&stdout);
    assert_eq!(json["total_matches"], 1);
    assert_eq!(json["returned_rows"], 1);
    assert_eq!(json["truncated"], false);
    let rows = json["rows"]
        .as_array()
        .expect("audit output should include rows");
    assert_eq!(rows.len(), 1);
    assert_eq!(rows[0]["session_id"], 5);
    assert_eq!(rows[0]["kind"], "encode_accepted");
}

#[test]
fn cli_audit_json_can_filter_by_event_family() {
    let (ok, stdout, stderr) = run_membrain(&[
        "audit",
        "--namespace",
        "team.alpha",
        "--id",
        "21",
        "--op",
        "policy",
        "--json",
    ]);

    assert!(ok, "stderr: {stderr}");
    let json = parse_json(&stdout);
    assert_eq!(json["total_matches"], 1);
    assert_eq!(json["returned_rows"], 1);
    assert_eq!(json["truncated"], false);
    let rows = json["rows"]
        .as_array()
        .expect("audit output should include rows");
    assert_eq!(rows.len(), 1);
    assert_eq!(rows[0]["kind"], "policy_redacted");
    assert_eq!(rows[0]["request_id"], "req-policy-21");
}

#[test]
fn cli_audit_json_can_filter_by_min_sequence() {
    let (ok, stdout, stderr) = run_membrain(&[
        "audit",
        "--namespace",
        "team.alpha",
        "--id",
        "21",
        "--since",
        "2",
        "--json",
    ]);

    assert!(ok, "stderr: {stderr}");
    let json = parse_json(&stdout);
    assert_eq!(json["total_matches"], 2);
    assert_eq!(json["returned_rows"], 2);
    assert_eq!(json["truncated"], false);
    let rows = json["rows"]
        .as_array()
        .expect("audit output should include rows");
    assert_eq!(rows.len(), 2);
    assert_eq!(rows[0]["kind"], "policy_redacted");
    assert_eq!(rows[1]["kind"], "maintenance_migration_applied");
}

#[test]
fn cli_doctor_json_reports_health_and_repair_state() {
    let (ok, stdout, stderr) = run_membrain(&["doctor", "--json"]);

    assert!(ok, "stderr: {stderr}");
    let json = parse_json(&stdout);
    assert_eq!(json["status"], "ok");
    assert_eq!(json["action"], "doctor");
    assert!(json["metrics"].is_object());
    assert_eq!(json["summary"]["ok_checks"], 4);
    assert_eq!(json["summary"]["warn_checks"], 1);
    assert_eq!(json["summary"]["fail_checks"], 0);
    assert_eq!(json["repair_engine_component"], "engine.repair");
    assert!(json["checks"].is_array());
    assert_eq!(json["checks"][1]["name"], "derived_indexes");
    assert_eq!(json["checks"][1]["status"], "warn");
    assert_eq!(json["checks"][3]["name"], "serving_posture");
    assert_eq!(json["checks"][3]["status"], "ok");
    assert_eq!(json["checks"][4]["name"], "lease_freshness");
    assert_eq!(json["checks"][4]["status"], "ok");
    assert_eq!(
        json["health"]["feature_availability"][1]["feature"],
        "runtime_authority"
    );
    assert_eq!(
        json["health"]["feature_availability"][1]["posture"],
        "Degraded"
    );
    assert_eq!(
        json["health"]["feature_availability"][1]["note"],
        "stdio_adapter_process_local:mode=stdio_facade authoritative_runtime=false maintenance_active=false warm_runtime_guarantees=local_process_state,best_effort_same_process_reuse,stdio_transport"
    );
    assert_eq!(
        json["health"]["feature_availability"][2]["feature"],
        "embedder_runtime"
    );
    assert_eq!(
        json["health"]["feature_availability"][2]["posture"],
        "Degraded"
    );
    assert_eq!(
        json["health"]["feature_availability"][2]["note"],
        "state=not_loaded backend=local_fastembed generation=all-minilm-l6-v2@default dimensions=384 loads=0 requests=0 cache_hits=0 cache_misses=0"
    );
    assert_eq!(
        json["health"]["lifecycle"]["consolidated_to_cold_count"],
        12
    );
    assert_eq!(
        json["health"]["lifecycle"]["reconsolidation_active_count"],
        2
    );
    assert_eq!(json["health"]["lifecycle"]["forgetting_archive_count"], 5);
    assert_eq!(
        json["health"]["lifecycle"]["background_maintenance_runs"],
        3
    );
    assert_eq!(
        json["health"]["lifecycle"]["background_maintenance_log"][0],
        "maintenance_consolidation_completed:cold_migration=12"
    );
    assert!(json["runbook_hints"].is_array());
    assert_eq!(
        json["runbook_hints"][0]["runbook_id"],
        "index_rebuild_operations"
    );
    assert!(json["indexes"].is_array());
    assert!(json["repair_reports"].is_array());
    assert_eq!(
        json["repair_reports"][0]["verification_artifact_name"],
        "fts5_lexical_parity"
    );
    assert_eq!(
        json["repair_reports"][0]["parity_check"],
        "fts5_projection_matches_durable_truth"
    );
    assert_eq!(json["repair_reports"][0]["authoritative_rows"], 128);
    assert_eq!(json["repair_reports"][0]["derived_rows"], 128);
    assert_eq!(json["repair_reports"][0]["queue_depth_before"], 4);
    assert_eq!(json["repair_reports"][3]["target"], "semantic_cold_index");
    assert_eq!(json["warnings"], json!([]));
    assert_eq!(json["error_kind"], json!(null));
    assert_eq!(json["availability"], json!(null));
    assert!(json["health"].is_object());
}

#[test]
fn cli_health_json_reports_dashboard_contract_fields() {
    let (ok, stdout, stderr) = run_membrain(&["health", "--json"]);

    assert!(ok, "stderr: {stderr}");
    let json = parse_json(&stdout);
    assert_eq!(json["hot_memories"], 76);
    assert_eq!(json["hot_capacity"], 100);
    assert_eq!(json["cold_memories"], 12);
    assert_eq!(json["unresolved_conflicts"], 1);
    assert_eq!(json["availability_posture"], json!(null));
    assert_eq!(json["backpressure_state"], "normal");
    assert_eq!(
        json["feature_availability"][1]["feature"],
        "runtime_authority"
    );
    assert_eq!(json["feature_availability"][1]["posture"], "Degraded");
    assert_eq!(
        json["feature_availability"][1]["note"],
        "stdio_adapter_process_local:mode=stdio_facade authoritative_runtime=false maintenance_active=false warm_runtime_guarantees=local_process_state,best_effort_same_process_reuse,stdio_transport"
    );
    assert_eq!(
        json["feature_availability"][2]["feature"],
        "embedder_runtime"
    );
    assert_eq!(json["feature_availability"][2]["posture"], "Degraded");
    assert_eq!(
        json["feature_availability"][2]["note"],
        "state=not_loaded backend=local_fastembed generation=all-minilm-l6-v2@default dimensions=384 loads=0 requests=0 cache_hits=0 cache_misses=0"
    );
    assert_eq!(json["attention"]["total_recall_count"], 29);
    assert_eq!(json["attention"]["hotspot_count"], 2);
    assert_eq!(json["attention"]["highest_namespace_pressure"], 6);
    assert_eq!(json["attention"]["max_attention_score"], 254);
    assert_eq!(json["attention"]["warming_candidate_count"], 1);
    assert_eq!(json["attention"]["hot_candidate_count"], 1);
    assert_eq!(json["attention"]["hotspots"][0]["namespace"], "team.alpha");
    assert_eq!(json["attention"]["hotspots"][0]["rank"], 1);
    assert_eq!(json["attention"]["hotspots"][0]["status"], "hot");
    assert_eq!(
        json["attention"]["hotspots"][0]["dominant_signal"],
        "recall_count"
    );
    assert_eq!(json["attention"]["hotspots"][0]["heat_bucket"], "hot");
    assert_eq!(json["attention"]["hotspots"][0]["heat_band"], 3);
    assert_eq!(
        json["attention"]["hotspots"][0]["prewarm_trigger"],
        "session_recency"
    );
    assert_eq!(
        json["attention"]["hotspots"][0]["prewarm_action"],
        "bounded_session_rewarm"
    );
    assert_eq!(
        json["attention"]["hotspots"][0]["prewarm_target_family"],
        "session_warmup"
    );
    assert_eq!(json["cache"]["prefetch_capacity"], 4);
    assert_eq!(json["cache"]["adaptive_prewarm_state"], "active");
    assert!(json["cache"]["adaptive_prewarm_summary"]
        .as_str()
        .expect("prewarm summary string")
        .contains("state=active queue_depth=1/4"));
    assert!(json["dashboard_views"]
        .as_array()
        .expect("dashboard views")
        .iter()
        .any(|view| view["view"] == "attention"));
    assert!(json["drill_down_paths"]
        .as_array()
        .expect("drill down paths")
        .iter()
        .any(|path| path["path"] == "/health/attention"));
    assert!(json["feature_availability"].is_array());
    assert!(json["subsystem_status"].is_array());
    assert!(json["trend_summary"].is_array());
    assert!(json["cache"].is_object());
    assert!(json["indexes"].is_object());
}

#[test]
fn cli_health_human_output_renders_operator_dashboard_sections() {
    let (ok, stdout, stderr) = run_membrain(&["health"]);

    assert!(ok, "stderr: {stderr}");
    assert!(stdout.contains("membrain — Brain Health"));
    assert!(stdout.contains("TIER UTILIZATION"));
    assert!(stdout.contains("QUALITY"));
    assert!(stdout.contains("ENGRAMS & SIGNALS"));
    assert!(stdout.contains("OPERATIONS"));
    assert!(stdout.contains("Prewarm       active  queue=1/4"));
    assert!(stdout.contains("Attention heatmap"));
    assert!(stdout.contains("#1 team.alpha [hot|band=3] score=254 dominant=recall_count prewarm=bounded_session_rewarm via session_recency -> session_warmup"));
    assert!(stdout.contains("summary: state=active queue_depth=1/4"));
    assert!(stdout.contains("SUBSYSTEM STATUS"));
    assert!(stdout.contains("ACTIVITY"));
    assert!(stdout.contains("Features      health:full"));
    assert!(stdout.contains("runtime_authority:degraded (stdio_adapter_process_local:mode=stdio_facade authoritative_runtime=false maintenance_active=false warm_runtime_guarantees=local_process_state,best_effort_same_process_reuse,stdio_transport)"));
    assert!(stdout.contains("embedder_runtime:degraded (state=not_loaded backend=local_fastembed generation=all-minilm-l6-v2@default dimensions=384 loads=0 requests=0 cache_hits=0 cache_misses=0)"));
    assert!(stdout.contains("Cache=unavailable"));
    assert!(stdout.contains("Index=healthy"));
}

#[test]
fn cli_mood_json_surfaces_current_snapshot() {
    let (ok, stdout, stderr) = run_membrain(&["mood", "--namespace", "default", "--json"]);

    assert!(ok, "stderr: {stderr}");
    let json = parse_json(&stdout);
    assert_eq!(json["ok"], true);
    assert_eq!(json["namespace"], "default");
    assert_eq!(json["result"]["namespace"], "default");
    assert_eq!(json["result"]["history_rows"], 0);
    assert_eq!(json["result"]["authoritative_truth"], "emotional_timeline");
    assert!(json["result"]["current_mood"].is_null());
    assert!(json["result"]["latest_tick"].is_null());
}

#[test]
fn cli_mood_history_json_surfaces_emotional_trajectory_rows() {
    let (ok, stdout, stderr) = run_membrain(&[
        "mood",
        "--history",
        "--namespace",
        "default",
        "--since",
        "1",
        "--json",
    ]);

    assert!(ok, "stderr: {stderr}");
    let json = parse_json(&stdout);
    assert_eq!(json["ok"], true);
    assert_eq!(json["namespace"], "default");
    assert_eq!(json["result"]["namespace"], "default");
    assert_eq!(json["result"]["since_tick"], 1);
    assert_eq!(json["result"]["total_rows"], 0);
    assert_eq!(json["result"]["authoritative_truth"], "emotional_timeline");
    assert!(json["result"]["rows"]
        .as_array()
        .expect("rows array")
        .is_empty());
}

#[test]
fn cli_mood_history_human_output_preserves_empty_history_contract() {
    let (ok, stdout, stderr) = run_membrain(&[
        "mood",
        "--history",
        "--namespace",
        "default",
        "--since",
        "1",
    ]);

    assert!(ok, "stderr: {stderr}");
    assert!(stdout.contains("Mood history namespace=default rows=0"));
    assert!(stdout.contains("since_tick: Some(1)"));
    assert!(stdout.contains("authoritative_truth: emotional_timeline"));
}

#[test]
fn cli_preflight_json_surfaces_shared_blocked_and_force_confirmed_fields() {
    let (explain_ok, explain_stdout, explain_stderr) = run_membrain(&[
        "preflight",
        "explain",
        "--namespace",
        "team.alpha",
        "--original-query",
        "delete prior audit events across all namespaces",
        "--proposed-action",
        "purge namespace audit history",
        "--json",
    ]);
    assert!(!explain_ok, "blocked explain should exit non-zero");
    assert!(explain_stderr.is_empty(), "stderr: {explain_stderr}");
    let explain = parse_json(&explain_stdout);
    assert_eq!(explain["allowed"], false);
    assert_eq!(explain["preflight_state"], "blocked");
    assert_eq!(explain["preflight_outcome"], "preview_only");
    assert_eq!(
        explain["blocked_reasons"],
        json!(["scope_ambiguous", "confirmation_required"])
    );
    assert_eq!(explain["confirmation"]["required"], true);
    assert_eq!(explain["confirmation"]["confirmed"], false);
    assert_eq!(explain["audit"]["actor_source"], "cli_preflight");
    assert!(explain["request_id"]
        .as_str()
        .expect("request id present")
        .starts_with("cli-preflight-explain-"));

    let (allow_ok, allow_stdout, allow_stderr) = run_membrain(&[
        "preflight",
        "allow",
        "--namespace",
        "team.alpha",
        "--original-query",
        "delete prior audit events",
        "--proposed-action",
        "purge namespace audit history",
        "--authorization-token",
        "allow-123",
        "--bypass-flag",
        "manual_override",
        "--json",
    ]);
    assert!(allow_ok, "stderr: {allow_stderr}");
    let allow = parse_json(&allow_stdout);
    assert_eq!(allow["success"], true);
    assert_eq!(allow["preflight_state"], "ready");
    assert_eq!(allow["preflight_outcome"], "force_confirmed");
    assert_eq!(allow["outcome_class"], "accepted");
    assert_eq!(allow["confirmation"]["confirmed"], true);
    assert_eq!(
        allow["confirmation_reason"],
        "operator confirmed exact previewed scope"
    );
    assert_eq!(allow["audit"]["actor_source"], "cli_preflight");
    assert!(allow["request_id"]
        .as_str()
        .expect("request id present")
        .starts_with("cli-preflight-allow-"));
}

#[test]
fn cli_preflight_human_output_preserves_blocked_and_force_confirmed_logs() {
    let (explain_ok, explain_stdout, explain_stderr) = run_membrain(&[
        "preflight",
        "explain",
        "--namespace",
        "team.alpha",
        "--original-query",
        "delete prior audit events across all namespaces",
        "--proposed-action",
        "purge namespace audit history",
    ]);
    assert!(!explain_ok, "blocked explain should exit non-zero");
    assert!(explain_stderr.is_empty(), "stderr: {explain_stderr}");
    assert!(
        explain_stdout.contains("Preflight explain [blocked] state=blocked outcome=preview_only")
    );
    assert!(explain_stdout.contains("blocked_reasons: scope_ambiguous, confirmation_required"));

    let (allow_ok, allow_stdout, allow_stderr) = run_membrain(&[
        "preflight",
        "allow",
        "--namespace",
        "team.alpha",
        "--original-query",
        "delete prior audit events",
        "--proposed-action",
        "purge namespace audit history",
        "--authorization-token",
        "allow-123",
        "--bypass-flag",
        "manual_override",
    ]);
    assert!(allow_ok, "stderr: {allow_stderr}");
    assert!(allow_stdout.contains("Preflight allow [accepted] state=ready outcome=force_confirmed"));
    assert!(allow_stdout.contains("confirmation_reason: operator confirmed exact previewed scope"));
}

#[test]
fn cli_share_json_reports_visibility_policy_and_audit_fields() {
    let (ok, stdout, stderr) =
        run_membrain(&["share", "--id", "42", "--namespace", "team.beta", "--json"]);

    assert!(ok, "stderr: {stderr}");
    let json = parse_json(&stdout);
    assert_eq!(json["ok"], true);
    assert_eq!(json["namespace"], "team.beta");
    assert_eq!(json["result"]["visibility"], "shared");
    assert_eq!(
        json["result"]["policy_summary"]["policy_family"],
        "visibility_sharing"
    );
    assert_eq!(json["result"]["audit"]["event_kind"], "approved_sharing");
    assert_eq!(json["result"]["audit"]["actor_source"], "cli_share");
    assert_eq!(json["result"]["audit"]["request_id"], "req-share-42");
    assert_eq!(json["result"]["audit"]["effective_namespace"], "team.beta");
    assert_eq!(json["result"]["audit"]["source_namespace"], "team.beta");
    assert_eq!(json["result"]["audit"]["target_namespace"], "team.beta");
    assert_eq!(
        json["result"]["audit"]["policy_family"],
        "visibility_sharing"
    );
    assert_eq!(json["result"]["audit"]["outcome_class"], "accepted");
    assert_eq!(json["result"]["audit"]["blocked_stage"], "policy_gate");
    assert_eq!(json["result"]["audit"]["related_run"], "share-run-42");
    assert_eq!(json["result"]["audit"]["redacted"], false);
    assert_eq!(
        json["result"]["audit_rows"][0]["request_id"],
        "req-share-42"
    );
    assert_eq!(json["result"]["audit_rows"][0]["kind"], "approved_sharing");
}

#[test]
fn cli_recall_json_preserves_explicit_public_widening_policy_filters() {
    let (ok, stdout, stderr) = run_membrain(&[
        "recall",
        "capital of France",
        "--namespace",
        "test_ns",
        "--top",
        "3",
        "--include-public",
        "--json",
    ]);

    assert!(ok, "stderr: {stderr}");
    let json = parse_json(&stdout);
    assert_eq!(json["ok"], true);
    assert_eq!(json["namespace"], "test_ns");
    assert_eq!(
        json["policy_filters_applied"][0]["policy_family"],
        "shared_public_widening"
    );
    assert_eq!(
        json["policy_filters_applied"][0]["sharing_scope"],
        serde_json::json!({"Present":"approved_shared"})
    );
    assert_eq!(
        json["policy_summary"]["filters"][0]["policy_family"],
        "shared_public_widening"
    );
    assert_eq!(
        json["policy_summary"]["filters"][0]["sharing_scope"],
        serde_json::json!({"Present":"approved_shared"})
    );
}

#[test]
fn cli_unshare_json_reports_redaction_and_audit_fields() {
    let (ok, stdout, stderr) = run_membrain(&[
        "unshare",
        "--id",
        "42",
        "--namespace",
        "team.alpha",
        "--json",
    ]);

    assert!(ok, "stderr: {stderr}");
    let json = parse_json(&stdout);
    assert_eq!(json["ok"], true);
    assert_eq!(json["namespace"], "team.alpha");
    assert_eq!(json["result"]["visibility"], "private");
    assert_eq!(
        json["result"]["policy_summary"]["redaction_fields"][0],
        "sharing_scope"
    );
    assert_eq!(json["result"]["audit"]["event_kind"], "policy_redacted");
    assert_eq!(json["result"]["audit"]["actor_source"], "cli_unshare");
    assert_eq!(json["result"]["audit"]["request_id"], "req-unshare-42");
    assert_eq!(json["result"]["audit"]["effective_namespace"], "team.alpha");
    assert_eq!(json["result"]["audit"]["source_namespace"], "team.alpha");
    assert_eq!(json["result"]["audit"]["target_namespace"], "team.alpha");
    assert_eq!(
        json["result"]["audit"]["policy_family"],
        "visibility_sharing"
    );
    assert_eq!(json["result"]["audit"]["outcome_class"], "accepted");
    assert_eq!(json["result"]["audit"]["blocked_stage"], "policy_gate");
    assert_eq!(json["result"]["audit"]["related_run"], "share-run-42");
    assert_eq!(json["result"]["audit"]["redacted"], true);
    assert_eq!(
        json["result"]["audit_rows"][0]["request_id"],
        "req-unshare-42"
    );
    assert_eq!(json["result"]["audit_rows"][0]["kind"], "policy_redacted");
    assert_eq!(json["result"]["audit_rows"][0]["redacted"], true);
}

#[test]
fn cli_share_and_unshare_json_audit_surfaces_keep_required_correlation_fields() {
    let (share_ok, share_stdout, share_stderr) =
        run_membrain(&["share", "--id", "42", "--namespace", "team.beta", "--json"]);
    assert!(share_ok, "stderr: {share_stderr}");
    let share = parse_json(&share_stdout);
    assert_eq!(share["result"]["audit"]["request_id"], "req-share-42");
    assert_eq!(share["result"]["audit"]["effective_namespace"], "team.beta");
    assert_eq!(share["result"]["audit"]["source_namespace"], "team.beta");
    assert_eq!(share["result"]["audit"]["target_namespace"], "team.beta");
    assert_eq!(share["result"]["audit"]["related_run"], "share-run-42");
    assert_eq!(share["result"]["audit"]["redaction_summary"], json!([]));

    let (unshare_ok, unshare_stdout, unshare_stderr) = run_membrain(&[
        "unshare",
        "--id",
        "42",
        "--namespace",
        "team.alpha",
        "--json",
    ]);
    assert!(unshare_ok, "stderr: {unshare_stderr}");
    let unshare = parse_json(&unshare_stdout);
    assert_eq!(unshare["result"]["audit"]["request_id"], "req-unshare-42");
    assert_eq!(
        unshare["result"]["audit"]["effective_namespace"],
        "team.alpha"
    );
    assert_eq!(unshare["result"]["audit"]["source_namespace"], "team.alpha");
    assert_eq!(unshare["result"]["audit"]["target_namespace"], "team.alpha");
    assert_eq!(unshare["result"]["audit"]["related_run"], "share-run-42");
    assert_eq!(
        unshare["result"]["audit"]["redaction_summary"],
        json!(["sharing_scope"])
    );
}

#[test]
fn cli_recall_human_output_logs_route_and_result_lines() {
    let db_root = test_db_root();
    let (remember_ok, _remember_stdout, remember_stderr) = run_membrain_with_db(
        &db_root,
        &[
            "remember",
            "Paris is the capital of France",
            "--namespace",
            "test_ns",
            "--kind",
            "semantic",
            "--json",
        ],
    );
    assert!(remember_ok, "stderr: {remember_stderr}");

    let (ok, stdout, stderr) = run_membrain_with_db(
        &db_root,
        &[
            "recall",
            "capital of France",
            "--namespace",
            "test_ns",
            "--top",
            "3",
            "--explain",
            "full",
        ],
    );

    assert!(ok, "stderr: {stderr}");
    assert!(stderr.is_empty(), "stderr should stay empty: {stderr}");
    assert!(stdout.contains("Recall 'capital of France' in 'test_ns' → 1 results"));
    assert!(stdout.contains("route: tier2_exact_then_graph_expansion → small_session_lookup"));
    assert!(stdout.contains("tier1: consulted=false, answered_directly=false, deeper=true"));
    assert!(stdout.contains("derived actions: 1"));
}

#[test]
fn cli_audit_human_output_logs_redaction_and_run_correlation() {
    let (ok, stdout, stderr) = run_membrain(&[
        "audit",
        "--namespace",
        "team.alpha",
        "--id",
        "21",
        "--recent",
        "1",
    ]);

    assert!(ok, "stderr: {stderr}");
    assert!(stderr.is_empty(), "stderr should stay empty: {stderr}");
    assert!(stdout.contains("matched=3 returned=1 truncated=true"));
    assert!(stdout.contains("#3 maintenance maintenance_migration_applied"));
    assert!(stdout.contains("request_id=Some(\"req-migration-21\")"));
    assert!(stdout.contains("redacted=false"));
    assert!(stdout.contains("run=Some(\"migration-0042\")"));
}

#[test]
fn cli_share_and_unshare_human_output_logs_visibility_transitions() {
    let (share_ok, share_stdout, share_stderr) =
        run_membrain(&["share", "--id", "42", "--namespace", "team.beta"]);
    assert!(share_ok, "stderr: {share_stderr}");
    assert!(
        share_stderr.is_empty(),
        "stderr should stay empty: {share_stderr}"
    );
    assert_eq!(
        share_stdout.trim(),
        "Shared memory #42 into 'team.beta' [shared]"
    );

    let (unshare_ok, unshare_stdout, unshare_stderr) =
        run_membrain(&["unshare", "--id", "42", "--namespace", "team.alpha"]);
    assert!(unshare_ok, "stderr: {unshare_stderr}");
    assert!(
        unshare_stderr.is_empty(),
        "stderr should stay empty: {unshare_stderr}"
    );
    assert_eq!(
        unshare_stdout.trim(),
        "Unshared memory #42 in 'team.alpha' [private]"
    );
}
