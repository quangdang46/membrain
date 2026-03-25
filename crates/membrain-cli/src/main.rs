use clap::{ArgAction, Parser, Subcommand};
use membrain_core::api::{
    AvailabilityReason, AvailabilitySummary, CacheMetricsSummary, ConflictMarker, FieldPresence,
    FreshnessMarker, NamespaceId, PassiveObservationInspectSummary, RemediationStep, RequestId,
    ResponseContext, ResponseWarning, TraceOmissionSummary, TracePolicySummary,
    TraceProvenanceSummary, TraceScoreComponent, TraceStage, UncertaintyMarker,
};
use membrain_core::engine::confidence::{ConfidenceInputs, ConfidencePolicy};
use membrain_core::engine::context_budget::{
    ContextBudgetRequest, ContextBudgetResponse, InjectionFormat,
};
use membrain_core::engine::maintenance::{
    MaintenanceController, MaintenanceJobHandle, MaintenanceJobState,
};
use membrain_core::engine::observe::{ObserveConfig, ObserveEngine};
use membrain_core::engine::ranking::{
    fuse_scores, ConfidenceDisplayConfig, ConfidenceExplain, RankingInput, RankingProfile,
};
use membrain_core::engine::recall::RecallRuntime;
use membrain_core::engine::repair::{IndexRepairEntrypoint, RepairTarget};
use membrain_core::engine::result::{
    AnsweredFrom, ResultBuilder, RetrievalExplain, RetrievalResultSet,
};
use membrain_core::engine::retrieval_planner::{
    RetrievalPlanTrace, RetrievalRequest, RetrievalRequestValidationError,
};
use membrain_core::health::{BrainHealthInputs, BrainHealthReport, FeatureAvailabilityEntry};
use membrain_core::index::{IndexApi, IndexModule};
use membrain_core::observability::{AuditEventCategory, AuditEventKind};
use membrain_core::store::audit::{
    AppendOnlyAuditLog, AuditLogEntry, AuditLogFilter, AuditLogSlice, AuditLogStore,
};
use membrain_core::store::cache::CacheManager;
use membrain_core::store::hot::Tier1HotMetadataStore;
use membrain_core::types::{
    CanonicalMemoryType, MemoryId, RawEncodeInput, RawIntakeKind, SessionId, Tier1HotRecord,
};
use membrain_core::{BrainStore, RuntimeConfig};
use membrain_daemon::daemon::{DaemonRuntime, DaemonRuntimeConfig};
use membrain_daemon::preflight::{
    evaluate_preflight as evaluate_shared_preflight,
    to_preflight_explain_response as to_shared_preflight_explain_response,
    to_preflight_outcome as to_shared_preflight_outcome, EvaluatedPreflight,
    PreflightExplainResponse, PreflightOutcome,
};
use membrain_daemon::rpc::{RuntimeMetrics, RuntimePosture, RuntimeStatus};
use serde::Serialize;
use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::Instant;

/// Global memory ID counter for CLI-local session.
static NEXT_MEMORY_ID: AtomicU64 = AtomicU64::new(1);

/// Global session ID for CLI-local session.
static SESSION_ID: AtomicU64 = AtomicU64::new(1);

/// Global preflight correlation ID for CLI-local session.
static NEXT_PREFLIGHT_ID: AtomicU64 = AtomicU64::new(1);

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
    is_landmark: bool,
    landmark_label: Option<String>,
    era_id: Option<String>,
    passive_observation: Option<PassiveObservationInspectSummary>,
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

#[derive(Parser, Debug, Clone)]
struct SharedOutputFlags {
    /// Emit machine-readable output for the same semantic result shown in text mode.
    #[arg(long, global = true, action = ArgAction::SetTrue)]
    json: bool,
    /// Suppress non-essential human-oriented narration.
    #[arg(long, short = 'q', global = true, action = ArgAction::SetTrue)]
    quiet: bool,
    /// Add detail without changing the underlying result semantics.
    #[arg(long, short = 'v', global = true, action = ArgAction::Count)]
    verbose: u8,
}

#[derive(Subcommand, Debug)]
enum DoctorCommands {
    /// Run the read-only diagnosis surface.
    Run,
}

#[derive(Subcommand, Debug)]
enum PreflightCommands {
    /// Evaluate whether the requested scope is ready to proceed.
    Run {
        #[arg(long)]
        namespace: String,
        #[arg(long = "original-query")]
        original_query: String,
        #[arg(long = "proposed-action")]
        proposed_action: String,
        #[command(flatten)]
        output: SharedOutputFlags,
    },
    /// Explain blocked, preview-only, or degraded safeguard state.
    Explain {
        #[arg(long)]
        namespace: String,
        #[arg(long = "original-query")]
        original_query: String,
        #[arg(long = "proposed-action")]
        proposed_action: String,
        #[command(flatten)]
        output: SharedOutputFlags,
    },
    /// Confirm local intent for the exact preflighted scope.
    Allow {
        #[arg(long)]
        namespace: String,
        #[arg(long = "original-query")]
        original_query: String,
        #[arg(long = "proposed-action")]
        proposed_action: String,
        #[arg(long = "authorization-token")]
        authorization_token: String,
        #[arg(long = "bypass-flag", action = ArgAction::Append)]
        bypass_flags: Vec<String>,
        #[command(flatten)]
        output: SharedOutputFlags,
    },
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
        #[command(flatten)]
        output: SharedOutputFlags,
    },
    /// Recall memories matching a query
    Recall {
        /// Query string to match. May be omitted when --like or --unlike supplies the primary cue.
        query: Option<String>,
        /// Namespace to search in
        #[arg(long)]
        namespace: Option<String>,
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
        /// Query-by-example cue via like_id.
        #[arg(long = "like")]
        like: Option<u64>,
        /// Query-by-example cue via unlike_id.
        #[arg(long = "unlike")]
        unlike: Option<u64>,
        /// Widen only to policy-approved shared/public surfaces.
        #[arg(long, default_value_t = false)]
        include_public: bool,
        /// Force graph_mode=off.
        #[arg(long, default_value_t = false)]
        no_engram: bool,
        /// Time-travel using as_of_tick.
        #[arg(long = "as-of")]
        as_of: Option<u64>,
        /// Historical recall at named snapshot.
        #[arg(long)]
        at: Option<String>,
        /// Filter recall to one explicit temporal era in the effective namespace.
        #[arg(long)]
        era: Option<String>,
        /// Minimum confidence score.
        #[arg(long = "min-confidence")]
        min_confidence: Option<f32>,
        /// Minimum effective strength.
        #[arg(long = "min-strength")]
        min_strength: Option<f32>,
        /// Include memories near decay threshold.
        #[arg(long = "show-decaying", default_value_t = false)]
        show_decaying: bool,
        /// Cold-tier routing hint: avoid, auto, allow.
        #[arg(long = "cold-tier", default_value = "auto")]
        cold_tier: String,
        #[command(flatten)]
        output: SharedOutputFlags,
    },
    /// Inspect a specific memory or entity by ID
    Inspect {
        /// The memory ID to inspect
        #[arg(long)]
        id: u64,
        /// Namespace of the memory
        #[arg(long, short = 'n', default_value = "default")]
        namespace: String,
        #[command(flatten)]
        output: SharedOutputFlags,
    },
    /// Explain the ranking and routing path for a recall query
    #[command(name = "why", visible_alias = "explain")]
    Explain {
        /// Query string to explain
        query: String,
        /// Namespace to explain over
        #[arg(long, short = 'n', default_value = "default")]
        namespace: String,
        #[command(flatten)]
        output: SharedOutputFlags,
    },
    /// Run maintenance tasks (repair, reclaim, metrics)
    Maintenance {
        /// The maintenance action to run (e.g. repair, repair_index, repair_metadata)
        #[arg(long)]
        action: String,
        /// Scope of maintenance
        #[arg(long)]
        namespace: Option<String>,
        #[command(flatten)]
        output: SharedOutputFlags,
    },
    /// Run core performance and correctness benchmarks
    Benchmark {
        /// Target metric to benchmark: encode, recall, intent, tier1, retrieval
        #[arg(long, default_value = "encode")]
        target: String,
        /// Number of iterations
        #[arg(long, default_value_t = 100)]
        iters: usize,
        #[command(flatten)]
        output: SharedOutputFlags,
    },
    /// Validate system configuration and index health
    Doctor {
        #[command(subcommand)]
        command: Option<DoctorCommands>,
        #[command(flatten)]
        output: SharedOutputFlags,
    },
    /// Run shared safeguard checks for risky actions.
    Preflight {
        #[command(subcommand)]
        command: PreflightCommands,
    },
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
        #[command(flatten)]
        output: SharedOutputFlags,
    },
    /// Share a memory into an approved namespace scope
    Share {
        /// The memory ID to share
        #[arg(long)]
        id: u64,
        /// Target namespace for approved sharing
        #[arg(long = "namespace")]
        namespace_id: String,
        #[command(flatten)]
        output: SharedOutputFlags,
    },
    /// Tighten a shared memory back to private visibility
    Unshare {
        /// The memory ID to unshare
        #[arg(long)]
        id: u64,
        /// Canonical namespace that retains ownership
        #[arg(long, short = 'n', default_value = "default")]
        namespace: String,
        #[command(flatten)]
        output: SharedOutputFlags,
    },
    /// Pack a ready-to-inject context window from bounded recall results
    Budget {
        /// Hard token budget for the packed output
        #[arg(long = "tokens")]
        token_budget: usize,
        /// Optional query string used to build the bounded shortlist
        query: Option<String>,
        /// Namespace to search in
        #[arg(long)]
        namespace: Option<String>,
        /// Current context; sharpens the bounded shortlist before packing
        #[arg(long, short = 'c')]
        context: Option<String>,
        /// Output rendering: plain, markdown, json
        #[arg(long, default_value = "plain")]
        format: String,
        /// Maximum shortlist size before token packing
        #[arg(long, short = 'n', visible_short_alias = 't', default_value_t = 5)]
        top: usize,
        /// Widen only to policy-approved shared/public surfaces.
        #[arg(long, default_value_t = false)]
        include_public: bool,
        #[command(flatten)]
        output: SharedOutputFlags,
    },
    /// Segment piped or watched content into passive-observation memories.
    Observe {
        /// Read from stdin and segment the supplied content into bounded fragments.
        #[arg(value_name = "CONTENT")]
        content: Option<String>,
        /// Namespace for the observed fragments.
        #[arg(long, short = 'n', default_value = "default")]
        namespace: String,
        /// Optional context attached to each observed fragment.
        #[arg(long, short = 'c')]
        context: Option<String>,
        /// Deterministic chunk-size hint in characters.
        #[arg(long = "chunk-size", default_value_t = 500)]
        chunk_size: usize,
        /// Topic-shift threshold in the range 0.0..=1.0.
        #[arg(long = "topic-threshold", default_value_t = 0.35)]
        topic_threshold: f32,
        /// Minimum chunk size before a boundary can flush.
        #[arg(long = "min-chunk-size", default_value_t = 50)]
        min_chunk_size: usize,
        /// Provenance-only source label preserved on each fragment.
        #[arg(long = "source-label")]
        source_label: Option<String>,
        /// Preview the fragments without writing memories.
        #[arg(long = "dry-run", default_value_t = false)]
        dry_run: bool,
        /// Watch one file or directory path instead of stdin.
        #[arg(long)]
        watch: Option<PathBuf>,
        /// Optional watch-mode glob/pattern hint preserved in output.
        #[arg(long)]
        pattern: Option<String>,
        #[command(flatten)]
        output: SharedOutputFlags,
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
    passive_observation: Option<PassiveObservationInspectSummary>,
}

#[derive(Debug, Clone, Serialize)]
struct ObservePreviewFragment {
    index: usize,
    write_decision: &'static str,
    captured_as_observation: bool,
    compact_text: String,
    fingerprint: u64,
    route_family: &'static str,
    observation_source: String,
    observation_chunk_id: String,
}

#[derive(Debug, Clone, Serialize)]
struct ObserveOutput {
    outcome: &'static str,
    namespace: String,
    watch_mode: bool,
    watched_path: Option<String>,
    pattern: Option<String>,
    dry_run: bool,
    observation_source: String,
    observation_chunk_id: String,
    bytes_processed: usize,
    topic_shifts: usize,
    fragments_previewed: usize,
    memories_created: usize,
    suppressed: usize,
    denied: usize,
    context: Option<String>,
    preview: Vec<ObservePreviewFragment>,
}

// ── Shared helper types ──────────────────────────────────────────────────────

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
struct RepairResultOutput {
    target: &'static str,
    status: &'static str,
    verification_passed: bool,
    rebuild_entrypoint: Option<&'static str>,
    rebuilt_outputs: Vec<&'static str>,
    durable_sources: Vec<&'static str>,
    verification_artifact_name: &'static str,
    parity_check: &'static str,
    authoritative_rows: u64,
    derived_rows: u64,
    authoritative_generation: &'static str,
    derived_generation: &'static str,
    affected_item_count: u32,
    error_count: u32,
    rebuild_duration_ms: u64,
    rollback_state: Option<&'static str>,
    queue_depth_before: u32,
    queue_depth_after: u32,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
struct MaintenanceOutput {
    outcome: &'static str,
    action: String,
    namespace: String,
    targets_checked: u32,
    rebuilt: u32,
    affected_item_count: u32,
    error_count: u32,
    rebuild_duration_ms: u64,
    rollback_state: Option<&'static str>,
    queue_depth_before: u32,
    queue_depth_after: u32,
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
struct ShareAuditView {
    event_kind: &'static str,
    actor_source: &'static str,
    request_id: String,
    effective_namespace: String,
    source_namespace: Option<String>,
    target_namespace: Option<String>,
    policy_family: &'static str,
    outcome_class: &'static str,
    blocked_stage: &'static str,
    redaction_summary: Vec<String>,
    related_run: Option<String>,
    redacted: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
struct ShareOutput {
    outcome: &'static str,
    memory_id: u64,
    namespace: String,
    visibility: &'static str,
    policy_summary: TracePolicySummary,
    policy_filters_applied: Vec<membrain_core::api::PolicyFilterSummary>,
    audit: ShareAuditView,
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
    durable_sources: Vec<&'static str>,
    verification_artifact_name: &'static str,
    parity_check: &'static str,
    authoritative_rows: u64,
    derived_rows: u64,
    authoritative_generation: &'static str,
    derived_generation: &'static str,
    affected_item_count: u32,
    error_count: u32,
    rebuild_duration_ms: u64,
    rollback_state: Option<&'static str>,
    queue_depth_before: u32,
    queue_depth_after: u32,
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

fn describe_retrieval_validation_error(error: RetrievalRequestValidationError) -> &'static str {
    match error {
        RetrievalRequestValidationError::MissingPrimaryCue => {
            "missing query text or query-by-example cue"
        }
        RetrievalRequestValidationError::DuplicateExampleCue(_) => {
            "like and unlike cues must reference different memories"
        }
        RetrievalRequestValidationError::ExactIdWithExampleCue => {
            "exact-id retrieval cannot be combined with query-by-example cues"
        }
    }
}

#[derive(Debug, Clone)]
struct RecallCommandConfig {
    query: Option<String>,
    context: Option<String>,
    top: usize,
    kind: Option<String>,
    confidence: String,
    explain: String,
    namespace: NamespaceId,
    include_public: bool,
    like: Option<MemoryId>,
    unlike: Option<MemoryId>,
    graph_expansion: bool,
    as_of: Option<u64>,
    at: Option<String>,
    era: Option<String>,
    min_confidence: Option<f32>,
    min_strength: Option<f32>,
    show_decaying: bool,
    cold_tier: String,
}

fn validate_recall_command(
    namespace: Option<&str>,
    query: Option<&str>,
    top: usize,
    kind: Option<&str>,
    confidence: &str,
    explain: &str,
    like: Option<u64>,
    unlike: Option<u64>,
    include_public: bool,
    no_engram: bool,
    as_of: Option<u64>,
    at: Option<&str>,
    era: Option<&str>,
    min_confidence: Option<f32>,
    min_strength: Option<f32>,
    show_decaying: bool,
    cold_tier: &str,
    context: Option<&str>,
) -> anyhow::Result<RecallCommandConfig> {
    let namespace = namespace.ok_or_else(|| anyhow::anyhow!("missing namespace"))?;
    let namespace = NamespaceId::new(namespace)?;

    if top == 0 {
        anyhow::bail!("result budget must be greater than zero");
    }

    let confidence = confidence.trim().to_lowercase();
    if !matches!(confidence.as_str(), "fast" | "normal" | "high") {
        anyhow::bail!(
            "invalid confidence `{}`; expected fast, normal, or high",
            confidence
        );
    }

    let explain = explain.trim().to_lowercase();
    if !matches!(explain.as_str(), "none" | "summary" | "full") {
        anyhow::bail!(
            "invalid explain verbosity `{}`; expected none, summary, or full",
            explain
        );
    }

    let cold_tier = cold_tier.trim().to_lowercase();
    if !matches!(cold_tier.as_str(), "avoid" | "auto" | "allow") {
        anyhow::bail!(
            "invalid cold-tier `{}`; expected avoid, auto, or allow",
            cold_tier
        );
    }

    if as_of.is_some() && at.is_some() {
        anyhow::bail!("--as-of and --at cannot be combined");
    }

    if let Some(value) = min_confidence {
        if !(0.0..=1.0).contains(&value) {
            anyhow::bail!("min-confidence must be between 0.0 and 1.0");
        }
    }

    if let Some(value) = min_strength {
        if !(0.0..=1.0).contains(&value) {
            anyhow::bail!("min-strength must be between 0.0 and 1.0");
        }
    }

    let kind = kind.map(|value| value.trim().to_lowercase());
    if let Some(kind_value) = kind.as_deref() {
        if !matches!(kind_value, "episodic" | "semantic" | "procedural") {
            anyhow::bail!(
                "invalid kind `{}`; expected episodic, semantic, or procedural",
                kind_value
            );
        }
    }

    let era = era
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(str::to_owned);
    let at = at
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(str::to_owned);
    let context = context
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(str::to_owned);
    let query = query.map(str::to_owned);

    let request =
        RetrievalRequest::hybrid(namespace.clone(), query.as_deref().unwrap_or_default(), top)
            .with_budget(top)
            .with_graph_expansion(false)
            .with_tier3_fallback(cold_tier != "avoid");
    let request = if let Some(memory_id) = like.map(MemoryId) {
        request.with_like_memory(memory_id)
    } else {
        request
    };
    let request = if let Some(memory_id) = unlike.map(MemoryId) {
        request.with_unlike_memory(memory_id)
    } else {
        request
    };
    request
        .normalize_query_by_example()
        .map_err(|error| anyhow::anyhow!(describe_retrieval_validation_error(error)))?;

    Ok(RecallCommandConfig {
        query,
        context,
        top,
        kind,
        confidence,
        explain,
        namespace,
        include_public,
        like: like.map(MemoryId),
        unlike: unlike.map(MemoryId),
        graph_expansion: !no_engram,
        as_of,
        at,
        era,
        min_confidence,
        min_strength,
        show_decaying,
        cold_tier,
    })
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
    let policy_filters = policy_summary.filters.clone();
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
    if !policy_filters.is_empty() {
        response = response.with_policy_filters(policy_filters);
    }
    if partial_success {
        response = response.with_partial_success();
    }
    response
}

fn parse_injection_format(raw: &str) -> anyhow::Result<InjectionFormat> {
    match raw.trim().to_ascii_lowercase().as_str() {
        "plain" => Ok(InjectionFormat::Plain),
        "markdown" => Ok(InjectionFormat::Markdown),
        "json" => Ok(InjectionFormat::Json),
        other => anyhow::bail!("invalid format `{other}`; expected plain, markdown, or json"),
    }
}

fn build_confidence_inputs(record: &LocalMemoryRecord) -> ConfidenceInputs {
    ConfidenceInputs {
        corroboration_count: u32::from((record.provisional_salience / 250).min(4)),
        reconsolidation_count: u32::from((record.payload_size_bytes / 512).min(16) as u16),
        ticks_since_last_access: u64::from(record.memory_id.0.saturating_mul(17)),
        age_ticks: u64::from(record.memory_id.0.saturating_mul(23)),
        resolution_state: membrain_core::engine::contradiction::ResolutionState::None,
        conflict_score: 0,
        causal_parent_count: u32::from((record.payload_size_bytes / 256).clamp(1, 4) as u16),
        authoritativeness: record.provisional_salience.saturating_add(100).min(1000),
        recall_count: u32::from((record.memory_id.0 % 6) as u16),
    }
}

fn build_retrieval_result_set(
    local_records: &[LocalMemoryRecord],
    config: &RecallCommandConfig,
    ranking_profile: RankingProfile,
    route_family: &'static str,
    route_reason: String,
    matched_ids: Vec<MemoryId>,
) -> RetrievalResultSet {
    let mut builder = ResultBuilder::new(config.top, config.namespace.clone());
    let confidence_display = if config.min_confidence.is_some() {
        ConfidenceDisplayConfig::strict()
    } else {
        ConfidenceDisplayConfig::default()
    };
    let confidence_policy = ConfidencePolicy::default();
    let matched_empty = matched_ids.is_empty();
    let selected_ids = if matched_empty {
        local_records
            .iter()
            .filter(|r| r.namespace == config.namespace)
            .filter(|record| {
                config
                    .era
                    .as_deref()
                    .is_none_or(|era| record.era_id.as_deref() == Some(era))
            })
            .rev()
            .take(config.top)
            .map(|r| r.memory_id)
            .collect::<Vec<_>>()
    } else {
        matched_ids
            .into_iter()
            .filter(|memory_id| {
                local_records
                    .iter()
                    .find(|r| r.namespace == config.namespace && r.memory_id == *memory_id)
                    .and_then(|record| {
                        config
                            .era
                            .as_deref()
                            .map(|era| record.era_id.as_deref() == Some(era))
                    })
                    .unwrap_or(true)
            })
            .collect::<Vec<_>>()
    };

    for memory_id in &selected_ids {
        if let Some(record) = local_records
            .iter()
            .find(|r| r.namespace == config.namespace && r.memory_id == *memory_id)
        {
            let confidence_inputs = build_confidence_inputs(record);
            let confidence_output = membrain_core::engine::confidence::ConfidenceEngine
                .compute(&confidence_inputs, &confidence_policy);
            let ranking = fuse_scores(
                RankingInput {
                    recency: 900,
                    salience: record.provisional_salience,
                    strength: 750,
                    provenance: 850,
                    conflict: 500,
                    confidence: confidence_output.confidence,
                },
                ranking_profile,
            );
            builder.add_with_confidence(
                record.memory_id,
                record.namespace.clone(),
                record.session_id,
                record.memory_type,
                record.compact_text.clone(),
                &ranking,
                AnsweredFrom::Tier1Hot,
                &confidence_inputs,
                &confidence_policy,
            );
        }
    }

    let mut explain = RetrievalExplain {
        recall_plan: if config.graph_expansion {
            membrain_core::engine::recall::RecallPlanKind::Tier2ExactThenGraphExpansion
        } else {
            membrain_core::engine::recall::RecallPlanKind::RecentTier1ThenTier2Exact
        },
        route_reason,
        tiers_consulted: if config.graph_expansion {
            vec!["tier2_exact".to_string(), "graph_expansion".to_string()]
        } else {
            vec!["tier1_recent".to_string()]
        },
        trace_stages: if config.graph_expansion {
            vec![
                membrain_core::engine::recall::RecallTraceStage::Tier2Exact,
                membrain_core::engine::recall::RecallTraceStage::GraphExpansion,
            ]
        } else {
            vec![membrain_core::engine::recall::RecallTraceStage::Tier1RecentWindow]
        },
        tier1_answered_directly: !config.graph_expansion,
        candidate_budget: config.top,
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
    if config.show_decaying {
        explain
            .result_reasons
            .push(membrain_core::engine::result::ResultReason {
                memory_id: None,
                reason_code: "show_decaying_enabled".to_string(),
                detail: "show-decaying requested inclusion of near-decay candidates when available"
                    .to_string(),
            });
    }
    if let Some(era) = config.era.as_deref() {
        let landmark_descriptors = local_records
            .iter()
            .filter(|record| {
                record.namespace == config.namespace
                    && record.is_landmark
                    && record.era_id.as_deref() == Some(era)
            })
            .map(|record| {
                record
                    .landmark_label
                    .as_deref()
                    .map(|label| format!("{label} (#{})", record.memory_id.0))
                    .unwrap_or_else(|| format!("memory #{}", record.memory_id.0))
            })
            .collect::<Vec<_>>();
        let matched_candidate_count = selected_ids.len();
        explain
            .result_reasons
            .push(membrain_core::engine::result::ResultReason {
                memory_id: None,
                reason_code: "era_filter_applied".to_string(),
                detail: if landmark_descriptors.is_empty() {
                    format!(
                        "bounded retrieval stayed inside era `{era}`; no returned landmark anchor opened that era and {matched_candidate_count} candidate(s) remained after era scoping"
                    )
                } else {
                    format!(
                        "bounded retrieval stayed inside era `{era}` opened by landmark(s) {}; {matched_candidate_count} candidate(s) remained after era scoping",
                        landmark_descriptors.join(", ")
                    )
                },
            });
    }
    if let Some(snapshot) = config.at.as_deref() {
        explain
            .result_reasons
            .push(membrain_core::engine::result::ResultReason {
                memory_id: None,
                reason_code: "snapshot_scope_applied".to_string(),
                detail: format!("historical recall anchored to snapshot `{snapshot}`"),
            });
    }
    if let Some(as_of) = config.as_of {
        explain
            .result_reasons
            .push(membrain_core::engine::result::ResultReason {
                memory_id: None,
                reason_code: "as_of_scope_applied".to_string(),
                detail: format!("historical recall bounded at as_of_tick={as_of}"),
            });
    }
    if let Some(min_confidence) = config.min_confidence {
        explain
            .result_reasons
            .push(membrain_core::engine::result::ResultReason {
                memory_id: None,
                reason_code: "confidence_filter_applied".to_string(),
                detail: format!("results were filtered with min_confidence={min_confidence:.3}"),
            });
        explain
            .result_reasons
            .push(membrain_core::engine::result::ResultReason {
                memory_id: None,
                reason_code: "confidence_display_rule".to_string(),
                detail: "confidence changes retrieval ordering through the ranking confidence signal and hides results only when min_confidence is set".to_string(),
            });
    } else {
        explain
            .result_reasons
            .push(membrain_core::engine::result::ResultReason {
                memory_id: None,
                reason_code: "confidence_display_rule".to_string(),
                detail: "confidence changes retrieval ordering through the ranking confidence signal; without min_confidence, low-confidence results remain visible with uncertainty markers".to_string(),
            });
    }
    if let Some(min_strength) = config.min_strength {
        explain
            .result_reasons
            .push(membrain_core::engine::result::ResultReason {
                memory_id: None,
                reason_code: "strength_filter_applied".to_string(),
                detail: format!("results were filtered with min_strength={min_strength:.3}"),
            });
    }
    if let Some(top_item) = builder.action_pack.as_ref().and_then(|_| None::<&membrain_core::engine::result::ActionArtifact>) {
        let _ = top_item;
    }
    if config.include_public {
        explain
            .result_reasons
            .push(membrain_core::engine::result::ResultReason {
                memory_id: None,
                reason_code: "include_public_enabled".to_string(),
                detail: "policy-approved shared/public widening remained explicit on the request"
                    .to_string(),
            });
    }
    if let Some(kind) = config.kind.as_deref() {
        explain
            .result_reasons
            .push(membrain_core::engine::result::ResultReason {
                memory_id: None,
                reason_code: "memory_kind_filter_applied".to_string(),
                detail: format!("retrieval filtered to memory kind `{kind}`"),
            });
    }
    if let Some(context) = config.context.as_deref() {
        explain
            .result_reasons
            .push(membrain_core::engine::result::ResultReason {
                memory_id: None,
                reason_code: "context_text_supplied".to_string(),
                detail: format!("context_text sharpened ranking with `{context}`"),
            });
    }
    if config.cold_tier == "avoid" {
        explain
            .result_reasons
            .push(membrain_core::engine::result::ResultReason {
                memory_id: None,
                reason_code: "cold_tier_avoided".to_string(),
                detail: "cold-tier fallback was disabled for this bounded request".to_string(),
            });
    } else if config.cold_tier == "allow" {
        explain
            .result_reasons
            .push(membrain_core::engine::result::ResultReason {
                memory_id: None,
                reason_code: "cold_tier_allowed".to_string(),
                detail:
                    "cold-tier fallback remained available without permitting pre-cut payload fetch"
                        .to_string(),
            });
    }
    if config.like.is_some() || config.unlike.is_some() {
        let request = RetrievalRequest::hybrid(
            config.namespace.clone(),
            config.query.as_deref().unwrap_or_default(),
            config.top,
        )
        .with_budget(config.top)
        .with_graph_expansion(false)
        .with_tier3_fallback(config.cold_tier != "avoid");
        let request = if let Some(memory_id) = config.like {
            request.with_like_memory(memory_id)
        } else {
            request
        };
        let request = if let Some(memory_id) = config.unlike {
            request.with_unlike_memory(memory_id)
        } else {
            request
        };
        if let Ok(normalization) = request.normalize_query_by_example() {
            let mut trace = RetrievalPlanTrace::new(&request);
            trace.set_query_by_example_materialization(&normalization, &selected_ids);
            trace.set_final_candidates(selected_ids.len());
            explain.set_query_by_example_trace(&trace);
        }
    }
    let mut result_set = builder.build(explain);
    result_set.freshness_markers.as_of_tick = config.as_of;
    result_set.policy_summary.filters = vec![membrain_core::api::PolicyFilterSummary::new(
        config.namespace.as_str(),
        if config.include_public {
            "shared_public_widening"
        } else {
            "namespace_only"
        },
        result_set.outcome_class,
        "policy_gate",
        if config.include_public {
            membrain_core::api::FieldPresence::Present("approved_shared".to_string())
        } else {
            membrain_core::api::FieldPresence::Present("same_namespace".to_string())
        },
        membrain_core::api::FieldPresence::Absent,
        Vec::new(),
    )];
    result_set.packaging_metadata.result_budget = config.top;
    result_set.packaging_metadata.degraded_summary =
        (config.cold_tier == "avoid").then(|| "cold_tier_avoid".to_string());
    result_set
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
    let input = local_records
        .iter()
        .rev()
        .find(|record| record.namespace == *namespace)
        .and_then(|record| record.era_id.clone())
        .map(|era_id| RawEncodeInput::new(intake_kind, content).with_active_era_id(era_id))
        .unwrap_or_else(|| RawEncodeInput::new(intake_kind, content));
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
        is_landmark: prepared.normalized.landmark.is_landmark,
        landmark_label: prepared.normalized.landmark.landmark_label.clone(),
        era_id: prepared.normalized.landmark.era_id.clone(),
        passive_observation: None,
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

fn budget_memories(
    store: &BrainStore,
    local_records: &[LocalMemoryRecord],
    namespace: &NamespaceId,
    query: Option<&str>,
    current_context: Option<&str>,
    top: usize,
    token_budget: usize,
    format: InjectionFormat,
    include_public: bool,
) -> ResponseContext<ContextBudgetResponse> {
    let config = RecallCommandConfig {
        query: query.map(str::to_owned),
        context: current_context.map(str::to_owned),
        top,
        kind: None,
        confidence: "normal".to_string(),
        explain: "summary".to_string(),
        namespace: namespace.clone(),
        include_public,
        like: None,
        unlike: None,
        graph_expansion: false,
        as_of: None,
        at: None,
        era: None,
        min_confidence: None,
        min_strength: None,
        show_decaying: false,
        cold_tier: "auto".to_string(),
    };
    let query_text = config.query.as_deref().unwrap_or_default();
    let intent_result = store.intent_engine().classify(query_text);
    let session_id = SessionId(SESSION_ID.load(Ordering::SeqCst));
    let recall_request =
        membrain_core::engine::recall::RecallRequest::small_session_lookup(session_id);
    let recall_plan = store
        .recall_engine()
        .plan_recall(recall_request, store.config());

    let query_lower = query_text.to_lowercase();
    let matched_ids = local_records
        .iter()
        .filter(|record| record.namespace == *namespace)
        .filter(|record| {
            if query_lower.is_empty() {
                true
            } else {
                let text_lower = record.compact_text.to_lowercase();
                text_lower.contains(&query_lower)
                    || query_lower.contains(&text_lower)
                    || record.memory_type.as_str().contains(&query_lower)
            }
        })
        .map(|record| record.memory_id)
        .collect::<Vec<_>>();

    let result_set = build_retrieval_result_set(
        local_records,
        &config,
        RankingProfile::balanced(),
        intent_result.route_inputs.ranking_profile.as_str(),
        recall_plan.route_summary.reason.to_string(),
        matched_ids,
    );

    let request = ContextBudgetRequest::new(token_budget)
        .with_working_memory(vec![])
        .with_format(format);
    let request = if let Some(context) = current_context {
        request.with_context(context)
    } else {
        request
    };
    let partial_success = result_set.truncated;
    let budget = store.context_budget(&request, &result_set);
    let budget_partial = budget.partial_success;
    let mut response = ResponseContext::success(
        namespace.clone(),
        RequestId::new(format!("budget-{}-{token_budget}", namespace.as_str()))
            .expect("budget request id"),
        budget,
    );
    if partial_success || budget_partial {
        response = response.with_partial_success();
    }
    if budget_partial {
        response.warnings.push(ResponseWarning::new(
            "budget_exhausted",
            "token budget truncated otherwise eligible injections",
        ));
    }
    response
}

fn recall_memories(
    store: &BrainStore,
    _hot: &Tier1HotMetadataStore,
    local_records: &[LocalMemoryRecord],
    config: &RecallCommandConfig,
) -> ResponseContext<RetrievalResultSet> {
    let query_text = config.query.as_deref().unwrap_or_default();
    let intent_result = store.intent_engine().classify(query_text);
    let session_id = SessionId(SESSION_ID.load(Ordering::SeqCst));

    let recall_request = if config.graph_expansion {
        membrain_core::engine::recall::RecallRequest::small_session_lookup(session_id)
            .with_graph_expansion(true)
    } else {
        membrain_core::engine::recall::RecallRequest::small_session_lookup(session_id)
    };
    let recall_plan = store
        .recall_engine()
        .plan_recall(recall_request, store.config());

    let query_lower = query_text.to_lowercase();
    let kind_filter = config.kind.as_deref().map(|kind| match kind {
        "semantic" => CanonicalMemoryType::Observation,
        "procedural" => CanonicalMemoryType::ToolOutcome,
        _ => CanonicalMemoryType::Event,
    });
    let matched_ids = local_records
        .iter()
        .filter(|r| r.namespace == config.namespace)
        .filter(|record| kind_filter.is_none_or(|kind| record.memory_type == kind))
        .filter(|record| {
            if query_lower.is_empty() {
                return config.like == Some(record.memory_id)
                    || config.unlike == Some(record.memory_id);
            }
            let text_lower = record.compact_text.to_lowercase();
            text_lower.contains(&query_lower)
                || query_lower.contains(&text_lower)
                || record.memory_type.as_str().contains(&query_lower)
        })
        .map(|record| record.memory_id)
        .collect::<Vec<_>>();

    let result_set = build_retrieval_result_set(
        local_records,
        config,
        RankingProfile::balanced(),
        intent_result.route_inputs.ranking_profile.as_str(),
        recall_plan.route_summary.reason.to_string(),
        matched_ids,
    );

    let request_id = RequestId::new(format!(
        "recall-{}-{}",
        config.namespace.as_str(),
        query_text.replace(' ', "-")
    ))
    .expect("recall request id");
    response_from_result_set(&config.namespace, request_id, result_set)
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
        let passive_observation = local_records
            .iter()
            .find(|local| local.memory_id == record.memory_id && local.namespace == record.namespace)
            .and_then(|local| local.passive_observation.clone());

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
            passive_observation,
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
            passive_observation: record.passive_observation.clone(),
        });
    }

    Err(format!(
        "memory {} not found in namespace '{}'",
        memory_id.0,
        namespace.as_str()
    ))
}

// ── Explain ──────────────────────────────────────────────────────────────────

fn observe_memories(
    store: &BrainStore,
    hot: &mut Tier1HotMetadataStore,
    local_records: &mut Vec<LocalMemoryRecord>,
    namespace: &NamespaceId,
    content: &str,
    config: &ObserveConfig,
    dry_run: bool,
    watched_path: Option<&std::path::Path>,
    pattern: Option<&str>,
) -> ResponseContext<ObserveOutput> {
    let mut seen_fingerprints = local_records
        .iter()
        .filter(|record| record.namespace == *namespace)
        .map(|record| record.fingerprint)
        .collect::<std::collections::HashSet<_>>();
    let report = ObserveEngine::observe_content(store.encode_engine(), content, config, true, |fingerprint| {
        let already_seen = seen_fingerprints.contains(&fingerprint);
        if !already_seen {
            seen_fingerprints.insert(fingerprint);
        }
        already_seen
    });

    if !dry_run {
        let session_id = SessionId(SESSION_ID.load(Ordering::SeqCst));
        for fragment in &report.fragments {
            if fragment.prepared.write_decision.as_str() != "capture" {
                continue;
            }
            let memory_id = MemoryId(NEXT_MEMORY_ID.fetch_add(1, Ordering::SeqCst));
            let prepared = &fragment.prepared;
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
                is_landmark: prepared.normalized.landmark.is_landmark,
                landmark_label: prepared.normalized.landmark.landmark_label.clone(),
                era_id: prepared.normalized.landmark.era_id.clone(),
                passive_observation: Some(PassiveObservationInspectSummary::from_encode(
                    &prepared.passive_observation_inspect,
                )),
            };
            hot.seed(local.as_hot_record());
            seen_fingerprints.insert(local.fingerprint);
            local_records.push(local);
        }
    }

    let output = ObserveOutput {
        outcome: if dry_run { "preview" } else { "accepted" },
        namespace: namespace.as_str().to_string(),
        watch_mode: watched_path.is_some(),
        watched_path: watched_path.map(|path| path.display().to_string()),
        pattern: pattern.map(str::to_string),
        dry_run,
        observation_source: report.observation_source.clone(),
        observation_chunk_id: report.observation_chunk_id.clone(),
        bytes_processed: report.bytes_processed,
        topic_shifts: report.topic_shifts_detected,
        fragments_previewed: report.fragments.len(),
        memories_created: if dry_run { 0 } else { report.memories_created },
        suppressed: report.suppressed,
        denied: report.denied,
        context: config.context.clone(),
        preview: report
            .fragments
            .iter()
            .map(|fragment| ObservePreviewFragment {
                index: fragment.index,
                write_decision: fragment.prepared.write_decision.as_str(),
                captured_as_observation: fragment.prepared.captured_as_observation,
                compact_text: fragment.prepared.normalized.compact_text.clone(),
                fingerprint: fragment.prepared.fingerprint,
                route_family: fragment.prepared.classification.route_family.as_str(),
                observation_source: report.observation_source.clone(),
                observation_chunk_id: report.observation_chunk_id.clone(),
            })
            .collect(),
    };
    let request_id = RequestId::new(format!(
        "observe-{}-{}",
        namespace.as_str(),
        report.observation_chunk_id
    ))
    .expect("observe request id");
    let mut response = ResponseContext::success(namespace.clone(), request_id, output);
    if dry_run || report.suppressed > 0 || report.denied > 0 {
        response = response.with_partial_success();
    }
    if let Some(fragment) = report.fragments.first() {
        response = response.with_passive_observation(PassiveObservationInspectSummary {
            source_kind: fragment.prepared.passive_observation_inspect.source_kind,
            write_decision: fragment.prepared.passive_observation_inspect.write_decision,
            captured_as_observation: fragment
                .prepared
                .passive_observation_inspect
                .captured_as_observation,
            observation_source: FieldPresence::Present(report.observation_source.clone()),
            observation_chunk_id: FieldPresence::Present(report.observation_chunk_id.clone()),
            retention_marker: FieldPresence::Present(
                fragment.prepared.passive_observation_inspect.retention_marker,
            ),
        });
    }
    response
}

fn explain_query(
    store: &BrainStore,
    local_records: &[LocalMemoryRecord],
    query: &str,
    namespace: &NamespaceId,
) -> ResponseContext<RetrievalResultSet> {
    let intent_result = store.intent_engine().classify(query);
    let config = RecallCommandConfig {
        query: Some(query.to_string()),
        context: None,
        top: 5,
        kind: None,
        confidence: "normal".to_string(),
        explain: "full".to_string(),
        namespace: namespace.clone(),
        include_public: false,
        like: None,
        unlike: None,
        graph_expansion: false,
        as_of: None,
        at: None,
        era: None,
        min_confidence: None,
        min_strength: None,
        show_decaying: false,
        cold_tier: "auto".to_string(),
    };
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
        &config,
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
        "maintenance_forgetting_evaluated" => {
            Some(AuditEventKind::MaintenanceForgettingEvaluated)
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

fn sharing_audit_view(
    namespace: &NamespaceId,
    target_namespace: Option<&NamespaceId>,
    policy_summary: &TracePolicySummary,
    request_id: String,
    actor_source: &'static str,
    event_kind: AuditEventKind,
    redacted: bool,
    related_run: Option<String>,
) -> ShareAuditView {
    ShareAuditView {
        event_kind: event_kind.as_str(),
        actor_source,
        request_id,
        effective_namespace: namespace.as_str().to_string(),
        source_namespace: Some(namespace.as_str().to_string()),
        target_namespace: target_namespace.map(|ns| ns.as_str().to_string()),
        policy_family: policy_summary.policy_family,
        outcome_class: policy_summary.outcome_class.as_str(),
        blocked_stage: policy_summary.blocked_stage,
        redaction_summary: policy_summary
            .redaction_fields
            .iter()
            .map(|field| (*field).to_string())
            .collect(),
        related_run,
        redacted,
    }
}

fn share_output(memory_id: u64, namespace: &NamespaceId, visibility: &'static str) -> ShareOutput {
    let policy_summary = sharing_trace_policy_summary(
        namespace,
        visibility,
        membrain_core::observability::OutcomeClass::Accepted,
        Vec::new(),
    );
    let request_id = format!("req-share-{memory_id}");
    let related_run = Some(format!("share-run-{memory_id}"));
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
        .with_request_id(request_id.clone())
        .with_related_run(related_run.clone().expect("share related run")),
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
        audit: sharing_audit_view(
            namespace,
            Some(namespace),
            &policy_summary,
            request_id,
            "cli_share",
            AuditEventKind::ApprovedSharing,
            false,
            related_run,
        ),
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
    let request_id = format!("req-unshare-{memory_id}");
    let related_run = Some(format!("share-run-{memory_id}"));
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
        .with_request_id(request_id.clone())
        .with_related_run(related_run.clone().expect("unshare related run"))
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
        audit: sharing_audit_view(
            namespace,
            Some(namespace),
            &policy_summary,
            request_id,
            "cli_unshare",
            AuditEventKind::PolicyRedacted,
            true,
            related_run,
        ),
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
        repair_engine.create_index_rebuild(namespace.clone(), IndexRepairEntrypoint::VerifyOnly),
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
                            .expect("repair operator report should exist for each doctor result");
                        let plan = result
                            .rebuild_entrypoint
                            .and_then(|entrypoint| {
                                store.repair_engine().plan_index_rebuild(result.target, entrypoint)
                            });
                        let artifact = summary
                            .verification_artifacts
                            .get(&result.target)
                            .expect("verification artifact should exist for each doctor result");
                        RepairReportRow {
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

fn next_preflight_correlation_id() -> u64 {
    NEXT_PREFLIGHT_ID.fetch_add(1, Ordering::SeqCst)
}

fn evaluate_cli_preflight(
    namespace: &str,
    original_query: &str,
    proposed_action: &str,
    local_confirmation: bool,
    preview_only: bool,
) -> anyhow::Result<EvaluatedPreflight> {
    let correlation_id = next_preflight_correlation_id();
    evaluate_shared_preflight(
        namespace,
        original_query,
        proposed_action,
        correlation_id,
        local_confirmation,
        preview_only,
        "cli_preflight",
        "cli",
    )
    .map_err(anyhow::Error::msg)
}

fn cli_preflight_explain(
    namespace: &str,
    original_query: &str,
    proposed_action: &str,
) -> anyhow::Result<PreflightExplainResponse> {
    let evaluated =
        evaluate_cli_preflight(namespace, original_query, proposed_action, false, true)?;
    Ok(to_shared_preflight_explain_response(namespace, evaluated))
}

fn cli_preflight_run(
    namespace: &str,
    original_query: &str,
    proposed_action: &str,
) -> anyhow::Result<PreflightOutcome> {
    let evaluated =
        evaluate_cli_preflight(namespace, original_query, proposed_action, false, false)?;
    Ok(to_shared_preflight_outcome(evaluated, false))
}

fn cli_preflight_allow(
    namespace: &str,
    original_query: &str,
    proposed_action: &str,
    authorization_token: &str,
    bypass_flags: &[String],
) -> anyhow::Result<PreflightOutcome> {
    let confirmed = authorization_token.starts_with("allow-")
        && bypass_flags.iter().any(|flag| flag == "manual_override");
    let evaluated =
        evaluate_cli_preflight(namespace, original_query, proposed_action, confirmed, false)?;
    Ok(to_shared_preflight_outcome(evaluated, confirmed))
}

fn preflight_outcome_class_label(outcome_class: &str, preflight_state: &str) -> &'static str {
    match outcome_class {
        "accepted" => "accepted",
        "degraded" => "degraded",
        "rejected" => "rejected",
        _ if preflight_state == "blocked"
            || preflight_state == "missing_data"
            || preflight_state == "stale_knowledge" =>
        {
            "blocked"
        }
        _ => "preview",
    }
}

fn preflight_exit_code(response: &PreflightOutcome) -> i32 {
    match response.outcome_class.as_str() {
        "rejected" => 2,
        "accepted" | "degraded" => 0,
        _ => 4,
    }
}

fn explain_blocked_by_policy(response: &PreflightExplainResponse) -> bool {
    response
        .blocked_reasons
        .iter()
        .any(|reason| reason == "policy_denied" || reason == "legal_hold")
}

fn explain_exit_code(response: &PreflightExplainResponse) -> i32 {
    if explain_blocked_by_policy(response) {
        2
    } else if response.blocked_reasons.is_empty() {
        0
    } else {
        4
    }
}

fn print_preflight_run_human(response: &PreflightOutcome) {
    println!(
        "Preflight run [{}] state={} outcome={}",
        preflight_outcome_class_label(&response.outcome_class, &response.preflight_state),
        response.preflight_state,
        response.preflight_outcome,
    );
    if !response.blocked_reasons.is_empty() {
        println!("  blocked_reasons: {}", response.blocked_reasons.join(", "));
    }
    println!(
        "  confirmation: required={} confirmed={} force_allowed={}",
        response.confirmation.required,
        response.confirmation.confirmed,
        response.confirmation.force_allowed,
    );
}

fn print_preflight_explain_human(response: &PreflightExplainResponse) {
    println!(
        "Preflight explain [{}] state={} outcome={}",
        preflight_outcome_class_label("preview", &response.preflight_state),
        response.preflight_state,
        response.preflight_outcome,
    );
    if !response.blocked_reasons.is_empty() {
        println!("  blocked_reasons: {}", response.blocked_reasons.join(", "));
    }
    if let Some(blocked_reason) = response.blocked_reason.as_deref() {
        println!("  blocked_reason: {blocked_reason}");
    }
}

fn print_preflight_allow_human(response: &PreflightOutcome) {
    println!(
        "Preflight allow [{}] state={} outcome={}",
        preflight_outcome_class_label(&response.outcome_class, &response.preflight_state),
        response.preflight_state,
        response.preflight_outcome,
    );
    if !response.blocked_reasons.is_empty() {
        println!("  blocked_reasons: {}", response.blocked_reasons.join(", "));
    }
    if let Some(reason) = response.confirmation_reason.as_deref() {
        println!("  confirmation_reason: {reason}");
    }
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
            output,
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
            if output.json {
                println!("{}", serde_json::to_string_pretty(&response)?);
            } else {
                let output_result = response.result.as_ref().expect("encode result present");
                if !output.quiet {
                    println!(
                        "Encoded memory #{} in '{}' [{} / {}]",
                        output_result.memory_id,
                        output_result.namespace,
                        output_result.memory_type,
                        output_result.route_family,
                    );
                }
                println!("  text: {}", output_result.compact_text);
                if output.verbose > 0 {
                    println!("  salience: {}", output_result.provisional_salience);
                    println!("  fingerprint: {}", output_result.fingerprint);
                }
                if output_result.is_landmark {
                    println!(
                        "  landmark: {}",
                        output_result.landmark_label.as_deref().unwrap_or("(auto)")
                    );
                }
            }
        }
        Commands::Recall {
            query,
            namespace,
            top,
            context,
            kind,
            confidence,
            explain,
            like,
            unlike,
            include_public,
            no_engram,
            as_of,
            at,
            era,
            min_confidence,
            min_strength,
            show_decaying,
            cold_tier,
            output,
        } => {
            let config = validate_recall_command(
                namespace.as_deref(),
                query.as_deref(),
                *top,
                kind.as_deref(),
                confidence,
                explain,
                *like,
                *unlike,
                *include_public,
                *no_engram,
                *as_of,
                at.as_deref(),
                era.as_deref(),
                *min_confidence,
                *min_strength,
                *show_decaying,
                cold_tier,
                context.as_deref(),
            )?;
            let response = recall_memories(&store, &hot, &local_records, &config);
            if output.json {
                println!("{}", serde_json::to_string_pretty(&response)?);
            } else {
                let result_set = response.result.as_ref().expect("recall result present");
                if !output.quiet {
                    println!(
                        "Recall '{}' in '{}' → {} results",
                        config.query.as_deref().unwrap_or(""),
                        config.namespace.as_str(),
                        result_set.evidence_pack.len(),
                    );
                }
                if let Some(route_summary) = response.route_summary.as_ref() {
                    println!(
                        "  route: {} → {}",
                        route_summary.route_family, route_summary.route_reason
                    );
                }
                if config.explain != "none" {
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
                if output.verbose > 0 {
                    if let Some(query_by_example) = result_set.explain.query_by_example.as_ref() {
                        println!(
                            "  query_by_example: primary_cue={} requested={:?} materialized={:?} missing={:?}",
                            query_by_example.primary_cue,
                            query_by_example.requested_seed_descriptors,
                            query_by_example.materialized_seed_descriptors,
                            query_by_example.missing_seed_descriptors,
                        );
                    }
                    let temporal_reasons = result_set
                        .explain
                        .result_reasons
                        .iter()
                        .filter(|reason| {
                            reason.reason_code == "era_filter_applied"
                                || reason.reason_code == "temporal_landmark_selected"
                                || reason.reason_code == "temporal_landmark_not_selected"
                        })
                        .map(|reason| reason.detail.as_str())
                        .collect::<Vec<_>>();
                    if !temporal_reasons.is_empty() {
                        println!("  temporal: {}", temporal_reasons.join(" | "));
                    }
                    if !response.warnings.is_empty() {
                        for warning in &response.warnings {
                            println!("  warning: {} [{}]", warning.detail, warning.code);
                        }
                    }
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
        Commands::Budget {
            token_budget,
            query,
            namespace,
            context,
            format,
            top,
            include_public,
            output,
        } => {
            let namespace = namespace
                .as_deref()
                .ok_or_else(|| anyhow::anyhow!("missing namespace"))?;
            let namespace = NamespaceId::new(namespace)?;
            if *top == 0 {
                anyhow::bail!("result budget must be greater than zero");
            }
            let format = parse_injection_format(format)?;
            let response = budget_memories(
                &store,
                &local_records,
                &namespace,
                query.as_deref(),
                context.as_deref(),
                *top,
                *token_budget,
                format,
                *include_public,
            );
            if output.json {
                println!("{}", serde_json::to_string_pretty(&response)?);
            } else if let Some(result) = response.result.as_ref() {
                println!(
                    "Budget '{}' in '{}' → {} injections ({} used / {} remaining)",
                    query.as_deref().unwrap_or(""),
                    namespace.as_str(),
                    result.injections.len(),
                    result.tokens_used,
                    result.tokens_remaining,
                );
                for (index, item) in result.injections.iter().enumerate() {
                    println!(
                        "  [{}] #{} utility={:.3} tokens={} reason={} | {}",
                        index + 1,
                        item.memory_id.0,
                        item.utility_score,
                        item.token_count,
                        item.reason.as_str(),
                        item.content,
                    );
                }
                if output.verbose > 0 {
                    for omitted in &result.omitted {
                        println!(
                            "  omitted #{} reason={} utility={:.3} tokens={}",
                            omitted.memory_id.0,
                            omitted.reason.as_str(),
                            omitted.utility_score,
                            omitted.token_count,
                        );
                    }
                    for warning in &response.warnings {
                        println!("  warning: {} [{}]", warning.detail, warning.code);
                    }
                }
            }
        }
        Commands::Inspect {
            id,
            namespace,
            output,
        } => {
            let ns = NamespaceId::new(namespace)?;
            let memory_id = MemoryId(*id);
            match inspect_memory(&mut hot, &local_records, &ns, memory_id) {
                Ok(output_result) => {
                    if output.json {
                        let mut response = ResponseContext::success(
                            ns.clone(),
                            RequestId::new(format!("inspect-{}", output_result.memory_id))?,
                            output_result.clone(),
                        );
                        if let Some(passive) = output_result.passive_observation.clone() {
                            response = response.with_passive_observation(passive);
                        }
                        println!("{}", serde_json::to_string_pretty(&response)?);
                    } else {
                        if !output.quiet {
                            println!(
                                "Inspect #{} in '{}' [{} / {}]",
                                output_result.memory_id,
                                output_result.namespace,
                                output_result.memory_type,
                                output_result.route_family,
                            );
                        }
                        println!("  text: {}", output_result.compact_text);
                        if output.verbose > 0 {
                            println!("  salience: {}", output_result.provisional_salience);
                            println!("  fingerprint: {}", output_result.fingerprint);
                            println!(
                                "  payload: {} bytes ({})",
                                output_result.payload_size_bytes, output_result.payload_state
                            );
                            println!("  session: {}", output_result.session_id);
                            if let Some(passive) = output_result.passive_observation.as_ref() {
                                let observation_chunk_id = match &passive.observation_chunk_id {
                                    FieldPresence::Present(value) => value.as_str(),
                                    FieldPresence::Absent => "absent",
                                    FieldPresence::Redacted => "redacted",
                                };
                                println!(
                                    "  passive_observation: {} / {} / {}",
                                    passive.source_kind,
                                    passive.write_decision,
                                    observation_chunk_id
                                );
                            }
                        }
                    }
                }
                Err(e) => {
                    if output.json {
                        let resp: ResponseContext<()> = ResponseContext::failure(
                            ns,
                            RequestId::new("inspect-not-found")?,
                            membrain_core::api::ErrorKind::ValidationFailure,
                            vec![],
                        );
                        println!("{}", serde_json::to_string_pretty(&resp)?);
                    } else {
                        eprintln!("Invalid request: {}", e);
                        std::process::exit(1);
                    }
                }
            }
        }
        Commands::Explain {
            query,
            namespace,
            output,
        } => {
            let ns = NamespaceId::new(namespace)?;
            let response = explain_query(&store, &local_records, query, &ns);
            if output.json {
                println!("{}", serde_json::to_string_pretty(&response)?);
            } else {
                if !output.quiet {
                    println!("Explain '{}' in '{}'", query, ns.as_str());
                }
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
            output,
        } => {
            let ns_str = namespace.as_deref().unwrap_or("default");
            let ns = NamespaceId::new(ns_str)?;

            let targets = match action.as_str() {
                "repair" | "repair_all" => vec![
                    RepairTarget::LexicalIndex,
                    RepairTarget::MetadataIndex,
                    RepairTarget::SemanticHotIndex,
                    RepairTarget::SemanticColdIndex,
                    RepairTarget::GraphConsistency,
                    RepairTarget::CacheWarmState,
                    RepairTarget::EngramIndex,
                ],
                "repair_index" | "repair_indexes" => vec![
                    RepairTarget::LexicalIndex,
                    RepairTarget::MetadataIndex,
                    RepairTarget::SemanticHotIndex,
                    RepairTarget::SemanticColdIndex,
                ],
                "repair_metadata" => vec![RepairTarget::MetadataIndex],
                "repair_graph" => vec![RepairTarget::GraphConsistency],
                "repair_lineage" => vec![RepairTarget::EngramIndex],
                "repair_cache" => vec![RepairTarget::CacheWarmState],
                _ => {
                    eprintln!(
                        "Unknown maintenance action '{}'. Available: repair, repair_index, repair_metadata, repair_graph, repair_lineage, repair_cache",
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

            let mut maintenance_output = None;
            for _ in 0..16 {
                let snapshot = handle.poll();
                match snapshot.state {
                    MaintenanceJobState::Completed(summary) => {
                        maintenance_output = Some(MaintenanceOutput {
                            outcome: "accepted",
                            action: action.clone(),
                            namespace: ns.as_str().to_string(),
                            targets_checked: summary.targets_checked,
                            rebuilt: summary.rebuilt,
                            affected_item_count: summary.affected_item_count,
                            error_count: summary.error_count,
                            rebuild_duration_ms: summary.rebuild_duration_ms,
                            rollback_state: summary.rollback_state,
                            queue_depth_before: summary.queue_report.queue_depth_before,
                            queue_depth_after: summary.queue_report.queue_depth_after,
                            results: summary
                                .results
                                .iter()
                                .map(|r| {
                                    let report = summary
                                        .operator_reports
                                        .iter()
                                        .find(|report| report.target == r.target)
                                        .expect("repair operator report should exist for each maintenance result");
                                    let plan = r
                                        .rebuild_entrypoint
                                        .and_then(|entrypoint| {
                                            store.repair_engine().plan_index_rebuild(r.target, entrypoint)
                                        });
                                    let artifact = summary
                                        .verification_artifacts
                                        .get(&r.target)
                                        .expect("verification artifact should exist for each maintenance result");
                                    RepairResultOutput {
                                        target: r.target.as_str(),
                                        status: r.status.as_str(),
                                        verification_passed: r.verification_passed,
                                        rebuild_entrypoint: r
                                            .rebuild_entrypoint
                                            .map(IndexRepairEntrypoint::as_str),
                                        rebuilt_outputs: r.rebuilt_outputs.clone(),
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
                                        queue_depth_before: report.queue_depth_before,
                                        queue_depth_after: report.queue_depth_after,
                                    }
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

            match maintenance_output {
                Some(result) => {
                    if output.json {
                        println!("{}", serde_json::to_string_pretty(&result)?);
                    } else {
                        println!(
                            "Maintenance '{}' on '{}' → {} checked, {} rebuilt, affected={}, errors={}, duration_ms={}, rollback_state={}, queue={}→{}",
                            result.action,
                            result.namespace,
                            result.targets_checked,
                            result.rebuilt,
                            result.affected_item_count,
                            result.error_count,
                            result.rebuild_duration_ms,
                            result.rollback_state.unwrap_or("none"),
                            result.queue_depth_before,
                            result.queue_depth_after,
                        );
                        for r in &result.results {
                            println!(
                                "  {} [{}] verified={} outputs={:?} affected={} errors={} duration_ms={} rollback_state={} queue={}→{}",
                                r.target,
                                r.status,
                                r.verification_passed,
                                r.rebuilt_outputs,
                                r.affected_item_count,
                                r.error_count,
                                r.rebuild_duration_ms,
                                r.rollback_state.unwrap_or("none"),
                                r.queue_depth_before,
                                r.queue_depth_after,
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
            output,
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

            let benchmark_output = BenchmarkOutput {
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

            if output.json {
                println!("{}", serde_json::to_string_pretty(&benchmark_output)?);
            } else {
                println!(
                    "Benchmark '{}': {} iters, avg={:.1}us, min={:.1}us, max={:.1}us, p50={:.1}us, p95={:.1}us, total={:.1}ms",
                    target, iters, avg_us, min_us, max_us, p50_us, p95_us, total_ms
                );
            }
        }
        Commands::Doctor { command, output } => {
            let _ = command.as_ref().unwrap_or(&DoctorCommands::Run);
            let report = doctor_report();
            if output.json {
                print_doctor_report(&report)?;
            } else {
                if !output.quiet {
                    println!("Doctor run [{}]", report.status);
                }
                println!("  posture: {}", report.posture);
                println!("  indexes: {}", report.indexes.len());
                if output.verbose > 0 {
                    println!("  warnings: {}", report.warnings.join(", "));
                }
            }
        }
        Commands::Observe {
            content,
            namespace,
            context,
            chunk_size,
            topic_threshold,
            min_chunk_size,
            source_label,
            dry_run,
            watch,
            pattern,
            output,
        } => {
            let ns = NamespaceId::new(namespace)?;
            let (observed_content, watched_path) = if let Some(path) = watch {
                (std::fs::read_to_string(path)?, Some(path.as_path()))
            } else if let Some(content) = content {
                (content.clone(), None)
            } else {
                use std::io::Read;
                let mut buffer = String::new();
                std::io::stdin().read_to_string(&mut buffer)?;
                (buffer, None)
            };
            let config = ObserveConfig {
                chunk_size_chars: *chunk_size,
                topic_shift_threshold: *topic_threshold,
                min_chunk_chars: *min_chunk_size,
                context: context.clone(),
                source_label: source_label.clone(),
            };
            let response = observe_memories(
                &store,
                &mut hot,
                &mut local_records,
                &ns,
                &observed_content,
                &config,
                *dry_run,
                watched_path,
                pattern.as_deref(),
            );
            if output.json {
                println!("{}", serde_json::to_string_pretty(&response)?);
            } else {
                let result = response.result.as_ref().expect("observe result present");
                if !output.quiet {
                    println!(
                        "Observe '{}' in '{}' → {} fragment(s)",
                        result.observation_source,
                        result.namespace,
                        result.fragments_previewed,
                    );
                }
                println!("  batch: {}", result.observation_chunk_id);
                println!("  bytes: {}", result.bytes_processed);
                println!("  topic_shifts: {}", result.topic_shifts);
                println!("  created: {}", result.memories_created);
                if result.suppressed > 0 || result.denied > 0 {
                    println!(
                        "  suppressed: {}, denied: {}",
                        result.suppressed, result.denied
                    );
                }
                if *dry_run {
                    println!("  dry_run: preview only");
                }
                if output.verbose > 0 {
                    for fragment in &result.preview {
                        println!(
                            "  fragment #{} [{} / {}] {}",
                            fragment.index,
                            fragment.route_family,
                            fragment.write_decision,
                            fragment.compact_text
                        );
                    }
                }
            }
        }
        Commands::Preflight { command } => match command {
            PreflightCommands::Run {
                namespace,
                original_query,
                proposed_action,
                output,
            } => {
                let response = cli_preflight_run(namespace, original_query, proposed_action)?;
                if output.json {
                    println!("{}", serde_json::to_string_pretty(&response)?);
                } else {
                    if !output.quiet {
                        print_preflight_run_human(&response);
                    }
                }
                let exit_code = preflight_exit_code(&response);
                if exit_code != 0 {
                    std::process::exit(exit_code);
                }
            }
            PreflightCommands::Explain {
                namespace,
                original_query,
                proposed_action,
                output,
            } => {
                let response = cli_preflight_explain(namespace, original_query, proposed_action)?;
                if output.json {
                    println!("{}", serde_json::to_string_pretty(&response)?);
                } else {
                    if !output.quiet {
                        print_preflight_explain_human(&response);
                    }
                }
                let exit_code = explain_exit_code(&response);
                if exit_code != 0 {
                    std::process::exit(exit_code);
                }
            }
            PreflightCommands::Allow {
                namespace,
                original_query,
                proposed_action,
                authorization_token,
                bypass_flags,
                output,
            } => {
                let response = cli_preflight_allow(
                    namespace,
                    original_query,
                    proposed_action,
                    authorization_token,
                    bypass_flags,
                )?;
                if output.json {
                    println!("{}", serde_json::to_string_pretty(&response)?);
                } else {
                    if !output.quiet {
                        print_preflight_allow_human(&response);
                    }
                }
                let exit_code = preflight_exit_code(&response);
                if exit_code != 0 {
                    std::process::exit(exit_code);
                }
            }
        },
        Commands::Audit {
            namespace,
            id,
            session,
            since,
            op,
            recent,
            output,
        } => {
            let ns = NamespaceId::new(namespace)?;
            let log = sample_audit_log(&ns);
            let export =
                filter_audit_rows(&log, &ns, *id, *session, *since, op.as_deref(), *recent)?;
            print_audit_rows(&export, output.json)?;
        }
        Commands::Share {
            id,
            namespace_id,
            output,
        } => {
            let ns = NamespaceId::new(namespace_id)?;
            let output_result = share_output(*id, &ns, "shared");
            if output.json {
                let response = ResponseContext::success(
                    ns.clone(),
                    RequestId::new(format!("share-{id}"))?,
                    output_result,
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
                    output_result.visibility
                );
            }
        }
        Commands::Unshare {
            id,
            namespace,
            output,
        } => {
            let ns = NamespaceId::new(namespace)?;
            let output_result = unshare_output(*id, &ns);
            if output.json {
                let response = ResponseContext::success(
                    ns.clone(),
                    RequestId::new(format!("unshare-{id}"))?,
                    output_result,
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
        build_retrieval_result_set, cli_preflight_allow, cli_preflight_explain,
        filter_audit_rows, inspect_memory, observe_memories, parse_audit_category,
        parse_audit_kind, print_audit_rows, response_trace_for_result_set, sample_audit_log,
        share_output, unshare_output, Cli, Commands, LocalMemoryRecord, PreflightCommands,
        RecallCommandConfig,
    };
    use clap::Parser;
    use membrain_core::api::{NamespaceId, TraceStage};
    use membrain_core::engine::observe::ObserveConfig;
    use membrain_core::{BrainStore, RuntimeConfig};
    use membrain_core::engine::confidence::{ConfidenceInputs, ConfidencePolicy};
    use membrain_core::engine::ranking::{fuse_scores, RankingInput, RankingProfile};
    use membrain_core::engine::recall::{RecallPlanKind, RecallTraceStage};
    use membrain_core::engine::result::{AnsweredFrom, ResultBuilder, RetrievalExplain};
    use membrain_core::types::{CanonicalMemoryType, FastPathRouteFamily, MemoryId, SessionId};

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

    fn parsed_recall_namespace_and_top(command: Commands) -> Option<(Option<String>, usize)> {
        match command {
            Commands::Recall { namespace, top, .. } => Some((namespace, top)),
            _ => None,
        }
    }

    fn parsed_budget_namespace_top_and_tokens(
        command: Commands,
    ) -> Option<(Option<String>, usize, usize)> {
        match command {
            Commands::Budget {
                namespace,
                top,
                token_budget,
                ..
            } => Some((namespace, top, token_budget)),
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
            Some((Some("team.alpha".to_string()), 7))
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
            Some((Some("team.alpha".to_string()), 4))
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
            Some((Some("team.alpha".to_string()), 3))
        );
    }

    #[test]
    fn budget_command_preserves_namespace_top_and_token_flags() {
        let cli = Cli::parse_from([
            "membrain",
            "budget",
            "incident timeline",
            "--tokens",
            "200",
            "--namespace",
            "team.alpha",
            "-n",
            "4",
            "--format",
            "markdown",
        ]);

        assert_eq!(
            parsed_budget_namespace_top_and_tokens(cli.command),
            Some((Some("team.alpha".to_string()), 4, 200))
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
    fn observe_command_preserves_passive_observation_surface() {
        let cli = Cli::parse_from([
            "membrain",
            "observe",
            "captured stream",
            "--namespace",
            "team.alpha",
            "--context",
            "coding session",
            "--chunk-size",
            "120",
            "--topic-threshold",
            "0.4",
            "--min-chunk-size",
            "32",
            "--source-label",
            "stdin:session",
            "--dry-run",
        ]);

        match cli.command {
            Commands::Observe {
                content,
                namespace,
                context,
                chunk_size,
                topic_threshold,
                min_chunk_size,
                source_label,
                dry_run,
                watch,
                pattern,
                ..
            } => {
                assert_eq!(content.as_deref(), Some("captured stream"));
                assert_eq!(namespace, "team.alpha");
                assert_eq!(context.as_deref(), Some("coding session"));
                assert_eq!(chunk_size, 120);
                assert_eq!(topic_threshold, 0.4);
                assert_eq!(min_chunk_size, 32);
                assert_eq!(source_label.as_deref(), Some("stdin:session"));
                assert!(dry_run);
                assert!(watch.is_none());
                assert!(pattern.is_none());
            }
            other => panic!("expected observe command, got {other:?}"),
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
    fn observe_memories_uses_shared_batch_metadata_and_dry_run_preview() {
        let store = BrainStore::new(RuntimeConfig::default());
        let mut hot = store.hot_store().new_metadata_store(64);
        let mut local_records = Vec::new();
        let namespace = NamespaceId::new("team.alpha").unwrap();
        let config = ObserveConfig {
            chunk_size_chars: 80,
            topic_shift_threshold: 0.30,
            min_chunk_chars: 20,
            context: Some("coding session".to_string()),
            source_label: Some("stdin:test".to_string()),
        };
        let content =
            "build stayed green across canary rollout.\n\nuser prefers dark mode in dashboard settings.";

        let preview = observe_memories(
            &store,
            &mut hot,
            &mut local_records,
            &namespace,
            content,
            &config,
            true,
            None,
            None,
        );
        let preview_result = preview.result.as_ref().unwrap();
        assert!(preview.partial_success);
        assert_eq!(preview_result.memories_created, 0);
        assert_eq!(preview_result.fragments_previewed, 2);
        assert_eq!(preview_result.observation_source, "stdin:test");
        assert_eq!(local_records.len(), 0);
        assert_eq!(preview.passive_observation.as_ref().unwrap().source_kind, "observation");

        let applied = observe_memories(
            &store,
            &mut hot,
            &mut local_records,
            &namespace,
            content,
            &config,
            false,
            None,
            None,
        );
        let applied_result = applied.result.as_ref().unwrap();
        assert_eq!(applied_result.memories_created, 2);
        assert_eq!(local_records.len(), 2);
        assert!(local_records.iter().all(|record| {
            record
                .passive_observation
                .as_ref()
                .and_then(|passive| match &passive.observation_chunk_id {
                    membrain_core::api::FieldPresence::Present(value) => Some(value.as_str()),
                    _ => None,
                })
                == Some(applied_result.observation_chunk_id.as_str())
        }));

        let inspected = inspect_memory(&mut hot, &local_records, &namespace, MemoryId(1)).unwrap();
        let passive = inspected.passive_observation.as_ref().unwrap();
        assert_eq!(passive.source_kind, "observation");
        assert_eq!(passive.write_decision, "capture");
        assert!(passive.captured_as_observation);
        assert_eq!(
            passive.observation_source,
            membrain_core::api::FieldPresence::Present("stdin:test".to_string())
        );
        assert_eq!(
            passive.observation_chunk_id,
            membrain_core::api::FieldPresence::Present(
                applied_result.observation_chunk_id.clone()
            )
        );
        assert_eq!(
            passive.retention_marker,
            membrain_core::api::FieldPresence::Present("volatile_observation")
        );
    }

    #[test]
    fn preflight_commands_preserve_shared_cli_surface() {
        let run = Cli::parse_from([
            "membrain",
            "preflight",
            "run",
            "--namespace",
            "team.alpha",
            "--original-query",
            "delete prior audit events",
            "--proposed-action",
            "purge namespace audit history",
        ]);
        let explain = Cli::parse_from([
            "membrain",
            "preflight",
            "explain",
            "--namespace",
            "team.alpha",
            "--original-query",
            "delete prior audit events",
            "--proposed-action",
            "purge namespace audit history",
        ]);
        let allow = Cli::parse_from([
            "membrain",
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

        match run.command {
            Commands::Preflight {
                command:
                    PreflightCommands::Run {
                        namespace,
                        original_query,
                        proposed_action,
                        ..
                    },
            } => {
                assert_eq!(namespace, "team.alpha");
                assert_eq!(original_query, "delete prior audit events");
                assert_eq!(proposed_action, "purge namespace audit history");
            }
            other => panic!("expected preflight run command, got {other:?}"),
        }

        assert!(matches!(
            explain.command,
            Commands::Preflight {
                command: PreflightCommands::Explain { .. }
            }
        ));
        match allow.command {
            Commands::Preflight {
                command:
                    PreflightCommands::Allow {
                        authorization_token,
                        bypass_flags,
                        ..
                    },
            } => {
                assert_eq!(authorization_token, "allow-123");
                assert_eq!(bypass_flags, vec!["manual_override"]);
            }
            other => panic!("expected preflight allow command, got {other:?}"),
        }
    }

    #[test]
    fn cli_preflight_helpers_preserve_shared_blocked_and_force_confirmed_semantics() {
        let explain = cli_preflight_explain(
            "team.alpha",
            "delete prior audit events across all namespaces",
            "purge namespace audit history",
        )
        .expect("preflight explain should succeed");
        assert_eq!(explain.preflight_state, "blocked");
        assert_eq!(explain.preflight_outcome, "preview_only");
        assert_eq!(
            explain.blocked_reasons,
            vec![
                "scope_ambiguous".to_string(),
                "confirmation_required".to_string()
            ]
        );
        assert_eq!(explain.audit.actor_source, "cli_preflight");
        assert!(explain
            .request_id
            .as_deref()
            .is_some_and(|id| id.starts_with("cli-preflight-explain-")));

        let allow = cli_preflight_allow(
            "team.alpha",
            "delete prior audit events",
            "purge namespace audit history",
            "allow-123",
            &["manual_override".to_string()],
        )
        .expect("preflight allow should succeed");
        assert!(allow.success);
        assert_eq!(allow.preflight_state, "ready");
        assert_eq!(allow.preflight_outcome, "force_confirmed");
        assert_eq!(allow.outcome_class, "accepted");
        assert_eq!(allow.confirmation.confirmed, true);
        assert_eq!(
            allow.confirmation_reason.as_deref(),
            Some("operator confirmed exact previewed scope")
        );
        assert_eq!(allow.audit.actor_source, "cli_preflight");
        assert!(allow
            .request_id
            .as_deref()
            .is_some_and(|id| id.starts_with("cli-preflight-allow-")));
    }

    #[test]
    fn response_trace_bundle_uses_canonical_explain_route_stages() {
        let namespace = NamespaceId::new("team.gamma").unwrap();
        let config = RecallCommandConfig {
            query: Some("search-term".to_string()),
            context: None,
            top: 3,
            kind: None,
            confidence: "normal".to_string(),
            explain: "summary".to_string(),
            namespace: namespace.clone(),
            include_public: false,
            like: None,
            unlike: None,
            graph_expansion: false,
            as_of: None,
            at: None,
            era: None,
            min_confidence: None,
            min_strength: None,
            show_decaying: false,
            cold_tier: "auto".to_string(),
        };
        let result_set = build_retrieval_result_set(
            &[],
            &config,
            RankingProfile::balanced(),
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
    fn build_retrieval_result_set_reports_landmark_anchors_for_era_scoped_recall() {
        let namespace = NamespaceId::new("team.gamma").unwrap();
        let local_records = vec![
            LocalMemoryRecord {
                memory_id: MemoryId(7),
                namespace: namespace.clone(),
                session_id: SessionId(2),
                memory_type: CanonicalMemoryType::Event,
                route_family: FastPathRouteFamily::Event,
                compact_text: "launch day pivot".to_string(),
                provisional_salience: 900,
                fingerprint: 700,
                payload_size_bytes: 32,
                is_landmark: true,
                landmark_label: Some("launch day pivot".to_string()),
                era_id: Some("era-launch-0042".to_string()),
                passive_observation: None,
            },
            LocalMemoryRecord {
                memory_id: MemoryId(8),
                namespace: namespace.clone(),
                session_id: SessionId(2),
                memory_type: CanonicalMemoryType::Observation,
                route_family: FastPathRouteFamily::Observation,
                compact_text: "follow-up task carried into launch era".to_string(),
                provisional_salience: 650,
                fingerprint: 800,
                payload_size_bytes: 48,
                is_landmark: false,
                landmark_label: None,
                era_id: Some("era-launch-0042".to_string()),
                passive_observation: None,
            },
            LocalMemoryRecord {
                memory_id: MemoryId(9),
                namespace: namespace.clone(),
                session_id: SessionId(2),
                memory_type: CanonicalMemoryType::Event,
                route_family: FastPathRouteFamily::Event,
                compact_text: "older unrelated era".to_string(),
                provisional_salience: 500,
                fingerprint: 900,
                payload_size_bytes: 24,
                is_landmark: true,
                landmark_label: Some("older unrelated era".to_string()),
                era_id: Some("era-older-0001".to_string()),
                passive_observation: None,
            },
        ];
        let config = RecallCommandConfig {
            query: Some("launch".to_string()),
            context: None,
            top: 5,
            kind: None,
            confidence: "normal".to_string(),
            explain: "full".to_string(),
            namespace: namespace.clone(),
            include_public: false,
            like: None,
            unlike: None,
            graph_expansion: false,
            as_of: None,
            at: None,
            era: Some("era-launch-0042".to_string()),
            min_confidence: None,
            min_strength: None,
            show_decaying: false,
            cold_tier: "auto".to_string(),
        };

        let result_set = build_retrieval_result_set(
            &local_records,
            &config,
            RankingProfile::balanced(),
            "balanced",
            "temporal recall stayed inside one landmark-defined era".to_string(),
            vec![MemoryId(7), MemoryId(8), MemoryId(9)],
        );

        let returned_ids = result_set
            .evidence_pack
            .iter()
            .map(|item| item.result.memory_id)
            .collect::<Vec<_>>();
        assert_eq!(returned_ids, vec![MemoryId(7), MemoryId(8)]);
        let era_reason = result_set
            .explain
            .result_reasons
            .iter()
            .find(|reason| reason.reason_code == "era_filter_applied")
            .expect("era filter reason");
        assert!(era_reason.detail.contains("era `era-launch-0042`"));
        assert!(era_reason.detail.contains("launch day pivot (#7)"));
        assert!(era_reason
            .detail
            .contains("2 candidate(s) remained after era scoping"));
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
        assert_eq!(
            uncertainty_markers[0].code,
            expected_uncertainty_markers[0].code
        );
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
                output,
            } => {
                assert_eq!(id, 42);
                assert_eq!(namespace_id, "team.beta");
                assert!(!output.json);
            }
            other => panic!("expected share command, got {other:?}"),
        }

        match unshare.command {
            Commands::Unshare {
                id,
                namespace,
                output,
            } => {
                assert_eq!(id, 42);
                assert_eq!(namespace, "team.alpha");
                assert!(!output.json);
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
        assert_eq!(shared.audit.event_kind, "approved_sharing");
        assert_eq!(shared.audit.actor_source, "cli_share");
        assert_eq!(shared.audit.request_id, "req-share-42");
        assert_eq!(shared.audit.effective_namespace, "team.beta");
        assert_eq!(shared.audit.source_namespace.as_deref(), Some("team.beta"));
        assert_eq!(shared.audit.target_namespace.as_deref(), Some("team.beta"));
        assert_eq!(shared.audit.policy_family, "visibility_sharing");
        assert_eq!(shared.audit.outcome_class, "accepted");
        assert_eq!(shared.audit.blocked_stage, "policy_gate");
        assert_eq!(shared.audit.related_run.as_deref(), Some("share-run-42"));
        assert!(!shared.audit.redacted);
        assert_eq!(shared.audit.redaction_summary, Vec::<String>::new());
        assert_eq!(shared.audit_rows.len(), 1);
        assert_eq!(
            shared.audit_rows[0].request_id.as_deref(),
            Some("req-share-42")
        );
        assert_eq!(
            shared.audit_rows[0].related_run.as_deref(),
            Some("share-run-42")
        );
        assert_eq!(shared.audit_rows[0].kind, "approved_sharing");

        let unshared = unshare_output(42, &NamespaceId::new("team.alpha").unwrap());
        assert_eq!(unshared.visibility, "private");
        assert_eq!(
            unshared.policy_summary.redaction_fields,
            vec!["sharing_scope"]
        );
        assert_eq!(unshared.audit.event_kind, "policy_redacted");
        assert_eq!(unshared.audit.actor_source, "cli_unshare");
        assert_eq!(unshared.audit.request_id, "req-unshare-42");
        assert_eq!(unshared.audit.effective_namespace, "team.alpha");
        assert_eq!(
            unshared.audit.source_namespace.as_deref(),
            Some("team.alpha")
        );
        assert_eq!(
            unshared.audit.target_namespace.as_deref(),
            Some("team.alpha")
        );
        assert_eq!(unshared.audit.policy_family, "visibility_sharing");
        assert_eq!(unshared.audit.outcome_class, "accepted");
        assert_eq!(unshared.audit.blocked_stage, "policy_gate");
        assert_eq!(unshared.audit.related_run.as_deref(), Some("share-run-42"));
        assert!(unshared.audit.redacted);
        assert_eq!(
            unshared.audit.redaction_summary,
            vec!["sharing_scope".to_string()]
        );
        assert_eq!(unshared.audit_rows.len(), 1);
        assert_eq!(
            unshared.audit_rows[0].request_id.as_deref(),
            Some("req-unshare-42")
        );
        assert_eq!(
            unshared.audit_rows[0].related_run.as_deref(),
            Some("share-run-42")
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
        assert_eq!(
            parse_audit_kind("maintenance_forgetting_evaluated"),
            Some(membrain_core::observability::AuditEventKind::MaintenanceForgettingEvaluated)
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
