//! Cross-engine deterministic integration tests.
//!
//! Tests the interaction between strength, interference, forgetting,
//! and reconsolidation engines without wall-clock dependencies.

use membrain_core::engine::forgetting::{
    EligibilityFactors, ForgettingAction, ForgettingEngine, ForgettingPolicy,
};
use membrain_core::engine::interference::{InterferenceEngine, InterferencePolicy};
use membrain_core::engine::maintenance::{
    DurableStateToken, InterruptedMaintenance, InterruptionReason, LogicalClock,
    MaintenanceController, MaintenanceJobHandle, MaintenanceJobState, MaintenanceOperation,
    MaintenanceProgress, MaintenanceStep, TickScenarioArtifact, TickSequenceFixture,
};
use membrain_core::engine::reconsolidation::{
    LabileMemory, LabileState, PendingUpdate, PreReopenState, ReconsolidationEngine,
    ReconsolidationOutcome, ReconsolidationPolicy, ReconsolidationRun, RefreshReadiness,
    ReopenStableState, UpdateSource,
};
use membrain_core::engine::strength::{
    DecayDecision, StrengthEngine, StrengthPolicy, StrengthState,
};
use membrain_core::types::MemoryId;

fn mid(id: u64) -> MemoryId {
    MemoryId(id)
}

fn policy_bundle() -> (
    StrengthPolicy,
    InterferencePolicy,
    ReconsolidationPolicy,
    ForgettingPolicy,
) {
    (
        StrengthPolicy::default(),
        InterferencePolicy::default(),
        ReconsolidationPolicy::default(),
        ForgettingPolicy::default(),
    )
}

fn replay_reconsolidation_apply_fixture(
) -> (TickScenarioArtifact, ReconsolidationOutcome, Option<f32>) {
    let (_, _, recon_policy, _) = policy_bundle();
    let recon_engine = ReconsolidationEngine::new();
    let mut ticks = TickSequenceFixture::new(10);
    let labile = LabileState::new(ticks.current_tick(), 50);
    let submitted_tick = ticks.advance_by(5);
    let update = PendingUpdate::new(mid(1), submitted_tick, UpdateSource::User)
        .with_content("revised".to_string());
    let current_tick = ticks.advance_by(15);
    let result = recon_engine.tick(
        &labile,
        Some(&update),
        current_tick,
        0.6,
        &recon_policy,
        membrain_core::engine::reconsolidation::RefreshReadiness::Ready,
    );

    (
        ticks.artifact("reconsolidation_apply_window"),
        result.outcome,
        result.new_strength,
    )
}

#[derive(Debug, Clone, PartialEq)]
struct DeterministicStrengthFixture {
    state: StrengthState,
    current_tick: u64,
    effective_strength: f32,
    factors: EligibilityFactors,
    decay: DecayDecision,
}

impl DeterministicStrengthFixture {
    fn new(base_strength: f32, stability: f32, last_accessed_tick: u64, current_tick: u64) -> Self {
        let engine = StrengthEngine::new();
        let policy = StrengthPolicy::default();
        let mut state = StrengthState::with_base(base_strength);
        state.stability = stability;
        state.last_accessed_tick = last_accessed_tick;
        let effective_strength = engine.effective_strength(&state, current_tick);
        let factors = engine.to_eligibility_factors(&state, current_tick);
        let decay = engine.evaluate_decay(&state, current_tick, &policy);
        Self {
            state,
            current_tick,
            effective_strength,
            factors,
            decay,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
struct DeterministicReconTickFixture {
    artifact: TickScenarioArtifact,
    result: membrain_core::engine::reconsolidation::ReconsolidationTickResult,
}

impl DeterministicReconTickFixture {
    fn with_readiness(
        scenario_name: &'static str,
        readiness: RefreshReadiness,
        window_ticks: u64,
        start_tick: u64,
        submit_after: u64,
        resolve_after: u64,
    ) -> Self {
        let (_, _, recon_policy, _) = policy_bundle();
        let recon_engine = ReconsolidationEngine::new();
        let mut ticks = TickSequenceFixture::new(start_tick);
        let labile = LabileState::new(ticks.current_tick(), window_ticks);
        let submitted_tick = ticks.advance_by(submit_after);
        let update = PendingUpdate::new(mid(1), submitted_tick, UpdateSource::User)
            .with_content("revised".to_string());
        let current_tick = ticks.advance_by(resolve_after);
        let result = recon_engine.tick(
            &labile,
            Some(&update),
            current_tick,
            0.6,
            &recon_policy,
            readiness,
        );
        Self {
            artifact: ticks.artifact(scenario_name),
            result,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct ScriptedTimeoutFixture {
    artifact: TickScenarioArtifact,
    interrupted: InterruptedMaintenance,
    polls_used: u32,
    recorded_interruptions: Vec<InterruptionReason>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct ScriptedMaintenanceFixture {
    steps: Vec<MaintenanceStep<&'static str>>,
    next_step: usize,
    preserved: DurableStateToken,
    interruptions: Vec<InterruptionReason>,
    ticks: TickSequenceFixture,
}

impl ScriptedMaintenanceFixture {
    fn timeout_fixture(max_polls: u32) -> ScriptedTimeoutFixture {
        let operation = Self {
            steps: vec![
                MaintenanceStep::Pending(MaintenanceProgress::new(1, 3)),
                MaintenanceStep::Pending(MaintenanceProgress::new(2, 3)),
                MaintenanceStep::Completed("too late"),
            ],
            next_step: 0,
            preserved: DurableStateToken(81),
            interruptions: Vec::new(),
            ticks: TickSequenceFixture::new(0),
        };
        let mut handle = MaintenanceJobHandle::new(operation, max_polls);
        handle.start();
        handle.poll();
        handle.poll();
        let timed_out = handle.poll();
        let interrupted = match timed_out.state {
            MaintenanceJobState::TimedOut(interrupted) => interrupted,
            _ => std::process::abort(),
        };
        ScriptedTimeoutFixture {
            artifact: handle
                .operation()
                .ticks
                .artifact("deterministic_timeout_window"),
            interrupted,
            polls_used: timed_out.polls_used,
            recorded_interruptions: handle.operation().interruptions.clone(),
        }
    }
}

impl MaintenanceOperation for ScriptedMaintenanceFixture {
    type Summary = &'static str;

    fn poll_step(&mut self) -> MaintenanceStep<Self::Summary> {
        self.ticks.advance_by(1);
        let step = self.steps[self.next_step].clone();
        self.next_step += 1;
        step
    }

    fn interrupt(&mut self, reason: InterruptionReason) -> InterruptedMaintenance {
        self.interruptions.push(reason);
        InterruptedMaintenance {
            reason,
            preserved_durable_state: self.preserved,
            artifact: None,
        }
    }
}

// ── Deterministic fixture helpers ────────────────────────────────────────────

#[test]
fn logical_clock_advances_without_wall_clock_dependencies() {
    let mut clock = LogicalClock::new(7);

    assert_eq!(clock.current_tick(), 7);
    assert_eq!(clock.advance_by(5), 12);
    assert_eq!(clock.advance_to(20), 20);
    assert_eq!(clock.advance_to(18), 20);
    assert_eq!(clock.current_tick(), 20);
}

#[test]
fn tick_sequence_fixture_records_replayable_history() {
    let mut fixture = TickSequenceFixture::new(100);

    assert_eq!(fixture.history(), &[100]);
    fixture.advance_by(4);
    fixture.advance_to(120);
    fixture.advance_to(115);

    assert_eq!(fixture.current_tick(), 120);
    assert_eq!(fixture.history(), &[100, 104, 120, 120]);
}

// ── Strength → Forgetting integration ────────────────────────────────────────

#[test]
fn strength_state_maps_to_forgetting_eligibility() {
    let mut fixture = DeterministicStrengthFixture::new(0.5, 5.0, 0, 5);
    fixture.state.access_count = 10;
    let engine = StrengthEngine::new();
    let factors = engine.to_eligibility_factors(&fixture.state, fixture.current_tick);

    let forgetting = ForgettingEngine;
    let (action, score) =
        forgetting.evaluate_memory_with_factors(mid(1), &factors, &ForgettingPolicy::default());

    assert_eq!(action, ForgettingAction::Skip);
    assert!(score.composite_score > 200);
    assert!(factors.recency > 0.9);
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

    // Zero strength should be eligible for bounded utility forgetting or demotion.
    assert!(matches!(
        action,
        ForgettingAction::Archive | ForgettingAction::Demote { .. }
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
    let fixture = DeterministicReconTickFixture::with_readiness(
        "reconsolidation_apply_window",
        RefreshReadiness::Ready,
        50,
        10,
        5,
        15,
    );

    assert_eq!(
        fixture.artifact.scenario_name,
        "reconsolidation_apply_window"
    );
    assert_eq!(fixture.artifact.start_tick, 10);
    assert_eq!(fixture.artifact.tick_history, vec![10, 15, 30]);
    assert_eq!(fixture.result.outcome, ReconsolidationOutcome::Applied);
    let new_strength = fixture.result.new_strength.unwrap_or_else(|| {
        std::process::abort();
    });
    assert!(new_strength > 0.6);
    assert!(new_strength <= 1.0);
}

#[test]
fn reconsolidation_apply_fixture_is_replayable() {
    let first = replay_reconsolidation_apply_fixture();
    let second = replay_reconsolidation_apply_fixture();

    assert_eq!(first, second);
}

#[test]
fn reconsolidation_discards_stale_updates() {
    let engine = ReconsolidationEngine::new();
    let policy = ReconsolidationPolicy::default();

    let labile = LabileState::new(100, 50); // expires at tick 150
    let update = PendingUpdate::new(mid(1), 105, UpdateSource::System);

    // Tick after window expires
    let result = engine.tick(
        &labile,
        Some(&update),
        200,
        0.5,
        &policy,
        membrain_core::engine::reconsolidation::RefreshReadiness::Ready,
    );
    assert_eq!(result.outcome, ReconsolidationOutcome::DiscardedStale);
    assert_eq!(result.new_strength, None);
    assert!(result.pending_update_cleared);
    assert!(result.restabilized);
}

#[test]
fn stale_reconsolidation_tick_records_update_source_without_mutation() {
    let engine = ReconsolidationEngine::new();
    let policy = ReconsolidationPolicy::default();
    let labile = LabileState::new(100, 50);
    let update = PendingUpdate::new(mid(11), 105, UpdateSource::System)
        .with_content("stale update".to_string());

    let result = engine.tick(
        &labile,
        Some(&update),
        200,
        0.6,
        &policy,
        membrain_core::engine::reconsolidation::RefreshReadiness::Ready,
    );
    let audit = engine.audit_tick(
        mid(11),
        200,
        &result,
        Some(0.6),
        Some(0.55),
        Some(ReopenStableState::Consolidated),
        Some(UpdateSource::System),
    );

    assert_eq!(audit.outcome, ReconsolidationOutcome::DiscardedStale);
    assert_eq!(audit.update_source, Some(UpdateSource::System));
    assert_eq!(audit.preserved_state, Some(ReopenStableState::Consolidated));
    assert_eq!(audit.strength_after, None);
    assert!(audit.pending_update_cleared);
    assert!(!audit.authoritative_state_mutated);
    assert!(audit.restabilized);
}

#[test]
fn reconsolidation_does_not_mutate_when_refresh_is_deferred_or_failed() {
    let deferred = DeterministicReconTickFixture::with_readiness(
        "reconsolidation_deferred_refresh",
        RefreshReadiness::Deferred,
        50,
        100,
        5,
        15,
    );
    assert_eq!(
        deferred.result.outcome,
        ReconsolidationOutcome::DeferredRefresh
    );
    assert_eq!(deferred.result.new_strength, None);
    assert!(!deferred.result.authoritative_state_mutated);
    assert!(deferred
        .result
        .refresh_triggers
        .contains(&membrain_core::engine::reconsolidation::RefreshTrigger::EmbeddingRefresh));
    assert!(deferred
        .result
        .refresh_triggers
        .contains(&membrain_core::engine::reconsolidation::RefreshTrigger::IndexRefresh));
    assert!(deferred
        .result
        .refresh_triggers
        .contains(&membrain_core::engine::reconsolidation::RefreshTrigger::CacheInvalidate));
    assert!(!deferred.result.pending_update_cleared);
    assert!(!deferred.result.restabilized);
    assert_eq!(deferred.artifact.tick_history, vec![100, 105, 120]);

    let failed = DeterministicReconTickFixture::with_readiness(
        "reconsolidation_failed_refresh",
        RefreshReadiness::Failed,
        50,
        100,
        5,
        15,
    );
    assert_eq!(
        failed.result.outcome,
        ReconsolidationOutcome::BlockedRefreshFailure
    );
    assert_eq!(failed.result.new_strength, None);
    assert!(!failed.result.authoritative_state_mutated);
    assert!(failed
        .result
        .refresh_triggers
        .contains(&membrain_core::engine::reconsolidation::RefreshTrigger::EmbeddingRefresh));
    assert!(failed
        .result
        .refresh_triggers
        .contains(&membrain_core::engine::reconsolidation::RefreshTrigger::IndexRefresh));
    assert!(failed
        .result
        .refresh_triggers
        .contains(&membrain_core::engine::reconsolidation::RefreshTrigger::CacheInvalidate));
    assert!(!failed.result.pending_update_cleared);
    assert!(!failed.result.restabilized);
    assert_eq!(failed.artifact.tick_history, vec![100, 105, 120]);
}

#[test]
fn reconsolidation_run_audit_entries_include_refresh_trigger_details() {
    let memories = vec![
        LabileMemory {
            memory_id: mid(21),
            labile_state: LabileState::new(100, 50),
            pending_update: Some(
                PendingUpdate::new(mid(21), 105, UpdateSource::User)
                    .with_content("apply me".to_string()),
            ),
            current_strength: 0.6,
            pre_reopen_state: PreReopenState {
                memory_id: mid(21),
                reopen_tick: 100,
                strength_at_reopen: 0.55,
                stability_at_reopen: 3.0,
                access_count_at_reopen: 5,
            },
            restabilize_to: ReopenStableState::Consolidated,
            refresh_readiness: RefreshReadiness::Ready,
        },
        LabileMemory {
            memory_id: mid(22),
            labile_state: LabileState::new(100, 10),
            pending_update: Some(
                PendingUpdate::new(mid(22), 105, UpdateSource::System)
                    .with_content("stale".to_string()),
            ),
            current_strength: 0.7,
            pre_reopen_state: PreReopenState {
                memory_id: mid(22),
                reopen_tick: 100,
                strength_at_reopen: 0.65,
                stability_at_reopen: 4.0,
                access_count_at_reopen: 6,
            },
            restabilize_to: ReopenStableState::SynapticDone,
            refresh_readiness: RefreshReadiness::Ready,
        },
        LabileMemory {
            memory_id: mid(23),
            labile_state: LabileState::new(100, 50),
            pending_update: Some(
                PendingUpdate::new(mid(23), 105, UpdateSource::Agent)
                    .with_content("defer".to_string()),
            ),
            current_strength: 0.8,
            pre_reopen_state: PreReopenState {
                memory_id: mid(23),
                reopen_tick: 100,
                strength_at_reopen: 0.75,
                stability_at_reopen: 5.0,
                access_count_at_reopen: 7,
            },
            restabilize_to: ReopenStableState::Consolidated,
            refresh_readiness: RefreshReadiness::Deferred,
        },
    ];

    let run = ReconsolidationRun::new(
        membrain_core::api::NamespaceId::new("test.recon.audit").unwrap(),
        ReconsolidationPolicy::default(),
        memories,
        120,
    );
    let mut handle = MaintenanceJobHandle::new(run, 10);
    handle.start();

    let completed_run = loop {
        let snapshot = handle.poll();
        match snapshot.state {
            MaintenanceJobState::Completed(_) => break handle.operation().clone(),
            MaintenanceJobState::Running { .. } => continue,
            _ => std::process::abort(),
        }
    };

    let entries = completed_run.append_only_audit_entries();
    assert_eq!(entries.len(), 3);
    assert!(entries[0].detail.contains("preserved_state=consolidated"));
    assert!(entries[1].detail.contains("preserved_state=synaptic_done"));
    assert!(entries[2].detail.contains("preserved_state=consolidated"));
    assert!(entries[0]
        .detail
        .contains("refresh_triggers=embedding_refresh,index_refresh,cache_invalidate"));
    assert!(entries[1].detail.contains("refresh_triggers=none"));
    assert!(entries[2]
        .detail
        .contains("refresh_triggers=embedding_refresh,index_refresh,cache_invalidate"));
}

// ── LTP/LTD lifecycle ────────────────────────────────────────────────────────

#[test]
fn ltp_then_recall_then_ltd_lifecycle() {
    let engine = StrengthEngine::new();
    let policy = StrengthPolicy::default();

    let mut state = StrengthState::with_base(0.3);
    state.stability = 2.0;
    state.last_accessed_tick = 0;

    engine.apply_ltp(&mut state, 1, &policy);
    assert!(state.base_strength > 0.3);

    engine.apply_ltp(&mut state, 5, &policy);
    assert!(state.base_strength > 0.4);

    let recall_fixture = DeterministicStrengthFixture::new(
        state.base_strength,
        state.stability,
        state.last_accessed_tick,
        50,
    );
    assert!(recall_fixture.effective_strength < recall_fixture.state.base_strength);
    assert!(recall_fixture.factors.recency < 0.7);

    let archive_fixture = DeterministicStrengthFixture::new(
        state.base_strength,
        state.stability,
        state.last_accessed_tick,
        1000,
    );
    if let DecayDecision::ApplyLTD { new_strength, .. } = archive_fixture.decay {
        assert!(new_strength < archive_fixture.state.base_strength);
    } else {
        assert!(matches!(
            archive_fixture.decay,
            DecayDecision::Archive { .. }
        ));
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
        idle_days: 0,
        guards: Default::default(),
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
        idle_days: 0,
        guards: Default::default(),
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
        let recon = recon_engine.tick(
            &labile,
            Some(&update),
            30,
            0.6,
            &recon_policy,
            membrain_core::engine::reconsolidation::RefreshReadiness::Ready,
        );
        assert_eq!(recon.outcome, ReconsolidationOutcome::Applied);
    }
}

// ── Edge cases ───────────────────────────────────────────────────────────────

#[test]
fn decayed_strength_produces_forgetting_eligible_score() {
    let fixture = DeterministicStrengthFixture::new(0.1, 1.0, 0, 1000);

    let forgetting = ForgettingEngine;
    let (action, _score) = forgetting.evaluate_memory_with_factors(
        mid(1),
        &fixture.factors,
        &ForgettingPolicy::default(),
    );

    assert!(!matches!(action, ForgettingAction::Skip));
    assert!(fixture.factors.recency < 0.1);
    assert!(fixture.effective_strength <= fixture.factors.effective_strength);
}

#[test]
fn max_strength_memory_resists_all_decay() {
    let policy = StrengthPolicy::default();
    let fixture = DeterministicStrengthFixture::new(1.0, 10.0, 0, 10);

    assert!(fixture.effective_strength > 0.3);

    if let DecayDecision::ApplyLTD { new_strength, .. } = fixture.decay {
        assert!(new_strength > 0.3);
    } else {
        assert!(!matches!(fixture.decay, DecayDecision::Archive { .. }));
    }
    assert!(fixture.factors.recency > policy.min_strength);
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

#[test]
fn timeout_fixture_replays_operator_visible_timeout_semantics() {
    let fixture = ScriptedMaintenanceFixture::timeout_fixture(2);

    assert_eq!(
        fixture.artifact.scenario_name,
        "deterministic_timeout_window"
    );
    assert_eq!(fixture.artifact.start_tick, 0);
    assert_eq!(fixture.artifact.tick_history, vec![0, 1, 2]);
    assert_eq!(fixture.polls_used, 2);
    assert_eq!(fixture.interrupted.reason, InterruptionReason::TimedOut);
    assert_eq!(
        fixture.interrupted.preserved_durable_state,
        DurableStateToken(81)
    );
    assert_eq!(
        fixture.recorded_interruptions,
        vec![InterruptionReason::TimedOut]
    );
}
