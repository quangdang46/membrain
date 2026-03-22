use crate::types::MemoryId;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ReconsolidationPolicy {
    pub base_window_ticks: u64,
    pub labile_strength_min: f32,
    pub labile_access_count_min: u32,
    pub reconsolidation_bonus: f32,
}

impl Default for ReconsolidationPolicy {
    fn default() -> Self {
        Self {
            base_window_ticks: 50,
            labile_strength_min: 0.2,
            labile_access_count_min: 1,
            reconsolidation_bonus: 0.05,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct LabileState {
    pub since_tick: u64,
    pub window_ticks: u64,
}

impl LabileState {
    pub fn new(since_tick: u64, window_ticks: u64) -> Self {
        Self {
            since_tick,
            window_ticks,
        }
    }

    pub fn is_expired(&self, current_tick: u64) -> bool {
        current_tick.saturating_sub(self.since_tick) >= self.window_ticks
    }

    pub fn remaining_ticks(&self, current_tick: u64) -> u64 {
        let elapsed = current_tick.saturating_sub(self.since_tick);
        self.window_ticks.saturating_sub(elapsed)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PendingUpdate {
    pub memory_id: MemoryId,
    pub new_content: Option<String>,
    pub new_emotional_arousal: Option<f32>,
    pub new_emotional_valence: Option<f32>,
    pub submitted_tick: u64,
    pub submitter: UpdateSource,
}

impl PendingUpdate {
    pub fn new(memory_id: MemoryId, submitted_tick: u64, submitter: UpdateSource) -> Self {
        Self {
            memory_id,
            new_content: None,
            new_emotional_arousal: None,
            new_emotional_valence: None,
            submitted_tick,
            submitter,
        }
    }

    pub fn with_content(mut self, content: String) -> Self {
        self.new_content = Some(content);
        self
    }

    pub fn with_emotional(mut self, arousal: f32, valence: f32) -> Self {
        self.new_emotional_arousal = Some(arousal);
        self.new_emotional_valence = Some(valence);
        self
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum UpdateSource {
    User,
    Agent,
    System,
    Consolidation,
}

impl UpdateSource {
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::User => "user",
            Self::Agent => "agent",
            Self::System => "system",
            Self::Consolidation => "consolidation",
        }
    }
}

pub struct ReconsolidationEngine;

impl ReconsolidationEngine {
    pub fn new() -> Self {
        Self
    }

    pub fn component_name(&self) -> &'static str {
        "engine.reconsolidation"
    }

    pub fn reconsolidation_window(
        &self,
        age_ticks: u64,
        base_strength: f32,
        policy: &ReconsolidationPolicy,
    ) -> u64 {
        if base_strength <= policy.labile_strength_min {
            return 0;
        }
        let base = policy.base_window_ticks as f32;
        let age_factor = 1.0_f32 / (1.0 + age_ticks as f32 / (10.0 * base));
        let strength_factor = 0.5 + base_strength * 0.5;
        (base * age_factor * strength_factor) as u64
    }

    pub fn should_enter_labile(
        &self,
        effective_strength: f32,
        access_count: u32,
        policy: &ReconsolidationPolicy,
    ) -> bool {
        effective_strength >= policy.labile_strength_min
            && access_count >= policy.labile_access_count_min
    }

    pub fn enter_labile(
        &self,
        current_tick: u64,
        age_ticks: u64,
        base_strength: f32,
        policy: &ReconsolidationPolicy,
    ) -> Option<LabileState> {
        let window = self.reconsolidation_window(age_ticks, base_strength, policy);
        if window == 0 {
            return None;
        }
        Some(LabileState::new(current_tick, window))
    }

    pub fn submit_pending_update(
        &self,
        memory_id: MemoryId,
        current_tick: u64,
        submitter: UpdateSource,
    ) -> PendingUpdate {
        PendingUpdate::new(memory_id, current_tick, submitter)
    }

    pub fn is_window_expired(&self, state: &LabileState, current_tick: u64) -> bool {
        state.is_expired(current_tick)
    }

    pub fn remaining_window(&self, state: &LabileState, current_tick: u64) -> u64 {
        state.remaining_ticks(current_tick)
    }

    pub fn apply_update_to_strength(
        &self,
        current_strength: f32,
        bonus: f32,
        max_strength: f32,
    ) -> f32 {
        (current_strength + bonus).min(max_strength)
    }
}

impl Default for ReconsolidationEngine {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn policy() -> ReconsolidationPolicy {
        ReconsolidationPolicy::default()
    }

    #[test]
    fn window_decreases_with_age() {
        let engine = ReconsolidationEngine::new();
        let policy = policy();
        let w0 = engine.reconsolidation_window(0, 0.5, &policy);
        let w100 = engine.reconsolidation_window(100, 0.5, &policy);
        let w500 = engine.reconsolidation_window(500, 0.5, &policy);
        assert!(w0 > w100);
        assert!(w100 > w500);
        assert!(w0 > 0);
    }

    #[test]
    fn window_increases_with_strength() {
        let engine = ReconsolidationEngine::new();
        let policy = policy();
        let w_weak = engine.reconsolidation_window(100, 0.25, &policy);
        let w_strong = engine.reconsolidation_window(100, 0.8, &policy);
        assert!(w_strong > w_weak);
    }

    #[test]
    fn window_zero_for_weak_memories() {
        let engine = ReconsolidationEngine::new();
        let policy = policy();
        assert_eq!(engine.reconsolidation_window(0, 0.1, &policy), 0);
        assert_eq!(engine.reconsolidation_window(0, 0.19, &policy), 0);
        assert_eq!(engine.reconsolidation_window(0, 0.2, &policy), 0);
        assert!(engine.reconsolidation_window(0, 0.21, &policy) > 0);
    }

    #[test]
    fn should_enter_labile_requires_strength_and_accesses() {
        let engine = ReconsolidationEngine::new();
        let policy = policy();
        assert!(!engine.should_enter_labile(0.1, 5, &policy));
        assert!(!engine.should_enter_labile(0.5, 0, &policy));
        assert!(engine.should_enter_labile(0.5, 1, &policy));
    }

    #[test]
    fn enter_labile_returns_none_for_weak_memory() {
        let engine = ReconsolidationEngine::new();
        let policy = policy();
        assert!(engine.enter_labile(100, 0, 0.1, &policy).is_none());
    }

    #[test]
    fn enter_labile_returns_state_for_eligible_memory() {
        let engine = ReconsolidationEngine::new();
        let policy = policy();
        let result = engine.enter_labile(100, 0, 0.5, &policy);
        assert!(result.is_some());
        let state = result.unwrap();
        assert_eq!(state.since_tick, 100);
        assert!(state.window_ticks > 0);
    }

    #[test]
    fn labile_state_expires() {
        let state = LabileState::new(100, 50);
        assert!(!state.is_expired(100));
        assert!(!state.is_expired(149));
        assert!(state.is_expired(150));
        assert!(state.is_expired(200));
    }

    #[test]
    fn labile_state_remaining_ticks() {
        let state = LabileState::new(100, 50);
        assert_eq!(state.remaining_ticks(100), 50);
        assert_eq!(state.remaining_ticks(125), 25);
        assert_eq!(state.remaining_ticks(150), 0);
        assert_eq!(state.remaining_ticks(200), 0);
    }

    #[test]
    fn pending_update_builder() {
        let update = PendingUpdate::new(MemoryId(42), 100, UpdateSource::User)
            .with_content("updated text".to_string())
            .with_emotional(0.8, 0.3);
        assert_eq!(update.memory_id, MemoryId(42));
        assert_eq!(update.submitted_tick, 100);
        assert_eq!(update.submitter, UpdateSource::User);
        assert_eq!(update.new_content, Some("updated text".to_string()));
        assert_eq!(update.new_emotional_arousal, Some(0.8));
        assert_eq!(update.new_emotional_valence, Some(0.3));
    }

    #[test]
    fn recall_alone_does_not_create_pending_update() {
        let engine = ReconsolidationEngine::new();
        let update = engine.submit_pending_update(MemoryId(1), 100, UpdateSource::System);
        assert_eq!(update.new_content, None);
        assert_eq!(update.new_emotional_arousal, None);
        assert_eq!(update.new_emotional_valence, None);
    }

    #[test]
    fn is_window_expired() {
        let engine = ReconsolidationEngine::new();
        let state = LabileState::new(100, 50);
        assert!(!engine.is_window_expired(&state, 149));
        assert!(engine.is_window_expired(&state, 150));
    }

    #[test]
    fn remaining_window() {
        let engine = ReconsolidationEngine::new();
        let state = LabileState::new(100, 50);
        assert_eq!(engine.remaining_window(&state, 125), 25);
        assert_eq!(engine.remaining_window(&state, 150), 0);
    }

    #[test]
    fn apply_update_bonus_respects_max() {
        let engine = ReconsolidationEngine::new();
        let result = engine.apply_update_to_strength(0.95, 0.10, 1.0);
        assert_eq!(result, 1.0);
        let result2 = engine.apply_update_to_strength(0.50, 0.05, 1.0);
        assert!((result2 - 0.55).abs() < 1e-6);
    }

    #[test]
    fn update_source_as_str() {
        assert_eq!(UpdateSource::User.as_str(), "user");
        assert_eq!(UpdateSource::Agent.as_str(), "agent");
        assert_eq!(UpdateSource::System.as_str(), "system");
        assert_eq!(UpdateSource::Consolidation.as_str(), "consolidation");
    }
}
