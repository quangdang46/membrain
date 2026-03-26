//! Bounded cache families, admission, prefetch, invalidation, and observability
//! for the retrieval hot path.
//!
//! Every cache is a derived accelerator — not a source of truth. Durable records,
//! canonical embeddings, and policy-bearing metadata remain authoritative.
//! Warm state may be dropped, bypassed, or rebuilt without changing retrieval
//! semantics.
//!
//! Refs: docs/CACHE_AND_PREFETCH.md and docs/PLAN.md section 20.

use crate::api::NamespaceId;
use crate::observability::{
    CacheEvalTrace, CacheEventLabel, CacheFamilyLabel, CacheLookupOutcome, CacheReasonLabel,
    GenerationStatusLabel, WarmSourceLabel,
};
use crate::types::MemoryId;
use std::collections::{HashMap, VecDeque};

// ──────────────────────────────────────────────────────────────────────────────
// § 1  Cache family taxonomy  (mb-23u.9.1)
// ──────────────────────────────────────────────────────────────────────────────

/// Cache family tags aligned with docs/CACHE_AND_PREFETCH.md taxonomy.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum CacheFamily {
    /// Exact and recent fetches for already-authorized items.
    Tier1Item,
    /// Repeated structurally valid misses under current policy scope.
    NegativeCache,
    /// Packaged recall results for normalized request shape.
    ResultCache,
    /// Graph neighborhood lookups derived from canonical relation tables.
    EntityNeighborhood,
    /// Derived summaries and projections.
    SummaryCache,
    /// Vector-search shortlist generation for current index generation.
    AnnProbeCache,
    /// Speculative handles or shortlists keyed to session or task intent.
    PrefetchHints,
    /// Session-local hot set for current scoped session.
    SessionWarmup,
    /// Task or goal-local shortlist for repeated retrieval.
    GoalConditioned,
    /// Process-local bootstrap warm artifacts only.
    ColdStartMitigation,
}

impl CacheFamily {
    pub const ALL: [Self; 10] = [
        Self::Tier1Item,
        Self::NegativeCache,
        Self::ResultCache,
        Self::EntityNeighborhood,
        Self::SummaryCache,
        Self::AnnProbeCache,
        Self::PrefetchHints,
        Self::SessionWarmup,
        Self::GoalConditioned,
        Self::ColdStartMitigation,
    ];

    /// Returns the canonical machine-readable tag for serialization and traces.
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::Tier1Item => "tier1_item",
            Self::NegativeCache => "negative_cache",
            Self::ResultCache => "result_cache",
            Self::EntityNeighborhood => "entity_neighborhood",
            Self::SummaryCache => "summary_cache",
            Self::AnnProbeCache => "ann_probe_cache",
            Self::PrefetchHints => "prefetch_hints",
            Self::SessionWarmup => "session_warmup",
            Self::GoalConditioned => "goal_conditioned",
            Self::ColdStartMitigation => "cold_start_mitigation",
        }
    }

    const fn metric_index(self) -> usize {
        match self {
            Self::Tier1Item => 0,
            Self::NegativeCache => 1,
            Self::ResultCache => 2,
            Self::EntityNeighborhood => 3,
            Self::SummaryCache => 4,
            Self::AnnProbeCache => 5,
            Self::PrefetchHints => 6,
            Self::SessionWarmup => 7,
            Self::GoalConditioned => 8,
            Self::ColdStartMitigation => 9,
        }
    }
}

/// Cache events aligned with the canonical event taxonomy.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum CacheEvent {
    /// Cache returned a valid entry for current request.
    Hit,
    /// No valid entry found in cache for current key and scope.
    Miss,
    /// Cache exists but was skipped due to staleness, version mismatch, etc.
    Bypass,
    /// Cache entry invalidated due to authoritative mutation or generation change.
    Invalidation,
    /// Cache populated or refreshed during repair operation.
    RepairWarmup,
    /// Cache entry exists but was rejected as stale.
    StaleWarning,
    /// Cache is explicitly disabled or in degraded mode.
    Disabled,
    /// Prefetch hint canceled due to intent change, budget, or policy.
    PrefetchDrop,
    /// Session warmup invalidated due to session end or scope change.
    SessionExpired,
}

impl CacheEvent {
    /// Returns the machine-readable event label.
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::Hit => "hit",
            Self::Miss => "miss",
            Self::Bypass => "bypass",
            Self::Invalidation => "invalidation",
            Self::RepairWarmup => "repair_warmup",
            Self::StaleWarning => "stale_warning",
            Self::Disabled => "disabled",
            Self::PrefetchDrop => "prefetch_drop",
            Self::SessionExpired => "session_expired",
        }
    }
}

/// Reasons for cache bypass or invalidation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum CacheReason {
    OwnerBoundaryMismatch,
    NamespaceMismatch,
    PolicyDenied,
    GenerationAnchorMismatch,
    VersionMismatch,
    ScopeTooBroad,
    RecordNotPresent,
    RepairIncomplete,
    PolicyChanged,
    RedactionChanged,
    SchemaChanged,
    IndexChanged,
    EmbeddingChanged,
    RankingChanged,
    IntentChanged,
    BudgetExhausted,
    NamespaceNarrowed,
    NamespaceWidened,
}

impl CacheReason {
    /// Returns the machine-readable reason label.
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::OwnerBoundaryMismatch => "owner_boundary_mismatch",
            Self::NamespaceMismatch => "namespace_mismatch",
            Self::PolicyDenied => "policy_denied",
            Self::GenerationAnchorMismatch => "generation_anchor_mismatch",
            Self::VersionMismatch => "version_mismatch",
            Self::ScopeTooBroad => "scope_too_broad",
            Self::RecordNotPresent => "record_not_present",
            Self::RepairIncomplete => "repair_incomplete",
            Self::PolicyChanged => "policy_changed",
            Self::RedactionChanged => "redaction_changed",
            Self::SchemaChanged => "schema_changed",
            Self::IndexChanged => "index_changed",
            Self::EmbeddingChanged => "embedding_changed",
            Self::RankingChanged => "ranking_changed",
            Self::IntentChanged => "intent_changed",
            Self::BudgetExhausted => "budget_exhausted",
            Self::NamespaceNarrowed => "namespace_narrowed",
            Self::NamespaceWidened => "namespace_widened",
        }
    }
}

/// Labels for warm-source provenance in traces.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum WarmSource {
    Tier1ItemCache,
    NegativeCache,
    ResultCache,
    EntityNeighborhood,
    SummaryCache,
    AnnProbeCache,
    PrefetchHints,
    SessionWarmup,
    GoalConditioned,
    ColdStartMitigation,
}

impl WarmSource {
    /// Returns the machine-readable warm-source label.
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::Tier1ItemCache => "tier1_item_cache",
            Self::NegativeCache => "negative_cache",
            Self::ResultCache => "result_cache",
            Self::EntityNeighborhood => "entity_neighborhood",
            Self::SummaryCache => "summary_cache",
            Self::AnnProbeCache => "ann_probe_cache",
            Self::PrefetchHints => "prefetch_hints",
            Self::SessionWarmup => "session_warmup",
            Self::GoalConditioned => "goal_conditioned",
            Self::ColdStartMitigation => "cold_start_mitigation",
        }
    }
}

/// Cache entry generation status for version-aware validation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum GenerationStatus {
    /// Cache entry has all required generation anchors.
    Valid,
    /// Cache entry exists but lacks fresh generation anchor.
    Stale,
    /// Cache entry generation version differs from current required version.
    VersionMismatched,
    /// Cache entry generation status cannot be determined.
    Unknown,
}

impl GenerationStatus {
    /// Returns the machine-readable status label.
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::Valid => "valid",
            Self::Stale => "stale",
            Self::VersionMismatched => "version_mismatched",
            Self::Unknown => "unknown",
        }
    }
}

// ──────────────────────────────────────────────────────────────────────────────
// § 2  Cache key and versioned entry  (mb-23u.9.1)
// ──────────────────────────────────────────────────────────────────────────────

/// Generation anchors that participate in cache key freshness validation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct CacheGenerationAnchors {
    /// Schema generation when the cache entry was populated.
    pub schema_generation: u64,
    /// Policy/redaction generation when the entry was populated.
    pub policy_generation: u64,
    /// Index generation (FTS5/USearch) when the entry was populated.
    pub index_generation: u64,
    /// Embedding model generation when the entry was populated.
    pub embedding_generation: u64,
    /// Ranking/reranker model generation when the entry was populated.
    pub ranking_generation: u64,
}

impl Default for CacheGenerationAnchors {
    fn default() -> Self {
        Self {
            schema_generation: 1,
            policy_generation: 1,
            index_generation: 1,
            embedding_generation: 1,
            ranking_generation: 1,
        }
    }
}

/// Composite cache key: family + namespace + item identity + generations.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct CacheKey {
    /// Which cache family this key belongs to.
    pub family: CacheFamily,
    /// Effective namespace the entry is scoped to.
    pub namespace: NamespaceId,
    /// Optional workspace binding when visibility depends on workspace scope.
    pub workspace_key: Option<u64>,
    /// Optional owner boundary for session/task-local families.
    pub owner_key: Option<u64>,
    /// Optional normalized request-shape hash for request-local families.
    pub request_shape_hash: Option<u64>,
    /// Family-specific item identity (memory id, request hash, etc.).
    pub item_key: u64,
    /// Generation anchors snapshotted when entry was stored.
    pub generations: CacheGenerationAnchors,
}

/// Admission decision for a cache candidate.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CacheAdmissionDecision {
    /// Admit the candidate into the cache family.
    Admit,
    /// Reject—cache is at capacity and candidate does not justify eviction.
    RejectCapacity,
    /// Bypass—request explicitly opted out of caching or is policy-denied.
    BypassPolicy,
    /// Bypass—family is disabled or in degraded mode.
    BypassDisabled,
}

/// Machine-readable reason explaining an admission decision.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CacheAdmissionReason {
    /// Entry was admitted under the family contract.
    Accepted,
    /// The request did not authorize cache participation for this family.
    PolicyDenied,
    /// The family is disabled or degraded for this request.
    Disabled,
    /// The candidate payload was empty for a family that requires payload bytes.
    EmptyPayload,
    /// Capacity was reached but eviction was allowed, so the oldest entry was replaced.
    CapacityEvicted,
    /// Capacity was reached and this request could not evict an older entry.
    CapacityRejected,
}

impl CacheAdmissionReason {
    /// Returns the machine-readable reason label.
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::Accepted => "accepted",
            Self::PolicyDenied => "policy_denied",
            Self::Disabled => "disabled",
            Self::EmptyPayload => "empty_payload",
            Self::CapacityEvicted => "capacity_evicted",
            Self::CapacityRejected => "capacity_rejected",
        }
    }
}

impl CacheKey {
    /// Returns whether two keys refer to the same lookup identity independent of generations.
    pub fn same_lookup_identity(&self, other: &Self) -> bool {
        self.family == other.family
            && self.namespace == other.namespace
            && self.workspace_key == other.workspace_key
            && self.owner_key == other.owner_key
            && self.request_shape_hash == other.request_shape_hash
            && self.item_key == other.item_key
    }
}

/// Request-scoped admission inputs that make family rules explicit.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CacheAdmissionRequest {
    /// Whether request policy allows this family to participate.
    pub policy_allowed: bool,
    /// Whether this request may replace an older entry if the family is full.
    pub allow_capacity_eviction: bool,
    /// Whether request-local families may bind a normalized request shape.
    pub request_shape_hash: Option<u64>,
}

impl Default for CacheAdmissionRequest {
    fn default() -> Self {
        Self {
            policy_allowed: true,
            allow_capacity_eviction: true,
            request_shape_hash: None,
        }
    }
}

/// Structured outcome for admission evaluation and trace logging.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CacheAdmissionOutcome {
    /// Final admission decision.
    pub decision: CacheAdmissionDecision,
    /// Machine-readable reason for the decision.
    pub reason: CacheAdmissionReason,
    /// Whether the admission replaced an older resident entry.
    pub evicted_existing: bool,
}

impl CacheAdmissionOutcome {
    /// Returns the canonical outcome for an admitted entry.
    pub const fn admitted(reason: CacheAdmissionReason, evicted_existing: bool) -> Self {
        Self {
            decision: CacheAdmissionDecision::Admit,
            reason,
            evicted_existing,
        }
    }

    /// Returns the canonical outcome for a bypassed or rejected entry.
    pub const fn rejected(decision: CacheAdmissionDecision, reason: CacheAdmissionReason) -> Self {
        Self {
            decision,
            reason,
            evicted_existing: false,
        }
    }
}

/// Explicit cache hot-path lookup stages in canonical evaluation order.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum HotPathLookupStage {
    PrefetchHints,
    SessionWarmup,
    Tier1Item,
    NegativeCache,
    ResultCache,
    EntityNeighborhood,
    SummaryCache,
    AnnProbeCache,
    GoalConditioned,
    ColdStartMitigation,
}

impl HotPathLookupStage {
    /// Returns the canonical stage name.
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::PrefetchHints => "prefetch_hints",
            Self::SessionWarmup => "session_warmup",
            Self::Tier1Item => "tier1_item",
            Self::NegativeCache => "negative_cache",
            Self::ResultCache => "result_cache",
            Self::EntityNeighborhood => "entity_neighborhood",
            Self::SummaryCache => "summary_cache",
            Self::AnnProbeCache => "ann_probe_cache",
            Self::GoalConditioned => "goal_conditioned",
            Self::ColdStartMitigation => "cold_start_mitigation",
        }
    }
}

// ──────────────────────────────────────────────────────────────────────────────
// § 3  Bounded retrieval cache store  (mb-23u.9.1)
// ──────────────────────────────────────────────────────────────────────────────

/// Family-local admission behavior derived from the canonical ownership map.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CacheAdmissionPolicy {
    /// Whether empty candidate payloads may be cached for this family.
    pub allows_empty_payload: bool,
    /// Whether this family requires an owner boundary for correctness.
    pub requires_owner_key: bool,
    /// Whether this family requires a normalized request-shape hash.
    pub requires_request_shape_hash: bool,
}

impl CacheAdmissionPolicy {
    /// Returns the canonical admission policy for one cache family.
    pub const fn for_family(family: CacheFamily) -> Self {
        match family {
            CacheFamily::Tier1Item => Self {
                allows_empty_payload: false,
                requires_owner_key: false,
                requires_request_shape_hash: false,
            },
            CacheFamily::NegativeCache => Self {
                allows_empty_payload: true,
                requires_owner_key: false,
                requires_request_shape_hash: true,
            },
            CacheFamily::ResultCache | CacheFamily::AnnProbeCache => Self {
                allows_empty_payload: false,
                requires_owner_key: false,
                requires_request_shape_hash: true,
            },
            CacheFamily::EntityNeighborhood | CacheFamily::SummaryCache => Self {
                allows_empty_payload: false,
                requires_owner_key: false,
                requires_request_shape_hash: false,
            },
            CacheFamily::PrefetchHints
            | CacheFamily::SessionWarmup
            | CacheFamily::GoalConditioned => Self {
                allows_empty_payload: false,
                requires_owner_key: true,
                requires_request_shape_hash: false,
            },
            CacheFamily::ColdStartMitigation => Self {
                allows_empty_payload: false,
                requires_owner_key: false,
                requires_request_shape_hash: false,
            },
        }
    }
}

/// Cached retrieval entry with version-aware metadata.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CacheEntry {
    /// Composite key this entry was stored under.
    pub key: CacheKey,
    /// Cached memory ids (candidate shortlist).
    pub memory_ids: Vec<MemoryId>,
    /// Monotonic tick when the entry was created/refreshed.
    pub stored_at_tick: u64,
}

/// Result of a cache lookup with full trace evidence.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CacheLookupResult {
    /// Family that was queried.
    pub family: CacheFamily,
    /// Event outcome.
    pub event: CacheEvent,
    /// Reason when event is Bypass, Invalidation, or StaleWarning.
    pub reason: Option<CacheReason>,
    /// Warm source when event is Hit.
    pub warm_source: Option<WarmSource>,
    /// Generation status validation.
    pub generation_status: GenerationStatus,
    /// Returned memory ids when event is Hit.
    pub memory_ids: Vec<MemoryId>,
    /// Candidate count after this cache evaluation.
    pub candidates_after: usize,
}

/// Bounded LRU cache store for one cache family.
///
/// Namespace-aware keys and generation-aware invalidation keep this cache
/// aligned with the CACHE_AND_PREFETCH.md contract.
#[derive(Debug, Clone)]
pub struct BoundedCacheStore {
    admission_policy: CacheAdmissionPolicy,
    family: CacheFamily,
    capacity: usize,
    entries: HashMap<CacheKey, CacheEntry>,
    access_order: VecDeque<CacheKey>,
    /// Monotonic tick counter for temporal ordering.
    tick: u64,
    /// Whether this cache family is in degraded/disabled mode.
    disabled: bool,
    /// Running metrics for this family.
    pub metrics: CacheFamilyMetrics,
}

/// Per-family metrics counters for cache observability (mb-23u.9.3).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct CacheFamilyMetrics {
    pub hit_count: u64,
    pub miss_count: u64,
    pub bypass_count: u64,
    pub invalidation_count: u64,
    pub stale_warning_count: u64,
}

impl BoundedCacheStore {
    /// Builds a new bounded cache for the given family with a hard capacity limit.
    pub fn new(family: CacheFamily, capacity: usize) -> Self {
        Self {
            admission_policy: CacheAdmissionPolicy::for_family(family),
            family,
            capacity: capacity.max(1),
            entries: HashMap::new(),
            access_order: VecDeque::new(),
            tick: 0,
            disabled: false,
            metrics: CacheFamilyMetrics::default(),
        }
    }

    /// Returns the family this store accelerates.
    pub fn family(&self) -> CacheFamily {
        self.family
    }

    /// Returns the number of resident entries.
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    /// Returns whether this cache store is empty.
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    /// Disables this cache family (degraded mode).
    pub fn disable(&mut self) {
        self.disabled = true;
    }

    /// Re-enables this cache family.
    pub fn enable(&mut self) {
        self.disabled = false;
    }

    /// Returns whether this cache is currently in disabled/degraded mode.
    pub fn is_disabled(&self) -> bool {
        self.disabled
    }

    /// Evaluates admission and stores the entry if admitted.
    pub fn admit(
        &mut self,
        mut key: CacheKey,
        memory_ids: Vec<MemoryId>,
        request: CacheAdmissionRequest,
    ) -> CacheAdmissionOutcome {
        if self.disabled {
            return CacheAdmissionOutcome::rejected(
                CacheAdmissionDecision::BypassDisabled,
                CacheAdmissionReason::Disabled,
            );
        }
        if !request.policy_allowed {
            return CacheAdmissionOutcome::rejected(
                CacheAdmissionDecision::BypassPolicy,
                CacheAdmissionReason::PolicyDenied,
            );
        }
        if self.admission_policy.requires_owner_key && key.owner_key.is_none() {
            return CacheAdmissionOutcome::rejected(
                CacheAdmissionDecision::BypassPolicy,
                CacheAdmissionReason::PolicyDenied,
            );
        }
        if self.admission_policy.requires_request_shape_hash
            && request
                .request_shape_hash
                .or(key.request_shape_hash)
                .is_none()
        {
            return CacheAdmissionOutcome::rejected(
                CacheAdmissionDecision::BypassPolicy,
                CacheAdmissionReason::PolicyDenied,
            );
        }

        if key.request_shape_hash.is_none() {
            key.request_shape_hash = request.request_shape_hash;
        }
        if !self.admission_policy.allows_empty_payload && memory_ids.is_empty() {
            return CacheAdmissionOutcome::rejected(
                CacheAdmissionDecision::BypassPolicy,
                CacheAdmissionReason::EmptyPayload,
            );
        }

        let replaced_keys: Vec<CacheKey> = self
            .entries
            .keys()
            .filter(|existing| existing.same_lookup_identity(&key) && *existing != &key)
            .cloned()
            .collect();
        let mut evicted_existing = !replaced_keys.is_empty();
        for replaced_key in &replaced_keys {
            self.entries.remove(replaced_key);
            self.access_order
                .retain(|existing| existing != replaced_key);
        }

        if self.entries.len() >= self.capacity && !self.entries.contains_key(&key) {
            if !request.allow_capacity_eviction {
                return CacheAdmissionOutcome::rejected(
                    CacheAdmissionDecision::RejectCapacity,
                    CacheAdmissionReason::CapacityRejected,
                );
            }
            if let Some(evicted_key) = self.access_order.pop_front() {
                self.entries.remove(&evicted_key);
                evicted_existing = true;
            }
        }

        self.tick += 1;
        let entry = CacheEntry {
            key: key.clone(),
            memory_ids,
            stored_at_tick: self.tick,
        };
        self.entries.insert(key.clone(), entry);
        self.touch(&key);

        CacheAdmissionOutcome::admitted(
            if evicted_existing {
                CacheAdmissionReason::CapacityEvicted
            } else {
                CacheAdmissionReason::Accepted
            },
            evicted_existing,
        )
    }

    /// Looks up an entry with full generation-aware validation.
    ///
    /// Returns a `CacheLookupResult` with trace evidence for observability.
    pub fn lookup(
        &mut self,
        key: &CacheKey,
        current_generations: &CacheGenerationAnchors,
    ) -> CacheLookupResult {
        if self.disabled {
            self.metrics.bypass_count += 1;
            return CacheLookupResult {
                family: self.family,
                event: CacheEvent::Disabled,
                reason: None,
                warm_source: None,
                generation_status: GenerationStatus::Unknown,
                memory_ids: Vec::new(),
                candidates_after: 0,
            };
        }

        let matched_key = self
            .entries
            .keys()
            .find(|candidate| candidate.same_lookup_identity(key))
            .cloned();
        let Some(matched_key) = matched_key else {
            self.metrics.miss_count += 1;
            return CacheLookupResult {
                family: self.family,
                event: CacheEvent::Miss,
                reason: None,
                warm_source: None,
                generation_status: GenerationStatus::Unknown,
                memory_ids: Vec::new(),
                candidates_after: 0,
            };
        };
        let entry = self
            .entries
            .get(&matched_key)
            .expect("matched cache key must exist");

        // --- Generation-anchor validation ---
        let gen_status = validate_generations(&entry.key.generations, current_generations);
        match gen_status {
            GenerationStatus::Valid => {
                let ids = entry.memory_ids.clone();
                let count = ids.len();
                self.touch(&matched_key);
                self.metrics.hit_count += 1;
                CacheLookupResult {
                    family: self.family,
                    event: CacheEvent::Hit,
                    reason: None,
                    warm_source: Some(family_to_warm_source(self.family)),
                    generation_status: gen_status,
                    memory_ids: ids,
                    candidates_after: count,
                }
            }
            GenerationStatus::Stale => {
                self.metrics.stale_warning_count += 1;
                self.metrics.bypass_count += 1;
                CacheLookupResult {
                    family: self.family,
                    event: CacheEvent::StaleWarning,
                    reason: Some(CacheReason::GenerationAnchorMismatch),
                    warm_source: None,
                    generation_status: gen_status,
                    memory_ids: Vec::new(),
                    candidates_after: 0,
                }
            }
            GenerationStatus::VersionMismatched => {
                self.metrics.bypass_count += 1;
                CacheLookupResult {
                    family: self.family,
                    event: CacheEvent::Bypass,
                    reason: Some(CacheReason::VersionMismatch),
                    warm_source: None,
                    generation_status: gen_status,
                    memory_ids: Vec::new(),
                    candidates_after: 0,
                }
            }
            GenerationStatus::Unknown => {
                self.metrics.bypass_count += 1;
                CacheLookupResult {
                    family: self.family,
                    event: CacheEvent::Bypass,
                    reason: Some(CacheReason::GenerationAnchorMismatch),
                    warm_source: None,
                    generation_status: gen_status,
                    memory_ids: Vec::new(),
                    candidates_after: 0,
                }
            }
        }
    }

    /// Invalidates all entries for the given namespace.
    ///
    /// Returns the number of entries invalidated.
    pub fn invalidate_namespace(&mut self, namespace: &NamespaceId) -> usize {
        let keys_to_remove: Vec<CacheKey> = self
            .entries
            .keys()
            .filter(|k| &k.namespace == namespace)
            .cloned()
            .collect();
        let count = keys_to_remove.len();
        for key in &keys_to_remove {
            self.entries.remove(key);
            self.access_order.retain(|k| k != key);
        }
        self.metrics.invalidation_count += count as u64;
        count
    }

    /// Invalidates all entries whose generation anchors are behind the given
    /// current anchors. Returns the number of entries invalidated.
    pub fn invalidate_stale(&mut self, current: &CacheGenerationAnchors) -> usize {
        let stale_keys: Vec<CacheKey> = self
            .entries
            .keys()
            .filter(|k| validate_generations(&k.generations, current) != GenerationStatus::Valid)
            .cloned()
            .collect();
        let count = stale_keys.len();
        for key in &stale_keys {
            self.entries.remove(key);
            self.access_order.retain(|k| k != key);
        }
        self.metrics.invalidation_count += count as u64;
        count
    }

    /// Drops all entries—used for full rebuild or repair.
    pub fn clear(&mut self) {
        let count = self.entries.len() as u64;
        self.entries.clear();
        self.access_order.clear();
        self.metrics.invalidation_count += count;
    }

    fn touch(&mut self, key: &CacheKey) {
        self.access_order.retain(|k| k != key);
        self.access_order.push_back(key.clone());
    }
}

/// Validates entry generations against current expected generations.
fn validate_generations(
    entry: &CacheGenerationAnchors,
    current: &CacheGenerationAnchors,
) -> GenerationStatus {
    if entry == current {
        return GenerationStatus::Valid;
    }
    // Any downgrade in a generation => stale
    if entry.schema_generation < current.schema_generation
        || entry.policy_generation < current.policy_generation
        || entry.index_generation < current.index_generation
        || entry.embedding_generation < current.embedding_generation
        || entry.ranking_generation < current.ranking_generation
    {
        return GenerationStatus::Stale;
    }
    // Entry from a future generation (possible after rollback)
    GenerationStatus::VersionMismatched
}

/// Maps a cache family to the canonical warm-source label.
fn family_to_warm_source(family: CacheFamily) -> WarmSource {
    match family {
        CacheFamily::Tier1Item => WarmSource::Tier1ItemCache,
        CacheFamily::NegativeCache => WarmSource::NegativeCache,
        CacheFamily::ResultCache => WarmSource::ResultCache,
        CacheFamily::EntityNeighborhood => WarmSource::EntityNeighborhood,
        CacheFamily::SummaryCache => WarmSource::SummaryCache,
        CacheFamily::AnnProbeCache => WarmSource::AnnProbeCache,
        CacheFamily::PrefetchHints => WarmSource::PrefetchHints,
        CacheFamily::SessionWarmup => WarmSource::SessionWarmup,
        CacheFamily::GoalConditioned => WarmSource::GoalConditioned,
        CacheFamily::ColdStartMitigation => WarmSource::ColdStartMitigation,
    }
}

// ──────────────────────────────────────────────────────────────────────────────
// § 4  Bounded prefetch controller  (mb-23u.9.2)
// ──────────────────────────────────────────────────────────────────────────────

/// Prefetch hint submitted by session or task intent signals.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PrefetchHint {
    /// Which namespace this hint belongs to.
    pub namespace: NamespaceId,
    /// Why this hint was queued.
    pub trigger: PrefetchTrigger,
    /// Canonical warm-source provenance for this hint.
    pub warm_source: WarmSource,
    /// Predicted memory ids to preload.
    pub predicted_ids: Vec<MemoryId>,
    /// Monotonic tick when the hint was submitted.
    pub submitted_at_tick: u64,
}

/// Prefetch trigger sourced from session or task analysis.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PrefetchTrigger {
    /// Session-local recent access patterns.
    SessionRecency,
    /// Task or goal-level intent analysis.
    TaskIntent,
    /// Entity-neighborhood follow-up.
    EntityFollow,
    /// Cold-start bootstrap.
    ColdStart,
}

impl PrefetchTrigger {
    /// Returns the machine-readable trigger label.
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::SessionRecency => "session_recency",
            Self::TaskIntent => "task_intent",
            Self::EntityFollow => "entity_follow",
            Self::ColdStart => "cold_start",
        }
    }
}

/// Bypass reason for why a prefetch hint was dropped or ignored.
///
/// These labels stay aligned with the shared cache taxonomy so prefetch drops
/// preserve explicit namespace/policy versus intent/budget causes.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PrefetchBypassReason {
    /// Budget for speculative work is exhausted.
    BudgetExhausted,
    /// User or task intent changed since the hint was queued.
    IntentChanged,
    /// Namespace scope narrowed since the hint was queued.
    NamespaceNarrowed,
    /// Namespace scope widened since the hint was queued.
    NamespaceWidened,
    /// Policy scope change invalidated the hint.
    PolicyDenied,
    /// Prefetch system is disabled.
    Disabled,
}

impl PrefetchBypassReason {
    /// Returns the machine-readable bypass reason label.
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::BudgetExhausted => "budget_exhausted",
            Self::IntentChanged => "intent_changed",
            Self::NamespaceNarrowed => "namespace_narrowed",
            Self::NamespaceWidened => "namespace_widened",
            Self::PolicyDenied => "policy_denied",
            Self::Disabled => "disabled",
        }
    }
}

/// Bounded prefetch controller that manages speculative hint queues.
#[derive(Debug, Clone)]
pub struct PrefetchController {
    /// Maximum queued hints before oldest hints are dropped.
    capacity: usize,
    /// Queued prefetch hints in submission order.
    queue: VecDeque<PrefetchHint>,
    /// Current monotonic tick for temporal ordering.
    tick: u64,
    /// Whether the prefetch system is active.
    enabled: bool,
    /// Running observability counters.
    pub metrics: PrefetchMetrics,
}

/// Running counters for prefetch observability (mb-23u.9.3).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct PrefetchMetrics {
    pub hints_submitted: u64,
    pub hints_consumed: u64,
    pub hints_dropped: u64,
    pub dropped_budget_exhausted: u64,
    pub dropped_intent_changed: u64,
    pub dropped_scope_changed: u64,
    pub dropped_disabled: u64,
}

impl PrefetchController {
    /// Builds a bounded prefetch controller.
    pub fn new(capacity: usize) -> Self {
        Self {
            capacity: capacity.max(1),
            queue: VecDeque::new(),
            tick: 0,
            enabled: true,
            metrics: PrefetchMetrics::default(),
        }
    }

    fn record_drop(&mut self, reason: PrefetchBypassReason, count: u64) {
        self.metrics.hints_dropped += count;
        match reason {
            PrefetchBypassReason::BudgetExhausted => {
                self.metrics.dropped_budget_exhausted += count;
            }
            PrefetchBypassReason::IntentChanged => {
                self.metrics.dropped_intent_changed += count;
            }
            PrefetchBypassReason::NamespaceNarrowed
            | PrefetchBypassReason::NamespaceWidened
            | PrefetchBypassReason::PolicyDenied => {
                self.metrics.dropped_scope_changed += count;
            }
            PrefetchBypassReason::Disabled => {
                self.metrics.dropped_disabled += count;
            }
        }
    }

    /// Submits a speculative prefetch hint. Oldest hints are evicted when
    /// the queue exceeds capacity.
    pub fn submit_hint(
        &mut self,
        namespace: NamespaceId,
        trigger: PrefetchTrigger,
        predicted_ids: Vec<MemoryId>,
    ) -> bool {
        if !self.enabled {
            self.record_drop(PrefetchBypassReason::Disabled, 1);
            return false;
        }

        self.tick += 1;
        let hint = PrefetchHint {
            namespace,
            trigger,
            warm_source: WarmSource::PrefetchHints,
            predicted_ids,
            submitted_at_tick: self.tick,
        };

        if self.queue.len() >= self.capacity {
            self.queue.pop_front();
            self.record_drop(PrefetchBypassReason::BudgetExhausted, 1);
        }

        self.queue.push_back(hint);
        self.metrics.hints_submitted += 1;
        true
    }

    /// Consumes the next queued hint for the given namespace.
    /// Returns `None` if no matching hint is available.
    pub fn consume_hint(&mut self, namespace: &NamespaceId) -> Option<PrefetchHint> {
        self.consume_matching_hint(namespace, |_| true)
    }

    /// Consumes the next queued hint for the given namespace that satisfies one predicate.
    pub fn consume_matching_hint<F>(
        &mut self,
        namespace: &NamespaceId,
        mut predicate: F,
    ) -> Option<PrefetchHint>
    where
        F: FnMut(&PrefetchHint) -> bool,
    {
        if !self.enabled {
            return None;
        }
        let pos = self
            .queue
            .iter()
            .position(|hint| &hint.namespace == namespace && predicate(hint))?;
        let hint = self.queue.remove(pos)?;
        self.metrics.hints_consumed += 1;
        Some(hint)
    }

    /// Cancels all hints for the given namespace (intent-change or scope-change).
    /// Returns the number of hints dropped.
    pub fn cancel_namespace(
        &mut self,
        namespace: &NamespaceId,
        reason: PrefetchBypassReason,
    ) -> usize {
        let before = self.queue.len();
        self.queue.retain(|h| &h.namespace != namespace);
        let dropped = before - self.queue.len();
        self.record_drop(reason, dropped as u64);
        dropped
    }

    /// Cancels all queued hints.
    pub fn cancel_all(&mut self) {
        let dropped = self.queue.len() as u64;
        self.queue.clear();
        self.record_drop(PrefetchBypassReason::PolicyDenied, dropped);
    }

    /// Returns the current queue depth.
    pub fn queue_depth(&self) -> usize {
        self.queue.len()
    }

    /// Returns the configured queue capacity.
    pub fn capacity(&self) -> usize {
        self.capacity
    }

    /// Disables the prefetch system and discards queued speculative hints.
    pub fn disable(&mut self) {
        if self.enabled {
            let dropped = self.queue.len() as u64;
            self.queue.clear();
            if dropped > 0 {
                self.record_drop(PrefetchBypassReason::Disabled, dropped);
            }
            self.enabled = false;
        }
    }

    /// Re-enables the prefetch system.
    pub fn enable(&mut self) {
        self.enabled = true;
    }

    /// Returns whether the prefetch system is enabled.
    pub fn is_enabled(&self) -> bool {
        self.enabled
    }
}

// ──────────────────────────────────────────────────────────────────────────────
// § 5  Per-request cache metrics  (mb-23u.9.3)
// ──────────────────────────────────────────────────────────────────────────────

/// Per-family breakdown of request cache outcomes.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct CacheFamilyRequestBreakdown {
    pub hit_count: u32,
    pub miss_count: u32,
    pub bypass_count: u32,
    pub invalidation_count: u32,
    pub stale_warning_count: u32,
    pub disabled_count: u32,
    pub prefetch_drop_count: u32,
    pub session_expired_count: u32,
    pub repair_warmup_count: u32,
}

/// Per-request cache metrics populated in the common response envelope.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CacheRequestMetrics {
    /// Number of cache hits across all families for this request.
    pub cache_hit_count: u32,
    /// Number of cache misses across all families for this request.
    pub cache_miss_count: u32,
    /// Number of cache entries bypassed for this request.
    pub cache_bypass_count: u32,
    /// Number of cache entries invalidated for this request.
    pub cache_invalidation_count: u32,
    /// Number of prefetch hints consumed for this request.
    pub prefetch_used_count: u32,
    /// Number of prefetch hints canceled for this request.
    pub prefetch_dropped_count: u32,
    /// Number of times request escalated to a colder authoritative path.
    pub cold_fallback_count: u32,
    /// `true` if response was served while system was in degraded mode.
    pub degraded_mode_served: bool,
    /// Hits served from the Tier1 item cache.
    pub tier1_item_hit_count: u32,
    /// Hits served from the negative cache.
    pub negative_cache_hit_count: u32,
    /// Hits served from the result cache.
    pub result_cache_hit_count: u32,
    /// Hits served from the entity-neighborhood cache.
    pub entity_neighborhood_hit_count: u32,
    /// Hits served from the summary cache.
    pub summary_cache_hit_count: u32,
    /// Hits served from the ANN probe cache.
    pub ann_probe_hit_count: u32,
    /// Hits served from the prefetch queue.
    pub prefetch_hit_count: u32,
    /// Per-family event breakdown for all cache families participating in a request.
    pub per_family: [CacheFamilyRequestBreakdown; CacheFamily::ALL.len()],
}

impl Default for CacheRequestMetrics {
    fn default() -> Self {
        Self {
            cache_hit_count: 0,
            cache_miss_count: 0,
            cache_bypass_count: 0,
            cache_invalidation_count: 0,
            prefetch_used_count: 0,
            prefetch_dropped_count: 0,
            cold_fallback_count: 0,
            degraded_mode_served: false,
            tier1_item_hit_count: 0,
            negative_cache_hit_count: 0,
            result_cache_hit_count: 0,
            entity_neighborhood_hit_count: 0,
            summary_cache_hit_count: 0,
            ann_probe_hit_count: 0,
            prefetch_hit_count: 0,
            per_family: [CacheFamilyRequestBreakdown::default(); CacheFamily::ALL.len()],
        }
    }
}

impl CacheRequestMetrics {
    pub fn family_breakdown(&self, family: CacheFamily) -> &CacheFamilyRequestBreakdown {
        &self.per_family[family.metric_index()]
    }

    fn family_breakdown_mut(&mut self, family: CacheFamily) -> &mut CacheFamilyRequestBreakdown {
        &mut self.per_family[family.metric_index()]
    }

    /// Records a cache lookup result into per-request metrics.
    pub fn record_lookup(&mut self, result: &CacheLookupResult) {
        let family = result.family;
        match result.event {
            CacheEvent::Hit => {
                self.cache_hit_count += 1;
                self.family_breakdown_mut(family).hit_count += 1;
                match family {
                    CacheFamily::Tier1Item => self.tier1_item_hit_count += 1,
                    CacheFamily::NegativeCache => self.negative_cache_hit_count += 1,
                    CacheFamily::ResultCache => self.result_cache_hit_count += 1,
                    CacheFamily::EntityNeighborhood => self.entity_neighborhood_hit_count += 1,
                    CacheFamily::SummaryCache => self.summary_cache_hit_count += 1,
                    CacheFamily::AnnProbeCache => self.ann_probe_hit_count += 1,
                    CacheFamily::PrefetchHints => {
                        self.prefetch_hit_count += 1;
                        self.prefetch_used_count += 1;
                    }
                    CacheFamily::SessionWarmup
                    | CacheFamily::GoalConditioned
                    | CacheFamily::ColdStartMitigation => {}
                }
            }
            CacheEvent::Miss => {
                self.cache_miss_count += 1;
                self.family_breakdown_mut(family).miss_count += 1;
            }
            CacheEvent::Bypass => {
                self.cache_bypass_count += 1;
                self.family_breakdown_mut(family).bypass_count += 1;
            }
            CacheEvent::StaleWarning => {
                self.cache_bypass_count += 1;
                let breakdown = self.family_breakdown_mut(family);
                breakdown.bypass_count += 1;
                breakdown.stale_warning_count += 1;
            }
            CacheEvent::Disabled => {
                self.degraded_mode_served = true;
                self.cache_bypass_count += 1;
                let breakdown = self.family_breakdown_mut(family);
                breakdown.bypass_count += 1;
                breakdown.disabled_count += 1;
            }
            CacheEvent::Invalidation => {
                self.cache_invalidation_count += 1;
                self.family_breakdown_mut(family).invalidation_count += 1;
            }
            CacheEvent::PrefetchDrop => {
                self.prefetch_dropped_count += 1;
                self.family_breakdown_mut(family).prefetch_drop_count += 1;
            }
            CacheEvent::SessionExpired => {
                self.cache_bypass_count += 1;
                self.prefetch_dropped_count += 1;
                let breakdown = self.family_breakdown_mut(family);
                breakdown.bypass_count += 1;
                breakdown.session_expired_count += 1;
            }
            CacheEvent::RepairWarmup => {
                self.family_breakdown_mut(family).repair_warmup_count += 1;
            }
        }
    }

    /// Records that this request escalated from warm evaluation to a colder authoritative path.
    pub fn record_cold_fallback(&mut self) {
        self.cold_fallback_count += 1;
    }
}

/// Candidate count checkpoint for routing-trace integration (mb-23u.9.3).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct CacheCandidateCheckpoint {
    /// Candidates available before any cache lookup.
    pub pre_cache_candidates: u32,
    /// Candidates after Tier1 cache evaluation.
    pub post_tier1_candidates: Option<u32>,
    /// Candidates after Tier2 cache evaluation.
    pub post_tier2_candidates: Option<u32>,
    /// Candidates after ANN probe cache evaluation.
    pub post_ann_candidates: Option<u32>,
    /// Candidates added by prefetch queue for this request.
    pub prefetch_added_candidates: Option<u32>,
}

// ──────────────────────────────────────────────────────────────────────────────
// § 6  Cache trace stage  (mb-23u.9.3)
// ──────────────────────────────────────────────────────────────────────────────

/// Structured cache trace stage for explain / inspect integration.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CacheTraceStage {
    /// Stage name (e.g., `tier1_cache_eval`).
    pub stage: String,
    /// Which cache family participated.
    pub cache_family: CacheFamily,
    /// Event type.
    pub cache_event: CacheEvent,
    /// Reason for bypass or invalidation when applicable.
    pub cache_reason: Option<CacheReason>,
    /// Which warm source provided the cached entry.
    pub warm_source: Option<WarmSource>,
    /// Generation status validation result.
    pub generation_status: GenerationStatus,
    /// Candidate count before cache evaluation at this stage.
    pub candidates_before: u32,
    /// Candidate count after cache evaluation at this stage.
    pub candidates_after: u32,
}

impl CacheTraceStage {
    /// Builds a trace stage from a cache lookup result and surrounding context.
    pub fn from_lookup(
        stage_name: impl Into<String>,
        result: &CacheLookupResult,
        candidates_before: u32,
    ) -> Self {
        Self {
            stage: stage_name.into(),
            cache_family: result.family,
            cache_event: result.event,
            cache_reason: result.reason,
            warm_source: result.warm_source,
            generation_status: result.generation_status,
            candidates_before,
            candidates_after: result.candidates_after as u32,
        }
    }

    /// Converts this cache-native trace stage into the shared observability envelope.
    pub fn to_observability_trace(&self) -> CacheEvalTrace {
        CacheEvalTrace {
            cache_family: self.cache_family.into(),
            cache_event: self.cache_event.into(),
            outcome: self.cache_event.into(),
            cache_reason: self.cache_reason.map(Into::into),
            warm_source: self.warm_source.map(Into::into),
            generation_status: self.generation_status.into(),
            candidates_before: self.candidates_before as usize,
            candidates_after: self.candidates_after as usize,
            warm_reuse: self.warm_source.is_some() && matches!(self.cache_event, CacheEvent::Hit),
        }
    }
}

impl From<CacheFamily> for CacheFamilyLabel {
    fn from(value: CacheFamily) -> Self {
        match value {
            CacheFamily::Tier1Item => Self::Tier1Item,
            CacheFamily::NegativeCache => Self::NegativeCache,
            CacheFamily::ResultCache => Self::ResultCache,
            CacheFamily::EntityNeighborhood => Self::EntityNeighborhood,
            CacheFamily::SummaryCache => Self::SummaryCache,
            CacheFamily::AnnProbeCache => Self::AnnProbeCache,
            CacheFamily::PrefetchHints => Self::PrefetchHints,
            CacheFamily::SessionWarmup => Self::SessionWarmup,
            CacheFamily::GoalConditioned => Self::GoalConditioned,
            CacheFamily::ColdStartMitigation => Self::ColdStartMitigation,
        }
    }
}

impl From<CacheEvent> for CacheEventLabel {
    fn from(value: CacheEvent) -> Self {
        match value {
            CacheEvent::Hit => Self::Hit,
            CacheEvent::Miss => Self::Miss,
            CacheEvent::Bypass => Self::Bypass,
            CacheEvent::Invalidation => Self::Invalidation,
            CacheEvent::RepairWarmup => Self::RepairWarmup,
            CacheEvent::StaleWarning => Self::StaleWarning,
            CacheEvent::Disabled => Self::Disabled,
            CacheEvent::PrefetchDrop => Self::PrefetchDrop,
            CacheEvent::SessionExpired => Self::SessionExpired,
        }
    }
}

impl From<CacheEvent> for CacheLookupOutcome {
    fn from(value: CacheEvent) -> Self {
        match value {
            CacheEvent::Hit => Self::Hit,
            CacheEvent::Miss => Self::Miss,
            CacheEvent::Bypass => Self::Bypass,
            CacheEvent::Invalidation | CacheEvent::RepairWarmup | CacheEvent::PrefetchDrop => {
                Self::Bypass
            }
            CacheEvent::StaleWarning => Self::StaleWarning,
            CacheEvent::Disabled | CacheEvent::SessionExpired => Self::Disabled,
        }
    }
}

impl From<CacheReason> for CacheReasonLabel {
    fn from(value: CacheReason) -> Self {
        match value {
            CacheReason::OwnerBoundaryMismatch => Self::OwnerBoundaryMismatch,
            CacheReason::NamespaceMismatch => Self::NamespaceMismatch,
            CacheReason::PolicyDenied => Self::PolicyDenied,
            CacheReason::GenerationAnchorMismatch => Self::GenerationAnchorMismatch,
            CacheReason::VersionMismatch => Self::VersionMismatch,
            CacheReason::ScopeTooBroad => Self::ScopeTooBroad,
            CacheReason::RecordNotPresent => Self::RecordNotPresent,
            CacheReason::RepairIncomplete => Self::RepairIncomplete,
            CacheReason::PolicyChanged => Self::PolicyChanged,
            CacheReason::RedactionChanged => Self::RedactionChanged,
            CacheReason::SchemaChanged => Self::SchemaChanged,
            CacheReason::IndexChanged => Self::IndexChanged,
            CacheReason::EmbeddingChanged => Self::EmbeddingChanged,
            CacheReason::RankingChanged => Self::RankingChanged,
            CacheReason::IntentChanged => Self::IntentChanged,
            CacheReason::BudgetExhausted => Self::BudgetExhausted,
            CacheReason::NamespaceNarrowed => Self::NamespaceNarrowed,
            CacheReason::NamespaceWidened => Self::NamespaceWidened,
        }
    }
}

impl From<WarmSource> for WarmSourceLabel {
    fn from(value: WarmSource) -> Self {
        match value {
            WarmSource::Tier1ItemCache => Self::Tier1ItemCache,
            WarmSource::NegativeCache => Self::NegativeCache,
            WarmSource::ResultCache => Self::ResultCache,
            WarmSource::EntityNeighborhood => Self::EntityNeighborhood,
            WarmSource::SummaryCache => Self::SummaryCache,
            WarmSource::AnnProbeCache => Self::AnnProbeCache,
            WarmSource::PrefetchHints => Self::PrefetchHints,
            WarmSource::SessionWarmup => Self::SessionWarmup,
            WarmSource::GoalConditioned => Self::GoalConditioned,
            WarmSource::ColdStartMitigation => Self::ColdStartMitigation,
        }
    }
}

impl From<GenerationStatus> for GenerationStatusLabel {
    fn from(value: GenerationStatus) -> Self {
        match value {
            GenerationStatus::Valid => Self::Valid,
            GenerationStatus::Stale => Self::Stale,
            GenerationStatus::VersionMismatched => Self::VersionMismatched,
            GenerationStatus::Unknown => Self::Unknown,
        }
    }
}

// ──────────────────────────────────────────────────────────────────────────────
// § 7  Invalidation hooks  (mb-23u.9.4)
// ──────────────────────────────────────────────────────────────────────────────

/// Invalidation trigger for cache repair hooks.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InvalidationTrigger {
    /// A durable memory was mutated (write, forget, supersession).
    MemoryMutation,
    /// Policy rules changed.
    PolicyChange,
    /// Redaction rules updated.
    RedactionChange,
    /// Schema generation changed (migration).
    SchemaChange,
    /// Index generation changed (rebuild/repair).
    IndexChange,
    /// Embedding model generation changed.
    EmbeddingChange,
    /// Ranking/reranker model version changed.
    RankingChange,
    /// Repair or rebuild started (broad invalidation).
    RepairStarted,
    /// Namespace change affecting visibility scope.
    NamespaceChange,
}

impl InvalidationTrigger {
    /// Returns the machine-readable trigger label.
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::MemoryMutation => "memory_mutation",
            Self::PolicyChange => "policy_change",
            Self::RedactionChange => "redaction_change",
            Self::SchemaChange => "schema_change",
            Self::IndexChange => "index_change",
            Self::EmbeddingChange => "embedding_change",
            Self::RankingChange => "ranking_change",
            Self::RepairStarted => "repair_started",
            Self::NamespaceChange => "namespace_change",
        }
    }
}

/// Traceable invalidation or repair event for one cache family.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CacheMaintenanceEvent {
    /// Which cache family emitted the maintenance event.
    pub family: CacheFamily,
    /// Machine-readable cache event label.
    pub event: CacheEvent,
    /// Explicit reason for invalidation or repair when applicable.
    pub reason: Option<CacheReason>,
    /// Warm source when repair repopulates a family.
    pub warm_source: Option<WarmSource>,
    /// Generation validation state attached to the event.
    pub generation_status: GenerationStatus,
    /// Number of entries or hints affected for this family.
    pub entries_affected: usize,
}

/// Outcome of a cache invalidation or repair operation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InvalidationOutcome {
    /// Trigger that caused the invalidation.
    pub trigger: InvalidationTrigger,
    /// Which families were affected.
    pub families_affected: Vec<CacheFamily>,
    /// Total entries invalidated across all families.
    pub entries_invalidated: usize,
    /// Per-family events suitable for inspect, explain, and audit surfaces.
    pub maintenance_events: Vec<CacheMaintenanceEvent>,
}

/// Verify-only hooks for checking cache warm-state parity without rewriting warm entries.
pub const CACHE_WARM_STATE_VERIFY_HOOKS: [&str; 2] = [
    "snapshot_current_generation_anchors",
    "verify_generation_anchor_report",
];

/// Full repair hooks for rebuilding cache warm state after truth or generation drift.
pub const CACHE_WARM_STATE_REBUILD_HOOKS: [&str; 8] = [
    "snapshot_current_generation_anchors",
    "invalidate_cache_families",
    "drop_prefetch_hints",
    "rebuild_tier1_item_cache",
    "rebuild_result_cache",
    "rebuild_summary_cache",
    "rebuild_ann_probe_cache",
    "verify_generation_anchor_report",
];

/// Families repopulated during cache warm-state repair after broad invalidation.
pub const CACHE_WARM_STATE_WARMUP_FAMILIES: [CacheFamily; 4] = [
    CacheFamily::Tier1Item,
    CacheFamily::ResultCache,
    CacheFamily::SummaryCache,
    CacheFamily::AnnProbeCache,
];

// ──────────────────────────────────────────────────────────────────────────────
// § 8  Composite cache manager  (mb-23u.9 umbrella)
// ──────────────────────────────────────────────────────────────────────────────

/// Composite cache manager owning all cache families and the prefetch controller.
///
/// This is the top-level entry point for all cache operations on the
/// retrieval hot path. It holds one `BoundedCacheStore` per family and
/// a single `PrefetchController`.
#[derive(Debug, Clone)]
pub struct CacheManager {
    pub tier1_item: BoundedCacheStore,
    pub negative: BoundedCacheStore,
    pub result: BoundedCacheStore,
    pub entity_neighborhood: BoundedCacheStore,
    pub summary: BoundedCacheStore,
    pub ann_probe: BoundedCacheStore,
    pub session_warmup: BoundedCacheStore,
    pub goal_conditioned: BoundedCacheStore,
    pub cold_start: BoundedCacheStore,
    /// Bounded prefetch hint controller.
    pub prefetch: PrefetchController,
}

impl CacheManager {
    /// Canonical cache hot-path lookup order.
    pub const HOT_PATH_LOOKUP_ORDER: [HotPathLookupStage; 10] = [
        HotPathLookupStage::PrefetchHints,
        HotPathLookupStage::SessionWarmup,
        HotPathLookupStage::Tier1Item,
        HotPathLookupStage::NegativeCache,
        HotPathLookupStage::ResultCache,
        HotPathLookupStage::EntityNeighborhood,
        HotPathLookupStage::SummaryCache,
        HotPathLookupStage::AnnProbeCache,
        HotPathLookupStage::GoalConditioned,
        HotPathLookupStage::ColdStartMitigation,
    ];

    /// Builds a cache manager with the given per-family capacity and prefetch budget.
    pub fn new(per_family_capacity: usize, prefetch_capacity: usize) -> Self {
        Self {
            tier1_item: BoundedCacheStore::new(CacheFamily::Tier1Item, per_family_capacity),
            negative: BoundedCacheStore::new(CacheFamily::NegativeCache, per_family_capacity),
            result: BoundedCacheStore::new(CacheFamily::ResultCache, per_family_capacity),
            entity_neighborhood: BoundedCacheStore::new(
                CacheFamily::EntityNeighborhood,
                per_family_capacity,
            ),
            summary: BoundedCacheStore::new(CacheFamily::SummaryCache, per_family_capacity),
            ann_probe: BoundedCacheStore::new(CacheFamily::AnnProbeCache, per_family_capacity),
            session_warmup: BoundedCacheStore::new(CacheFamily::SessionWarmup, per_family_capacity),
            goal_conditioned: BoundedCacheStore::new(
                CacheFamily::GoalConditioned,
                per_family_capacity,
            ),
            cold_start: BoundedCacheStore::new(
                CacheFamily::ColdStartMitigation,
                per_family_capacity,
            ),
            prefetch: PrefetchController::new(prefetch_capacity),
        }
    }

    /// Returns the canonical cache hot-path lookup order.
    pub fn hot_path_lookup_order(&self) -> &'static [HotPathLookupStage] {
        &Self::HOT_PATH_LOOKUP_ORDER
    }

    /// Returns the store for a given cache family when that family is backed by a cache store.
    pub fn store_for(&mut self, family: CacheFamily) -> Option<&mut BoundedCacheStore> {
        match family {
            CacheFamily::Tier1Item => Some(&mut self.tier1_item),
            CacheFamily::NegativeCache => Some(&mut self.negative),
            CacheFamily::ResultCache => Some(&mut self.result),
            CacheFamily::EntityNeighborhood => Some(&mut self.entity_neighborhood),
            CacheFamily::SummaryCache => Some(&mut self.summary),
            CacheFamily::AnnProbeCache => Some(&mut self.ann_probe),
            CacheFamily::PrefetchHints => None,
            CacheFamily::SessionWarmup => Some(&mut self.session_warmup),
            CacheFamily::GoalConditioned => Some(&mut self.goal_conditioned),
            CacheFamily::ColdStartMitigation => Some(&mut self.cold_start),
        }
    }

    /// Handles a broad invalidation trigger across all affected families.
    pub fn handle_invalidation(
        &mut self,
        trigger: InvalidationTrigger,
        namespace: &NamespaceId,
    ) -> InvalidationOutcome {
        let mut total = 0usize;
        let mut affected = Vec::new();
        let mut maintenance_events = Vec::new();
        let namespace_scoped = matches!(
            trigger,
            InvalidationTrigger::MemoryMutation
                | InvalidationTrigger::PolicyChange
                | InvalidationTrigger::RedactionChange
        );
        let reason = match trigger {
            InvalidationTrigger::MemoryMutation => Some(CacheReason::RecordNotPresent),
            InvalidationTrigger::PolicyChange => Some(CacheReason::PolicyChanged),
            InvalidationTrigger::RedactionChange => Some(CacheReason::RedactionChanged),
            InvalidationTrigger::SchemaChange => Some(CacheReason::SchemaChanged),
            InvalidationTrigger::IndexChange => Some(CacheReason::IndexChanged),
            InvalidationTrigger::EmbeddingChange => Some(CacheReason::EmbeddingChanged),
            InvalidationTrigger::RankingChange => Some(CacheReason::RankingChanged),
            InvalidationTrigger::RepairStarted => Some(CacheReason::RepairIncomplete),
            InvalidationTrigger::NamespaceChange => Some(CacheReason::NamespaceNarrowed),
        };

        let families: &mut [&mut BoundedCacheStore] = &mut [
            &mut self.tier1_item,
            &mut self.negative,
            &mut self.result,
            &mut self.entity_neighborhood,
            &mut self.summary,
            &mut self.ann_probe,
            &mut self.session_warmup,
            &mut self.goal_conditioned,
            &mut self.cold_start,
        ];

        for store in families.iter_mut() {
            let count = if namespace_scoped {
                store.invalidate_namespace(namespace)
            } else {
                let count = store.len();
                store.clear();
                count
            };
            if count > 0 {
                let family = store.family();
                affected.push(family);
                total += count;
                maintenance_events.push(CacheMaintenanceEvent {
                    family,
                    event: CacheEvent::Invalidation,
                    reason,
                    warm_source: None,
                    generation_status: GenerationStatus::Unknown,
                    entries_affected: count,
                });
            }
        }

        let (prefetch_reason, prefetch_event) = (reason, CacheEvent::PrefetchDrop);
        let prefetch_dropped = if namespace_scoped {
            let bypass_reason = match trigger {
                InvalidationTrigger::MemoryMutation => PrefetchBypassReason::IntentChanged,
                InvalidationTrigger::PolicyChange => PrefetchBypassReason::PolicyDenied,
                InvalidationTrigger::RedactionChange => PrefetchBypassReason::PolicyDenied,
                _ => PrefetchBypassReason::NamespaceNarrowed,
            };
            self.prefetch.cancel_namespace(namespace, bypass_reason)
        } else {
            let dropped = self.prefetch.queue_depth();
            self.prefetch.cancel_all();
            dropped
        };
        if prefetch_dropped > 0 {
            affected.push(CacheFamily::PrefetchHints);
            total += prefetch_dropped;
            maintenance_events.push(CacheMaintenanceEvent {
                family: CacheFamily::PrefetchHints,
                event: prefetch_event,
                reason: prefetch_reason,
                warm_source: Some(WarmSource::PrefetchHints),
                generation_status: GenerationStatus::Unknown,
                entries_affected: prefetch_dropped,
            });
        }

        InvalidationOutcome {
            trigger,
            families_affected: affected,
            entries_invalidated: total,
            maintenance_events,
        }
    }

    /// Records per-family repair warmup events after a bounded rebuild repopulates warm state.
    pub fn repair_warmup(
        &mut self,
        families: &[CacheFamily],
        entries_per_family: usize,
    ) -> Vec<CacheMaintenanceEvent> {
        let mut events = Vec::new();
        for family in families {
            if *family == CacheFamily::PrefetchHints || entries_per_family == 0 {
                continue;
            }
            events.push(CacheMaintenanceEvent {
                family: *family,
                event: CacheEvent::RepairWarmup,
                reason: None,
                warm_source: Some(family_to_warm_source(*family)),
                generation_status: GenerationStatus::Valid,
                entries_affected: entries_per_family,
            });
        }
        events
    }

    /// Clears all cache families and prefetch queue.
    pub fn clear_all(&mut self) {
        self.tier1_item.clear();
        self.negative.clear();
        self.result.clear();
        self.entity_neighborhood.clear();
        self.summary.clear();
        self.ann_probe.clear();
        self.session_warmup.clear();
        self.goal_conditioned.clear();
        self.cold_start.clear();
        self.prefetch.cancel_all();
    }
}

// ──────────────────────────────────────────────────────────────────────────────
// § 9  Tests  (mb-23u.9.4 regression coverage)
// ──────────────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    fn ns(name: &str) -> NamespaceId {
        NamespaceId::new(name).unwrap()
    }

    fn gen_anchors(v: u64) -> CacheGenerationAnchors {
        CacheGenerationAnchors {
            schema_generation: v,
            policy_generation: v,
            index_generation: v,
            embedding_generation: v,
            ranking_generation: v,
        }
    }

    fn make_key(family: CacheFamily, ns_str: &str, item: u64) -> CacheKey {
        let owner_key = matches!(
            family,
            CacheFamily::PrefetchHints | CacheFamily::SessionWarmup | CacheFamily::GoalConditioned
        )
        .then_some(item + 10_000);
        let request_shape_hash = matches!(
            family,
            CacheFamily::NegativeCache | CacheFamily::ResultCache | CacheFamily::AnnProbeCache
        )
        .then_some(item + 20_000);

        CacheKey {
            family,
            namespace: ns(ns_str),
            workspace_key: None,
            owner_key,
            request_shape_hash,
            item_key: item,
            generations: gen_anchors(1),
        }
    }

    // ── § 9.1  Family contract and hit/miss visibility ───────────────────

    #[test]
    fn cache_family_policies_match_owner_boundary_contract() {
        let request_local = CacheAdmissionPolicy::for_family(CacheFamily::ResultCache);
        assert!(request_local.requires_request_shape_hash);
        assert!(!request_local.requires_owner_key);
        assert!(!request_local.allows_empty_payload);

        let negative = CacheAdmissionPolicy::for_family(CacheFamily::NegativeCache);
        assert!(negative.requires_request_shape_hash);
        assert!(negative.allows_empty_payload);
        assert!(!negative.requires_owner_key);

        let session_local = CacheAdmissionPolicy::for_family(CacheFamily::SessionWarmup);
        assert!(session_local.requires_owner_key);
        assert!(!session_local.requires_request_shape_hash);
        assert!(!session_local.allows_empty_payload);
    }

    #[test]
    fn request_local_family_admission_uses_bound_request_shape_hash() {
        let mut store = BoundedCacheStore::new(CacheFamily::ResultCache, 10);
        let key = make_key(CacheFamily::ResultCache, "team.alpha", 42);

        let outcome = store.admit(
            key.clone(),
            vec![MemoryId(1)],
            CacheAdmissionRequest::default(),
        );

        assert_eq!(outcome.decision, CacheAdmissionDecision::Admit);
        let stored = store
            .entries
            .values()
            .next()
            .expect("entry should be stored");
        assert_eq!(stored.key.request_shape_hash, key.request_shape_hash);
    }

    #[test]
    fn request_local_family_rejects_missing_request_shape_hash() {
        let mut store = BoundedCacheStore::new(CacheFamily::ResultCache, 10);
        let mut key = make_key(CacheFamily::ResultCache, "team.alpha", 42);
        key.request_shape_hash = None;

        let outcome = store.admit(key, vec![MemoryId(1)], CacheAdmissionRequest::default());

        assert_eq!(outcome.decision, CacheAdmissionDecision::BypassPolicy);
        assert_eq!(outcome.reason, CacheAdmissionReason::PolicyDenied);
        assert!(store.is_empty());
    }

    #[test]
    fn hot_path_lookup_order_matches_canonical_contract() {
        let manager = CacheManager::new(10, 5);

        assert_eq!(
            manager.hot_path_lookup_order(),
            &[
                HotPathLookupStage::PrefetchHints,
                HotPathLookupStage::SessionWarmup,
                HotPathLookupStage::Tier1Item,
                HotPathLookupStage::NegativeCache,
                HotPathLookupStage::ResultCache,
                HotPathLookupStage::EntityNeighborhood,
                HotPathLookupStage::SummaryCache,
                HotPathLookupStage::AnnProbeCache,
                HotPathLookupStage::GoalConditioned,
                HotPathLookupStage::ColdStartMitigation,
            ],
        );
    }

    #[test]
    fn fresh_cache_hit_returns_hit_event_and_increments_metric() {
        let mut store = BoundedCacheStore::new(CacheFamily::Tier1Item, 10);
        let key = make_key(CacheFamily::Tier1Item, "team.alpha", 42);
        store.admit(
            key.clone(),
            vec![MemoryId(1), MemoryId(2)],
            CacheAdmissionRequest::default(),
        );

        let result = store.lookup(&key, &gen_anchors(1));

        assert_eq!(result.event, CacheEvent::Hit);
        assert_eq!(result.memory_ids.len(), 2);
        assert_eq!(result.warm_source, Some(WarmSource::Tier1ItemCache));
        assert_eq!(result.generation_status, GenerationStatus::Valid);
        assert_eq!(store.metrics.hit_count, 1);
        assert_eq!(store.metrics.miss_count, 0);
    }

    #[test]
    fn cache_miss_returns_miss_event_and_increments_metric() {
        let mut store = BoundedCacheStore::new(CacheFamily::ResultCache, 10);
        let key = make_key(CacheFamily::ResultCache, "team.alpha", 99);

        let result = store.lookup(&key, &gen_anchors(1));

        assert_eq!(result.event, CacheEvent::Miss);
        assert!(result.memory_ids.is_empty());
        assert_eq!(store.metrics.miss_count, 1);
    }

    // ── § 9.2  Stale-result bypass ───────────────────────────────────────

    #[test]
    fn stale_generation_returns_stale_warning_not_silent_miss() {
        let mut store = BoundedCacheStore::new(CacheFamily::AnnProbeCache, 10);
        let key = make_key(CacheFamily::AnnProbeCache, "team.alpha", 1);
        store.admit(
            key.clone(),
            vec![MemoryId(5)],
            CacheAdmissionRequest::default(),
        );

        // Advance current generations beyond what is stored
        let result = store.lookup(&key, &gen_anchors(2));

        assert_eq!(result.event, CacheEvent::StaleWarning);
        assert_eq!(result.reason, Some(CacheReason::GenerationAnchorMismatch));
        assert!(result.memory_ids.is_empty());
        assert_eq!(store.metrics.stale_warning_count, 1);
    }

    #[test]
    fn replacing_same_lookup_identity_removes_older_generation_entry() {
        let mut store = BoundedCacheStore::new(CacheFamily::ResultCache, 10);
        let stale_key = CacheKey {
            generations: gen_anchors(1),
            ..make_key(CacheFamily::ResultCache, "team.alpha", 7)
        };
        let fresh_key = CacheKey {
            generations: gen_anchors(2),
            ..make_key(CacheFamily::ResultCache, "team.alpha", 7)
        };

        store.admit(
            stale_key,
            vec![MemoryId(1)],
            CacheAdmissionRequest::default(),
        );
        let outcome = store.admit(
            fresh_key.clone(),
            vec![MemoryId(2)],
            CacheAdmissionRequest::default(),
        );

        assert_eq!(store.len(), 1);
        assert!(outcome.evicted_existing);

        let lookup_key = CacheKey {
            generations: gen_anchors(999),
            ..fresh_key
        };
        let result = store.lookup(&lookup_key, &gen_anchors(2));
        assert_eq!(result.event, CacheEvent::Hit);
        assert_eq!(result.memory_ids, vec![MemoryId(2)]);
    }

    // ── § 9.3  Degraded mode serving ─────────────────────────────────────

    #[test]
    fn disabled_cache_returns_disabled_event() {
        let mut store = BoundedCacheStore::new(CacheFamily::NegativeCache, 10);
        store.disable();
        let key = make_key(CacheFamily::NegativeCache, "team.alpha", 1);

        let result = store.lookup(&key, &gen_anchors(1));

        assert_eq!(result.event, CacheEvent::Disabled);
        assert_eq!(store.metrics.bypass_count, 1);

        let mut req_metrics = CacheRequestMetrics::default();
        req_metrics.record_lookup(&result);
        assert!(req_metrics.degraded_mode_served);
    }

    // ── § 9.4  Candidate count preservation ──────────────────────────────

    #[test]
    fn cache_trace_stage_preserves_candidate_counts() {
        let result = CacheLookupResult {
            family: CacheFamily::Tier1Item,
            event: CacheEvent::Hit,
            reason: None,
            warm_source: Some(WarmSource::Tier1ItemCache),
            generation_status: GenerationStatus::Valid,
            memory_ids: vec![MemoryId(1), MemoryId(2), MemoryId(3)],
            candidates_after: 3,
        };

        let stage = CacheTraceStage::from_lookup("tier1_cache_eval", &result, 10);

        assert_eq!(stage.candidates_before, 10);
        assert_eq!(stage.candidates_after, 3);
        assert_eq!(stage.cache_family, CacheFamily::Tier1Item);
    }

    // ── § 9.5  Prefetch transparency ─────────────────────────────────────

    #[test]
    fn prefetch_submit_and_consume_tracks_metrics() {
        let mut pf = PrefetchController::new(5);
        assert!(pf.submit_hint(
            ns("team.alpha"),
            PrefetchTrigger::SessionRecency,
            vec![MemoryId(1)]
        ));
        assert!(pf.submit_hint(
            ns("team.alpha"),
            PrefetchTrigger::TaskIntent,
            vec![MemoryId(2)]
        ));

        let hint = pf.consume_hint(&ns("team.alpha")).expect("matching hint");
        assert_eq!(hint.trigger, PrefetchTrigger::SessionRecency);
        assert_eq!(hint.warm_source, WarmSource::PrefetchHints);
        assert_eq!(pf.metrics.hints_submitted, 2);
        assert_eq!(pf.metrics.hints_consumed, 1);
    }

    #[test]
    fn prefetch_drops_oldest_when_over_capacity() {
        let mut pf = PrefetchController::new(2);
        pf.submit_hint(
            ns("team.alpha"),
            PrefetchTrigger::SessionRecency,
            vec![MemoryId(1)],
        );
        pf.submit_hint(
            ns("team.alpha"),
            PrefetchTrigger::TaskIntent,
            vec![MemoryId(2)],
        );
        pf.submit_hint(
            ns("team.alpha"),
            PrefetchTrigger::EntityFollow,
            vec![MemoryId(3)],
        );

        assert_eq!(pf.queue_depth(), 2);
        assert_eq!(pf.metrics.hints_dropped, 1);
        assert_eq!(pf.metrics.dropped_budget_exhausted, 1);
    }

    #[test]
    fn disabled_prefetch_drops_all_submissions() {
        let mut pf = PrefetchController::new(5);
        pf.disable();
        assert!(!pf.submit_hint(
            ns("team.alpha"),
            PrefetchTrigger::SessionRecency,
            vec![MemoryId(1)]
        ));
        assert_eq!(pf.metrics.hints_dropped, 1);
        assert_eq!(pf.metrics.dropped_disabled, 1);
    }

    #[test]
    fn disable_prefetch_clears_queued_hints_and_counts_them_disabled() {
        let mut pf = PrefetchController::new(5);
        pf.submit_hint(
            ns("team.alpha"),
            PrefetchTrigger::SessionRecency,
            vec![MemoryId(1)],
        );
        pf.submit_hint(
            ns("team.beta"),
            PrefetchTrigger::TaskIntent,
            vec![MemoryId(2)],
        );

        pf.disable();

        assert_eq!(pf.queue_depth(), 0);
        assert_eq!(pf.metrics.hints_dropped, 2);
        assert_eq!(pf.metrics.dropped_disabled, 2);

        pf.enable();
        assert!(pf.consume_hint(&ns("team.alpha")).is_none());
        assert!(pf.consume_hint(&ns("team.beta")).is_none());
    }

    #[test]
    fn cancel_namespace_tracks_namespace_narrowed_reason() {
        let mut pf = PrefetchController::new(5);
        pf.submit_hint(
            ns("team.alpha"),
            PrefetchTrigger::SessionRecency,
            vec![MemoryId(1)],
        );
        pf.submit_hint(
            ns("team.alpha"),
            PrefetchTrigger::TaskIntent,
            vec![MemoryId(2)],
        );
        pf.submit_hint(
            ns("team.beta"),
            PrefetchTrigger::EntityFollow,
            vec![MemoryId(3)],
        );

        let dropped =
            pf.cancel_namespace(&ns("team.alpha"), PrefetchBypassReason::NamespaceNarrowed);

        assert_eq!(dropped, 2);
        assert_eq!(pf.queue_depth(), 1);
        assert_eq!(pf.metrics.hints_dropped, 2);
        assert_eq!(pf.metrics.dropped_scope_changed, 2);
    }

    #[test]
    fn cancel_namespace_tracks_namespace_widened_reason() {
        let mut pf = PrefetchController::new(5);
        pf.submit_hint(
            ns("team.alpha"),
            PrefetchTrigger::TaskIntent,
            vec![MemoryId(1)],
        );

        let dropped =
            pf.cancel_namespace(&ns("team.alpha"), PrefetchBypassReason::NamespaceWidened);

        assert_eq!(dropped, 1);
        assert_eq!(pf.metrics.hints_dropped, 1);
        assert_eq!(pf.metrics.dropped_scope_changed, 1);
    }

    #[test]
    fn cancel_namespace_tracks_intent_changed_reason() {
        let mut pf = PrefetchController::new(5);
        pf.submit_hint(
            ns("team.alpha"),
            PrefetchTrigger::TaskIntent,
            vec![MemoryId(1)],
        );

        let dropped = pf.cancel_namespace(&ns("team.alpha"), PrefetchBypassReason::IntentChanged);

        assert_eq!(dropped, 1);
        assert_eq!(pf.metrics.dropped_intent_changed, 1);
    }

    #[test]
    fn cancel_all_tracks_policy_denied_reason() {
        let mut pf = PrefetchController::new(5);
        pf.submit_hint(
            ns("team.alpha"),
            PrefetchTrigger::SessionRecency,
            vec![MemoryId(1)],
        );
        pf.submit_hint(
            ns("team.beta"),
            PrefetchTrigger::ColdStart,
            vec![MemoryId(2)],
        );

        pf.cancel_all();

        assert_eq!(pf.queue_depth(), 0);
        assert_eq!(pf.metrics.hints_dropped, 2);
        assert_eq!(pf.metrics.dropped_scope_changed, 2);
    }

    // ── § 9.6  Namespace invalidation ────────────────────────────────────

    #[test]
    fn invalidate_namespace_clears_matching_entries_only() {
        let mut store = BoundedCacheStore::new(CacheFamily::ResultCache, 10);
        store.admit(
            make_key(CacheFamily::ResultCache, "team.alpha", 1),
            vec![MemoryId(10)],
            CacheAdmissionRequest {
                request_shape_hash: Some(1),
                ..CacheAdmissionRequest::default()
            },
        );
        store.admit(
            make_key(CacheFamily::ResultCache, "team.beta", 2),
            vec![MemoryId(20)],
            CacheAdmissionRequest {
                request_shape_hash: Some(2),
                ..CacheAdmissionRequest::default()
            },
        );

        let count = store.invalidate_namespace(&ns("team.alpha"));
        assert_eq!(count, 1);
        assert_eq!(store.len(), 1);
        assert_eq!(store.metrics.invalidation_count, 1);
    }

    #[test]
    fn cache_trace_stage_converts_to_shared_observability_trace() {
        let result = CacheLookupResult {
            family: CacheFamily::ResultCache,
            event: CacheEvent::Hit,
            reason: None,
            warm_source: Some(WarmSource::ResultCache),
            generation_status: GenerationStatus::Valid,
            memory_ids: vec![MemoryId(10), MemoryId(20)],
            candidates_after: 2,
        };

        let trace =
            CacheTraceStage::from_lookup("result_cache_eval", &result, 6).to_observability_trace();

        assert_eq!(trace.cache_family.as_str(), "result_cache");
        assert_eq!(trace.cache_event.as_str(), "hit");
        assert_eq!(trace.outcome.as_str(), "hit");
        assert_eq!(
            trace.warm_source.map(|source| source.as_str()),
            Some("result_cache")
        );
        assert_eq!(trace.generation_status.as_str(), "valid");
        assert_eq!(trace.candidates_before, 6);
        assert_eq!(trace.candidates_after, 2);
        assert!(trace.warm_reuse);
    }

    #[test]
    fn maintenance_and_session_events_map_to_observability_outcomes() {
        let invalidation_trace = CacheTraceStage {
            stage: "repair_invalidation".into(),
            cache_family: CacheFamily::SummaryCache,
            cache_event: CacheEvent::Invalidation,
            cache_reason: Some(CacheReason::SchemaChanged),
            warm_source: None,
            generation_status: GenerationStatus::Unknown,
            candidates_before: 3,
            candidates_after: 0,
        }
        .to_observability_trace();
        assert_eq!(invalidation_trace.cache_event.as_str(), "invalidation");
        assert_eq!(invalidation_trace.outcome.as_str(), "bypass");
        assert_eq!(
            invalidation_trace
                .cache_reason
                .map(|reason| reason.as_str()),
            Some("schema_changed")
        );
        assert!(!invalidation_trace.warm_reuse);

        let session_expired_trace = CacheTraceStage {
            stage: "session_expired".into(),
            cache_family: CacheFamily::PrefetchHints,
            cache_event: CacheEvent::SessionExpired,
            cache_reason: Some(CacheReason::IntentChanged),
            warm_source: None,
            generation_status: GenerationStatus::Unknown,
            candidates_before: 2,
            candidates_after: 0,
        }
        .to_observability_trace();
        assert_eq!(
            session_expired_trace.cache_event.as_str(),
            "session_expired"
        );
        assert_eq!(session_expired_trace.outcome.as_str(), "disabled");
        assert_eq!(
            session_expired_trace
                .cache_reason
                .map(|reason| reason.as_str()),
            Some("intent_changed")
        );
    }

    // ── § 9.7  LRU eviction ─────────────────────────────────────────────

    #[test]
    fn lru_eviction_removes_oldest_on_capacity_overflow() {
        let mut store = BoundedCacheStore::new(CacheFamily::Tier1Item, 2);
        store.admit(
            make_key(CacheFamily::Tier1Item, "ns", 1),
            vec![MemoryId(1)],
            CacheAdmissionRequest::default(),
        );
        store.admit(
            make_key(CacheFamily::Tier1Item, "ns", 2),
            vec![MemoryId(2)],
            CacheAdmissionRequest::default(),
        );
        store.admit(
            make_key(CacheFamily::Tier1Item, "ns", 3),
            vec![MemoryId(3)],
            CacheAdmissionRequest::default(),
        );

        // Oldest (key=1) should have been evicted.
        let miss = store.lookup(&make_key(CacheFamily::Tier1Item, "ns", 1), &gen_anchors(1));
        assert_eq!(miss.event, CacheEvent::Miss);

        let hit = store.lookup(&make_key(CacheFamily::Tier1Item, "ns", 3), &gen_anchors(1));
        assert_eq!(hit.event, CacheEvent::Hit);
    }

    // ── § 9.8  Composite manager ─────────────────────────────────────────

    #[test]
    fn cache_manager_invalidation_cascades_across_families() {
        let mut mgr = CacheManager::new(10, 5);
        mgr.tier1_item.admit(
            make_key(CacheFamily::Tier1Item, "team.alpha", 1),
            vec![MemoryId(1)],
            CacheAdmissionRequest::default(),
        );
        mgr.negative.admit(
            make_key(CacheFamily::NegativeCache, "team.alpha", 2),
            vec![],
            CacheAdmissionRequest {
                request_shape_hash: Some(2),
                ..CacheAdmissionRequest::default()
            },
        );
        mgr.result.admit(
            make_key(CacheFamily::ResultCache, "team.beta", 3),
            vec![MemoryId(3)],
            CacheAdmissionRequest {
                request_shape_hash: Some(3),
                ..CacheAdmissionRequest::default()
            },
        );
        mgr.prefetch.submit_hint(
            ns("team.alpha"),
            PrefetchTrigger::SessionRecency,
            vec![MemoryId(10)],
        );

        let outcome = mgr.handle_invalidation(InvalidationTrigger::PolicyChange, &ns("team.alpha"));

        assert_eq!(outcome.trigger, InvalidationTrigger::PolicyChange);
        assert!(outcome.entries_invalidated >= 3); // tier1_item + negative + prefetch
        assert!(outcome.families_affected.contains(&CacheFamily::Tier1Item));
        assert!(outcome
            .families_affected
            .contains(&CacheFamily::NegativeCache));
        assert!(outcome
            .families_affected
            .contains(&CacheFamily::PrefetchHints));
        assert!(outcome.maintenance_events.iter().any(|event| {
            event.family == CacheFamily::Tier1Item
                && event.event == CacheEvent::Invalidation
                && event.reason == Some(CacheReason::PolicyChanged)
                && event.entries_affected == 1
        }));
        assert!(outcome.maintenance_events.iter().any(|event| {
            event.family == CacheFamily::PrefetchHints
                && event.event == CacheEvent::PrefetchDrop
                && event.reason == Some(CacheReason::PolicyChanged)
                && event.warm_source == Some(WarmSource::PrefetchHints)
        }));
        // team.beta should survive
        assert_eq!(mgr.result.len(), 1);
    }

    #[test]
    fn namespace_change_broadly_invalidates_all_families_and_prefetch() {
        let mut mgr = CacheManager::new(4, 4);
        mgr.tier1_item.admit(
            make_key(CacheFamily::Tier1Item, "team.alpha", 1),
            vec![MemoryId(1)],
            CacheAdmissionRequest::default(),
        );
        mgr.result.admit(
            make_key(CacheFamily::ResultCache, "team.beta", 3),
            vec![MemoryId(3)],
            CacheAdmissionRequest {
                request_shape_hash: Some(3),
                ..CacheAdmissionRequest::default()
            },
        );
        mgr.summary.admit(
            make_key(CacheFamily::SummaryCache, "team.gamma", 4),
            vec![MemoryId(4)],
            CacheAdmissionRequest {
                request_shape_hash: Some(4),
                ..CacheAdmissionRequest::default()
            },
        );
        mgr.prefetch.submit_hint(
            ns("team.alpha"),
            PrefetchTrigger::SessionRecency,
            vec![MemoryId(10)],
        );
        mgr.prefetch.submit_hint(
            ns("team.beta"),
            PrefetchTrigger::SessionRecency,
            vec![MemoryId(11)],
        );

        let outcome =
            mgr.handle_invalidation(InvalidationTrigger::NamespaceChange, &ns("team.alpha"));

        assert_eq!(outcome.trigger, InvalidationTrigger::NamespaceChange);
        assert!(outcome.entries_invalidated >= 5);
        assert!(outcome.families_affected.contains(&CacheFamily::Tier1Item));
        assert!(outcome
            .families_affected
            .contains(&CacheFamily::ResultCache));
        assert!(outcome
            .families_affected
            .contains(&CacheFamily::SummaryCache));
        assert!(outcome
            .families_affected
            .contains(&CacheFamily::PrefetchHints));
        assert!(outcome.maintenance_events.iter().any(|event| {
            event.family == CacheFamily::ResultCache
                && event.event == CacheEvent::Invalidation
                && event.reason == Some(CacheReason::NamespaceNarrowed)
        }));
        assert!(outcome.maintenance_events.iter().any(|event| {
            event.family == CacheFamily::PrefetchHints
                && event.event == CacheEvent::PrefetchDrop
                && event.reason == Some(CacheReason::NamespaceNarrowed)
                && event.entries_affected == 2
        }));
        assert_eq!(mgr.tier1_item.len(), 0);
        assert_eq!(mgr.result.len(), 0);
        assert_eq!(mgr.summary.len(), 0);
        assert_eq!(mgr.prefetch.queue_depth(), 0);
    }

    #[test]
    fn repair_and_generation_changes_broadly_invalidate_all_families_and_prefetch() {
        for trigger in [
            InvalidationTrigger::SchemaChange,
            InvalidationTrigger::IndexChange,
            InvalidationTrigger::EmbeddingChange,
            InvalidationTrigger::RankingChange,
            InvalidationTrigger::RepairStarted,
        ] {
            let mut mgr = CacheManager::new(4, 4);
            mgr.tier1_item.admit(
                make_key(CacheFamily::Tier1Item, "team.alpha", 1),
                vec![MemoryId(1)],
                CacheAdmissionRequest::default(),
            );
            mgr.result.admit(
                make_key(CacheFamily::ResultCache, "team.beta", 3),
                vec![MemoryId(3)],
                CacheAdmissionRequest {
                    request_shape_hash: Some(3),
                    ..CacheAdmissionRequest::default()
                },
            );
            mgr.summary.admit(
                make_key(CacheFamily::SummaryCache, "team.gamma", 4),
                vec![MemoryId(4)],
                CacheAdmissionRequest {
                    request_shape_hash: Some(4),
                    ..CacheAdmissionRequest::default()
                },
            );
            mgr.prefetch.submit_hint(
                ns("team.alpha"),
                PrefetchTrigger::SessionRecency,
                vec![MemoryId(10)],
            );
            mgr.prefetch.submit_hint(
                ns("team.beta"),
                PrefetchTrigger::SessionRecency,
                vec![MemoryId(11)],
            );

            let outcome = mgr.handle_invalidation(trigger, &ns("team.alpha"));

            assert_eq!(outcome.trigger, trigger);
            assert!(
                outcome.entries_invalidated >= 5,
                "trigger={}",
                trigger.as_str()
            );
            assert!(outcome.families_affected.contains(&CacheFamily::Tier1Item));
            assert!(outcome
                .families_affected
                .contains(&CacheFamily::ResultCache));
            assert!(outcome
                .families_affected
                .contains(&CacheFamily::SummaryCache));
            assert!(outcome
                .families_affected
                .contains(&CacheFamily::PrefetchHints));
            assert!(outcome.maintenance_events.iter().any(|event| {
                event.family == CacheFamily::Tier1Item
                    && event.event == CacheEvent::Invalidation
                    && event.reason
                        == Some(match trigger {
                            InvalidationTrigger::SchemaChange => CacheReason::SchemaChanged,
                            InvalidationTrigger::IndexChange => CacheReason::IndexChanged,
                            InvalidationTrigger::EmbeddingChange => CacheReason::EmbeddingChanged,
                            InvalidationTrigger::RankingChange => CacheReason::RankingChanged,
                            InvalidationTrigger::RepairStarted => CacheReason::RepairIncomplete,
                            _ => unreachable!("unexpected trigger in regression loop"),
                        })
            }));
            assert!(outcome.maintenance_events.iter().any(|event| {
                event.family == CacheFamily::PrefetchHints
                    && event.event == CacheEvent::PrefetchDrop
                    && event.reason
                        == Some(match trigger {
                            InvalidationTrigger::SchemaChange => CacheReason::SchemaChanged,
                            InvalidationTrigger::IndexChange => CacheReason::IndexChanged,
                            InvalidationTrigger::EmbeddingChange => CacheReason::EmbeddingChanged,
                            InvalidationTrigger::RankingChange => CacheReason::RankingChanged,
                            InvalidationTrigger::RepairStarted => CacheReason::RepairIncomplete,
                            _ => unreachable!("unexpected trigger in regression loop"),
                        })
            }));
            assert_eq!(mgr.tier1_item.len(), 0);
            assert_eq!(mgr.result.len(), 0);
            assert_eq!(mgr.summary.len(), 0);
            assert_eq!(mgr.prefetch.queue_depth(), 0);
        }
    }

    #[test]
    fn repair_warmup_emits_traceable_per_family_events() {
        let mut mgr = CacheManager::new(4, 4);

        let events = mgr.repair_warmup(
            &[
                CacheFamily::Tier1Item,
                CacheFamily::ResultCache,
                CacheFamily::PrefetchHints,
            ],
            2,
        );

        assert_eq!(events.len(), 2);
        assert!(events.iter().all(|event| {
            event.event == CacheEvent::RepairWarmup
                && event.generation_status == GenerationStatus::Valid
                && event.entries_affected == 2
        }));
        assert!(events.iter().any(|event| {
            event.family == CacheFamily::Tier1Item
                && event.warm_source == Some(WarmSource::Tier1ItemCache)
        }));
        assert!(events.iter().any(|event| {
            event.family == CacheFamily::ResultCache
                && event.warm_source == Some(WarmSource::ResultCache)
        }));
        assert!(events
            .iter()
            .all(|event| event.family != CacheFamily::PrefetchHints));
    }

    #[test]
    fn store_for_returns_none_for_prefetch_family() {
        let mut mgr = CacheManager::new(4, 4);

        assert!(mgr.store_for(CacheFamily::PrefetchHints).is_none());
        assert!(mgr.store_for(CacheFamily::Tier1Item).is_some());
    }

    // ── § 9.9  Version-mismatch bypass ───────────────────────────────────

    #[test]
    fn future_generation_returns_version_mismatch_bypass() {
        let mut store = BoundedCacheStore::new(CacheFamily::SummaryCache, 10);
        // Store at gen=2 but look up expecting gen=1 (rollback scenario)
        let key = CacheKey {
            family: CacheFamily::SummaryCache,
            namespace: ns("ns"),
            workspace_key: None,
            owner_key: None,
            request_shape_hash: None,
            item_key: 1,
            generations: gen_anchors(2),
        };
        store.admit(
            key.clone(),
            vec![MemoryId(1)],
            CacheAdmissionRequest::default(),
        );

        let result = store.lookup(&key, &gen_anchors(1));
        assert_eq!(result.event, CacheEvent::Bypass);
        assert_eq!(result.reason, Some(CacheReason::VersionMismatch));
        assert_eq!(
            result.generation_status,
            GenerationStatus::VersionMismatched
        );
    }

    // ── § 9.10  Per-request metrics accumulation ─────────────────────────

    #[test]
    fn per_request_metrics_accumulate_across_families() {
        let mut req = CacheRequestMetrics::default();
        req.record_lookup(&CacheLookupResult {
            family: CacheFamily::Tier1Item,
            event: CacheEvent::Hit,
            reason: None,
            warm_source: Some(WarmSource::Tier1ItemCache),
            generation_status: GenerationStatus::Valid,
            memory_ids: vec![MemoryId(1)],
            candidates_after: 1,
        });
        req.record_lookup(&CacheLookupResult {
            family: CacheFamily::AnnProbeCache,
            event: CacheEvent::Miss,
            reason: None,
            warm_source: None,
            generation_status: GenerationStatus::Unknown,
            memory_ids: vec![],
            candidates_after: 0,
        });
        req.record_cold_fallback();

        assert_eq!(req.cache_hit_count, 1);
        assert_eq!(req.cache_miss_count, 1);
        assert_eq!(req.cold_fallback_count, 1);
        assert_eq!(req.tier1_item_hit_count, 1);
        assert_eq!(req.ann_probe_hit_count, 0);
    }

    #[test]
    fn per_request_metrics_track_prefetch_drops_and_prefetch_hits() {
        let mut req = CacheRequestMetrics::default();
        req.record_lookup(&CacheLookupResult {
            family: CacheFamily::PrefetchHints,
            event: CacheEvent::Hit,
            reason: None,
            warm_source: Some(WarmSource::PrefetchHints),
            generation_status: GenerationStatus::Valid,
            memory_ids: vec![MemoryId(7)],
            candidates_after: 1,
        });
        req.record_lookup(&CacheLookupResult {
            family: CacheFamily::PrefetchHints,
            event: CacheEvent::PrefetchDrop,
            reason: Some(CacheReason::BudgetExhausted),
            warm_source: None,
            generation_status: GenerationStatus::Unknown,
            memory_ids: vec![],
            candidates_after: 0,
        });
        req.record_lookup(&CacheLookupResult {
            family: CacheFamily::SessionWarmup,
            event: CacheEvent::SessionExpired,
            reason: Some(CacheReason::NamespaceNarrowed),
            warm_source: None,
            generation_status: GenerationStatus::Unknown,
            memory_ids: vec![],
            candidates_after: 0,
        });

        assert_eq!(req.cache_hit_count, 1);
        assert_eq!(req.prefetch_hit_count, 1);
        assert_eq!(req.prefetch_used_count, 1);
        assert_eq!(req.prefetch_dropped_count, 2);
        assert_eq!(req.cache_bypass_count, 1);
        assert_eq!(req.cold_fallback_count, 0);
    }

    #[test]
    fn per_request_metrics_preserve_per_family_hit_breakdown() {
        let mut req = CacheRequestMetrics::default();
        for family in [
            CacheFamily::NegativeCache,
            CacheFamily::ResultCache,
            CacheFamily::EntityNeighborhood,
            CacheFamily::SummaryCache,
            CacheFamily::AnnProbeCache,
        ] {
            req.record_lookup(&CacheLookupResult {
                family,
                event: CacheEvent::Hit,
                reason: None,
                warm_source: Some(match family {
                    CacheFamily::NegativeCache => WarmSource::NegativeCache,
                    CacheFamily::ResultCache => WarmSource::ResultCache,
                    CacheFamily::EntityNeighborhood => WarmSource::EntityNeighborhood,
                    CacheFamily::SummaryCache => WarmSource::SummaryCache,
                    CacheFamily::AnnProbeCache => WarmSource::AnnProbeCache,
                    _ => unreachable!(),
                }),
                generation_status: GenerationStatus::Valid,
                memory_ids: vec![MemoryId(9)],
                candidates_after: 1,
            });
        }

        assert_eq!(req.cache_hit_count, 5);
        assert_eq!(req.negative_cache_hit_count, 1);
        assert_eq!(req.result_cache_hit_count, 1);
        assert_eq!(req.entity_neighborhood_hit_count, 1);
        assert_eq!(req.summary_cache_hit_count, 1);
        assert_eq!(req.ann_probe_hit_count, 1);
        assert_eq!(
            req.family_breakdown(CacheFamily::NegativeCache).hit_count,
            1
        );
        assert_eq!(req.family_breakdown(CacheFamily::ResultCache).hit_count, 1);
        assert_eq!(
            req.family_breakdown(CacheFamily::EntityNeighborhood)
                .hit_count,
            1
        );
        assert_eq!(req.family_breakdown(CacheFamily::SummaryCache).hit_count, 1);
        assert_eq!(
            req.family_breakdown(CacheFamily::AnnProbeCache).hit_count,
            1
        );
    }

    #[test]
    fn per_request_metrics_preserve_per_family_event_breakdown() {
        let mut req = CacheRequestMetrics::default();
        req.record_lookup(&CacheLookupResult {
            family: CacheFamily::Tier1Item,
            event: CacheEvent::Miss,
            reason: None,
            warm_source: None,
            generation_status: GenerationStatus::Unknown,
            memory_ids: vec![],
            candidates_after: 0,
        });
        req.record_lookup(&CacheLookupResult {
            family: CacheFamily::ResultCache,
            event: CacheEvent::StaleWarning,
            reason: Some(CacheReason::GenerationAnchorMismatch),
            warm_source: None,
            generation_status: GenerationStatus::Stale,
            memory_ids: vec![],
            candidates_after: 0,
        });
        req.record_lookup(&CacheLookupResult {
            family: CacheFamily::SummaryCache,
            event: CacheEvent::Disabled,
            reason: None,
            warm_source: None,
            generation_status: GenerationStatus::Unknown,
            memory_ids: vec![],
            candidates_after: 0,
        });
        req.record_lookup(&CacheLookupResult {
            family: CacheFamily::PrefetchHints,
            event: CacheEvent::PrefetchDrop,
            reason: Some(CacheReason::BudgetExhausted),
            warm_source: None,
            generation_status: GenerationStatus::Unknown,
            memory_ids: vec![],
            candidates_after: 0,
        });
        req.record_lookup(&CacheLookupResult {
            family: CacheFamily::SessionWarmup,
            event: CacheEvent::SessionExpired,
            reason: Some(CacheReason::NamespaceNarrowed),
            warm_source: None,
            generation_status: GenerationStatus::Unknown,
            memory_ids: vec![],
            candidates_after: 0,
        });
        req.record_lookup(&CacheLookupResult {
            family: CacheFamily::GoalConditioned,
            event: CacheEvent::Invalidation,
            reason: Some(CacheReason::PolicyChanged),
            warm_source: None,
            generation_status: GenerationStatus::Unknown,
            memory_ids: vec![],
            candidates_after: 0,
        });
        req.record_lookup(&CacheLookupResult {
            family: CacheFamily::ColdStartMitigation,
            event: CacheEvent::RepairWarmup,
            reason: None,
            warm_source: Some(WarmSource::ColdStartMitigation),
            generation_status: GenerationStatus::Valid,
            memory_ids: vec![MemoryId(99)],
            candidates_after: 1,
        });

        assert_eq!(req.cache_miss_count, 1);
        assert_eq!(req.cache_bypass_count, 3);
        assert_eq!(req.cache_invalidation_count, 1);
        assert_eq!(req.prefetch_dropped_count, 2);
        assert!(req.degraded_mode_served);
        assert_eq!(req.family_breakdown(CacheFamily::Tier1Item).miss_count, 1);
        let result_breakdown = req.family_breakdown(CacheFamily::ResultCache);
        assert_eq!(result_breakdown.bypass_count, 1);
        assert_eq!(result_breakdown.stale_warning_count, 1);
        let summary_breakdown = req.family_breakdown(CacheFamily::SummaryCache);
        assert_eq!(summary_breakdown.bypass_count, 1);
        assert_eq!(summary_breakdown.disabled_count, 1);
        assert_eq!(
            req.family_breakdown(CacheFamily::PrefetchHints)
                .prefetch_drop_count,
            1
        );
        let session_breakdown = req.family_breakdown(CacheFamily::SessionWarmup);
        assert_eq!(session_breakdown.bypass_count, 1);
        assert_eq!(session_breakdown.session_expired_count, 1);
        assert_eq!(
            req.family_breakdown(CacheFamily::GoalConditioned)
                .invalidation_count,
            1
        );
        assert_eq!(
            req.family_breakdown(CacheFamily::ColdStartMitigation)
                .repair_warmup_count,
            1
        );
    }

    #[test]
    fn per_request_metrics_do_not_treat_intermediate_miss_or_stale_as_cold_fallback() {
        let mut req = CacheRequestMetrics::default();
        req.record_lookup(&CacheLookupResult {
            family: CacheFamily::Tier1Item,
            event: CacheEvent::Miss,
            reason: None,
            warm_source: None,
            generation_status: GenerationStatus::Unknown,
            memory_ids: vec![],
            candidates_after: 0,
        });
        req.record_lookup(&CacheLookupResult {
            family: CacheFamily::ResultCache,
            event: CacheEvent::StaleWarning,
            reason: Some(CacheReason::GenerationAnchorMismatch),
            warm_source: None,
            generation_status: GenerationStatus::Stale,
            memory_ids: vec![],
            candidates_after: 0,
        });
        req.record_lookup(&CacheLookupResult {
            family: CacheFamily::AnnProbeCache,
            event: CacheEvent::Hit,
            reason: None,
            warm_source: Some(WarmSource::AnnProbeCache),
            generation_status: GenerationStatus::Valid,
            memory_ids: vec![MemoryId(11)],
            candidates_after: 1,
        });

        assert_eq!(req.cache_miss_count, 1);
        assert_eq!(req.cache_bypass_count, 1);
        assert_eq!(req.cache_hit_count, 1);
        assert_eq!(req.cold_fallback_count, 0);
    }
}
