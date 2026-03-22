//! Proactive and retroactive interference detection and penalty surfaces.
//!
//! Interference occurs when similar but non-identical memories compete
//! for retrieval or encoding, weakening each other without creating
//! explicit contradictions. This module owns:
//!
//! - Similarity-band detection (excludes duplicates and contradictions)
//! - Retroactive interference: new memories weaken similar older ones
//! - Proactive interference: old memories make new similar recall harder
//! - Bounded penalty computation using similarity scores
//! - Inspectable interference event logging

use crate::types::MemoryId;

// ── Interference configuration ──────────────────────────────────────────────

/// Policy controlling interference detection and penalty behavior.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct InterferencePolicy {
    /// Minimum similarity (0..1) for interference band. Below this, memories are unrelated.
    pub min_similarity: f32,
    /// Maximum similarity (0..1) for interference band. Above this, memories are duplicates.
    pub max_similarity: f32,
    /// Base penalty factor applied on retroactive interference (new weakens old).
    pub retroactive_penalty_factor: f32,
    /// Retrieval difficulty increase on proactive interference (old confuses new).
    pub proactive_difficulty_delta: f32,
    /// Maximum number of interference events to process per encode batch.
    pub batch_event_limit: usize,
}

impl Default for InterferencePolicy {
    fn default() -> Self {
        Self {
            min_similarity: 0.7,
            max_similarity: 0.99,
            retroactive_penalty_factor: 0.05,
            proactive_difficulty_delta: 0.02,
            batch_event_limit: 100,
        }
    }
}

// ── Interference kinds ──────────────────────────────────────────────────────

/// Classification of an interference relationship.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub enum InterferenceKind {
    /// New memory weakens a similar older memory (retroactive).
    Retroactive,
    /// Old memory makes a new similar memory harder to retrieve (proactive).
    Proactive,
    /// Retrieval competition between similar memories.
    RetrievalCompetition,
    /// Confusable neighbor in the embedding space.
    ConfusableNeighbor,
}

impl InterferenceKind {
    /// Returns the stable machine-readable name.
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Retroactive => "retroactive",
            Self::Proactive => "proactive",
            Self::RetrievalCompetition => "retrieval_competition",
            Self::ConfusableNeighbor => "confusable_neighbor",
        }
    }
}

/// Backwards-compatible alias for older callers.
pub type InterferenceFamily = InterferenceKind;

// ── Interference event ──────────────────────────────────────────────────────

/// Record of one interference event for operator inspection.
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct InterferenceEvent {
    /// The new memory that triggered interference.
    pub source_memory_id: MemoryId,
    /// The existing memory being interfered with.
    pub target_memory_id: MemoryId,
    /// Kind of interference.
    pub kind: InterferenceKind,
    /// Similarity score that placed this pair in the interference band.
    pub similarity: f32,
    /// Penalty applied to the target memory's strength.
    pub strength_penalty: f32,
    /// Retrieval difficulty adjustment applied.
    pub retrieval_difficulty_adjustment: f32,
    /// Logical tick when the event was recorded.
    pub tick: u64,
    /// Why this was classified as interference rather than duplicate or contradiction.
    pub classification_reason: &'static str,
}

// ── Interference state ──────────────────────────────────────────────────────

/// Per-memory interference tracking for inspectability.
#[derive(Debug, Clone, PartialEq)]
pub struct InterferenceState {
    /// Memory this state belongs to.
    pub memory_id: MemoryId,
    /// Total retroactive penalty accumulated.
    pub cumulative_retroactive_penalty: f32,
    /// Total proactive difficulty accumulated.
    pub cumulative_proactive_difficulty: f32,
    /// Interfering memory IDs with their similarity scores.
    pub competing_memories: Vec<(MemoryId, f32)>,
    /// Last interference event tick.
    pub last_interference_tick: u64,
}

impl InterferenceState {
    /// Creates a new empty interference state for a memory.
    pub fn new(memory_id: MemoryId) -> Self {
        Self {
            memory_id,
            cumulative_retroactive_penalty: 0.0,
            cumulative_proactive_difficulty: 0.0,
            competing_memories: Vec::new(),
            last_interference_tick: 0,
        }
    }

    /// Returns the total interference-adjusted difficulty for retrieval ranking.
    pub fn total_retrieval_difficulty(&self) -> f32 {
        self.cumulative_retroactive_penalty + self.cumulative_proactive_difficulty
    }
}

// ── Interference results ────────────────────────────────────────────────────

/// Result of evaluating one similarity pair for interference.
#[derive(Debug, Clone, PartialEq)]
pub struct InterferenceEvaluation {
    /// Whether interference was detected.
    pub is_interference: bool,
    /// The kind of interference if detected.
    pub kind: Option<InterferenceKind>,
    /// Similarity score.
    pub similarity: f32,
    /// Classification reason.
    pub reason: &'static str,
    /// Strength penalty to apply.
    pub penalty: f32,
    /// Retrieval difficulty adjustment.
    pub difficulty_adjustment: f32,
}

/// Batch result of interference evaluation for an encode operation.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct InterferenceBatchResult {
    /// Events generated during this batch.
    pub events: Vec<InterferenceEvent>,
    /// Total retroactive penalties applied.
    pub retroactive_count: u32,
    /// Total proactive difficulty adjustments applied.
    pub proactive_count: u32,
    /// Pairs excluded as duplicates (similarity > max_similarity).
    pub duplicate_excluded: u32,
    /// Pairs excluded as unrelated (similarity < min_similarity).
    pub unrelated_excluded: u32,
}

// ── Interference engine ─────────────────────────────────────────────────────

/// Canonical interference engine owned by the core crate.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct InterferenceEngine;

impl InterferenceEngine {
    /// Returns the stable component identifier.
    pub const fn component_name(&self) -> &'static str {
        "engine.interference"
    }

    /// Evaluates a single similarity pair for interference.
    ///
    /// Returns classification + penalty without mutating any state.
    pub fn evaluate_pair(
        &self,
        _source_id: MemoryId,
        _target_id: MemoryId,
        similarity: f32,
        policy: &InterferencePolicy,
    ) -> InterferenceEvaluation {
        if similarity >= policy.max_similarity {
            return InterferenceEvaluation {
                is_interference: false,
                kind: None,
                similarity,
                reason: "duplicate_cutoff",
                penalty: 0.0,
                difficulty_adjustment: 0.0,
            };
        }
        if similarity < policy.min_similarity {
            return InterferenceEvaluation {
                is_interference: false,
                kind: None,
                similarity,
                reason: "below_interference_threshold",
                penalty: 0.0,
                difficulty_adjustment: 0.0,
            };
        }

        // In the interference band: penalize target (retroactive) and increase difficulty
        let penalty = similarity * policy.retroactive_penalty_factor;
        let difficulty = policy.proactive_difficulty_delta * (1.0 + similarity);

        InterferenceEvaluation {
            is_interference: true,
            kind: Some(InterferenceKind::Retroactive),
            similarity,
            reason: "interference_band",
            penalty,
            difficulty_adjustment: difficulty,
        }
    }

    /// Processes interference for a new memory against existing candidates.
    ///
    /// Returns batch result with events and counts. Does not mutate state —
    /// callers apply penalties to their own storage.
    pub fn process_encode(
        &self,
        new_memory_id: MemoryId,
        candidates: &[(MemoryId, f32)],
        policy: &InterferencePolicy,
        tick: u64,
    ) -> InterferenceBatchResult {
        let mut result = InterferenceBatchResult::default();
        let mut processed = 0;

        for &(target_id, similarity) in candidates {
            if target_id == new_memory_id {
                continue;
            }
            if processed >= policy.batch_event_limit {
                break;
            }

            let eval = self.evaluate_pair(new_memory_id, target_id, similarity, policy);

            match eval.reason {
                "duplicate_cutoff" => result.duplicate_excluded += 1,
                "below_interference_threshold" => result.unrelated_excluded += 1,
                _ => {
                    result.retroactive_count += 1;
                    result.events.push(InterferenceEvent {
                        source_memory_id: new_memory_id,
                        target_memory_id: target_id,
                        kind: InterferenceKind::Retroactive,
                        similarity,
                        strength_penalty: eval.penalty,
                        retrieval_difficulty_adjustment: eval.difficulty_adjustment,
                        tick,
                        classification_reason: "interference_band",
                    });
                    processed += 1;
                }
            }
        }

        result
    }

    /// Builds proactive interference events for recall difficulty.
    ///
    /// When recalling `source_memory_id`, finds similar newer memories that
    /// could confuse retrieval. Returns events without mutating state.
    pub fn evaluate_proactive(
        &self,
        source_memory_id: MemoryId,
        similar_newer: &[(MemoryId, f32)],
        policy: &InterferencePolicy,
        tick: u64,
    ) -> Vec<InterferenceEvent> {
        let mut events = Vec::new();

        for &(newer_id, similarity) in similar_newer {
            if similarity >= policy.max_similarity || similarity < policy.min_similarity {
                continue;
            }

            events.push(InterferenceEvent {
                source_memory_id,
                target_memory_id: newer_id,
                kind: InterferenceKind::Proactive,
                similarity,
                strength_penalty: 0.0,
                retrieval_difficulty_adjustment: policy.proactive_difficulty_delta
                    * (1.0 + similarity),
                tick,
                classification_reason: "proactive_confusion",
            });
        }

        events
    }

    /// Filters out pairs that should be handled by contradiction or duplicate
    /// systems instead of interference.
    pub fn classify_pair(
        &self,
        similarity: f32,
        has_contradiction: bool,
        policy: &InterferencePolicy,
    ) -> &'static str {
        if has_contradiction {
            "contradiction_excluded"
        } else if similarity >= policy.max_similarity {
            "duplicate_cutoff"
        } else if similarity < policy.min_similarity {
            "below_interference_threshold"
        } else {
            "interference_band"
        }
    }

    /// Computes cumulative interference penalty for a memory.
    pub fn compute_cumulative_penalty(
        &self,
        events: &[InterferenceEvent],
        target_memory_id: MemoryId,
    ) -> InterferenceState {
        let mut state = InterferenceState::new(target_memory_id);

        for event in events {
            if event.target_memory_id == target_memory_id {
                match event.kind {
                    InterferenceKind::Retroactive => {
                        state.cumulative_retroactive_penalty += event.strength_penalty;
                    }
                    InterferenceKind::Proactive => {
                        state.cumulative_proactive_difficulty +=
                            event.retrieval_difficulty_adjustment;
                    }
                    _ => {
                        // RetrievalCompetition and ConfusableNeighbor treated as retroactive
                        state.cumulative_retroactive_penalty += event.strength_penalty;
                    }
                }
                state
                    .competing_memories
                    .push((event.source_memory_id, event.similarity));
                state.last_interference_tick = state.last_interference_tick.max(event.tick);
            }
        }

        state
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn policy() -> InterferencePolicy {
        InterferencePolicy::default()
    }

    fn mid(id: u64) -> MemoryId {
        MemoryId(id)
    }

    // ── Classification tests ──────────────────────────────────────────────

    #[test]
    fn exact_duplicate_is_not_interference() {
        let engine = InterferenceEngine;
        let eval = engine.evaluate_pair(mid(1), mid(2), 0.995, &policy());
        assert!(!eval.is_interference);
        assert_eq!(eval.reason, "duplicate_cutoff");
        assert_eq!(eval.penalty, 0.0);
    }

    #[test]
    fn unrelated_memories_are_not_interference() {
        let engine = InterferenceEngine;
        let eval = engine.evaluate_pair(mid(1), mid(2), 0.3, &policy());
        assert!(!eval.is_interference);
        assert_eq!(eval.reason, "below_interference_threshold");
    }

    #[test]
    fn interference_band_is_detected() {
        let engine = InterferenceEngine;
        let eval = engine.evaluate_pair(mid(1), mid(2), 0.85, &policy());
        assert!(eval.is_interference);
        assert_eq!(eval.kind, Some(InterferenceKind::Retroactive));
        assert_eq!(eval.reason, "interference_band");
        assert!(eval.penalty > 0.0);
        assert!(eval.difficulty_adjustment > 0.0);
    }

    #[test]
    fn boundary_at_min_similarity_is_interference() {
        let engine = InterferenceEngine;
        let eval = engine.evaluate_pair(mid(1), mid(2), 0.7, &policy());
        assert!(eval.is_interference);
    }

    #[test]
    fn boundary_below_min_similarity_is_unrelated() {
        let engine = InterferenceEngine;
        let eval = engine.evaluate_pair(mid(1), mid(2), 0.699, &policy());
        assert!(!eval.is_interference);
        assert_eq!(eval.reason, "below_interference_threshold");
    }

    #[test]
    fn boundary_at_max_similarity_is_duplicate() {
        let engine = InterferenceEngine;
        let eval = engine.evaluate_pair(mid(1), mid(2), 0.99, &policy());
        assert!(!eval.is_interference);
        assert_eq!(eval.reason, "duplicate_cutoff");
    }

    // ── Penalty tests ─────────────────────────────────────────────────────

    #[test]
    fn penalty_scales_with_similarity() {
        let engine = InterferenceEngine;
        let low = engine.evaluate_pair(mid(1), mid(2), 0.75, &policy());
        let high = engine.evaluate_pair(mid(1), mid(2), 0.95, &policy());
        assert!(high.penalty > low.penalty);
    }

    #[test]
    fn penalty_is_bounded() {
        let engine = InterferenceEngine;
        let p = policy();
        let eval = engine.evaluate_pair(mid(1), mid(2), 0.98, &p);
        assert!(eval.penalty <= p.max_similarity * p.retroactive_penalty_factor);
    }

    // ── Batch processing tests ────────────────────────────────────────────

    #[test]
    fn batch_excludes_duplicates_and_unrelated() {
        let engine = InterferenceEngine;
        let candidates = vec![
            (mid(10), 0.995), // duplicate
            (mid(11), 0.3),   // unrelated
            (mid(12), 0.85),  // interference
            (mid(13), 0.80),  // interference
        ];
        let result = engine.process_encode(mid(1), &candidates, &policy(), 42);

        assert_eq!(result.duplicate_excluded, 1);
        assert_eq!(result.unrelated_excluded, 1);
        assert_eq!(result.retroactive_count, 2);
        assert_eq!(result.events.len(), 2);
    }

    #[test]
    fn batch_skips_self_pair() {
        let engine = InterferenceEngine;
        let candidates = vec![(mid(1), 0.85)];
        let result = engine.process_encode(mid(1), &candidates, &policy(), 42);
        assert_eq!(result.retroactive_count, 0);
    }

    #[test]
    fn batch_respects_event_limit() {
        let engine = InterferenceEngine;
        let p = InterferencePolicy {
            batch_event_limit: 2,
            ..Default::default()
        };
        let candidates: Vec<_> = (10..20).map(|i| (mid(i), 0.85)).collect();
        let result = engine.process_encode(mid(1), &candidates, &p, 42);
        assert_eq!(result.events.len(), 2);
        assert_eq!(result.retroactive_count, 2);
    }

    // ── Proactive interference tests ──────────────────────────────────────

    #[test]
    fn proactive_events_are_generated_for_similar_newer() {
        let engine = InterferenceEngine;
        let similar_newer = vec![(mid(2), 0.85), (mid(3), 0.90)];
        let events = engine.evaluate_proactive(mid(1), &similar_newer, &policy(), 100);
        assert_eq!(events.len(), 2);
        assert!(events.iter().all(|e| e.kind == InterferenceKind::Proactive));
        assert!(events.iter().all(|e| e.source_memory_id == mid(1)));
    }

    #[test]
    fn proactive_excludes_duplicates_and_unrelated() {
        let engine = InterferenceEngine;
        let similar_newer = vec![
            (mid(2), 0.995), // duplicate
            (mid(3), 0.85),  // interference
            (mid(4), 0.3),   // unrelated
        ];
        let events = engine.evaluate_proactive(mid(1), &similar_newer, &policy(), 100);
        assert_eq!(events.len(), 1);
        assert_eq!(events[0].target_memory_id, mid(3));
    }

    // ── Classification reason tests ───────────────────────────────────────

    #[test]
    fn classify_pair_respects_contradiction() {
        let engine = InterferenceEngine;
        let reason = engine.classify_pair(0.85, true, &policy());
        assert_eq!(reason, "contradiction_excluded");
    }

    #[test]
    fn classify_pair_handles_all_cases() {
        let engine = InterferenceEngine;
        let p = policy();
        assert_eq!(engine.classify_pair(0.995, false, &p), "duplicate_cutoff");
        assert_eq!(
            engine.classify_pair(0.3, false, &p),
            "below_interference_threshold"
        );
        assert_eq!(engine.classify_pair(0.85, false, &p), "interference_band");
    }

    // ── Cumulative state tests ────────────────────────────────────────────

    #[test]
    fn cumulative_penalty_accumulates_correctly() {
        let engine = InterferenceEngine;
        let events = vec![
            InterferenceEvent {
                source_memory_id: mid(2),
                target_memory_id: mid(1),
                kind: InterferenceKind::Retroactive,
                similarity: 0.85,
                strength_penalty: 0.0425,
                retrieval_difficulty_adjustment: 0.0,
                tick: 10,
                classification_reason: "interference_band",
            },
            InterferenceEvent {
                source_memory_id: mid(3),
                target_memory_id: mid(1),
                kind: InterferenceKind::Retroactive,
                similarity: 0.90,
                strength_penalty: 0.045,
                retrieval_difficulty_adjustment: 0.0,
                tick: 20,
                classification_reason: "interference_band",
            },
        ];
        let state = engine.compute_cumulative_penalty(&events, mid(1));
        assert!((state.cumulative_retroactive_penalty - 0.0875).abs() < 1e-6);
        assert_eq!(state.competing_memories.len(), 2);
        assert_eq!(state.last_interference_tick, 20);
    }

    #[test]
    fn cumulative_state_ignores_irrelevant_events() {
        let engine = InterferenceEngine;
        let events = vec![InterferenceEvent {
            source_memory_id: mid(1),
            target_memory_id: mid(99),
            kind: InterferenceKind::Retroactive,
            similarity: 0.85,
            strength_penalty: 0.05,
            retrieval_difficulty_adjustment: 0.0,
            tick: 10,
            classification_reason: "interference_band",
        }];
        let state = engine.compute_cumulative_penalty(&events, mid(1));
        assert_eq!(state.cumulative_retroactive_penalty, 0.0);
        assert!(state.competing_memories.is_empty());
    }

    #[test]
    fn total_retrieval_difficulty_sums_both_components() {
        let state = InterferenceState {
            memory_id: mid(1),
            cumulative_retroactive_penalty: 0.05,
            cumulative_proactive_difficulty: 0.03,
            competing_memories: vec![],
            last_interference_tick: 0,
        };
        assert!((state.total_retrieval_difficulty() - 0.08).abs() < 1e-6);
    }

    // ── Determinism tests ─────────────────────────────────────────────────

    #[test]
    fn deterministic_evaluation_for_same_inputs() {
        let engine = InterferenceEngine;
        let p = policy();
        let e1 = engine.evaluate_pair(mid(1), mid(2), 0.85, &p);
        let e2 = engine.evaluate_pair(mid(1), mid(2), 0.85, &p);
        assert_eq!(e1, e2);
    }

    #[test]
    fn deterministic_batch_for_same_inputs() {
        let engine = InterferenceEngine;
        let p = policy();
        let candidates = vec![(mid(10), 0.85), (mid(11), 0.90)];
        let r1 = engine.process_encode(mid(1), &candidates, &p, 42);
        let r2 = engine.process_encode(mid(1), &candidates, &p, 42);
        assert_eq!(r1, r2);
    }

    // ── Enum tests ────────────────────────────────────────────────────────

    #[test]
    fn interference_kind_as_str() {
        assert_eq!(InterferenceKind::Retroactive.as_str(), "retroactive");
        assert_eq!(InterferenceKind::Proactive.as_str(), "proactive");
        assert_eq!(
            InterferenceKind::RetrievalCompetition.as_str(),
            "retrieval_competition"
        );
        assert_eq!(
            InterferenceKind::ConfusableNeighbor.as_str(),
            "confusable_neighbor"
        );
    }

    // ── Integration: interference vs contradiction vs duplicate ───────────

    #[test]
    fn interference_excludes_both_duplicates_and_contradictions() {
        let engine = InterferenceEngine;
        let p = policy();

        let dup_eval = engine.evaluate_pair(mid(1), mid(2), 0.995, &p);
        assert!(!dup_eval.is_interference);

        let contra_reason = engine.classify_pair(0.85, true, &p);
        assert_eq!(contra_reason, "contradiction_excluded");

        let interf_eval = engine.evaluate_pair(mid(1), mid(3), 0.85, &p);
        assert!(interf_eval.is_interference);
    }
}
