use crate::api::ApiModule;
use crate::config::RuntimeConfig;
use crate::embed::EmbedModule;
use crate::engine::consolidation::ConsolidationEngine;
use crate::engine::contradiction::ContradictionEngine;
use crate::engine::encode::EncodeEngine;
use crate::engine::forgetting::ForgettingEngine;
use crate::engine::ranking::RankingEngine;
use crate::engine::recall::RecallEngine;
use crate::engine::repair::RepairEngine;
use crate::graph::GraphModule;
use crate::index::IndexModule;
use crate::migrate::MigrationModule;
use crate::observability::ObservabilityModule;
use crate::policy::PolicyModule;
use crate::store::cold::ColdStore;
use crate::store::hot::HotStore;
use crate::store::tier2::Tier2Store;
use crate::types::CoreApiVersion;

/// Stable top-level core facade for the initial workspace bootstrap.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BrainStore {
    config: RuntimeConfig,
    api: ApiModule,
    policy: PolicyModule,
    observability: ObservabilityModule,
    encode: EncodeEngine,
    recall: RecallEngine,
    ranking: RankingEngine,
    contradiction: ContradictionEngine,
    consolidation: ConsolidationEngine,
    forgetting: ForgettingEngine,
    repair: RepairEngine,
    hot_store: HotStore,
    tier2_store: Tier2Store,
    cold_store: ColdStore,
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
            ranking: RankingEngine,
            contradiction: ContradictionEngine::new(),
            consolidation: ConsolidationEngine,
            forgetting: ForgettingEngine,
            repair: RepairEngine,
            hot_store: HotStore,
            tier2_store: Tier2Store,
            cold_store: ColdStore,
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

    /// Returns the canonical cold storage surface owned by the core crate.
    pub fn cold_store(&self) -> &ColdStore {
        &self.cold_store
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
