use crate::rpc::{
    busy_payload, cancelled_payload, JsonRpcRequest, JsonRpcResponse, RuntimeMaintenanceAccepted,
    RuntimeMethodRequest, RuntimeMetrics, RuntimePosture, RuntimeRequest, RuntimeStatus,
};
use anyhow::Context;
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
                            let in_service = state.active_requests.load(Ordering::SeqCst);
                            if queued + in_service > config.max_queue_depth {
                                state.queued_requests.fetch_sub(1, Ordering::SeqCst);
                                let response = JsonRpcResponse::error(
                                    None,
                                    -32001,
                                    "runtime queue is full",
                                    Some(busy_payload(queued + in_service - 1, config.max_queue_depth)),
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
                Some(JsonRpcResponse::error(
                    None,
                    -32002,
                    "request cancelled during shutdown",
                    Some(cancelled_payload()),
                ))
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
                    for _ in 0..polls_budget {
                        if state_clone.is_shutdown() {
                            break;
                        }
                        sleep(step_delay).await;
                    }
                    state_clone.record_maintenance_run(maintenance_id).await;
                });
                JsonRpcResponse::success(
                    request_id,
                    json!(RuntimeMaintenanceAccepted {
                        maintenance_id,
                        polls_budget,
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
                        for _ in 0..polls_budget {
                            if state_clone.is_shutdown() {
                                break;
                            }
                            sleep(step_delay).await;
                        }
                        state_clone.record_maintenance_run(maintenance_id).await;
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

        let busy = send_request(
            &socket_path,
            json!({"jsonrpc":"2.0","method":"sleep","params":{"millis":1},"id":"busy"}),
        )
        .await;
        assert_eq!(busy["error"]["code"], json!(-32001));
        assert_eq!(busy["error"]["data"]["max_queue_depth"], json!(1));

        let slow_response = slow.await.unwrap();
        assert_eq!(slow_response["result"]["slept_ms"], json!(300));

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
}
