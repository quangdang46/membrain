use crate::api::NamespaceId;
use crate::migrate::DurableSchemaObject;
use crate::policy::SharingVisibility;
use crate::store::ProceduralStoreApi;
use crate::types::MemoryId;
use xxhash_rust::xxh64::xxh64;

/// Authoritative lifecycle state for one accepted procedural entry.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ProceduralEntryState {
    Active,
    RolledBack,
}

impl ProceduralEntryState {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Active => "active",
            Self::RolledBack => "rolled_back",
        }
    }
}

/// Accepted pattern→action mapping preserved in the authoritative procedural store.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProceduralMemoryRecord {
    pub namespace: NamespaceId,
    pub pattern_handle: String,
    pub pattern_hash: u64,
    pub pattern: String,
    pub action: String,
    pub confidence: u16,
    pub source_fixture_name: String,
    pub source_engram_id: Option<u64>,
    pub lineage_ancestors: Vec<MemoryId>,
    pub supporting_memory_count: usize,
    pub source_citation_count: usize,
    pub query_cues: Vec<String>,
    pub accepted_by: String,
    pub acceptance_note: String,
    pub visibility: SharingVisibility,
    pub state: ProceduralEntryState,
    pub version: u32,
    pub promotion_audit_sequence: u64,
    pub last_transition_sequence: u64,
    pub last_transition_kind: &'static str,
    pub rollback_note: Option<String>,
}

/// Machine-readable failure reasons for procedural-store mutations.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProceduralStoreErrorReason {
    CandidateNotFound,
    EntryNotFound,
    AlreadyRolledBack,
    PolicyDenied,
}

impl ProceduralStoreErrorReason {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::CandidateNotFound => "candidate_not_found",
            Self::EntryNotFound => "entry_not_found",
            Self::AlreadyRolledBack => "already_rolled_back",
            Self::PolicyDenied => "policy_denied",
        }
    }
}

/// Stable error envelope for procedural-store operations.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProceduralStoreError {
    pub namespace: NamespaceId,
    pub pattern_handle: String,
    pub reason: ProceduralStoreErrorReason,
}

impl ProceduralStoreError {
    pub fn new(
        namespace: NamespaceId,
        pattern_handle: impl Into<String>,
        reason: ProceduralStoreErrorReason,
    ) -> Self {
        Self {
            namespace,
            pattern_handle: pattern_handle.into(),
            reason,
        }
    }
}

/// Dedicated authoritative procedural-store boundary.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct ProceduralStore;

impl ProceduralStore {
    /// Computes the stable hash used for O(1)-style procedural lookup inside one namespace.
    pub fn pattern_hash(namespace: &NamespaceId, pattern: &str) -> u64 {
        let mut input = Vec::with_capacity(namespace.as_str().len() + pattern.len() + 1);
        input.extend_from_slice(namespace.as_str().as_bytes());
        input.push(0xff);
        input.extend_from_slice(pattern.trim().as_bytes());
        xxh64(&input, 0)
    }

    /// Builds the stable lookup handle for one accepted procedural entry.
    pub fn pattern_handle(namespace: &NamespaceId, pattern: &str) -> String {
        format!(
            "procedural://{}/{:016x}",
            namespace.as_str(),
            Self::pattern_hash(namespace, pattern)
        )
    }

    /// Returns the canonical direct-lookup strategy used by the procedural store.
    pub const fn lookup_strategy(&self) -> &'static str {
        "pattern_hash_exact"
    }

    /// Returns whether procedural lookup ever requires full recall traversal.
    pub const fn requires_full_recall_traversal(&self) -> bool {
        false
    }
}

impl ProceduralStoreApi for ProceduralStore {
    fn component_name(&self) -> &'static str {
        "store.procedural"
    }

    fn authoritative_schema_objects(&self) -> Vec<DurableSchemaObject> {
        vec![
            DurableSchemaObject::ProceduralMemoriesTable,
            DurableSchemaObject::ProceduralLineageTable,
        ]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pattern_handle_is_stable_and_namespace_scoped() {
        let alpha = NamespaceId::new("team.alpha").unwrap();
        let beta = NamespaceId::new("team.beta").unwrap();

        let first = ProceduralStore::pattern_handle(&alpha, "deploy after incident");
        let second = ProceduralStore::pattern_handle(&alpha, "deploy after incident");
        let other_namespace = ProceduralStore::pattern_handle(&beta, "deploy after incident");

        assert_eq!(first, second);
        assert_ne!(first, other_namespace);
        assert!(first.starts_with("procedural://team.alpha/"));
    }

    #[test]
    fn procedural_store_exposes_authoritative_schema_objects() {
        let store = ProceduralStore;

        assert_eq!(
            store.authoritative_schema_objects(),
            vec![
                DurableSchemaObject::ProceduralMemoriesTable,
                DurableSchemaObject::ProceduralLineageTable,
            ]
        );
    }

    #[test]
    fn procedural_lookup_contract_stays_direct() {
        let store = ProceduralStore;

        assert_eq!(store.lookup_strategy(), "pattern_hash_exact");
        assert!(!store.requires_full_recall_traversal());
    }
}
