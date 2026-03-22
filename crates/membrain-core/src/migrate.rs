//! Durable schema migration and compatibility surfaces.
//!
//! Owns migration versioning, compatibility checks between core
//! schema generations, and upgrade/downgrade path validation.

/// Stable durable schema objects owned by the canonical storage contract.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum DurableSchemaObject {
    MemoryItemsTable,
    MemoryPayloadsTable,
    MemoryLineageEdgesTable,
    MemoryEntityRefsTable,
    MemoryRelationRefsTable,
    MemoryTagsTable,
    ConflictRecordsTable,
    DurableMemoryRecords,
    EngramsTable,
    EngramMembershipTable,
    GraphEdgeTable,
}

impl DurableSchemaObject {
    /// Stable machine-readable table/object name.
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::MemoryItemsTable => "memory_items",
            Self::MemoryPayloadsTable => "memory_payloads",
            Self::MemoryLineageEdgesTable => "memory_lineage_edges",
            Self::MemoryEntityRefsTable => "memory_entity_refs",
            Self::MemoryRelationRefsTable => "memory_relation_refs",
            Self::MemoryTagsTable => "memory_tags",
            Self::ConflictRecordsTable => "conflict_records",
            Self::DurableMemoryRecords => "durable_memory_records",
            Self::EngramsTable => "engrams_table",
            Self::EngramMembershipTable => "engram_membership_table",
            Self::GraphEdgeTable => "graph_edge_table",
        }
    }
}

/// Schema version identifier for durable storage.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct SchemaVersion {
    /// Major schema version (breaking changes).
    pub major: u32,
    /// Minor schema version (additive changes).
    pub minor: u32,
}

impl SchemaVersion {
    /// The current canonical schema version.
    pub const CURRENT: Self = Self { major: 0, minor: 1 };

    /// Builds a new schema version.
    pub const fn new(major: u32, minor: u32) -> Self {
        Self { major, minor }
    }

    /// Whether this version is compatible with the target.
    pub const fn compatible_with(&self, target: &Self) -> bool {
        self.major == target.major && self.minor <= target.minor
    }
}

/// Migration direction between schema versions.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MigrationDirection {
    Upgrade,
    Downgrade,
    NoOp,
}

/// Result of a compatibility check between on-disk and runtime schema.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CompatibilityReport {
    /// On-disk schema version.
    pub on_disk: SchemaVersion,
    /// Runtime schema version.
    pub runtime: SchemaVersion,
    /// Required migration direction.
    pub direction: MigrationDirection,
    /// Whether automatic migration is safe.
    pub auto_safe: bool,
}

/// Stable migration boundary owned by `membrain-core`.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct MigrationModule;

/// Durable schema manifest for one runtime generation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DurableSchemaManifest {
    pub version: SchemaVersion,
    pub authoritative_tables: Vec<DurableSchemaObject>,
}

impl MigrationModule {
    /// Returns the stable component identifier for this migration surface.
    pub const fn component_name(&self) -> &'static str {
        "migrate"
    }

    /// Returns the current runtime schema version.
    pub const fn current_version(&self) -> SchemaVersion {
        SchemaVersion::CURRENT
    }

    /// Returns the current durable schema manifest.
    pub fn durable_schema_manifest(&self) -> DurableSchemaManifest {
        DurableSchemaManifest {
            version: SchemaVersion::CURRENT,
            authoritative_tables: vec![
                DurableSchemaObject::MemoryItemsTable,
                DurableSchemaObject::MemoryPayloadsTable,
                DurableSchemaObject::MemoryLineageEdgesTable,
                DurableSchemaObject::MemoryEntityRefsTable,
                DurableSchemaObject::MemoryRelationRefsTable,
                DurableSchemaObject::MemoryTagsTable,
                DurableSchemaObject::ConflictRecordsTable,
                DurableSchemaObject::DurableMemoryRecords,
                DurableSchemaObject::EngramsTable,
                DurableSchemaObject::EngramMembershipTable,
                DurableSchemaObject::GraphEdgeTable,
            ],
        }
    }

    /// Checks compatibility between on-disk and runtime schema versions.
    pub fn check_compatibility(&self, on_disk: SchemaVersion) -> CompatibilityReport {
        let runtime = SchemaVersion::CURRENT;
        let direction = if on_disk == runtime {
            MigrationDirection::NoOp
        } else if on_disk < runtime {
            MigrationDirection::Upgrade
        } else {
            MigrationDirection::Downgrade
        };

        let auto_safe = match direction {
            MigrationDirection::NoOp => true,
            MigrationDirection::Upgrade => on_disk.major == runtime.major,
            MigrationDirection::Downgrade => false,
        };

        CompatibilityReport {
            on_disk,
            runtime,
            direction,
            auto_safe,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn current_version_is_compatible_with_itself() {
        let v = SchemaVersion::CURRENT;
        assert!(v.compatible_with(&v));
    }

    #[test]
    fn same_version_is_noop() {
        let module = MigrationModule;
        let report = module.check_compatibility(SchemaVersion::CURRENT);
        assert_eq!(report.direction, MigrationDirection::NoOp);
        assert!(report.auto_safe);
    }

    #[test]
    fn older_minor_upgrades_safely() {
        let module = MigrationModule;
        let older = SchemaVersion::new(0, 0);
        let report = module.check_compatibility(older);
        assert_eq!(report.direction, MigrationDirection::Upgrade);
        assert!(report.auto_safe);
        assert!(older.compatible_with(&SchemaVersion::CURRENT));
    }

    #[test]
    fn newer_minor_is_not_compatible_with_current_runtime() {
        let newer_minor = SchemaVersion::new(0, 2);
        assert!(!newer_minor.compatible_with(&SchemaVersion::CURRENT));
    }

    #[test]
    fn newer_version_downgrades_unsafely() {
        let module = MigrationModule;
        let report = module.check_compatibility(SchemaVersion::new(1, 0));
        assert_eq!(report.direction, MigrationDirection::Downgrade);
        assert!(!report.auto_safe);
    }

    #[test]
    fn durable_schema_manifest_exposes_split_tier2_tables_as_first_class_objects() {
        let module = MigrationModule;
        let manifest = module.durable_schema_manifest();

        assert_eq!(manifest.version, SchemaVersion::CURRENT);
        assert!(manifest
            .authoritative_tables
            .contains(&DurableSchemaObject::MemoryItemsTable));
        assert!(manifest
            .authoritative_tables
            .contains(&DurableSchemaObject::MemoryPayloadsTable));
        assert!(manifest
            .authoritative_tables
            .contains(&DurableSchemaObject::MemoryLineageEdgesTable));
        assert!(manifest
            .authoritative_tables
            .contains(&DurableSchemaObject::MemoryEntityRefsTable));
        assert!(manifest
            .authoritative_tables
            .contains(&DurableSchemaObject::MemoryRelationRefsTable));
        assert!(manifest
            .authoritative_tables
            .contains(&DurableSchemaObject::MemoryTagsTable));
        assert!(manifest
            .authoritative_tables
            .contains(&DurableSchemaObject::ConflictRecordsTable));
        assert!(manifest
            .authoritative_tables
            .contains(&DurableSchemaObject::EngramsTable));
        assert!(manifest
            .authoritative_tables
            .contains(&DurableSchemaObject::EngramMembershipTable));
        assert!(manifest
            .authoritative_tables
            .contains(&DurableSchemaObject::GraphEdgeTable));
    }

    #[test]
    fn durable_schema_object_names_stay_machine_readable() {
        assert_eq!(
            DurableSchemaObject::MemoryItemsTable.as_str(),
            "memory_items"
        );
        assert_eq!(
            DurableSchemaObject::MemoryPayloadsTable.as_str(),
            "memory_payloads"
        );
        assert_eq!(
            DurableSchemaObject::MemoryLineageEdgesTable.as_str(),
            "memory_lineage_edges"
        );
        assert_eq!(
            DurableSchemaObject::MemoryEntityRefsTable.as_str(),
            "memory_entity_refs"
        );
        assert_eq!(
            DurableSchemaObject::MemoryRelationRefsTable.as_str(),
            "memory_relation_refs"
        );
        assert_eq!(DurableSchemaObject::MemoryTagsTable.as_str(), "memory_tags");
        assert_eq!(
            DurableSchemaObject::ConflictRecordsTable.as_str(),
            "conflict_records"
        );
        assert_eq!(DurableSchemaObject::EngramsTable.as_str(), "engrams_table");
        assert_eq!(
            DurableSchemaObject::EngramMembershipTable.as_str(),
            "engram_membership_table"
        );
        assert_eq!(
            DurableSchemaObject::GraphEdgeTable.as_str(),
            "graph_edge_table"
        );
    }
}
