use crate::types::MemoryId;

/// Policy controlling reconsolidation window behavior and apply bonuses.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ReconsolidationPolicy {
    /// Base labile window duration in logical ticks.
    pub base_window_ticks: u64,
    /// Minimum effective strength required to reopen a labile window.
    pub labile_strength_min: f32,
    /// Minimum access count required to reopen a labile window.
    pub labile_access_count_min: u32,
    /// Strength bonus awarded on successful reconsolidation apply.
    pub reconsolidation_bonus: f32,
    /// Maximum strength cap applied after reconsolidation bonus.
    pub max_strength: f32,
}

impl Default for ReconsolidationPolicy {
    fn default() -> Self {
        Self {
            base_window_ticks: 50,
            labile_strength_min: 0.2,
            labile_access_count_min: 1,
            reconsolidation_bonus: 0.05,
            max_strength: 1.0,
        }
    }
}

/// Explicit outcome of a reconsolidation tick evaluation.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ReconsolidationOutcome {
    /// Pending update was applied and the memory restabilized.
    Applied,
    /// Pending update was discarded because the labile window expired.
    DiscardedStale,
    /// No pending update existed when the tick ran.
    NoPendingUpdate,
    /// The labile window was not yet expired and no update existed yet.
    WindowStillOpen,
    /// The update stayed pending because derived-state refresh must be retried later.
    DeferredRefresh,
    /// The update could not be applied because derived-state refresh was blocked.
    BlockedRefreshFailure,
}

impl ReconsolidationOutcome {
    /// Stable machine-readable name.
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Applied => "applied",
            Self::DiscardedStale => "discarded_stale",
            Self::NoPendingUpdate => "no_pending_update",
            Self::WindowStillOpen => "window_still_open",
            Self::DeferredRefresh => "deferred_refresh",
            Self::BlockedRefreshFailure => "blocked_refresh_failure",
        }
    }
}

/// Inspectable readiness state for derived-state refresh work.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RefreshReadiness {
    /// Refresh hooks are ready, so apply may proceed.
    Ready,
    /// Refresh hooks are deferred; leave the update pending and try again later.
    Deferred,
    /// Refresh hooks failed or are blocked; do not mutate authoritative state.
    Failed,
}

/// Derived-state refresh signals emitted after a successful reconsolidation apply.
///
/// These are not performed inline — they are emitted as inspectable triggers
/// so downstream consumers (cache, index, embedding) can refresh from durable truth.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RefreshTrigger {
    /// Embedding must be recomputed for the updated memory.
    EmbeddingRefresh,
    /// Index entries referencing this memory must be reprojected.
    IndexRefresh,
    /// Cache entries for this memory must be invalidated and re-fetched.
    CacheInvalidate,
}

impl RefreshTrigger {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::EmbeddingRefresh => "embedding_refresh",
            Self::IndexRefresh => "index_refresh",
            Self::CacheInvalidate => "cache_invalidate",
        }
    }
}

/// Immutable audit record for one reconsolidation tick evaluation.
#[derive(Debug, Clone, PartialEq)]
pub struct ReconsolidationAuditRecord {
    /// Memory that was evaluated.
    pub memory_id: MemoryId,
    /// Logical tick when the evaluation ran.
    pub tick: u64,
    /// Outcome of the tick.
    pub outcome: ReconsolidationOutcome,
    /// Strength before the tick (if apply occurred).
    pub strength_before: Option<f32>,
    /// Strength after the tick (if apply occurred).
    pub strength_after: Option<f32>,
    /// Refresh triggers emitted (populated only on Applied).
    pub refresh_triggers: Vec<RefreshTrigger>,
    /// Source of the pending update that was applied (if any).
    pub applied_update_source: Option<UpdateSource>,
    /// Whether the tick consumed or discarded any pending update.
    pub pending_update_cleared: bool,
    /// Whether the memory returned to stable state after this tick.
    pub restabilized: bool,
}

/// Captured stable state before a labile window reopens.
///
/// This preserves the pre-reopen authoritative state so that discard or
/// restabilization can restore it without recomputation.
#[derive(Debug, Clone, PartialEq)]
pub struct PreReopenState {
    /// Memory this snapshot belongs to.
    pub memory_id: MemoryId,
    /// Tick when the window was opened.
    pub reopen_tick: u64,
    /// Base strength at the moment of reopen.
    pub strength_at_reopen: f32,
    /// Stability at the moment of reopen.
    pub stability_at_reopen: f32,
    /// Access count at the moment of reopen.
    pub access_count_at_reopen: u32,
}

/// Rejection reasons for pending-update submission.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UpdateRejectionReason {
    /// The labile window has already expired.
    WindowExpired,
    /// The update has no content or emotional fields set.
    EmptyUpdate,
    /// The submitted tick is before the window opened.
    SubmittedBeforeWindow,
    /// Emotional arousal is outside the valid 0.0..=1.0 range.
    ArousalOutOfRange,
    /// Emotional valence is outside the valid -1.0..=1.0 range.
    ValenceOutOfRange,
}

impl UpdateRejectionReason {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::WindowExpired => "window_expired",
            Self::EmptyUpdate => "empty_update",
            Self::SubmittedBeforeWindow => "submitted_before_window",
            Self::ArousalOutOfRange => "arousal_out_of_range",
            Self::ValenceOutOfRange => "valence_out_of_range",
        }
    }
}

/// Result of validating a pending update submission.
#[derive(Debug, Clone, PartialEq)]
pub enum UpdateValidationResult {
    /// Update is valid and accepted.
    Accepted(PendingUpdate),
    /// Update is rejected with a reason.
    Rejected(UpdateRejectionReason),
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

/// Canonical reconsolidation engine owned by the core crate.
pub struct ReconsolidationEngine;

impl ReconsolidationEngine {
    pub fn new() -> Self {
        Self
    }

    pub fn component_name(&self) -> &'static str {
        "engine.reconsolidation"
    }

    /// Computes the labile window size based on age and strength.
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

    /// Returns whether the memory meets the thresholds to enter labile state.
    pub fn should_enter_labile(
        &self,
        effective_strength: f32,
        access_count: u32,
        policy: &ReconsolidationPolicy,
    ) -> bool {
        effective_strength >= policy.labile_strength_min
            && access_count >= policy.labile_access_count_min
    }

    /// Opens a labile window if eligibility rules are met.
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

    /// Creates a bare pending update for the given memory.
    pub fn submit_pending_update(
        &self,
        memory_id: MemoryId,
        current_tick: u64,
        submitter: UpdateSource,
    ) -> PendingUpdate {
        PendingUpdate::new(memory_id, current_tick, submitter)
    }

    /// Captures the pre-reopen stable state before a labile window opens.
    pub fn capture_pre_reopen_state(
        &self,
        memory_id: MemoryId,
        reopen_tick: u64,
        strength: f32,
        stability: f32,
        access_count: u32,
    ) -> PreReopenState {
        PreReopenState {
            memory_id,
            reopen_tick,
            strength_at_reopen: strength,
            stability_at_reopen: stability,
            access_count_at_reopen: access_count,
        }
    }

    /// Validates a pending update before acceptance.
    ///
    /// Rejects updates that are empty, submitted before the window opened,
    /// or have out-of-range emotional values.
    pub fn validate_pending_update(
        &self,
        update: &PendingUpdate,
        labile_state: &LabileState,
        current_tick: u64,
    ) -> UpdateValidationResult {
        if labile_state.is_expired(current_tick) {
            return UpdateValidationResult::Rejected(UpdateRejectionReason::WindowExpired);
        }

        if update.new_content.is_none()
            && update.new_emotional_arousal.is_none()
            && update.new_emotional_valence.is_none()
        {
            return UpdateValidationResult::Rejected(UpdateRejectionReason::EmptyUpdate);
        }

        if update.submitted_tick < labile_state.since_tick {
            return UpdateValidationResult::Rejected(UpdateRejectionReason::SubmittedBeforeWindow);
        }

        if let Some(arousal) = update.new_emotional_arousal {
            if !(0.0..=1.0).contains(&arousal) {
                return UpdateValidationResult::Rejected(UpdateRejectionReason::ArousalOutOfRange);
            }
        }

        if let Some(valence) = update.new_emotional_valence {
            if !(-1.0..=1.0).contains(&valence) {
                return UpdateValidationResult::Rejected(UpdateRejectionReason::ValenceOutOfRange);
            }
        }

        UpdateValidationResult::Accepted(update.clone())
    }

    pub fn is_window_expired(&self, state: &LabileState, current_tick: u64) -> bool {
        state.is_expired(current_tick)
    }

    pub fn remaining_window(&self, state: &LabileState, current_tick: u64) -> u64 {
        state.remaining_ticks(current_tick)
    }

    /// Applies the reconsolidation bonus, capped at max_strength.
    pub fn apply_update_to_strength(
        &self,
        current_strength: f32,
        bonus: f32,
        max_strength: f32,
    ) -> f32 {
        (current_strength + bonus).min(max_strength)
    }

    fn refresh_triggers_for_update(&self, update: &PendingUpdate) -> Vec<RefreshTrigger> {
        let mut triggers = Vec::new();
        if update.new_content.is_some() {
            triggers.push(RefreshTrigger::EmbeddingRefresh);
            triggers.push(RefreshTrigger::IndexRefresh);
        }
        if update.new_emotional_arousal.is_some() || update.new_emotional_valence.is_some() {
            triggers.push(RefreshTrigger::CacheInvalidate);
        }
        if !triggers.contains(&RefreshTrigger::CacheInvalidate) {
            triggers.push(RefreshTrigger::CacheInvalidate);
        }
        triggers
    }

    /// Core tick: evaluates a labile memory at the current tick and decides
    /// whether to apply, discard, wait, or defer until refresh hooks are ready.
    ///
    /// - If the window is still open and no pending update exists → `WindowStillOpen`
    /// - If the window is still open and a pending update exists and refresh is ready → `Applied`
    /// - If the window is still open and refresh is deferred → `DeferredRefresh`
    /// - If the window is still open and refresh failed → `BlockedRefreshFailure`
    /// - If the window expired and a pending update exists → `DiscardedStale`
    /// - If the window expired and no pending update exists → `NoPendingUpdate`
    pub fn tick(
        &self,
        labile_state: &LabileState,
        pending: Option<&PendingUpdate>,
        current_tick: u64,
        current_strength: f32,
        policy: &ReconsolidationPolicy,
        refresh_readiness: RefreshReadiness,
    ) -> ReconsolidationTickResult {
        let expired = labile_state.is_expired(current_tick);

        match (expired, pending) {
            (false, None) => ReconsolidationTickResult {
                outcome: ReconsolidationOutcome::WindowStillOpen,
                new_strength: None,
                refresh_triggers: Vec::new(),
                pending_update_cleared: false,
                restabilized: false,
            },
            (false, Some(update)) => {
                let triggers = self.refresh_triggers_for_update(update);
                match refresh_readiness {
                    RefreshReadiness::Ready => {
                        let new_strength = self.apply_update_to_strength(
                            current_strength,
                            policy.reconsolidation_bonus,
                            policy.max_strength,
                        );
                        ReconsolidationTickResult {
                            outcome: ReconsolidationOutcome::Applied,
                            new_strength: Some(new_strength),
                            refresh_triggers: triggers,
                            pending_update_cleared: true,
                            restabilized: true,
                        }
                    }
                    RefreshReadiness::Deferred => ReconsolidationTickResult {
                        outcome: ReconsolidationOutcome::DeferredRefresh,
                        new_strength: None,
                        refresh_triggers: triggers,
                        pending_update_cleared: false,
                        restabilized: false,
                    },
                    RefreshReadiness::Failed => ReconsolidationTickResult {
                        outcome: ReconsolidationOutcome::BlockedRefreshFailure,
                        new_strength: None,
                        refresh_triggers: triggers,
                        pending_update_cleared: false,
                        restabilized: false,
                    },
                }
            }
            (true, Some(_)) => ReconsolidationTickResult {
                outcome: ReconsolidationOutcome::DiscardedStale,
                new_strength: None,
                refresh_triggers: Vec::new(),
                pending_update_cleared: true,
                restabilized: true,
            },
            (true, None) => ReconsolidationTickResult {
                outcome: ReconsolidationOutcome::NoPendingUpdate,
                new_strength: None,
                refresh_triggers: Vec::new(),
                pending_update_cleared: false,
                restabilized: true,
            },
        }
    }

    /// Builds an audit record for one tick evaluation.
    pub fn audit_tick(
        &self,
        memory_id: MemoryId,
        tick: u64,
        result: &ReconsolidationTickResult,
        strength_before: Option<f32>,
        applied_source: Option<UpdateSource>,
    ) -> ReconsolidationAuditRecord {
        ReconsolidationAuditRecord {
            memory_id,
            tick,
            outcome: result.outcome,
            strength_before,
            strength_after: result.new_strength,
            refresh_triggers: result.refresh_triggers.clone(),
            applied_update_source: applied_source,
            pending_update_cleared: result.pending_update_cleared,
            restabilized: result.restabilized,
        }
    }
}

/// Result of a single reconsolidation tick evaluation.
#[derive(Debug, Clone, PartialEq)]
pub struct ReconsolidationTickResult {
    pub outcome: ReconsolidationOutcome,
    pub new_strength: Option<f32>,
    pub refresh_triggers: Vec<RefreshTrigger>,
    pub pending_update_cleared: bool,
    pub restabilized: bool,
}

impl Default for ReconsolidationEngine {
    fn default() -> Self {
        Self::new()
    }
}

// ── Maintenance operation ────────────────────────────────────────────────────

use crate::api::NamespaceId;
use crate::engine::maintenance::{
    DurableStateToken, InterruptedMaintenance, InterruptionReason, MaintenanceOperation,
    MaintenanceProgress, MaintenanceStep,
};

/// One memory currently in a labile state with an optional pending update.
#[derive(Debug, Clone, PartialEq)]
pub struct LabileMemory {
    pub memory_id: MemoryId,
    pub labile_state: LabileState,
    pub pending_update: Option<PendingUpdate>,
    pub current_strength: f32,
    pub pre_reopen_state: PreReopenState,
}

/// Operator-visible summary after a reconsolidation run completes.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReconsolidationRunSummary {
    pub memories_evaluated: u32,
    pub applied: u32,
    pub discarded_stale: u32,
    pub no_pending_update: u32,
    pub still_open: u32,
    pub deferred_refresh: u32,
    pub blocked_refresh_failure: u32,
    pub audit_records: Vec<ReconsolidationAuditRecordFlat>,
}

/// Flat audit record for the run summary (no heap allocations in Eq path).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReconsolidationAuditRecordFlat {
    pub memory_id: MemoryId,
    pub tick: u64,
    pub outcome: &'static str,
    pub strength_before: Option<u32>,
    pub strength_after: Option<u32>,
    pub refresh_triggers: Vec<&'static str>,
    pub applied_update_source: Option<&'static str>,
    pub pending_update_cleared: bool,
    pub restabilized: bool,
}

/// Bounded reconsolidation run that processes labile memories through tick evaluation.
#[derive(Debug, Clone, PartialEq)]
pub struct ReconsolidationRun {
    namespace: NamespaceId,
    policy: ReconsolidationPolicy,
    labile_memories: Vec<LabileMemory>,
    current_tick: u64,
    processed: usize,
    applied: u32,
    discarded_stale: u32,
    no_pending_update: u32,
    still_open: u32,
    deferred_refresh: u32,
    blocked_refresh_failure: u32,
    audit_records: Vec<ReconsolidationAuditRecord>,
    completed: bool,
    durable_state_token: DurableStateToken,
}

impl ReconsolidationRun {
    /// Creates a new reconsolidation run over the given labile memories.
    pub fn new(
        namespace: NamespaceId,
        policy: ReconsolidationPolicy,
        labile_memories: Vec<LabileMemory>,
        current_tick: u64,
    ) -> Self {
        Self {
            namespace,
            policy,
            labile_memories,
            current_tick,
            processed: 0,
            applied: 0,
            discarded_stale: 0,
            no_pending_update: 0,
            still_open: 0,
            deferred_refresh: 0,
            blocked_refresh_failure: 0,
            audit_records: Vec::new(),
            completed: false,
            durable_state_token: DurableStateToken(0),
        }
    }

    /// Returns the audit records collected so far.
    pub fn audit_records(&self) -> &[ReconsolidationAuditRecord] {
        &self.audit_records
    }
}

impl MaintenanceOperation for ReconsolidationRun {
    type Summary = ReconsolidationRunSummary;

    fn poll_step(&mut self) -> MaintenanceStep<Self::Summary> {
        if self.completed || self.processed >= self.labile_memories.len() {
            self.completed = true;
            return MaintenanceStep::Completed(self.summary());
        }

        let engine = ReconsolidationEngine::new();
        let mem = &self.labile_memories[self.processed];
        let result = engine.tick(
            &mem.labile_state,
            mem.pending_update.as_ref(),
            self.current_tick,
            mem.current_strength,
            &self.policy,
            RefreshReadiness::Ready,
        );

        match result.outcome {
            ReconsolidationOutcome::Applied => self.applied += 1,
            ReconsolidationOutcome::DiscardedStale => self.discarded_stale += 1,
            ReconsolidationOutcome::NoPendingUpdate => self.no_pending_update += 1,
            ReconsolidationOutcome::WindowStillOpen => self.still_open += 1,
            ReconsolidationOutcome::DeferredRefresh => self.deferred_refresh += 1,
            ReconsolidationOutcome::BlockedRefreshFailure => self.blocked_refresh_failure += 1,
        }

        let audit = engine.audit_tick(
            mem.memory_id,
            self.current_tick,
            &result,
            Some(mem.current_strength),
            mem.pending_update.as_ref().map(|p| p.submitter),
        );
        self.audit_records.push(audit);

        self.processed += 1;
        self.durable_state_token.0 = self.processed as u64;

        if self.processed >= self.labile_memories.len() {
            self.completed = true;
            MaintenanceStep::Completed(self.summary())
        } else {
            MaintenanceStep::Pending(MaintenanceProgress::new(
                self.processed as u32,
                self.labile_memories.len() as u32,
            ))
        }
    }

    fn interrupt(&mut self, reason: InterruptionReason) -> InterruptedMaintenance {
        InterruptedMaintenance {
            reason,
            preserved_durable_state: self.durable_state_token,
        }
    }
}

impl ReconsolidationRun {
    fn summary(&self) -> ReconsolidationRunSummary {
        let audit_flat = self
            .audit_records
            .iter()
            .map(|r| ReconsolidationAuditRecordFlat {
                memory_id: r.memory_id,
                tick: r.tick,
                outcome: r.outcome.as_str(),
                strength_before: r.strength_before.map(|s| (s * 1000.0) as u32),
                strength_after: r.strength_after.map(|s| (s * 1000.0) as u32),
                refresh_triggers: r.refresh_triggers.iter().map(|t| t.as_str()).collect(),
                applied_update_source: r.applied_update_source.map(|s| s.as_str()),
                pending_update_cleared: r.pending_update_cleared,
                restabilized: r.restabilized,
            })
            .collect();

        ReconsolidationRunSummary {
            memories_evaluated: self.processed as u32,
            applied: self.applied,
            discarded_stale: self.discarded_stale,
            no_pending_update: self.no_pending_update,
            still_open: self.still_open,
            deferred_refresh: self.deferred_refresh,
            blocked_refresh_failure: self.blocked_refresh_failure,
            audit_records: audit_flat,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::engine::maintenance::{
        MaintenanceController, MaintenanceJobHandle, MaintenanceJobState,
    };

    fn ns(s: &str) -> NamespaceId {
        NamespaceId::new(s).unwrap_or_else(|_err| {
            std::process::abort();
        })
    }

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
        let state = result.unwrap_or_else(|| {
            std::process::abort();
        });
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

    // ── Tick mechanism tests ──────────────────────────────────────────────

    #[test]
    fn tick_window_still_open_with_no_pending_update() {
        let engine = ReconsolidationEngine::new();
        let policy = policy();
        let state = LabileState::new(100, 50);

        let result = engine.tick(&state, None, 120, 0.5, &policy, RefreshReadiness::Ready);
        assert_eq!(result.outcome, ReconsolidationOutcome::WindowStillOpen);
        assert_eq!(result.new_strength, None);
        assert!(result.refresh_triggers.is_empty());
        assert!(!result.pending_update_cleared);
        assert!(!result.restabilized);
    }

    #[test]
    fn tick_applies_pending_update_within_window() {
        let engine = ReconsolidationEngine::new();
        let policy = policy();
        let state = LabileState::new(100, 50);
        let update = PendingUpdate::new(MemoryId(1), 105, UpdateSource::User)
            .with_content("revised content".to_string());

        let result = engine.tick(
            &state,
            Some(&update),
            120,
            0.5,
            &policy,
            RefreshReadiness::Ready,
        );
        assert_eq!(result.outcome, ReconsolidationOutcome::Applied);
        let new_strength = result.new_strength.unwrap_or_else(|| {
            std::process::abort();
        });
        assert!(new_strength > 0.5);
        // Content update triggers embedding, index, and cache refresh.
        assert!(result
            .refresh_triggers
            .contains(&RefreshTrigger::EmbeddingRefresh));
        assert!(result
            .refresh_triggers
            .contains(&RefreshTrigger::IndexRefresh));
        assert!(result
            .refresh_triggers
            .contains(&RefreshTrigger::CacheInvalidate));
        assert!(result.pending_update_cleared);
        assert!(result.restabilized);
    }

    #[test]
    fn tick_discards_stale_pending_update_after_window_expires() {
        let engine = ReconsolidationEngine::new();
        let policy = policy();
        let state = LabileState::new(100, 50);
        let update = PendingUpdate::new(MemoryId(1), 105, UpdateSource::Agent);

        // Tick at 150 — window just expired.
        let result = engine.tick(
            &state,
            Some(&update),
            150,
            0.5,
            &policy,
            RefreshReadiness::Ready,
        );
        assert_eq!(result.outcome, ReconsolidationOutcome::DiscardedStale);
        assert_eq!(result.new_strength, None);
        assert!(result.refresh_triggers.is_empty());
        assert!(result.pending_update_cleared);
        assert!(result.restabilized);
    }

    #[test]
    fn tick_no_pending_update_after_window_expires() {
        let engine = ReconsolidationEngine::new();
        let policy = policy();
        let state = LabileState::new(100, 50);

        let result = engine.tick(&state, None, 200, 0.5, &policy, RefreshReadiness::Ready);
        assert_eq!(result.outcome, ReconsolidationOutcome::NoPendingUpdate);
        assert_eq!(result.new_strength, None);
        assert!(!result.pending_update_cleared);
        assert!(result.restabilized);
    }

    #[test]
    fn tick_emotional_update_triggers_cache_invalidate() {
        let engine = ReconsolidationEngine::new();
        let policy = policy();
        let state = LabileState::new(100, 50);
        let update =
            PendingUpdate::new(MemoryId(1), 105, UpdateSource::System).with_emotional(0.9, 0.3);

        let result = engine.tick(
            &state,
            Some(&update),
            120,
            0.5,
            &policy,
            RefreshReadiness::Ready,
        );
        assert_eq!(result.outcome, ReconsolidationOutcome::Applied);
        assert!(result
            .refresh_triggers
            .contains(&RefreshTrigger::CacheInvalidate));
        assert!(result.pending_update_cleared);
        assert!(result.restabilized);
    }

    #[test]
    fn tick_strength_bonus_respects_max_cap() {
        let engine = ReconsolidationEngine::new();
        let policy = policy();
        let state = LabileState::new(100, 50);
        let update = PendingUpdate::new(MemoryId(1), 105, UpdateSource::User);

        let result = engine.tick(
            &state,
            Some(&update),
            120,
            0.98,
            &policy,
            RefreshReadiness::Ready,
        );
        assert_eq!(result.outcome, ReconsolidationOutcome::Applied);
        assert_eq!(result.new_strength, Some(1.0));
        assert!(result.pending_update_cleared);
        assert!(result.restabilized);
    }

    #[test]
    fn tick_defers_update_when_refresh_is_not_ready() {
        let engine = ReconsolidationEngine::new();
        let policy = policy();
        let state = LabileState::new(100, 50);
        let update = PendingUpdate::new(MemoryId(1), 105, UpdateSource::User)
            .with_content("revised content".to_string());

        let result = engine.tick(
            &state,
            Some(&update),
            120,
            0.5,
            &policy,
            RefreshReadiness::Deferred,
        );
        assert_eq!(result.outcome, ReconsolidationOutcome::DeferredRefresh);
        assert_eq!(result.new_strength, None);
        assert!(result
            .refresh_triggers
            .contains(&RefreshTrigger::EmbeddingRefresh));
        assert!(result
            .refresh_triggers
            .contains(&RefreshTrigger::IndexRefresh));
        assert!(result
            .refresh_triggers
            .contains(&RefreshTrigger::CacheInvalidate));
        assert!(!result.pending_update_cleared);
        assert!(!result.restabilized);
    }

    #[test]
    fn tick_blocks_mutation_when_refresh_has_failed() {
        let engine = ReconsolidationEngine::new();
        let policy = policy();
        let state = LabileState::new(100, 50);
        let update = PendingUpdate::new(MemoryId(1), 105, UpdateSource::Agent)
            .with_content("revised content".to_string());

        let result = engine.tick(
            &state,
            Some(&update),
            120,
            0.5,
            &policy,
            RefreshReadiness::Failed,
        );
        assert_eq!(
            result.outcome,
            ReconsolidationOutcome::BlockedRefreshFailure
        );
        assert_eq!(result.new_strength, None);
        assert!(result
            .refresh_triggers
            .contains(&RefreshTrigger::EmbeddingRefresh));
        assert!(result
            .refresh_triggers
            .contains(&RefreshTrigger::IndexRefresh));
        assert!(result
            .refresh_triggers
            .contains(&RefreshTrigger::CacheInvalidate));
        assert!(!result.pending_update_cleared);
        assert!(!result.restabilized);
    }

    #[test]
    fn audit_tick_records_correctly() {
        let engine = ReconsolidationEngine::new();
        let result = ReconsolidationTickResult {
            outcome: ReconsolidationOutcome::Applied,
            new_strength: Some(0.55),
            refresh_triggers: vec![RefreshTrigger::CacheInvalidate],
            pending_update_cleared: true,
            restabilized: true,
        };
        let audit = engine.audit_tick(
            MemoryId(42),
            120,
            &result,
            Some(0.5),
            Some(UpdateSource::User),
        );
        assert_eq!(audit.memory_id, MemoryId(42));
        assert_eq!(audit.tick, 120);
        assert_eq!(audit.outcome, ReconsolidationOutcome::Applied);
        assert_eq!(audit.strength_before, Some(0.5));
        assert_eq!(audit.strength_after, Some(0.55));
        assert_eq!(audit.applied_update_source, Some(UpdateSource::User));
        assert!(audit.pending_update_cleared);
        assert!(audit.restabilized);
    }

    #[test]
    fn reconsolidation_outcome_as_str() {
        assert_eq!(ReconsolidationOutcome::Applied.as_str(), "applied");
        assert_eq!(
            ReconsolidationOutcome::DiscardedStale.as_str(),
            "discarded_stale"
        );
        assert_eq!(
            ReconsolidationOutcome::NoPendingUpdate.as_str(),
            "no_pending_update"
        );
        assert_eq!(
            ReconsolidationOutcome::WindowStillOpen.as_str(),
            "window_still_open"
        );
        assert_eq!(
            ReconsolidationOutcome::DeferredRefresh.as_str(),
            "deferred_refresh"
        );
        assert_eq!(
            ReconsolidationOutcome::BlockedRefreshFailure.as_str(),
            "blocked_refresh_failure"
        );
    }

    #[test]
    fn refresh_trigger_as_str() {
        assert_eq!(
            RefreshTrigger::EmbeddingRefresh.as_str(),
            "embedding_refresh"
        );
        assert_eq!(RefreshTrigger::IndexRefresh.as_str(), "index_refresh");
        assert_eq!(RefreshTrigger::CacheInvalidate.as_str(), "cache_invalidate");
    }

    // ── ReconsolidationRun tests ──────────────────────────────────────────

    #[test]
    fn reconsolidation_run_completes_with_mixed_outcomes() {
        let memories = vec![
            LabileMemory {
                memory_id: MemoryId(1),
                labile_state: LabileState::new(100, 50),
                pending_update: Some(
                    PendingUpdate::new(MemoryId(1), 105, UpdateSource::User)
                        .with_content("updated".to_string()),
                ),
                current_strength: 0.5,
                pre_reopen_state: PreReopenState {
                    memory_id: MemoryId(1),
                    reopen_tick: 100,
                    strength_at_reopen: 0.45,
                    stability_at_reopen: 3.0,
                    access_count_at_reopen: 4,
                },
            },
            LabileMemory {
                memory_id: MemoryId(2),
                labile_state: LabileState::new(50, 10),
                pending_update: Some(PendingUpdate::new(MemoryId(2), 55, UpdateSource::Agent)),
                current_strength: 0.6,
                pre_reopen_state: PreReopenState {
                    memory_id: MemoryId(2),
                    reopen_tick: 50,
                    strength_at_reopen: 0.6,
                    stability_at_reopen: 2.0,
                    access_count_at_reopen: 3,
                },
            },
            LabileMemory {
                memory_id: MemoryId(3),
                labile_state: LabileState::new(180, 50), // open until tick 230
                pending_update: None,
                current_strength: 0.4,
                pre_reopen_state: PreReopenState {
                    memory_id: MemoryId(3),
                    reopen_tick: 180,
                    strength_at_reopen: 0.4,
                    stability_at_reopen: 1.5,
                    access_count_at_reopen: 2,
                },
            },
        ];

        let run = ReconsolidationRun::new(ns("test"), policy(), memories, 200);
        let mut handle = MaintenanceJobHandle::new(run, 10);
        handle.start();

        let summary = loop {
            let snap = handle.poll();
            match snap.state {
                MaintenanceJobState::Completed(s) => break s,
                MaintenanceJobState::Running { .. } => continue,
                _other => std::process::abort(),
            }
        };

        assert_eq!(summary.memories_evaluated, 3);
        // Memory 1: window (100..150) expired at 200, has update → DiscardedStale
        // Memory 2: window (50..60) expired at 200, has update → DiscardedStale
        // Memory 3: window (180..230) open at 200, no update → WindowStillOpen
        assert_eq!(summary.discarded_stale, 2);
        assert_eq!(summary.still_open, 1);
        assert_eq!(summary.applied, 0);
        assert_eq!(summary.deferred_refresh, 0);
        assert_eq!(summary.blocked_refresh_failure, 0);
        assert_eq!(summary.audit_records.len(), 3);

        // Verify audit for discarded memory.
        let discarded_audit = &summary.audit_records[0];
        assert_eq!(discarded_audit.memory_id, MemoryId(1));
        assert_eq!(discarded_audit.outcome, "discarded_stale");
        assert_eq!(discarded_audit.applied_update_source, Some("user"));
        assert!(discarded_audit.pending_update_cleared);
        assert!(discarded_audit.restabilized);

        // Verify audit for window-still-open memory.
        let open_audit = &summary.audit_records[2];
        assert_eq!(open_audit.memory_id, MemoryId(3));
        assert_eq!(open_audit.outcome, "window_still_open");
        assert!(!open_audit.pending_update_cleared);
        assert!(!open_audit.restabilized);
    }

    #[test]
    fn reconsolidation_run_can_be_cancelled() {
        let memories = vec![
            LabileMemory {
                memory_id: MemoryId(1),
                labile_state: LabileState::new(100, 50),
                pending_update: None,
                current_strength: 0.5,
                pre_reopen_state: PreReopenState {
                    memory_id: MemoryId(1),
                    reopen_tick: 100,
                    strength_at_reopen: 0.5,
                    stability_at_reopen: 2.0,
                    access_count_at_reopen: 1,
                },
            },
            LabileMemory {
                memory_id: MemoryId(2),
                labile_state: LabileState::new(100, 50),
                pending_update: None,
                current_strength: 0.5,
                pre_reopen_state: PreReopenState {
                    memory_id: MemoryId(2),
                    reopen_tick: 100,
                    strength_at_reopen: 0.5,
                    stability_at_reopen: 2.0,
                    access_count_at_reopen: 1,
                },
            },
        ];

        let run = ReconsolidationRun::new(ns("test"), policy(), memories, 120);
        let mut handle = MaintenanceJobHandle::new(run, 10);
        handle.start();
        let snap = handle.cancel();

        assert!(matches!(
            snap.state,
            MaintenanceJobState::CancelRequested { .. }
        ));
    }

    #[test]
    fn reconsolidation_run_empty_list_completes_immediately() {
        let run = ReconsolidationRun::new(ns("test"), policy(), vec![], 100);
        let mut handle = MaintenanceJobHandle::new(run, 10);
        handle.start();
        let snap = handle.poll();

        let MaintenanceJobState::Completed(summary) = snap.state else {
            std::process::abort();
        };
        assert_eq!(summary.memories_evaluated, 0);
        assert_eq!(summary.applied, 0);
        assert_eq!(summary.deferred_refresh, 0);
        assert_eq!(summary.blocked_refresh_failure, 0);
    }

    #[test]
    fn reconsolidation_run_discards_all_stale_updates() {
        let memories: Vec<LabileMemory> = (1..=5)
            .map(|i| LabileMemory {
                memory_id: MemoryId(i),
                labile_state: LabileState::new(10, 5), // window ends at tick 15
                pending_update: Some(PendingUpdate::new(MemoryId(i), 12, UpdateSource::System)),
                current_strength: 0.5,
                pre_reopen_state: PreReopenState {
                    memory_id: MemoryId(i),
                    reopen_tick: 10,
                    strength_at_reopen: 0.5,
                    stability_at_reopen: 2.0,
                    access_count_at_reopen: 1,
                },
            })
            .collect();

        let run = ReconsolidationRun::new(ns("test"), policy(), memories, 100);
        let mut handle = MaintenanceJobHandle::new(run, 10);
        handle.start();

        let summary = loop {
            let snap = handle.poll();
            match snap.state {
                MaintenanceJobState::Completed(s) => break s,
                MaintenanceJobState::Running { .. } => continue,
                _other => std::process::abort(),
            }
        };

        assert_eq!(summary.memories_evaluated, 5);
        assert_eq!(summary.discarded_stale, 5);
        assert_eq!(summary.applied, 0);
        assert_eq!(summary.deferred_refresh, 0);
        assert_eq!(summary.blocked_refresh_failure, 0);
        assert!(summary
            .audit_records
            .iter()
            .all(|r| r.outcome == "discarded_stale"));
    }

    #[test]
    fn reconsolidation_run_applies_all_within_window() {
        let memories: Vec<LabileMemory> = (1..=3)
            .map(|i| LabileMemory {
                memory_id: MemoryId(i),
                labile_state: LabileState::new(100, 50),
                pending_update: Some(
                    PendingUpdate::new(MemoryId(i), 105, UpdateSource::User)
                        .with_content(format!("update {}", i)),
                ),
                current_strength: 0.5,
                pre_reopen_state: PreReopenState {
                    memory_id: MemoryId(i),
                    reopen_tick: 10,
                    strength_at_reopen: 0.5,
                    stability_at_reopen: 2.0,
                    access_count_at_reopen: 1,
                },
            })
            .collect();

        let run = ReconsolidationRun::new(ns("test"), policy(), memories, 120);
        let mut handle = MaintenanceJobHandle::new(run, 10);
        handle.start();

        let summary = loop {
            let snap = handle.poll();
            match snap.state {
                MaintenanceJobState::Completed(s) => break s,
                MaintenanceJobState::Running { .. } => continue,
                _other => std::process::abort(),
            }
        };

        assert_eq!(summary.applied, 3);
        assert_eq!(summary.discarded_stale, 0);
        assert_eq!(summary.deferred_refresh, 0);
        assert_eq!(summary.blocked_refresh_failure, 0);
        assert!(summary.audit_records.iter().all(|r| r.outcome == "applied"));
        assert!(summary
            .audit_records
            .iter()
            .all(|r| r.refresh_triggers.contains(&"embedding_refresh")));
        assert!(summary
            .audit_records
            .iter()
            .all(|r| r.pending_update_cleared && r.restabilized));
    }

    #[test]
    fn run_summary_tracks_refresh_blockers_without_silent_mutation() {
        let engine = ReconsolidationEngine::new();
        let policy = policy();
        let state = LabileState::new(100, 50);
        let update = PendingUpdate::new(MemoryId(9), 105, UpdateSource::User)
            .with_content("updated".to_string());

        let deferred = engine.tick(
            &state,
            Some(&update),
            120,
            0.5,
            &policy,
            RefreshReadiness::Deferred,
        );
        assert_eq!(deferred.outcome, ReconsolidationOutcome::DeferredRefresh);
        assert_eq!(deferred.new_strength, None);
        assert!(!deferred.pending_update_cleared);
        assert!(!deferred.restabilized);

        let blocked = engine.tick(
            &state,
            Some(&update),
            120,
            0.5,
            &policy,
            RefreshReadiness::Failed,
        );
        assert_eq!(
            blocked.outcome,
            ReconsolidationOutcome::BlockedRefreshFailure
        );
        assert_eq!(blocked.new_strength, None);
        assert!(!blocked.pending_update_cleared);
        assert!(!blocked.restabilized);
    }

    // ── Pre-reopen state tests ───────────────────────────────────────────

    #[test]
    fn capture_pre_reopen_state_preserves_strength_and_stability() {
        let engine = ReconsolidationEngine::new();
        let state = engine.capture_pre_reopen_state(MemoryId(1), 100, 0.7, 5.0, 12);
        assert_eq!(state.memory_id, MemoryId(1));
        assert_eq!(state.reopen_tick, 100);
        assert_eq!(state.strength_at_reopen, 0.7);
        assert_eq!(state.stability_at_reopen, 5.0);
        assert_eq!(state.access_count_at_reopen, 12);
    }

    #[test]
    fn young_memory_gets_larger_window_than_old_memory() {
        let engine = ReconsolidationEngine::new();
        let policy = policy();
        let young_window = engine.reconsolidation_window(10, 0.5, &policy);
        let old_window = engine.reconsolidation_window(500, 0.5, &policy);
        assert!(young_window > old_window);
        assert!(old_window > 0);
    }

    #[test]
    fn strong_memory_gets_larger_window_than_weak_memory() {
        let engine = ReconsolidationEngine::new();
        let policy = policy();
        let strong_window = engine.reconsolidation_window(100, 0.9, &policy);
        let weak_window = engine.reconsolidation_window(100, 0.25, &policy);
        assert!(strong_window > weak_window);
    }

    // ── Update validation tests ──────────────────────────────────────────

    #[test]
    fn validate_accepts_content_update_within_window() {
        let engine = ReconsolidationEngine::new();
        let state = LabileState::new(100, 50);
        let update = PendingUpdate::new(MemoryId(1), 110, UpdateSource::User)
            .with_content("revised".to_string());

        let result = engine.validate_pending_update(&update, &state, 120);
        assert!(matches!(result, UpdateValidationResult::Accepted(_)));
    }

    #[test]
    fn validate_rejects_update_after_window_expired() {
        let engine = ReconsolidationEngine::new();
        let state = LabileState::new(100, 50);
        let update = PendingUpdate::new(MemoryId(1), 110, UpdateSource::User)
            .with_content("late".to_string());

        let result = engine.validate_pending_update(&update, &state, 200);
        assert_eq!(
            result,
            UpdateValidationResult::Rejected(UpdateRejectionReason::WindowExpired)
        );
    }

    #[test]
    fn validate_rejects_empty_update() {
        let engine = ReconsolidationEngine::new();
        let state = LabileState::new(100, 50);
        let update = PendingUpdate::new(MemoryId(1), 110, UpdateSource::System);

        let result = engine.validate_pending_update(&update, &state, 120);
        assert_eq!(
            result,
            UpdateValidationResult::Rejected(UpdateRejectionReason::EmptyUpdate)
        );
    }

    #[test]
    fn validate_rejects_update_submitted_before_window() {
        let engine = ReconsolidationEngine::new();
        let state = LabileState::new(100, 50);
        // Submitted at tick 50, which is before window start (100)
        let update = PendingUpdate::new(MemoryId(1), 50, UpdateSource::Agent)
            .with_content("too early".to_string());

        let result = engine.validate_pending_update(&update, &state, 120);
        assert_eq!(
            result,
            UpdateValidationResult::Rejected(UpdateRejectionReason::SubmittedBeforeWindow)
        );
    }

    #[test]
    fn validate_rejects_arousal_out_of_range() {
        let engine = ReconsolidationEngine::new();
        let state = LabileState::new(100, 50);
        let update =
            PendingUpdate::new(MemoryId(1), 110, UpdateSource::User).with_emotional(1.5, 0.0);

        let result = engine.validate_pending_update(&update, &state, 120);
        assert_eq!(
            result,
            UpdateValidationResult::Rejected(UpdateRejectionReason::ArousalOutOfRange)
        );
    }

    #[test]
    fn validate_rejects_valence_out_of_range() {
        let engine = ReconsolidationEngine::new();
        let state = LabileState::new(100, 50);
        let update =
            PendingUpdate::new(MemoryId(1), 110, UpdateSource::User).with_emotional(0.5, 2.0);

        let result = engine.validate_pending_update(&update, &state, 120);
        assert_eq!(
            result,
            UpdateValidationResult::Rejected(UpdateRejectionReason::ValenceOutOfRange)
        );
    }

    #[test]
    fn validate_accepts_emotional_update_within_range() {
        let engine = ReconsolidationEngine::new();
        let state = LabileState::new(100, 50);
        let update =
            PendingUpdate::new(MemoryId(1), 110, UpdateSource::User).with_emotional(0.8, -0.5);

        let result = engine.validate_pending_update(&update, &state, 120);
        assert!(matches!(result, UpdateValidationResult::Accepted(_)));
    }

    #[test]
    fn rejection_reason_as_str() {
        assert_eq!(
            UpdateRejectionReason::WindowExpired.as_str(),
            "window_expired"
        );
        assert_eq!(UpdateRejectionReason::EmptyUpdate.as_str(), "empty_update");
        assert_eq!(
            UpdateRejectionReason::SubmittedBeforeWindow.as_str(),
            "submitted_before_window"
        );
        assert_eq!(
            UpdateRejectionReason::ArousalOutOfRange.as_str(),
            "arousal_out_of_range"
        );
        assert_eq!(
            UpdateRejectionReason::ValenceOutOfRange.as_str(),
            "valence_out_of_range"
        );
    }
}
