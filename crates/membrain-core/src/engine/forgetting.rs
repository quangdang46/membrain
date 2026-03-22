//! Forgetting, demotion, archive, and restore surfaces.
//!
//! Owns the memory lifecycle past consolidation: demoting memories
//! between tiers, archiving cold memories, forgetting (soft-delete),
//! and restoring accidentally forgotten items.

use crate::api::NamespaceId;
use crate::engine::maintenance::{
    DurableStateToken, InterruptedMaintenance, InterruptionReason, MaintenanceOperation,
    MaintenanceProgress, MaintenanceStep,
};
use crate::types::MemoryId;

// ── Forgetting policy ────────────────────────────────────────────────────────

/// Policy controlling forgetting and demotion behavior.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ForgettingPolicy {
    /// Score below which memories are eligible for forgetting (0..1000).
    pub forget_score_threshold: u16,
    /// Score below which memories are demoted one tier (0..1000).
    pub demotion_score_threshold: u16,
    /// Maximum memories to process in one bounded run.
    pub batch_size: usize,
    /// Days without access before a memory is eligible for demotion.
    pub idle_days_before_demotion: u32,
    /// Whether to allow hard-delete or only soft-delete (archive).
    pub allow_hard_delete: bool,
}

impl Default for ForgettingPolicy {
    fn default() -> Self {
        Self {
            forget_score_threshold: 50,
            demotion_score_threshold: 200,
            batch_size: 100,
            idle_days_before_demotion: 90,
            allow_hard_delete: false,
        }
    }
}

// ── Forgetting actions ───────────────────────────────────────────────────────

/// Actions the forgetting engine can take on a memory.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ForgettingAction {
    /// Soft-delete: mark as forgotten but keep in cold storage for restore.
    SoftForget,
    /// Demote from hot → tier2, or tier2 → cold.
    Demote {
        from_tier: &'static str,
        to_tier: &'static str,
    },
    /// Archive to cold storage for long-term retention.
    Archive,
    /// Restore a previously forgotten memory.
    Restore,
    /// Skip — memory is still above threshold.
    Skip,
}

/// Decision record for one memory evaluated by the forgetting engine.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ForgettingDecision {
    /// Memory being evaluated.
    pub memory_id: MemoryId,
    /// Action taken.
    pub action: ForgettingAction,
    /// Current score that triggered the decision.
    pub current_score: u16,
    /// Machine-readable reason.
    pub reason: &'static str,
}

// ── Forgetting summary ──────────────────────────────────────────────────────

/// Operator-visible summary after a forgetting run.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ForgettingSummary {
    /// Total memories evaluated.
    pub evaluated: u32,
    /// Memories soft-forgotten.
    pub forgotten: u32,
    /// Memories demoted.
    pub demoted: u32,
    /// Memories archived.
    pub archived: u32,
    /// Memories skipped (still healthy).
    pub skipped: u32,
}

// ── Reversibility classification ─────────────────────────────────────────────

/// Whether a forgetting action can be undone and under what conditions.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Reversibility {
    /// Fully reversible: archive or restore can undo the action.
    Reversible,
    /// Soft-delete: reversible via explicit restore, but requires operator action.
    SoftReversible,
    /// Hard-delete: irreversible; evidence is gone.
    Irreversible,
    /// No action was taken; nothing to reverse.
    NoAction,
}

impl Reversibility {
    /// Returns the stable machine-readable label.
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Reversible => "reversible",
            Self::SoftReversible => "soft_reversible",
            Self::Irreversible => "irreversible",
            Self::NoAction => "no_action",
        }
    }
}

// ── Audit records ────────────────────────────────────────────────────────────

/// Structured audit record for a single forgetting decision.
///
/// Carries enough context for an operator to understand why a memory
/// was kept, demoted, archived, or forgotten without inspecting internals.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ForgettingAuditRecord {
    /// Memory that was evaluated.
    pub memory_id: MemoryId,
    /// Action taken on the memory.
    pub action: ForgettingAction,
    /// Effective score at evaluation time.
    pub current_score: u16,
    /// Forget threshold used for this run.
    pub forget_threshold: u16,
    /// Demotion threshold used for this run.
    pub demotion_threshold: u16,
    /// Machine-readable reason code.
    pub reason: &'static str,
    /// Whether the action is reversible and under what conditions.
    pub reversibility: Reversibility,
    /// Source tier before the action (for demotions).
    pub source_tier: Option<&'static str>,
    /// Target tier after the action (for demotions).
    pub target_tier: Option<&'static str>,
    /// Logical tick when the decision was recorded.
    pub tick: u64,
}

/// Operator-facing explain output for a single forgetting decision.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ForgettingExplainEntry {
    /// Memory that was evaluated.
    pub memory_id: MemoryId,
    /// Human-readable explanation of the decision.
    pub explanation: String,
    /// Action taken.
    pub action: ForgettingAction,
    /// Effective score at evaluation time.
    pub current_score: u16,
    /// Threshold that triggered the action.
    pub threshold_exceeded: &'static str,
    /// Whether the action is reversible.
    pub reversibility: Reversibility,
}

/// Operator-facing explain output for an entire forgetting run.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ForgettingExplainOutput {
    /// Namespace the run operated on.
    pub namespace: NamespaceId,
    /// Forget threshold used.
    pub forget_threshold: u16,
    /// Demotion threshold used.
    pub demotion_threshold: u16,
    /// Total candidates evaluated.
    pub evaluated: u32,
    /// Per-decision explain entries.
    pub entries: Vec<ForgettingExplainEntry>,
    /// Summary counts by reversibility class.
    pub reversible_count: u32,
    /// Count of soft-reversible actions.
    pub soft_reversible_count: u32,
    /// Count of irreversible actions.
    pub irreversible_count: u32,
    /// Count of no-action (skip) entries.
    pub no_action_count: u32,
}

/// Reversibility summary for operator review.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReversibilitySummary {
    /// Total decisions evaluated.
    pub total: u32,
    /// Fully reversible decisions.
    pub reversible: u32,
    /// Soft-reversible decisions (require operator action).
    pub soft_reversible: u32,
    /// Irreversible decisions (hard-delete).
    pub irreversible: u32,
    /// No-action decisions (skipped).
    pub no_action: u32,
}

impl ReversibilitySummary {
    /// Returns true if any irreversible actions were taken.
    pub const fn has_irreversible(&self) -> bool {
        self.irreversible > 0
    }

    /// Returns the fraction of decisions that are fully reversible.
    pub fn reversible_fraction(&self) -> f32 {
        if self.total == 0 {
            return 1.0;
        }
        self.reversible as f32 / self.total as f32
    }
}

// ── Forgetting operation ─────────────────────────────────────────────────────

/// Candidate memory evaluated by the forgetting run.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ForgettingCandidate {
    /// Memory to evaluate.
    pub memory_id: MemoryId,
    /// Current effective score (0..1000).
    pub current_score: u16,
}

/// Bounded forgetting operation for the maintenance controller.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ForgettingRun {
    namespace: NamespaceId,
    policy: ForgettingPolicy,
    candidates: Vec<ForgettingCandidate>,
    decisions: Vec<ForgettingDecision>,
    audit_records: Vec<ForgettingAuditRecord>,
    processed: u32,
    total: u32,
    forgotten: u32,
    demoted: u32,
    archived: u32,
    skipped: u32,
    completed: bool,
    durable_token: DurableStateToken,
    current_tick: u64,
}

impl ForgettingRun {
    /// Creates a new bounded forgetting run with explicit candidate scores.
    pub fn new(
        namespace: NamespaceId,
        policy: ForgettingPolicy,
        candidates: Vec<ForgettingCandidate>,
    ) -> Self {
        let total = candidates.len() as u32;
        Self {
            namespace,
            policy,
            candidates,
            decisions: Vec::new(),
            audit_records: Vec::new(),
            processed: 0,
            total,
            forgotten: 0,
            demoted: 0,
            archived: 0,
            skipped: 0,
            completed: false,
            durable_token: DurableStateToken(0),
            current_tick: 0,
        }
    }

    /// Sets the logical tick for audit records.
    pub fn with_tick(mut self, tick: u64) -> Self {
        self.current_tick = tick;
        self
    }

    /// Returns the recorded forgetting decisions.
    pub fn decisions(&self) -> &[ForgettingDecision] {
        &self.decisions
    }

    /// Returns the structured audit trail for this run.
    pub fn audit_records(&self) -> &[ForgettingAuditRecord] {
        &self.audit_records
    }

    /// Returns the namespace this run operated on.
    pub fn namespace(&self) -> &NamespaceId {
        &self.namespace
    }

    /// Builds an operator-facing explain output from the audit trail.
    pub fn explain(&self) -> ForgettingExplainOutput {
        let entries: Vec<ForgettingExplainEntry> = self
            .audit_records
            .iter()
            .map(|record| {
                let (explanation, threshold_exceeded) = match record.action {
                    ForgettingAction::SoftForget => (
                        format!(
                            "Memory {} scored {} which is below the forget threshold {} — \
                             marked for soft-delete (reversible via restore)",
                            record.memory_id.0,
                            record.current_score,
                            record.forget_threshold,
                        ),
                        "forget_threshold",
                    ),
                    ForgettingAction::Demote {
                        from_tier,
                        to_tier,
                    } => (
                        format!(
                            "Memory {} scored {} which is between forget threshold {} and \
                             demotion threshold {} — demoted from {} to {}",
                            record.memory_id.0,
                            record.current_score,
                            record.forget_threshold,
                            record.demotion_threshold,
                            from_tier,
                            to_tier,
                        ),
                        "demotion_threshold",
                    ),
                    ForgettingAction::Archive => (
                        format!(
                            "Memory {} scored {} — archived to cold storage for long-term retention",
                            record.memory_id.0, record.current_score,
                        ),
                        "archive_policy",
                    ),
                    ForgettingAction::Restore => (
                        format!(
                            "Memory {} was restored from a previous forgotten state",
                            record.memory_id.0,
                        ),
                        "restore_request",
                    ),
                    ForgettingAction::Skip => (
                        format!(
                            "Memory {} scored {} which is above demotion threshold {} — no action taken",
                            record.memory_id.0,
                            record.current_score,
                            record.demotion_threshold,
                        ),
                        "none",
                    ),
                };
                ForgettingExplainEntry {
                    memory_id: record.memory_id,
                    explanation,
                    action: record.action,
                    current_score: record.current_score,
                    threshold_exceeded,
                    reversibility: record.reversibility,
                }
            })
            .collect();

        let mut reversible_count = 0u32;
        let mut soft_reversible_count = 0u32;
        let mut irreversible_count = 0u32;
        let mut no_action_count = 0u32;
        for record in &self.audit_records {
            match record.reversibility {
                Reversibility::Reversible => reversible_count += 1,
                Reversibility::SoftReversible => soft_reversible_count += 1,
                Reversibility::Irreversible => irreversible_count += 1,
                Reversibility::NoAction => no_action_count += 1,
            }
        }

        ForgettingExplainOutput {
            namespace: self.namespace.clone(),
            forget_threshold: self.policy.forget_score_threshold,
            demotion_threshold: self.policy.demotion_score_threshold,
            evaluated: self.processed,
            entries,
            reversible_count,
            soft_reversible_count,
            irreversible_count,
            no_action_count,
        }
    }

    /// Builds a reversibility summary for operator review.
    pub fn reversibility_summary(&self) -> ReversibilitySummary {
        let mut reversible = 0u32;
        let mut soft_reversible = 0u32;
        let mut irreversible = 0u32;
        let mut no_action = 0u32;
        for record in &self.audit_records {
            match record.reversibility {
                Reversibility::Reversible => reversible += 1,
                Reversibility::SoftReversible => soft_reversible += 1,
                Reversibility::Irreversible => irreversible += 1,
                Reversibility::NoAction => no_action += 1,
            }
        }
        ReversibilitySummary {
            total: self.audit_records.len() as u32,
            reversible,
            soft_reversible,
            irreversible,
            no_action,
        }
    }
}

impl MaintenanceOperation for ForgettingRun {
    type Summary = ForgettingSummary;

    fn poll_step(&mut self) -> MaintenanceStep<Self::Summary> {
        if self.completed || self.processed >= self.total {
            self.completed = true;
            return MaintenanceStep::Completed(ForgettingSummary {
                evaluated: self.processed,
                forgotten: self.forgotten,
                demoted: self.demoted,
                archived: self.archived,
                skipped: self.skipped,
            });
        }

        let batch_size = (self.policy.batch_size as u32).max(1);
        let batch_end = (self.processed + batch_size).min(self.total);
        let start = self.processed as usize;
        let end = batch_end as usize;

        let engine = ForgettingEngine;
        for candidate in &self.candidates[start..end] {
            let action =
                engine.evaluate_memory(candidate.memory_id, candidate.current_score, &self.policy);
            let reason = match action {
                ForgettingAction::SoftForget => "below_forget_threshold",
                ForgettingAction::Demote { .. } => "below_demotion_threshold",
                ForgettingAction::Archive => "archive_eligible",
                ForgettingAction::Restore => "restored",
                ForgettingAction::Skip => "above_threshold",
            };
            let reversibility = match action {
                ForgettingAction::SoftForget => Reversibility::SoftReversible,
                ForgettingAction::Demote { .. } => Reversibility::Reversible,
                ForgettingAction::Archive => Reversibility::Reversible,
                ForgettingAction::Restore => Reversibility::Reversible,
                ForgettingAction::Skip => Reversibility::NoAction,
            };
            let (source_tier, target_tier) = match action {
                ForgettingAction::Demote { from_tier, to_tier } => (Some(from_tier), Some(to_tier)),
                _ => (None, None),
            };
            match action {
                ForgettingAction::SoftForget => self.forgotten += 1,
                ForgettingAction::Demote { .. } => self.demoted += 1,
                ForgettingAction::Archive => self.archived += 1,
                ForgettingAction::Skip | ForgettingAction::Restore => self.skipped += 1,
            }
            self.decisions.push(ForgettingDecision {
                memory_id: candidate.memory_id,
                action,
                current_score: candidate.current_score,
                reason,
            });
            self.audit_records.push(ForgettingAuditRecord {
                memory_id: candidate.memory_id,
                action,
                current_score: candidate.current_score,
                forget_threshold: self.policy.forget_score_threshold,
                demotion_threshold: self.policy.demotion_score_threshold,
                reason,
                reversibility,
                source_tier,
                target_tier,
                tick: self.current_tick,
            });
        }
        self.processed = batch_end;
        self.durable_token = DurableStateToken(self.processed as u64);

        if self.processed >= self.total {
            self.completed = true;
            MaintenanceStep::Completed(ForgettingSummary {
                evaluated: self.processed,
                forgotten: self.forgotten,
                demoted: self.demoted,
                archived: self.archived,
                skipped: self.skipped,
            })
        } else {
            MaintenanceStep::Pending(MaintenanceProgress::new(self.processed, self.total))
        }
    }

    fn interrupt(&mut self, reason: InterruptionReason) -> InterruptedMaintenance {
        InterruptedMaintenance {
            reason,
            preserved_durable_state: self.durable_token,
        }
    }
}

// ── Eligibility scoring ──────────────────────────────────────────────────────

/// Multi-factor input for eligibility scoring.
///
/// Each factor is normalized to 0.0..=1.0 so the scoring formula
/// stays deterministic and inspectable.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct EligibilityFactors {
    /// Effective strength normalized to 0.0..=1.0 (1.0 = strongest).
    pub effective_strength: f32,
    /// Recency: 1.0 = just accessed, 0.0 = never accessed or very old.
    pub recency: f32,
    /// Access frequency normalized (1.0 = very frequent, 0.0 = never accessed).
    pub access_frequency: f32,
    /// Number of ticks since last access (higher = more eligible for forgetting).
    pub ticks_since_access: u64,
    /// Whether this memory is in a contradiction neighborhood.
    pub in_contradiction: bool,
    /// Emotional arousal (higher = more resistant to forgetting).
    pub emotional_arousal: f32,
    /// Whether the memory bypasses decay (emotionally tagged).
    pub bypass_decay: bool,
}

impl EligibilityFactors {
    /// Builds default factors for a memory with the given strength.
    pub fn with_strength(strength: f32) -> Self {
        Self {
            effective_strength: strength,
            recency: 0.5,
            access_frequency: 0.5,
            ticks_since_access: 0,
            in_contradiction: false,
            emotional_arousal: 0.0,
            bypass_decay: false,
        }
    }
}

/// Computed eligibility score with component breakdown.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct EligibilityScore {
    /// Composite score 0..1000 (lower = more eligible for forgetting).
    pub composite_score: u16,
    /// Strength component contribution.
    pub strength_component: f32,
    /// Recency component contribution.
    pub recency_component: f32,
    /// Access frequency component contribution.
    pub access_component: f32,
    /// Contradiction penalty applied.
    pub contradiction_penalty: f32,
    /// Emotional resistance bonus applied.
    pub emotional_bonus: f32,
}

// ── Engine ───────────────────────────────────────────────────────────────────

/// Canonical forgetting engine owned by the core crate.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct ForgettingEngine;

impl ForgettingEngine {
    /// Returns the stable component identifier.
    pub const fn component_name(&self) -> &'static str {
        "engine.forgetting"
    }

    /// Creates a bounded forgetting run for the given namespace with placeholder candidates.
    pub fn create_run(
        &self,
        namespace: NamespaceId,
        policy: ForgettingPolicy,
        estimated_candidates: u32,
    ) -> ForgettingRun {
        let candidates: Vec<ForgettingCandidate> = (1..=estimated_candidates)
            .map(|i| ForgettingCandidate {
                memory_id: MemoryId(i as u64),
                current_score: 500, // Default mid-range score for placeholder candidates
            })
            .collect();
        ForgettingRun::new(namespace, policy, candidates)
    }

    /// Evaluates a single memory for forgetting/demotion.
    pub fn evaluate_memory(
        &self,
        _memory_id: MemoryId,
        current_score: u16,
        policy: &ForgettingPolicy,
    ) -> ForgettingAction {
        if current_score < policy.forget_score_threshold {
            ForgettingAction::SoftForget
        } else if current_score < policy.demotion_score_threshold {
            ForgettingAction::Demote {
                from_tier: "tier1",
                to_tier: "tier2",
            }
        } else {
            ForgettingAction::Skip
        }
    }

    /// Evaluates a single memory and returns its reversibility classification.
    pub fn evaluate_reversibility(
        &self,
        action: &ForgettingAction,
        policy: &ForgettingPolicy,
    ) -> Reversibility {
        match action {
            ForgettingAction::SoftForget => {
                if policy.allow_hard_delete {
                    Reversibility::Irreversible
                } else {
                    Reversibility::SoftReversible
                }
            }
            ForgettingAction::Demote { .. } => Reversibility::Reversible,
            ForgettingAction::Archive => Reversibility::Reversible,
            ForgettingAction::Restore => Reversibility::Reversible,
            ForgettingAction::Skip => Reversibility::NoAction,
        }
    }

    /// Builds an explain output from a completed forgetting run.
    pub fn explain_run(&self, run: &ForgettingRun) -> ForgettingExplainOutput {
        run.explain()
    }

    /// Builds a reversibility summary from a completed forgetting run.
    pub fn reversibility_summary(&self, run: &ForgettingRun) -> ReversibilitySummary {
        run.reversibility_summary()
    }

    /// Computes a multi-factor eligibility score for a memory.
    ///
    /// The composite score is 0..1000 where lower means more eligible for forgetting.
    /// Each component contributes proportionally: strength (50%), recency (25%),
    /// access frequency (25%), with contradiction and emotional modifiers.
    pub fn compute_eligibility_score(&self, factors: &EligibilityFactors) -> EligibilityScore {
        let strength_component = factors.effective_strength.clamp(0.0, 1.0);
        let recency_component = factors.recency.clamp(0.0, 1.0);
        let access_component = factors.access_frequency.clamp(0.0, 1.0);

        // Base composite: weighted blend of strength, recency, access
        let base = strength_component * 0.5 + recency_component * 0.25 + access_component * 0.25;

        // Contradiction memories are slightly more eligible for forgetting
        // (they add noise), but not aggressively — small penalty.
        let contradiction_penalty = if factors.in_contradiction { 0.1 } else { 0.0 };

        // Emotional memories resist forgetting.
        let emotional_bonus = if factors.bypass_decay {
            0.3
        } else {
            factors.emotional_arousal * 0.15
        };

        let adjusted = (base - contradiction_penalty + emotional_bonus).clamp(0.0, 1.0);
        let composite_score = (adjusted * 1000.0) as u16;

        EligibilityScore {
            composite_score,
            strength_component,
            recency_component,
            access_component,
            contradiction_penalty,
            emotional_bonus,
        }
    }

    /// Evaluates a memory using multi-factor eligibility scoring.
    ///
    /// Returns the action based on the composite eligibility score
    /// rather than a single raw score threshold.
    pub fn evaluate_memory_with_factors(
        &self,
        memory_id: MemoryId,
        factors: &EligibilityFactors,
        policy: &ForgettingPolicy,
    ) -> (ForgettingAction, EligibilityScore) {
        let score = self.compute_eligibility_score(factors);
        let action = self.evaluate_memory(memory_id, score.composite_score, policy);
        (action, score)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::engine::maintenance::{
        MaintenanceController, MaintenanceJobHandle, MaintenanceJobState,
    };

    fn ns(s: &str) -> NamespaceId {
        NamespaceId::new(s).unwrap()
    }

    #[test]
    fn forgetting_run_completes() {
        let engine = ForgettingEngine;
        let run = engine.create_run(ns("test"), ForgettingPolicy::default(), 30);
        let mut handle = MaintenanceJobHandle::new(run, 10);

        handle.start();
        let snap = handle.poll();

        assert!(matches!(snap.state, MaintenanceJobState::Completed(_)));
    }

    #[test]
    fn evaluate_memory_respects_thresholds() {
        let engine = ForgettingEngine;
        let policy = ForgettingPolicy::default();

        assert_eq!(
            engine.evaluate_memory(MemoryId(1), 30, &policy),
            ForgettingAction::SoftForget,
        );
        assert!(matches!(
            engine.evaluate_memory(MemoryId(2), 100, &policy),
            ForgettingAction::Demote { .. },
        ));
        assert_eq!(
            engine.evaluate_memory(MemoryId(3), 500, &policy),
            ForgettingAction::Skip,
        );
    }

    #[test]
    fn forgetting_run_makes_progress_when_batch_size_is_zero() {
        let engine = ForgettingEngine;
        let run = engine.create_run(
            ns("test"),
            ForgettingPolicy {
                batch_size: 0,
                ..Default::default()
            },
            2,
        );
        let mut handle = MaintenanceJobHandle::new(run, 3);

        let first = handle.poll();
        assert_eq!(
            first.state,
            MaintenanceJobState::Running {
                progress: Some(MaintenanceProgress::new(1, 2)),
            }
        );
        assert_eq!(first.polls_used, 1);

        let second = handle.poll();
        assert!(matches!(second.state, MaintenanceJobState::Completed(_)));
        assert_eq!(second.polls_used, 2);
    }

    // ── Audit record tests ────────────────────────────────────────────────

    #[test]
    fn audit_records_produced_for_all_decisions() {
        let candidates = vec![
            ForgettingCandidate {
                memory_id: MemoryId(1),
                current_score: 30,
            },
            ForgettingCandidate {
                memory_id: MemoryId(2),
                current_score: 100,
            },
            ForgettingCandidate {
                memory_id: MemoryId(3),
                current_score: 500,
            },
        ];
        let run = ForgettingRun::new(ns("audit"), ForgettingPolicy::default(), candidates);
        let mut handle = MaintenanceJobHandle::new(run, 5);

        handle.start();
        handle.poll();

        assert!(matches!(
            handle.snapshot().state,
            MaintenanceJobState::Completed(_)
        ));
        let audit = handle.operation().audit_records();
        assert_eq!(audit.len(), 3);
    }

    #[test]
    fn audit_records_carry_thresholds() {
        let candidates = vec![ForgettingCandidate {
            memory_id: MemoryId(1),
            current_score: 30,
        }];
        let policy = ForgettingPolicy {
            forget_score_threshold: 50,
            demotion_score_threshold: 200,
            ..Default::default()
        };
        let run = ForgettingRun::new(ns("thresh"), policy, candidates);
        let mut handle = MaintenanceJobHandle::new(run, 5);

        handle.start();
        handle.poll();

        let record = &handle.operation().audit_records()[0];
        assert_eq!(record.forget_threshold, 50);
        assert_eq!(record.demotion_threshold, 200);
        assert_eq!(record.current_score, 30);
        assert_eq!(record.reason, "below_forget_threshold");
    }

    #[test]
    fn audit_records_carry_reversibility_class() {
        let candidates = vec![
            ForgettingCandidate {
                memory_id: MemoryId(1),
                current_score: 30,
            },
            ForgettingCandidate {
                memory_id: MemoryId(2),
                current_score: 100,
            },
            ForgettingCandidate {
                memory_id: MemoryId(3),
                current_score: 500,
            },
        ];
        let run = ForgettingRun::new(ns("rev"), ForgettingPolicy::default(), candidates);
        let mut handle = MaintenanceJobHandle::new(run, 5);

        handle.start();
        handle.poll();

        let audit = handle.operation().audit_records();
        assert_eq!(audit[0].reversibility, Reversibility::SoftReversible);
        assert_eq!(audit[1].reversibility, Reversibility::Reversible);
        assert_eq!(audit[2].reversibility, Reversibility::NoAction);
    }

    #[test]
    fn audit_records_carry_tier_info_for_demotions() {
        let candidates = vec![ForgettingCandidate {
            memory_id: MemoryId(1),
            current_score: 100,
        }];
        let run = ForgettingRun::new(ns("tier"), ForgettingPolicy::default(), candidates);
        let mut handle = MaintenanceJobHandle::new(run, 5);

        handle.start();
        handle.poll();

        let record = &handle.operation().audit_records()[0];
        assert_eq!(record.source_tier, Some("tier1"));
        assert_eq!(record.target_tier, Some("tier2"));
    }

    #[test]
    fn audit_records_no_tier_info_for_skip() {
        let candidates = vec![ForgettingCandidate {
            memory_id: MemoryId(1),
            current_score: 500,
        }];
        let run = ForgettingRun::new(ns("notier"), ForgettingPolicy::default(), candidates);
        let mut handle = MaintenanceJobHandle::new(run, 5);

        handle.start();
        handle.poll();

        let record = &handle.operation().audit_records()[0];
        assert_eq!(record.source_tier, None);
        assert_eq!(record.target_tier, None);
    }

    #[test]
    fn audit_records_carry_tick() {
        let candidates = vec![ForgettingCandidate {
            memory_id: MemoryId(1),
            current_score: 30,
        }];
        let run =
            ForgettingRun::new(ns("tick"), ForgettingPolicy::default(), candidates).with_tick(42);
        let mut handle = MaintenanceJobHandle::new(run, 5);

        handle.start();
        handle.poll();

        let record = &handle.operation().audit_records()[0];
        assert_eq!(record.tick, 42);
    }

    // ── Explain output tests ──────────────────────────────────────────────

    #[test]
    fn explain_output_covers_all_entries() {
        let candidates = vec![
            ForgettingCandidate {
                memory_id: MemoryId(1),
                current_score: 30,
            },
            ForgettingCandidate {
                memory_id: MemoryId(2),
                current_score: 100,
            },
            ForgettingCandidate {
                memory_id: MemoryId(3),
                current_score: 500,
            },
        ];
        let run = ForgettingRun::new(ns("explain"), ForgettingPolicy::default(), candidates);
        let mut handle = MaintenanceJobHandle::new(run, 5);

        handle.start();
        handle.poll();

        let explain = handle.operation().explain();
        assert_eq!(explain.entries.len(), 3);
        assert_eq!(explain.namespace, ns("explain"));
        assert_eq!(explain.evaluated, 3);
    }

    #[test]
    fn explain_entries_have_threshold_references() {
        let candidates = vec![
            ForgettingCandidate {
                memory_id: MemoryId(1),
                current_score: 30,
            },
            ForgettingCandidate {
                memory_id: MemoryId(2),
                current_score: 100,
            },
            ForgettingCandidate {
                memory_id: MemoryId(3),
                current_score: 500,
            },
        ];
        let run = ForgettingRun::new(ns("thresh_ex"), ForgettingPolicy::default(), candidates);
        let mut handle = MaintenanceJobHandle::new(run, 5);

        handle.start();
        handle.poll();

        let explain = handle.operation().explain();
        assert_eq!(explain.entries[0].threshold_exceeded, "forget_threshold");
        assert_eq!(explain.entries[1].threshold_exceeded, "demotion_threshold");
        assert_eq!(explain.entries[2].threshold_exceeded, "none");
    }

    #[test]
    fn explain_entries_have_reversibility() {
        let candidates = vec![
            ForgettingCandidate {
                memory_id: MemoryId(1),
                current_score: 30,
            },
            ForgettingCandidate {
                memory_id: MemoryId(3),
                current_score: 500,
            },
        ];
        let run = ForgettingRun::new(ns("rev_ex"), ForgettingPolicy::default(), candidates);
        let mut handle = MaintenanceJobHandle::new(run, 5);

        handle.start();
        handle.poll();

        let explain = handle.operation().explain();
        assert_eq!(
            explain.entries[0].reversibility,
            Reversibility::SoftReversible
        );
        assert_eq!(explain.entries[1].reversibility, Reversibility::NoAction);
    }

    #[test]
    fn explain_entries_contain_human_readable_text() {
        let candidates = vec![ForgettingCandidate {
            memory_id: MemoryId(1),
            current_score: 30,
        }];
        let run = ForgettingRun::new(ns("readable"), ForgettingPolicy::default(), candidates);
        let mut handle = MaintenanceJobHandle::new(run, 5);

        handle.start();
        handle.poll();

        let explain = handle.operation().explain();
        assert!(explain.entries[0].explanation.contains("Memory 1"));
        assert!(explain.entries[0].explanation.contains("30"));
        assert!(explain.entries[0].explanation.contains("forget threshold"));
    }

    #[test]
    fn explain_reversibility_counts_match_decisions() {
        let candidates = vec![
            ForgettingCandidate {
                memory_id: MemoryId(1),
                current_score: 30,
            },
            ForgettingCandidate {
                memory_id: MemoryId(2),
                current_score: 100,
            },
            ForgettingCandidate {
                memory_id: MemoryId(3),
                current_score: 500,
            },
        ];
        let run = ForgettingRun::new(ns("counts"), ForgettingPolicy::default(), candidates);
        let mut handle = MaintenanceJobHandle::new(run, 5);

        handle.start();
        handle.poll();

        let explain = handle.operation().explain();
        assert_eq!(explain.soft_reversible_count, 1);
        assert_eq!(explain.reversible_count, 1);
        assert_eq!(explain.no_action_count, 1);
        assert_eq!(explain.irreversible_count, 0);
    }

    // ── Reversibility summary tests ───────────────────────────────────────

    #[test]
    fn reversibility_summary_counts() {
        let candidates = vec![
            ForgettingCandidate {
                memory_id: MemoryId(1),
                current_score: 30,
            },
            ForgettingCandidate {
                memory_id: MemoryId(2),
                current_score: 100,
            },
            ForgettingCandidate {
                memory_id: MemoryId(3),
                current_score: 500,
            },
        ];
        let run = ForgettingRun::new(ns("sum"), ForgettingPolicy::default(), candidates);
        let mut handle = MaintenanceJobHandle::new(run, 5);

        handle.start();
        handle.poll();

        let summary = handle.operation().reversibility_summary();
        assert_eq!(summary.total, 3);
        assert_eq!(summary.reversible, 1);
        assert_eq!(summary.soft_reversible, 1);
        assert_eq!(summary.irreversible, 0);
        assert_eq!(summary.no_action, 1);
        assert!(!summary.has_irreversible());
    }

    #[test]
    fn reversibility_summary_fraction() {
        let candidates = vec![
            ForgettingCandidate {
                memory_id: MemoryId(1),
                current_score: 100,
            },
            ForgettingCandidate {
                memory_id: MemoryId(2),
                current_score: 100,
            },
            ForgettingCandidate {
                memory_id: MemoryId(3),
                current_score: 500,
            },
        ];
        let run = ForgettingRun::new(ns("frac"), ForgettingPolicy::default(), candidates);
        let mut handle = MaintenanceJobHandle::new(run, 5);

        handle.start();
        handle.poll();

        let summary = handle.operation().reversibility_summary();
        let fraction = summary.reversible_fraction();
        assert!((fraction - 2.0 / 3.0).abs() < 1e-6);
    }

    #[test]
    fn reversibility_summary_empty_run() {
        let run = ForgettingRun::new(ns("empty"), ForgettingPolicy::default(), vec![]);
        let summary = run.reversibility_summary();
        assert_eq!(summary.total, 0);
        assert_eq!(summary.reversible_fraction(), 1.0);
        assert!(!summary.has_irreversible());
    }

    // ── Engine-level explain/reversibility tests ──────────────────────────

    #[test]
    fn engine_explain_run_matches_run_explain() {
        let engine = ForgettingEngine;
        let candidates = vec![ForgettingCandidate {
            memory_id: MemoryId(1),
            current_score: 30,
        }];
        let run = ForgettingRun::new(ns("eng"), ForgettingPolicy::default(), candidates);
        let mut handle = MaintenanceJobHandle::new(run, 5);

        handle.start();
        handle.poll();

        let from_run = handle.operation().explain();
        let from_engine = engine.explain_run(handle.operation());
        assert_eq!(from_run, from_engine);
    }

    #[test]
    fn engine_reversibility_summary_matches_run() {
        let engine = ForgettingEngine;
        let candidates = vec![ForgettingCandidate {
            memory_id: MemoryId(1),
            current_score: 30,
        }];
        let run = ForgettingRun::new(ns("eng_rev"), ForgettingPolicy::default(), candidates);
        let mut handle = MaintenanceJobHandle::new(run, 5);

        handle.start();
        handle.poll();

        let from_run = handle.operation().reversibility_summary();
        let from_engine = engine.reversibility_summary(handle.operation());
        assert_eq!(from_run, from_engine);
    }

    #[test]
    fn evaluate_reversibility_hard_delete_policy() {
        let engine = ForgettingEngine;
        let soft_policy = ForgettingPolicy::default();
        let hard_policy = ForgettingPolicy {
            allow_hard_delete: true,
            ..Default::default()
        };

        assert_eq!(
            engine.evaluate_reversibility(&ForgettingAction::SoftForget, &soft_policy),
            Reversibility::SoftReversible,
        );
        assert_eq!(
            engine.evaluate_reversibility(&ForgettingAction::SoftForget, &hard_policy),
            Reversibility::Irreversible,
        );
    }

    #[test]
    fn evaluate_reversibility_always_reversible_for_demote_and_archive() {
        let engine = ForgettingEngine;
        let policy = ForgettingPolicy::default();

        assert_eq!(
            engine.evaluate_reversibility(
                &ForgettingAction::Demote {
                    from_tier: "tier1",
                    to_tier: "tier2"
                },
                &policy
            ),
            Reversibility::Reversible,
        );
        assert_eq!(
            engine.evaluate_reversibility(&ForgettingAction::Archive, &policy),
            Reversibility::Reversible,
        );
        assert_eq!(
            engine.evaluate_reversibility(&ForgettingAction::Skip, &policy),
            Reversibility::NoAction,
        );
    }

    // ── Multi-batch audit continuity ──────────────────────────────────────

    #[test]
    fn audit_records_span_multiple_batches() {
        let candidates: Vec<ForgettingCandidate> = (1..=5)
            .map(|i| ForgettingCandidate {
                memory_id: MemoryId(i),
                current_score: (i as u16) * 50,
            })
            .collect();
        let policy = ForgettingPolicy {
            batch_size: 2,
            ..Default::default()
        };
        let run = ForgettingRun::new(ns("multi"), policy, candidates);
        let mut handle = MaintenanceJobHandle::new(run, 10);

        handle.start();
        loop {
            let snap = handle.poll();
            if matches!(snap.state, MaintenanceJobState::Completed(_)) {
                break;
            }
        }

        let audit = handle.operation().audit_records();
        assert_eq!(audit.len(), 5);
        let ids: Vec<u64> = audit.iter().map(|r| r.memory_id.0).collect();
        assert_eq!(ids, vec![1, 2, 3, 4, 5]);
    }

    // ── Reversibility enum tests ──────────────────────────────────────────

    #[test]
    fn reversibility_as_str() {
        assert_eq!(Reversibility::Reversible.as_str(), "reversible");
        assert_eq!(Reversibility::SoftReversible.as_str(), "soft_reversible");
        assert_eq!(Reversibility::Irreversible.as_str(), "irreversible");
        assert_eq!(Reversibility::NoAction.as_str(), "no_action");
    }

    // ── Deterministic behavior: no wall-clock dependency ─────────────────

    #[test]
    fn audit_records_are_deterministic_with_same_tick() {
        let candidates = vec![
            ForgettingCandidate {
                memory_id: MemoryId(1),
                current_score: 30,
            },
            ForgettingCandidate {
                memory_id: MemoryId(2),
                current_score: 100,
            },
        ];

        let run1 = ForgettingRun::new(ns("det"), ForgettingPolicy::default(), candidates.clone())
            .with_tick(42);
        let mut handle1 = MaintenanceJobHandle::new(run1, 5);
        handle1.start();
        handle1.poll();

        let run2 =
            ForgettingRun::new(ns("det"), ForgettingPolicy::default(), candidates).with_tick(42);
        let mut handle2 = MaintenanceJobHandle::new(run2, 5);
        handle2.start();
        handle2.poll();

        assert_eq!(
            handle1.operation().audit_records(),
            handle2.operation().audit_records()
        );
        assert_eq!(handle1.operation().explain(), handle2.operation().explain());
    }

    // ── Eligibility scoring tests ─────────────────────────────────────────

    #[test]
    fn eligibility_score_high_strength_means_less_eligible() {
        let engine = ForgettingEngine;

        let weak = EligibilityFactors::with_strength(0.1);
        let strong = EligibilityFactors::with_strength(0.9);

        let score_weak = engine.compute_eligibility_score(&weak);
        let score_strong = engine.compute_eligibility_score(&strong);

        // Higher strength → higher composite score → less eligible for forgetting
        assert!(score_strong.composite_score > score_weak.composite_score);
    }

    #[test]
    fn eligibility_score_contradiction_reduces_score() {
        let engine = ForgettingEngine;

        let normal = EligibilityFactors {
            effective_strength: 0.5,
            in_contradiction: false,
            ..EligibilityFactors::with_strength(0.5)
        };
        let contradicted = EligibilityFactors {
            effective_strength: 0.5,
            in_contradiction: true,
            ..EligibilityFactors::with_strength(0.5)
        };

        let score_normal = engine.compute_eligibility_score(&normal);
        let score_contra = engine.compute_eligibility_score(&contradicted);

        // Contradiction penalty makes the memory more eligible for forgetting
        assert!(score_contra.composite_score < score_normal.composite_score);
    }

    #[test]
    fn eligibility_score_emotional_bypass_resists_forgetting() {
        let engine = ForgettingEngine;

        let neutral = EligibilityFactors::with_strength(0.5);
        let emotional = EligibilityFactors {
            effective_strength: 0.5,
            bypass_decay: true,
            emotional_arousal: 0.9,
            ..EligibilityFactors::with_strength(0.5)
        };

        let score_neutral = engine.compute_eligibility_score(&neutral);
        let score_emotional = engine.compute_eligibility_score(&emotional);

        // Emotional bypass makes memory more resistant to forgetting
        assert!(score_emotional.composite_score > score_neutral.composite_score);
    }

    #[test]
    fn eligibility_score_high_recency_means_less_eligible() {
        let engine = ForgettingEngine;

        let old = EligibilityFactors {
            effective_strength: 0.5,
            recency: 0.1,
            ..EligibilityFactors::with_strength(0.5)
        };
        let recent = EligibilityFactors {
            effective_strength: 0.5,
            recency: 0.9,
            ..EligibilityFactors::with_strength(0.5)
        };

        let score_old = engine.compute_eligibility_score(&old);
        let score_recent = engine.compute_eligibility_score(&recent);

        assert!(score_recent.composite_score > score_old.composite_score);
    }

    #[test]
    fn eligibility_score_components_sum_reasonably() {
        let engine = ForgettingEngine;
        let factors = EligibilityFactors {
            effective_strength: 0.6,
            recency: 0.8,
            access_frequency: 0.4,
            in_contradiction: false,
            emotional_arousal: 0.0,
            bypass_decay: false,
            ticks_since_access: 0,
        };

        let score = engine.compute_eligibility_score(&factors);

        // Verify components match input
        assert!((score.strength_component - 0.6).abs() < 1e-6);
        assert!((score.recency_component - 0.8).abs() < 1e-6);
        assert!((score.access_component - 0.4).abs() < 1e-6);
        assert_eq!(score.contradiction_penalty, 0.0);
        assert_eq!(score.emotional_bonus, 0.0);

        // Composite should be in valid range
        assert!(score.composite_score <= 1000);
    }

    #[test]
    fn evaluate_memory_with_factors_returns_score_and_action() {
        let engine = ForgettingEngine;
        let policy = ForgettingPolicy::default();

        // Very weak memory: strength=0.0, recency=0.0, access=0.0
        // Composite: 0.0*0.5 + 0.0*0.25 + 0.0*0.25 = 0.0 → score 0
        let weak_factors = EligibilityFactors {
            effective_strength: 0.0,
            recency: 0.0,
            access_frequency: 0.0,
            ..EligibilityFactors::with_strength(0.0)
        };

        let (action, score) =
            engine.evaluate_memory_with_factors(MemoryId(1), &weak_factors, &policy);

        // Weak memory should be forgotten (score 0 < forget threshold 50)
        assert_eq!(action, ForgettingAction::SoftForget);
        assert!(score.composite_score < policy.forget_score_threshold);
    }

    #[test]
    fn eligibility_score_combined_contradiction_and_emotion() {
        let engine = ForgettingEngine;

        // Memory with both contradiction and emotional arousal
        let conflicted_emotional = EligibilityFactors {
            effective_strength: 0.5,
            in_contradiction: true,
            bypass_decay: true,
            emotional_arousal: 0.9,
            ..EligibilityFactors::with_strength(0.5)
        };

        let plain = EligibilityFactors::with_strength(0.5);

        let score_conflicted = engine.compute_eligibility_score(&conflicted_emotional);
        let score_plain = engine.compute_eligibility_score(&plain);

        // Emotional bonus (0.3) should outweigh contradiction penalty (0.1)
        assert!(score_conflicted.composite_score > score_plain.composite_score);
    }

    #[test]
    fn eligibility_score_clamps_to_valid_range() {
        let engine = ForgettingEngine;

        // Extreme values should still produce valid scores
        let max_factors = EligibilityFactors {
            effective_strength: 1.0,
            recency: 1.0,
            access_frequency: 1.0,
            bypass_decay: true,
            emotional_arousal: 1.0,
            ..EligibilityFactors::with_strength(1.0)
        };
        let min_factors = EligibilityFactors {
            effective_strength: 0.0,
            recency: 0.0,
            access_frequency: 0.0,
            in_contradiction: true,
            ..EligibilityFactors::with_strength(0.0)
        };

        let score_max = engine.compute_eligibility_score(&max_factors);
        let score_min = engine.compute_eligibility_score(&min_factors);

        assert!(score_max.composite_score <= 1000);
        assert!(score_min.composite_score <= 1000);
    }
}
