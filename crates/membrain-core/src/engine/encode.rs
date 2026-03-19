use crate::config::RuntimeConfig;
use crate::observability::{
    AdmissionOutcomeKind, EncodeFastPathStage, EncodeFastPathTrace, WorkingMemoryTrace,
};
use crate::types::{
    CanonicalMemoryType, FastPathRouteFamily, NormalizedMemoryEnvelope, RawEncodeInput,
    WorkingMemoryId, WorkingMemoryItem,
};
use xxhash_rust::xxh64::xxh64;

const NORMALIZATION_GENERATION: &str = "normalize-v1";
const FAST_PATH_STAGES: [EncodeFastPathStage; 4] = [
    EncodeFastPathStage::Normalize,
    EncodeFastPathStage::Fingerprint,
    EncodeFastPathStage::ShallowClassify,
    EncodeFastPathStage::ProvisionalSalience,
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

/// Prepared encode candidate emitted by the synchronous fast path.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PreparedEncodeCandidate {
    /// Canonical normalized envelope frozen before persistence.
    pub normalized: NormalizedMemoryEnvelope,
    /// Stable duplicate-family fingerprint derived from the normalized form.
    pub fingerprint: u64,
    /// Bounded shallow classification summary.
    pub classification: ShallowClassification,
    /// First-pass salience scalar used for initial routing inputs.
    pub provisional_salience: u16,
    /// Structured trace proving the ordered fast-path stages.
    pub trace: EncodeFastPathTrace,
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
                    slot_pressure: self.slots.len(),
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

        NormalizedMemoryEnvelope {
            memory_type: input.kind.canonical_memory_type(),
            source_kind: input.kind,
            raw_text: input.raw_text,
            compact_text,
            normalization_generation: NORMALIZATION_GENERATION,
            payload_size_bytes,
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
        let normalized = self.normalize_input(input);
        let fingerprint = self.fingerprint(&normalized);
        let classification = self.shallow_classify(&normalized);
        let provisional_salience = self.provisional_salience(&normalized, classification);
        let trace = EncodeFastPathTrace {
            stages: FAST_PATH_STAGES,
            normalization_generation: normalized.normalization_generation,
            memory_type: normalized.memory_type,
            route_family: classification.route_family,
            provisional_salience,
            duplicate_hint_candidate_count: 0,
            stayed_within_latency_budget: true,
        };

        PreparedEncodeCandidate {
            normalized,
            fingerprint,
            classification,
            provisional_salience,
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
        self.prepare_fast_path(input)
    }
}
