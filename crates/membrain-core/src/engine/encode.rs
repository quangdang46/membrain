use crate::api::NamespaceId;
use crate::config::RuntimeConfig;
use crate::engine::contradiction::{
    ContradictionCandidate, ContradictionError, ContradictionId, ContradictionKind,
    ContradictionRecord, ContradictionStore, DetectionResult,
};
use crate::observability::{
    AdmissionOutcomeKind, EncodeFastPathStage, EncodeFastPathTrace, WorkingMemoryTrace,
};
use crate::policy::{
    IngestMode, ObservationWriteOutcome, PassiveObservationDecision, PolicyGateway,
};
use crate::types::{
    CanonicalMemoryType, CompressionMetadata, FastPathRouteFamily, LandmarkMetadata,
    LandmarkSignals, MemoryId, NormalizedMemoryEnvelope, RawEncodeInput, SharingMetadata,
    WorkingMemoryId, WorkingMemoryItem,
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

/// Outcome of the integrated detect-and-branch write path.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum WriteBranchOutcome {
    /// No contradiction detected; write may proceed normally.
    Accepted {
        /// Trace proving the branch decision and detection inputs.
        trace: ContradictionWriteBranchTrace,
    },
    /// A contradiction was detected and recorded; write branches into conflict storage.
    ContradictionRecorded {
        /// The contradiction write outcome with record details.
        outcome: ContradictionWriteOutcome,
        /// Trace proving the branch decision and detection inputs.
        trace: ContradictionWriteBranchTrace,
    },
}

impl WriteBranchOutcome {
    /// Returns whether the write was accepted without contradiction.
    pub const fn is_accepted(&self) -> bool {
        matches!(self, Self::Accepted { .. })
    }

    /// Returns whether a contradiction was detected and recorded.
    pub const fn is_contradiction(&self) -> bool {
        matches!(self, Self::ContradictionRecorded { .. })
    }

    /// Returns the inner contradiction outcome if one was recorded.
    pub fn contradiction_outcome(&self) -> Option<&ContradictionWriteOutcome> {
        match self {
            Self::ContradictionRecorded { outcome, .. } => Some(outcome),
            Self::Accepted { .. } => None,
        }
    }

    /// Returns the trace artifact proving which branch was taken.
    pub const fn trace(&self) -> &ContradictionWriteBranchTrace {
        match self {
            Self::Accepted { trace } | Self::ContradictionRecorded { trace, .. } => trace,
        }
    }
}

/// Machine-readable trace proving which branch the write path took and why.
///
/// This artifact satisfies the acceptance requirement that "logs show the branch
/// taken when a contradiction is detected." It is produced by every call to
/// `detect_and_branch` or `detect_all_and_branch` so callers can record or
/// inspect the decision without re-deriving it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ContradictionWriteBranchTrace {
    /// Which branch the write path selected.
    pub branch: EncodeWriteBranch,
    /// Number of candidate memories consulted during detection.
    pub candidates_examined: usize,
    /// The contradiction kind if a conflict was detected, `None` when accepted.
    pub detected_kind: Option<ContradictionKind>,
    /// Conflict score in 0..1000 if a conflict was detected.
    pub conflict_score: Option<u16>,
    /// The existing memory that conflicted (if any).
    pub existing_memory: Option<MemoryId>,
    /// The incoming memory being written.
    pub incoming_memory: MemoryId,
}

impl ContradictionWriteBranchTrace {
    /// Builds a trace for an accepted (no-conflict) write.
    pub fn accepted(candidates_examined: usize, incoming_memory: MemoryId) -> Self {
        Self {
            branch: EncodeWriteBranch::Accepted,
            candidates_examined,
            detected_kind: None,
            conflict_score: None,
            existing_memory: None,
            incoming_memory,
        }
    }

    /// Builds a trace for a contradiction-recorded write.
    pub fn contradiction(
        candidates_examined: usize,
        existing_memory: MemoryId,
        incoming_memory: MemoryId,
        kind: ContradictionKind,
        conflict_score: u16,
    ) -> Self {
        Self {
            branch: EncodeWriteBranch::ContradictionRecorded,
            candidates_examined,
            detected_kind: Some(kind),
            conflict_score: Some(conflict_score),
            existing_memory: Some(existing_memory),
            incoming_memory,
        }
    }

    /// Returns a stable machine-readable label for the branch taken.
    pub const fn branch_label(&self) -> &'static str {
        match self.branch {
            EncodeWriteBranch::Accepted => "accepted",
            EncodeWriteBranch::ContradictionRecorded => "contradiction_recorded",
        }
    }
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

fn landmark_detection_score(signals: LandmarkSignals) -> u16 {
    let arousal = (signals.arousal.clamp(0.0, 1.0) * 400.0).round() as u16;
    let novelty = (signals.novelty.clamp(0.0, 1.0) * 350.0).round() as u16;
    let dissimilarity = ((1.0 - signals.recent_similarity.clamp(0.0, 1.0)) * 150.0).round() as u16;
    let gap = ((signals.ticks_since_last_landmark.min(200) as f32 / 200.0) * 100.0).round() as u16;
    arousal
        .saturating_add(novelty)
        .saturating_add(dissimilarity)
        .saturating_add(gap)
        .min(1_000)
}

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

    /// Detects contradictions for an incoming candidate and branches into recording if found.
    ///
    /// This is the integrated write-path entry point: it runs contradiction detection
    /// against all indexed memories in the namespace, and if a conflict is found,
    /// records an explicit contradiction artifact instead of silently overwriting.
    ///
    /// Returns `WriteBranchOutcome::Accepted` when no contradiction is detected,
    /// or `WriteBranchOutcome::ContradictionRecorded` when the write branches into
    /// contradiction recording. Both variants carry a `ContradictionWriteBranchTrace`
    /// so callers can log or inspect the decision.
    pub fn detect_and_branch(
        &self,
        contradictions: &mut crate::engine::contradiction::ContradictionEngine,
        namespace: NamespaceId,
        incoming_memory: MemoryId,
        candidate: &ContradictionCandidate,
    ) -> Result<WriteBranchOutcome, ContradictionError> {
        let candidates_examined = contradictions.indexed_memory_count(&candidate.namespace);
        let detection = contradictions.detect(candidate);

        match detection {
            DetectionResult::NoConflict => {
                let trace =
                    ContradictionWriteBranchTrace::accepted(candidates_examined, incoming_memory);
                Ok(WriteBranchOutcome::Accepted { trace })
            }
            DetectionResult::ConflictDetected {
                existing_memory,
                kind,
                conflict_score,
            } => {
                let trace = ContradictionWriteBranchTrace::contradiction(
                    candidates_examined,
                    existing_memory,
                    incoming_memory,
                    kind,
                    conflict_score,
                );
                let outcome = self.record_contradiction_branch(
                    contradictions,
                    namespace,
                    existing_memory,
                    incoming_memory,
                    kind,
                    conflict_score,
                )?;
                Ok(WriteBranchOutcome::ContradictionRecorded { outcome, trace })
            }
        }
    }

    /// Detects all contradictions for an incoming candidate and records each as a separate artifact.
    ///
    /// Unlike `detect_and_branch` which stops at the best match, this method records
    /// contradictions against all conflicting memories. Use this when the write path
    /// must preserve a complete conflict picture rather than just the strongest signal.
    pub fn detect_all_and_branch(
        &self,
        contradictions: &mut crate::engine::contradiction::ContradictionEngine,
        namespace: NamespaceId,
        incoming_memory: MemoryId,
        candidate: &ContradictionCandidate,
    ) -> Result<Vec<ContradictionWriteOutcome>, ContradictionError> {
        let detections = contradictions.detect_all(candidate);
        let mut outcomes = Vec::with_capacity(detections.len());

        for detection in detections {
            if let DetectionResult::ConflictDetected {
                existing_memory,
                kind,
                conflict_score,
            } = detection
            {
                match self.record_contradiction_branch(
                    contradictions,
                    namespace.clone(),
                    existing_memory,
                    incoming_memory,
                    kind,
                    conflict_score,
                ) {
                    Ok(outcome) => outcomes.push(outcome),
                    Err(ContradictionError::DuplicateRecord) => {
                        // Skip duplicate pairs — the contradiction is already recorded
                    }
                    Err(e) => return Err(e),
                }
            }
        }

        Ok(outcomes)
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
        let payload_size_bytes = input.raw_text.len();

        let affect = input
            .affect_signals
            .map(crate::types::AffectSignals::clamped);
        let landmark = self.derive_landmark_metadata(
            &compact_text,
            input.landmark_signals,
            input.active_era_id.as_deref(),
            input.current_tick,
        );

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
            affect,
            landmark,
            observation_source,
            observation_chunk_id,
            has_causal_parents: false,
            has_causal_children: false,
            compression: CompressionMetadata::default(),
            sharing: SharingMetadata::default(),
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
        signals: Option<LandmarkSignals>,
        active_era_id: Option<&str>,
        current_tick: Option<u64>,
    ) -> LandmarkMetadata {
        let Some(signals) = signals else {
            let mut metadata = LandmarkMetadata::non_landmark();
            metadata.era_id = active_era_id.map(str::to_owned);
            return metadata;
        };

        let qualifies = signals.arousal >= LANDMARK_AROUSAL_THRESHOLD
            && signals.novelty >= LANDMARK_NOVELTY_THRESHOLD
            && signals.recent_similarity < LANDMARK_SIMILARITY_FLOOR
            && signals.ticks_since_last_landmark >= LANDMARK_MIN_ERA_GAP_TICKS;

        if !qualifies {
            let mut metadata = LandmarkMetadata::non_landmark();
            metadata.era_id = active_era_id.map(str::to_owned);
            return metadata;
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
        let era_started_at_tick = current_tick.unwrap_or(signals.ticks_since_last_landmark);
        let era_id = format!("era-{}-{:04}", era_slug, era_started_at_tick.min(9_999));
        let detection_score = landmark_detection_score(signals);
        let detection_reason = format!(
            "arousal={:.2} novelty={:.2} recent_similarity={:.2} gap_ticks={} crossed landmark thresholds (arousal>={:.2}, novelty>={:.2}, recent_similarity<={:.2}, gap_ticks>={})",
            signals.arousal,
            signals.novelty,
            signals.recent_similarity,
            signals.ticks_since_last_landmark,
            LANDMARK_AROUSAL_THRESHOLD,
            LANDMARK_NOVELTY_THRESHOLD,
            LANDMARK_SIMILARITY_FLOOR,
            LANDMARK_MIN_ERA_GAP_TICKS
        );

        LandmarkMetadata {
            is_landmark: true,
            landmark_label: Some(label),
            era_id: Some(era_id),
            era_started_at_tick: Some(era_started_at_tick),
            detection_score,
            detection_reason: Some(detection_reason),
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
    use super::{EncodeEngine, EncodeWriteBranch, WriteBranchOutcome};
    use crate::api::NamespaceId;
    use crate::engine::contradiction::{
        ContradictionCandidate, ContradictionEngine, ContradictionError, ContradictionKind,
        ContradictionStore,
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
                "camera  summary  noted   a recurring   hallway  blockage",
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
        assert_eq!(
            prepared.normalized.payload_size_bytes,
            "camera  summary  noted   a recurring   hallway  blockage".len()
        );
        assert!(prepared.normalized.payload_size_bytes > prepared.normalized.compact_text.len());
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

    // ── Integrated detect-and-branch tests ────────────────────────────────────

    #[test]
    fn detect_and_branch_accepts_when_no_conflict() {
        let engine = test_engine();
        let mut contradictions = ContradictionEngine::new();

        let candidate = ContradictionCandidate {
            memory_id: MemoryId(100),
            fingerprint: 42,
            compact_text: "unique memory content with no matches".into(),
            namespace: test_namespace(),
        };

        let outcome = engine
            .detect_and_branch(
                &mut contradictions,
                test_namespace(),
                MemoryId(100),
                &candidate,
            )
            .unwrap();

        assert!(outcome.is_accepted());
        assert!(!outcome.is_contradiction());
        assert_eq!(outcome.contradiction_outcome(), None);
        assert_eq!(contradictions.count_in_namespace(&test_namespace()), 0);
    }

    #[test]
    fn detect_and_branch_records_contradiction_on_duplicate_fingerprint() {
        let engine = test_engine();
        let mut contradictions = ContradictionEngine::new();

        // Register an existing memory in the detection index
        contradictions.register_memory(
            test_namespace(),
            MemoryId(1),
            42,
            "the server is running on port 8080".into(),
        );

        // Incoming memory has the same fingerprint — exact duplicate
        let candidate = ContradictionCandidate {
            memory_id: MemoryId(2),
            fingerprint: 42,
            compact_text: "the server is running on port 8080".into(),
            namespace: test_namespace(),
        };

        let outcome = engine
            .detect_and_branch(
                &mut contradictions,
                test_namespace(),
                MemoryId(2),
                &candidate,
            )
            .unwrap();

        assert!(outcome.is_contradiction());
        let co = outcome.contradiction_outcome().unwrap();
        assert_eq!(co.branch, EncodeWriteBranch::ContradictionRecorded);
        assert_eq!(co.existing_memory, MemoryId(1));
        assert_eq!(co.incoming_memory, MemoryId(2));
        assert_eq!(co.kind, ContradictionKind::Duplicate);
        assert_eq!(contradictions.count_in_namespace(&test_namespace()), 1);
    }

    #[test]
    fn detect_and_branch_records_revision_on_high_text_similarity() {
        let engine = test_engine();
        let mut contradictions = ContradictionEngine::new();

        // Register with 14 distinct words so one-word swap gives Jaccard = 13/15 ≈ 0.867
        contradictions.register_memory(
            test_namespace(),
            MemoryId(10),
            100,
            "alpha beta gamma delta epsilon zeta eta theta iota kappa lambda mu nu".into(),
        );

        let candidate = ContradictionCandidate {
            memory_id: MemoryId(20),
            fingerprint: 200,
            compact_text: "alpha beta gamma delta epsilon zeta eta theta iota kappa lambda mu xi"
                .into(),
            namespace: test_namespace(),
        };

        let outcome = engine
            .detect_and_branch(
                &mut contradictions,
                test_namespace(),
                MemoryId(20),
                &candidate,
            )
            .unwrap();

        assert!(outcome.is_contradiction());
        let co = outcome.contradiction_outcome().unwrap();
        assert_eq!(co.kind, ContradictionKind::Revision);
        assert_eq!(co.existing_memory, MemoryId(10));
        assert_eq!(co.incoming_memory, MemoryId(20));
    }

    #[test]
    fn detect_and_branch_accepts_when_similarity_below_threshold() {
        let engine = test_engine();
        let mut contradictions = ContradictionEngine::new();

        contradictions.register_memory(
            test_namespace(),
            MemoryId(1),
            100,
            "database connection pool configured".into(),
        );

        let candidate = ContradictionCandidate {
            memory_id: MemoryId(2),
            fingerprint: 200,
            compact_text: "quantum physics measurements observed".into(),
            namespace: test_namespace(),
        };

        let outcome = engine
            .detect_and_branch(
                &mut contradictions,
                test_namespace(),
                MemoryId(2),
                &candidate,
            )
            .unwrap();

        assert!(outcome.is_accepted());
    }

    #[test]
    fn detect_and_branch_isolates_across_namespaces() {
        let engine = test_engine();
        let mut contradictions = ContradictionEngine::new();

        contradictions.register_memory(
            NamespaceId::new("alpha").unwrap(),
            MemoryId(1),
            42,
            "hello world".into(),
        );

        let candidate = ContradictionCandidate {
            memory_id: MemoryId(2),
            fingerprint: 42,
            compact_text: "hello world".into(),
            namespace: NamespaceId::new("beta").unwrap(),
        };

        let outcome = engine
            .detect_and_branch(
                &mut contradictions,
                NamespaceId::new("beta").unwrap(),
                MemoryId(2),
                &candidate,
            )
            .unwrap();

        assert!(outcome.is_accepted());
        assert_eq!(
            contradictions.count_in_namespace(&NamespaceId::new("alpha").unwrap()),
            0
        );
        assert_eq!(
            contradictions.count_in_namespace(&NamespaceId::new("beta").unwrap()),
            0
        );
    }

    #[test]
    fn detect_all_and_branch_records_multiple_conflicts() {
        let engine = test_engine();
        let mut contradictions = ContradictionEngine::new();

        // Register two memories that will both conflict with the candidate
        contradictions.register_memory(
            test_namespace(),
            MemoryId(1),
            100,
            "the server is running on port 8080".into(),
        );
        contradictions.register_memory(
            test_namespace(),
            MemoryId(3),
            300,
            "the server is running on port 8080 in production environment".into(),
        );

        let candidate = ContradictionCandidate {
            memory_id: MemoryId(2),
            fingerprint: 200,
            compact_text: "the server is running on port 9090 in production".into(),
            namespace: test_namespace(),
        };

        let outcomes = engine
            .detect_all_and_branch(
                &mut contradictions,
                test_namespace(),
                MemoryId(2),
                &candidate,
            )
            .unwrap();

        assert!(!outcomes.is_empty());
        // Both memories should have produced contradiction records
        assert!(contradictions.count_in_namespace(&test_namespace()) >= 1);
    }

    #[test]
    fn detect_all_and_branch_returns_empty_on_no_conflict() {
        let engine = test_engine();
        let mut contradictions = ContradictionEngine::new();

        let candidate = ContradictionCandidate {
            memory_id: MemoryId(1),
            fingerprint: 42,
            compact_text: "unique content".into(),
            namespace: test_namespace(),
        };

        let outcomes = engine
            .detect_all_and_branch(
                &mut contradictions,
                test_namespace(),
                MemoryId(1),
                &candidate,
            )
            .unwrap();

        assert!(outcomes.is_empty());
        assert_eq!(contradictions.count_in_namespace(&test_namespace()), 0);
    }

    #[test]
    fn detect_all_and_branch_skips_duplicate_pairs() {
        let engine = test_engine();
        let mut contradictions = ContradictionEngine::new();

        // Register the same memory twice (shouldn't happen in practice, but tests idempotency)
        contradictions.register_memory(test_namespace(), MemoryId(1), 42, "hello world".into());

        let candidate = ContradictionCandidate {
            memory_id: MemoryId(2),
            fingerprint: 42,
            compact_text: "hello world".into(),
            namespace: test_namespace(),
        };

        // First call records the contradiction
        let outcomes1 = engine
            .detect_all_and_branch(
                &mut contradictions,
                test_namespace(),
                MemoryId(2),
                &candidate,
            )
            .unwrap();
        assert_eq!(outcomes1.len(), 1);

        // Second call should skip the duplicate pair without erroring
        let outcomes2 = engine
            .detect_all_and_branch(
                &mut contradictions,
                test_namespace(),
                MemoryId(2),
                &candidate,
            )
            .unwrap();
        assert!(outcomes2.is_empty());
        // Still only one contradiction record
        assert_eq!(contradictions.count_in_namespace(&test_namespace()), 1);
    }

    // ── WriteBranchOutcome tests ──────────────────────────────────────────────

    #[test]
    fn write_branch_outcome_accessor_methods() {
        let trace = super::ContradictionWriteBranchTrace::accepted(0, MemoryId(1));
        let accepted = WriteBranchOutcome::Accepted { trace };
        assert!(accepted.is_accepted());
        assert!(!accepted.is_contradiction());
        assert_eq!(accepted.contradiction_outcome(), None);

        let trace = super::ContradictionWriteBranchTrace::contradiction(
            1,
            MemoryId(1),
            MemoryId(2),
            ContradictionKind::Supersession,
            800,
        );
        let branch = WriteBranchOutcome::ContradictionRecorded {
            outcome: super::ContradictionWriteOutcome {
                contradiction_id: crate::engine::contradiction::ContradictionId(1),
                branch: EncodeWriteBranch::ContradictionRecorded,
                existing_memory: MemoryId(1),
                incoming_memory: MemoryId(2),
                kind: ContradictionKind::Supersession,
            },
            trace,
        };
        assert!(!branch.is_accepted());
        assert!(branch.is_contradiction());
        assert!(branch.contradiction_outcome().is_some());
        assert_eq!(
            branch.contradiction_outcome().unwrap().kind,
            ContradictionKind::Supersession
        );
    }

    // ── ContradictionWriteBranchTrace tests ───────────────────────────────────

    #[test]
    fn trace_shows_branch_on_accepted_write() {
        let engine = test_engine();
        let mut contradictions = ContradictionEngine::new();

        contradictions.register_memory(
            test_namespace(),
            MemoryId(1),
            100,
            "database connection pool configured".into(),
        );

        let candidate = ContradictionCandidate {
            memory_id: MemoryId(2),
            fingerprint: 200,
            compact_text: "quantum physics measurements observed".into(),
            namespace: test_namespace(),
        };

        let outcome = engine
            .detect_and_branch(
                &mut contradictions,
                test_namespace(),
                MemoryId(2),
                &candidate,
            )
            .unwrap();

        let trace = outcome.trace();
        assert_eq!(trace.branch, EncodeWriteBranch::Accepted);
        assert_eq!(trace.branch_label(), "accepted");
        assert_eq!(trace.candidates_examined, 1);
        assert_eq!(trace.detected_kind, None);
        assert_eq!(trace.conflict_score, None);
        assert_eq!(trace.existing_memory, None);
        assert_eq!(trace.incoming_memory, MemoryId(2));
    }

    #[test]
    fn trace_shows_branch_on_contradiction_detected() {
        let engine = test_engine();
        let mut contradictions = ContradictionEngine::new();

        contradictions.register_memory(
            test_namespace(),
            MemoryId(1),
            42,
            "the server is running on port 8080".into(),
        );

        let candidate = ContradictionCandidate {
            memory_id: MemoryId(2),
            fingerprint: 42,
            compact_text: "the server is running on port 8080".into(),
            namespace: test_namespace(),
        };

        let outcome = engine
            .detect_and_branch(
                &mut contradictions,
                test_namespace(),
                MemoryId(2),
                &candidate,
            )
            .unwrap();

        let trace = outcome.trace();
        assert_eq!(trace.branch, EncodeWriteBranch::ContradictionRecorded);
        assert_eq!(trace.branch_label(), "contradiction_recorded");
        assert_eq!(trace.candidates_examined, 1);
        assert_eq!(trace.detected_kind, Some(ContradictionKind::Duplicate));
        assert_eq!(trace.conflict_score, Some(1000));
        assert_eq!(trace.existing_memory, Some(MemoryId(1)));
        assert_eq!(trace.incoming_memory, MemoryId(2));
    }

    #[test]
    fn trace_shows_correct_candidates_examined_count() {
        let engine = test_engine();
        let mut contradictions = ContradictionEngine::new();

        // Register 3 memories in the namespace
        contradictions.register_memory(test_namespace(), MemoryId(1), 100, "first memory".into());
        contradictions.register_memory(test_namespace(), MemoryId(2), 200, "second memory".into());
        contradictions.register_memory(test_namespace(), MemoryId(3), 300, "third memory".into());

        let candidate = ContradictionCandidate {
            memory_id: MemoryId(10),
            fingerprint: 999,
            compact_text: "completely unrelated content".into(),
            namespace: test_namespace(),
        };

        let outcome = engine
            .detect_and_branch(
                &mut contradictions,
                test_namespace(),
                MemoryId(10),
                &candidate,
            )
            .unwrap();

        let trace = outcome.trace();
        assert_eq!(trace.candidates_examined, 3);
        assert_eq!(trace.branch, EncodeWriteBranch::Accepted);
    }
}
