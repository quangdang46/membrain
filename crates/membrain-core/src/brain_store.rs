use crate::api::{
    ApiModule, DeadZoneEntry, DeadZonesOutput, FieldPresence, ForkInfoOutput, ForkInheritance,
    ForkStatus, HotPathEntry, HotPathsOutput, MergeConflictOutput, MergeConflictStrategy,
    MergeReportOutput, NamespaceId, ProceduralEntryAuditView, ProceduralEntryRecallView,
    ProceduralEntryReviewView, ProceduralEntryStorageView, ProceduralEntrySummary,
    ProceduralStoreOutput, ReflectionArtifactView, SkillArtifactRecallView,
    SkillArtifactReviewView, SkillArtifactStorageView, SkillArtifactSummary, SkillArtifactsOutput,
};
use crate::config::RuntimeConfig;
use crate::embed::EmbedModule;
use crate::engine::belief_history::BeliefHistoryEngine;
use crate::engine::compression::{
    CompressionApplyResult, CompressionEngine, CompressionLogEntry, CompressionPolicy,
    CompressionSummary, CompressionTrigger,
};
use crate::engine::confidence::ConfidenceEngine;
use crate::engine::consolidation::{
    ConsolidationEngine, ConsolidationPolicy, ConsolidationSummary, DerivedArtifact,
    DerivedArtifactKind,
};
use crate::engine::context_budget::{
    ContextBudgetEngine, ContextBudgetRequest, ContextBudgetResponse,
};
use crate::engine::contradiction::{
    ContradictionCandidate, ContradictionEngine, ContradictionError, ContradictionKind,
};
use crate::engine::encode::{ContradictionWriteOutcome, EncodeEngine, WriteBranchOutcome};
use crate::engine::forgetting::{
    ForgettingEngine, ForgettingExplainOutput, ForgettingPolicy, ForgettingSummary,
    ReversibilitySummary,
};
use crate::engine::intent::IntentEngine;
use crate::engine::maintenance::MaintenanceController;
use crate::engine::ranking::RankingEngine;
use crate::engine::recall::RecallEngine;
use crate::engine::reconsolidation::{LabileMemory, ReconsolidationPolicy, ReconsolidationRun};
use crate::engine::repair::RepairEngine;
use crate::engine::working_state::{GoalWorkingState, WorkingStateEngine};
use crate::graph::{CausalInvalidationReport, CausalLink, CausalTrace, GraphModule};
use crate::health::AttentionNamespaceInputs;
use crate::index::IndexModule;
use crate::migrate::{DurableSchemaObject, MigrationModule};
use crate::observability::{
    AuditEventCategory, AuditEventKind, ObservabilityModule, Tier2PrefilterTrace,
};
use crate::policy::{PolicyModule, SharingAccessDecision, SharingScope, SharingVisibility};
use crate::store::audit::{
    AppendOnlyAuditLog, AuditLogEntry, AuditLogFilter, AuditLogSlice, AuditLogStore,
};
use crate::store::cold::ColdStore;
use crate::store::hot::HotStore;
use crate::store::procedural::{
    ProceduralEntryState, ProceduralMemoryRecord, ProceduralStore, ProceduralStoreError,
    ProceduralStoreErrorReason,
};
use crate::store::tier2::{Tier2DurableItemLayout, Tier2Store};
use crate::store::tier_router::{TierRouter, TierRoutingInput, TierRoutingTrace};
use crate::store::{AuditLogStoreApi, HotStoreApi, ProceduralStoreApi, Tier2StoreApi};
use crate::types::{
    AffectSignals, AffectTrajectoryHistory, AffectTrajectoryRow, CompressionMetadata,
    CoreApiVersion, MemoryId, NormalizedMemoryEnvelope, RawEncodeInput, RawIntakeKind,
    SemanticDiff, SemanticDiffCategory, SemanticDiffEntry, SessionId, SharingMetadata,
    SnapshotAnchor, SnapshotId, SnapshotMetadata, SnapshotRetentionClass,
};
use std::collections::{BTreeSet, HashMap, HashSet};

const PROCEDURAL_PROMOTION_ACTOR: &str = "procedural_store_promotion";
const PROCEDURAL_ROLLBACK_ACTOR: &str = "procedural_store_rollback";
const FORK_MERGE_ACTOR: &str = "fork_merge";

#[derive(Debug, Clone)]
struct PlannedMergeConflict {
    output: MergeConflictOutput,
    auto_resolved: bool,
    preferred_side: &'static str,
}

#[derive(Debug, Clone)]
struct PlannedForkMerge {
    merged_items: Vec<String>,
    conflict_items: Vec<PlannedMergeConflict>,
}

/// Inspectable result returned when the core facade prepares a Tier2 layout from encode output.
#[derive(Debug, Clone, PartialEq)]
pub struct PreparedTier2Layout {
    pub layout: Tier2DurableItemLayout,
    pub prefilter_trace: Tier2PrefilterTrace,
}

impl PreparedTier2Layout {
    /// Returns whether the prepared layout remained metadata-only during prefilter planning.
    pub const fn prefilter_stays_metadata_only(&self) -> bool {
        self.prefilter_trace.payload_fetch_count == 0
    }
}

/// Machine-readable failure for named snapshot deletion safeguards.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SnapshotDeleteError {
    pub namespace: NamespaceId,
    pub snapshot_name: String,
    pub reason: SnapshotDeleteErrorReason,
}

impl SnapshotDeleteError {
    fn not_found(namespace: NamespaceId, snapshot_name: impl Into<String>) -> Self {
        Self {
            namespace,
            snapshot_name: snapshot_name.into(),
            reason: SnapshotDeleteErrorReason::NotFound,
        }
    }

    fn last_restorable_anchor(snapshot: SnapshotMetadata) -> Self {
        Self {
            namespace: snapshot.namespace,
            snapshot_name: snapshot.snapshot_name,
            reason: SnapshotDeleteErrorReason::LastRestorableAnchor,
        }
    }
}

/// Stable machine-readable reason for snapshot deletion failure.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SnapshotDeleteErrorReason {
    NotFound,
    LastRestorableAnchor,
}

/// Captured governed fork metadata stored by the core facade.
#[derive(Debug, Clone, PartialEq, Eq)]
struct BrainForkRecord {
    name: String,
    namespace: NamespaceId,
    parent_namespace: NamespaceId,
    forked_at_tick: u64,
    inherit_visibility: ForkInheritance,
    status: ForkStatus,
    merged_at_tick: Option<u64>,
    inherited_count_at_fork: usize,
    note: Option<String>,
}

#[derive(Debug, Clone, PartialEq)]
struct StoredAffectTrajectoryRow {
    namespace: NamespaceId,
    era_id: Option<String>,
    memory_id: MemoryId,
    tick_start: u64,
    tick_end: Option<u64>,
    avg_valence: f32,
    avg_arousal: f32,
    memory_count: u64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct StoredCompressionLogRow {
    schema_memory_id: MemoryId,
    source_memory_count: usize,
    tick: u64,
    namespace: NamespaceId,
    keyword_summary: Option<String>,
}

/// Stable configuration for creating one governed namespace fork.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ForkConfig {
    pub name: String,
    pub parent_namespace: NamespaceId,
    pub inherit_visibility: ForkInheritance,
    pub note: Option<String>,
}

/// Stable configuration for merging one governed namespace fork.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MergeConfig {
    pub fork_name: String,
    pub target_namespace: NamespaceId,
    pub conflict_strategy: MergeConflictStrategy,
    pub dry_run: bool,
}

fn semantic_diff_caution() -> &'static str {
    "semantic diff summarizes bounded historical evidence and does not prove consensus or truth"
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
struct RetrievalAttentionSignals {
    recall_count: u64,
    session_recall_count: u64,
    last_recall_tick: Option<u64>,
    deny_count: u64,
    working_set_pins: usize,
    task_pressure: usize,
}

fn retrieval_attention_score(signals: &RetrievalAttentionSignals) -> u64 {
    signals.recall_count * 5
        + signals.session_recall_count * 11
        + signals.deny_count * 3
        + (signals.working_set_pins as u64 * 17)
        + (signals.task_pressure as u64 * 13)
}

fn retrieval_dominant_signal(signals: &RetrievalAttentionSignals) -> &'static str {
    let candidates = [
        ("working_set_pins", signals.working_set_pins as u64 * 17),
        ("task_pressure", signals.task_pressure as u64 * 13),
        ("session_recall_count", signals.session_recall_count * 11),
        ("recall_count", signals.recall_count * 5),
        ("deny_count", signals.deny_count * 3),
    ];
    candidates
        .into_iter()
        .max_by_key(|(_, weight)| *weight)
        .map(|(signal, _)| signal)
        .unwrap_or("recall_count")
}

fn retrieval_heat_bucket(score: u64, signals: &RetrievalAttentionSignals) -> &'static str {
    if score >= 160 || signals.working_set_pins >= 2 || signals.task_pressure >= 3 {
        "hot"
    } else if score >= 60 || signals.session_recall_count >= 2 {
        "warming"
    } else if score > 0 {
        "warm"
    } else {
        "idle"
    }
}

fn retrieval_prewarm_guidance(
    bucket: &'static str,
    dominant_signal: &'static str,
    signals: &RetrievalAttentionSignals,
) -> (&'static str, &'static str, &'static str) {
    match (bucket, dominant_signal) {
        ("hot", "working_set_pins") | ("hot", "task_pressure") => {
            ("task_intent", "bounded_goal_rewarm", "goal_conditioned")
        }
        ("hot", "session_recall_count") | ("warming", "session_recall_count") => (
            "session_recency",
            "bounded_session_rewarm",
            "session_warmup",
        ),
        ("warm", "recall_count") | ("warming", "recall_count") if signals.recall_count >= 4 => {
            ("session_recency", "queue_prefetch_hint", "prefetch_hints")
        }
        _ => ("none", "observe_only", "none"),
    }
}

fn dead_zone_reason(
    current_tick: u64,
    min_age_ticks: u64,
    signals: &RetrievalAttentionSignals,
) -> (&'static str, &'static str) {
    match signals.last_recall_tick {
        None => ("never_recalled", "cold_start_mitigation"),
        Some(last_tick)
            if current_tick.saturating_sub(last_tick) >= min_age_ticks.saturating_mul(4) =>
        {
            ("long_idle", "cold_start_mitigation")
        }
        Some(_) if signals.recall_count <= 1 => ("single_touch_stale", "prefetch_hints"),
        Some(_) => ("cooling_after_activity", "session_warmup"),
    }
}

fn semantic_diff_category_order(category: SemanticDiffCategory) -> u8 {
    match category {
        SemanticDiffCategory::New => 0,
        SemanticDiffCategory::Strengthened => 1,
        SemanticDiffCategory::Weakened => 2,
        SemanticDiffCategory::Archived => 3,
        SemanticDiffCategory::Conflicting => 4,
        SemanticDiffCategory::DerivedState => 5,
    }
}

fn semantic_diff_category_for_event(kind: AuditEventKind, detail: &str) -> SemanticDiffCategory {
    match kind {
        AuditEventKind::EncodeAccepted => SemanticDiffCategory::New,
        AuditEventKind::EncodeRejected => SemanticDiffCategory::Conflicting,
        AuditEventKind::MaintenanceConsolidationStarted
        | AuditEventKind::MaintenanceConsolidationCompleted
        | AuditEventKind::MaintenanceConsolidationPartial
        | AuditEventKind::MaintenanceReconsolidationApplied
        | AuditEventKind::MaintenanceReconsolidationDiscarded
        | AuditEventKind::MaintenanceReconsolidationDeferred
        | AuditEventKind::MaintenanceReconsolidationBlocked
        | AuditEventKind::ApprovedSharing => SemanticDiffCategory::DerivedState,
        AuditEventKind::MaintenanceForgettingEvaluated => {
            if detail.contains("archive") || detail.contains("archived") {
                SemanticDiffCategory::Archived
            } else if detail.contains("strengthened") || detail.contains("reinforced") {
                SemanticDiffCategory::Strengthened
            } else {
                SemanticDiffCategory::Weakened
            }
        }
        AuditEventKind::ArchiveRecorded => {
            if detail.contains("deleted snapshot")
                || detail.contains("rolled back")
                || detail.contains("archive")
                || detail.contains("archived")
            {
                SemanticDiffCategory::Archived
            } else {
                SemanticDiffCategory::DerivedState
            }
        }
        AuditEventKind::PolicyDenied
        | AuditEventKind::PolicyRedacted
        | AuditEventKind::RecallServed
        | AuditEventKind::RecallDenied
        | AuditEventKind::MaintenanceRepairStarted
        | AuditEventKind::MaintenanceRepairCompleted
        | AuditEventKind::MaintenanceRepairDegraded
        | AuditEventKind::MaintenanceRepairRollbackTriggered
        | AuditEventKind::MaintenanceRepairRollbackCompleted
        | AuditEventKind::MaintenanceMigrationApplied
        | AuditEventKind::MaintenanceCompactionApplied
        | AuditEventKind::IncidentRecorded => SemanticDiffCategory::DerivedState,
    }
}

/// Stable top-level core facade for the initial workspace bootstrap.
#[derive(Debug, Clone, PartialEq)]
pub struct BrainStore {
    config: RuntimeConfig,
    api: ApiModule,
    policy: PolicyModule,
    observability: ObservabilityModule,
    encode: EncodeEngine,
    recall: RecallEngine,
    intent: IntentEngine,
    ranking: RankingEngine,
    contradiction: ContradictionEngine,
    belief_history: BeliefHistoryEngine,
    confidence: ConfidenceEngine,
    consolidation: ConsolidationEngine,
    compression: CompressionEngine,
    forgetting: ForgettingEngine,
    repair: RepairEngine,
    hot_store: HotStore,
    tier2_store: Tier2Store,
    tier_router: TierRouter,
    cold_store: ColdStore,
    procedural_store: ProceduralStore,
    audit_log_store: AuditLogStore,
    graph: GraphModule,
    index: IndexModule,
    embed: EmbedModule,
    migrate: MigrationModule,
    audit_log: AppendOnlyAuditLog,
    procedural_entries: HashMap<String, ProceduralMemoryRecord>,
    working_state: WorkingStateEngine,
    snapshots: HashMap<SnapshotId, SnapshotMetadata>,
    snapshot_name_index: HashMap<(NamespaceId, String), SnapshotId>,
    forks: HashMap<String, BrainForkRecord>,
    affect_trajectory: Vec<StoredAffectTrajectoryRow>,
    compression_log: Vec<StoredCompressionLogRow>,
    compression_memories: HashMap<MemoryId, Tier2DurableItemLayout>,
    next_snapshot_id: u64,
}

impl BrainStore {
    /// Builds a new core facade from the shared runtime configuration.
    pub fn new(config: RuntimeConfig) -> Self {
        Self {
            config,
            api: ApiModule,
            policy: PolicyModule,
            observability: ObservabilityModule,
            encode: EncodeEngine::new(config),
            recall: RecallEngine,
            intent: IntentEngine,
            ranking: RankingEngine,
            contradiction: ContradictionEngine::new(),
            belief_history: BeliefHistoryEngine::new(),
            confidence: ConfidenceEngine,
            consolidation: ConsolidationEngine,
            compression: CompressionEngine,
            forgetting: ForgettingEngine,
            repair: RepairEngine,
            hot_store: HotStore,
            tier2_store: Tier2Store,
            tier_router: TierRouter::default(),
            cold_store: ColdStore,
            procedural_store: ProceduralStore,
            audit_log_store: AuditLogStore,
            graph: GraphModule,
            index: IndexModule,
            embed: EmbedModule,
            migrate: MigrationModule,
            audit_log: AuditLogStore.new_default_log(),
            procedural_entries: HashMap::new(),
            working_state: WorkingStateEngine::default(),
            snapshots: HashMap::new(),
            snapshot_name_index: HashMap::new(),
            forks: HashMap::new(),
            affect_trajectory: Vec::new(),
            compression_log: Vec::new(),
            compression_memories: HashMap::new(),
            next_snapshot_id: 1,
        }
    }

    /// Returns the runtime configuration carried by this facade.
    pub fn config(&self) -> RuntimeConfig {
        self.config
    }

    /// Returns the shared API envelope and validation surface used by wrappers.
    pub fn api(&self) -> &ApiModule {
        &self.api
    }

    /// Returns the shared policy surface used by wrappers.
    pub fn policy(&self) -> &PolicyModule {
        &self.policy
    }

    /// Returns the shared observability surface used by wrappers.
    pub fn observability(&self) -> &ObservabilityModule {
        &self.observability
    }

    /// Returns the shared encode engine surface used by wrappers.
    pub fn encode_engine(&self) -> &EncodeEngine {
        &self.encode
    }

    /// Returns the mutable shared encode engine surface used by wrappers.
    pub fn encode_engine_mut(&mut self) -> &mut EncodeEngine {
        &mut self.encode
    }

    /// Returns the shared recall engine surface used by wrappers.
    pub fn recall_engine(&self) -> &RecallEngine {
        &self.recall
    }

    /// Packs one canonical retrieval result set into a bounded token budget.
    pub fn context_budget(
        &self,
        request: &ContextBudgetRequest,
        result_set: &crate::engine::result::RetrievalResultSet,
    ) -> ContextBudgetResponse {
        ContextBudgetEngine::new().pack_result_set(request, result_set)
    }

    /// Returns the shared intent-classification surface used by wrappers.
    pub fn intent_engine(&self) -> &IntentEngine {
        &self.intent
    }

    /// Returns the shared ranking engine surface used by wrappers.
    pub fn ranking_engine(&self) -> &RankingEngine {
        &self.ranking
    }

    /// Returns the shared contradiction engine surface used by wrappers.
    pub fn contradiction_engine(&self) -> &ContradictionEngine {
        &self.contradiction
    }

    /// Returns the mutable contradiction engine surface used by wrappers.
    pub fn contradiction_engine_mut(&mut self) -> &mut ContradictionEngine {
        &mut self.contradiction
    }

    /// Returns the shared belief-history surface owned by the core crate.
    pub fn belief_history_engine(&self) -> &BeliefHistoryEngine {
        &self.belief_history
    }

    /// Returns the mutable belief-history surface owned by the core crate.
    pub fn belief_history_engine_mut(&mut self) -> &mut BeliefHistoryEngine {
        &mut self.belief_history
    }

    /// Returns the belief timeline for one memory by following its version chain.
    pub fn belief_timeline_for_memory(
        &self,
        memory_id: &MemoryId,
    ) -> Result<
        crate::engine::belief_history::BeliefTimelineView,
        crate::engine::belief_history::BeliefHistoryError,
    > {
        let chain_id = self
            .belief_history
            .chain_for_memory(memory_id)
            .map(|chain| chain.chain_id)
            .ok_or(crate::engine::belief_history::BeliefHistoryError::ChainNotFound)?;
        self.belief_history.timeline(chain_id)
    }

    /// Returns the belief resolution view for one memory.
    pub fn belief_resolution_for_memory(
        &self,
        memory_id: &MemoryId,
    ) -> Result<
        crate::engine::belief_history::BeliefResolutionView,
        crate::engine::belief_history::BeliefHistoryError,
    > {
        self.belief_history.resolve_view_for_memory(memory_id)
    }

    /// Returns the full historical explain output for one memory's belief chain.
    pub fn belief_history_explain_for_memory(
        &self,
        memory_id: &MemoryId,
    ) -> Result<
        Vec<crate::engine::belief_history::HistoricalExplain>,
        crate::engine::belief_history::BeliefHistoryError,
    > {
        let chain_id = self
            .belief_history
            .chain_for_memory(memory_id)
            .map(|chain| chain.chain_id)
            .ok_or(crate::engine::belief_history::BeliefHistoryError::ChainNotFound)?;
        self.belief_history.historical_explain(chain_id)
    }

    /// Returns the first belief-history chain matching one query string.
    pub fn belief_history_for_query(
        &self,
        query: &str,
    ) -> Result<
        crate::engine::belief_history::BeliefTimelineView,
        crate::engine::belief_history::BeliefHistoryError,
    > {
        self.belief_history.belief_history_for_query(query)
    }

    /// Returns all currently open belief conflicts without flattening disagreement.
    pub fn open_belief_conflicts(&self) -> Vec<crate::engine::belief_history::BeliefTimelineView> {
        self.belief_history.open_conflicts()
    }

    /// Returns the shared confidence engine surface owned by the core crate.
    pub fn confidence_engine(&self) -> &ConfidenceEngine {
        &self.confidence
    }

    /// Records an encode-side contradiction branch without silently overwriting either memory.
    pub fn record_encode_contradiction(
        &mut self,
        namespace: NamespaceId,
        existing_memory: MemoryId,
        incoming_memory: MemoryId,
        kind: ContradictionKind,
        conflict_score: u16,
    ) -> Result<ContradictionWriteOutcome, ContradictionError> {
        let outcome = self.encode.record_contradiction_branch(
            &mut self.contradiction,
            namespace.clone(),
            existing_memory,
            incoming_memory,
            kind,
            conflict_score,
        )?;

        let existing_text = self
            .contradiction
            .indexed_memory_text(&namespace, existing_memory)
            .map(str::to_owned)
            .unwrap_or_else(|| format!("memory {}", existing_memory.0));
        let incoming_text = self
            .contradiction
            .indexed_memory_text(&namespace, incoming_memory)
            .map(str::to_owned)
            .unwrap_or_else(|| format!("memory {}", incoming_memory.0));

        let chain_id =
            if let Some(existing_chain) = self.belief_history.chain_for_memory(&existing_memory) {
                existing_chain.chain_id
            } else {
                self.belief_history.create_chain(
                    namespace.clone(),
                    existing_memory,
                    existing_text,
                    conflict_score,
                    0,
                )
            };

        let _ = self.belief_history.record_contradiction(
            chain_id,
            incoming_memory,
            outcome.contradiction_id,
            kind,
            incoming_text,
            conflict_score,
            1,
        );

        let tick = self.current_tick().saturating_add(1);
        self.append_audit_entry(
            AuditLogEntry::new(
                crate::observability::AuditEventCategory::Encode,
                crate::observability::AuditEventKind::EncodeRejected,
                namespace,
                "encode_branch",
                format!(
                    "branch=contradiction_recorded existing_memory={} incoming_memory={} contradiction_kind={} conflict_score={}",
                    existing_memory.0,
                    incoming_memory.0,
                    kind.as_str(),
                    conflict_score
                ),
            )
            .with_memory_id(incoming_memory)
            .with_tick(tick),
        );

        Ok(outcome)
    }

    /// Detects contradictions for an incoming candidate and branches into recording if found.
    ///
    /// This is the primary write-path entry point for contradiction-aware encoding.
    /// It runs detection against indexed memories and, if a conflict is found,
    /// records an explicit contradiction artifact instead of silently overwriting.
    pub fn detect_and_branch_encode(
        &mut self,
        namespace: NamespaceId,
        incoming_memory: MemoryId,
        candidate: &ContradictionCandidate,
    ) -> Result<WriteBranchOutcome, ContradictionError> {
        let outcome = self.encode.detect_and_branch(
            &mut self.contradiction,
            namespace.clone(),
            incoming_memory,
            candidate,
        )?;

        let trace = outcome.trace().clone();
        let (kind, detail) = match &outcome {
            WriteBranchOutcome::Accepted { .. } => (
                crate::observability::AuditEventKind::EncodeAccepted,
                format!(
                    "branch={} incoming_memory={}",
                    trace.branch_label(),
                    trace.incoming_memory.0
                ),
            ),
            WriteBranchOutcome::ContradictionRecorded { .. } => (
                crate::observability::AuditEventKind::EncodeRejected,
                format!(
                    "branch={} existing_memory={} incoming_memory={} contradiction_kind={} conflict_score={}",
                    trace.branch_label(),
                    trace
                        .existing_memory
                        .expect("contradiction trace should include existing memory")
                        .0,
                    trace.incoming_memory.0,
                    trace
                        .detected_kind
                        .expect("contradiction trace should include kind")
                        .as_str(),
                    trace
                        .conflict_score
                        .expect("contradiction trace should include score")
                ),
            ),
        };
        let tick = self.current_tick().saturating_add(1);
        self.append_audit_entry(
            AuditLogEntry::new(
                crate::observability::AuditEventCategory::Encode,
                kind,
                namespace,
                "encode_branch",
                detail,
            )
            .with_memory_id(trace.incoming_memory)
            .with_tick(tick),
        );

        Ok(outcome)
    }

    /// Detects all contradictions for an incoming candidate and records each.
    ///
    /// Use this when the write path must preserve the complete conflict picture
    /// rather than just the strongest signal.
    pub fn detect_all_and_branch_encode(
        &mut self,
        namespace: NamespaceId,
        incoming_memory: MemoryId,
        candidate: &ContradictionCandidate,
    ) -> Result<Vec<ContradictionWriteOutcome>, ContradictionError> {
        let outcomes = self.encode.detect_all_and_branch(
            &mut self.contradiction,
            namespace.clone(),
            incoming_memory,
            candidate,
        )?;

        let tick = self.current_tick().saturating_add(1);
        if outcomes.is_empty() {
            self.append_audit_entry(
                AuditLogEntry::new(
                    crate::observability::AuditEventCategory::Encode,
                    crate::observability::AuditEventKind::EncodeAccepted,
                    namespace,
                    "encode_branch",
                    format!("branch=accepted incoming_memory={}", incoming_memory.0),
                )
                .with_memory_id(incoming_memory)
                .with_tick(tick),
            );
        } else {
            for outcome in &outcomes {
                self.append_audit_entry(
                    AuditLogEntry::new(
                        crate::observability::AuditEventCategory::Encode,
                        crate::observability::AuditEventKind::EncodeRejected,
                        namespace.clone(),
                        "encode_branch",
                        format!(
                            "branch=contradiction_recorded existing_memory={} incoming_memory={} contradiction_kind={} conflict_score=multiple",
                            outcome.existing_memory.0,
                            outcome.incoming_memory.0,
                            outcome.kind.as_str()
                        ),
                    )
                    .with_memory_id(outcome.incoming_memory)
                    .with_tick(tick),
                );
            }
        }

        Ok(outcomes)
    }

    /// Returns the shared consolidation surface owned by the core crate.
    pub fn consolidation_engine(&self) -> &ConsolidationEngine {
        &self.consolidation
    }

    /// Returns the shared compression surface owned by the core crate.
    pub fn compression_engine(&self) -> &CompressionEngine {
        &self.compression
    }

    /// Returns an inspectable summary of one bounded compression pass.
    pub fn summarize_compression_pass(
        &self,
        namespace: NamespaceId,
        policy: CompressionPolicy,
        estimated_candidates: u32,
    ) -> CompressionSummary {
        use crate::engine::maintenance::{MaintenanceJobHandle, MaintenanceJobState};

        let last_run_tick = self.current_tick().checked_sub(1);
        let status =
            self.compression
                .status(namespace, CompressionTrigger::Manual, policy, last_run_tick);
        let run = self.compression.create_run(status);
        let mut handle = MaintenanceJobHandle::new(run, estimated_candidates.max(1) + 1);

        loop {
            let snapshot = handle.poll();
            match snapshot.state {
                MaintenanceJobState::Completed(summary) => return summary,
                MaintenanceJobState::Running { .. } => continue,
                other => panic!("unexpected compression state: {other:?}"),
            }
        }
    }

    /// Applies the first eligible bounded compression candidate for the namespace.
    pub fn apply_compression_pass(
        &mut self,
        namespace: NamespaceId,
        policy: CompressionPolicy,
        estimated_candidates: u32,
        dry_run: bool,
    ) -> CompressionApplyResult {
        let _ = estimated_candidates;
        let last_run_tick = self.current_tick().checked_sub(1);
        let status =
            self.compression
                .status(namespace, CompressionTrigger::Manual, policy, last_run_tick);
        let applied = self.compression.apply_first_candidate(&status, dry_run);
        if !dry_run {
            if let Some(artifact) = applied.schema_artifact.clone() {
                self.persist_compression_artifact(&status.namespace, &artifact);
            }
            if let Some(entry) = applied.compression_log_entry.clone() {
                self.record_compression_log_entry(entry);
            }
        }
        applied
    }

    /// Returns a bounded operator-facing skills surface backed by derived skill artifacts.
    pub fn skill_artifacts(
        &self,
        namespace: NamespaceId,
        policy: ConsolidationPolicy,
        estimated_groups: u32,
        extract: bool,
    ) -> SkillArtifactsOutput {
        let summary =
            self.summarize_consolidation_pass(namespace.clone(), policy, estimated_groups);
        let procedures = summary
            .derived_artifacts
            .iter()
            .filter(|artifact| artifact.kind == DerivedArtifactKind::Skill)
            .map(Self::skill_artifact_summary)
            .collect::<Vec<_>>();
        let extracted_count = procedures.len();

        SkillArtifactsOutput {
            namespace: namespace.as_str().to_string(),
            extraction_trigger: if extract {
                "explicit_skill_extraction"
            } else {
                "stored_skill_review"
            },
            extracted_count,
            skipped_count: summary.groups_evaluated as usize - extracted_count,
            procedures,
        }
    }

    /// Promotes one reviewed derived skill into the authoritative procedural store.
    pub fn promote_skill_to_procedural(
        &mut self,
        namespace: NamespaceId,
        pattern_handle: &str,
        accepted_by: impl Into<String>,
        acceptance_note: impl Into<String>,
        include_public: bool,
    ) -> Result<ProceduralEntrySummary, ProceduralStoreError> {
        let visibility = if include_public {
            SharingVisibility::Public
        } else {
            SharingVisibility::Shared
        };
        let request_context = crate::api::RequestContext {
            namespace: Some(namespace.clone()),
            workspace_id: None,
            agent_id: None,
            session_id: None,
            task_id: None,
            request_id: crate::api::RequestId::new(format!(
                "promote-procedural-{}",
                pattern_handle.replace("/", "-")
            ))
            .expect("procedural promotion request id"),
            policy_context: crate::api::PolicyContext {
                include_public,
                sharing_visibility: visibility,
                caller_identity_bound: true,
                workspace_acl_allowed: true,
                agent_acl_allowed: true,
                session_visibility_allowed: true,
                legal_hold: false,
            },
            time_budget_ms: None,
        };
        self.promote_skill_to_procedural_with_context(
            request_context,
            pattern_handle,
            accepted_by,
            acceptance_note,
        )
    }

    /// Promotes one reviewed derived skill into the procedural store under explicit request policy context.
    pub fn promote_skill_to_procedural_with_context(
        &mut self,
        request_context: crate::api::RequestContext,
        pattern_handle: &str,
        accepted_by: impl Into<String>,
        acceptance_note: impl Into<String>,
    ) -> Result<ProceduralEntrySummary, ProceduralStoreError> {
        let namespace = request_context.namespace.clone().ok_or_else(|| {
            ProceduralStoreError::new(
                NamespaceId::new("default").expect("static namespace should validate"),
                pattern_handle,
                ProceduralStoreErrorReason::PolicyDenied,
            )
        })?;
        let policy = ConsolidationPolicy {
            minimum_candidates: 1,
            batch_size: 2,
            min_skill_members: 2,
            ..Default::default()
        };
        let summary = self.summarize_consolidation_pass(namespace.clone(), policy, 2);
        let candidate = summary
            .derived_artifacts
            .iter()
            .find(|artifact| {
                artifact.kind == DerivedArtifactKind::Skill
                    && Self::skill_candidate_pattern_handle(&artifact.namespace, &artifact.content)
                        == pattern_handle
            })
            .cloned()
            .ok_or_else(|| {
                ProceduralStoreError::new(
                    namespace.clone(),
                    pattern_handle,
                    ProceduralStoreErrorReason::CandidateNotFound,
                )
            })?;

        let bound = request_context
            .bind_namespace(Some(namespace.clone()))
            .expect("promotion namespace binding should succeed");
        if bound.evaluate_policy(&self.policy).decision == crate::policy::PolicyDecision::Deny {
            return Err(ProceduralStoreError::new(
                namespace,
                pattern_handle,
                ProceduralStoreErrorReason::PolicyDenied,
            ));
        }
        let sharing = bound.evaluate_sharing_access(&self.policy);
        if sharing.decision == SharingAccessDecision::Deny {
            return Err(ProceduralStoreError::new(
                namespace,
                pattern_handle,
                ProceduralStoreErrorReason::PolicyDenied,
            ));
        }

        let accepted_by = accepted_by.into();
        let acceptance_note = acceptance_note.into();
        let visibility = bound.request().policy_context.sharing_visibility;
        let detail = format!(
            "promoted procedural pattern_handle={} source_fixture={} accepted_by={} visibility={}",
            pattern_handle,
            candidate.fixture_name,
            accepted_by,
            visibility.as_str()
        );
        let audit = self.append_audit_entry(
            AuditLogEntry::new(
                AuditEventCategory::Policy,
                AuditEventKind::ApprovedSharing,
                namespace.clone(),
                PROCEDURAL_PROMOTION_ACTOR,
                detail,
            )
            .with_request_id(bound.request().request_id.as_str())
            .with_related_run("procedural-store-promotion")
            .with_tick(self.current_tick().saturating_add(1)),
        );

        let mut record = Self::procedural_record_from_candidate(
            &candidate,
            accepted_by,
            acceptance_note,
            visibility,
            audit.sequence,
            audit.kind.as_str(),
        );
        if let Some(previous) = self.procedural_entries.get(pattern_handle) {
            record.version = previous.version.saturating_add(1);
            record.promotion_audit_sequence = previous.promotion_audit_sequence;
        }
        self.procedural_entries
            .insert(record.pattern_handle.clone(), record.clone());
        Ok(Self::procedural_entry_summary(
            &record,
            sharing,
            bound.request().request_id.as_str().to_string(),
            audit,
        ))
    }

    /// Rolls back one accepted procedural entry without deleting its lineage-bearing source skill.
    pub fn rollback_procedural_entry(
        &mut self,
        namespace: NamespaceId,
        pattern_handle: &str,
        note: impl Into<String>,
    ) -> Result<ProceduralEntrySummary, ProceduralStoreError> {
        let mut record = self
            .procedural_entries
            .get(pattern_handle)
            .filter(|record| record.namespace == namespace)
            .cloned()
            .ok_or_else(|| {
                ProceduralStoreError::new(
                    namespace.clone(),
                    pattern_handle,
                    ProceduralStoreErrorReason::EntryNotFound,
                )
            })?;
        if record.state == ProceduralEntryState::RolledBack {
            return Err(ProceduralStoreError::new(
                namespace,
                pattern_handle,
                ProceduralStoreErrorReason::AlreadyRolledBack,
            ));
        }

        let rollback_note = note.into();
        let request_id = format!("rollback-procedural-{}", pattern_handle.replace("/", "-"));
        let audit = self.append_audit_entry(
            AuditLogEntry::new(
                AuditEventCategory::Archive,
                AuditEventKind::ArchiveRecorded,
                namespace.clone(),
                PROCEDURAL_ROLLBACK_ACTOR,
                format!(
                    "rolled back procedural pattern_handle={} note={}",
                    pattern_handle, rollback_note
                ),
            )
            .with_request_id(request_id.clone())
            .with_related_run("procedural-store-rollback")
            .with_tick(self.current_tick().saturating_add(1)),
        );

        record.state = ProceduralEntryState::RolledBack;
        record.version += 1;
        record.last_transition_sequence = audit.sequence;
        record.last_transition_kind = audit.kind.as_str();
        record.rollback_note = Some(rollback_note);
        self.procedural_entries
            .insert(pattern_handle.to_string(), record.clone());

        let sharing = crate::policy::SharingAccessOutcome {
            decision: SharingAccessDecision::Allow,
            policy_summary: crate::policy::PolicySummary::allow(true),
            sharing_scope: Some(SharingScope::NamespaceOnly),
            denial_reasons: Vec::new(),
            redaction_fields: Vec::new(),
        };
        Ok(Self::procedural_entry_summary(
            &record, sharing, request_id, audit,
        ))
    }

    /// Lists accepted procedural-store entries for one namespace.
    pub fn procedural_entries(&self, namespace: NamespaceId) -> Vec<ProceduralEntrySummary> {
        let request_id = format!("procedural-list-{}", namespace.as_str());
        let sharing = crate::policy::SharingAccessOutcome {
            decision: SharingAccessDecision::Allow,
            policy_summary: crate::policy::PolicySummary::allow(true),
            sharing_scope: Some(SharingScope::NamespaceOnly),
            denial_reasons: Vec::new(),
            redaction_fields: Vec::new(),
        };

        let mut entries = self
            .procedural_entries
            .values()
            .filter(|record| record.namespace == namespace)
            .cloned()
            .collect::<Vec<_>>();
        entries.sort_by(|left, right| left.pattern_handle.cmp(&right.pattern_handle));
        entries
            .into_iter()
            .map(|record| {
                let actor_source = if record.state == ProceduralEntryState::RolledBack {
                    PROCEDURAL_ROLLBACK_ACTOR
                } else {
                    PROCEDURAL_PROMOTION_ACTOR
                };
                let detail = if record.state == ProceduralEntryState::RolledBack {
                    format!(
                        "rolled back procedural pattern_handle={} note={}",
                        record.pattern_handle,
                        record.rollback_note.clone().unwrap_or_else(|| "rollback".to_string())
                    )
                } else {
                    format!(
                        "promoted procedural pattern_handle={} source_fixture={} accepted_by={} visibility={}",
                        record.pattern_handle,
                        record.source_fixture_name,
                        record.accepted_by,
                        record.visibility.as_str()
                    )
                };
                Self::procedural_entry_summary(
                    &record,
                    sharing.clone(),
                    request_id.clone(),
                    AuditLogEntry {
                        sequence: record.last_transition_sequence,
                        category: if record.state == ProceduralEntryState::RolledBack {
                            AuditEventCategory::Archive
                        } else {
                            AuditEventCategory::Policy
                        },
                        kind: if record.state == ProceduralEntryState::RolledBack {
                            AuditEventKind::ArchiveRecorded
                        } else {
                            AuditEventKind::ApprovedSharing
                        },
                        namespace: record.namespace.clone(),
                        memory_id: None,
                        session_id: None,
                        actor_source,
                        request_id: Some(request_id.clone()),
                        tick: None,
                        before_strength: None,
                        after_strength: None,
                        before_confidence: None,
                        after_confidence: None,
                        related_snapshot: None,
                        related_run: Some(if record.state == ProceduralEntryState::RolledBack {
                            "procedural-store-rollback".to_string()
                        } else {
                            "procedural-store-promotion".to_string()
                        }),
                        redacted: false,
                        detail,
                    },
                )
            })
            .collect()
    }

    /// Performs direct pattern-handle lookup against the authoritative procedural store.
    pub fn lookup_procedural_entry(
        &self,
        namespace: NamespaceId,
        pattern_handle: &str,
    ) -> Option<ProceduralEntrySummary> {
        let record = self
            .procedural_entries
            .get(pattern_handle)
            .filter(|record| record.namespace == namespace)?;
        let request_context = crate::api::RequestContext {
            namespace: Some(namespace.clone()),
            workspace_id: None,
            agent_id: None,
            session_id: None,
            task_id: None,
            request_id: crate::api::RequestId::new(format!(
                "procedural-lookup-{}",
                pattern_handle.replace("/", "-")
            ))
            .expect("procedural lookup request id"),
            policy_context: crate::api::PolicyContext {
                include_public: record.visibility == SharingVisibility::Public,
                sharing_visibility: record.visibility,
                caller_identity_bound: true,
                workspace_acl_allowed: true,
                agent_acl_allowed: true,
                session_visibility_allowed: true,
                legal_hold: false,
            },
            time_budget_ms: None,
        };
        let bound = request_context.bind_namespace(Some(namespace)).ok()?;
        let sharing = bound.evaluate_sharing_access(&self.policy);
        if sharing.decision == SharingAccessDecision::Deny {
            return None;
        }
        let event_kind = if record.state == ProceduralEntryState::RolledBack {
            AuditEventKind::ArchiveRecorded
        } else {
            AuditEventKind::ApprovedSharing
        };
        let actor_source = if record.state == ProceduralEntryState::RolledBack {
            PROCEDURAL_ROLLBACK_ACTOR
        } else {
            PROCEDURAL_PROMOTION_ACTOR
        };
        Some(Self::procedural_entry_summary(
            record,
            sharing,
            bound.request().request_id.as_str().to_string(),
            AuditLogEntry {
                sequence: record.last_transition_sequence,
                category: if record.state == ProceduralEntryState::RolledBack {
                    AuditEventCategory::Archive
                } else {
                    AuditEventCategory::Policy
                },
                kind: event_kind,
                namespace: record.namespace.clone(),
                memory_id: None,
                session_id: None,
                actor_source,
                request_id: Some(bound.request().request_id.as_str().to_string()),
                tick: None,
                before_strength: None,
                after_strength: None,
                before_confidence: None,
                after_confidence: None,
                related_snapshot: None,
                related_run: Some(if record.state == ProceduralEntryState::RolledBack {
                    "procedural-store-rollback".to_string()
                } else {
                    "procedural-store-promotion".to_string()
                }),
                redacted: false,
                detail: if record.state == ProceduralEntryState::RolledBack {
                    format!(
                        "rolled back procedural pattern_handle={} note={}",
                        record.pattern_handle,
                        record
                            .rollback_note
                            .clone()
                            .unwrap_or_else(|| "rollback".to_string())
                    )
                } else {
                    format!(
                        "promoted procedural pattern_handle={} source_fixture={} accepted_by={} visibility={}",
                        record.pattern_handle,
                        record.source_fixture_name,
                        record.accepted_by,
                        record.visibility.as_str()
                    )
                },
            },
        ))
    }

    /// Returns the authoritative procedural-store surface backed by reviewed skills.
    pub fn procedural_store_surface(
        &self,
        namespace: NamespaceId,
        policy: ConsolidationPolicy,
        estimated_groups: u32,
        extract: bool,
    ) -> ProceduralStoreOutput {
        let skill_artifacts =
            self.skill_artifacts(namespace.clone(), policy, estimated_groups, extract);
        ProceduralStoreOutput {
            namespace: namespace.as_str().to_string(),
            outcome: "accepted",
            extraction_trigger: skill_artifacts.extraction_trigger,
            reviewed_candidate_count: skill_artifacts.procedures.len(),
            procedural_count: self
                .procedural_entries
                .values()
                .filter(|record| record.namespace == namespace)
                .count(),
            direct_lookup_supported: !self.procedural_store.requires_full_recall_traversal(),
            procedures: self.procedural_entries(namespace),
        }
    }

    fn skill_artifact_summary(artifact: &DerivedArtifact) -> SkillArtifactSummary {
        let tentative = artifact.content.contains("tentative=true");
        let accepted = artifact.content.contains("accepted=true");
        let source_engram_id = artifact
            .content
            .split_whitespace()
            .find_map(|segment| segment.strip_prefix("source_engram_id="))
            .and_then(|raw| raw.parse::<u64>().ok())
            .filter(|id| *id > 0);
        let member_count = artifact
            .content
            .split_whitespace()
            .find_map(|segment| segment.strip_prefix("member_count="))
            .and_then(|raw| raw.parse::<usize>().ok())
            .unwrap_or(artifact.source_ids.len());
        let query_cues = artifact
            .content
            .split("keywords=")
            .nth(1)
            .and_then(|tail| tail.split(" citations=").next())
            .map(|raw| {
                raw.split(',')
                    .map(str::trim)
                    .filter(|value| !value.is_empty() && *value != "none")
                    .map(str::to_string)
                    .collect::<Vec<_>>()
            })
            .unwrap_or_default();

        SkillArtifactSummary {
            namespace: artifact.namespace.as_str().to_string(),
            fixture_name: artifact.fixture_name.clone(),
            content: artifact.content.clone(),
            confidence: artifact.explain.confidence,
            storage: SkillArtifactStorageView {
                storage_class: "derived_durable_artifact",
                authority_class: "derived",
                acceptance_state: if accepted { "accepted" } else { "tentative" },
                review_status: if tentative && !accepted {
                    "operator_review_required"
                } else {
                    "accepted_or_promoted"
                },
                durable: true,
                rebuildable: true,
                canonical_rebuild_source: "authoritative_memories_and_lineage",
                freshness_status: "current_generation",
                repair_status: "rebuildable_from_lineage",
            },
            review: SkillArtifactReviewView {
                derivation_rule: artifact.explain.derivation_rule,
                tentative,
                accepted,
                supporting_memory_count: artifact.source_ids.len(),
                source_citation_count: artifact.source_citations.len(),
                supporting_fields: artifact.explain.supporting_fields.clone(),
                operator_review_required: tentative && !accepted,
                review_reason: if tentative && !accepted {
                    "tentative_skill_requires_explicit_acceptance"
                } else {
                    "accepted_or_promoted"
                },
                reflection: artifact
                    .reflection
                    .as_ref()
                    .map(|reflection| ReflectionArtifactView {
                        artifact_class: reflection.primary_guidance,
                        source_outcome: reflection.source_outcome,
                        checklist_items: reflection.checklist_items.clone(),
                        advisory: reflection.advisory,
                        trusted_by_default: reflection.trusted_by_default,
                        release_rule: reflection.release_rule,
                        promotion_basis: reflection.promotion_basis,
                    }),
            },
            recall: SkillArtifactRecallView {
                recall_surface: "skills",
                retrievable_as_procedural_hint: true,
                retrieval_kind: if accepted {
                    "authoritative_procedural"
                } else {
                    "tentative_procedural_hint"
                },
                pattern_handle: Self::skill_candidate_pattern_handle(
                    &artifact.namespace,
                    &artifact.content,
                ),
                pattern_hash_hex: format!(
                    "{:016x}",
                    ProceduralStore::pattern_hash(
                        &artifact.namespace,
                        &Self::skill_candidate_pattern(&artifact.content),
                    )
                ),
                query_cues,
                source_engram_id: source_engram_id
                    .map(FieldPresence::Present)
                    .unwrap_or(FieldPresence::Absent),
                member_count,
            },
        }
    }

    pub fn skill_candidate_pattern_handle(namespace: &NamespaceId, content: &str) -> String {
        let pattern = Self::skill_candidate_pattern(content);
        ProceduralStore::pattern_handle(namespace, &pattern)
    }

    fn skill_candidate_pattern(content: &str) -> String {
        content
            .split("context=")
            .nth(1)
            .and_then(|tail| tail.split(" action_pattern=").next())
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .unwrap_or(content)
            .to_string()
    }

    fn skill_candidate_action(content: &str) -> String {
        content
            .split("action_pattern=")
            .nth(1)
            .and_then(|tail| tail.split(" keywords=").next())
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .unwrap_or("repeat the shared pattern")
            .to_string()
    }

    fn procedural_record_from_candidate(
        artifact: &DerivedArtifact,
        accepted_by: String,
        acceptance_note: String,
        visibility: SharingVisibility,
        sequence: u64,
        transition_kind: &'static str,
    ) -> ProceduralMemoryRecord {
        let pattern = Self::skill_candidate_pattern(&artifact.content);
        let action = Self::skill_candidate_action(&artifact.content);
        let pattern_handle = ProceduralStore::pattern_handle(&artifact.namespace, &pattern);
        let pattern_hash = ProceduralStore::pattern_hash(&artifact.namespace, &pattern);
        let source_engram_id = artifact
            .content
            .split_whitespace()
            .find_map(|segment| segment.strip_prefix("source_engram_id="))
            .and_then(|raw| raw.parse::<u64>().ok())
            .filter(|id| *id > 0);
        let query_cues = artifact
            .content
            .split("keywords=")
            .nth(1)
            .and_then(|tail| tail.split(" citations=").next())
            .map(|raw| {
                raw.split(',')
                    .map(str::trim)
                    .filter(|value| !value.is_empty() && *value != "none")
                    .map(str::to_string)
                    .collect::<Vec<_>>()
            })
            .unwrap_or_default();

        ProceduralMemoryRecord {
            namespace: artifact.namespace.clone(),
            pattern_handle,
            pattern_hash,
            pattern,
            action,
            confidence: artifact.explain.confidence,
            source_fixture_name: artifact.fixture_name.clone(),
            source_engram_id,
            lineage_ancestors: artifact.provenance.lineage_ancestors.clone(),
            supporting_memory_count: artifact.source_ids.len(),
            source_citation_count: artifact.source_citations.len(),
            query_cues,
            accepted_by,
            acceptance_note,
            visibility,
            state: ProceduralEntryState::Active,
            version: 1,
            promotion_audit_sequence: sequence,
            last_transition_sequence: sequence,
            last_transition_kind: transition_kind,
            rollback_note: None,
        }
    }

    fn procedural_sharing_scope(scope: Option<SharingScope>) -> FieldPresence<&'static str> {
        scope
            .map(|scope| FieldPresence::Present(scope.as_str()))
            .unwrap_or(FieldPresence::Absent)
    }

    fn procedural_entry_summary(
        record: &ProceduralMemoryRecord,
        sharing: crate::policy::SharingAccessOutcome,
        request_id: String,
        audit: AuditLogEntry,
    ) -> ProceduralEntrySummary {
        ProceduralEntrySummary {
            namespace: record.namespace.as_str().to_string(),
            pattern: record.pattern.clone(),
            action: record.action.clone(),
            confidence: record.confidence,
            storage: ProceduralEntryStorageView {
                storage_class: "procedural_durable_surface",
                authority_class: "authoritative_durable",
                durable_store: "procedural.db",
                acceptance_state: "accepted",
                lookup_strategy: "pattern_hash_exact",
                direct_lookup_without_full_recall: true,
                durable: true,
                rebuildable: false,
                canonical_rebuild_source: "snapshot_or_authoritative_procedural_copy",
                freshness_status: "current_generation",
                repair_status: if record.state == ProceduralEntryState::RolledBack {
                    "rolled_back_but_auditable"
                } else {
                    "authoritative_row"
                },
                state: record.state.as_str(),
                version: record.version,
            },
            review: ProceduralEntryReviewView {
                accepted: true,
                accepted_by: record.accepted_by.clone(),
                acceptance_reason: record.acceptance_note.clone(),
                source_fixture_name: record.source_fixture_name.clone(),
                derivation_rule: "skill_extraction",
                supporting_memory_count: record.supporting_memory_count,
                source_citation_count: record.source_citation_count,
                source_engram_id: record
                    .source_engram_id
                    .map(FieldPresence::Present)
                    .unwrap_or(FieldPresence::Absent),
                lineage_ancestors: record.lineage_ancestors.iter().map(|id| id.0).collect(),
                supersession_state: if record.state == ProceduralEntryState::RolledBack {
                    "rolled_back"
                } else {
                    "active"
                },
                rollback_capable: record.state == ProceduralEntryState::Active,
            },
            recall: ProceduralEntryRecallView {
                recall_surface: "procedural_store",
                retrieval_kind: if record.state == ProceduralEntryState::RolledBack {
                    "rolled_back_procedural"
                } else {
                    "authoritative_procedural"
                },
                pattern_handle: record.pattern_handle.clone(),
                pattern_hash_hex: format!("{:016x}", record.pattern_hash),
                query_cues: record.query_cues.clone(),
                visibility: record.visibility.as_str(),
                sharing_scope: Self::procedural_sharing_scope(sharing.sharing_scope),
                policy_outcome: sharing.policy_summary.outcome_class,
                policy_blocked_stage: if sharing.decision == SharingAccessDecision::Deny {
                    "policy_gate"
                } else {
                    "not_blocked"
                },
                policy_denial_reasons: sharing
                    .denial_reasons
                    .iter()
                    .map(|reason| reason.as_str())
                    .collect(),
            },
            audit: ProceduralEntryAuditView {
                event_kind: audit.kind.as_str(),
                actor_source: audit.actor_source,
                request_id,
                sequence: audit.sequence,
                redacted: audit.redacted,
                detail: audit.detail,
                rollback_supported: record.state == ProceduralEntryState::Active,
            },
        }
    }

    /// Returns an inspectable summary of one bounded consolidation pass.
    pub fn summarize_consolidation_pass(
        &self,
        namespace: NamespaceId,
        policy: ConsolidationPolicy,
        estimated_groups: u32,
    ) -> ConsolidationSummary {
        use crate::engine::maintenance::{MaintenanceJobHandle, MaintenanceJobState};

        let run = self
            .consolidation
            .create_run(namespace, policy, estimated_groups);
        let mut handle = MaintenanceJobHandle::new(run, estimated_groups.max(1) + 1);

        loop {
            let snapshot = handle.poll();
            match snapshot.state {
                MaintenanceJobState::Completed(summary) => return summary,
                MaintenanceJobState::Running { .. } => continue,
                other => panic!("unexpected consolidation state: {other:?}"),
            }
        }
    }

    /// Returns the shared forgetting surface owned by the core crate.
    pub fn forgetting_engine(&self) -> &ForgettingEngine {
        &self.forgetting
    }

    fn forgetting_handle(
        &self,
        namespace: NamespaceId,
        policy: ForgettingPolicy,
        estimated_candidates: u32,
    ) -> crate::engine::maintenance::MaintenanceJobHandle<crate::engine::forgetting::ForgettingRun>
    {
        let tick = self.current_tick().saturating_add(1);
        let run = self
            .forgetting
            .create_run(namespace, policy, estimated_candidates)
            .with_tick(tick);
        crate::engine::maintenance::MaintenanceJobHandle::new(run, estimated_candidates.max(1) + 1)
    }

    /// Returns an inspectable summary of one bounded forgetting pass.
    pub fn summarize_forgetting_pass(
        &self,
        namespace: NamespaceId,
        policy: ForgettingPolicy,
        estimated_candidates: u32,
    ) -> ForgettingSummary {
        use crate::engine::maintenance::MaintenanceJobState;

        let mut handle = self.forgetting_handle(namespace, policy, estimated_candidates);

        loop {
            let snapshot = handle.poll();
            match snapshot.state {
                MaintenanceJobState::Completed(summary) => return summary,
                MaintenanceJobState::Running { .. } => continue,
                other => panic!("unexpected forgetting state: {other:?}"),
            }
        }
    }

    /// Returns the reversibility breakdown for one bounded forgetting pass.
    pub fn summarize_forgetting_reversibility(
        &self,
        namespace: NamespaceId,
        policy: ForgettingPolicy,
        estimated_candidates: u32,
    ) -> ReversibilitySummary {
        use crate::engine::maintenance::MaintenanceJobState;

        let mut handle = self.forgetting_handle(namespace, policy, estimated_candidates);

        loop {
            let snapshot = handle.poll();
            match snapshot.state {
                MaintenanceJobState::Completed(_) => {
                    return self.forgetting.reversibility_summary(handle.operation());
                }
                MaintenanceJobState::Running { .. } => continue,
                other => panic!("unexpected forgetting state: {other:?}"),
            }
        }
    }

    /// Returns the operator-facing explain payload for one bounded forgetting pass.
    pub fn summarize_forgetting_explain(
        &self,
        namespace: NamespaceId,
        policy: ForgettingPolicy,
        estimated_candidates: u32,
    ) -> ForgettingExplainOutput {
        use crate::engine::maintenance::MaintenanceJobState;

        let mut handle = self.forgetting_handle(namespace, policy, estimated_candidates);

        loop {
            let snapshot = handle.poll();
            match snapshot.state {
                MaintenanceJobState::Completed(_) => {
                    return self.forgetting.explain_run(handle.operation());
                }
                MaintenanceJobState::Running { .. } => continue,
                other => panic!("unexpected forgetting state: {other:?}"),
            }
        }
    }

    /// Returns append-only audit rows for one bounded forgetting pass.
    pub fn summarize_forgetting_audit(
        &self,
        namespace: NamespaceId,
        policy: ForgettingPolicy,
        estimated_candidates: u32,
    ) -> Vec<AuditLogEntry> {
        use crate::engine::maintenance::MaintenanceJobState;

        let mut handle = self.forgetting_handle(namespace, policy, estimated_candidates);

        loop {
            let snapshot = handle.poll();
            match snapshot.state {
                MaintenanceJobState::Completed(_) => {
                    return self
                        .forgetting
                        .append_only_audit_entries(handle.operation());
                }
                MaintenanceJobState::Running { .. } => continue,
                other => panic!("unexpected forgetting state: {other:?}"),
            }
        }
    }

    fn reconsolidation_handle(
        &self,
        namespace: NamespaceId,
        policy: ReconsolidationPolicy,
        labile_memories: Vec<LabileMemory>,
    ) -> crate::engine::maintenance::MaintenanceJobHandle<ReconsolidationRun> {
        let estimated_candidates = labile_memories.len() as u32;
        let tick = self.current_tick().saturating_add(1);
        let run = ReconsolidationRun::new(namespace, policy, labile_memories, tick);
        crate::engine::maintenance::MaintenanceJobHandle::new(run, estimated_candidates.max(1) + 1)
    }

    /// Returns append-only audit rows for one bounded reconsolidation pass.
    pub fn summarize_reconsolidation_audit(
        &self,
        namespace: NamespaceId,
        policy: ReconsolidationPolicy,
        labile_memories: Vec<LabileMemory>,
    ) -> Vec<AuditLogEntry> {
        use crate::engine::maintenance::MaintenanceJobState;

        let mut handle = self.reconsolidation_handle(namespace, policy, labile_memories);

        loop {
            let snapshot = handle.poll();
            match snapshot.state {
                MaintenanceJobState::Completed(_) => {
                    return handle.operation().append_only_audit_entries();
                }
                MaintenanceJobState::Running { .. } => continue,
                other => panic!("unexpected reconsolidation state: {other:?}"),
            }
        }
    }

    /// Runs one bounded reconsolidation pass and appends the resulting audit rows into the store.
    pub fn record_reconsolidation_audit(
        &mut self,
        namespace: NamespaceId,
        policy: ReconsolidationPolicy,
        labile_memories: Vec<LabileMemory>,
    ) -> Vec<AuditLogEntry> {
        let rows = self.summarize_reconsolidation_audit(namespace, policy, labile_memories);
        rows.into_iter()
            .map(|entry| self.append_audit_entry(entry))
            .collect()
    }

    /// Runs one bounded forgetting pass and appends the resulting audit rows into the store.
    pub fn record_forgetting_audit(
        &mut self,
        namespace: NamespaceId,
        policy: ForgettingPolicy,
        estimated_candidates: u32,
    ) -> Vec<AuditLogEntry> {
        let rows = self.summarize_forgetting_audit(namespace, policy, estimated_candidates);
        rows.into_iter()
            .map(|entry| self.append_audit_entry(entry))
            .collect()
    }

    /// Returns append-only audit rows retained for one memory in canonical sequence order.
    pub fn audit_memory(&self, memory_id: MemoryId) -> Vec<AuditLogEntry> {
        self.audit_log.entries_for_memory(memory_id)
    }

    /// Returns one bounded append-only audit slice filtered by tick floor and optional operation.
    pub fn audit_range(
        &self,
        namespace: NamespaceId,
        since_tick: Option<u64>,
        op: Option<crate::observability::AuditEventKind>,
        limit: Option<usize>,
    ) -> AuditLogSlice {
        self.audit_log.slice(
            &AuditLogFilter {
                namespace: Some(namespace),
                kind: op,
                min_tick: since_tick,
                ..AuditLogFilter::default()
            },
            limit,
        )
    }

    /// Appends one audit row into the bounded write-ahead log and returns the stored sequence row.
    pub fn append_audit_entry(&mut self, entry: AuditLogEntry) -> AuditLogEntry {
        self.audit_log.append(entry)
    }

    /// Returns all retained append-only audit rows in canonical sequence order.
    pub fn audit_entries(&self) -> Vec<AuditLogEntry> {
        self.audit_log.entries()
    }

    /// Returns the shared repair surface owned by the core crate.
    pub fn repair_engine(&self) -> &RepairEngine {
        &self.repair
    }

    /// Returns the canonical hot storage surface owned by the core crate.
    pub fn hot_store(&self) -> &HotStore {
        &self.hot_store
    }

    /// Returns the stable Tier1 hot-store component identifier exposed through the core facade.
    pub fn hot_store_component_name(&self) -> &'static str {
        self.hot_store.component_name()
    }

    /// Returns the stable audit-log component identifier exposed through the core facade.
    pub fn audit_log_store_component_name(&self) -> &'static str {
        self.audit_log_store.component_name()
    }

    /// Returns the stable procedural-store component identifier exposed through the core facade.
    pub fn procedural_store_component_name(&self) -> &'static str {
        self.procedural_store.component_name()
    }

    /// Returns the canonical Tier2 storage surface owned by the core crate.
    pub fn tier2_store(&self) -> &Tier2Store {
        &self.tier2_store
    }

    /// Returns the authoritative durable Tier2 schema objects exposed by the core-owned Tier2 store.
    pub fn tier2_authoritative_schema_objects(&self) -> Vec<DurableSchemaObject> {
        self.tier2_store.authoritative_schema_objects()
    }

    /// Returns the authoritative durable procedural-store schema objects exposed by the core facade.
    pub fn procedural_authoritative_schema_objects(&self) -> Vec<DurableSchemaObject> {
        self.procedural_store.authoritative_schema_objects()
    }

    /// Returns whether migration manifests still include the full Tier2 durable-truth ownership set.
    pub fn tier2_schema_matches_migration_manifest(&self) -> bool {
        let store_objects = self.tier2_authoritative_schema_objects();
        let manifest = self.migrate.durable_schema_manifest();

        store_objects
            .iter()
            .all(|object| manifest.authoritative_tables.contains(object))
    }

    /// Returns the next logical snapshot identity without mutating stored metadata.
    pub fn next_snapshot_id(&self) -> SnapshotId {
        SnapshotId(self.next_snapshot_id)
    }

    fn inherited_count_for_visibility(&self, inherit_visibility: ForkInheritance) -> usize {
        match inherit_visibility {
            ForkInheritance::PublicOnly => 0,
            ForkInheritance::SharedToo => self
                .procedural_entries
                .values()
                .filter(|record| record.visibility == SharingVisibility::Shared)
                .count(),
            ForkInheritance::All => self.procedural_entries.len(),
        }
    }

    fn fork_local_procedure_count(&self, fork_namespace: &NamespaceId) -> usize {
        self.procedural_entries
            .values()
            .filter(|record| &record.namespace == fork_namespace)
            .count()
    }

    fn fork_working_state_count(&self, fork_namespace: &NamespaceId) -> usize {
        self.working_state
            .states()
            .filter(|state| &state.namespace == fork_namespace)
            .count()
    }

    fn fork_diverged(&self, fork_namespace: &NamespaceId) -> bool {
        self.fork_local_procedure_count(fork_namespace) > 0
            || self.fork_working_state_count(fork_namespace) > 0
    }

    fn plan_fork_merge(&self, fork: &BrainForkRecord, config: &MergeConfig) -> PlannedForkMerge {
        let mut merged_items = Vec::new();
        let mut conflict_items = Vec::new();

        let mut fork_procedures = self
            .procedural_entries
            .values()
            .filter(|record| record.namespace == fork.namespace)
            .cloned()
            .collect::<Vec<_>>();
        fork_procedures.sort_by(|left, right| left.pattern_handle.cmp(&right.pattern_handle));

        for fork_record in fork_procedures {
            let parent_handle =
                ProceduralStore::pattern_handle(&config.target_namespace, &fork_record.pattern);
            let target_record = self.procedural_entries.get(&parent_handle);
            let item_label = format!("procedural:{}", fork_record.pattern_handle);
            match target_record {
                None => merged_items.push(item_label),
                Some(target_record)
                    if target_record.action == fork_record.action
                        && target_record.confidence == fork_record.confidence
                        && target_record.state == fork_record.state =>
                {
                    merged_items.push(item_label);
                }
                Some(target_record) => {
                    let (resolution_state, preferred_side) = match config.conflict_strategy {
                        MergeConflictStrategy::Manual => ("unresolved", "manual"),
                        MergeConflictStrategy::ForkWins => ("auto_resolved", "fork"),
                        MergeConflictStrategy::ParentWins => ("auto_resolved", "parent"),
                        MergeConflictStrategy::RecencyWins => {
                            if fork_record.version >= target_record.version {
                                ("auto_resolved", "fork")
                            } else {
                                ("auto_resolved", "parent")
                            }
                        }
                    };
                    let detail = format!(
                        "procedural divergence pattern={} fork_action={} target_action={} fork_version={} target_version={}",
                        fork_record.pattern,
                        fork_record.action,
                        target_record.action,
                        fork_record.version,
                        target_record.version
                    );
                    conflict_items.push(PlannedMergeConflict {
                        auto_resolved: resolution_state == "auto_resolved",
                        preferred_side,
                        output: MergeConflictOutput {
                            item_kind: "procedural_entry",
                            item_handle: fork_record.pattern_handle.clone(),
                            target_handle: Some(target_record.pattern_handle.clone()),
                            fork_memory_id: fork_record
                                .lineage_ancestors
                                .last()
                                .map(|memory| memory.0),
                            target_memory_id: target_record
                                .lineage_ancestors
                                .last()
                                .map(|memory| memory.0),
                            conflict_kind: "contradiction_revision",
                            resolution_state,
                            preferred_side,
                            detail,
                        },
                    });
                }
            }
        }

        let mut working_states = self
            .working_state
            .states()
            .filter(|state| state.namespace == fork.namespace)
            .cloned()
            .collect::<Vec<_>>();
        working_states.sort_by(|left, right| left.task_id.as_str().cmp(right.task_id.as_str()));
        for state in working_states {
            let item_handle = state.task_id.as_str().to_string();
            let detail = format!(
                "working-state divergence task_id={} status={} active_evidence={} pending_dependencies={}",
                state.task_id.as_str(),
                state.status.as_str(),
                state.blackboard.active_evidence.len(),
                state.pending_dependencies.len()
            );
            match config.conflict_strategy {
                MergeConflictStrategy::Manual => conflict_items.push(PlannedMergeConflict {
                    auto_resolved: false,
                    preferred_side: "manual",
                    output: MergeConflictOutput {
                        item_kind: "working_state",
                        item_handle,
                        target_handle: None,
                        fork_memory_id: None,
                        target_memory_id: None,
                        conflict_kind: "working_state_pending",
                        resolution_state: "unresolved",
                        preferred_side: "manual",
                        detail,
                    },
                }),
                _ => merged_items.push(format!("working_state:{}", state.task_id.as_str())),
            }
        }

        PlannedForkMerge {
            merged_items,
            conflict_items,
        }
    }

    fn fork_info_output(&self, record: &BrainForkRecord) -> ForkInfoOutput {
        let fork_local_procedure_count = self.fork_local_procedure_count(&record.namespace);
        let fork_working_state_count = self.fork_working_state_count(&record.namespace);
        let diverged = self.fork_diverged(&record.namespace);
        ForkInfoOutput {
            name: record.name.clone(),
            namespace: record.namespace.as_str().to_string(),
            parent_namespace: record.parent_namespace.as_str().to_string(),
            inherit_visibility: record.inherit_visibility.as_str(),
            status: record.status.as_str(),
            forked_at_tick: record.forked_at_tick,
            inherited_count: record.inherited_count_at_fork,
            fork_local_procedure_count,
            fork_working_state_count,
            diverged,
            divergence_basis: "fork_namespace_local_state",
            isolation_semantics: "inherit_by_reference_until_explicit_merge",
            note: record
                .note
                .clone()
                .map(FieldPresence::Present)
                .unwrap_or(FieldPresence::Absent),
            authoritative_truth: "fork_metadata",
        }
    }

    /// Captures or reuses one governed namespace fork with inheritance-by-reference metadata.
    pub fn fork(&mut self, config: ForkConfig) -> ForkInfoOutput {
        if let Some(existing) = self.forks.get(&config.name) {
            return self.fork_info_output(existing);
        }

        let forked_at_tick = self.current_tick().saturating_add(1);
        let namespace = NamespaceId::new(config.name.clone())
            .expect("fork config name should already be a valid namespace");
        let inherited_count = self.inherited_count_for_visibility(config.inherit_visibility);
        let record = BrainForkRecord {
            name: config.name.clone(),
            namespace: namespace.clone(),
            parent_namespace: config.parent_namespace.clone(),
            forked_at_tick,
            inherit_visibility: config.inherit_visibility,
            status: ForkStatus::Active,
            merged_at_tick: None,
            inherited_count_at_fork: inherited_count,
            note: config.note.clone(),
        };
        self.forks.insert(record.name.clone(), record.clone());
        self.append_audit_entry(
            AuditLogEntry::new(
                AuditEventCategory::Policy,
                AuditEventKind::ApprovedSharing,
                namespace.clone(),
                "fork_capture",
                format!(
                    "forked namespace={} parent_namespace={} inherit_visibility={} inherited_count={} isolation_semantics=inherit_by_reference_until_explicit_merge divergence_basis=fork_namespace_local_state status={}",
                    namespace.as_str(),
                    record.parent_namespace.as_str(),
                    record.inherit_visibility.as_str(),
                    inherited_count,
                    record.status.as_str()
                ),
            )
            .with_tick(forked_at_tick),
        );
        self.fork_info_output(&record)
    }

    /// Returns all known governed namespace forks in stable fork order.
    pub fn list_forks(&self) -> Vec<ForkInfoOutput> {
        let mut forks = self
            .forks
            .values()
            .map(|record| self.fork_info_output(record))
            .collect::<Vec<_>>();
        forks.sort_by(|left, right| {
            left.forked_at_tick
                .cmp(&right.forked_at_tick)
                .then_with(|| left.name.cmp(&right.name))
        });
        forks
    }

    /// Returns one governed fork merge report and records merge lifecycle state on non-dry runs.
    pub fn merge_fork(&mut self, config: MergeConfig) -> Option<MergeReportOutput> {
        let fork = self.forks.get(&config.fork_name)?.clone();
        let fork_local_procedure_count = self.fork_local_procedure_count(&fork.namespace);
        let fork_working_state_count = self.fork_working_state_count(&fork.namespace);
        let divergence_detected = self.fork_diverged(&fork.namespace);
        let planned = self.plan_fork_merge(&fork, &config);
        let conflicts_found = planned.conflict_items.len();
        let conflicts_auto_resolved = planned
            .conflict_items
            .iter()
            .filter(|conflict| conflict.auto_resolved)
            .count();
        let conflicts_pending = conflicts_found.saturating_sub(conflicts_auto_resolved);
        let merged_count = planned.merged_items.len() + conflicts_auto_resolved;
        let tick = self.current_tick().saturating_add(1);
        let mut audit_sequences = Vec::new();

        if !config.dry_run {
            for conflict in &planned.conflict_items {
                let audit = self.append_audit_entry(
                    AuditLogEntry::new(
                        AuditEventCategory::Maintenance,
                        if conflict.auto_resolved {
                            AuditEventKind::MaintenanceReconsolidationApplied
                        } else {
                            AuditEventKind::MaintenanceReconsolidationDeferred
                        },
                        config.target_namespace.clone(),
                        FORK_MERGE_ACTOR,
                        format!(
                            "fork_merge_conflict fork={} item_kind={} item_handle={} target_handle={} preferred_side={} resolution_state={} detail={}",
                            config.fork_name,
                            conflict.output.item_kind,
                            conflict.output.item_handle,
                            conflict.output.target_handle.as_deref().unwrap_or("none"),
                            conflict.preferred_side,
                            conflict.output.resolution_state,
                            conflict.output.detail
                        ),
                    )
                    .with_tick(tick)
                    .with_related_run(format!("merge-run-{}", config.fork_name)),
                );
                audit_sequences.push(audit.sequence);
            }
            let summary_audit = self.append_audit_entry(
                AuditLogEntry::new(
                    AuditEventCategory::Maintenance,
                    AuditEventKind::MaintenanceReconsolidationApplied,
                    config.target_namespace.clone(),
                    FORK_MERGE_ACTOR,
                    format!(
                        "merged fork={} target_namespace={} merged_items={} conflicts_found={} conflicts_auto_resolved={} conflicts_pending={} strategy={} divergence_detected={} isolation_semantics=inherit_by_reference_until_explicit_merge",
                        config.fork_name,
                        config.target_namespace.as_str(),
                        planned.merged_items.join(","),
                        conflicts_found,
                        conflicts_auto_resolved,
                        conflicts_pending,
                        config.conflict_strategy.as_str(),
                        divergence_detected
                    ),
                )
                .with_tick(tick)
                .with_related_run(format!("merge-run-{}", config.fork_name)),
            );
            audit_sequences.push(summary_audit.sequence);
            if let Some(record) = self.forks.get_mut(&config.fork_name) {
                record.status = ForkStatus::Merged;
                record.merged_at_tick = Some(tick);
            }
        }

        let fork_status = if config.dry_run {
            fork.status.as_str()
        } else {
            ForkStatus::Merged.as_str()
        };
        Some(MergeReportOutput {
            fork_name: config.fork_name,
            target_namespace: config.target_namespace.as_str().to_string(),
            dry_run: config.dry_run,
            conflict_strategy: config.conflict_strategy.as_str(),
            memories_merged: merged_count,
            merged_items: planned.merged_items,
            conflicts_found,
            conflicts_auto_resolved,
            conflicts_pending,
            conflict_items: planned
                .conflict_items
                .into_iter()
                .map(|conflict| conflict.output)
                .collect(),
            engrams_merged: merged_count,
            fork_status,
            fork_local_procedure_count,
            fork_working_state_count,
            audit_sequences,
            divergence_detected,
            divergence_basis: "fork_namespace_local_state",
            isolation_semantics: "inherit_by_reference_until_explicit_merge",
            authoritative_truth: "fork_metadata",
        })
    }

    fn affect_trajectory_row_from_signals(
        &self,
        namespace: NamespaceId,
        memory_id: MemoryId,
        era_id: Option<String>,
        tick_start: u64,
        affect: AffectSignals,
    ) -> StoredAffectTrajectoryRow {
        let affect = affect.clamped();
        StoredAffectTrajectoryRow {
            namespace,
            era_id,
            memory_id,
            tick_start,
            tick_end: None,
            avg_valence: affect.valence,
            avg_arousal: affect.arousal,
            memory_count: 1,
        }
    }

    /// Appends one durable affect-trajectory row captured from encode-time signals.
    pub fn record_affect_trajectory(
        &mut self,
        namespace: NamespaceId,
        memory_id: MemoryId,
        era_id: Option<String>,
        tick_start: u64,
        affect: AffectSignals,
    ) -> AffectTrajectoryRow {
        let row = self
            .affect_trajectory_row_from_signals(namespace, memory_id, era_id, tick_start, affect);
        self.affect_trajectory.push(row.clone());
        AffectTrajectoryRow {
            namespace: row.namespace,
            era_id: row.era_id,
            memory_id: row.memory_id,
            tick_start: row.tick_start,
            tick_end: row.tick_end,
            avg_valence: row.avg_valence,
            avg_arousal: row.avg_arousal,
            memory_count: row.memory_count,
            authoritative_truth: "emotional_timeline",
        }
    }

    fn stored_compression_log_row(entry: CompressionLogEntry) -> StoredCompressionLogRow {
        StoredCompressionLogRow {
            schema_memory_id: entry.schema_memory_id,
            source_memory_count: entry.source_memory_count,
            tick: entry.tick,
            namespace: entry.namespace,
            keyword_summary: entry.keyword_summary,
        }
    }

    fn persist_compression_artifact(
        &mut self,
        namespace: &NamespaceId,
        artifact: &crate::engine::compression::CompressionSchemaArtifact,
    ) {
        let schema_tick = self.current_tick().saturating_add(1);
        let schema_envelope = NormalizedMemoryEnvelope {
            memory_type: crate::types::CanonicalMemoryType::Observation,
            source_kind: RawIntakeKind::Observation,
            raw_text: artifact.compact_text.clone(),
            compact_text: artifact.compact_text.clone(),
            normalization_generation: "compression-schema-v1",
            payload_size_bytes: artifact.compact_text.len(),
            affect: None,
            landmark: crate::types::LandmarkMetadata::non_landmark(),
            observation_source: Some("compression".to_string()),
            observation_chunk_id: Some(artifact.cluster_id.clone()),
            has_causal_parents: true,
            has_causal_children: false,
            compression: CompressionMetadata {
                compressed_into: None,
                compression_tick: Some(schema_tick),
                source_memory_ids: artifact.source_memory_ids.clone(),
            },
            sharing: SharingMetadata::default(),
        };
        let schema_layout = self.tier2_store.layout_item(
            namespace.clone(),
            artifact.schema_memory_id,
            SessionId(0),
            artifact.schema_memory_id.0,
            &schema_envelope,
            None,
            None,
        );
        self.compression_memories
            .insert(artifact.schema_memory_id, schema_layout);

        for source_memory_id in &artifact.source_memory_ids {
            let source_text = format!(
                "compressed source memory {} for schema {}",
                source_memory_id.0, artifact.schema_memory_id.0
            );
            let source_envelope = NormalizedMemoryEnvelope {
                memory_type: crate::types::CanonicalMemoryType::Observation,
                source_kind: RawIntakeKind::Observation,
                raw_text: source_text.clone(),
                compact_text: source_text,
                normalization_generation: "compression-source-v1",
                payload_size_bytes: artifact.compact_text.len(),
                affect: None,
                landmark: crate::types::LandmarkMetadata::non_landmark(),
                observation_source: Some("compression".to_string()),
                observation_chunk_id: Some(artifact.cluster_id.clone()),
                has_causal_parents: false,
                has_causal_children: true,
                compression: CompressionMetadata {
                    compressed_into: Some(artifact.schema_memory_id),
                    compression_tick: Some(schema_tick),
                    source_memory_ids: Vec::new(),
                },
                sharing: SharingMetadata::default(),
            };
            let source_layout = self.tier2_store.layout_item(
                namespace.clone(),
                *source_memory_id,
                SessionId(0),
                source_memory_id.0,
                &source_envelope,
                None,
                None,
            );
            self.compression_memories
                .insert(*source_memory_id, source_layout);
        }
    }

    pub fn compression_memory_layout(
        &self,
        memory_id: MemoryId,
    ) -> Option<&Tier2DurableItemLayout> {
        self.compression_memories.get(&memory_id)
    }

    /// Appends one durable compression-log row captured from an accepted compression apply.
    pub fn record_compression_log_entry(
        &mut self,
        entry: CompressionLogEntry,
    ) -> CompressionLogEntry {
        let row = Self::stored_compression_log_row(entry.clone());
        self.compression_log.push(row);
        entry
    }

    /// Returns the bounded compression-log history for one namespace and optional tick floor.
    pub fn compression_log_entries(
        &self,
        namespace: NamespaceId,
        since_tick: Option<u64>,
    ) -> Vec<CompressionLogEntry> {
        let mut rows = self
            .compression_log
            .iter()
            .filter(|row| row.namespace == namespace)
            .filter(|row| since_tick.is_none_or(|min_tick| row.tick >= min_tick))
            .cloned()
            .collect::<Vec<_>>();
        rows.sort_by(|left, right| {
            left.tick
                .cmp(&right.tick)
                .then_with(|| left.schema_memory_id.0.cmp(&right.schema_memory_id.0))
        });
        rows.into_iter()
            .map(|row| CompressionLogEntry {
                schema_memory_id: row.schema_memory_id,
                source_memory_count: row.source_memory_count,
                tick: row.tick,
                namespace: row.namespace,
                keyword_summary: row.keyword_summary,
            })
            .collect()
    }

    /// Returns the bounded affect-trajectory history for one namespace and optional tick floor.
    pub fn mood_history(
        &self,
        namespace: NamespaceId,
        since_tick: Option<u64>,
    ) -> AffectTrajectoryHistory {
        let mut rows = self
            .affect_trajectory
            .iter()
            .filter(|row| row.namespace == namespace)
            .filter(|row| since_tick.is_none_or(|min_tick| row.tick_start >= min_tick))
            .cloned()
            .collect::<Vec<_>>();
        rows.sort_by(|left, right| {
            left.tick_start
                .cmp(&right.tick_start)
                .then_with(|| left.memory_id.0.cmp(&right.memory_id.0))
        });
        let rows = rows
            .into_iter()
            .map(|row| AffectTrajectoryRow {
                namespace: row.namespace,
                era_id: row.era_id,
                memory_id: row.memory_id,
                tick_start: row.tick_start,
                tick_end: row.tick_end,
                avg_valence: row.avg_valence,
                avg_arousal: row.avg_arousal,
                memory_count: row.memory_count,
                authoritative_truth: "emotional_timeline",
            })
            .collect::<Vec<_>>();
        AffectTrajectoryHistory {
            namespace,
            since_tick,
            total_rows: rows.len(),
            rows,
            authoritative_truth: "emotional_timeline",
        }
    }

    /// Returns the current logical tick inferred from retained snapshot and audit metadata.
    pub fn current_tick(&self) -> u64 {
        self.snapshots
            .values()
            .map(|snapshot| snapshot.created_at_tick.max(snapshot.as_of_tick))
            .chain(self.audit_log.max_tick())
            .max()
            .unwrap_or(0)
    }

    fn retrieval_attention_signals(
        &self,
        namespace: &NamespaceId,
    ) -> HashMap<MemoryId, RetrievalAttentionSignals> {
        let mut signals = HashMap::<MemoryId, RetrievalAttentionSignals>::new();

        for entry in self.audit_log.entries_for_namespace(namespace) {
            let Some(memory_id) = entry.memory_id else {
                continue;
            };
            let signal = signals.entry(memory_id).or_default();
            match entry.kind {
                AuditEventKind::RecallServed => {
                    signal.recall_count = signal.recall_count.saturating_add(1);
                    if entry.session_id.is_some() {
                        signal.session_recall_count = signal.session_recall_count.saturating_add(1);
                    }
                    signal.last_recall_tick = match (signal.last_recall_tick, entry.tick) {
                        (Some(existing), Some(candidate)) => Some(existing.max(candidate)),
                        (None, Some(candidate)) => Some(candidate),
                        (existing, None) => existing,
                    };
                }
                AuditEventKind::RecallDenied => {
                    signal.deny_count = signal.deny_count.saturating_add(1);
                    signal.last_recall_tick = match (signal.last_recall_tick, entry.tick) {
                        (Some(existing), Some(candidate)) => Some(existing.max(candidate)),
                        (None, Some(candidate)) => Some(candidate),
                        (existing, None) => existing,
                    };
                }
                _ => {}
            }
        }

        let mut pinned_by_memory = HashSet::new();
        for state in self.working_state.states() {
            if &state.namespace != namespace {
                continue;
            }
            let task_pressure = state.blackboard.active_evidence.len();
            for handle in &state.blackboard.active_evidence {
                let signal = signals.entry(handle.memory_id).or_default();
                signal.task_pressure = signal.task_pressure.max(task_pressure);
                if handle.pinned && pinned_by_memory.insert(handle.memory_id) {
                    signal.working_set_pins = signal.working_set_pins.saturating_add(1);
                }
            }
            for memory_id in &state.selected_evidence_handles {
                let signal = signals.entry(*memory_id).or_default();
                signal.task_pressure = signal.task_pressure.max(task_pressure);
                if pinned_by_memory.insert(*memory_id) {
                    signal.working_set_pins = signal.working_set_pins.saturating_add(1);
                }
            }
        }

        signals
    }

    fn hot_path_entry_for_signals(
        &self,
        namespace: &NamespaceId,
        memory_id: MemoryId,
        signals: RetrievalAttentionSignals,
    ) -> Option<HotPathEntry> {
        let attention_score = retrieval_attention_score(&signals);
        if attention_score == 0 {
            return None;
        }
        let dominant_signal = retrieval_dominant_signal(&signals);
        let heat_bucket = retrieval_heat_bucket(attention_score, &signals);
        let (prewarm_trigger, prewarm_action, prewarm_target_family) =
            retrieval_prewarm_guidance(heat_bucket, dominant_signal, &signals);
        Some(HotPathEntry {
            namespace: namespace.as_str().to_string(),
            memory_id: memory_id.0,
            attention_score,
            recall_count: signals.recall_count,
            session_recall_count: signals.session_recall_count,
            working_set_pins: signals.working_set_pins,
            dominant_signal,
            heat_bucket,
            prewarm_trigger,
            prewarm_action,
            prewarm_target_family,
            sample_log: format!(
                "memory_id={} attention_score={} recalls={} session_recalls={} pins={} task_pressure={} denies={}",
                memory_id.0,
                attention_score,
                signals.recall_count,
                signals.session_recall_count,
                signals.working_set_pins,
                signals.task_pressure,
                signals.deny_count
            ),
        })
    }

    /// Returns one bounded hot-path report derived from retrieval events and working-state pressure.
    pub fn hot_paths(&self, namespace: NamespaceId, top_n: usize) -> HotPathsOutput {
        let mut entries = self
            .retrieval_attention_signals(&namespace)
            .into_iter()
            .filter_map(|(memory_id, signals)| {
                self.hot_path_entry_for_signals(&namespace, memory_id, signals)
            })
            .collect::<Vec<_>>();

        entries.sort_by(|left, right| {
            right
                .attention_score
                .cmp(&left.attention_score)
                .then_with(|| right.recall_count.cmp(&left.recall_count))
                .then_with(|| left.memory_id.cmp(&right.memory_id))
        });
        let total_candidates = entries.len();
        entries.truncate(top_n.max(1));

        HotPathsOutput {
            namespace: namespace.as_str().to_string(),
            top_n: top_n.max(1),
            total_candidates,
            entries,
            authoritative_truth: "durable_memory",
        }
    }

    /// Predicts the next recall-derived hot-path advisory for one memory without mutating durable state.
    pub fn predict_recall_hot_path(
        &self,
        namespace: &NamespaceId,
        memory_id: MemoryId,
        session_id: Option<SessionId>,
    ) -> Option<HotPathEntry> {
        let mut signals = self
            .retrieval_attention_signals(namespace)
            .remove(&memory_id)
            .unwrap_or_default();
        signals.recall_count = signals.recall_count.saturating_add(1);
        if session_id.is_some() {
            signals.session_recall_count = signals.session_recall_count.saturating_add(1);
        }
        signals.last_recall_tick = Some(self.current_tick().saturating_add(1));
        self.hot_path_entry_for_signals(namespace, memory_id, signals)
    }

    /// Returns one bounded dead-zone report for stale or never-reused retrieval candidates.
    pub fn dead_zones(&self, namespace: NamespaceId, min_age_ticks: u64) -> DeadZonesOutput {
        let current_tick = self.current_tick();
        let threshold = min_age_ticks.max(1);
        let mut entries = self
            .retrieval_attention_signals(&namespace)
            .into_iter()
            .filter_map(|(memory_id, signals)| {
                let ticks_since_last_recall = signals
                    .last_recall_tick
                    .map(|tick| current_tick.saturating_sub(tick));
                let stale_enough = match ticks_since_last_recall {
                    Some(age) => age >= threshold,
                    None => true,
                };
                if !stale_enough || signals.working_set_pins > 0 {
                    return None;
                }
                let (stale_reason, candidate_rewarm_family) =
                    dead_zone_reason(current_tick, threshold, &signals);
                Some(DeadZoneEntry {
                    namespace: namespace.as_str().to_string(),
                    memory_id: memory_id.0,
                    last_recall_tick: signals.last_recall_tick,
                    ticks_since_last_recall,
                    recall_count: signals.recall_count,
                    stale_reason,
                    candidate_rewarm_family,
                    sample_log: format!(
                        "memory_id={} recalls={} last_recall_tick={} ticks_since_last_recall={} denies={}",
                        memory_id.0,
                        signals.recall_count,
                        signals
                            .last_recall_tick
                            .map(|tick| tick.to_string())
                            .unwrap_or_else(|| "never".to_string()),
                        ticks_since_last_recall
                            .map(|age| age.to_string())
                            .unwrap_or_else(|| "none".to_string()),
                        signals.deny_count
                    ),
                })
            })
            .collect::<Vec<_>>();

        entries.sort_by(|left, right| {
            right
                .ticks_since_last_recall
                .unwrap_or(u64::MAX)
                .cmp(&left.ticks_since_last_recall.unwrap_or(u64::MAX))
                .then_with(|| left.recall_count.cmp(&right.recall_count))
                .then_with(|| left.memory_id.cmp(&right.memory_id))
        });
        let total_candidates = entries.len();

        DeadZonesOutput {
            namespace: namespace.as_str().to_string(),
            min_age_ticks: threshold,
            total_candidates,
            entries,
            authoritative_truth: "durable_memory",
        }
    }

    /// Returns canonical namespace-scoped attention inputs derived from live audit and working-state signals.
    pub fn attention_namespaces(&self) -> Vec<AttentionNamespaceInputs> {
        let mut namespaces = BTreeSet::new();
        for entry in self.audit_log.entries() {
            namespaces.insert(entry.namespace.as_str().to_string());
        }
        for state in self.working_state.states() {
            namespaces.insert(state.namespace.as_str().to_string());
        }

        namespaces
            .into_iter()
            .filter_map(|raw_namespace| {
                let namespace = NamespaceId::new(raw_namespace.clone()).ok()?;
                let mut recall_count = 0_u64;
                let mut encode_count = 0_u64;
                let mut promotion_count = 0_u64;
                let mut overflow_count = 0_u64;

                for entry in self.audit_log.entries_for_namespace(&namespace) {
                    match entry.kind {
                        AuditEventKind::RecallServed => {
                            recall_count = recall_count.saturating_add(1);
                        }
                        AuditEventKind::EncodeAccepted => {
                            encode_count = encode_count.saturating_add(1);
                        }
                        AuditEventKind::MaintenanceConsolidationCompleted
                        | AuditEventKind::MaintenanceConsolidationPartial => {
                            promotion_count = promotion_count.saturating_add(1);
                            if entry.detail.contains("overflow") {
                                overflow_count = overflow_count.saturating_add(1);
                            }
                        }
                        _ => {}
                    }
                }

                let working_memory_pressure = self
                    .working_state
                    .states()
                    .filter(|state| state.namespace == namespace)
                    .map(|state| state.blackboard.active_evidence.len())
                    .max()
                    .unwrap_or(0);

                if recall_count == 0
                    && encode_count == 0
                    && promotion_count == 0
                    && overflow_count == 0
                    && working_memory_pressure == 0
                {
                    return None;
                }

                Some(AttentionNamespaceInputs {
                    namespace: raw_namespace,
                    recall_count,
                    encode_count,
                    working_memory_pressure,
                    promotion_count,
                    overflow_count,
                })
            })
            .collect()
    }

    /// Registers or replaces one task-scoped working-state record.
    pub fn upsert_goal_working_state(&mut self, state: GoalWorkingState) {
        self.working_state.upsert(state);
    }

    /// Returns the current blackboard and checkpoint view for one task.
    pub fn goal_state(&self, task_id: &crate::api::TaskId) -> Option<crate::api::GoalStateOutput> {
        self.working_state.goal_state(task_id)
    }

    /// Pins one evidence handle into the task blackboard.
    pub fn blackboard_pin(
        &mut self,
        task_id: &crate::api::TaskId,
        memory_id: MemoryId,
    ) -> Option<crate::api::GoalStateOutput> {
        self.working_state.pin_evidence(task_id, memory_id)
    }

    /// Dismisses one evidence handle from the task blackboard.
    pub fn blackboard_dismiss(
        &mut self,
        task_id: &crate::api::TaskId,
        memory_id: MemoryId,
    ) -> Option<crate::api::GoalStateOutput> {
        self.working_state.dismiss_evidence(task_id, memory_id)
    }

    /// Persists the latest resumability checkpoint and marks the task dormant.
    pub fn goal_pause(
        &mut self,
        task_id: &crate::api::TaskId,
        note: Option<String>,
    ) -> Option<crate::api::GoalPauseOutput> {
        let paused = self.working_state.pause_goal(
            task_id,
            note.clone(),
            self.current_tick().saturating_add(1),
        )?;
        let task_label = match &paused.task_id {
            crate::api::FieldPresence::Present(task_id) => task_id.as_str(),
            crate::api::FieldPresence::Absent => "absent",
            crate::api::FieldPresence::Redacted => "redacted",
        };
        let note_label = match &paused.note {
            crate::api::FieldPresence::Present(note) => note.as_str(),
            crate::api::FieldPresence::Absent => "none",
            crate::api::FieldPresence::Redacted => "redacted",
        };
        self.append_audit_entry(
            AuditLogEntry::new(
                AuditEventCategory::Maintenance,
                AuditEventKind::IncidentRecorded,
                crate::api::NamespaceId::new(paused.namespace.clone())
                    .expect("goal_pause namespace should remain valid"),
                "goal_pause",
                format!(
                    "paused task={} checkpoint={} status={} note={}",
                    task_label,
                    paused.checkpoint.checkpoint_id,
                    paused.status.as_str(),
                    note_label
                ),
            )
            .with_tick(paused.paused_at_tick),
        );
        Some(paused)
    }

    /// Rehydrates one dormant goal stack from its newest valid checkpoint.
    pub fn goal_resume(
        &mut self,
        task_id: &crate::api::TaskId,
    ) -> Result<crate::api::GoalResumeOutput, crate::engine::working_state::ResumeWarning> {
        let resumed = self
            .working_state
            .resume_goal(task_id, self.current_tick().saturating_add(1))?;
        let task_label = match &resumed.task_id {
            crate::api::FieldPresence::Present(task_id) => task_id.as_str(),
            crate::api::FieldPresence::Absent => "absent",
            crate::api::FieldPresence::Redacted => "redacted",
        };
        self.append_audit_entry(
            AuditLogEntry::new(
                AuditEventCategory::Maintenance,
                AuditEventKind::IncidentRecorded,
                crate::api::NamespaceId::new(resumed.namespace.clone())
                    .expect("goal_resume namespace should remain valid"),
                "goal_resume",
                format!(
                    "resumed task={} checkpoint={} status={} warnings={}",
                    task_label,
                    resumed.checkpoint.checkpoint_id,
                    resumed.status.as_str(),
                    resumed.warnings.len()
                ),
            )
            .with_tick(resumed.resumed_at_tick),
        );
        Ok(resumed)
    }

    /// Intentionally abandons one active or dormant goal while preserving checkpoint metadata.
    pub fn goal_abandon(
        &mut self,
        task_id: &crate::api::TaskId,
        reason: Option<String>,
    ) -> Option<crate::api::GoalAbandonOutput> {
        let abandoned = self.working_state.abandon_goal(
            task_id,
            reason.clone(),
            self.current_tick().saturating_add(1),
        )?;
        let task_label = match &abandoned.task_id {
            crate::api::FieldPresence::Present(task_id) => task_id.as_str(),
            crate::api::FieldPresence::Absent => "absent",
            crate::api::FieldPresence::Redacted => "redacted",
        };
        let checkpoint_label = match &abandoned.checkpoint {
            crate::api::FieldPresence::Present(checkpoint) => checkpoint.checkpoint_id.as_str(),
            crate::api::FieldPresence::Absent => "none",
            crate::api::FieldPresence::Redacted => "redacted",
        };
        let reason_label = match &abandoned.reason {
            crate::api::FieldPresence::Present(reason) => reason.as_str(),
            crate::api::FieldPresence::Absent => "none",
            crate::api::FieldPresence::Redacted => "redacted",
        };
        self.append_audit_entry(
            AuditLogEntry::new(
                AuditEventCategory::Maintenance,
                AuditEventKind::IncidentRecorded,
                crate::api::NamespaceId::new(abandoned.namespace.clone())
                    .expect("goal_abandon namespace should remain valid"),
                "goal_abandon",
                format!(
                    "abandoned task={} checkpoint={} status={} reason={}",
                    task_label,
                    checkpoint_label,
                    abandoned.status.as_str(),
                    reason_label
                ),
            )
            .with_tick(abandoned.abandoned_at_tick),
        );
        Some(abandoned)
    }

    /// Returns the number of active snapshots in the effective namespace.
    pub fn active_snapshot_count(&self, namespace: &NamespaceId) -> usize {
        self.snapshots
            .values()
            .filter(|snapshot| snapshot.active && snapshot.namespace == *namespace)
            .count()
    }

    /// Returns the number of active restorable safety anchors in the effective namespace.
    pub fn active_restorable_snapshot_count(&self, namespace: &NamespaceId) -> usize {
        self.snapshots
            .values()
            .filter(|snapshot| {
                snapshot.active && snapshot.namespace == *namespace && snapshot.is_restorable()
            })
            .count()
    }

    /// Captures or reuses one named snapshot anchored to the current logical durable tick.
    pub fn capture_snapshot(
        &mut self,
        namespace: NamespaceId,
        snapshot_name: impl Into<String>,
        note: Option<String>,
        memory_count: u64,
        retention_class: SnapshotRetentionClass,
    ) -> SnapshotMetadata {
        let snapshot_name = snapshot_name.into();
        let key = (namespace.clone(), snapshot_name.clone());
        if let Some(snapshot_id) = self.snapshot_name_index.get(&key).copied() {
            return self
                .snapshots
                .get(&snapshot_id)
                .cloned()
                .expect("snapshot name index should resolve to stored metadata");
        }

        let as_of_tick = self.current_tick();
        let created_at_tick = as_of_tick.saturating_add(1);
        let snapshot_id = SnapshotId(self.next_snapshot_id);
        self.next_snapshot_id += 1;
        let snapshot = SnapshotMetadata::captured(
            snapshot_id,
            namespace.clone(),
            snapshot_name.clone(),
            as_of_tick,
            created_at_tick,
            note,
            memory_count,
            retention_class,
        );
        self.snapshot_name_index.insert(key, snapshot_id);
        self.snapshots.insert(snapshot_id, snapshot.clone());
        self.append_audit_entry(
            AuditLogEntry::new(
                crate::observability::AuditEventCategory::Archive,
                crate::observability::AuditEventKind::ArchiveRecorded,
                namespace,
                "snapshot_capture",
                format!(
                    "captured snapshot={} as_of_tick={} memory_count={} retention={}",
                    snapshot.snapshot_name,
                    snapshot.as_of_tick,
                    snapshot.memory_count,
                    snapshot.retention_class.as_str()
                ),
            )
            .with_tick(snapshot.created_at_tick)
            .with_related_snapshot(snapshot.snapshot_name.clone()),
        );
        snapshot
    }

    /// Returns all active snapshots in stable creation order for one namespace.
    pub fn list_snapshots(&self, namespace: &NamespaceId) -> Vec<SnapshotMetadata> {
        let mut snapshots = self
            .snapshots
            .values()
            .filter(|snapshot| snapshot.active && snapshot.namespace == *namespace)
            .cloned()
            .collect::<Vec<_>>();
        snapshots.sort_by_key(|snapshot| (snapshot.created_at_tick, snapshot.snapshot_id.0));
        snapshots
    }

    /// Resolves one active named snapshot in the effective namespace.
    pub fn get_snapshot(
        &self,
        namespace: &NamespaceId,
        snapshot_name: &str,
    ) -> Option<SnapshotMetadata> {
        self.snapshot_name_index
            .get(&(namespace.clone(), snapshot_name.to_string()))
            .and_then(|snapshot_id| self.snapshots.get(snapshot_id))
            .filter(|snapshot| snapshot.active)
            .cloned()
    }

    /// Deletes one named snapshot handle unless it is the last active restorable safety anchor.
    pub fn delete_snapshot(
        &mut self,
        namespace: &NamespaceId,
        snapshot_name: &str,
    ) -> Result<SnapshotMetadata, SnapshotDeleteError> {
        let key = (namespace.clone(), snapshot_name.to_string());
        let snapshot_id = self
            .snapshot_name_index
            .get(&key)
            .copied()
            .ok_or_else(|| SnapshotDeleteError::not_found(namespace.clone(), snapshot_name))?;
        let snapshot = self
            .snapshots
            .get(&snapshot_id)
            .cloned()
            .ok_or_else(|| SnapshotDeleteError::not_found(namespace.clone(), snapshot_name))?;
        if !snapshot.active {
            return Err(SnapshotDeleteError::not_found(
                namespace.clone(),
                snapshot_name,
            ));
        }
        if snapshot.is_restorable() && self.active_restorable_snapshot_count(namespace) <= 1 {
            return Err(SnapshotDeleteError::last_restorable_anchor(snapshot));
        }
        let deleted = snapshot.deleted();
        self.snapshots.insert(snapshot_id, deleted.clone());
        self.snapshot_name_index.remove(&key);
        let tick = self.current_tick().saturating_add(1);
        self.append_audit_entry(
            AuditLogEntry::new(
                crate::observability::AuditEventCategory::Archive,
                crate::observability::AuditEventKind::ArchiveRecorded,
                namespace.clone(),
                "snapshot_delete",
                format!(
                    "deleted snapshot={} active={} retention={}",
                    deleted.snapshot_name,
                    deleted.active,
                    deleted.retention_class.as_str()
                ),
            )
            .with_tick(tick)
            .with_related_snapshot(deleted.snapshot_name.clone()),
        );
        Ok(deleted)
    }

    fn audit_rows_between(
        &self,
        namespace: &NamespaceId,
        before_tick: u64,
        after_tick: u64,
    ) -> Vec<AuditLogEntry> {
        let range = self.audit_log.slice(
            &AuditLogFilter {
                namespace: Some(namespace.clone()),
                min_tick: Some(before_tick.saturating_add(1)),
                max_tick: Some(after_tick),
                ..AuditLogFilter::default()
            },
            None,
        );
        range.rows
    }

    /// Compares two historical tick anchors using bounded audit and snapshot evidence.
    pub fn semantic_diff_between_anchors(
        &self,
        namespace: NamespaceId,
        before_anchor: SnapshotAnchor,
        after_anchor: SnapshotAnchor,
    ) -> SemanticDiff {
        let before_tick = before_anchor.as_of_tick();
        let after_tick = after_anchor.as_of_tick();
        let mut entries = if after_tick < before_tick {
            Vec::new()
        } else {
            self.audit_rows_between(&namespace, before_tick, after_tick)
                .into_iter()
                .map(|row| SemanticDiffEntry {
                    category: semantic_diff_category_for_event(row.kind, &row.detail),
                    summary: row.detail,
                    memory_id: row.memory_id,
                    audit_kind: Some(row.kind),
                    before_anchor: before_anchor.clone(),
                    after_anchor: after_anchor.clone(),
                    unresolved: matches!(row.kind, AuditEventKind::EncodeRejected),
                })
                .collect::<Vec<_>>()
        };
        entries.sort_by_key(|entry| {
            (
                semantic_diff_category_order(entry.category),
                entry.audit_kind.map(AuditEventKind::as_str),
                entry.memory_id.map(|memory_id| memory_id.0),
                entry.summary.clone(),
            )
        });

        let mut category_counts = [
            SemanticDiffCategory::New,
            SemanticDiffCategory::Strengthened,
            SemanticDiffCategory::Weakened,
            SemanticDiffCategory::Archived,
            SemanticDiffCategory::Conflicting,
            SemanticDiffCategory::DerivedState,
        ]
        .into_iter()
        .filter_map(|category| {
            let count = entries
                .iter()
                .filter(|entry| entry.category == category)
                .count();
            (count > 0).then_some((category, count))
        })
        .collect::<Vec<_>>();
        if category_counts.is_empty() {
            category_counts.push((SemanticDiffCategory::DerivedState, 0));
        }

        SemanticDiff {
            namespace,
            before_anchor,
            after_anchor,
            category_counts,
            entries,
            caution: semantic_diff_caution(),
        }
    }

    /// Compares two named snapshots using their captured historical anchors.
    pub fn semantic_diff_between_snapshots(
        &self,
        namespace: NamespaceId,
        before_snapshot_name: &str,
        after_snapshot_name: &str,
    ) -> Option<SemanticDiff> {
        let before = self.get_snapshot(&namespace, before_snapshot_name)?;
        let after = self.get_snapshot(&namespace, after_snapshot_name)?;
        Some(self.semantic_diff_between_anchors(namespace, before.anchor(), after.anchor()))
    }

    /// Compares two explicit ticks using stable tick anchors.
    pub fn semantic_diff_between_ticks(
        &self,
        namespace: NamespaceId,
        before_tick: u64,
        after_tick: u64,
    ) -> SemanticDiff {
        self.semantic_diff_between_anchors(
            namespace,
            SnapshotAnchor::Tick {
                as_of_tick: before_tick,
            },
            SnapshotAnchor::Tick {
                as_of_tick: after_tick,
            },
        )
    }

    /// Runs the encode fast path and materializes a durable Tier2 layout that preserves additive
    /// landmark detection facts, era anchors, and metadata-first storage keys.
    pub fn prepare_tier2_layout_from_encode(
        &self,
        namespace: NamespaceId,
        memory_id: MemoryId,
        session_id: SessionId,
        input: RawEncodeInput,
    ) -> Tier2DurableItemLayout {
        let prepared = self.encode.prepare_fast_path(input);
        let observation_boost = if prepared.normalized.observation_source.is_some() {
            75
        } else {
            0
        };
        let confidence_inputs = crate::engine::confidence::ConfidenceInputs {
            corroboration_count: u32::from((prepared.provisional_salience / 250).min(4)),
            reconsolidation_count: (prepared.normalized.payload_size_bytes / 512).min(16) as u32,
            ticks_since_last_access: 0,
            age_ticks: 0,
            resolution_state: crate::engine::contradiction::ResolutionState::None,
            conflict_score: 0,
            causal_parent_count: if prepared.normalized.has_causal_parents {
                1
            } else {
                0
            },
            authoritativeness: prepared
                .provisional_salience
                .saturating_add(100)
                .saturating_add(observation_boost)
                .min(1000),
            recall_count: 0,
        };
        let confidence_output = self.confidence.compute(
            &confidence_inputs,
            &crate::engine::confidence::ConfidencePolicy::default(),
        );
        self.tier2_store.layout_item(
            namespace,
            memory_id,
            session_id,
            prepared.fingerprint,
            &prepared.normalized,
            Some(confidence_inputs),
            Some(confidence_output),
        )
    }

    /// Runs the encode fast path, materializes a durable Tier2 layout, and preserves the
    /// metadata-first prefilter trace alongside the prepared result.
    pub fn prepare_tier2_layout_with_trace_from_encode(
        &self,
        namespace: NamespaceId,
        memory_id: MemoryId,
        session_id: SessionId,
        input: RawEncodeInput,
    ) -> PreparedTier2Layout {
        let layout = self.prepare_tier2_layout_from_encode(namespace, memory_id, session_id, input);
        let prefilter_trace = layout.prefilter_trace();

        PreparedTier2Layout {
            layout,
            prefilter_trace,
        }
    }

    /// Returns the canonical Tier2 routing surface owned by the core crate.
    pub fn tier_router(&self) -> &TierRouter {
        &self.tier_router
    }

    /// Evaluates one inspectable Tier2 routing decision through the shared core facade.
    pub fn evaluate_tier_routing(&self, input: &TierRoutingInput) -> TierRoutingTrace {
        self.tier_router.evaluate_with_trace(input)
    }

    /// Returns the canonical cold storage surface owned by the core crate.
    pub fn cold_store(&self) -> &ColdStore {
        &self.cold_store
    }

    /// Returns the append-only audit-log storage surface owned by the core crate.
    pub fn audit_log_store(&self) -> &AuditLogStore {
        &self.audit_log_store
    }

    /// Returns the stable high-level audit categories accepted by the core-owned audit surface.
    pub const fn audit_event_categories(
        &self,
    ) -> &'static [crate::observability::AuditEventCategory] {
        self.audit_log_store.categories()
    }

    /// Returns the stable audit event taxonomy accepted by the core-owned audit surface.
    pub const fn audit_event_kinds(&self) -> &'static [crate::observability::AuditEventKind] {
        self.audit_log_store.event_kinds()
    }

    /// Returns the stable exportable audit taxonomy rows accepted by the core-owned audit surface.
    pub fn audit_taxonomy(&self) -> Vec<crate::store::audit::AuditTaxonomyRow> {
        self.audit_log_store.taxonomy()
    }

    /// Returns the authoritative durable audit-log schema objects exposed by the core facade.
    pub fn audit_authoritative_schema_objects(&self) -> Vec<DurableSchemaObject> {
        self.audit_log_store.authoritative_schema_objects()
    }

    /// Builds a bounded append-only audit log with the canonical default row cap.
    pub fn new_default_audit_log(&self) -> crate::store::audit::AppendOnlyAuditLog {
        self.audit_log_store.new_default_log()
    }

    /// Returns the current retained bounded write-ahead audit log snapshot.
    pub fn audit_log(&self) -> &AppendOnlyAuditLog {
        &self.audit_log
    }

    /// Returns the bounded graph surface owned by the core crate.
    pub fn graph(&self) -> &GraphModule {
        &self.graph
    }

    /// Builds a bounded causal trace by traversing explicit source-backed causal links.
    pub fn trace_causality(&self, target_memory_id: MemoryId, links: &[CausalLink]) -> CausalTrace {
        self.graph.trace_causality(
            target_memory_id,
            links,
            self.config.graph_max_depth as usize,
            self.config.graph_max_nodes,
        )
    }

    /// Builds a bounded invalidation cascade from one causal root.
    pub fn invalidate_causal_chain(
        &self,
        root_memory_id: MemoryId,
        links: &[CausalLink],
    ) -> CausalInvalidationReport {
        self.graph.invalidate_causal_chain(
            root_memory_id,
            links,
            self.config.graph_max_depth as usize,
            self.config.graph_max_nodes,
        )
    }

    /// Returns the shared index surface owned by the core crate.
    pub fn index(&self) -> &IndexModule {
        &self.index
    }

    /// Returns the shared embedding surface owned by the core crate.
    pub fn embed(&self) -> &EmbedModule {
        &self.embed
    }

    /// Returns the shared migration surface owned by the core crate.
    pub fn migrate(&self) -> &MigrationModule {
        &self.migrate
    }

    /// Returns the shared core API version expected by wrapper crates.
    pub const fn api_version() -> CoreApiVersion {
        CoreApiVersion::current()
    }
}

impl Default for BrainStore {
    fn default() -> Self {
        Self::new(RuntimeConfig::default())
    }
}

#[cfg(test)]
mod tests {
    use super::{
        semantic_diff_caution, BrainStore, ForkConfig, MergeConfig, PreparedTier2Layout,
        SnapshotDeleteErrorReason, FORK_MERGE_ACTOR,
    };
    use crate::api::{ForkInheritance, MergeConflictStrategy, NamespaceId, TaskId};
    use crate::engine::compression::CompressionPolicy;
    use crate::engine::consolidation::{ConsolidationPolicy, DerivedArtifactKind};
    use crate::engine::contradiction::{ContradictionKind, ContradictionStore};
    use crate::engine::forgetting::ForgettingPolicy;
    use crate::engine::reconsolidation::{
        LabileMemory, LabileState, PendingUpdate, PreReopenState, ReconsolidationPolicy,
        RefreshReadiness, ReopenStableState, UpdateSource,
    };
    use crate::engine::working_state::GoalWorkingState;
    use crate::migrate::DurableSchemaObject;
    use crate::observability::{AuditEventCategory, AuditEventKind, Tier1LookupOutcome};
    use crate::policy::SharingVisibility;
    use crate::store::audit::AuditLogEntry;
    use crate::store::{
        HotStoreApi, LifecycleState, ProceduralEntryState, ProceduralMemoryRecord, ProceduralStore,
        TierOwnership, TierRoutingInput, TierRoutingReason,
    };
    use crate::types::{
        AffectSignals, BlackboardEvidenceHandle, BlackboardState, CanonicalMemoryType,
        FastPathRouteFamily, GoalStackFrame, MemoryId, RawEncodeInput, RawIntakeKind,
        SemanticDiffCategory, SessionId, SnapshotId, SnapshotRetentionClass, Tier1HotRecord,
    };

    #[test]
    fn prepare_tier2_layout_from_encode_preserves_landmark_metadata() {
        let store = BrainStore::default();
        let layout = store.prepare_tier2_layout_from_encode(
            NamespaceId::new("tests/landmarks").unwrap(),
            MemoryId(77),
            SessionId(4),
            RawEncodeInput::new(RawIntakeKind::Event, "project launch deadline was moved")
                .with_affect_signals(AffectSignals::new(0.45, 0.91))
                .with_landmark_signals(crate::types::LandmarkSignals::new(0.91, 0.83, 0.31, 88)),
        );
        let landmark_record = layout.landmark_record();

        assert!(layout.metadata.landmark.is_landmark);
        assert_eq!(
            layout.metadata.landmark.landmark_label.as_deref(),
            Some("project launch deadline was moved")
        );
        assert_eq!(
            layout.metadata.landmark.era_id.as_deref(),
            Some("era-projectlaunc-0088")
        );
        assert_eq!(layout.metadata.landmark.era_started_at_tick, Some(88));
        assert_eq!(layout.metadata.landmark.detection_score, 803);
        assert!(layout
            .metadata
            .landmark
            .detection_reason
            .as_deref()
            .is_some_and(|reason| reason.contains("crossed landmark thresholds")));
        assert_eq!(landmark_record.namespace.as_str(), "tests/landmarks");
        assert_eq!(landmark_record.memory_id, MemoryId(77));
        assert!(landmark_record.is_landmark);
        assert_eq!(
            landmark_record.landmark_label.as_deref(),
            Some("project launch deadline was moved")
        );
        assert_eq!(
            landmark_record.era_id.as_deref(),
            Some("era-projectlaunc-0088")
        );
        assert_eq!(landmark_record.era_started_at_tick, Some(88));
        assert_eq!(landmark_record.detection_score, 803);
        assert!(landmark_record
            .detection_reason
            .as_deref()
            .is_some_and(|reason| reason.contains("crossed landmark thresholds")));
        assert_eq!(layout.prefilter_view().landmark, &layout.metadata.landmark);
        assert_eq!(
            layout.metadata_index_key().landmark,
            &layout.metadata.landmark
        );
        assert!(layout.metadata.confidence_inputs.is_some());
        assert!(layout.metadata.confidence_output.is_some());
        let expected_confidence = store.confidence_engine().compute(
            layout.metadata.confidence_inputs.as_ref().unwrap(),
            &crate::engine::confidence::ConfidencePolicy::default(),
        );
        assert_eq!(
            layout.metadata.confidence_output.as_ref(),
            Some(&expected_confidence)
        );
        assert_eq!(layout.prefilter_trace().payload_fetch_count, 0);
        assert_eq!(layout.metadata.affect, Some(AffectSignals::new(0.45, 0.91)));
    }

    #[test]
    fn mood_history_returns_recorded_affect_trajectory_rows() {
        let mut store = BrainStore::default();
        let namespace = NamespaceId::new("tests.affect").unwrap();
        let row = store.record_affect_trajectory(
            namespace.clone(),
            MemoryId(41),
            Some("era-affect-0001".to_string()),
            12,
            AffectSignals::new(0.35, 0.82),
        );

        assert_eq!(row.namespace, namespace);
        assert_eq!(row.memory_id, MemoryId(41));
        assert_eq!(row.era_id.as_deref(), Some("era-affect-0001"));
        assert_eq!(row.avg_valence, 0.35);
        assert_eq!(row.avg_arousal, 0.82);
        assert_eq!(row.authoritative_truth, "emotional_timeline");

        let history = store.mood_history(namespace.clone(), Some(12));
        assert_eq!(history.namespace, namespace);
        assert_eq!(history.since_tick, Some(12));
        assert_eq!(history.total_rows, 1);
        assert_eq!(history.rows.len(), 1);
        assert_eq!(history.rows[0].memory_id, MemoryId(41));
        assert_eq!(history.rows[0].authoritative_truth, "emotional_timeline");
        assert_eq!(history.authoritative_truth, "emotional_timeline");
    }

    #[test]
    fn summarize_consolidation_pass_exposes_lineage_preserving_artifacts() {
        let store = BrainStore::default();
        let summary = store.summarize_consolidation_pass(
            NamespaceId::new("tests/consolidation").unwrap(),
            ConsolidationPolicy {
                minimum_candidates: 1,
                batch_size: 2,
                min_skill_members: 2,
                ..Default::default()
            },
            2,
        );

        assert_eq!(summary.groups_evaluated, 2);
        assert_eq!(summary.episode_source_sets, 1);
        assert_eq!(summary.derivations_emitted, 5);
        assert_eq!(summary.derivation_partial_failures, 0);
        assert_eq!(summary.derived_artifacts.len(), 5);
        assert_eq!(summary.derivation_failures.len(), 0);
        assert!(summary.failure_artifacts.is_empty());
        assert_eq!(
            summary.derived_artifacts[0].kind,
            DerivedArtifactKind::Summary
        );
        assert_eq!(summary.derived_artifacts[1].kind, DerivedArtifactKind::Fact);
        assert_eq!(summary.derived_artifacts[2].kind, DerivedArtifactKind::Gist);
        assert_eq!(
            summary.derived_artifacts[3].kind,
            DerivedArtifactKind::Relation
        );
        assert_eq!(
            summary.derived_artifacts[4].kind,
            DerivedArtifactKind::Skill
        );
        assert_eq!(
            summary.derived_artifacts[0].source_ids,
            vec![MemoryId(1), MemoryId(2)]
        );
        assert_eq!(
            summary.derived_artifacts[0].source_citations,
            vec![
                crate::engine::consolidation::DerivedSourceCitation {
                    memory_id: MemoryId(1),
                    source_ref: "memory://tests/consolidation/1".to_string(),
                    timestamp_ms: 900_000,
                    evidence_kind: "durable_memory",
                },
                crate::engine::consolidation::DerivedSourceCitation {
                    memory_id: MemoryId(2),
                    source_ref: "memory://tests/consolidation/2".to_string(),
                    timestamp_ms: 1_020_000,
                    evidence_kind: "durable_memory",
                }
            ]
        );
        assert_eq!(
            summary.derived_artifacts[0].continuity_keys,
            vec![
                "session_cluster",
                "session_id",
                "task_id",
                "goal_context",
                "tool_chain_context",
                "entity_overlap"
            ]
        );
        assert_eq!(
            summary.derived_artifacts[0].provenance.source_kind,
            "consolidation"
        );
        assert_eq!(
            summary.derived_artifacts[0].provenance.source_ref,
            "consolidation://tests/consolidation/episode_1_session_cluster_1_2#summary"
        );
        assert_eq!(
            summary.derived_artifacts[0].explain.derivation_rule,
            "episode_summary"
        );
        assert_eq!(summary.derived_artifacts[0].explain.status, "complete");
        assert!(summary.derived_artifacts[0].explain.confidence >= 700);
        assert_eq!(
            summary.derived_artifacts[0].provenance.derived_from,
            vec![MemoryId(1), MemoryId(2)]
        );
        assert_eq!(
            summary.derived_artifacts[0].provenance.lineage_ancestors,
            vec![MemoryId(1), MemoryId(2)]
        );
        assert!(summary.derived_artifacts[0]
            .content
            .contains("summary(session_cluster)"));
        assert!(summary.derived_artifacts[1]
            .content
            .contains("fact(session_cluster)"));
        assert_eq!(
            summary.derived_artifacts[1].explain.derivation_rule,
            "fact_extraction"
        );
        assert_eq!(summary.derived_artifacts[1].explain.status, "complete");
        assert!(summary.derived_artifacts[3].content.contains(
            "relation(session_cluster): namespace=tests/consolidation relation=shared_topic"
        ));
        assert!(summary.derived_artifacts[3]
            .content
            .contains("status=reinforced"));
        assert_eq!(
            summary.derived_artifacts[3].explain.derivation_rule,
            "relation_reinforcement"
        );
        assert_eq!(summary.derived_artifacts[3].explain.status, "reinforced");
        assert_eq!(
            summary.derived_artifacts[4].explain.derivation_rule,
            "skill_extraction"
        );
        assert_eq!(summary.derived_artifacts[4].explain.status, "tentative");
        assert!(summary.derived_artifacts[4].content.contains(
            "skill(session_cluster): namespace=tests/consolidation source_engram_id=11 confidence="
        ));
        assert!(summary.derived_artifacts[4]
            .content
            .contains("tentative=true accepted=false"));
        let reflection = summary.derived_artifacts[4]
            .reflection
            .as_ref()
            .expect("reflection compiler metadata present");
        assert_eq!(reflection.primary_guidance, "procedure");
        assert_eq!(reflection.source_outcome, "successful_episode");
        assert!(reflection.advisory);
        assert!(!reflection.trusted_by_default);
        assert!(!reflection.checklist_items.is_empty());
        assert_eq!(
            reflection.release_rule,
            "explicit_acceptance_or_repeated_use_with_lineage"
        );
        assert!(summary.derivation_failures.is_empty());
    }

    #[test]
    fn skill_artifacts_surface_storage_review_and_recall_semantics() {
        let store = BrainStore::default();
        let result = store.skill_artifacts(
            NamespaceId::new("tests/skills").unwrap(),
            ConsolidationPolicy {
                minimum_candidates: 1,
                batch_size: 2,
                min_skill_members: 2,
                ..Default::default()
            },
            2,
            true,
        );

        assert_eq!(result.namespace, "tests/skills");
        assert_eq!(result.extraction_trigger, "explicit_skill_extraction");
        assert_eq!(result.extracted_count, 1);
        assert_eq!(result.skipped_count, 1);
        assert_eq!(result.procedures.len(), 1);

        let procedure = &result.procedures[0];
        assert_eq!(procedure.storage.storage_class, "derived_durable_artifact");
        assert_eq!(procedure.storage.authority_class, "derived");
        assert_eq!(procedure.storage.acceptance_state, "tentative");
        assert_eq!(procedure.storage.review_status, "operator_review_required");
        assert!(procedure.storage.durable);
        assert!(procedure.storage.rebuildable);
        assert_eq!(
            procedure.storage.canonical_rebuild_source,
            "authoritative_memories_and_lineage"
        );
        assert_eq!(procedure.review.derivation_rule, "skill_extraction");
        assert!(procedure.review.tentative);
        assert!(!procedure.review.accepted);
        assert!(procedure.review.operator_review_required);
        assert_eq!(
            procedure.review.review_reason,
            "tentative_skill_requires_explicit_acceptance"
        );
        let reflection = procedure
            .review
            .reflection
            .as_ref()
            .expect("reflection compiler metadata present");
        assert_eq!(reflection.artifact_class, "procedure");
        assert_eq!(reflection.source_outcome, "successful_episode");
        assert!(reflection.advisory);
        assert!(!reflection.trusted_by_default);
        assert_eq!(
            reflection.release_rule,
            "explicit_acceptance_or_repeated_use_with_lineage"
        );
        assert_eq!(
            reflection.promotion_basis,
            "human_approval_or_repeated_usefulness"
        );
        assert!(!reflection.checklist_items.is_empty());
        assert_eq!(procedure.recall.recall_surface, "skills");
        assert!(procedure.recall.retrievable_as_procedural_hint);
        assert_eq!(procedure.recall.retrieval_kind, "tentative_procedural_hint");
        assert_eq!(
            procedure.recall.source_engram_id,
            crate::api::FieldPresence::Present(11)
        );
        assert!(procedure
            .recall
            .pattern_handle
            .starts_with("procedural://tests/skills/"));
        assert!(!procedure.recall.pattern_hash_hex.is_empty());
        assert_eq!(procedure.recall.member_count, 2);
        assert!(!procedure.recall.query_cues.is_empty());
    }

    #[test]
    fn procedural_store_promotes_reviewed_skill_into_authoritative_lookup_surface() {
        let namespace = NamespaceId::new("tests/procedural-store").unwrap();
        let mut store = BrainStore::default();
        let candidate = store.skill_artifacts(
            namespace.clone(),
            ConsolidationPolicy {
                minimum_candidates: 1,
                batch_size: 2,
                min_skill_members: 2,
                ..Default::default()
            },
            2,
            true,
        );
        let pattern_handle = BrainStore::skill_candidate_pattern_handle(
            &namespace,
            &candidate.procedures[0].content,
        );

        let promoted = store
            .promote_skill_to_procedural(
                namespace.clone(),
                &pattern_handle,
                "operator.alice",
                "approved after review",
                false,
            )
            .expect("promotion should succeed");

        assert_eq!(promoted.namespace, "tests/procedural-store");
        assert_eq!(promoted.storage.storage_class, "procedural_durable_surface");
        assert_eq!(promoted.storage.durable_store, "procedural.db");
        assert_eq!(promoted.storage.lookup_strategy, "pattern_hash_exact");
        assert!(promoted.storage.direct_lookup_without_full_recall);
        assert!(!promoted.storage.rebuildable);
        assert_eq!(promoted.review.accepted_by, "operator.alice");
        assert_eq!(promoted.review.acceptance_reason, "approved after review");
        assert_eq!(promoted.review.derivation_rule, "skill_extraction");
        assert_eq!(promoted.recall.recall_surface, "procedural_store");
        assert_eq!(promoted.recall.retrieval_kind, "authoritative_procedural");
        assert_eq!(promoted.recall.visibility, "shared");
        assert_eq!(promoted.audit.event_kind, "approved_sharing");
        assert!(promoted.audit.rollback_supported);

        let direct = store
            .lookup_procedural_entry(namespace.clone(), &pattern_handle)
            .expect("direct lookup should succeed");
        assert_eq!(direct.recall.pattern_handle, pattern_handle);
        assert_eq!(direct.audit.sequence, promoted.audit.sequence);
        assert_eq!(direct.storage.state, "active");

        let surface = store.procedural_store_surface(
            namespace.clone(),
            ConsolidationPolicy {
                minimum_candidates: 1,
                batch_size: 2,
                min_skill_members: 2,
                ..Default::default()
            },
            2,
            false,
        );
        assert_eq!(surface.reviewed_candidate_count, 1);
        assert_eq!(surface.procedural_count, 1);
        assert!(surface.direct_lookup_supported);
        assert_eq!(surface.procedures.len(), 1);
    }

    #[test]
    fn procedural_store_rollback_keeps_lineage_and_marks_entry_non_active() {
        let namespace = NamespaceId::new("tests/procedural-rollback").unwrap();
        let mut store = BrainStore::default();
        let candidate = store.skill_artifacts(
            namespace.clone(),
            ConsolidationPolicy {
                minimum_candidates: 1,
                batch_size: 2,
                min_skill_members: 2,
                ..Default::default()
            },
            2,
            true,
        );
        let pattern_handle = BrainStore::skill_candidate_pattern_handle(
            &namespace,
            &candidate.procedures[0].content,
        );
        store
            .promote_skill_to_procedural(
                namespace.clone(),
                &pattern_handle,
                "operator.bob",
                "approved for rollout",
                true,
            )
            .expect("promotion should succeed");

        let rolled_back = store
            .rollback_procedural_entry(
                namespace.clone(),
                &pattern_handle,
                "superseded by new runbook",
            )
            .expect("rollback should succeed");

        assert_eq!(rolled_back.storage.state, "rolled_back");
        assert_eq!(rolled_back.storage.version, 2);
        assert_eq!(rolled_back.review.supersession_state, "rolled_back");
        assert!(!rolled_back.review.rollback_capable);
        assert_eq!(rolled_back.recall.retrieval_kind, "rolled_back_procedural");
        assert_eq!(rolled_back.audit.event_kind, "archive_recorded");
        assert!(!rolled_back.audit.rollback_supported);

        let listed = store.procedural_entries(namespace.clone());
        assert_eq!(listed.len(), 1);
        assert_eq!(
            listed[0].storage.state,
            ProceduralEntryState::RolledBack.as_str()
        );
        assert_eq!(listed[0].review.lineage_ancestors.len(), 2);
    }

    #[test]
    fn procedural_store_schema_is_reflected_in_core_and_migration_manifests() {
        let store = BrainStore::default();
        let schema_objects = store.procedural_authoritative_schema_objects();

        assert_eq!(
            schema_objects,
            vec![
                DurableSchemaObject::ProceduralMemoriesTable,
                DurableSchemaObject::ProceduralLineageTable,
            ]
        );
        assert!(store
            .migrate()
            .durable_schema_manifest()
            .authoritative_tables
            .contains(&DurableSchemaObject::ProceduralMemoriesTable));
        assert!(store
            .migrate()
            .durable_schema_manifest()
            .authoritative_tables
            .contains(&DurableSchemaObject::ProceduralLineageTable));
    }

    #[test]
    fn procedural_store_rejects_missing_candidate_pattern_handle() {
        let namespace = NamespaceId::new("tests/procedural-denial").unwrap();
        let mut store = BrainStore::default();

        let error = store
            .promote_skill_to_procedural(
                namespace.clone(),
                "procedural://tests/procedural-denial/missing",
                "operator.carol",
                "no such candidate",
                false,
            )
            .expect_err("missing candidate should fail");

        assert_eq!(error.namespace, namespace);
        assert_eq!(error.reason.as_str(), "candidate_not_found");
    }

    #[test]
    fn summarize_consolidation_pass_respects_minimum_candidate_gate() {
        let store = BrainStore::default();
        let summary = store.summarize_consolidation_pass(
            NamespaceId::new("tests/consolidation-gated").unwrap(),
            ConsolidationPolicy {
                minimum_candidates: 3,
                batch_size: 2,
                ..Default::default()
            },
            2,
        );

        assert_eq!(summary.groups_evaluated, 0);
        assert_eq!(summary.episode_source_sets, 0);
        assert_eq!(summary.derivations_emitted, 0);
        assert!(summary.derived_artifacts.is_empty());
        assert!(summary.derivation_failures.is_empty());
        assert!(summary.failure_artifacts.is_empty());
    }

    #[test]
    fn summarize_compression_pass_exposes_candidate_lineage_and_counts() {
        let store = BrainStore::default();
        let summary = store.summarize_compression_pass(
            NamespaceId::new("tests/compression").unwrap(),
            CompressionPolicy {
                min_episode_count: 3,
                batch_size: 2,
                ..Default::default()
            },
            3,
        );

        assert_eq!(summary.clusters_evaluated, 3);
        assert_eq!(summary.eligible_clusters, 1);
        assert_eq!(summary.review_clusters, 1);
        assert_eq!(summary.blocked_clusters, 1);
        assert_eq!(summary.inspect.candidate_count, 3);
        assert_eq!(
            summary.inspect.run_handle,
            "compression://tests/compression/manual@0"
        );
        assert!(summary
            .inspect
            .inspect_path
            .contains("compression://tests/compression/manual@0"));
        assert!(summary
            .inspect
            .outputs
            .iter()
            .any(|decision| decision.authoritative_truth == "durable_memory"));
    }

    #[test]
    fn apply_compression_pass_returns_schema_artifact_for_first_eligible_cluster() {
        let mut store = BrainStore::default();
        let namespace = NamespaceId::new("tests/compression").unwrap();
        let applied = store.apply_compression_pass(
            namespace.clone(),
            CompressionPolicy {
                min_episode_count: 3,
                ..Default::default()
            },
            3,
            false,
        );

        assert_eq!(applied.decision.disposition.as_str(), "eligible");
        assert_eq!(applied.schemas_created, 1);
        assert!(applied.episodes_compressed > 0);
        assert!(applied.storage_reduction_pct > 0);
        assert!(applied.blocked_reasons.is_empty());
        assert!(applied.schema_artifact.is_some());
        assert!(applied
            .verification
            .as_ref()
            .is_some_and(|report| report.verified));
        let artifact = applied.schema_artifact.as_ref().expect("schema artifact");
        let schema_layout = store
            .compression_memory_layout(artifact.schema_memory_id)
            .expect("schema layout persisted");
        assert_eq!(
            schema_layout.metadata.compression.source_memory_ids,
            artifact.source_memory_ids
        );
        assert!(schema_layout.metadata.compression.compressed_into.is_none());
        for source_memory_id in &artifact.source_memory_ids {
            let source_layout = store
                .compression_memory_layout(*source_memory_id)
                .expect("source layout persisted");
            assert_eq!(
                source_layout.metadata.compression.compressed_into,
                Some(artifact.schema_memory_id)
            );
            assert!(source_layout
                .metadata
                .compression
                .source_memory_ids
                .is_empty());
        }
        assert_eq!(store.compression_log_entries(namespace, None).len(), 1);
    }

    #[test]
    fn apply_compression_pass_persists_compression_log_entry_for_namespace() {
        let mut store = BrainStore::default();
        let namespace = NamespaceId::new("tests/compression-log").unwrap();
        let applied = store.apply_compression_pass(
            namespace.clone(),
            CompressionPolicy {
                min_episode_count: 3,
                ..Default::default()
            },
            3,
            false,
        );

        let entries = store.compression_log_entries(namespace.clone(), None);
        assert_eq!(entries.len(), 1);
        let entry = &entries[0];
        let applied_entry = applied
            .compression_log_entry
            .as_ref()
            .expect("compression log entry");
        assert_eq!(entry, applied_entry);
        assert_eq!(entry.namespace, namespace);
        assert!(entry.keyword_summary.as_deref().is_some());
    }

    #[test]
    fn dry_run_compression_pass_does_not_persist_compression_log_entry() {
        let mut store = BrainStore::default();
        let namespace = NamespaceId::new("tests/compression-log-dry-run").unwrap();
        let applied = store.apply_compression_pass(
            namespace.clone(),
            CompressionPolicy {
                min_episode_count: 3,
                ..Default::default()
            },
            3,
            true,
        );

        assert!(applied.compression_log_entry.is_some());
        assert!(store.compression_log_entries(namespace, None).is_empty());
    }

    #[test]
    fn summarize_forgetting_pass_exposes_archive_not_delete_semantics() {
        let store = BrainStore::default();
        let summary = store.summarize_forgetting_pass(
            NamespaceId::new("tests/forgetting").unwrap(),
            ForgettingPolicy {
                batch_size: 2,
                ..Default::default()
            },
            2,
        );

        assert_eq!(summary.evaluated, 2);
        assert_eq!(summary.archived, 0);
        assert_eq!(summary.demoted, 0);
        assert_eq!(summary.policy_deleted, 0);
        assert_eq!(summary.skipped, 2);
        assert_eq!(summary.review_required, 0);
        assert_eq!(summary.audit_records.len(), 2);
        assert_eq!(
            summary.audit_records[0].action,
            crate::engine::forgetting::ForgettingAction::Skip
        );
        assert_eq!(
            summary.audit_records[0].audit_kind,
            "maintenance_forgetting_evaluated"
        );
    }

    #[test]
    fn summarize_forgetting_reversibility_reports_archive_restore_requirement() {
        let store = BrainStore::default();
        let reversibility = store.summarize_forgetting_reversibility(
            NamespaceId::new("tests/forgetting-reversibility").unwrap(),
            ForgettingPolicy {
                batch_size: 1,
                ..Default::default()
            },
            1,
        );

        assert_eq!(reversibility.total, 1);
        assert_eq!(reversibility.reversible, 0);
        assert_eq!(reversibility.restore_required, 0);
        assert_eq!(reversibility.irreversible, 0);
        assert_eq!(reversibility.no_action, 1);
        assert_eq!(reversibility.review_required, 0);
    }

    #[test]
    fn summarize_forgetting_reversibility_reports_restore_required_for_archive_candidates() {
        let store = BrainStore::default();
        let reversibility = store.summarize_forgetting_reversibility(
            NamespaceId::new("tests/forgetting-archive-reversibility").unwrap(),
            ForgettingPolicy {
                forget_score_threshold: 600,
                batch_size: 1,
                ..Default::default()
            },
            1,
        );

        assert_eq!(reversibility.total, 1);
        assert_eq!(reversibility.reversible, 0);
        assert_eq!(reversibility.restore_required, 1);
        assert_eq!(reversibility.irreversible, 0);
        assert_eq!(reversibility.no_action, 0);
        assert_eq!(reversibility.review_required, 0);
    }

    #[test]
    fn summarize_forgetting_explain_exposes_operator_review_fields() {
        let store = BrainStore::default();
        let explain = store.summarize_forgetting_explain(
            NamespaceId::new("tests/forgetting-explain").unwrap(),
            ForgettingPolicy {
                batch_size: 1,
                ..Default::default()
            },
            1,
        );

        assert_eq!(explain.evaluated, 1);
        assert_eq!(explain.entries.len(), 1);
        assert_eq!(explain.entries[0].reason_code, "ineligible_or_retained");
        assert_eq!(explain.entries[0].disposition, "ineligible");
        assert_eq!(
            explain.entries[0].audit_kind,
            "maintenance_forgetting_evaluated"
        );
        assert_eq!(explain.review_count, 0);
    }

    #[test]
    fn summarize_forgetting_audit_returns_append_only_rows() {
        let store = BrainStore::default();
        let rows = store.summarize_forgetting_audit(
            NamespaceId::new("tests/forgetting-audit").unwrap(),
            ForgettingPolicy {
                forget_score_threshold: 600,
                batch_size: 1,
                ..Default::default()
            },
            1,
        );

        assert_eq!(rows.len(), 1);
        assert_eq!(
            rows[0].kind,
            crate::observability::AuditEventKind::MaintenanceForgettingEvaluated
        );
        assert_eq!(rows[0].memory_id, Some(MemoryId(1)));
        assert!(rows[0].detail.contains("action=archive"));
        assert!(rows[0]
            .detail
            .contains("reason_code=below_forget_threshold"));
    }

    #[test]
    fn summarize_reconsolidation_audit_returns_append_only_rows() {
        let store = BrainStore::default();
        let namespace = NamespaceId::new("tests/reconsolidation-audit").unwrap();
        let rows = store.summarize_reconsolidation_audit(
            namespace,
            ReconsolidationPolicy::default(),
            vec![
                LabileMemory {
                    memory_id: MemoryId(1),
                    labile_state: LabileState::new(100, 50),
                    pending_update: Some(
                        PendingUpdate::new(MemoryId(1), 105, UpdateSource::User)
                            .with_content("apply me".to_string()),
                    ),
                    current_strength: 0.6,
                    pre_reopen_state: PreReopenState {
                        memory_id: MemoryId(1),
                        reopen_tick: 100,
                        strength_at_reopen: 0.55,
                        stability_at_reopen: 3.0,
                        access_count_at_reopen: 5,
                    },
                    restabilize_to: ReopenStableState::Consolidated,
                    refresh_readiness: RefreshReadiness::Ready,
                },
                LabileMemory {
                    memory_id: MemoryId(2),
                    labile_state: LabileState::new(100, 10),
                    pending_update: Some(
                        PendingUpdate::new(MemoryId(2), 105, UpdateSource::System)
                            .with_content("stale".to_string()),
                    ),
                    current_strength: 0.8,
                    pre_reopen_state: PreReopenState {
                        memory_id: MemoryId(2),
                        reopen_tick: 100,
                        strength_at_reopen: 0.75,
                        stability_at_reopen: 5.0,
                        access_count_at_reopen: 7,
                    },
                    restabilize_to: ReopenStableState::Consolidated,
                    refresh_readiness: RefreshReadiness::Ready,
                },
                LabileMemory {
                    memory_id: MemoryId(3),
                    labile_state: LabileState::new(100, 50),
                    pending_update: Some(
                        PendingUpdate::new(MemoryId(3), 105, UpdateSource::Agent)
                            .with_content("wait".to_string()),
                    ),
                    current_strength: 0.7,
                    pre_reopen_state: PreReopenState {
                        memory_id: MemoryId(3),
                        reopen_tick: 100,
                        strength_at_reopen: 0.65,
                        stability_at_reopen: 4.0,
                        access_count_at_reopen: 6,
                    },
                    restabilize_to: ReopenStableState::SynapticDone,
                    refresh_readiness: RefreshReadiness::Deferred,
                },
                LabileMemory {
                    memory_id: MemoryId(4),
                    labile_state: LabileState::new(100, 50),
                    pending_update: Some(
                        PendingUpdate::new(MemoryId(4), 105, UpdateSource::Agent)
                            .with_content("blocked".to_string()),
                    ),
                    current_strength: 0.7,
                    pre_reopen_state: PreReopenState {
                        memory_id: MemoryId(4),
                        reopen_tick: 100,
                        strength_at_reopen: 0.65,
                        stability_at_reopen: 4.0,
                        access_count_at_reopen: 6,
                    },
                    restabilize_to: ReopenStableState::SynapticDone,
                    refresh_readiness: RefreshReadiness::Failed,
                },
            ],
        );

        assert_eq!(rows.len(), 4);
        assert_eq!(
            rows[0].kind,
            crate::observability::AuditEventKind::MaintenanceReconsolidationApplied
        );
        assert_eq!(
            rows[1].kind,
            crate::observability::AuditEventKind::MaintenanceReconsolidationApplied
        );
        assert_eq!(
            rows[2].kind,
            crate::observability::AuditEventKind::MaintenanceReconsolidationDeferred
        );
        assert_eq!(
            rows[3].kind,
            crate::observability::AuditEventKind::MaintenanceReconsolidationBlocked
        );
        assert!(rows
            .iter()
            .all(|row| row.actor_source == "reconsolidation_run"));
        assert!(rows[0].detail.contains("reconsolidation applied"));
    }

    #[test]
    fn record_reconsolidation_audit_appends_rows_into_store_history() {
        let namespace = NamespaceId::new("tests/reconsolidation-record").unwrap();
        let mut store = BrainStore::default();

        let recorded = store.record_reconsolidation_audit(
            namespace.clone(),
            ReconsolidationPolicy::default(),
            vec![LabileMemory {
                memory_id: MemoryId(41),
                labile_state: LabileState::new(100, 50),
                pending_update: Some(
                    PendingUpdate::new(MemoryId(41), 105, UpdateSource::User)
                        .with_content("apply me".to_string()),
                ),
                current_strength: 0.6,
                pre_reopen_state: PreReopenState {
                    memory_id: MemoryId(41),
                    reopen_tick: 100,
                    strength_at_reopen: 0.55,
                    stability_at_reopen: 3.0,
                    access_count_at_reopen: 5,
                },
                restabilize_to: ReopenStableState::Consolidated,
                refresh_readiness: RefreshReadiness::Ready,
            }],
        );

        assert_eq!(recorded.len(), 1);
        let row = &recorded[0];
        assert_eq!(
            row.kind,
            crate::observability::AuditEventKind::MaintenanceReconsolidationApplied
        );
        assert_eq!(row.memory_id, Some(MemoryId(41)));
        assert_eq!(store.audit_memory(MemoryId(41)), recorded);

        let range = store.audit_range(
            namespace,
            Some(1),
            Some(crate::observability::AuditEventKind::MaintenanceReconsolidationApplied),
            Some(8),
        );
        assert_eq!(range.total_matches, 1);
        assert_eq!(range.rows, vec![row.clone()]);
    }

    #[test]
    fn brain_store_hot_paths_and_dead_zones_follow_bounded_attention_signals() {
        let namespace = NamespaceId::new("tests/f13-hot-paths").unwrap();
        let mut store = BrainStore::default();

        store.append_audit_entry(
            AuditLogEntry::new(
                crate::observability::AuditEventCategory::Recall,
                crate::observability::AuditEventKind::RecallServed,
                namespace.clone(),
                "recall_engine",
                "served exact hot path",
            )
            .with_memory_id(MemoryId(11))
            .with_session_id(SessionId(7))
            .with_request_id("req-recall-11a")
            .with_tick(10),
        );
        store.append_audit_entry(
            AuditLogEntry::new(
                crate::observability::AuditEventCategory::Recall,
                crate::observability::AuditEventKind::RecallServed,
                namespace.clone(),
                "recall_engine",
                "served exact hot path again",
            )
            .with_memory_id(MemoryId(11))
            .with_session_id(SessionId(7))
            .with_request_id("req-recall-11b")
            .with_tick(14),
        );
        store.append_audit_entry(
            AuditLogEntry::new(
                crate::observability::AuditEventCategory::Recall,
                crate::observability::AuditEventKind::RecallDenied,
                namespace.clone(),
                "recall_engine",
                "policy denied deeper recall",
            )
            .with_memory_id(MemoryId(22))
            .with_request_id("req-recall-22")
            .with_tick(8),
        );
        store.append_audit_entry(
            AuditLogEntry::new(
                crate::observability::AuditEventCategory::Recall,
                crate::observability::AuditEventKind::RecallServed,
                namespace.clone(),
                "recall_engine",
                "old cold recall",
            )
            .with_memory_id(MemoryId(33))
            .with_request_id("req-recall-33")
            .with_tick(2),
        );

        let mut blackboard = BlackboardState::new(
            namespace.clone(),
            Some(TaskId::new("heat-task")),
            Some(SessionId(7)),
            "investigate hot path",
        );
        blackboard.active_evidence =
            vec![BlackboardEvidenceHandle::new(MemoryId(11), "primary").pinned()];
        let mut state = GoalWorkingState::new(
            TaskId::new("heat-task"),
            namespace.clone(),
            Some(SessionId(7)),
            vec![GoalStackFrame::new("investigate hot path")],
            blackboard,
        );
        state.selected_evidence_handles = vec![MemoryId(11)];
        store.upsert_goal_working_state(state);

        let hot_paths = store.hot_paths(namespace.clone(), 3);
        assert_eq!(hot_paths.namespace, namespace.as_str());
        assert_eq!(hot_paths.authoritative_truth, "durable_memory");
        assert_eq!(hot_paths.total_candidates, 3);
        assert_eq!(hot_paths.entries[0].memory_id, 11);
        assert_eq!(hot_paths.entries[0].heat_bucket, "warming");
        assert_eq!(
            hot_paths.entries[0].prewarm_action,
            "bounded_session_rewarm"
        );
        assert_eq!(hot_paths.entries[0].prewarm_target_family, "session_warmup");
        assert!(hot_paths.entries[0].attention_score > hot_paths.entries[1].attention_score);

        let dead_zones = store.dead_zones(namespace.clone(), 5);
        assert_eq!(dead_zones.namespace, namespace.as_str());
        assert_eq!(dead_zones.authoritative_truth, "durable_memory");
        assert!(dead_zones.entries.iter().any(|entry| entry.memory_id == 33));
        let cold = dead_zones
            .entries
            .iter()
            .find(|entry| entry.memory_id == 33)
            .expect("cold memory should appear in dead zones");
        assert_eq!(cold.stale_reason, "single_touch_stale");
        assert_eq!(cold.candidate_rewarm_family, "prefetch_hints");
        assert_eq!(cold.ticks_since_last_recall, Some(12));
        assert!(!dead_zones.entries.iter().any(|entry| entry.memory_id == 11));
    }

    #[test]
    fn attention_namespaces_derive_heatmap_inputs_from_live_store_state() {
        let namespace = NamespaceId::new("tests/f13-heatmap").unwrap();
        let mut store = BrainStore::default();

        store.append_audit_entry(
            AuditLogEntry::new(
                crate::observability::AuditEventCategory::Encode,
                crate::observability::AuditEventKind::EncodeAccepted,
                namespace.clone(),
                "encode_engine",
                "encoded working-set candidate",
            )
            .with_memory_id(MemoryId(51))
            .with_tick(3),
        );
        store.append_audit_entry(
            AuditLogEntry::new(
                crate::observability::AuditEventCategory::Recall,
                crate::observability::AuditEventKind::RecallServed,
                namespace.clone(),
                "recall_engine",
                "served attention candidate",
            )
            .with_memory_id(MemoryId(51))
            .with_session_id(SessionId(9))
            .with_tick(5),
        );
        store.append_audit_entry(
            AuditLogEntry::new(
                crate::observability::AuditEventCategory::Maintenance,
                crate::observability::AuditEventKind::MaintenanceConsolidationPartial,
                namespace.clone(),
                "maintenance_engine",
                "overflow promotion retained bounded prewarm signal",
            )
            .with_tick(8),
        );

        let mut blackboard = BlackboardState::new(
            namespace.clone(),
            Some(TaskId::new("heatmap-task")),
            Some(SessionId(9)),
            "surface heatmap attention",
        );
        blackboard.active_evidence = vec![
            BlackboardEvidenceHandle::new(MemoryId(51), "primary").pinned(),
            BlackboardEvidenceHandle::new(MemoryId(52), "supporting"),
        ];
        let state = GoalWorkingState::new(
            TaskId::new("heatmap-task"),
            namespace.clone(),
            Some(SessionId(9)),
            vec![GoalStackFrame::new("surface heatmap attention")],
            blackboard,
        );
        store.upsert_goal_working_state(state);

        let attention = store.attention_namespaces();
        assert_eq!(attention.len(), 1);
        assert_eq!(attention[0].namespace, namespace.as_str());
        assert_eq!(attention[0].recall_count, 1);
        assert_eq!(attention[0].encode_count, 1);
        assert_eq!(attention[0].promotion_count, 1);
        assert_eq!(attention[0].overflow_count, 1);
        assert_eq!(attention[0].working_memory_pressure, 2);
    }

    #[test]
    fn predict_recall_hot_path_reuses_canonical_attention_policy() {
        let namespace = NamespaceId::new("tests/predict-hot-path").unwrap();
        let mut store = BrainStore::default();
        let predicted = store
            .predict_recall_hot_path(&namespace, MemoryId(41), None)
            .expect("predicted hot path should exist after recall");
        assert_eq!(predicted.memory_id, 41);
        assert_eq!(predicted.attention_score, 5);
        assert_eq!(predicted.heat_bucket, "warm");
        assert_eq!(predicted.prewarm_action, "observe_only");
        assert_eq!(predicted.prewarm_target_family, "none");

        let mut blackboard = BlackboardState::new(
            namespace.clone(),
            Some(TaskId::new("predict-hot-task")),
            Some(SessionId(7)),
            "investigate predicted hot path",
        );
        blackboard.active_evidence =
            vec![BlackboardEvidenceHandle::new(MemoryId(41), "primary").pinned()];
        let mut state = GoalWorkingState::new(
            TaskId::new("predict-hot-task"),
            namespace.clone(),
            Some(SessionId(7)),
            vec![GoalStackFrame::new("investigate predicted hot path")],
            blackboard,
        );
        state.selected_evidence_handles = vec![MemoryId(41)];
        store.upsert_goal_working_state(state);

        let predicted = store
            .predict_recall_hot_path(&namespace, MemoryId(41), Some(SessionId(7)))
            .expect("predicted hot path should exist with working-state pressure");
        assert_eq!(predicted.heat_bucket, "warm");
        assert_eq!(predicted.prewarm_action, "observe_only");
        assert_eq!(predicted.prewarm_target_family, "none");
        assert_eq!(predicted.dominant_signal, "working_set_pins");
        assert!(predicted.attention_score > 5);
    }

    #[test]
    fn predict_recall_hot_path_only_queues_prefetch_for_repeated_warming_recalls() {
        let namespace = NamespaceId::new("tests/predict-hot-prefetch-threshold").unwrap();
        let mut store = BrainStore::default();

        for tick in 1..=3 {
            store.append_audit_entry(
                AuditLogEntry::new(
                    crate::observability::AuditEventCategory::Recall,
                    crate::observability::AuditEventKind::RecallServed,
                    namespace.clone(),
                    "recall_engine",
                    "warming recall",
                )
                .with_memory_id(MemoryId(77))
                .with_request_id(format!("req-recall-77-{tick}"))
                .with_tick(tick),
            );
        }

        let predicted = store
            .predict_recall_hot_path(&namespace, MemoryId(77), None)
            .expect("predicted hot path should exist after repeated recall");
        assert_eq!(predicted.heat_bucket, "warm");
        assert_eq!(predicted.dominant_signal, "recall_count");
        assert_eq!(predicted.prewarm_action, "queue_prefetch_hint");
        assert_eq!(predicted.prewarm_trigger, "session_recency");
        assert_eq!(predicted.prewarm_target_family, "prefetch_hints");

        let mut store = BrainStore::default();
        for tick in 1..=2 {
            store.append_audit_entry(
                AuditLogEntry::new(
                    crate::observability::AuditEventCategory::Recall,
                    crate::observability::AuditEventKind::RecallServed,
                    namespace.clone(),
                    "recall_engine",
                    "insufficient warming recall",
                )
                .with_memory_id(MemoryId(78))
                .with_request_id(format!("req-recall-78-{tick}"))
                .with_tick(tick),
            );
        }

        let predicted = store
            .predict_recall_hot_path(&namespace, MemoryId(78), None)
            .expect("predicted hot path should exist for low repeated recall");
        assert_eq!(predicted.heat_bucket, "warm");
        assert_eq!(predicted.dominant_signal, "recall_count");
        assert_eq!(predicted.prewarm_action, "observe_only");
        assert_eq!(predicted.prewarm_target_family, "none");
    }

    #[test]
    fn brain_store_audit_log_queries_preserve_per_memory_and_range_history() {
        let namespace = NamespaceId::new("tests/audit-query").unwrap();
        let mut store = BrainStore::default();
        let encode = store.append_audit_entry(
            AuditLogEntry::new(
                crate::observability::AuditEventCategory::Encode,
                crate::observability::AuditEventKind::EncodeAccepted,
                namespace.clone(),
                "encode_engine",
                "accepted new memory",
            )
            .with_memory_id(MemoryId(41))
            .with_session_id(SessionId(5))
            .with_request_id("req-encode-41")
            .with_tick(7),
        );
        store.capture_snapshot(
            namespace.clone(),
            "baseline",
            Some("pre-forgetting anchor".to_string()),
            1,
            SnapshotRetentionClass::Restorable,
        );
        let forgetting = store
            .record_forgetting_audit(
                namespace.clone(),
                ForgettingPolicy {
                    forget_score_threshold: 600,
                    batch_size: 1,
                    ..Default::default()
                },
                1,
            )
            .into_iter()
            .next()
            .expect("forgetting summary should yield one audit row");

        let memory_history = store.audit_memory(MemoryId(1));
        assert_eq!(memory_history, vec![forgetting.clone()]);

        let missing_history = store.audit_memory(MemoryId(999));
        assert!(missing_history.is_empty());

        let range = store.audit_range(
            namespace.clone(),
            Some(1),
            Some(crate::observability::AuditEventKind::MaintenanceForgettingEvaluated),
            Some(4),
        );
        assert_eq!(range.total_matches, 1);
        assert!(!range.truncated);
        assert_eq!(range.rows, vec![forgetting.clone()]);

        let forgetting_history = store.audit_log().entries_for_related_run(
            forgetting
                .related_run
                .as_deref()
                .expect("forgetting audit should carry related run"),
        );
        assert_eq!(forgetting_history, vec![forgetting.clone()]);

        let full_history = store.audit_entries();
        assert_eq!(full_history.len(), 3);
        assert_eq!(full_history[0], encode.clone());
        assert_eq!(
            full_history[1].kind,
            crate::observability::AuditEventKind::ArchiveRecorded
        );
        assert_eq!(
            full_history[1].related_snapshot.as_deref(),
            Some("baseline")
        );
        assert_eq!(full_history[2], forgetting.clone());
        assert_eq!(store.audit_log().entries(), full_history);
        assert_eq!(store.audit_log().last_sequence(), Some(forgetting.sequence));
    }

    #[test]
    fn contradiction_and_snapshot_mutations_append_audit_rows() {
        let namespace = NamespaceId::new("tests/audit-mutations").unwrap();
        let mut store = BrainStore::default();

        let accepted = store
            .detect_and_branch_encode(
                namespace.clone(),
                MemoryId(77),
                &crate::engine::contradiction::ContradictionCandidate {
                    memory_id: MemoryId(77),
                    fingerprint: 404,
                    compact_text: "new unique memory".into(),
                    namespace: namespace.clone(),
                },
            )
            .unwrap();
        assert!(accepted.is_accepted());
        let accepted_audit = store.audit_entries()[0].clone();

        store.contradiction_engine_mut().register_memory(
            namespace.clone(),
            MemoryId(10),
            42,
            "server deployed to production".into(),
        );
        let contradiction = store
            .detect_and_branch_encode(
                namespace.clone(),
                MemoryId(11),
                &crate::engine::contradiction::ContradictionCandidate {
                    memory_id: MemoryId(11),
                    fingerprint: 42,
                    compact_text: "server deployed to production".into(),
                    namespace: namespace.clone(),
                },
            )
            .unwrap();
        assert!(contradiction.is_contradiction());
        let contradiction_audit = store.audit_entries()[1].clone();

        store.capture_snapshot(
            namespace.clone(),
            "checkpoint",
            Some("freeze audit baseline".to_string()),
            2,
            SnapshotRetentionClass::Standard,
        );
        let capture_audit = store.audit_entries()[2].clone();
        store.delete_snapshot(&namespace, "checkpoint").unwrap();
        let delete_audit = store.audit_entries()[3].clone();

        assert_eq!(
            accepted_audit.kind,
            crate::observability::AuditEventKind::EncodeAccepted
        );
        assert_eq!(accepted_audit.memory_id, Some(MemoryId(77)));
        assert_eq!(accepted_audit.tick, Some(1));
        assert_eq!(accepted_audit.detail, "branch=accepted incoming_memory=77");
        assert_eq!(
            contradiction_audit.kind,
            crate::observability::AuditEventKind::EncodeRejected
        );
        assert_eq!(contradiction_audit.memory_id, Some(MemoryId(11)));
        assert_eq!(contradiction_audit.tick, Some(2));
        assert!(contradiction_audit.detail.contains("existing_memory=10"));
        assert!(contradiction_audit.detail.contains("incoming_memory=11"));
        assert!(contradiction_audit
            .detail
            .contains("contradiction_kind=duplicate"));
        assert!(contradiction_audit.detail.contains("conflict_score=1000"));

        let by_memory = store.audit_memory(MemoryId(11));
        assert_eq!(by_memory, vec![contradiction_audit.clone()]);

        let encode_range = store.audit_range(
            namespace.clone(),
            Some(0),
            Some(crate::observability::AuditEventKind::EncodeRejected),
            Some(8),
        );
        assert_eq!(encode_range.rows, vec![contradiction_audit.clone()]);

        let snapshot_range = store.audit_range(namespace, Some(1), None, Some(8));
        assert_eq!(
            snapshot_range.rows,
            vec![
                accepted_audit,
                contradiction_audit,
                capture_audit,
                delete_audit,
            ]
        );
        assert_eq!(store.current_tick(), 4);
    }

    #[test]
    fn prepare_tier2_layout_from_encode_keeps_non_landmarks_explicit() {
        let store = BrainStore::default();
        let layout = store.prepare_tier2_layout_from_encode(
            NamespaceId::new("tests/landmarks").unwrap(),
            MemoryId(78),
            SessionId(5),
            RawEncodeInput::new(RawIntakeKind::Event, "routine standup note"),
        );

        assert_eq!(
            layout.metadata.landmark,
            crate::types::LandmarkMetadata::non_landmark()
        );
        assert_eq!(layout.prefilter_view().landmark, &layout.metadata.landmark);
        assert_eq!(
            layout.metadata_index_key().landmark,
            &layout.metadata.landmark
        );
    }

    #[test]
    fn prepare_tier2_layout_with_trace_from_encode_keeps_prefilter_metadata_only() {
        let store = BrainStore::default();
        let prepared: PreparedTier2Layout = store.prepare_tier2_layout_with_trace_from_encode(
            NamespaceId::new("tests/landmarks").unwrap(),
            MemoryId(79),
            SessionId(6),
            RawEncodeInput::new(
                RawIntakeKind::Event,
                "launch retro captured a turning point",
            )
            .with_landmark_signals(crate::types::LandmarkSignals::new(0.92, 0.84, 0.29, 91)),
        );

        assert!(prepared.layout.metadata.landmark.is_landmark);
        assert_eq!(
            prepared.layout.prefilter_view().era_started_at_tick(),
            Some(91)
        );
        assert_eq!(
            prepared.layout.prefilter_view().landmark_detection_score(),
            815
        );
        assert!(prepared
            .layout
            .prefilter_view()
            .landmark_detection_reason()
            .is_some_and(|reason| reason.contains("crossed landmark thresholds")));
        assert_eq!(
            prepared.layout.metadata.namespace.as_str(),
            "tests/landmarks"
        );
        assert_eq!(
            prepared.layout.payload.namespace.as_str(),
            "tests/landmarks"
        );
        assert_eq!(prepared.prefilter_trace.metadata_candidate_count, 1);
        assert_eq!(prepared.prefilter_trace.payload_fetch_count, 0);
        assert!(prepared.prefilter_stays_metadata_only());
        assert!(prepared.layout.payload_size_matches_raw_body());
    }

    #[test]
    fn brain_store_exposes_hot_store_component_identity() {
        let store = BrainStore::default();

        assert_eq!(store.hot_store_component_name(), "store.hot");
    }

    #[test]
    fn brain_store_hot_store_zero_budget_lookups_preserve_tier1_bypass_invariants() {
        let store = BrainStore::default();
        let namespace = NamespaceId::new("tests/tier1-zero-budget").unwrap();
        let mut hot = store.hot_store().new_metadata_store(3);
        hot.seed(Tier1HotRecord::metadata_only(
            namespace.clone(),
            MemoryId(1),
            SessionId(10),
            CanonicalMemoryType::Event,
            FastPathRouteFamily::Event,
            "older",
            10,
            500,
            4_096,
        ));
        hot.seed(Tier1HotRecord::metadata_only(
            namespace.clone(),
            MemoryId(2),
            SessionId(10),
            CanonicalMemoryType::Event,
            FastPathRouteFamily::Event,
            "newer",
            20,
            500,
            4_096,
        ));

        assert_eq!(
            store.hot_store_component_name(),
            store.hot_store().component_name()
        );
        assert_eq!(hot.capacity(), 3);
        assert_eq!(hot.len(), 2);
        assert!(!hot.is_empty());

        let exact = hot.exact_lookup_with_budget(&namespace, MemoryId(1), 0);
        let recent = hot.recent_for_session_with_budget(&namespace, SessionId(10), 2, 0);

        assert_eq!(exact.trace.outcome, Tier1LookupOutcome::Bypass);
        assert_eq!(recent.trace.outcome, Tier1LookupOutcome::Bypass);
        assert_eq!(exact.trace.payload_fetch_count, 0);
        assert_eq!(recent.trace.payload_fetch_count, 0);
        assert_eq!(exact.trace.recent_candidates_inspected, 0);
        assert_eq!(recent.trace.recent_candidates_inspected, 0);
        assert_eq!(hot.capacity(), 3);
        assert_eq!(hot.len(), 2);
        assert!(!hot.is_empty());
    }

    #[test]
    fn brain_store_exposes_audit_taxonomy_and_default_log_builder() {
        let store = BrainStore::default();
        let log = store.new_default_audit_log();
        let taxonomy = store.audit_taxonomy();

        assert_eq!(
            store.audit_event_categories(),
            store.audit_log_store().categories()
        );
        assert_eq!(
            store.audit_event_kinds(),
            store.audit_log_store().event_kinds()
        );
        assert_eq!(taxonomy, store.audit_log_store().taxonomy());
        assert_eq!(
            taxonomy.first().map(|row| row.kind),
            Some(crate::observability::AuditEventKind::EncodeAccepted)
        );
        assert_eq!(
            taxonomy.last().map(|row| row.kind),
            Some(crate::observability::AuditEventKind::ArchiveRecorded)
        );
        assert!(taxonomy.iter().any(|row| {
            row.kind == crate::observability::AuditEventKind::MaintenanceReconsolidationApplied
        }));
        assert!(taxonomy.iter().any(|row| {
            row.kind == crate::observability::AuditEventKind::MaintenanceReconsolidationDiscarded
        }));
        assert!(taxonomy.iter().any(|row| {
            row.kind == crate::observability::AuditEventKind::MaintenanceReconsolidationDeferred
        }));
        assert!(taxonomy.iter().any(|row| {
            row.kind == crate::observability::AuditEventKind::MaintenanceReconsolidationBlocked
        }));
        assert_eq!(
            log.capacity(),
            crate::store::audit::AppendOnlyAuditLog::DEFAULT_CAPACITY
        );
        assert_eq!(log.next_sequence(), 1);
    }

    #[test]
    fn brain_store_exposes_audit_store_component_identity() {
        let store = BrainStore::default();

        assert_eq!(store.audit_log_store_component_name(), "store.audit");
    }

    #[test]
    fn audit_authoritative_schema_objects_expose_memory_audit_log_table() {
        let store = BrainStore::default();

        assert_eq!(
            store.audit_authoritative_schema_objects(),
            vec![DurableSchemaObject::MemoryAuditLogTable]
        );
        assert!(store
            .migrate()
            .durable_schema_manifest()
            .authoritative_tables
            .contains(&DurableSchemaObject::MemoryAuditLogTable));
    }

    #[test]
    fn brain_store_exposes_confidence_engine_component_identity() {
        let store = BrainStore::default();

        assert_eq!(
            store.confidence_engine().component_name(),
            "engine.confidence"
        );
    }

    #[test]
    fn brain_store_confidence_engine_matches_direct_engine_default() {
        let store = BrainStore::default();

        assert_eq!(
            store.confidence_engine(),
            &crate::engine::confidence::ConfidenceEngine
        );
    }

    #[test]
    fn tier2_authoritative_schema_objects_expose_split_durable_memory_tables() {
        let store = BrainStore::default();
        let schema_objects = store.tier2_authoritative_schema_objects();

        assert_eq!(
            schema_objects,
            vec![
                DurableSchemaObject::MemoryItemsTable,
                DurableSchemaObject::MemoryPayloadsTable,
                DurableSchemaObject::MemoryLineageEdgesTable,
                DurableSchemaObject::CausalLinksTable,
                DurableSchemaObject::MemoryEntityRefsTable,
                DurableSchemaObject::MemoryRelationRefsTable,
                DurableSchemaObject::MemoryTagsTable,
                DurableSchemaObject::ConflictRecordsTable,
                DurableSchemaObject::DurableMemoryRecords,
                DurableSchemaObject::SnapshotMetadataTable,
                DurableSchemaObject::CompressionLogTable,
                DurableSchemaObject::LandmarksTable,
            ]
        );
    }

    #[test]
    fn tier2_schema_matches_migration_manifest_for_durable_truth() {
        let store = BrainStore::default();

        assert!(store.tier2_schema_matches_migration_manifest());
    }

    #[test]
    fn capture_snapshot_records_bounded_metadata_and_reuses_existing_name() {
        let mut store = BrainStore::default();
        let namespace = NamespaceId::new("tests/snapshot").unwrap();

        let first = store.capture_snapshot(
            namespace.clone(),
            "before-refactor",
            Some("pre-change anchor".to_string()),
            3,
            SnapshotRetentionClass::Restorable,
        );
        let second = store.capture_snapshot(
            namespace.clone(),
            "before-refactor",
            Some("ignored on idempotent replay".to_string()),
            99,
            SnapshotRetentionClass::Standard,
        );

        assert_eq!(first.snapshot_id, SnapshotId(1));
        assert_eq!(first.snapshot_name, "before-refactor");
        assert_eq!(first.as_of_tick, 0);
        assert_eq!(first.created_at_tick, 1);
        assert_eq!(first.note.as_deref(), Some("pre-change anchor"));
        assert_eq!(first.memory_count, 3);
        assert_eq!(first.retention_class, SnapshotRetentionClass::Restorable);
        assert!(first.active);
        assert_eq!(second, first);
        assert_eq!(store.current_tick(), 1);
        assert_eq!(store.next_snapshot_id(), SnapshotId(2));
        assert_eq!(store.active_snapshot_count(&namespace), 1);
        assert_eq!(store.active_restorable_snapshot_count(&namespace), 1);
        assert_eq!(
            store.get_snapshot(&namespace, "before-refactor"),
            Some(first)
        );
    }

    #[test]
    fn delete_snapshot_blocks_last_restorable_anchor_but_allows_standard_handle() {
        let mut store = BrainStore::default();
        let namespace = NamespaceId::new("tests/snapshot-delete").unwrap();

        store.capture_snapshot(
            namespace.clone(),
            "baseline",
            None,
            2,
            SnapshotRetentionClass::Restorable,
        );
        store.capture_snapshot(
            namespace.clone(),
            "scratch",
            None,
            1,
            SnapshotRetentionClass::Standard,
        );

        let deleted_standard = store.delete_snapshot(&namespace, "scratch").unwrap();
        assert!(!deleted_standard.active);
        assert_eq!(store.active_snapshot_count(&namespace), 1);

        let error = store.delete_snapshot(&namespace, "baseline").unwrap_err();
        assert_eq!(error.snapshot_name, "baseline");
        assert_eq!(
            error.reason,
            SnapshotDeleteErrorReason::LastRestorableAnchor
        );
        assert_eq!(store.active_snapshot_count(&namespace), 1);
        assert_eq!(store.active_restorable_snapshot_count(&namespace), 1);
    }

    #[test]
    fn delete_snapshot_marks_metadata_inactive_when_restorable_peer_exists() {
        let mut store = BrainStore::default();
        let namespace = NamespaceId::new("tests/snapshot-delete-peer").unwrap();

        store.capture_snapshot(
            namespace.clone(),
            "baseline-a",
            None,
            1,
            SnapshotRetentionClass::Restorable,
        );
        store.capture_snapshot(
            namespace.clone(),
            "baseline-b",
            Some("second restore point".to_string()),
            4,
            SnapshotRetentionClass::Restorable,
        );

        let deleted = store.delete_snapshot(&namespace, "baseline-a").unwrap();
        assert!(!deleted.active);
        assert_eq!(deleted.retention_class, SnapshotRetentionClass::Restorable);
        assert_eq!(store.active_snapshot_count(&namespace), 1);
        assert_eq!(store.active_restorable_snapshot_count(&namespace), 1);
        assert!(store.get_snapshot(&namespace, "baseline-a").is_none());
        let listed = store.list_snapshots(&namespace);
        assert_eq!(listed.len(), 1);
        assert_eq!(listed[0].snapshot_name, "baseline-b");
        assert_eq!(listed[0].created_at_tick, 2);
    }

    #[test]
    fn delete_snapshot_reports_missing_handles() {
        let mut store = BrainStore::default();
        let namespace = NamespaceId::new("tests/snapshot-missing").unwrap();

        let error = store.delete_snapshot(&namespace, "missing").unwrap_err();
        assert_eq!(error.reason, SnapshotDeleteErrorReason::NotFound);
        assert_eq!(error.snapshot_name, "missing");
    }

    #[test]
    fn list_snapshots_returns_active_rows_in_creation_order() {
        let mut store = BrainStore::default();
        let namespace = NamespaceId::new("tests/snapshot-list").unwrap();

        let first = store.capture_snapshot(
            namespace.clone(),
            "first",
            None,
            1,
            SnapshotRetentionClass::Standard,
        );
        let second = store.capture_snapshot(
            namespace.clone(),
            "second",
            None,
            2,
            SnapshotRetentionClass::Restorable,
        );
        let listed = store.list_snapshots(&namespace);

        assert_eq!(listed, vec![first, second]);
    }

    #[test]
    fn next_snapshot_id_advances_with_each_new_capture() {
        let mut store = BrainStore::default();
        let namespace = NamespaceId::new("tests/snapshot-counter").unwrap();

        assert_eq!(store.next_snapshot_id(), SnapshotId(1));
        store.capture_snapshot(
            namespace.clone(),
            "one",
            None,
            1,
            SnapshotRetentionClass::Standard,
        );
        assert_eq!(store.next_snapshot_id(), SnapshotId(2));
        store.capture_snapshot(namespace, "two", None, 1, SnapshotRetentionClass::Standard);
        assert_eq!(store.next_snapshot_id(), SnapshotId(3));
    }

    #[test]
    fn semantic_diff_between_ticks_surfaces_explicit_category_counts() {
        let namespace = NamespaceId::new("tests/semantic-diff-ticks").unwrap();
        let mut store = BrainStore::default();

        store.append_audit_entry(
            AuditLogEntry::new(
                AuditEventCategory::Encode,
                AuditEventKind::EncodeAccepted,
                namespace.clone(),
                "encode_branch",
                "branch=accepted incoming_memory=7",
            )
            .with_memory_id(MemoryId(7))
            .with_tick(1),
        );
        store.append_audit_entry(
            AuditLogEntry::new(
                AuditEventCategory::Maintenance,
                AuditEventKind::MaintenanceForgettingEvaluated,
                namespace.clone(),
                "forgetting_engine",
                "weakened stale memory=7 after forgetting review",
            )
            .with_memory_id(MemoryId(7))
            .with_tick(2),
        );
        store.append_audit_entry(
            AuditLogEntry::new(
                AuditEventCategory::Encode,
                AuditEventKind::EncodeRejected,
                namespace.clone(),
                "encode_branch",
                "branch=contradiction_recorded existing_memory=7 incoming_memory=8 contradiction_kind=supersession conflict_score=880",
            )
            .with_memory_id(MemoryId(8))
            .with_tick(3),
        );

        let diff = store.semantic_diff_between_ticks(namespace, 0, 3);

        assert_eq!(diff.before_anchor.kind(), "tick");
        assert_eq!(diff.after_anchor.kind(), "tick");
        assert_eq!(
            diff.category_counts,
            vec![
                (SemanticDiffCategory::New, 1),
                (SemanticDiffCategory::Weakened, 1),
                (SemanticDiffCategory::Conflicting, 1),
            ]
        );
        assert_eq!(diff.entries.len(), 3);
        assert_eq!(diff.entries[0].category, SemanticDiffCategory::New);
        assert_eq!(diff.entries[1].category, SemanticDiffCategory::Weakened);
        assert_eq!(diff.entries[2].category, SemanticDiffCategory::Conflicting);
        assert!(diff.entries[2].unresolved);
        assert_eq!(diff.entries[2].memory_id, Some(MemoryId(8)));
        assert_eq!(diff.caution, semantic_diff_caution());
    }

    #[test]
    fn semantic_diff_between_snapshots_uses_snapshot_anchors() {
        let namespace = NamespaceId::new("tests/semantic-diff-snapshots").unwrap();
        let mut store = BrainStore::default();

        let baseline = store.capture_snapshot(
            namespace.clone(),
            "baseline",
            Some("before changes".to_string()),
            0,
            SnapshotRetentionClass::Standard,
        );
        store.append_audit_entry(
            AuditLogEntry::new(
                AuditEventCategory::Archive,
                AuditEventKind::ArchiveRecorded,
                namespace.clone(),
                "snapshot_delete",
                "deleted snapshot=old-checkpoint active=false retention=standard",
            )
            .with_tick(2)
            .with_related_snapshot("old-checkpoint"),
        );
        let after = store.capture_snapshot(
            namespace.clone(),
            "after",
            Some("after changes".to_string()),
            1,
            SnapshotRetentionClass::Restorable,
        );

        let diff = store
            .semantic_diff_between_snapshots(namespace, "baseline", "after")
            .expect("both snapshots should resolve");

        assert_eq!(diff.before_anchor, baseline.anchor());
        assert_eq!(diff.after_anchor, after.anchor());
        assert_eq!(diff.entries.len(), 2);
        assert!(diff
            .entries
            .iter()
            .all(|entry| entry.before_anchor == baseline.anchor()));
        assert!(diff
            .entries
            .iter()
            .all(|entry| entry.after_anchor == after.anchor()));
        assert!(diff
            .entries
            .iter()
            .all(|entry| entry.audit_kind == Some(AuditEventKind::ArchiveRecorded)));
        assert!(diff
            .entries
            .iter()
            .any(|entry| entry.category == SemanticDiffCategory::Archived
                && entry.summary.contains("deleted snapshot=old-checkpoint")));
        assert!(diff
            .entries
            .iter()
            .any(|entry| entry.category == SemanticDiffCategory::DerivedState
                && entry.summary.contains("captured snapshot=baseline")));
        assert!(diff.entries.iter().all(|entry| !entry.unresolved));
    }

    #[test]
    fn semantic_diff_between_snapshots_returns_none_when_snapshot_missing() {
        let namespace = NamespaceId::new("tests/semantic-diff-missing").unwrap();
        let mut store = BrainStore::default();
        store.capture_snapshot(
            namespace.clone(),
            "baseline",
            None,
            0,
            SnapshotRetentionClass::Standard,
        );

        assert!(store
            .semantic_diff_between_snapshots(namespace, "baseline", "missing")
            .is_none());
    }

    #[test]
    fn evaluate_tier_routing_exposes_inspectable_trace_through_brain_store() {
        let store = BrainStore::default();
        let trace = store.evaluate_tier_routing(&TierRoutingInput {
            namespace: NamespaceId::new("tests/tier2").unwrap(),
            memory_id: MemoryId(91),
            session_id: SessionId(12),
            memory_type: CanonicalMemoryType::Event,
            current_tier: TierOwnership::Cold,
            lifecycle_state: LifecycleState::Active,
            salience: 850,
            ticks_since_recall: 0,
            payload_size_bytes: 4_096,
            pinned: false,
        });

        assert_eq!(trace.memory_id, MemoryId(91));
        assert_eq!(trace.lifecycle_state, LifecycleState::Active);
        assert_eq!(trace.salience, 850);
        assert_eq!(trace.ticks_since_recall, 0);
        assert_eq!(trace.payload_size_bytes, 4_096);
        assert!(!trace.pinned);
        assert!(matches!(
            trace.decision,
            crate::store::TierRoutingDecision::PromoteToHot {
                reason: TierRoutingReason::RecallActivity
            }
        ));
        assert!(trace.summary().contains("PROMOTE to hot"));
        assert!(trace.summary().contains("salience=850"));
    }

    // ── Contradiction write-path branching through BrainStore facade ─────────

    #[test]
    fn detect_and_branch_encode_records_contradiction_through_facade() {
        let mut store = BrainStore::default();
        let namespace = NamespaceId::new("tests/branch").unwrap();

        // Register an existing memory in the contradiction engine
        store.contradiction_engine_mut().register_memory(
            namespace.clone(),
            MemoryId(1),
            42,
            "server deployed to production".into(),
        );

        let candidate = crate::engine::contradiction::ContradictionCandidate {
            memory_id: MemoryId(2),
            fingerprint: 42,
            compact_text: "server deployed to production".into(),
            namespace: namespace.clone(),
        };

        let outcome = store
            .detect_and_branch_encode(namespace.clone(), MemoryId(2), &candidate)
            .unwrap();

        assert!(outcome.is_contradiction());
        let co = outcome.contradiction_outcome().unwrap();
        assert_eq!(co.existing_memory, MemoryId(1));
        assert_eq!(co.incoming_memory, MemoryId(2));
        assert_eq!(
            store.contradiction_engine().count_in_namespace(&namespace),
            1
        );
    }

    #[test]
    fn record_encode_contradiction_populates_belief_history_surface() {
        let mut store = BrainStore::default();
        let namespace = NamespaceId::new("tests/belief-history").unwrap();

        store.contradiction_engine_mut().register_memory(
            namespace.clone(),
            MemoryId(10),
            101,
            "deployment target is prod-a".into(),
        );
        store.contradiction_engine_mut().register_memory(
            namespace.clone(),
            MemoryId(20),
            202,
            "deployment target is prod-b".into(),
        );

        let outcome = store
            .record_encode_contradiction(
                namespace.clone(),
                MemoryId(10),
                MemoryId(20),
                ContradictionKind::Supersession,
                880,
            )
            .unwrap();

        let chain = store
            .belief_history_engine()
            .chain_for_contradiction(outcome.contradiction_id)
            .unwrap();
        assert_eq!(chain.primary_memory_id, MemoryId(10));
        assert_eq!(chain.current_version().memory_id, MemoryId(20));
        assert_eq!(
            chain.current_version().content_snapshot,
            "deployment target is prod-b"
        );

        let timeline = store.belief_timeline_for_memory(&MemoryId(20)).unwrap();
        assert_eq!(timeline.preferred_memory_id, MemoryId(20));
        assert_eq!(timeline.resolution_state, "superseded");
        assert_eq!(timeline.conflicts, 1);
        assert_eq!(timeline.versions.len(), 2);
        assert_eq!(timeline.versions[1].conflict_state, "superseded");

        let resolution = store.belief_resolution_for_memory(&MemoryId(20)).unwrap();
        assert_eq!(resolution.current_memory_id, MemoryId(20));
        assert_eq!(resolution.conflict_state, "superseded");

        let history = store
            .belief_history_explain_for_memory(&MemoryId(20))
            .unwrap();
        assert_eq!(history.len(), 2);
        assert_eq!(history[1].contradiction_id, Some(outcome.contradiction_id));

        let by_query = store.belief_history_for_query("prod-b").unwrap();
        assert_eq!(by_query.chain_id, chain.chain_id);
        assert!(store.open_belief_conflicts().is_empty());
    }

    #[test]
    fn goal_working_state_flows_through_brain_store_facade() {
        let mut store = BrainStore::default();
        let namespace = NamespaceId::new("tests.goal.facade").unwrap();
        let task_id = TaskId::new("deploy-incident");
        let mut blackboard = BlackboardState::new(
            namespace.clone(),
            Some(task_id.clone()),
            None,
            "restore service",
        );
        blackboard.subgoals = vec!["check alarms".to_string()];
        blackboard.active_evidence = vec![BlackboardEvidenceHandle::new(MemoryId(41), "primary")];
        blackboard.active_beliefs = vec!["service can recover".to_string()];
        blackboard.unknowns = vec!["impact window".to_string()];
        blackboard.next_action = Some("page on-call".to_string());
        blackboard.blocked_reason = Some("waiting on approval".to_string());

        let mut state = GoalWorkingState::new(
            task_id.clone(),
            namespace.clone(),
            None,
            vec![GoalStackFrame::new("restore service")],
            blackboard,
        );
        state.selected_evidence_handles = vec![MemoryId(41)];
        state.pending_dependencies = vec!["approval-ticket".to_string()];
        store.upsert_goal_working_state(state);

        let pinned = store
            .blackboard_pin(&task_id, MemoryId(43))
            .expect("pin result");
        let crate::api::FieldPresence::Present(blackboard) = pinned.blackboard_state else {
            panic!("expected blackboard state");
        };
        assert_eq!(blackboard.active_evidence.len(), 2);
        assert!(blackboard
            .active_evidence
            .iter()
            .any(|handle| handle.memory_id == MemoryId(43) && handle.pinned));

        let paused = store
            .goal_pause(&task_id, Some("waiting for approval".to_string()))
            .expect("pause result");
        assert_eq!(paused.status.as_str(), "dormant");

        let resumed = store.goal_resume(&task_id).expect("resume result");
        assert_eq!(resumed.status.as_str(), "active");
        assert!(resumed.restored_evidence_handles.contains(&41));

        let abandoned = store
            .goal_abandon(&task_id, Some("rollback superseded".to_string()))
            .expect("abandon result");
        assert_eq!(abandoned.status.as_str(), "abandoned");

        let entries = store.audit_entries();
        assert!(entries.iter().any(|entry| {
            entry.kind == AuditEventKind::IncidentRecorded
                && entry.detail.contains("paused task=deploy-incident")
        }));
        assert!(entries.iter().any(|entry| {
            entry.kind == AuditEventKind::IncidentRecorded
                && entry.detail.contains("resumed task=deploy-incident")
        }));
        assert!(entries.iter().any(|entry| {
            entry.kind == AuditEventKind::IncidentRecorded
                && entry.detail.contains("abandoned task=deploy-incident")
        }));
    }

    #[test]
    fn merge_fork_reports_explicit_conflicts_and_audit_sequences() {
        let mut store = BrainStore::default();
        let target_namespace = NamespaceId::new("team.alpha").unwrap();
        let fork_namespace = NamespaceId::new("agent-specialist").unwrap();

        store.fork(ForkConfig {
            name: "agent-specialist".to_string(),
            parent_namespace: target_namespace.clone(),
            inherit_visibility: ForkInheritance::SharedToo,
            note: Some("testing merge".to_string()),
        });

        let parent_pattern = "deploy after incident".to_string();
        let parent_handle = ProceduralStore::pattern_handle(&target_namespace, &parent_pattern);
        let parent_hash = ProceduralStore::pattern_hash(&target_namespace, &parent_pattern);
        store.procedural_entries.insert(
            parent_handle.clone(),
            ProceduralMemoryRecord {
                namespace: target_namespace.clone(),
                pattern_handle: parent_handle,
                pattern_hash: parent_hash,
                pattern: parent_pattern.clone(),
                action: "page primary on-call".to_string(),
                confidence: 700,
                source_fixture_name: "fixture-parent".to_string(),
                source_engram_id: Some(1),
                lineage_ancestors: vec![MemoryId(41)],
                supporting_memory_count: 1,
                source_citation_count: 1,
                query_cues: vec!["deploy".to_string()],
                accepted_by: "tester".to_string(),
                acceptance_note: "parent".to_string(),
                visibility: SharingVisibility::Shared,
                state: ProceduralEntryState::Active,
                version: 1,
                promotion_audit_sequence: 1,
                last_transition_sequence: 1,
                last_transition_kind: "approved_sharing",
                rollback_note: None,
            },
        );
        let fork_handle = ProceduralStore::pattern_handle(&fork_namespace, &parent_pattern);
        let fork_hash = ProceduralStore::pattern_hash(&fork_namespace, &parent_pattern);
        store.procedural_entries.insert(
            fork_handle.clone(),
            ProceduralMemoryRecord {
                namespace: fork_namespace.clone(),
                pattern_handle: fork_handle,
                pattern_hash: fork_hash,
                pattern: parent_pattern,
                action: "rollback immediately".to_string(),
                confidence: 900,
                source_fixture_name: "fixture-fork".to_string(),
                source_engram_id: Some(2),
                lineage_ancestors: vec![MemoryId(99)],
                supporting_memory_count: 1,
                source_citation_count: 1,
                query_cues: vec!["deploy".to_string()],
                accepted_by: "tester".to_string(),
                acceptance_note: "fork".to_string(),
                visibility: SharingVisibility::Shared,
                state: ProceduralEntryState::Active,
                version: 2,
                promotion_audit_sequence: 2,
                last_transition_sequence: 2,
                last_transition_kind: "approved_sharing",
                rollback_note: None,
            },
        );

        let mut blackboard = BlackboardState::new(
            fork_namespace.clone(),
            Some(TaskId::new("fork-task")),
            Some(SessionId(9)),
            "compare branch outputs",
        );
        blackboard.active_evidence = vec![BlackboardEvidenceHandle::new(MemoryId(99), "selected")];
        let state = GoalWorkingState::new(
            TaskId::new("fork-task"),
            fork_namespace,
            Some(SessionId(9)),
            vec![GoalStackFrame::new("compare branch outputs")],
            blackboard,
        );
        store.upsert_goal_working_state(state);

        let preview = store
            .merge_fork(MergeConfig {
                fork_name: "agent-specialist".to_string(),
                target_namespace: target_namespace.clone(),
                conflict_strategy: MergeConflictStrategy::Manual,
                dry_run: true,
            })
            .expect("merge preview should exist");
        assert_eq!(preview.conflicts_found, 2);
        assert_eq!(preview.conflicts_auto_resolved, 0);
        assert_eq!(preview.conflicts_pending, 2);
        assert!(preview.merged_items.is_empty());
        assert!(preview.audit_sequences.is_empty());
        assert_eq!(preview.conflict_items.len(), 2);
        assert!(preview
            .conflict_items
            .iter()
            .any(|item| item.item_kind == "procedural_entry"
                && item.resolution_state == "unresolved"));
        assert!(preview
            .conflict_items
            .iter()
            .any(|item| item.item_kind == "working_state" && item.preferred_side == "manual"));

        let applied = store
            .merge_fork(MergeConfig {
                fork_name: "agent-specialist".to_string(),
                target_namespace,
                conflict_strategy: MergeConflictStrategy::ForkWins,
                dry_run: false,
            })
            .expect("merge apply should exist");
        assert_eq!(applied.conflicts_found, 1);
        assert_eq!(applied.conflicts_auto_resolved, 1);
        assert_eq!(applied.conflicts_pending, 0);
        assert_eq!(applied.memories_merged, 2);
        assert_eq!(applied.engrams_merged, 2);
        assert_eq!(applied.audit_sequences.len(), 2);
        assert_eq!(applied.fork_status, "merged");
        assert!(applied
            .conflict_items
            .iter()
            .all(|item| item.resolution_state == "auto_resolved"
                || item.item_kind == "working_state"));
        let audit_entries = store.audit_entries();
        assert!(audit_entries.iter().any(|entry| {
            entry.actor_source == FORK_MERGE_ACTOR
                && entry.detail.contains("fork_merge_conflict")
                && entry.related_run.as_deref() == Some("merge-run-agent-specialist")
        }));
        assert!(audit_entries.iter().any(|entry| {
            entry.actor_source == FORK_MERGE_ACTOR
                && entry.detail.contains("merged fork=agent-specialist")
        }));
    }

    #[test]
    fn detect_and_branch_encode_accepts_on_no_conflict_through_facade() {
        let mut store = BrainStore::default();
        let namespace = NamespaceId::new("tests/branch").unwrap();

        let candidate = crate::engine::contradiction::ContradictionCandidate {
            memory_id: MemoryId(1),
            fingerprint: 42,
            compact_text: "unique content".into(),
            namespace: namespace.clone(),
        };

        let outcome = store
            .detect_and_branch_encode(namespace.clone(), MemoryId(1), &candidate)
            .unwrap();

        assert!(outcome.is_accepted());
        assert_eq!(
            store.contradiction_engine().count_in_namespace(&namespace),
            0
        );
        assert_eq!(store.audit_entries().len(), 1);
        assert_eq!(
            store.audit_entries()[0].kind,
            crate::observability::AuditEventKind::EncodeAccepted
        );
        assert_eq!(store.audit_entries()[0].memory_id, Some(MemoryId(1)));
        assert_eq!(store.audit_entries()[0].tick, Some(1));
    }

    #[test]
    fn detect_all_and_branch_encode_records_multiple_through_facade() {
        let mut store = BrainStore::default();
        let namespace = NamespaceId::new("tests/branch").unwrap();

        store.contradiction_engine_mut().register_memory(
            namespace.clone(),
            MemoryId(1),
            100,
            "the server is running on port 8080".into(),
        );
        store.contradiction_engine_mut().register_memory(
            namespace.clone(),
            MemoryId(3),
            300,
            "the server is running on port 8080 in production environment".into(),
        );

        let candidate = crate::engine::contradiction::ContradictionCandidate {
            memory_id: MemoryId(2),
            fingerprint: 200,
            compact_text: "the server is running on port 9090 in production".into(),
            namespace: namespace.clone(),
        };

        let outcomes = store
            .detect_all_and_branch_encode(namespace.clone(), MemoryId(2), &candidate)
            .unwrap();

        assert!(!outcomes.is_empty());
        assert!(store.contradiction_engine().count_in_namespace(&namespace) >= 1);
        let audit = store.audit_entries();
        assert_eq!(audit.len(), outcomes.len());
        assert!(audit.iter().all(|entry| {
            entry.kind == crate::observability::AuditEventKind::EncodeRejected
                && entry.memory_id == Some(MemoryId(2))
                && entry.tick == Some(1)
                && entry.detail.contains("branch=contradiction_recorded")
                && entry.detail.contains("incoming_memory=2")
        }));
    }
}
