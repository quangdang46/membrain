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
    Demote { from_tier: &'static str, to_tier: &'static str },
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

// ── Forgetting operation ─────────────────────────────────────────────────────

/// Bounded forgetting operation for the maintenance controller.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ForgettingRun {
    namespace: NamespaceId,
    policy: ForgettingPolicy,
    processed: u32,
    total: u32,
    forgotten: u32,
    demoted: u32,
    archived: u32,
    skipped: u32,
    completed: bool,
    durable_token: DurableStateToken,
}

impl ForgettingRun {
    /// Creates a new bounded forgetting run.
    pub fn new(namespace: NamespaceId, policy: ForgettingPolicy, total_candidates: u32) -> Self {
        Self {
            namespace,
            policy,
            processed: 0,
            total: total_candidates,
            forgotten: 0,
            demoted: 0,
            archived: 0,
            skipped: 0,
            completed: false,
            durable_token: DurableStateToken(0),
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

        let batch_end = (self.processed + self.policy.batch_size as u32).min(self.total);
        let batch_count = batch_end - self.processed;

        // In a real implementation, each memory would be scored and evaluated
        self.skipped += batch_count;
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

// ── Engine ───────────────────────────────────────────────────────────────────

/// Canonical forgetting engine owned by the core crate.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct ForgettingEngine;

impl ForgettingEngine {
    /// Returns the stable component identifier.
    pub const fn component_name(&self) -> &'static str {
        "engine.forgetting"
    }

    /// Creates a bounded forgetting run for the given namespace.
    pub fn create_run(
        &self,
        namespace: NamespaceId,
        policy: ForgettingPolicy,
        estimated_candidates: u32,
    ) -> ForgettingRun {
        ForgettingRun::new(namespace, policy, estimated_candidates)
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

        // Below forget threshold
        assert_eq!(
            engine.evaluate_memory(MemoryId(1), 30, &policy),
            ForgettingAction::SoftForget,
        );

        // Between forget and demotion thresholds
        assert!(matches!(
            engine.evaluate_memory(MemoryId(2), 100, &policy),
            ForgettingAction::Demote { .. },
        ));

        // Above demotion threshold — skip
        assert_eq!(
            engine.evaluate_memory(MemoryId(3), 500, &policy),
            ForgettingAction::Skip,
        );
    }
}
