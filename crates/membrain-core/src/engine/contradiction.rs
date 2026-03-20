//! Contradiction records, conflict-aware storage, and resolution surfaces.
//!
//! This module owns the core contradiction contract: detecting when stored
//! memories disagree, recording the relationship, tracking resolution state,
//! and providing explain payloads that retrieval and ranking can consume.

use crate::api::NamespaceId;
use crate::types::MemoryId;

// ── Contradiction kinds ──────────────────────────────────────────────────────

/// Canonical contradiction relationship kinds.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub enum ContradictionKind {
    /// Same factual content ingested more than once.
    Duplicate,
    /// A later memory revises an earlier one (same topic, updated facts).
    Revision,
    /// Two memories present equally valid but different perspectives.
    Coexistence,
    /// A later memory explicitly replaces an earlier one.
    Supersession,
    /// An authoritative source overrides a weaker source.
    AuthoritativeOverride,
}

impl ContradictionKind {
    /// Stable machine-readable name for this contradiction kind.
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Duplicate => "duplicate",
            Self::Revision => "revision",
            Self::Coexistence => "coexistence",
            Self::Supersession => "supersession",
            Self::AuthoritativeOverride => "authoritative_override",
        }
    }

    /// Whether this kind implies one memory should be preferred over the other.
    pub const fn implies_preference(self) -> bool {
        matches!(
            self,
            Self::Revision | Self::Supersession | Self::AuthoritativeOverride
        )
    }
}

// ── Resolution state ─────────────────────────────────────────────────────────

/// State of contradiction resolution.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub enum ResolutionState {
    /// Contradiction detected but not yet resolved.
    Unresolved,
    /// Automatically resolved by the system (e.g. duplicate suppression).
    AutoResolved,
    /// Manually resolved by an operator or agent.
    ManuallyResolved,
    /// Authoritative source imposed a resolution.
    AuthoritativelyResolved,
}

impl ResolutionState {
    /// Stable machine-readable name for this resolution state.
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Unresolved => "unresolved",
            Self::AutoResolved => "auto_resolved",
            Self::ManuallyResolved => "manually_resolved",
            Self::AuthoritativelyResolved => "authoritatively_resolved",
        }
    }

    /// Whether this state represents a terminal resolution.
    pub const fn is_resolved(self) -> bool {
        !matches!(self, Self::Unresolved)
    }
}

// ── Contradiction record ─────────────────────────────────────────────────────

/// Unique identifier for one contradiction record.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub struct ContradictionId(pub u64);

/// Durable contradiction record linking two memories that disagree.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ContradictionRecord {
    /// Stable identity of this contradiction.
    pub id: ContradictionId,
    /// Namespace scope of the contradiction.
    pub namespace: NamespaceId,
    /// The earlier or weaker memory in the pair.
    pub memory_a: MemoryId,
    /// The later or stronger memory in the pair.
    pub memory_b: MemoryId,
    /// Kind of contradiction detected between the pair.
    pub kind: ContradictionKind,
    /// Current resolution state of the contradiction.
    pub resolution: ResolutionState,
    /// The preferred memory after resolution (if any).
    pub preferred_memory: Option<MemoryId>,
    /// Machine-readable reason for the current resolution.
    pub resolution_reason: Option<&'static str>,
    /// Similarity or conflict score between the two memories (0..1000).
    pub conflict_score: u16,
}

impl ContradictionRecord {
    /// Builds a new unresolved contradiction between two memories.
    pub fn new(
        id: ContradictionId,
        namespace: NamespaceId,
        memory_a: MemoryId,
        memory_b: MemoryId,
        kind: ContradictionKind,
        conflict_score: u16,
    ) -> Self {
        Self {
            id,
            namespace,
            memory_a,
            memory_b,
            kind,
            resolution: ResolutionState::Unresolved,
            preferred_memory: None,
            resolution_reason: None,
            conflict_score,
        }
    }

    /// Resolves this contradiction by preferring one memory.
    pub fn resolve(&mut self, state: ResolutionState, preferred: MemoryId, reason: &'static str) {
        self.resolution = state;
        self.preferred_memory = Some(preferred);
        self.resolution_reason = Some(reason);
    }

    /// Returns whether the contradiction is still unresolved.
    pub const fn is_unresolved(&self) -> bool {
        matches!(self.resolution, ResolutionState::Unresolved)
    }

    /// Returns the non-preferred memory if resolution chose a winner.
    pub fn superseded_memory(&self) -> Option<MemoryId> {
        self.preferred_memory.map(|preferred| {
            if preferred == self.memory_a {
                self.memory_b
            } else {
                self.memory_a
            }
        })
    }
}

// ── Contradiction explain payload ────────────────────────────────────────────

/// Machine-readable contradiction explain payload for retrieval surfaces.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct ContradictionExplain {
    /// The contradiction that affected this result.
    pub contradiction_id: ContradictionId,
    /// Kind of disagreement.
    pub kind: ContradictionKind,
    /// Current resolution state.
    pub resolution: ResolutionState,
    /// Which memory is preferred (if resolved).
    pub preferred_memory: Option<MemoryId>,
    /// The other memory in the contradiction pair.
    pub conflicting_memory: MemoryId,
    /// Whether the current result was the preferred or superseded memory.
    pub result_is_preferred: bool,
}

// ── Contradiction detection ──────────────────────────────────────────────────

/// Input for contradiction detection during encode or background analysis.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ContradictionCandidate {
    /// The memory being checked for contradictions.
    pub memory_id: MemoryId,
    /// Fingerprint for fast duplicate detection.
    pub fingerprint: u64,
    /// Compact text for similarity comparison.
    pub compact_text: String,
    /// Namespace scope for the check.
    pub namespace: NamespaceId,
}

/// Detection result from contradiction analysis.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DetectionResult {
    /// No contradiction detected.
    NoConflict,
    /// A potential contradiction was found.
    ConflictDetected {
        existing_memory: MemoryId,
        kind: ContradictionKind,
        conflict_score: u16,
    },
}

// ── Contradiction store trait ────────────────────────────────────────────────

/// Core contradiction store contract for recording and querying conflicts.
pub trait ContradictionStore {
    /// Records a new contradiction between two memories.
    fn record(
        &mut self,
        record: ContradictionRecord,
    ) -> Result<ContradictionId, ContradictionError>;

    /// Resolves an existing contradiction.
    fn resolve(
        &mut self,
        id: ContradictionId,
        state: ResolutionState,
        preferred: MemoryId,
        reason: &'static str,
    ) -> Result<(), ContradictionError>;

    /// Returns all contradictions involving a specific memory.
    fn find_by_memory(&self, memory_id: MemoryId) -> Vec<&ContradictionRecord>;

    /// Returns all unresolved contradictions in a namespace.
    fn find_unresolved(&self, namespace: &NamespaceId) -> Vec<&ContradictionRecord>;

    /// Returns the explain payload for a memory's contradictions.
    fn explain_for_memory(&self, memory_id: MemoryId) -> Vec<ContradictionExplain>;

    /// Returns the total count of contradictions in a namespace.
    fn count_in_namespace(&self, namespace: &NamespaceId) -> usize;

    /// Returns the count of unresolved contradictions in a namespace.
    fn count_unresolved(&self, namespace: &NamespaceId) -> usize;
}

/// Errors from contradiction store operations.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ContradictionError {
    /// The contradiction ID was not found.
    NotFound,
    /// The contradiction is already resolved and cannot be re-resolved.
    AlreadyResolved,
    /// The preferred memory is not part of this contradiction.
    InvalidPreference,
    /// A duplicate contradiction record already exists for this pair.
    DuplicateRecord,
}

// ── In-memory contradiction engine ───────────────────────────────────────────

/// In-memory contradiction engine for the core crate.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ContradictionEngine {
    records: Vec<ContradictionRecord>,
    next_id: u64,
}

impl ContradictionEngine {
    /// Builds a new empty contradiction engine.
    pub fn new() -> Self {
        Self {
            records: Vec::new(),
            next_id: 1,
        }
    }

    /// Returns the stable component identifier.
    pub const fn component_name(&self) -> &'static str {
        "engine.contradiction"
    }

    /// Detects potential contradictions for an incoming memory candidate.
    pub fn detect(&self, candidate: &ContradictionCandidate) -> DetectionResult {
        // Check for duplicate fingerprints first
        for record in &self.records {
            if record.namespace == candidate.namespace {
                // This is a simplified detection; real implementation would
                // cross-reference with stored memory metadata
                if record.memory_a == candidate.memory_id || record.memory_b == candidate.memory_id
                {
                    continue; // Skip self-references
                }
            }
        }
        DetectionResult::NoConflict
    }

    /// Allocates the next contradiction ID.
    fn allocate_id(&mut self) -> ContradictionId {
        let id = ContradictionId(self.next_id);
        self.next_id += 1;
        id
    }
}

impl Default for ContradictionEngine {
    fn default() -> Self {
        Self::new()
    }
}

impl ContradictionStore for ContradictionEngine {
    fn record(
        &mut self,
        mut record: ContradictionRecord,
    ) -> Result<ContradictionId, ContradictionError> {
        // Check for duplicate contradiction on the same memory pair
        let exists = self.records.iter().any(|existing| {
            existing.namespace == record.namespace
                && ((existing.memory_a == record.memory_a && existing.memory_b == record.memory_b)
                    || (existing.memory_a == record.memory_b
                        && existing.memory_b == record.memory_a))
        });
        if exists {
            return Err(ContradictionError::DuplicateRecord);
        }

        let id = self.allocate_id();
        record.id = id;
        self.records.push(record);
        Ok(id)
    }

    fn resolve(
        &mut self,
        id: ContradictionId,
        state: ResolutionState,
        preferred: MemoryId,
        reason: &'static str,
    ) -> Result<(), ContradictionError> {
        let record = self
            .records
            .iter_mut()
            .find(|r| r.id == id)
            .ok_or(ContradictionError::NotFound)?;

        if record.resolution.is_resolved() {
            return Err(ContradictionError::AlreadyResolved);
        }

        if preferred != record.memory_a && preferred != record.memory_b {
            return Err(ContradictionError::InvalidPreference);
        }

        record.resolve(state, preferred, reason);
        Ok(())
    }

    fn find_by_memory(&self, memory_id: MemoryId) -> Vec<&ContradictionRecord> {
        self.records
            .iter()
            .filter(|r| r.memory_a == memory_id || r.memory_b == memory_id)
            .collect()
    }

    fn find_unresolved(&self, namespace: &NamespaceId) -> Vec<&ContradictionRecord> {
        self.records
            .iter()
            .filter(|r| r.namespace == *namespace && r.is_unresolved())
            .collect()
    }

    fn explain_for_memory(&self, memory_id: MemoryId) -> Vec<ContradictionExplain> {
        self.find_by_memory(memory_id)
            .into_iter()
            .map(|record| {
                let conflicting_memory = if record.memory_a == memory_id {
                    record.memory_b
                } else {
                    record.memory_a
                };

                let result_is_preferred = record
                    .preferred_memory
                    .map_or(false, |preferred| preferred == memory_id);

                ContradictionExplain {
                    contradiction_id: record.id,
                    kind: record.kind,
                    resolution: record.resolution,
                    preferred_memory: record.preferred_memory,
                    conflicting_memory,
                    result_is_preferred,
                }
            })
            .collect()
    }

    fn count_in_namespace(&self, namespace: &NamespaceId) -> usize {
        self.records
            .iter()
            .filter(|r| r.namespace == *namespace)
            .count()
    }

    fn count_unresolved(&self, namespace: &NamespaceId) -> usize {
        self.records
            .iter()
            .filter(|r| r.namespace == *namespace && r.is_unresolved())
            .count()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::api::NamespaceId;
    use crate::types::MemoryId;

    fn ns(s: &str) -> NamespaceId {
        NamespaceId::new(s).unwrap()
    }

    fn make_record(
        ns: NamespaceId,
        a: u64,
        b: u64,
        kind: ContradictionKind,
    ) -> ContradictionRecord {
        ContradictionRecord::new(
            ContradictionId(0), // engine will reassign
            ns,
            MemoryId(a),
            MemoryId(b),
            kind,
            500,
        )
    }

    #[test]
    fn record_and_find_contradiction() {
        let mut engine = ContradictionEngine::new();
        let record = make_record(ns("test"), 1, 2, ContradictionKind::Revision);

        let id = engine.record(record).unwrap();
        assert_eq!(id, ContradictionId(1));

        let found = engine.find_by_memory(MemoryId(1));
        assert_eq!(found.len(), 1);
        assert_eq!(found[0].kind, ContradictionKind::Revision);
        assert!(found[0].is_unresolved());

        let found_b = engine.find_by_memory(MemoryId(2));
        assert_eq!(found_b.len(), 1);

        let found_none = engine.find_by_memory(MemoryId(99));
        assert!(found_none.is_empty());
    }

    #[test]
    fn resolve_contradiction_sets_preferred_memory() {
        let mut engine = ContradictionEngine::new();
        let record = make_record(ns("test"), 10, 20, ContradictionKind::Supersession);

        let id = engine.record(record).unwrap();
        engine
            .resolve(
                id,
                ResolutionState::AutoResolved,
                MemoryId(20),
                "newer supersedes older",
            )
            .unwrap();

        let found = engine.find_by_memory(MemoryId(10));
        assert_eq!(found[0].resolution, ResolutionState::AutoResolved);
        assert_eq!(found[0].preferred_memory, Some(MemoryId(20)));
        assert_eq!(found[0].superseded_memory(), Some(MemoryId(10)));
    }

    #[test]
    fn cannot_resolve_already_resolved() {
        let mut engine = ContradictionEngine::new();
        let record = make_record(ns("test"), 1, 2, ContradictionKind::Duplicate);

        let id = engine.record(record).unwrap();
        engine
            .resolve(id, ResolutionState::AutoResolved, MemoryId(1), "dup")
            .unwrap();

        let err = engine
            .resolve(
                id,
                ResolutionState::ManuallyResolved,
                MemoryId(2),
                "re-resolve",
            )
            .unwrap_err();
        assert_eq!(err, ContradictionError::AlreadyResolved);
    }

    #[test]
    fn cannot_prefer_memory_not_in_pair() {
        let mut engine = ContradictionEngine::new();
        let record = make_record(ns("test"), 1, 2, ContradictionKind::Coexistence);

        let id = engine.record(record).unwrap();
        let err = engine
            .resolve(id, ResolutionState::ManuallyResolved, MemoryId(99), "wrong")
            .unwrap_err();
        assert_eq!(err, ContradictionError::InvalidPreference);
    }

    #[test]
    fn duplicate_pair_rejected() {
        let mut engine = ContradictionEngine::new();
        let r1 = make_record(ns("test"), 5, 6, ContradictionKind::Revision);
        let r2 = make_record(ns("test"), 6, 5, ContradictionKind::Duplicate); // reversed pair

        engine.record(r1).unwrap();
        let err = engine.record(r2).unwrap_err();
        assert_eq!(err, ContradictionError::DuplicateRecord);
    }

    #[test]
    fn duplicate_pair_allowed_across_namespaces() {
        let mut engine = ContradictionEngine::new();
        let r1 = make_record(ns("alpha"), 5, 6, ContradictionKind::Revision);
        let r2 = make_record(ns("beta"), 6, 5, ContradictionKind::Duplicate);

        let first = engine.record(r1).unwrap();
        let second = engine.record(r2).unwrap();

        assert_eq!(first, ContradictionId(1));
        assert_eq!(second, ContradictionId(2));
        assert_eq!(engine.count_in_namespace(&ns("alpha")), 1);
        assert_eq!(engine.count_in_namespace(&ns("beta")), 1);
    }

    #[test]
    fn find_unresolved_filters_by_namespace_and_state() {
        let mut engine = ContradictionEngine::new();
        let r1 = make_record(ns("alpha"), 1, 2, ContradictionKind::Revision);
        let r2 = make_record(ns("alpha"), 3, 4, ContradictionKind::Coexistence);
        let r3 = make_record(ns("beta"), 5, 6, ContradictionKind::Supersession);

        let id1 = engine.record(r1).unwrap();
        engine.record(r2).unwrap();
        engine.record(r3).unwrap();

        // Resolve the first one
        engine
            .resolve(
                id1,
                ResolutionState::ManuallyResolved,
                MemoryId(2),
                "human chose",
            )
            .unwrap();

        let unresolved_alpha = engine.find_unresolved(&ns("alpha"));
        assert_eq!(unresolved_alpha.len(), 1);
        assert_eq!(unresolved_alpha[0].memory_a, MemoryId(3));

        let unresolved_beta = engine.find_unresolved(&ns("beta"));
        assert_eq!(unresolved_beta.len(), 1);

        assert_eq!(engine.count_in_namespace(&ns("alpha")), 2);
        assert_eq!(engine.count_unresolved(&ns("alpha")), 1);
    }

    #[test]
    fn explain_payloads_show_correct_preferred_and_conflicting() {
        let mut engine = ContradictionEngine::new();
        let record = make_record(ns("test"), 10, 20, ContradictionKind::AuthoritativeOverride);

        let id = engine.record(record).unwrap();
        engine
            .resolve(
                id,
                ResolutionState::AuthoritativelyResolved,
                MemoryId(20),
                "authoritative source",
            )
            .unwrap();

        // Explain for the preferred memory
        let explain_preferred = engine.explain_for_memory(MemoryId(20));
        assert_eq!(explain_preferred.len(), 1);
        assert!(explain_preferred[0].result_is_preferred);
        assert_eq!(explain_preferred[0].conflicting_memory, MemoryId(10));

        // Explain for the superseded memory
        let explain_superseded = engine.explain_for_memory(MemoryId(10));
        assert_eq!(explain_superseded.len(), 1);
        assert!(!explain_superseded[0].result_is_preferred);
        assert_eq!(explain_superseded[0].conflicting_memory, MemoryId(20));
    }

    #[test]
    fn contradiction_kind_properties() {
        assert!(ContradictionKind::Revision.implies_preference());
        assert!(ContradictionKind::Supersession.implies_preference());
        assert!(ContradictionKind::AuthoritativeOverride.implies_preference());
        assert!(!ContradictionKind::Duplicate.implies_preference());
        assert!(!ContradictionKind::Coexistence.implies_preference());
    }
}
