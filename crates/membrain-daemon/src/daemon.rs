use crate::mcp::{
    McpInspectPayload, McpResource, McpResourceListing, McpResourceReadPayload, McpResponse,
    McpRetrievalPayload, McpStream, McpStreamListing,
};

const DAEMON_RESOURCE_NAMESPACE: &str = "daemon.runtime";
const RUNTIME_STATUS_URI: &str = "membrain://daemon/runtime/status";
const RUNTIME_DOCTOR_URI: &str = "membrain://daemon/runtime/doctor";
const RUNTIME_STREAMS_URI: &str = "membrain://daemon/runtime/streams";
const INSPECT_RESOURCE_URI_TEMPLATE: &str = "membrain://{namespace}/memories/{memory_id}";
const SNAPSHOT_RESOURCE_URI_TEMPLATE: &str = "membrain://{namespace}/snapshots/{snapshot_name}";
const MAINTENANCE_STATUS_METHOD: &str = "maintenance.status";
use crate::preflight::{
    PreflightAllowRequest, PreflightExplainResponse, PreflightOutcome, PreflightRunRequest,
};
use crate::rpc::{
    busy_payload, cancelled_payload, JsonRpcRequest, JsonRpcResponse, RuntimeDoctorIndex,
    RuntimeDoctorReport, RuntimeMaintenanceAccepted, RuntimeMethodRequest, RuntimeMetrics,
    RuntimePosture, RuntimeRequest, RuntimeStatus,
};
use anyhow::Context;
use membrain_core::api::{NamespaceId, RequestId};
use membrain_core::config::RuntimeConfig;
use membrain_core::engine::recall::{RecallEngine, RecallRequest, RecallRuntime};
use membrain_core::engine::result::{
    QueryByExampleExplain, ResultReason, RetrievalExplain, RetrievalResultSet,
};
use membrain_core::engine::retrieval_planner::{QueryByExampleNormalization, RetrievalRequest};
use membrain_core::observability::OutcomeClass;
use membrain_core::policy::{
    OperationClass, PolicyDecision, PolicyGateway, PolicyModule, PreflightCheck,
    PreflightCheckStatus, PreflightState, SafeguardOutcome, SafeguardReasonCode, SafeguardRequest,
};
use membrain_core::types::{MemoryId, SessionId};
use serde_json::json;
use std::collections::VecDeque;
use std::os::unix::fs::FileTypeExt;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, AtomicU64, AtomicUsize, Ordering};
use std::sync::Arc;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::{UnixListener, UnixStream};
use tokio::sync::{Mutex, Notify, Semaphore};
use tokio::task::JoinSet;
use tokio::time::{sleep, Duration};

#[derive(Debug, Clone, PartialEq, Eq)]
struct NormalizedRecallContract {
    planner_request: RecallRequest,
    normalized_query_by_example:
        membrain_core::engine::retrieval_planner::QueryByExampleNormalization,
    result_budget: usize,
}

#[derive(Debug, Clone)]
struct EvaluatedPreflight {
    request_id: String,
    preflight_id: String,
    execution_id: Option<String>,
    outcome: SafeguardOutcome,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DaemonRuntimeConfig {
    pub socket_path: PathBuf,
    pub request_concurrency: usize,
    pub max_queue_depth: usize,
    pub maintenance_interval: Duration,
    pub maintenance_poll_budget: u32,
    pub maintenance_step_delay: Duration,
}

impl DaemonRuntimeConfig {
    pub fn new<P: AsRef<Path>>(socket_path: P) -> Self {
        Self {
            socket_path: socket_path.as_ref().to_path_buf(),
            request_concurrency: 8,
            max_queue_depth: 32,
            maintenance_interval: Duration::from_secs(60),
            maintenance_poll_budget: 4,
            maintenance_step_delay: Duration::from_millis(25),
        }
    }
}

#[derive(Debug)]
pub struct DaemonRuntime {
    config: DaemonRuntimeConfig,
    state: Arc<RuntimeState>,
}

#[derive(Debug)]
struct RuntimeState {
    posture: Mutex<RuntimePosture>,
    degraded_reasons: Mutex<Vec<String>>,
    active_requests: AtomicUsize,
    queued_requests: AtomicUsize,
    background_jobs: AtomicUsize,
    cancelled_requests: AtomicUsize,
    maintenance_runs: AtomicU64,
    next_request_id: AtomicU64,
    next_maintenance_id: AtomicU64,
    shutdown_requested: AtomicBool,
    shutdown_notify: Notify,
    request_slots: Semaphore,
    maintenance_history: Mutex<VecDeque<u64>>,
}

impl RuntimeState {
    fn new(config: &DaemonRuntimeConfig) -> Self {
        Self {
            posture: Mutex::new(RuntimePosture::Full),
            degraded_reasons: Mutex::new(Vec::new()),
            active_requests: AtomicUsize::new(0),
            queued_requests: AtomicUsize::new(0),
            background_jobs: AtomicUsize::new(0),
            cancelled_requests: AtomicUsize::new(0),
            maintenance_runs: AtomicU64::new(0),
            next_request_id: AtomicU64::new(1),
            next_maintenance_id: AtomicU64::new(1),
            shutdown_requested: AtomicBool::new(false),
            shutdown_notify: Notify::new(),
            request_slots: Semaphore::new(config.request_concurrency),
            maintenance_history: Mutex::new(VecDeque::new()),
        }
    }

    async fn status(&self) -> RuntimeStatus {
        RuntimeStatus {
            posture: self.posture.lock().await.clone(),
            degraded_reasons: self.degraded_reasons.lock().await.clone(),
            metrics: RuntimeMetrics {
                queue_depth: self.queued_requests.load(Ordering::SeqCst),
                active_requests: self.active_requests.load(Ordering::SeqCst),
                background_jobs: self.background_jobs.load(Ordering::SeqCst),
                cancelled_requests: self.cancelled_requests.load(Ordering::SeqCst),
                maintenance_runs: self.maintenance_runs.load(Ordering::SeqCst),
            },
        }
    }

    async fn set_posture(&self, posture: RuntimePosture, reasons: Vec<String>) -> RuntimeStatus {
        *self.posture.lock().await = posture;
        *self.degraded_reasons.lock().await = reasons;
        self.status().await
    }

    async fn doctor_report(&self) -> RuntimeDoctorReport {
        let status = self.status().await;
        let posture = status.posture.clone();
        let overall_status = match posture {
            RuntimePosture::Full => "ok",
            RuntimePosture::Degraded => "warn",
            RuntimePosture::ReadOnly | RuntimePosture::Offline => "fail",
        };
        let cache_health = if matches!(posture, RuntimePosture::Offline) {
            "fail"
        } else if matches!(posture, RuntimePosture::Degraded | RuntimePosture::ReadOnly) {
            "warn"
        } else {
            "ok"
        };
        let cache_usable = !matches!(posture, RuntimePosture::Offline);
        let warnings = if matches!(posture, RuntimePosture::Full) {
            Vec::new()
        } else {
            vec!["operator_review_recommended"]
        };

        RuntimeDoctorReport {
            status: overall_status,
            action: "doctor",
            posture,
            degraded_reasons: status.degraded_reasons,
            metrics: status.metrics,
            indexes: vec![
                RuntimeDoctorIndex {
                    family: "schema",
                    health: "ok",
                    usable: true,
                    entry_count: 1,
                    generation: "schema.v1",
                },
                RuntimeDoctorIndex {
                    family: "index",
                    health: "ok",
                    usable: true,
                    entry_count: 128,
                    generation: "durable.v1",
                },
                RuntimeDoctorIndex {
                    family: "graph",
                    health: "ok",
                    usable: true,
                    entry_count: 96,
                    generation: "durable.v1",
                },
                RuntimeDoctorIndex {
                    family: "cache",
                    health: cache_health,
                    usable: cache_usable,
                    entry_count: 32,
                    generation: "cache.v1",
                },
            ],
            warnings,
        }
    }

    fn next_request_id(&self) -> u64 {
        self.next_request_id.fetch_add(1, Ordering::SeqCst)
    }

    fn next_maintenance_id(&self) -> u64 {
        self.next_maintenance_id.fetch_add(1, Ordering::SeqCst)
    }

    fn request_shutdown(&self) {
        self.shutdown_requested.store(true, Ordering::SeqCst);
        self.shutdown_notify.notify_waiters();
    }

    fn is_shutdown(&self) -> bool {
        self.shutdown_requested.load(Ordering::SeqCst)
    }

    async fn record_maintenance_run(&self, maintenance_id: u64) {
        self.maintenance_runs.fetch_add(1, Ordering::SeqCst);
        let mut history = self.maintenance_history.lock().await;
        history.push_back(maintenance_id);
        if history.len() > 16 {
            history.pop_front();
        }
    }
}

impl DaemonRuntime {
    pub fn new<P: AsRef<Path>>(socket_path: P) -> Self {
        Self::with_config(DaemonRuntimeConfig::new(socket_path))
    }

    pub fn with_config(config: DaemonRuntimeConfig) -> Self {
        let state = Arc::new(RuntimeState::new(&config));
        Self { config, state }
    }

    pub fn socket_path(&self) -> &Path {
        &self.config.socket_path
    }

    pub async fn run_until_stopped(&self) -> anyhow::Result<()> {
        if let Some(parent) = self.config.socket_path.parent() {
            tokio::fs::create_dir_all(parent).await?;
        }

        self.remove_stale_socket().await?;

        let listener = UnixListener::bind(&self.config.socket_path)?;
        let state = Arc::clone(&self.state);
        let config = self.config.clone();
        let accept_state = Arc::clone(&self.state);
        let accept_config = config.clone();
        let mut tasks = JoinSet::new();

        tasks.spawn(async move {
            Self::accept_loop(listener, accept_state, accept_config).await;
        });

        let maintenance_state = Arc::clone(&self.state);
        let maintenance_config = config.clone();
        tasks.spawn(async move {
            Self::maintenance_loop(maintenance_state, maintenance_config).await;
        });

        state.shutdown_notify.notified().await;

        while let Some(result) = tasks.join_next().await {
            match result {
                Ok(()) => {}
                Err(err) if err.is_cancelled() => {}
                Err(err) => return Err(err.into()),
            }
        }

        self.remove_stale_socket().await?;
        Ok(())
    }

    async fn accept_loop(
        listener: UnixListener,
        state: Arc<RuntimeState>,
        config: DaemonRuntimeConfig,
    ) {
        let mut request_tasks = JoinSet::new();
        loop {
            if state.is_shutdown() {
                break;
            }

            tokio::select! {
                accept_res = tokio::time::timeout(Duration::from_millis(50), listener.accept()) => {
                    match accept_res {
                        Ok(Ok((stream, _addr))) => {
                            let queued = state.queued_requests.fetch_add(1, Ordering::SeqCst) + 1;
                            if queued > config.max_queue_depth {
                                state.queued_requests.fetch_sub(1, Ordering::SeqCst);
                                let response = JsonRpcResponse::error(
                                    None,
                                    -32001,
                                    "runtime queue is full",
                                    Some(busy_payload(queued - 1, config.max_queue_depth)),
                                );
                                let _ = Self::write_response(stream, &response).await;
                                continue;
                            }

                            let state_clone = Arc::clone(&state);
                            request_tasks.spawn(async move {
                                Self::handle_connection(stream, state_clone).await;
                            });
                        }
                        Ok(Err(err)) => {
                            if !state.is_shutdown() {
                                eprintln!("Failed to accept socket connection: {err}");
                            }
                        }
                        Err(_) => {}
                    }
                }
                Some(joined) = request_tasks.join_next() => {
                    if let Err(err) = joined {
                        if !err.is_cancelled() {
                            eprintln!("Request task failed: {err}");
                        }
                    }
                }
            }
        }

        request_tasks.abort_all();
        while let Some(joined) = request_tasks.join_next().await {
            if let Err(err) = joined {
                if !err.is_cancelled() {
                    eprintln!("Request task failed during shutdown: {err}");
                }
            }
        }
    }

    async fn handle_connection(stream: UnixStream, state: Arc<RuntimeState>) {
        let queued_guard = QueueGuard::new(Arc::clone(&state));
        let request_correlation_id = state.next_request_id();

        let (reader_half, mut writer_half) = stream.into_split();
        let mut reader = BufReader::new(reader_half);
        let mut line = String::new();
        let response = tokio::select! {
            _ = state.shutdown_notify.notified() => {
                state.cancelled_requests.fetch_add(1, Ordering::SeqCst);
                None
            }
            read_result = reader.read_line(&mut line) => {
                match read_result {
                    Ok(0) => {
                        drop(queued_guard);
                        Some(JsonRpcResponse::error(None, -32700, "empty request", None))
                    }
                    Ok(_) => match serde_json::from_str::<JsonRpcRequest>(&line) {
                        Ok(request) => {
                            let is_notification = request.id.is_none();
                            if request.method == "shutdown" {
                                drop(queued_guard);
                                let response = Self::dispatch_request(request, Arc::clone(&state), request_correlation_id).await;
                                (!is_notification).then_some(response)
                            } else {
                                let permit = tokio::select! {
                                    _ = state.shutdown_notify.notified() => {
                                        state.cancelled_requests.fetch_add(1, Ordering::SeqCst);
                                        if is_notification {
                                            return;
                                        }
                                        return Self::write_serialized_response(
                                            &mut writer_half,
                                            &JsonRpcResponse::error(
                                                request.id.clone(),
                                                -32002,
                                                "request cancelled during shutdown",
                                                Some(cancelled_payload()),
                                            ),
                                        ).await;
                                    }
                                    permit = state.request_slots.acquire() => {
                                        match permit {
                                            Ok(permit) => permit,
                                            Err(_) => {
                                                return;
                                            }
                                        }
                                    }
                                };
                                drop(queued_guard);
                                let _active_guard = ActiveRequestGuard::new(Arc::clone(&state));
                                let response = Self::dispatch_request(request, Arc::clone(&state), request_correlation_id).await;
                                drop(permit);
                                (!is_notification).then_some(response)
                            }
                        }
                        Err(err) => {
                            drop(queued_guard);
                            Some(JsonRpcResponse::error(None, -32700, format!("invalid json: {err}"), None))
                        }
                    },
                    Err(err) => {
                        drop(queued_guard);
                        Some(JsonRpcResponse::error(None, -32000, format!("failed to read request: {err}"), None))
                    }
                }
            }
        };

        let Some(response) = response else {
            return;
        };

        let serialized = match serde_json::to_vec(&response) {
            Ok(bytes) => bytes,
            Err(err) => {
                let fallback = JsonRpcResponse::error(
                    None,
                    -32603,
                    format!("failed to encode response: {err}"),
                    None,
                );
                match serde_json::to_vec(&fallback) {
                    Ok(bytes) => bytes,
                    Err(_) => return,
                }
            }
        };

        let _ = writer_half.write_all(&serialized).await;
        let _ = writer_half.write_all(b"\n").await;
    }

    async fn write_serialized_response(
        writer_half: &mut tokio::net::unix::OwnedWriteHalf,
        response: &JsonRpcResponse,
    ) {
        if let Ok(serialized) = serde_json::to_vec(response) {
            let _ = writer_half.write_all(&serialized).await;
            let _ = writer_half.write_all(b"\n").await;
        }
    }

    async fn dispatch_request(
        request: JsonRpcRequest,
        state: Arc<RuntimeState>,
        request_correlation_id: u64,
    ) -> JsonRpcResponse {
        let envelope = RuntimeMethodRequest {
            jsonrpc: request.jsonrpc,
            method: request.method,
            params: request.params.unwrap_or_else(|| json!({})),
            id: request.id,
        };

        let request_id = envelope.id.clone();
        let runtime_request = match envelope.parse_method() {
            Ok(runtime_request) => runtime_request,
            Err(err) => return JsonRpcResponse::error(request_id, err.code, err.message, err.data),
        };

        match runtime_request {
            RuntimeRequest::Ping => JsonRpcResponse::success(
                request_id,
                json!({ "ok": true, "request_correlation_id": request_correlation_id }),
            ),
            RuntimeRequest::Status => {
                let status = state.status().await;
                JsonRpcResponse::success(request_id, json!(status))
            }
            RuntimeRequest::Doctor => {
                let report = state.doctor_report().await;
                JsonRpcResponse::success(request_id, json!(report))
            }
            RuntimeRequest::Encode {
                content,
                namespace,
                memory_type: _,
            } => {
                let _ = (content, namespace);
                JsonRpcResponse::success(
                    request_id,
                    json!({
                        "status": "accepted",
                        "message": "encode envelope accepted; storage pipeline not yet wired"
                    }),
                )
            }
            RuntimeRequest::Recall {
                query_text,
                namespace,
                mode,
                result_budget,
                token_budget,
                time_budget_ms,
                context_text,
                effort,
                include_public,
                like_id,
                unlike_id,
                graph_mode,
                cold_tier,
                workspace_id,
                agent_id,
                session_id,
                task_id,
                memory_kinds,
                era_id,
                as_of_tick,
                at_snapshot,
                min_strength,
                min_confidence,
                show_decaying,
                mood_congruent,
            } => match Self::handle_recall(
                query_text,
                &namespace,
                mode,
                result_budget,
                token_budget,
                time_budget_ms,
                context_text,
                effort,
                include_public,
                like_id,
                unlike_id,
                graph_mode,
                cold_tier,
                workspace_id,
                agent_id,
                session_id,
                task_id,
                memory_kinds,
                era_id,
                as_of_tick,
                at_snapshot,
                min_strength,
                min_confidence,
                show_decaying,
                mood_congruent,
                request_correlation_id,
            ) {
                Ok(response) => JsonRpcResponse::success(request_id, json!(response)),
                Err(message) => JsonRpcResponse::error(request_id, -32602, message, None),
            },
            RuntimeRequest::Inspect { id, namespace } => {
                match Self::handle_inspect(id, &namespace, request_correlation_id) {
                    Ok(response) => JsonRpcResponse::success(request_id, json!(response)),
                    Err(message) => JsonRpcResponse::error(request_id, -32602, message, None),
                }
            }
            RuntimeRequest::Explain {
                query,
                namespace,
                limit,
            } => match Self::handle_explain(&query, &namespace, limit, request_correlation_id) {
                Ok(response) => JsonRpcResponse::success(request_id, json!(response)),
                Err(message) => JsonRpcResponse::error(request_id, -32602, message, None),
            },
            RuntimeRequest::PreflightRun {
                namespace,
                original_query,
                proposed_action,
            } => match Self::handle_preflight_run(
                &namespace,
                &original_query,
                &proposed_action,
                request_correlation_id,
            ) {
                Ok(response) => JsonRpcResponse::success(request_id, json!(response)),
                Err(message) => JsonRpcResponse::error(request_id, -32602, message, None),
            },
            RuntimeRequest::PreflightExplain {
                namespace,
                original_query,
                proposed_action,
            } => match Self::handle_preflight_explain(
                &namespace,
                &original_query,
                &proposed_action,
                request_correlation_id,
            ) {
                Ok(response) => JsonRpcResponse::success(request_id, json!(response)),
                Err(message) => JsonRpcResponse::error(request_id, -32602, message, None),
            },
            RuntimeRequest::PreflightAllow {
                namespace,
                authorization_token,
                bypass_flags,
            } => match Self::handle_preflight_allow(
                &namespace,
                &authorization_token,
                &bypass_flags,
                request_correlation_id,
            ) {
                Ok(response) => JsonRpcResponse::success(request_id, json!(response)),
                Err(message) => JsonRpcResponse::error(request_id, -32602, message, None),
            },
            RuntimeRequest::ResourcesList => JsonRpcResponse::success(
                request_id,
                json!(McpResponse::success(
                    serde_json::to_value(McpResourceListing {
                        request_id: RequestId::new(format!(
                            "daemon-resources-{request_correlation_id}"
                        ))
                        .expect("daemon-generated resource list request ids are valid"),
                        namespace: NamespaceId::new(DAEMON_RESOURCE_NAMESPACE)
                            .expect("static daemon resource namespace is valid"),
                        resources: vec![
                            McpResource {
                                uri: RUNTIME_STATUS_URI.to_string(),
                                name: "runtime-status".to_string(),
                                mime_type: "application/json".to_string(),
                                resource_kind: "runtime_status".to_string(),
                                description: Some(
                                    "Bounded runtime posture, queue, and maintenance status"
                                        .to_string(),
                                ),
                                uri_template: None,
                                examples: vec![RUNTIME_STATUS_URI.to_string()],
                            },
                            McpResource {
                                uri: RUNTIME_DOCTOR_URI.to_string(),
                                name: "runtime-doctor".to_string(),
                                mime_type: "application/json".to_string(),
                                resource_kind: "runtime_doctor".to_string(),
                                description: Some(
                                    "Inspectable runtime doctor report and index availability"
                                        .to_string(),
                                ),
                                uri_template: None,
                                examples: vec![RUNTIME_DOCTOR_URI.to_string()],
                            },
                            McpResource {
                                uri: RUNTIME_STREAMS_URI.to_string(),
                                name: "runtime-streams".to_string(),
                                mime_type: "application/json".to_string(),
                                resource_kind: "stream_listing".to_string(),
                                description: Some(
                                    "Declared daemon notification and streaming surfaces"
                                        .to_string(),
                                ),
                                uri_template: None,
                                examples: vec![RUNTIME_STREAMS_URI.to_string()],
                            },
                            McpResource {
                                uri: "membrain://team.alpha/memories/42".to_string(),
                                name: "memory-inspect".to_string(),
                                mime_type: "application/json".to_string(),
                                resource_kind: "inspect_payload".to_string(),
                                description: Some(
                                    "Canonical inspect payload shape for a namespace-bound memory"
                                        .to_string(),
                                ),
                                uri_template: Some(INSPECT_RESOURCE_URI_TEMPLATE.to_string()),
                                examples: vec!["membrain://team.alpha/memories/42".to_string()],
                            },
                            McpResource {
                                uri: "membrain://team.alpha/snapshots/current".to_string(),
                                name: "snapshot-view".to_string(),
                                mime_type: "application/json".to_string(),
                                resource_kind: "snapshot_view".to_string(),
                                description: Some(
                                    "Representative namespace snapshot resource shape for parity tests"
                                        .to_string(),
                                ),
                                uri_template: Some(SNAPSHOT_RESOURCE_URI_TEMPLATE.to_string()),
                                examples: vec!["membrain://team.alpha/snapshots/current".to_string()],
                            },
                        ],
                    })
                    .expect("resource listing serializes")
                )),
            ),
            RuntimeRequest::ResourceRead { uri } => match uri.as_str() {
                RUNTIME_STATUS_URI => {
                    let status = state.status().await;
                    let read_request_id = RequestId::new(format!(
                        "daemon-resource-read-status-{request_correlation_id}"
                    ))
                    .expect("daemon-generated resource read request ids are valid");
                    JsonRpcResponse::success(
                        request_id,
                        json!(McpResponse::success(
                            serde_json::to_value(McpResourceReadPayload {
                                request_id: read_request_id,
                                namespace: NamespaceId::new(DAEMON_RESOURCE_NAMESPACE)
                                    .expect("static daemon resource namespace is valid"),
                                uri,
                                mime_type: "application/json".to_string(),
                                resource_kind: "runtime_status".to_string(),
                                bounded: true,
                                payload: serde_json::to_value(status)
                                    .expect("runtime status serializes"),
                            })
                            .expect("resource read payload serializes")
                        )),
                    )
                }
                RUNTIME_DOCTOR_URI => {
                    let report = state.doctor_report().await;
                    let read_request_id = RequestId::new(format!(
                        "daemon-resource-read-doctor-{request_correlation_id}"
                    ))
                    .expect("daemon-generated resource read request ids are valid");
                    JsonRpcResponse::success(
                        request_id,
                        json!(McpResponse::success(
                            serde_json::to_value(McpResourceReadPayload {
                                request_id: read_request_id,
                                namespace: NamespaceId::new(DAEMON_RESOURCE_NAMESPACE)
                                    .expect("static daemon resource namespace is valid"),
                                uri,
                                mime_type: "application/json".to_string(),
                                resource_kind: "runtime_doctor".to_string(),
                                bounded: true,
                                payload: serde_json::to_value(report)
                                    .expect("runtime doctor report serializes"),
                            })
                            .expect("resource read payload serializes")
                        )),
                    )
                }
                RUNTIME_STREAMS_URI => {
                    let read_request_id = RequestId::new(format!(
                        "daemon-resource-read-streams-{request_correlation_id}"
                    ))
                    .expect("daemon-generated resource read request ids are valid");
                    JsonRpcResponse::success(
                        request_id,
                        json!(McpResponse::success(
                            serde_json::to_value(McpResourceReadPayload {
                                request_id: read_request_id,
                                namespace: NamespaceId::new(DAEMON_RESOURCE_NAMESPACE)
                                    .expect("static daemon resource namespace is valid"),
                                uri,
                                mime_type: "application/json".to_string(),
                                resource_kind: "stream_listing".to_string(),
                                bounded: true,
                                payload: serde_json::to_value(McpStreamListing {
                                    request_id: RequestId::new(format!(
                                        "daemon-streams-{request_correlation_id}"
                                    ))
                                    .expect("daemon-generated stream request ids are valid"),
                                    namespace: NamespaceId::new(DAEMON_RESOURCE_NAMESPACE)
                                        .expect("static daemon resource namespace is valid"),
                                    streams: vec![McpStream {
                                        name: "maintenance-status".to_string(),
                                        method: MAINTENANCE_STATUS_METHOD.to_string(),
                                        delivery: "jsonrpc_notification".to_string(),
                                        description: "Async maintenance acceptance and posture updates"
                                            .to_string(),
                                        example_subscriptions: vec![
                                            MAINTENANCE_STATUS_METHOD.to_string(),
                                        ],
                                    }],
                                })
                                .expect("stream listing serializes"),
                            })
                            .expect("resource read payload serializes")
                        )),
                    )
                }
                _ => JsonRpcResponse::error(
                    request_id,
                    -32602,
                    format!("unknown resource uri '{uri}'"),
                    Some(json!({"error_kind": "validation_failure"})),
                ),
            },
            RuntimeRequest::StreamsList => JsonRpcResponse::success(
                request_id,
                json!(McpResponse::success(
                    serde_json::to_value(McpStreamListing {
                        request_id: RequestId::new(format!(
                            "daemon-streams-{request_correlation_id}"
                        ))
                        .expect("daemon-generated stream request ids are valid"),
                        namespace: NamespaceId::new(DAEMON_RESOURCE_NAMESPACE)
                            .expect("static daemon resource namespace is valid"),
                        streams: vec![McpStream {
                            name: "maintenance-status".to_string(),
                            method: MAINTENANCE_STATUS_METHOD.to_string(),
                            delivery: "jsonrpc_notification".to_string(),
                            description: "Async maintenance acceptance and posture updates"
                                .to_string(),
                            example_subscriptions: vec![MAINTENANCE_STATUS_METHOD.to_string()],
                        }],
                    })
                    .expect("stream listing serializes")
                )),
            ),
            RuntimeRequest::Sleep { millis } => {
                tokio::select! {
                    _ = state.shutdown_notify.notified() => {
                        state.cancelled_requests.fetch_add(1, Ordering::SeqCst);
                        JsonRpcResponse::error(
                            request_id,
                            -32002,
                            "request cancelled during shutdown",
                            Some(cancelled_payload()),
                        )
                    }
                    _ = sleep(Duration::from_millis(millis)) => {
                        JsonRpcResponse::success(
                            request_id,
                            json!({ "slept_ms": millis, "request_correlation_id": request_correlation_id }),
                        )
                    }
                }
            }
            RuntimeRequest::SetPosture { posture, reasons } => {
                let posture = match posture.as_str() {
                    "full" => RuntimePosture::Full,
                    "degraded" => RuntimePosture::Degraded,
                    "read_only" => RuntimePosture::ReadOnly,
                    "offline" => RuntimePosture::Offline,
                    _ => {
                        return JsonRpcResponse::error(
                            request_id,
                            -32602,
                            format!("unknown posture '{posture}'"),
                            None,
                        );
                    }
                };
                let status = state.set_posture(posture, reasons).await;
                JsonRpcResponse::success(request_id, json!(status))
            }
            RuntimeRequest::RunMaintenance {
                polls_budget,
                step_delay_ms,
            } => {
                let maintenance_id = state.next_maintenance_id();
                let state_clone = Arc::clone(&state);
                let polls_budget = polls_budget.unwrap_or(4);
                let step_delay = Duration::from_millis(step_delay_ms.unwrap_or(25));
                tokio::spawn(async move {
                    let _guard = BackgroundJobGuard::new(Arc::clone(&state_clone));
                    if Self::run_maintenance_budget(&state_clone, polls_budget, step_delay).await {
                        state_clone.record_maintenance_run(maintenance_id).await;
                    }
                });
                JsonRpcResponse::success(
                    request_id,
                    json!(RuntimeMaintenanceAccepted {
                        maintenance_id,
                        polls_budget,
                    }),
                )
            }
            RuntimeRequest::Forget {
                id,
                namespace,
                mode,
                reason,
            } => {
                let _ = (&namespace, &mode, &reason);
                JsonRpcResponse::success(
                    request_id,
                    json!({
                        "status": "accepted",
                        "id": id,
                        "mode": mode.unwrap_or_else(|| "archive".to_string()),
                        "message": "forget envelope accepted; forgetting pipeline not yet wired"
                    }),
                )
            }
            RuntimeRequest::Pin {
                id,
                namespace,
                reason,
            } => {
                let _ = (&namespace, &reason);
                JsonRpcResponse::success(
                    request_id,
                    json!({
                        "status": "accepted",
                        "id": id,
                        "message": "pin envelope accepted; retention pipeline not yet wired"
                    }),
                )
            }
            RuntimeRequest::Consolidate { namespace, scope } => {
                let _ = (&namespace, &scope);
                JsonRpcResponse::success(
                    request_id,
                    json!({
                        "status": "accepted",
                        "scope": scope.unwrap_or_else(|| "session".to_string()),
                        "message": "consolidate envelope accepted; consolidation pipeline not yet wired"
                    }),
                )
            }
            RuntimeRequest::Share { id, namespace_id } => {
                let namespace = match NamespaceId::new(&namespace_id) {
                    Ok(namespace) => namespace,
                    Err(_) => {
                        return JsonRpcResponse::error(
                            request_id,
                            -32602,
                            "malformed namespace_id",
                            Some(json!({"error_kind": "validation_failure"})),
                        )
                    }
                };
                let response = json!({
                    "status": "accepted",
                    "id": id,
                    "namespace": namespace.as_str(),
                    "visibility": "shared",
                    "policy_summary": {
                        "effective_namespace": namespace.as_str(),
                        "policy_family": "visibility_sharing",
                        "outcome_class": "accepted",
                        "blocked_stage": "policy_gate",
                        "redaction_fields": [],
                        "retention_state": "absent",
                        "sharing_scope": "shared"
                    },
                    "policy_filters_applied": [{
                        "effective_namespace": namespace.as_str(),
                        "policy_family": "visibility_sharing",
                        "outcome_class": "accepted",
                        "blocked_stage": "policy_gate",
                        "sharing_scope": "shared",
                        "retention_marker": "absent",
                        "redaction_fields": []
                    }],
                    "audit": {
                        "request_id": format!("req-share-{id}"),
                        "event_kind": "policy_redacted",
                        "redacted": false
                    }
                });
                JsonRpcResponse::success(request_id, response)
            }
            RuntimeRequest::Unshare { id, namespace } => {
                let namespace = match NamespaceId::new(&namespace) {
                    Ok(namespace) => namespace,
                    Err(_) => {
                        return JsonRpcResponse::error(
                            request_id,
                            -32602,
                            "malformed namespace",
                            Some(json!({"error_kind": "validation_failure"})),
                        )
                    }
                };
                let response = json!({
                    "status": "accepted",
                    "id": id,
                    "namespace": namespace.as_str(),
                    "visibility": "private",
                    "policy_summary": {
                        "effective_namespace": namespace.as_str(),
                        "policy_family": "visibility_sharing",
                        "outcome_class": "accepted",
                        "blocked_stage": "policy_gate",
                        "redaction_fields": ["sharing_scope"],
                        "retention_state": "absent",
                        "sharing_scope": "private"
                    },
                    "policy_filters_applied": [{
                        "effective_namespace": namespace.as_str(),
                        "policy_family": "visibility_sharing",
                        "outcome_class": "accepted",
                        "blocked_stage": "policy_gate",
                        "sharing_scope": "private",
                        "retention_marker": "absent",
                        "redaction_fields": ["sharing_scope"]
                    }],
                    "audit": {
                        "request_id": format!("req-unshare-{id}"),
                        "event_kind": "policy_denied",
                        "redacted": true
                    }
                });
                JsonRpcResponse::success(request_id, response)
            }
            RuntimeRequest::Link {
                source_id,
                target_id,
                namespace,
                link_type,
            } => {
                let _ = (&namespace, &link_type);
                JsonRpcResponse::success(
                    request_id,
                    json!({
                        "status": "accepted",
                        "source_id": source_id,
                        "target_id": target_id,
                        "message": "link envelope accepted; graph pipeline not yet wired"
                    }),
                )
            }
            RuntimeRequest::Shutdown => {
                state.request_shutdown();
                JsonRpcResponse::success(
                    request_id,
                    json!({ "shutting_down": true, "request_correlation_id": request_correlation_id }),
                )
            }
        }
    }

    #[allow(clippy::too_many_arguments)]
    fn handle_recall(
        query_text: Option<String>,
        namespace: &str,
        mode: Option<String>,
        result_budget: Option<usize>,
        token_budget: Option<usize>,
        time_budget_ms: Option<u32>,
        context_text: Option<String>,
        effort: Option<String>,
        include_public: Option<bool>,
        like_id: Option<u64>,
        unlike_id: Option<u64>,
        graph_mode: Option<String>,
        cold_tier: Option<bool>,
        workspace_id: Option<String>,
        agent_id: Option<String>,
        session_id: Option<String>,
        task_id: Option<String>,
        memory_kinds: Option<Vec<String>>,
        era_id: Option<String>,
        as_of_tick: Option<u64>,
        at_snapshot: Option<String>,
        min_strength: Option<u16>,
        min_confidence: Option<f64>,
        show_decaying: Option<bool>,
        mood_congruent: Option<bool>,
        request_correlation_id: u64,
    ) -> Result<McpResponse, String> {
        let namespace_id = NamespaceId::new(namespace).map_err(|err| err.to_string())?;
        let normalized = Self::normalize_recall_contract(
            namespace_id,
            query_text,
            like_id,
            unlike_id,
            graph_mode.as_deref(),
            result_budget,
        )?;
        let degraded_summary = Self::recall_degraded_summary(
            &mode,
            token_budget,
            time_budget_ms,
            &context_text,
            &effort,
            include_public,
            &graph_mode,
            cold_tier,
            workspace_id.as_deref(),
            agent_id.as_deref(),
            session_id.as_deref(),
            task_id.as_deref(),
            memory_kinds.as_deref(),
            era_id.as_deref(),
            as_of_tick,
            at_snapshot.as_deref(),
            min_strength,
            min_confidence,
            show_decaying,
            mood_congruent,
            &normalized,
        );
        Self::handle_retrieval_method(
            normalized.planner_request,
            Some(&normalized.normalized_query_by_example),
            namespace,
            Some(normalized.result_budget),
            request_correlation_id,
            "recall",
            &degraded_summary,
        )
    }

    fn handle_inspect(
        id: u64,
        namespace: &str,
        request_correlation_id: u64,
    ) -> Result<McpResponse, String> {
        let namespace = NamespaceId::new(namespace).map_err(|err| err.to_string())?;
        let request_id = RequestId::new(format!("daemon-inspect-{request_correlation_id}"))
            .map_err(|err| err.to_string())?;
        let plan =
            RecallEngine.plan_recall(RecallRequest::exact(MemoryId(id)), RuntimeConfig::default());
        let mut result = RetrievalResultSet::empty(
            RetrievalExplain::from_plan(&plan, "balanced"),
            namespace.clone(),
        );
        result.outcome_class = OutcomeClass::Degraded;
        result.policy_summary.outcome_class = OutcomeClass::Degraded;
        result.packaging_metadata.result_budget = 1;
        result.packaging_metadata.degraded_summary =
            Some("planner-only inspect envelope; item hydration not implemented".to_string());
        let mut payload =
            McpInspectPayload::from_result(request_id, namespace.clone(), id, &result)
                .map_err(|err| err.to_string())?;
        let inspect_resource_uri = format!("membrain://{}/memories/{id}", namespace.as_str());
        payload.explain_trace.passive_observation = Some(json!({
            "resource_uri": inspect_resource_uri,
            "resource_kind": "inspect_payload",
            "resource_template": INSPECT_RESOURCE_URI_TEMPLATE,
            "resource_examples": [format!("membrain://{}/memories/{id}", namespace.as_str())],
        }));
        Ok(McpResponse::success(
            serde_json::to_value(payload).map_err(|err| err.to_string())?,
        ))
    }

    fn handle_explain(
        query: &str,
        namespace: &str,
        limit: Option<usize>,
        request_correlation_id: u64,
    ) -> Result<McpResponse, String> {
        let normalized = Self::normalize_recall_contract(
            NamespaceId::new(namespace).map_err(|err| err.to_string())?,
            Some(query.to_string()),
            None,
            None,
            None,
            limit,
        )?;
        Self::handle_retrieval_method(
            normalized.planner_request,
            Some(&normalized.normalized_query_by_example),
            namespace,
            Some(normalized.result_budget),
            request_correlation_id,
            "explain",
            "planner-only explain envelope; evidence hydration not implemented",
        )
    }

    fn handle_retrieval_method(
        request: RecallRequest,
        query_by_example: Option<&QueryByExampleNormalization>,
        namespace: &str,
        limit: Option<usize>,
        request_correlation_id: u64,
        method_name: &str,
        degraded_summary: &str,
    ) -> Result<McpResponse, String> {
        let namespace = NamespaceId::new(namespace).map_err(|err| err.to_string())?;
        let request_id = RequestId::new(format!("daemon-{method_name}-{request_correlation_id}"))
            .map_err(|err| err.to_string())?;
        let plan = RecallEngine.plan_recall(request, RuntimeConfig::default());
        let result_budget = Self::canonical_result_budget(&request, limit);
        let mut explain = RetrievalExplain::from_plan(&plan, "balanced");
        if let Some(normalized) =
            query_by_example.filter(|normalized| normalized.has_example_seeds())
        {
            let requested_seed_descriptors = normalized.seed_descriptors();
            let expanded_candidate_count = requested_seed_descriptors.len().min(result_budget);
            let influence_summary = format!(
                "primary cue {} expanded {} candidate(s) from {} requested seed(s); planner-only daemon envelope did not materialize stored evidence yet",
                normalized.primary_cue.as_str(),
                expanded_candidate_count,
                requested_seed_descriptors.len(),
            );
            explain.query_by_example = Some(QueryByExampleExplain {
                primary_cue: normalized.primary_cue.as_str().to_string(),
                requested_seed_descriptors: requested_seed_descriptors.clone(),
                materialized_seed_descriptors: Vec::new(),
                missing_seed_descriptors: requested_seed_descriptors.clone(),
                expanded_candidate_count,
                influence_summary: influence_summary.clone(),
            });
            explain.result_reasons.extend(requested_seed_descriptors.iter().map(|descriptor| {
                ResultReason {
                    memory_id: None,
                    reason_code: "query_by_example_seed_missing".to_string(),
                    detail: format!(
                        "seed {descriptor} was requested but planner-only daemon recall did not materialize stored evidence"
                    ),
                }
            }));
            explain.result_reasons.push(ResultReason {
                memory_id: None,
                reason_code: "query_by_example_candidate_expansion".to_string(),
                detail: influence_summary,
            });
        }
        let mut result = RetrievalResultSet::empty(explain, namespace.clone());
        result.outcome_class = membrain_core::observability::OutcomeClass::Degraded;
        result.policy_summary.outcome_class = membrain_core::observability::OutcomeClass::Degraded;
        result.packaging_metadata.result_budget = result_budget;
        result.packaging_metadata.degraded_summary = Some(degraded_summary.to_string());
        result.truncated = false;
        let payload = McpRetrievalPayload::from_result(request_id, namespace, false, result)
            .map_err(|err| err.to_string())?;
        Ok(McpResponse::retrieval_success(payload))
    }

    fn normalize_recall_contract(
        namespace: NamespaceId,
        query_text: Option<String>,
        like_id: Option<u64>,
        unlike_id: Option<u64>,
        graph_mode: Option<&str>,
        result_budget: Option<usize>,
    ) -> Result<NormalizedRecallContract, String> {
        let result_budget = result_budget.unwrap_or(10);
        let planner_request = if let Some(query) = query_text.as_deref() {
            if let Some(memory_id) = query.strip_prefix("memory:") {
                let memory_id = memory_id
                    .parse::<u64>()
                    .map_err(|_| format!("invalid memory query '{query}'"))?;
                RecallRequest::exact(MemoryId(memory_id))
            } else if let Some(session_id) = query.strip_prefix("session:") {
                let session_id = session_id
                    .parse::<u64>()
                    .map_err(|_| format!("invalid session query '{query}'"))?;
                RecallRequest::small_session_lookup(SessionId(session_id))
            } else {
                RecallRequest::default()
            }
        } else {
            RecallRequest::default()
        }
        .with_graph_expansion(matches!(graph_mode, Some("expand")));

        let graph_expansion = matches!(graph_mode, Some("expand"));
        let retrieval_request =
            RetrievalRequest::hybrid(namespace, query_text.unwrap_or_default(), result_budget)
                .with_budget(result_budget)
                .with_tier3_fallback(true)
                .with_graph_expansion(graph_expansion);
        let retrieval_request = if let Some(like_id) = like_id {
            retrieval_request.with_like_memory(MemoryId(like_id))
        } else {
            retrieval_request
        };
        let retrieval_request = if let Some(unlike_id) = unlike_id {
            retrieval_request.with_unlike_memory(MemoryId(unlike_id))
        } else {
            retrieval_request
        };
        let normalized_query_by_example = retrieval_request
            .normalize_query_by_example()
            .map_err(|err| err.as_str().to_string())?;

        Ok(NormalizedRecallContract {
            planner_request,
            normalized_query_by_example,
            result_budget,
        })
    }

    #[allow(clippy::too_many_arguments)]
    fn recall_degraded_summary(
        mode: &Option<String>,
        token_budget: Option<usize>,
        time_budget_ms: Option<u32>,
        context_text: &Option<String>,
        effort: &Option<String>,
        include_public: Option<bool>,
        graph_mode: &Option<String>,
        cold_tier: Option<bool>,
        workspace_id: Option<&str>,
        agent_id: Option<&str>,
        session_id: Option<&str>,
        task_id: Option<&str>,
        memory_kinds: Option<&[String]>,
        era_id: Option<&str>,
        as_of_tick: Option<u64>,
        at_snapshot: Option<&str>,
        min_strength: Option<u16>,
        min_confidence: Option<f64>,
        show_decaying: Option<bool>,
        mood_congruent: Option<bool>,
        normalized: &NormalizedRecallContract,
    ) -> String {
        let mut facets = Vec::new();
        if normalized
            .normalized_query_by_example
            .normalized_query_text
            .is_some()
        {
            facets.push("query_text");
        }
        if mode.is_some() {
            facets.push("mode");
        }
        if token_budget.is_some() {
            facets.push("token_budget");
        }
        if time_budget_ms.is_some() {
            facets.push("time_budget_ms");
        }
        if context_text.is_some() {
            facets.push("context_text");
        }
        if effort.is_some() {
            facets.push("effort");
        }
        if include_public.unwrap_or(false) {
            facets.push("include_public");
        }
        if graph_mode.is_some() {
            facets.push("graph_mode");
        }
        if cold_tier.is_some() {
            facets.push("cold_tier");
        }
        if workspace_id.is_some() {
            facets.push("workspace_id");
        }
        if agent_id.is_some() {
            facets.push("agent_id");
        }
        if session_id.is_some() {
            facets.push("session_id");
        }
        if task_id.is_some() {
            facets.push("task_id");
        }
        if memory_kinds.is_some() {
            facets.push("memory_kinds");
        }
        if era_id.is_some() {
            facets.push("era_id");
        }
        if normalized.normalized_query_by_example.has_example_seeds() {
            facets.push("query_by_example");
        }
        if as_of_tick.is_some() {
            facets.push("as_of_tick");
        }
        if at_snapshot.is_some() {
            facets.push("at_snapshot");
        }
        if min_strength.is_some() {
            facets.push("min_strength");
        }
        if min_confidence.is_some() {
            facets.push("min_confidence");
        }
        if show_decaying.is_some() {
            facets.push("show_decaying");
        }
        if mood_congruent.is_some() {
            facets.push("mood_congruent");
        }

        let mut summary = if facets.is_empty() {
            "planner-only recall envelope; evidence hydration not implemented".to_string()
        } else {
            format!(
                "planner-only recall envelope; evidence hydration not implemented; normalized params: {}",
                facets.join(", ")
            )
        };

        if normalized.normalized_query_by_example.has_example_seeds() {
            let seed_ids = normalized
                .normalized_query_by_example
                .seed_memory_ids()
                .into_iter()
                .map(|id| id.0.to_string())
                .collect::<Vec<_>>()
                .join(", ");
            let seed_polarities = normalized
                .normalized_query_by_example
                .seed_polarities()
                .join(", ");
            summary.push_str(&format!(
                "; primary_cue={}, seed_memory_ids=[{}], seed_polarities=[{}]",
                normalized.normalized_query_by_example.primary_cue.as_str(),
                seed_ids,
                seed_polarities
            ));
        }

        summary
    }

    fn canonical_result_budget(request: &RecallRequest, limit: Option<usize>) -> usize {
        if request.exact_memory_id.is_some() {
            return 1;
        }

        limit.unwrap_or(10)
    }

    async fn run_maintenance_budget(
        state: &RuntimeState,
        polls_budget: u32,
        step_delay: Duration,
    ) -> bool {
        for _ in 0..polls_budget {
            if state.is_shutdown() {
                return false;
            }
            sleep(step_delay).await;
        }
        !state.is_shutdown()
    }

    fn validate_preflight_namespace(namespace: &str) -> Result<(), String> {
        NamespaceId::new(namespace)
            .map(|_| ())
            .map_err(|err| err.to_string())
    }

    fn preflight_request(
        namespace: &str,
        original_query: &str,
        proposed_action: &str,
        local_confirmation: bool,
        confirmed_generation: Option<u64>,
    ) -> SafeguardRequest {
        let namespace_bound = NamespaceId::new(namespace).is_ok();
        let query = format!("{original_query} {proposed_action}").to_ascii_lowercase();
        let is_destructive = query.contains("delete") || query.contains("purge");
        let legal_hold = query.contains("legal hold");
        let snapshot_required = query.contains("snapshot") || query.contains("archive");
        let stale = query.contains("stale");
        let missing_scope = query.contains("ambiguous") || query.contains("all namespaces");
        let maintenance_window_required = query.contains("rewrite") || query.contains("reindex");
        let maintenance_window_active = !query.contains("window closed");
        let dependencies_ready = !query.contains("dependency pending");
        let confidence_ready = !query.contains("low confidence");
        let authoritative_input_readable = !query.contains("input unreadable");
        let can_degrade = query.contains("degraded") || query.contains("fallback");

        SafeguardRequest {
            operation_class: if legal_hold || query.contains("contradiction") {
                OperationClass::ContradictionArchive
            } else if is_destructive {
                OperationClass::IrreversibleMutation
            } else if maintenance_window_required {
                OperationClass::AuthoritativeRewrite
            } else {
                OperationClass::ReadOnlyAssessment
            },
            preview_only: false,
            namespace_bound,
            policy_allowed: namespace_bound && !legal_hold,
            requires_confirmation: is_destructive,
            local_confirmation,
            force_allowed: is_destructive,
            generation_bound: confirmed_generation,
            generation_matches: !stale,
            snapshot_required,
            snapshot_available: !query.contains("snapshot missing"),
            maintenance_window_required,
            maintenance_window_active,
            dependencies_ready,
            scope_precise: !missing_scope,
            authoritative_input_readable,
            confidence_ready,
            can_degrade,
            legal_hold,
        }
    }

    fn evaluate_preflight(
        namespace: &str,
        original_query: &str,
        proposed_action: &str,
        request_correlation_id: u64,
        local_confirmation: bool,
        preview_only: bool,
    ) -> Result<EvaluatedPreflight, String> {
        Self::validate_preflight_namespace(namespace)?;
        let _request = PreflightRunRequest {
            namespace: namespace.to_string(),
            original_query: original_query.to_string(),
            proposed_action: proposed_action.to_string(),
        };
        let request_id = if preview_only {
            format!("daemon-preflight-explain-{request_correlation_id}")
        } else if local_confirmation {
            format!("daemon-preflight-allow-{request_correlation_id}")
        } else {
            format!("daemon-preflight-run-{request_correlation_id}")
        };
        let preflight_id = format!("preflight-{request_correlation_id}");
        let mut request = Self::preflight_request(
            namespace,
            original_query,
            proposed_action,
            local_confirmation,
            Some(request_correlation_id),
        );
        request.preview_only = preview_only;
        let mut outcome = PolicyModule.evaluate_safeguard(request);
        outcome.audit.actor_source = "daemon_jsonrpc";
        outcome.audit.request_id = Box::leak(request_id.clone().into_boxed_str());
        outcome.audit.preview_id = Some(Box::leak(preflight_id.clone().into_boxed_str()));
        Ok(EvaluatedPreflight {
            request_id,
            preflight_id,
            execution_id: (!preview_only
                && matches!(
                    outcome.outcome_class,
                    OutcomeClass::Accepted | OutcomeClass::Degraded
                ))
            .then(|| format!("exec-{request_correlation_id}")),
            outcome,
        })
    }

    fn check_status_label(status: PreflightCheckStatus) -> String {
        match status {
            PreflightCheckStatus::Passed => "passed",
            PreflightCheckStatus::Blocked => "blocked",
            PreflightCheckStatus::Degraded => "degraded",
            PreflightCheckStatus::Rejected => "rejected",
        }
        .to_string()
    }

    fn reason_code_label(reason: SafeguardReasonCode) -> String {
        match reason {
            SafeguardReasonCode::ConfirmationRequired => "confirmation_required",
            SafeguardReasonCode::StalePreflight => "stale_preflight",
            SafeguardReasonCode::GenerationMismatch => "generation_mismatch",
            SafeguardReasonCode::SnapshotRequired => "snapshot_required",
            SafeguardReasonCode::MaintenanceWindowRequired => "maintenance_window_required",
            SafeguardReasonCode::DependencyPending => "dependency_pending",
            SafeguardReasonCode::ScopeAmbiguous => "scope_ambiguous",
            SafeguardReasonCode::AuthoritativeInputUnreadable => "authoritative_input_unreadable",
            SafeguardReasonCode::ConfidenceTooLow => "confidence_too_low",
            SafeguardReasonCode::PolicyDenied => "policy_denied",
            SafeguardReasonCode::LegalHold => "legal_hold",
        }
        .to_string()
    }

    fn check_view(check: &PreflightCheck) -> crate::preflight::PreflightCheckView {
        crate::preflight::PreflightCheckView {
            check_name: check.check_name.to_string(),
            status: Self::check_status_label(check.status),
            reason_codes: check
                .reason_codes
                .iter()
                .copied()
                .map(Self::reason_code_label)
                .collect(),
            checked_scope: check.checked_scope.to_string(),
        }
    }

    fn preflight_state_label(state: PreflightState) -> &'static str {
        match state {
            PreflightState::Ready => "ready",
            PreflightState::Blocked => "blocked",
            PreflightState::MissingData => "missing_data",
            PreflightState::StaleKnowledge => "stale_knowledge",
        }
    }

    fn policy_decision_label(decision: PolicyDecision) -> &'static str {
        match decision {
            PolicyDecision::Allow => "allow",
            PolicyDecision::Deny => "deny",
        }
    }

    fn preflight_outcome_label(
        outcome: &SafeguardOutcome,
        local_confirmation: bool,
    ) -> &'static str {
        if local_confirmation
            && matches!(
                outcome.outcome_class,
                OutcomeClass::Accepted | OutcomeClass::Degraded
            )
        {
            "force_confirmed"
        } else {
            match outcome.outcome_class {
                OutcomeClass::Preview => "preview_only",
                OutcomeClass::Blocked => "blocked",
                OutcomeClass::Degraded => "degraded",
                OutcomeClass::Accepted => "ready",
                OutcomeClass::Rejected => "blocked",
                OutcomeClass::Partial => "degraded",
            }
        }
    }

    fn confirmation_reason(outcome: &SafeguardOutcome, local_confirmation: bool) -> Option<String> {
        (local_confirmation
            && matches!(
                outcome.outcome_class,
                OutcomeClass::Accepted | OutcomeClass::Degraded
            ))
        .then_some("operator confirmed exact previewed scope".to_string())
    }

    fn blocked_reason_message(reasons: &[String]) -> Option<String> {
        reasons.first().map(|reason| {
            match reason.as_str() {
                "confirmation_required" => "destructive action requires explicit confirmation",
                "policy_denied" => "policy denied the requested action",
                "legal_hold" => "legal hold blocks the requested action",
                "snapshot_required" => "snapshot is required before this action can proceed",
                "stale_preflight" | "generation_mismatch" => {
                    "preflight confirmation is stale for the requested scope"
                }
                "scope_ambiguous" => "requested scope is ambiguous",
                "maintenance_window_required" => "maintenance window is required",
                "dependency_pending" => "operation dependencies are not ready",
                "confidence_too_low" => "confidence is too low for this action",
                "authoritative_input_unreadable" => "authoritative inputs are unreadable",
                _ => "preflight blocked the requested action",
            }
            .to_string()
        })
    }

    fn required_overrides(outcome: &SafeguardOutcome) -> Vec<String> {
        let mut overrides = Vec::new();
        if outcome
            .blocked_reasons
            .contains(&SafeguardReasonCode::ConfirmationRequired)
        {
            overrides.push("human_confirmation".to_string());
        }
        overrides
    }

    fn policy_context(namespace: &str, outcome: &SafeguardOutcome) -> String {
        let action = match outcome.operation_class {
            OperationClass::ReadOnlyAssessment => "read-only assessment",
            OperationClass::DerivedSurfaceMutation => "derived surface mutation",
            OperationClass::AuthoritativeRewrite => "authoritative rewrite",
            OperationClass::IrreversibleMutation => "irreversible mutation",
            OperationClass::ContradictionArchive => "contradiction archive",
        };
        format!(
            "namespace {namespace} preflight safeguard evaluation ({action}; {})",
            outcome.impact_summary
        )
    }

    fn to_preflight_outcome(
        evaluated: EvaluatedPreflight,
        local_confirmation: bool,
    ) -> PreflightOutcome {
        let outcome = evaluated.outcome;
        let blocked_reasons = outcome
            .blocked_reasons
            .iter()
            .copied()
            .map(Self::reason_code_label)
            .collect::<Vec<_>>();
        let outcome_class = outcome.outcome_class.as_str().to_string();
        let degraded = matches!(outcome.outcome_class, OutcomeClass::Degraded);
        let success = matches!(
            outcome.outcome_class,
            OutcomeClass::Accepted | OutcomeClass::Degraded
        );
        PreflightOutcome {
            success,
            preflight_state: Self::preflight_state_label(outcome.preflight_state).to_string(),
            preflight_outcome: Self::preflight_outcome_label(&outcome, local_confirmation)
                .to_string(),
            outcome_class,
            blocked_reasons,
            check_results: outcome.check_results.iter().map(Self::check_view).collect(),
            confirmation: crate::preflight::ConfirmationView {
                required: outcome.confirmation.required,
                force_allowed: outcome.confirmation.force_allowed,
                confirmed: outcome.confirmation.confirmed,
                generation_bound: outcome.confirmation.generation_bound,
            },
            audit: crate::preflight::AuditView {
                event_kind: outcome.audit.event_kind.to_string(),
                actor_source: outcome.audit.actor_source.to_string(),
                request_id: outcome.audit.request_id.to_string(),
                preview_id: outcome.audit.preview_id.map(str::to_string),
                related_run: outcome.audit.related_run.map(str::to_string),
                scope_handle: outcome.audit.scope_handle.to_string(),
            },
            policy_summary: crate::preflight::PolicySummaryView {
                decision: Self::policy_decision_label(outcome.policy_summary.decision).to_string(),
                namespace_bound: outcome.policy_summary.namespace_bound,
                outcome_class: outcome.policy_summary.outcome_class.as_str().to_string(),
            },
            request_id: Some(evaluated.request_id),
            preflight_id: Some(evaluated.preflight_id),
            execution_id: evaluated.execution_id,
            degraded,
            confirmation_reason: Self::confirmation_reason(&outcome, local_confirmation),
        }
    }

    fn to_preflight_explain_response(
        namespace: &str,
        evaluated: EvaluatedPreflight,
    ) -> PreflightExplainResponse {
        let outcome = evaluated.outcome;
        let mut blocked_reasons = outcome
            .blocked_reasons
            .iter()
            .copied()
            .map(Self::reason_code_label)
            .collect::<Vec<_>>();
        if outcome.confirmation.required && !outcome.confirmation.confirmed {
            blocked_reasons.push("confirmation_required".to_string());
        }
        let allowed = matches!(
            outcome.outcome_class,
            OutcomeClass::Accepted | OutcomeClass::Degraded
        );
        let preflight_state = if blocked_reasons.is_empty() {
            Self::preflight_state_label(outcome.preflight_state).to_string()
        } else {
            "blocked".to_string()
        };
        PreflightExplainResponse {
            allowed,
            preflight_state,
            preflight_outcome: Self::preflight_outcome_label(&outcome, false).to_string(),
            blocked_reasons: blocked_reasons.clone(),
            blocked_reason: Self::blocked_reason_message(&blocked_reasons),
            required_overrides: if outcome.confirmation.required && !outcome.confirmation.confirmed
            {
                vec!["human_confirmation".to_string()]
            } else {
                Self::required_overrides(&outcome)
            },
            policy_context: Self::policy_context(namespace, &outcome),
            check_results: outcome.check_results.iter().map(Self::check_view).collect(),
            confirmation: crate::preflight::ConfirmationView {
                required: outcome.confirmation.required,
                force_allowed: outcome.confirmation.force_allowed,
                confirmed: outcome.confirmation.confirmed,
                generation_bound: outcome.confirmation.generation_bound,
            },
            audit: crate::preflight::AuditView {
                event_kind: outcome.audit.event_kind.to_string(),
                actor_source: outcome.audit.actor_source.to_string(),
                request_id: outcome.audit.request_id.to_string(),
                preview_id: outcome.audit.preview_id.map(str::to_string),
                related_run: outcome.audit.related_run.map(str::to_string),
                scope_handle: outcome.audit.scope_handle.to_string(),
            },
            policy_summary: crate::preflight::PolicySummaryView {
                decision: Self::policy_decision_label(outcome.policy_summary.decision).to_string(),
                namespace_bound: outcome.policy_summary.namespace_bound,
                outcome_class: outcome.policy_summary.outcome_class.as_str().to_string(),
            },
            request_id: Some(evaluated.request_id),
            preflight_id: Some(evaluated.preflight_id),
        }
    }

    fn handle_preflight_run(
        namespace: &str,
        original_query: &str,
        proposed_action: &str,
        request_correlation_id: u64,
    ) -> Result<PreflightOutcome, String> {
        let evaluated = Self::evaluate_preflight(
            namespace,
            original_query,
            proposed_action,
            request_correlation_id,
            false,
            false,
        )?;
        Ok(Self::to_preflight_outcome(evaluated, false))
    }

    fn handle_preflight_explain(
        namespace: &str,
        original_query: &str,
        proposed_action: &str,
        request_correlation_id: u64,
    ) -> Result<PreflightExplainResponse, String> {
        let evaluated = Self::evaluate_preflight(
            namespace,
            original_query,
            proposed_action,
            request_correlation_id,
            false,
            true,
        )?;
        Ok(Self::to_preflight_explain_response(namespace, evaluated))
    }

    fn handle_preflight_allow(
        namespace: &str,
        authorization_token: &str,
        bypass_flags: &[String],
        request_correlation_id: u64,
    ) -> Result<PreflightOutcome, String> {
        Self::validate_preflight_namespace(namespace)?;
        let _request = PreflightAllowRequest {
            namespace: namespace.to_string(),
            authorization_token: authorization_token.to_string(),
            bypass_flags: bypass_flags.to_vec(),
        };
        let confirmed = authorization_token.starts_with("allow-")
            && bypass_flags.iter().any(|flag| flag == "manual_override");
        let action = "allow exact previewed scope delete";
        let evaluated = Self::evaluate_preflight(
            namespace,
            "confirm previously previewed action",
            action,
            request_correlation_id,
            confirmed,
            false,
        )?;
        Ok(Self::to_preflight_outcome(evaluated, confirmed))
    }

    async fn maintenance_loop(state: Arc<RuntimeState>, config: DaemonRuntimeConfig) {
        let mut ticker = tokio::time::interval(config.maintenance_interval);
        ticker.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);
        ticker.tick().await;

        loop {
            if state.is_shutdown() {
                break;
            }

            tokio::select! {
                _ = state.shutdown_notify.notified() => {
                    break;
                }
                _ = ticker.tick() => {
                    if state.is_shutdown() {
                        break;
                    }
                    if state.background_jobs.load(Ordering::SeqCst) > 0 {
                        continue;
                    }
                    let maintenance_id = state.next_maintenance_id();
                    let state_clone = Arc::clone(&state);
                    let step_delay = config.maintenance_step_delay;
                    let polls_budget = config.maintenance_poll_budget;
                    tokio::spawn(async move {
                        let _guard = BackgroundJobGuard::new(Arc::clone(&state_clone));
                        if Self::run_maintenance_budget(&state_clone, polls_budget, step_delay).await {
                            state_clone.record_maintenance_run(maintenance_id).await;
                        }
                    });
                }
            }
        }
    }

    async fn write_response(
        mut stream: UnixStream,
        response: &JsonRpcResponse,
    ) -> anyhow::Result<()> {
        let bytes = serde_json::to_vec(response).context("serialize JSON-RPC response")?;
        stream.write_all(&bytes).await?;
        stream.write_all(b"\n").await?;
        Ok(())
    }

    async fn remove_stale_socket(&self) -> anyhow::Result<()> {
        match tokio::fs::symlink_metadata(&self.config.socket_path).await {
            Ok(metadata) if metadata.file_type().is_socket() => {
                match UnixStream::connect(&self.config.socket_path).await {
                    Ok(stream) => {
                        drop(stream);
                        anyhow::bail!(
                            "refusing to remove live daemon socket before binding: {}",
                            self.config.socket_path.display()
                        );
                    }
                    Err(err)
                        if matches!(
                            err.kind(),
                            std::io::ErrorKind::ConnectionRefused | std::io::ErrorKind::NotFound
                        ) =>
                    {
                        tokio::fs::remove_file(&self.config.socket_path).await?;
                        Ok(())
                    }
                    Err(err) => anyhow::bail!(
                        "refusing to remove existing socket path before binding daemon {}: {err}",
                        self.config.socket_path.display()
                    ),
                }
            }
            Ok(_) => anyhow::bail!(
                "refusing to remove non-socket path before binding daemon: {}",
                self.config.socket_path.display()
            ),
            Err(err) if err.kind() == std::io::ErrorKind::NotFound => Ok(()),
            Err(err) => Err(err.into()),
        }
    }
}

struct QueueGuard {
    state: Arc<RuntimeState>,
}

impl QueueGuard {
    fn new(state: Arc<RuntimeState>) -> Self {
        Self { state }
    }
}

impl Drop for QueueGuard {
    fn drop(&mut self) {
        self.state.queued_requests.fetch_sub(1, Ordering::SeqCst);
    }
}

struct ActiveRequestGuard {
    state: Arc<RuntimeState>,
}

impl ActiveRequestGuard {
    fn new(state: Arc<RuntimeState>) -> Self {
        state.active_requests.fetch_add(1, Ordering::SeqCst);
        Self { state }
    }
}

impl Drop for ActiveRequestGuard {
    fn drop(&mut self) {
        self.state.active_requests.fetch_sub(1, Ordering::SeqCst);
    }
}

struct BackgroundJobGuard {
    state: Arc<RuntimeState>,
}

impl BackgroundJobGuard {
    fn new(state: Arc<RuntimeState>) -> Self {
        state.background_jobs.fetch_add(1, Ordering::SeqCst);
        Self { state }
    }
}

impl Drop for BackgroundJobGuard {
    fn drop(&mut self) {
        self.state.background_jobs.fetch_sub(1, Ordering::SeqCst);
    }
}

#[cfg(test)]
mod tests {
    use super::{DaemonRuntime, DaemonRuntimeConfig};
    use crate::rpc::RuntimeStatus;
    use membrain_core::api::NamespaceId;
    use membrain_core::engine::recall::RecallRequest;
    use membrain_core::types::MemoryId;
    use serde_json::{json, Value};
    use std::path::PathBuf;
    use std::sync::Arc;
    use std::time::{SystemTime, UNIX_EPOCH};
    use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
    use tokio::net::UnixStream;
    use tokio::time::{timeout, Duration};

    fn unique_path(name: &str) -> PathBuf {
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        std::env::temp_dir().join(format!("membrain-daemon-{name}-{nanos}.sock"))
    }

    async fn send_request(socket_path: &std::path::Path, request: Value) -> Value {
        let mut stream = UnixStream::connect(socket_path).await.unwrap();
        let payload = serde_json::to_vec(&request).unwrap();
        stream.write_all(&payload).await.unwrap();
        stream.write_all(b"\n").await.unwrap();

        let mut line = String::new();
        let mut reader = BufReader::new(stream);
        reader.read_line(&mut line).await.unwrap();
        serde_json::from_str(&line).unwrap()
    }

    async fn send_notification(socket_path: &std::path::Path, request: Value) {
        let mut stream = UnixStream::connect(socket_path).await.unwrap();
        let payload = serde_json::to_vec(&request).unwrap();
        stream.write_all(&payload).await.unwrap();
        stream.write_all(b"\n").await.unwrap();

        let mut line = String::new();
        let mut reader = BufReader::new(stream);
        let read_result = timeout(Duration::from_millis(150), reader.read_line(&mut line)).await;
        match read_result {
            Err(_) | Ok(Ok(0)) => {}
            Ok(Ok(_)) => assert!(
                line.is_empty(),
                "notification unexpectedly received a response: {line}"
            ),
            Ok(Err(err)) => {
                assert!(
                    std::io::Result::<()>::Ok(()).is_err(),
                    "failed to read notification response state: {err}"
                );
            }
        }
    }

    async fn spawn_runtime(
        config: DaemonRuntimeConfig,
    ) -> tokio::task::JoinHandle<anyhow::Result<()>> {
        let runtime = DaemonRuntime::with_config(config);
        tokio::spawn(async move { runtime.run_until_stopped().await })
    }

    #[tokio::test]
    async fn remove_stale_socket_ignores_missing_path() {
        let runtime = DaemonRuntime::new(unique_path("missing"));
        runtime.remove_stale_socket().await.unwrap();
    }

    #[tokio::test]
    async fn remove_stale_socket_rejects_non_socket_path() {
        let path = unique_path("regular-file");
        tokio::fs::write(&path, b"not-a-socket").await.unwrap();

        let runtime = DaemonRuntime::new(&path);
        let error = runtime.remove_stale_socket().await.unwrap_err();
        assert!(error
            .to_string()
            .contains("refusing to remove non-socket path before binding daemon"));

        assert!(tokio::fs::symlink_metadata(&path).await.is_ok());
        tokio::fs::remove_file(path).await.unwrap();
    }

    #[tokio::test]
    async fn remove_stale_socket_rejects_live_daemon_socket() {
        let socket_path = unique_path("live-socket");
        let mut config = DaemonRuntimeConfig::new(&socket_path);
        config.maintenance_interval = Duration::from_secs(3600);
        let handle = spawn_runtime(config).await;

        timeout(Duration::from_secs(2), async {
            while tokio::fs::metadata(&socket_path).await.is_err() {
                tokio::task::yield_now().await;
            }
        })
        .await
        .unwrap();

        let runtime = DaemonRuntime::new(&socket_path);
        let error = runtime.remove_stale_socket().await.unwrap_err();
        assert!(error
            .to_string()
            .contains("refusing to remove live daemon socket before binding"));
        assert!(tokio::fs::symlink_metadata(&socket_path).await.is_ok());

        let shutdown_response = send_request(
            &socket_path,
            json!({"jsonrpc":"2.0","method":"shutdown","params":{},"id":"cleanup"}),
        )
        .await;
        assert_eq!(shutdown_response["result"]["shutting_down"], json!(true));

        timeout(Duration::from_secs(2), handle)
            .await
            .unwrap()
            .unwrap()
            .unwrap();
    }

    #[tokio::test]
    async fn runtime_serves_status_and_shutdown_over_unix_socket() {
        let socket_path = unique_path("status-shutdown");
        let mut config = DaemonRuntimeConfig::new(&socket_path);
        config.maintenance_interval = Duration::from_secs(3600);
        let handle = spawn_runtime(config).await;

        timeout(Duration::from_secs(2), async {
            while tokio::fs::metadata(&socket_path).await.is_err() {
                tokio::task::yield_now().await;
            }
        })
        .await
        .unwrap();

        let status_response = send_request(
            &socket_path,
            json!({"jsonrpc":"2.0","method":"status","params":{},"id":1}),
        )
        .await;
        let status: RuntimeStatus =
            serde_json::from_value(status_response.get("result").cloned().unwrap()).unwrap();
        assert_eq!(status.posture.as_str(), "full");
        assert_eq!(status.metrics.queue_depth, 0);

        let shutdown_response = send_request(
            &socket_path,
            json!({"jsonrpc":"2.0","method":"shutdown","params":{},"id":2}),
        )
        .await;
        assert_eq!(shutdown_response["result"]["shutting_down"], json!(true));

        timeout(Duration::from_secs(2), handle)
            .await
            .unwrap()
            .unwrap()
            .unwrap();
        assert!(tokio::fs::metadata(&socket_path).await.is_err());
    }

    #[tokio::test]
    async fn runtime_serves_doctor_report_over_unix_socket() {
        let socket_path = unique_path("doctor-report");
        let mut config = DaemonRuntimeConfig::new(&socket_path);
        config.maintenance_interval = Duration::from_secs(3600);
        let handle = spawn_runtime(config).await;

        timeout(Duration::from_secs(2), async {
            while tokio::fs::metadata(&socket_path).await.is_err() {
                tokio::task::yield_now().await;
            }
        })
        .await
        .unwrap();

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
        assert_eq!(doctor_response["result"]["status"], json!("warn"));
        assert_eq!(doctor_response["result"]["action"], json!("doctor"));
        assert_eq!(doctor_response["result"]["posture"], json!("degraded"));
        assert_eq!(
            doctor_response["result"]["degraded_reasons"],
            json!(["repair_in_flight"])
        );
        assert_eq!(
            doctor_response["result"]["indexes"]
                .as_array()
                .unwrap()
                .len(),
            4
        );
        assert_eq!(
            doctor_response["result"]["indexes"][0]["family"],
            json!("schema")
        );
        assert_eq!(
            doctor_response["result"]["indexes"][1]["family"],
            json!("index")
        );
        assert_eq!(
            doctor_response["result"]["indexes"][2]["family"],
            json!("graph")
        );
        assert_eq!(
            doctor_response["result"]["indexes"][3]["family"],
            json!("cache")
        );
        assert_eq!(
            doctor_response["result"]["indexes"][3]["health"],
            json!("warn")
        );
        assert_eq!(
            doctor_response["result"]["warnings"],
            json!(["operator_review_recommended"])
        );

        let shutdown_response = send_request(
            &socket_path,
            json!({"jsonrpc":"2.0","method":"shutdown","params":{},"id":"done"}),
        )
        .await;
        assert_eq!(shutdown_response["result"]["shutting_down"], json!(true));

        timeout(Duration::from_secs(2), handle)
            .await
            .unwrap()
            .unwrap()
            .unwrap();
    }

    #[tokio::test]
    async fn runtime_share_and_unshare_surface_policy_and_audit_fields() {
        let socket_path = unique_path("share-unshare");
        let mut config = DaemonRuntimeConfig::new(&socket_path);
        config.maintenance_interval = Duration::from_secs(3600);
        let handle = spawn_runtime(config).await;

        timeout(Duration::from_secs(2), async {
            while tokio::fs::metadata(&socket_path).await.is_err() {
                tokio::task::yield_now().await;
            }
        })
        .await
        .unwrap();

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
            share_response["result"]["policy_summary"]["policy_family"],
            json!("visibility_sharing")
        );
        assert_eq!(
            share_response["result"]["audit"]["request_id"],
            json!("req-share-42")
        );

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
            unshare_response["result"]["policy_summary"]["redaction_fields"],
            json!(["sharing_scope"])
        );
        assert_eq!(
            unshare_response["result"]["audit"]["request_id"],
            json!("req-unshare-42")
        );
        assert_eq!(unshare_response["result"]["audit"]["redacted"], json!(true));

        let shutdown_response = send_request(
            &socket_path,
            json!({"jsonrpc":"2.0","method":"shutdown","params":{},"id":"done"}),
        )
        .await;
        assert_eq!(shutdown_response["result"]["shutting_down"], json!(true));

        timeout(Duration::from_secs(2), handle)
            .await
            .unwrap()
            .unwrap()
            .unwrap();
    }

    #[tokio::test]
    async fn runtime_share_rejects_malformed_namespace_id() {
        let socket_path = unique_path("share-invalid-namespace");
        let mut config = DaemonRuntimeConfig::new(&socket_path);
        config.maintenance_interval = Duration::from_secs(3600);
        let handle = spawn_runtime(config).await;

        timeout(Duration::from_secs(2), async {
            while tokio::fs::metadata(&socket_path).await.is_err() {
                tokio::task::yield_now().await;
            }
        })
        .await
        .unwrap();

        let response = send_request(
            &socket_path,
            json!({
                "jsonrpc":"2.0",
                "method":"share",
                "params":{"id":42,"namespace_id":"bad namespace"},
                "id":"share-invalid"
            }),
        )
        .await;
        assert_eq!(response["error"]["code"], json!(-32602));
        assert_eq!(
            response["error"]["message"],
            json!("malformed namespace_id")
        );
        assert_eq!(
            response["error"]["data"]["error_kind"],
            json!("validation_failure")
        );

        let shutdown_response = send_request(
            &socket_path,
            json!({"jsonrpc":"2.0","method":"shutdown","params":{},"id":"done"}),
        )
        .await;
        assert_eq!(shutdown_response["result"]["shutting_down"], json!(true));

        timeout(Duration::from_secs(2), handle)
            .await
            .unwrap()
            .unwrap()
            .unwrap();
    }

    #[tokio::test]
    async fn runtime_handles_concurrent_readers_while_background_job_runs() {
        let socket_path = unique_path("concurrency");
        let mut config = DaemonRuntimeConfig::new(&socket_path);
        config.request_concurrency = 2;
        config.max_queue_depth = 8;
        config.maintenance_interval = Duration::from_secs(3600);
        let handle = spawn_runtime(config).await;

        timeout(Duration::from_secs(2), async {
            while tokio::fs::metadata(&socket_path).await.is_err() {
                tokio::task::yield_now().await;
            }
        })
        .await
        .unwrap();

        let maintenance = send_request(
            &socket_path,
            json!({
                "jsonrpc":"2.0",
                "method":"run_maintenance",
                "params":{"polls_budget":4,"step_delay_ms":50},
                "id":"maintenance"
            }),
        )
        .await;
        assert!(maintenance.get("result").is_some());

        let (first, second) = tokio::join!(
            send_request(
                &socket_path,
                json!({"jsonrpc":"2.0","method":"sleep","params":{"millis":100},"id":"a"}),
            ),
            send_request(
                &socket_path,
                json!({"jsonrpc":"2.0","method":"status","params":{},"id":"b"}),
            )
        );

        assert_eq!(first["result"]["slept_ms"], json!(100));
        assert_eq!(second["result"]["metrics"]["background_jobs"], json!(1));

        let shutdown_response = send_request(
            &socket_path,
            json!({"jsonrpc":"2.0","method":"shutdown","params":{},"id":"done"}),
        )
        .await;
        assert_eq!(shutdown_response["result"]["shutting_down"], json!(true));
        timeout(Duration::from_secs(2), handle)
            .await
            .unwrap()
            .unwrap()
            .unwrap();
    }

    #[tokio::test]
    async fn runtime_enforces_bounded_queue_depth() {
        let socket_path = unique_path("queue-depth");
        let mut config = DaemonRuntimeConfig::new(&socket_path);
        config.request_concurrency = 1;
        config.max_queue_depth = 1;
        config.maintenance_interval = Duration::from_secs(3600);
        let handle = spawn_runtime(config).await;

        timeout(Duration::from_secs(2), async {
            while tokio::fs::metadata(&socket_path).await.is_err() {
                tokio::task::yield_now().await;
            }
        })
        .await
        .unwrap();

        let socket_path_clone = socket_path.clone();
        let slow = tokio::spawn(async move {
            send_request(
                &socket_path_clone,
                json!({"jsonrpc":"2.0","method":"sleep","params":{"millis":300},"id":"slow"}),
            )
            .await
        });

        tokio::time::sleep(Duration::from_millis(25)).await;

        let socket_path_clone = socket_path.clone();
        let queued = tokio::spawn(async move {
            send_request(
                &socket_path_clone,
                json!({"jsonrpc":"2.0","method":"sleep","params":{"millis":300},"id":"queued"}),
            )
            .await
        });

        tokio::time::sleep(Duration::from_millis(25)).await;

        let busy = send_request(
            &socket_path,
            json!({"jsonrpc":"2.0","method":"sleep","params":{"millis":1},"id":"busy"}),
        )
        .await;
        assert_eq!(busy["error"]["code"], json!(-32001));
        assert_eq!(busy["error"]["data"]["max_queue_depth"], json!(1));
        assert_eq!(busy["error"]["data"]["queue_depth"], json!(1));

        let slow_response = slow.await.unwrap();
        let queued_response = queued.await.unwrap();
        assert_eq!(slow_response["result"]["slept_ms"], json!(300));
        assert_eq!(queued_response["result"]["slept_ms"], json!(300));

        let shutdown_response = send_request(
            &socket_path,
            json!({"jsonrpc":"2.0","method":"shutdown","params":{},"id":"done"}),
        )
        .await;
        assert_eq!(shutdown_response["result"]["shutting_down"], json!(true));
        timeout(Duration::from_secs(2), handle)
            .await
            .unwrap()
            .unwrap()
            .unwrap();
    }

    #[tokio::test]
    async fn runtime_recall_returns_typed_mcp_retrieval_payload() {
        let socket_path = unique_path("recall");
        let mut config = DaemonRuntimeConfig::new(&socket_path);
        config.maintenance_interval = Duration::from_secs(3600);
        let handle = spawn_runtime(config).await;

        timeout(Duration::from_secs(2), async {
            while tokio::fs::metadata(&socket_path).await.is_err() {
                tokio::task::yield_now().await;
            }
        })
        .await
        .unwrap();

        let recall_response = send_request(
            &socket_path,
            json!({
                "jsonrpc":"2.0",
                "method":"recall",
                "params":{"query_text":"memory:42","namespace":"team.alpha","result_budget":3},
                "id":"recall"
            }),
        )
        .await;

        assert_eq!(recall_response["result"]["status"], json!("ok"));
        assert!(recall_response["result"].get("retrieval").is_some());
        assert!(recall_response["result"].get("payload").is_none());
        assert_eq!(
            recall_response["result"]["retrieval"]["namespace"],
            json!("team.alpha")
        );
        assert_eq!(
            recall_response["result"]["retrieval"]["result"]["explain"]["recall_plan"],
            json!("ExactIdTier1")
        );
        assert_eq!(
            recall_response["result"]["retrieval"]["result"]["packaging_metadata"]["result_budget"],
            json!(1)
        );
        assert!(recall_response["result"]["retrieval"]
            .get("explain_trace")
            .is_some());

        let shutdown_response = send_request(
            &socket_path,
            json!({"jsonrpc":"2.0","method":"shutdown","params":{},"id":"done"}),
        )
        .await;
        assert_eq!(shutdown_response["result"]["shutting_down"], json!(true));
        timeout(Duration::from_secs(5), handle)
            .await
            .unwrap()
            .unwrap()
            .unwrap();
    }

    #[tokio::test]
    async fn runtime_recall_keeps_canonical_payload_families_out_of_generic_payload_slot() {
        let socket_path = unique_path("recall-canonical-payload-families");
        let mut config = DaemonRuntimeConfig::new(&socket_path);
        config.maintenance_interval = Duration::from_secs(3600);
        let handle = spawn_runtime(config).await;

        timeout(Duration::from_secs(2), async {
            while tokio::fs::metadata(&socket_path).await.is_err() {
                tokio::task::yield_now().await;
            }
        })
        .await
        .unwrap();

        let recall_response = send_request(
            &socket_path,
            json!({
                "jsonrpc":"2.0",
                "method":"recall",
                "params":{"query_text":"memory:42","namespace":"team.alpha","result_budget":3},
                "id":"recall-canonical-payload-families"
            }),
        )
        .await;

        let result = &recall_response["result"];
        assert_eq!(result["status"], json!("ok"));
        assert!(result.get("payload").is_none());
        assert!(result.get("error").is_none());

        let retrieval = &result["retrieval"];
        assert_eq!(retrieval["request_id"], json!("daemon-recall-1"));
        assert_eq!(retrieval["outcome_class"], json!("degraded"));
        assert_eq!(retrieval["partial_success"], json!(false));
        assert!(retrieval["result"].get("evidence_pack").is_some());
        assert!(retrieval["result"].get("action_pack").is_some());
        assert!(retrieval["result"].get("deferred_payloads").is_some());
        assert!(retrieval["result"].get("omitted_summary").is_some());
        assert!(retrieval["result"].get("policy_summary").is_some());
        assert!(retrieval["result"].get("provenance_summary").is_some());
        assert!(retrieval["result"].get("freshness_markers").is_some());
        assert!(retrieval["result"].get("packaging_metadata").is_some());
        assert!(retrieval["result"].get("explain").is_some());
        assert!(retrieval.get("explain_trace").is_some());
        assert!(retrieval["explain_trace"].get("route_summary").is_some());
        assert!(retrieval["explain_trace"].get("omitted_summary").is_some());
        assert!(retrieval["explain_trace"].get("policy_summary").is_some());
        assert!(retrieval["explain_trace"]
            .get("provenance_summary")
            .is_some());
        assert!(retrieval["explain_trace"]
            .get("freshness_markers")
            .is_some());
        assert!(retrieval["explain_trace"].get("conflict_markers").is_some());
        assert!(retrieval["explain_trace"]
            .get("uncertainty_markers")
            .is_some());
        assert_eq!(
            retrieval["result"]["packaging_metadata"]["degraded_summary"],
            json!("planner-only recall envelope; evidence hydration not implemented; normalized params: query_text")
        );
        assert!(retrieval["result"]["explain"]["query_by_example"].is_null());

        let shutdown_response = send_request(
            &socket_path,
            json!({"jsonrpc":"2.0","method":"shutdown","params":{},"id":"done"}),
        )
        .await;
        assert_eq!(shutdown_response["result"]["shutting_down"], json!(true));
        timeout(Duration::from_secs(5), handle)
            .await
            .unwrap()
            .unwrap()
            .unwrap();
    }

    #[tokio::test]
    async fn runtime_recall_rejects_invalid_memory_query() {
        let socket_path = unique_path("recall-invalid");
        let mut config = DaemonRuntimeConfig::new(&socket_path);
        config.maintenance_interval = Duration::from_secs(3600);
        let handle = spawn_runtime(config).await;

        timeout(Duration::from_secs(2), async {
            while tokio::fs::metadata(&socket_path).await.is_err() {
                tokio::task::yield_now().await;
            }
        })
        .await
        .unwrap();

        let recall_response = send_request(
            &socket_path,
            json!({
                "jsonrpc":"2.0",
                "method":"recall",
                "params":{"query_text":"memory:not-a-number","namespace":"team.alpha"},
                "id":"recall-invalid"
            }),
        )
        .await;

        assert_eq!(recall_response["error"]["code"], json!(-32602));
        assert_eq!(
            recall_response["error"]["message"],
            json!("invalid memory query 'memory:not-a-number'")
        );

        let shutdown_response = send_request(
            &socket_path,
            json!({"jsonrpc":"2.0","method":"shutdown","params":{},"id":"done"}),
        )
        .await;
        assert_eq!(shutdown_response["result"]["shutting_down"], json!(true));
        timeout(Duration::from_secs(5), handle)
            .await
            .unwrap()
            .unwrap()
            .unwrap();
    }

    #[tokio::test]
    async fn runtime_recall_rejects_non_positive_limit() {
        let socket_path = unique_path("recall-zero-limit");
        let mut config = DaemonRuntimeConfig::new(&socket_path);
        config.maintenance_interval = Duration::from_secs(3600);
        let handle = spawn_runtime(config).await;

        timeout(Duration::from_secs(2), async {
            while tokio::fs::metadata(&socket_path).await.is_err() {
                tokio::task::yield_now().await;
            }
        })
        .await
        .unwrap();

        let recall_response = send_request(
            &socket_path,
            json!({
                "jsonrpc":"2.0",
                "method":"recall",
                "params":{"query":"session:7","namespace":"team.alpha","limit":0},
                "id":"recall-zero-limit"
            }),
        )
        .await;

        assert_eq!(recall_response["error"]["code"], json!(-32602));
        assert_eq!(
            recall_response["error"]["message"],
            json!("limit must be at least 1")
        );

        let shutdown_response = send_request(
            &socket_path,
            json!({"jsonrpc":"2.0","method":"shutdown","params":{},"id":"done"}),
        )
        .await;
        assert_eq!(shutdown_response["result"]["shutting_down"], json!(true));
        timeout(Duration::from_secs(5), handle)
            .await
            .unwrap()
            .unwrap()
            .unwrap();
    }

    #[tokio::test]
    async fn runtime_recall_preserves_requested_budget_for_non_exact_queries() {
        let socket_path = unique_path("recall-session-budget");
        let mut config = DaemonRuntimeConfig::new(&socket_path);
        config.maintenance_interval = Duration::from_secs(3600);
        let handle = spawn_runtime(config).await;

        timeout(Duration::from_secs(2), async {
            while tokio::fs::metadata(&socket_path).await.is_err() {
                tokio::task::yield_now().await;
            }
        })
        .await
        .unwrap();

        let recall_response = send_request(
            &socket_path,
            json!({
                "jsonrpc":"2.0",
                "method":"recall",
                "params":{"query_text":"session:7","namespace":"team.alpha","result_budget":3},
                "id":"recall-session-budget"
            }),
        )
        .await;

        assert_eq!(recall_response["result"]["status"], json!("ok"));
        assert_eq!(
            recall_response["result"]["retrieval"]["outcome_class"],
            json!("degraded")
        );
        assert_eq!(
            recall_response["result"]["retrieval"]["result"]["outcome_class"],
            json!("degraded")
        );
        assert_eq!(
            recall_response["result"]["retrieval"]["result"]["packaging_metadata"]["result_budget"],
            json!(3)
        );
        assert_eq!(
            recall_response["result"]["retrieval"]["result"]["packaging_metadata"]
                ["degraded_summary"],
            json!("planner-only recall envelope; evidence hydration not implemented; normalized params: query_text")
        );
        assert_eq!(
            recall_response["result"]["retrieval"]["result"]["explain"]["recall_plan"],
            json!("RecentTier1ThenTier2Exact")
        );

        let shutdown_response = send_request(
            &socket_path,
            json!({"jsonrpc":"2.0","method":"shutdown","params":{},"id":"done"}),
        )
        .await;
        assert_eq!(shutdown_response["result"]["shutting_down"], json!(true));
        timeout(Duration::from_secs(5), handle)
            .await
            .unwrap()
            .unwrap()
            .unwrap();
    }

    #[tokio::test]
    async fn runtime_recall_accepts_query_by_example_and_richer_normalized_fields() {
        let socket_path = unique_path("recall-query-by-example");
        let mut config = DaemonRuntimeConfig::new(&socket_path);
        config.maintenance_interval = Duration::from_secs(3600);
        let handle = spawn_runtime(config).await;

        timeout(Duration::from_secs(2), async {
            while tokio::fs::metadata(&socket_path).await.is_err() {
                tokio::task::yield_now().await;
            }
        })
        .await
        .unwrap();

        let recall_response = send_request(
            &socket_path,
            json!({
                "jsonrpc":"2.0",
                "method":"recall",
                "params":{
                    "namespace":"team.alpha",
                    "like_id":7,
                    "context_text":"triaging parity drift",
                    "effort":"high",
                    "include_public":true,
                    "min_confidence":0.8,
                    "result_budget":4
                },
                "id":"recall-query-by-example"
            }),
        )
        .await;

        assert_eq!(recall_response["result"]["status"], json!("ok"));
        assert_eq!(
            recall_response["result"]["retrieval"]["result"]["packaging_metadata"]["result_budget"],
            json!(4)
        );
        assert_eq!(
            recall_response["result"]["retrieval"]["result"]["explain"]["recall_plan"],
            json!("Tier2ExactThenTier3Fallback")
        );
        let degraded_summary = recall_response["result"]["retrieval"]["result"]
            ["packaging_metadata"]["degraded_summary"]
            .as_str()
            .unwrap();
        assert!(degraded_summary.contains("query_by_example"));
        assert!(degraded_summary.contains("context_text"));
        assert!(degraded_summary.contains("effort"));
        assert!(degraded_summary.contains("include_public"));
        assert!(degraded_summary.contains("min_confidence"));
        assert!(degraded_summary.contains("primary_cue=like_id"));
        assert!(degraded_summary.contains("seed_memory_ids=[7]"));
        assert!(degraded_summary.contains("seed_polarities=[like]"));
        assert_eq!(
            recall_response["result"]["retrieval"]["result"]["explain"]["query_by_example"]
                ["primary_cue"],
            json!("like_id")
        );
        assert_eq!(
            recall_response["result"]["retrieval"]["result"]["explain"]["query_by_example"]
                ["requested_seed_descriptors"],
            json!(["like:7"])
        );
        assert_eq!(
            recall_response["result"]["retrieval"]["result"]["explain"]["query_by_example"]
                ["materialized_seed_descriptors"],
            json!([])
        );
        assert_eq!(
            recall_response["result"]["retrieval"]["result"]["explain"]["query_by_example"]
                ["missing_seed_descriptors"],
            json!(["like:7"])
        );
        assert_eq!(
            recall_response["result"]["retrieval"]["result"]["explain"]["query_by_example"]
                ["expanded_candidate_count"],
            json!(1)
        );
        let result_reasons = recall_response["result"]["retrieval"]["explain_trace"]
            ["result_reasons"]
            .as_array()
            .unwrap();
        assert!(result_reasons.iter().any(|reason| {
            reason["reason_code"] == json!("query_by_example_seed_missing")
                && reason["detail"].as_str().is_some_and(|detail| {
                    detail
                        .contains("planner-only daemon recall did not materialize stored evidence")
                })
        }));
        assert!(result_reasons.iter().any(|reason| {
            reason["reason_code"] == json!("query_by_example_candidate_expansion")
                && reason["detail"].as_str().is_some_and(|detail| {
                    detail.contains(
                        "planner-only daemon envelope did not materialize stored evidence yet",
                    )
                })
        }));

        let full_contract_recall_response = send_request(
            &socket_path,
            json!({
                "jsonrpc":"2.0",
                "method":"recall",
                "params":{
                    "query_text":"memory:42",
                    "namespace":"team.alpha",
                    "mode":"semantic",
                    "result_budget":3,
                    "token_budget":256,
                    "time_budget_ms":75,
                    "context_text":"triaging parity drift",
                    "effort":"high",
                    "include_public":true,
                    "graph_mode":"expand",
                    "cold_tier":true,
                    "workspace_id":"ws-7",
                    "agent_id":"agent-3",
                    "session_id":"session-9",
                    "task_id":"task-2",
                    "memory_kinds":["user_preference","session_note"],
                    "era_id":"incident-2026",
                    "min_strength":200,
                    "min_confidence":0.8,
                    "show_decaying":true,
                    "mood_congruent":true
                },
                "id":"recall-full-contract"
            }),
        )
        .await;
        assert_eq!(
            full_contract_recall_response["result"]["status"],
            json!("ok")
        );
        let degraded_summary = full_contract_recall_response["result"]["retrieval"]["result"]
            ["packaging_metadata"]["degraded_summary"]
            .as_str()
            .unwrap();
        assert!(degraded_summary.contains("mode"));
        assert!(degraded_summary.contains("token_budget"));
        assert!(degraded_summary.contains("time_budget_ms"));
        assert!(degraded_summary.contains("graph_mode"));
        assert!(degraded_summary.contains("cold_tier"));
        assert!(degraded_summary.contains("workspace_id"));
        assert!(degraded_summary.contains("agent_id"));
        assert!(degraded_summary.contains("session_id"));
        assert!(degraded_summary.contains("task_id"));
        assert!(degraded_summary.contains("memory_kinds"));
        assert!(degraded_summary.contains("era_id"));
        assert!(degraded_summary.contains("min_strength"));
        assert!(degraded_summary.contains("show_decaying"));
        assert!(degraded_summary.contains("mood_congruent"));

        let shutdown_response = send_request(
            &socket_path,
            json!({"jsonrpc":"2.0","method":"shutdown","params":{},"id":"done"}),
        )
        .await;
        assert_eq!(shutdown_response["result"]["shutting_down"], json!(true));
        timeout(Duration::from_secs(2), handle)
            .await
            .unwrap()
            .unwrap()
            .unwrap();
    }

    #[test]
    fn normalize_recall_contract_preserves_query_by_example_seed_semantics() {
        let normalized = DaemonRuntime::normalize_recall_contract(
            NamespaceId::new("team.alpha").unwrap(),
            Some("  debugging trail  ".to_string()),
            Some(7),
            Some(9),
            None,
            Some(4),
        )
        .unwrap();

        assert_eq!(normalized.result_budget, 4);
        assert_eq!(normalized.planner_request, RecallRequest::default());
        assert_eq!(
            normalized
                .normalized_query_by_example
                .normalized_query_text
                .as_deref(),
            Some("debugging trail")
        );
        assert_eq!(
            normalized.normalized_query_by_example.primary_cue.as_str(),
            "query_text"
        );
        assert_eq!(
            normalized.normalized_query_by_example.seed_memory_ids(),
            vec![MemoryId(7), MemoryId(9)]
        );
        assert_eq!(
            normalized.normalized_query_by_example.seed_polarities(),
            vec!["like", "unlike"]
        );
    }

    #[test]
    fn normalize_recall_contract_rejects_duplicate_query_by_example_cues() {
        let error = DaemonRuntime::normalize_recall_contract(
            NamespaceId::new("team.alpha").unwrap(),
            None,
            Some(7),
            Some(7),
            None,
            Some(4),
        )
        .unwrap_err();

        assert_eq!(error, "duplicate_example_cue");
    }

    #[test]
    fn normalize_recall_contract_respects_graph_mode_expand() {
        let normalized = DaemonRuntime::normalize_recall_contract(
            NamespaceId::new("team.alpha").unwrap(),
            Some("debugging trail".to_string()),
            None,
            None,
            Some("expand"),
            Some(4),
        )
        .unwrap();

        assert_eq!(
            normalized.planner_request,
            RecallRequest::default().with_graph_expansion(true)
        );
    }

    #[test]
    fn normalize_recall_contract_keeps_graph_expansion_disabled_without_expand_mode() {
        let normalized = DaemonRuntime::normalize_recall_contract(
            NamespaceId::new("team.alpha").unwrap(),
            Some("debugging trail".to_string()),
            None,
            None,
            Some("off"),
            Some(4),
        )
        .unwrap();

        assert_eq!(normalized.planner_request, RecallRequest::default());
    }

    #[tokio::test]
    async fn runtime_recall_rejects_conflicting_history_params() {
        let socket_path = unique_path("recall-conflicting-history");
        let mut config = DaemonRuntimeConfig::new(&socket_path);
        config.maintenance_interval = Duration::from_secs(3600);
        let handle = spawn_runtime(config).await;

        timeout(Duration::from_secs(2), async {
            while tokio::fs::metadata(&socket_path).await.is_err() {
                tokio::task::yield_now().await;
            }
        })
        .await
        .unwrap();

        let recall_response = send_request(
            &socket_path,
            json!({
                "jsonrpc":"2.0",
                "method":"recall",
                "params":{
                    "query_text":"session:7",
                    "namespace":"team.alpha",
                    "as_of_tick":42,
                    "at_snapshot":"before-refactor"
                },
                "id":"recall-conflicting-history"
            }),
        )
        .await;

        assert_eq!(recall_response["error"]["code"], json!(-32602));
        assert_eq!(
            recall_response["error"]["message"],
            json!("as_of_tick and at_snapshot cannot be combined")
        );

        let shutdown_response = send_request(
            &socket_path,
            json!({"jsonrpc":"2.0","method":"shutdown","params":{},"id":"done"}),
        )
        .await;
        assert_eq!(shutdown_response["result"]["shutting_down"], json!(true));
        timeout(Duration::from_secs(2), handle)
            .await
            .unwrap()
            .unwrap()
            .unwrap();
    }

    #[tokio::test]
    async fn runtime_recall_uses_default_budget_when_limit_is_omitted() {
        let socket_path = unique_path("recall-default-budget");
        let mut config = DaemonRuntimeConfig::new(&socket_path);
        config.maintenance_interval = Duration::from_secs(3600);
        let handle = spawn_runtime(config).await;

        timeout(Duration::from_secs(2), async {
            while tokio::fs::metadata(&socket_path).await.is_err() {
                tokio::task::yield_now().await;
            }
        })
        .await
        .unwrap();

        let recall_response = send_request(
            &socket_path,
            json!({
                "jsonrpc":"2.0",
                "method":"recall",
                "params":{"query_text":"session:7","namespace":"team.alpha"},
                "id":"recall-default-budget"
            }),
        )
        .await;

        assert_eq!(recall_response["result"]["status"], json!("ok"));
        assert_eq!(
            recall_response["result"]["retrieval"]["result"]["packaging_metadata"]["result_budget"],
            json!(10)
        );
        assert_eq!(
            recall_response["result"]["retrieval"]["result"]["packaging_metadata"]
                ["degraded_summary"],
            json!("planner-only recall envelope; evidence hydration not implemented; normalized params: query_text")
        );

        let shutdown_response = send_request(
            &socket_path,
            json!({"jsonrpc":"2.0","method":"shutdown","params":{},"id":"done"}),
        )
        .await;
        assert_eq!(shutdown_response["result"]["shutting_down"], json!(true));
        timeout(Duration::from_secs(2), handle)
            .await
            .unwrap()
            .unwrap()
            .unwrap();
    }

    #[tokio::test]
    async fn runtime_preflight_methods_serve_canonical_jsonrpc_shapes() {
        let socket_path = unique_path("preflight-methods");
        let mut config = DaemonRuntimeConfig::new(&socket_path);
        config.maintenance_interval = Duration::from_secs(3600);
        let handle = spawn_runtime(config).await;

        timeout(Duration::from_secs(2), async {
            while tokio::fs::metadata(&socket_path).await.is_err() {
                tokio::task::yield_now().await;
            }
        })
        .await
        .unwrap();

        let explain_response = send_request(
            &socket_path,
            json!({
                "jsonrpc":"2.0",
                "method":"preflight.explain",
                "params":{
                    "namespace":"team.alpha",
                    "original_query":"delete prior audit events",
                    "proposed_action":"purge namespace audit history"
                },
                "id":"preflight-explain"
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
            json!(["confirmation_required"])
        );
        assert_eq!(
            explain_response["result"]["required_overrides"],
            json!(["human_confirmation"])
        );
        assert_eq!(
            explain_response["result"]["policy_context"],
            json!("namespace team.alpha preflight safeguard evaluation (irreversible mutation; irreversible_mutation)")
        );
        assert_eq!(
            explain_response["result"]["confirmation"]["required"],
            json!(true)
        );
        assert_eq!(
            explain_response["result"]["confirmation"]["confirmed"],
            json!(false)
        );
        assert_eq!(
            explain_response["result"]["check_results"][0]["check_name"],
            json!("policy")
        );
        assert_eq!(
            explain_response["result"]["audit"]["actor_source"],
            json!("daemon_jsonrpc")
        );

        let run_response = send_request(
            &socket_path,
            json!({
                "jsonrpc":"2.0",
                "method":"preflight.run",
                "params":{
                    "namespace":"team.alpha",
                    "original_query":"show maintenance summary",
                    "proposed_action":"inspect maintenance status"
                },
                "id":"preflight-run"
            }),
        )
        .await;
        assert_eq!(run_response["result"]["success"], json!(true));
        assert_eq!(run_response["result"]["preflight_state"], json!("ready"));
        assert_eq!(run_response["result"]["preflight_outcome"], json!("ready"));
        assert_eq!(run_response["result"]["outcome_class"], json!("accepted"));
        assert_eq!(run_response["result"]["blocked_reasons"], json!([]));
        assert_eq!(run_response["result"]["degraded"], json!(false));
        assert_eq!(
            run_response["result"]["policy_summary"]["decision"],
            json!("allow")
        );
        assert_eq!(
            run_response["result"]["check_results"][0]["status"],
            json!("passed")
        );
        assert!(run_response["result"].get("execution_id").is_some());

        let allow_response = send_request(
            &socket_path,
            json!({
                "jsonrpc":"2.0",
                "method":"preflight.allow",
                "params":{
                    "namespace":"team.alpha",
                    "authorization_token":"allow-123",
                    "bypass_flags":["manual_override"]
                },
                "id":"preflight-allow"
            }),
        )
        .await;
        assert_eq!(allow_response["result"]["success"], json!(true));
        assert_eq!(allow_response["result"]["preflight_state"], json!("ready"));
        assert_eq!(
            allow_response["result"]["preflight_outcome"],
            json!("force_confirmed")
        );
        assert_eq!(allow_response["result"]["outcome_class"], json!("accepted"));
        assert_eq!(
            allow_response["result"]["confirmation"]["confirmed"],
            json!(true)
        );
        assert_eq!(
            allow_response["result"]["confirmation_reason"],
            json!("operator confirmed exact previewed scope")
        );

        let shutdown_response = send_request(
            &socket_path,
            json!({"jsonrpc":"2.0","method":"shutdown","params":{},"id":"done"}),
        )
        .await;
        assert_eq!(shutdown_response["result"]["shutting_down"], json!(true));
        timeout(Duration::from_secs(2), handle)
            .await
            .unwrap()
            .unwrap()
            .unwrap();
    }

    #[tokio::test]
    async fn runtime_preflight_rejects_invalid_namespace_and_missing_confirmation() {
        let socket_path = unique_path("preflight-invalid");
        let mut config = DaemonRuntimeConfig::new(&socket_path);
        config.maintenance_interval = Duration::from_secs(3600);
        let handle = spawn_runtime(config).await;

        timeout(Duration::from_secs(2), async {
            while tokio::fs::metadata(&socket_path).await.is_err() {
                tokio::task::yield_now().await;
            }
        })
        .await
        .unwrap();

        let invalid_namespace = send_request(
            &socket_path,
            json!({
                "jsonrpc":"2.0",
                "method":"preflight.run",
                "params":{
                    "namespace":"bad namespace",
                    "original_query":"delete prior audit events",
                    "proposed_action":"purge namespace audit history"
                },
                "id":"preflight-bad-namespace"
            }),
        )
        .await;
        assert_eq!(invalid_namespace["error"]["code"], json!(-32602));

        let denied_allow = send_request(
            &socket_path,
            json!({
                "jsonrpc":"2.0",
                "method":"preflight.allow",
                "params":{
                    "namespace":"team.alpha",
                    "authorization_token":"token-123",
                    "bypass_flags":[]
                },
                "id":"preflight-denied-allow"
            }),
        )
        .await;
        assert_eq!(denied_allow["result"]["success"], json!(false));
        assert_eq!(denied_allow["result"]["preflight_state"], json!("blocked"));
        assert_eq!(
            denied_allow["result"]["preflight_outcome"],
            json!("blocked")
        );
        assert_eq!(
            denied_allow["result"]["blocked_reasons"],
            json!(["confirmation_required"])
        );
        assert_eq!(
            denied_allow["result"]["confirmation"]["confirmed"],
            json!(false)
        );
        assert!(denied_allow["result"].get("execution_id").is_none());

        let shutdown_response = send_request(
            &socket_path,
            json!({"jsonrpc":"2.0","method":"shutdown","params":{},"id":"done"}),
        )
        .await;
        assert_eq!(shutdown_response["result"]["shutting_down"], json!(true));
        timeout(Duration::from_secs(2), handle)
            .await
            .unwrap()
            .unwrap()
            .unwrap();
    }

    #[tokio::test]
    async fn runtime_preflight_force_confirm_stays_blocked_for_policy_namespace_retention_and_legal_hold(
    ) {
        let socket_path = unique_path("preflight-force-confirm-blocked");
        let mut config = DaemonRuntimeConfig::new(&socket_path);
        config.maintenance_interval = Duration::from_secs(3600);
        let handle = spawn_runtime(config).await;

        timeout(Duration::from_secs(2), async {
            while tokio::fs::metadata(&socket_path).await.is_err() {
                tokio::task::yield_now().await;
            }
        })
        .await
        .unwrap();

        let cases = [
            (
                "policy_denied",
                "team.alpha",
                "delete prior audit events after legal hold review",
                "purge namespace audit history legal hold",
                json!(["legal_hold", "confirmation_required"]),
                json!("legal hold blocks the requested action"),
                json!("deny"),
                json!("allow"),
                json!(true),
                json!("effective_namespace"),
                json!("policy"),
                json!(false),
                json!(true),
                json!("blocked"),
                json!("blocked"),
                json!("rejected"),
                json!("accepted"),
                json!(true),
                json!("ready"),
                json!("force_confirmed"),
                json!([]),
                json!("accepted"),
                json!(true),
                json!(true),
            ),
            (
                "scope_ambiguous",
                "team.alpha",
                "delete prior audit events across all namespaces",
                "purge namespace audit history",
                json!(["scope_ambiguous", "confirmation_required"]),
                json!("requested scope is ambiguous"),
                json!("allow"),
                json!("allow"),
                json!(true),
                json!("requested_scope"),
                json!("scope_precision"),
                json!(false),
                json!(true),
                json!("blocked"),
                json!("preview_only"),
                json!("preview_only"),
                json!("preview_only"),
                json!(false),
                json!("blocked"),
                json!("blocked"),
                json!(["scope_ambiguous", "confirmation_required"]),
                json!("blocked"),
                json!(false),
                json!(false),
            ),
            (
                "snapshot_required",
                "team.alpha",
                "delete archive snapshot missing evidence",
                "purge namespace audit history",
                json!(["snapshot_required", "confirmation_required"]),
                json!("snapshot is required before this action can proceed"),
                json!("allow"),
                json!("allow"),
                json!(true),
                json!("effective_namespace"),
                json!("required_input"),
                json!(false),
                json!(true),
                json!("blocked"),
                json!("blocked"),
                json!("blocked"),
                json!("blocked"),
                json!(false),
                json!("blocked"),
                json!("blocked"),
                json!(["snapshot_required", "confirmation_required"]),
                json!("blocked"),
                json!(false),
                json!(false),
            ),
            (
                "legal_hold",
                "team.alpha",
                "delete prior audit events under legal hold",
                "purge namespace audit history legal hold",
                json!(["legal_hold", "confirmation_required"]),
                json!("legal hold blocks the requested action"),
                json!("deny"),
                json!("allow"),
                json!(true),
                json!("effective_namespace"),
                json!("policy"),
                json!(false),
                json!(true),
                json!("blocked"),
                json!("blocked"),
                json!("rejected"),
                json!("accepted"),
                json!(true),
                json!("ready"),
                json!("force_confirmed"),
                json!([]),
                json!("accepted"),
                json!(true),
                json!(true),
            ),
        ];

        for (
            case_id,
            namespace,
            original_query,
            proposed_action,
            blocked_reasons,
            blocked_reason,
            explain_policy_decision,
            allow_policy_decision,
            namespace_bound,
            checked_scope,
            check_name,
            explain_confirmed,
            allow_confirmed,
            explain_preflight_state,
            explain_preflight_outcome,
            explain_outcome_class,
            allow_outcome_class,
            allow_success,
            allow_preflight_state,
            allow_preflight_outcome,
            allow_blocked_reasons,
            allow_policy_outcome_class,
            allow_has_execution_id,
            allow_has_confirmation_reason,
        ) in cases
        {
            let explain_response = send_request(
                &socket_path,
                json!({
                    "jsonrpc":"2.0",
                    "method":"preflight.explain",
                    "params":{
                        "namespace": namespace,
                        "original_query": original_query,
                        "proposed_action": proposed_action
                    },
                    "id": format!("preflight-explain-{case_id}")
                }),
            )
            .await;
            assert_eq!(
                explain_response["result"]["allowed"],
                json!(false),
                "{case_id}"
            );
            assert_eq!(
                explain_response["result"]["preflight_state"], explain_preflight_state,
                "{case_id}"
            );
            assert_eq!(
                explain_response["result"]["preflight_outcome"], explain_preflight_outcome,
                "{case_id}"
            );
            assert_eq!(
                explain_response["result"]["blocked_reasons"], blocked_reasons,
                "{case_id}"
            );
            assert_eq!(
                explain_response["result"]["blocked_reason"], blocked_reason,
                "{case_id}"
            );
            assert_eq!(
                explain_response["result"]["policy_summary"]["decision"], explain_policy_decision,
                "{case_id}"
            );
            assert_eq!(
                explain_response["result"]["policy_summary"]["namespace_bound"], namespace_bound,
                "{case_id}"
            );
            assert_eq!(
                explain_response["result"]["policy_summary"]["outcome_class"],
                explain_outcome_class,
                "{case_id}"
            );
            assert_eq!(
                explain_response["result"]["confirmation"]["required"],
                json!(true),
                "{case_id}"
            );
            assert_eq!(
                explain_response["result"]["confirmation"]["force_allowed"],
                json!(true),
                "{case_id}"
            );
            assert_eq!(
                explain_response["result"]["confirmation"]["confirmed"], explain_confirmed,
                "{case_id}"
            );
            assert_eq!(
                explain_response["result"]["check_results"][0]["check_name"], check_name,
                "{case_id}"
            );
            assert_eq!(
                explain_response["result"]["check_results"][0]["checked_scope"], checked_scope,
                "{case_id}"
            );

            let allow_response = send_request(
                &socket_path,
                json!({
                    "jsonrpc":"2.0",
                    "method":"preflight.allow",
                    "params":{
                        "namespace": namespace,
                        "authorization_token":"allow-123",
                        "bypass_flags":["manual_override"]
                    },
                    "id": format!("preflight-allow-{case_id}")
                }),
            )
            .await;
            assert_eq!(
                allow_response["result"]["success"], allow_success,
                "{case_id}"
            );
            assert_eq!(
                allow_response["result"]["preflight_state"], allow_preflight_state,
                "{case_id}"
            );
            assert_eq!(
                allow_response["result"]["preflight_outcome"], allow_preflight_outcome,
                "{case_id}"
            );
            assert_eq!(
                allow_response["result"]["outcome_class"], allow_outcome_class,
                "{case_id}"
            );
            assert_eq!(
                allow_response["result"]["blocked_reasons"], allow_blocked_reasons,
                "{case_id}"
            );
            assert_eq!(
                allow_response["result"]["policy_summary"]["decision"], allow_policy_decision,
                "{case_id}"
            );
            assert_eq!(
                allow_response["result"]["policy_summary"]["namespace_bound"], namespace_bound,
                "{case_id}"
            );
            assert_eq!(
                allow_response["result"]["policy_summary"]["outcome_class"],
                allow_policy_outcome_class,
                "{case_id}"
            );
            assert_eq!(
                allow_response["result"]["confirmation"]["required"],
                json!(true),
                "{case_id}"
            );
            assert_eq!(
                allow_response["result"]["confirmation"]["force_allowed"],
                json!(true),
                "{case_id}"
            );
            assert_eq!(
                allow_response["result"]["confirmation"]["confirmed"], allow_confirmed,
                "{case_id}"
            );
            assert_eq!(
                allow_response["result"]["check_results"][0]["check_name"], check_name,
                "{case_id}"
            );
            assert_eq!(
                allow_response["result"]["check_results"][0]["checked_scope"], checked_scope,
                "{case_id}"
            );
            assert_eq!(
                json!(allow_response["result"].get("execution_id").is_some()),
                allow_has_execution_id,
                "{case_id}"
            );
            assert_eq!(
                json!(allow_response["result"]
                    .get("confirmation_reason")
                    .is_some()),
                allow_has_confirmation_reason,
                "{case_id}"
            );
        }

        let shutdown_response = send_request(
            &socket_path,
            json!({"jsonrpc":"2.0","method":"shutdown","params":{},"id":"done"}),
        )
        .await;
        assert_eq!(shutdown_response["result"]["shutting_down"], json!(true));
        timeout(Duration::from_secs(2), handle)
            .await
            .unwrap()
            .unwrap()
            .unwrap();
    }

    #[tokio::test]
    async fn runtime_inspect_returns_typed_mcp_inspect_payload() {
        let socket_path = unique_path("inspect");
        let mut config = DaemonRuntimeConfig::new(&socket_path);
        config.maintenance_interval = Duration::from_secs(3600);
        let handle = spawn_runtime(config).await;

        timeout(Duration::from_secs(2), async {
            while tokio::fs::metadata(&socket_path).await.is_err() {
                tokio::task::yield_now().await;
            }
        })
        .await
        .unwrap();

        let inspect_response = send_request(
            &socket_path,
            json!({
                "jsonrpc":"2.0",
                "method":"inspect",
                "params":{"id":42,"namespace":"team.alpha"},
                "id":"inspect"
            }),
        )
        .await;

        assert_eq!(inspect_response["result"]["status"], json!("ok"));
        assert!(inspect_response["result"].get("retrieval").is_none());
        assert_eq!(
            inspect_response["result"]["payload"]["namespace"],
            json!("team.alpha")
        );
        assert_eq!(
            inspect_response["result"]["payload"]["memory_id"],
            json!(42)
        );
        assert_eq!(
            inspect_response["result"]["payload"]["tier"],
            json!("tier1_exact")
        );
        assert_eq!(
            inspect_response["result"]["payload"]["lifecycle_state"]["degraded_summary"],
            json!("planner-only inspect envelope; item hydration not implemented")
        );
        assert_eq!(
            inspect_response["result"]["payload"]["index_presence"]["graph_assistance"],
            json!("none")
        );
        assert!(inspect_response["result"]["payload"]
            .get("explain_trace")
            .is_some());

        let shutdown_response = send_request(
            &socket_path,
            json!({"jsonrpc":"2.0","method":"shutdown","params":{},"id":"done"}),
        )
        .await;
        assert_eq!(shutdown_response["result"]["shutting_down"], json!(true));
        timeout(Duration::from_secs(2), handle)
            .await
            .unwrap()
            .unwrap()
            .unwrap();
    }

    #[tokio::test]
    async fn runtime_explain_returns_typed_mcp_retrieval_payload() {
        let socket_path = unique_path("explain");
        let mut config = DaemonRuntimeConfig::new(&socket_path);
        config.maintenance_interval = Duration::from_secs(3600);
        let handle = spawn_runtime(config).await;

        timeout(Duration::from_secs(2), async {
            while tokio::fs::metadata(&socket_path).await.is_err() {
                tokio::task::yield_now().await;
            }
        })
        .await
        .unwrap();

        let explain_response = send_request(
            &socket_path,
            json!({
                "jsonrpc":"2.0",
                "method":"explain",
                "params":{"query":"session:7","namespace":"team.alpha","limit":2},
                "id":"explain"
            }),
        )
        .await;

        assert_eq!(explain_response["result"]["status"], json!("ok"));
        assert_eq!(
            explain_response["result"]["retrieval"]["namespace"],
            json!("team.alpha")
        );
        assert_eq!(
            explain_response["result"]["retrieval"]["result"]["explain"]["recall_plan"],
            json!("RecentTier1ThenTier2Exact")
        );
        assert_eq!(
            explain_response["result"]["retrieval"]["result"]["packaging_metadata"]
                ["result_budget"],
            json!(2)
        );
        assert_eq!(
            explain_response["result"]["retrieval"]["result"]["packaging_metadata"]
                ["degraded_summary"],
            json!("planner-only explain envelope; evidence hydration not implemented")
        );
        assert!(explain_response["result"]["retrieval"]
            .get("explain_trace")
            .is_some());
        assert!(
            explain_response["result"]["retrieval"]["result"]["explain"]["query_by_example"]
                .is_null()
        );

        let shutdown_response = send_request(
            &socket_path,
            json!({"jsonrpc":"2.0","method":"shutdown","params":{},"id":"done"}),
        )
        .await;
        assert_eq!(shutdown_response["result"]["shutting_down"], json!(true));
        timeout(Duration::from_secs(2), handle)
            .await
            .unwrap()
            .unwrap()
            .unwrap();
    }

    #[tokio::test]
    async fn runtime_inspect_keeps_canonical_payload_families_in_typed_payload_slot() {
        let socket_path = unique_path("inspect-canonical-payload-families");
        let mut config = DaemonRuntimeConfig::new(&socket_path);
        config.maintenance_interval = Duration::from_secs(3600);
        let handle = spawn_runtime(config).await;

        timeout(Duration::from_secs(2), async {
            while tokio::fs::metadata(&socket_path).await.is_err() {
                tokio::task::yield_now().await;
            }
        })
        .await
        .unwrap();

        let inspect_response = send_request(
            &socket_path,
            json!({
                "jsonrpc":"2.0",
                "method":"inspect",
                "params":{"id":42,"namespace":"team.alpha"},
                "id":"inspect-canonical-payload-families"
            }),
        )
        .await;

        let result = &inspect_response["result"];
        assert_eq!(result["status"], json!("ok"));
        assert!(result.get("retrieval").is_none());
        assert!(result.get("error").is_none());

        let payload = &result["payload"];
        assert_eq!(payload["request_id"], json!("daemon-inspect-1"));
        assert_eq!(payload["memory_id"], json!(42));
        assert_eq!(payload["tier"], json!("tier1_exact"));
        assert!(payload.get("lineage").is_some());
        assert!(payload.get("policy_flags").is_some());
        assert!(payload.get("lifecycle_state").is_some());
        assert!(payload.get("index_presence").is_some());
        assert!(payload.get("graph_neighborhood_summary").is_some());
        assert!(payload.get("decay_retention").is_some());
        assert!(payload.get("explain_trace").is_some());
        assert_eq!(
            payload["explain_trace"]["passive_observation"]["resource_uri"],
            json!("membrain://team.alpha/memories/42")
        );
        assert_eq!(
            payload["explain_trace"]["passive_observation"]["resource_template"],
            json!("membrain://{namespace}/memories/{memory_id}")
        );
        assert_eq!(
            payload["lifecycle_state"]["degraded_summary"],
            json!("planner-only inspect envelope; item hydration not implemented")
        );
        assert_eq!(
            payload["explain_trace"]["policy_summary"]["effective_namespace"],
            json!("team.alpha")
        );

        let shutdown_response = send_request(
            &socket_path,
            json!({"jsonrpc":"2.0","method":"shutdown","params":{},"id":"done"}),
        )
        .await;
        assert_eq!(shutdown_response["result"]["shutting_down"], json!(true));
        timeout(Duration::from_secs(2), handle)
            .await
            .unwrap()
            .unwrap()
            .unwrap();
    }

    #[tokio::test]
    async fn runtime_inspect_missing_id_returns_invalid_params_error() {
        let socket_path = unique_path("inspect-missing-id");
        let mut config = DaemonRuntimeConfig::new(&socket_path);
        config.maintenance_interval = Duration::from_secs(3600);
        let handle = spawn_runtime(config).await;

        timeout(Duration::from_secs(2), async {
            while tokio::fs::metadata(&socket_path).await.is_err() {
                tokio::task::yield_now().await;
            }
        })
        .await
        .unwrap();

        let response = send_request(
            &socket_path,
            json!({
                "jsonrpc":"2.0",
                "method":"inspect",
                "params":{"namespace":"team.alpha"},
                "id":"inspect-missing-id"
            }),
        )
        .await;

        assert_eq!(response["error"]["code"], json!(-32602));
        assert_eq!(response["error"]["message"], json!("missing id"));

        let zero_id_response = send_request(
            &socket_path,
            json!({
                "jsonrpc":"2.0",
                "method":"inspect",
                "params":{"id":0,"namespace":"team.alpha"},
                "id":"inspect-zero-id"
            }),
        )
        .await;

        assert_eq!(zero_id_response["error"]["code"], json!(-32602));
        assert_eq!(
            zero_id_response["error"]["message"],
            json!("id must be at least 1")
        );

        let fractional_id_response = send_request(
            &socket_path,
            json!({
                "jsonrpc":"2.0",
                "method":"inspect",
                "params":{"id":1.5,"namespace":"team.alpha"},
                "id":"inspect-fractional-id"
            }),
        )
        .await;

        assert_eq!(fractional_id_response["error"]["code"], json!(-32602));
        assert_eq!(
            fractional_id_response["error"]["message"],
            json!("id must be a positive integer")
        );

        let shutdown_response = send_request(
            &socket_path,
            json!({"jsonrpc":"2.0","method":"shutdown","params":{},"id":"done"}),
        )
        .await;
        assert_eq!(shutdown_response["result"]["shutting_down"], json!(true));
        timeout(Duration::from_secs(2), handle)
            .await
            .unwrap()
            .unwrap()
            .unwrap();
    }

    #[tokio::test]
    async fn runtime_explain_invalid_limit_returns_invalid_params_error() {
        let socket_path = unique_path("explain-invalid-limit");
        let mut config = DaemonRuntimeConfig::new(&socket_path);
        config.maintenance_interval = Duration::from_secs(3600);
        let handle = spawn_runtime(config).await;

        timeout(Duration::from_secs(2), async {
            while tokio::fs::metadata(&socket_path).await.is_err() {
                tokio::task::yield_now().await;
            }
        })
        .await
        .unwrap();

        let response = send_request(
            &socket_path,
            json!({
                "jsonrpc":"2.0",
                "method":"explain",
                "params":{"query":"session:7","namespace":"team.alpha","limit":0},
                "id":"explain-invalid-limit"
            }),
        )
        .await;

        assert_eq!(response["error"]["code"], json!(-32602));
        assert_eq!(
            response["error"]["message"],
            json!("limit must be at least 1")
        );

        let shutdown_response = send_request(
            &socket_path,
            json!({"jsonrpc":"2.0","method":"shutdown","params":{},"id":"done"}),
        )
        .await;
        assert_eq!(shutdown_response["result"]["shutting_down"], json!(true));
        timeout(Duration::from_secs(2), handle)
            .await
            .unwrap()
            .unwrap()
            .unwrap();
    }

    #[tokio::test]
    async fn runtime_reuses_error_model_for_invalid_jsonrpc_and_unknown_methods() {
        let socket_path = unique_path("error-model-parity");
        let mut config = DaemonRuntimeConfig::new(&socket_path);
        config.maintenance_interval = Duration::from_secs(3600);
        let handle = spawn_runtime(config).await;

        timeout(Duration::from_secs(2), async {
            while tokio::fs::metadata(&socket_path).await.is_err() {
                tokio::task::yield_now().await;
            }
        })
        .await
        .unwrap();

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

        let shutdown_response = send_request(
            &socket_path,
            json!({"jsonrpc":"2.0","method":"shutdown","params":{},"id":"done"}),
        )
        .await;
        assert_eq!(shutdown_response["result"]["shutting_down"], json!(true));
        timeout(Duration::from_secs(2), handle)
            .await
            .unwrap()
            .unwrap()
            .unwrap();
    }

    #[tokio::test]
    async fn runtime_reports_degraded_and_read_only_postures() {
        let socket_path = unique_path("posture");
        let mut config = DaemonRuntimeConfig::new(&socket_path);
        config.maintenance_interval = Duration::from_secs(3600);
        let handle = spawn_runtime(config).await;

        timeout(Duration::from_secs(2), async {
            while tokio::fs::metadata(&socket_path).await.is_err() {
                tokio::task::yield_now().await;
            }
        })
        .await
        .unwrap();

        let degraded = send_request(
            &socket_path,
            json!({
                "jsonrpc":"2.0",
                "method":"set_posture",
                "params":{"posture":"degraded","reasons":["repair_in_flight"]},
                "id":"degraded"
            }),
        )
        .await;
        assert_eq!(degraded["result"]["posture"], json!("degraded"));
        assert_eq!(
            degraded["result"]["degraded_reasons"],
            json!(["repair_in_flight"])
        );

        let read_only = send_request(
            &socket_path,
            json!({
                "jsonrpc":"2.0",
                "method":"set_posture",
                "params":{"posture":"read_only","reasons":["authoritative_input_unreadable"]},
                "id":"read_only"
            }),
        )
        .await;
        assert_eq!(read_only["result"]["posture"], json!("read_only"));

        let shutdown_response = send_request(
            &socket_path,
            json!({"jsonrpc":"2.0","method":"shutdown","params":{},"id":"done"}),
        )
        .await;
        assert_eq!(shutdown_response["result"]["shutting_down"], json!(true));
        timeout(Duration::from_secs(2), handle)
            .await
            .unwrap()
            .unwrap()
            .unwrap();
    }

    #[tokio::test]
    async fn runtime_resources_list_and_read_return_bounded_typed_payloads() {
        let socket_path = unique_path("resources-list-and-read");
        let mut config = DaemonRuntimeConfig::new(&socket_path);
        config.maintenance_interval = Duration::from_secs(3600);
        let handle = spawn_runtime(config).await;

        timeout(Duration::from_secs(2), async {
            while tokio::fs::metadata(&socket_path).await.is_err() {
                tokio::task::yield_now().await;
            }
        })
        .await
        .unwrap();

        let resources_response = send_request(
            &socket_path,
            json!({
                "jsonrpc":"2.0",
                "method":"resources.list",
                "params":{},
                "id":"resources-list"
            }),
        )
        .await;
        assert_eq!(resources_response["result"]["status"], json!("ok"));
        assert_eq!(
            resources_response["result"]["payload"]["namespace"],
            json!("daemon.runtime")
        );
        assert_eq!(
            resources_response["result"]["payload"]["resources"][0]["uri"],
            json!("membrain://daemon/runtime/status")
        );
        assert_eq!(
            resources_response["result"]["payload"]["resources"][0]["resource_kind"],
            json!("runtime_status")
        );
        assert_eq!(
            resources_response["result"]["payload"]["resources"][1]["uri"],
            json!("membrain://daemon/runtime/doctor")
        );
        assert_eq!(
            resources_response["result"]["payload"]["resources"][2]["uri"],
            json!("membrain://daemon/runtime/streams")
        );
        assert_eq!(
            resources_response["result"]["payload"]["resources"][2]["resource_kind"],
            json!("stream_listing")
        );
        assert_eq!(
            resources_response["result"]["payload"]["resources"][3]["uri_template"],
            json!("membrain://{namespace}/memories/{memory_id}")
        );
        assert_eq!(
            resources_response["result"]["payload"]["resources"][3]["examples"][0],
            json!("membrain://team.alpha/memories/42")
        );

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
            status_resource["result"]["payload"]["request_id"],
            json!("daemon-resource-read-status-2")
        );
        assert_eq!(
            status_resource["result"]["payload"]["namespace"],
            json!("daemon.runtime")
        );
        assert_eq!(
            status_resource["result"]["payload"]["uri"],
            json!("membrain://daemon/runtime/status")
        );
        assert_eq!(
            status_resource["result"]["payload"]["mime_type"],
            json!("application/json")
        );
        assert_eq!(
            status_resource["result"]["payload"]["resource_kind"],
            json!("runtime_status")
        );
        assert_eq!(status_resource["result"]["payload"]["bounded"], json!(true));
        assert_eq!(
            status_resource["result"]["payload"]["payload"]["posture"],
            json!("full")
        );

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
        assert_eq!(doctor_resource["result"]["status"], json!("ok"));
        assert_eq!(
            doctor_resource["result"]["payload"]["request_id"],
            json!("daemon-resource-read-doctor-3")
        );
        assert_eq!(
            doctor_resource["result"]["payload"]["uri"],
            json!("membrain://daemon/runtime/doctor")
        );
        assert_eq!(
            doctor_resource["result"]["payload"]["resource_kind"],
            json!("runtime_doctor")
        );
        assert_eq!(
            doctor_resource["result"]["payload"]["payload"]["status"],
            json!("ok")
        );
        assert!(doctor_resource["result"]["payload"]["payload"]
            .get("indexes")
            .is_some());

        let stream_resource = send_request(
            &socket_path,
            json!({
                "jsonrpc":"2.0",
                "method":"resource.read",
                "params":{"uri":"membrain://daemon/runtime/streams"},
                "id":"resource-read-streams"
            }),
        )
        .await;
        assert_eq!(stream_resource["result"]["status"], json!("ok"));
        assert_eq!(
            stream_resource["result"]["payload"]["request_id"],
            json!("daemon-resource-read-streams-4")
        );
        assert_eq!(
            stream_resource["result"]["payload"]["resource_kind"],
            json!("stream_listing")
        );
        assert_eq!(
            stream_resource["result"]["payload"]["payload"]["streams"][0]["method"],
            json!("maintenance.status")
        );
        assert_eq!(
            stream_resource["result"]["payload"]["payload"]["streams"][0]["delivery"],
            json!("jsonrpc_notification")
        );
        assert_eq!(
            stream_resource["result"]["payload"]["payload"]["streams"][0]["example_subscriptions"]
                [0],
            json!("maintenance.status")
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
            streams_list["result"]["payload"]["streams"][0]["name"],
            json!("maintenance-status")
        );
        assert_eq!(
            streams_list["result"]["payload"]["streams"][0]["example_subscriptions"][0],
            json!("maintenance.status")
        );

        let unknown_resource = send_request(
            &socket_path,
            json!({
                "jsonrpc":"2.0",
                "method":"resource.read",
                "params":{"uri":"membrain://daemon/runtime/missing"},
                "id":"resource-read-missing"
            }),
        )
        .await;
        assert_eq!(unknown_resource["error"]["code"], json!(-32602));
        assert_eq!(
            unknown_resource["error"]["message"],
            json!("unknown resource uri 'membrain://daemon/runtime/missing'")
        );

        let shutdown_response = send_request(
            &socket_path,
            json!({"jsonrpc":"2.0","method":"shutdown","params":{},"id":"done"}),
        )
        .await;
        assert_eq!(shutdown_response["result"]["shutting_down"], json!(true));
        timeout(Duration::from_secs(2), handle)
            .await
            .unwrap()
            .unwrap()
            .unwrap();
    }

    #[tokio::test]
    async fn shutdown_cancels_inflight_request_with_structured_error() {
        let socket_path = unique_path("shutdown-cancel");
        let mut config = DaemonRuntimeConfig::new(&socket_path);
        config.request_concurrency = 1;
        config.max_queue_depth = 4;
        config.maintenance_interval = Duration::from_secs(3600);
        let handle = spawn_runtime(config).await;

        timeout(Duration::from_secs(2), async {
            while tokio::fs::metadata(&socket_path).await.is_err() {
                tokio::task::yield_now().await;
            }
        })
        .await
        .unwrap();

        let socket_path_clone = socket_path.clone();
        let inflight = tokio::spawn(async move {
            send_request(
                &socket_path_clone,
                json!({"jsonrpc":"2.0","method":"sleep","params":{"millis":1000},"id":"slow"}),
            )
            .await
        });

        tokio::time::sleep(Duration::from_millis(100)).await;
        let shutdown_response = send_request(
            &socket_path,
            json!({"jsonrpc":"2.0","method":"shutdown","params":{},"id":"shutdown"}),
        )
        .await;
        assert_eq!(shutdown_response["result"]["shutting_down"], json!(true));

        let inflight_response = inflight.await.unwrap();
        assert_eq!(inflight_response["error"]["code"], json!(-32002));
        assert_eq!(
            inflight_response["error"]["data"]["reason"],
            json!("server_shutdown")
        );

        timeout(Duration::from_secs(2), handle)
            .await
            .unwrap()
            .unwrap()
            .unwrap();
    }

    #[tokio::test]
    async fn runtime_treats_run_maintenance_without_id_as_notification() {
        let socket_path = unique_path("maintenance-notification");
        let mut config = DaemonRuntimeConfig::new(&socket_path);
        config.maintenance_interval = Duration::from_secs(3600);
        let handle = spawn_runtime(config).await;

        timeout(Duration::from_secs(2), async {
            while tokio::fs::metadata(&socket_path).await.is_err() {
                tokio::task::yield_now().await;
            }
        })
        .await
        .unwrap();

        send_notification(
            &socket_path,
            json!({
                "jsonrpc":"2.0",
                "method":"run_maintenance",
                "params":{"polls_budget":2,"step_delay_ms":10}
            }),
        )
        .await;

        tokio::time::sleep(Duration::from_millis(40)).await;
        let status_response = send_request(
            &socket_path,
            json!({"jsonrpc":"2.0","method":"status","params":{},"id":"status"}),
        )
        .await;
        assert_eq!(
            status_response["result"]["metrics"]["maintenance_runs"],
            json!(1)
        );

        let shutdown_response = send_request(
            &socket_path,
            json!({"jsonrpc":"2.0","method":"shutdown","params":{},"id":"done"}),
        )
        .await;
        assert_eq!(shutdown_response["result"]["shutting_down"], json!(true));

        timeout(Duration::from_secs(2), handle)
            .await
            .unwrap()
            .unwrap()
            .unwrap();
    }

    #[tokio::test]
    async fn shutdown_skips_manual_run_maintenance_accounting_when_cancelled_before_first_step() {
        let config =
            DaemonRuntimeConfig::new(unique_path("maintenance-budget-cancelled").as_path());
        let state = Arc::new(super::RuntimeState::new(&config));

        let maintenance_finished = super::DaemonRuntime::run_maintenance_budget(
            state.as_ref(),
            1,
            Duration::from_millis(500),
        );
        tokio::pin!(maintenance_finished);

        tokio::task::yield_now().await;
        state.request_shutdown();

        assert!(!maintenance_finished.await);
        assert_eq!(state.status().await.metrics.maintenance_runs, 0);
    }

    #[tokio::test]
    async fn runtime_does_not_reply_to_notifications_cancelled_while_waiting_for_capacity() {
        let socket_path = unique_path("notification-shutdown-capacity");
        let mut config = DaemonRuntimeConfig::new(&socket_path);
        config.request_concurrency = 1;
        config.maintenance_interval = Duration::from_secs(3600);
        let handle = spawn_runtime(config).await;

        timeout(Duration::from_secs(2), async {
            while tokio::fs::metadata(&socket_path).await.is_err() {
                tokio::task::yield_now().await;
            }
        })
        .await
        .unwrap();

        let sleep_socket = socket_path.clone();
        let inflight = tokio::spawn(async move {
            send_request(
                &sleep_socket,
                json!({"jsonrpc":"2.0","method":"sleep","params":{"millis":300},"id":"sleep"}),
            )
            .await
        });

        tokio::time::sleep(Duration::from_millis(50)).await;

        let notify_socket = socket_path.clone();
        let notification = tokio::spawn(async move {
            send_notification(
                &notify_socket,
                json!({"jsonrpc":"2.0","method":"status","params":{}}),
            )
            .await
        });

        let idle_stream = UnixStream::connect(&socket_path).await.unwrap();

        tokio::time::sleep(Duration::from_millis(50)).await;
        let shutdown_response = send_request(
            &socket_path,
            json!({"jsonrpc":"2.0","method":"shutdown","params":{},"id":"shutdown"}),
        )
        .await;
        assert_eq!(shutdown_response["result"]["shutting_down"], json!(true));

        notification.await.unwrap();

        let inflight_response = inflight.await.unwrap();
        assert_eq!(inflight_response["error"]["code"], json!(-32002));
        assert_eq!(
            inflight_response["error"]["data"]["reason"],
            json!("server_shutdown")
        );

        drop(idle_stream);

        timeout(Duration::from_secs(2), handle)
            .await
            .unwrap()
            .unwrap()
            .unwrap();
    }
}
