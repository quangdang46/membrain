use crate::api::{
    ApiModule, FieldPresence, NamespaceId, ProceduralEntryAuditView, ProceduralEntryRecallView,
    ProceduralEntryReviewView, ProceduralEntryStorageView, ProceduralEntrySummary,
    ProceduralStoreOutput, ReflectionArtifactView, SkillArtifactRecallView,
    SkillArtifactReviewView, SkillArtifactStorageView, SkillArtifactSummary, SkillArtifactsOutput,
};
use crate::config::RuntimeConfig;
use crate::embed::EmbedModule;
use crate::engine::belief_history::BeliefHistoryEngine;
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
use crate::graph::{CausalInvalidationReport, CausalLink, CausalTrace, GraphModule};
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
    CoreApiVersion, MemoryId, RawEncodeInput, SemanticDiff, SemanticDiffCategory,
    SemanticDiffEntry, SessionId, SnapshotAnchor, SnapshotId, SnapshotMetadata,
    SnapshotRetentionClass,
};
use std::collections::HashMap;

const PROCEDURAL_PROMOTION_ACTOR: &str = "procedural_store_promotion";
const PROCEDURAL_ROLLBACK_ACTOR: &str = "procedural_store_rollback";

/// Inspectable result returned when the core facade prepares a Tier2 layout from encode output.
#[derive(Debug, Clone, PartialEq, Eq)]
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

fn semantic_diff_caution() -> &'static str {
    "semantic diff summarizes bounded historical evidence and does not prove consensus or truth"
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
    snapshots: HashMap<SnapshotId, SnapshotMetadata>,
    snapshot_name_index: HashMap<(NamespaceId, String), SnapshotId>,
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
            snapshots: HashMap::new(),
            snapshot_name_index: HashMap::new(),
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

    /// Returns the current logical tick inferred from retained snapshot and audit metadata.
    pub fn current_tick(&self) -> u64 {
        self.snapshots
            .values()
            .map(|snapshot| snapshot.created_at_tick.max(snapshot.as_of_tick))
            .chain(self.audit_log.max_tick())
            .max()
            .unwrap_or(0)
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
        semantic_diff_caution, BrainStore, PreparedTier2Layout, SnapshotDeleteErrorReason,
    };
    use crate::api::NamespaceId;
    use crate::engine::consolidation::{ConsolidationPolicy, DerivedArtifactKind};
    use crate::engine::contradiction::{ContradictionKind, ContradictionStore};
    use crate::engine::forgetting::ForgettingPolicy;
    use crate::engine::maintenance::MaintenanceJobState;
    use crate::engine::reconsolidation::{
        LabileMemory, LabileState, PendingUpdate, PreReopenState, ReconsolidationPolicy,
        RefreshReadiness, ReopenStableState, UpdateSource,
    };
    use crate::migrate::DurableSchemaObject;
    use crate::observability::{AuditEventCategory, AuditEventKind, Tier1LookupOutcome};
    use crate::store::audit::AuditLogEntry;
    use crate::store::{
        HotStoreApi, LifecycleState, ProceduralEntryState, TierOwnership, TierRoutingInput,
        TierRoutingReason,
    };
    use crate::types::{
        CanonicalMemoryType, FastPathRouteFamily, MemoryId, RawEncodeInput, RawIntakeKind,
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
            crate::observability::AuditEventKind::MaintenanceReconsolidationDiscarded
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
        assert_eq!(diff.entries.len(), 1);
        assert_eq!(diff.entries[0].category, SemanticDiffCategory::Archived);
        assert_eq!(diff.entries[0].before_anchor, baseline.anchor());
        assert_eq!(diff.entries[0].after_anchor, after.anchor());
        assert_eq!(
            diff.entries[0].audit_kind,
            Some(AuditEventKind::ArchiveRecorded)
        );
        assert!(!diff.entries[0].unresolved);
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
