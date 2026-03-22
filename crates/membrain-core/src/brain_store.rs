use crate::api::ApiModule;
use crate::api::NamespaceId;
use crate::config::RuntimeConfig;
use crate::embed::EmbedModule;
use crate::engine::consolidation::ConsolidationEngine;
use crate::engine::contradiction::{
    ContradictionCandidate, ContradictionEngine, ContradictionError, ContradictionKind,
};
use crate::engine::encode::{ContradictionWriteOutcome, EncodeEngine, WriteBranchOutcome};
use crate::engine::forgetting::ForgettingEngine;
use crate::engine::intent::IntentEngine;
use crate::engine::ranking::RankingEngine;
use crate::engine::recall::RecallEngine;
use crate::engine::repair::RepairEngine;
use crate::graph::GraphModule;
use crate::index::IndexModule;
use crate::migrate::{DurableSchemaObject, MigrationModule};
use crate::observability::{ObservabilityModule, Tier2PrefilterTrace};
use crate::policy::PolicyModule;
use crate::store::audit::AuditLogStore;
use crate::store::cold::ColdStore;
use crate::store::hot::HotStore;
use crate::store::tier2::{Tier2DurableItemLayout, Tier2Store};
use crate::store::tier_router::{TierRouter, TierRoutingInput, TierRoutingTrace};
use crate::store::{AuditLogStoreApi, HotStoreApi, Tier2StoreApi};
use crate::types::{CoreApiVersion, MemoryId, RawEncodeInput, SessionId};

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

/// Stable top-level core facade for the initial workspace bootstrap.
#[derive(Debug, Clone, PartialEq, Eq)]
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
    consolidation: ConsolidationEngine,
    forgetting: ForgettingEngine,
    repair: RepairEngine,
    hot_store: HotStore,
    tier2_store: Tier2Store,
    tier_router: TierRouter,
    cold_store: ColdStore,
    audit_log_store: AuditLogStore,
    graph: GraphModule,
    index: IndexModule,
    embed: EmbedModule,
    migrate: MigrationModule,
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
            consolidation: ConsolidationEngine,
            forgetting: ForgettingEngine,
            repair: RepairEngine,
            hot_store: HotStore,
            tier2_store: Tier2Store,
            tier_router: TierRouter::default(),
            cold_store: ColdStore,
            audit_log_store: AuditLogStore,
            graph: GraphModule,
            index: IndexModule,
            embed: EmbedModule,
            migrate: MigrationModule,
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

    /// Records an encode-side contradiction branch without silently overwriting either memory.
    pub fn record_encode_contradiction(
        &mut self,
        namespace: NamespaceId,
        existing_memory: MemoryId,
        incoming_memory: MemoryId,
        kind: ContradictionKind,
        conflict_score: u16,
    ) -> Result<ContradictionWriteOutcome, ContradictionError> {
        self.encode.record_contradiction_branch(
            &mut self.contradiction,
            namespace,
            existing_memory,
            incoming_memory,
            kind,
            conflict_score,
        )
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
        self.encode.detect_and_branch(
            &mut self.contradiction,
            namespace,
            incoming_memory,
            candidate,
        )
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
        self.encode.detect_all_and_branch(
            &mut self.contradiction,
            namespace,
            incoming_memory,
            candidate,
        )
    }

    /// Returns the shared consolidation surface owned by the core crate.
    pub fn consolidation_engine(&self) -> &ConsolidationEngine {
        &self.consolidation
    }

    /// Returns the shared forgetting surface owned by the core crate.
    pub fn forgetting_engine(&self) -> &ForgettingEngine {
        &self.forgetting
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

    /// Returns the canonical Tier2 storage surface owned by the core crate.
    pub fn tier2_store(&self) -> &Tier2Store {
        &self.tier2_store
    }

    /// Returns the authoritative durable Tier2 schema objects exposed by the core-owned Tier2 store.
    pub fn tier2_authoritative_schema_objects(&self) -> Vec<DurableSchemaObject> {
        self.tier2_store.authoritative_schema_objects()
    }

    /// Returns whether migration manifests still include the full Tier2 durable-truth ownership set.
    pub fn tier2_schema_matches_migration_manifest(&self) -> bool {
        let store_objects = self.tier2_authoritative_schema_objects();
        let manifest = self.migrate.durable_schema_manifest();

        store_objects
            .iter()
            .all(|object| manifest.authoritative_tables.contains(object))
    }

    /// Runs the encode fast path and materializes a durable Tier2 layout that preserves additive
    /// landmark and era metadata alongside metadata-first storage keys.
    pub fn prepare_tier2_layout_from_encode(
        &self,
        namespace: NamespaceId,
        memory_id: MemoryId,
        session_id: SessionId,
        input: RawEncodeInput,
    ) -> Tier2DurableItemLayout {
        let prepared = self.encode.prepare_fast_path(input);
        self.tier2_store.layout_item(
            namespace,
            memory_id,
            session_id,
            prepared.fingerprint,
            &prepared.normalized,
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

    /// Returns the bounded graph surface owned by the core crate.
    pub fn graph(&self) -> &GraphModule {
        &self.graph
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
    use super::{BrainStore, PreparedTier2Layout};
    use crate::api::NamespaceId;
    use crate::engine::contradiction::ContradictionStore;
    use crate::migrate::DurableSchemaObject;
    use crate::observability::Tier1LookupOutcome;
    use crate::store::{
        HotStoreApi, LifecycleState, TierOwnership, TierRoutingInput, TierRoutingReason,
    };
    use crate::types::{
        CanonicalMemoryType, FastPathRouteFamily, MemoryId, RawEncodeInput, RawIntakeKind,
        SessionId, Tier1HotRecord,
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

        assert!(layout.metadata.landmark.is_landmark);
        assert_eq!(
            layout.metadata.landmark.landmark_label.as_deref(),
            Some("project launch deadline was moved")
        );
        assert_eq!(
            layout.metadata.landmark.era_id.as_deref(),
            Some("era-projectlaunc-0088")
        );
        assert_eq!(layout.prefilter_view().landmark, &layout.metadata.landmark);
        assert_eq!(
            layout.metadata_index_key().landmark,
            &layout.metadata.landmark
        );
        assert_eq!(layout.prefilter_trace().payload_fetch_count, 0);
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
    fn brain_store_exposes_audit_store_component_identity() {
        let store = BrainStore::default();

        assert_eq!(store.audit_log_store_component_name(), "store.audit");
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
                DurableSchemaObject::MemoryEntityRefsTable,
                DurableSchemaObject::MemoryRelationRefsTable,
                DurableSchemaObject::MemoryTagsTable,
                DurableSchemaObject::ConflictRecordsTable,
                DurableSchemaObject::DurableMemoryRecords,
            ]
        );
    }

    #[test]
    fn tier2_schema_matches_migration_manifest_for_durable_truth() {
        let store = BrainStore::default();

        assert!(store.tier2_schema_matches_migration_manifest());
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
    }
}
