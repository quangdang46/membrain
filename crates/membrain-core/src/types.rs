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
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
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
#[derive(Debug, Clone, Copy, PartialEq)]
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
#[derive(Debug, Clone, PartialEq, Eq)]
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

use crate::api::NamespaceId;

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
