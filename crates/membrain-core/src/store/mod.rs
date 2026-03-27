/// Bounded cache families, admission, prefetch, invalidation, and observability.
pub mod audit;
pub mod cache;
pub mod cold;
pub mod hot;
pub mod procedural;
pub mod tier2;
/// Tier routing, promotion, demotion, and lifecycle-aware placement decisions.
pub mod tier_router;

pub use audit::AuditLogStore;
pub use cache::CacheManager;
pub use cold::ColdStore;
pub use hot::HotStore;
pub use procedural::{
    ProceduralEntryState, ProceduralMemoryRecord, ProceduralStore, ProceduralStoreError,
    ProceduralStoreErrorReason,
};
pub use tier2::Tier2Store;

use crate::migrate::DurableSchemaObject;
pub use tier_router::{
    DurableLifecycleState, LifecycleState, TierOwnership, TierRouter, TierRoutingConfig,
    TierRoutingDecision, TierRoutingInput, TierRoutingReason, TierRoutingTrace,
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
    }
}

/// Shared cold-store boundary for archive and repair flows.
pub trait ColdStoreApi {
    /// Returns the stable component identifier for this cold-store surface.
    fn component_name(&self) -> &'static str;
}

/// Shared authoritative procedural-store boundary for accepted pattern→action mappings.
pub trait ProceduralStoreApi {
    /// Returns the stable component identifier for this procedural-store surface.
    fn component_name(&self) -> &'static str;

    /// Returns the authoritative durable schema objects this procedural store owns.
    fn authoritative_schema_objects(&self) -> Vec<DurableSchemaObject> {
        vec![
            DurableSchemaObject::ProceduralMemoriesTable,
            DurableSchemaObject::ProceduralLineageTable,
        ]
    }
}

/// Shared append-only audit-log boundary for forensic and governance history.
pub trait AuditLogStoreApi {
    /// Returns the stable component identifier for this audit-log surface.
    fn component_name(&self) -> &'static str;

    /// Returns the authoritative durable schema objects this audit surface owns.
    fn authoritative_schema_objects(&self) -> Vec<DurableSchemaObject> {
        vec![DurableSchemaObject::MemoryAuditLogTable]
    }
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
    procedural: ProceduralStore,
    audit: AuditLogStore,
}

impl CoreStores {
    /// Builds the stable store set from the canonical core-owned store surfaces.
    pub const fn new(
        hot: HotStore,
        tier2: Tier2Store,
        cold: ColdStore,
        procedural: ProceduralStore,
        audit: AuditLogStore,
    ) -> Self {
        Self {
            hot,
            tier2,
            cold,
            procedural,
            audit,
        }
    }

    /// Returns the stable Tier1 hot-store component identifier exposed by the shared store facade.
    pub fn hot_store_component_name(&self) -> &'static str {
        self.hot.component_name()
    }

    /// Returns the stable Tier1 hot-store component identifier exposed by the shared store facade.
    pub fn hot_component_name(&self) -> &'static str {
        self.hot_store_component_name()
    }

    /// Returns the stable append-only audit-log component identifier exposed by the shared store facade.
    pub fn audit_component_name(&self) -> &'static str {
        self.audit.component_name()
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

    /// Returns the authoritative procedural-store boundary.
    pub fn procedural(&self) -> &ProceduralStore {
        &self.procedural
    }

    /// Returns the append-only audit-log store boundary.
    pub fn audit(&self) -> &AuditLogStore {
        &self.audit
    }

    /// Returns the authoritative durable schema objects exposed by the Tier2 store.
    pub fn tier2_authoritative_schema_objects(&self) -> Vec<DurableSchemaObject> {
        self.tier2.authoritative_schema_objects()
    }

    /// Returns the authoritative durable schema objects exposed by the procedural store.
    pub fn procedural_authoritative_schema_objects(&self) -> Vec<DurableSchemaObject> {
        self.procedural.authoritative_schema_objects()
    }
}

#[cfg(test)]
mod tests {
    use super::{CoreStores, HotStoreApi, ProceduralStoreApi, Tier2StoreApi};
    use crate::migrate::DurableSchemaObject;

    #[test]
    fn core_stores_exposes_hot_store_component_identity() {
        let stores = CoreStores::default();

        assert_eq!(stores.hot_store_component_name(), "store.hot");
    }

    #[test]
    fn core_stores_hot_facade_matches_underlying_component_identity() {
        let stores = CoreStores::default();

        assert_eq!(
            stores.hot_store_component_name(),
            stores.hot().component_name()
        );
    }

    #[test]
    fn core_stores_hot_component_name_alias_matches_hot_store_accessor() {
        let stores = CoreStores::default();

        assert_eq!(
            stores.hot_component_name(),
            stores.hot_store_component_name()
        );
    }

    #[test]
    fn core_stores_exposes_audit_store_component_identity() {
        let stores = CoreStores::default();

        assert_eq!(stores.audit_component_name(), "store.audit");
    }

    #[test]
    fn core_stores_exposes_procedural_store_component_identity() {
        let stores = CoreStores::default();

        assert_eq!(stores.procedural().component_name(), "store.procedural");
    }

    #[test]
    fn core_stores_exposes_procedural_store_schema_objects() {
        let stores = CoreStores::default();

        assert_eq!(
            stores.procedural_authoritative_schema_objects(),
            stores.procedural().authoritative_schema_objects()
        );
    }

    #[test]
    fn tier2_store_schema_objects_include_compression_log_table() {
        let stores = CoreStores::default();
        let schema_objects = stores.tier2_authoritative_schema_objects();

        assert_eq!(
            schema_objects,
            stores.tier2().authoritative_schema_objects()
        );
        assert!(schema_objects.contains(&DurableSchemaObject::CompressionLogTable));
    }
}
