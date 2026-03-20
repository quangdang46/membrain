use crate::types::{CanonicalMemoryType, FastPathRouteFamily, LandmarkMetadata, LandmarkSignals};

/// Canonical outcome classes shared across core APIs and wrappers.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OutcomeClass {
    Accepted,
    Rejected,
    Partial,
    Preview,
    Blocked,
    Degraded,
}

/// Ordered synchronous stages on the encode fast path.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EncodeFastPathStage {
    Normalize,
    Fingerprint,
    ShallowClassify,
    ProvisionalSalience,
    LandmarkTagging,
}

/// Stable trace artifact for the synchronous encode fast path.
#[derive(Debug, Clone, PartialEq)]
pub struct EncodeFastPathTrace {
    /// Ordered fast-path stages executed before persistence.
    pub stages: [EncodeFastPathStage; 5],
    /// Stable normalization generation used for the normalized envelope.
    pub normalization_generation: &'static str,
    /// Canonical memory family frozen by normalization.
    pub memory_type: CanonicalMemoryType,
    /// Provisional route family selected by shallow classification.
    pub route_family: FastPathRouteFamily,
    /// First-pass salience scalar used for bounded routing inputs.
    pub provisional_salience: u16,
    /// Number of duplicate-hint candidates consulted on the fast path.
    pub duplicate_hint_candidate_count: usize,
    /// Landmark signals consulted for additive temporal enrichment.
    pub landmark_signals: Option<LandmarkSignals>,
    /// Landmark and era metadata derived during the fast path.
    pub landmark: LandmarkMetadata,
    /// Whether the fast path stayed inside its declared bounded latency contract.
    pub stayed_within_latency_budget: bool,
}

/// Tier1 lookup lanes that remain inspectable on the request path.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Tier1LookupLane {
    ExactHandle,
    RecentWindow,
}

/// Machine-readable Tier1 outcomes for exact and recent hot-set reuse.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Tier1LookupOutcome {
    Hit,
    Miss,
    Bypass,
    StaleBypass,
}

/// Stable trace artifact for Tier1 exact and recent lookups.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Tier1LookupTrace {
    /// Which Tier1 lane fired on the request path.
    pub lane: Tier1LookupLane,
    /// Final lookup outcome for that Tier1 lane.
    pub outcome: Tier1LookupOutcome,
    /// Number of recent candidates inspected when scanning one session window.
    pub recent_candidates_inspected: usize,
    /// Whether the recent-window lane produced a hit.
    pub session_window_hit: bool,
    /// Number of heavyweight payload fetches triggered by the Tier1 lane.
    pub payload_fetch_count: usize,
}

/// Machine-readable Tier2 outcomes for metadata-first durable item planning.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Tier2PrefilterOutcome {
    Ready,
    Bypass,
}

/// Stable trace artifact for Tier2 metadata-first prefilter and index planning.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Tier2PrefilterTrace {
    /// Whether the operation stayed on metadata-only durable rows.
    pub outcome: Tier2PrefilterOutcome,
    /// Number of durable metadata candidates exposed to the planner.
    pub metadata_candidate_count: usize,
    /// Number of heavyweight payload fetches triggered before the final cut.
    pub payload_fetch_count: usize,
}

/// Machine-readable admission outcomes for the working-memory controller.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AdmissionOutcomeKind {
    /// The candidate was dropped before entering controller state.
    Discarded,
    /// The candidate was buffered in working memory without durable promotion.
    Buffered,
    /// The candidate or an overflow victim should be promoted to encode.
    Promoted,
}

/// Stable trace artifact for working-memory admission decisions.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct WorkingMemoryTrace {
    /// The final admission outcome.
    pub outcome: AdmissionOutcomeKind,
    /// Slot pressure observed when the decision was made.
    pub slot_pressure: usize,
    /// Threshold consulted for the decision.
    pub threshold: u16,
    /// Whether an overflow path was involved.
    pub overflowed: bool,
}

/// Stable observability boundary for shared trace and audit vocabularies.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct ObservabilityModule;

impl ObservabilityModule {
    /// Returns the stable component identifier for this observability surface.
    pub const fn component_name(&self) -> &'static str {
        "observability"
    }
}

/// Machine-readable cache lookup outcome for the observability trace stream.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CacheLookupOutcome {
    Hit,
    Miss,
    Bypass,
    StaleWarning,
    Disabled,
}

/// Stable trace artifact for one cache-family evaluation on the request path.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CacheEvalTrace {
    /// Which cache family was evaluated.
    pub outcome: CacheLookupOutcome,
    /// Candidate count before this cache evaluation.
    pub candidates_before: usize,
    /// Candidate count after this cache evaluation.
    pub candidates_after: usize,
    /// Whether the result came from a warm source.
    pub warm_reuse: bool,
}
