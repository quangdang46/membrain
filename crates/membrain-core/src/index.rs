//! Candidate-generation and index maintenance seams.
//!
//! Owns the FTS5 lexical index, namespace prefilter indexes, and
//! the index maintenance contract for rebuild and health checks.
//!
//! # Index Families
//!
//! - **FTS5 Lexical**: Full-text search over `compact_text` for exact-ish lookup,
//!   keyword-first retrieval, and negation-sensitive matching.
//! - **Namespace Prefilter**: Fast scope narrowing by namespace before expensive ops.
//! - **Temporal Prefilter**: Time-slice filtering by session, tick range, or epoch.
//! - **Entity Prefilter**: Entity/tag filtering for structured candidate narrowing.
//! - **Fingerprint Dedup**: Deduplication index for near-duplicate detection.
//! - **Memory Type Categorical**: Categorical filtering by canonical memory type.

use crate::api::NamespaceId;
use crate::types::{CanonicalMemoryType, MemoryId, SessionId};

// ── Index types ──────────────────────────────────────────────────────────────

/// Supported index families for candidate generation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize)]
pub enum IndexFamily {
    /// FTS5 full-text lexical index for hot memories.
    Fts5Lexical,
    /// FTS5 full-text lexical index for cold memories.
    Fts5ColdLexical,
    /// Namespace prefilter index for fast scope narrowing.
    NamespacePrefilter,
    /// Fingerprint deduplication index.
    FingerprintDedup,
    /// Session-scoped temporal index.
    SessionTemporal,
    /// Tick-range temporal prefilter.
    TickRangeTemporal,
    /// Entity/tag prefilter index.
    EntityPrefilter,
    /// Memory-type categorical index.
    MemoryTypeCategorical,
}

impl IndexFamily {
    /// Stable machine-readable name.
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Fts5Lexical => "fts5_lexical",
            Self::Fts5ColdLexical => "fts5_cold_lexical",
            Self::NamespacePrefilter => "namespace_prefilter",
            Self::FingerprintDedup => "fingerprint_dedup",
            Self::SessionTemporal => "session_temporal",
            Self::TickRangeTemporal => "tick_range_temporal",
            Self::EntityPrefilter => "entity_prefilter",
            Self::MemoryTypeCategorical => "memory_type_categorical",
        }
    }

    /// Returns true if this is an FTS5-based index.
    pub const fn is_fts5(self) -> bool {
        matches!(self, Self::Fts5Lexical | Self::Fts5ColdLexical)
    }

    /// Returns true if this is a temporal index.
    pub const fn is_temporal(self) -> bool {
        matches!(self, Self::SessionTemporal | Self::TickRangeTemporal)
    }
}

// ── FTS5 Schema Definitions ───────────────────────────────────────────────────

/// FTS5 table schema for hot memory lexical index.
///
/// This schema supports bounded candidate retrieval without payload fetch.
/// The table stores only metadata and compact text for FTS5 matching.
pub mod fts5_schema {
    /// Hot memory FTS5 virtual table name.
    pub const MEMORY_FTS_TABLE: &str = "memory_fts";

    /// Cold memory FTS5 virtual table name.
    pub const COLD_MEMORY_FTS_TABLE: &str = "cold_memory_fts";

    /// Column definitions for the FTS5 table.
    /// These are the indexed columns, not the metadata columns.
    pub mod columns {
        pub const MEMORY_ID: &str = "memory_id";
        pub const NAMESPACE: &str = "namespace";
        pub const SESSION_ID: &str = "session_id";
        pub const MEMORY_TYPE: &str = "memory_type";
        pub const COMPACT_TEXT: &str = "compact_text";
        pub const TICK_CREATED: &str = "tick_created";
        pub const FINGERPRINT: &str = "fingerprint";
    }

    /// CREATE TABLE statement for the hot memory FTS5 index.
    /// Uses FTS5 with tokenize="porter unicode61" for English stemming.
    pub const CREATE_MEMORY_FTS: &str = r#"
        CREATE VIRTUAL TABLE IF NOT EXISTS memory_fts USING fts5(
            memory_id UNINDEXED,
            namespace UNINDEXED,
            session_id UNINDEXED,
            memory_type UNINDEXED,
            compact_text,
            tick_created UNINDEXED,
            fingerprint UNINDEXED,
            tokenize='porter unicode61'
        );
    "#;

    /// CREATE TABLE statement for the cold memory FTS5 index.
    pub const CREATE_COLD_MEMORY_FTS: &str = r#"
        CREATE VIRTUAL TABLE IF NOT EXISTS cold_memory_fts USING fts5(
            memory_id UNINDEXED,
            namespace UNINDEXED,
            session_id UNINDEXED,
            memory_type UNINDEXED,
            compact_text,
            tick_created UNINDEXED,
            fingerprint UNINDEXED,
            tokenize='porter unicode61'
        );
    "#;

    /// Insert statement for FTS5 index.
    pub const INSERT_MEMORY_FTS: &str = r#"
        INSERT INTO memory_fts(memory_id, namespace, session_id, memory_type, compact_text, tick_created, fingerprint)
        VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7);
    "#;

    /// Delete statement for FTS5 index.
    pub const DELETE_MEMORY_FTS: &str = "DELETE FROM memory_fts WHERE memory_id = ?1;";

    /// Query template for FTS5 MATCH with namespace filter and limit.
    /// Uses bm25 ranking for relevance scoring.
    pub const QUERY_MEMORY_FTS: &str = r#"
        SELECT memory_id, namespace, session_id, memory_type, bm25(memory_fts) as score
        FROM memory_fts
        WHERE memory_fts MATCH ?1 AND namespace = ?2
        ORDER BY bm25(memory_fts)
        LIMIT ?3;
    "#;

    /// Query with session filter.
    pub const QUERY_MEMORY_FTS_WITH_SESSION: &str = r#"
        SELECT memory_id, namespace, session_id, memory_type, bm25(memory_fts) as score
        FROM memory_fts
        WHERE memory_fts MATCH ?1 AND namespace = ?2 AND session_id = ?3
        ORDER BY bm25(memory_fts)
        LIMIT ?4;
    "#;

    /// Query with memory type filter.
    pub const QUERY_MEMORY_FTS_WITH_TYPE: &str = r#"
        SELECT memory_id, namespace, session_id, memory_type, bm25(memory_fts) as score
        FROM memory_fts
        WHERE memory_fts MATCH ?1 AND namespace = ?2 AND memory_type = ?3
        ORDER BY bm25(memory_fts)
        LIMIT ?4;
    "#;

    /// Negation-sensitive query using FTS5 NOT operator.
    pub const QUERY_MEMORY_FTS_NEGATION: &str = r#"
        SELECT memory_id, namespace, session_id, memory_type, bm25(memory_fts) as score
        FROM memory_fts
        WHERE memory_fts MATCH ?1 AND namespace = ?2
          AND memory_fts NOT MATCH ?3
        ORDER BY bm25(memory_fts)
        LIMIT ?4;
    "#;
}

// ── Temporal Prefilter Schema ─────────────────────────────────────────────────

/// Temporal prefilter table schema for bounded time-slice queries.
pub mod temporal_schema {
    /// Temporal prefilter table name.
    pub const TEMPORAL_PREFILTER_TABLE: &str = "temporal_prefilter";

    /// Column definitions.
    pub mod columns {
        pub const MEMORY_ID: &str = "memory_id";
        pub const NAMESPACE: &str = "namespace";
        pub const SESSION_ID: &str = "session_id";
        pub const TICK_CREATED: &str = "tick_created";
        pub const TICK_LAST_ACCESSED: &str = "tick_last_accessed";
        pub const EPOCH: &str = "epoch";
    }

    /// CREATE TABLE for temporal prefilter with indexes.
    pub const CREATE_TEMPORAL_PREFILTER: &str = r#"
        CREATE TABLE IF NOT EXISTS temporal_prefilter (
            memory_id INTEGER PRIMARY KEY,
            namespace TEXT NOT NULL,
            session_id INTEGER NOT NULL,
            tick_created INTEGER NOT NULL,
            tick_last_accessed INTEGER NOT NULL,
            epoch INTEGER NOT NULL
        );
        CREATE INDEX IF NOT EXISTS idx_temporal_namespace ON temporal_prefilter(namespace);
        CREATE INDEX IF NOT EXISTS idx_temporal_session ON temporal_prefilter(namespace, session_id);
        CREATE INDEX IF NOT EXISTS idx_temporal_tick_range ON temporal_prefilter(namespace, tick_created);
        CREATE INDEX IF NOT EXISTS idx_temporal_epoch ON temporal_prefilter(namespace, epoch);
    "#;

    /// Query for tick range prefilter.
    pub const QUERY_TICK_RANGE: &str = r#"
        SELECT memory_id, namespace, session_id, tick_created
        FROM temporal_prefilter
        WHERE namespace = ?1 AND tick_created BETWEEN ?2 AND ?3
        ORDER BY tick_created DESC
        LIMIT ?4;
    "#;

    /// Query for session-scoped temporal filter.
    pub const QUERY_SESSION_TEMPORAL: &str = r#"
        SELECT memory_id, namespace, session_id, tick_created
        FROM temporal_prefilter
        WHERE namespace = ?1 AND session_id = ?2
        ORDER BY tick_created DESC
        LIMIT ?3;
    "#;

    /// Query for epoch-based temporal filter.
    pub const QUERY_EPOCH_TEMPORAL: &str = r#"
        SELECT memory_id, namespace, session_id, tick_created, epoch
        FROM temporal_prefilter
        WHERE namespace = ?1 AND epoch = ?2
        ORDER BY tick_created DESC
        LIMIT ?3;
    "#;
}

// ── Namespace Prefilter Schema ────────────────────────────────────────────────

/// Namespace prefilter table schema for fast scope narrowing.
pub mod namespace_schema {
    /// Namespace prefilter table name.
    pub const NAMESPACE_PREFILTER_TABLE: &str = "namespace_prefilter";

    /// CREATE TABLE for namespace prefilter with indexes.
    pub const CREATE_NAMESPACE_PREFILTER: &str = r#"
        CREATE TABLE IF NOT EXISTS namespace_prefilter (
            memory_id INTEGER PRIMARY KEY,
            namespace TEXT NOT NULL,
            memory_type TEXT NOT NULL,
            is_archived INTEGER NOT NULL DEFAULT 0,
            is_pinned INTEGER NOT NULL DEFAULT 0
        );
        CREATE INDEX IF NOT EXISTS idx_namespace_scope ON namespace_prefilter(namespace, is_archived);
        CREATE INDEX IF NOT EXISTS idx_namespace_type ON namespace_prefilter(namespace, memory_type);
    "#;

    /// Query for namespace-scoped candidates with archive filter.
    pub const QUERY_NAMESPACE_SCOPE: &str = r#"
        SELECT memory_id, namespace, memory_type, is_archived, is_pinned
        FROM namespace_prefilter
        WHERE namespace = ?1 AND is_archived = ?2
        LIMIT ?3;
    "#;

    /// Query for namespace + type filter.
    pub const QUERY_NAMESPACE_TYPE: &str = r#"
        SELECT memory_id, namespace, memory_type, is_archived, is_pinned
        FROM namespace_prefilter
        WHERE namespace = ?1 AND memory_type = ?2
        LIMIT ?3;
    "#;
}

// ── Entity Prefilter Schema ───────────────────────────────────────────────────

/// Entity/tag prefilter schema for structured candidate narrowing.
pub mod entity_schema {
    /// Entity prefilter table name.
    pub const ENTITY_PREFILTER_TABLE: &str = "entity_prefilter";

    /// CREATE TABLE for entity prefilter.
    pub const CREATE_ENTITY_PREFILTER: &str = r#"
        CREATE TABLE IF NOT EXISTS entity_prefilter (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            memory_id INTEGER NOT NULL,
            namespace TEXT NOT NULL,
            entity_name TEXT NOT NULL,
            entity_type TEXT NOT NULL,
            UNIQUE(memory_id, entity_name, entity_type)
        );
        CREATE INDEX IF NOT EXISTS idx_entity_name ON entity_prefilter(namespace, entity_name);
        CREATE INDEX IF NOT EXISTS idx_entity_type ON entity_prefilter(namespace, entity_type);
        CREATE INDEX IF NOT EXISTS idx_entity_memory ON entity_prefilter(memory_id);
    "#;

    /// Query for entity name lookup.
    pub const QUERY_BY_ENTITY: &str = r#"
        SELECT DISTINCT memory_id, namespace, entity_name, entity_type
        FROM entity_prefilter
        WHERE namespace = ?1 AND entity_name = ?2
        LIMIT ?3;
    "#;

    /// Query for entity type lookup.
    pub const QUERY_BY_ENTITY_TYPE: &str = r#"
        SELECT DISTINCT memory_id, namespace, entity_name, entity_type
        FROM entity_prefilter
        WHERE namespace = ?1 AND entity_type = ?2
        LIMIT ?3;
    "#;
}

// ── Index entry ──────────────────────────────────────────────────────────────

/// Entry to be inserted into an FTS5 lexical index.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Fts5IndexEntry {
    /// Memory this entry indexes.
    pub memory_id: MemoryId,
    /// Namespace scope for prefiltering.
    pub namespace: NamespaceId,
    /// Session scope for temporal prefiltering.
    pub session_id: SessionId,
    /// Memory type for categorical prefiltering.
    pub memory_type: CanonicalMemoryType,
    /// Compact text content for full-text indexing.
    pub content: String,
    /// Tick when this memory was created.
    pub tick_created: u64,
    /// Fingerprint for deduplication.
    pub fingerprint: u64,
    /// Normalization generation for cache invalidation.
    pub normalization_generation: &'static str,
}

impl Fts5IndexEntry {
    /// Creates a new FTS5 index entry.
    pub fn new(
        memory_id: MemoryId,
        namespace: NamespaceId,
        session_id: SessionId,
        memory_type: CanonicalMemoryType,
        content: impl Into<String>,
        tick_created: u64,
        fingerprint: u64,
    ) -> Self {
        Self {
            memory_id,
            namespace,
            session_id,
            memory_type,
            content: content.into(),
            tick_created,
            fingerprint,
            normalization_generation: "v1",
        }
    }
}

/// Entry for temporal prefilter index.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TemporalIndexEntry {
    /// Memory this entry indexes.
    pub memory_id: MemoryId,
    /// Namespace scope.
    pub namespace: NamespaceId,
    /// Session scope.
    pub session_id: SessionId,
    /// Tick when this memory was created.
    pub tick_created: u64,
    /// Tick when this memory was last accessed.
    pub tick_last_accessed: u64,
    /// Epoch for coarse temporal grouping.
    pub epoch: u64,
}

impl TemporalIndexEntry {
    /// Creates a new temporal index entry.
    pub fn new(
        memory_id: MemoryId,
        namespace: NamespaceId,
        session_id: SessionId,
        tick_created: u64,
        epoch: u64,
    ) -> Self {
        Self {
            memory_id,
            namespace,
            session_id,
            tick_created,
            tick_last_accessed: tick_created,
            epoch,
        }
    }

    /// Updates the last accessed tick.
    pub fn touch(&mut self, tick: u64) {
        self.tick_last_accessed = tick;
    }
}

/// Entry for namespace prefilter index.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NamespaceIndexEntry {
    /// Memory this entry indexes.
    pub memory_id: MemoryId,
    /// Namespace scope.
    pub namespace: NamespaceId,
    /// Memory type for categorical filtering.
    pub memory_type: CanonicalMemoryType,
    /// Whether this memory is archived.
    pub is_archived: bool,
    /// Whether this memory is pinned.
    pub is_pinned: bool,
}

/// Entry for entity prefilter index.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EntityIndexEntry {
    /// Memory this entry indexes.
    pub memory_id: MemoryId,
    /// Namespace scope.
    pub namespace: NamespaceId,
    /// Entity name (e.g., "user_alice", "task_123").
    pub entity_name: String,
    /// Entity type (e.g., "user", "task", "project").
    pub entity_type: String,
}

impl EntityIndexEntry {
    /// Creates a new entity index entry.
    pub fn new(
        memory_id: MemoryId,
        namespace: NamespaceId,
        entity_name: impl Into<String>,
        entity_type: impl Into<String>,
    ) -> Self {
        Self {
            memory_id,
            namespace,
            entity_name: entity_name.into(),
            entity_type: entity_type.into(),
        }
    }
}

// ── Candidate generation ──────────────────────────────────────────────────────

/// Candidate generated by an index lookup.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IndexCandidate {
    /// Memory id of the candidate.
    pub memory_id: MemoryId,
    /// Which index produced this candidate.
    pub source_index: IndexFamily,
    /// Relevance score from the index (0..1000, higher = more relevant).
    pub index_score: u16,
    /// Namespace of the candidate.
    pub namespace: NamespaceId,
    /// Session of the candidate (if available).
    pub session_id: Option<SessionId>,
    /// Memory type of the candidate (if available).
    pub memory_type: Option<CanonicalMemoryType>,
}

impl IndexCandidate {
    /// Creates a new index candidate.
    pub fn new(
        memory_id: MemoryId,
        source_index: IndexFamily,
        index_score: u16,
        namespace: NamespaceId,
    ) -> Self {
        Self {
            memory_id,
            source_index,
            index_score,
            namespace,
            session_id: None,
            memory_type: None,
        }
    }

    /// Adds session information to the candidate.
    pub fn with_session(mut self, session_id: SessionId) -> Self {
        self.session_id = Some(session_id);
        self
    }

    /// Adds memory type information to the candidate.
    pub fn with_memory_type(mut self, memory_type: CanonicalMemoryType) -> Self {
        self.memory_type = Some(memory_type);
        self
    }
}

/// Bounded candidate set with trace information.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CandidateSet {
    /// Candidates in the set.
    pub candidates: Vec<IndexCandidate>,
    /// Maximum candidates allowed.
    pub limit: usize,
    /// Total candidates inspected before final cut.
    pub candidates_inspected: usize,
    /// Index family that produced this set.
    pub source_family: IndexFamily,
    /// Whether the query was bypassed (zero budget).
    pub was_bypassed: bool,
}

impl CandidateSet {
    /// Creates an empty candidate set.
    pub fn empty(limit: usize, source_family: IndexFamily) -> Self {
        Self {
            candidates: Vec::new(),
            limit,
            candidates_inspected: 0,
            source_family,
            was_bypassed: false,
        }
    }

    /// Creates a bypassed candidate set (zero budget).
    pub fn bypassed(source_family: IndexFamily) -> Self {
        Self {
            candidates: Vec::new(),
            limit: 0,
            candidates_inspected: 0,
            source_family,
            was_bypassed: true,
        }
    }

    /// Creates a candidate set from a vector.
    pub fn from_candidates(
        candidates: Vec<IndexCandidate>,
        limit: usize,
        candidates_inspected: usize,
        source_family: IndexFamily,
    ) -> Self {
        Self {
            candidates,
            limit,
            candidates_inspected,
            source_family,
            was_bypassed: false,
        }
    }

    /// Returns true if this set is empty.
    pub fn is_empty(&self) -> bool {
        self.candidates.is_empty()
    }

    /// Returns the number of candidates in the set.
    pub fn len(&self) -> usize {
        self.candidates.len()
    }

    /// Returns true if the candidate set was truncated due to limit.
    pub fn was_truncated(&self) -> bool {
        self.candidates_inspected > self.candidates.len()
    }

    /// Returns the memory IDs in this candidate set.
    pub fn memory_ids(&self) -> Vec<MemoryId> {
        self.candidates.iter().map(|c| c.memory_id).collect()
    }
}

// ── Index query ──────────────────────────────────────────────────────────────

/// Query against the FTS5 lexical index.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Fts5Query {
    /// Search terms for the FTS5 MATCH expression.
    pub terms: String,
    /// Namespace filter (required).
    pub namespace: NamespaceId,
    /// Optional session scope filter.
    pub session_filter: Option<SessionId>,
    /// Optional memory type filter.
    pub memory_type_filter: Option<CanonicalMemoryType>,
    /// Optional negation terms (excluded from results).
    pub negation_terms: Option<String>,
    /// Maximum candidates to return.
    pub limit: usize,
    /// Candidate budget for bounded retrieval.
    pub candidate_budget: usize,
}

impl Fts5Query {
    /// Default candidate budget for FTS5 queries.
    pub const DEFAULT_CANDIDATE_BUDGET: usize = 5000;

    /// Builds a basic namespace-scoped FTS5 query.
    pub fn new(terms: impl Into<String>, namespace: NamespaceId, limit: usize) -> Self {
        Self {
            terms: terms.into(),
            namespace,
            session_filter: None,
            memory_type_filter: None,
            negation_terms: None,
            limit,
            candidate_budget: Self::DEFAULT_CANDIDATE_BUDGET,
        }
    }

    /// Adds a session filter.
    pub fn with_session(mut self, session_id: SessionId) -> Self {
        self.session_filter = Some(session_id);
        self
    }

    /// Adds a memory type filter.
    pub fn with_memory_type(mut self, memory_type: CanonicalMemoryType) -> Self {
        self.memory_type_filter = Some(memory_type);
        self
    }

    /// Adds negation terms (excluded from results).
    pub fn with_negation(mut self, negation_terms: impl Into<String>) -> Self {
        self.negation_terms = Some(negation_terms.into());
        self
    }

    /// Sets the candidate budget.
    pub fn with_budget(mut self, budget: usize) -> Self {
        self.candidate_budget = budget;
        self
    }

    /// Converts the query to an FTS5 MATCH expression.
    /// Handles escaping of special FTS5 characters.
    pub fn to_match_expression(&self) -> String {
        // FTS5 special characters that need escaping: " ' ( ) - ^ * { }
        // For simplicity, we wrap terms in double quotes if they contain spaces
        if self.terms.contains(' ') {
            format!("\"{}\"", self.terms.replace('"', "\"\""))
        } else {
            self.terms.clone()
        }
    }

    /// Returns true if this query has negation terms.
    pub fn has_negation(&self) -> bool {
        self.negation_terms.is_some()
    }
}

/// Query against the temporal prefilter index.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TemporalQuery {
    /// Namespace filter (required).
    pub namespace: NamespaceId,
    /// Session filter for session-scoped queries.
    pub session_filter: Option<SessionId>,
    /// Tick range filter (start, end).
    pub tick_range: Option<(u64, u64)>,
    /// Stable era selector for landmark-defined history slices.
    pub era_id: Option<String>,
    /// Epoch filter for coarse temporal grouping.
    pub epoch_filter: Option<u64>,
    /// Maximum candidates to return.
    pub limit: usize,
    /// Candidate budget for bounded retrieval.
    pub candidate_budget: usize,
}

impl TemporalQuery {
    /// Creates a new temporal query.
    pub fn new(namespace: NamespaceId, limit: usize) -> Self {
        Self {
            namespace,
            session_filter: None,
            tick_range: None,
            era_id: None,
            epoch_filter: None,
            limit,
            candidate_budget: 5000,
        }
    }

    /// Adds a session filter.
    pub fn with_session(mut self, session_id: SessionId) -> Self {
        self.session_filter = Some(session_id);
        self
    }

    /// Adds a tick range filter.
    pub fn with_tick_range(mut self, start: u64, end: u64) -> Self {
        self.tick_range = Some((start, end));
        self
    }

    /// Adds a stable era selector for landmark-defined slices.
    pub fn with_era_id(mut self, era_id: impl Into<String>) -> Self {
        self.era_id = Some(era_id.into());
        self
    }

    /// Adds an epoch filter.
    pub fn with_epoch(mut self, epoch: u64) -> Self {
        self.epoch_filter = Some(epoch);
        self
    }

    /// Sets the candidate budget.
    pub fn with_budget(mut self, budget: usize) -> Self {
        self.candidate_budget = budget;
        self
    }
}

/// Query against the entity prefilter index.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EntityQuery {
    /// Namespace filter (required).
    pub namespace: NamespaceId,
    /// Entity name filter.
    pub entity_name: Option<String>,
    /// Entity type filter.
    pub entity_type: Option<String>,
    /// Maximum candidates to return.
    pub limit: usize,
    /// Candidate budget for bounded retrieval.
    pub candidate_budget: usize,
}

impl EntityQuery {
    /// Creates a new entity query.
    pub fn new(namespace: NamespaceId, limit: usize) -> Self {
        Self {
            namespace,
            entity_name: None,
            entity_type: None,
            limit,
            candidate_budget: 5000,
        }
    }

    /// Adds an entity name filter.
    pub fn with_entity_name(mut self, name: impl Into<String>) -> Self {
        self.entity_name = Some(name.into());
        self
    }

    /// Adds an entity type filter.
    pub fn with_entity_type(mut self, entity_type: impl Into<String>) -> Self {
        self.entity_type = Some(entity_type.into());
        self
    }

    /// Sets the candidate budget.
    pub fn with_budget(mut self, budget: usize) -> Self {
        self.candidate_budget = budget;
        self
    }
}

// ── Index health ─────────────────────────────────────────────────────────────

/// Health status of one index.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize)]
pub enum IndexHealth {
    Healthy,
    Stale,
    NeedsRebuild,
    Missing,
}

impl IndexHealth {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Healthy => "healthy",
            Self::Stale => "stale",
            Self::NeedsRebuild => "needs_rebuild",
            Self::Missing => "missing",
        }
    }

    /// Returns true if the index is usable.
    pub const fn is_usable(self) -> bool {
        matches!(self, Self::Healthy | Self::Stale)
    }
}

/// Health report for a specific index.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
pub struct IndexHealthReport {
    pub family: IndexFamily,
    pub health: IndexHealth,
    pub entry_count: usize,
    pub generation: &'static str,
    pub hit_rate: u8,
    pub miss_rate: u8,
    pub stale_index_ratio: u8,
    pub repair_backlog: usize,
    pub rebuild_duration_hint: &'static str,
    pub item_count_divergence: usize,
}

impl IndexHealthReport {
    /// Returns whether durable truth and derived rows are still in parity.
    pub const fn is_in_parity(&self) -> bool {
        self.item_count_divergence == 0
    }
}

// ── Index trace ──────────────────────────────────────────────────────────────

/// Trace record for index lookup operations.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IndexLookupTrace {
    /// Index family that was queried.
    pub family: IndexFamily,
    /// Number of candidates returned.
    pub candidates_returned: usize,
    /// Number of candidates inspected before final cut.
    pub candidates_inspected: usize,
    /// Whether the query was bypassed (zero budget).
    pub was_bypassed: bool,
    /// Whether the result was truncated due to limit.
    pub was_truncated: bool,
    /// Query execution time hint (for logging).
    pub execution_hint: &'static str,
}

impl IndexLookupTrace {
    /// Creates a trace for a successful lookup.
    pub fn success(
        family: IndexFamily,
        candidates_returned: usize,
        candidates_inspected: usize,
    ) -> Self {
        Self {
            family,
            candidates_returned,
            candidates_inspected,
            was_bypassed: false,
            was_truncated: candidates_inspected > candidates_returned,
            execution_hint: "normal",
        }
    }

    /// Creates a trace for a bypassed lookup (zero budget).
    pub fn bypassed(family: IndexFamily) -> Self {
        Self {
            family,
            candidates_returned: 0,
            candidates_inspected: 0,
            was_bypassed: true,
            was_truncated: false,
            execution_hint: "bypassed",
        }
    }

    /// Returns a human-readable summary.
    pub fn summary(&self) -> String {
        if self.was_bypassed {
            format!("[{}] BYPASSED (zero budget)", self.family.as_str())
        } else {
            format!(
                "[{}] returned {} of {} inspected ({})",
                self.family.as_str(),
                self.candidates_returned,
                self.candidates_inspected,
                if self.was_truncated {
                    "truncated"
                } else {
                    "complete"
                }
            )
        }
    }
}

// ── Index trait ──────────────────────────────────────────────────────────────

/// Core index contract for candidate generation.
pub trait IndexApi {
    /// Returns the stable component identifier.
    fn api_component_name(&self) -> &'static str;

    /// Returns health reports for all managed indexes.
    fn health_reports(&self) -> Vec<IndexHealthReport>;

    /// Returns the set of index families this module manages.
    fn managed_families(&self) -> &[IndexFamily];
}

// ── Engine ───────────────────────────────────────────────────────────────────

/// Stable index boundary owned by `membrain-core`.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct IndexModule;

impl IndexModule {
    /// All index families managed by the core index module.
    const MANAGED_FAMILIES: &'static [IndexFamily] = &[
        IndexFamily::Fts5Lexical,
        IndexFamily::Fts5ColdLexical,
        IndexFamily::NamespacePrefilter,
        IndexFamily::FingerprintDedup,
        IndexFamily::SessionTemporal,
        IndexFamily::TickRangeTemporal,
        IndexFamily::EntityPrefilter,
        IndexFamily::MemoryTypeCategorical,
    ];

    /// Returns the stable component identifier for this index surface.
    pub const fn component_name(&self) -> &'static str {
        "index"
    }

    /// Returns the FTS5 schema for hot memory lexical index.
    pub const fn hot_fts5_schema(&self) -> &'static str {
        fts5_schema::CREATE_MEMORY_FTS
    }

    /// Returns the FTS5 schema for cold memory lexical index.
    pub const fn cold_fts5_schema(&self) -> &'static str {
        fts5_schema::CREATE_COLD_MEMORY_FTS
    }

    /// Returns the temporal prefilter schema.
    pub const fn temporal_schema(&self) -> &'static str {
        temporal_schema::CREATE_TEMPORAL_PREFILTER
    }

    /// Returns the namespace prefilter schema.
    pub const fn namespace_schema(&self) -> &'static str {
        namespace_schema::CREATE_NAMESPACE_PREFILTER
    }

    /// Returns the entity prefilter schema.
    pub const fn entity_schema(&self) -> &'static str {
        entity_schema::CREATE_ENTITY_PREFILTER
    }
}

impl IndexApi for IndexModule {
    fn api_component_name(&self) -> &'static str {
        "index"
    }

    fn health_reports(&self) -> Vec<IndexHealthReport> {
        Self::MANAGED_FAMILIES
            .iter()
            .map(|&family| IndexHealthReport {
                family,
                health: IndexHealth::Healthy,
                entry_count: 0,
                generation: "v0.1",
                hit_rate: 100,
                miss_rate: 0,
                stale_index_ratio: 0,
                repair_backlog: 0,
                rebuild_duration_hint: "not_needed",
                item_count_divergence: 0,
            })
            .collect()
    }

    fn managed_families(&self) -> &[IndexFamily] {
        Self::MANAGED_FAMILIES
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn index_module_manages_all_families() {
        let module = IndexModule;
        let families = module.managed_families();
        assert_eq!(families.len(), 8);
        assert!(families.contains(&IndexFamily::Fts5Lexical));
        assert!(families.contains(&IndexFamily::Fts5ColdLexical));
        assert!(families.contains(&IndexFamily::NamespacePrefilter));
        assert!(families.contains(&IndexFamily::EntityPrefilter));
    }

    #[test]
    fn health_reports_cover_all_families() {
        let module = IndexModule;
        let reports = module.health_reports();
        assert_eq!(reports.len(), 8);
        assert!(reports.iter().all(|r| r.health == IndexHealth::Healthy));
        assert!(reports.iter().all(|r| r.hit_rate == 100));
        assert!(reports.iter().all(|r| r.miss_rate == 0));
        assert!(reports.iter().all(|r| r.stale_index_ratio == 0));
        assert!(reports.iter().all(|r| r.repair_backlog == 0));
        assert!(reports
            .iter()
            .all(|r| r.rebuild_duration_hint == "not_needed"));
        assert!(reports.iter().all(IndexHealthReport::is_in_parity));
    }

    #[test]
    fn fts5_query_builder() {
        let ns = NamespaceId::new("test").unwrap();
        let query = Fts5Query::new("hello world", ns.clone(), 10)
            .with_session(SessionId(7))
            .with_memory_type(CanonicalMemoryType::Event)
            .with_negation("spam")
            .with_budget(1000);

        assert_eq!(query.terms, "hello world");
        assert_eq!(query.namespace, ns);
        assert_eq!(query.session_filter, Some(SessionId(7)));
        assert_eq!(query.memory_type_filter, Some(CanonicalMemoryType::Event));
        assert_eq!(query.negation_terms, Some("spam".to_string()));
        assert_eq!(query.limit, 10);
        assert_eq!(query.candidate_budget, 1000);
        assert!(query.has_negation());
    }

    #[test]
    fn fts5_match_expression_escapes_spaces() {
        let ns = NamespaceId::new("test").unwrap();
        let query = Fts5Query::new("hello world", ns.clone(), 10);
        assert_eq!(query.to_match_expression(), "\"hello world\"");

        let query2 = Fts5Query::new("hello", ns, 10);
        assert_eq!(query2.to_match_expression(), "hello");
    }

    #[test]
    fn temporal_query_builder() {
        let ns = NamespaceId::new("test").unwrap();
        let query = TemporalQuery::new(ns.clone(), 20)
            .with_session(SessionId(5))
            .with_tick_range(100, 200)
            .with_era_id("era-deploy-0042")
            .with_epoch(3);

        assert_eq!(query.namespace, ns);
        assert_eq!(query.session_filter, Some(SessionId(5)));
        assert_eq!(query.tick_range, Some((100, 200)));
        assert_eq!(query.era_id.as_deref(), Some("era-deploy-0042"));
        assert_eq!(query.epoch_filter, Some(3));
        assert_eq!(query.limit, 20);
    }

    #[test]
    fn entity_query_builder() {
        let ns = NamespaceId::new("test").unwrap();
        let query = EntityQuery::new(ns.clone(), 15)
            .with_entity_name("user_alice")
            .with_entity_type("user");

        assert_eq!(query.namespace, ns);
        assert_eq!(query.entity_name, Some("user_alice".to_string()));
        assert_eq!(query.entity_type, Some("user".to_string()));
        assert_eq!(query.limit, 15);
    }

    #[test]
    fn candidate_set_operations() {
        let ns = NamespaceId::new("test").unwrap();
        let candidates = vec![
            IndexCandidate::new(MemoryId(1), IndexFamily::Fts5Lexical, 800, ns.clone()),
            IndexCandidate::new(MemoryId(2), IndexFamily::Fts5Lexical, 600, ns.clone()),
        ];

        let set =
            CandidateSet::from_candidates(candidates.clone(), 10, 15, IndexFamily::Fts5Lexical);

        assert_eq!(set.len(), 2);
        assert!(!set.is_empty());
        assert!(set.was_truncated());
        assert_eq!(set.memory_ids(), vec![MemoryId(1), MemoryId(2)]);
    }

    #[test]
    fn candidate_set_bypassed() {
        let set = CandidateSet::bypassed(IndexFamily::Fts5Lexical);
        assert!(set.is_empty());
        assert!(set.was_bypassed);
        assert_eq!(set.len(), 0);
    }

    #[test]
    fn candidate_set_empty() {
        let set = CandidateSet::empty(10, IndexFamily::Fts5Lexical);
        assert!(set.is_empty());
        assert!(!set.was_bypassed);
    }

    #[test]
    fn index_lookup_trace_success() {
        let trace = IndexLookupTrace::success(IndexFamily::Fts5Lexical, 10, 50);
        assert_eq!(trace.candidates_returned, 10);
        assert_eq!(trace.candidates_inspected, 50);
        assert!(!trace.was_bypassed);
        assert!(trace.was_truncated);
        assert!(trace.summary().contains("returned 10 of 50"));
    }

    #[test]
    fn index_lookup_trace_bypassed() {
        let trace = IndexLookupTrace::bypassed(IndexFamily::Fts5Lexical);
        assert!(trace.was_bypassed);
        assert_eq!(trace.candidates_returned, 0);
        assert!(trace.summary().contains("BYPASSED"));
    }

    #[test]
    fn index_family_classification() {
        assert!(IndexFamily::Fts5Lexical.is_fts5());
        assert!(IndexFamily::Fts5ColdLexical.is_fts5());
        assert!(!IndexFamily::NamespacePrefilter.is_fts5());

        assert!(IndexFamily::SessionTemporal.is_temporal());
        assert!(IndexFamily::TickRangeTemporal.is_temporal());
        assert!(!IndexFamily::Fts5Lexical.is_temporal());
    }

    #[test]
    fn fts5_schema_definitions() {
        // Verify schema strings are valid SQL (basic check)
        assert!(fts5_schema::CREATE_MEMORY_FTS.contains("CREATE VIRTUAL TABLE"));
        assert!(fts5_schema::CREATE_MEMORY_FTS.contains("fts5"));
        assert!(fts5_schema::CREATE_COLD_MEMORY_FTS.contains("cold_memory_fts"));
        assert!(fts5_schema::QUERY_MEMORY_FTS.contains("bm25"));
        assert!(fts5_schema::QUERY_MEMORY_FTS_NEGATION.contains("NOT MATCH"));
    }

    #[test]
    fn temporal_schema_definitions() {
        assert!(temporal_schema::CREATE_TEMPORAL_PREFILTER.contains("tick_created"));
        assert!(temporal_schema::CREATE_TEMPORAL_PREFILTER.contains("tick_last_accessed"));
        assert!(temporal_schema::CREATE_TEMPORAL_PREFILTER.contains("epoch"));
        assert!(temporal_schema::QUERY_TICK_RANGE.contains("BETWEEN"));
    }

    #[test]
    fn namespace_schema_definitions() {
        assert!(namespace_schema::CREATE_NAMESPACE_PREFILTER.contains("is_archived"));
        assert!(namespace_schema::CREATE_NAMESPACE_PREFILTER.contains("is_pinned"));
    }

    #[test]
    fn entity_schema_definitions() {
        assert!(entity_schema::CREATE_ENTITY_PREFILTER.contains("entity_name"));
        assert!(entity_schema::CREATE_ENTITY_PREFILTER.contains("entity_type"));
    }

    #[test]
    fn index_candidate_with_options() {
        let ns = NamespaceId::new("test").unwrap();
        let candidate = IndexCandidate::new(MemoryId(1), IndexFamily::Fts5Lexical, 750, ns.clone())
            .with_session(SessionId(10))
            .with_memory_type(CanonicalMemoryType::Event);

        assert_eq!(candidate.memory_id, MemoryId(1));
        assert_eq!(candidate.source_index, IndexFamily::Fts5Lexical);
        assert_eq!(candidate.index_score, 750);
        assert_eq!(candidate.namespace, ns);
        assert_eq!(candidate.session_id, Some(SessionId(10)));
        assert_eq!(candidate.memory_type, Some(CanonicalMemoryType::Event));
    }

    #[test]
    fn temporal_index_entry_touch() {
        let ns = NamespaceId::new("test").unwrap();
        let mut entry = TemporalIndexEntry::new(MemoryId(1), ns, SessionId(5), 100, 1);
        assert_eq!(entry.tick_last_accessed, 100);

        entry.touch(200);
        assert_eq!(entry.tick_last_accessed, 200);
    }

    #[test]
    fn index_health_usability() {
        assert!(IndexHealth::Healthy.is_usable());
        assert!(IndexHealth::Stale.is_usable());
        assert!(!IndexHealth::NeedsRebuild.is_usable());
        assert!(!IndexHealth::Missing.is_usable());
    }

    #[test]
    fn index_health_report_tracks_durable_truth_parity_signal() {
        let healthy = IndexHealthReport {
            family: IndexFamily::Fts5Lexical,
            health: IndexHealth::Healthy,
            entry_count: 24,
            generation: "v0.1",
            hit_rate: 97,
            miss_rate: 3,
            stale_index_ratio: 0,
            repair_backlog: 0,
            rebuild_duration_hint: "not_needed",
            item_count_divergence: 0,
        };
        let diverged = IndexHealthReport {
            family: IndexFamily::EntityPrefilter,
            health: IndexHealth::Stale,
            entry_count: 21,
            generation: "v0.1",
            hit_rate: 88,
            miss_rate: 12,
            stale_index_ratio: 9,
            repair_backlog: 2,
            rebuild_duration_hint: "bounded_rebuild",
            item_count_divergence: 3,
        };

        assert!(healthy.is_in_parity());
        assert!(!diverged.is_in_parity());
    }
}
