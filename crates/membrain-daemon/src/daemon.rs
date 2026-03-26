use crate::mcp::{
    ContextBudgetParams, McpAuditPayload, McpAuditRow, McpInspectPayload, McpResource,
    McpResourceListing, McpResourceReadPayload, McpResponse, McpRetrievalPayload, McpStream,
    McpStreamListing,
};

const DAEMON_RESOURCE_NAMESPACE: &str = "daemon.runtime";
const RUNTIME_STATUS_URI: &str = "membrain://daemon/runtime/status";
const RUNTIME_HEALTH_URI: &str = "membrain://daemon/runtime/health";
const RUNTIME_DOCTOR_URI: &str = "membrain://daemon/runtime/doctor";
const RUNTIME_STREAMS_URI: &str = "membrain://daemon/runtime/streams";
const INSPECT_RESOURCE_URI_TEMPLATE: &str = "membrain://{namespace}/memories/{memory_id}";

fn parse_inspect_resource_uri(uri: &str) -> Option<(String, u64)> {
    let prefix = "membrain://";
    let rest = uri.strip_prefix(prefix)?;
    let (namespace, path) = rest.split_once('/')?;
    let memory_id = path.strip_prefix("memories/")?.parse::<u64>().ok()?;
    Some((namespace.to_string(), memory_id))
}
const SNAPSHOT_RESOURCE_URI_TEMPLATE: &str = "membrain://{namespace}/snapshots/{snapshot_name}";
const MAINTENANCE_STATUS_METHOD: &str = "maintenance.status";
use crate::preflight::{
    evaluate_preflight as evaluate_shared_preflight, preflight_allow as run_shared_preflight_allow,
    to_preflight_explain_response as to_shared_preflight_explain_response,
    to_preflight_outcome as to_shared_preflight_outcome, PreflightExplainResponse,
    PreflightOutcome,
};
use crate::rpc::{
    busy_payload, cancelled_payload, JsonRpcRequest, JsonRpcResponse, RuntimeAuthorityMode,
    RuntimeAvailability, RuntimeDoctorCheck, RuntimeDoctorIndex, RuntimeDoctorReport,
    RuntimeDoctorRunbookHint, RuntimeDoctorSummary, RuntimeMaintenanceAccepted,
    RuntimeMethodRequest, RuntimeMetrics, RuntimePosture, RuntimeRemediation, RuntimeRequest,
    RuntimeStatus,
};
use anyhow::Context;
use tokio::io::{self, AsyncBufReadExt, AsyncWriteExt};
use membrain_core::api::{
    AgentId, BlackboardSnapshotOutput, DeadZonesOutput, FieldPresence, ForkInheritance,
    GoalAbandonOutput, GoalPauseOutput, GoalResumeOutput, GoalStateOutput, HotPathsOutput,
    MergeConflictStrategy, NamespaceId, PassiveObservationInspectSummary, PolicyContext,
    PolicyFilterSummary, RequestContext, RequestId, ResponseContext, ResponseWarning, TaskId,
    WorkspaceId,
};
use membrain_core::brain_store::{ForkConfig, MergeConfig};
use membrain_core::config::RuntimeConfig;
use membrain_core::engine::compression::{CompressionPolicy, CompressionTrigger};
use membrain_core::engine::confidence::{
    ConfidenceEngine, ConfidenceInputs, ConfidenceOutput, ConfidencePolicy,
};
use membrain_core::engine::context_budget::{ContextBudgetRequest, InjectionFormat};
use membrain_core::engine::forgetting::{ForgettingAction, ForgettingPolicy};
use membrain_core::engine::observe::{ObserveConfig, ObserveEngine};
use membrain_core::engine::ranking::{
    adjusted_ranking_input_for_affect, fuse_scores, ranking_profile_for_recall,
    ranking_profile_name_for_recall, RankingInput,
};
use membrain_core::engine::recall::{RecallEngine, RecallRequest, RecallRuntime};
use membrain_core::engine::result::{
    AnsweredFrom, EntryLane, EvidenceRole, QueryByExampleExplain, ResultBuilder, ResultReason,
    RetrievalExplain, RetrievalResultSet,
};
use membrain_core::engine::retrieval_planner::{
    PrimaryCue, QueryByExampleNormalization, RetrievalRequest,
};
use membrain_core::engine::working_state::GoalWorkingState;
use membrain_core::graph::{
    CausalEvidenceAttribution, CausalEvidenceKind, CausalLink, CausalLinkType, EntityId,
    RelationKind,
};
use membrain_core::observability::{
    AuditEventCategory, AuditEventKind, CacheEvalTrace, CacheEventLabel, CacheFamilyLabel,
    CacheLookupOutcome, CacheReasonLabel, GenerationStatusLabel, OutcomeClass, WarmSourceLabel,
};
use membrain_core::policy::{
    PolicyModule, SharingAccessDecision, SharingAccessOutcome, SharingVisibility,
};
use membrain_core::persistence::{load_cli_records, open_hot_db};
use membrain_core::store::audit::{AuditLogEntry, AuditLogFilter};
use membrain_core::store::cache::{
    CacheEvent, CacheFamily, CacheGenerationAnchors, CacheKey, CacheLookupResult, CacheManager,
    GenerationStatus, InvalidationTrigger, PrefetchTrigger, WarmSource,
};
use membrain_core::store::tier2::Tier2DurableItemLayout;
use membrain_core::types::{
    AffectSignals, BlackboardEvidenceHandle, BlackboardState, GoalCheckpoint, GoalLifecycleStatus,
    GoalStackFrame, MemoryId, RawEncodeInput, RawIntakeKind, SessionId,
};
use serde_json::{json, Value};
use std::collections::{HashMap, VecDeque};
use std::hash::{Hash, Hasher};
use std::time::Instant;

fn cache_family_label(family: CacheFamily) -> CacheFamilyLabel {
    match family {
        CacheFamily::Tier1Item => CacheFamilyLabel::Tier1Item,
        CacheFamily::NegativeCache => CacheFamilyLabel::Negative,
        CacheFamily::ResultCache => CacheFamilyLabel::Result,
        CacheFamily::EntityNeighborhood => CacheFamilyLabel::Result,
        CacheFamily::SummaryCache => CacheFamilyLabel::Summary,
        CacheFamily::AnnProbeCache => CacheFamilyLabel::AnnProbe,
        CacheFamily::PrefetchHints
        | CacheFamily::SessionWarmup
        | CacheFamily::GoalConditioned
        | CacheFamily::ColdStartMitigation => CacheFamilyLabel::Tier2Query,
    }
}

fn cache_event_label(event: CacheEvent) -> CacheEventLabel {
    match event {
        CacheEvent::Hit => CacheEventLabel::Hit,
        CacheEvent::Miss => CacheEventLabel::Miss,
        CacheEvent::Bypass | CacheEvent::StaleWarning | CacheEvent::Disabled => {
            CacheEventLabel::Bypass
        }
        CacheEvent::Invalidation => CacheEventLabel::Invalidate,
        CacheEvent::RepairWarmup | CacheEvent::PrefetchDrop | CacheEvent::SessionExpired => {
            CacheEventLabel::Prefetch
        }
    }
}

fn cache_lookup_outcome(result: &CacheLookupResult) -> CacheLookupOutcome {
    match result.event {
        CacheEvent::Hit => CacheLookupOutcome::Hit,
        CacheEvent::Miss => CacheLookupOutcome::Miss,
        CacheEvent::Bypass => CacheLookupOutcome::Bypass,
        CacheEvent::StaleWarning => CacheLookupOutcome::StaleWarning,
        CacheEvent::Disabled => CacheLookupOutcome::Disabled,
        CacheEvent::Invalidation
        | CacheEvent::RepairWarmup
        | CacheEvent::PrefetchDrop
        | CacheEvent::SessionExpired => CacheLookupOutcome::Bypass,
    }
}

fn cache_reason_label(reason: membrain_core::store::cache::CacheReason) -> CacheReasonLabel {
    match reason {
        membrain_core::store::cache::CacheReason::OwnerBoundaryMismatch
        | membrain_core::store::cache::CacheReason::PolicyDenied
        | membrain_core::store::cache::CacheReason::ScopeTooBroad
        | membrain_core::store::cache::CacheReason::PolicyChanged
        | membrain_core::store::cache::CacheReason::RedactionChanged => {
            CacheReasonLabel::PolicyBoundary
        }
        membrain_core::store::cache::CacheReason::NamespaceMismatch
        | membrain_core::store::cache::CacheReason::NamespaceNarrowed
        | membrain_core::store::cache::CacheReason::NamespaceWidened => {
            CacheReasonLabel::NamespaceMismatch
        }
        membrain_core::store::cache::CacheReason::GenerationAnchorMismatch
        | membrain_core::store::cache::CacheReason::VersionMismatch
        | membrain_core::store::cache::CacheReason::SchemaChanged
        | membrain_core::store::cache::CacheReason::IndexChanged
        | membrain_core::store::cache::CacheReason::EmbeddingChanged
        | membrain_core::store::cache::CacheReason::RankingChanged
        | membrain_core::store::cache::CacheReason::RepairIncomplete
        | membrain_core::store::cache::CacheReason::RecordNotPresent => {
            CacheReasonLabel::StaleGeneration
        }
        membrain_core::store::cache::CacheReason::IntentChanged
        | membrain_core::store::cache::CacheReason::BudgetExhausted => CacheReasonLabel::ColdStart,
    }
}

fn warm_source_label(warm_source: WarmSource) -> WarmSourceLabel {
    match warm_source {
        WarmSource::Tier1ItemCache => WarmSourceLabel::Tier1ItemCache,
        WarmSource::NegativeCache => WarmSourceLabel::Tier2QueryCache,
        WarmSource::ResultCache => WarmSourceLabel::ResultCache,
        WarmSource::EntityNeighborhood => WarmSourceLabel::SummaryCache,
        WarmSource::SummaryCache => WarmSourceLabel::SummaryCache,
        WarmSource::AnnProbeCache => WarmSourceLabel::AnnProbeCache,
        WarmSource::PrefetchHints => WarmSourceLabel::PrefetchHints,
        WarmSource::SessionWarmup | WarmSource::GoalConditioned => WarmSourceLabel::Tier2QueryCache,
        WarmSource::ColdStartMitigation => WarmSourceLabel::ColdStartMitigation,
    }
}

fn generation_status_label(status: GenerationStatus) -> GenerationStatusLabel {
    match status {
        GenerationStatus::Valid => GenerationStatusLabel::Valid,
        GenerationStatus::Stale | GenerationStatus::VersionMismatched => {
            GenerationStatusLabel::Stale
        }
        GenerationStatus::Unknown => GenerationStatusLabel::Unknown,
    }
}

fn current_mood_snapshot(state: &RuntimeState, namespace: &NamespaceId) -> Option<AffectSignals> {
    let audit = state
        .audit
        .lock()
        .expect("runtime audit state lock should be available");
    let latest = audit
        .store
        .mood_history(namespace.clone(), None)
        .rows
        .pop()?;
    Some(AffectSignals::new(latest.avg_valence, latest.avg_arousal).clamped())
}

fn cache_eval_trace_from_lookup(
    result: &CacheLookupResult,
    candidates_before: usize,
) -> CacheEvalTrace {
    CacheEvalTrace {
        cache_family: cache_family_label(result.family),
        cache_event: cache_event_label(result.event),
        outcome: cache_lookup_outcome(result),
        cache_reason: result.reason.map(cache_reason_label),
        warm_source: result.warm_source.map(warm_source_label),
        generation_status: generation_status_label(result.generation_status),
        candidates_before,
        candidates_after: result.candidates_after,
        warm_reuse: result.warm_source.is_some(),
    }
}

fn cache_metrics_json(
    cache_traces: Vec<CacheEvalTrace>,
    cold_fallback_count: usize,
    prefetch_added_candidates: Option<usize>,
    prefetch_dropped_count: usize,
    degraded_mode_served: bool,
) -> Result<Value, serde_json::Error> {
    let mut summary = membrain_core::api::CacheMetricsSummary::from_cache_traces(
        cache_traces,
        degraded_mode_served,
    );
    summary.cold_fallback_count = cold_fallback_count;
    summary.prefetch_added_candidates = prefetch_added_candidates;
    summary.prefetch_dropped_count = prefetch_dropped_count;
    serde_json::to_value(summary)
}
use std::os::unix::fs::FileTypeExt;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, AtomicU64, AtomicUsize, Ordering};
use std::sync::{Arc, Mutex as StdMutex};
use tokio::io::BufReader;
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
    mood_congruent: bool,
}

#[derive(Debug, Clone, PartialEq)]
struct RuntimeMemoryRecord {
    layout: Tier2DurableItemLayout,
    confidence_inputs: ConfidenceInputs,
    confidence_output: ConfidenceOutput,
    passive_observation: Option<PassiveObservationInspectSummary>,
    causal_parents: Vec<MemoryId>,
    causal_link_type: Option<CausalLinkType>,
}

#[derive(Debug, Clone)]
struct RuntimeAuditState {
    store: membrain_core::BrainStore,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DaemonRuntimeConfig {
    pub socket_path: PathBuf,
    pub hot_db_path: Option<PathBuf>,
    pub cold_db_path: Option<PathBuf>,
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
            hot_db_path: None,
            cold_db_path: None,
            request_concurrency: 8,
            max_queue_depth: 32,
            maintenance_interval: Duration::from_secs(60),
            maintenance_poll_budget: 4,
            maintenance_step_delay: Duration::from_millis(25),
        }
    }

    pub fn with_db_paths<P1: AsRef<Path>, P2: AsRef<Path>>(mut self, hot_db: P1, cold_db: P2) -> Self {
        self.hot_db_path = Some(hot_db.as_ref().to_path_buf());
        self.cold_db_path = Some(cold_db.as_ref().to_path_buf());
        self
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
    authority_mode: Mutex<RuntimeAuthorityMode>,
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
    memories: StdMutex<HashMap<u64, RuntimeMemoryRecord>>,
    audit: StdMutex<RuntimeAuditState>,
    cache: StdMutex<CacheManager>,
}

impl RuntimeState {
    fn new(config: &DaemonRuntimeConfig) -> Self {
        let state = Self {
            posture: Mutex::new(RuntimePosture::Full),
            authority_mode: Mutex::new(RuntimeAuthorityMode::StdioFacade),
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
            memories: StdMutex::new(HashMap::new()),
            audit: StdMutex::new(RuntimeAuditState {
                store: membrain_core::BrainStore::new(RuntimeConfig::default()),
            }),
            cache: StdMutex::new(CacheManager::new(
                RuntimeConfig::default().cache_per_family_capacity,
                RuntimeConfig::default().prefetch_queue_capacity,
            )),
        };

        if let Some(hot_db_path) = &config.hot_db_path {
            if let Ok(conn) = open_hot_db(hot_db_path) {
                if let Ok(records) = load_cli_records(&conn) {
                    for persisted in records {
                        let namespace = match NamespaceId::new(&persisted.namespace) {
                            Ok(ns) => ns,
                            Err(_) => continue,
                        };
                        let common = crate::rpc::RuntimeCommonFields::default();
                        let record = state.encode_memory(
                            namespace,
                            MemoryId(persisted.memory_id),
                            &persisted.raw_text,
                            None,
                            &common,
                            SharingVisibility::Private,
                        );
                        state.store_encoded_memory(record);
                    }
                }
            }
        }

        state
    }

    async fn status(&self) -> RuntimeStatus {
        let authority_mode = self.authority_mode.lock().await.clone();
        let maintenance_active = matches!(authority_mode, RuntimeAuthorityMode::UnixSocketDaemon)
            && !self.shutdown_requested.load(Ordering::SeqCst);
        let warm_runtime_guarantees = match authority_mode {
            RuntimeAuthorityMode::UnixSocketDaemon => {
                let mut guarantees = vec![
                    "shared_process_state".to_string(),
                    "unix_socket_authority".to_string(),
                ];
                if maintenance_active {
                    guarantees.push("background_maintenance_loop".to_string());
                }
                guarantees
            }
            RuntimeAuthorityMode::StdioFacade => vec![
                "single_process_request_state".to_string(),
                "stdio_transport".to_string(),
            ],
        };

        RuntimeStatus {
            posture: self.posture.lock().await.clone(),
            authority_mode: authority_mode.clone(),
            authoritative_runtime: matches!(authority_mode, RuntimeAuthorityMode::UnixSocketDaemon),
            maintenance_active,
            warm_runtime_guarantees,
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

    async fn set_authority_mode(&self, authority_mode: RuntimeAuthorityMode) {
        *self.authority_mode.lock().await = authority_mode;
    }

    async fn doctor_report(&self) -> RuntimeDoctorReport {
        use membrain_core::api::{
            AvailabilityPosture, AvailabilityReason, AvailabilitySummary, ErrorKind,
            RemediationStep,
        };
        use membrain_core::engine::maintenance::{
            MaintenanceController, MaintenanceJobHandle, MaintenanceJobState,
        };
        use membrain_core::engine::repair::IndexRepairEntrypoint;
        use membrain_core::health::{BrainHealthInputs, FeatureAvailabilityEntry};
        use membrain_core::index::{IndexApi, IndexModule};

        let status = self.status().await;
        let posture = status.posture.clone();
        let authority_mode = status.authority_mode.clone();
        let authoritative_runtime = status.authoritative_runtime;
        let maintenance_active = status.maintenance_active;
        let warm_runtime_guarantees = status.warm_runtime_guarantees.clone();
        let degraded_reasons = status.degraded_reasons.clone();
        let metrics = status.metrics.clone();
        let posture_status = match posture {
            RuntimePosture::Full => "ok",
            RuntimePosture::Degraded => "warn",
            RuntimePosture::ReadOnly | RuntimePosture::Offline => "fail",
        };
        let cache_usable = !matches!(posture, RuntimePosture::Offline);
        let mut warnings = if matches!(posture, RuntimePosture::Full) {
            Vec::new()
        } else {
            vec!["operator_review_recommended"]
        };
        if !authoritative_runtime {
            warnings.push("stdio_mode_has_no_background_maintenance_loop");
        }

        let store = membrain_core::BrainStore::new(RuntimeConfig::default());
        let repair_engine = store.repair_engine();
        let namespace =
            NamespaceId::new("doctor.system").expect("doctor namespace should be valid");
        let mut repair_handle = MaintenanceJobHandle::new(
            repair_engine
                .create_index_rebuild(namespace.clone(), IndexRepairEntrypoint::VerifyOnly),
            8,
        );
        repair_handle.start();
        let mut repair_reports = Vec::new();
        let mut health_repair_summary = None;
        loop {
            let snapshot = repair_handle.poll();
            match snapshot.state {
                MaintenanceJobState::Completed(summary) => {
                    repair_reports = summary
                        .results
                        .iter()
                        .map(|result| {
                            let report = summary
                                .operator_reports
                                .iter()
                                .find(|report| report.target == result.target)
                                .expect(
                                    "repair operator report should exist for each doctor result",
                                );
                            let plan = result.rebuild_entrypoint.and_then(|entrypoint| {
                                repair_engine.plan_index_rebuild(result.target, entrypoint)
                            });
                            let artifact =
                                summary.verification_artifacts.get(&result.target).expect(
                                    "verification artifact should exist for each doctor result",
                                );
                            crate::rpc::RuntimeRepairReport {
                                target: result.target.as_str(),
                                status: result.status.as_str(),
                                verification_passed: result.verification_passed,
                                rebuild_entrypoint: result
                                    .rebuild_entrypoint
                                    .map(IndexRepairEntrypoint::as_str),
                                rebuilt_outputs: result.rebuilt_outputs.clone(),
                                durable_sources: plan
                                    .as_ref()
                                    .map(|plan| plan.durable_sources.clone())
                                    .unwrap_or_default(),
                                verification_artifact_name: artifact.artifact_name,
                                parity_check: artifact.parity_check,
                                authoritative_rows: artifact.authoritative_rows,
                                derived_rows: artifact.derived_rows,
                                authoritative_generation: artifact.authoritative_generation,
                                derived_generation: artifact.derived_generation,
                                affected_item_count: report.affected_item_count,
                                error_count: report.error_count,
                                rebuild_duration_ms: report.rebuild_duration_ms,
                                rollback_state: report.rollback_state,
                                degraded_mode: report.degraded_mode.map(|mode| mode.as_str()),
                                rollback_trigger: report
                                    .rollback_trigger
                                    .map(|trigger| trigger.as_str()),
                                remediation_steps: report
                                    .remediation_steps
                                    .iter()
                                    .map(|step| step.as_str())
                                    .collect(),
                                queue_depth_before: report.queue_depth_before,
                                queue_depth_after: report.queue_depth_after,
                            }
                        })
                        .collect();
                    health_repair_summary = Some(summary);
                    break;
                }
                MaintenanceJobState::Running { .. } => continue,
                _ => break,
            }
        }

        use membrain_core::engine::lease::{
            LeaseMetadata, LeasePolicy, LeaseScanItem, LeaseScanner,
        };

        let index_reports = IndexModule.health_reports();
        let availability_reasons = {
            let mut reasons = degraded_reasons
                .iter()
                .map(|reason| match reason.as_str() {
                    "graph_unavailable" => AvailabilityReason::GraphUnavailable,
                    "index_bypassed" => AvailabilityReason::IndexBypassed,
                    "cache_invalidated" => AvailabilityReason::CacheInvalidated,
                    "repair_rollback_required" => AvailabilityReason::RepairRollbackRequired,
                    "repair_rollback_in_progress" => AvailabilityReason::RepairRollbackInProgress,
                    "authoritative_input_unreadable" => {
                        AvailabilityReason::AuthoritativeInputUnreadable
                    }
                    _ => AvailabilityReason::RepairInFlight,
                })
                .collect::<Vec<_>>();
            if !matches!(posture, RuntimePosture::Full) && reasons.is_empty() {
                reasons.push(AvailabilityReason::RepairInFlight);
            }
            reasons
        };
        let corruption_reason_present = availability_reasons.iter().any(|reason| {
            matches!(
                reason,
                AvailabilityReason::AuthoritativeInputUnreadable
                    | AvailabilityReason::RepairRollbackRequired
                    | AvailabilityReason::RepairRollbackInProgress
            )
        });
        let availability = match posture {
            RuntimePosture::Full => None,
            RuntimePosture::Degraded => Some(AvailabilitySummary::degraded(
                vec!["doctor", "health", "audit"],
                vec!["encode", "maintenance"],
                availability_reasons.clone(),
                vec![RemediationStep::CheckHealth, RemediationStep::RunRepair],
            )),
            RuntimePosture::ReadOnly => Some(AvailabilitySummary::new(
                AvailabilityPosture::ReadOnly,
                vec!["doctor", "health", "audit", "inspect"],
                vec!["maintenance_preview_only"],
                availability_reasons.clone(),
                vec![RemediationStep::CheckHealth, RemediationStep::RunRepair],
            )),
            RuntimePosture::Offline => Some(AvailabilitySummary::new(
                AvailabilityPosture::Offline,
                vec!["doctor", "health", "audit"],
                Vec::new(),
                availability_reasons.clone(),
                vec![RemediationStep::CheckHealth, RemediationStep::RunRepair],
            )),
        };
        let runtime_availability = availability
            .as_ref()
            .map(|availability| RuntimeAvailability {
                posture: availability.posture.as_str(),
                query_capabilities: availability.query_capabilities.clone(),
                mutation_capabilities: availability.mutation_capabilities.clone(),
                degraded_reasons: availability.reason_names(),
                recovery_conditions: availability.recovery_condition_names(),
            });
        let error_kind = if matches!(posture, RuntimePosture::ReadOnly | RuntimePosture::Offline)
            || corruption_reason_present
        {
            Some(ErrorKind::CorruptionFailure.as_str())
        } else {
            None
        };
        let remediation = runtime_availability
            .as_ref()
            .map(|availability| RuntimeRemediation {
                summary: format!(
                    "runtime posture {} requires operator follow-up before normal service resumes",
                    availability.posture
                ),
                next_steps: availability
                    .recovery_conditions
                    .iter()
                    .map(|step| (*step).to_string())
                    .collect(),
            });

        let (
            attention_namespaces,
            total_recalls,
            total_encodes,
            current_tick,
            affect_history_rows,
            latest_affect_snapshot,
            latest_affect_tick,
        ) = {
            let audit = self
                .audit
                .lock()
                .expect("runtime audit state lock should be available");
            let entries = audit.store.audit_log().entries();
            let history = audit.store.mood_history(
                NamespaceId::new("default").expect("default namespace should parse"),
                None,
            );
            let latest = history.rows.last().cloned();
            (
                audit.store.attention_namespaces(),
                entries
                    .iter()
                    .filter(|entry| matches!(entry.kind, AuditEventKind::RecallServed))
                    .count() as u64,
                entries
                    .iter()
                    .filter(|entry| matches!(entry.kind, AuditEventKind::EncodeAccepted))
                    .count() as u64,
                audit.store.current_tick(),
                history.total_rows,
                latest
                    .as_ref()
                    .map(|row| (row.avg_valence, row.avg_arousal)),
                latest.map(|row| row.tick_end.unwrap_or(row.tick_start)),
            )
        };
        let previous_prefetch_queue_depth = self
            .cache
            .lock()
            .expect("runtime cache lock should be available")
            .prefetch
            .queue_depth();
        let runtime_memory_records = self
            .memories
            .lock()
            .expect("runtime memory registry lock should be available")
            .values()
            .cloned()
            .collect::<Vec<_>>();
        let hot_memories = runtime_memory_records.len();
        let hot_capacity = 100usize;
        let cold_memories = 0usize;
        let confidence_sum = runtime_memory_records
            .iter()
            .map(|record| f64::from(record.confidence_output.confidence) / 1000.0)
            .sum::<f64>();
        let avg_confidence = if hot_memories > 0 {
            confidence_sum / hot_memories as f64
        } else {
            0.0
        };
        let strength_sum = runtime_memory_records
            .iter()
            .map(|record| {
                f64::from(record.confidence_output.confidence.saturating_sub(80)) / 1000.0
            })
            .sum::<f64>();
        let avg_strength = if hot_memories > 0 {
            strength_sum / hot_memories as f64
        } else {
            0.0
        };
        let low_confidence_count = runtime_memory_records
            .iter()
            .filter(|record| record.confidence_output.confidence < 500)
            .count();
        let landmark_count = runtime_memory_records
            .iter()
            .filter(|record| record.layout.metadata.landmark.is_landmark)
            .count();
        let uncertain_count = runtime_memory_records
            .iter()
            .filter(|record| record.confidence_output.confidence < 700)
            .count();
        let top_engrams = runtime_memory_records
            .iter()
            .filter_map(|record| record.layout.metadata.landmark.landmark_label.clone())
            .fold(std::collections::HashMap::<String, usize>::new(), |mut acc, label| {
                *acc.entry(label).or_insert(0) += 1;
                acc
            })
            .into_iter()
            .collect::<Vec<_>>();
        let total_engrams = landmark_count;
        let avg_cluster_size = if landmark_count > 0 {
            hot_memories as f64 / landmark_count as f64
        } else {
            0.0
        };
        let health = {
            let cache = self
                .cache
                .lock()
                .expect("runtime cache lock should be available");
            membrain_core::health::BrainHealthReport::from_inputs(
                BrainHealthInputs {
                    hot_memories,
                    hot_capacity,
                    cold_memories,
                    avg_strength: avg_strength as f32,
                    avg_confidence: avg_confidence as f32,
                    low_confidence_count,
                    decay_rate: 0.012,
                    archive_count: 0,
                    total_engrams,
                    avg_cluster_size: avg_cluster_size as f32,
                    top_engrams,
                    landmark_count,
                    unresolved_conflicts: 0,
                    uncertain_count,
                    dream_links_total: 0,
                    last_dream_tick: None,
                    affect_history_rows,
                    latest_affect_snapshot,
                    latest_affect_tick,
                    attention_namespaces,
                    total_recalls,
                    total_encodes,
                    current_tick,
                    daemon_uptime_ticks: current_tick,
                    index_reports: index_reports.clone(),
                    availability: availability.clone(),
                    feature_availability: vec![
                        FeatureAvailabilityEntry {
                            feature: "health".to_string(),
                            posture: membrain_core::api::AvailabilityPosture::Full,
                            note: Some("daemon_doctor_embeds_brain_health_report".to_string()),
                        },
                        FeatureAvailabilityEntry {
                            feature: "runtime_authority".to_string(),
                            posture: if authoritative_runtime {
                                membrain_core::api::AvailabilityPosture::Full
                            } else {
                                membrain_core::api::AvailabilityPosture::Degraded
                            },
                            note: Some(format!(
                                "mode={} authoritative_runtime={} maintenance_active={} warm_runtime_guarantees={}",
                                authority_mode.as_str(),
                                authoritative_runtime,
                                maintenance_active,
                                warm_runtime_guarantees.join(",")
                            )),
                        },
                    ],
                    previous_total_recalls: Some(total_recalls.saturating_sub(1)),
                    previous_total_encodes: Some(total_encodes.saturating_sub(1)),
                    previous_repair_queue_depth: Some(0),
                    previous_hot_memories: Some(hot_memories.saturating_sub(1)),
                    previous_low_confidence_count: Some(low_confidence_count.saturating_sub(1)),
                    previous_unresolved_conflicts: Some(0),
                    previous_uncertain_count: Some(uncertain_count.saturating_sub(1)),
                    previous_cache_hit_count: Some(0),
                    previous_cache_miss_count: Some(0),
                    previous_cache_bypass_count: Some(0),
                    previous_prefetch_queue_depth: Some(previous_prefetch_queue_depth),
                    previous_prefetch_drop_count: Some(0),
                    previous_index_stale_count: Some(1),
                    previous_index_missing_count: Some(0),
                    previous_index_repair_backlog_total: Some(1),
                    previous_availability_posture: Some(
                        membrain_core::api::AvailabilityPosture::Full,
                    ),
                },
                &cache,
                health_repair_summary.as_ref(),
            )
        };

        let repair_engine_component = repair_engine.component_name();
        let schema_related_repairs = repair_reports
            .iter()
            .filter(|report| {
                matches!(
                    report.target,
                    "hot_store_consistency" | "payload_integrity" | "contradiction_consistency"
                )
            })
            .collect::<Vec<_>>();
        let index_related_repairs = repair_reports
            .iter()
            .filter(|report| {
                matches!(
                    report.target,
                    "lexical_index"
                        | "metadata_index"
                        | "semantic_hot_index"
                        | "semantic_cold_index"
                        | "engram_index"
                )
            })
            .collect::<Vec<_>>();
        let graph_related_repairs = repair_reports
            .iter()
            .filter(|report| report.target == "graph_consistency")
            .collect::<Vec<_>>();
        let cache_related_repairs = repair_reports
            .iter()
            .filter(|report| report.target == "cache_warm_state")
            .collect::<Vec<_>>();
        let index_state_warn = index_reports
            .iter()
            .any(|report| !report.health.is_usable())
            || degraded_reasons
                .iter()
                .any(|reason| reason == "index_bypassed")
            || index_related_repairs
                .iter()
                .any(|report| report.status != "healthy");
        let graph_state_warn = degraded_reasons
            .iter()
            .any(|reason| reason == "graph_unavailable")
            || graph_related_repairs
                .iter()
                .any(|report| !report.verification_passed)
            || health.repair.as_ref().is_some_and(|summary| {
                summary.state == membrain_core::health::SubsystemHealthState::Blocked
            });
        let cache_state_warn = health.cache.state
            != membrain_core::health::SubsystemHealthState::Healthy
            || degraded_reasons
                .iter()
                .any(|reason| reason == "cache_invalidated")
            || cache_related_repairs
                .iter()
                .any(|report| !report.verification_passed);
        let schema_state_warn = schema_related_repairs
            .iter()
            .any(|report| !report.verification_passed)
            || error_kind.is_some();

        let schema_entry_count = schema_related_repairs.len().max(1);
        let index_entry_count = index_reports.len();
        let graph_entry_count = health.total_engrams.max(1);
        let cache_entry_count = health.cache.family_status.len();
        let graph_generation = graph_related_repairs
            .first()
            .map(|report| report.derived_generation)
            .unwrap_or("durable.v1");
        let cache_generation = cache_related_repairs
            .first()
            .map(|report| report.derived_generation)
            .unwrap_or("cache.v1");
        let indexes = vec![
            RuntimeDoctorIndex {
                family: "schema",
                health: if schema_state_warn { "warn" } else { "ok" },
                usable: error_kind.is_none(),
                entry_count: schema_entry_count,
                generation: "schema.v1",
            },
            RuntimeDoctorIndex {
                family: "index",
                health: if index_state_warn { "warn" } else { "ok" },
                usable: !index_reports
                    .iter()
                    .any(|report| report.health.as_str() == "missing"),
                entry_count: index_entry_count,
                generation: index_reports
                    .iter()
                    .find(|report| !report.generation.is_empty())
                    .map(|report| report.generation)
                    .unwrap_or("durable.v1"),
            },
            RuntimeDoctorIndex {
                family: "graph",
                health: if graph_state_warn { "warn" } else { "ok" },
                usable: !degraded_reasons
                    .iter()
                    .any(|reason| reason == "graph_unavailable"),
                entry_count: graph_entry_count,
                generation: graph_generation,
            },
            RuntimeDoctorIndex {
                family: "cache",
                health: if cache_state_warn { "warn" } else { "ok" },
                usable: cache_usable,
                entry_count: cache_entry_count,
                generation: cache_generation,
            },
        ];
        let lease_items = if matches!(posture, RuntimePosture::Full) {
            vec![
                LeaseScanItem {
                    memory_id: MemoryId(11),
                    lease: LeaseMetadata::new(LeasePolicy::Normal, 0),
                    action_critical: true,
                },
                LeaseScanItem {
                    memory_id: MemoryId(12),
                    lease: LeaseMetadata::new(LeasePolicy::Volatile, 20),
                    action_critical: true,
                },
                LeaseScanItem {
                    memory_id: MemoryId(13),
                    lease: LeaseMetadata::new(LeasePolicy::Durable, 0),
                    action_critical: false,
                },
            ]
        } else {
            vec![
                LeaseScanItem {
                    memory_id: MemoryId(11),
                    lease: LeaseMetadata::new(LeasePolicy::Normal, 0),
                    action_critical: true,
                },
                LeaseScanItem {
                    memory_id: MemoryId(12),
                    lease: LeaseMetadata::new(LeasePolicy::Volatile, 0),
                    action_critical: true,
                },
                LeaseScanItem {
                    memory_id: MemoryId(13),
                    lease: LeaseMetadata::new(LeasePolicy::Durable, 1800),
                    action_critical: false,
                },
            ]
        };
        let lease_report = LeaseScanner.scan(
            &lease_items,
            if matches!(posture, RuntimePosture::Full) {
                40
            } else {
                400
            },
            3,
        );
        let index_issue_count = indexes.iter().filter(|index| index.health != "ok").count();
        let failing_repair_targets = repair_reports
            .iter()
            .filter(|report| report.status != "healthy")
            .count();
        let stale_action_critical = lease_report.recheck_required_items > 0;
        let mut warnings = warnings;
        if stale_action_critical {
            warnings.push("stale_action_critical_recheck_required");
        }
        if error_kind.is_some() {
            warnings.push("safe_serving_impaired");
        }

        let checks = vec![
            RuntimeDoctorCheck {
                name: "schema_consistency",
                surface_kind: "schema",
                status: if schema_state_warn { "warn" } else { "ok" },
                severity: if schema_state_warn { "warning" } else { "info" },
                affected_scope: format!(
                    "schema_generation=schema.v1 integrity_targets={schema_entry_count}"
                ),
                degraded_impact: schema_state_warn.then(|| {
                    "authoritative schema integrity needs review before trusting derived state"
                        .to_string()
                }),
                remediation: schema_state_warn.then(|| RuntimeRemediation {
                    summary: "inspect authoritative store integrity before attempting repair"
                        .to_string(),
                    next_steps: vec!["check_health".to_string(), "run_repair".to_string()],
                }),
            },
            RuntimeDoctorCheck {
                name: "index_state",
                surface_kind: "derived_index",
                status: if index_issue_count > 0 { "warn" } else { "ok" },
                severity: if index_issue_count > 0 { "warning" } else { "info" },
                affected_scope: format!("index_families={index_entry_count}"),
                degraded_impact: (index_issue_count > 0).then(|| {
                    format!(
                        "{index_issue_count} derived families show drift, rebuild need, or degraded cache/index state"
                    )
                }),
                remediation: (index_issue_count > 0).then(|| RuntimeRemediation {
                    summary: "repair derived indexes and verify parity against durable truth"
                        .to_string(),
                    next_steps: vec!["check_health".to_string(), "run_repair".to_string()],
                }),
            },
            RuntimeDoctorCheck {
                name: "graph_health",
                surface_kind: "graph",
                status: if graph_state_warn { "warn" } else { "ok" },
                severity: if graph_state_warn { "warning" } else { "info" },
                affected_scope: format!("graph_entries={graph_entry_count}"),
                degraded_impact: graph_state_warn.then(|| {
                    let failed_targets = graph_related_repairs
                        .iter()
                        .filter(|report| !report.verification_passed)
                        .count();
                    format!(
                        "graph consistency degraded; failed_targets={failed_targets} graph_unavailable={} ",
                        degraded_reasons.iter().any(|reason| reason == "graph_unavailable")
                    )
                    .trim()
                    .to_string()
                }),
                remediation: graph_state_warn.then(|| RuntimeRemediation {
                    summary: "rebuild graph projections from durable truth before trusting traversal state"
                        .to_string(),
                    next_steps: vec!["check_health".to_string(), "run_repair".to_string()],
                }),
            },
            RuntimeDoctorCheck {
                name: "serving_posture",
                surface_kind: "availability",
                status: posture_status,
                severity: match posture_status {
                    "fail" => "critical",
                    "warn" => "warning",
                    _ => "info",
                },
                affected_scope: posture.as_str().to_string(),
                degraded_impact: runtime_availability.as_ref().map(|availability| {
                    format!(
                        "query_capabilities={} mutation_capabilities={} cache_state={}",
                        availability.query_capabilities.join(","),
                        availability.mutation_capabilities.join(","),
                        indexes[3].health,
                    )
                }),
                remediation: remediation.clone(),
            },
            RuntimeDoctorCheck {
                name: "runtime_authority",
                surface_kind: "runtime_mode",
                status: if authoritative_runtime { "ok" } else { "warn" },
                severity: if authoritative_runtime { "info" } else { "warning" },
                affected_scope: authority_mode.as_str().to_string(),
                degraded_impact: Some(format!(
                    "authoritative_runtime={} maintenance_active={} warm_runtime_guarantees={}",
                    authoritative_runtime,
                    maintenance_active,
                    warm_runtime_guarantees.join(",")
                )),
                remediation: (!authoritative_runtime).then(|| RuntimeRemediation {
                    summary: "start the unix-socket daemon when you need authoritative warm-runtime guarantees"
                        .to_string(),
                    next_steps: vec!["run_daemon".to_string(), "check_health".to_string()],
                }),
            },
            RuntimeDoctorCheck {
                name: "lease_freshness",
                surface_kind: "freshness",
                status: if stale_action_critical { "warn" } else { "ok" },
                severity: if stale_action_critical { "warning" } else { "info" },
                affected_scope: format!("scanned_items={}", lease_report.scanned_items),
                degraded_impact: stale_action_critical.then(|| {
                    format!(
                        "{} action-critical items reached recheck_required; {} volatile items would be withheld",
                        lease_report.recheck_required_items, lease_report.withheld_items
                    )
                }),
                remediation: stale_action_critical.then(|| RuntimeRemediation {
                    summary: "inspect stale action-critical evidence before following action guidance"
                        .to_string(),
                    next_steps: vec!["inspect_state".to_string(), "check_health".to_string()],
                }),
            },
        ];
        let status = if stale_action_critical && posture_status == "ok" {
            "warn"
        } else {
            posture_status
        };
        let summary = RuntimeDoctorSummary {
            ok_checks: checks.iter().filter(|check| check.status == "ok").count(),
            warn_checks: checks.iter().filter(|check| check.status == "warn").count(),
            fail_checks: checks.iter().filter(|check| check.status == "fail").count(),
        };
        let mut runbook_hints = Vec::new();
        if index_issue_count > 0
            || degraded_reasons
                .iter()
                .any(|reason| reason == "index_bypassed")
        {
            runbook_hints.push(RuntimeDoctorRunbookHint {
                runbook_id: "index_rebuild_operations",
                source_doc: "docs/OPERATIONS.md",
                section: "## 5. Index Rebuild Operations",
                reason: "derived index serving is degraded or bypassed; rebuild and parity proof should be reviewed"
                    .to_string(),
            });
            runbook_hints.push(RuntimeDoctorRunbookHint {
                runbook_id: "tier2_index_drift",
                source_doc: "docs/FAILURE_PLAYBOOK.md",
                section: "## 2. Tier2 index drift",
                reason: "index-related degraded mode should follow the canonical drift containment matrix"
                    .to_string(),
            });
        }
        if failing_repair_targets > 0
            || degraded_reasons.iter().any(|reason| {
                matches!(
                    reason.as_str(),
                    "repair_in_flight" | "repair_rollback_required" | "repair_rollback_in_progress"
                )
            })
        {
            runbook_hints.push(RuntimeDoctorRunbookHint {
                runbook_id: "repair_backlog_growth",
                source_doc: "docs/FAILURE_PLAYBOOK.md",
                section: "## 9. Repair backlog growth",
                reason: "repair follow-up is still visible in operator diagnostics and should be drained before declaring recovery"
                    .to_string(),
            });
        }
        if error_kind.is_some() || stale_action_critical {
            runbook_hints.push(RuntimeDoctorRunbookHint {
                runbook_id: "incident_response",
                source_doc: "docs/OPERATIONS.md",
                section: "## 7. Incident Response",
                reason: if error_kind.is_some() {
                    "safe serving is impaired and incident-response containment should be followed"
                        .to_string()
                } else {
                    "stale action-critical evidence requires explicit recheck/withhold review before acting"
                        .to_string()
                },
            });
        }

        RuntimeDoctorReport {
            status,
            action: "doctor",
            posture,
            degraded_reasons,
            metrics,
            summary,
            indexes,
            repair_engine_component,
            repair_reports,
            checks,
            runbook_hints,
            warnings,
            error_kind,
            retryable: false,
            remediation,
            availability: runtime_availability,
            health: serde_json::to_value(health)
                .expect("daemon doctor health report should serialize"),
        }
    }

    async fn health_report(&self) -> serde_json::Value {
        self.doctor_report().await.health
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

    fn encode_memory(
        &self,
        namespace: NamespaceId,
        memory_id: MemoryId,
        content: &str,
        emotional_annotations: Option<&serde_json::Value>,
        common: &crate::rpc::RuntimeCommonFields,
        visibility: SharingVisibility,
    ) -> RuntimeMemoryRecord {
        let store = membrain_core::BrainStore::new(RuntimeConfig::default());
        let affect_signals = emotional_annotations.and_then(Self::parse_affect_signals);
        let mut input = RawEncodeInput::new(RawIntakeKind::Event, content);
        if let Some(affect_signals) = affect_signals {
            input = input.with_affect_signals(affect_signals);
        }
        let mut prepared = store.encode_engine().prepare_fast_path(input);
        prepared.normalized.sharing.visibility = visibility;
        if let Some(workspace_id) = common.workspace_id.as_deref() {
            prepared.normalized.sharing.workspace_id = Some(WorkspaceId::new(workspace_id));
        }
        if let Some(agent_id) = common.agent_id.as_deref() {
            prepared.normalized.sharing.agent_id = Some(AgentId::new(agent_id));
        }
        let confidence_inputs = Self::build_confidence_inputs(
            &prepared.normalized,
            prepared.provisional_salience,
            None,
            memory_id,
            0,
            None,
        );
        let confidence_output =
            ConfidenceEngine.compute(&confidence_inputs, &ConfidencePolicy::default());
        let layout = store.tier2_store().layout_item(
            namespace,
            memory_id,
            SessionId(1),
            prepared.fingerprint,
            &prepared.normalized,
            Some(confidence_inputs.clone()),
            Some(confidence_output.clone()),
        );
        RuntimeMemoryRecord {
            layout,
            confidence_inputs,
            confidence_output,
            passive_observation: None,
            causal_parents: Vec::new(),
            causal_link_type: None,
        }
    }

    fn parse_affect_signals(value: &serde_json::Value) -> Option<AffectSignals> {
        let object = value.as_object()?;
        let valence = object.get("valence")?.as_f64()? as f32;
        let arousal = object.get("arousal")?.as_f64()? as f32;
        Some(AffectSignals::new(valence, arousal).clamped())
    }

    fn store_encoded_memory(&self, record: RuntimeMemoryRecord) {
        let namespace = record.layout.metadata.namespace.clone();
        let memory_id = record.layout.metadata.memory_id;
        let confidence = record.confidence_output.confidence;
        let strength = confidence.saturating_sub(80);
        let note = format!(
            "encoded {} into {}",
            record.layout.metadata.memory_type.as_str(),
            record.layout.metadata.route_family.as_str()
        );
        let mut cache = self
            .cache
            .lock()
            .expect("runtime cache lock should be available");
        let _ = cache.handle_invalidation(InvalidationTrigger::MemoryMutation, &namespace);
        let tier1_key = Self::cache_key_for_memory(CacheFamily::Tier1Item, &record);
        self.memories
            .lock()
            .expect("runtime memory registry lock should be available")
            .insert(record.layout.metadata.memory_id.0, record.clone());
        if let Some(store) = cache.store_for(CacheFamily::Tier1Item) {
            let _ = store.admit(
                tier1_key,
                vec![record.layout.metadata.memory_id],
                Default::default(),
            );
        }
        self.append_runtime_audit_entry(
            namespace,
            AuditLogEntry::new(
                AuditEventCategory::Encode,
                AuditEventKind::EncodeAccepted,
                record.layout.metadata.namespace.clone(),
                "daemon_encode",
                note,
            )
            .with_memory_id(memory_id)
            .with_tick(self.runtime_current_tick(&record.layout.metadata.namespace))
            .with_strength_delta(None, Some(strength))
            .with_confidence_delta(None, Some(confidence)),
        );
    }

    fn build_confidence_inputs(
        normalized: &membrain_core::types::NormalizedMemoryEnvelope,
        provisional_salience: u16,
        passive_observation: Option<&PassiveObservationInspectSummary>,
        memory_id: MemoryId,
        causal_parent_count: u32,
        causal_link_type: Option<CausalLinkType>,
    ) -> ConfidenceInputs {
        let recall_count = (memory_id.0 % 6) as u32;
        let observation_boost = passive_observation
            .filter(|inspect| inspect.captured_as_observation)
            .map(|_| 75)
            .unwrap_or(0);
        let authoritativeness = provisional_salience
            .saturating_add(100)
            .saturating_add(observation_boost)
            .min(1000);
        let reconsolidation_count = match causal_link_type {
            Some(CausalLinkType::Reconsolidated) => 1,
            _ => (normalized.payload_size_bytes / 512).min(16) as u32,
        };

        ConfidenceInputs {
            corroboration_count: u32::from((provisional_salience / 250).min(4)),
            reconsolidation_count,
            ticks_since_last_access: u64::from(memory_id.0.saturating_mul(17)),
            age_ticks: u64::from(memory_id.0.saturating_mul(23)),
            resolution_state: membrain_core::engine::contradiction::ResolutionState::None,
            conflict_score: 0,
            causal_parent_count,
            authoritativeness,
            recall_count,
        }
    }

    fn append_runtime_audit_entry(
        &self,
        _namespace: NamespaceId,
        entry: AuditLogEntry,
    ) -> membrain_core::store::audit::AuditLogEntry {
        let mut audit = self
            .audit
            .lock()
            .expect("runtime audit state lock should be available");
        let tick = audit.store.current_tick().saturating_add(1);
        let memory_id = entry.memory_id.unwrap_or(MemoryId(0));
        let entry = entry.with_tick(tick).with_memory_id(memory_id);
        audit.store.append_audit_entry(entry)
    }

    fn runtime_current_tick(&self, _namespace: &NamespaceId) -> u64 {
        self.audit
            .lock()
            .expect("runtime audit state lock should be available")
            .store
            .current_tick()
            .saturating_add(1)
    }

    fn update_visibility(
        &self,
        memory_id: MemoryId,
        visibility: SharingVisibility,
        _target_namespace: Option<&NamespaceId>,
    ) -> Option<Tier2DurableItemLayout> {
        let mut memories = self
            .memories
            .lock()
            .expect("runtime memory registry lock should be available");
        let record = memories.get_mut(&memory_id.0)?;
        record.layout.metadata.visibility = visibility;
        Some(record.layout.clone())
    }

    fn memory_record(&self, memory_id: MemoryId) -> Option<RuntimeMemoryRecord> {
        self.memories
            .lock()
            .expect("runtime memory registry lock should be available")
            .get(&memory_id.0)
            .cloned()
    }

    fn current_cache_generations() -> CacheGenerationAnchors {
        CacheGenerationAnchors::default()
    }

    fn stable_string_key(value: &str) -> u64 {
        let mut hasher = std::collections::hash_map::DefaultHasher::new();
        value.hash(&mut hasher);
        hasher.finish()
    }

    fn request_shape_hash(
        request: &RecallRequest,
        normalized: Option<&QueryByExampleNormalization>,
        result_budget: usize,
        output_mode_label: Option<&str>,
        method_name: &str,
    ) -> u64 {
        let mut hasher = std::collections::hash_map::DefaultHasher::new();
        request.exact_memory_id.hash(&mut hasher);
        request.session_id.hash(&mut hasher);
        request.small_lookup.hash(&mut hasher);
        request.graph_expansion.hash(&mut hasher);
        request.predictive_preroll.hash(&mut hasher);
        result_budget.hash(&mut hasher);
        output_mode_label.unwrap_or("").hash(&mut hasher);
        method_name.hash(&mut hasher);
        if let Some(normalized) = normalized {
            normalized.normalized_query_text.hash(&mut hasher);
            normalized.primary_cue.as_str().hash(&mut hasher);
            for descriptor in normalized.seed_descriptors() {
                descriptor.hash(&mut hasher);
            }
        }
        hasher.finish()
    }

    fn request_owner_key(common: &crate::rpc::RuntimeCommonFields) -> Option<u64> {
        common
            .session_id
            .as_deref()
            .and_then(|value| value.parse::<u64>().ok())
            .or_else(|| common.task_id.as_deref().map(Self::stable_string_key))
    }

    fn workspace_key(common: &crate::rpc::RuntimeCommonFields) -> Option<u64> {
        common.workspace_id.as_deref().map(Self::stable_string_key)
    }

    fn cache_key_for_request(
        family: CacheFamily,
        namespace: &NamespaceId,
        common: &crate::rpc::RuntimeCommonFields,
        item_key: u64,
        request_shape_hash: Option<u64>,
    ) -> CacheKey {
        CacheKey {
            family,
            namespace: namespace.clone(),
            workspace_key: Self::workspace_key(common),
            owner_key: match family {
                CacheFamily::PrefetchHints
                | CacheFamily::SessionWarmup
                | CacheFamily::GoalConditioned => Self::request_owner_key(common),
                _ => None,
            },
            request_shape_hash,
            item_key,
            generations: Self::current_cache_generations(),
        }
    }

    fn cache_key_for_memory(family: CacheFamily, record: &RuntimeMemoryRecord) -> CacheKey {
        CacheKey {
            family,
            namespace: record.layout.metadata.namespace.clone(),
            workspace_key: record
                .layout
                .metadata
                .workspace_id
                .as_ref()
                .map(|workspace_id| Self::stable_string_key(workspace_id.as_str())),
            owner_key: match family {
                CacheFamily::PrefetchHints
                | CacheFamily::SessionWarmup
                | CacheFamily::GoalConditioned => Some(record.layout.metadata.session_id.0),
                _ => None,
            },
            request_shape_hash: None,
            item_key: record.layout.metadata.memory_id.0,
            generations: Self::current_cache_generations(),
        }
    }

    fn cache_lookup_result_for_prefetch(
        hint_memory_ids: Vec<MemoryId>,
        candidates_before: usize,
    ) -> CacheLookupResult {
        CacheLookupResult {
            family: CacheFamily::PrefetchHints,
            event: CacheEvent::Hit,
            reason: None,
            warm_source: Some(membrain_core::store::cache::WarmSource::PrefetchHints),
            generation_status: GenerationStatus::Valid,
            candidates_after: candidates_before + hint_memory_ids.len(),
            memory_ids: hint_memory_ids,
        }
    }

    fn current_tick(request_correlation_id: u64) -> u64 {
        request_correlation_id
    }

    fn materialize_blackboard_state(
        namespace: NamespaceId,
        task_id: Option<String>,
    ) -> BlackboardState {
        let task = task_id.map(TaskId::new);
        let current_goal = task
            .as_ref()
            .map(|task_id| format!("resume task {}", task_id.as_str()))
            .unwrap_or_else(|| "resume active work".to_string());
        let mut state = BlackboardState::new(namespace, task, None, current_goal);
        state.subgoals = vec![
            "inspect selected evidence".to_string(),
            "resolve pending dependency".to_string(),
        ];
        state.active_evidence = vec![
            BlackboardEvidenceHandle::new(MemoryId(1), "selected_evidence").pinned(),
            BlackboardEvidenceHandle::new(MemoryId(2), "supporting_context"),
        ];
        state.active_beliefs = vec![
            "working-state remains a projection".to_string(),
            "durable memory remains authoritative".to_string(),
        ];
        state.unknowns = vec!["pending dependency resolution".to_string()];
        state.next_action = Some("resume from newest valid checkpoint".to_string());
        state.blocked_reason = Some("waiting_for_dependency".to_string());
        state
    }

    fn checkpoint_for_blackboard(
        namespace: NamespaceId,
        task_id: Option<String>,
        blackboard: &BlackboardState,
        created_tick: u64,
        status: GoalLifecycleStatus,
        stale: bool,
    ) -> GoalCheckpoint {
        let task_handle = task_id.or_else(|| {
            blackboard
                .task_id
                .as_ref()
                .map(|task| task.as_str().to_string())
        });
        GoalCheckpoint {
            checkpoint_id: format!(
                "goal-checkpoint-{}-{}",
                namespace.as_str(),
                task_handle.as_deref().unwrap_or("default")
            ),
            created_tick,
            status,
            evidence_handles: blackboard
                .active_evidence
                .iter()
                .map(|handle| handle.memory_id)
                .collect(),
            pending_dependencies: vec!["dependency:external-review".to_string()],
            blocked_reason: blackboard.blocked_reason.clone(),
            blackboard_summary: Some(format!(
                "goal={} next_action={} projection={} truth={}",
                blackboard.current_goal,
                blackboard.next_action.as_deref().unwrap_or("none"),
                blackboard.projection_kind,
                blackboard.authoritative_truth
            )),
            stale,
            namespace,
            task_id: task_handle.map(TaskId::new),
            authoritative_truth: "durable_memory",
        }
    }

    fn ensure_goal_working_state(
        store: &mut membrain_core::BrainStore,
        namespace: NamespaceId,
        task_id: TaskId,
    ) {
        if store.goal_state(&task_id).is_some() {
            return;
        }
        let blackboard = RuntimeState::materialize_blackboard_state(
            namespace.clone(),
            Some(task_id.as_str().to_string()),
        );
        let mut working_state = GoalWorkingState::new(
            task_id,
            namespace,
            None,
            vec![
                GoalStackFrame::new(blackboard.current_goal.clone()),
                GoalStackFrame {
                    goal: "resolve external dependency".to_string(),
                    parent_goal: Some(blackboard.current_goal.clone()),
                    priority: Some(1),
                    blocked_reason: blackboard.blocked_reason.clone(),
                },
            ],
            blackboard,
        );
        working_state.selected_evidence_handles = vec![MemoryId(1), MemoryId(2)];
        working_state.pending_dependencies = vec!["dependency:external-review".to_string()];
        store.upsert_goal_working_state(working_state);
    }

    fn runtime_request_context(
        namespace: NamespaceId,
        common: &crate::rpc::RuntimeCommonFields,
        visibility: SharingVisibility,
        fallback_request_id: String,
    ) -> RequestContext {
        RequestContext {
            namespace: Some(namespace),
            workspace_id: common.workspace_id.as_deref().map(WorkspaceId::new),
            agent_id: common.agent_id.as_deref().map(AgentId::new),
            session_id: common
                .session_id
                .as_deref()
                .and_then(|value| value.parse::<u64>().ok())
                .map(SessionId),
            task_id: common.task_id.as_deref().map(TaskId::new),
            request_id: common
                .request_id
                .as_deref()
                .map(RequestId::new)
                .transpose()
                .expect("transport request_id should validate")
                .unwrap_or_else(|| {
                    RequestId::new(fallback_request_id)
                        .expect("daemon-generated request ids should validate")
                }),
            policy_context: PolicyContext {
                include_public: common
                    .policy_context
                    .as_ref()
                    .map(|ctx| ctx.include_public)
                    .unwrap_or(false),
                sharing_visibility: visibility,
                caller_identity_bound: common
                    .policy_context
                    .as_ref()
                    .map(|ctx| ctx.caller_identity_bound)
                    .unwrap_or(true),
                workspace_acl_allowed: common
                    .policy_context
                    .as_ref()
                    .map(|ctx| ctx.workspace_acl_allowed)
                    .unwrap_or(true),
                agent_acl_allowed: common
                    .policy_context
                    .as_ref()
                    .map(|ctx| ctx.agent_acl_allowed)
                    .unwrap_or(true),
                session_visibility_allowed: common
                    .policy_context
                    .as_ref()
                    .map(|ctx| ctx.session_visibility_allowed)
                    .unwrap_or(true),
                legal_hold: common
                    .policy_context
                    .as_ref()
                    .map(|ctx| ctx.legal_hold)
                    .unwrap_or(false),
            },
            time_budget_ms: common.time_budget_ms,
        }
    }

    fn sharing_scope_label(outcome: &SharingAccessOutcome) -> FieldPresence<String> {
        match outcome.decision {
            SharingAccessDecision::Allow | SharingAccessDecision::Redact => outcome
                .sharing_scope
                .map(|scope| FieldPresence::Present(scope.as_str().to_string()))
                .unwrap_or(FieldPresence::Absent),
            SharingAccessDecision::Deny => FieldPresence::Redacted,
        }
    }

    fn share_policy_summary(
        namespace: &NamespaceId,
        outcome: &SharingAccessOutcome,
    ) -> membrain_core::api::TracePolicySummary {
        membrain_core::api::TracePolicySummary {
            effective_namespace: namespace.as_str().to_string(),
            policy_family: "visibility_sharing",
            outcome_class: outcome.policy_summary.outcome_class,
            blocked_stage: "policy_gate",
            redaction_fields: outcome.redaction_fields.clone(),
            retention_state: FieldPresence::Absent,
            sharing_scope: match outcome.decision {
                SharingAccessDecision::Allow | SharingAccessDecision::Redact => outcome
                    .sharing_scope
                    .map(|scope| FieldPresence::Present(scope.as_str()))
                    .unwrap_or(FieldPresence::Absent),
                SharingAccessDecision::Deny => FieldPresence::Redacted,
            },
            filters: vec![PolicyFilterSummary::new(
                namespace.as_str(),
                "visibility_sharing",
                outcome.policy_summary.outcome_class,
                "policy_gate",
                Self::sharing_scope_label(outcome),
                FieldPresence::Absent,
                outcome
                    .redaction_fields
                    .iter()
                    .map(|field| (*field).to_string())
                    .collect(),
            )],
        }
    }
}

impl DaemonRuntime {
    fn current_cache_generations() -> CacheGenerationAnchors {
        RuntimeState::current_cache_generations()
    }

    fn request_shape_hash(
        request: &RecallRequest,
        normalized: Option<&QueryByExampleNormalization>,
        result_budget: usize,
        output_mode_label: Option<&str>,
        method_name: &str,
    ) -> u64 {
        RuntimeState::request_shape_hash(
            request,
            normalized,
            result_budget,
            output_mode_label,
            method_name,
        )
    }

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

    pub async fn run_stdio_server(&self) -> anyhow::Result<()> {
        self.state
            .set_authority_mode(RuntimeAuthorityMode::StdioFacade)
            .await;
        eprintln!("membrain mcp server listening on stdio");
        let stdin = io::stdin();
        let stdout = io::stdout();
        let mut reader = io::BufReader::new(stdin);
        let mut writer = io::BufWriter::new(stdout);
        let mut line = String::new();

        loop {
            line.clear();
            let bytes_read = reader.read_line(&mut line).await?;
            if bytes_read == 0 {
                break;
            }
            if line.trim().is_empty() {
                continue;
            }

            let request = match serde_json::from_str::<JsonRpcRequest>(&line) {
                Ok(request) => request,
                Err(err) => {
                    let response = JsonRpcResponse::error(
                        None,
                        -32700,
                        format!("invalid json: {err}"),
                        None,
                    );
                    let encoded = serde_json::to_vec(&response)?;
                    writer.write_all(&encoded).await?;
                    writer.write_all(b"\n").await?;
                    writer.flush().await?;
                    continue;
                }
            };

            // In JSON-RPC 2.0, notifications have no id and don't expect responses
            let is_notification = request.id.is_none();
            let is_shutdown = request.method == "shutdown";

            if is_notification {
                // Process notification but don't send response
                let _ = Self::dispatch_request(request, Arc::clone(&self.state), self.state.next_request_id()).await;
                continue;
            }

            let response =
                Self::dispatch_request(request, Arc::clone(&self.state), self.state.next_request_id())
                    .await;
            let encoded = serde_json::to_vec(&response)?;
            writer.write_all(&encoded).await?;
            writer.write_all(b"\n").await?;
            writer.flush().await?;
            if is_shutdown {
                break;
            }
        }

        Ok(())
    }

    pub async fn run_until_stopped(&self) -> anyhow::Result<()> {
        self.state
            .set_authority_mode(RuntimeAuthorityMode::UnixSocketDaemon)
            .await;
        if let Some(parent) = self.config.socket_path.parent() {
            tokio::fs::create_dir_all(parent).await?;
        }

        self.remove_stale_socket().await?;

        let listener = UnixListener::bind(&self.config.socket_path)?;
        eprintln!(
            "membrain daemon listening on unix socket {}",
            self.config.socket_path.display()
        );
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
            RuntimeRequest::Health => {
                let report = state.health_report().await;
                JsonRpcResponse::success(request_id, report)
            }
            RuntimeRequest::Encode {
                content,
                namespace,
                memory_type: _,
                visibility,
                emotional_annotations,
                common,
            } => {
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
                let requested_visibility = visibility.as_deref().or_else(|| {
                    common
                        .policy_context
                        .as_ref()
                        .and_then(|ctx| ctx.sharing_visibility.as_deref())
                });
                let visibility = match requested_visibility {
                    Some(raw) => match SharingVisibility::parse(raw) {
                        Some(visibility) => visibility,
                        None => {
                            return JsonRpcResponse::error(
                                request_id,
                                -32602,
                                "invalid visibility",
                                Some(json!({"error_kind": "validation_failure"})),
                            )
                        }
                    },
                    None => SharingVisibility::Private,
                };
                let memory_id = MemoryId(request_correlation_id);
                let record = state.encode_memory(
                    namespace.clone(),
                    memory_id,
                    &content,
                    emotional_annotations.as_ref(),
                    &common,
                    visibility,
                );
                state.store_encoded_memory(record.clone());
                if let Some(affect) = record.layout.metadata.affect {
                    let mut audit = state
                        .audit
                        .lock()
                        .expect("runtime audit state lock should be available");
                    let tick = audit.store.current_tick().saturating_add(1);
                    let _ = audit.store.record_affect_trajectory(
                        namespace.clone(),
                        memory_id,
                        record.layout.metadata.landmark.era_id.clone(),
                        tick,
                        affect,
                    );
                }
                JsonRpcResponse::success(
                    request_id,
                    json!({
                        "status": "accepted",
                        "memory_id": memory_id.0,
                        "namespace": namespace.as_str(),
                        "visibility": visibility.as_str(),
                        "message": "encode envelope accepted with namespace-aware visibility metadata"
                    }),
                )
            }
            RuntimeRequest::Observe {
                content,
                namespace,
                context,
                chunk_size,
                source_label,
                topic_threshold,
                min_chunk_size,
                dry_run,
                common: _,
            } => {
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
                let config = ObserveConfig {
                    chunk_size_chars: chunk_size.unwrap_or(500),
                    topic_shift_threshold: topic_threshold.unwrap_or(0.35),
                    min_chunk_chars: min_chunk_size.unwrap_or(50),
                    context,
                    source_label,
                };
                let dry_run = dry_run.unwrap_or(false);
                let mut seen_fingerprints = state
                    .memories
                    .lock()
                    .expect("runtime memory registry lock should be available")
                    .values()
                    .filter(|record| record.layout.metadata.namespace == namespace)
                    .map(|record| record.layout.metadata.fingerprint)
                    .collect::<std::collections::HashSet<_>>();
                let report = ObserveEngine::observe_content(
                    &membrain_core::BrainStore::new(RuntimeConfig::default()).encode_engine(),
                    &content,
                    &config,
                    true,
                    |fingerprint| {
                        let already_seen = seen_fingerprints.contains(&fingerprint);
                        if !already_seen {
                            seen_fingerprints.insert(fingerprint);
                        }
                        already_seen
                    },
                );
                if !dry_run {
                    for fragment in &report.fragments {
                        if fragment.prepared.write_decision.as_str() != "capture" {
                            continue;
                        }
                        let memory_id = MemoryId(state.next_request_id());
                        let passive_observation = PassiveObservationInspectSummary::from_encode(
                            &fragment.prepared.passive_observation_inspect,
                        );
                        let confidence_inputs = RuntimeState::build_confidence_inputs(
                            &fragment.prepared.normalized,
                            fragment.prepared.provisional_salience,
                            Some(&passive_observation),
                            memory_id,
                            0,
                            None,
                        );
                        let confidence_output =
                            ConfidenceEngine.compute(&confidence_inputs, &ConfidencePolicy::default());
                        let layout = membrain_core::BrainStore::new(RuntimeConfig::default())
                            .tier2_store()
                            .layout_item(
                                namespace.clone(),
                                memory_id,
                                SessionId(1),
                                fragment.prepared.fingerprint,
                                &fragment.prepared.normalized,
                                Some(confidence_inputs.clone()),
                                Some(confidence_output.clone()),
                            );
                        state.store_encoded_memory(RuntimeMemoryRecord {
                            layout,
                            confidence_inputs,
                            confidence_output,
                            passive_observation: Some(passive_observation),
                            causal_parents: Vec::new(),
                            causal_link_type: None,
                        });
                    }
                }
                JsonRpcResponse::success(
                    request_id,
                    json!({
                        "status": if dry_run { "preview" } else { "accepted" },
                        "namespace": namespace.as_str(),
                        "dry_run": dry_run,
                        "observation_source": report.observation_source,
                        "observation_chunk_id": report.observation_chunk_id,
                        "bytes_processed": report.bytes_processed,
                        "topic_shifts": report.topic_shifts_detected,
                        "fragments_previewed": report.fragments.len(),
                        "memories_created": if dry_run { 0 } else { report.memories_created },
                        "suppressed": report.suppressed,
                        "denied": report.denied,
                        "preview": report.fragments.iter().map(|fragment| json!({
                            "index": fragment.index,
                            "write_decision": fragment.prepared.write_decision.as_str(),
                            "captured_as_observation": fragment.prepared.captured_as_observation,
                            "compact_text": fragment.prepared.normalized.compact_text,
                            "fingerprint": fragment.prepared.fingerprint,
                            "route_family": fragment.prepared.classification.route_family.as_str(),
                        })).collect::<Vec<_>>()
                    }),
                )
            }
            RuntimeRequest::Skills {
                namespace,
                extract,
                common: _,
            } => {
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
                let result = membrain_core::BrainStore::default().skill_artifacts(
                    namespace.clone(),
                    membrain_core::engine::consolidation::ConsolidationPolicy {
                        minimum_candidates: 1,
                        batch_size: 2,
                        min_skill_members: 2,
                        ..Default::default()
                    },
                    2,
                    extract.unwrap_or(false),
                );
                let accepted = extract.unwrap_or(false);
                JsonRpcResponse::success(
                    request_id,
                    json!({
                        "status": if accepted { "accepted" } else { "review" },
                        "namespace": result.namespace,
                        "extraction_trigger": result.extraction_trigger,
                        "extracted_count": result.extracted_count,
                        "skipped_count": result.skipped_count,
                        "reflection_compiler_active": result.procedures.iter().any(|procedure| procedure.review.reflection.is_some()),
                        "procedures": result.procedures,
                    }),
                )
            }
            RuntimeRequest::Procedures {
                namespace,
                promote,
                rollback,
                note,
                approved_by,
                public,
                common,
            } => {
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
                let mut store = membrain_core::BrainStore::default();
                let request_context = RuntimeState::runtime_request_context(
                    namespace.clone(),
                    &common,
                    if public.unwrap_or(false) {
                        SharingVisibility::Public
                    } else {
                        SharingVisibility::Shared
                    },
                    format!("procedures-{request_correlation_id}"),
                );
                let candidate = store.skill_artifacts(
                    namespace.clone(),
                    membrain_core::engine::consolidation::ConsolidationPolicy {
                        minimum_candidates: 1,
                        batch_size: 2,
                        min_skill_members: 2,
                        ..Default::default()
                    },
                    2,
                    true,
                );
                let default_pattern_handle = candidate.procedures.first().map(|procedure| {
                    procedure.recall.pattern_handle.clone()
                });
                if let Some(pattern_handle) = promote
                    .as_deref()
                    .or(default_pattern_handle.as_deref())
                {
                    if let Err(error) = store.promote_skill_to_procedural_with_context(
                        request_context,
                        pattern_handle,
                        approved_by
                            .clone()
                            .unwrap_or_else(|| "daemon.operator".to_string()),
                        note.clone()
                            .unwrap_or_else(|| "approved via daemon procedures surface".to_string()),
                    ) {
                        return JsonRpcResponse::error(
                            request_id,
                            -32602,
                            error.reason.as_str(),
                            Some(json!({"error_kind": error.reason.as_str()})),
                        );
                    }
                }
                if let Some(pattern_handle) = rollback.as_deref() {
                    if let Err(error) = store.rollback_procedural_entry(
                        namespace.clone(),
                        pattern_handle,
                        note.clone()
                            .unwrap_or_else(|| "rolled back via daemon procedures surface".to_string()),
                    ) {
                        return JsonRpcResponse::error(
                            request_id,
                            -32602,
                            error.reason.as_str(),
                            Some(json!({"error_kind": error.reason.as_str()})),
                        );
                    }
                }
                let result = store.procedural_store_surface(
                    namespace.clone(),
                    membrain_core::engine::consolidation::ConsolidationPolicy {
                        minimum_candidates: 1,
                        batch_size: 2,
                        min_skill_members: 2,
                        ..Default::default()
                    },
                    2,
                    promote.is_some(),
                );
                JsonRpcResponse::success(
                    request_id,
                    json!({
                        "status": result.outcome,
                        "namespace": result.namespace,
                        "extraction_trigger": result.extraction_trigger,
                        "reviewed_candidate_count": result.reviewed_candidate_count,
                        "procedural_count": result.procedural_count,
                        "direct_lookup_supported": result.direct_lookup_supported,
                        "procedures": result.procedures,
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
                common,
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
                common,
                request_correlation_id,
                &state,
            ) {
                Ok(response) => JsonRpcResponse::success(request_id, json!(response)),
                Err(message) => JsonRpcResponse::error(request_id, -32602, message, None),
            },
            RuntimeRequest::ContextBudget {
                token_budget,
                namespace,
                current_context,
                working_memory_ids,
                format,
                mood_congruent,
                common,
            } => match Self::handle_context_budget(
                token_budget,
                &namespace,
                current_context,
                working_memory_ids,
                format,
                mood_congruent,
                common,
                request_correlation_id,
                &state,
            ) {
                Ok(response) => JsonRpcResponse::success(request_id, json!(response)),
                Err(message) => JsonRpcResponse::error(request_id, -32602, message, None),
            },
            RuntimeRequest::GoalState {
                namespace,
                task_id,
                common,
            } => match Self::handle_goal_state(&namespace, task_id, common, request_correlation_id, &state) {
                Ok(response) => JsonRpcResponse::success(request_id, json!(response)),
                Err(message) => JsonRpcResponse::error(request_id, -32602, message, None),
            },
            RuntimeRequest::HotPaths {
                namespace,
                top_n,
                common,
            } => match Self::handle_hot_paths(&namespace, top_n, common, &state) {
                Ok(response) => JsonRpcResponse::success(request_id, json!(response)),
                Err(message) => JsonRpcResponse::error(request_id, -32602, message, None),
            },
            RuntimeRequest::DeadZones {
                namespace,
                min_age_ticks,
                common,
            } => match Self::handle_dead_zones(&namespace, min_age_ticks, common, &state) {
                Ok(response) => JsonRpcResponse::success(request_id, json!(response)),
                Err(message) => JsonRpcResponse::error(request_id, -32602, message, None),
            },
            RuntimeRequest::Compress {
                namespace,
                dry_run,
                common,
            } => match Self::handle_compress(&namespace, dry_run, common, &state) {
                Ok(response) => JsonRpcResponse::success(request_id, response),
                Err(message) => JsonRpcResponse::error(request_id, -32602, message, None),
            },
            RuntimeRequest::Schemas {
                namespace,
                top_n,
                common,
            } => match Self::handle_schemas(&namespace, top_n, common, &state) {
                Ok(response) => JsonRpcResponse::success(request_id, response),
                Err(message) => JsonRpcResponse::error(request_id, -32602, message, None),
            },
            RuntimeRequest::GoalPause {
                namespace,
                task_id,
                note,
                common,
            } => match Self::handle_goal_pause(&namespace, task_id, note, common, request_correlation_id, &state) {
                Ok(response) => JsonRpcResponse::success(request_id, json!(response)),
                Err(message) => JsonRpcResponse::error(request_id, -32602, message, None),
            },
            RuntimeRequest::GoalPin {
                namespace,
                task_id,
                memory_id,
                common,
            } => match Self::handle_goal_pin(&namespace, task_id, memory_id, common, request_correlation_id, &state) {
                Ok(response) => JsonRpcResponse::success(request_id, json!(response)),
                Err(message) => JsonRpcResponse::error(request_id, -32602, message, None),
            },
            RuntimeRequest::GoalDismiss {
                namespace,
                task_id,
                memory_id,
                common,
            } => match Self::handle_goal_dismiss(&namespace, task_id, memory_id, common, request_correlation_id, &state) {
                Ok(response) => JsonRpcResponse::success(request_id, json!(response)),
                Err(message) => JsonRpcResponse::error(request_id, -32602, message, None),
            },
            RuntimeRequest::GoalSnapshot {
                namespace,
                task_id,
                note,
                common,
            } => match Self::handle_goal_snapshot(&namespace, task_id, note, common, request_correlation_id, &state) {
                Ok(response) => JsonRpcResponse::success(request_id, json!(response)),
                Err(message) => JsonRpcResponse::error(request_id, -32602, message, None),
            },
            RuntimeRequest::GoalResume {
                namespace,
                task_id,
                common,
            } => match Self::handle_goal_resume(&namespace, task_id, common, request_correlation_id, &state) {
                Ok(response) => JsonRpcResponse::success(request_id, json!(response)),
                Err(message) => JsonRpcResponse::error(request_id, -32602, message, None),
            },
            RuntimeRequest::GoalAbandon {
                namespace,
                task_id,
                reason,
                common,
            } => match Self::handle_goal_abandon(&namespace, task_id, reason, common, request_correlation_id, &state) {
                Ok(response) => JsonRpcResponse::success(request_id, json!(response)),
                Err(message) => JsonRpcResponse::error(request_id, -32602, message, None),
            },
            RuntimeRequest::Inspect {
                id,
                namespace,
                common,
            } => match Self::handle_inspect(id, &namespace, common, request_correlation_id, &state) {
                Ok(response) => JsonRpcResponse::success(request_id, json!(response)),
                Err(message) => JsonRpcResponse::error(request_id, -32602, message, None),
            },
            RuntimeRequest::Explain {
                query,
                namespace,
                limit,
                depth,
                common,
            } => match Self::handle_explain(
                &query,
                &namespace,
                limit,
                depth,
                common,
                request_correlation_id,
                &state,
            ) {
                Ok(response) => JsonRpcResponse::success(request_id, json!(response)),
                Err(message) => JsonRpcResponse::error(request_id, -32602, message, None),
            },
            RuntimeRequest::Invalidate {
                id,
                namespace,
                dry_run,
                common,
            } => match Self::handle_invalidate(
                MemoryId(id),
                &namespace,
                dry_run,
                common,
                request_correlation_id,
                &state,
            ) {
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
                original_query,
                proposed_action,
                authorization_token,
                bypass_flags,
            } => match Self::handle_preflight_allow(
                &namespace,
                &original_query,
                &proposed_action,
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
                                uri: RUNTIME_HEALTH_URI.to_string(),
                                name: "runtime-health".to_string(),
                                mime_type: "application/json".to_string(),
                                resource_kind: "runtime_health".to_string(),
                                description: Some(
                                    "Bounded machine-readable brain health dashboard"
                                        .to_string(),
                                ),
                                uri_template: None,
                                examples: vec![RUNTIME_HEALTH_URI.to_string()],
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
                RUNTIME_HEALTH_URI => {
                    let report = state.health_report().await;
                    let read_request_id = RequestId::new(format!(
                        "daemon-resource-read-health-{request_correlation_id}"
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
                                resource_kind: "runtime_health".to_string(),
                                bounded: true,
                                payload: report,
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
                _ => {
                    if let Some((resource_namespace, memory_id)) =
                        parse_inspect_resource_uri(uri.as_str())
                    {
                        match Self::handle_inspect(
                            memory_id,
                            &resource_namespace,
                            crate::rpc::RuntimeCommonFields::default(),
                            request_correlation_id,
                            &state,
                        ) {
                            Ok(response) => JsonRpcResponse::success(request_id, json!(response)),
                            Err(message) => JsonRpcResponse::error(
                                request_id,
                                -32602,
                                message,
                                Some(json!({"error_kind": "validation_failure"})),
                            ),
                        }
                    } else {
                        JsonRpcResponse::error(
                            request_id,
                            -32602,
                            format!("unknown resource uri '{uri}'"),
                            Some(json!({"error_kind": "validation_failure"})),
                        )
                    }
                }
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
                let mode = mode.unwrap_or_else(|| "archive".to_string());
                let brain_store = membrain_core::BrainStore::default();
                let forgetting_engine = brain_store.forgetting_engine();
                let action = match mode.as_str() {
                    "archive" => forgetting_engine.evaluate_memory(
                        MemoryId(id),
                        0,
                        &ForgettingPolicy::default(),
                    ),
                    "restore" => forgetting_engine.plan_restore(false),
                    "restore_partial" => forgetting_engine.plan_restore(true),
                    "delete" => match forgetting_engine.plan_policy_delete(&ForgettingPolicy {
                        allow_hard_delete: true,
                        ..Default::default()
                    }) {
                        Ok(action) => action,
                        Err(message) => {
                            return JsonRpcResponse::error(
                                request_id,
                                -32602,
                                message,
                                Some(json!({"error_kind": "validation_failure"})),
                            )
                        }
                    },
                    "demote" => ForgettingAction::Demote {
                        from_tier: "tier1",
                        to_tier: "tier2",
                    },
                    _ => {
                        return JsonRpcResponse::error(
                            request_id,
                            -32602,
                            format!("unknown forget mode '{mode}'"),
                            Some(json!({"error_kind": "validation_failure"})),
                        )
                    }
                };
                let reversibility = forgetting_engine
                    .evaluate_reversibility(&action, &ForgettingPolicy::default())
                    .as_str();
                let (prior_archive_state, resulting_archive_state) =
                    forgetting_engine.archive_state_transition(&action);
                let policy_surface = forgetting_engine.action_policy_surface(&action);
                let reason_code = forgetting_engine.action_reason(&action);
                let audit_kind = forgetting_engine.audit_event_kind(&action).as_str();
                let disposition = match action {
                    ForgettingAction::Archive | ForgettingAction::Demote { .. } => "eligible",
                    ForgettingAction::Restore { .. } | ForgettingAction::PolicyDelete => "explicit",
                    ForgettingAction::Skip => "ineligible",
                };
                let operator_review_required = matches!(action, ForgettingAction::Skip)
                    && reason_code == "ineligible_or_retained";
                JsonRpcResponse::success(
                    request_id,
                    json!({
                        "status": "accepted",
                        "id": id,
                        "namespace": namespace.as_str(),
                        "mode": mode,
                        "action": action.as_str(),
                        "reason": reason.unwrap_or_else(|| reason_code.to_string()),
                        "reason_code": reason_code,
                        "disposition": disposition,
                        "policy_surface": policy_surface,
                        "reversibility": reversibility,
                        "prior_archive_state": prior_archive_state,
                        "resulting_archive_state": resulting_archive_state,
                        "partial_restore": matches!(action, ForgettingAction::Restore { partial: true, .. }),
                        "audit_kind": audit_kind,
                        "operator_review_required": operator_review_required,
                        "message": "forget lifecycle evaluated with distinct archive, restore, demotion, and policy-delete semantics"
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
            RuntimeRequest::Audit {
                namespace,
                memory_id,
                since_tick,
                op,
                limit,
                common: _,
            } => {
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
                let audit = state
                    .audit
                    .lock()
                    .expect("runtime audit state lock should be available");
                let kind = op.as_deref().and_then(|name| {
                    audit.store
                        .audit_entries()
                        .into_iter()
                        .find(|entry| entry.kind.as_str() == name)
                        .map(|entry| entry.kind)
                });
                let slice = audit.store.audit_log().slice(
                    &AuditLogFilter {
                        namespace: Some(namespace.clone()),
                        memory_id: memory_id.map(MemoryId),
                        kind,
                        min_tick: since_tick,
                        ..AuditLogFilter::default()
                    },
                    limit,
                );
                let payload = McpAuditPayload {
                    request_id: RequestId::new(format!("daemon-audit-{request_correlation_id}"))
                        .expect("daemon-generated audit request ids are valid"),
                    namespace,
                    total_matches: slice.total_matches,
                    returned_rows: slice.returned_rows(),
                    truncated: slice.truncated,
                    entries: slice.rows.into_iter().map(McpAuditRow::from).collect(),
                };
                JsonRpcResponse::success(
                    request_id,
                    serde_json::to_value(payload).expect("audit payload serializes"),
                )
            }
            RuntimeRequest::MoodHistory {
                namespace,
                since_tick,
                common: _,
            } => {
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
                let audit = state
                    .audit
                    .lock()
                    .expect("runtime audit state lock should be available");
                let history = audit.store.mood_history(namespace.clone(), since_tick);
                JsonRpcResponse::success(
                    request_id,
                    serde_json::to_value(history).expect("mood history serializes"),
                )
            }
            RuntimeRequest::Share {
                id,
                namespace_id,
                common,
            } => {
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
                let layout = match state.update_visibility(
                    MemoryId(id),
                    SharingVisibility::Shared,
                    Some(&namespace),
                ) {
                    Some(layout) => layout,
                    None => {
                        let record = state.encode_memory(
                            namespace.clone(),
                            MemoryId(id),
                            &format!("shared memory {id}"),
                            None,
                            &common,
                            SharingVisibility::Shared,
                        );
                        let layout = record.layout.clone();
                        state.store_encoded_memory(record);
                        layout
                    }
                };
                let policy_summary = json!({
                    "effective_namespace": namespace.as_str(),
                    "policy_family": "visibility_sharing",
                    "outcome_class": "accepted",
                    "blocked_stage": "policy_gate",
                    "redaction_fields": [],
                    "retention_state": "absent",
                    "sharing_scope": "shared"
                });
                let response = json!({
                    "status": "accepted",
                    "id": id,
                    "namespace": namespace.as_str(),
                    "visibility": layout.metadata.visibility.as_str(),
                    "policy_summary": policy_summary,
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
                        "event_kind": "approved_sharing",
                        "redacted": false
                    }
                });
                JsonRpcResponse::success(request_id, response)
            }
            RuntimeRequest::Unshare {
                id,
                namespace,
                common,
            } => {
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
                let layout = match state.update_visibility(MemoryId(id), SharingVisibility::Private, None)
                {
                    Some(layout) => layout,
                    None => {
                        let record = state.encode_memory(
                            namespace.clone(),
                            MemoryId(id),
                            &format!("private memory {id}"),
                            None,
                            &common,
                            SharingVisibility::Private,
                        );
                        let layout = record.layout.clone();
                        state.store_encoded_memory(record);
                        layout
                    }
                };
                let policy_summary = json!({
                    "effective_namespace": namespace.as_str(),
                    "policy_family": "visibility_sharing",
                    "outcome_class": "accepted",
                    "blocked_stage": "policy_gate",
                    "redaction_fields": ["sharing_scope"],
                    "retention_state": "absent",
                    "sharing_scope": "private"
                });
                let response = json!({
                    "status": "accepted",
                    "id": id,
                    "namespace": namespace.as_str(),
                    "visibility": layout.metadata.visibility.as_str(),
                    "policy_summary": policy_summary,
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
                        "event_kind": "policy_redacted",
                        "redacted": true
                    }
                });
                JsonRpcResponse::success(request_id, response)
            }
            RuntimeRequest::Fork {
                name,
                namespace,
                parent_namespace,
                inherit,
                note,
                common: _,
            } => {
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
                let parent_namespace = match parent_namespace {
                    Some(parent) => match NamespaceId::new(parent) {
                        Ok(namespace) => namespace,
                        Err(_) => {
                            return JsonRpcResponse::error(
                                request_id,
                                -32602,
                                "malformed parent_namespace",
                                Some(json!({"error_kind": "validation_failure"})),
                            )
                        }
                    },
                    None => namespace.clone(),
                };
                let inherit_visibility = match inherit.as_deref().unwrap_or("public") {
                    "public" => ForkInheritance::PublicOnly,
                    "shared" => ForkInheritance::SharedToo,
                    "all" => ForkInheritance::All,
                    _ => {
                        return JsonRpcResponse::error(
                            request_id,
                            -32602,
                            "invalid inherit value",
                            Some(json!({"error_kind": "validation_failure"})),
                        )
                    }
                };
                let mut audit = state
                    .audit
                    .lock()
                    .expect("runtime audit state lock should be available");
                let output = audit.store.fork(ForkConfig {
                    name,
                    parent_namespace,
                    inherit_visibility,
                    note,
                });
                JsonRpcResponse::success(
                    request_id,
                    serde_json::to_value(output).expect("fork output serializes"),
                )
            }
            RuntimeRequest::MergeFork {
                fork_name,
                target_namespace,
                conflict_strategy,
                dry_run,
                common: _,
            } => {
                let target_namespace = match NamespaceId::new(&target_namespace) {
                    Ok(namespace) => namespace,
                    Err(_) => {
                        return JsonRpcResponse::error(
                            request_id,
                            -32602,
                            "malformed target_namespace",
                            Some(json!({"error_kind": "validation_failure"})),
                        )
                    }
                };
                let conflict_strategy = match conflict_strategy.as_deref().unwrap_or("manual") {
                    "fork-wins" => MergeConflictStrategy::ForkWins,
                    "parent-wins" => MergeConflictStrategy::ParentWins,
                    "recency-wins" => MergeConflictStrategy::RecencyWins,
                    "manual" => MergeConflictStrategy::Manual,
                    _ => {
                        return JsonRpcResponse::error(
                            request_id,
                            -32602,
                            "invalid conflict_strategy",
                            Some(json!({"error_kind": "validation_failure"})),
                        )
                    }
                };
                let mut audit = state
                    .audit
                    .lock()
                    .expect("runtime audit state lock should be available");
                let output = match audit.store.merge_fork(MergeConfig {
                    fork_name,
                    target_namespace,
                    conflict_strategy,
                    dry_run,
                }) {
                    Some(output) => output,
                    None => {
                        return JsonRpcResponse::error(
                            request_id,
                            -32004,
                            "unknown fork",
                            Some(json!({"error_kind": "not_found"})),
                        )
                    }
                };
                JsonRpcResponse::success(
                    request_id,
                    serde_json::to_value(output).expect("merge output serializes"),
                )
            }
            RuntimeRequest::Link {
                source_id,
                target_id,
                namespace,
                link_type,
                common,
            } => {
                let _ = (&namespace, &link_type, &common);
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

            // -- MCP Protocol handlers -------------------------------------------------
            RuntimeRequest::McpInitialize {
                protocol_version,
                capabilities: _,
                client_info: _,
            } => {
                JsonRpcResponse::success(
                    request_id,
                    json!({
                        "protocolVersion": protocol_version,
                        "capabilities": {
                            "tools": {
                                "listChanged": false
                            },
                            "resources": {
                                "subscribe": false,
                                "listChanged": false
                            },
                            "prompts": {
                                "listChanged": false
                            }
                        },
                        "serverInfo": {
                            "name": "membrain",
                            "version": env!("CARGO_PKG_VERSION")
                        }
                    }),
                )
            }
            RuntimeRequest::McpInitialized => {
                // Notification - no response needed, but we return empty success for stdio mode
                JsonRpcResponse::success(request_id, json!({}))
            }
            RuntimeRequest::McpToolsList => {
                JsonRpcResponse::success(
                    request_id,
                    json!({
                        "tools": [
                            {
                                "name": "encode",
                                "title": "Store Memory",
                                "description": "Store a new memory in the specified namespace.\n\nThis tool ingests content into the Membrain memory system, making it available for future recall and inspection. The memory is stored in the specified namespace with optional metadata.\n\nArgs:\n  - content (string, required): The memory content to store\n  - namespace (string, required): The namespace to store the memory in (e.g., 'default', 'project-x')\n  - memory_type (string, optional): Classification of the memory type\n  - visibility (string, optional): Visibility setting ('private', 'shared', 'public')\n\nReturns:\n  { \"status\": \"accepted\", \"memory_id\": number, \"namespace\": string, \"visibility\": string }\n\nExamples:\n  - Store a fact: { \"content\": \"API endpoint /users requires authentication\", \"namespace\": \"project-api\" }\n  - Store with type: { \"content\": \"...\", \"namespace\": \"default\", \"memory_type\": \"incident\" }\n\nError Handling:\n  - Returns error if namespace is malformed\n  - Returns error if visibility value is invalid",
                                "inputSchema": {
                                    "type": "object",
                                    "properties": {
                                        "content": { "type": "string", "description": "The memory content to store" },
                                        "namespace": { "type": "string", "description": "The namespace to store the memory in" },
                                        "memory_type": { "type": "string", "description": "Optional memory type classification" },
                                        "visibility": { "type": "string", "enum": ["private", "shared", "public"], "description": "Visibility setting (default: private)" }
                                    },
                                    "required": ["content", "namespace"]
                                },
                                "annotations": {
                                    "readOnlyHint": false,
                                    "destructiveHint": false,
                                    "idempotentHint": false,
                                    "openWorldHint": false
                                }
                            },
                            {
                                "name": "recall",
                                "title": "Search Memories",
                                "description": "Search and retrieve memories from a namespace.\n\nThis tool searches the Membrain memory system using semantic similarity. It returns the most relevant memories matching the query.\n\nArgs:\n  - query_text (string, required): The search query for semantic matching\n  - namespace (string, required): The namespace to search in\n  - limit (integer, optional): Maximum number of results (default: 10, max: 100)\n\nReturns:\n  { \"memories\": [...], \"namespace\": string }\n\nExamples:\n  - Search by query: { \"query_text\": \"authentication error\", \"namespace\": \"default\" }\n  - Search with limit: { \"query_text\": \"database\", \"namespace\": \"project-x\", \"limit\": 20 }\n\nError Handling:\n  - Returns empty results if no matches found\n  - Returns error if namespace is malformed",
                                "inputSchema": {
                                    "type": "object",
                                    "properties": {
                                        "query_text": { "type": "string", "description": "The search query for semantic matching" },
                                        "namespace": { "type": "string", "description": "The namespace to search in" },
                                        "limit": { "type": "integer", "minimum": 1, "maximum": 100, "description": "Maximum number of results (default: 10)" }
                                    },
                                    "required": ["query_text", "namespace"]
                                },
                                "annotations": {
                                    "readOnlyHint": true,
                                    "destructiveHint": false,
                                    "idempotentHint": true,
                                    "openWorldHint": false
                                }
                            },
                            {
                                "name": "inspect",
                                "title": "Inspect Memory",
                                "description": "Get detailed information about a specific memory.\n\nThis tool retrieves the full content and metadata of a specific memory by its ID.\n\nArgs:\n  - id (integer, required): The memory ID to inspect\n  - namespace (string, required): The namespace containing the memory\n\nReturns:\n  Full memory object with content, metadata, timestamps, etc.\n\nExamples:\n  - Inspect memory: { \"id\": 42, \"namespace\": \"default\" }\n\nError Handling:\n  - Returns error if memory ID not found in namespace\n  - Returns error if namespace is malformed",
                                "inputSchema": {
                                    "type": "object",
                                    "properties": {
                                        "id": { "type": "integer", "description": "The memory ID to inspect" },
                                        "namespace": { "type": "string", "description": "The namespace containing the memory" }
                                    },
                                    "required": ["id", "namespace"]
                                },
                                "annotations": {
                                    "readOnlyHint": true,
                                    "destructiveHint": false,
                                    "idempotentHint": true,
                                    "openWorldHint": false
                                }
                            },
                            {
                                "name": "why",
                                "title": "Explain Memory Relevance",
                                "description": "Explain why a memory was retrieved or how it relates to a query.\n\nThis tool provides explainability for memory retrieval, showing why certain memories match a query and their relevance scores.\n\nArgs:\n  - query (string, required): The query to explain\n  - namespace (string, required): The namespace to search in\n  - limit (integer, optional): Maximum number of results to explain (default: 5)\n\nReturns:\n  Explanation object with relevance scores, matching factors, and memory references\n\nExamples:\n  - Explain retrieval: { \"query\": \"database connection timeout\", \"namespace\": \"incidents\" }\n\nError Handling:\n  - Returns empty explanation if no relevant memories found",
                                "inputSchema": {
                                    "type": "object",
                                    "properties": {
                                        "query": { "type": "string", "description": "The query to explain" },
                                        "namespace": { "type": "string", "description": "The namespace to search in" },
                                        "limit": { "type": "integer", "minimum": 1, "maximum": 20, "description": "Maximum number of results to explain" }
                                    },
                                    "required": ["query", "namespace"]
                                },
                                "annotations": {
                                    "readOnlyHint": true,
                                    "destructiveHint": false,
                                    "idempotentHint": true,
                                    "openWorldHint": false
                                }
                            },
                            {
                                "name": "health",
                                "title": "Check System Health",
                                "description": "Check the health status of the memory system.\n\nThis tool returns the current health status including subsystem states, alerts, and key metrics.\n\nArgs:\n  (none)\n\nReturns:\n  Health report with subsystem status, alerts, cache metrics, and attention heatmap\n\nExamples:\n  - Check health: {}\n\nError Handling:\n  - Always returns a health report, even if some subsystems are degraded",
                                "inputSchema": {
                                    "type": "object",
                                    "properties": {}
                                },
                                "annotations": {
                                    "readOnlyHint": true,
                                    "destructiveHint": false,
                                    "idempotentHint": true,
                                    "openWorldHint": false
                                }
                            },
                            {
                                "name": "doctor",
                                "title": "Run Diagnostics",
                                "description": "Run diagnostics on the memory system.\n\nThis tool performs comprehensive diagnostics and returns a detailed report of system status, including any issues found and recommended remediation steps.\n\nArgs:\n  (none)\n\nReturns:\n  Doctor report with status, checks performed, and any issues found\n\nExamples:\n  - Run diagnostics: {}\n\nError Handling:\n  - Always returns a doctor report",
                                "inputSchema": {
                                    "type": "object",
                                    "properties": {}
                                },
                                "annotations": {
                                    "readOnlyHint": true,
                                    "destructiveHint": false,
                                    "idempotentHint": true,
                                    "openWorldHint": false
                                }
                            }
                        ]
                    }),
                )
            }
            RuntimeRequest::McpToolsCall { name, arguments } => {
                // Dispatch tool call to the appropriate runtime method
                match name.as_str() {
                    "encode" => {
                        let content = arguments.get("content").and_then(|v| v.as_str()).unwrap_or("");
                        let namespace = arguments.get("namespace").and_then(|v| v.as_str()).unwrap_or("default").to_string();
                        let _memory_type = arguments.get("memory_type").and_then(|v| v.as_str()).map(|s| s.to_string());
                        let visibility = arguments.get("visibility").and_then(|v| v.as_str()).map(|s| s.to_string());

                        let namespace_id = match NamespaceId::new(&namespace) {
                            Ok(ns) => ns,
                            Err(_) => {
                                return JsonRpcResponse::error(
                                    request_id,
                                    -32602,
                                    "malformed namespace",
                                    Some(json!({"error_kind": "validation_failure"})),
                                )
                            }
                        };
                        let requested_visibility = visibility.as_deref();
                        let visibility = match requested_visibility {
                            Some(raw) => match SharingVisibility::parse(raw) {
                                Some(visibility) => visibility,
                                None => {
                                    return JsonRpcResponse::error(
                                        request_id,
                                        -32602,
                                        "invalid visibility",
                                        Some(json!({"error_kind": "validation_failure"})),
                                    )
                                }
                            },
                            None => SharingVisibility::Private,
                        };
                        let memory_id = MemoryId(request_correlation_id);
                        let common = crate::rpc::RuntimeCommonFields::default();
                        let record = state.encode_memory(
                            namespace_id.clone(),
                            memory_id,
                            content,
                            None,
                            &common,
                            visibility,
                        );
                        state.store_encoded_memory(record.clone());
                        if let Some(affect) = record.layout.metadata.affect {
                            let mut audit = state.audit.lock().expect("runtime audit state lock should be available");
                            let tick = audit.store.current_tick().saturating_add(1);
                            let _ = audit.store.record_affect_trajectory(
                                namespace_id,
                                memory_id,
                                record.layout.metadata.landmark.era_id.clone(),
                                tick,
                                affect,
                            );
                        }
                        JsonRpcResponse::success(
                            request_id,
                            json!({
                                "status": "accepted",
                                "memory_id": memory_id.0,
                                "namespace": namespace,
                                "visibility": visibility.as_str(),
                                "message": "encode envelope accepted"
                            }),
                        )
                    }
                    "recall" => {
                        let query_text = arguments.get("query_text").and_then(|v| v.as_str()).map(|s| s.to_string());
                        let namespace = arguments.get("namespace").and_then(|v| v.as_str()).unwrap_or("default").to_string();
                        let result_budget = arguments.get("limit").and_then(|v| v.as_u64()).map(|n| n as usize);

                        match Self::handle_recall(
                            query_text,
                            &namespace,
                            None, // mode
                            result_budget,
                            None, // token_budget
                            None, // time_budget_ms
                            None, // context_text
                            None, // effort
                            None, // include_public
                            None, // like_id
                            None, // unlike_id
                            None, // graph_mode
                            None, // cold_tier
                            None, // workspace_id
                            None, // agent_id
                            None, // session_id
                            None, // task_id
                            None, // memory_kinds
                            None, // era_id
                            None, // as_of_tick
                            None, // at_snapshot
                            None, // min_strength
                            None, // min_confidence
                            None, // show_decaying
                            None, // mood_congruent
                            crate::rpc::RuntimeCommonFields::default(),
                            request_correlation_id,
                            &state,
                        ) {
                            Ok(response) => JsonRpcResponse::success(request_id, json!(response)),
                            Err(message) => JsonRpcResponse::error(request_id, -32602, message, None),
                        }
                    }
                    "inspect" => {
                        let id = arguments.get("id").and_then(|v| v.as_u64()).unwrap_or(0);
                        let namespace = arguments.get("namespace").and_then(|v| v.as_str()).unwrap_or("default").to_string();

                        match Self::handle_inspect(
                            id,
                            &namespace,
                            crate::rpc::RuntimeCommonFields::default(),
                            request_correlation_id,
                            &state,
                        ) {
                            Ok(response) => JsonRpcResponse::success(request_id, json!(response)),
                            Err(message) => JsonRpcResponse::error(request_id, -32602, message, None),
                        }
                    }
                    "why" => {
                        let query = arguments.get("query").and_then(|v| v.as_str()).unwrap_or("");
                        let namespace = arguments.get("namespace").and_then(|v| v.as_str()).unwrap_or("default").to_string();
                        let limit = arguments.get("limit").and_then(|v| v.as_u64()).map(|n| n as usize);

                        match Self::handle_explain(
                            query,
                            &namespace,
                            limit,
                            None, // depth
                            crate::rpc::RuntimeCommonFields::default(),
                            request_correlation_id,
                            &state,
                        ) {
                            Ok(response) => JsonRpcResponse::success(request_id, json!(response)),
                            Err(message) => JsonRpcResponse::error(request_id, -32602, message, None),
                        }
                    }
                    "health" => {
                        let report = state.health_report().await;
                        JsonRpcResponse::success(request_id, report)
                    }
                    "doctor" => {
                        let report = state.doctor_report().await;
                        JsonRpcResponse::success(request_id, json!(report))
                    }
                    _ => {
                        JsonRpcResponse::error(
                            request_id,
                            -32601,
                            format!("unknown tool: {}", name),
                            None,
                        )
                    }
                }
            }
            RuntimeRequest::McpResourcesList => {
                JsonRpcResponse::success(
                    request_id,
                    json!({
                        "resources": [
                            {
                                "uri": "membrain://default/status",
                                "name": "Memory System Status",
                                "description": "Current status and statistics of the memory system",
                                "mimeType": "application/json"
                            }
                        ]
                    }),
                )
            }
            RuntimeRequest::McpResourcesRead { uri } => {
                if uri == "membrain://default/status" {
                    let status = state.status().await;
                    JsonRpcResponse::success(
                        request_id,
                        json!({
                            "contents": [
                                {
                                    "uri": uri,
                                    "mimeType": "application/json",
                                    "text": serde_json::to_string(&status).unwrap_or_default()
                                }
                            ]
                        }),
                    )
                } else {
                    JsonRpcResponse::error(
                        request_id,
                        -32602,
                        format!("unknown resource: {}", uri),
                        None,
                    )
                }
            }
            RuntimeRequest::McpPromptsList => {
                JsonRpcResponse::success(
                    request_id,
                    json!({
                        "prompts": []
                    }),
                )
            }
            RuntimeRequest::McpPromptsGet { name, arguments: _ } => {
                JsonRpcResponse::error(
                    request_id,
                    -32601,
                    format!("unknown prompt: {}", name),
                    None,
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
        common: crate::rpc::RuntimeCommonFields,
        request_correlation_id: u64,
        state: &RuntimeState,
    ) -> Result<McpResponse, String> {
        let namespace_id = NamespaceId::new(namespace).map_err(|err| err.to_string())?;
        let normalized = Self::normalize_recall_contract(
            namespace_id,
            query_text,
            like_id,
            unlike_id,
            graph_mode.as_deref(),
            result_budget,
            mood_congruent,
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
            &common,
            &normalized,
        );
        Self::handle_retrieval_method(
            normalized.planner_request,
            Some(&normalized.normalized_query_by_example),
            normalized.mood_congruent,
            namespace,
            Some(normalized.result_budget),
            mode.as_deref().or(effort.as_deref()),
            common,
            request_correlation_id,
            "recall",
            &degraded_summary,
            state,
        )
    }

    fn handle_context_budget(
        token_budget: usize,
        namespace: &str,
        current_context: Option<String>,
        working_memory_ids: Option<Vec<u64>>,
        format: Option<String>,
        mood_congruent: Option<bool>,
        common: crate::rpc::RuntimeCommonFields,
        request_correlation_id: u64,
        state: &RuntimeState,
    ) -> Result<McpResponse, String> {
        let namespace = NamespaceId::new(namespace).map_err(|err| err.to_string())?;
        let format = match format.as_deref().unwrap_or("plain") {
            "plain" => InjectionFormat::Plain,
            "markdown" => InjectionFormat::Markdown,
            "json" => InjectionFormat::Json,
            other => {
                return Err(format!(
                    "invalid format `{other}`; expected plain, markdown, or json"
                ));
            }
        };

        let query = current_context.clone().unwrap_or_default();
        let normalized = Self::normalize_recall_contract(
            namespace.clone(),
            Some(query),
            None,
            None,
            None,
            Some(RuntimeConfig::default().tier1_candidate_budget),
            mood_congruent,
        )?;
        let degraded_summary = Self::recall_degraded_summary(
            &None,
            Some(token_budget),
            common.time_budget_ms,
            &current_context,
            &None,
            common.policy_context.as_ref().map(|ctx| ctx.include_public),
            &None,
            None,
            common.workspace_id.as_deref(),
            common.agent_id.as_deref(),
            common.session_id.as_deref(),
            common.task_id.as_deref(),
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            &common,
            &normalized,
        );
        let result_set = Self::build_context_budget_result_set(
            &namespace,
            &common,
            &normalized.normalized_query_by_example,
            normalized.mood_congruent,
            &degraded_summary,
            state,
        );

        let request = ContextBudgetRequest {
            token_budget,
            current_context: current_context.clone(),
            working_memory_ids: working_memory_ids
                .clone()
                .unwrap_or_default()
                .into_iter()
                .map(MemoryId)
                .collect(),
            format,
        };
        let budget = membrain_core::BrainStore::default().context_budget(&request, &result_set);

        let mut response = ResponseContext::success(
            namespace.clone(),
            RequestId::new(format!("daemon-context-budget-{request_correlation_id}"))
                .map_err(|err| err.to_string())?,
            budget,
        );
        if response
            .result
            .as_ref()
            .is_some_and(|result| result.partial_success)
        {
            response = response.with_partial_success();
            response.warnings.push(ResponseWarning::new(
                "budget_exhausted",
                "token budget truncated otherwise eligible injections",
            ));
        }
        Ok(McpResponse::success(
            serde_json::to_value(ContextBudgetParams {
                token_budget,
                namespace: namespace.as_str().to_string(),
                current_context,
                working_memory_ids,
                format: Some(request.format.as_str().to_string()),
                common: crate::mcp::CommonRequestFields {
                    request_id: Some(response.request_id.as_str().to_string()),
                    workspace_id: common.workspace_id,
                    agent_id: common.agent_id,
                    session_id: common.session_id,
                    task_id: common.task_id,
                    time_budget_ms: common.time_budget_ms,
                    policy_context: common.policy_context.map(|ctx| {
                        crate::mcp::PolicyContextHint {
                            include_public: ctx.include_public,
                            sharing_visibility: ctx.sharing_visibility,
                            caller_identity_bound: ctx.caller_identity_bound,
                            workspace_acl_allowed: ctx.workspace_acl_allowed,
                            agent_acl_allowed: ctx.agent_acl_allowed,
                            session_visibility_allowed: ctx.session_visibility_allowed,
                            legal_hold: ctx.legal_hold,
                        }
                    }),
                },
            })
            .map_err(|err| err.to_string())?
            .as_object()
            .cloned()
            .map(|mut payload| {
                let mut result_value = serde_json::to_value(response.result)
                    .expect("context budget result should serialize");
                if let serde_json::Value::Object(result_object) = &mut result_value {
                    result_object.insert(
                        "explain".to_string(),
                        serde_json::to_value(&result_set.explain)
                            .expect("context budget explain should serialize"),
                    );
                }
                payload.insert("result".to_string(), result_value);
                payload.insert(
                    "partial_success".to_string(),
                    json!(response.partial_success),
                );
                payload.insert("warnings".to_string(), json!(response.warnings));
                serde_json::Value::Object(payload)
            })
            .expect("context budget params should serialize to object"),
        ))
    }

    fn handle_goal_state(
        namespace: &str,
        task_id: Option<String>,
        common: crate::rpc::RuntimeCommonFields,
        request_correlation_id: u64,
        state: &RuntimeState,
    ) -> Result<GoalStateOutput, String> {
        let namespace = NamespaceId::new(namespace).map_err(|err| err.to_string())?;
        let request_context = RuntimeState::runtime_request_context(
            namespace.clone(),
            &common,
            SharingVisibility::Private,
            format!("daemon-goal-state-{request_correlation_id}"),
        );
        let task_id = request_context
            .task_id
            .clone()
            .or_else(|| task_id.clone().or(common.task_id.clone()).map(TaskId::new))
            .unwrap_or_else(|| TaskId::new("active-goal"));
        let mut audit = state
            .audit
            .lock()
            .expect("runtime audit lock should be available");
        RuntimeState::ensure_goal_working_state(
            &mut audit.store,
            namespace.clone(),
            task_id.clone(),
        );
        audit
            .store
            .goal_state(&task_id)
            .ok_or_else(|| "goal working state should exist".to_string())
    }

    fn handle_hot_paths(
        namespace: &str,
        top_n: Option<usize>,
        _common: crate::rpc::RuntimeCommonFields,
        state: &RuntimeState,
    ) -> Result<HotPathsOutput, String> {
        let namespace = NamespaceId::new(namespace).map_err(|err| err.to_string())?;
        let top_n = top_n.unwrap_or(10);
        let audit = state
            .audit
            .lock()
            .expect("runtime audit lock should be available");
        Ok(audit.store.hot_paths(namespace, top_n))
    }

    fn handle_dead_zones(
        namespace: &str,
        min_age_ticks: Option<u64>,
        _common: crate::rpc::RuntimeCommonFields,
        state: &RuntimeState,
    ) -> Result<DeadZonesOutput, String> {
        let namespace = NamespaceId::new(namespace).map_err(|err| err.to_string())?;
        let min_age_ticks = min_age_ticks.unwrap_or(1);
        let audit = state
            .audit
            .lock()
            .expect("runtime audit lock should be available");
        Ok(audit.store.dead_zones(namespace, min_age_ticks))
    }

    fn handle_compress(
        namespace: &str,
        dry_run: bool,
        _common: crate::rpc::RuntimeCommonFields,
        state: &RuntimeState,
    ) -> Result<Value, String> {
        let namespace = NamespaceId::new(namespace).map_err(|err| err.to_string())?;
        let mut audit = state
            .audit
            .lock()
            .expect("runtime audit lock should be available");
        let applied =
            audit
                .store
                .apply_compression_pass(namespace.clone(), Default::default(), 3, dry_run);

        let compression_log_entries = audit
            .store
            .compression_log_entries(namespace.clone(), None)
            .into_iter()
            .map(|entry| {
                json!({
                    "schema_memory_id": entry.schema_memory_id.0,
                    "source_memory_count": entry.source_memory_count,
                    "tick": entry.tick,
                    "namespace": entry.namespace.as_str(),
                    "keyword_summary": entry.keyword_summary,
                })
            })
            .collect::<Vec<_>>();

        Ok(json!({
            "status": "accepted",
            "namespace": namespace.as_str(),
            "dry_run": applied.dry_run,
            "schemas_created": applied.schemas_created,
            "episodes_compressed": applied.episodes_compressed,
            "storage_reduction_pct": applied.storage_reduction_pct,
            "blocked_reasons": applied.blocked_reasons,
            "related_run": applied.related_run,
            "decision": {
                "cluster_id": applied.decision.cluster_id,
                "disposition": applied.decision.disposition.as_str(),
                "reason_code": applied.decision.reason_code,
                "coherence_millis": applied.decision.coherence_millis,
                "majority_kind": applied.decision.majority_kind.as_str(),
                "source_memory_ids": applied.decision.source_memory_ids.into_iter().map(|id| id.0).collect::<Vec<_>>(),
                "representative_memory_ids": applied.decision.representative_memory_ids.into_iter().map(|id| id.0).collect::<Vec<_>>(),
                "protected_source_ids": applied.decision.protected_source_ids.into_iter().map(|id| id.0).collect::<Vec<_>>(),
                "review_source_ids": applied.decision.review_source_ids.into_iter().map(|id| id.0).collect::<Vec<_>>(),
                "dominant_keywords": applied.decision.dominant_keywords,
                "authoritative_truth": applied.decision.authoritative_truth,
                "inspect_path": applied.decision.inspect_path,
            },
            "schema_artifact": applied.schema_artifact.map(|artifact| json!({
                "schema_memory_id": artifact.schema_memory_id.0,
                "cluster_id": artifact.cluster_id,
                "compact_text": artifact.compact_text,
                "source_memory_ids": artifact.source_memory_ids.into_iter().map(|id| id.0).collect::<Vec<_>>(),
                "compressed_member_ids": artifact.compressed_member_ids.into_iter().map(|id| id.0).collect::<Vec<_>>(),
                "dominant_keywords": artifact.dominant_keywords,
                "confidence_millis": artifact.confidence_millis,
                "source_lineage_paths": artifact.source_lineage_paths,
                "compressed_into_paths": artifact.compressed_into_paths,
                "inspect_path": artifact.inspect_path,
            })),
            "verification": applied.verification.map(|report| json!({
                "schema_memory_id": report.schema_memory_id.0,
                "verified": report.verified,
                "expected_source_count": report.expected_source_count,
                "reconstructed_source_count": report.reconstructed_source_count,
                "missing_source_ids": report.missing_source_ids.into_iter().map(|id| id.0).collect::<Vec<_>>(),
                "authoritative_truth": report.authoritative_truth,
                "verification_rule": report.verification_rule,
                "inspect_path": report.inspect_path,
            })),
            "compression_log_entry": applied.compression_log_entry.map(|entry| json!({
                "schema_memory_id": entry.schema_memory_id.0,
                "source_memory_count": entry.source_memory_count,
                "tick": entry.tick,
                "namespace": entry.namespace.as_str(),
                "keyword_summary": entry.keyword_summary,
            })),
            "compression_log_entries": compression_log_entries,
            "operator_log": applied.operator_log,
        }))
    }

    fn handle_schemas(
        namespace: &str,
        top_n: Option<usize>,
        _common: crate::rpc::RuntimeCommonFields,
        state: &RuntimeState,
    ) -> Result<Value, String> {
        let namespace = NamespaceId::new(namespace).map_err(|err| err.to_string())?;
        let top_n = top_n.unwrap_or(5).max(1);
        let audit = state
            .audit
            .lock()
            .expect("runtime audit lock should be available");
        let status = audit.store.compression_engine().status(
            namespace.clone(),
            CompressionTrigger::Manual,
            CompressionPolicy::default(),
            None,
        );
        let schemas = audit
            .store
            .compression_engine()
            .candidate_clusters(&status)
            .into_iter()
            .filter_map(|cluster| {
                let decision = audit
                    .store
                    .compression_engine()
                    .evaluate_candidate(&status, &cluster);
                if decision.disposition.as_str() == "eligible" {
                    audit.store
                        .compression_engine()
                        .apply_candidate(&status, &cluster, true)
                        .schema_artifact
                        .map(|artifact| {
                            json!({
                                "id": artifact.schema_memory_id.0,
                                "content": artifact.compact_text,
                                "source_count": artifact.source_memory_ids.len(),
                                "confidence": artifact.confidence_millis,
                                "keywords": artifact.dominant_keywords,
                                "compressed_member_ids": artifact.compressed_member_ids.into_iter().map(|id| id.0).collect::<Vec<_>>(),
                                "inspect_path": artifact.inspect_path,
                            })
                        })
                } else {
                    None
                }
            })
            .take(top_n)
            .collect::<Vec<_>>();

        Ok(json!({
            "status": "accepted",
            "namespace": namespace.as_str(),
            "top": top_n,
            "schemas": schemas,
        }))
    }

    fn handle_goal_pause(
        namespace: &str,
        task_id: Option<String>,
        note: Option<String>,
        common: crate::rpc::RuntimeCommonFields,
        _request_correlation_id: u64,
        state: &RuntimeState,
    ) -> Result<GoalPauseOutput, String> {
        let namespace = NamespaceId::new(namespace).map_err(|err| err.to_string())?;
        let task_id = task_id
            .clone()
            .or(common.task_id.clone())
            .map(TaskId::new)
            .unwrap_or_else(|| TaskId::new("active-goal"));
        let mut audit = state
            .audit
            .lock()
            .expect("runtime audit lock should be available");
        RuntimeState::ensure_goal_working_state(&mut audit.store, namespace, task_id.clone());
        audit
            .store
            .goal_pause(&task_id, note)
            .ok_or_else(|| "goal working state should exist".to_string())
    }

    fn handle_goal_pin(
        namespace: &str,
        task_id: Option<String>,
        memory_id: u64,
        common: crate::rpc::RuntimeCommonFields,
        _request_correlation_id: u64,
        state: &RuntimeState,
    ) -> Result<GoalStateOutput, String> {
        let namespace = NamespaceId::new(namespace).map_err(|err| err.to_string())?;
        let task_id = task_id
            .clone()
            .or(common.task_id.clone())
            .map(TaskId::new)
            .unwrap_or_else(|| TaskId::new("active-goal"));
        let mut audit = state
            .audit
            .lock()
            .expect("runtime audit lock should be available");
        RuntimeState::ensure_goal_working_state(&mut audit.store, namespace, task_id.clone());
        audit
            .store
            .blackboard_pin(&task_id, MemoryId(memory_id))
            .ok_or_else(|| "goal working state should exist".to_string())
    }

    fn handle_goal_dismiss(
        namespace: &str,
        task_id: Option<String>,
        memory_id: u64,
        common: crate::rpc::RuntimeCommonFields,
        _request_correlation_id: u64,
        state: &RuntimeState,
    ) -> Result<GoalStateOutput, String> {
        let namespace = NamespaceId::new(namespace).map_err(|err| err.to_string())?;
        let task_id = task_id
            .clone()
            .or(common.task_id.clone())
            .map(TaskId::new)
            .unwrap_or_else(|| TaskId::new("active-goal"));
        let mut audit = state
            .audit
            .lock()
            .expect("runtime audit lock should be available");
        RuntimeState::ensure_goal_working_state(&mut audit.store, namespace, task_id.clone());
        audit
            .store
            .blackboard_dismiss(&task_id, MemoryId(memory_id))
            .ok_or_else(|| "goal working state should exist".to_string())
    }

    fn handle_goal_snapshot(
        namespace: &str,
        task_id: Option<String>,
        note: Option<String>,
        common: crate::rpc::RuntimeCommonFields,
        request_correlation_id: u64,
        _state: &RuntimeState,
    ) -> Result<BlackboardSnapshotOutput, String> {
        let namespace = NamespaceId::new(namespace).map_err(|err| err.to_string())?;
        let blackboard = RuntimeState::materialize_blackboard_state(
            namespace.clone(),
            task_id.clone().or(common.task_id.clone()),
        );
        let snapshot = membrain_core::types::BlackboardSnapshotArtifact {
            snapshot_id: format!(
                "blackboard-snapshot-{}-{}",
                namespace.as_str(),
                task_id
                    .as_deref()
                    .or(common.task_id.as_deref())
                    .unwrap_or("default")
            ),
            created_tick: RuntimeState::current_tick(request_correlation_id),
            evidence_handles: blackboard
                .active_evidence
                .iter()
                .map(|handle| handle.memory_id)
                .collect(),
            note,
            artifact_kind: "blackboard_snapshot",
            authoritative_truth: "durable_memory",
        };
        Ok(BlackboardSnapshotOutput {
            snapshot,
            namespace: namespace.as_str().to_string(),
            authoritative_truth: "durable_memory",
        })
    }

    fn handle_goal_resume(
        namespace: &str,
        task_id: Option<String>,
        common: crate::rpc::RuntimeCommonFields,
        request_correlation_id: u64,
        state: &RuntimeState,
    ) -> Result<GoalResumeOutput, String> {
        let namespace = NamespaceId::new(namespace).map_err(|err| err.to_string())?;
        let task_id = task_id
            .clone()
            .or(common.task_id.clone())
            .map(TaskId::new)
            .unwrap_or_else(|| TaskId::new("active-goal"));
        let mut audit = state
            .audit
            .lock()
            .expect("runtime audit lock should be available");
        if audit.store.goal_state(&task_id).is_none() {
            let blackboard = RuntimeState::materialize_blackboard_state(
                namespace.clone(),
                Some(task_id.as_str().to_string()),
            );
            let mut checkpoint = RuntimeState::checkpoint_for_blackboard(
                namespace.clone(),
                Some(task_id.as_str().to_string()),
                &blackboard,
                RuntimeState::current_tick(request_correlation_id),
                GoalLifecycleStatus::Stale,
                true,
            );
            checkpoint.blackboard_summary = Some(
                "no persisted checkpoint found; active state is reconstructable only as an explicit stale projection"
                    .to_string(),
            );
            return Ok(GoalResumeOutput {
                task_id: checkpoint
                    .task_id
                    .as_ref()
                    .map(|task| FieldPresence::Present(task.as_str().to_string()))
                    .unwrap_or(FieldPresence::Absent),
                status: checkpoint.status,
                resumed_at_tick: RuntimeState::current_tick(request_correlation_id),
                restored_evidence_handles: checkpoint
                    .evidence_handles
                    .iter()
                    .map(|memory_id| memory_id.0)
                    .collect(),
                restored_dependencies: checkpoint.pending_dependencies.clone(),
                checkpoint,
                warnings: vec![ResponseWarning::new(
                    "stale_checkpoint",
                    "resume degraded explicitly because no valid persisted checkpoint was available",
                )],
                namespace: namespace.as_str().to_string(),
                authoritative_truth: "durable_memory",
            });
        }
        match audit.store.goal_resume(&task_id) {
            Ok(output) => Ok(output),
            Err(warning) => {
                let blackboard = RuntimeState::materialize_blackboard_state(
                    namespace.clone(),
                    Some(task_id.as_str().to_string()),
                );
                let mut checkpoint = RuntimeState::checkpoint_for_blackboard(
                    namespace.clone(),
                    Some(task_id.as_str().to_string()),
                    &blackboard,
                    RuntimeState::current_tick(request_correlation_id),
                    GoalLifecycleStatus::Stale,
                    true,
                );
                checkpoint.blackboard_summary = Some(
                    "no persisted checkpoint found; active state is reconstructable only as an explicit stale projection"
                        .to_string(),
                );
                Ok(GoalResumeOutput {
                    task_id: checkpoint
                        .task_id
                        .as_ref()
                        .map(|task| FieldPresence::Present(task.as_str().to_string()))
                        .unwrap_or(FieldPresence::Absent),
                    status: checkpoint.status,
                    resumed_at_tick: RuntimeState::current_tick(request_correlation_id),
                    restored_evidence_handles: checkpoint
                        .evidence_handles
                        .iter()
                        .map(|memory_id| memory_id.0)
                        .collect(),
                    restored_dependencies: checkpoint.pending_dependencies.clone(),
                    checkpoint,
                    warnings: vec![ResponseWarning::new(
                        warning.as_str(),
                        "resume degraded explicitly because no valid persisted checkpoint was available",
                    )],
                    namespace: namespace.as_str().to_string(),
                    authoritative_truth: "durable_memory",
                })
            }
        }
    }

    fn handle_goal_abandon(
        namespace: &str,
        task_id: Option<String>,
        reason: Option<String>,
        common: crate::rpc::RuntimeCommonFields,
        _request_correlation_id: u64,
        state: &RuntimeState,
    ) -> Result<GoalAbandonOutput, String> {
        let namespace = NamespaceId::new(namespace).map_err(|err| err.to_string())?;
        let task_id = task_id
            .clone()
            .or(common.task_id.clone())
            .map(TaskId::new)
            .unwrap_or_else(|| TaskId::new("active-goal"));
        let mut audit = state
            .audit
            .lock()
            .expect("runtime audit lock should be available");
        RuntimeState::ensure_goal_working_state(&mut audit.store, namespace, task_id.clone());
        audit
            .store
            .goal_abandon(&task_id, reason)
            .ok_or_else(|| "goal working state should exist".to_string())
    }

    fn handle_inspect(
        id: u64,
        namespace: &str,
        _common: crate::rpc::RuntimeCommonFields,
        request_correlation_id: u64,
        state: &RuntimeState,
    ) -> Result<McpResponse, String> {
        let namespace = NamespaceId::new(namespace).map_err(|err| err.to_string())?;
        let request_id = RequestId::new(format!("daemon-inspect-{request_correlation_id}"))
            .map_err(|err| err.to_string())?;

        let Some(record) = state.memory_record(MemoryId(id)) else {
            return Ok(McpResponse::failure(crate::mcp::McpError {
                code: "validation_failure".to_string(),
                message: format!(
                    "memory {id} not found in namespace '{}'",
                    namespace.as_str()
                ),
                is_policy_denial: false,
            }));
        };

        let layout = &record.layout;
        let request = RuntimeState::runtime_request_context(
            namespace.clone(),
            &_common,
            layout.metadata.visibility,
            format!("daemon-inspect-{request_correlation_id}"),
        );
        let outcome = request
            .bind_namespace(None)
            .expect("inspect runtime namespace should already be valid")
            .evaluate_cross_namespace_sharing_access(&PolicyModule, &layout.metadata.namespace);
        if matches!(outcome.decision, SharingAccessDecision::Deny) {
            return Ok(McpResponse::failure(crate::mcp::McpError {
                code: "policy_denied".to_string(),
                message: "namespace isolation prevents inspect".to_string(),
                is_policy_denial: true,
            }));
        }

        let mut result = RetrievalResultSet::empty(
            RetrievalExplain::from_plan(
                &RecallEngine
                    .plan_recall(RecallRequest::exact(MemoryId(id)), RuntimeConfig::default()),
                "balanced",
            ),
            namespace.clone(),
        );
        result.outcome_class = OutcomeClass::Accepted;
        result.policy_summary.outcome_class = OutcomeClass::Accepted;
        result.policy_summary.redactions_applied =
            matches!(outcome.decision, SharingAccessDecision::Redact);
        result.policy_summary.filters =
            RuntimeState::share_policy_summary(&namespace, &outcome).filters;
        result.packaging_metadata.result_budget = 1;
        result.packaging_metadata.degraded_summary = None;

        let mut payload =
            McpInspectPayload::from_result(request_id, namespace.clone(), id, &result)
                .map_err(|err| err.to_string())?;
        payload.tier = layout.metadata.route_family.as_str().to_string();
        payload.lineage = json!({
            "source_kind": "memory",
            "source_reference": "memory_id",
            "lineage_ancestors": [],
            "relation_to_seed": "absent",
            "graph_seed": "absent"
        });
        payload.policy_flags =
            serde_json::to_value(RuntimeState::share_policy_summary(&namespace, &outcome))
                .map_err(|err| err.to_string())?;
        payload.lifecycle_state = json!({
            "outcome_class": OutcomeClass::Accepted,
            "truncated": false,
            "degraded_summary": null,
            "visibility": layout.metadata.visibility.as_str(),
            "memory_type": layout.metadata.memory_type.as_str(),
        });
        payload.index_presence = json!({
            "tiers_consulted": [layout.metadata.route_family.as_str()],
            "graph_assistance": "none",
            "sharing_visibility": layout.metadata.visibility.as_str(),
        });
        payload.graph_neighborhood_summary = json!({
            "graph_seed": null,
            "relation_to_seed": null,
            "graph_assistance": "none",
        });
        payload.decay_retention = json!({
            "oldest_item_days": 0,
            "newest_item_days": 0,
            "volatile_items_included": false,
            "stale_warning": false,
            "as_of_tick": null,
        });
        if let Some(passive) = state
            .memories
            .lock()
            .expect("runtime memory registry lock should be available")
            .get(&id)
            .and_then(|record| record.passive_observation.clone())
        {
            payload.explain_trace.passive_observation =
                Some(serde_json::to_value(passive).map_err(|err| err.to_string())?);
        } else {
            let inspect_resource_uri = format!("membrain://{}/memories/{id}", namespace.as_str());
            payload.explain_trace.passive_observation = Some(json!({
                "resource_uri": inspect_resource_uri,
                "resource_kind": "inspect_payload",
                "resource_template": INSPECT_RESOURCE_URI_TEMPLATE,
                "resource_examples": [format!("membrain://{}/memories/{id}", namespace.as_str())],
            }));
        }
        Ok(McpResponse::success(
            serde_json::to_value(payload).map_err(|err| err.to_string())?,
        ))
    }

    fn handle_invalidate(
        memory_id: MemoryId,
        namespace: &str,
        dry_run: bool,
        _common: crate::rpc::RuntimeCommonFields,
        request_correlation_id: u64,
        state: &RuntimeState,
    ) -> Result<serde_json::Value, String> {
        let namespace = NamespaceId::new(namespace).map_err(|err| err.to_string())?;
        let links = state
            .memories
            .lock()
            .expect("runtime memory registry lock should be available")
            .values()
            .filter(|record| record.layout.metadata.namespace == namespace)
            .flat_map(|record| {
                record
                    .causal_parents
                    .iter()
                    .copied()
                    .map(move |parent_id| CausalLink {
                        src_memory_id: parent_id,
                        dst_memory_id: record.layout.metadata.memory_id,
                        link_type: record.causal_link_type.unwrap_or(CausalLinkType::Derived),
                        created_at_ms: record.layout.metadata.memory_id.0,
                        agent_id: record
                            .layout
                            .metadata
                            .agent_id
                            .as_ref()
                            .map(|id| id.as_str().to_string()),
                        evidence: vec![CausalEvidenceAttribution {
                            evidence_kind: CausalEvidenceKind::DurableMemory,
                            source_ref: format!(
                                "memory://{}/{}",
                                record.layout.metadata.namespace.as_str(),
                                parent_id.0
                            ),
                            supporting_memory_ids: vec![parent_id],
                            confidence: 800,
                        }],
                    })
            })
            .collect::<Vec<_>>();
        let report = membrain_core::BrainStore::new(RuntimeConfig::default())
            .invalidate_causal_chain(memory_id, &links);
        let request_id = RequestId::new(format!("daemon-invalidate-{request_correlation_id}"))
            .map_err(|err| err.to_string())?;
        Ok(json!({
            "request_id": request_id.as_str(),
            "status": if dry_run { "preview" } else { "accepted" },
            "root_memory_id": memory_id.0,
            "namespace": namespace.as_str(),
            "dry_run": dry_run,
            "chain_length": report.chain_length,
            "memories_penalized": report.memories_penalized,
            "avg_confidence_delta": report.avg_confidence_delta,
            "steps": report.steps.iter().map(|step| json!({
                "memory_id": step.memory_id.0,
                "depth": step.depth,
                "confidence_delta": step.confidence_delta,
                "link_type": step.link_type.as_str(),
                "source_backed": step.source_backed,
                "evidence_log": step.evidence_log,
            })).collect::<Vec<_>>(),
            "policy_summary": {
                "effective_namespace": namespace.as_str(),
                "policy_family": "causal_invalidation",
                "outcome_class": "accepted",
                "blocked_stage": "policy_gate",
                "redaction_fields": [],
                "retention_state": "absent",
                "sharing_scope": "private"
            },
            "result_reasons": report.steps.iter().map(|step| {
                let detail = format!(
                    "memory #{} receives bounded causal penalty {:.2} at depth {} via {} link",
                    step.memory_id.0,
                    step.confidence_delta,
                    step.depth,
                    step.link_type.as_str()
                );
                json!({
                    "memory_id": step.memory_id.0,
                    "reason_code": "causal_chain_invalidated",
                    "detail": detail,
                })
            }).collect::<Vec<_>>(),
            "trace_stages": ["tier2_exact", "graph_expansion", "packaging"],
        }))
    }

    fn handle_explain(
        query: &str,
        namespace: &str,
        limit: Option<usize>,
        depth: Option<usize>,
        common: crate::rpc::RuntimeCommonFields,
        request_correlation_id: u64,
        state: &RuntimeState,
    ) -> Result<McpResponse, String> {
        if let Ok(memory_id) = query.trim().parse::<u64>() {
            let namespace = NamespaceId::new(namespace).map_err(|err| err.to_string())?;
            let request_id = RequestId::new(format!("daemon-explain-{request_correlation_id}"))
                .map_err(|err| err.to_string())?;
            let links = state
                .memories
                .lock()
                .expect("runtime memory registry lock should be available")
                .values()
                .filter(|record| record.layout.metadata.namespace == namespace)
                .flat_map(|record| {
                    record
                        .causal_parents
                        .iter()
                        .copied()
                        .map(move |parent_id| CausalLink {
                            src_memory_id: parent_id,
                            dst_memory_id: record.layout.metadata.memory_id,
                            link_type: record.causal_link_type.unwrap_or(CausalLinkType::Derived),
                            created_at_ms: record.layout.metadata.memory_id.0,
                            agent_id: record
                                .layout
                                .metadata
                                .agent_id
                                .as_ref()
                                .map(|id| id.as_str().to_string()),
                            evidence: vec![CausalEvidenceAttribution {
                                evidence_kind: CausalEvidenceKind::DurableMemory,
                                source_ref: format!(
                                    "memory://{}/{}",
                                    record.layout.metadata.namespace.as_str(),
                                    parent_id.0
                                ),
                                supporting_memory_ids: vec![parent_id],
                                confidence: 800,
                            }],
                        })
                })
                .collect::<Vec<_>>();
            let requested_depth =
                depth.unwrap_or(RuntimeConfig::default().graph_max_depth as usize);
            let trace = membrain_core::BrainStore::new(RuntimeConfig::default())
                .graph()
                .trace_causality(
                    MemoryId(memory_id),
                    &links,
                    requested_depth,
                    RuntimeConfig::default().graph_max_nodes,
                );
            let current_mood = current_mood_snapshot(state, &namespace);
            let mut explain = RetrievalExplain::from_plan(
                &RecallEngine.plan_recall(
                    RecallRequest::exact(MemoryId(memory_id)).with_graph_expansion(true),
                    RuntimeConfig::default(),
                ),
                ranking_profile_name_for_recall(false, current_mood, None),
            );
            explain.route_reason =
                "bounded causal trace walked explicit source-backed links from the target memory"
                    .to_string();
            explain.query_by_example = Some(QueryByExampleExplain {
                primary_cue: "memory_id".to_string(),
                requested_seed_descriptors: vec![format!("memory:{memory_id}")],
                materialized_seed_descriptors: trace
                    .steps
                    .iter()
                    .map(|step| format!("memory:{}", step.memory_id.0))
                    .collect(),
                missing_seed_descriptors: Vec::new(),
                expanded_candidate_count: trace.steps.len(),
                influence_summary: format!(
                    "memory:{memory_id} expanded {} causal chain step(s) within bounded traversal caps",
                    trace.steps.len()
                ),
            });
            explain.result_reasons.extend(trace.steps.iter().map(|step| ResultReason {
                memory_id: Some(step.memory_id),
                reason_code: "query_by_example_seed_materialized".to_string(),
                detail: if step.depth == 0 {
                    format!(
                        "memory #{} is the causal trace seed; traversal starts here before following bounded parent links",
                        step.memory_id.0
                    )
                } else {
                    format!(
                        "memory #{} entered the bounded causal chain at depth {} via {:?}",
                        step.memory_id.0,
                        step.depth,
                        step.link_type
                    )
                },
            }));
            if trace.all_roots_valid {
                explain.result_reasons.push(ResultReason {
                    memory_id: None,
                    reason_code: "causal_chain_root_validated".to_string(),
                    detail: format!(
                        "causal trace resolved to source-backed root evidence {:?}",
                        trace.root_memory_ids
                    ),
                });
            }
            for cutoff in &trace.explain.cutoff_reasons {
                let (reason_code, detail) = match cutoff {
                    membrain_core::graph::CutoffReason::MaxDepthReached(depth) => (
                        "graph_cutoff_max_depth",
                        format!(
                            "causal traversal stopped at requested depth cap {requested_depth} (effective depth {depth})"
                        ),
                    ),
                    membrain_core::graph::CutoffReason::MaxNodesReached(nodes) => (
                        "graph_cutoff_budget",
                        format!(
                            "causal traversal stopped after reaching declared node cap {nodes}"
                        ),
                    ),
                    membrain_core::graph::CutoffReason::BudgetExhausted => (
                        "graph_cutoff_budget",
                        "causal traversal exhausted its bounded traversal budget".to_string(),
                    ),
                    membrain_core::graph::CutoffReason::PolicyNamespaceBlocked => (
                        "graph_cutoff_policy_namespace",
                        "causal traversal omitted one or more parents because policy blocked them"
                            .to_string(),
                    ),
                };
                explain.result_reasons.push(ResultReason {
                    memory_id: None,
                    reason_code: reason_code.to_string(),
                    detail,
                });
            }
            let mut result = RetrievalResultSet::empty(explain, namespace.clone());
            result.outcome_class = OutcomeClass::Accepted;
            result.policy_summary.outcome_class = OutcomeClass::Accepted;
            result.packaging_metadata.result_budget = trace.steps.len().max(1);
            result.packaging_metadata.graph_assistance =
                "graph_expanded_supporting_neighbors".to_string();
            result.packaging_metadata.degraded_summary = None;
            result.provenance_summary.graph_seed = Some(EntityId(memory_id));
            result.provenance_summary.relation_to_seed = Some(RelationKind::Causal);
            result.provenance_summary.lineage_ancestors = trace.root_memory_ids.clone();

            let layouts = state
                .memories
                .lock()
                .expect("runtime memory registry lock should be available")
                .values()
                .filter(|record| {
                    trace
                        .steps
                        .iter()
                        .any(|step| step.memory_id == record.layout.metadata.memory_id)
                })
                .cloned()
                .collect::<Vec<_>>();
            let mut builder = ResultBuilder::new(trace.steps.len().max(1), namespace.clone());
            for record in &layouts {
                let entry_lane = if record.layout.metadata.memory_id == MemoryId(memory_id) {
                    EntryLane::Exact
                } else {
                    EntryLane::Graph
                };
                let candidate_affect = record.layout.metadata.affect;
                result.explain.ranking_profile =
                    ranking_profile_name_for_recall(false, current_mood, candidate_affect)
                        .to_string();
                let ranking_input = adjusted_ranking_input_for_affect(
                    RankingInput {
                        recency: 900,
                        salience: 880,
                        strength: 860,
                        provenance: 700,
                        conflict: 500,
                        confidence: record.confidence_output.confidence,
                    },
                    false,
                    current_mood,
                    candidate_affect,
                );
                let ranking = fuse_scores(
                    ranking_input,
                    ranking_profile_for_recall(false, current_mood, candidate_affect),
                );
                builder.add_with_confidence(
                    record.layout.metadata.memory_id,
                    record.layout.metadata.namespace.clone(),
                    record.layout.metadata.session_id,
                    record.layout.metadata.memory_type,
                    record.layout.metadata.compact_text.clone(),
                    &ranking,
                    AnsweredFrom::Tier2Indexed,
                    &record.confidence_inputs,
                    &ConfidencePolicy::default(),
                );
                let _ = entry_lane;
            }
            let mut built = builder.build(result.explain.clone());
            for item in &mut built.evidence_pack {
                if item.result.memory_id == MemoryId(memory_id) {
                    item.result.entry_lane = EntryLane::Exact;
                    item.result.role = EvidenceRole::Primary;
                } else {
                    item.result.entry_lane = EntryLane::Graph;
                    item.result.role = EvidenceRole::Supporting;
                }
                item.provenance_summary.graph_seed = Some(EntityId(memory_id));
                item.provenance_summary.relation_to_seed = Some(RelationKind::Causal);
                item.provenance_summary.lineage_ancestors = trace.root_memory_ids.clone();
            }
            built.outcome_class = OutcomeClass::Accepted;
            built.policy_summary.outcome_class = OutcomeClass::Accepted;
            built.packaging_metadata.graph_assistance =
                "graph_expanded_supporting_neighbors".to_string();
            built.packaging_metadata.result_budget = trace.steps.len().max(1);
            built.provenance_summary.graph_seed = Some(EntityId(memory_id));
            built.provenance_summary.relation_to_seed = Some(RelationKind::Causal);
            built.provenance_summary.lineage_ancestors = trace.root_memory_ids.clone();
            let payload = McpRetrievalPayload::from_result(request_id, namespace, false, built)
                .map_err(|err| err.to_string())?;
            return Ok(McpResponse::retrieval_success(payload));
        }

        let normalized = Self::normalize_recall_contract(
            NamespaceId::new(namespace).map_err(|err| err.to_string())?,
            Some(query.to_string()),
            None,
            None,
            None,
            limit,
            None,
        )?;
        Self::handle_retrieval_method(
            normalized.planner_request,
            Some(&normalized.normalized_query_by_example),
            normalized.mood_congruent,
            namespace,
            Some(normalized.result_budget),
            None,
            common,
            request_correlation_id,
            "explain",
            "planner-only explain envelope; evidence hydration not implemented",
            state,
        )
    }

    fn build_context_budget_result_set(
        namespace: &NamespaceId,
        common: &crate::rpc::RuntimeCommonFields,
        query_by_example: &QueryByExampleNormalization,
        mood_congruent: bool,
        degraded_summary: &str,
        state: &RuntimeState,
    ) -> RetrievalResultSet {
        let current_mood = current_mood_snapshot(state, namespace);
        let mut explain = RetrievalExplain::from_plan(
            &RecallEngine.plan_recall(
                RecallRequest::small_session_lookup(SessionId(1)),
                RuntimeConfig::default(),
            ),
            ranking_profile_name_for_recall(mood_congruent, current_mood, None),
        );
        let mut builder = ResultBuilder::new(
            RuntimeConfig::default().tier1_candidate_budget,
            namespace.clone(),
        );
        let mut explain_candidate_affect = None;

        if query_by_example.has_example_seeds() {
            let requested_seed_descriptors = query_by_example.seed_descriptors();
            let influence_summary = format!(
                "primary cue {} expanded {} candidate(s) from {} requested seed(s)",
                query_by_example.primary_cue.as_str(),
                requested_seed_descriptors.len(),
                requested_seed_descriptors.len(),
            );
            explain.query_by_example = Some(QueryByExampleExplain {
                primary_cue: query_by_example.primary_cue.as_str().to_string(),
                requested_seed_descriptors: requested_seed_descriptors.clone(),
                materialized_seed_descriptors: requested_seed_descriptors.clone(),
                missing_seed_descriptors: Vec::new(),
                expanded_candidate_count: requested_seed_descriptors.len(),
                influence_summary: influence_summary.clone(),
            });
            explain.result_reasons.push(ResultReason {
                memory_id: None,
                reason_code: "query_by_example_candidate_expansion".to_string(),
                detail: influence_summary,
            });
        }

        let records = state
            .memories
            .lock()
            .expect("runtime memory registry lock should be available")
            .values()
            .cloned()
            .collect::<Vec<_>>();

        for record in records {
            let layout = &record.layout;
            let request_context = RuntimeState::runtime_request_context(
                namespace.clone(),
                common,
                layout.metadata.visibility,
                "context-budget".to_string(),
            );
            let sharing_outcome = request_context
                .bind_namespace(None)
                .expect("context budget namespace should already be valid")
                .evaluate_cross_namespace_sharing_access(&PolicyModule, &layout.metadata.namespace);
            if matches!(sharing_outcome.decision, SharingAccessDecision::Deny) {
                explain.result_reasons.push(ResultReason {
                    memory_id: Some(layout.metadata.memory_id),
                    reason_code: "policy_filtered".to_string(),
                    detail: "candidate omitted by namespace or sharing policy".to_string(),
                });
                continue;
            }
            let matched = query_by_example
                .normalized_query_text
                .as_deref()
                .map(|query| {
                    layout
                        .metadata
                        .compact_text
                        .to_lowercase()
                        .contains(&query.to_lowercase())
                })
                .unwrap_or(true);
            let candidate_affect = layout.metadata.affect;
            if explain_candidate_affect.is_none() {
                explain_candidate_affect = candidate_affect;
            }
            let ranking_input = adjusted_ranking_input_for_affect(
                RankingInput {
                    recency: 900,
                    salience: if matched { 880 } else { 420 },
                    strength: if matched { 820 } else { 520 },
                    provenance: 700,
                    conflict: 500,
                    confidence: record.confidence_output.confidence,
                },
                mood_congruent,
                current_mood,
                candidate_affect,
            );
            let ranking = fuse_scores(
                ranking_input,
                ranking_profile_for_recall(mood_congruent, current_mood, candidate_affect),
            );
            builder.add_with_confidence(
                layout.metadata.memory_id,
                layout.metadata.namespace.clone(),
                layout.metadata.session_id,
                layout.metadata.memory_type,
                layout.metadata.compact_text.clone(),
                &ranking,
                AnsweredFrom::Tier2Indexed,
                &record.confidence_inputs,
                &ConfidencePolicy::default(),
            );
            explain.result_reasons.push(ResultReason {
                memory_id: Some(layout.metadata.memory_id),
                reason_code: if matched {
                    "score_kept"
                } else {
                    "fallback_candidate"
                }
                .to_string(),
                detail: if matched {
                    "context budget shortlist matched compact text inside the bounded runtime store"
                        .to_string()
                } else {
                    "context budget shortlist retained a lower-priority fallback candidate"
                        .to_string()
                },
            });
        }

        let mut result = builder.build(explain);
        result.explain.ranking_profile =
            ranking_profile_name_for_recall(mood_congruent, current_mood, explain_candidate_affect)
                .to_string();
        let include_public = common
            .policy_context
            .as_ref()
            .map(|ctx| ctx.include_public)
            .unwrap_or(false);
        result.policy_summary.filters = vec![PolicyFilterSummary::new(
            namespace.as_str(),
            if include_public {
                "shared_public_widening"
            } else {
                "namespace_only"
            },
            result.outcome_class,
            "policy_gate",
            if include_public {
                FieldPresence::Present("approved_shared".to_string())
            } else {
                FieldPresence::Present("same_namespace".to_string())
            },
            FieldPresence::Absent,
            Vec::new(),
        )];
        result.packaging_metadata.result_budget = RuntimeConfig::default().tier1_candidate_budget;
        result.packaging_metadata.degraded_summary = Some(degraded_summary.to_string());
        result
    }

    fn handle_retrieval_method(
        request: RecallRequest,
        query_by_example: Option<&QueryByExampleNormalization>,
        mood_congruent: bool,
        namespace: &str,
        limit: Option<usize>,
        output_mode_label: Option<&str>,
        common: crate::rpc::RuntimeCommonFields,
        request_correlation_id: u64,
        method_name: &str,
        degraded_summary: &str,
        state: &RuntimeState,
    ) -> Result<McpResponse, String> {
        let namespace = NamespaceId::new(namespace).map_err(|err| err.to_string())?;
        let request_id = RequestId::new(format!("daemon-{method_name}-{request_correlation_id}"))
            .map_err(|err| err.to_string())?;
        let plan = RecallEngine.plan_recall(request, RuntimeConfig::default());
        let result_budget = Self::canonical_result_budget(&request, limit);
        let output_mode = output_mode_label
            .and_then(membrain_core::engine::result::DualOutputMode::from_label)
            .unwrap_or(membrain_core::engine::result::DualOutputMode::Balanced);
        let request_shape_hash = Self::request_shape_hash(
            &request,
            query_by_example,
            result_budget,
            output_mode_label,
            method_name,
        );
        let recall_started_at = Instant::now();
        let current_generations = Self::current_cache_generations();
        let mut cache = state
            .cache
            .lock()
            .expect("runtime cache lock should be available");
        let current_mood = current_mood_snapshot(state, &namespace);
        let explain_profile = ranking_profile_name_for_recall(mood_congruent, current_mood, None);
        let mut explain = RetrievalExplain::from_plan(&plan, explain_profile);
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
        let exact_match = request.exact_memory_id.and_then(|memory_id| {
            state
                .memory_record(memory_id)
                .filter(|record| record.layout.metadata.namespace == namespace)
                .map(|record| (memory_id, record))
        });
        let empty_query_by_example = QueryByExampleNormalization {
            normalized_query_text: None,
            primary_cue: PrimaryCue::QueryText,
            seeds: Vec::new(),
        };
        let fallback_result = if exact_match.is_none() {
            Some(Self::build_context_budget_result_set(
                &namespace,
                &common,
                query_by_example.unwrap_or(&empty_query_by_example),
                mood_congruent,
                degraded_summary,
                state,
            ))
        } else {
            None
        };
        if let Some(mut result) = fallback_result {
            let request_key = RuntimeState::cache_key_for_request(
                CacheFamily::ResultCache,
                &namespace,
                &common,
                request_shape_hash,
                Some(request_shape_hash),
            );
            let request_lookup = cache
                .store_for(CacheFamily::ResultCache)
                .expect("result cache store should exist")
                .lookup(&request_key, &current_generations);
            let request_trace = cache_eval_trace_from_lookup(&request_lookup, result_budget);
            result.packaging_metadata.result_budget = result_budget;
            result.packaging_metadata.degraded_summary = if result.evidence_pack.is_empty() {
                Some(degraded_summary.to_string())
            } else {
                None
            };
            if result.evidence_pack.is_empty() {
                result.outcome_class = membrain_core::observability::OutcomeClass::Degraded;
                result.policy_summary.outcome_class =
                    membrain_core::observability::OutcomeClass::Degraded;
            }
            result.truncated = false;
            let degraded = result.evidence_pack.is_empty();
            let mut payload =
                McpRetrievalPayload::from_result(request_id, namespace, false, result)
                    .map_err(|err| err.to_string())?;
            payload.explain_trace.cache_metrics = cache_metrics_json(
                vec![request_trace],
                usize::from(matches!(
                    request_lookup.event,
                    CacheEvent::Miss
                        | CacheEvent::Bypass
                        | CacheEvent::StaleWarning
                        | CacheEvent::Disabled
                )),
                None,
                payload.result.total_candidates,
                degraded,
            )
            .map_err(|err| err.to_string())?;
            return Ok(McpResponse::retrieval_success(payload));
        }
        let (memory_id, record) =
            exact_match.expect("exact match should be present when no fallback result was built");

        let layout = &record.layout;
        let request_context = RuntimeState::runtime_request_context(
            namespace.clone(),
            &common,
            layout.metadata.visibility,
            format!("daemon-{method_name}-{request_correlation_id}"),
        );
        let sharing_outcome = request_context
            .bind_namespace(None)
            .expect("retrieval runtime namespace should already be valid")
            .evaluate_cross_namespace_sharing_access(&PolicyModule, &layout.metadata.namespace);
        if matches!(sharing_outcome.decision, SharingAccessDecision::Deny) {
            let mut audit_entry = AuditLogEntry::new(
                AuditEventCategory::Recall,
                AuditEventKind::RecallDenied,
                namespace.clone(),
                "daemon_recall",
                format!(
                    "request_id={} memory_id={} reason=policy_denied route={} query_hash={:016x}",
                    request_id.as_str(),
                    memory_id.0,
                    plan.route_summary.reason,
                    request_shape_hash
                ),
            )
            .with_memory_id(memory_id)
            .with_request_id(request_id.as_str());
            if let Some(session_id) = request.session_id {
                audit_entry = audit_entry.with_session_id(session_id);
            }
            state.append_runtime_audit_entry(namespace.clone(), audit_entry);
            return Ok(McpResponse::failure(crate::mcp::McpError {
                code: "policy_denied".to_string(),
                message: "namespace isolation prevents retrieval".to_string(),
                is_policy_denial: true,
            }));
        }

        let confidence_inputs = &record.confidence_inputs;
        let confidence_output = &record.confidence_output;
        let prefetch_hint = cache
            .prefetch
            .consume_matching_hint(&namespace, |hint| hint.predicted_ids.contains(&memory_id));
        let mut cache_traces = Vec::new();
        let mut prefetch_added_candidates = None;
        if let Some(hint) = prefetch_hint {
            let prefetch_lookup = RuntimeState::cache_lookup_result_for_prefetch(
                hint.predicted_ids.clone(),
                result_budget,
            );
            prefetch_added_candidates = Some(hint.predicted_ids.len());
            cache_traces.push(cache_eval_trace_from_lookup(
                &prefetch_lookup,
                result_budget,
            ));
        }
        let tier1_key = RuntimeState::cache_key_for_memory(CacheFamily::Tier1Item, &record);
        let tier1_lookup = cache
            .store_for(CacheFamily::Tier1Item)
            .expect("tier1 cache store should exist")
            .lookup(&tier1_key, &current_generations);
        let tier1_candidates_before = cache_traces
            .last()
            .map(|trace| trace.candidates_after)
            .unwrap_or(result_budget);
        cache_traces.push(cache_eval_trace_from_lookup(
            &tier1_lookup,
            tier1_candidates_before,
        ));
        let current_mood = current_mood_snapshot(state, &namespace);
        let candidate_affect = layout.metadata.affect;
        let ranking_input = adjusted_ranking_input_for_affect(
            RankingInput {
                recency: 900,
                salience: 880,
                strength: 860,
                provenance: 700,
                conflict: 500,
                confidence: confidence_output.confidence,
            },
            mood_congruent,
            current_mood,
            candidate_affect,
        );
        let ranking = fuse_scores(
            ranking_input,
            ranking_profile_for_recall(mood_congruent, current_mood, candidate_affect),
        );
        let mut builder =
            ResultBuilder::new(result_budget, namespace.clone()).with_output_mode(output_mode);
        builder.add_with_confidence(
            layout.metadata.memory_id,
            layout.metadata.namespace.clone(),
            layout.metadata.session_id,
            layout.metadata.memory_type,
            layout.metadata.compact_text.clone(),
            &ranking,
            AnsweredFrom::Tier2Indexed,
            confidence_inputs,
            &ConfidencePolicy::default(),
        );
        builder.action_pack = Some(vec![membrain_core::engine::result::ActionArtifact {
            action_type: "inspect_runtime_evidence".to_string(),
            suggestion: format!(
                "Inspect evidence #{} before taking follow-up action: {}",
                layout.metadata.memory_id.0, layout.metadata.compact_text
            ),
            supporting_evidence: vec![layout.metadata.memory_id],
            confidence_score: confidence_output.confidence,
            uncertainty_markers: {
                let mut markers = Vec::new();
                if confidence_output.confidence < 500 {
                    markers.push("low_confidence".to_string());
                }
                if confidence_output.reconsolidation_uncertainty >= 500 {
                    markers.push("reconsolidation_churn".to_string());
                }
                if confidence_output.missing_evidence_uncertainty >= 500 {
                    markers.push("missing_evidence".to_string());
                }
                if markers.is_empty() {
                    markers.push(if confidence_output.uncertainty_score >= 500 {
                        "high_uncertainty".to_string()
                    } else {
                        "low_uncertainty".to_string()
                    });
                }
                markers
            },
            policy_caveats: Vec::new(),
            freshness_caveats: Vec::new(),
        }]);
        let mut result = builder.build(explain);
        result.explain.ranking_profile =
            ranking_profile_name_for_recall(mood_congruent, current_mood, candidate_affect)
                .to_string();
        result.policy_summary.filters =
            RuntimeState::share_policy_summary(&namespace, &sharing_outcome).filters;
        result.policy_summary.redactions_applied =
            matches!(sharing_outcome.decision, SharingAccessDecision::Redact);
        if matches!(sharing_outcome.decision, SharingAccessDecision::Redact) {
            if let Some(item) = result.evidence_pack.first_mut() {
                item.omitted_fields = sharing_outcome
                    .redaction_fields
                    .iter()
                    .map(|field| (*field).to_string())
                    .collect();
            }
        }
        result.packaging_metadata.result_budget = result_budget;
        result.packaging_metadata.degraded_summary = None;
        result.policy_summary.outcome_class = result.outcome_class;
        let request_key = RuntimeState::cache_key_for_request(
            CacheFamily::ResultCache,
            &namespace,
            &common,
            request_shape_hash,
            Some(request_shape_hash),
        );
        let result_lookup = cache
            .store_for(CacheFamily::ResultCache)
            .expect("result cache store should exist")
            .lookup(&request_key, &current_generations);
        let result_candidates_before = cache_traces
            .last()
            .map(|trace| trace.candidates_after)
            .unwrap_or(result_budget);
        cache_traces.push(cache_eval_trace_from_lookup(
            &result_lookup,
            result_candidates_before,
        ));
        let result_ids = result
            .evidence_pack
            .iter()
            .map(|item| item.result.memory_id)
            .collect::<Vec<_>>();
        let _ = cache
            .store_for(CacheFamily::ResultCache)
            .expect("result cache store should exist")
            .admit(
                request_key,
                result_ids,
                membrain_core::store::cache::CacheAdmissionRequest {
                    request_shape_hash: Some(request_shape_hash),
                    ..Default::default()
                },
            );
        if request.session_id.is_some() {
            let warmup_key = RuntimeState::cache_key_for_request(
                CacheFamily::SessionWarmup,
                &namespace,
                &common,
                memory_id.0,
                None,
            );
            let _ = cache
                .store_for(CacheFamily::SessionWarmup)
                .expect("session warmup cache store should exist")
                .admit(warmup_key, vec![memory_id], Default::default());
        }
        if request.graph_expansion {
            let ann_key = RuntimeState::cache_key_for_request(
                CacheFamily::AnnProbeCache,
                &namespace,
                &common,
                request_shape_hash,
                Some(request_shape_hash),
            );
            let ann_lookup = cache
                .store_for(CacheFamily::AnnProbeCache)
                .expect("ann probe cache store should exist")
                .lookup(&ann_key, &current_generations);
            let ann_candidates_before = cache_traces
                .last()
                .map(|trace| trace.candidates_after)
                .unwrap_or(result_budget);
            cache_traces.push(cache_eval_trace_from_lookup(
                &ann_lookup,
                ann_candidates_before,
            ));
            let _ = cache
                .store_for(CacheFamily::AnnProbeCache)
                .expect("ann probe cache store should exist")
                .admit(
                    ann_key,
                    vec![memory_id],
                    membrain_core::store::cache::CacheAdmissionRequest {
                        request_shape_hash: Some(request_shape_hash),
                        ..Default::default()
                    },
                );
        }
        let predicted_hot_path = state
            .audit
            .lock()
            .expect("runtime audit state lock should be available")
            .store
            .predict_recall_hot_path(&namespace, memory_id, request.session_id);
        if !request.graph_expansion {
            if let Some(predicted) = predicted_hot_path.as_ref() {
                let prefetch_trigger = match predicted.prewarm_trigger {
                    "session_recency" => Some(PrefetchTrigger::SessionRecency),
                    "task_intent" => Some(PrefetchTrigger::TaskIntent),
                    "entity_follow" => Some(PrefetchTrigger::EntityFollow),
                    _ => None,
                };
                if predicted.prewarm_action != "observe_only" {
                    if let Some(prefetch_trigger) = prefetch_trigger {
                        let _ = cache.prefetch.submit_hint(
                            namespace.clone(),
                            prefetch_trigger,
                            vec![memory_id],
                        );
                    }
                }
            }
        }
        let predicted_heat_bucket = predicted_hot_path
            .as_ref()
            .map(|entry| entry.heat_bucket)
            .unwrap_or("idle");
        let predicted_prewarm_action = predicted_hot_path
            .as_ref()
            .map(|entry| entry.prewarm_action)
            .unwrap_or("observe_only");
        let predicted_prewarm_target = predicted_hot_path
            .as_ref()
            .map(|entry| entry.prewarm_target_family)
            .unwrap_or("none");
        let predicted_attention_score = predicted_hot_path
            .as_ref()
            .map(|entry| entry.attention_score)
            .unwrap_or(0);
        let recall_latency_us = recall_started_at.elapsed().as_micros() as u64;
        drop(cache);
        let mut audit_entry = AuditLogEntry::new(
            AuditEventCategory::Recall,
            AuditEventKind::RecallServed,
            namespace.clone(),
            "daemon_recall",
            format!(
                "request_id={} memory_id={} route={} result_budget={} query_hash={:016x} latency_us={} retrieved_ids=[{}] heat_bucket={} prewarm_action={} prewarm_target={} attention_score={}",
                request_id.as_str(),
                memory_id.0,
                plan.route_summary.reason,
                result_budget,
                request_shape_hash,
                recall_latency_us,
                memory_id.0,
                predicted_heat_bucket,
                predicted_prewarm_action,
                predicted_prewarm_target,
                predicted_attention_score
            ),
        )
        .with_memory_id(memory_id)
        .with_request_id(request_id.as_str());
        if let Some(session_id) = request.session_id {
            audit_entry = audit_entry.with_session_id(session_id);
        }
        state.append_runtime_audit_entry(namespace.clone(), audit_entry);
        let mut payload = McpRetrievalPayload::from_result(request_id, namespace, false, result)
            .map_err(|err| err.to_string())?;
        let cold_fallback_count = usize::from(matches!(
            tier1_lookup.event,
            CacheEvent::Miss | CacheEvent::Bypass | CacheEvent::StaleWarning | CacheEvent::Disabled
        ));
        payload.explain_trace.cache_metrics = cache_metrics_json(
            cache_traces,
            cold_fallback_count,
            prefetch_added_candidates,
            0,
            false,
        )
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
        mood_congruent: Option<bool>,
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
            mood_congruent: mood_congruent.unwrap_or(false),
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
        common: &crate::rpc::RuntimeCommonFields,
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
        if common.request_id.is_some() {
            facets.push("request_id");
        }
        if common.policy_context.is_some() {
            facets.push("policy_context");
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

    fn handle_preflight_run(
        namespace: &str,
        original_query: &str,
        proposed_action: &str,
        request_correlation_id: u64,
    ) -> Result<PreflightOutcome, String> {
        let evaluated = evaluate_shared_preflight(
            namespace,
            original_query,
            proposed_action,
            request_correlation_id,
            false,
            false,
            "daemon_jsonrpc",
            "daemon",
        )?;
        Ok(to_shared_preflight_outcome(evaluated, false))
    }

    fn handle_preflight_explain(
        namespace: &str,
        original_query: &str,
        proposed_action: &str,
        request_correlation_id: u64,
    ) -> Result<PreflightExplainResponse, String> {
        let evaluated = evaluate_shared_preflight(
            namespace,
            original_query,
            proposed_action,
            request_correlation_id,
            false,
            true,
            "daemon_jsonrpc",
            "daemon",
        )?;
        Ok(to_shared_preflight_explain_response(namespace, evaluated))
    }

    fn handle_preflight_allow(
        namespace: &str,
        original_query: &str,
        proposed_action: &str,
        authorization_token: &str,
        bypass_flags: &[String],
        request_correlation_id: u64,
    ) -> Result<PreflightOutcome, String> {
        run_shared_preflight_allow(
            namespace,
            original_query,
            proposed_action,
            authorization_token,
            bypass_flags,
            request_correlation_id,
            "daemon_jsonrpc",
            "daemon",
        )
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
    use crate::mcp::{McpError, McpResponse};
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
        assert_eq!(status.authority_mode.as_str(), "unix_socket_daemon");
        assert!(status.authoritative_runtime);
        assert!(status.maintenance_active);
        assert!(status
            .warm_runtime_guarantees
            .contains(&"background_maintenance_loop".to_string()));
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
        assert_eq!(doctor_response["result"]["health"]["feature_availability"][1]["feature"], json!("runtime_authority"));
        assert!(doctor_response["result"]["health"]["feature_availability"][1]["note"]
            .as_str()
            .is_some_and(|note| note.contains("mode=unix_socket_daemon")));
        assert!(doctor_response["result"]["checks"]
            .as_array()
            .unwrap()
            .iter()
            .any(|check| check["name"] == json!("runtime_authority")
                && check["status"] == json!("ok")
                && check["affected_scope"] == json!("unix_socket_daemon")));
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
            json!("ok")
        );
        assert_eq!(
            doctor_response["result"]["warnings"],
            json!([
                "operator_review_recommended",
                "stale_action_critical_recheck_required"
            ])
        );
        assert_eq!(
            doctor_response["result"]["repair_reports"][0]["target"],
            json!("lexical_index")
        );
        assert_eq!(
            doctor_response["result"]["repair_reports"][1]["target"],
            json!("metadata_index")
        );
        assert_eq!(
            doctor_response["result"]["repair_reports"][0]["rebuild_entrypoint"],
            json!(null)
        );
        assert_eq!(
            doctor_response["result"]["repair_reports"][0]["affected_item_count"],
            json!(128)
        );
        assert_eq!(
            doctor_response["result"]["repair_reports"][0]["error_count"],
            json!(0)
        );
        assert_eq!(
            doctor_response["result"]["repair_reports"][0]["queue_depth_before"],
            json!(4)
        );
        assert_eq!(
            doctor_response["result"]["repair_reports"][0]["queue_depth_after"],
            json!(0)
        );
        assert_eq!(
            doctor_response["result"]["health"]["availability_posture"],
            json!("Degraded")
        );
        assert_eq!(
            doctor_response["result"]["health"]["repair_queue_depth"],
            json!(0)
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
    async fn runtime_encode_records_affect_history_and_serves_mood_history() {
        let socket_path = unique_path("encode-affect-history");
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

        let encode_response = send_request(
            &socket_path,
            json!({
                "jsonrpc":"2.0",
                "method":"encode",
                "params":{
                    "content":"emotionally salient incident summary",
                    "namespace":"default",
                    "emotional_annotations":{"valence":0.4,"arousal":0.9}
                },
                "id":"encode-affect"
            }),
        )
        .await;
        assert_eq!(encode_response["result"]["status"], json!("accepted"));

        let mood_history = send_request(
            &socket_path,
            json!({
                "jsonrpc":"2.0",
                "method":"mood_history",
                "params":{
                    "namespace":"default",
                    "since_tick":1
                },
                "id":"mood-history"
            }),
        )
        .await;
        assert_eq!(
            mood_history["result"]["authoritative_truth"],
            json!("emotional_timeline")
        );
        assert_eq!(mood_history["result"]["total_rows"], json!(1));

        let health = send_request(
            &socket_path,
            json!({"jsonrpc":"2.0","method":"health","params":{},"id":"health"}),
        )
        .await;
        assert!(health["result"]["dashboard_views"]
            .as_array()
            .expect("dashboard views should be present")
            .iter()
            .any(|view| {
                view["view"] == json!("affect_trajectory")
                    && view["summary"]
                        .as_str()
                        .is_some_and(|summary| summary.contains("history=/mood_history"))
            }));
        let avg_valence = mood_history["result"]["rows"][0]["avg_valence"]
            .as_f64()
            .expect("avg_valence should be numeric");
        let avg_arousal = mood_history["result"]["rows"][0]["avg_arousal"]
            .as_f64()
            .expect("avg_arousal should be numeric");
        assert!((avg_valence - 0.4).abs() < 1e-6);
        assert!((avg_arousal - 0.9).abs() < 1e-6);
        assert_eq!(
            mood_history["result"]["rows"][0]["authoritative_truth"],
            json!("emotional_timeline")
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
    async fn runtime_encode_accepts_visibility_metadata_for_cross_agent_sharing() {
        let socket_path = unique_path("encode-visibility");
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
                "method":"encode",
                "params":{
                    "content":"deploy checklist",
                    "namespace":"team.alpha",
                    "visibility":"shared",
                    "request_id":"encode-vis-1",
                    "policy_context":{"include_public":false,"sharing_visibility":"public"}
                },
                "id":"encode-visibility"
            }),
        )
        .await;
        assert_eq!(response["result"]["status"], json!("accepted"));
        assert_eq!(response["result"]["namespace"], json!("team.alpha"));
        assert_eq!(response["result"]["visibility"], json!("shared"));
        assert_eq!(
            response["result"]["message"],
            json!("encode envelope accepted with namespace-aware visibility metadata")
        );

        let policy_only_response = send_request(
            &socket_path,
            json!({
                "jsonrpc":"2.0",
                "method":"encode",
                "params":{
                    "content":"incident summary",
                    "namespace":"team.alpha",
                    "policy_context":{"include_public":false,"sharing_visibility":"public"}
                },
                "id":"encode-visibility-policy"
            }),
        )
        .await;
        assert_eq!(
            policy_only_response["result"]["visibility"],
            json!("public")
        );

        let invalid_visibility = send_request(
            &socket_path,
            json!({
                "jsonrpc":"2.0",
                "method":"encode",
                "params":{
                    "content":"bad visibility",
                    "namespace":"team.alpha",
                    "visibility":"clustered"
                },
                "id":"encode-invalid-visibility"
            }),
        )
        .await;
        assert_eq!(invalid_visibility["error"]["code"], json!(-32602));
        assert_eq!(
            invalid_visibility["error"]["message"],
            json!("invalid visibility")
        );
        assert_eq!(
            invalid_visibility["error"]["data"]["error_kind"],
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
    async fn runtime_forget_distinguishes_archive_restore_and_policy_delete() {
        let socket_path = unique_path("forget-semantics");
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

        let archive = send_request(
            &socket_path,
            json!({
                "jsonrpc":"2.0",
                "method":"forget",
                "params":{"id":42,"namespace":"team.alpha","mode":"archive"},
                "id":"forget-archive"
            }),
        )
        .await;
        assert_eq!(archive["result"]["action"], json!("archive"));
        assert_eq!(
            archive["result"]["policy_surface"],
            json!("utility_forgetting")
        );
        assert_eq!(archive["result"]["disposition"], json!("eligible"));
        assert_eq!(
            archive["result"]["reversibility"],
            json!("restore_required")
        );
        assert_eq!(
            archive["result"]["audit_kind"],
            json!("maintenance_forgetting_evaluated")
        );
        assert_eq!(archive["result"]["operator_review_required"], json!(false));
        assert_eq!(
            archive["result"]["resulting_archive_state"],
            json!("archived")
        );
        assert_eq!(
            archive["result"]["reason_code"],
            json!("below_forget_threshold")
        );

        let restore = send_request(
            &socket_path,
            json!({
                "jsonrpc":"2.0",
                "method":"forget",
                "params":{"id":42,"namespace":"team.alpha","mode":"restore_partial"},
                "id":"forget-restore"
            }),
        )
        .await;
        assert_eq!(restore["result"]["action"], json!("restore"));
        assert_eq!(
            restore["result"]["policy_surface"],
            json!("explicit_restore")
        );
        assert_eq!(restore["result"]["disposition"], json!("explicit"));
        assert_eq!(
            restore["result"]["audit_kind"],
            json!("maintenance_forgetting_evaluated")
        );
        assert_eq!(restore["result"]["partial_restore"], json!(true));
        assert_eq!(restore["result"]["prior_archive_state"], json!("archived"));

        let delete = send_request(
            &socket_path,
            json!({
                "jsonrpc":"2.0",
                "method":"forget",
                "params":{"id":42,"namespace":"team.alpha","mode":"delete"},
                "id":"forget-delete"
            }),
        )
        .await;
        assert_eq!(delete["result"]["action"], json!("policy_delete"));
        assert_eq!(delete["result"]["policy_surface"], json!("policy_delete"));
        assert_eq!(delete["result"]["disposition"], json!("explicit"));
        assert_eq!(
            delete["result"]["audit_kind"],
            json!("maintenance_forgetting_evaluated")
        );
        assert_eq!(delete["result"]["reversibility"], json!("irreversible"));

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
    async fn runtime_skills_returns_typed_payload() {
        let socket_path = unique_path("skills");
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

        let skills_response = send_request(
            &socket_path,
            json!({
                "jsonrpc":"2.0",
                "method":"skills",
                "params":{"namespace":"team.alpha","extract":true},
                "id":"skills"
            }),
        )
        .await;

        assert_eq!(skills_response["result"]["status"], json!("accepted"));
        assert_eq!(skills_response["result"]["namespace"], json!("team.alpha"));
        assert_eq!(
            skills_response["result"]["extraction_trigger"],
            json!("explicit_skill_extraction")
        );
        assert_eq!(
            skills_response["result"]["reflection_compiler_active"],
            json!(true)
        );
        assert!(
            skills_response["result"]["extracted_count"]
                .as_u64()
                .unwrap()
                >= 1
        );
        assert!(skills_response["result"]["procedures"].is_array());
        assert_eq!(
            skills_response["result"]["procedures"][0]["storage"]["storage_class"],
            json!("derived_durable_artifact")
        );
        assert_eq!(
            skills_response["result"]["procedures"][0]["review"]["derivation_rule"],
            json!("skill_extraction")
        );
        assert_eq!(
            skills_response["result"]["procedures"][0]["review"]["reflection"]["artifact_class"],
            json!("procedure")
        );
        assert_eq!(
            skills_response["result"]["procedures"][0]["recall"]["recall_surface"],
            json!("skills")
        );

        let procedures_response = send_request(
            &socket_path,
            json!({
                "jsonrpc":"2.0",
                "method":"procedures",
                "params":{
                    "namespace":"team.alpha",
                    "approved_by":"daemon.tester",
                    "note":"approved via daemon test"
                },
                "id":"procedures"
            }),
        )
        .await;

        assert_eq!(procedures_response["result"]["status"], json!("accepted"));
        assert_eq!(
            procedures_response["result"]["namespace"],
            json!("team.alpha")
        );
        assert_eq!(procedures_response["result"]["procedural_count"], json!(1));
        assert_eq!(
            procedures_response["result"]["direct_lookup_supported"],
            json!(true)
        );
        assert_eq!(
            procedures_response["result"]["procedures"][0]["storage"]["storage_class"],
            json!("procedural_durable_surface")
        );
        assert_eq!(
            procedures_response["result"]["procedures"][0]["review"]["accepted_by"],
            json!("daemon.tester")
        );
        assert_eq!(
            procedures_response["result"]["procedures"][0]["recall"]["recall_surface"],
            json!("procedural_store")
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
        assert!(
            recall_response["result"]["retrieval"]["result"]["evidence_pack"][0]["result"]
                ["uncertainty_markers"]["confidence_interval"]
                .is_null()
        );
        assert!(
            recall_response["result"]["retrieval"]["result"]["evidence_pack"][0]["result"]
                ["uncertainty_markers"]["corroboration_uncertainty"]
                .is_null()
        );
        assert_eq!(
            recall_response["result"]["retrieval"]["explain_trace"]["cache_metrics"]
                ["cache_hit_count"],
            json!(0)
        );
        assert_eq!(
            recall_response["result"]["retrieval"]["explain_trace"]["cache_metrics"]
                ["tier1_item_hit_count"],
            json!(0)
        );
        assert_eq!(
            recall_response["result"]["retrieval"]["explain_trace"]["cache_metrics"]
                ["cold_fallback_count"],
            json!(1)
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
        assert_eq!(retrieval["result"]["output_mode"], json!("balanced"));
        assert!(retrieval["result"]["action_pack"].is_null());
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
            json!(
                "planner-only recall envelope; evidence hydration not implemented; normalized params: query_text"
            )
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
    async fn runtime_explain_memory_query_keeps_balanced_ranking_profile() {
        let socket_path = unique_path("explain-memory-balanced-profile");
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

        let _ = send_request(
            &socket_path,
            json!({
                "jsonrpc":"2.0",
                "method":"encode",
                "params":{
                    "content":"emotionally salient seed memory",
                    "namespace":"team.alpha",
                    "emotional_annotations":{"valence":0.4,"arousal":0.9}
                },
                "id":"encode-explain-seed"
            }),
        )
        .await;

        let explain_response = send_request(
            &socket_path,
            json!({
                "jsonrpc":"2.0",
                "method":"explain",
                "params":{"query":"1","namespace":"team.alpha"},
                "id":"explain-memory-balanced-profile"
            }),
        )
        .await;

        assert_eq!(explain_response["result"]["status"], json!("ok"));
        assert_eq!(
            explain_response["result"]["retrieval"]["result"]["explain"]["ranking_profile"],
            json!("balanced")
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
    async fn runtime_context_budget_returns_ready_to_inject_payload() {
        let socket_path = unique_path("context-budget");
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

        let _ = send_request(
            &socket_path,
            json!({
                "jsonrpc":"2.0",
                "method":"encode",
                "params":{
                    "content":"deploy checklist for launch day",
                    "namespace":"team.alpha",
                    "emotional_annotations":{"valence":0.4,"arousal":0.9}
                },
                "id":"encode-budget"
            }),
        )
        .await;

        let budget_response = send_request(
            &socket_path,
            json!({
                "jsonrpc":"2.0",
                "method":"context_budget",
                "params":{
                    "token_budget": 32,
                    "namespace":"team.alpha",
                    "current_context":"launch checklist",
                    "working_memory_ids":[999],
                    "format":"markdown",
                    "mood_congruent": true,
                    "request_id":"req-budget-1"
                },
                "id":"context-budget"
            }),
        )
        .await;

        assert_eq!(budget_response["result"]["status"], json!("ok"));
        assert_eq!(
            budget_response["result"]["payload"]["namespace"],
            json!("team.alpha")
        );
        assert_eq!(
            budget_response["result"]["payload"]["partial_success"],
            json!(false)
        );
        assert!(budget_response["result"]["payload"]["result"]["injections"]
            .as_array()
            .is_some_and(|items| !items.is_empty()));
        assert_eq!(
            budget_response["result"]["payload"]["result"]["injections"][0]["reason"],
            json!("high_utility")
        );
        assert_eq!(
            budget_response["result"]["payload"]["result"]["injections"][0]["source_kind"],
            json!("retrieval_result")
        );
        assert_eq!(
            budget_response["result"]["payload"]["result"]["explain"]["ranking_profile"],
            json!("mood_congruent")
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
            recall_response["result"]["retrieval"]["result"]["output_mode"],
            json!("balanced")
        );
        assert_eq!(
            recall_response["result"]["retrieval"]["result"]["packaging_metadata"]["degraded_summary"],
            json!(
                "planner-only recall envelope; evidence hydration not implemented; normalized params: query_text"
            )
        );
        assert!(
            recall_response["result"]["retrieval"]["result"]["evidence_pack"]
                .as_array()
                .is_some_and(|items| items.is_empty())
        );
        assert_eq!(
            recall_response["result"]["retrieval"]["result"]["explain"]["recall_plan"],
            json!("RecentTier1ThenTier2Exact")
        );

        let hot_paths = send_request(
            &socket_path,
            json!({
                "jsonrpc":"2.0",
                "method":"hot_paths",
                "params":{"namespace":"team.alpha","top_n":3},
                "id":"hot-paths-after-session-recall"
            }),
        )
        .await;
        assert_eq!(hot_paths["result"]["namespace"], json!("team.alpha"));
        assert_eq!(hot_paths["result"]["total_candidates"], json!(0));
        assert!(hot_paths["result"]["entries"]
            .as_array()
            .is_some_and(|entries| entries.is_empty()));

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
    async fn runtime_recall_records_attention_signals_for_hot_paths_and_dead_zones() {
        let socket_path = unique_path("recall-attention-audit");
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

        let encode_response = send_request(
            &socket_path,
            json!({
                "jsonrpc":"2.0",
                "method":"encode",
                "params":{"content":"attention logging runtime proof","namespace":"team.alpha"},
                "id":"encode-attention-audit"
            }),
        )
        .await;
        assert_eq!(encode_response["result"]["status"], json!("accepted"));
        let memory_id = encode_response["result"]["memory_id"]
            .as_u64()
            .expect("encode should return memory id");

        let recall_response = send_request(
            &socket_path,
            json!({
                "jsonrpc":"2.0",
                "method":"recall",
                "params":{"query_text":format!("memory:{}", memory_id),"namespace":"team.alpha","result_budget":3},
                "id":"recall-attention-audit"
            }),
        )
        .await;
        assert_eq!(recall_response["result"]["status"], json!("ok"));

        let hot_paths = send_request(
            &socket_path,
            json!({
                "jsonrpc":"2.0",
                "method":"hot_paths",
                "params":{"namespace":"team.alpha","top_n":3},
                "id":"hot-paths-after-recall"
            }),
        )
        .await;
        assert_eq!(hot_paths["result"]["namespace"], json!("team.alpha"));
        assert_eq!(hot_paths["result"]["total_candidates"], json!(1));
        assert_eq!(
            hot_paths["result"]["entries"][0]["memory_id"],
            json!(memory_id)
        );
        assert_eq!(hot_paths["result"]["entries"][0]["recall_count"], json!(1));
        assert_eq!(
            hot_paths["result"]["entries"][0]["prewarm_action"],
            json!("observe_only")
        );
        assert_eq!(
            recall_response["result"]["retrieval"]["explain_trace"]["cache_metrics"]
                ["prefetch_added_candidates"],
            Value::Null
        );

        let health = send_request(
            &socket_path,
            json!({"jsonrpc":"2.0","method":"health","params":{},"id":"health-after-recall"}),
        )
        .await;
        assert_eq!(health["result"]["cache"]["hints_submitted"], json!(0));
        assert_eq!(health["result"]["cache"]["prefetch_queue_depth"], json!(0));

        let dead_zones = send_request(
            &socket_path,
            json!({
                "jsonrpc":"2.0",
                "method":"dead_zones",
                "params":{"namespace":"team.alpha","min_age_ticks":5},
                "id":"dead-zones-after-recall"
            }),
        )
        .await;
        assert_eq!(dead_zones["result"]["namespace"], json!("team.alpha"));
        assert_eq!(dead_zones["result"]["total_candidates"], json!(0));
        assert!(dead_zones["result"]["entries"]
            .as_array()
            .is_some_and(|entries| entries.is_empty()));

        let audit = send_request(
            &socket_path,
            json!({
                "jsonrpc":"2.0",
                "method":"audit",
                "params":{"namespace":"team.alpha","op":"recall_served","limit":5},
                "id":"audit-after-recall"
            }),
        )
        .await;
        let detail = audit["result"]["entries"][0]["note"]
            .as_str()
            .expect("recall audit detail should be present");
        assert!(detail.contains(&format!("retrieved_ids=[{}]", memory_id)));
        assert!(detail.contains("latency_us="));
        assert!(detail.contains("heat_bucket=warm"));
        assert!(detail.contains("prewarm_action=observe_only"));
        assert!(detail.contains("prewarm_target=none"));
        assert!(detail.contains("attention_score=5"));

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
    async fn runtime_compress_applies_schema_compression_and_returns_log_history() {
        let socket_path = unique_path("compress-runtime");
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

        let compress_response = send_request(
            &socket_path,
            json!({
                "jsonrpc":"2.0",
                "method":"compress",
                "params":{"namespace":"team.alpha","dry_run":false},
                "id":"compress-runtime"
            }),
        )
        .await;

        assert_eq!(compress_response["result"]["status"], json!("accepted"));
        assert_eq!(
            compress_response["result"]["namespace"],
            json!("team.alpha")
        );
        assert_eq!(compress_response["result"]["dry_run"], json!(false));
        assert_eq!(
            compress_response["result"]["decision"]["disposition"],
            json!("eligible")
        );
        assert_eq!(compress_response["result"]["schemas_created"], json!(1));
        assert!(compress_response["result"]["schema_artifact"].is_object());
        assert!(compress_response["result"]["verification"]["verified"] == json!(true));
        assert_eq!(
            compress_response["result"]["compression_log_entries"]
                .as_array()
                .map(|rows| rows.len()),
            Some(1)
        );
        assert_eq!(
            compress_response["result"]["compression_log_entries"][0]["namespace"],
            json!("team.alpha")
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
    async fn runtime_schemas_lists_eligible_schema_previews() {
        let socket_path = unique_path("schemas-runtime");
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

        let schemas_response = send_request(
            &socket_path,
            json!({
                "jsonrpc":"2.0",
                "method":"schemas",
                "params":{"namespace":"team.alpha","top_n":2},
                "id":"schemas-runtime"
            }),
        )
        .await;

        assert_eq!(schemas_response["result"]["status"], json!("accepted"));
        assert_eq!(schemas_response["result"]["namespace"], json!("team.alpha"));
        assert_eq!(schemas_response["result"]["top"], json!(2));
        assert_eq!(
            schemas_response["result"]["schemas"]
                .as_array()
                .map(|rows| rows.len()),
            Some(1)
        );
        assert!(schemas_response["result"]["schemas"][0]["id"].is_u64());
        assert!(schemas_response["result"]["schemas"][0]["content"].is_string());
        assert!(schemas_response["result"]["schemas"][0]["compressed_member_ids"].is_array());

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
    async fn runtime_recall_strict_mode_suppresses_action_pack_when_caveats_exist() {
        let socket_path = unique_path("recall-strict-output-mode");
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

        let encode_response = send_request(
            &socket_path,
            json!({
                "jsonrpc":"2.0",
                "method":"encode",
                "params":{"content":"deploy checklist for launch day","namespace":"team.alpha"},
                "id":"encode-strict-output-mode"
            }),
        )
        .await;
        assert_eq!(encode_response["result"]["status"], json!("accepted"));

        let recall_response = send_request(
            &socket_path,
            json!({
                "jsonrpc":"2.0",
                "method":"recall",
                "params":{"query_text":"memory:42","namespace":"team.alpha","result_budget":3,"mode":"strict"},
                "id":"recall-strict-output-mode"
            }),
        )
        .await;

        assert_eq!(recall_response["result"]["status"], json!("ok"));
        assert_eq!(
            recall_response["result"]["retrieval"]["result"]["output_mode"],
            json!("balanced")
        );
        assert!(
            recall_response["result"]["retrieval"]["result"]["action_pack"]
                .as_array()
                .is_none_or(|items| items.is_empty())
        );
        assert_eq!(
            recall_response["result"]["retrieval"]["result"]["packaging_metadata"]
                ["packaging_mode"],
            json!("evidence_only")
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
            recall_response["result"]["retrieval"]["result"]["output_mode"],
            json!("balanced")
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
        assert!(
            recall_response["result"]["retrieval"]["result"]["action_pack"]
                .as_array()
                .is_none_or(|items| items.is_empty())
        );
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
        assert_eq!(
            full_contract_recall_response["result"]["retrieval"]["result"]["explain"]
                ["ranking_profile"],
            json!("balanced")
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

    #[test]
    fn normalize_recall_contract_preserves_query_by_example_seed_semantics() {
        let normalized = DaemonRuntime::normalize_recall_contract(
            NamespaceId::new("team.alpha").unwrap(),
            Some("  debugging trail  ".to_string()),
            Some(7),
            Some(9),
            None,
            Some(4),
            None,
        )
        .unwrap();

        assert_eq!(normalized.result_budget, 4);
        assert!(!normalized.mood_congruent);
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
            None,
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
            None,
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
            None,
        )
        .unwrap();

        assert_eq!(normalized.planner_request, RecallRequest::default());
    }

    #[test]
    fn normalize_recall_contract_preserves_mood_congruent_opt_in() {
        let normalized = DaemonRuntime::normalize_recall_contract(
            NamespaceId::new("team.alpha").unwrap(),
            Some("debugging trail".to_string()),
            None,
            None,
            None,
            Some(4),
            Some(true),
        )
        .unwrap();

        assert!(normalized.mood_congruent);
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
            recall_response["result"]["retrieval"]["result"]["packaging_metadata"]["degraded_summary"],
            json!(
                "planner-only recall envelope; evidence hydration not implemented; normalized params: query_text"
            )
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
            json!(
                "namespace team.alpha preflight safeguard evaluation (irreversible mutation; irreversible_mutation)"
            )
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
                    "original_query":"delete prior audit events",
                    "proposed_action":"purge namespace audit history",
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
                    "original_query":"delete prior audit events",
                    "proposed_action":"purge namespace audit history",
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
                json!("deny"),
                json!(true),
                json!("effective_namespace"),
                json!("policy"),
                json!(false),
                json!(true),
                json!("blocked"),
                json!("blocked"),
                json!("rejected"),
                json!("rejected"),
                json!(false),
                json!("blocked"),
                json!("blocked"),
                json!(["legal_hold"]),
                json!("rejected"),
                json!(false),
                json!(false),
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
                json!("effective_namespace"),
                json!("policy"),
                json!(false),
                json!(true),
                json!("blocked"),
                json!("preview_only"),
                json!("accepted"),
                json!("blocked"),
                json!(false),
                json!("missing_data"),
                json!("blocked"),
                json!(["scope_ambiguous"]),
                json!("accepted"),
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
                json!("policy"),
                json!(false),
                json!(true),
                json!("blocked"),
                json!("preview_only"),
                json!("accepted"),
                json!("blocked"),
                json!(false),
                json!("missing_data"),
                json!("blocked"),
                json!(["snapshot_required"]),
                json!("accepted"),
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
                json!("deny"),
                json!(true),
                json!("effective_namespace"),
                json!("policy"),
                json!(false),
                json!(true),
                json!("blocked"),
                json!("blocked"),
                json!("rejected"),
                json!("rejected"),
                json!(false),
                json!("blocked"),
                json!("blocked"),
                json!(["legal_hold"]),
                json!("rejected"),
                json!(false),
                json!(false),
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
                        "original_query": original_query,
                        "proposed_action": proposed_action,
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

        let observe_response = send_request(
            &socket_path,
            json!({
                "jsonrpc":"2.0",
                "method":"observe",
                "params":{
                    "content":"watcher noticed a file change",
                    "namespace":"team.alpha",
                    "source_label":"stdin:test"
                },
                "id":"observe"
            }),
        )
        .await;
        assert_eq!(observe_response["result"]["status"], json!("accepted"));

        let inspect_response = send_request(
            &socket_path,
            json!({
                "jsonrpc":"2.0",
                "method":"inspect",
                "params":{"id":2,"namespace":"team.alpha"},
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
        assert_eq!(inspect_response["result"]["payload"]["memory_id"], json!(2));
        assert_eq!(
            inspect_response["result"]["payload"]["tier"],
            json!("observation")
        );
        assert_eq!(
            inspect_response["result"]["payload"]["lifecycle_state"]["degraded_summary"],
            json!(null)
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

        let observe_response = send_request(
            &socket_path,
            json!({
                "jsonrpc":"2.0",
                "method":"observe",
                "params":{
                    "content":"watcher noticed a file change",
                    "namespace":"team.alpha",
                    "source_label":"stdin:test"
                },
                "id":"observe-canonical-payload-families"
            }),
        )
        .await;
        assert_eq!(observe_response["result"]["status"], json!("accepted"));

        let inspect_response = send_request(
            &socket_path,
            json!({
                "jsonrpc":"2.0",
                "method":"inspect",
                "params":{"id":2,"namespace":"team.alpha"},
                "id":"inspect-canonical-payload-families"
            }),
        )
        .await;

        let result = &inspect_response["result"];
        assert_eq!(result["status"], json!("ok"));
        assert!(result.get("retrieval").is_none());
        assert!(result.get("error").is_none());

        let payload = &result["payload"];
        assert_eq!(payload["request_id"], json!("daemon-inspect-3"));
        assert_eq!(payload["memory_id"], json!(2));
        assert_eq!(payload["tier"], json!("observation"));
        assert!(payload.get("lineage").is_some());
        assert!(payload.get("policy_flags").is_some());
        assert!(payload.get("lifecycle_state").is_some());
        assert!(payload.get("index_presence").is_some());
        assert!(payload.get("graph_neighborhood_summary").is_some());
        assert!(payload.get("decay_retention").is_some());
        assert!(payload.get("explain_trace").is_some());
        assert_eq!(
            payload["explain_trace"]["passive_observation"]["source_kind"],
            json!("observation")
        );
        assert_eq!(
            payload["explain_trace"]["passive_observation"]["write_decision"],
            json!("capture")
        );
        assert_eq!(
            payload["explain_trace"]["passive_observation"]["observation_source"],
            json!({"Present":"stdin:test"})
        );
        assert_eq!(
            payload["explain_trace"]["passive_observation"]["retention_marker"],
            json!({"Present":"volatile_observation"})
        );
        assert_eq!(
            payload["lifecycle_state"]["degraded_summary"],
            serde_json::Value::Null
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
    async fn runtime_goal_pause_and_resume_surface_checkpointed_working_state() {
        let socket_path = unique_path("goal-pause-resume");
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

        let pause = send_request(
            &socket_path,
            json!({
                "jsonrpc":"2.0",
                "method":"goal_pause",
                "params":{"namespace":"team.alpha","task_id":"task-42","note":"waiting for review"},
                "id":"goal-pause"
            }),
        )
        .await;
        assert_eq!(pause["result"]["status"], json!("dormant"));
        assert_eq!(
            pause["result"]["checkpoint"]["authoritative_truth"],
            json!("durable_memory")
        );
        assert_eq!(
            pause["result"]["note"],
            json!({"Present":"waiting for review"})
        );

        let resume = send_request(
            &socket_path,
            json!({
                "jsonrpc":"2.0",
                "method":"goal_resume",
                "params":{"namespace":"team.alpha","task_id":"task-42"},
                "id":"goal-resume"
            }),
        )
        .await;
        assert_eq!(resume["result"]["status"], json!("active"));
        assert_eq!(
            resume["result"]["authoritative_truth"],
            json!("durable_memory")
        );
        assert_eq!(resume["result"]["restored_evidence_handles"], json!([1, 2]));
        assert_eq!(resume["result"]["warnings"], json!([]));

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
    async fn runtime_goal_resume_without_checkpoint_reports_stale_explicitly() {
        let socket_path = unique_path("goal-resume-stale");
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

        let resume = send_request(
            &socket_path,
            json!({
                "jsonrpc":"2.0",
                "method":"goal_resume",
                "params":{"namespace":"team.alpha","task_id":"task-missing"},
                "id":"goal-resume-stale"
            }),
        )
        .await;
        assert_eq!(resume["result"]["status"], json!("stale"));
        assert_eq!(resume["result"]["checkpoint"]["stale"], json!(true));
        assert_eq!(
            resume["result"]["warnings"][0]["code"],
            json!("stale_checkpoint")
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
    async fn runtime_share_unshare_parity_matches_core_policy_denial_and_redaction_contract() {
        let socket_path = unique_path("share-unshare-core-parity");
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

        let same_namespace_request = membrain_core::api::RequestContext {
            namespace: Some(NamespaceId::new("team.alpha").unwrap()),
            workspace_id: None,
            agent_id: None,
            session_id: None,
            task_id: None,
            request_id: membrain_core::api::RequestId::new("req-share-core").unwrap(),
            policy_context: membrain_core::api::PolicyContext {
                include_public: false,
                sharing_visibility: membrain_core::policy::SharingVisibility::Private,
                caller_identity_bound: true,
                workspace_acl_allowed: true,
                agent_acl_allowed: true,
                session_visibility_allowed: true,
                legal_hold: false,
            },
            time_budget_ms: None,
        };
        let same_namespace_outcome = same_namespace_request
            .bind_namespace(None)
            .unwrap()
            .evaluate_sharing_access(&membrain_core::policy::PolicyModule);
        assert_eq!(
            same_namespace_outcome.decision,
            membrain_core::policy::SharingAccessDecision::Allow
        );
        assert_eq!(
            same_namespace_outcome.sharing_scope.unwrap().as_str(),
            "namespace_only"
        );
        assert!(same_namespace_outcome.redaction_fields.is_empty());

        let cross_namespace_request = membrain_core::api::RequestContext {
            namespace: Some(NamespaceId::new("team.alpha").unwrap()),
            workspace_id: None,
            agent_id: None,
            session_id: None,
            task_id: None,
            request_id: membrain_core::api::RequestId::new("req-unshare-core").unwrap(),
            policy_context: membrain_core::api::PolicyContext {
                include_public: false,
                sharing_visibility: membrain_core::policy::SharingVisibility::Private,
                caller_identity_bound: true,
                workspace_acl_allowed: true,
                agent_acl_allowed: true,
                session_visibility_allowed: true,
                legal_hold: false,
            },
            time_budget_ms: None,
        };
        let cross_namespace_outcome = cross_namespace_request
            .bind_namespace(None)
            .unwrap()
            .evaluate_cross_namespace_sharing_access(
                &membrain_core::policy::PolicyModule,
                &NamespaceId::new("team.beta").unwrap(),
            );
        assert_eq!(
            cross_namespace_outcome.decision,
            membrain_core::policy::SharingAccessDecision::Deny
        );
        assert!(cross_namespace_outcome
            .denial_reasons
            .iter()
            .any(|reason| reason.as_str() == "namespace_isolation"));
        assert_eq!(
            cross_namespace_outcome.redaction_fields,
            vec!["memory_id", "sharing_scope", "workspace_id", "session_id"]
        );

        let share_response = send_request(
            &socket_path,
            json!({
                "jsonrpc":"2.0",
                "method":"share",
                "params":{"id":42,"namespace_id":"team.beta"},
                "id":"share-core-parity"
            }),
        )
        .await;
        assert_eq!(share_response["result"]["status"], json!("accepted"));
        assert_eq!(share_response["result"]["visibility"], json!("shared"));
        assert_eq!(
            share_response["result"]["policy_summary"]["outcome_class"],
            json!(membrain_core::observability::OutcomeClass::Accepted.as_str())
        );
        assert_eq!(
            share_response["result"]["policy_filters_applied"][0]["redaction_fields"],
            json!([])
        );
        assert_eq!(share_response["result"]["audit"]["redacted"], json!(false));

        let unshare_response = send_request(
            &socket_path,
            json!({
                "jsonrpc":"2.0",
                "method":"unshare",
                "params":{"id":42,"namespace":"team.alpha"},
                "id":"unshare-core-parity"
            }),
        )
        .await;
        assert_eq!(unshare_response["result"]["status"], json!("accepted"));
        assert_eq!(unshare_response["result"]["visibility"], json!("private"));
        assert_eq!(
            unshare_response["result"]["policy_summary"]["outcome_class"],
            json!(membrain_core::observability::OutcomeClass::Accepted.as_str())
        );
        assert_eq!(
            unshare_response["result"]["policy_summary"]["redaction_fields"],
            json!(["sharing_scope"])
        );
        assert_eq!(
            unshare_response["result"]["policy_filters_applied"][0]["redaction_fields"],
            json!(["sharing_scope"])
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

    #[test]
    fn mcp_failure_policy_denial_flag_stays_distinct_from_non_policy_internal_failures() {
        let policy_denied = McpResponse::failure(McpError {
            code: "policy_denied".to_string(),
            message: "namespace isolation prevents export".to_string(),
            is_policy_denial: true,
        });
        let internal_failure = McpResponse::failure(McpError {
            code: "internal_failure".to_string(),
            message: "repair controller diverged from authoritative state".to_string(),
            is_policy_denial: false,
        });

        let policy_denied_json = serde_json::to_value(policy_denied).unwrap();
        let internal_failure_json = serde_json::to_value(internal_failure).unwrap();

        assert_eq!(policy_denied_json["status"], json!("error"));
        assert_eq!(policy_denied_json["error"]["code"], json!("policy_denied"));
        assert_eq!(policy_denied_json["error"]["is_policy_denial"], json!(true));

        assert_eq!(internal_failure_json["status"], json!("error"));
        assert_eq!(
            internal_failure_json["error"]["code"],
            json!("internal_failure")
        );
        assert_eq!(
            internal_failure_json["error"]["is_policy_denial"],
            json!(false)
        );
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
            json!("membrain://daemon/runtime/health")
        );
        assert_eq!(
            resources_response["result"]["payload"]["resources"][1]["resource_kind"],
            json!("runtime_health")
        );
        assert_eq!(
            resources_response["result"]["payload"]["resources"][2]["uri"],
            json!("membrain://daemon/runtime/doctor")
        );
        assert_eq!(
            resources_response["result"]["payload"]["resources"][2]["resource_kind"],
            json!("runtime_doctor")
        );
        assert!(resources_response["result"]["payload"]["resources"][3]["uri_template"].is_null());
        assert_eq!(
            resources_response["result"]["payload"]["resources"][4]["examples"][0],
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
        assert_eq!(
            doctor_resource["result"]["payload"]["payload"]["repair_reports"][0]["target"],
            json!("lexical_index")
        );
        assert_eq!(
            doctor_resource["result"]["payload"]["payload"]["repair_reports"][0]
                ["rebuild_entrypoint"],
            json!(null)
        );
        assert_eq!(
            doctor_resource["result"]["payload"]["payload"]["repair_reports"][0]
                ["affected_item_count"],
            json!(128)
        );
        assert!(doctor_resource["result"]["payload"]["payload"]["health"].is_object());
        assert_eq!(
            doctor_resource["result"]["payload"]["payload"]["health"]["availability_posture"],
            json!(null)
        );

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
