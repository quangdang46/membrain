use crate::types::{CanonicalMemoryType, FastPathRouteFamily, LandmarkMetadata, LandmarkSignals};

/// High-level audit event families preserved in append-only storage.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum AuditEventCategory {
    Encode,
    Recall,
    Policy,
    Maintenance,
    Archive,
}

impl AuditEventCategory {
    /// Returns the stable machine-readable category name.
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Encode => "encode",
            Self::Recall => "recall",
            Self::Policy => "policy",
            Self::Maintenance => "maintenance",
            Self::Archive => "archive",
        }
    }
}

/// Stable audit event taxonomy for append-only log rows.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum AuditEventKind {
    EncodeAccepted,
    EncodeRejected,
    RecallServed,
    RecallDenied,
    PolicyDenied,
    PolicyRedacted,
    MaintenanceRepairStarted,
    MaintenanceRepairCompleted,
    MaintenanceMigrationApplied,
    MaintenanceCompactionApplied,
    IncidentRecorded,
    ArchiveRecorded,
}

impl AuditEventKind {
    /// Returns the stable machine-readable event name.
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::EncodeAccepted => "encode_accepted",
            Self::EncodeRejected => "encode_rejected",
            Self::RecallServed => "recall_served",
            Self::RecallDenied => "recall_denied",
            Self::PolicyDenied => "policy_denied",
            Self::PolicyRedacted => "policy_redacted",
            Self::MaintenanceRepairStarted => "maintenance_repair_started",
            Self::MaintenanceRepairCompleted => "maintenance_repair_completed",
            Self::MaintenanceMigrationApplied => "maintenance_migration_applied",
            Self::MaintenanceCompactionApplied => "maintenance_compaction_applied",
            Self::IncidentRecorded => "incident_recorded",
            Self::ArchiveRecorded => "archive_recorded",
        }
    }

    /// Returns the canonical category for this event kind.
    pub const fn category(self) -> AuditEventCategory {
        match self {
            Self::EncodeAccepted | Self::EncodeRejected => AuditEventCategory::Encode,
            Self::RecallServed | Self::RecallDenied => AuditEventCategory::Recall,
            Self::PolicyDenied | Self::PolicyRedacted => AuditEventCategory::Policy,
            Self::MaintenanceRepairStarted
            | Self::MaintenanceRepairCompleted
            | Self::MaintenanceMigrationApplied
            | Self::MaintenanceCompactionApplied
            | Self::IncidentRecorded => AuditEventCategory::Maintenance,
            Self::ArchiveRecorded => AuditEventCategory::Archive,
        }
    }
}

/// Canonical outcome classes shared across core APIs and wrappers.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum OutcomeClass {
    Accepted,
    Rejected,
    Partial,
    Preview,
    Blocked,
    Degraded,
}

impl OutcomeClass {
    /// Returns the stable machine-readable retrieval outcome label.
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Accepted => "accepted",
            Self::Rejected => "rejected",
            Self::Partial => "partial",
            Self::Preview => "preview",
            Self::Blocked => "blocked",
            Self::Degraded => "degraded",
        }
    }
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

impl EncodeFastPathStage {
    /// Returns the stable machine-readable fast-path stage label.
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Normalize => "normalize",
            Self::Fingerprint => "fingerprint",
            Self::ShallowClassify => "shallow_classify",
            Self::ProvisionalSalience => "provisional_salience",
            Self::LandmarkTagging => "landmark_tagging",
        }
    }
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

impl Tier1LookupLane {
    /// Returns the stable machine-readable Tier1 lane label.
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::ExactHandle => "exact_handle",
            Self::RecentWindow => "recent_window",
        }
    }
}

/// Machine-readable Tier1 outcomes for exact and recent hot-set reuse.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Tier1LookupOutcome {
    Hit,
    Miss,
    Bypass,
    StaleBypass,
}

impl Tier1LookupOutcome {
    /// Returns the stable machine-readable Tier1 outcome label.
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Hit => "hit",
            Self::Miss => "miss",
            Self::Bypass => "bypass",
            Self::StaleBypass => "stale_bypass",
        }
    }
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

impl Tier2PrefilterOutcome {
    /// Returns the stable machine-readable Tier2 prefilter outcome label.
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Ready => "ready",
            Self::Bypass => "bypass",
        }
    }
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

impl AdmissionOutcomeKind {
    /// Returns the stable machine-readable working-memory outcome label.
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Discarded => "discarded",
            Self::Buffered => "buffered",
            Self::Promoted => "promoted",
        }
    }
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

impl CacheLookupOutcome {
    /// Returns the stable machine-readable cache lookup outcome label.
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Hit => "hit",
            Self::Miss => "miss",
            Self::Bypass => "bypass",
            Self::StaleWarning => "stale_warning",
            Self::Disabled => "disabled",
        }
    }
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

#[cfg(test)]
mod tests {
    use super::{
        AdmissionOutcomeKind, CacheLookupOutcome, EncodeFastPathStage, OutcomeClass,
        Tier1LookupLane, Tier1LookupOutcome, Tier2PrefilterOutcome,
    };

    #[test]
    fn retrieval_outcome_class_labels_match_contract() {
        assert_eq!(OutcomeClass::Accepted.as_str(), "accepted");
        assert_eq!(OutcomeClass::Rejected.as_str(), "rejected");
        assert_eq!(OutcomeClass::Partial.as_str(), "partial");
        assert_eq!(OutcomeClass::Preview.as_str(), "preview");
        assert_eq!(OutcomeClass::Blocked.as_str(), "blocked");
        assert_eq!(OutcomeClass::Degraded.as_str(), "degraded");
    }

    #[test]
    fn trace_lane_and_outcome_labels_remain_machine_readable() {
        assert_eq!(EncodeFastPathStage::Normalize.as_str(), "normalize");
        assert_eq!(
            EncodeFastPathStage::ProvisionalSalience.as_str(),
            "provisional_salience"
        );
        assert_eq!(Tier1LookupLane::ExactHandle.as_str(), "exact_handle");
        assert_eq!(Tier1LookupLane::RecentWindow.as_str(), "recent_window");
        assert_eq!(Tier1LookupOutcome::Hit.as_str(), "hit");
        assert_eq!(Tier1LookupOutcome::Miss.as_str(), "miss");
        assert_eq!(Tier1LookupOutcome::Bypass.as_str(), "bypass");
        assert_eq!(Tier1LookupOutcome::StaleBypass.as_str(), "stale_bypass");
        assert_eq!(Tier2PrefilterOutcome::Ready.as_str(), "ready");
        assert_eq!(Tier2PrefilterOutcome::Bypass.as_str(), "bypass");
    }

    #[test]
    fn working_memory_and_cache_labels_remain_stable() {
        assert_eq!(AdmissionOutcomeKind::Discarded.as_str(), "discarded");
        assert_eq!(AdmissionOutcomeKind::Buffered.as_str(), "buffered");
        assert_eq!(AdmissionOutcomeKind::Promoted.as_str(), "promoted");
        assert_eq!(CacheLookupOutcome::Hit.as_str(), "hit");
        assert_eq!(CacheLookupOutcome::Miss.as_str(), "miss");
        assert_eq!(CacheLookupOutcome::Bypass.as_str(), "bypass");
        assert_eq!(CacheLookupOutcome::StaleWarning.as_str(), "stale_warning");
        assert_eq!(CacheLookupOutcome::Disabled.as_str(), "disabled");
    }
}
