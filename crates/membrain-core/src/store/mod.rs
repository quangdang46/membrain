pub mod cold;
pub mod hot;
pub mod tier2;

pub use cold::ColdStore;
pub use hot::HotStore;
pub use tier2::Tier2Store;

/// Shared hot-store boundary for request-path planners.
pub trait HotStoreApi {
    /// Returns the stable component identifier for this Tier1 surface.
    fn component_name(&self) -> &'static str;
}

/// Shared Tier2 store boundary for indexed retrieval planners.
pub trait Tier2StoreApi {
    /// Returns the stable component identifier for this Tier2 surface.
    fn component_name(&self) -> &'static str;
}

/// Shared cold-store boundary for archive and repair flows.
pub trait ColdStoreApi {
    /// Returns the stable component identifier for this cold-store surface.
    fn component_name(&self) -> &'static str;
}

/// Stable store set composed by the core facade.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct CoreStores {
    hot: HotStore,
    tier2: Tier2Store,
    cold: ColdStore,
}

impl CoreStores {
    /// Builds the stable store set from the canonical core-owned store surfaces.
    pub const fn new(hot: HotStore, tier2: Tier2Store, cold: ColdStore) -> Self {
        Self { hot, tier2, cold }
    }

    /// Returns the Tier1 store boundary.
    pub fn hot(&self) -> &HotStore {
        &self.hot
    }

    /// Returns the Tier2 store boundary.
    pub fn tier2(&self) -> &Tier2Store {
        &self.tier2
    }

    /// Returns the cold/archive store boundary.
    pub fn cold(&self) -> &ColdStore {
        &self.cold
    }
}
