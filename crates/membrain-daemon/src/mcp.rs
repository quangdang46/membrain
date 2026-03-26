use membrain_core::api::{
    CacheMetricsSummary, NamespaceId, PolicyFilterSummary, RequestId, TracePolicySummary,
};
use membrain_core::engine::result::RetrievalResultSet;
use membrain_core::observability::OutcomeClass;
use membrain_core::store::audit::AuditLogEntry;
use serde::{Deserialize, Serialize};

pub use crate::preflight::{
    PreflightAllowRequest, PreflightExplainResponse, PreflightOutcome, PreflightRunRequest,
};

// ---------------------------------------------------------------------------
// Common request envelope fields
// ---------------------------------------------------------------------------

/// Shared envelope fields carried by every MCP method request.
/// These are execution context, not authorization shortcuts.
#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct CommonRequestFields {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub request_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub workspace_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub agent_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub session_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub task_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub time_budget_ms: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub policy_context: Option<PolicyContextHint>,
}

const fn default_true() -> bool {
    true
}

/// Policy hints carried by the request envelope before core policy evaluation.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct PolicyContextHint {
    #[serde(default)]
    pub include_public: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sharing_visibility: Option<String>,
    #[serde(default = "default_true")]
    pub caller_identity_bound: bool,
    #[serde(default = "default_true")]
    pub workspace_acl_allowed: bool,
    #[serde(default = "default_true")]
    pub agent_acl_allowed: bool,
    #[serde(default = "default_true")]
    pub session_visibility_allowed: bool,
    #[serde(default)]
    pub legal_hold: bool,
}

// ---------------------------------------------------------------------------
// MCP Request method envelopes — Core Tools
// ---------------------------------------------------------------------------

/// Canonical MCP request tagged by method name.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "method", content = "params")]
pub enum McpRequest {
    // -- Encode / intake ---------------------------------------------------
    #[serde(rename = "encode")]
    Encode(EncodeParams),

    // -- Recall / query ----------------------------------------------------
    #[serde(rename = "recall")]
    Recall(RecallParams),
    #[serde(rename = "search")]
    Search(SearchParams),
    #[serde(rename = "get")]
    Get(GetParams),
    #[serde(rename = "ask")]
    Ask(AskParams),

    // -- Inspect / explain -------------------------------------------------
    #[serde(rename = "inspect")]
    Inspect(InspectParams),
    #[serde(rename = "explain")]
    Explain(ExplainParams),

    // -- Policy-aware mutation ---------------------------------------------
    #[serde(rename = "link")]
    Link(LinkParams),
    #[serde(rename = "pin")]
    Pin(PinParams),
    #[serde(rename = "forget")]
    Forget(ForgetParams),
    #[serde(rename = "repair")]
    Repair(RepairParams),
    #[serde(rename = "consolidate")]
    Consolidate(ConsolidateParams),
    #[serde(rename = "share")]
    Share(ShareParams),
    #[serde(rename = "unshare")]
    Unshare(UnshareParams),
    #[serde(rename = "preflight.run")]
    PreflightRun(PreflightRunRequest),
    #[serde(rename = "preflight.explain")]
    PreflightExplain(PreflightRunRequest),
    #[serde(rename = "preflight.allow")]
    PreflightAllow(PreflightAllowRequest),

    // -- Operator surfaces -------------------------------------------------
    #[serde(rename = "stats")]
    Stats(StatsParams),
    #[serde(rename = "health")]
    Health(HealthParams),
    #[serde(rename = "doctor")]
    Doctor(DoctorParams),
    #[serde(rename = "export")]
    Export(ExportParams),
    #[serde(rename = "import")]
    Import(ImportParams),

    // -- Feature-specific tools --------------------------------------------
    #[serde(rename = "dream")]
    Dream(DreamParams),
    #[serde(rename = "belief_history")]
    BeliefHistory(BeliefHistoryParams),
    #[serde(rename = "context_budget")]
    ContextBudget(ContextBudgetParams),
    #[serde(rename = "timeline")]
    Timeline(TimelineParams),
    #[serde(rename = "observe")]
    Observe(ObserveParams),
    #[serde(rename = "uncertain")]
    Uncertain(UncertainParams),
    #[serde(rename = "skills")]
    Skills(SkillsParams),
    #[serde(rename = "procedures")]
    Procedures(ProceduresParams),
    #[serde(rename = "why")]
    Why(WhyParams),
    #[serde(rename = "invalidate")]
    Invalidate(InvalidateParams),
    #[serde(rename = "snapshot")]
    Snapshot(SnapshotParams),
    #[serde(rename = "list_snapshots")]
    ListSnapshots(ListSnapshotsParams),
    #[serde(rename = "diff")]
    Diff(DiffParams),
    #[serde(rename = "fork")]
    Fork(ForkParams),
    #[serde(rename = "merge_fork")]
    MergeFork(MergeForkParams),
    #[serde(rename = "compress")]
    Compress(CompressParams),
    #[serde(rename = "schemas")]
    Schemas(SchemasParams),
    #[serde(rename = "hot_paths")]
    HotPaths(HotPathsParams),
    #[serde(rename = "dead_zones")]
    DeadZones(DeadZonesParams),
    #[serde(rename = "audit")]
    Audit(AuditParams),
    #[serde(rename = "mood_history")]
    MoodHistory(MoodHistoryParams),
    #[serde(rename = "query_by_example")]
    QueryByExample(QueryByExampleParams),

    // -- Goal management ---------------------------------------------------
    #[serde(rename = "goal_state")]
    GoalState(GoalStateParams),
    #[serde(rename = "goal_pause")]
    GoalPause(GoalPauseParams),
    #[serde(rename = "goal_pin")]
    GoalPin(GoalPinParams),
    #[serde(rename = "goal_dismiss")]
    GoalDismiss(GoalDismissParams),
    #[serde(rename = "goal_snapshot")]
    GoalSnapshot(GoalSnapshotParams),
    #[serde(rename = "goal_resume")]
    GoalResume(GoalResumeParams),
    #[serde(rename = "goal_abandon")]
    GoalAbandon(GoalAbandonParams),
}

// ---------------------------------------------------------------------------
// Encode / intake params
// ---------------------------------------------------------------------------

/// Ingest a new memory item.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct EncodeParams {
    pub content: String,
    pub namespace: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub memory_type: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source_metadata: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub context_bindings: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub emotional_annotations: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub salience: Option<u16>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tags: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub entity_refs: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub relation_refs: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub retention_hint: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub visibility: Option<String>,
    #[serde(flatten)]
    pub common: CommonRequestFields,
}

// ---------------------------------------------------------------------------
// Recall / query params
// ---------------------------------------------------------------------------

/// Task-oriented bounded retrieval for context construction.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RecallParams {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub query_text: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub context_text: Option<String>,
    pub namespace: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mode: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub result_budget: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub token_budget: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub time_budget_ms: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub effort: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub explain: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub include_public: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub like_id: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub unlike_id: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub graph_mode: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cold_tier: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub memory_kinds: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub era_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub as_of_tick: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub at_snapshot: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub min_strength: Option<u16>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub min_confidence: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub show_decaying: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mood_congruent: Option<bool>,
    #[serde(flatten)]
    pub common: CommonRequestFields,
}

/// Bounded search over indexes, tags, entities, time ranges, or semantic hints.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SearchParams {
    pub namespace: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub query: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub filters: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub memory_types: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub result_budget: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub min_confidence: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub min_uncertainty: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub include_uncertainty_interval: Option<bool>,
    #[serde(flatten)]
    pub common: CommonRequestFields,
}

/// Retrieve a memory item by ID.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct GetParams {
    pub id: u64,
    pub namespace: String,
    #[serde(flatten)]
    pub common: CommonRequestFields,
}

/// Auto-classify query intent and route to optimal recall config.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AskParams {
    pub query: String,
    pub namespace: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub explain_intent: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub override_intent: Option<String>,
    #[serde(flatten)]
    pub common: CommonRequestFields,
}

// ---------------------------------------------------------------------------
// Inspect / explain params
// ---------------------------------------------------------------------------

/// Retrieve diagnostic and structural details about a memory.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct InspectParams {
    pub id: u64,
    pub namespace: String,
    #[serde(flatten)]
    pub common: CommonRequestFields,
}

/// Explain why a memory was stored, routed, recalled, ranked, filtered, or forgotten.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ExplainParams {
    pub query: String,
    pub namespace: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub limit: Option<usize>,
    #[serde(flatten)]
    pub common: CommonRequestFields,
}

// ---------------------------------------------------------------------------
// Policy-aware mutation params
// ---------------------------------------------------------------------------

/// Create or update explicit relations between memories, entities, or goals.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct LinkParams {
    pub source_id: u64,
    pub target_id: u64,
    pub namespace: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub link_type: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub strength: Option<u16>,
    #[serde(flatten)]
    pub common: CommonRequestFields,
}

/// Raise retention protection or bypass normal forgetting/demotion.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PinParams {
    pub id: u64,
    pub namespace: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reason: Option<String>,
    #[serde(flatten)]
    pub common: CommonRequestFields,
}

/// Controlled forgetting: suppress, decay, demote, compact, archive, redact, or delete.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ForgetParams {
    pub id: u64,
    pub namespace: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mode: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reason: Option<String>,
    #[serde(flatten)]
    pub common: CommonRequestFields,
}

/// Run or schedule repair: indexes, graph, lineage, summaries, shards.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RepairParams {
    pub namespace: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub surfaces: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub dry_run: Option<bool>,
    #[serde(flatten)]
    pub common: CommonRequestFields,
}

/// Trigger or schedule consolidation workloads.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ConsolidateParams {
    pub namespace: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub scope: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub session_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub task_id: Option<String>,
    #[serde(flatten)]
    pub common: CommonRequestFields,
}

/// Adjust visibility for cross-agent access.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ShareParams {
    pub id: u64,
    pub namespace_id: String,
    #[serde(flatten)]
    pub common: CommonRequestFields,
}

/// Tighten visibility back to private/non-shared.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct UnshareParams {
    pub id: u64,
    pub namespace: String,
    #[serde(flatten)]
    pub common: CommonRequestFields,
}

// ---------------------------------------------------------------------------
// Operator surfaces
// ---------------------------------------------------------------------------

/// Return the bounded operator summary.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct StatsParams {
    pub namespace: String,
    #[serde(flatten)]
    pub common: CommonRequestFields,
}

/// Return the BrainHealthReport.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct HealthParams {
    pub namespace: String,
    #[serde(flatten)]
    pub common: CommonRequestFields,
}

/// Diagnose corruption, stale derived state, and degraded-serving posture.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DoctorParams {
    #[serde(flatten)]
    pub common: CommonRequestFields,
}

/// Externalize memories within the caller's allowed scope.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ExportParams {
    pub namespace: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub format: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub include_cold: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub include_archive: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub kind: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub min_strength: Option<u16>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub at_snapshot: Option<String>,
    #[serde(flatten)]
    pub common: CommonRequestFields,
}

/// Import externalized memories through normal governed ingest.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ImportParams {
    pub namespace: String,
    pub payload_ref: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub format: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub dry_run: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub merge: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub kind: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source: Option<String>,
    #[serde(flatten)]
    pub common: CommonRequestFields,
}

// ---------------------------------------------------------------------------
// Feature-specific tools
// ---------------------------------------------------------------------------

/// Trigger a later-stage offline synthesis cycle.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DreamParams {
    pub namespace: String,
    #[serde(flatten)]
    pub common: CommonRequestFields,
}

/// Returns the inspectable belief chain for a topic or query.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct BeliefHistoryParams {
    pub query: String,
    pub namespace: String,
    #[serde(flatten)]
    pub common: CommonRequestFields,
}

/// Ranked, deduplicated memory list that fits within token budget.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ContextBudgetParams {
    pub token_budget: usize,
    pub namespace: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub current_context: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub working_memory_ids: Option<Vec<u64>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub format: Option<String>,
    #[serde(flatten)]
    pub common: CommonRequestFields,
}

/// Temporal navigation surface over landmark-defined eras.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TimelineParams {
    pub namespace: String,
    #[serde(flatten)]
    pub common: CommonRequestFields,
}

/// Segment content into memories via topic boundary detection.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ObserveParams {
    pub content: String,
    pub namespace: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub context: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub chunk_size: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source_label: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub topic_threshold: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub min_chunk_size: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub dry_run: Option<bool>,
    #[serde(flatten)]
    pub common: CommonRequestFields,
}

/// List memories with high uncertainty scores.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct UncertainParams {
    pub namespace: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub top_k: Option<usize>,
    #[serde(flatten)]
    pub common: CommonRequestFields,
}

/// List or extract procedural memory skills.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SkillsParams {
    pub namespace: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub extract: Option<bool>,
    #[serde(flatten)]
    pub common: CommonRequestFields,
}

/// List, promote, or roll back authoritative procedural-store entries.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ProceduresParams {
    pub namespace: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub promote: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rollback: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub note: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub approved_by: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub public: Option<bool>,
    #[serde(flatten)]
    pub common: CommonRequestFields,
}

/// Trace causal chain to root evidence.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct WhyParams {
    pub id: u64,
    pub namespace: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub depth: Option<usize>,
    #[serde(flatten)]
    pub common: CommonRequestFields,
}

/// Cascade confidence penalty from invalidated root.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct InvalidateParams {
    pub id: u64,
    pub namespace: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub dry_run: Option<bool>,
    #[serde(flatten)]
    pub common: CommonRequestFields,
}

/// Create a named historical inspection anchor.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SnapshotParams {
    pub name: String,
    pub namespace: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub note: Option<String>,
    #[serde(flatten)]
    pub common: CommonRequestFields,
}

/// List existing named snapshots.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ListSnapshotsParams {
    pub namespace: String,
    #[serde(flatten)]
    pub common: CommonRequestFields,
}

/// Return BrainDiff between time boundaries.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DiffParams {
    pub since: u64,
    pub namespace: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub until: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub top_n: Option<usize>,
    #[serde(flatten)]
    pub common: CommonRequestFields,
}

/// Fork a namespace for experimental or branch work.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ForkParams {
    pub name: String,
    pub namespace: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub parent_namespace: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub inherit: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub note: Option<String>,
    #[serde(flatten)]
    pub common: CommonRequestFields,
}

/// Merge a fork back into the target namespace.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct MergeForkParams {
    pub fork_name: String,
    pub target_namespace: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub conflict_strategy: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub dry_run: Option<bool>,
    #[serde(flatten)]
    pub common: CommonRequestFields,
}

/// Compress episodic memories into schema artifacts.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CompressParams {
    pub namespace: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub dry_run: Option<bool>,
    #[serde(flatten)]
    pub common: CommonRequestFields,
}

/// Preview the highest-confidence schema artifacts without applying compression.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SchemasParams {
    pub namespace: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub top_n: Option<usize>,
    #[serde(flatten)]
    pub common: CommonRequestFields,
}

/// List hot zones (frequently accessed memory neighborhoods).
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct HotPathsParams {
    pub namespace: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub top_n: Option<usize>,
    #[serde(flatten)]
    pub common: CommonRequestFields,
}

/// List dead zones (stale, rarely accessed memory neighborhoods).
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DeadZonesParams {
    pub namespace: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub min_age_ticks: Option<u64>,
    #[serde(flatten)]
    pub common: CommonRequestFields,
}

/// Read-only forensic surface for memory and operation history.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AuditParams {
    pub namespace: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub memory_id: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub since_tick: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub op: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub limit: Option<usize>,
    #[serde(flatten)]
    pub common: CommonRequestFields,
}

/// Introspection surface over emotional trajectory rows.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct MoodHistoryParams {
    pub namespace: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub since_tick: Option<u64>,
    #[serde(flatten)]
    pub common: CommonRequestFields,
}

/// Find memories similar to a provided example memory.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct QueryByExampleParams {
    pub example_id: u64,
    pub namespace: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub result_budget: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub token_budget: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub time_budget_ms: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub effort: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub explain: Option<bool>,
    #[serde(flatten)]
    pub common: CommonRequestFields,
}

// ---------------------------------------------------------------------------
// Goal management params
// ---------------------------------------------------------------------------

/// Query the current goal-stack state for a task.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct GoalStateParams {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub task_id: Option<String>,
    #[serde(flatten)]
    pub common: CommonRequestFields,
}

/// Pause a task goal and persist a resumability checkpoint.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct GoalPauseParams {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub task_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub note: Option<String>,
    #[serde(flatten)]
    pub common: CommonRequestFields,
}

/// Pin one evidence handle into the visible goal blackboard projection.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct GoalPinParams {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub task_id: Option<String>,
    pub memory_id: u64,
    #[serde(flatten)]
    pub common: CommonRequestFields,
}

/// Dismiss one evidence handle from the visible goal blackboard projection.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct GoalDismissParams {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub task_id: Option<String>,
    pub memory_id: u64,
    #[serde(flatten)]
    pub common: CommonRequestFields,
}

/// Emit a derived blackboard snapshot artifact for one task.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct GoalSnapshotParams {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub task_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub note: Option<String>,
    #[serde(flatten)]
    pub common: CommonRequestFields,
}

/// Resume a paused task goal from its latest valid checkpoint.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct GoalResumeParams {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub task_id: Option<String>,
    #[serde(flatten)]
    pub common: CommonRequestFields,
}

/// Abandon a task goal, preserving checkpoint evidence for later inspection.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct GoalAbandonParams {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub task_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reason: Option<String>,
    #[serde(flatten)]
    pub common: CommonRequestFields,
}

/// Canonical MCP Response Envelope.
///
/// Retrieval-facing tools should prefer the explicit `retrieval` payload so MCP preserves the
/// same stable envelope family as CLI JSON and daemon/JSON-RPC instead of hiding it behind an
/// untyped blob.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct McpResponse {
    pub status: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub retrieval: Option<McpRetrievalPayload>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub payload: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<McpError>,
}

impl McpResponse {
    pub fn success(payload: serde_json::Value) -> Self {
        Self {
            status: "ok".to_string(),
            retrieval: None,
            payload: Some(payload),
            error: None,
        }
    }

    pub fn retrieval_success(retrieval: McpRetrievalPayload) -> Self {
        Self {
            status: "ok".to_string(),
            retrieval: Some(retrieval),
            payload: None,
            error: None,
        }
    }

    pub fn failure(error: McpError) -> Self {
        Self {
            status: "error".to_string(),
            retrieval: None,
            payload: None,
            error: Some(error),
        }
    }
}

/// MCP retrieval payload preserving the canonical retrieval-result envelope families.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct McpRetrievalPayload {
    pub request_id: RequestId,
    pub namespace: NamespaceId,
    pub outcome_class: OutcomeClass,
    pub partial_success: bool,
    pub explain_trace: McpExplainTrace,
    pub result: RetrievalResultSet,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct McpExplainTrace {
    pub route_summary: serde_json::Value,
    pub trace_stages: Vec<String>,
    pub result_reasons: serde_json::Value,
    pub omitted_summary: serde_json::Value,
    pub cache_metrics: serde_json::Value,
    pub policy_summary: serde_json::Value,
    pub provenance_summary: serde_json::Value,
    pub freshness_markers: serde_json::Value,
    pub conflict_markers: serde_json::Value,
    pub uncertainty_markers: serde_json::Value,
}

impl McpRetrievalPayload {
    pub fn from_result(
        request_id: RequestId,
        namespace: NamespaceId,
        partial_success: bool,
        result: RetrievalResultSet,
    ) -> Result<Self, serde_json::Error> {
        let partial_success = partial_success
            || matches!(result.outcome_class, OutcomeClass::Partial)
            || result.truncated;
        let outcome_class = result.outcome_class;
        let (route_summary, trace_stages) = result.explain_route();
        let result_reasons = result.explain_result_reasons();
        let (policy_summary, provenance_summary) = result.explain_policy_and_provenance();
        let (freshness_markers, conflict_markers, uncertainty_markers) = result.explain_markers();
        let explain_trace = McpExplainTrace {
            route_summary: serde_json::to_value(&route_summary)?,
            trace_stages: trace_stages
                .into_iter()
                .map(|stage| stage.as_str().to_string())
                .collect(),
            result_reasons: serde_json::to_value(&result_reasons)?,
            omitted_summary: serde_json::to_value(&result.omitted_summary)?,
            cache_metrics: serde_json::to_value(&result)
                .ok()
                .and_then(|value| value.get("cache_metrics").cloned())
                .unwrap_or_else(|| {
                    serde_json::to_value(CacheMetricsSummary::from_cache_traces(Vec::new(), false))
                        .expect("cache metrics fallback should serialize")
                }),
            policy_summary: serde_json::to_value(&policy_summary)?,
            provenance_summary: serde_json::to_value(&provenance_summary)?,
            freshness_markers: serde_json::to_value(&freshness_markers)?,
            conflict_markers: serde_json::to_value(&conflict_markers)?,
            uncertainty_markers: serde_json::to_value(&uncertainty_markers)?,
        };

        Ok(Self {
            request_id,
            namespace,
            outcome_class,
            partial_success,
            explain_trace,
            result,
        })
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct McpError {
    pub code: String,
    pub message: String,
    pub is_policy_denial: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct McpAuditView {
    pub event_kind: String,
    pub actor_source: String,
    pub request_id: String,
    pub effective_namespace: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source_namespace: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub target_namespace: Option<String>,
    pub policy_family: String,
    pub outcome_class: String,
    pub blocked_stage: String,
    pub redaction_summary: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub related_run: Option<String>,
    pub redacted: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct McpAuditRow {
    pub sequence: u64,
    pub op: String,
    pub category: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub memory_id: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub session_id: Option<u64>,
    pub tick: Option<u64>,
    pub before_strength: Option<u16>,
    pub after_strength: Option<u16>,
    pub before_confidence: Option<u16>,
    pub after_confidence: Option<u16>,
    pub triggered_by: String,
    pub note: String,
    pub namespace: String,
    pub redaction: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub request_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub related_snapshot: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub related_run: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct McpAuditPayload {
    pub request_id: RequestId,
    pub namespace: NamespaceId,
    pub total_matches: usize,
    pub returned_rows: usize,
    pub truncated: bool,
    pub entries: Vec<McpAuditRow>,
}

impl From<AuditLogEntry> for McpAuditRow {
    fn from(entry: AuditLogEntry) -> Self {
        Self {
            sequence: entry.sequence,
            op: entry.kind.as_str().to_string(),
            category: entry.category.as_str().to_string(),
            memory_id: entry.memory_id.map(|id| id.0),
            session_id: entry.session_id.map(|id| id.0),
            tick: entry.tick,
            before_strength: entry.before_strength,
            after_strength: entry.after_strength,
            before_confidence: entry.before_confidence,
            after_confidence: entry.after_confidence,
            triggered_by: entry.actor_source.to_string(),
            note: entry.detail,
            namespace: entry.namespace.as_str().to_string(),
            redaction: entry.redacted,
            request_id: entry.request_id,
            related_snapshot: entry.related_snapshot,
            related_run: entry.related_run,
        }
    }
}

#[derive(Debug, Clone, Serialize)]
#[serde(deny_unknown_fields)]
pub struct McpSharePayload {
    pub request_id: RequestId,
    pub namespace: NamespaceId,
    pub outcome_class: OutcomeClass,
    pub result: serde_json::Value,
    pub policy_filters_applied: Vec<PolicyFilterSummary>,
    pub policy_summary: TracePolicySummary,
    pub audit: McpAuditView,
}

/// Typed inspect payload that reuses the canonical explain families instead of inventing an
/// inspect-only wrapper-local schema.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct McpInspectPayload {
    pub request_id: RequestId,
    pub namespace: NamespaceId,
    pub memory_id: u64,
    pub tier: String,
    pub lineage: serde_json::Value,
    pub policy_flags: serde_json::Value,
    pub lifecycle_state: serde_json::Value,
    pub index_presence: serde_json::Value,
    pub graph_neighborhood_summary: serde_json::Value,
    pub decay_retention: serde_json::Value,
    pub explain_trace: McpInspectExplainTrace,
}

impl McpInspectPayload {
    pub fn from_result(
        request_id: RequestId,
        namespace: NamespaceId,
        memory_id: u64,
        result: &RetrievalResultSet,
    ) -> Result<Self, serde_json::Error> {
        let (policy_summary, provenance_summary) = result.explain_policy_and_provenance();
        let (freshness_markers, conflict_markers, _) = result.explain_markers();
        let (_, trace_stages) = result.explain_route();
        let trace_stages = trace_stages
            .into_iter()
            .map(|stage| stage.as_str().to_string())
            .collect::<Vec<_>>();

        Ok(Self {
            request_id,
            namespace,
            memory_id,
            tier: result
                .explain
                .tiers_consulted
                .first()
                .cloned()
                .unwrap_or_else(|| "unavailable".to_string()),
            lineage: serde_json::to_value(&provenance_summary)?,
            policy_flags: serde_json::to_value(&policy_summary)?,
            lifecycle_state: serde_json::json!({
                "outcome_class": result.outcome_class,
                "truncated": result.truncated,
                "degraded_summary": result.packaging_metadata.degraded_summary,
            }),
            index_presence: serde_json::json!({
                "tiers_consulted": result.explain.tiers_consulted,
                "graph_assistance": result.packaging_metadata.graph_assistance,
            }),
            graph_neighborhood_summary: serde_json::json!({
                "graph_seed": result.provenance_summary.graph_seed,
                "relation_to_seed": result.provenance_summary.relation_to_seed,
                "graph_assistance": result.packaging_metadata.graph_assistance,
            }),
            decay_retention: serde_json::to_value(&freshness_markers)?,
            explain_trace: McpInspectExplainTrace {
                policy_summary: serde_json::to_value(&policy_summary)?,
                provenance_summary: serde_json::to_value(&provenance_summary)?,
                freshness_markers: serde_json::to_value(&freshness_markers)?,
                conflict_markers: serde_json::to_value(&conflict_markers)?,
                passive_observation: None,
                trace_stages,
            },
        })
    }
}

/// Embedded explain families for inspect responses.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct McpInspectExplainTrace {
    pub policy_summary: serde_json::Value,
    pub provenance_summary: serde_json::Value,
    pub freshness_markers: serde_json::Value,
    pub conflict_markers: serde_json::Value,
    pub passive_observation: Option<serde_json::Value>,
    pub trace_stages: Vec<String>,
}

/// Typed MCP resource descriptor for explicit resource listings and later integration tests.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct McpResource {
    pub uri: String,
    pub name: String,
    pub mime_type: String,
    pub resource_kind: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub uri_template: Option<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub examples: Vec<String>,
}

/// Typed MCP resource-list payload.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct McpResourceListing {
    pub request_id: RequestId,
    pub namespace: NamespaceId,
    pub resources: Vec<McpResource>,
}

/// Typed payload for reading one explicit MCP resource.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct McpResourceReadPayload {
    pub request_id: RequestId,
    pub namespace: NamespaceId,
    pub uri: String,
    pub mime_type: String,
    pub resource_kind: String,
    pub bounded: bool,
    pub payload: serde_json::Value,
}

/// Typed streaming surface descriptor for explicit daemon notification support.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct McpStream {
    pub name: String,
    pub method: String,
    pub delivery: String,
    pub description: String,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub example_subscriptions: Vec<String>,
}

/// Typed listing of supported daemon streaming surfaces.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct McpStreamListing {
    pub request_id: RequestId,
    pub namespace: NamespaceId,
    pub streams: Vec<McpStream>,
}

#[cfg(test)]
mod tests {
    use super::{
        CommonRequestFields, CompressParams, ContextBudgetParams, EncodeParams, ExplainParams,
        InspectParams, McpAuditView, McpError, McpInspectExplainTrace, McpInspectPayload,
        McpRequest, McpResource, McpResourceListing, McpResourceReadPayload, McpResponse,
        McpRetrievalPayload, McpSharePayload, McpStream, McpStreamListing, ObserveParams,
        PolicyContextHint, ProceduresParams, RecallParams, SchemasParams,
    };
    use membrain_core::api::{
        FieldPresence, NamespaceId, PassiveObservationInspectSummary, PolicyFilterSummary,
        RequestId, TracePolicySummary,
    };
    use membrain_core::engine::recall::RecallPlanKind;
    use membrain_core::engine::result::RetrievalExplain;
    use membrain_core::engine::result::RetrievalResultSet;
    use membrain_core::engine::result::{DualOutputMode, PackagingMetadata};
    use membrain_core::observability::OutcomeClass;

    fn sample_common() -> CommonRequestFields {
        CommonRequestFields {
            request_id: Some("test-req-1".to_string()),
            workspace_id: Some("ws-7".to_string()),
            agent_id: Some("agent-3".to_string()),
            session_id: Some("session-9".to_string()),
            task_id: Some("task-2".to_string()),
            time_budget_ms: Some(75),
            policy_context: Some(PolicyContextHint {
                include_public: true,
                sharing_visibility: Some("public".to_string()),
                caller_identity_bound: true,
                workspace_acl_allowed: true,
                agent_acl_allowed: true,
                session_visibility_allowed: true,
                legal_hold: false,
            }),
        }
    }

    fn sample_result_set() -> RetrievalResultSet {
        RetrievalResultSet {
            outcome_class: OutcomeClass::Accepted,
            evidence_pack: Vec::new(),
            action_pack: Some(vec![membrain_core::engine::result::ActionArtifact {
                action_type: "review_observation".to_string(),
                suggestion: "Inspect the observation evidence before acting".to_string(),
                supporting_evidence: vec![membrain_core::types::MemoryId(7)],
                confidence_score: 720,
                uncertainty_markers: vec!["low_uncertainty".to_string()],
                policy_caveats: Vec::new(),
                freshness_caveats: Vec::new(),
            }]),
            deferred_payloads: Vec::new(),
            explain: RetrievalExplain {
                recall_plan: RecallPlanKind::ExactIdTier1,
                route_reason: "tier1 exact route".to_string(),
                tiers_consulted: vec!["tier1_exact".to_string()],
                trace_stages: Vec::new(),
                tier1_answered_directly: true,
                candidate_budget: 8,
                time_consumed_ms: Some(1),
                ranking_profile: "balanced".to_string(),
                contradictions_found: 0,
                historical_context: None,
                query_by_example: None,
                result_reasons: Vec::new(),
            },
            policy_summary: membrain_core::engine::result::PolicySummary {
                namespace_applied: NamespaceId::new("mcp.team").unwrap(),
                outcome_class: OutcomeClass::Accepted,
                redactions_applied: false,
                restrictions_active: Vec::new(),
                filters: Vec::new(),
            },
            provenance_summary: membrain_core::engine::result::ProvenanceSummary {
                source_kind: "retrieval_pipeline".to_string(),
                source_reference: "result_set".to_string(),
                source_agent: "mcp".to_string(),
                original_namespace: NamespaceId::new("mcp.team").unwrap(),
                derived_from: None,
                lineage_ancestors: Vec::new(),
                relation_to_seed: None,
                graph_seed: None,
            },
            omitted_summary: membrain_core::engine::result::OmissionSummary {
                policy_redacted: 0,
                threshold_dropped: 0,
                dedup_dropped: 0,
                budget_capped: 0,
                duplicate_collapsed: 0,
                low_confidence_suppressed: 0,
                stale_bypassed: 0,
                confidence_filtered: 0,
            },
            freshness_markers: membrain_core::engine::result::FreshnessMarkers {
                oldest_item_days: 0,
                newest_item_days: 0,
                volatile_items_included: false,
                stale_warning: false,
                lease_sensitive: false,
                recheck_required: false,
                as_of_tick: None,
            },
            packaging_metadata: PackagingMetadata {
                result_budget: 8,
                token_budget: Some(256),
                graph_assistance: "none".to_string(),
                degraded_summary: None,
                packaging_mode: "evidence_plus_action".to_string(),
                rerank_metadata: None,
            },
            output_mode: DualOutputMode::Balanced,
            truncated: false,
            total_candidates: 0,
        }
    }

    #[test]
    fn retrieval_payload_preserves_canonical_result_families() {
        let payload = McpRetrievalPayload::from_result(
            RequestId::new("req-1").unwrap(),
            NamespaceId::new("mcp.team").unwrap(),
            false,
            sample_result_set(),
        )
        .unwrap();

        let json = serde_json::to_value(&payload).unwrap();
        assert_eq!(json["request_id"], "req-1");
        assert_eq!(json["namespace"], "mcp.team");
        assert_eq!(json["outcome_class"], "accepted");
        assert!(json["result"].get("evidence_pack").is_some());
        assert!(json["result"].get("action_pack").is_some());
        assert!(json["result"].get("deferred_payloads").is_some());
        assert_eq!(json["result"]["output_mode"], "balanced");
        assert_eq!(
            json["result"]["action_pack"][0]["action_type"],
            "review_observation"
        );
        assert_eq!(
            json["result"]["action_pack"][0]["supporting_evidence"][0],
            7
        );
        assert!(json["result"].get("omitted_summary").is_some());
        assert!(json["result"].get("policy_summary").is_some());
        assert!(json["result"].get("provenance_summary").is_some());
        assert!(json["result"].get("freshness_markers").is_some());
        assert!(json["result"].get("packaging_metadata").is_some());
        assert!(json["result"].get("explain").is_some());
        assert_eq!(
            json["explain_trace"]["route_summary"]["route_family"],
            "exact_id_tier1"
        );
        assert!(json["explain_trace"].get("omitted_summary").is_some());
        assert!(json["explain_trace"].get("cache_metrics").is_some());
        assert!(json["explain_trace"].get("policy_summary").is_some());
        assert!(json["explain_trace"].get("provenance_summary").is_some());
        assert!(json["explain_trace"].get("freshness_markers").is_some());
        assert!(json["explain_trace"].get("conflict_markers").is_some());
        assert!(json["explain_trace"].get("uncertainty_markers").is_some());
        assert_eq!(
            json["explain_trace"]["trace_stages"],
            serde_json::json!(["policy_gate", "packaging"])
        );
    }

    #[test]
    fn retrieval_payload_marks_partial_success_without_mutating_result() {
        let payload = McpRetrievalPayload::from_result(
            RequestId::new("req-2").unwrap(),
            NamespaceId::new("mcp.team").unwrap(),
            true,
            sample_result_set(),
        )
        .unwrap();

        assert_eq!(payload.outcome_class, OutcomeClass::Accepted);
        assert_eq!(payload.result.outcome_class, OutcomeClass::Accepted);
        assert_eq!(payload.explain_trace.omitted_summary["budget_capped"], 0);
        assert_eq!(payload.explain_trace.cache_metrics["cache_hit_count"], 0);
        assert_eq!(
            payload.explain_trace.policy_summary["effective_namespace"],
            "mcp.team"
        );
        assert_eq!(
            payload.explain_trace.provenance_summary["source_reference"],
            "result_set"
        );
        assert!(payload.partial_success);
    }

    #[test]
    fn retrieval_payload_derives_partial_success_from_core_result() {
        let mut result = sample_result_set();
        result.outcome_class = OutcomeClass::Partial;

        let payload = McpRetrievalPayload::from_result(
            RequestId::new("req-2b").unwrap(),
            NamespaceId::new("mcp.team").unwrap(),
            false,
            result,
        )
        .unwrap();

        assert_eq!(payload.outcome_class, OutcomeClass::Partial);
        assert_eq!(payload.result.outcome_class, OutcomeClass::Partial);
        assert!(payload.partial_success);
    }

    #[test]
    fn retrieval_success_uses_typed_transport_slot() {
        let response = McpResponse::retrieval_success(
            McpRetrievalPayload::from_result(
                RequestId::new("req-3").unwrap(),
                NamespaceId::new("mcp.team").unwrap(),
                false,
                sample_result_set(),
            )
            .unwrap(),
        );

        let json = serde_json::to_value(&response).unwrap();
        assert_eq!(json["status"], "ok");
        assert!(json.get("retrieval").is_some());
        assert!(json["retrieval"].get("explain_trace").is_some());
        assert!(json.get("payload").is_none());
        assert!(json.get("error").is_none());
    }

    #[test]
    fn retrieval_payload_derives_partial_success_from_truncation_without_inventing_outcome() {
        let mut result = sample_result_set();
        result.truncated = true;

        let payload = McpRetrievalPayload::from_result(
            RequestId::new("req-4").unwrap(),
            NamespaceId::new("mcp.team").unwrap(),
            false,
            result,
        )
        .unwrap();

        assert_eq!(payload.outcome_class, OutcomeClass::Accepted);
        assert_eq!(payload.result.outcome_class, OutcomeClass::Accepted);
        assert!(payload.partial_success);
    }

    #[test]
    fn explain_request_round_trips_optional_limit() {
        let request = McpRequest::Explain(ExplainParams {
            query: "session:7".to_string(),
            namespace: "team.alpha".to_string(),
            limit: Some(2),
            common: sample_common(),
        });

        let json = serde_json::to_value(&request).unwrap();
        assert_eq!(json["method"], "explain");
        assert_eq!(json["params"]["query"], "session:7");
        assert_eq!(json["params"]["namespace"], "team.alpha");
        assert_eq!(json["params"]["limit"], 2);

        let decoded: McpRequest = serde_json::from_value(json).unwrap();
        match decoded {
            McpRequest::Explain(params) => assert_eq!(params.limit, Some(2)),
            _ => std::process::abort(),
        }
    }

    #[test]
    fn explain_request_omits_limit_when_not_provided() {
        let request = McpRequest::Explain(ExplainParams {
            query: "session:7".to_string(),
            namespace: "team.alpha".to_string(),
            limit: None,
            common: sample_common(),
        });

        let json = serde_json::to_value(&request).unwrap();
        assert!(json["params"].get("limit").is_none());
    }

    #[test]
    fn inspect_payload_reuses_canonical_explain_families() {
        let payload = McpInspectPayload {
            request_id: RequestId::new("inspect-req-1").unwrap(),
            namespace: NamespaceId::new("team.alpha").unwrap(),
            memory_id: 42,
            tier: "tier1".to_string(),
            lineage: serde_json::json!({"ancestors": [7, 9]}),
            policy_flags: serde_json::json!({"sharing_scope": "private"}),
            lifecycle_state: serde_json::json!({"state": "active"}),
            index_presence: serde_json::json!({"tier1": true, "tier2": false}),
            graph_neighborhood_summary: serde_json::json!({"neighbors": 3}),
            decay_retention: serde_json::json!({"pinned": false, "strength": 400}),
            explain_trace: McpInspectExplainTrace {
                policy_summary: serde_json::json!({"effective_namespace": "team.alpha"}),
                provenance_summary: serde_json::json!({"source_reference": "memory:42"}),
                freshness_markers: serde_json::json!({"stale_warning": false}),
                conflict_markers: serde_json::json!({"conflicted": false}),
                passive_observation: Some(serde_json::json!({"source": "user_input"})),
                trace_stages: vec!["inspect_lookup".to_string(), "policy_gate".to_string()],
            },
        };

        let json = serde_json::to_value(&payload).unwrap();
        assert_eq!(json["request_id"], "inspect-req-1");
        assert_eq!(json["namespace"], "team.alpha");
        assert_eq!(json["memory_id"], 42);
        assert_eq!(json["tier"], "tier1");
        assert_eq!(
            json["explain_trace"]["policy_summary"]["effective_namespace"],
            "team.alpha"
        );
        assert_eq!(
            json["explain_trace"]["provenance_summary"]["source_reference"],
            "memory:42"
        );
        assert_eq!(
            json["explain_trace"]["trace_stages"],
            serde_json::json!(["inspect_lookup", "policy_gate"])
        );
        assert!(json["explain_trace"].get("passive_observation").is_some());
    }

    #[test]
    fn resource_listing_round_trips_typed_resources() {
        let listing = McpResourceListing {
            request_id: RequestId::new("resource-req-1").unwrap(),
            namespace: NamespaceId::new("team.alpha").unwrap(),
            resources: vec![
                McpResource {
                    uri: "membrain://team.alpha/memories/42".to_string(),
                    name: "memory-42".to_string(),
                    mime_type: "application/json".to_string(),
                    resource_kind: "inspect_payload".to_string(),
                    description: Some("Typed inspect payload for memory 42".to_string()),
                    uri_template: Some("membrain://{namespace}/memories/{memory_id}".to_string()),
                    examples: vec!["membrain://team.alpha/memories/42".to_string()],
                },
                McpResource {
                    uri: "membrain://team.alpha/snapshots/pre-migration".to_string(),
                    name: "snapshot-pre-migration".to_string(),
                    mime_type: "application/json".to_string(),
                    resource_kind: "snapshot_view".to_string(),
                    description: None,
                    uri_template: Some(
                        "membrain://{namespace}/snapshots/{snapshot_name}".to_string(),
                    ),
                    examples: vec!["membrain://team.alpha/snapshots/pre-migration".to_string()],
                },
            ],
        };

        let json = serde_json::to_value(&listing).unwrap();
        assert_eq!(json["request_id"], "resource-req-1");
        assert_eq!(json["namespace"], "team.alpha");
        assert_eq!(
            json["resources"][0]["uri"],
            "membrain://team.alpha/memories/42"
        );
        assert_eq!(json["resources"][0]["mime_type"], "application/json");
        assert_eq!(json["resources"][0]["resource_kind"], "inspect_payload");
        assert_eq!(
            json["resources"][0]["uri_template"],
            "membrain://{namespace}/memories/{memory_id}"
        );
        assert_eq!(
            json["resources"][0]["examples"][0],
            "membrain://team.alpha/memories/42"
        );
        assert_eq!(json["resources"][1]["name"], "snapshot-pre-migration");
        assert_eq!(json["resources"][1]["resource_kind"], "snapshot_view");
        assert!(json["resources"][1].get("description").is_none());

        let decoded: McpResourceListing = serde_json::from_value(json).unwrap();
        assert_eq!(decoded.resources.len(), 2);
        assert_eq!(decoded.resources[0].name, "memory-42");
        assert!(decoded.resources[1].description.is_none());
    }

    #[test]
    fn inspect_payload_can_be_derived_from_canonical_retrieval_result() {
        let payload = McpInspectPayload::from_result(
            RequestId::new("inspect-derived-1").unwrap(),
            NamespaceId::new("team.alpha").unwrap(),
            42,
            &sample_result_set(),
        )
        .unwrap();

        assert_eq!(payload.memory_id, 42);
        assert_eq!(payload.namespace, NamespaceId::new("team.alpha").unwrap());
        assert_eq!(payload.tier, "tier1_exact");
        assert_eq!(
            payload.explain_trace.policy_summary["effective_namespace"],
            "mcp.team"
        );
        assert_eq!(
            payload.index_presence["graph_assistance"],
            serde_json::json!("none")
        );
        assert_eq!(
            payload.lifecycle_state["degraded_summary"],
            serde_json::Value::Null
        );
        assert_eq!(
            payload.explain_trace.trace_stages,
            vec!["policy_gate".to_string(), "packaging".to_string()]
        );
    }

    #[test]
    fn resource_read_payload_and_stream_listing_round_trip() {
        let read_payload = McpResourceReadPayload {
            request_id: RequestId::new("resource-read-1").unwrap(),
            namespace: NamespaceId::new("daemon.runtime").unwrap(),
            uri: "membrain://daemon/runtime/status".to_string(),
            mime_type: "application/json".to_string(),
            resource_kind: "runtime_status".to_string(),
            bounded: true,
            payload: serde_json::json!({"posture": "full"}),
        };
        let read_json = serde_json::to_value(&read_payload).unwrap();
        assert_eq!(read_json["request_id"], "resource-read-1");
        assert_eq!(read_json["namespace"], "daemon.runtime");
        assert_eq!(read_json["uri"], "membrain://daemon/runtime/status");
        assert_eq!(read_json["resource_kind"], "runtime_status");
        assert_eq!(read_json["bounded"], true);
        assert_eq!(read_json["payload"]["posture"], "full");
        let decoded_read: McpResourceReadPayload = serde_json::from_value(read_json).unwrap();
        assert!(decoded_read.bounded);

        let streams = McpStreamListing {
            request_id: RequestId::new("stream-req-1").unwrap(),
            namespace: NamespaceId::new("daemon.runtime").unwrap(),
            streams: vec![McpStream {
                name: "maintenance-status".to_string(),
                method: "maintenance.status".to_string(),
                delivery: "jsonrpc_notification".to_string(),
                description: "Background maintenance acceptance and posture updates".to_string(),
                example_subscriptions: vec!["maintenance.status".to_string()],
            }],
        };
        let streams_json = serde_json::to_value(&streams).unwrap();
        assert_eq!(streams_json["namespace"], "daemon.runtime");
        assert_eq!(streams_json["streams"][0]["method"], "maintenance.status");
        assert_eq!(
            streams_json["streams"][0]["delivery"],
            "jsonrpc_notification"
        );
        assert_eq!(
            streams_json["streams"][0]["example_subscriptions"][0],
            "maintenance.status"
        );
        let decoded_streams: McpStreamListing = serde_json::from_value(streams_json).unwrap();
        assert_eq!(decoded_streams.streams.len(), 1);
        assert_eq!(decoded_streams.streams[0].name, "maintenance-status");
    }

    #[test]
    fn mcp_param_structs_reject_unknown_fields() {
        let encode_error = serde_json::from_value::<EncodeParams>(serde_json::json!({
            "content": "hello",
            "namespace": "team.alpha",
            "unexpected": true
        }))
        .unwrap_err();
        assert!(encode_error
            .to_string()
            .contains("unknown field `unexpected`"));

        let recall_error = serde_json::from_value::<RecallParams>(serde_json::json!({
            "query_text": "session:7",
            "namespace": "team.alpha",
            "result_budget": 3,
            "unexpected": true
        }))
        .unwrap_err();
        assert!(recall_error
            .to_string()
            .contains("unknown field `unexpected`"));

        let inspect_error = serde_json::from_value::<InspectParams>(serde_json::json!({
            "id": 7,
            "namespace": "team.alpha",
            "unexpected": true
        }))
        .unwrap_err();
        assert!(inspect_error
            .to_string()
            .contains("unknown field `unexpected`"));

        let explain_error = serde_json::from_value::<ExplainParams>(serde_json::json!({
            "query": "session:7",
            "namespace": "team.alpha",
            "limit": 2,
            "unexpected": true
        }))
        .unwrap_err();
        assert!(explain_error
            .to_string()
            .contains("unknown field `unexpected`"));
    }

    #[test]
    fn failure_response_preserves_policy_denial_metadata() {
        let response = McpResponse::failure(McpError {
            code: "policy_denied".to_string(),
            message: "namespace isolation prevents export".to_string(),
            is_policy_denial: true,
        });

        let json = serde_json::to_value(&response).unwrap();
        assert_eq!(json["status"], "error");
        assert!(json.get("retrieval").is_none());
        assert!(json.get("payload").is_none());
        assert_eq!(json["error"]["code"], "policy_denied");
        assert_eq!(
            json["error"]["message"],
            "namespace isolation prevents export"
        );
        assert_eq!(json["error"]["is_policy_denial"], true);
    }

    #[test]
    fn share_payload_can_preserve_required_policy_and_audit_fields() {
        let payload = McpSharePayload {
            request_id: RequestId::new("req-share-42").unwrap(),
            namespace: NamespaceId::new("team.beta").unwrap(),
            outcome_class: OutcomeClass::Accepted,
            result: serde_json::json!({
                "status": "accepted",
                "id": 42,
                "namespace": "team.beta",
                "visibility": "shared"
            }),
            policy_filters_applied: vec![PolicyFilterSummary::new(
                "team.beta",
                "visibility_sharing",
                OutcomeClass::Accepted,
                "policy_gate",
                FieldPresence::Present("shared".to_string()),
                FieldPresence::Absent,
                Vec::new(),
            )],
            policy_summary: TracePolicySummary {
                effective_namespace: "team.beta".to_string(),
                policy_family: "visibility_sharing",
                outcome_class: OutcomeClass::Accepted,
                blocked_stage: "policy_gate",
                redaction_fields: Vec::new(),
                retention_state: FieldPresence::Absent,
                sharing_scope: FieldPresence::Present("shared"),
                filters: vec![PolicyFilterSummary::new(
                    "team.beta",
                    "visibility_sharing",
                    OutcomeClass::Accepted,
                    "policy_gate",
                    FieldPresence::Present("shared".to_string()),
                    FieldPresence::Absent,
                    Vec::new(),
                )],
            },
            audit: McpAuditView {
                event_kind: "approved_sharing".to_string(),
                actor_source: "mcp_share".to_string(),
                request_id: "req-share-42".to_string(),
                effective_namespace: "team.beta".to_string(),
                source_namespace: Some("team.alpha".to_string()),
                target_namespace: Some("team.beta".to_string()),
                policy_family: "visibility_sharing".to_string(),
                outcome_class: "accepted".to_string(),
                blocked_stage: "policy_gate".to_string(),
                redaction_summary: Vec::new(),
                related_run: Some("share-run-42".to_string()),
                redacted: false,
            },
        };

        let response = McpResponse::success(serde_json::to_value(&payload).unwrap());
        let json = serde_json::to_value(&response).unwrap();
        assert_eq!(json["status"], "ok");
        assert_eq!(json["payload"]["request_id"], "req-share-42");
        assert_eq!(json["payload"]["namespace"], "team.beta");
        assert_eq!(json["payload"]["outcome_class"], "accepted");
        assert_eq!(
            json["payload"]["policy_summary"]["policy_family"],
            "visibility_sharing"
        );
        assert_eq!(json["payload"]["audit"]["event_kind"], "approved_sharing");
        assert_eq!(json["payload"]["audit"]["effective_namespace"], "team.beta");
        assert_eq!(json["payload"]["audit"]["source_namespace"], "team.alpha");
        assert_eq!(json["payload"]["audit"]["target_namespace"], "team.beta");
        assert_eq!(
            json["payload"]["audit"]["policy_family"],
            "visibility_sharing"
        );
        assert_eq!(json["payload"]["audit"]["outcome_class"], "accepted");
        assert_eq!(json["payload"]["audit"]["blocked_stage"], "policy_gate");
        assert_eq!(json["payload"]["audit"]["related_run"], "share-run-42");
        assert_eq!(
            json["payload"]["audit"]["redaction_summary"],
            serde_json::json!([])
        );
        assert_eq!(json["payload"]["audit"]["redacted"], false);
    }

    #[test]
    fn mcp_response_rejects_unknown_top_level_fields() {
        let result = serde_json::from_value::<McpResponse>(serde_json::json!({
            "status": "ok",
            "retrieval": {
                "request_id": "req-5",
                "namespace": "mcp.team",
                "outcome_class": "accepted",
                "partial_success": false,
                "explain_trace": {
                    "route_summary": {"route_family": "exact_id_tier1"},
                    "trace_stages": ["tier1_exact"],
                    "result_reasons": [],
                    "omitted_summary": {"budget_capped": 0},
                    "policy_summary": {"effective_namespace": "mcp.team"},
                    "provenance_summary": {"source_reference": "result_set"},
                    "freshness_markers": [],
                    "conflict_markers": [],
                    "uncertainty_markers": []
                },
                "result": serde_json::to_value(sample_result_set()).unwrap()
            },
            "unexpected": true
        }));

        assert!(result.is_err());
    }

    #[test]
    fn mcp_response_rejects_unknown_retrieval_fields() {
        let result = serde_json::from_value::<McpResponse>(serde_json::json!({
            "status": "ok",
            "retrieval": {
                "request_id": "req-6",
                "namespace": "mcp.team",
                "outcome_class": "accepted",
                "partial_success": false,
                "explain_trace": {
                    "route_summary": {"route_family": "exact_id_tier1"},
                    "trace_stages": ["tier1_exact"],
                    "result_reasons": [],
                    "omitted_summary": {"budget_capped": 0},
                    "policy_summary": {"effective_namespace": "mcp.team"},
                    "provenance_summary": {"source_reference": "result_set"},
                    "freshness_markers": [],
                    "conflict_markers": [],
                    "uncertainty_markers": []
                },
                "result": serde_json::to_value(sample_result_set()).unwrap(),
                "unexpected": true
            }
        }));

        assert!(result.is_err());
    }

    #[test]
    fn mcp_response_rejects_unknown_error_fields() {
        let error = serde_json::from_value::<McpResponse>(serde_json::json!({
            "status": "error",
            "error": {
                "code": "policy_denied",
                "message": "namespace isolation prevents export",
                "is_policy_denial": true,
                "unexpected": true
            }
        }))
        .unwrap_err();

        assert!(
            error.to_string().contains("unknown field") && error.to_string().contains("unexpected")
        );
    }

    #[test]
    fn inspect_payload_rejects_unknown_fields() {
        let error = serde_json::from_value::<McpInspectPayload>(serde_json::json!({
            "request_id": "inspect-req-1",
            "namespace": "team.alpha",
            "memory_id": 42,
            "tier": "tier1",
            "lineage": {},
            "policy_flags": {},
            "lifecycle_state": {},
            "index_presence": {},
            "graph_neighborhood_summary": {},
            "decay_retention": {},
            "explain_trace": {
                "policy_summary": {},
                "provenance_summary": {},
                "freshness_markers": {},
                "conflict_markers": {},
                "trace_stages": []
            },
            "unexpected": true
        }))
        .unwrap_err();
        assert!(
            error.to_string().contains("unknown field") && error.to_string().contains("unexpected")
        );
    }

    #[test]
    fn resource_listing_rejects_unknown_nested_resource_fields() {
        let error = serde_json::from_value::<McpResourceListing>(serde_json::json!({
            "request_id": "resource-req-1",
            "namespace": "team.alpha",
            "resources": [
                {
                    "uri": "membrain://team.alpha/memories/42",
                    "name": "memory-42",
                    "mime_type": "application/json",
                    "unexpected": true
                }
            ]
        }))
        .unwrap_err();
        assert!(
            error.to_string().contains("unknown field") && error.to_string().contains("unexpected")
        );
    }

    #[test]
    fn observe_request_round_trips_with_extended_fields() {
        let request = McpRequest::Observe(ObserveParams {
            content: "streamed content".to_string(),
            namespace: "team.alpha".to_string(),
            context: Some("coding session".to_string()),
            chunk_size: Some(120),
            source_label: Some("stdin:test".to_string()),
            topic_threshold: Some(0.4),
            min_chunk_size: Some(24),
            dry_run: Some(true),
            common: sample_common(),
        });

        let json = serde_json::to_value(&request).unwrap();
        assert_eq!(json["method"], "observe");
        assert_eq!(json["params"]["content"], "streamed content");
        assert_eq!(json["params"]["namespace"], "team.alpha");
        assert_eq!(json["params"]["context"], "coding session");
        assert_eq!(json["params"]["chunk_size"], 120);
        assert_eq!(json["params"]["source_label"], "stdin:test");
        assert_eq!(
            json["params"]["topic_threshold"],
            serde_json::Value::from(0.4_f32)
        );
        assert_eq!(json["params"]["min_chunk_size"], 24);
        assert_eq!(json["params"]["dry_run"], true);
        assert_eq!(json["params"]["request_id"], "test-req-1");

        let decoded: McpRequest = serde_json::from_value(json).unwrap();
        match decoded {
            McpRequest::Observe(params) => {
                assert_eq!(params.content, "streamed content");
                assert_eq!(params.context.as_deref(), Some("coding session"));
                assert_eq!(params.chunk_size, Some(120));
                assert_eq!(params.source_label.as_deref(), Some("stdin:test"));
                assert_eq!(params.topic_threshold, Some(0.4));
                assert_eq!(params.min_chunk_size, Some(24));
                assert_eq!(params.dry_run, Some(true));
                assert_eq!(params.common, sample_common());
            }
            _ => std::process::abort(),
        }
    }

    #[test]
    fn inspect_payload_can_embed_passive_observation_summary() {
        let mut payload = McpInspectPayload::from_result(
            RequestId::new("req-observe-inspect").unwrap(),
            NamespaceId::new("mcp.team").unwrap(),
            7,
            &sample_result_set(),
        )
        .unwrap();
        payload.explain_trace.passive_observation = Some(
            serde_json::to_value(PassiveObservationInspectSummary {
                source_kind: "observation",
                write_decision: "capture",
                captured_as_observation: true,
                observation_source: FieldPresence::Present("stdin:test".to_string()),
                observation_chunk_id: FieldPresence::Present("obs-0000000000000042".to_string()),
                retention_marker: FieldPresence::Present("volatile_observation"),
            })
            .unwrap(),
        );

        let json = serde_json::to_value(&payload).unwrap();
        assert_eq!(
            json["explain_trace"]["passive_observation"]["source_kind"],
            "observation"
        );
        assert_eq!(
            json["explain_trace"]["passive_observation"]["observation_chunk_id"],
            serde_json::json!({"Present":"obs-0000000000000042"})
        );
    }

    #[test]
    fn context_budget_request_round_trips_optional_fields() {
        let request = McpRequest::ContextBudget(ContextBudgetParams {
            token_budget: 256,
            namespace: "team.alpha".to_string(),
            current_context: Some("debugging session".to_string()),
            working_memory_ids: Some(vec![7, 8]),
            format: Some("markdown".to_string()),
            common: sample_common(),
        });

        let json = serde_json::to_value(&request).unwrap();
        assert_eq!(json["method"], "context_budget");
        assert_eq!(json["params"]["token_budget"], 256);
        assert_eq!(json["params"]["namespace"], "team.alpha");
        assert_eq!(json["params"]["current_context"], "debugging session");
        assert_eq!(
            json["params"]["working_memory_ids"],
            serde_json::json!([7, 8])
        );
        assert_eq!(json["params"]["format"], "markdown");
        assert_eq!(json["params"]["request_id"], "test-req-1");

        let decoded: McpRequest = serde_json::from_value(json).unwrap();
        match decoded {
            McpRequest::ContextBudget(params) => {
                assert_eq!(params.token_budget, 256);
                assert_eq!(params.current_context.as_deref(), Some("debugging session"));
                assert_eq!(params.working_memory_ids, Some(vec![7, 8]));
                assert_eq!(params.format.as_deref(), Some("markdown"));
                assert_eq!(params.common, sample_common());
            }
            _ => std::process::abort(),
        }
    }

    #[test]
    fn procedures_request_round_trips_with_mutation_fields() {
        let request = McpRequest::Procedures(ProceduresParams {
            namespace: "team.alpha".to_string(),
            promote: Some("procedural://team.alpha/000000000000002a".to_string()),
            rollback: None,
            note: Some("approved".to_string()),
            approved_by: Some("mcp.user".to_string()),
            public: Some(true),
            common: sample_common(),
        });

        let json = serde_json::to_value(&request).unwrap();
        assert_eq!(json["method"], "procedures");
        assert_eq!(json["params"]["namespace"], "team.alpha");
        assert_eq!(
            json["params"]["promote"],
            "procedural://team.alpha/000000000000002a"
        );
        assert_eq!(json["params"]["note"], "approved");
        assert_eq!(json["params"]["approved_by"], "mcp.user");
        assert_eq!(json["params"]["public"], true);
        assert_eq!(json["params"]["request_id"], "test-req-1");

        let decoded: McpRequest = serde_json::from_value(json).unwrap();
        match decoded {
            McpRequest::Procedures(params) => {
                assert_eq!(
                    params.promote.as_deref(),
                    Some("procedural://team.alpha/000000000000002a")
                );
                assert_eq!(params.note.as_deref(), Some("approved"));
                assert_eq!(params.approved_by.as_deref(), Some("mcp.user"));
                assert_eq!(params.public, Some(true));
                assert_eq!(params.common, sample_common());
            }
            _ => std::process::abort(),
        }
    }

    #[test]
    fn encode_request_round_trips_with_common_envelope_fields() {
        let request = McpRequest::Encode(EncodeParams {
            content: "user prefers dark mode".to_string(),
            namespace: "team.alpha".to_string(),
            memory_type: Some("user_preference".to_string()),
            source_metadata: None,
            context_bindings: None,
            emotional_annotations: None,
            salience: Some(500),
            tags: Some(vec!["preference".to_string()]),
            entity_refs: None,
            relation_refs: None,
            retention_hint: None,
            visibility: Some("shared".to_string()),
            common: sample_common(),
        });

        let json = serde_json::to_value(&request).unwrap();
        assert_eq!(json["method"], "encode");
        assert_eq!(json["params"]["content"], "user prefers dark mode");
        assert_eq!(json["params"]["namespace"], "team.alpha");
        assert_eq!(json["params"]["memory_type"], "user_preference");
        assert_eq!(json["params"]["visibility"], "shared");
        assert_eq!(json["params"]["salience"], 500);
        assert_eq!(json["params"]["tags"], serde_json::json!(["preference"]));
        assert_eq!(json["params"]["request_id"], "test-req-1");
        assert_eq!(json["params"]["workspace_id"], "ws-7");
        assert_eq!(json["params"]["agent_id"], "agent-3");
        assert_eq!(json["params"]["session_id"], "session-9");
        assert_eq!(json["params"]["task_id"], "task-2");
        assert_eq!(json["params"]["time_budget_ms"], 75);
        assert_eq!(json["params"]["policy_context"]["include_public"], true);
        assert_eq!(
            json["params"]["policy_context"]["sharing_visibility"],
            "public"
        );

        let decoded: McpRequest = serde_json::from_value(json).unwrap();
        match decoded {
            McpRequest::Encode(params) => {
                assert_eq!(params.content, "user prefers dark mode");
                assert_eq!(params.salience, Some(500));
                assert_eq!(params.visibility.as_deref(), Some("shared"));
                assert!(params.source_metadata.is_none());
                assert_eq!(params.common, sample_common());
            }
            _ => std::process::abort(),
        }
    }

    #[test]
    fn recall_request_supports_full_mcp_api_fields() {
        let request = McpRequest::Recall(RecallParams {
            query_text: Some("dark mode preference".to_string()),
            context_text: Some("user settings".to_string()),
            namespace: "team.alpha".to_string(),
            mode: Some("balanced".to_string()),
            result_budget: Some(5),
            token_budget: Some(512),
            time_budget_ms: Some(100),
            effort: Some("moderate".to_string()),
            explain: Some(true),
            include_public: Some(false),
            like_id: Some(42),
            unlike_id: None,
            graph_mode: Some("bounded".to_string()),
            cold_tier: Some(false),
            memory_kinds: Some(vec!["user_preference".to_string()]),
            era_id: None,
            as_of_tick: None,
            at_snapshot: None,
            min_strength: Some(100),
            min_confidence: Some(0.7),
            show_decaying: Some(false),
            mood_congruent: Some(true),
            common: sample_common(),
        });

        let json = serde_json::to_value(&request).unwrap();
        assert_eq!(json["method"], "recall");
        assert_eq!(json["params"]["query_text"], "dark mode preference");
        assert_eq!(json["params"]["result_budget"], 5);
        assert_eq!(json["params"]["token_budget"], 512);
        assert_eq!(json["params"]["like_id"], 42);
        assert_eq!(json["params"]["mood_congruent"], true);
        assert_eq!(json["params"]["request_id"], "test-req-1");
        assert_eq!(json["params"]["workspace_id"], "ws-7");
        assert_eq!(json["params"]["policy_context"]["include_public"], true);

        let decoded: McpRequest = serde_json::from_value(json).unwrap();
        match decoded {
            McpRequest::Recall(params) => {
                assert_eq!(params.result_budget, Some(5));
                assert_eq!(params.like_id, Some(42));
                assert_eq!(params.mood_congruent, Some(true));
                let mut expected = sample_common();
                expected.time_budget_ms = None;
                assert_eq!(params.common, expected);
            }
            _ => std::process::abort(),
        }
    }

    #[test]
    fn mutation_method_envelopes_round_trip_via_serde() {
        // Link
        let link_json = serde_json::to_value(McpRequest::Link(
            serde_json::from_value(serde_json::json!({
                "source_id": 42,
                "target_id": 99,
                "namespace": "team.alpha",
                "link_type": "supports"
            }))
            .unwrap(),
        ))
        .unwrap();
        assert_eq!(link_json["method"], "link");
        assert_eq!(link_json["params"]["source_id"], 42);
        assert_eq!(link_json["params"]["link_type"], "supports");

        // Pin
        let pin_json = serde_json::to_value(McpRequest::Pin(
            serde_json::from_value(serde_json::json!({
                "id": 42,
                "namespace": "team.alpha",
                "reason": "critical reference"
            }))
            .unwrap(),
        ))
        .unwrap();
        assert_eq!(pin_json["method"], "pin");
        assert_eq!(pin_json["params"]["id"], 42);

        // Forget
        let forget_json = serde_json::to_value(McpRequest::Forget(
            serde_json::from_value(serde_json::json!({
                "id": 42,
                "namespace": "team.alpha",
                "mode": "archive",
                "reason": "stale data"
            }))
            .unwrap(),
        ))
        .unwrap();
        assert_eq!(forget_json["method"], "forget");
        assert_eq!(forget_json["params"]["mode"], "archive");

        // Why
        let why_json = serde_json::to_value(McpRequest::Why(
            serde_json::from_value(serde_json::json!({
                "id": 42,
                "namespace": "team.alpha",
                "depth": 3
            }))
            .unwrap(),
        ))
        .unwrap();
        assert_eq!(why_json["method"], "why");
        assert_eq!(why_json["params"]["depth"], 3);

        // Invalidate
        let invalidate_json = serde_json::to_value(McpRequest::Invalidate(
            serde_json::from_value(serde_json::json!({
                "id": 42,
                "namespace": "team.alpha",
                "dry_run": true
            }))
            .unwrap(),
        ))
        .unwrap();
        assert_eq!(invalidate_json["method"], "invalidate");
        assert_eq!(invalidate_json["params"]["dry_run"], true);

        // Share
        let share_json = serde_json::to_value(McpRequest::Share(
            serde_json::from_value(serde_json::json!({
                "id": 42,
                "namespace_id": "team.beta",
                "request_id": "req-share-42",
                "policy_context": {"include_public": false, "sharing_visibility": "shared"}
            }))
            .unwrap(),
        ))
        .unwrap();
        assert_eq!(share_json["method"], "share");
        assert_eq!(share_json["params"]["namespace_id"], "team.beta");
        assert_eq!(share_json["params"]["request_id"], "req-share-42");
        assert_eq!(
            share_json["params"]["policy_context"]["sharing_visibility"],
            "shared"
        );

        // Fork
        let fork_json = serde_json::to_value(McpRequest::Fork(
            serde_json::from_value(serde_json::json!({
                "name": "agent-specialist",
                "namespace": "team.alpha",
                "inherit": "shared",
                "note": "testing"
            }))
            .unwrap(),
        ))
        .unwrap();
        assert_eq!(fork_json["method"], "fork");
        assert_eq!(fork_json["params"]["inherit"], "shared");

        // Unshare
        let unshare_json = serde_json::to_value(McpRequest::Unshare(
            serde_json::from_value(serde_json::json!({
                "id": 42,
                "namespace": "team.alpha",
                "request_id": "req-unshare-42",
                "session_id": "session-9"
            }))
            .unwrap(),
        ))
        .unwrap();
        assert_eq!(unshare_json["method"], "unshare");
        assert_eq!(unshare_json["params"]["request_id"], "req-unshare-42");
        assert_eq!(unshare_json["params"]["session_id"], "session-9");

        // Audit
        let audit_json = serde_json::to_value(McpRequest::Audit(
            serde_json::from_value(serde_json::json!({
                "namespace": "team.alpha",
                "memory_id": 42,
                "since_tick": 7,
                "op": "maintenance",
                "limit": 3
            }))
            .unwrap(),
        ))
        .unwrap();
        assert_eq!(audit_json["method"], "audit");
        assert_eq!(audit_json["params"]["namespace"], "team.alpha");
        assert_eq!(audit_json["params"]["memory_id"], 42);
        assert_eq!(audit_json["params"]["since_tick"], 7);
        assert_eq!(audit_json["params"]["op"], "maintenance");
        assert_eq!(audit_json["params"]["limit"], 3);

        // Consolidate
        let consolidate_json = serde_json::to_value(McpRequest::Consolidate(
            serde_json::from_value(serde_json::json!({
                "namespace": "team.alpha",
                "scope": "session"
            }))
            .unwrap(),
        ))
        .unwrap();
        assert_eq!(consolidate_json["method"], "consolidate");
        assert_eq!(consolidate_json["params"]["scope"], "session");

        // Repair
        let repair_json = serde_json::to_value(McpRequest::Repair(
            serde_json::from_value(serde_json::json!({
                "namespace": "team.alpha",
                "surfaces": ["index", "graph"],
                "dry_run": true
            }))
            .unwrap(),
        ))
        .unwrap();
        assert_eq!(repair_json["method"], "repair");
        assert_eq!(repair_json["params"]["dry_run"], true);

        // Compress
        let compress_common = sample_common();
        let compress_json = serde_json::to_value(McpRequest::Compress(CompressParams {
            namespace: "team.alpha".to_string(),
            dry_run: Some(true),
            common: compress_common.clone(),
        }))
        .unwrap();
        assert_eq!(compress_json["method"], "compress");
        assert_eq!(compress_json["params"]["namespace"], "team.alpha");
        assert_eq!(compress_json["params"]["dry_run"], true);
        assert_eq!(compress_json["params"]["request_id"], "test-req-1");
        assert_eq!(compress_json["params"]["workspace_id"], "ws-7");
        assert_eq!(compress_json["params"]["agent_id"], "agent-3");
        assert_eq!(compress_json["params"]["session_id"], "session-9");
        assert_eq!(compress_json["params"]["task_id"], "task-2");
        assert_eq!(compress_json["params"]["time_budget_ms"], 75);
        assert_eq!(
            compress_json["params"]["policy_context"]["sharing_visibility"],
            "public"
        );

        let decoded_compress: McpRequest = serde_json::from_value(compress_json).unwrap();
        match decoded_compress {
            McpRequest::Compress(params) => {
                assert_eq!(params.namespace, "team.alpha");
                assert_eq!(params.dry_run, Some(true));
                assert_eq!(params.common, compress_common);
            }
            _ => std::process::abort(),
        }

        // Schemas
        let schemas_common = sample_common();
        let schemas_json = serde_json::to_value(McpRequest::Schemas(SchemasParams {
            namespace: "team.alpha".to_string(),
            top_n: Some(4),
            common: schemas_common.clone(),
        }))
        .unwrap();
        assert_eq!(schemas_json["method"], "schemas");
        assert_eq!(schemas_json["params"]["namespace"], "team.alpha");
        assert_eq!(schemas_json["params"]["top_n"], 4);
        assert_eq!(schemas_json["params"]["request_id"], "test-req-1");
        assert_eq!(schemas_json["params"]["workspace_id"], "ws-7");
        assert_eq!(schemas_json["params"]["agent_id"], "agent-3");
        assert_eq!(schemas_json["params"]["session_id"], "session-9");
        assert_eq!(schemas_json["params"]["task_id"], "task-2");
        assert_eq!(schemas_json["params"]["time_budget_ms"], 75);
        assert_eq!(
            schemas_json["params"]["policy_context"]["sharing_visibility"],
            "public"
        );

        let decoded_schemas: McpRequest = serde_json::from_value(schemas_json).unwrap();
        match decoded_schemas {
            McpRequest::Schemas(params) => {
                assert_eq!(params.namespace, "team.alpha");
                assert_eq!(params.top_n, Some(4));
                assert_eq!(params.common, schemas_common);
            }
            _ => std::process::abort(),
        }

        // Preflight run
        let preflight_run_json = serde_json::to_value(McpRequest::PreflightRun(
            serde_json::from_value(serde_json::json!({
                "namespace": "team.alpha",
                "original_query": "delete prior audit events",
                "proposed_action": "purge namespace audit history"
            }))
            .unwrap(),
        ))
        .unwrap();
        assert_eq!(preflight_run_json["method"], "preflight.run");
        assert_eq!(preflight_run_json["params"]["namespace"], "team.alpha");

        // Preflight explain
        let preflight_explain_json = serde_json::to_value(McpRequest::PreflightExplain(
            serde_json::from_value(serde_json::json!({
                "namespace": "team.alpha",
                "original_query": "delete prior audit events",
                "proposed_action": "purge namespace audit history"
            }))
            .unwrap(),
        ))
        .unwrap();
        assert_eq!(preflight_explain_json["method"], "preflight.explain");
        assert_eq!(
            preflight_explain_json["params"]["proposed_action"],
            "purge namespace audit history"
        );

        // Preflight allow
        let preflight_allow_json = serde_json::to_value(McpRequest::PreflightAllow(
            serde_json::from_value(serde_json::json!({
                "namespace": "team.alpha",
                "original_query": "delete prior audit events",
                "proposed_action": "purge namespace audit history",
                "authorization_token": "allow-123",
                "bypass_flags": ["manual_override"]
            }))
            .unwrap(),
        ))
        .unwrap();
        assert_eq!(preflight_allow_json["method"], "preflight.allow");
        assert_eq!(
            preflight_allow_json["params"]["bypass_flags"],
            serde_json::json!(["manual_override"])
        );
    }

    #[test]
    fn operator_and_feature_methods_round_trip() {
        // Stats
        let stats = serde_json::to_value(McpRequest::Stats(
            serde_json::from_value(serde_json::json!({"namespace": "team.alpha"})).unwrap(),
        ))
        .unwrap();
        assert_eq!(stats["method"], "stats");

        // Health
        let health = serde_json::to_value(McpRequest::Health(
            serde_json::from_value(serde_json::json!({"namespace": "team.alpha"})).unwrap(),
        ))
        .unwrap();
        assert_eq!(health["method"], "health");

        // Doctor
        let doctor = serde_json::to_value(McpRequest::Doctor(
            serde_json::from_value(serde_json::json!({})).unwrap(),
        ))
        .unwrap();
        assert_eq!(doctor["method"], "doctor");
        assert!(doctor["params"].as_object().unwrap().is_empty());

        // Dream
        let dream = serde_json::to_value(McpRequest::Dream(
            serde_json::from_value(serde_json::json!({"namespace": "team.alpha"})).unwrap(),
        ))
        .unwrap();
        assert_eq!(dream["method"], "dream");

        // Snapshot
        let snapshot = serde_json::to_value(McpRequest::Snapshot(
            serde_json::from_value(serde_json::json!({
                "name": "pre-migration",
                "namespace": "team.alpha",
                "note": "before schema v2"
            }))
            .unwrap(),
        ))
        .unwrap();
        assert_eq!(snapshot["method"], "snapshot");
        assert_eq!(snapshot["params"]["name"], "pre-migration");
    }

    #[test]
    fn forget_params_rejects_unknown_fields() {
        let error = serde_json::from_value::<super::ForgetParams>(serde_json::json!({
            "id": 42,
            "namespace": "team.alpha",
            "mode": "archive",
            "unexpected": true
        }))
        .unwrap_err();
        assert!(
            error.to_string().contains("unknown field") && error.to_string().contains("unexpected")
        );
    }

    #[test]
    fn link_params_rejects_unknown_fields() {
        let error = serde_json::from_value::<super::LinkParams>(serde_json::json!({
            "source_id": 42,
            "target_id": 99,
            "namespace": "team.alpha",
            "unexpected": true
        }))
        .unwrap_err();
        assert!(
            error.to_string().contains("unknown field") && error.to_string().contains("unexpected")
        );
    }

    // ── Goal management round-trip tests ─────────────────────────────────

    #[test]
    fn policy_context_hint_rejects_unknown_fields() {
        let error = serde_json::from_value::<PolicyContextHint>(serde_json::json!({
            "include_public": true,
            "sharing_visibility": "public",
            "unexpected": true
        }))
        .unwrap_err();
        assert!(error.to_string().contains("unknown field"));
        assert!(error.to_string().contains("unexpected"));
    }

    #[test]
    fn goal_methods_round_trip() {
        // goal_state
        let state = serde_json::to_value(McpRequest::GoalState(
            serde_json::from_value(serde_json::json!({})).unwrap(),
        ))
        .unwrap();
        assert_eq!(state["method"], "goal_state");

        // goal_state with task_id
        let state_with_task = serde_json::to_value(McpRequest::GoalState(
            serde_json::from_value(serde_json::json!({"task_id": "task-42"})).unwrap(),
        ))
        .unwrap();
        assert_eq!(state_with_task["method"], "goal_state");
        assert_eq!(state_with_task["params"]["task_id"], "task-42");

        // goal_pause
        let pause = serde_json::to_value(McpRequest::GoalPause(
            serde_json::from_value(serde_json::json!({
                "task_id": "task-42",
                "note": "waiting for review"
            }))
            .unwrap(),
        ))
        .unwrap();
        assert_eq!(pause["method"], "goal_pause");
        assert_eq!(pause["params"]["note"], "waiting for review");

        // goal_pin
        let pin = serde_json::to_value(McpRequest::GoalPin(
            serde_json::from_value(serde_json::json!({
                "task_id": "task-42",
                "memory_id": 7
            }))
            .unwrap(),
        ))
        .unwrap();
        assert_eq!(pin["method"], "goal_pin");
        assert_eq!(pin["params"]["memory_id"], 7);

        // goal_dismiss
        let dismiss = serde_json::to_value(McpRequest::GoalDismiss(
            serde_json::from_value(serde_json::json!({
                "task_id": "task-42",
                "memory_id": 7
            }))
            .unwrap(),
        ))
        .unwrap();
        assert_eq!(dismiss["method"], "goal_dismiss");
        assert_eq!(dismiss["params"]["memory_id"], 7);

        // goal_snapshot
        let snapshot = serde_json::to_value(McpRequest::GoalSnapshot(
            serde_json::from_value(serde_json::json!({
                "task_id": "task-42",
                "note": "handoff"
            }))
            .unwrap(),
        ))
        .unwrap();
        assert_eq!(snapshot["method"], "goal_snapshot");
        assert_eq!(snapshot["params"]["note"], "handoff");

        // goal_resume
        let resume = serde_json::to_value(McpRequest::GoalResume(
            serde_json::from_value(serde_json::json!({"task_id": "task-42"})).unwrap(),
        ))
        .unwrap();
        assert_eq!(resume["method"], "goal_resume");

        // goal_abandon
        let abandon = serde_json::to_value(McpRequest::GoalAbandon(
            serde_json::from_value(serde_json::json!({
                "task_id": "task-42",
                "reason": "requirements changed"
            }))
            .unwrap(),
        ))
        .unwrap();
        assert_eq!(abandon["method"], "goal_abandon");
        assert_eq!(abandon["params"]["reason"], "requirements changed");
    }

    #[test]
    fn goal_params_rejects_unknown_fields() {
        let error = serde_json::from_value::<super::GoalStateParams>(serde_json::json!({
            "task_id": "task-42",
            "unexpected": true
        }))
        .unwrap_err();
        assert!(
            error.to_string().contains("unknown field") && error.to_string().contains("unexpected")
        );

        let error = serde_json::from_value::<super::GoalPauseParams>(serde_json::json!({
            "task_id": "task-42",
            "note": "paused",
            "unexpected": true
        }))
        .unwrap_err();
        assert!(
            error.to_string().contains("unknown field") && error.to_string().contains("unexpected")
        );

        let error = serde_json::from_value::<super::GoalPinParams>(serde_json::json!({
            "task_id": "task-42",
            "memory_id": 9,
            "unexpected": true
        }))
        .unwrap_err();
        assert!(
            error.to_string().contains("unknown field") && error.to_string().contains("unexpected")
        );
    }

    #[test]
    fn goal_pause_round_trips_through_serde() {
        let original = serde_json::json!({
            "method": "goal_pause",
            "params": {
                "task_id": "task-100",
                "note": "blocked on external API"
            }
        });

        let request: McpRequest = serde_json::from_value(original.clone()).unwrap();
        let round_tripped = serde_json::to_value(&request).unwrap();

        assert_eq!(round_tripped["method"], "goal_pause");
        assert_eq!(round_tripped["params"]["task_id"], "task-100");
        assert_eq!(round_tripped["params"]["note"], "blocked on external API");
    }
}
