/// Bounded cache families, admission, prefetch, invalidation, and observability.
pub mod audit;
pub mod cache;
pub mod cold;
pub mod hot;
pub mod tier2;
/// Tier routing, promotion, demotion, and lifecycle-aware placement decisions.
pub mod tier_router;

pub use audit::AuditLogStore;
pub use cache::CacheManager;
pub use cold::ColdStore;
pub use hot::HotStore;
pub use tier2::Tier2Store;

use crate::migrate::DurableSchemaObject;
pub use tier_router::{
    LifecycleState, TierOwnership, TierRouter, TierRoutingConfig, TierRoutingDecision,
    TierRoutingInput, TierRoutingReason, TierRoutingTrace,
};

/// Shared hot-store boundary for request-path planners.
pub trait HotStoreApi {
    /// Returns the stable component identifier for this Tier1 surface.
    fn component_name(&self) -> &'static str;
}

/// Shared Tier2 store boundary for indexed retrieval planners.
pub trait Tier2StoreApi {
    /// Returns the stable component identifier for this Tier2 surface.
    fn component_name(&self) -> &'static str;

    /// Returns the authoritative durable schema objects this store owns.
    fn authoritative_schema_objects(&self) -> Vec<DurableSchemaObject> {
        vec![DurableSchemaObject::DurableMemoryRecords]
    }
}

/// Shared cold-store boundary for archive and repair flows.
pub trait ColdStoreApi {
    /// Returns the stable component identifier for this cold-store surface.
    fn component_name(&self) -> &'static str;
}

/// Shared append-only audit-log boundary for forensic and governance history.
pub trait AuditLogStoreApi {
    /// Returns the stable component identifier for this audit-log surface.
    fn component_name(&self) -> &'static str;
}

/// Shared engram durability boundary for stores that own authoritative engram rows.
pub trait EngramStoreSchemaApi {
    /// Returns the authoritative durable engram schema objects owned by this store surface.
    fn engram_schema_objects(&self) -> Vec<DurableSchemaObject> {
        vec![
            DurableSchemaObject::EngramsTable,
            DurableSchemaObject::EngramMembershipTable,
        ]
    }
}

/// Stable store set composed by the core facade.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct CoreStores {
    hot: HotStore,
    tier2: Tier2Store,
    cold: ColdStore,
    audit: AuditLogStore,
}

impl CoreStores {
    /// Builds the stable store set from the canonical core-owned store surfaces.
    pub const fn new(
        hot: HotStore,
        tier2: Tier2Store,
        cold: ColdStore,
        audit: AuditLogStore,
    ) -> Self {
        Self {
            hot,
            tier2,
            cold,
            audit,
        }
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

    /// Returns the append-only audit-log store boundary.
    pub fn audit(&self) -> &AuditLogStore {
        &self.audit
    }

    /// Returns the authoritative durable schema objects exposed by the Tier2 store.
    pub fn tier2_authoritative_schema_objects(&self) -> Vec<DurableSchemaObject> {
        self.tier2.authoritative_schema_objects()
    }
}
