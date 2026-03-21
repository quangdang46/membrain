use crate::api::NamespaceId;
use crate::config::RuntimeConfig;
use crate::engine::contradiction::{
    ContradictionError, ContradictionId, ContradictionKind, ContradictionRecord, ContradictionStore,
};
use crate::observability::{
    AdmissionOutcomeKind, EncodeFastPathStage, EncodeFastPathTrace, WorkingMemoryTrace,
};
use crate::policy::{
    IngestMode, ObservationWriteOutcome, PassiveObservationDecision, PolicyGateway,
};
use crate::types::{
    CanonicalMemoryType, FastPathRouteFamily, LandmarkMetadata, LandmarkSignals, MemoryId,
    NormalizedMemoryEnvelope, RawEncodeInput, WorkingMemoryId, WorkingMemoryItem,
};
use xxhash_rust::xxh64::xxh64;

const NORMALIZATION_GENERATION: &str = "normalize-v1";
const FAST_PATH_STAGES: [EncodeFastPathStage; 5] = [
    EncodeFastPathStage::Normalize,
    EncodeFastPathStage::Fingerprint,
    EncodeFastPathStage::ShallowClassify,
    EncodeFastPathStage::ProvisionalSalience,
    EncodeFastPathStage::LandmarkTagging,
];

/// Shared interface for Tier1 encode orchestration owned by `membrain-core`.
pub trait EncodeRuntime {
    /// Returns the maximum Tier1 candidate budget this encode surface may consume.
    fn tier1_candidate_budget(&self, config: RuntimeConfig) -> usize;

    /// Runs the bounded synchronous encode fast path before persistence.
    fn prepare_fast_path(&self, input: RawEncodeInput) -> PreparedEncodeCandidate;
}

/// Provisional route summary selected by bounded shallow classification.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ShallowClassification {
    /// Canonical memory family frozen during normalization.
    pub memory_type: CanonicalMemoryType,
    /// First route family selected from bounded local features.
    pub route_family: FastPathRouteFamily,
}

/// Passive-observation provenance and retention facts exposed to inspect surfaces.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PassiveObservationInspect {
    /// Stable source intake family preserved in normalized provenance.
    pub source_kind: &'static str,
    /// Stable passive-observation write decision.
    pub write_decision: &'static str,
    /// Whether this candidate was admitted through passive-observation capture.
    pub captured_as_observation: bool,
    /// Stable provenance source label when passive observation applies.
    pub observation_source: Option<String>,
    /// Stable provenance chunk label when passive observation applies.
    pub observation_chunk_id: Option<String>,
    /// Stable retention marker for passive-observation artifacts.
    pub retention_marker: &'static str,
}

/// Prepared encode candidate emitted by the synchronous fast path.
#[derive(Debug, Clone, PartialEq)]
pub struct PreparedEncodeCandidate {
    /// Canonical normalized envelope frozen before persistence.
    pub normalized: NormalizedMemoryEnvelope,
    /// Stable duplicate-family fingerprint derived from the normalized form.
    pub fingerprint: u64,
    /// Bounded shallow classification summary.
    pub classification: ShallowClassification,
    /// First-pass salience scalar used for initial routing inputs.
    pub provisional_salience: u16,
    /// Whether this input should be captured, suppressed, or denied before persistence.
    pub write_decision: PassiveObservationDecision,
    /// Whether this candidate was admitted through passive-observation capture.
    pub captured_as_observation: bool,
    /// Structured passive-observation inspect facts for provenance and retention.
    pub passive_observation_inspect: PassiveObservationInspect,
    /// Structured trace proving the ordered fast-path stages.
    pub trace: EncodeFastPathTrace,
}

/// Machine-readable write-path branch taken when a contradiction is detected.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EncodeWriteBranch {
    /// The candidate may continue through the ordinary write path.
    Accepted,
    /// The candidate branched into explicit contradiction recording.
    ContradictionRecorded,
}

/// Result of routing an encode candidate through contradiction-aware write branching.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ContradictionWriteOutcome {
    /// Stable contradiction artifact created for the conflicting pair.
    pub contradiction_id: ContradictionId,
    /// Machine-readable branch taken by the write path.
    pub branch: EncodeWriteBranch,
    /// Existing durable memory preserved as one side of the contradiction.
    pub existing_memory: MemoryId,
    /// Incoming durable memory preserved as the other side of the contradiction.
    pub incoming_memory: MemoryId,
    /// Canonical contradiction relationship recorded for the pair.
    pub kind: ContradictionKind,
}

const LANDMARK_AROUSAL_THRESHOLD: f32 = 0.7;
const LANDMARK_NOVELTY_THRESHOLD: f32 = 0.75;
const LANDMARK_SIMILARITY_FLOOR: f32 = 0.85;
const LANDMARK_MIN_ERA_GAP_TICKS: u64 = 50;
const PASSIVE_OBSERVATION_SOURCE: &str = "passive_observation";
const PASSIVE_OBSERVATION_RETENTION_MARKER: &str = "volatile_observation";

/// Final controller decision for a working-memory admission attempt.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WorkingMemoryAdmission {
    /// The candidate passed into the controller.
    pub item: WorkingMemoryItem,
    /// Final machine-readable admission outcome.
    pub outcome: AdmissionOutcomeKind,
    /// Optional overflow victim that should be promoted to encode.
    pub promoted_item: Option<WorkingMemoryItem>,
    /// Optional overflow victim that was discarded instead of promoted.
    pub evicted_item: Option<WorkingMemoryItem>,
    /// Shared trace artifact describing the decision.
    pub trace: WorkingMemoryTrace,
}

/// Errors returned when working memory cannot admit a candidate safely.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WorkingMemoryError {
    /// All available slots were pinned, so no deterministic eviction path existed.
    AllSlotsPinned,
}

/// Bounded working-memory controller sitting in front of durable encode.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WorkingMemoryController {
    config: RuntimeConfig,
    slots: Vec<WorkingMemoryItem>,
}

impl WorkingMemoryController {
    /// Builds a new bounded working-memory controller from shared runtime config.
    pub fn new(config: RuntimeConfig) -> Self {
        Self {
            config,
            slots: Vec::new(),
        }
    }

    /// Returns the currently buffered controller slots.
    pub fn slots(&self) -> &[WorkingMemoryItem] {
        &self.slots
    }

    /// Attempts to admit a candidate into working memory.
    pub fn admit(
        &mut self,
        item: WorkingMemoryItem,
    ) -> Result<WorkingMemoryAdmission, WorkingMemoryError> {
        let initial_pressure = self.slots.len();

        if item.attention_score < self.config.working_memory_attention_threshold {
            return Ok(WorkingMemoryAdmission {
                item,
                outcome: AdmissionOutcomeKind::Discarded,
                promoted_item: None,
                evicted_item: None,
                trace: WorkingMemoryTrace {
                    outcome: AdmissionOutcomeKind::Discarded,
                    slot_pressure: initial_pressure,
                    threshold: self.config.working_memory_attention_threshold,
                    overflowed: false,
                },
            });
        }

        if self.slots.len() < self.config.working_memory_capacity {
            self.slots.push(item.clone());
            return Ok(WorkingMemoryAdmission {
                item,
                outcome: AdmissionOutcomeKind::Buffered,
                promoted_item: None,
                evicted_item: None,
                trace: WorkingMemoryTrace {
                    outcome: AdmissionOutcomeKind::Buffered,
                    slot_pressure: initial_pressure,
                    threshold: self.config.working_memory_attention_threshold,
                    overflowed: false,
                },
            });
        }

        let evict_index = self
            .slots
            .iter()
            .enumerate()
            .filter(|(_, candidate)| !candidate.pinned)
            .min_by_key(|(_, candidate)| (candidate.attention_score, candidate.id.0))
            .map(|(index, _)| index)
            .ok_or(WorkingMemoryError::AllSlotsPinned)?;

        let evicted = self.slots.remove(evict_index);
        self.slots.push(item.clone());

        let promoted_item = (evicted.attention_score
            >= self.config.working_memory_promote_threshold)
            .then_some(evicted.clone());
        let outcome = if promoted_item.is_some() {
            AdmissionOutcomeKind::Promoted
        } else {
            AdmissionOutcomeKind::Buffered
        };

        Ok(WorkingMemoryAdmission {
            item,
            outcome,
            promoted_item,
            evicted_item: Some(evicted),
            trace: WorkingMemoryTrace {
                outcome,
                slot_pressure: self.slots.len(),
                threshold: self.config.working_memory_promote_threshold,
                overflowed: true,
            },
        })
    }

    /// Raises the attention score for a buffered candidate if it exists.
    pub fn focus(&mut self, id: WorkingMemoryId, delta: u16) -> bool {
        if let Some(item) = self.slots.iter_mut().find(|candidate| candidate.id == id) {
            item.attention_score = item.attention_score.saturating_add(delta);
            true
        } else {
            false
        }
    }
}

/// Canonical encode engine placeholder owned by the core crate.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EncodeEngine {
    working_memory: WorkingMemoryController,
}

impl EncodeEngine {
    /// Builds the encode engine with its bounded working-memory controller.
    pub fn new(config: RuntimeConfig) -> Self {
        Self {
            working_memory: WorkingMemoryController::new(config),
        }
    }

    /// Branches a conflicting write into an explicit contradiction artifact instead of overwrite.
    pub fn record_contradiction_branch(
        &self,
        contradictions: &mut impl ContradictionStore,
        namespace: NamespaceId,
        existing_memory: MemoryId,
        incoming_memory: MemoryId,
        kind: ContradictionKind,
        conflict_score: u16,
    ) -> Result<ContradictionWriteOutcome, ContradictionError> {
        let contradiction_id = contradictions.record(ContradictionRecord::new(
            ContradictionId(0),
            namespace,
            existing_memory,
            incoming_memory,
            kind,
            conflict_score,
        ))?;

        Ok(ContradictionWriteOutcome {
            contradiction_id,
            branch: EncodeWriteBranch::ContradictionRecorded,
            existing_memory,
            incoming_memory,
            kind,
        })
    }

    fn write_gate(
        &self,
        policy: &impl PolicyGateway,
        ingest_mode: IngestMode,
        namespace_bound: bool,
        duplicate_hint: bool,
    ) -> ObservationWriteOutcome {
        policy.evaluate_observation_write(crate::policy::ObservationWriteRequest {
            ingest_mode,
            namespace_bound,
            policy_allowed: namespace_bound,
            duplicate_hint,
        })
    }

    /// Returns the bounded working-memory controller.
    pub fn working_memory(&self) -> &WorkingMemoryController {
        &self.working_memory
    }

    /// Returns the mutable bounded working-memory controller.
    pub fn working_memory_mut(&mut self) -> &mut WorkingMemoryController {
        &mut self.working_memory
    }

    fn normalize_input(&self, input: RawEncodeInput) -> NormalizedMemoryEnvelope {
        let compact_text = input
            .raw_text
            .split_whitespace()
            .collect::<Vec<_>>()
            .join(" ");
        let payload_size_bytes = compact_text.len();

        let landmark = input
            .landmark_signals
            .map(|signals| self.derive_landmark_metadata(&compact_text, signals))
            .unwrap_or_else(LandmarkMetadata::non_landmark);

        let observation_source = matches!(input.kind, crate::types::RawIntakeKind::Observation)
            .then(|| PASSIVE_OBSERVATION_SOURCE.to_string());
        let observation_chunk_id = matches!(input.kind, crate::types::RawIntakeKind::Observation)
            .then(|| format!("obs-{:016x}", xxh64(compact_text.as_bytes(), 0)));

        NormalizedMemoryEnvelope {
            memory_type: input.kind.canonical_memory_type(),
            source_kind: input.kind,
            raw_text: input.raw_text,
            compact_text,
            normalization_generation: NORMALIZATION_GENERATION,
            payload_size_bytes,
            landmark,
            observation_source,
            observation_chunk_id,
        }
    }

    fn fingerprint(&self, normalized: &NormalizedMemoryEnvelope) -> u64 {
        let mut fingerprint_input = Vec::with_capacity(
            normalized.normalization_generation.len()
                + normalized.memory_type.as_str().len()
                + normalized.compact_text.len()
                + 2,
        );
        fingerprint_input.extend_from_slice(normalized.normalization_generation.as_bytes());
        fingerprint_input.push(0xff);
        fingerprint_input.extend_from_slice(normalized.memory_type.as_str().as_bytes());
        fingerprint_input.push(0xff);
        fingerprint_input.extend_from_slice(normalized.compact_text.as_bytes());
        xxh64(&fingerprint_input, 0)
    }

    fn shallow_classify(&self, normalized: &NormalizedMemoryEnvelope) -> ShallowClassification {
        ShallowClassification {
            memory_type: normalized.memory_type,
            route_family: FastPathRouteFamily::from_memory_type(normalized.memory_type),
        }
    }

    fn derive_landmark_metadata(
        &self,
        compact_text: &str,
        signals: LandmarkSignals,
    ) -> LandmarkMetadata {
        let qualifies = signals.arousal >= LANDMARK_AROUSAL_THRESHOLD
            && signals.novelty >= LANDMARK_NOVELTY_THRESHOLD
            && signals.recent_similarity < LANDMARK_SIMILARITY_FLOOR
            && signals.ticks_since_last_landmark >= LANDMARK_MIN_ERA_GAP_TICKS;

        if !qualifies {
            return LandmarkMetadata::non_landmark();
        }

        let label = compact_text
            .split_whitespace()
            .take(8)
            .collect::<Vec<_>>()
            .join(" ");
        let label = if label.is_empty() {
            String::from("landmark")
        } else {
            label
        };
        let era_slug = compact_text
            .chars()
            .filter(|ch| ch.is_ascii_alphanumeric())
            .take(12)
            .collect::<String>()
            .to_ascii_lowercase();
        let era_slug = if era_slug.is_empty() {
            String::from("landmark")
        } else {
            era_slug
        };
        let era_id = format!(
            "era-{}-{:04}",
            era_slug,
            signals.ticks_since_last_landmark.min(9_999)
        );

        LandmarkMetadata {
            is_landmark: true,
            landmark_label: Some(label),
            era_id: Some(era_id),
        }
    }

    fn provisional_salience(
        &self,
        normalized: &NormalizedMemoryEnvelope,
        classification: ShallowClassification,
    ) -> u16 {
        let base: u16 = match classification.route_family {
            FastPathRouteFamily::Event => 350,
            FastPathRouteFamily::Observation => 450,
            FastPathRouteFamily::ToolOutcome => 700,
            FastPathRouteFamily::UserPreference => 850,
            FastPathRouteFamily::SessionMarker => 250,
        };
        let bounded_token_count = normalized.compact_text.split_whitespace().count().min(8) as u16;
        let size_bonus = bounded_token_count.saturating_mul(25).min(200);
        base.saturating_add(size_bonus).min(1_000)
    }

    /// Runs the ordered synchronous encode fast path before persistence.
    pub fn prepare_fast_path(&self, input: RawEncodeInput) -> PreparedEncodeCandidate {
        self.prepare_ingest_candidate(input, IngestMode::Active, true, false)
    }

    /// Runs the bounded encode fast path with explicit ingest-mode write gating.
    pub fn prepare_ingest_candidate(
        &self,
        input: RawEncodeInput,
        ingest_mode: IngestMode,
        namespace_bound: bool,
        duplicate_hint: bool,
    ) -> PreparedEncodeCandidate {
        let landmark_signals = input.landmark_signals;
        let normalized = self.normalize_input(input);
        let fingerprint = self.fingerprint(&normalized);
        let classification = self.shallow_classify(&normalized);
        let provisional_salience = self.provisional_salience(&normalized, classification);
        let write_gate = self.write_gate(
            &crate::policy::PolicyModule,
            ingest_mode,
            namespace_bound,
            duplicate_hint,
        );
        let passive_observation_inspect = PassiveObservationInspect {
            source_kind: normalized.source_kind.as_str(),
            write_decision: write_gate.decision.as_str(),
            captured_as_observation: write_gate.captured_as_observation,
            observation_source: normalized.observation_source.clone(),
            observation_chunk_id: normalized.observation_chunk_id.clone(),
            retention_marker: if matches!(ingest_mode, IngestMode::PassiveObservation) {
                PASSIVE_OBSERVATION_RETENTION_MARKER
            } else {
                "absent"
            },
        };
        let trace = EncodeFastPathTrace {
            stages: FAST_PATH_STAGES,
            normalization_generation: normalized.normalization_generation,
            memory_type: normalized.memory_type,
            route_family: classification.route_family,
            provisional_salience,
            duplicate_hint_candidate_count: usize::from(matches!(
                write_gate.decision,
                PassiveObservationDecision::Suppress
            )),
            landmark_signals,
            landmark: normalized.landmark.clone(),
            stayed_within_latency_budget: true,
        };

        PreparedEncodeCandidate {
            normalized,
            fingerprint,
            classification,
            provisional_salience,
            write_decision: write_gate.decision,
            captured_as_observation: write_gate.captured_as_observation,
            passive_observation_inspect,
            trace,
        }
    }
}

impl Default for EncodeEngine {
    fn default() -> Self {
        Self::new(RuntimeConfig::default())
    }
}

impl EncodeRuntime for EncodeEngine {
    fn tier1_candidate_budget(&self, config: RuntimeConfig) -> usize {
        config.tier1_candidate_budget
    }

    fn prepare_fast_path(&self, input: RawEncodeInput) -> PreparedEncodeCandidate {
        EncodeEngine::prepare_fast_path(self, input)
    }
}

#[cfg(test)]
mod tests {
    use super::{EncodeEngine, EncodeWriteBranch};
    use crate::api::NamespaceId;
    use crate::engine::contradiction::{
        ContradictionEngine, ContradictionError, ContradictionKind, ContradictionStore,
    };
    use crate::policy::IngestMode;
    use crate::policy::PassiveObservationDecision;
    use crate::types::{LandmarkSignals, MemoryId, RawEncodeInput, RawIntakeKind};
    use crate::RuntimeConfig;

    fn test_engine() -> EncodeEngine {
        EncodeEngine::new(RuntimeConfig::default())
    }

    fn test_namespace() -> NamespaceId {
        NamespaceId::new("tests/encode").unwrap()
    }

    #[test]
    fn active_ingest_capture_and_denial_remain_explicit() {
        let engine = test_engine();

        let captured = engine.prepare_ingest_candidate(
            RawEncodeInput::new(RawIntakeKind::Event, "active ingest"),
            IngestMode::Active,
            true,
            false,
        );
        let denied = engine.prepare_ingest_candidate(
            RawEncodeInput::new(RawIntakeKind::Event, "denied active ingest"),
            IngestMode::Active,
            false,
            false,
        );

        assert_eq!(captured.write_decision, PassiveObservationDecision::Capture);
        assert!(!captured.captured_as_observation);
        assert_eq!(denied.write_decision, PassiveObservationDecision::Deny);
        assert!(!denied.captured_as_observation);
    }

    #[test]
    fn passive_observation_capture_and_suppression_remain_distinct() {
        let engine = test_engine();

        let captured = engine.prepare_ingest_candidate(
            RawEncodeInput::new(RawIntakeKind::Observation, "fresh passive signal"),
            IngestMode::PassiveObservation,
            true,
            false,
        );
        let suppressed = engine.prepare_ingest_candidate(
            RawEncodeInput::new(RawIntakeKind::Observation, "passive duplicate hint"),
            IngestMode::PassiveObservation,
            true,
            true,
        );

        assert_eq!(captured.write_decision, PassiveObservationDecision::Capture);
        assert!(captured.captured_as_observation);
        assert_eq!(
            suppressed.write_decision,
            PassiveObservationDecision::Suppress
        );
        assert!(!suppressed.captured_as_observation);
        assert_eq!(suppressed.trace.duplicate_hint_candidate_count, 1);
    }

    #[test]
    fn passive_observation_preserves_tier2_provenance_fields() {
        let engine = test_engine();

        let prepared = engine.prepare_ingest_candidate(
            RawEncodeInput::new(
                RawIntakeKind::Observation,
                "camera summary noted a recurring hallway blockage",
            ),
            IngestMode::PassiveObservation,
            true,
            false,
        );

        assert_eq!(
            prepared.normalized.observation_source.as_deref(),
            Some("passive_observation")
        );
        assert_eq!(prepared.write_decision, PassiveObservationDecision::Capture);
        assert!(prepared.captured_as_observation);
        assert_eq!(
            prepared.passive_observation_inspect.source_kind,
            "observation"
        );
        assert_eq!(
            prepared.passive_observation_inspect.write_decision,
            PassiveObservationDecision::Capture.as_str()
        );
        assert!(prepared.passive_observation_inspect.captured_as_observation);
        assert_eq!(
            prepared.passive_observation_inspect.retention_marker,
            "volatile_observation"
        );
        assert_eq!(
            prepared
                .passive_observation_inspect
                .observation_source
                .as_deref(),
            prepared.normalized.observation_source.as_deref()
        );
        assert_eq!(
            prepared
                .passive_observation_inspect
                .observation_chunk_id
                .as_deref(),
            prepared.normalized.observation_chunk_id.as_deref()
        );
        assert!(prepared
            .normalized
            .observation_chunk_id
            .as_deref()
            .is_some_and(|chunk_id| chunk_id.starts_with("obs-")));
    }

    #[test]
    fn landmark_metadata_is_additive_when_signals_clear_thresholds() {
        let engine = test_engine();

        let prepared = engine.prepare_fast_path(
            RawEncodeInput::new(RawIntakeKind::Event, "project launch deadline was moved")
                .with_landmark_signals(LandmarkSignals::new(0.91, 0.83, 0.31, 88)),
        );

        assert!(prepared.normalized.landmark.is_landmark);
        assert_eq!(
            prepared.trace.landmark_signals,
            Some(LandmarkSignals::new(0.91, 0.83, 0.31, 88))
        );
        assert_eq!(prepared.trace.landmark, prepared.normalized.landmark);
        assert!(prepared
            .normalized
            .landmark
            .landmark_label
            .as_ref()
            .is_some_and(|label| label.contains("project launch deadline was moved")));
        assert!(prepared
            .normalized
            .landmark
            .era_id
            .as_ref()
            .is_some_and(|era| era.starts_with("era-projectlaunc-0088")));
    }

    #[test]
    fn landmark_metadata_stays_non_authoritative_when_signals_do_not_qualify() {
        let engine = test_engine();

        let prepared = engine.prepare_fast_path(
            RawEncodeInput::new(RawIntakeKind::Event, "routine standup note")
                .with_landmark_signals(LandmarkSignals::new(0.62, 0.91, 0.20, 88)),
        );

        assert!(!prepared.normalized.landmark.is_landmark);
        assert_eq!(prepared.normalized.landmark.landmark_label, None);
        assert_eq!(prepared.normalized.landmark.era_id, None);
        assert_eq!(
            prepared.trace.landmark_signals,
            Some(LandmarkSignals::new(0.62, 0.91, 0.20, 88))
        );
    }

    #[test]
    fn contradiction_branch_records_explicit_artifact() {
        let engine = test_engine();
        let mut contradictions = ContradictionEngine::new();

        let outcome = engine
            .record_contradiction_branch(
                &mut contradictions,
                test_namespace(),
                MemoryId(41),
                MemoryId(42),
                ContradictionKind::Supersession,
                777,
            )
            .unwrap();

        assert_eq!(outcome.branch, EncodeWriteBranch::ContradictionRecorded);
        assert_eq!(outcome.existing_memory, MemoryId(41));
        assert_eq!(outcome.incoming_memory, MemoryId(42));
        assert_eq!(contradictions.find_by_memory(MemoryId(41)).len(), 1);
    }

    #[test]
    fn contradiction_branch_does_not_silently_duplicate_existing_pair() {
        let engine = test_engine();
        let mut contradictions = ContradictionEngine::new();

        engine
            .record_contradiction_branch(
                &mut contradictions,
                test_namespace(),
                MemoryId(41),
                MemoryId(42),
                ContradictionKind::Revision,
                650,
            )
            .unwrap();

        let duplicate = engine.record_contradiction_branch(
            &mut contradictions,
            test_namespace(),
            MemoryId(42),
            MemoryId(41),
            ContradictionKind::Duplicate,
            650,
        );

        assert_eq!(duplicate.unwrap_err(), ContradictionError::DuplicateRecord);
    }
}
