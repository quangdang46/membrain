use crate::mcp::{McpResponse, McpRetrievalPayload};
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
use membrain_core::engine::result::{RetrievalExplain, RetrievalResultSet};
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
                query,
                namespace,
                limit,
            } => match Self::handle_recall(&query, &namespace, limit, request_correlation_id) {
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
                let _ = namespace_id;
                JsonRpcResponse::success(
                    request_id,
                    json!({
                        "status": "accepted",
                        "id": id,
                        "message": "share envelope accepted; sharing pipeline not yet wired"
                    }),
                )
            }
            RuntimeRequest::Unshare { id, namespace } => {
                let _ = namespace;
                JsonRpcResponse::success(
                    request_id,
                    json!({
                        "status": "accepted",
                        "id": id,
                        "message": "unshare envelope accepted; sharing pipeline not yet wired"
                    }),
                )
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

    fn handle_recall(
        query: &str,
        namespace: &str,
        limit: Option<usize>,
        request_correlation_id: u64,
    ) -> Result<McpResponse, String> {
        let request = Self::recall_request_from_query(query)?;
        Self::handle_retrieval_method(
            request,
            namespace,
            limit,
            request_correlation_id,
            "recall",
            "planner-only recall envelope; evidence hydration not implemented",
        )
    }

    fn handle_inspect(
        id: u64,
        namespace: &str,
        request_correlation_id: u64,
    ) -> Result<McpResponse, String> {
        Self::handle_retrieval_method(
            RecallRequest::exact(MemoryId(id)),
            namespace,
            Some(1),
            request_correlation_id,
            "inspect",
            "planner-only inspect envelope; item hydration not implemented",
        )
    }

    fn handle_explain(
        query: &str,
        namespace: &str,
        limit: Option<usize>,
        request_correlation_id: u64,
    ) -> Result<McpResponse, String> {
        let request = Self::recall_request_from_query(query)?;
        Self::handle_retrieval_method(
            request,
            namespace,
            limit,
            request_correlation_id,
            "explain",
            "planner-only explain envelope; evidence hydration not implemented",
        )
    }

    fn handle_retrieval_method(
        request: RecallRequest,
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
        let result_budget = Self::canonical_result_budget(request, limit);
        let mut result = RetrievalResultSet::empty(
            RetrievalExplain::from_plan(&plan, "balanced"),
            namespace.clone(),
        );
        result.outcome_class = membrain_core::observability::OutcomeClass::Degraded;
        result.policy_summary.outcome_class = membrain_core::observability::OutcomeClass::Degraded;
        result.packaging_metadata.result_budget = result_budget;
        result.packaging_metadata.degraded_summary = Some(degraded_summary.to_string());
        result.truncated = false;
        let payload = McpRetrievalPayload::from_result(request_id, namespace, false, result)
            .map_err(|err| err.to_string())?;
        Ok(McpResponse::retrieval_success(payload))
    }

    fn recall_request_from_query(query: &str) -> Result<RecallRequest, String> {
        if let Some(memory_id) = query.strip_prefix("memory:") {
            let memory_id = memory_id
                .parse::<u64>()
                .map_err(|_| format!("invalid memory query '{query}'"))?;
            return Ok(RecallRequest::exact(MemoryId(memory_id)));
        }

        if let Some(session_id) = query.strip_prefix("session:") {
            let session_id = session_id
                .parse::<u64>()
                .map_err(|_| format!("invalid session query '{query}'"))?;
            return Ok(RecallRequest::small_session_lookup(SessionId(session_id)));
        }

        Ok(RecallRequest::default())
    }

    fn canonical_result_budget(request: RecallRequest, limit: Option<usize>) -> usize {
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

    fn handle_preflight_run(
        namespace: &str,
        original_query: &str,
        proposed_action: &str,
        request_correlation_id: u64,
    ) -> Result<PreflightOutcome, String> {
        Self::validate_preflight_namespace(namespace)?;
        let _request = PreflightRunRequest {
            namespace: namespace.to_string(),
            original_query: original_query.to_string(),
            proposed_action: proposed_action.to_string(),
        };
        let preflight_id = format!("preflight-{request_correlation_id}");
        let execution_id = format!("exec-{request_correlation_id}");
        let lowered = proposed_action.to_ascii_lowercase();
        let blocked_reasons = if lowered.contains("purge") || lowered.contains("delete") {
            vec!["confirmation_required".to_string()]
        } else {
            Vec::new()
        };
        let preflight_state = if blocked_reasons.is_empty() {
            "ready"
        } else {
            "blocked"
        };
        Ok(PreflightOutcome {
            success: blocked_reasons.is_empty(),
            preflight_state: preflight_state.to_string(),
            blocked_reasons,
            request_id: Some(format!("daemon-preflight-run-{request_correlation_id}")),
            preflight_id: Some(preflight_id),
            execution_id: Some(execution_id),
            degraded: false,
            confirmation_reason: None,
        })
    }

    fn handle_preflight_explain(
        namespace: &str,
        original_query: &str,
        proposed_action: &str,
        request_correlation_id: u64,
    ) -> Result<PreflightExplainResponse, String> {
        Self::validate_preflight_namespace(namespace)?;
        let _request = PreflightRunRequest {
            namespace: namespace.to_string(),
            original_query: original_query.to_string(),
            proposed_action: proposed_action.to_string(),
        };
        let lowered = proposed_action.to_ascii_lowercase();
        let blocked_reasons = if lowered.contains("purge") || lowered.contains("delete") {
            vec!["confirmation_required".to_string()]
        } else {
            Vec::new()
        };
        let blocked_reason = blocked_reasons
            .first()
            .map(|_| "destructive action requires explicit confirmation".to_string());
        let allowed = blocked_reasons.is_empty();
        let required_overrides = if allowed {
            Vec::new()
        } else {
            vec!["human_confirmation".to_string()]
        };
        Ok(PreflightExplainResponse {
            allowed,
            preflight_state: if allowed { "ready" } else { "blocked" }.to_string(),
            blocked_reasons,
            blocked_reason,
            required_overrides,
            policy_context: format!("namespace {namespace} preflight safeguard evaluation"),
            request_id: Some(format!("daemon-preflight-explain-{request_correlation_id}")),
            preflight_id: Some(format!("preflight-{request_correlation_id}")),
        })
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
        let blocked_reasons = if confirmed {
            Vec::new()
        } else {
            vec!["confirmation_required".to_string()]
        };
        Ok(PreflightOutcome {
            success: confirmed,
            preflight_state: if confirmed { "ready" } else { "blocked" }.to_string(),
            blocked_reasons,
            request_id: Some(format!("daemon-preflight-allow-{request_correlation_id}")),
            preflight_id: Some(format!("preflight-{request_correlation_id}")),
            execution_id: if confirmed {
                Some(format!("exec-{request_correlation_id}"))
            } else {
                None
            },
            degraded: false,
            confirmation_reason: if confirmed {
                Some("operator confirmed exact previewed scope".to_string())
            } else {
                None
            },
        })
    }

    async fn maintenance_loop(state: Arc<RuntimeState>, config: DaemonRuntimeConfig) {
        let mut ticker = tokio::time::interval(config.maintenance_interval);
        ticker.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);
        ticker.tick().await;

        loop {
            tokio::select! {
                _ = state.shutdown_notify.notified() => {
                    break;
                }
                _ = ticker.tick() => {
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
            Ok(Ok(_)) => panic!("notification unexpectedly received a response: {line}"),
            Ok(Err(err)) => panic!("failed to read notification response state: {err}"),
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
                "params":{"query":"memory:42","namespace":"team.alpha","limit":3},
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
                "params":{"query":"memory:42","namespace":"team.alpha","limit":3},
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
        assert_eq!(retrieval["outcome_class"], json!("Degraded"));
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
            json!("planner-only recall envelope; evidence hydration not implemented")
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
                "params":{"query":"memory:not-a-number","namespace":"team.alpha"},
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
                "params":{"query":"session:7","namespace":"team.alpha","limit":3},
                "id":"recall-session-budget"
            }),
        )
        .await;

        assert_eq!(recall_response["result"]["status"], json!("ok"));
        assert_eq!(
            recall_response["result"]["retrieval"]["outcome_class"],
            json!("Degraded")
        );
        assert_eq!(
            recall_response["result"]["retrieval"]["result"]["outcome_class"],
            json!("Degraded")
        );
        assert_eq!(
            recall_response["result"]["retrieval"]["result"]["packaging_metadata"]["result_budget"],
            json!(3)
        );
        assert_eq!(
            recall_response["result"]["retrieval"]["result"]["packaging_metadata"]
                ["degraded_summary"],
            json!("planner-only recall envelope; evidence hydration not implemented")
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
                "params":{"query":"session:7","namespace":"team.alpha"},
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
            json!("planner-only recall envelope; evidence hydration not implemented")
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
            explain_response["result"]["blocked_reasons"],
            json!(["confirmation_required"])
        );
        assert_eq!(
            explain_response["result"]["required_overrides"],
            json!(["human_confirmation"])
        );
        assert_eq!(
            explain_response["result"]["policy_context"],
            json!("namespace team.alpha preflight safeguard evaluation")
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
        assert_eq!(run_response["result"]["blocked_reasons"], json!([]));
        assert_eq!(run_response["result"]["degraded"], json!(false));
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
            denied_allow["result"]["blocked_reasons"],
            json!(["confirmation_required"])
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
    async fn runtime_inspect_returns_typed_mcp_retrieval_payload() {
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
        assert_eq!(
            inspect_response["result"]["retrieval"]["namespace"],
            json!("team.alpha")
        );
        assert_eq!(
            inspect_response["result"]["retrieval"]["result"]["explain"]["recall_plan"],
            json!("ExactIdTier1")
        );
        assert_eq!(
            inspect_response["result"]["retrieval"]["result"]["packaging_metadata"]
                ["result_budget"],
            json!(1)
        );
        assert_eq!(
            inspect_response["result"]["retrieval"]["result"]["packaging_metadata"]
                ["degraded_summary"],
            json!("planner-only inspect envelope; item hydration not implemented")
        );
        assert!(inspect_response["result"]["retrieval"]
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
    async fn runtime_inspect_keeps_canonical_payload_families_out_of_generic_payload_slot() {
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
        assert!(result.get("payload").is_none());
        assert!(result.get("error").is_none());

        let retrieval = &result["retrieval"];
        assert_eq!(retrieval["request_id"], json!("daemon-inspect-1"));
        assert_eq!(retrieval["outcome_class"], json!("Degraded"));
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
        assert_eq!(
            retrieval["result"]["packaging_metadata"]["degraded_summary"],
            json!("planner-only inspect envelope; item hydration not implemented")
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
