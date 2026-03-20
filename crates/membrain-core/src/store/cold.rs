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
    /// Archived records are retained in the cold tier, omitted from ordinary recall, but available to explicit cold-tier inspect and restore surfaces.
    Archived,
    /// Soft-forgotten: omitted from ordinary recall but available to explicit cold-tier recall and restore surfaces.
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

impl ColdRecord {
    /// Returns whether this record can still be surfaced to explicit cold-tier recall flows.
    pub const fn recall_visible(&self) -> bool {
        matches!(self.state, ArchiveState::Forgotten)
    }

    /// Returns whether this record may be restored back into active storage.
    pub const fn restorable(&self) -> bool {
        self.state.restorable()
    }

    /// Evaluates the restore outcome for this record without mutating storage.
    pub const fn restore_result(&self) -> RestoreResult {
        if self.restorable() {
            RestoreResult::Restored
        } else {
            RestoreResult::NotRestorable
        }
    }
}

/// Result of a restore operation.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RestoreResult {
    Restored,
    NotFound,
    NotRestorable,
}

impl RestoreResult {
    /// Returns the stable machine-readable restore outcome name.
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Restored => "restored",
            Self::NotFound => "not_found",
            Self::NotRestorable => "not_restorable",
        }
    }
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

    fn record(state: ArchiveState) -> ColdRecord {
        ColdRecord {
            memory_id: MemoryId(7),
            namespace: NamespaceId::new("test.namespace").unwrap(),
            state,
            compact_text: "cold summary".to_string(),
            original_tier: "hot",
        }
    }

    #[test]
    fn archive_states_have_correct_restorability() {
        assert!(ArchiveState::Archived.restorable());
        assert!(ArchiveState::Forgotten.restorable());
        assert!(!ArchiveState::Expired.restorable());
    }

    #[test]
    fn cold_record_visibility_and_restore_contract_follow_archive_state() {
        let archived = record(ArchiveState::Archived);
        assert!(!archived.recall_visible());
        assert!(archived.restorable());
        assert_eq!(archived.restore_result(), RestoreResult::Restored);

        let forgotten = record(ArchiveState::Forgotten);
        assert!(forgotten.recall_visible());
        assert!(forgotten.restorable());
        assert_eq!(forgotten.restore_result(), RestoreResult::Restored);

        let expired = record(ArchiveState::Expired);
        assert!(!expired.recall_visible());
        assert!(!expired.restorable());
        assert_eq!(expired.restore_result(), RestoreResult::NotRestorable);
    }

    #[test]
    fn restore_result_names_are_stable() {
        assert_eq!(RestoreResult::Restored.as_str(), "restored");
        assert_eq!(RestoreResult::NotFound.as_str(), "not_found");
        assert_eq!(RestoreResult::NotRestorable.as_str(), "not_restorable");
    }
}
