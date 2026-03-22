//! Cross-engine deterministic integration tests.
//!
//! Tests the interaction between strength, interference, forgetting,
//! and reconsolidation engines without wall-clock dependencies.

use membrain_core::engine::forgetting::{
    EligibilityFactors, ForgettingAction, ForgettingEngine, ForgettingPolicy,
};
use membrain_core::engine::interference::{InterferenceEngine, InterferencePolicy};
use membrain_core::engine::reconsolidation::{
    LabileState, PendingUpdate, ReconsolidationEngine, ReconsolidationOutcome,
    ReconsolidationPolicy, UpdateSource,
};
use membrain_core::engine::strength::{
    DecayDecision, StrengthEngine, StrengthPolicy, StrengthState,
};
use membrain_core::types::MemoryId;

fn mid(id: u64) -> MemoryId {
    MemoryId(id)
}

// ── Strength → Forgetting integration ────────────────────────────────────────

#[test]
fn strength_state_maps_to_forgetting_eligibility() {
    let engine = StrengthEngine::new();
    let mut state = StrengthState::with_base(0.5);
    state.stability = 5.0;
    state.last_accessed_tick = 0;
    state.access_count = 10;

    // Use small elapsed time so recency stays high
    let factors = engine.to_eligibility_factors(&state, 5);

    let forgetting = ForgettingEngine;
    let (action, score) =
        forgetting.evaluate_memory_with_factors(mid(1), &factors, &ForgettingPolicy::default());

    // Memory with moderate strength should not be forgotten
    assert_eq!(action, ForgettingAction::Skip);
    assert!(score.composite_score > 200);
}

#[test]
fn zero_strength_memory_is_immediately_forgetting_eligible() {
    let engine = StrengthEngine::new();
    let mut state = StrengthState::with_base(0.0);
    state.stability = 0.0;
    state.last_accessed_tick = 0;

    // Zero strength with any elapsed time → very low effective strength
    let factors = engine.to_eligibility_factors(&state, 1000);

    let forgetting = ForgettingEngine;
    let (action, _) =
        forgetting.evaluate_memory_with_factors(mid(1), &factors, &ForgettingPolicy::default());

    // Zero strength should be eligible for forgetting
    assert!(matches!(
        action,
        ForgettingAction::SoftForget | ForgettingAction::Demote { .. }
    ));
}

#[test]
fn emotional_bypass_prevents_forgetting() {
    let engine = StrengthEngine::new();
    let state = StrengthState::with_emotional(0.3, 0.9, 0.6);

    let factors = engine.to_eligibility_factors(&state, 100);

    let forgetting = ForgettingEngine;
    let (action, score) =
        forgetting.evaluate_memory_with_factors(mid(1), &factors, &ForgettingPolicy::default());

    // Emotional bypass should resist forgetting even with low base strength
    assert_eq!(action, ForgettingAction::Skip);
    assert!(score.composite_score > 50);
    assert!(factors.bypass_decay);
}

// ── Strength → Interference integration ──────────────────────────────────────

#[test]
fn interference_penalty_affects_effective_strength() {
    let interference_engine = InterferenceEngine;
    let policy = InterferencePolicy::default();

    // New memory interferes with old one
    let result = interference_engine.process_encode(mid(2), &[(mid(1), 0.85)], &policy, 100);

    assert_eq!(result.retroactive_count, 1);
    let penalty = result.events[0].strength_penalty;

    // Apply penalty to old memory's strength
    let mut old_state = StrengthState::with_base(0.7);
    old_state.base_strength -= penalty;

    assert!(old_state.base_strength < 0.7);
    assert!(old_state.base_strength > 0.0);
}

#[test]
fn interference_excludes_duplicates_from_strength_penalties() {
    let engine = InterferenceEngine;
    let policy = InterferencePolicy::default();

    let result = engine.process_encode(
        mid(2),
        &[(mid(1), 0.995)], // duplicate, not interference
        &policy,
        100,
    );

    assert_eq!(result.retroactive_count, 0);
    assert_eq!(result.duplicate_excluded, 1);
}

// ── Strength → Reconsolidation integration ───────────────────────────────────

#[test]
fn reconsolidation_applies_strength_bonus() {
    let recon_engine = ReconsolidationEngine::new();
    let policy = ReconsolidationPolicy::default();

    let labile = LabileState::new(100, 50);
    let update =
        PendingUpdate::new(mid(1), 105, UpdateSource::User).with_content("revised".to_string());

    let result = recon_engine.tick(&labile, Some(&update), 120, 0.5, &policy);
    assert_eq!(result.outcome, ReconsolidationOutcome::Applied);
    assert!(result.new_strength.unwrap() > 0.5);

    // Strength bonus should be capped
    assert!(result.new_strength.unwrap() <= 1.0);
}

#[test]
fn reconsolidation_discards_stale_updates() {
    let engine = ReconsolidationEngine::new();
    let policy = ReconsolidationPolicy::default();

    let labile = LabileState::new(100, 50); // expires at tick 150
    let update = PendingUpdate::new(mid(1), 105, UpdateSource::System);

    // Tick after window expires
    let result = engine.tick(&labile, Some(&update), 200, 0.5, &policy);
    assert_eq!(result.outcome, ReconsolidationOutcome::DiscardedStale);
    assert_eq!(result.new_strength, None);
}

// ── LTP/LTD lifecycle ────────────────────────────────────────────────────────

#[test]
fn ltp_then_recall_then_ltd_lifecycle() {
    let engine = StrengthEngine::new();
    let policy = StrengthPolicy::default();

    let mut state = StrengthState::with_base(0.3);
    state.stability = 2.0;
    state.last_accessed_tick = 0;

    // Encode → strengthen (LTP)
    engine.apply_ltp(&mut state, 1, &policy);
    assert!(state.base_strength > 0.3);

    // Strengthen again
    engine.apply_ltp(&mut state, 5, &policy);
    assert!(state.base_strength > 0.4);

    // Decay over time (LTD)
    let effective = engine.effective_strength(&state, 50);
    assert!(effective < state.base_strength);

    // Archive when below threshold
    let decision = engine.evaluate_decay(&state, 1000, &policy);
    match decision {
        DecayDecision::Archive { .. } => {
            // Expected for highly decayed memories
        }
        DecayDecision::ApplyLTD { new_strength, .. } => {
            assert!(new_strength < state.base_strength);
        }
        _ => panic!("unexpected decay decision"),
    }
}

// ── Interference → Forgetting integration ────────────────────────────────────

#[test]
fn interfered_memories_become_forgetting_eligible() {
    let interference = InterferenceEngine;
    let forgetting = ForgettingEngine;
    let int_policy = InterferencePolicy::default();
    let forget_policy = ForgettingPolicy::default();

    // Memory starts with moderate strength
    let factors_no_interference = EligibilityFactors {
        effective_strength: 0.5,
        recency: 0.8,
        access_frequency: 0.5,
        in_contradiction: false,
        emotional_arousal: 0.0,
        bypass_decay: false,
    };
    let (_, score_before) =
        forgetting.evaluate_memory_with_factors(mid(1), &factors_no_interference, &forget_policy);

    // After interference, effective strength drops
    let result = interference.process_encode(mid(2), &[(mid(1), 0.85)], &int_policy, 100);
    let penalty = result.events[0].strength_penalty;

    let factors_with_interference = EligibilityFactors {
        effective_strength: (0.5 - penalty).max(0.0),
        recency: 0.8,
        access_frequency: 0.5,
        in_contradiction: false,
        emotional_arousal: 0.0,
        bypass_decay: false,
    };
    let (_, score_after) =
        forgetting.evaluate_memory_with_factors(mid(1), &factors_with_interference, &forget_policy);

    // Interfered memory should score lower (more eligible for forgetting)
    assert!(score_after.composite_score < score_before.composite_score);
}

// ── Determinism verification ─────────────────────────────────────────────────

#[test]
fn full_lifecycle_is_deterministic() {
    for _ in 0..10 {
        let strength_engine = StrengthEngine::new();
        let interference_engine = InterferenceEngine;
        let recon_engine = ReconsolidationEngine::new();
        let forgetting_engine = ForgettingEngine;

        let policy = StrengthPolicy::default();
        let int_policy = InterferencePolicy::default();
        let recon_policy = ReconsolidationPolicy::default();
        let forget_policy = ForgettingPolicy::default();

        let mut state = StrengthState::with_base(0.5);
        state.stability = 3.0;
        state.last_accessed_tick = 0;
        state.access_count = 5;

        // LTP
        let ltp = strength_engine.apply_ltp(&mut state, 10, &policy);
        assert_eq!(ltp.new_strength, 0.6);

        // Interference
        let interf = interference_engine.process_encode(mid(2), &[(mid(1), 0.85)], &int_policy, 20);
        assert_eq!(interf.retroactive_count, 1);

        // Forgetting — use small elapsed time
        let factors = strength_engine.to_eligibility_factors(&state, 15);
        let (action, _) =
            forgetting_engine.evaluate_memory_with_factors(mid(1), &factors, &forget_policy);
        // Strong memory with small elapsed = skip
        assert!(matches!(
            action,
            ForgettingAction::Skip | ForgettingAction::Demote { .. }
        ));

        // Reconsolidation
        let labile = LabileState::new(10, 50);
        let update =
            PendingUpdate::new(mid(1), 15, UpdateSource::User).with_content("revised".to_string());
        let recon = recon_engine.tick(&labile, Some(&update), 30, 0.6, &recon_policy);
        assert_eq!(recon.outcome, ReconsolidationOutcome::Applied);
    }
}

// ── Edge cases ───────────────────────────────────────────────────────────────

#[test]
fn decayed_strength_produces_forgetting_eligible_score() {
    let engine = StrengthEngine::new();
    let mut state = StrengthState::with_base(0.1);
    state.stability = 1.0;
    state.last_accessed_tick = 0;

    // Heavily decayed memory (elapsed >> stability)
    let factors = engine.to_eligibility_factors(&state, 1000);

    let forgetting = ForgettingEngine;
    let (action, _score) =
        forgetting.evaluate_memory_with_factors(mid(1), &factors, &ForgettingPolicy::default());

    // Heavily decayed memory should not be skipped (should be demoted or forgotten)
    assert!(!matches!(action, ForgettingAction::Skip));
}

#[test]
fn max_strength_memory_resists_all_decay() {
    let engine = StrengthEngine::new();
    let policy = StrengthPolicy::default();

    let mut state = StrengthState::with_base(1.0);
    state.stability = 10.0;
    state.last_accessed_tick = 0;

    // Even after moderate decay, max strength with high stability stays strong
    let effective = engine.effective_strength(&state, 10);
    assert!(effective > 0.3);

    let decision = engine.evaluate_decay(&state, 10, &policy);
    match decision {
        DecayDecision::ApplyLTD { new_strength, .. } => {
            assert!(new_strength > 0.3);
        }
        DecayDecision::Archive { .. } => panic!("max strength memory should not archive"),
        _ => {}
    }
}

#[test]
fn interference_batch_limit_prevents_unbounded_processing() {
    let engine = InterferenceEngine;
    let policy = InterferencePolicy {
        batch_event_limit: 3,
        ..Default::default()
    };

    let candidates: Vec<_> = (10..100).map(|i| (mid(i), 0.85)).collect();

    let result = engine.process_encode(mid(1), &candidates, &policy, 42);
    assert_eq!(result.events.len(), 3);
    assert_eq!(result.retroactive_count, 3);
}

#[test]
fn reconsolidation_empty_update_is_rejected() {
    let engine = ReconsolidationEngine::new();

    let labile = LabileState::new(100, 50);
    let empty_update = PendingUpdate::new(mid(1), 105, UpdateSource::System);

    let validation = engine.validate_pending_update(&empty_update, &labile, 120);
    assert!(matches!(
        validation,
        membrain_core::engine::reconsolidation::UpdateValidationResult::Rejected(
            membrain_core::engine::reconsolidation::UpdateRejectionReason::EmptyUpdate
        )
    ));
}
