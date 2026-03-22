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

    pub fn effective_strength(&self, state: &StrengthState, _current_tick: u64) -> f32 {
        if state.bypass_decay {
            return state.base_strength;
        }
        let retention = Self::ebbinghaus_retention(state.stability, state.access_count as u64);
        state.base_strength * retention
    }

    pub fn apply_ltd(
        &self,
        state: &mut StrengthState,
        _current_tick: u64,
        policy: &StrengthPolicy,
    ) -> Option<LtdResult> {
        if state.bypass_decay {
            return None;
        }
        let interactions_elapsed = state.access_count as u64;
        let retention = Self::ebbinghaus_retention(state.stability, interactions_elapsed);
        let prev_strength = state.base_strength;
        state.base_strength *= retention;
        let exceeded_min = state.base_strength < policy.min_strength;
        Some(LtdResult {
            previous_strength: prev_strength,
            new_strength: state.base_strength,
            retention,
            interactions_elapsed,
            exceeded_min,
            archive_eligible: exceeded_min && !state.bypass_decay,
        })
    }

    pub fn evaluate_decay(
        &self,
        state: &StrengthState,
        _current_tick: u64,
        policy: &StrengthPolicy,
    ) -> DecayDecision {
        if state.bypass_decay {
            return DecayDecision::Bypass {
                reason: "emotional_bypass",
                effective_strength: state.base_strength,
            };
        }
        let retention = Self::ebbinghaus_retention(state.stability, state.access_count as u64);
        let effective = state.base_strength * retention;
        if effective < policy.min_strength {
            return DecayDecision::Archive {
                reason: "below_min_strength",
                final_strength: effective,
            };
        }
        DecayDecision::ApplyLTD {
            previous_strength: state.base_strength,
            new_strength: state.base_strength * retention,
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
        s.access_count = 10;
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
        weak.access_count = 10;
        let mut strong = state(0.5);
        strong.stability = 10.0;
        strong.access_count = 10;
        let weak_eff = engine.effective_strength(&weak, 10);
        let strong_eff = engine.effective_strength(&strong, 10);
        assert!(strong_eff > weak_eff);
    }

    #[test]
    fn ltd_applies_decay_without_bypass() {
        let engine = StrengthEngine::new();
        let mut s = state(0.5);
        s.stability = 1.0;
        s.access_count = 10;
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
        s.access_count = 100;
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
        s.access_count = 3;
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
        s.access_count = 100;
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
        s.access_count = 20;
        let result = engine.apply_ltd(&mut s, 20, &policy());
        assert!(result.is_some());
        let r = result.unwrap();
        assert_eq!(s.stability, 5.0);
        assert!(r.retention > 0.0);
    }

    #[test]
    fn no_wall_clock_dependency() {
        let engine = StrengthEngine::new();
        let mut s = state(0.5);
        s.stability = 10.0;
        s.last_accessed_tick = 5;
        s.access_count = 3;
        let eff_now = engine.effective_strength(&s, 100);
        let eff_later = engine.effective_strength(&s, 1_000_000);
        assert_eq!(eff_now, eff_later);
    }
}
