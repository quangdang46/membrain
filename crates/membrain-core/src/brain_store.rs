use crate::api::ApiModule;
use crate::api::NamespaceId;
use crate::config::RuntimeConfig;
use crate::embed::EmbedModule;
use crate::engine::consolidation::ConsolidationEngine;
use crate::engine::contradiction::{ContradictionEngine, ContradictionError, ContradictionKind};
use crate::engine::encode::{ContradictionWriteOutcome, EncodeEngine};
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
use crate::store::Tier2StoreApi;
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
    use crate::migrate::DurableSchemaObject;
    use crate::types::{MemoryId, RawEncodeInput, RawIntakeKind, SessionId};

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
    }

    #[test]
    fn tier2_authoritative_schema_objects_expose_durable_memory_records() {
        let store = BrainStore::default();
        let schema_objects = store.tier2_authoritative_schema_objects();

        assert_eq!(
            schema_objects,
            vec![DurableSchemaObject::DurableMemoryRecords]
        );
    }

    #[test]
    fn tier2_schema_matches_migration_manifest_for_durable_truth() {
        let store = BrainStore::default();

        assert!(store.tier2_schema_matches_migration_manifest());
    }
}
