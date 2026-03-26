//! Semantic comparison inputs and snapshot diff model.
//!
//! This module defines the first comparison layer for semantic diff: how live
//! memory state and named snapshot metadata are normalized into stable,
//! inspectable comparison inputs before any higher-level diffing or rendering
//! runs.

use crate::engine::lease::{LeaseMetadata, LeasePolicy};
use crate::types::{MemoryId, NormalizedMemoryEnvelope, SnapshotMetadata};
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;

/// Inspectable summary of one field-level semantic diff row.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SnapshotFieldInspectRow {
    pub field: ComparisonField,
    pub field_name: String,
    pub change_kind: String,
    pub before: Option<String>,
    pub after: Option<String>,
    pub explanation: String,
}

/// Inspectable surface emitted for structural semantic diff exploration.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SnapshotDiffInspectSurface {
    pub subject_kind: ComparisonSubjectKind,
    pub before_subject_label: String,
    pub before_subject_identity: String,
    pub after_subject_label: String,
    pub after_subject_identity: String,
    pub changed_count: usize,
    pub unchanged_count: usize,
    pub changed_fields: Vec<SnapshotFieldInspectRow>,
    pub unchanged_fields: Vec<String>,
    pub explanation_summary: String,
}

/// Stable kind of object participating in semantic comparison.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ComparisonSubjectKind {
    MemoryState,
    SnapshotState,
}

impl ComparisonSubjectKind {
    /// Returns the stable machine-readable subject label.
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::MemoryState => "memory_state",
            Self::SnapshotState => "snapshot_state",
        }
    }
}

/// Stable field names emitted by normalized semantic comparison and diff.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ComparisonField {
    SubjectLabel,
    SubjectIdentity,
    CanonicalFamily,
    CompactText,
    PayloadSizeBytes,
    NormalizationGeneration,
    ObservationSource,
    ObservationChunkId,
    SharingVisibility,
    SharingWorkspace,
    SharingAgent,
    LeasePolicy,
    FreshnessState,
    HasCausalParents,
    HasCausalChildren,
    SnapshotName,
    SnapshotAsOfTick,
    SnapshotCreatedAtTick,
    SnapshotNote,
    SnapshotMemoryCount,
    SnapshotRetentionClass,
    SnapshotActive,
}

impl ComparisonField {
    /// Returns the stable machine-readable field label.
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::SubjectLabel => "subject_label",
            Self::SubjectIdentity => "subject_identity",
            Self::CanonicalFamily => "canonical_family",
            Self::CompactText => "compact_text",
            Self::PayloadSizeBytes => "payload_size_bytes",
            Self::NormalizationGeneration => "normalization_generation",
            Self::ObservationSource => "observation_source",
            Self::ObservationChunkId => "observation_chunk_id",
            Self::SharingVisibility => "sharing_visibility",
            Self::SharingWorkspace => "sharing_workspace",
            Self::SharingAgent => "sharing_agent",
            Self::LeasePolicy => "lease_policy",
            Self::FreshnessState => "freshness_state",
            Self::HasCausalParents => "has_causal_parents",
            Self::HasCausalChildren => "has_causal_children",
            Self::SnapshotName => "snapshot_name",
            Self::SnapshotAsOfTick => "snapshot_as_of_tick",
            Self::SnapshotCreatedAtTick => "snapshot_created_at_tick",
            Self::SnapshotNote => "snapshot_note",
            Self::SnapshotMemoryCount => "snapshot_memory_count",
            Self::SnapshotRetentionClass => "snapshot_retention_class",
            Self::SnapshotActive => "snapshot_active",
        }
    }
}

/// One stable normalized field carried by a semantic comparison input.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ComparisonFieldValue {
    pub field: ComparisonField,
    pub value: String,
}

impl ComparisonFieldValue {
    /// Builds one normalized comparison field entry.
    pub fn new(field: ComparisonField, value: impl Into<String>) -> Self {
        Self {
            field,
            value: value.into(),
        }
    }
}

/// First-layer normalized input used by semantic diff consumers.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SemanticComparisonInput {
    /// Whether the input represents a memory state or a snapshot state.
    pub subject_kind: ComparisonSubjectKind,
    /// Stable inspect label for human-facing surfaces.
    pub subject_label: String,
    /// Stable subject identity used for pairing and audit.
    pub subject_identity: String,
    /// Ordered normalized comparison fields.
    pub fields: Vec<ComparisonFieldValue>,
}

impl SemanticComparisonInput {
    /// Builds a normalized comparison input from one memory state envelope plus lease metadata.
    pub fn from_memory_state(
        memory_id: MemoryId,
        normalized: &NormalizedMemoryEnvelope,
        lease: LeaseMetadata,
    ) -> Self {
        let subject_label = format!("memory:{}", memory_id.0);
        let subject_identity = subject_label.clone();
        let mut fields = vec![
            ComparisonFieldValue::new(ComparisonField::SubjectLabel, subject_label.clone()),
            ComparisonFieldValue::new(ComparisonField::SubjectIdentity, subject_identity.clone()),
            ComparisonFieldValue::new(
                ComparisonField::CanonicalFamily,
                normalized.memory_type.as_str(),
            ),
            ComparisonFieldValue::new(
                ComparisonField::CompactText,
                normalize_text_field(&normalized.compact_text),
            ),
            ComparisonFieldValue::new(
                ComparisonField::PayloadSizeBytes,
                normalized.payload_size_bytes.to_string(),
            ),
            ComparisonFieldValue::new(
                ComparisonField::NormalizationGeneration,
                normalized.normalization_generation,
            ),
            ComparisonFieldValue::new(
                ComparisonField::ObservationSource,
                optional_str(normalized.observation_source.as_deref()),
            ),
            ComparisonFieldValue::new(
                ComparisonField::ObservationChunkId,
                optional_str(normalized.observation_chunk_id.as_deref()),
            ),
            ComparisonFieldValue::new(
                ComparisonField::SharingVisibility,
                normalized.sharing.visibility.as_str(),
            ),
            ComparisonFieldValue::new(
                ComparisonField::SharingWorkspace,
                optional_str(
                    normalized
                        .sharing
                        .workspace_id
                        .as_ref()
                        .map(|id| id.as_str()),
                ),
            ),
            ComparisonFieldValue::new(
                ComparisonField::SharingAgent,
                optional_str(normalized.sharing.agent_id.as_ref().map(|id| id.as_str())),
            ),
            ComparisonFieldValue::new(ComparisonField::LeasePolicy, lease.lease_policy.as_str()),
            ComparisonFieldValue::new(
                ComparisonField::FreshnessState,
                lease.freshness_state.as_str(),
            ),
            ComparisonFieldValue::new(
                ComparisonField::HasCausalParents,
                bool_str(normalized.has_causal_parents),
            ),
            ComparisonFieldValue::new(
                ComparisonField::HasCausalChildren,
                bool_str(normalized.has_causal_children),
            ),
        ];
        fields.sort_by_key(|entry| entry.field.as_str());
        Self {
            subject_kind: ComparisonSubjectKind::MemoryState,
            subject_label,
            subject_identity,
            fields,
        }
    }

    /// Builds a normalized comparison input from one named snapshot metadata record.
    pub fn from_snapshot_state(snapshot: &SnapshotMetadata) -> Self {
        let subject_label = format!("snapshot:{}", snapshot.snapshot_name);
        let subject_identity = format!("snapshot_id:{}", snapshot.snapshot_id.0);
        let mut fields = vec![
            ComparisonFieldValue::new(ComparisonField::SubjectLabel, subject_label.clone()),
            ComparisonFieldValue::new(ComparisonField::SubjectIdentity, subject_identity.clone()),
            ComparisonFieldValue::new(
                ComparisonField::SnapshotName,
                snapshot.snapshot_name.clone(),
            ),
            ComparisonFieldValue::new(
                ComparisonField::SnapshotAsOfTick,
                snapshot.as_of_tick.to_string(),
            ),
            ComparisonFieldValue::new(
                ComparisonField::SnapshotCreatedAtTick,
                snapshot.created_at_tick.to_string(),
            ),
            ComparisonFieldValue::new(
                ComparisonField::SnapshotNote,
                optional_str(snapshot.note.as_deref()),
            ),
            ComparisonFieldValue::new(
                ComparisonField::SnapshotMemoryCount,
                snapshot.memory_count.to_string(),
            ),
            ComparisonFieldValue::new(
                ComparisonField::SnapshotRetentionClass,
                snapshot.retention_class.as_str(),
            ),
            ComparisonFieldValue::new(ComparisonField::SnapshotActive, bool_str(snapshot.active)),
        ];
        fields.sort_by_key(|entry| entry.field.as_str());
        Self {
            subject_kind: ComparisonSubjectKind::SnapshotState,
            subject_label,
            subject_identity,
            fields,
        }
    }

    /// Returns the normalized value for one stable field name.
    pub fn field_value(&self, field: ComparisonField) -> Option<&str> {
        self.fields
            .iter()
            .find(|entry| entry.field == field)
            .map(|entry| entry.value.as_str())
    }

    /// Returns the stable ordered field names carried by this input.
    pub fn field_names(&self) -> Vec<&'static str> {
        self.fields
            .iter()
            .map(|entry| entry.field.as_str())
            .collect()
    }
}

/// One field-level change between two normalized comparison inputs.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SnapshotFieldDiff {
    pub field: ComparisonField,
    pub before: Option<String>,
    pub after: Option<String>,
}

impl SnapshotFieldDiff {
    /// Returns the stable machine-readable change kind.
    pub const fn change_kind(&self) -> &'static str {
        match (&self.before, &self.after) {
            (None, Some(_)) => "added",
            (Some(_), None) => "removed",
            (Some(_), Some(_)) => "changed",
            (None, None) => "unchanged",
        }
    }
}

/// Deterministic first-layer semantic diff between two normalized inputs.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SnapshotDiff {
    pub subject_kind: ComparisonSubjectKind,
    pub before_subject_label: String,
    pub before_subject_identity: String,
    pub after_subject_label: String,
    pub after_subject_identity: String,
    pub changed_fields: Vec<SnapshotFieldDiff>,
    pub unchanged_fields: Vec<ComparisonField>,
}

impl SnapshotDiff {
    /// Compares two normalized semantic inputs of the same kind.
    pub fn between(
        before: &SemanticComparisonInput,
        after: &SemanticComparisonInput,
    ) -> Result<Self, ComparisonKindMismatch> {
        if before.subject_kind != after.subject_kind {
            return Err(ComparisonKindMismatch {
                before: before.subject_kind,
                after: after.subject_kind,
            });
        }

        let all_fields = before
            .fields
            .iter()
            .map(|entry| entry.field)
            .chain(after.fields.iter().map(|entry| entry.field))
            .collect::<BTreeSet<_>>();

        let mut changed_fields = Vec::new();
        let mut unchanged_fields = Vec::new();

        for field in all_fields {
            let before_value = before.field_value(field).map(str::to_string);
            let after_value = after.field_value(field).map(str::to_string);
            if before_value == after_value {
                unchanged_fields.push(field);
            } else {
                changed_fields.push(SnapshotFieldDiff {
                    field,
                    before: before_value,
                    after: after_value,
                });
            }
        }

        Ok(Self {
            subject_kind: before.subject_kind,
            before_subject_label: before.subject_label.clone(),
            before_subject_identity: before.subject_identity.clone(),
            after_subject_label: after.subject_label.clone(),
            after_subject_identity: after.subject_identity.clone(),
            changed_fields,
            unchanged_fields,
        })
    }

    /// Returns whether any normalized field changed.
    pub fn has_changes(&self) -> bool {
        !self.changed_fields.is_empty()
    }

    /// Returns the ordered stable field labels that changed.
    pub fn changed_field_names(&self) -> Vec<&'static str> {
        self.changed_fields
            .iter()
            .map(|entry| entry.field.as_str())
            .collect()
    }

    /// Builds the inspectable structural diff surface used by callers and wrappers.
    pub fn inspect_surface(&self) -> SnapshotDiffInspectSurface {
        let changed_fields = self
            .changed_fields
            .iter()
            .map(|entry| SnapshotFieldInspectRow {
                field: entry.field,
                field_name: entry.field.as_str().to_string(),
                change_kind: entry.change_kind().to_string(),
                before: entry.before.clone(),
                after: entry.after.clone(),
                explanation: explain_field_change(entry),
            })
            .collect::<Vec<_>>();
        let unchanged_fields = self
            .unchanged_fields
            .iter()
            .map(|field| field.as_str().to_string())
            .collect::<Vec<_>>();

        SnapshotDiffInspectSurface {
            subject_kind: self.subject_kind,
            before_subject_label: self.before_subject_label.clone(),
            before_subject_identity: self.before_subject_identity.clone(),
            after_subject_label: self.after_subject_label.clone(),
            after_subject_identity: self.after_subject_identity.clone(),
            changed_count: changed_fields.len(),
            unchanged_count: unchanged_fields.len(),
            changed_fields,
            unchanged_fields,
            explanation_summary: explain_snapshot_diff_summary(self),
        }
    }
}

/// Error returned when semantic diff compares different subject kinds.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct ComparisonKindMismatch {
    pub before: ComparisonSubjectKind,
    pub after: ComparisonSubjectKind,
}

impl LeasePolicy {
    /// Returns whether this policy is durable enough to preserve as a stable semantic hint.
    pub const fn diff_priority_bucket(self) -> &'static str {
        match self {
            Self::Volatile => "short_lived",
            Self::Normal => "standard",
            Self::Durable => "durable",
            Self::Pinned => "pinned",
        }
    }
}

fn optional_str(value: Option<&str>) -> String {
    value.unwrap_or("").trim().to_string()
}

fn bool_str(value: bool) -> &'static str {
    if value {
        "true"
    } else {
        "false"
    }
}

fn normalize_text_field(value: &str) -> String {
    value.split_whitespace().collect::<Vec<_>>().join(" ")
}

fn explain_snapshot_diff_summary(diff: &SnapshotDiff) -> String {
    if diff.changed_fields.is_empty() {
        return format!(
            "no semantic changes between {} and {}",
            diff.before_subject_label, diff.after_subject_label
        );
    }

    let field_list = diff
        .changed_fields
        .iter()
        .map(|entry| entry.field.as_str())
        .collect::<Vec<_>>()
        .join(", ");
    format!(
        "{} semantic field(s) changed between {} and {}: {}",
        diff.changed_fields.len(),
        diff.before_subject_label,
        diff.after_subject_label,
        field_list
    )
}

fn explain_field_change(entry: &SnapshotFieldDiff) -> String {
    match entry.change_kind() {
        "added" => format!(
            "{} was added with value '{}'",
            entry.field.as_str(),
            entry.after.as_deref().unwrap_or("")
        ),
        "removed" => format!(
            "{} was removed from value '{}'",
            entry.field.as_str(),
            entry.before.as_deref().unwrap_or("")
        ),
        "changed" => format!(
            "{} changed from '{}' to '{}'",
            entry.field.as_str(),
            entry.before.as_deref().unwrap_or(""),
            entry.after.as_deref().unwrap_or("")
        ),
        _ => format!("{} did not change", entry.field.as_str()),
    }
}

#[cfg(test)]
mod tests {
    use super::{
        ComparisonField, ComparisonKindMismatch, ComparisonSubjectKind, SemanticComparisonInput,
        SnapshotDiff,
    };
    use crate::api::{AgentId, NamespaceId, WorkspaceId};
    use crate::engine::lease::{FreshnessState, LeaseMetadata, LeasePolicy};
    use crate::policy::SharingVisibility;
    use crate::types::{
        CompressionMetadata, LandmarkMetadata, MemoryId, NormalizedMemoryEnvelope, RawIntakeKind,
        SharingMetadata, SnapshotId, SnapshotMetadata, SnapshotRetentionClass,
    };

    fn normalized_memory() -> NormalizedMemoryEnvelope {
        NormalizedMemoryEnvelope {
            memory_type: crate::types::CanonicalMemoryType::Observation,
            source_kind: RawIntakeKind::Observation,
            raw_text: "raw body".to_string(),
            compact_text: "  compact   summary  ".to_string(),
            normalization_generation: "norm-v2",
            payload_size_bytes: 128,
            affect: None,
            landmark: LandmarkMetadata::non_landmark(),
            observation_source: Some("stdin".to_string()),
            observation_chunk_id: Some("chunk-1".to_string()),
            has_causal_parents: true,
            has_causal_children: false,
            compression: CompressionMetadata::default(),
            sharing: SharingMetadata::new(SharingVisibility::Shared)
                .with_workspace_id(WorkspaceId::new("ws.alpha"))
                .with_agent_id(AgentId::new("agent.writer")),
        }
    }

    #[test]
    fn memory_comparison_input_normalizes_text_and_policy_fields() {
        let input = SemanticComparisonInput::from_memory_state(
            MemoryId(7),
            &normalized_memory(),
            LeaseMetadata {
                lease_policy: LeasePolicy::Durable,
                freshness_state: FreshnessState::LeaseSensitive,
                lease_expires_at_tick: Some(99),
                last_refreshed_at_tick: 10,
            },
        );

        assert_eq!(input.subject_kind, ComparisonSubjectKind::MemoryState);
        assert_eq!(input.subject_identity, "memory:7");
        assert_eq!(
            input.field_value(ComparisonField::CompactText),
            Some("compact summary")
        );
        assert_eq!(
            input.field_value(ComparisonField::SharingVisibility),
            Some("shared")
        );
        assert_eq!(
            input.field_value(ComparisonField::SharingWorkspace),
            Some("ws.alpha")
        );
        assert_eq!(
            input.field_value(ComparisonField::SharingAgent),
            Some("agent.writer")
        );
        assert_eq!(
            input.field_value(ComparisonField::LeasePolicy),
            Some("durable")
        );
        assert_eq!(
            input.field_value(ComparisonField::FreshnessState),
            Some("lease_sensitive")
        );
    }

    #[test]
    fn snapshot_comparison_input_preserves_named_snapshot_fields() {
        let namespace = NamespaceId::new("team.alpha").unwrap();
        let snapshot = SnapshotMetadata::captured(
            SnapshotId(3),
            namespace,
            "baseline",
            42,
            43,
            Some("pre migration".to_string()),
            17,
            SnapshotRetentionClass::Restorable,
        );

        let input = SemanticComparisonInput::from_snapshot_state(&snapshot);

        assert_eq!(input.subject_kind, ComparisonSubjectKind::SnapshotState);
        assert_eq!(input.subject_identity, "snapshot_id:3");
        assert_eq!(
            input.field_value(ComparisonField::SnapshotName),
            Some("baseline")
        );
        assert_eq!(
            input.field_value(ComparisonField::SnapshotAsOfTick),
            Some("42")
        );
        assert_eq!(
            input.field_value(ComparisonField::SnapshotRetentionClass),
            Some("restorable")
        );
        assert_eq!(
            input.field_value(ComparisonField::SnapshotActive),
            Some("true")
        );
    }

    #[test]
    fn snapshot_diff_reports_changed_fields_deterministically() {
        let namespace = NamespaceId::new("team.alpha").unwrap();
        let before = SnapshotMetadata::captured(
            SnapshotId(8),
            namespace.clone(),
            "baseline",
            20,
            21,
            Some("before patch".to_string()),
            11,
            SnapshotRetentionClass::Standard,
        );
        let after = SnapshotMetadata::captured(
            SnapshotId(8),
            namespace,
            "baseline",
            25,
            26,
            Some("after patch".to_string()),
            14,
            SnapshotRetentionClass::Restorable,
        );

        let diff = SnapshotDiff::between(
            &SemanticComparisonInput::from_snapshot_state(&before),
            &SemanticComparisonInput::from_snapshot_state(&after),
        )
        .unwrap();

        assert!(diff.has_changes());
        assert_eq!(
            diff.changed_field_names(),
            vec![
                "snapshot_as_of_tick",
                "snapshot_created_at_tick",
                "snapshot_note",
                "snapshot_memory_count",
                "snapshot_retention_class",
            ]
        );
        assert!(diff
            .unchanged_fields
            .iter()
            .any(|field| field.as_str() == "snapshot_active"));
        assert_eq!(diff.changed_fields[0].change_kind(), "changed");
    }

    #[test]
    fn snapshot_diff_rejects_mixed_subject_kinds() {
        let namespace = NamespaceId::new("team.alpha").unwrap();
        let snapshot = SnapshotMetadata::named(SnapshotId(1), namespace, "baseline", 9);
        let memory = normalized_memory();

        let error = SnapshotDiff::between(
            &SemanticComparisonInput::from_memory_state(
                MemoryId(1),
                &memory,
                LeaseMetadata::new(LeasePolicy::Normal, 0),
            ),
            &SemanticComparisonInput::from_snapshot_state(&snapshot),
        )
        .unwrap_err();

        assert_eq!(
            error,
            ComparisonKindMismatch {
                before: ComparisonSubjectKind::MemoryState,
                after: ComparisonSubjectKind::SnapshotState,
            }
        );
    }

    #[test]
    fn snapshot_diff_inspect_surface_explains_changed_and_unchanged_fields() {
        let namespace = NamespaceId::new("team.alpha").unwrap();
        let before = SnapshotMetadata::captured(
            SnapshotId(8),
            namespace.clone(),
            "baseline",
            20,
            21,
            Some("before patch".to_string()),
            11,
            SnapshotRetentionClass::Standard,
        );
        let after = SnapshotMetadata::captured(
            SnapshotId(8),
            namespace,
            "baseline",
            25,
            26,
            Some("after patch".to_string()),
            14,
            SnapshotRetentionClass::Restorable,
        );

        let diff = SnapshotDiff::between(
            &SemanticComparisonInput::from_snapshot_state(&before),
            &SemanticComparisonInput::from_snapshot_state(&after),
        )
        .unwrap();
        let inspect = diff.inspect_surface();

        assert_eq!(inspect.before_subject_label, "snapshot:baseline");
        assert_eq!(inspect.after_subject_identity, "snapshot_id:8");
        assert_eq!(inspect.changed_count, 5);
        assert!(inspect
            .changed_fields
            .iter()
            .any(|row| row.field_name == "snapshot_note"
                && row
                    .explanation
                    .contains("changed from 'before patch' to 'after patch'")));
        assert!(inspect
            .unchanged_fields
            .iter()
            .any(|field| *field == "snapshot_active"));
        assert!(inspect
            .explanation_summary
            .contains("semantic field(s) changed between snapshot:baseline and snapshot:baseline"));
    }

    #[test]
    fn lease_policy_priority_bucket_tracks_retention_classes() {
        assert_eq!(LeasePolicy::Volatile.diff_priority_bucket(), "short_lived");
        assert_eq!(LeasePolicy::Normal.diff_priority_bucket(), "standard");
        assert_eq!(LeasePolicy::Durable.diff_priority_bucket(), "durable");
        assert_eq!(LeasePolicy::Pinned.diff_priority_bucket(), "pinned");
    }
}
