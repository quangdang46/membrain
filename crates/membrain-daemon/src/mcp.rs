use membrain_core::api::{NamespaceId, RequestId};
use membrain_core::engine::result::RetrievalResultSet;
use membrain_core::observability::OutcomeClass;
use serde::{Deserialize, Serialize};

// ---------------------------------------------------------------------------
// Common request envelope fields
// ---------------------------------------------------------------------------

/// Shared envelope fields carried by every MCP method request.
/// These are execution context, not authorization shortcuts.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
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

/// Policy hints carried by the request envelope before core policy evaluation.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PolicyContextHint {
    #[serde(default)]
    pub include_public: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sharing_visibility: Option<String>,
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
    pub namespace: String,
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

/// Trace causal chain to root evidence.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct WhyParams {
    pub id: u64,
    pub namespace: String,
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
    pub inherit: Option<bool>,
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

/// Extensions for MCP Resources (`mb-23u.7.2`)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpResource {
    pub uri: String,
    pub name: String,
    pub mime_type: String,
    pub description: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::{
        CommonRequestFields, EncodeParams, ExplainParams, InspectParams, McpError, McpRequest,
        McpResponse, McpRetrievalPayload, RecallParams,
    };
    use membrain_core::api::{NamespaceId, RequestId};
    use membrain_core::engine::recall::RecallPlanKind;
    use membrain_core::engine::result::RetrievalExplain;
    use membrain_core::engine::result::RetrievalResultSet;
    use membrain_core::engine::result::{DualOutputMode, PackagingMetadata};
    use membrain_core::observability::OutcomeClass;

    fn sample_common() -> CommonRequestFields {
        CommonRequestFields {
            request_id: Some("test-req-1".to_string()),
            workspace_id: None,
            agent_id: None,
            session_id: None,
            task_id: None,
            time_budget_ms: None,
            policy_context: None,
        }
    }

    fn sample_result_set() -> RetrievalResultSet {
        RetrievalResultSet {
            outcome_class: OutcomeClass::Accepted,
            evidence_pack: Vec::new(),
            action_pack: None,
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
                as_of_tick: None,
            },
            packaging_metadata: PackagingMetadata {
                result_budget: 8,
                token_budget: Some(256),
                graph_assistance: "none".to_string(),
                degraded_summary: None,
                packaging_mode: "bounded".to_string(),
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
        assert!(json["explain_trace"].get("policy_summary").is_some());
        assert!(json["explain_trace"].get("provenance_summary").is_some());
        assert!(json["explain_trace"].get("freshness_markers").is_some());
        assert!(json["explain_trace"].get("conflict_markers").is_some());
        assert!(json["explain_trace"].get("uncertainty_markers").is_some());
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
    fn mcp_response_rejects_unknown_top_level_fields() {
        let error = serde_json::from_value::<McpResponse>(serde_json::json!({
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
        }))
        .unwrap_err();

        assert!(error.to_string().contains("unknown field `unexpected`"));
    }

    #[test]
    fn mcp_response_rejects_unknown_retrieval_fields() {
        let error = serde_json::from_value::<McpResponse>(serde_json::json!({
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
        }))
        .unwrap_err();

        assert!(error.to_string().contains("unknown field `unexpected`"));
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

        assert!(error.to_string().contains("unknown field `unexpected`"));
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
            visibility: None,
            common: sample_common(),
        });

        let json = serde_json::to_value(&request).unwrap();
        assert_eq!(json["method"], "encode");
        assert_eq!(json["params"]["content"], "user prefers dark mode");
        assert_eq!(json["params"]["namespace"], "team.alpha");
        assert_eq!(json["params"]["memory_type"], "user_preference");
        assert_eq!(json["params"]["salience"], 500);
        assert_eq!(json["params"]["tags"], serde_json::json!(["preference"]));
        assert_eq!(json["params"]["request_id"], "test-req-1");

        let decoded: McpRequest = serde_json::from_value(json).unwrap();
        match decoded {
            McpRequest::Encode(params) => {
                assert_eq!(params.content, "user prefers dark mode");
                assert_eq!(params.salience, Some(500));
                assert!(params.source_metadata.is_none());
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

        let decoded: McpRequest = serde_json::from_value(json).unwrap();
        match decoded {
            McpRequest::Recall(params) => {
                assert_eq!(params.result_budget, Some(5));
                assert_eq!(params.like_id, Some(42));
                assert_eq!(params.mood_congruent, Some(true));
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

        // Share
        let share_json = serde_json::to_value(McpRequest::Share(
            serde_json::from_value(serde_json::json!({
                "id": 42,
                "namespace_id": "team.beta"
            }))
            .unwrap(),
        ))
        .unwrap();
        assert_eq!(share_json["method"], "share");
        assert_eq!(share_json["params"]["namespace_id"], "team.beta");

        // Unshare
        let unshare_json = serde_json::to_value(McpRequest::Unshare(
            serde_json::from_value(serde_json::json!({
                "id": 42,
                "namespace": "team.alpha"
            }))
            .unwrap(),
        ))
        .unwrap();
        assert_eq!(unshare_json["method"], "unshare");

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
            serde_json::from_value(serde_json::json!({"namespace": "team.alpha"})).unwrap(),
        ))
        .unwrap();
        assert_eq!(doctor["method"], "doctor");

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
        assert!(error.to_string().contains("unknown field `unexpected`"));
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
        assert!(error.to_string().contains("unknown field `unexpected`"));
    }

    // ── Goal management round-trip tests ─────────────────────────────────

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
        assert!(error.to_string().contains("unknown field `unexpected`"));

        let error = serde_json::from_value::<super::GoalPauseParams>(serde_json::json!({
            "task_id": "task-42",
            "note": "paused",
            "unexpected": true
        }))
        .unwrap_err();
        assert!(error.to_string().contains("unknown field `unexpected`"));
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
