//! Cold/archive store boundary.
//!
//! Owns cold-tier storage for archived, demoted, and long-retention
//! memories. Provides restore surfaces for retrieving forgotten items.

use crate::api::NamespaceId;
use crate::store::ColdStoreApi;
use crate::types::MemoryId;

/// Archive state of a cold-stored memory.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ArchiveState {
    /// Actively archived but retrievable.
    Archived,
    /// Soft-forgotten: hidden from recall but restorable.
    Forgotten,
    /// Permanently expired and eligible for cleanup.
    Expired,
}

impl ArchiveState {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Archived => "archived",
            Self::Forgotten => "forgotten",
            Self::Expired => "expired",
        }
    }

    pub const fn restorable(self) -> bool {
        matches!(self, Self::Archived | Self::Forgotten)
    }
}

/// Metadata record for one cold-stored memory.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ColdRecord {
    pub memory_id: MemoryId,
    pub namespace: NamespaceId,
    pub state: ArchiveState,
    pub compact_text: String,
    pub original_tier: &'static str,
}

/// Result of a restore operation.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RestoreResult {
    Restored,
    NotFound,
    NotRestorable,
}

/// Cold/archive store boundary owned by `membrain-core`.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct ColdStore;

impl ColdStore {
    /// Returns the stable component identifier.
    pub const fn component_name_inner(&self) -> &'static str {
        "store.cold"
    }
}

impl ColdStoreApi for ColdStore {
    fn component_name(&self) -> &'static str {
        "store.cold"
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn archive_states_have_correct_restorability() {
        assert!(ArchiveState::Archived.restorable());
        assert!(ArchiveState::Forgotten.restorable());
        assert!(!ArchiveState::Expired.restorable());
    }
}
