#[derive(Debug, Clone, Copy, PartialEq)]
pub struct StrengthPolicy {
    pub ltp_delta: f32,
    pub stability_increment: f32,
    pub max_strength: f32,
    pub min_strength: f32,
    pub base_strength: f32,
    pub arousal_threshold: f32,
    pub valence_threshold: f32,
    pub decay_interaction_budget: usize,
}

impl Default for StrengthPolicy {
    fn default() -> Self {
        Self {
            ltp_delta: 0.1,
            stability_increment: 0.1,
            max_strength: 1.0,
            min_strength: 0.1,
            base_strength: 0.5,
            arousal_threshold: 0.7,
            valence_threshold: 0.5,
            decay_interaction_budget: 100,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct StrengthState {
    pub base_strength: f32,
    pub stability: f32,
    pub last_accessed_tick: u64,
    pub access_count: u32,
    pub bypass_decay: bool,
    pub emotional_arousal: f32,
    pub emotional_valence: f32,
}

impl StrengthState {
    pub fn new(base_strength: f32, bypass_decay: bool, arousal: f32, valence: f32) -> Self {
        Self {
            base_strength,
            stability: 1.0,
            last_accessed_tick: 0,
            access_count: 0,
            bypass_decay,
            emotional_arousal: arousal,
            emotional_valence: valence,
        }
    }

    pub fn with_base(base: f32) -> Self {
        Self::new(base, false, 0.0, 0.0)
    }

    pub fn with_emotional(base: f32, arousal: f32, valence: f32) -> Self {
        let bypass = arousal > 0.7 && valence.abs() > 0.5;
        Self::new(base, bypass, arousal, valence)
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct LtpResult {
    pub previous_strength: f32,
    pub new_strength: f32,
    pub previous_stability: f32,
    pub new_stability: f32,
    pub new_access_count: u32,
    pub tick: u64,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct LtdResult {
    pub previous_strength: f32,
    pub new_strength: f32,
    pub retention: f32,
    pub interactions_elapsed: u64,
    pub exceeded_min: bool,
    pub archive_eligible: bool,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum DecayDecision {
    ApplyLTD {
        previous_strength: f32,
        new_strength: f32,
        retention: f32,
    },
    Bypass {
        reason: &'static str,
        effective_strength: f32,
    },
    Archive {
        reason: &'static str,
        final_strength: f32,
    },
    Skip,
}

#[derive(Debug, Clone, PartialEq)]
pub struct StrengthTrace {
    pub memory_id: u64,
    pub operation: &'static str,
    pub previous_strength: f32,
    pub new_strength: f32,
    pub previous_stability: f32,
    pub new_stability: f32,
    pub effective_strength: f32,
    pub access_count: u32,
    pub tick: u64,
    pub bypass_decay: bool,
}

pub struct StrengthEngine;

impl StrengthEngine {
    pub fn new() -> Self {
        Self
    }

    pub fn component_name(&self) -> &'static str {
        "engine.strength"
    }

    pub fn apply_ltp(
        &self,
        state: &mut StrengthState,
        current_tick: u64,
        policy: &StrengthPolicy,
    ) -> LtpResult {
        let prev_strength = state.base_strength;
        let prev_stability = state.stability;
        state.base_strength = (state.base_strength + policy.ltp_delta).min(policy.max_strength);
        let max_stability = policy.max_strength * 10.0;
        state.stability = (state.stability + policy.stability_increment).min(max_stability);
        state.last_accessed_tick = current_tick;
        state.access_count += 1;
        LtpResult {
            previous_strength: prev_strength,
            new_strength: state.base_strength,
            previous_stability: prev_stability,
            new_stability: state.stability,
            new_access_count: state.access_count,
            tick: current_tick,
        }
    }

    /// Returns effective strength using lazy Ebbinghaus decay.
    ///
    /// R(t) = e^(-elapsed/stability) * base_strength
    /// where elapsed = current_tick - last_accessed_tick.
    /// O(1) computation; no wall-clock iteration.
    pub fn effective_strength(&self, state: &StrengthState, current_tick: u64) -> f32 {
        if state.bypass_decay {
            return state.base_strength;
        }
        let elapsed = current_tick.saturating_sub(state.last_accessed_tick);
        let retention = Self::ebbinghaus_retention(state.stability, elapsed);
        state.base_strength * retention
    }

    /// Applies LTD decay by computing the new base_strength from tick-based elapsed time.
    ///
    /// Returns None if bypass_decay is set (emotional memories resist LTD).
    /// On return, state.base_strength is updated to the decayed value and
    /// last_accessed_tick is set to current_tick.
    pub fn apply_ltd(
        &self,
        state: &mut StrengthState,
        current_tick: u64,
        policy: &StrengthPolicy,
    ) -> Option<LtdResult> {
        if state.bypass_decay {
            return None;
        }
        let elapsed = current_tick.saturating_sub(state.last_accessed_tick);
        let retention = Self::ebbinghaus_retention(state.stability, elapsed);
        let prev_strength = state.base_strength;
        let new_strength = (prev_strength * retention).max(0.0);
        state.base_strength = new_strength;
        state.last_accessed_tick = current_tick;
        let exceeded_min = new_strength < policy.min_strength;
        Some(LtdResult {
            previous_strength: prev_strength,
            new_strength,
            retention,
            interactions_elapsed: elapsed,
            exceeded_min,
            archive_eligible: exceeded_min && !state.bypass_decay,
        })
    }

    /// Evaluates decay decision using tick-based elapsed time.
    ///
    /// Returns the appropriate action without mutating state.
    pub fn evaluate_decay(
        &self,
        state: &StrengthState,
        current_tick: u64,
        policy: &StrengthPolicy,
    ) -> DecayDecision {
        if state.bypass_decay {
            return DecayDecision::Bypass {
                reason: "emotional_bypass",
                effective_strength: state.base_strength,
            };
        }
        let elapsed = current_tick.saturating_sub(state.last_accessed_tick);
        let retention = Self::ebbinghaus_retention(state.stability, elapsed);
        let effective = state.base_strength * retention;
        if effective < policy.min_strength {
            return DecayDecision::Archive {
                reason: "below_min_strength",
                final_strength: effective,
            };
        }
        DecayDecision::ApplyLTD {
            previous_strength: state.base_strength,
            new_strength: effective,
            retention,
        }
    }

    pub fn trace_ltp(
        &self,
        memory_id: u64,
        prev: &StrengthState,
        new_strength: f32,
        new_stability: f32,
        tick: u64,
    ) -> StrengthTrace {
        StrengthTrace {
            memory_id,
            operation: "ltp",
            previous_strength: prev.base_strength,
            new_strength,
            previous_stability: prev.stability,
            new_stability,
            effective_strength: new_strength,
            access_count: prev.access_count + 1,
            tick,
            bypass_decay: prev.bypass_decay,
        }
    }

    /// Builds an EligibilityFactors from strength state for ForgettingEngine integration.
    ///
    /// Maps strength, emotional, and bypass signals into the multi-factor
    /// eligibility model without duplicating decay logic.
    pub fn to_eligibility_factors(
        &self,
        state: &StrengthState,
        current_tick: u64,
    ) -> crate::engine::forgetting::EligibilityFactors {
        let effective = self.effective_strength(state, current_tick);
        let recency = if current_tick <= state.last_accessed_tick {
            1.0
        } else {
            let elapsed = current_tick - state.last_accessed_tick;
            (1.0 / (1.0 + elapsed as f32 * 0.01)).clamp(0.0, 1.0)
        };
        let access_frequency = if state.access_count == 0 {
            0.0
        } else {
            (state.access_count as f32 / 100.0).clamp(0.0, 1.0)
        };
        crate::engine::forgetting::EligibilityFactors {
            effective_strength: effective,
            recency,
            access_frequency,
            in_contradiction: false,
            emotional_arousal: state.emotional_arousal,
            bypass_decay: state.bypass_decay,
            idle_days: current_tick.saturating_sub(state.last_accessed_tick) as u32,
            guards: crate::engine::forgetting::EligibilityGuards::default(),
        }
    }

    fn ebbinghaus_retention(stability: f32, interactions_elapsed: u64) -> f32 {
        if stability <= 0.0 {
            return 0.0;
        }
        let exponent = -(interactions_elapsed as f32) / stability;
        exponent.exp().clamp(0.0, 1.0)
    }

    pub fn should_bypass_decay(&self, arousal: f32, valence: f32, policy: &StrengthPolicy) -> bool {
        arousal > policy.arousal_threshold && valence.abs() > policy.valence_threshold
    }

    pub fn create_initial_state(
        &self,
        base_strength: f32,
        arousal: f32,
        valence: f32,
        _policy: &StrengthPolicy,
    ) -> StrengthState {
        StrengthState::with_emotional(base_strength, arousal, valence)
    }

    pub fn is_ltp_monotonic(&self, current_strength: f32, policy: &StrengthPolicy) -> bool {
        let after = (current_strength + policy.ltp_delta).min(policy.max_strength);
        after >= current_strength
    }
}

impl Default for StrengthEngine {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn policy() -> StrengthPolicy {
        StrengthPolicy::default()
    }

    fn state(base: f32) -> StrengthState {
        StrengthState::with_base(base)
    }

    fn emotional_state(base: f32, arousal: f32, valence: f32) -> StrengthState {
        StrengthState::with_emotional(base, arousal, valence)
    }

    #[test]
    fn ltp_increases_strength_and_stability() {
        let engine = StrengthEngine::new();
        let mut s = state(0.5);
        s.stability = 1.0;
        let result = engine.apply_ltp(&mut s, 100, &policy());
        assert!(result.new_strength > result.previous_strength);
        assert_eq!(result.previous_strength, 0.5);
        assert!(result.new_strength > 0.5);
        assert!(result.new_stability > result.previous_stability);
        assert_eq!(result.tick, 100);
        assert_eq!(result.new_access_count, 1);
    }

    #[test]
    fn ltp_respects_max_strength_cap() {
        let engine = StrengthEngine::new();
        let mut s = state(0.95);
        s.stability = 1.0;
        let result = engine.apply_ltp(&mut s, 1, &policy());
        assert_eq!(result.new_strength, 1.0);
    }

    #[test]
    fn ltp_is_monotonic() {
        let engine = StrengthEngine::new();
        let policy = policy();
        for base in [0.0_f32, 0.3, 0.5, 0.9, 0.99] {
            assert!(engine.is_ltp_monotonic(base, &policy));
        }
    }

    #[test]
    fn effective_strength_returns_base_when_bypass() {
        let engine = StrengthEngine::new();
        let s = emotional_state(0.8, 0.9, 0.6);
        let effective = engine.effective_strength(&s, 1000);
        assert_eq!(effective, s.base_strength);
    }

    #[test]
    fn effective_strength_applies_ebbinghaus_decay_without_bypass() {
        let engine = StrengthEngine::new();
        let mut s = state(0.5);
        s.stability = 1.0;
        s.last_accessed_tick = 0;
        let effective = engine.effective_strength(&s, 10);
        let retention = (-(10_f32) / 1.0).exp();
        let expected = 0.5 * retention;
        assert!((effective - expected).abs() < 1e-6);
        assert!(effective < 0.5);
    }

    #[test]
    fn effective_strength_increases_with_stability() {
        let engine = StrengthEngine::new();
        let mut weak = state(0.5);
        weak.stability = 1.0;
        weak.last_accessed_tick = 0;
        let mut strong = state(0.5);
        strong.stability = 10.0;
        strong.last_accessed_tick = 0;
        let weak_eff = engine.effective_strength(&weak, 10);
        let strong_eff = engine.effective_strength(&strong, 10);
        assert!(strong_eff > weak_eff);
    }

    #[test]
    fn ltd_applies_decay_without_bypass() {
        let engine = StrengthEngine::new();
        let mut s = state(0.5);
        s.stability = 1.0;
        s.last_accessed_tick = 0;
        let result = engine.apply_ltd(&mut s, 10, &policy());
        assert!(result.is_some());
        let r = result.unwrap();
        assert!(r.new_strength < r.previous_strength);
        assert!(r.retention < 1.0);
        assert!(r.retention >= 0.0);
    }

    #[test]
    fn ltd_returns_none_for_bypass() {
        let engine = StrengthEngine::new();
        let mut s = emotional_state(0.8, 0.9, 0.6);
        let result = engine.apply_ltd(&mut s, 100, &policy());
        assert!(result.is_none());
    }

    #[test]
    fn ltd_detects_archive_eligibility() {
        let engine = StrengthEngine::new();
        let mut s = state(0.12);
        s.stability = 1.0;
        s.last_accessed_tick = 0;
        let result = engine.apply_ltd(&mut s, 100, &policy());
        assert!(result.is_some());
        let r = result.unwrap();
        assert!(r.exceeded_min);
        assert!(r.archive_eligible);
    }

    #[test]
    fn evaluate_decay_bypass_emotional() {
        let engine = StrengthEngine::new();
        let s = emotional_state(0.5, 0.9, 0.6);
        let decision = engine.evaluate_decay(&s, 100, &policy());
        assert!(
            matches!(decision, DecayDecision::Bypass { reason, .. } if reason == "emotional_bypass")
        );
    }

    #[test]
    fn evaluate_decay_apply_ltd() {
        let engine = StrengthEngine::new();
        let mut s = state(0.5);
        s.stability = 2.0;
        s.last_accessed_tick = 0;
        let decision = engine.evaluate_decay(&s, 3, &policy());
        match decision {
            DecayDecision::ApplyLTD { new_strength, .. } => {
                assert!(new_strength < 0.5);
            }
            other => panic!("expected ApplyLTD, got {:?}", other),
        }
    }

    #[test]
    fn evaluate_decay_archive_when_below_min() {
        let engine = StrengthEngine::new();
        let mut s = state(0.12);
        s.stability = 1.0;
        s.last_accessed_tick = 0;
        let decision = engine.evaluate_decay(&s, 100, &policy());
        match decision {
            DecayDecision::Archive {
                reason,
                final_strength,
            } => {
                assert_eq!(reason, "below_min_strength");
                assert!(final_strength < 0.1);
            }
            other => panic!("expected Archive, got {:?}", other),
        }
    }

    #[test]
    fn ebbinghaus_retention_decreases_with_interactions() {
        let stability = 10.0;
        let r0 = StrengthEngine::ebbinghaus_retention(stability, 0);
        let r1 = StrengthEngine::ebbinghaus_retention(stability, 1);
        let r10 = StrengthEngine::ebbinghaus_retention(stability, 10);
        assert!((r0 - 1.0).abs() < 1e-6);
        assert!(r10 < r1);
        assert!(r1 < r0);
    }

    #[test]
    fn ebbinghaus_retention_returns_zero_for_zero_stability() {
        assert_eq!(StrengthEngine::ebbinghaus_retention(0.0, 5), 0.0);
    }

    #[test]
    fn should_bypass_decay_requires_both_thresholds() {
        let engine = StrengthEngine::new();
        let policy = policy();
        assert!(!engine.should_bypass_decay(0.8, 0.3, &policy));
        assert!(!engine.should_bypass_decay(0.3, 0.8, &policy));
        assert!(engine.should_bypass_decay(0.8, 0.6, &policy));
    }

    #[test]
    fn create_initial_state_applies_bypass() {
        let engine = StrengthEngine::new();
        let state = engine.create_initial_state(0.5, 0.9, 0.6, &policy());
        assert!(state.bypass_decay);
        assert_eq!(state.base_strength, 0.5);
        assert_eq!(state.stability, 1.0);
        assert_eq!(state.access_count, 0);
    }

    #[test]
    fn ltd_retains_stability() {
        let engine = StrengthEngine::new();
        let mut s = state(0.5);
        s.stability = 5.0;
        s.last_accessed_tick = 0;
        let result = engine.apply_ltd(&mut s, 20, &policy());
        assert!(result.is_some());
        let r = result.unwrap();
        assert_eq!(s.stability, 5.0);
        assert!(r.retention > 0.0);
    }

    #[test]
    fn effective_strength_is_deterministic_for_same_state_and_tick() {
        let engine = StrengthEngine::new();
        let mut s = state(0.5);
        s.stability = 10.0;
        s.last_accessed_tick = 5;
        let eff1 = engine.effective_strength(&s, 100);
        let eff2 = engine.effective_strength(&s, 100);
        assert_eq!(eff1, eff2);
    }

    #[test]
    fn no_wall_clock_dependency() {
        let engine = StrengthEngine::new();
        let mut s = state(0.5);
        s.stability = 10.0;
        s.last_accessed_tick = 5;
        // Same input ticks always produce same output — no wall clock
        let eff_a = engine.effective_strength(&s, 100);
        let eff_b = engine.effective_strength(&s, 100);
        assert_eq!(eff_a, eff_b);
        // Different elapsed ticks produce different but deterministic results
        let eff_c = engine.effective_strength(&s, 200);
        let eff_d = engine.effective_strength(&s, 200);
        assert_eq!(eff_c, eff_d);
        assert!(eff_c < eff_a, "more elapsed = less effective strength");
    }

    // ── Deterministic fixtures ──────────────────────────────────────────

    #[test]
    fn fixture_ltp_monotonic_over_sequence() {
        let engine = StrengthEngine::new();
        let mut s = state(0.1);
        s.stability = 1.0;
        let p = policy();
        let mut prev_strength = s.base_strength;
        let mut prev_stability = s.stability;
        for tick in 1..=20 {
            let result = engine.apply_ltp(&mut s, tick, &p);
            assert!(
                result.new_strength >= prev_strength,
                "LTP must be monotonic at tick {tick}"
            );
            assert!(
                result.new_stability >= prev_stability,
                "stability must increase at tick {tick}"
            );
            prev_strength = result.new_strength;
            prev_stability = result.new_stability;
        }
        assert_eq!(s.base_strength, 1.0); // capped at max after 20 LTPs from 0.1
        assert_eq!(s.access_count, 20);
    }

    #[test]
    fn fixture_ltd_bounded_decay() {
        let engine = StrengthEngine::new();
        let mut s = state(0.5);
        s.stability = 10.0;
        s.last_accessed_tick = 0;
        let p = policy();

        let mut strengths = Vec::new();
        for tick in [10, 50, 100, 200] {
            let result = engine.apply_ltd(&mut s, tick, &p);
            assert!(result.is_some());
            strengths.push(result.unwrap().new_strength);
        }

        // Each successive LTD at higher tick should produce lower or equal strength
        for w in strengths.windows(2) {
            assert!(w[1] <= w[0], "LTD must be bounded");
        }
    }

    #[test]
    fn fixture_bypass_decay_blocks_ltd() {
        let engine = StrengthEngine::new();
        let mut s = emotional_state(0.8, 0.9, 0.6);
        let p = policy();

        let result = engine.apply_ltd(&mut s, 1000, &p);
        assert!(result.is_none(), "bypass must block LTD");

        let decision = engine.evaluate_decay(&s, 1000, &p);
        assert!(
            matches!(decision, DecayDecision::Bypass { reason, .. } if reason == "emotional_bypass"),
            "bypass must be preserved in decay decision"
        );
    }

    #[test]
    fn fixture_archive_threshold_proven() {
        let engine = StrengthEngine::new();
        let mut s = state(0.15);
        s.stability = 1.0;
        s.last_accessed_tick = 0;
        let p = policy();

        // At tick 1, retention = e^(-1) ~ 0.368, effective ~ 0.055 < 0.1
        let decision = engine.evaluate_decay(&s, 1, &p);
        match decision {
            DecayDecision::Archive {
                reason,
                final_strength,
            } => {
                assert_eq!(reason, "below_min_strength");
                assert!(final_strength < p.min_strength);
            }
            other => panic!("expected Archive at tick 1, got {:?}", other),
        }
    }

    #[test]
    fn fixture_no_wall_clock_in_strength_engine() {
        let engine = StrengthEngine::new();
        let mut s = state(0.5);
        s.stability = 5.0;
        s.last_accessed_tick = 100;

        // effective_strength at tick 200 and tick 2000000 should differ (more elapsed = less strength)
        let eff_200 = engine.effective_strength(&s, 200);
        let eff_2m = engine.effective_strength(&s, 2_000_000);
        assert!(
            eff_2m < eff_200,
            "more elapsed tick must mean less effective strength"
        );

        // But for the same tick, result is always the same (deterministic)
        assert_eq!(eff_200, engine.effective_strength(&s, 200));
        assert_eq!(eff_2m, engine.effective_strength(&s, 2_000_000));
    }

    #[test]
    fn fixture_ltp_to_ltd_roundtrip() {
        let engine = StrengthEngine::new();
        let mut s = state(0.3);
        s.stability = 2.0;
        s.last_accessed_tick = 0;
        let p = policy();

        // Apply LTP to strengthen
        let ltp = engine.apply_ltp(&mut s, 5, &p);
        assert!(ltp.new_strength > 0.3);

        // Apply LTD to decay
        let ltd = engine.apply_ltd(&mut s, 20, &p);
        assert!(ltd.is_some());
        let ltd = ltd.unwrap();
        assert!(ltd.new_strength < ltp.new_strength);
        assert!(ltd.new_strength >= 0.0);
    }

    #[test]
    fn fixture_stability_increases_ltd_resistance() {
        let engine = StrengthEngine::new();
        let p = policy();

        let mut low_stab = state(0.5);
        low_stab.stability = 1.0;
        low_stab.last_accessed_tick = 0;

        let mut high_stab = state(0.5);
        high_stab.stability = 20.0;
        high_stab.last_accessed_tick = 0;

        let low_ltd = engine.apply_ltd(&mut low_stab, 10, &p).unwrap();
        let high_ltd = engine.apply_ltd(&mut high_stab, 10, &p).unwrap();

        // Higher stability = more retention after same elapsed time
        assert!(
            high_ltd.new_strength > low_ltd.new_strength,
            "higher stability must resist LTD better"
        );
    }

    #[test]
    fn fixture_to_eligibility_factors_maps_correctly() {
        let engine = StrengthEngine::new();
        let mut s = state(0.5);
        s.stability = 1.0;
        s.last_accessed_tick = 100;
        s.access_count = 50;
        s.emotional_arousal = 0.8;

        let factors = engine.to_eligibility_factors(&s, 200);
        assert!(factors.effective_strength > 0.0);
        assert!(factors.effective_strength <= 0.5);
        assert!(factors.recency > 0.0 && factors.recency <= 1.0);
        assert_eq!(factors.emotional_arousal, 0.8);
        assert!(!factors.bypass_decay);
    }

    #[test]
    fn fixture_to_eligibility_factors_preserves_bypass() {
        let engine = StrengthEngine::new();
        let s = emotional_state(0.8, 0.9, 0.6);
        let factors = engine.to_eligibility_factors(&s, 1000);
        assert!(factors.bypass_decay);
        assert_eq!(factors.effective_strength, 0.8);
    }
}
