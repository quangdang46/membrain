/// Version of the shared core API consumed by wrapper crates.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CoreApiVersion {
    /// Major version for breaking API changes.
    pub major: u16,
    /// Minor version for additive API changes.
    pub minor: u16,
}

impl CoreApiVersion {
    pub(crate) const fn current() -> Self {
        Self { major: 0, minor: 1 }
    }
}

/// Canonical memory families frozen by the synchronous encode fast path.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub enum CanonicalMemoryType {
    Event,
    Observation,
    ToolOutcome,
    UserPreference,
    SessionMarker,
}

impl CanonicalMemoryType {
    /// Returns the stable machine-readable name for this canonical memory family.
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::Event => "event",
            Self::Observation => "observation",
            Self::ToolOutcome => "tool_outcome",
            Self::UserPreference => "user_preference",
            Self::SessionMarker => "session_marker",
        }
    }
}

/// Raw intake kinds admitted to the synchronous encode fast path.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum RawIntakeKind {
    Event,
    Observation,
    ToolOutcome,
    UserPreference,
    SessionMarker,
}

impl RawIntakeKind {
    /// Returns the stable machine-readable name for this intake kind.
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::Event => "event",
            Self::Observation => "observation",
            Self::ToolOutcome => "tool_outcome",
            Self::UserPreference => "user_preference",
            Self::SessionMarker => "session_marker",
        }
    }

    /// Returns the first canonical memory family frozen during normalization.
    pub const fn canonical_memory_type(self) -> CanonicalMemoryType {
        match self {
            Self::Event => CanonicalMemoryType::Event,
            Self::Observation => CanonicalMemoryType::Observation,
            Self::ToolOutcome => CanonicalMemoryType::ToolOutcome,
            Self::UserPreference => CanonicalMemoryType::UserPreference,
            Self::SessionMarker => CanonicalMemoryType::SessionMarker,
        }
    }
}

/// Route families selected by bounded shallow classification on the encode fast path.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub enum FastPathRouteFamily {
    Event,
    Observation,
    ToolOutcome,
    UserPreference,
    SessionMarker,
}

impl FastPathRouteFamily {
    /// Returns the stable machine-readable name for this route family.
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::Event => "event",
            Self::Observation => "observation",
            Self::ToolOutcome => "tool_outcome",
            Self::UserPreference => "user_preference",
            Self::SessionMarker => "session_marker",
        }
    }

    /// Returns the route family selected for a canonical memory family.
    pub const fn from_memory_type(memory_type: CanonicalMemoryType) -> Self {
        match memory_type {
            CanonicalMemoryType::Event => Self::Event,
            CanonicalMemoryType::Observation => Self::Observation,
            CanonicalMemoryType::ToolOutcome => Self::ToolOutcome,
            CanonicalMemoryType::UserPreference => Self::UserPreference,
            CanonicalMemoryType::SessionMarker => Self::SessionMarker,
        }
    }
}

/// Bounded temporal-landmark signals evaluated during encode.
#[derive(Debug, Clone, Copy, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct LandmarkSignals {
    /// Emotional arousal signal normalized to 0.0..=1.0.
    pub arousal: f32,
    /// Novelty signal normalized to 0.0..=1.0.
    pub novelty: f32,
    /// Highest recent similarity score against nearby landmark candidates.
    pub recent_similarity: f32,
    /// Distance from the last accepted landmark in logical ticks.
    pub ticks_since_last_landmark: u64,
}

impl LandmarkSignals {
    /// Builds bounded landmark signals for one encode candidate.
    pub const fn new(
        arousal: f32,
        novelty: f32,
        recent_similarity: f32,
        ticks_since_last_landmark: u64,
    ) -> Self {
        Self {
            arousal,
            novelty,
            recent_similarity,
            ticks_since_last_landmark,
        }
    }
}

/// Additive landmark and era metadata derived during encode.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct LandmarkMetadata {
    /// Whether the memory qualifies as a temporal landmark.
    pub is_landmark: bool,
    /// Human-readable auto-label for landmark surfaces.
    pub landmark_label: Option<String>,
    /// Era identifier opened by this landmark, when one is created.
    pub era_id: Option<String>,
}

impl LandmarkMetadata {
    /// Returns the canonical non-landmark metadata shape.
    pub fn non_landmark() -> Self {
        Self {
            is_landmark: false,
            landmark_label: None,
            era_id: None,
        }
    }
}

/// Raw intake submitted to the synchronous encode fast path.
#[derive(Debug, Clone, PartialEq)]
pub struct RawEncodeInput {
    /// Caller-supplied kind used to freeze the first canonical memory family.
    pub kind: RawIntakeKind,
    /// Raw evidence preserved for normalization and later persistence.
    pub raw_text: String,
    /// Optional bounded temporal-landmark signals supplied by the caller.
    pub landmark_signals: Option<LandmarkSignals>,
}

impl RawEncodeInput {
    /// Builds a new raw intake payload for the encode fast path.
    pub fn new(kind: RawIntakeKind, raw_text: impl Into<String>) -> Self {
        Self {
            kind,
            raw_text: raw_text.into(),
            landmark_signals: None,
        }
    }

    /// Attaches bounded landmark signals for additive temporal enrichment.
    pub fn with_landmark_signals(mut self, landmark_signals: LandmarkSignals) -> Self {
        self.landmark_signals = Some(landmark_signals);
        self
    }
}

/// Canonical normalized envelope produced before the first durable write.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NormalizedMemoryEnvelope {
    /// First canonical memory family chosen for the durable record.
    pub memory_type: CanonicalMemoryType,
    /// Source intake family preserved for provenance.
    pub source_kind: RawIntakeKind,
    /// Raw evidence preserved for later persistence or inspect surfaces.
    pub raw_text: String,
    /// Human-readable normalized text used for fingerprinting and debugging.
    pub compact_text: String,
    /// Stable normalization generation used for cache and fingerprint invalidation.
    pub normalization_generation: &'static str,
    /// Bounded normalized payload size consulted by later routing.
    pub payload_size_bytes: usize,
    /// Additive temporal landmark and era metadata derived during encode.
    pub landmark: LandmarkMetadata,
    /// Optional passive-observation batch source preserved for provenance and inspect.
    pub observation_source: Option<String>,
    /// Optional passive-observation batch grouping preserved for inspect and repair.
    pub observation_chunk_id: Option<String>,
}

/// Stable identifier for persisted memories used by exact and recent Tier1 lookups.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub struct MemoryId(pub u64);

/// Stable identifier for one named time-travel snapshot.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub struct SnapshotId(pub u64);

/// Durable anchor used when retrieval or maintenance targets historical state.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum SnapshotAnchor {
    /// Resolve historical state from an explicit named snapshot.
    Named {
        snapshot_id: SnapshotId,
        snapshot_name: String,
        as_of_tick: u64,
    },
    /// Resolve historical state directly from a logical tick.
    Tick { as_of_tick: u64 },
}

impl SnapshotAnchor {
    /// Returns the stable machine-readable anchor kind.
    pub const fn kind(&self) -> &'static str {
        match self {
            Self::Named { .. } => "named_snapshot",
            Self::Tick { .. } => "tick",
        }
    }

    /// Returns the effective as-of tick carried by this anchor.
    pub const fn as_of_tick(&self) -> u64 {
        match self {
            Self::Named { as_of_tick, .. } => *as_of_tick,
            Self::Tick { as_of_tick } => *as_of_tick,
        }
    }

    /// Returns the named snapshot label when one exists.
    pub fn snapshot_name(&self) -> Option<&str> {
        match self {
            Self::Named { snapshot_name, .. } => Some(snapshot_name.as_str()),
            Self::Tick { .. } => None,
        }
    }
}

/// Durable metadata stored for one named snapshot without copying memory payloads.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct SnapshotMetadata {
    /// Stable durable identity for the snapshot record.
    pub snapshot_id: SnapshotId,
    /// Namespace the snapshot belongs to.
    pub namespace: NamespaceId,
    /// Human-readable snapshot label.
    pub snapshot_name: String,
    /// Logical tick captured by the snapshot.
    pub as_of_tick: u64,
    /// Whether the snapshot is still active for historical recall.
    pub active: bool,
}

impl SnapshotMetadata {
    /// Builds additive metadata for one active named snapshot.
    pub fn named(
        snapshot_id: SnapshotId,
        namespace: NamespaceId,
        snapshot_name: impl Into<String>,
        as_of_tick: u64,
    ) -> Self {
        Self {
            snapshot_id,
            namespace,
            snapshot_name: snapshot_name.into(),
            as_of_tick,
            active: true,
        }
    }

    /// Returns the canonical anchor derived from this snapshot metadata.
    pub fn anchor(&self) -> SnapshotAnchor {
        SnapshotAnchor::Named {
            snapshot_id: self.snapshot_id,
            snapshot_name: self.snapshot_name.clone(),
            as_of_tick: self.as_of_tick,
        }
    }

    /// Marks the snapshot as deleted while preserving its durable identity.
    pub fn deleted(mut self) -> Self {
        self.active = false;
        self
    }
}

use crate::api::NamespaceId;

#[cfg(test)]
mod tests {
    use super::{SnapshotAnchor, SnapshotId, SnapshotMetadata};
    use crate::api::NamespaceId;

    #[test]
    fn named_snapshot_anchor_preserves_tick_and_label() {
        let namespace = NamespaceId::new("tests.snapshot").unwrap();
        let metadata = SnapshotMetadata::named(SnapshotId(7), namespace, "baseline", 42);
        let anchor = metadata.anchor();

        assert_eq!(anchor.kind(), "named_snapshot");
        assert_eq!(anchor.as_of_tick(), 42);
        assert_eq!(anchor.snapshot_name(), Some("baseline"));
    }

    #[test]
    fn deleted_snapshot_metadata_keeps_identity_but_flips_active_flag() {
        let namespace = NamespaceId::new("tests.snapshot").unwrap();
        let metadata = SnapshotMetadata::named(SnapshotId(9), namespace, "pre_repair", 64).deleted();

        assert_eq!(metadata.snapshot_id, SnapshotId(9));
        assert_eq!(metadata.snapshot_name, "pre_repair");
        assert_eq!(metadata.as_of_tick, 64);
        assert!(!metadata.active);
    }

    #[test]
    fn tick_anchor_reports_no_snapshot_name() {
        let anchor = SnapshotAnchor::Tick { as_of_tick: 99 };

        assert_eq!(anchor.kind(), "tick");
        assert_eq!(anchor.as_of_tick(), 99);
        assert_eq!(anchor.snapshot_name(), None);
    }
}

/// Stable identifier for one session-local hot window.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub struct SessionId(pub u64);

/// Payload residency state for Tier1 metadata entries.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Tier1PayloadState {
    MetadataOnly,
    PreviewInline,
}

/// Bounded Tier1 hot metadata entry used for exact and recent lookups.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Tier1HotRecord {
    /// Effective namespace the hot memory is bound to.
    pub namespace: NamespaceId,
    /// Stable durable identity of the hot memory.
    pub memory_id: MemoryId,
    /// Session-local hot window this record belongs to.
    pub session_id: SessionId,
    /// Canonical memory family frozen on first persistence.
    pub memory_type: CanonicalMemoryType,
    /// Route family selected by the encode fast path.
    pub route_family: FastPathRouteFamily,
    /// Bounded human-readable text surface safe for Tier1 reuse.
    pub compact_text: String,
    /// Stable duplicate-family hint used for exact/recent reuse.
    pub fingerprint: u64,
    /// First-pass salience scalar carried in hot metadata.
    pub provisional_salience: u16,
    /// Original payload size so the hot set can avoid fetching it eagerly.
    pub payload_size_bytes: usize,
    /// Whether Tier1 carries only metadata or a bounded inline preview.
    pub payload_state: Tier1PayloadState,
}

impl Tier1HotRecord {
    /// Builds a metadata-only Tier1 record.
    #[allow(clippy::too_many_arguments)]
    pub fn metadata_only(
        namespace: NamespaceId,
        memory_id: MemoryId,
        session_id: SessionId,
        memory_type: CanonicalMemoryType,
        route_family: FastPathRouteFamily,
        compact_text: impl Into<String>,
        fingerprint: u64,
        provisional_salience: u16,
        payload_size_bytes: usize,
    ) -> Self {
        Self {
            namespace,
            memory_id,
            session_id,
            memory_type,
            route_family,
            compact_text: compact_text.into(),
            fingerprint,
            provisional_salience,
            payload_size_bytes,
            payload_state: Tier1PayloadState::MetadataOnly,
        }
    }
}

/// Stable identifier for working-memory candidates before durable persistence.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct WorkingMemoryId(pub u64);

/// Bounded working-memory candidate tracked by the encode controller.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WorkingMemoryItem {
    /// Stable candidate identifier within the controller.
    pub id: WorkingMemoryId,
    /// Attention score used for admission, eviction, and promotion decisions.
    pub attention_score: u16,
    /// Whether the candidate is pinned and therefore not eligible for eviction.
    pub pinned: bool,
}

impl WorkingMemoryItem {
    /// Builds a new unpinned working-memory candidate.
    pub const fn new(id: WorkingMemoryId, attention_score: u16) -> Self {
        Self {
            id,
            attention_score,
            pinned: false,
        }
    }

    /// Returns a copy of this candidate marked as pinned.
    pub const fn pinned(mut self) -> Self {
        self.pinned = true;
        self
    }
}
