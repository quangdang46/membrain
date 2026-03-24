use clap::{Parser, Subcommand};
use membrain_core::api::{
    AvailabilityReason, AvailabilitySummary, CacheMetricsSummary, ConflictMarker, FreshnessMarker,
    NamespaceId, RemediationStep, RequestId, ResponseContext, TraceOmissionSummary,
    TracePolicySummary, TraceProvenanceSummary, TraceScoreComponent, TraceStage, UncertaintyMarker,
};
use membrain_core::engine::maintenance::{
    MaintenanceController, MaintenanceJobHandle, MaintenanceJobState,
};
use membrain_core::engine::ranking::{fuse_scores, RankingInput, RankingProfile};
use membrain_core::engine::recall::RecallRuntime;
use membrain_core::engine::repair::{IndexRepairEntrypoint, RepairTarget};
use membrain_core::engine::result::{
    AnsweredFrom, ResultBuilder, RetrievalExplain, RetrievalResultSet,
};
use membrain_core::health::{BrainHealthInputs, BrainHealthReport, FeatureAvailabilityEntry};
use membrain_core::index::{IndexApi, IndexModule};
use membrain_core::observability::{AuditEventCategory, AuditEventKind};
use membrain_core::store::audit::{
    AppendOnlyAuditLog, AuditLogEntry, AuditLogFilter, AuditLogSlice, AuditLogStore,
};
use membrain_core::store::cache::CacheManager;
use membrain_core::store::hot::Tier1HotMetadataStore;
use membrain_core::types::{MemoryId, RawEncodeInput, RawIntakeKind, SessionId, Tier1HotRecord};
use membrain_core::{BrainStore, RuntimeConfig};
use membrain_daemon::daemon::{DaemonRuntime, DaemonRuntimeConfig};
use membrain_daemon::rpc::{RuntimeMetrics, RuntimePosture, RuntimeStatus};
use serde::Serialize;
use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::Instant;

/// Global memory ID counter for CLI-local session.
static NEXT_MEMORY_ID: AtomicU64 = AtomicU64::new(1);

/// Global session ID for CLI-local session.
static SESSION_ID: AtomicU64 = AtomicU64::new(1);

/// Local record kept for CLI-session recall and inspect.
#[derive(Debug, Clone)]
struct LocalMemoryRecord {
    memory_id: MemoryId,
    namespace: NamespaceId,
    session_id: SessionId,
    memory_type: membrain_core::types::CanonicalMemoryType,
    route_family: membrain_core::types::FastPathRouteFamily,
    compact_text: String,
    provisional_salience: u16,
    fingerprint: u64,
    payload_size_bytes: usize,
}

impl LocalMemoryRecord {
    fn as_hot_record(&self) -> Tier1HotRecord {
        Tier1HotRecord::metadata_only(
            self.namespace.clone(),
            self.memory_id,
            self.session_id,
            self.memory_type,
            self.route_family,
            &self.compact_text,
            self.fingerprint,
            self.provisional_salience,
            self.payload_size_bytes,
        )
    }
}

#[derive(Parser, Debug)]
#[command(name = "membrain", version, about = "Membrain CLI", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// Encode (store) a new memory
    #[command(name = "remember", visible_alias = "encode")]
    Encode {
        /// Content to store
        content: String,
        /// Namespace for the memory
        #[arg(long, short = 'n', default_value = "default")]
        namespace: String,
        /// Memory kind: episodic, semantic, procedural
        #[arg(long, short = 'k', default_value = "episodic")]
        kind: String,
        /// Current task/context (enhances retrieval later)
        #[arg(long, short = 'c')]
        context: Option<String>,
        /// Attention level 0.0–1.0 (below 0.2: discarded)
        #[arg(long, short = 'a', default_value_t = 0.7)]
        attention: f32,
        /// Emotional valence -1.0 to +1.0
        #[arg(long, default_value_t = 0.0)]
        valence: f32,
        /// Emotional arousal 0.0–1.0
        #[arg(long, default_value_t = 0.0)]
        arousal: f32,
        /// Source tag: cli, mcp, api
        #[arg(long, default_value = "cli")]
        source: String,
        /// Emit JSON instead of human-readable text
        #[arg(long, default_value_t = false)]
        json: bool,
    },
    /// Recall memories matching a query
    Recall {
        /// Query string to match
        query: String,
        /// Namespace to search in
        #[arg(long, default_value = "default")]
        namespace: String,
        /// Maximum number of results to return
        #[arg(long, short = 'n', visible_short_alias = 't', default_value_t = 5)]
        top: usize,
        /// Current context; maps to context_text
        #[arg(long, short = 'c')]
        context: Option<String>,
        /// Filter: episodic, semantic, procedural
        #[arg(long, short = 'k')]
        kind: Option<String>,
        /// Bounded effort level: fast, normal, high
        #[arg(long, default_value = "normal")]
        confidence: String,
        /// Explain verbosity: none, summary, full
        #[arg(long, default_value = "summary")]
        explain: String,
        /// Emit JSON instead of human-readable text
        #[arg(long, default_value_t = false)]
        json: bool,
    },
    /// Inspect a specific memory or entity by ID
    Inspect {
        /// The memory ID to inspect
        #[arg(long)]
        id: u64,
        /// Namespace of the memory
        #[arg(long, short = 'n', default_value = "default")]
        namespace: String,
        /// Emit JSON instead of human-readable text
        #[arg(long, default_value_t = false)]
        json: bool,
    },
    /// Explain the ranking and routing path for a recall query
    #[command(name = "why", visible_alias = "explain")]
    Explain {
        /// Query string to explain
        query: String,
        /// Namespace to explain over
        #[arg(long, short = 'n', default_value = "default")]
        namespace: String,
        /// Emit JSON instead of human-readable text
        #[arg(long, default_value_t = false)]
        json: bool,
    },
    /// Run maintenance tasks (repair, reclaim, metrics)
    Maintenance {
        /// The maintenance action to run (e.g. repair, repair_index, repair_metadata)
        #[arg(long)]
        action: String,
        /// Scope of maintenance
        #[arg(long)]
        namespace: Option<String>,
        /// Emit JSON instead of text
        #[arg(long, default_value_t = false)]
        json: bool,
    },
    /// Run core performance and correctness benchmarks
    Benchmark {
        /// Target metric to benchmark: encode, recall, intent, tier1, retrieval
        #[arg(long, default_value = "encode")]
        target: String,
        /// Number of iterations
        #[arg(long, default_value_t = 100)]
        iters: usize,
        /// Emit JSON instead of text
        #[arg(long, default_value_t = false)]
        json: bool,
    },
    /// Validate system configuration and index health
    Doctor,
    /// Query and export bounded audit history slices
    Audit {
        /// Namespace to inspect
        #[arg(long)]
        namespace: String,
        /// Optional memory id filter
        #[arg(long)]
        id: Option<u64>,
        /// Optional session id filter
        #[arg(long)]
        session: Option<u64>,
        /// Optional minimum sequence filter
        #[arg(long)]
        since: Option<u64>,
        /// Optional event or category filter
        #[arg(long)]
        op: Option<String>,
        /// Optional tail count after filtering
        #[arg(long)]
        recent: Option<usize>,
        /// Emit JSON instead of text
        #[arg(long, default_value_t = false)]
        json: bool,
    },
    /// Share a memory into an approved namespace scope
    Share {
        /// The memory ID to share
        #[arg(long)]
        id: u64,
        /// Target namespace for approved sharing
        #[arg(long = "namespace")]
        namespace_id: String,
        /// Emit JSON instead of text
        #[arg(long, default_value_t = false)]
        json: bool,
    },
    /// Tighten a shared memory back to private visibility
    Unshare {
        /// The memory ID to unshare
        #[arg(long)]
        id: u64,
        /// Canonical namespace that retains ownership
        #[arg(long, short = 'n', default_value = "default")]
        namespace: String,
        /// Emit JSON instead of text
        #[arg(long, default_value_t = false)]
        json: bool,
    },
    /// Run the local daemon inside the CLI process
    Daemon {
        /// Unix socket path to bind
        #[arg(long, default_value = "/tmp/membrain.sock")]
        socket_path: PathBuf,
        /// Maximum number of concurrent request handlers
        #[arg(long, default_value_t = 8)]
        request_concurrency: usize,
        /// Maximum queued requests before new requests are rejected
        #[arg(long, default_value_t = 32)]
        max_queue_depth: usize,
    },
}

// ── Output types ─────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize)]
struct EncodeOutput {
    outcome: &'static str,
    memory_id: u64,
    namespace: String,
    memory_type: &'static str,
    route_family: &'static str,
    compact_text: String,
    provisional_salience: u16,
    fingerprint: u64,
    payload_size_bytes: usize,
    is_landmark: bool,
    landmark_label: Option<String>,
    context: Option<String>,
    source: String,
}

#[derive(Debug, Clone, Serialize)]
struct InspectOutput {
    outcome: &'static str,
    memory_id: u64,
    namespace: String,
    memory_type: &'static str,
    route_family: &'static str,
    compact_text: String,
    provisional_salience: u16,
    fingerprint: u64,
    payload_size_bytes: usize,
    payload_state: &'static str,
    is_landmark: bool,
    session_id: u64,
}

// ── Shared helper types ──────────────────────────────────────────────────────

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
struct RepairResultOutput {
    target: &'static str,
    status: &'static str,
    verification_passed: bool,
    rebuild_entrypoint: Option<&'static str>,
    rebuilt_outputs: Vec<&'static str>,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
struct MaintenanceOutput {
    outcome: &'static str,
    action: String,
    namespace: String,
    targets_checked: u32,
    rebuilt: u32,
    results: Vec<RepairResultOutput>,
    warnings: Vec<&'static str>,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
struct BenchmarkOutput {
    outcome: &'static str,
    target: String,
    iterations: usize,
    total_duration_ms: f64,
    avg_duration_us: f64,
    min_duration_us: f64,
    max_duration_us: f64,
    p50_duration_us: f64,
    p95_duration_us: f64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
struct AuditRow {
    sequence: u64,
    category: &'static str,
    kind: &'static str,
    namespace: String,
    memory_id: Option<u64>,
    session_id: Option<u64>,
    triggered_by: &'static str,
    request_id: Option<String>,
    related_run: Option<String>,
    redacted: bool,
    note: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
struct AuditExport {
    total_matches: usize,
    returned_rows: usize,
    truncated: bool,
    rows: Vec<AuditRow>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
struct ShareOutput {
    outcome: &'static str,
    memory_id: u64,
    namespace: String,
    visibility: &'static str,
    policy_summary: TracePolicySummary,
    policy_filters_applied: Vec<membrain_core::api::PolicyFilterSummary>,
    audit_rows: Vec<AuditRow>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
struct DoctorIndexRow {
    family: &'static str,
    health: &'static str,
    usable: bool,
    entry_count: usize,
    generation: &'static str,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
struct RepairReportRow {
    target: &'static str,
    status: &'static str,
    verification_passed: bool,
    rebuild_entrypoint: Option<&'static str>,
    rebuilt_outputs: Vec<&'static str>,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
struct DoctorReport {
    status: &'static str,
    action: &'static str,
    posture: &'static str,
    degraded_reasons: Vec<String>,
    metrics: RuntimeMetrics,
    indexes: Vec<DoctorIndexRow>,
    repair_engine_component: &'static str,
    repair_reports: Vec<RepairReportRow>,
    warnings: Vec<&'static str>,
    health: BrainHealthReport,
}

impl From<AuditLogEntry> for AuditRow {
    fn from(entry: AuditLogEntry) -> Self {
        Self {
            sequence: entry.sequence,
            category: entry.category.as_str(),
            kind: entry.kind.as_str(),
            namespace: entry.namespace.as_str().to_owned(),
            memory_id: entry.memory_id.map(|id| id.0),
            session_id: entry.session_id.map(|id| id.0),
            triggered_by: entry.actor_source,
            request_id: entry.request_id,
            related_run: entry.related_run,
            redacted: entry.redacted,
            note: entry.detail,
        }
    }
}

// ── Mapping helpers ──────────────────────────────────────────────────────────

fn parse_memory_kind(raw: &str) -> RawIntakeKind {
    match raw.to_lowercase().as_str() {
        "semantic" | "observation" => RawIntakeKind::Observation,
        "procedural" | "tool_outcome" => RawIntakeKind::ToolOutcome,
        _ => RawIntakeKind::Event,
    }
}

fn intent_confidence_label(low_confidence_fallback: bool) -> &'static str {
    if low_confidence_fallback {
        "low"
    } else {
        "high"
    }
}

type ResponseTraceBundle = (
    membrain_core::api::RouteSummary,
    Vec<TraceStage>,
    Vec<membrain_core::api::ResultReason>,
    TraceOmissionSummary,
    membrain_core::api::GraphExpansionSummary,
    Vec<TraceScoreComponent>,
    TracePolicySummary,
    TraceProvenanceSummary,
    Vec<FreshnessMarker>,
    Vec<ConflictMarker>,
    Vec<UncertaintyMarker>,
);

fn response_trace_for_result_set(result_set: &RetrievalResultSet) -> ResponseTraceBundle {
    let route_summary = membrain_core::api::RouteSummary::from_result_set(result_set);
    let trace_stages = result_set
        .explain
        .trace_stages
        .iter()
        .copied()
        .map(membrain_core::api::TraceStage::from_recall)
        .chain([
            membrain_core::api::TraceStage::PolicyGate,
            membrain_core::api::TraceStage::Packaging,
        ])
        .collect();
    let result_reasons = result_set
        .explain
        .result_reasons
        .iter()
        .map(membrain_core::api::ResultReason::from_result_reason)
        .collect();
    let omitted_summary = TraceOmissionSummary::from_result_set(result_set);
    let graph_expansion = result_set.explain_graph_expansion();
    let policy_summary = TracePolicySummary::from_result_set(result_set);
    let provenance_summary = TraceProvenanceSummary::from_result_set(result_set);
    let (freshness_markers, conflict_markers, uncertainty_markers) = result_set.explain_markers();
    let freshness_markers = freshness_markers
        .into_iter()
        .map(|marker| FreshnessMarker {
            code: marker.code,
            detail: marker.detail,
        })
        .collect();
    let conflict_markers = conflict_markers
        .into_iter()
        .map(|marker| ConflictMarker {
            code: marker.code,
            detail: marker.detail,
        })
        .collect();
    let uncertainty_markers = uncertainty_markers
        .into_iter()
        .map(|marker| UncertaintyMarker {
            code: marker.code,
            detail: marker.detail,
        })
        .collect();
    let score_components = result_set
        .top()
        .map(|top| {
            top.result
                .score_summary
                .signal_breakdown
                .iter()
                .map(
                    |(signal_family, raw_value, weight, _)| TraceScoreComponent {
                        signal_family: match signal_family.as_str() {
                            "recency" => "recency",
                            "salience" => "salience",
                            "strength" => "strength",
                            "provenance" => "provenance",
                            "conflict_adjustment" => "conflict_adjustment",
                            "confidence" => "confidence",
                            _ => "custom",
                        },
                        raw_value: *raw_value,
                        weight: *weight,
                    },
                )
                .collect()
        })
        .unwrap_or_default();

    (
        route_summary,
        trace_stages,
        result_reasons,
        omitted_summary,
        graph_expansion,
        score_components,
        policy_summary,
        provenance_summary,
        freshness_markers,
        conflict_markers,
        uncertainty_markers,
    )
}

fn response_from_result_set(
    namespace: &NamespaceId,
    request_id: RequestId,
    result_set: RetrievalResultSet,
) -> ResponseContext<RetrievalResultSet> {
    let partial_success = matches!(
        result_set.outcome_class,
        membrain_core::observability::OutcomeClass::Partial
    ) || result_set.truncated;
    let (
        route_summary,
        trace_stages,
        result_reasons,
        omitted_summary,
        graph_expansion,
        score_components,
        policy_summary,
        provenance_summary,
        freshness_markers,
        conflict_markers,
        uncertainty_markers,
    ) = response_trace_for_result_set(&result_set);
    let mut response = ResponseContext::success(namespace.clone(), request_id, result_set)
        .with_trace_schema(
            route_summary,
            trace_stages,
            result_reasons,
            omitted_summary,
            graph_expansion,
            CacheMetricsSummary::from_cache_traces(Vec::new(), false),
            score_components,
            policy_summary,
            provenance_summary,
            freshness_markers,
            conflict_markers,
            uncertainty_markers,
        );
    if partial_success {
        response = response.with_partial_success();
    }
    response
}

fn build_retrieval_result_set(
    local_records: &[LocalMemoryRecord],
    namespace: &NamespaceId,
    top: usize,
    ranking_profile: RankingProfile,
    route_family: &'static str,
    route_reason: String,
    matched_ids: Vec<MemoryId>,
) -> RetrievalResultSet {
    let mut builder = ResultBuilder::new(top, namespace.clone());
    let matched_empty = matched_ids.is_empty();
    let selected_ids = if matched_empty {
        local_records
            .iter()
            .filter(|r| r.namespace == *namespace)
            .rev()
            .take(top)
            .map(|r| r.memory_id)
            .collect::<Vec<_>>()
    } else {
        matched_ids
    };

    for memory_id in &selected_ids {
        if let Some(record) = local_records
            .iter()
            .find(|r| r.namespace == *namespace && r.memory_id == *memory_id)
        {
            let ranking = fuse_scores(
                RankingInput {
                    recency: 900,
                    salience: record.provisional_salience,
                    strength: 750,
                    provenance: 850,
                    conflict: 500,
                    confidence: 700,
                },
                ranking_profile,
            );
            builder.add(
                record.memory_id,
                record.namespace.clone(),
                record.session_id,
                record.memory_type,
                record.compact_text.clone(),
                &ranking,
                AnsweredFrom::Tier1Hot,
            );
        }
    }

    let mut explain = RetrievalExplain {
        recall_plan: membrain_core::engine::recall::RecallPlanKind::RecentTier1ThenTier2Exact,
        route_reason,
        tiers_consulted: vec!["tier1_recent".to_string()],
        trace_stages: vec![membrain_core::engine::recall::RecallTraceStage::Tier1RecentWindow],
        tier1_answered_directly: true,
        candidate_budget: top,
        time_consumed_ms: None,
        ranking_profile: route_family.to_string(),
        contradictions_found: 0,
        query_by_example: None,
        result_reasons: selected_ids
            .iter()
            .map(|memory_id| membrain_core::engine::result::ResultReason {
                memory_id: Some(*memory_id),
                reason_code: "score_kept".to_string(),
                detail: if matched_empty {
                    "fallback returned a recent memory from the bounded hot window".to_string()
                } else {
                    "query matched the compact text inside the bounded hot window".to_string()
                },
            })
            .collect(),
    };
    if selected_ids.is_empty() {
        explain
            .result_reasons
            .push(membrain_core::engine::result::ResultReason {
                memory_id: None,
                reason_code: "no_match".to_string(),
                detail: "bounded hot-window scan returned no visible evidence".to_string(),
            });
    }
    builder.build(explain)
}

// ── Encode ───────────────────────────────────────────────────────────────────

#[allow(clippy::too_many_arguments)]
fn encode_memory(
    store: &BrainStore,
    hot: &mut Tier1HotMetadataStore,
    local_records: &mut Vec<LocalMemoryRecord>,
    content: &str,
    namespace: &NamespaceId,
    kind: &str,
    _context: Option<&str>,
    _attention: f32,
    _valence: f32,
    _arousal: f32,
    source: &str,
) -> ResponseContext<EncodeOutput> {
    let intake_kind = parse_memory_kind(kind);
    let input = RawEncodeInput::new(intake_kind, content);
    let prepared = store.encode_engine().prepare_fast_path(input);

    let memory_id = MemoryId(NEXT_MEMORY_ID.fetch_add(1, Ordering::SeqCst));
    let session_id = SessionId(SESSION_ID.load(Ordering::SeqCst));

    let local = LocalMemoryRecord {
        memory_id,
        namespace: namespace.clone(),
        session_id,
        memory_type: prepared.normalized.memory_type,
        route_family: prepared.classification.route_family,
        compact_text: prepared.normalized.compact_text.clone(),
        provisional_salience: prepared.provisional_salience,
        fingerprint: prepared.fingerprint,
        payload_size_bytes: prepared.normalized.payload_size_bytes,
    };

    hot.seed(local.as_hot_record());
    local_records.push(local);

    ResponseContext::success(
        namespace.clone(),
        RequestId::new(format!("encode-{}", memory_id.0)).expect("encode request id"),
        EncodeOutput {
            outcome: "accepted",
            memory_id: memory_id.0,
            namespace: namespace.as_str().to_string(),
            memory_type: prepared.normalized.memory_type.as_str(),
            route_family: prepared.classification.route_family.as_str(),
            compact_text: prepared.normalized.compact_text.clone(),
            provisional_salience: prepared.provisional_salience,
            fingerprint: prepared.fingerprint,
            payload_size_bytes: prepared.normalized.payload_size_bytes,
            is_landmark: prepared.normalized.landmark.is_landmark,
            landmark_label: prepared.normalized.landmark.landmark_label.clone(),
            context: _context.map(String::from),
            source: source.to_string(),
        },
    )
}

// ── Recall ───────────────────────────────────────────────────────────────────

fn recall_memories(
    store: &BrainStore,
    _hot: &Tier1HotMetadataStore,
    local_records: &[LocalMemoryRecord],
    query: &str,
    namespace: &NamespaceId,
    top: usize,
    _explain_verbosity: &str,
) -> ResponseContext<RetrievalResultSet> {
    let intent_result = store.intent_engine().classify(query);
    let session_id = SessionId(SESSION_ID.load(Ordering::SeqCst));

    let recall_request =
        membrain_core::engine::recall::RecallRequest::small_session_lookup(session_id);
    let recall_plan = store
        .recall_engine()
        .plan_recall(recall_request, store.config());

    let query_lower = query.to_lowercase();
    let matched_ids = local_records
        .iter()
        .filter(|r| r.namespace == *namespace)
        .filter(|record| {
            let text_lower = record.compact_text.to_lowercase();
            text_lower.contains(&query_lower)
                || query_lower.contains(&text_lower)
                || record.memory_type.as_str().contains(&query_lower)
        })
        .map(|record| record.memory_id)
        .collect::<Vec<_>>();

    let result_set = build_retrieval_result_set(
        local_records,
        namespace,
        top,
        RankingProfile::balanced(),
        intent_result.route_inputs.ranking_profile.as_str(),
        recall_plan.route_summary.reason.to_string(),
        matched_ids,
    );

    let request_id = RequestId::new(format!(
        "recall-{}-{}",
        namespace.as_str(),
        query.replace(' ', "-")
    ))
    .expect("recall request id");
    response_from_result_set(namespace, request_id, result_set)
}

// ── Inspect ──────────────────────────────────────────────────────────────────

fn inspect_memory(
    _hot: &mut Tier1HotMetadataStore,
    local_records: &[LocalMemoryRecord],
    namespace: &NamespaceId,
    memory_id: MemoryId,
) -> Result<InspectOutput, String> {
    // Try hot store first, then fall back to local records
    let lookup = _hot.exact_lookup(namespace, memory_id);
    if let Some(record) = lookup.record {
        return Ok(InspectOutput {
            outcome: "accepted",
            memory_id: record.memory_id.0,
            namespace: record.namespace.as_str().to_string(),
            memory_type: record.memory_type.as_str(),
            route_family: record.route_family.as_str(),
            compact_text: record.compact_text,
            provisional_salience: record.provisional_salience,
            fingerprint: record.fingerprint,
            payload_size_bytes: record.payload_size_bytes,
            payload_state: match record.payload_state {
                membrain_core::types::Tier1PayloadState::MetadataOnly => "metadata_only",
                membrain_core::types::Tier1PayloadState::PreviewInline => "preview_inline",
            },
            is_landmark: false,
            session_id: record.session_id.0,
        });
    }

    // Fall back to local records
    if let Some(record) = local_records
        .iter()
        .find(|r| r.memory_id == memory_id && r.namespace == *namespace)
    {
        return Ok(InspectOutput {
            outcome: "accepted",
            memory_id: record.memory_id.0,
            namespace: record.namespace.as_str().to_string(),
            memory_type: record.memory_type.as_str(),
            route_family: record.route_family.as_str(),
            compact_text: record.compact_text.clone(),
            provisional_salience: record.provisional_salience,
            fingerprint: record.fingerprint,
            payload_size_bytes: record.payload_size_bytes,
            payload_state: "metadata_only",
            is_landmark: false,
            session_id: record.session_id.0,
        });
    }

    Err(format!(
        "memory {} not found in namespace '{}'",
        memory_id.0,
        namespace.as_str()
    ))
}

// ── Explain ──────────────────────────────────────────────────────────────────

fn explain_query(
    store: &BrainStore,
    local_records: &[LocalMemoryRecord],
    query: &str,
    namespace: &NamespaceId,
) -> ResponseContext<RetrievalResultSet> {
    let intent_result = store.intent_engine().classify(query);
    let session_id = SessionId(SESSION_ID.load(Ordering::SeqCst));

    let recall_request =
        membrain_core::engine::recall::RecallRequest::small_session_lookup(session_id);
    let recall_plan = store
        .recall_engine()
        .plan_recall(recall_request, store.config());

    let query_lower = query.to_lowercase();
    let matched_ids = local_records
        .iter()
        .filter(|r| r.namespace == *namespace)
        .filter(|record| {
            let text_lower = record.compact_text.to_lowercase();
            text_lower.contains(&query_lower)
                || query_lower.contains(&text_lower)
                || record.memory_type.as_str().contains(&query_lower)
        })
        .map(|record| record.memory_id)
        .collect::<Vec<_>>();

    let mut result_set = build_retrieval_result_set(
        local_records,
        namespace,
        5,
        RankingProfile::balanced(),
        intent_result.route_inputs.ranking_profile.as_str(),
        recall_plan.route_summary.reason.to_string(),
        matched_ids,
    );
    result_set.explain.ranking_profile = intent_result
        .route_inputs
        .ranking_profile
        .as_str()
        .to_string();
    result_set
        .explain
        .result_reasons
        .push(membrain_core::engine::result::ResultReason {
            memory_id: None,
            reason_code: "score_kept".to_string(),
            detail: format!(
                "intent={} confidence={} query_path={} matched_patterns={}",
                intent_result.intent.as_str(),
                intent_confidence_label(intent_result.low_confidence_fallback),
                intent_result.route_inputs.query_path.as_str(),
                intent_result.log_record().matched_patterns.join(",")
            ),
        });

    let request_id = RequestId::new(format!(
        "explain-{}-{}",
        namespace.as_str(),
        query.replace(' ', "-")
    ))
    .expect("explain request id");
    response_from_result_set(namespace, request_id, result_set)
}

// ── Shared helpers for non-core commands ─────────────────────────────────────

fn sample_audit_log(namespace: &NamespaceId) -> AppendOnlyAuditLog {
    let mut log = AuditLogStore.new_log(8);
    log.append(
        AuditLogEntry::new(
            AuditEventCategory::Encode,
            AuditEventKind::EncodeAccepted,
            namespace.clone(),
            "encode_engine",
            "encoded memory into durable flow",
        )
        .with_memory_id(MemoryId(21))
        .with_session_id(SessionId(5))
        .with_request_id("req-encode-21"),
    );
    log.append(
        AuditLogEntry::new(
            AuditEventCategory::Policy,
            AuditEventKind::PolicyRedacted,
            namespace.clone(),
            "policy_module",
            "redacted protected actor details for export",
        )
        .with_memory_id(MemoryId(21))
        .with_request_id("req-policy-21")
        .with_related_run("incident-2026-03-20")
        .with_redaction(),
    );
    log.append(
        AuditLogEntry::new(
            AuditEventCategory::Maintenance,
            AuditEventKind::MaintenanceMigrationApplied,
            namespace.clone(),
            "migration_runner",
            "applied audit-log schema migration",
        )
        .with_memory_id(MemoryId(21))
        .with_request_id("req-migration-21")
        .with_related_run("migration-0042"),
    );
    log.append(AuditLogEntry::new(
        AuditEventCategory::Archive,
        AuditEventKind::ArchiveRecorded,
        namespace.clone(),
        "cold_store",
        "archived superseded evidence",
    ));
    log.append(AuditLogEntry::new(
        AuditEventCategory::Recall,
        AuditEventKind::RecallServed,
        namespace.clone(),
        "recall_engine",
        "served filtered audit history preview",
    ));
    log
}

fn parse_audit_category(value: &str) -> Option<AuditEventCategory> {
    match value {
        "encode" => Some(AuditEventCategory::Encode),
        "recall" => Some(AuditEventCategory::Recall),
        "policy" => Some(AuditEventCategory::Policy),
        "maintenance" => Some(AuditEventCategory::Maintenance),
        "archive" => Some(AuditEventCategory::Archive),
        _ => None,
    }
}

fn parse_audit_kind(value: &str) -> Option<AuditEventKind> {
    match value {
        "encode_accepted" => Some(AuditEventKind::EncodeAccepted),
        "encode_rejected" => Some(AuditEventKind::EncodeRejected),
        "recall_served" => Some(AuditEventKind::RecallServed),
        "recall_denied" => Some(AuditEventKind::RecallDenied),
        "policy_denied" => Some(AuditEventKind::PolicyDenied),
        "policy_redacted" => Some(AuditEventKind::PolicyRedacted),
        "approved_sharing" => Some(AuditEventKind::ApprovedSharing),
        "maintenance_repair_started" => Some(AuditEventKind::MaintenanceRepairStarted),
        "maintenance_repair_completed" => Some(AuditEventKind::MaintenanceRepairCompleted),
        "maintenance_repair_degraded" => Some(AuditEventKind::MaintenanceRepairDegraded),
        "maintenance_repair_rollback_triggered" => {
            Some(AuditEventKind::MaintenanceRepairRollbackTriggered)
        }
        "maintenance_repair_rollback_completed" => {
            Some(AuditEventKind::MaintenanceRepairRollbackCompleted)
        }
        "maintenance_migration_applied" => Some(AuditEventKind::MaintenanceMigrationApplied),
        "maintenance_compaction_applied" => Some(AuditEventKind::MaintenanceCompactionApplied),
        "maintenance_consolidation_started" => {
            Some(AuditEventKind::MaintenanceConsolidationStarted)
        }
        "maintenance_consolidation_completed" => {
            Some(AuditEventKind::MaintenanceConsolidationCompleted)
        }
        "maintenance_consolidation_partial" => {
            Some(AuditEventKind::MaintenanceConsolidationPartial)
        }
        "maintenance_reconsolidation_applied" => {
            Some(AuditEventKind::MaintenanceReconsolidationApplied)
        }
        "maintenance_reconsolidation_discarded" => {
            Some(AuditEventKind::MaintenanceReconsolidationDiscarded)
        }
        "maintenance_reconsolidation_deferred" => {
            Some(AuditEventKind::MaintenanceReconsolidationDeferred)
        }
        "maintenance_reconsolidation_blocked" => {
            Some(AuditEventKind::MaintenanceReconsolidationBlocked)
        }
        "incident_recorded" => Some(AuditEventKind::IncidentRecorded),
        "archive_recorded" => Some(AuditEventKind::ArchiveRecorded),
        _ => None,
    }
}

fn sharing_trace_policy_summary(
    namespace: &NamespaceId,
    visibility: &'static str,
    outcome_class: membrain_core::observability::OutcomeClass,
    redaction_fields: Vec<&'static str>,
) -> TracePolicySummary {
    TracePolicySummary {
        effective_namespace: namespace.as_str().to_string(),
        policy_family: "visibility_sharing",
        outcome_class,
        blocked_stage: "policy_gate",
        filters: vec![membrain_core::api::PolicyFilterSummary::new(
            namespace.as_str(),
            "visibility_sharing",
            outcome_class,
            "policy_gate",
            membrain_core::api::FieldPresence::Present(visibility.to_string()),
            membrain_core::api::FieldPresence::Absent,
            redaction_fields
                .iter()
                .map(|field| (*field).to_string())
                .collect(),
        )],
        redaction_fields,
        retention_state: membrain_core::api::FieldPresence::Absent,
        sharing_scope: membrain_core::api::FieldPresence::Present(visibility),
    }
}

fn share_output(memory_id: u64, namespace: &NamespaceId, visibility: &'static str) -> ShareOutput {
    let policy_summary = sharing_trace_policy_summary(
        namespace,
        visibility,
        membrain_core::observability::OutcomeClass::Accepted,
        Vec::new(),
    );
    let mut audit = sample_audit_log(namespace);
    audit.append(
        AuditLogEntry::new(
            AuditEventCategory::Policy,
            AuditEventKind::ApprovedSharing,
            namespace.clone(),
            "cli_share",
            format!("visibility set to {visibility}"),
        )
        .with_memory_id(MemoryId(memory_id))
        .with_request_id(format!("req-share-{memory_id}")),
    );
    let rows = filter_audit_rows(
        &audit,
        namespace,
        Some(memory_id),
        None,
        None,
        Some("approved_sharing"),
        Some(1),
    )
    .expect("known audit op should produce filtered rows")
    .rows;

    ShareOutput {
        outcome: "accepted",
        memory_id,
        namespace: namespace.as_str().to_string(),
        visibility,
        policy_filters_applied: policy_summary.filters.clone(),
        policy_summary,
        audit_rows: rows,
    }
}

fn unshare_output(memory_id: u64, namespace: &NamespaceId) -> ShareOutput {
    let policy_summary = sharing_trace_policy_summary(
        namespace,
        "private",
        membrain_core::observability::OutcomeClass::Accepted,
        vec!["sharing_scope"],
    );
    let mut audit = sample_audit_log(namespace);
    audit.append(
        AuditLogEntry::new(
            AuditEventCategory::Policy,
            AuditEventKind::PolicyRedacted,
            namespace.clone(),
            "cli_unshare",
            "tightened visibility back to private",
        )
        .with_memory_id(MemoryId(memory_id))
        .with_request_id(format!("req-unshare-{memory_id}"))
        .with_redaction(),
    );
    let rows = filter_audit_rows(
        &audit,
        namespace,
        Some(memory_id),
        None,
        None,
        Some("policy_redacted"),
        Some(1),
    )
    .expect("known audit op should produce filtered rows")
    .rows;

    ShareOutput {
        outcome: "accepted",
        memory_id,
        namespace: namespace.as_str().to_string(),
        visibility: "private",
        policy_filters_applied: policy_summary.filters.clone(),
        policy_summary,
        audit_rows: rows,
    }
}

fn filter_audit_rows(
    log: &AppendOnlyAuditLog,
    namespace: &NamespaceId,
    memory_id: Option<u64>,
    session_id: Option<u64>,
    since: Option<u64>,
    op: Option<&str>,
    recent: Option<usize>,
) -> anyhow::Result<AuditExport> {
    let op = op.map(str::trim).filter(|value| !value.is_empty());
    let category = op.and_then(parse_audit_category);
    let kind = op.and_then(parse_audit_kind);
    if let Some(op_value) = op.filter(|_| category.is_none() && kind.is_none()) {
        anyhow::bail!(
            "unknown audit --op value `{}`; expected a known category or kind",
            op_value
        );
    }
    let filter = AuditLogFilter {
        namespace: Some(namespace.clone()),
        memory_id: memory_id.map(MemoryId),
        session_id: session_id.map(SessionId),
        category,
        kind,
        min_sequence: since,
        ..AuditLogFilter::default()
    };
    let AuditLogSlice {
        rows,
        total_matches,
        truncated,
    } = log.slice(&filter, recent);
    let rows = rows.into_iter().map(AuditRow::from).collect::<Vec<_>>();

    Ok(AuditExport {
        total_matches,
        returned_rows: rows.len(),
        truncated,
        rows,
    })
}

fn print_audit_rows(export: &AuditExport, json: bool) -> anyhow::Result<()> {
    if json {
        println!("{}", serde_json::to_string_pretty(export)?);
        return Ok(());
    }

    println!(
        "matched={} returned={} truncated={}",
        export.total_matches, export.returned_rows, export.truncated
    );

    if export.rows.is_empty() {
        if export.total_matches == 0 {
            println!("No audit rows matched the requested filters.");
        } else {
            println!("Audit rows matched, but the requested slice returned zero rows.");
        }
        return Ok(());
    }

    for row in &export.rows {
        println!(
            "#{} {} {} ns={} memory={:?} session={:?} actor={} request_id={:?} redacted={} run={:?} note={}",
            row.sequence,
            row.category,
            row.kind,
            row.namespace,
            row.memory_id,
            row.session_id,
            row.triggered_by,
            row.request_id,
            row.redacted,
            row.related_run,
            row.note,
        );
    }

    Ok(())
}

fn sample_runtime_status() -> RuntimeStatus {
    RuntimeStatus {
        posture: RuntimePosture::Full,
        degraded_reasons: Vec::new(),
        metrics: RuntimeMetrics {
            queue_depth: 0,
            active_requests: 0,
            background_jobs: 0,
            cancelled_requests: 0,
            maintenance_runs: 0,
        },
    }
}

fn doctor_report() -> DoctorReport {
    let status = sample_runtime_status();
    let index_reports = IndexModule.health_reports();
    let indexes = index_reports
        .iter()
        .map(|report| DoctorIndexRow {
            family: report.family.as_str(),
            health: report.health.as_str(),
            usable: report.health.is_usable(),
            entry_count: report.entry_count,
            generation: report.generation,
        })
        .collect::<Vec<_>>();
    let warnings = indexes
        .iter()
        .filter_map(|row| match row.health {
            "stale" => Some("index_stale"),
            "needs_rebuild" => Some("index_needs_rebuild"),
            "missing" => Some("index_missing"),
            _ => None,
        })
        .collect::<Vec<_>>();
    let overall_status = if warnings.is_empty() { "ok" } else { "warn" };
    let store = BrainStore::new(RuntimeConfig::default());
    let repair_engine = store.repair_engine();
    let namespace = NamespaceId::new("doctor.system").expect("doctor namespace should be valid");
    let mut repair_handle = MaintenanceJobHandle::new(
        repair_engine.create_targeted(
            namespace.clone(),
            vec![RepairTarget::LexicalIndex, RepairTarget::MetadataIndex],
            IndexRepairEntrypoint::RebuildIfNeeded,
        ),
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
                    .map(|result| RepairReportRow {
                        target: result.target.as_str(),
                        status: result.status.as_str(),
                        verification_passed: result.verification_passed,
                        rebuild_entrypoint: result
                            .rebuild_entrypoint
                            .map(IndexRepairEntrypoint::as_str),
                        rebuilt_outputs: result.rebuilt_outputs.clone(),
                    })
                    .collect();
                health_repair_summary = Some(summary);
                break;
            }
            MaintenanceJobState::Running { .. } => continue,
            _ => break,
        }
    }

    let availability = (!status.degraded_reasons.is_empty()).then(|| {
        AvailabilitySummary::degraded(
            vec!["doctor", "health", "audit"],
            vec!["encode", "maintenance"],
            vec![AvailabilityReason::RepairInFlight],
            vec![RemediationStep::CheckHealth, RemediationStep::RunRepair],
        )
    });

    let mut cache = CacheManager::new(4, 4);
    cache.result.disable();
    cache.prefetch.submit_hint(
        namespace.clone(),
        membrain_core::store::cache::PrefetchTrigger::SessionRecency,
        vec![],
    );

    let health = BrainHealthReport::from_inputs(
        BrainHealthInputs {
            hot_memories: 76,
            hot_capacity: 100,
            cold_memories: 12,
            avg_strength: 0.71,
            avg_confidence: 0.84,
            low_confidence_count: 3,
            decay_rate: 0.012,
            archive_count: 5,
            total_engrams: 14,
            avg_cluster_size: 2.5,
            top_engrams: vec![("ops".to_string(), 4)],
            landmark_count: 2,
            unresolved_conflicts: 1,
            uncertain_count: 3,
            dream_links_total: 9,
            last_dream_tick: Some(42),
            total_recalls: 55,
            total_encodes: 12,
            current_tick: 200,
            daemon_uptime_ticks: 180,
            index_reports,
            availability,
            feature_availability: vec![FeatureAvailabilityEntry {
                feature: "health".to_string(),
                posture: membrain_core::api::AvailabilityPosture::Full,
                note: Some("cli_doctor_embeds_brain_health_report".to_string()),
            }],
            previous_total_recalls: Some(44),
            previous_total_encodes: Some(10),
            previous_repair_queue_depth: Some(0),
        },
        &cache,
        health_repair_summary.as_ref(),
    );

    DoctorReport {
        status: overall_status,
        action: "doctor",
        posture: status.posture.as_str(),
        degraded_reasons: status.degraded_reasons,
        metrics: status.metrics,
        indexes,
        repair_engine_component: repair_engine.component_name(),
        repair_reports,
        warnings,
        health,
    }
}

fn print_doctor_report(report: &DoctorReport) -> anyhow::Result<()> {
    println!("{}", serde_json::to_string_pretty(report)?);
    Ok(())
}

// ── Entry point ──────────────────────────────────────────────────────────────

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    // Shared core store and hot metadata store for encode/recall/inspect/explain
    let store = BrainStore::new(RuntimeConfig::default());
    let mut hot = store.hot_store().new_metadata_store(64);
    let mut local_records: Vec<LocalMemoryRecord> = Vec::new();
    let _session_id = SessionId(SESSION_ID.load(Ordering::SeqCst));

    match &cli.command {
        Commands::Encode {
            content,
            namespace,
            kind,
            context,
            attention,
            valence,
            arousal,
            source,
            json,
        } => {
            let ns = NamespaceId::new(namespace)?;
            let response = encode_memory(
                &store,
                &mut hot,
                &mut local_records,
                content,
                &ns,
                kind,
                context.as_deref(),
                *attention,
                *valence,
                *arousal,
                source,
            );
            if *json {
                println!("{}", serde_json::to_string_pretty(&response)?);
            } else {
                let output = response.result.as_ref().expect("encode result present");
                println!(
                    "Encoded memory #{} in '{}' [{} / {}]",
                    output.memory_id, output.namespace, output.memory_type, output.route_family,
                );
                println!("  text: {}", output.compact_text);
                println!("  salience: {}", output.provisional_salience);
                println!("  fingerprint: {}", output.fingerprint);
                if output.is_landmark {
                    println!(
                        "  landmark: {}",
                        output.landmark_label.as_deref().unwrap_or("(auto)")
                    );
                }
            }
        }
        Commands::Recall {
            query,
            namespace,
            top,
            context: _,
            kind: _,
            confidence: _,
            explain,
            json,
        } => {
            let ns = NamespaceId::new(namespace)?;
            let response = recall_memories(&store, &hot, &local_records, query, &ns, *top, explain);
            if *json {
                println!("{}", serde_json::to_string_pretty(&response)?);
            } else {
                let result_set = response.result.as_ref().expect("recall result present");
                println!(
                    "Recall '{}' in '{}' → {} results",
                    query,
                    ns.as_str(),
                    result_set.evidence_pack.len(),
                );
                if let Some(route_summary) = response.route_summary.as_ref() {
                    println!(
                        "  route: {} → {}",
                        route_summary.route_family, route_summary.route_reason
                    );
                }
                if explain != "none" {
                    println!(
                        "  tier1: consulted={}, answered_directly={}, deeper={}",
                        response
                            .route_summary
                            .as_ref()
                            .map(|summary| summary.tier1_consulted_first)
                            .unwrap_or(false),
                        response
                            .route_summary
                            .as_ref()
                            .map(|summary| summary.tier1_answered_directly)
                            .unwrap_or(false),
                        response
                            .route_summary
                            .as_ref()
                            .map(|summary| summary.routes_to_deeper_tiers)
                            .unwrap_or(false),
                    );
                }
                for (i, item) in result_set.evidence_pack.iter().enumerate() {
                    println!(
                        "  [{}] #{} score={} lane={} | {}",
                        i + 1,
                        item.result.memory_id.0,
                        item.result.score,
                        item.result.entry_lane.as_str(),
                        item.result.compact_text,
                    );
                }
                if let Some(action_pack) = result_set.action_pack.as_ref() {
                    println!("  derived actions: {}", action_pack.len());
                    for (i, action) in action_pack.iter().enumerate() {
                        println!(
                            "    ({}) {} [{}] -> evidence {:?}",
                            i + 1,
                            action.action_type,
                            action.confidence_score,
                            action.supporting_evidence,
                        );
                        println!("       {}", action.suggestion);
                        if !action.uncertainty_markers.is_empty() {
                            println!(
                                "       uncertainty: {}",
                                action.uncertainty_markers.join(", ")
                            );
                        }
                        if !action.policy_caveats.is_empty() {
                            println!("       policy: {}", action.policy_caveats.join(", "));
                        }
                        if !action.freshness_caveats.is_empty() {
                            println!("       freshness: {}", action.freshness_caveats.join(", "));
                        }
                    }
                }
            }
        }
        Commands::Inspect {
            id,
            namespace,
            json,
        } => {
            let ns = NamespaceId::new(namespace)?;
            let memory_id = MemoryId(*id);
            match inspect_memory(&mut hot, &local_records, &ns, memory_id) {
                Ok(output) => {
                    if *json {
                        let response = ResponseContext::success(
                            ns.clone(),
                            RequestId::new(format!("inspect-{}", output.memory_id))?,
                            output,
                        );
                        println!("{}", serde_json::to_string_pretty(&response)?);
                    } else {
                        println!(
                            "Inspect #{} in '{}' [{} / {}]",
                            output.memory_id,
                            output.namespace,
                            output.memory_type,
                            output.route_family,
                        );
                        println!("  text: {}", output.compact_text);
                        println!("  salience: {}", output.provisional_salience);
                        println!("  fingerprint: {}", output.fingerprint);
                        println!(
                            "  payload: {} bytes ({})",
                            output.payload_size_bytes, output.payload_state
                        );
                        println!("  session: {}", output.session_id);
                    }
                }
                Err(e) => {
                    if *json {
                        let resp: ResponseContext<()> = ResponseContext::failure(
                            ns,
                            RequestId::new("inspect-not-found")?,
                            membrain_core::api::ErrorKind::ValidationFailure,
                            vec![],
                        );
                        let mut json = serde_json::to_value(&resp)?;
                        if let Some(object) = json.as_object_mut() {
                            object.insert(
                                "error_kind".to_string(),
                                serde_json::Value::String("validation_failure".to_string()),
                            );
                            if let Some(remediation) = object
                                .get_mut("remediation")
                                .and_then(serde_json::Value::as_object_mut)
                            {
                                remediation.insert(
                                    "summary".to_string(),
                                    serde_json::Value::String("validation_failure".to_string()),
                                );
                                remediation.insert(
                                    "next_steps".to_string(),
                                    serde_json::Value::Array(vec![serde_json::Value::String(
                                        "fix_request".to_string(),
                                    )]),
                                );
                            }
                        }
                        println!("{}", serde_json::to_string_pretty(&json)?);
                    } else {
                        eprintln!("Error: {}", e);
                        std::process::exit(1);
                    }
                }
            }
        }
        Commands::Explain {
            query,
            namespace,
            json,
        } => {
            let ns = NamespaceId::new(namespace)?;
            let response = explain_query(&store, &local_records, query, &ns);
            if *json {
                println!("{}", serde_json::to_string_pretty(&response)?);
            } else {
                println!("Explain '{}' in '{}'", query, ns.as_str());
                if let Some(route_summary) = response.route_summary.as_ref() {
                    println!(
                        "  route: {} → {}",
                        route_summary.route_family, route_summary.route_reason
                    );
                    println!(
                        "  tier1: consulted={}, answered_directly={}, deeper={}",
                        route_summary.tier1_consulted_first,
                        route_summary.tier1_answered_directly,
                        route_summary.routes_to_deeper_tiers,
                    );
                }
                let trace_stages = response
                    .trace_stages
                    .iter()
                    .map(|stage| stage.as_str())
                    .collect::<Vec<_>>();
                println!("  trace_stages: {}", trace_stages.join(" → "));
                let details = response
                    .result_reasons
                    .iter()
                    .map(|reason| reason.detail.as_str())
                    .collect::<Vec<_>>();
                if !details.is_empty() {
                    println!("  reasons: {}", details.join(" | "));
                }
            }
        }
        Commands::Maintenance {
            action,
            namespace,
            json,
        } => {
            let ns_str = namespace.as_deref().unwrap_or("default");
            let ns = NamespaceId::new(ns_str)?;

            let targets = match action.as_str() {
                "repair" | "repair_all" => {
                    vec![RepairTarget::LexicalIndex, RepairTarget::MetadataIndex]
                }
                "repair_index" | "repair_indexes" => vec![RepairTarget::LexicalIndex],
                "repair_metadata" => vec![RepairTarget::MetadataIndex],
                "repair_graph" | "repair_lineage" | "repair_cache" => {
                    eprintln!(
                        "Warning: repair target '{}' not yet implemented, falling back to index repair",
                        action
                    );
                    vec![RepairTarget::LexicalIndex]
                }
                _ => {
                    eprintln!(
                        "Unknown maintenance action '{}'. Available: repair, repair_index, repair_metadata",
                        action
                    );
                    std::process::exit(1);
                }
            };

            let run = store.repair_engine().create_targeted(
                ns.clone(),
                targets,
                IndexRepairEntrypoint::RebuildIfNeeded,
            );
            let mut handle = MaintenanceJobHandle::new(run, 8);
            handle.start();

            let mut output = None;
            for _ in 0..16 {
                let snapshot = handle.poll();
                match snapshot.state {
                    MaintenanceJobState::Completed(summary) => {
                        output = Some(MaintenanceOutput {
                            outcome: "accepted",
                            action: action.clone(),
                            namespace: ns.as_str().to_string(),
                            targets_checked: summary.targets_checked,
                            rebuilt: summary.rebuilt,
                            results: summary
                                .results
                                .iter()
                                .map(|r| RepairResultOutput {
                                    target: r.target.as_str(),
                                    status: r.status.as_str(),
                                    verification_passed: r.verification_passed,
                                    rebuild_entrypoint: r
                                        .rebuild_entrypoint
                                        .map(IndexRepairEntrypoint::as_str),
                                    rebuilt_outputs: r.rebuilt_outputs.clone(),
                                })
                                .collect(),
                            warnings: Vec::new(),
                        });
                        break;
                    }
                    MaintenanceJobState::Running { .. } => continue,
                    _ => {
                        eprintln!("Maintenance job did not complete normally");
                        std::process::exit(3);
                    }
                }
            }

            match output {
                Some(result) => {
                    if *json {
                        println!("{}", serde_json::to_string_pretty(&result)?);
                    } else {
                        println!(
                            "Maintenance '{}' on '{}' → {} checked, {} rebuilt",
                            result.action, result.namespace, result.targets_checked, result.rebuilt,
                        );
                        for r in &result.results {
                            println!(
                                "  {} [{}] verified={} outputs={:?}",
                                r.target, r.status, r.verification_passed, r.rebuilt_outputs
                            );
                        }
                    }
                }
                None => {
                    eprintln!("Maintenance job timed out");
                    std::process::exit(3);
                }
            }
        }
        Commands::Benchmark {
            target,
            iters,
            json,
        } => {
            let store_bench = BrainStore::new(RuntimeConfig::default());
            let mut durations_ns: Vec<u128> = Vec::with_capacity(*iters);

            match target.as_str() {
                "encode" | "tier1" => {
                    for i in 0..*iters {
                        let input = RawEncodeInput::new(
                            RawIntakeKind::Event,
                            format!("benchmark test content iteration {}", i),
                        );
                        let start = Instant::now();
                        let _ = store_bench.encode_engine().prepare_fast_path(input);
                        durations_ns.push(start.elapsed().as_nanos());
                    }
                }
                "recall" | "intent" | "retrieval" => {
                    for i in 0..*iters {
                        let query = format!("benchmark query {}", i);
                        let start = Instant::now();
                        if target == "intent" {
                            let _ = store_bench.intent_engine().classify(&query);
                        } else {
                            let _ = store_bench.recall_engine().plan_recall(
                                membrain_core::engine::recall::RecallRequest::small_session_lookup(
                                    SessionId(1),
                                ),
                                store_bench.config(),
                            );
                        }
                        durations_ns.push(start.elapsed().as_nanos());
                    }
                }
                _ => {
                    eprintln!(
                        "Unknown benchmark target '{}'. Available: encode, recall, intent, tier1, retrieval",
                        target
                    );
                    std::process::exit(1);
                }
            }

            durations_ns.sort();
            let total_ns: u128 = durations_ns.iter().sum();
            let total_ms = total_ns as f64 / 1_000_000.0;
            let avg_us = (total_ns / (*iters as u128)) as f64 / 1_000.0;
            let min_us = durations_ns[0] as f64 / 1_000.0;
            let max_us = durations_ns[durations_ns.len() - 1] as f64 / 1_000.0;
            let p50_idx = durations_ns.len() / 2;
            let p95_idx = (durations_ns.len() as f64 * 0.95) as usize;
            let p50_us = durations_ns[p50_idx] as f64 / 1_000.0;
            let p95_us = durations_ns[p95_idx.min(durations_ns.len() - 1)] as f64 / 1_000.0;

            let output = BenchmarkOutput {
                outcome: "accepted",
                target: target.clone(),
                iterations: *iters,
                total_duration_ms: total_ms,
                avg_duration_us: avg_us,
                min_duration_us: min_us,
                max_duration_us: max_us,
                p50_duration_us: p50_us,
                p95_duration_us: p95_us,
            };

            if *json {
                println!("{}", serde_json::to_string_pretty(&output)?);
            } else {
                println!(
                    "Benchmark '{}': {} iters, avg={:.1}us, min={:.1}us, max={:.1}us, p50={:.1}us, p95={:.1}us, total={:.1}ms",
                    target, iters, avg_us, min_us, max_us, p50_us, p95_us, total_ms
                );
            }
        }
        Commands::Doctor => {
            let report = doctor_report();
            print_doctor_report(&report)?;
        }
        Commands::Audit {
            namespace,
            id,
            session,
            since,
            op,
            recent,
            json,
        } => {
            let ns = NamespaceId::new(namespace)?;
            let log = sample_audit_log(&ns);
            let export =
                filter_audit_rows(&log, &ns, *id, *session, *since, op.as_deref(), *recent)?;
            print_audit_rows(&export, *json)?;
        }
        Commands::Share {
            id,
            namespace_id,
            json,
        } => {
            let ns = NamespaceId::new(namespace_id)?;
            let output = share_output(*id, &ns, "shared");
            if *json {
                let response = ResponseContext::success(
                    ns.clone(),
                    RequestId::new(format!("share-{id}"))?,
                    output,
                )
                .with_policy_filters(vec![
                    membrain_core::api::PolicyFilterSummary::new(
                        ns.as_str(),
                        "visibility_sharing",
                        membrain_core::observability::OutcomeClass::Accepted,
                        "policy_gate",
                        membrain_core::api::FieldPresence::Present("shared".to_string()),
                        membrain_core::api::FieldPresence::Absent,
                        Vec::new(),
                    ),
                ]);
                println!("{}", serde_json::to_string_pretty(&response)?);
            } else {
                println!(
                    "Shared memory #{} into '{}' [{}]",
                    id,
                    ns.as_str(),
                    output.visibility
                );
            }
        }
        Commands::Unshare {
            id,
            namespace,
            json,
        } => {
            let ns = NamespaceId::new(namespace)?;
            let output = unshare_output(*id, &ns);
            if *json {
                let response = ResponseContext::success(
                    ns.clone(),
                    RequestId::new(format!("unshare-{id}"))?,
                    output,
                )
                .with_policy_filters(vec![
                    membrain_core::api::PolicyFilterSummary::new(
                        ns.as_str(),
                        "visibility_sharing",
                        membrain_core::observability::OutcomeClass::Accepted,
                        "policy_gate",
                        membrain_core::api::FieldPresence::Present("private".to_string()),
                        membrain_core::api::FieldPresence::Absent,
                        vec!["sharing_scope".to_string()],
                    ),
                ]);
                println!("{}", serde_json::to_string_pretty(&response)?);
            } else {
                println!("Unshared memory #{} in '{}' [private]", id, ns.as_str());
            }
        }
        Commands::Daemon {
            socket_path,
            request_concurrency,
            max_queue_depth,
        } => {
            let mut config = DaemonRuntimeConfig::new(socket_path);
            config.request_concurrency = *request_concurrency;
            config.max_queue_depth = *max_queue_depth;
            let runtime = DaemonRuntime::with_config(config);
            runtime.run_until_stopped().await?;
        }
    }

    Ok(())
}

#[cfg(test)]
#[allow(clippy::items_after_test_module)]
mod tests {
    use super::{
        filter_audit_rows, parse_audit_category, parse_audit_kind, print_audit_rows,
        response_trace_for_result_set, sample_audit_log, share_output, unshare_output, Cli,
        Commands,
    };
    use clap::Parser;
    use membrain_core::api::{NamespaceId, TraceStage};
    use membrain_core::engine::confidence::{ConfidenceInputs, ConfidencePolicy};
    use membrain_core::engine::ranking::{fuse_scores, RankingInput, RankingProfile};
    use membrain_core::engine::result::{AnsweredFrom, ResultBuilder, RetrievalExplain};
    use membrain_core::engine::recall::{RecallPlanKind, RecallTraceStage};
    use membrain_core::types::{CanonicalMemoryType, MemoryId, SessionId};

    #[test]
    fn audit_rows_preserve_request_id_in_json_export() {
        let namespace = NamespaceId::new("team.alpha").expect("valid namespace");
        let log = sample_audit_log(&namespace);
        let export = filter_audit_rows(&log, &namespace, Some(21), None, None, None, None)
            .expect("valid audit export");

        assert_eq!(export.total_matches, 3);
        assert_eq!(export.returned_rows, 3);
        assert!(!export.truncated);
        assert_eq!(export.rows[0].request_id.as_deref(), Some("req-encode-21"));
        assert_eq!(export.rows[1].request_id.as_deref(), Some("req-policy-21"));
        assert_eq!(
            export.rows[2].request_id.as_deref(),
            Some("req-migration-21")
        );
    }

    fn parsed_recall_namespace_and_top(command: Commands) -> Option<(String, usize)> {
        match command {
            Commands::Recall { namespace, top, .. } => Some((namespace, top)),
            _ => None,
        }
    }

    #[test]
    fn recall_top_option_accepts_namespace_short_alias() {
        let cli = Cli::parse_from([
            "membrain",
            "recall",
            "search-term",
            "-n",
            "7",
            "--namespace",
            "team.alpha",
        ]);

        assert_eq!(
            parsed_recall_namespace_and_top(cli.command),
            Some(("team.alpha".to_string(), 7))
        );
    }

    #[test]
    fn recall_top_option_accepts_legacy_short_alias() {
        let cli = Cli::parse_from([
            "membrain",
            "recall",
            "search-term",
            "-t",
            "4",
            "--namespace",
            "team.alpha",
        ]);

        assert_eq!(
            parsed_recall_namespace_and_top(cli.command),
            Some(("team.alpha".to_string(), 4))
        );
    }

    #[test]
    fn recall_namespace_long_flag_still_works_with_top_short_alias() {
        let cli = Cli::parse_from([
            "membrain",
            "recall",
            "search-term",
            "-n",
            "3",
            "--namespace",
            "team.alpha",
        ]);

        assert_eq!(
            parsed_recall_namespace_and_top(cli.command),
            Some(("team.alpha".to_string(), 3))
        );
    }

    #[test]
    fn remember_command_preserves_encode_surface() {
        let cli = Cli::parse_from([
            "membrain",
            "remember",
            "captured memory",
            "--namespace",
            "team.alpha",
        ]);

        match cli.command {
            Commands::Encode {
                content,
                namespace,
                kind,
                ..
            } => {
                assert_eq!(content, "captured memory");
                assert_eq!(namespace, "team.alpha");
                assert_eq!(kind, "episodic");
            }
            other => panic!("expected remember to parse as encode surface, got {other:?}"),
        }
    }

    #[test]
    fn why_command_preserves_explain_surface() {
        let cli = Cli::parse_from([
            "membrain",
            "why",
            "routing details",
            "--namespace",
            "team.alpha",
        ]);

        match cli.command {
            Commands::Explain {
                query, namespace, ..
            } => {
                assert_eq!(query, "routing details");
                assert_eq!(namespace, "team.alpha");
            }
            other => panic!("expected why to parse as explain surface, got {other:?}"),
        }
    }

    #[test]
    fn response_trace_bundle_uses_canonical_explain_route_stages() {
        let namespace = NamespaceId::new("team.gamma").unwrap();
        let result_set = super::build_retrieval_result_set(
            &[],
            &namespace,
            3,
            membrain_core::engine::ranking::RankingProfile::balanced(),
            "balanced",
            "small lookup for active session can stay on hot recent window before durable fallback"
                .to_string(),
            Vec::new(),
        );

        let (_, trace_stages, ..) = response_trace_for_result_set(&result_set);

        assert_eq!(
            trace_stages,
            vec![
                TraceStage::Tier1RecentWindow,
                TraceStage::PolicyGate,
                TraceStage::Packaging
            ]
        );
    }

    #[test]
    fn response_trace_bundle_uses_canonical_uncertainty_markers() {
        let namespace = NamespaceId::new("team.gamma").unwrap();
        let mut builder = ResultBuilder::new(1, namespace.clone());
        let ranked = fuse_scores(
            RankingInput {
                recency: 250,
                salience: 250,
                strength: 250,
                provenance: 250,
                conflict: 500,
                confidence: 250,
            },
            RankingProfile::balanced(),
        );
        builder.add_with_confidence(
            MemoryId(77),
            namespace.clone(),
            SessionId(3),
            CanonicalMemoryType::Event,
            "high uncertainty result".into(),
            &ranked,
            AnsweredFrom::Tier2Indexed,
            &ConfidenceInputs {
                corroboration_count: 0,
                ticks_since_last_access: 128,
                age_ticks: 256,
                resolution_state: membrain_core::engine::contradiction::ResolutionState::None,
                conflict_score: 0,
                causal_parent_count: 0,
                authoritativeness: 0,
                recall_count: 0,
            },
            &ConfidencePolicy::default(),
        );
        let explain = RetrievalExplain {
            recall_plan: RecallPlanKind::Tier2ExactThenTier3Fallback,
            route_reason: "uncertainty marker test".to_string(),
            tiers_consulted: vec!["tier2_exact".to_string()],
            trace_stages: vec![RecallTraceStage::Tier2Exact],
            tier1_answered_directly: false,
            candidate_budget: 1,
            time_consumed_ms: Some(5),
            ranking_profile: "balanced".to_string(),
            contradictions_found: 0,
            query_by_example: None,
            result_reasons: vec![],
        };
        let result_set = builder.build(explain);

        let (_, _, _, _, _, _, _, _, _, _, uncertainty_markers) =
            response_trace_for_result_set(&result_set);
        let (_, _, expected_uncertainty_markers) = result_set.explain_markers();

        assert_eq!(uncertainty_markers.len(), 1);
        assert_eq!(expected_uncertainty_markers.len(), 1);
        assert_eq!(uncertainty_markers[0].code, expected_uncertainty_markers[0].code);
        assert_eq!(
            uncertainty_markers[0].detail,
            expected_uncertainty_markers[0].detail
        );
    }

    #[test]
    fn legacy_encode_and_explain_aliases_still_parse() {
        let encode = Cli::parse_from(["membrain", "encode", "legacy path"]);
        let explain = Cli::parse_from(["membrain", "explain", "legacy why"]);

        assert!(matches!(encode.command, Commands::Encode { .. }));
        assert!(matches!(explain.command, Commands::Explain { .. }));
    }

    #[test]
    fn share_and_unshare_commands_parse_canonical_fields() {
        let share = Cli::parse_from([
            "membrain",
            "share",
            "--id",
            "42",
            "--namespace",
            "team.beta",
        ]);
        let unshare = Cli::parse_from([
            "membrain",
            "unshare",
            "--id",
            "42",
            "--namespace",
            "team.alpha",
        ]);

        match share.command {
            Commands::Share {
                id,
                namespace_id,
                json,
            } => {
                assert_eq!(id, 42);
                assert_eq!(namespace_id, "team.beta");
                assert!(!json);
            }
            other => panic!("expected share command, got {other:?}"),
        }

        match unshare.command {
            Commands::Unshare {
                id,
                namespace,
                json,
            } => {
                assert_eq!(id, 42);
                assert_eq!(namespace, "team.alpha");
                assert!(!json);
            }
            other => panic!("expected unshare command, got {other:?}"),
        }
    }

    #[test]
    fn share_and_unshare_outputs_preserve_policy_and_audit_fields() {
        let shared = share_output(42, &NamespaceId::new("team.beta").unwrap(), "shared");
        assert_eq!(shared.outcome, "accepted");
        assert_eq!(shared.memory_id, 42);
        assert_eq!(shared.visibility, "shared");
        assert_eq!(shared.policy_summary.policy_family, "visibility_sharing");
        assert_eq!(shared.policy_summary.sharing_scope.state_name(), "present");
        assert_eq!(shared.audit_rows.len(), 1);
        assert_eq!(
            shared.audit_rows[0].request_id.as_deref(),
            Some("req-share-42")
        );
        assert_eq!(shared.audit_rows[0].kind, "approved_sharing");

        let unshared = unshare_output(42, &NamespaceId::new("team.alpha").unwrap());
        assert_eq!(unshared.visibility, "private");
        assert_eq!(
            unshared.policy_summary.redaction_fields,
            vec!["sharing_scope"]
        );
        assert_eq!(unshared.audit_rows.len(), 1);
        assert_eq!(
            unshared.audit_rows[0].request_id.as_deref(),
            Some("req-unshare-42")
        );
        assert_eq!(unshared.audit_rows[0].kind, "policy_redacted");
        assert!(unshared.audit_rows[0].redacted);
    }

    #[test]
    fn audit_export_reports_truncation_for_recent_limit() {
        let namespace = NamespaceId::new("team.alpha").expect("valid namespace");
        let log = sample_audit_log(&namespace);
        let export = filter_audit_rows(&log, &namespace, Some(21), None, None, None, Some(1))
            .expect("valid audit export");

        assert_eq!(export.total_matches, 3);
        assert_eq!(export.returned_rows, 1);
        assert!(export.truncated);
        assert_eq!(
            export.rows[0].request_id.as_deref(),
            Some("req-migration-21")
        );
    }

    #[test]
    fn audit_op_filters_accept_both_category_and_kind_names() {
        assert_eq!(
            parse_audit_category("policy"),
            Some(membrain_core::observability::AuditEventCategory::Policy)
        );
        assert_eq!(
            parse_audit_kind("policy_redacted"),
            Some(membrain_core::observability::AuditEventKind::PolicyRedacted)
        );
        assert_eq!(
            parse_audit_kind("maintenance_consolidation_partial"),
            Some(membrain_core::observability::AuditEventKind::MaintenanceConsolidationPartial)
        );
        assert_eq!(
            parse_audit_kind("maintenance_reconsolidation_applied"),
            Some(membrain_core::observability::AuditEventKind::MaintenanceReconsolidationApplied)
        );
        assert_eq!(
            parse_audit_kind("maintenance_reconsolidation_discarded"),
            Some(membrain_core::observability::AuditEventKind::MaintenanceReconsolidationDiscarded)
        );
        assert_eq!(
            parse_audit_kind("maintenance_reconsolidation_deferred"),
            Some(membrain_core::observability::AuditEventKind::MaintenanceReconsolidationDeferred)
        );
        assert_eq!(
            parse_audit_kind("maintenance_reconsolidation_blocked"),
            Some(membrain_core::observability::AuditEventKind::MaintenanceReconsolidationBlocked)
        );
        assert_eq!(parse_audit_category("unknown"), None);
        assert_eq!(parse_audit_kind("unknown"), None);
    }

    #[test]
    fn audit_export_can_filter_by_session_id() {
        let namespace = NamespaceId::new("team.alpha").expect("valid namespace");
        let log = sample_audit_log(&namespace);
        let export = filter_audit_rows(&log, &namespace, None, Some(5), None, None, None)
            .expect("valid audit export");

        assert_eq!(export.total_matches, 1);
        assert_eq!(export.returned_rows, 1);
        assert!(!export.truncated);
        assert_eq!(export.rows[0].session_id, Some(5));
        assert_eq!(export.rows[0].kind, "encode_accepted");
    }

    #[test]
    fn audit_filter_rejects_unknown_op_values() {
        let namespace = NamespaceId::new("team.alpha").expect("valid namespace");
        let log = sample_audit_log(&namespace);
        let error = filter_audit_rows(
            &log,
            &namespace,
            Some(21),
            None,
            None,
            Some("not_a_real_op"),
            None,
        )
        .expect_err("unknown op should fail instead of silently widening the query");

        assert_eq!(
            error.to_string(),
            "unknown audit --op value `not_a_real_op`; expected a known category or kind"
        );
    }

    #[test]
    fn text_export_includes_request_id_field() {
        let namespace = NamespaceId::new("team.alpha").expect("valid namespace");
        let log = sample_audit_log(&namespace);
        let export = filter_audit_rows(&log, &namespace, Some(21), None, None, None, Some(1))
            .expect("valid audit export");

        let rendered = export
            .rows
            .iter()
            .map(|row| {
                format!(
                    "#{} {} {} ns={} memory={:?} session={:?} actor={} request_id={:?} redacted={} run={:?} note={}",
                    row.sequence,
                    row.category,
                    row.kind,
                    row.namespace,
                    row.memory_id,
                    row.session_id,
                    row.triggered_by,
                    row.request_id,
                    row.redacted,
                    row.related_run,
                    row.note,
                )
            })
            .collect::<Vec<_>>()
            .join("\n");

        assert!(rendered.contains("request_id=Some(\"req-migration-21\")"));
        assert!(rendered.contains("run=Some(\"migration-0042\")"));

        print_audit_rows(&export, false).expect("text export should render");
    }

    #[test]
    fn audit_text_export_distinguishes_empty_slice_from_no_matches() {
        let namespace = NamespaceId::new("team.alpha").expect("valid namespace");
        let log = sample_audit_log(&namespace);
        let export = filter_audit_rows(&log, &namespace, Some(21), None, None, None, Some(0))
            .expect("valid audit export");

        assert_eq!(export.total_matches, 3);
        assert_eq!(export.returned_rows, 0);
        assert!(export.truncated);

        print_audit_rows(&export, false).expect("text export should render zero-row slice");
    }
}
