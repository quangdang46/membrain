use crate::api::{AgentId, NamespaceId, TaskId, WorkspaceId};
use crate::observability::AuditEventKind;
use crate::policy::SharingVisibility;

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
        Self { major: 0, minor: 5 }
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

/// Bounded affect signals evaluated during encode and trajectory capture.
#[derive(Debug, Clone, Copy, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct AffectSignals {
    /// Emotional valence normalized to -1.0..=1.0.
    pub valence: f32,
    /// Emotional arousal normalized to 0.0..=1.0.
    pub arousal: f32,
}

impl AffectSignals {
    /// Builds bounded affect signals for one encode candidate.
    pub const fn new(valence: f32, arousal: f32) -> Self {
        Self { valence, arousal }
    }

    /// Returns a clamped copy using the canonical durable bounds.
    pub fn clamped(self) -> Self {
        Self {
            valence: self.valence.clamp(-1.0, 1.0),
            arousal: self.arousal.clamp(0.0, 1.0),
        }
    }
}

/// One durable affect-trajectory row captured from encode-time signals.
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct AffectTrajectoryRow {
    /// Namespace owning this trajectory row.
    pub namespace: NamespaceId,
    /// Optional era anchor associated with the recorded affect state.
    pub era_id: Option<String>,
    /// Memory whose encode-time affect signal produced this row.
    pub memory_id: MemoryId,
    /// Logical tick where this row begins.
    pub tick_start: u64,
    /// Logical tick where this row ends.
    pub tick_end: Option<u64>,
    /// Averaged valence carried by the stored row.
    pub avg_valence: f32,
    /// Averaged arousal carried by the stored row.
    pub avg_arousal: f32,
    /// Number of contributing memories represented by this row.
    pub memory_count: u64,
    /// Stable label naming where the authoritative trajectory lives.
    pub authoritative_truth: &'static str,
}

/// Bounded mood-history surface returned by core and wrapper introspection paths.
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct AffectTrajectoryHistory {
    /// Namespace whose affect trajectory was queried.
    pub namespace: NamespaceId,
    /// Optional inclusive lower tick bound applied to the result.
    pub since_tick: Option<u64>,
    /// Total rows returned after filtering.
    pub total_rows: usize,
    /// Ordered durable trajectory rows.
    pub rows: Vec<AffectTrajectoryRow>,
    /// Stable label naming where the authoritative trajectory lives.
    pub authoritative_truth: &'static str,
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
    /// Logical tick where the era opened for this landmark.
    pub era_started_at_tick: Option<u64>,
    /// Bounded landmark qualification score in 0..=1000.
    pub detection_score: u16,
    /// Inspectable reason string describing why the memory became a landmark.
    pub detection_reason: Option<String>,
}

impl LandmarkMetadata {
    /// Returns the canonical non-landmark metadata shape.
    pub fn non_landmark() -> Self {
        Self {
            is_landmark: false,
            landmark_label: None,
            era_id: None,
            era_started_at_tick: None,
            detection_score: 0,
            detection_reason: None,
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
    /// Optional bounded affect signals supplied by the caller.
    pub affect_signals: Option<AffectSignals>,
    /// Optional bounded temporal-landmark signals supplied by the caller.
    pub landmark_signals: Option<LandmarkSignals>,
    /// Optional currently active era carried by the caller for non-landmark memories.
    pub active_era_id: Option<String>,
    /// Optional current logical tick so landmark era boundaries can use an absolute anchor.
    pub current_tick: Option<u64>,
}

impl RawEncodeInput {
    /// Builds a new raw intake payload for the encode fast path.
    pub fn new(kind: RawIntakeKind, raw_text: impl Into<String>) -> Self {
        Self {
            kind,
            raw_text: raw_text.into(),
            affect_signals: None,
            landmark_signals: None,
            active_era_id: None,
            current_tick: None,
        }
    }

    /// Attaches bounded affect signals for additive emotional enrichment.
    pub fn with_affect_signals(mut self, affect_signals: AffectSignals) -> Self {
        self.affect_signals = Some(affect_signals);
        self
    }

    /// Attaches bounded landmark signals for additive temporal enrichment.
    pub fn with_landmark_signals(mut self, landmark_signals: LandmarkSignals) -> Self {
        self.landmark_signals = Some(landmark_signals);
        self
    }

    /// Attaches the caller's currently active era for additive temporal anchoring.
    pub fn with_active_era_id(mut self, active_era_id: impl Into<String>) -> Self {
        self.active_era_id = Some(active_era_id.into());
        self
    }

    /// Attaches the caller's current logical tick for absolute era boundary anchoring.
    pub fn with_current_tick(mut self, current_tick: u64) -> Self {
        self.current_tick = Some(current_tick);
        self
    }
}

/// Cross-agent sharing metadata preserved with a durable memory record.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct SharingMetadata {
    /// Canonical visibility governing cross-agent reuse.
    pub visibility: SharingVisibility,
    /// Optional workspace scope preserved for audit and redaction.
    pub workspace_id: Option<WorkspaceId>,
    /// Optional producing agent identity preserved for provenance and policy scope.
    pub agent_id: Option<AgentId>,
}

impl SharingMetadata {
    /// Builds a sharing envelope with one explicit visibility level.
    pub fn new(visibility: SharingVisibility) -> Self {
        Self {
            visibility,
            ..Self::default()
        }
    }

    /// Attaches one workspace scope to the sharing envelope.
    pub fn with_workspace_id(mut self, workspace_id: WorkspaceId) -> Self {
        self.workspace_id = Some(workspace_id);
        self
    }

    /// Attaches one producing agent identity to the sharing envelope.
    pub fn with_agent_id(mut self, agent_id: AgentId) -> Self {
        self.agent_id = Some(agent_id);
        self
    }
}

/// Durable compression lineage metadata preserved beside canonical memory records.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct CompressionMetadata {
    /// Schema memory this memory was compressed into, when the row is a compressed source.
    pub compressed_into: Option<MemoryId>,
    /// Logical tick when compression state was recorded.
    pub compression_tick: Option<u64>,
    /// Source memories distilled into this row when the row itself is a schema memory.
    pub source_memory_ids: Vec<MemoryId>,
}

impl CompressionMetadata {
    /// Returns true when no durable compression lineage has been recorded.
    pub fn is_empty(&self) -> bool {
        self.compressed_into.is_none()
            && self.compression_tick.is_none()
            && self.source_memory_ids.is_empty()
    }
}

/// Canonical normalized envelope produced before the first durable write.
#[derive(Debug, Clone, PartialEq)]
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
    /// Optional bounded affect signals preserved as encode-time trajectory input.
    pub affect: Option<AffectSignals>,
    /// Additive temporal landmark and era metadata derived during encode.
    pub landmark: LandmarkMetadata,
    /// Optional passive-observation batch source preserved for provenance and inspect.
    pub observation_source: Option<String>,
    /// Optional passive-observation batch grouping preserved for inspect and repair.
    pub observation_chunk_id: Option<String>,
    /// Whether this record already has causal parents in durable truth.
    pub has_causal_parents: bool,
    /// Whether this record is already a source for downstream causal children.
    pub has_causal_children: bool,
    /// Durable schema-compression lineage preserved for source and schema rows.
    pub compression: CompressionMetadata,
    /// Cross-agent sharing envelope preserved for later namespace and visibility mediation.
    pub sharing: SharingMetadata,
}

/// Stable identifier for persisted memories used by exact and recent Tier1 lookups.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub struct MemoryId(pub u64);

/// Stable identifier for one named time-travel snapshot.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub struct SnapshotId(pub u64);

/// Retention semantics attached to one named snapshot handle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SnapshotRetentionClass {
    /// Ordinary historical anchor that can be deleted when policy allows.
    Standard,
    /// Restorable safety anchor that must not be silently dropped when it is the last active one.
    Restorable,
}

impl SnapshotRetentionClass {
    /// Returns the stable machine-readable retention label.
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Standard => "standard",
            Self::Restorable => "restorable",
        }
    }

    /// Returns whether deletion must preserve at least one active anchor in scope.
    pub const fn requires_anchor_protection(self) -> bool {
        matches!(self, Self::Restorable)
    }
}

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

    /// Returns a stable display label for operator-facing surfaces.
    pub fn display_label(&self) -> String {
        match self {
            Self::Named { snapshot_name, .. } => format!("snapshot:{snapshot_name}"),
            Self::Tick { as_of_tick } => format!("tick:{as_of_tick}"),
        }
    }
}

/// Stable category names surfaced by semantic diff outputs.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SemanticDiffCategory {
    New,
    Strengthened,
    Weakened,
    Archived,
    Conflicting,
    DerivedState,
}

impl SemanticDiffCategory {
    /// Returns the stable machine-readable category label.
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::New => "new",
            Self::Strengthened => "strengthened",
            Self::Weakened => "weakened",
            Self::Archived => "archived",
            Self::Conflicting => "conflicting",
            Self::DerivedState => "derived_state",
        }
    }

    /// Returns a stable explanation phrase for operator-facing inspect surfaces.
    pub const fn explanation_phrase(self) -> &'static str {
        match self {
            Self::New => "new evidence or facts became visible",
            Self::Strengthened => "existing evidence gained stronger support or visibility",
            Self::Weakened => "existing evidence lost support, confidence, or availability",
            Self::Archived => {
                "previously available state moved behind archival or deletion boundaries"
            }
            Self::Conflicting => {
                "the compared interval surfaced unresolved contradiction or rejection evidence"
            }
            Self::DerivedState => {
                "the compared interval changed a derived or maintenance-backed state"
            }
        }
    }
}

/// Directional change between two anchored historical states.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct SemanticDiffEntry {
    /// Stable diff category for downstream formatting.
    pub category: SemanticDiffCategory,
    /// Human-readable bounded summary of the detected change.
    pub summary: String,
    /// Optional memory identity linked to the change when evidence names one.
    pub memory_id: Option<MemoryId>,
    /// Optional audit kind that supplied the evidence for this diff row.
    pub audit_kind: Option<AuditEventKind>,
    /// First anchor where the change was absent.
    pub before_anchor: SnapshotAnchor,
    /// Second anchor where the change became visible.
    pub after_anchor: SnapshotAnchor,
    /// Whether the diff row reflects unresolved conflict rather than consensus.
    pub unresolved: bool,
}

/// Structured semantic diff artifact comparing two historical anchors.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct SemanticDiff {
    /// Namespace the compared anchors belong to.
    pub namespace: NamespaceId,
    /// Earlier anchor used as the baseline.
    pub before_anchor: SnapshotAnchor,
    /// Later anchor used as the comparison target.
    pub after_anchor: SnapshotAnchor,
    /// Stable category counts across all surfaced diff rows.
    pub category_counts: Vec<(SemanticDiffCategory, usize)>,
    /// Bounded inspectable diff rows ordered by category and tick.
    pub entries: Vec<SemanticDiffEntry>,
    /// Explicit caveat describing what this artifact does not prove.
    pub caution: &'static str,
}

impl SemanticDiff {
    /// Builds the inspectable semantic-diff surface used by wrappers and explanation layers.
    pub fn inspect_surface(&self) -> SemanticDiffInspectSurface {
        let before_anchor_label = self.before_anchor.display_label();
        let after_anchor_label = self.after_anchor.display_label();
        let category_counts = self
            .category_counts
            .iter()
            .map(|(category, count)| (category.as_str().to_string(), *count))
            .collect::<Vec<_>>();
        let changed_rows = self
            .entries
            .iter()
            .map(|entry| SemanticDiffInspectRow {
                category: entry.category,
                category_name: entry.category.as_str().to_string(),
                summary: entry.summary.clone(),
                explanation: format!(
                    "{} between {} and {}: {}",
                    entry.category.explanation_phrase(),
                    entry.before_anchor.display_label(),
                    entry.after_anchor.display_label(),
                    entry.summary
                ),
                memory_id: entry.memory_id,
                audit_kind: entry.audit_kind,
                before_anchor_label: entry.before_anchor.display_label(),
                after_anchor_label: entry.after_anchor.display_label(),
                unresolved: entry.unresolved,
            })
            .collect::<Vec<_>>();

        SemanticDiffInspectSurface {
            namespace: self.namespace.clone(),
            before_anchor: self.before_anchor.clone(),
            after_anchor: self.after_anchor.clone(),
            before_anchor_label: before_anchor_label.clone(),
            after_anchor_label: after_anchor_label.clone(),
            category_counts,
            explanation_summary: semantic_diff_explanation_summary(
                &before_anchor_label,
                &after_anchor_label,
                &self.category_counts,
                changed_rows.len(),
            ),
            changed_rows,
            caution: self.caution,
        }
    }
}

/// One inspectable semantic-diff row with a stable explanation string.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct SemanticDiffInspectRow {
    pub category: SemanticDiffCategory,
    pub category_name: String,
    pub summary: String,
    pub explanation: String,
    pub memory_id: Option<MemoryId>,
    pub audit_kind: Option<AuditEventKind>,
    pub before_anchor_label: String,
    pub after_anchor_label: String,
    pub unresolved: bool,
}

/// Inspectable and explainable semantic-diff surface for wrapper callers.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct SemanticDiffInspectSurface {
    pub namespace: NamespaceId,
    pub before_anchor: SnapshotAnchor,
    pub after_anchor: SnapshotAnchor,
    pub before_anchor_label: String,
    pub after_anchor_label: String,
    pub category_counts: Vec<(String, usize)>,
    pub changed_rows: Vec<SemanticDiffInspectRow>,
    pub explanation_summary: String,
    pub caution: &'static str,
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
    /// Logical tick when the snapshot metadata row was created.
    pub created_at_tick: u64,
    /// Optional operator note attached to the named historical anchor.
    pub note: Option<String>,
    /// Number of visible namespace-scoped memories counted at capture time.
    pub memory_count: u64,
    /// Retention class controlling how eagerly the handle may be deleted.
    pub retention_class: SnapshotRetentionClass,
    /// Whether the snapshot is still active for historical recall.
    pub active: bool,
}

fn semantic_diff_explanation_summary(
    before_anchor_label: &str,
    after_anchor_label: &str,
    category_counts: &[(SemanticDiffCategory, usize)],
    changed_rows: usize,
) -> String {
    let counts = category_counts
        .iter()
        .map(|(category, count)| format!("{}={}", category.as_str(), count))
        .collect::<Vec<_>>()
        .join(", ");
    format!(
        "semantic diff from {} to {} surfaced {} changed row(s); category totals: {}",
        before_anchor_label, after_anchor_label, changed_rows, counts
    )
}

impl SnapshotMetadata {
    /// Builds additive metadata for one active named snapshot.
    pub fn named(
        snapshot_id: SnapshotId,
        namespace: NamespaceId,
        snapshot_name: impl Into<String>,
        as_of_tick: u64,
    ) -> Self {
        Self::captured(
            snapshot_id,
            namespace,
            snapshot_name,
            as_of_tick,
            as_of_tick,
            None,
            0,
            SnapshotRetentionClass::Standard,
        )
    }

    /// Builds one explicit captured snapshot with retention and bounded storage metadata.
    #[allow(clippy::too_many_arguments)]
    pub fn captured(
        snapshot_id: SnapshotId,
        namespace: NamespaceId,
        snapshot_name: impl Into<String>,
        as_of_tick: u64,
        created_at_tick: u64,
        note: Option<String>,
        memory_count: u64,
        retention_class: SnapshotRetentionClass,
    ) -> Self {
        Self {
            snapshot_id,
            namespace,
            snapshot_name: snapshot_name.into(),
            as_of_tick,
            created_at_tick,
            note,
            memory_count,
            retention_class,
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

    /// Returns whether this snapshot is protected as a restorable safety anchor.
    pub const fn is_restorable(&self) -> bool {
        self.retention_class.requires_anchor_protection()
    }

    /// Marks the snapshot as deleted while preserving its durable identity.
    pub fn deleted(mut self) -> Self {
        self.active = false;
        self
    }
}

#[cfg(test)]
mod tests {
    use super::{
        SemanticDiff, SemanticDiffCategory, SemanticDiffEntry, SharingMetadata, SnapshotAnchor,
        SnapshotId, SnapshotMetadata, SnapshotRetentionClass,
    };
    use crate::api::{AgentId, NamespaceId, WorkspaceId};
    use crate::observability::AuditEventKind;
    use crate::policy::SharingVisibility;
    use crate::types::MemoryId;

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
        let metadata =
            SnapshotMetadata::named(SnapshotId(9), namespace, "pre_repair", 64).deleted();

        assert_eq!(metadata.snapshot_id, SnapshotId(9));
        assert_eq!(metadata.snapshot_name, "pre_repair");
        assert_eq!(metadata.as_of_tick, 64);
        assert_eq!(metadata.created_at_tick, 64);
        assert_eq!(metadata.memory_count, 0);
        assert_eq!(metadata.retention_class, SnapshotRetentionClass::Standard);
        assert!(!metadata.active);
    }

    #[test]
    fn captured_snapshot_metadata_preserves_note_memory_count_and_retention() {
        let namespace = NamespaceId::new("tests.snapshot").unwrap();
        let metadata = SnapshotMetadata::captured(
            SnapshotId(11),
            namespace,
            "before_refactor",
            120,
            123,
            Some("pre-change safety anchor".to_string()),
            7,
            SnapshotRetentionClass::Restorable,
        );

        assert_eq!(metadata.snapshot_id, SnapshotId(11));
        assert_eq!(metadata.snapshot_name, "before_refactor");
        assert_eq!(metadata.as_of_tick, 120);
        assert_eq!(metadata.created_at_tick, 123);
        assert_eq!(metadata.note.as_deref(), Some("pre-change safety anchor"));
        assert_eq!(metadata.memory_count, 7);
        assert_eq!(metadata.retention_class, SnapshotRetentionClass::Restorable);
        assert!(metadata.is_restorable());
        assert!(metadata.active);
    }

    #[test]
    fn tick_anchor_reports_no_snapshot_name() {
        let anchor = SnapshotAnchor::Tick { as_of_tick: 99 };

        assert_eq!(anchor.kind(), "tick");
        assert_eq!(anchor.as_of_tick(), 99);
        assert_eq!(anchor.snapshot_name(), None);
    }

    #[test]
    fn sharing_metadata_preserves_visibility_workspace_and_agent_scope() {
        let sharing = SharingMetadata::new(SharingVisibility::Shared)
            .with_workspace_id(WorkspaceId::new("ws.alpha"))
            .with_agent_id(AgentId::new("agent.writer"));

        assert_eq!(sharing.visibility.as_str(), "shared");
        assert_eq!(sharing.workspace_id.unwrap().as_str(), "ws.alpha");
        assert_eq!(sharing.agent_id.unwrap().as_str(), "agent.writer");
    }

    #[test]
    fn snapshot_anchor_display_label_is_stable() {
        let named = SnapshotAnchor::Named {
            snapshot_id: SnapshotId(3),
            snapshot_name: "baseline".to_string(),
            as_of_tick: 44,
        };
        let tick = SnapshotAnchor::Tick { as_of_tick: 91 };

        assert_eq!(named.display_label(), "snapshot:baseline");
        assert_eq!(tick.display_label(), "tick:91");
    }

    #[test]
    fn semantic_diff_category_labels_are_stable() {
        assert_eq!(SemanticDiffCategory::New.as_str(), "new");
        assert_eq!(SemanticDiffCategory::Strengthened.as_str(), "strengthened");
        assert_eq!(SemanticDiffCategory::Weakened.as_str(), "weakened");
        assert_eq!(SemanticDiffCategory::Archived.as_str(), "archived");
        assert_eq!(SemanticDiffCategory::Conflicting.as_str(), "conflicting");
        assert_eq!(SemanticDiffCategory::DerivedState.as_str(), "derived_state");
    }

    #[test]
    fn semantic_diff_preserves_entries_and_category_counts() {
        let namespace = NamespaceId::new("tests.semantic_diff").unwrap();
        let before_anchor = SnapshotAnchor::Tick { as_of_tick: 10 };
        let after_anchor = SnapshotAnchor::Named {
            snapshot_id: SnapshotId(8),
            snapshot_name: "after".to_string(),
            as_of_tick: 20,
        };
        let entry = SemanticDiffEntry {
            category: SemanticDiffCategory::Conflicting,
            summary: "conflicting belief surfaced for memory=7".to_string(),
            memory_id: Some(MemoryId(7)),
            audit_kind: Some(AuditEventKind::EncodeRejected),
            before_anchor: before_anchor.clone(),
            after_anchor: after_anchor.clone(),
            unresolved: true,
        };
        let diff = SemanticDiff {
            namespace,
            before_anchor,
            after_anchor,
            category_counts: vec![(SemanticDiffCategory::Conflicting, 1)],
            entries: vec![entry.clone()],
            caution: "semantic diff summarizes bounded historical evidence and does not prove consensus or truth",
        };

        assert_eq!(diff.entries, vec![entry]);
        assert_eq!(
            diff.category_counts,
            vec![(SemanticDiffCategory::Conflicting, 1)]
        );
        assert!(diff.caution.contains("does not prove consensus or truth"));
    }

    #[test]
    fn semantic_diff_inspect_surface_exposes_changed_rows_and_explanations() {
        let namespace = NamespaceId::new("tests.semantic_diff").unwrap();
        let before_anchor = SnapshotAnchor::Tick { as_of_tick: 10 };
        let after_anchor = SnapshotAnchor::Named {
            snapshot_id: SnapshotId(8),
            snapshot_name: "after".to_string(),
            as_of_tick: 20,
        };
        let diff = SemanticDiff {
            namespace: namespace.clone(),
            before_anchor: before_anchor.clone(),
            after_anchor: after_anchor.clone(),
            category_counts: vec![
                (SemanticDiffCategory::New, 1),
                (SemanticDiffCategory::Conflicting, 1),
            ],
            entries: vec![
                SemanticDiffEntry {
                    category: SemanticDiffCategory::New,
                    summary: "fresh supporting fact appeared".to_string(),
                    memory_id: Some(MemoryId(4)),
                    audit_kind: Some(AuditEventKind::EncodeAccepted),
                    before_anchor: before_anchor.clone(),
                    after_anchor: after_anchor.clone(),
                    unresolved: false,
                },
                SemanticDiffEntry {
                    category: SemanticDiffCategory::Conflicting,
                    summary: "conflicting belief surfaced for memory=7".to_string(),
                    memory_id: Some(MemoryId(7)),
                    audit_kind: Some(AuditEventKind::EncodeRejected),
                    before_anchor: before_anchor.clone(),
                    after_anchor: after_anchor.clone(),
                    unresolved: true,
                },
            ],
            caution: "semantic diff summarizes bounded historical evidence and does not prove consensus or truth",
        };

        let inspect = diff.inspect_surface();

        assert_eq!(inspect.namespace, namespace);
        assert_eq!(inspect.before_anchor_label, "tick:10");
        assert_eq!(inspect.after_anchor_label, "snapshot:after");
        assert_eq!(
            inspect.category_counts,
            vec![("new".to_string(), 1), ("conflicting".to_string(), 1)]
        );
        assert_eq!(inspect.changed_rows.len(), 2);
        assert!(inspect.changed_rows.iter().any(|row| {
            row.category_name == "conflicting"
                && row.unresolved
                && row
                    .explanation
                    .contains("unresolved contradiction or rejection evidence")
        }));
        assert!(inspect
            .explanation_summary
            .contains("semantic diff from tick:10 to snapshot:after surfaced 2 changed row(s)"));
    }
}

/// Stable identifier for one session-local hot window.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub struct SessionId(pub u64);

/// Stable identity handle for one persisted blackboard snapshot artifact.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub struct BlackboardSnapshotId(pub u64);

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

/// Compact evidence handle promoted into visible working-state surfaces.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct BlackboardEvidenceHandle {
    /// Durable memory id referenced by the working-state projection.
    pub memory_id: MemoryId,
    /// Machine-readable role for this evidence in the active task.
    pub role: String,
    /// Whether the evidence is pinned in the visible blackboard.
    pub pinned: bool,
}

impl BlackboardEvidenceHandle {
    /// Builds a new blackboard evidence handle with one stable role label.
    pub fn new(memory_id: MemoryId, role: impl Into<String>) -> Self {
        Self {
            memory_id,
            role: role.into(),
            pinned: false,
        }
    }

    /// Marks this evidence handle as pinned in the blackboard projection.
    pub fn pinned(mut self) -> Self {
        self.pinned = true;
        self
    }
}

/// Visible working-state projection for one active task or session.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct BlackboardState {
    /// Namespace owning the active work.
    pub namespace: NamespaceId,
    /// Optional governing task identity for this projection.
    pub task_id: Option<TaskId>,
    /// Optional session identity associated with the active work.
    pub session_id: Option<SessionId>,
    /// Current primary goal shown in the blackboard.
    pub current_goal: String,
    /// Ordered active subgoals visible for steering and handoff.
    pub subgoals: Vec<String>,
    /// Selected evidence handles promoted into active working state.
    pub active_evidence: Vec<BlackboardEvidenceHandle>,
    /// Compact active beliefs surfaced for inspect and handoff.
    pub active_beliefs: Vec<String>,
    /// Explicit unknowns preserved instead of guessed away.
    pub unknowns: Vec<String>,
    /// Next intended bounded action, when one is known.
    pub next_action: Option<String>,
    /// Explicit blocked reason when progress is currently impeded.
    pub blocked_reason: Option<String>,
    /// Stable label proving this object is a projection, not durable truth.
    pub projection_kind: &'static str,
    /// Stable label naming where authoritative truth still lives.
    pub authoritative_truth: &'static str,
}

impl BlackboardState {
    /// Builds a new visible working-state projection for one task or session.
    pub fn new(
        namespace: NamespaceId,
        task_id: Option<TaskId>,
        session_id: Option<SessionId>,
        current_goal: impl Into<String>,
    ) -> Self {
        Self {
            namespace,
            task_id,
            session_id,
            current_goal: current_goal.into(),
            subgoals: Vec::new(),
            active_evidence: Vec::new(),
            active_beliefs: Vec::new(),
            unknowns: Vec::new(),
            next_action: None,
            blocked_reason: None,
            projection_kind: "working_state_projection",
            authoritative_truth: "durable_memory",
        }
    }
}

/// One frame in the bounded active goal stack.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct GoalStackFrame {
    /// Human-readable goal label for the frame.
    pub goal: String,
    /// Optional parent goal label for hierarchy inspection.
    pub parent_goal: Option<String>,
    /// Optional bounded priority within the current stack.
    pub priority: Option<u8>,
    /// Explicit blocked reason for this frame when one exists.
    pub blocked_reason: Option<String>,
}

impl GoalStackFrame {
    /// Builds a new goal-stack frame with one goal label.
    pub fn new(goal: impl Into<String>) -> Self {
        Self {
            goal: goal.into(),
            parent_goal: None,
            priority: None,
            blocked_reason: None,
        }
    }
}

/// Explicit lifecycle state for active resumable work.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum GoalLifecycleStatus {
    Active,
    Dormant,
    Stale,
    Abandoned,
}

impl GoalLifecycleStatus {
    /// Returns the stable machine-readable name for this lifecycle state.
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Active => "active",
            Self::Dormant => "dormant",
            Self::Stale => "stale",
            Self::Abandoned => "abandoned",
        }
    }
}

/// Bounded resumability checkpoint for active work.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct GoalCheckpoint {
    /// Stable checkpoint identifier for inspect and handoff.
    pub checkpoint_id: String,
    /// Logical tick or request-derived anchor where the checkpoint was produced.
    pub created_tick: u64,
    /// Lifecycle state captured by this checkpoint.
    pub status: GoalLifecycleStatus,
    /// Selected evidence handles preserved for resume without copying authority.
    pub evidence_handles: Vec<MemoryId>,
    /// Pending dependency handles or labels required before work can continue.
    pub pending_dependencies: Vec<String>,
    /// Explicit blocked reason preserved in the checkpoint when present.
    pub blocked_reason: Option<String>,
    /// Compact blackboard summary for inspect, resume, and handoff.
    pub blackboard_summary: Option<String>,
    /// Whether the checkpoint is stale relative to the active runtime surface.
    pub stale: bool,
    /// Namespace owning the checkpointed work.
    pub namespace: NamespaceId,
    /// Optional governing task for the checkpoint.
    pub task_id: Option<TaskId>,
    /// Stable label proving the checkpoint is a working-state anchor only.
    pub authoritative_truth: &'static str,
}

/// Persisted blackboard snapshot artifact for inspect, resume, or handoff.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct BlackboardSnapshotArtifact {
    /// Stable snapshot identifier.
    pub snapshot_id: String,
    /// Logical tick or request-derived anchor where the snapshot was emitted.
    pub created_tick: u64,
    /// Evidence handles referenced by the snapshot.
    pub evidence_handles: Vec<MemoryId>,
    /// Compact human-readable note about the snapshot.
    pub note: Option<String>,
    /// Stable label proving this snapshot is derived and rebuildable.
    pub artifact_kind: &'static str,
    /// Stable label naming where authoritative truth still lives.
    pub authoritative_truth: &'static str,
}
