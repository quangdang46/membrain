//! Bounded Tier2/Tier3 query planners with explicit escalation logic.
//!
//! This module implements the canonical retrieval escalation chain:
//! 1. SQLite/FTS5 prefilter → bounded candidate ids
//! 2. USearch HNSW hot search → bounded top-K semantic hits
//! 3. Float32 rescore of bounded hit set
//! 4. Optional Tier3 USearch mmap cold search (fallback only)
//! 5. Optional graph/engram expansion
//! 6. Final packaging with trace artifacts
//!
//! Key invariants:
//! - No planner path performs a full-store scan
//! - No cold-payload fetch before the final candidate cut
//! - Tier3 is a declared fallback, not a silent parallel sweep
//! - Every hit is eligible to refresh Tier1 mirrors without changing durable ownership

use crate::api::NamespaceId;
use crate::index::{CandidateSet, EntityQuery, Fts5Query, TemporalQuery};
use crate::types::{CanonicalMemoryType, MemoryId, SessionId};

// ── Planner input ─────────────────────────────────────────────────────────────

/// Query path selection for retrieval planning.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub enum QueryPath {
    /// Direct ID lookup (exact handle).
    ExactId,
    /// Temporal/slice-based retrieval.
    Temporal,
    /// Entity-heavy structured retrieval.
    EntityHeavy,
    /// Hybrid lexical + semantic retrieval.
    Hybrid,
    /// Partial cue fallback (low confidence).
    PartialCue,
    /// Semantic-only vector search.
    SemanticOnly,
}

impl QueryPath {
    /// Returns the stable machine-readable name.
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::ExactId => "exact_id",
            Self::Temporal => "temporal",
            Self::EntityHeavy => "entity_heavy",
            Self::Hybrid => "hybrid",
            Self::PartialCue => "partial_cue",
            Self::SemanticOnly => "semantic_only",
        }
    }

    /// Returns true if this path requires lexical prefiltering.
    pub const fn requires_lexical_prefilter(self) -> bool {
        matches!(self, Self::Hybrid | Self::PartialCue)
    }

    /// Returns true if this path requires semantic search.
    pub const fn requires_semantic_search(self) -> bool {
        matches!(self, Self::Hybrid | Self::SemanticOnly | Self::PartialCue)
    }
}

/// Query-by-example polarity for a seed memory.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum QueryExamplePolarity {
    /// A memory to retrieve results similar to.
    Like,
    /// A memory to retrieve results dissimilar to.
    Unlike,
}

impl QueryExamplePolarity {
    /// Returns the stable machine-readable name.
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Like => "like",
            Self::Unlike => "unlike",
        }
    }
}

/// Stable query-by-example seed descriptor.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct QueryExampleSeed {
    /// Stored evidence selected as a retrieval seed.
    pub memory_id: MemoryId,
    /// Whether the seed is a positive or negative cue.
    pub polarity: QueryExamplePolarity,
}

impl QueryExampleSeed {
    /// Returns the stable machine-readable polarity for this seed.
    pub const fn polarity_name(&self) -> &'static str {
        self.polarity.as_str()
    }
}

/// Declared primary cue after request normalization.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PrimaryCue {
    /// Normalized query text remains the main cue.
    QueryText,
    /// A like-by-example reference is the main cue.
    LikeId,
    /// An unlike-by-example reference is the main cue.
    UnlikeId,
}

impl PrimaryCue {
    /// Returns the stable machine-readable name.
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::QueryText => "query_text",
            Self::LikeId => "like_id",
            Self::UnlikeId => "unlike_id",
        }
    }
}

/// Normalized query-by-example request surface.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct QueryByExampleNormalization {
    /// Trimmed query text when present.
    pub normalized_query_text: Option<String>,
    /// The primary cue chosen for this request.
    pub primary_cue: PrimaryCue,
    /// Ordered seed memories referenced by the request.
    pub seeds: Vec<QueryExampleSeed>,
}

impl QueryByExampleNormalization {
    /// Returns true when query text remains the canonical primary cue.
    pub const fn uses_query_text_as_primary_cue(&self) -> bool {
        matches!(self.primary_cue, PrimaryCue::QueryText)
    }

    /// Returns true when any stored memory seed participates in the request.
    pub fn has_example_seeds(&self) -> bool {
        !self.seeds.is_empty()
    }

    /// Returns ordered machine-readable seed descriptors for explain and smoke surfaces.
    pub fn seed_descriptors(&self) -> Vec<String> {
        self.seeds
            .iter()
            .map(|seed| format!("{}:{}", seed.polarity_name(), seed.memory_id.0))
            .collect()
    }

    /// Returns ordered machine-readable seed polarity names for explain and smoke surfaces.
    pub fn seed_polarities(&self) -> Vec<&'static str> {
        self.seeds
            .iter()
            .map(QueryExampleSeed::polarity_name)
            .collect()
    }

    /// Returns ordered seed memory ids exactly as normalized from the request.
    pub fn seed_memory_ids(&self) -> Vec<MemoryId> {
        self.seeds.iter().map(|seed| seed.memory_id).collect()
    }
}

/// Validation failures for canonical retrieval cues.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum RetrievalRequestValidationError {
    /// Request omitted both query text and query-by-example references.
    MissingPrimaryCue,
    /// Request reused the same memory as both like and unlike cue.
    DuplicateExampleCue(MemoryId),
}

impl RetrievalRequestValidationError {
    /// Returns the stable machine-readable name.
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::MissingPrimaryCue => "missing_primary_cue",
            Self::DuplicateExampleCue(_) => "duplicate_example_cue",
        }
    }
}

/// Retrieval planner request.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RetrievalRequest {
    /// Namespace scope (required).
    pub namespace: NamespaceId,
    /// Query text for lexical matching.
    pub query_text: Option<String>,
    /// Query-by-example positive cue.
    pub like_memory_id: Option<MemoryId>,
    /// Query-by-example negative cue.
    pub unlike_memory_id: Option<MemoryId>,
    /// Query path selection hint.
    pub query_path: QueryPath,
    /// Exact memory ID for direct lookup.
    pub exact_memory_id: Option<MemoryId>,
    /// Session scope for temporal filtering.
    pub session_filter: Option<SessionId>,
    /// Memory type filter.
    pub memory_type_filter: Option<CanonicalMemoryType>,
    /// Optional entity-name filter for entity-heavy retrieval.
    pub entity_name_filter: Option<String>,
    /// Optional entity-type filter for entity-heavy retrieval.
    pub entity_type_filter: Option<String>,
    /// Maximum final results to return.
    pub limit: usize,
    /// Candidate budget for bounded retrieval.
    pub candidate_budget: usize,
    /// Whether to enable Tier3 fallback.
    pub enable_tier3_fallback: bool,
    /// Whether to enable graph/engram expansion.
    pub enable_graph_expansion: bool,
}

impl RetrievalRequest {
    /// Creates a new hybrid retrieval request.
    pub fn hybrid(namespace: NamespaceId, query_text: impl Into<String>, limit: usize) -> Self {
        Self {
            namespace,
            query_text: Some(query_text.into()),
            like_memory_id: None,
            unlike_memory_id: None,
            query_path: QueryPath::Hybrid,
            exact_memory_id: None,
            session_filter: None,
            memory_type_filter: None,
            entity_name_filter: None,
            entity_type_filter: None,
            limit,
            candidate_budget: Self::DEFAULT_CANDIDATE_BUDGET,
            enable_tier3_fallback: true,
            enable_graph_expansion: false,
        }
    }

    /// Creates an exact ID lookup request.
    pub fn exact_id(namespace: NamespaceId, memory_id: MemoryId) -> Self {
        Self {
            namespace,
            query_text: None,
            like_memory_id: None,
            unlike_memory_id: None,
            query_path: QueryPath::ExactId,
            exact_memory_id: Some(memory_id),
            session_filter: None,
            memory_type_filter: None,
            entity_name_filter: None,
            entity_type_filter: None,
            limit: 1,
            candidate_budget: 1,
            enable_tier3_fallback: false,
            enable_graph_expansion: false,
        }
    }

    /// Creates a query-by-example request using a positive memory cue.
    pub fn query_by_example(
        namespace: NamespaceId,
        like_memory_id: MemoryId,
        limit: usize,
    ) -> Self {
        Self {
            namespace,
            query_text: None,
            like_memory_id: Some(like_memory_id),
            unlike_memory_id: None,
            query_path: QueryPath::Hybrid,
            exact_memory_id: None,
            session_filter: None,
            memory_type_filter: None,
            entity_name_filter: None,
            entity_type_filter: None,
            limit,
            candidate_budget: Self::DEFAULT_CANDIDATE_BUDGET,
            enable_tier3_fallback: true,
            enable_graph_expansion: false,
        }
    }

    /// Default candidate budget for retrieval requests.
    pub const DEFAULT_CANDIDATE_BUDGET: usize = 5000;

    /// Adds a positive query-by-example cue.
    pub fn with_like_memory(mut self, memory_id: MemoryId) -> Self {
        self.like_memory_id = Some(memory_id);
        self
    }

    /// Adds a negative query-by-example cue.
    pub fn with_unlike_memory(mut self, memory_id: MemoryId) -> Self {
        self.unlike_memory_id = Some(memory_id);
        self
    }

    /// Returns the normalized query-by-example cue contract for this request.
    pub fn normalize_query_by_example(
        &self,
    ) -> Result<QueryByExampleNormalization, RetrievalRequestValidationError> {
        let normalized_query_text = self
            .query_text
            .as_ref()
            .map(|text| text.trim())
            .filter(|text| !text.is_empty())
            .map(str::to_owned);

        if let (Some(like_id), Some(unlike_id)) = (self.like_memory_id, self.unlike_memory_id) {
            if like_id == unlike_id {
                return Err(RetrievalRequestValidationError::DuplicateExampleCue(
                    like_id,
                ));
            }
        }

        let mut seeds = Vec::new();
        if let Some(memory_id) = self.like_memory_id {
            seeds.push(QueryExampleSeed {
                memory_id,
                polarity: QueryExamplePolarity::Like,
            });
        }
        if let Some(memory_id) = self.unlike_memory_id {
            seeds.push(QueryExampleSeed {
                memory_id,
                polarity: QueryExamplePolarity::Unlike,
            });
        }

        let primary_cue = if normalized_query_text.is_some() {
            PrimaryCue::QueryText
        } else if self.like_memory_id.is_some() {
            PrimaryCue::LikeId
        } else if self.unlike_memory_id.is_some() {
            PrimaryCue::UnlikeId
        } else {
            return Err(RetrievalRequestValidationError::MissingPrimaryCue);
        };

        Ok(QueryByExampleNormalization {
            normalized_query_text,
            primary_cue,
            seeds,
        })
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

    /// Adds an entity-name filter for entity-heavy retrieval.
    pub fn with_entity_name(mut self, name: impl Into<String>) -> Self {
        self.entity_name_filter = Some(name.into());
        self
    }

    /// Adds an entity-type filter for entity-heavy retrieval.
    pub fn with_entity_type(mut self, entity_type: impl Into<String>) -> Self {
        self.entity_type_filter = Some(entity_type.into());
        self
    }

    /// Sets the candidate budget.
    pub fn with_budget(mut self, budget: usize) -> Self {
        self.candidate_budget = budget;
        self
    }

    /// Enables Tier3 fallback.
    pub fn with_tier3_fallback(mut self, enabled: bool) -> Self {
        self.enable_tier3_fallback = enabled;
        self
    }

    /// Enables graph/engram expansion.
    pub fn with_graph_expansion(mut self, enabled: bool) -> Self {
        self.enable_graph_expansion = enabled;
        self
    }

    /// Returns whether this request should schedule lexical prefiltering.
    pub fn requires_lexical_prefilter(&self) -> bool {
        self.query_path.requires_lexical_prefilter()
            && self
                .query_text
                .as_ref()
                .map(|text| !text.trim().is_empty())
                .unwrap_or(false)
    }

    /// Returns whether this request should schedule semantic search.
    pub fn requires_semantic_search(&self) -> bool {
        self.query_path.requires_semantic_search()
            || self.like_memory_id.is_some()
            || self.unlike_memory_id.is_some()
    }

    /// Builds the bounded temporal prefilter query when this request uses the temporal lane.
    pub fn temporal_prefilter_query(&self) -> Option<TemporalQuery> {
        (self.query_path == QueryPath::Temporal).then(|| {
            let mut query = TemporalQuery::new(self.namespace.clone(), self.limit)
                .with_budget(self.candidate_budget);
            if let Some(session_id) = self.session_filter {
                query = query.with_session(session_id);
            }
            query
        })
    }

    /// Builds the bounded entity prefilter query when this request uses the entity-heavy lane.
    pub fn entity_prefilter_query(&self) -> Option<EntityQuery> {
        (self.query_path == QueryPath::EntityHeavy).then(|| {
            let mut query = EntityQuery::new(self.namespace.clone(), self.limit)
                .with_budget(self.candidate_budget);
            if let Some(entity_name) = &self.entity_name_filter {
                query = query.with_entity_name(entity_name.clone());
            }
            if let Some(entity_type) = &self.entity_type_filter {
                query = query.with_entity_type(entity_type.clone());
            }
            query
        })
    }

    /// Builds the bounded lexical prefilter query when this request schedules lexical prefiltering.
    pub fn lexical_prefilter_query(&self) -> Option<Fts5Query> {
        self.query_path
            .requires_lexical_prefilter()
            .then_some(())
            .and(
                self.query_text
                    .as_ref()
                    .filter(|text| !text.trim().is_empty()),
            )
            .map(|text| {
                let mut query = Fts5Query::new(text.trim(), self.namespace.clone(), self.limit)
                    .with_budget(self.candidate_budget);
                if let Some(session_id) = self.session_filter {
                    query = query.with_session(session_id);
                }
                if let Some(memory_type) = self.memory_type_filter {
                    query = query.with_memory_type(memory_type);
                }
                query
            })
    }
}

// ── Planner stages ────────────────────────────────────────────────────────────

/// Retrieval planner stage identifiers for trace artifacts.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PlannerStage {
    /// FTS5 lexical prefilter stage.
    LexicalPrefilter,
    /// USearch hot HNSW search stage.
    HotSemanticSearch,
    /// Float32 rescore of quantized hits.
    Float32Rescore,
    /// Tier3 cold mmap search (fallback).
    Tier3ColdSearch,
    /// Graph/engram expansion stage.
    GraphExpansion,
    /// Final packaging stage.
    FinalPackaging,
}

impl PlannerStage {
    /// Returns the stable machine-readable name.
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::LexicalPrefilter => "lexical_prefilter",
            Self::HotSemanticSearch => "hot_semantic_search",
            Self::Float32Rescore => "float32_rescore",
            Self::Tier3ColdSearch => "tier3_cold_search",
            Self::GraphExpansion => "graph_expansion",
            Self::FinalPackaging => "final_packaging",
        }
    }

    /// Returns whether this stage participates in the bounded rerank slice.
    pub const fn is_rerank_stage(self) -> bool {
        matches!(self, Self::Float32Rescore)
    }

    /// Returns true if this is a Tier2 stage.
    pub const fn is_tier2(self) -> bool {
        matches!(
            self,
            Self::LexicalPrefilter | Self::HotSemanticSearch | Self::Float32Rescore
        )
    }

    /// Returns true if this is a Tier3 stage.
    pub const fn is_tier3(self) -> bool {
        matches!(self, Self::Tier3ColdSearch)
    }
}

// ── Escalation logic ──────────────────────────────────────────────────────────

/// Reasons for escalating from Tier2 to Tier3.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum EscalationReason {
    /// Tier2 returned zero results.
    Tier2ZeroResults,
    /// Tier2 returned insufficient results (underfill).
    Tier2Underfill,
    /// Low confidence in Tier2 results.
    LowConfidence,
    /// Explicit user request for cold search.
    ExplicitRequest,
    /// No applicable reason (Tier3 not triggered).
    NotTriggered,
}

impl EscalationReason {
    /// Returns the stable machine-readable name.
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Tier2ZeroResults => "tier2_zero_results",
            Self::Tier2Underfill => "tier2_underfill",
            Self::LowConfidence => "low_confidence",
            Self::ExplicitRequest => "explicit_request",
            Self::NotTriggered => "not_triggered",
        }
    }

    /// Returns true if escalation is needed.
    pub const fn needs_escalation(self) -> bool {
        !matches!(self, Self::NotTriggered)
    }
}

/// Configuration for Tier2/Tier3 escalation decisions.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EscalationConfig {
    /// Minimum results from Tier2 before considering it sufficient.
    pub tier2_min_results: usize,
    /// Confidence threshold below which Tier3 is considered.
    pub confidence_threshold: u16,
    /// Maximum Tier3 candidates to fetch.
    pub tier3_candidate_limit: usize,
    /// Whether Tier3 is enabled by default.
    pub tier3_enabled_by_default: bool,
}

impl Default for EscalationConfig {
    fn default() -> Self {
        Self {
            tier2_min_results: 5,
            confidence_threshold: 500, // 0-1000 scale
            tier3_candidate_limit: 100,
            tier3_enabled_by_default: true,
        }
    }
}

/// Decision result for Tier3 escalation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EscalationDecision {
    /// Whether to escalate to Tier3.
    pub should_escalate: bool,
    /// Reason for the decision.
    pub reason: EscalationReason,
    /// Tier2 candidate count at decision time.
    pub tier2_candidate_count: usize,
    /// Tier2 confidence score at decision time.
    pub tier2_confidence: u16,
    /// Maximum Tier3 candidates to fetch if escalating.
    pub tier3_candidate_budget: usize,
}

impl EscalationDecision {
    /// Creates a decision to not escalate.
    pub fn no_escalation(tier2_candidate_count: usize, tier2_confidence: u16) -> Self {
        Self {
            should_escalate: false,
            reason: EscalationReason::NotTriggered,
            tier2_candidate_count,
            tier2_confidence,
            tier3_candidate_budget: 0,
        }
    }

    /// Creates a decision to escalate with a specific reason.
    pub fn escalate(
        reason: EscalationReason,
        tier2_candidate_count: usize,
        tier2_confidence: u16,
        tier3_budget: usize,
    ) -> Self {
        Self {
            should_escalate: true,
            reason,
            tier2_candidate_count,
            tier2_confidence,
            tier3_candidate_budget: tier3_budget,
        }
    }

    /// Evaluates whether to escalate based on Tier2 results.
    pub fn evaluate(
        tier2_results: &CandidateSet,
        tier2_confidence: u16,
        config: &EscalationConfig,
        request: &RetrievalRequest,
    ) -> Self {
        // If request explicitly disables Tier3 fallback, respect that
        if !request.enable_tier3_fallback {
            return Self::no_escalation(tier2_results.len(), tier2_confidence);
        }

        let candidate_count = tier2_results.len();

        // Tier2 returned zero results
        if candidate_count == 0 {
            return Self::escalate(
                EscalationReason::Tier2ZeroResults,
                0,
                tier2_confidence,
                config.tier3_candidate_limit,
            );
        }

        // Tier2 returned insufficient results
        if candidate_count < config.tier2_min_results {
            return Self::escalate(
                EscalationReason::Tier2Underfill,
                candidate_count,
                tier2_confidence,
                config.tier3_candidate_limit,
            );
        }

        // Low confidence in Tier2 results
        if tier2_confidence < config.confidence_threshold {
            return Self::escalate(
                EscalationReason::LowConfidence,
                candidate_count,
                tier2_confidence,
                config.tier3_candidate_limit,
            );
        }

        Self::no_escalation(candidate_count, tier2_confidence)
    }
}

// ── Planner trace ─────────────────────────────────────────────────────────────

/// Per-stage trace record for retrieval execution.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StageTrace {
    /// Stage that was executed.
    pub stage: PlannerStage,
    /// Candidates before this stage.
    pub candidates_in: usize,
    /// Candidates after this stage.
    pub candidates_out: usize,
    /// Bounded candidate cap or budget applied to this stage.
    pub stage_budget: usize,
    /// Whether this stage was bypassed (zero budget).
    pub was_bypassed: bool,
    /// Execution hint for logging.
    pub execution_hint: &'static str,
    /// Whether this stage contributed to the bounded rerank slice.
    pub rerank_related: bool,
}

impl StageTrace {
    /// Creates a trace for an executed stage.
    pub fn executed(
        stage: PlannerStage,
        candidates_in: usize,
        candidates_out: usize,
        stage_budget: usize,
    ) -> Self {
        Self {
            stage,
            candidates_in,
            candidates_out,
            stage_budget,
            was_bypassed: false,
            execution_hint: "normal",
            rerank_related: stage.is_rerank_stage(),
        }
    }

    /// Creates a trace for a bypassed stage.
    pub fn bypassed(stage: PlannerStage, stage_budget: usize) -> Self {
        Self {
            stage,
            candidates_in: 0,
            candidates_out: 0,
            stage_budget,
            was_bypassed: true,
            execution_hint: "bypassed",
            rerank_related: stage.is_rerank_stage(),
        }
    }

    /// Returns a human-readable summary.
    pub fn summary(&self) -> String {
        let rerank_suffix = if self.rerank_related {
            ", rerank slice"
        } else {
            ""
        };
        if self.was_bypassed {
            format!(
                "[{}] BYPASSED (budget cap {}{})",
                self.stage.as_str(),
                self.stage_budget,
                rerank_suffix
            )
        } else {
            format!(
                "[{}] {} → {} candidates (budget cap {}{})",
                self.stage.as_str(),
                self.candidates_in,
                self.candidates_out,
                self.stage_budget,
                rerank_suffix
            )
        }
    }
}

/// Complete retrieval plan trace.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RetrievalPlanTrace {
    /// Request that generated this plan.
    pub query_path: QueryPath,
    /// Namespace scope.
    pub namespace: NamespaceId,
    /// Requested limit.
    pub requested_limit: usize,
    /// Candidate budget.
    pub candidate_budget: usize,
    /// Optional lexical prefilter query prepared from request hints.
    pub lexical_query: Option<Fts5Query>,
    /// Optional temporal prefilter query prepared from request hints.
    pub temporal_query: Option<TemporalQuery>,
    /// Optional entity prefilter query prepared from request hints.
    pub entity_query: Option<EntityQuery>,
    /// Stage traces in execution order.
    pub stages: Vec<StageTrace>,
    /// Final candidate count.
    pub final_candidates: usize,
    /// Whether Tier3 was triggered.
    pub tier3_triggered: bool,
    /// Escalation reason (if Tier3 triggered).
    pub escalation_reason: EscalationReason,
    /// Whether graph expansion was triggered.
    pub graph_expansion_triggered: bool,
    /// Total candidates inspected across all stages.
    pub total_candidates_inspected: usize,
}

impl RetrievalPlanTrace {
    /// Creates a new empty trace.
    pub fn new(request: &RetrievalRequest) -> Self {
        Self {
            query_path: request.query_path,
            namespace: request.namespace.clone(),
            requested_limit: request.limit,
            candidate_budget: request.candidate_budget,
            lexical_query: request.lexical_prefilter_query(),
            temporal_query: request.temporal_prefilter_query(),
            entity_query: request.entity_prefilter_query(),
            stages: Vec::new(),
            final_candidates: 0,
            tier3_triggered: false,
            escalation_reason: EscalationReason::NotTriggered,
            graph_expansion_triggered: false,
            total_candidates_inspected: 0,
        }
    }

    /// Adds a stage trace.
    pub fn add_stage(&mut self, trace: StageTrace) {
        self.total_candidates_inspected += trace.candidates_in.max(trace.candidates_out);
        self.stages.push(trace);
    }

    /// Marks the final candidate count.
    pub fn set_final_candidates(&mut self, count: usize) {
        self.final_candidates = count;
    }

    /// Marks Tier3 as triggered with a reason.
    pub fn set_tier3_triggered(&mut self, reason: EscalationReason) {
        self.tier3_triggered = true;
        self.escalation_reason = reason;
    }

    /// Marks graph expansion as triggered.
    pub fn set_graph_expansion_triggered(&mut self) {
        self.graph_expansion_triggered = true;
    }

    /// Returns a human-readable summary.
    pub fn summary(&self) -> String {
        let mut lines = vec![format!(
            "RetrievalPlanTrace [path={}, limit={}, budget={}]",
            self.query_path.as_str(),
            self.requested_limit,
            self.candidate_budget
        )];

        for stage in &self.stages {
            lines.push(format!("  {}", stage.summary()));
        }

        if let Some(query) = &self.lexical_query {
            lines.push(format!(
                "  lexical_query: terms={:?}, session={:?}, memory_type={:?}, budget={}",
                query.terms, query.session_filter, query.memory_type_filter, query.candidate_budget
            ));
        }
        if let Some(query) = &self.temporal_query {
            lines.push(format!(
                "  temporal_query: session={:?}, tick_range={:?}, epoch={:?}, budget={}",
                query.session_filter, query.tick_range, query.epoch_filter, query.candidate_budget
            ));
        }
        if let Some(query) = &self.entity_query {
            lines.push(format!(
                "  entity_query: name={:?}, type={:?}, budget={}",
                query.entity_name, query.entity_type, query.candidate_budget
            ));
        }

        lines.push(format!(
            "  Final: {} candidates, Tier3: {}, Graph: {}",
            self.final_candidates,
            if self.tier3_triggered {
                self.escalation_reason.as_str()
            } else {
                "no"
            },
            if self.graph_expansion_triggered {
                "yes"
            } else {
                "no"
            }
        ));

        lines.join("\n")
    }

    /// Returns true if any stage was bypassed.
    pub fn has_bypassed_stages(&self) -> bool {
        self.stages.iter().any(|s| s.was_bypassed)
    }

    /// Returns true if any stage performed a full scan (should never happen).
    pub fn has_full_scan(&self) -> bool {
        // By design, our planner never does full scans
        false
    }
}

// ── Retrieval plan ────────────────────────────────────────────────────────────

/// Bounded retrieval plan with explicit stages.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RetrievalPlan {
    /// Request that generated this plan.
    pub request: RetrievalRequest,
    /// Escalation configuration.
    pub escalation_config: EscalationConfig,
    /// Planned stages in execution order.
    pub stages: Vec<PlannerStage>,
    /// Pre-computed candidate caps per stage.
    pub stage_budgets: Vec<usize>,
}

impl RetrievalPlan {
    /// Creates a retrieval plan for the given request.
    pub fn new(request: RetrievalRequest) -> Self {
        Self::with_config(request, EscalationConfig::default())
    }

    /// Creates a retrieval plan with explicit escalation config.
    pub fn with_config(request: RetrievalRequest, escalation_config: EscalationConfig) -> Self {
        let mut stages = Vec::new();
        let mut stage_budgets = Vec::new();

        // Stage 1: Lexical prefilter (if applicable)
        if request.requires_lexical_prefilter() {
            stages.push(PlannerStage::LexicalPrefilter);
            stage_budgets.push(request.candidate_budget);
        }

        // Stage 2: Hot semantic search (if applicable)
        if request.requires_semantic_search() {
            stages.push(PlannerStage::HotSemanticSearch);
            stage_budgets.push(request.candidate_budget.min(100)); // Bounded top-K for HNSW
        }

        // Stage 3: Float32 rescore
        stages.push(PlannerStage::Float32Rescore);
        stage_budgets.push(request.limit.min(request.candidate_budget).min(20)); // Final bounded rerank slice

        // Stage 4: Tier3 fallback (conditional)
        if request.enable_tier3_fallback {
            stages.push(PlannerStage::Tier3ColdSearch);
            stage_budgets.push(escalation_config.tier3_candidate_limit);
        }

        // Stage 5: Graph expansion (conditional)
        if request.enable_graph_expansion {
            stages.push(PlannerStage::GraphExpansion);
            stage_budgets.push(50); // Bounded expansion
        }

        // Stage 6: Final packaging
        stages.push(PlannerStage::FinalPackaging);
        stage_budgets.push(request.limit);

        Self {
            request,
            escalation_config,
            stages,
            stage_budgets,
        }
    }

    /// Returns the total number of stages.
    pub fn stage_count(&self) -> usize {
        self.stages.len()
    }

    /// Returns the budget for a specific stage.
    pub fn budget_for_stage(&self, stage: PlannerStage) -> Option<usize> {
        self.stages
            .iter()
            .position(|&s| s == stage)
            .map(|idx| self.stage_budgets[idx])
    }

    /// Returns true if this plan includes Tier3 fallback.
    pub fn includes_tier3(&self) -> bool {
        self.stages.contains(&PlannerStage::Tier3ColdSearch)
    }

    /// Returns true if this plan includes graph expansion.
    pub fn includes_graph_expansion(&self) -> bool {
        self.stages.contains(&PlannerStage::GraphExpansion)
    }
}

// ── Retrieval planner ──────────────────────────────────────────────────────────

/// Bounded retrieval planner implementing the canonical escalation chain.
#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct RetrievalPlanner {
    escalation_config: EscalationConfig,
}

impl RetrievalPlanner {
    /// Creates a new retrieval planner with default config.
    pub fn new() -> Self {
        Self::default()
    }

    /// Creates a retrieval planner with explicit escalation config.
    pub fn with_config(escalation_config: EscalationConfig) -> Self {
        Self { escalation_config }
    }

    /// Plans the retrieval for the given request.
    pub fn plan(&self, request: RetrievalRequest) -> RetrievalPlan {
        RetrievalPlan::with_config(request, self.escalation_config.clone())
    }

    /// Evaluates whether to escalate to Tier3 based on Tier2 results.
    pub fn evaluate_escalation(
        &self,
        tier2_results: &CandidateSet,
        tier2_confidence: u16,
        request: &RetrievalRequest,
    ) -> EscalationDecision {
        EscalationDecision::evaluate(
            tier2_results,
            tier2_confidence,
            &self.escalation_config,
            request,
        )
    }

    /// Creates a trace for a new retrieval execution.
    pub fn start_trace(&self, request: &RetrievalRequest) -> RetrievalPlanTrace {
        RetrievalPlanTrace::new(request)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::index::{CandidateSet, IndexFamily};
    use crate::types::{CanonicalMemoryType, MemoryId, SessionId};

    fn test_namespace() -> NamespaceId {
        NamespaceId::new("test.namespace").unwrap()
    }

    #[test]
    fn hybrid_request_includes_lexical_and_semantic() {
        let ns = test_namespace();
        let request = RetrievalRequest::hybrid(ns, "hello world", 10);

        assert!(request.query_path.requires_lexical_prefilter());
        assert!(request.query_path.requires_semantic_search());
        assert_eq!(request.limit, 10);
        assert!(request.enable_tier3_fallback);
        assert_eq!(request.query_text.as_deref(), Some("hello world"));
        assert!(request.like_memory_id.is_none());
        assert!(request.unlike_memory_id.is_none());
    }

    #[test]
    fn query_by_example_request_uses_like_seed_as_primary_cue() {
        let ns = test_namespace();
        let request = RetrievalRequest::query_by_example(ns, MemoryId(7), 3);

        let normalized = request.normalize_query_by_example().unwrap();

        assert_eq!(normalized.normalized_query_text, None);
        assert_eq!(normalized.primary_cue, PrimaryCue::LikeId);
        assert_eq!(normalized.seeds.len(), 1);
        assert_eq!(normalized.seeds[0].memory_id, MemoryId(7));
        assert_eq!(normalized.seeds[0].polarity, QueryExamplePolarity::Like);
        assert!(!request.requires_lexical_prefilter());
        assert!(request.requires_semantic_search());
    }

    #[test]
    fn normalization_preserves_query_text_as_primary_cue_when_examples_exist() {
        let ns = test_namespace();
        let request = RetrievalRequest::hybrid(ns, "  example cue  ", 5)
            .with_like_memory(MemoryId(11))
            .with_unlike_memory(MemoryId(12));

        let normalized = request.normalize_query_by_example().unwrap();

        assert_eq!(
            normalized.normalized_query_text.as_deref(),
            Some("example cue")
        );
        assert_eq!(normalized.primary_cue, PrimaryCue::QueryText);
        assert!(normalized.uses_query_text_as_primary_cue());
        assert!(normalized.has_example_seeds());
        assert_eq!(normalized.seeds.len(), 2);
        assert_eq!(normalized.seeds[0].polarity, QueryExamplePolarity::Like);
        assert_eq!(normalized.seeds[1].polarity, QueryExamplePolarity::Unlike);
        assert_eq!(normalized.seed_descriptors(), vec!["like:11", "unlike:12"]);
        assert_eq!(normalized.seed_polarities(), vec!["like", "unlike"]);
        assert_eq!(
            normalized.seed_memory_ids(),
            vec![MemoryId(11), MemoryId(12)]
        );
    }

    #[test]
    fn normalization_rejects_requests_without_query_text_or_example_seed() {
        let ns = test_namespace();
        let request = RetrievalRequest::hybrid(ns, "   ", 5);

        let error = request.normalize_query_by_example().unwrap_err();

        assert_eq!(error, RetrievalRequestValidationError::MissingPrimaryCue);
        assert_eq!(error.as_str(), "missing_primary_cue");
    }

    #[test]
    fn normalization_uses_unlike_seed_as_primary_cue_when_query_text_is_absent() {
        let ns = test_namespace();
        let request = RetrievalRequest::exact_id(ns, MemoryId(44)).with_unlike_memory(MemoryId(17));

        let normalized = request.normalize_query_by_example().unwrap();

        assert_eq!(normalized.normalized_query_text, None);
        assert_eq!(normalized.primary_cue, PrimaryCue::UnlikeId);
        assert!(!normalized.uses_query_text_as_primary_cue());
        assert!(normalized.has_example_seeds());
        assert_eq!(normalized.seed_descriptors(), vec!["unlike:17"]);
        assert_eq!(normalized.seed_polarities(), vec!["unlike"]);
        assert_eq!(normalized.seed_memory_ids(), vec![MemoryId(17)]);
    }

    #[test]
    fn normalization_rejects_duplicate_like_and_unlike_seed() {
        let ns = test_namespace();
        let request =
            RetrievalRequest::query_by_example(ns, MemoryId(9), 5).with_unlike_memory(MemoryId(9));

        let error = request.normalize_query_by_example().unwrap_err();

        assert_eq!(
            error,
            RetrievalRequestValidationError::DuplicateExampleCue(MemoryId(9))
        );
        assert_eq!(error.as_str(), "duplicate_example_cue");
    }

    #[test]
    fn exact_id_request_skips_lexical_and_semantic() {
        let ns = test_namespace();
        let request = RetrievalRequest::exact_id(ns, MemoryId(42));

        assert!(!request.query_path.requires_lexical_prefilter());
        assert!(!request.query_path.requires_semantic_search());
        assert_eq!(request.limit, 1);
        assert!(!request.enable_tier3_fallback);
    }

    #[test]
    fn temporal_requests_build_bounded_temporal_prefilter_queries() {
        let ns = test_namespace();
        let request = RetrievalRequest::hybrid(ns.clone(), "recent deploys", 8)
            .with_session(SessionId(33))
            .with_budget(77);
        let request = RetrievalRequest {
            query_path: QueryPath::Temporal,
            ..request
        };

        let query = request.temporal_prefilter_query().unwrap();

        assert_eq!(query.namespace, ns);
        assert_eq!(query.session_filter, Some(SessionId(33)));
        assert_eq!(query.limit, 8);
        assert_eq!(query.candidate_budget, 77);
        assert!(request.entity_prefilter_query().is_none());
    }

    #[test]
    fn entity_heavy_requests_build_bounded_entity_prefilter_queries() {
        let ns = test_namespace();
        let request = RetrievalRequest::hybrid(ns.clone(), "service ownership", 6)
            .with_entity_name("payments-api")
            .with_entity_type("service")
            .with_budget(55);
        let request = RetrievalRequest {
            query_path: QueryPath::EntityHeavy,
            ..request
        };

        let query = request.entity_prefilter_query().unwrap();

        assert_eq!(query.namespace, ns);
        assert_eq!(query.entity_name.as_deref(), Some("payments-api"));
        assert_eq!(query.entity_type.as_deref(), Some("service"));
        assert_eq!(query.limit, 6);
        assert_eq!(query.candidate_budget, 55);
        assert!(request.temporal_prefilter_query().is_none());
    }

    #[test]
    fn lexical_prefilter_query_preserves_session_and_memory_type_filters() {
        let ns = test_namespace();
        let request = RetrievalRequest::hybrid(ns.clone(), " rollback plan ", 4)
            .with_session(SessionId(9))
            .with_memory_type(CanonicalMemoryType::Event)
            .with_budget(88);

        let query = request.lexical_prefilter_query().unwrap();

        assert_eq!(query.namespace, ns);
        assert_eq!(query.terms, "rollback plan");
        assert_eq!(query.session_filter, Some(SessionId(9)));
        assert_eq!(query.memory_type_filter, Some(CanonicalMemoryType::Event));
        assert_eq!(query.limit, 4);
        assert_eq!(query.candidate_budget, 88);
    }

    #[test]
    fn temporal_and_entity_paths_do_not_emit_lexical_queries_in_trace() {
        let ns = test_namespace();
        let temporal = RetrievalRequest {
            query_path: QueryPath::Temporal,
            ..RetrievalRequest::hybrid(ns.clone(), "recent deploys", 8).with_session(SessionId(33))
        };
        let entity = RetrievalRequest {
            query_path: QueryPath::EntityHeavy,
            ..RetrievalRequest::hybrid(ns, "service ownership", 6)
                .with_entity_name("payments-api")
                .with_entity_type("service")
        };

        let temporal_trace = RetrievalPlanTrace::new(&temporal);
        let entity_trace = RetrievalPlanTrace::new(&entity);

        assert!(temporal_trace.lexical_query.is_none());
        assert!(temporal_trace.temporal_query.is_some());
        assert!(entity_trace.lexical_query.is_none());
        assert!(entity_trace.entity_query.is_some());
    }

    #[test]
    fn retrieval_plan_hybrid_includes_all_stages() {
        let ns = test_namespace();
        let request = RetrievalRequest::hybrid(ns, "test", 10).with_graph_expansion(true);

        let plan = RetrievalPlan::new(request);

        assert!(plan.stages.contains(&PlannerStage::LexicalPrefilter));
        assert!(plan.stages.contains(&PlannerStage::HotSemanticSearch));
        assert!(plan.stages.contains(&PlannerStage::Float32Rescore));
        assert!(plan.includes_tier3());
        assert!(plan.includes_graph_expansion());
        assert!(plan.stages.contains(&PlannerStage::FinalPackaging));
    }

    #[test]
    fn query_by_example_plan_skips_lexical_prefilter_without_query_text() {
        let ns = test_namespace();
        let request = RetrievalRequest::query_by_example(ns, MemoryId(3), 10);

        let plan = RetrievalPlan::new(request);

        assert!(!plan.stages.contains(&PlannerStage::LexicalPrefilter));
        assert!(plan.stages.contains(&PlannerStage::HotSemanticSearch));
        assert!(plan.stages.contains(&PlannerStage::Float32Rescore));
    }

    #[test]
    fn unlike_only_query_by_example_still_schedules_semantic_search() {
        let ns = test_namespace();
        let request = RetrievalRequest::exact_id(ns, MemoryId(44)).with_unlike_memory(MemoryId(17));

        let plan = RetrievalPlan::new(request);

        assert!(!plan.stages.contains(&PlannerStage::LexicalPrefilter));
        assert!(plan.stages.contains(&PlannerStage::HotSemanticSearch));
        assert!(plan.stages.contains(&PlannerStage::Float32Rescore));
    }

    #[test]
    fn whitespace_only_query_text_with_example_seed_skips_lexical_prefilter() {
        let ns = test_namespace();
        let request = RetrievalRequest::hybrid(ns, "   ", 10).with_like_memory(MemoryId(3));

        let normalized = request.normalize_query_by_example().unwrap();
        let plan = RetrievalPlan::new(request);

        assert_eq!(normalized.normalized_query_text, None);
        assert_eq!(normalized.primary_cue, PrimaryCue::LikeId);
        assert!(!plan.stages.contains(&PlannerStage::LexicalPrefilter));
        assert!(plan.stages.contains(&PlannerStage::HotSemanticSearch));
        assert!(plan.stages.contains(&PlannerStage::Float32Rescore));
    }

    #[test]
    fn retrieval_plan_exact_id_minimal_stages() {
        let ns = test_namespace();
        let request = RetrievalRequest::exact_id(ns, MemoryId(1));
        let plan = RetrievalPlan::new(request);

        // Exact ID should only have rescore and packaging
        assert!(!plan.stages.contains(&PlannerStage::LexicalPrefilter));
        assert!(!plan.stages.contains(&PlannerStage::HotSemanticSearch));
        assert!(!plan.includes_tier3());
    }

    #[test]
    fn stage_budgets_are_bounded() {
        let ns = test_namespace();
        let request = RetrievalRequest::hybrid(ns, "test", 10).with_budget(1000);

        let plan = RetrievalPlan::new(request);

        // All budgets should be bounded
        for &budget in &plan.stage_budgets {
            assert!(budget > 0);
            assert!(budget <= 5000); // Max reasonable budget
        }
    }

    #[test]
    fn escalation_decision_zero_results() {
        let config = EscalationConfig::default();
        let ns = test_namespace();
        let request = RetrievalRequest::hybrid(ns, "test", 10);
        let empty_results = CandidateSet::empty(100, IndexFamily::Fts5Lexical);

        let decision = EscalationDecision::evaluate(&empty_results, 800, &config, &request);

        assert!(decision.should_escalate);
        assert_eq!(decision.reason, EscalationReason::Tier2ZeroResults);
        assert_eq!(decision.tier2_candidate_count, 0);
    }

    #[test]
    fn escalation_decision_underfill() {
        let config = EscalationConfig {
            tier2_min_results: 5,
            ..Default::default()
        };
        let ns = test_namespace();
        let request = RetrievalRequest::hybrid(ns.clone(), "test", 10);

        // Create a candidate set with only 2 candidates (underfill)
        let candidates = vec![
            crate::index::IndexCandidate::new(
                MemoryId(1),
                IndexFamily::Fts5Lexical,
                800,
                ns.clone(),
            ),
            crate::index::IndexCandidate::new(MemoryId(2), IndexFamily::Fts5Lexical, 700, ns),
        ];
        let small_results =
            CandidateSet::from_candidates(candidates, 10, 2, IndexFamily::Fts5Lexical);
        let decision = EscalationDecision::evaluate(&small_results, 800, &config, &request);

        assert!(decision.should_escalate);
        assert_eq!(decision.reason, EscalationReason::Tier2Underfill);
    }

    #[test]
    fn escalation_decision_low_confidence() {
        let config = EscalationConfig {
            confidence_threshold: 500,
            tier2_min_results: 1,
            ..Default::default()
        };
        let ns = test_namespace();
        let request = RetrievalRequest::hybrid(ns.clone(), "test", 10);

        // Create a candidate set with 10 candidates but low confidence
        let candidates: Vec<crate::index::IndexCandidate> = (1..=10)
            .map(|i| {
                crate::index::IndexCandidate::new(
                    MemoryId(i),
                    IndexFamily::Fts5Lexical,
                    800 - i as u16,
                    ns.clone(),
                )
            })
            .collect();
        let results = CandidateSet::from_candidates(candidates, 10, 10, IndexFamily::Fts5Lexical);
        let decision = EscalationDecision::evaluate(&results, 300, &config, &request);

        assert!(decision.should_escalate);
        assert_eq!(decision.reason, EscalationReason::LowConfidence);
    }

    #[test]
    fn escalation_decision_sufficient_results() {
        let config = EscalationConfig {
            tier2_min_results: 5,
            confidence_threshold: 500,
            ..Default::default()
        };
        let ns = test_namespace();
        let request = RetrievalRequest::hybrid(ns.clone(), "test", 10);

        // Create a candidate set with 10 candidates and high confidence
        let candidates: Vec<crate::index::IndexCandidate> = (1..=10)
            .map(|i| {
                crate::index::IndexCandidate::new(
                    MemoryId(i),
                    IndexFamily::Fts5Lexical,
                    900 - i as u16,
                    ns.clone(),
                )
            })
            .collect();
        let results = CandidateSet::from_candidates(candidates, 10, 10, IndexFamily::Fts5Lexical);
        let decision = EscalationDecision::evaluate(&results, 800, &config, &request);

        assert!(!decision.should_escalate);
        assert_eq!(decision.reason, EscalationReason::NotTriggered);
    }

    #[test]
    fn escalation_disabled_by_request() {
        let config = EscalationConfig::default();
        let ns = test_namespace();
        let request = RetrievalRequest::hybrid(ns, "test", 10).with_tier3_fallback(false);

        let empty_results = CandidateSet::empty(0, IndexFamily::Fts5Lexical);
        let decision = EscalationDecision::evaluate(&empty_results, 800, &config, &request);

        // Even with zero results, Tier3 should not be triggered when disabled by request
        assert!(!decision.should_escalate);
        assert_eq!(decision.reason, EscalationReason::NotTriggered);
    }

    #[test]
    fn stage_trace_summary() {
        let trace = StageTrace::executed(PlannerStage::LexicalPrefilter, 1000, 50, 5_000);
        assert!(trace.summary().contains("1000 → 50"));
        assert!(trace.summary().contains("budget cap 5000"));
        assert_eq!(trace.stage_budget, 5_000);

        let bypassed = StageTrace::bypassed(PlannerStage::HotSemanticSearch, 100);
        assert!(bypassed.summary().contains("BYPASSED"));
        assert!(bypassed.summary().contains("budget cap 100"));
        assert_eq!(bypassed.stage_budget, 100);
    }

    #[test]
    fn retrieval_plan_trace_summary() {
        let ns = test_namespace();
        let request = RetrievalRequest::hybrid(ns, "test", 10);
        let mut trace = RetrievalPlanTrace::new(&request);

        trace.add_stage(StageTrace::executed(
            PlannerStage::LexicalPrefilter,
            1000,
            50,
            5_000,
        ));
        trace.add_stage(StageTrace::executed(
            PlannerStage::HotSemanticSearch,
            50,
            20,
            100,
        ));
        trace.add_stage(StageTrace::executed(
            PlannerStage::Float32Rescore,
            20,
            10,
            10,
        ));
        trace.set_final_candidates(10);
        trace.set_tier3_triggered(EscalationReason::Tier2Underfill);

        let summary = trace.summary();
        assert!(summary.contains("hybrid"));
        assert!(summary.contains("lexical_prefilter"));
        assert!(summary.contains("lexical_query: terms=\"test\""));
        assert!(summary.contains("budget cap 5000"));
        assert!(summary.contains("budget cap 100"));
        assert!(summary.contains("rerank slice"));
        assert!(summary.contains("tier2_underfill"));
    }

    #[test]
    fn retrieval_plan_trace_no_full_scan() {
        let ns = test_namespace();
        let request = RetrievalRequest::hybrid(ns, "test", 10);
        let trace = RetrievalPlanTrace::new(&request);

        // By design, our planner never does full scans
        assert!(!trace.has_full_scan());
    }

    #[test]
    fn planner_creates_correct_plan() {
        let planner = RetrievalPlanner::new();
        let ns = test_namespace();
        let request = RetrievalRequest::hybrid(ns, "test", 10);

        let plan = planner.plan(request);
        assert!(plan.stage_count() >= 3); // At least prefilter, rescore, packaging
    }

    #[test]
    fn query_path_classification() {
        assert!(QueryPath::Hybrid.requires_lexical_prefilter());
        assert!(QueryPath::Hybrid.requires_semantic_search());

        assert!(!QueryPath::Temporal.requires_lexical_prefilter());
        assert!(!QueryPath::Temporal.requires_semantic_search());

        assert!(!QueryPath::EntityHeavy.requires_lexical_prefilter());
        assert!(!QueryPath::EntityHeavy.requires_semantic_search());

        assert!(!QueryPath::SemanticOnly.requires_lexical_prefilter());
        assert!(QueryPath::SemanticOnly.requires_semantic_search());

        assert!(!QueryPath::ExactId.requires_lexical_prefilter());
        assert!(!QueryPath::ExactId.requires_semantic_search());
    }

    #[test]
    fn planner_stage_tier_classification() {
        assert!(PlannerStage::LexicalPrefilter.is_tier2());
        assert!(PlannerStage::HotSemanticSearch.is_tier2());
        assert!(PlannerStage::Float32Rescore.is_tier2());
        assert!(!PlannerStage::Tier3ColdSearch.is_tier2());

        assert!(PlannerStage::Tier3ColdSearch.is_tier3());
        assert!(!PlannerStage::HotSemanticSearch.is_tier3());
        assert!(PlannerStage::Float32Rescore.is_rerank_stage());
        assert!(!PlannerStage::GraphExpansion.is_rerank_stage());
    }

    #[test]
    fn stage_budgets_respect_request_limit_for_rerank_slice() {
        let ns = test_namespace();
        let request = RetrievalRequest::hybrid(ns, "test", 7).with_budget(80);

        let plan = RetrievalPlan::new(request);

        assert_eq!(
            plan.budget_for_stage(PlannerStage::HotSemanticSearch),
            Some(80)
        );
        assert_eq!(plan.budget_for_stage(PlannerStage::Float32Rescore), Some(7));
    }
}
