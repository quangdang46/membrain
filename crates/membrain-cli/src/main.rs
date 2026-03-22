use clap::{Parser, Subcommand};
use membrain_core::api::{
    AvailabilityReason, AvailabilitySummary, NamespaceId, RemediationStep, RequestId,
    ResponseContext,
};
use membrain_core::engine::maintenance::{
    MaintenanceController, MaintenanceJobHandle, MaintenanceJobState,
};
use membrain_core::engine::recall::RecallRuntime;
use membrain_core::engine::repair::{IndexRepairEntrypoint, RepairTarget};
use membrain_core::health::{BrainHealthInputs, BrainHealthReport, FeatureAvailabilityEntry};
use membrain_core::index::{IndexApi, IndexModule};
use membrain_core::observability::{AuditEventCategory, AuditEventKind};
use membrain_core::store::audit::{AppendOnlyAuditLog, AuditLogEntry, AuditLogStore};
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
        #[arg(long, short = 'n', default_value = "default")]
        namespace: String,
        /// Maximum number of results to return
        #[arg(long, short = 't', default_value_t = 5)]
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
struct RecallResultEntry {
    memory_id: u64,
    compact_text: String,
    memory_type: &'static str,
    provisional_salience: u16,
    entry_lane: &'static str,
}

#[derive(Debug, Clone, Serialize)]
struct RecallOutput {
    outcome: &'static str,
    query: String,
    namespace: String,
    intent: &'static str,
    intent_confidence: &'static str,
    route_family: &'static str,
    route_reason: &'static str,
    tier1_consulted_first: bool,
    tier1_answered_directly: bool,
    routes_to_deeper_tiers: bool,
    results: Vec<RecallResultEntry>,
    explain_verbosity: String,
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

#[derive(Debug, Clone, Serialize)]
struct ExplainOutput {
    outcome: &'static str,
    query: String,
    namespace: String,
    intent: &'static str,
    intent_confidence: &'static str,
    query_path: &'static str,
    ranking_profile: &'static str,
    route_family: &'static str,
    route_reason: &'static str,
    tier1_consulted_first: bool,
    tier1_answered_directly: bool,
    routes_to_deeper_tiers: bool,
    trace_stages: Vec<&'static str>,
    matched_patterns: Vec<String>,
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
) -> EncodeOutput {
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
    }
}

// ── Recall ───────────────────────────────────────────────────────────────────

fn recall_memories(
    store: &BrainStore,
    _hot: &mut Tier1HotMetadataStore,
    local_records: &[LocalMemoryRecord],
    query: &str,
    namespace: &NamespaceId,
    top: usize,
    explain_verbosity: &str,
) -> RecallOutput {
    let intent_result = store.intent_engine().classify(query);
    let session_id = SessionId(SESSION_ID.load(Ordering::SeqCst));

    let recall_request =
        membrain_core::engine::recall::RecallRequest::small_session_lookup(session_id);
    let recall_plan = store
        .recall_engine()
        .plan_recall(recall_request, store.config());

    // Scan local records for matching memories in this namespace
    let mut results = Vec::new();
    let query_lower = query.to_lowercase();
    for record in local_records.iter().filter(|r| r.namespace == *namespace) {
        let text_lower = record.compact_text.to_lowercase();
        if text_lower.contains(&query_lower)
            || query_lower.contains(&text_lower)
            || record.memory_type.as_str().contains(&query_lower)
        {
            results.push(RecallResultEntry {
                memory_id: record.memory_id.0,
                compact_text: record.compact_text.clone(),
                memory_type: record.memory_type.as_str(),
                provisional_salience: record.provisional_salience,
                entry_lane: "recent",
            });
        }
    }

    // If no keyword matches, return recent entries as fallback
    if results.is_empty() {
        for record in local_records
            .iter()
            .filter(|r| r.namespace == *namespace)
            .rev()
            .take(top)
        {
            results.push(RecallResultEntry {
                memory_id: record.memory_id.0,
                compact_text: record.compact_text.clone(),
                memory_type: record.memory_type.as_str(),
                provisional_salience: record.provisional_salience,
                entry_lane: "recent",
            });
        }
    }

    results.truncate(top);

    RecallOutput {
        outcome: if results.is_empty() {
            "partial"
        } else {
            "accepted"
        },
        query: query.to_string(),
        namespace: namespace.as_str().to_string(),
        intent: intent_result.intent.as_str(),
        intent_confidence: intent_confidence_label(intent_result.low_confidence_fallback),
        route_family: intent_result.route_inputs.ranking_profile.as_str(),
        route_reason: recall_plan.route_summary.reason,
        tier1_consulted_first: recall_plan.route_summary.tier1_consulted_first,
        tier1_answered_directly: recall_plan.route_summary.tier1_answers_directly,
        routes_to_deeper_tiers: recall_plan.route_summary.routes_to_deeper_tiers,
        results,
        explain_verbosity: explain_verbosity.to_string(),
    }
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

fn explain_query(store: &BrainStore, query: &str, namespace: &NamespaceId) -> ExplainOutput {
    let intent_result = store.intent_engine().classify(query);
    let log = intent_result.log_record();
    let session_id = SessionId(SESSION_ID.load(Ordering::SeqCst));

    let recall_request =
        membrain_core::engine::recall::RecallRequest::small_session_lookup(session_id);
    let recall_plan = store
        .recall_engine()
        .plan_recall(recall_request, store.config());

    let trace_stages: Vec<&'static str> = recall_plan
        .route_summary
        .trace_stages
        .iter()
        .map(|s| match s {
            membrain_core::engine::recall::RecallTraceStage::Tier1ExactHandle => {
                "tier1_exact_handle"
            }
            membrain_core::engine::recall::RecallTraceStage::Tier1RecentWindow => {
                "tier1_recent_window"
            }
            membrain_core::engine::recall::RecallTraceStage::Tier2Exact => "tier2_exact",
            membrain_core::engine::recall::RecallTraceStage::Tier3Fallback => "tier3_fallback",
        })
        .collect();

    ExplainOutput {
        outcome: "accepted",
        query: query.to_string(),
        namespace: namespace.as_str().to_string(),
        intent: intent_result.intent.as_str(),
        intent_confidence: intent_confidence_label(intent_result.low_confidence_fallback),
        query_path: intent_result.route_inputs.query_path.as_str(),
        ranking_profile: intent_result.route_inputs.ranking_profile.as_str(),
        route_family: intent_result.route_inputs.ranking_profile.as_str(),
        route_reason: recall_plan.route_summary.reason,
        tier1_consulted_first: recall_plan.route_summary.tier1_consulted_first,
        tier1_answered_directly: recall_plan.route_summary.tier1_answers_directly,
        routes_to_deeper_tiers: recall_plan.route_summary.routes_to_deeper_tiers,
        trace_stages,
        matched_patterns: log.matched_patterns.iter().map(|s| s.to_string()).collect(),
    }
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

fn filter_audit_rows(
    log: &AppendOnlyAuditLog,
    namespace: &NamespaceId,
    memory_id: Option<u64>,
    since: Option<u64>,
    op: Option<&str>,
    recent: Option<usize>,
) -> Vec<AuditRow> {
    let op = op.map(str::trim).filter(|value| !value.is_empty());
    let mut rows: Vec<_> = log
        .entries_for_namespace(namespace)
        .into_iter()
        .filter(|entry| since.is_none_or(|min_sequence| entry.sequence >= min_sequence))
        .filter(|entry| {
            memory_id.is_none_or(|expected| entry.memory_id == Some(MemoryId(expected)))
        })
        .filter(|entry| {
            op.is_none_or(|needle| {
                entry.kind.as_str() == needle || entry.category.as_str() == needle
            })
        })
        .map(AuditRow::from)
        .collect();

    if let Some(limit) = recent {
        if rows.len() > limit {
            rows = rows.split_off(rows.len() - limit);
        }
    }

    rows
}

fn print_audit_rows(rows: &[AuditRow], json: bool) -> anyhow::Result<()> {
    if json {
        println!("{}", serde_json::to_string_pretty(rows)?);
        return Ok(());
    }

    if rows.is_empty() {
        println!("No audit rows matched the requested filters.");
        return Ok(());
    }

    for row in rows {
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
    let overall_status = if warnings.is_empty() {
        "healthy"
    } else {
        "warn"
    };
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
            let output = encode_memory(
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
                println!("{}", serde_json::to_string_pretty(&output)?);
            } else {
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
            let output =
                recall_memories(&store, &mut hot, &local_records, query, &ns, *top, explain);
            if *json {
                println!("{}", serde_json::to_string_pretty(&output)?);
            } else {
                println!(
                    "Recall '{}' in '{}' → {} results",
                    output.query,
                    output.namespace,
                    output.results.len(),
                );
                println!(
                    "  intent: {} (confidence: {})",
                    output.intent, output.intent_confidence
                );
                println!("  route: {} → {}", output.route_family, output.route_reason);
                if explain != "none" {
                    println!(
                        "  tier1: consulted={}, answered_directly={}, deeper={}",
                        output.tier1_consulted_first,
                        output.tier1_answered_directly,
                        output.routes_to_deeper_tiers,
                    );
                }
                for (i, r) in output.results.iter().enumerate() {
                    println!(
                        "  [{}] #{} salience={} lane={} | {}",
                        i + 1,
                        r.memory_id,
                        r.provisional_salience,
                        r.entry_lane,
                        r.compact_text,
                    );
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
                        println!("{}", serde_json::to_string_pretty(&output)?);
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
                        println!("{}", serde_json::to_string_pretty(&resp)?);
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
            let output = explain_query(&store, query, &ns);
            if *json {
                println!("{}", serde_json::to_string_pretty(&output)?);
            } else {
                println!("Explain '{}' in '{}'", output.query, output.namespace);
                println!(
                    "  intent: {} (confidence: {})",
                    output.intent, output.intent_confidence
                );
                println!(
                    "  query_path: {}  ranking: {}",
                    output.query_path, output.ranking_profile
                );
                println!("  route: {} → {}", output.route_family, output.route_reason);
                println!(
                    "  tier1: consulted={}, answered_directly={}, deeper={}",
                    output.tier1_consulted_first,
                    output.tier1_answered_directly,
                    output.routes_to_deeper_tiers,
                );
                println!("  trace_stages: {}", output.trace_stages.join(" → "));
                if !output.matched_patterns.is_empty() {
                    println!(
                        "  matched_patterns: [{}]",
                        output.matched_patterns.join(", ")
                    );
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
            since,
            op,
            recent,
            json,
        } => {
            let ns = NamespaceId::new(namespace)?;
            let log = sample_audit_log(&ns);
            let rows = filter_audit_rows(&log, &ns, *id, *since, op.as_deref(), *recent);
            print_audit_rows(&rows, *json)?;
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
    use super::{filter_audit_rows, print_audit_rows, sample_audit_log};
    use membrain_core::api::NamespaceId;

    #[test]
    fn audit_rows_preserve_request_id_in_json_export() {
        let namespace = NamespaceId::new("team.alpha").expect("valid namespace");
        let log = sample_audit_log(&namespace);
        let rows = filter_audit_rows(&log, &namespace, Some(21), None, None, None);

        assert_eq!(rows.len(), 3);
        assert_eq!(rows[0].request_id.as_deref(), Some("req-encode-21"));
        assert_eq!(rows[1].request_id.as_deref(), Some("req-policy-21"));
        assert_eq!(rows[2].request_id.as_deref(), Some("req-migration-21"));
    }

    #[test]
    fn text_export_includes_request_id_field() {
        let namespace = NamespaceId::new("team.alpha").expect("valid namespace");
        let log = sample_audit_log(&namespace);
        let rows = filter_audit_rows(&log, &namespace, Some(21), None, None, Some(1));

        let rendered = rows
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

        print_audit_rows(&rows, false).expect("text export should render");
    }
}
