//! Durable schema migration and compatibility surfaces.
//!
//! Owns migration versioning, compatibility checks between core
//! schema generations, and upgrade/downgrade path validation.

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

impl MigrationModule {
    /// Returns the stable component identifier for this migration surface.
    pub const fn component_name(&self) -> &'static str {
        "migrate"
    }

    /// Returns the current runtime schema version.
    pub const fn current_version(&self) -> SchemaVersion {
        SchemaVersion::CURRENT
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
}
