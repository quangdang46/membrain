//! Consolidation pipeline surfaces.
//!
//! Owns the merge/compaction logic that combines related memories,
//! deduplicates similar entries, and produces consolidated summaries.

use crate::api::NamespaceId;
use crate::engine::episode::EpisodeGroupingReport;
use crate::engine::maintenance::{
    DurableStateToken, InterruptedMaintenance, InterruptionReason, MaintenanceOperation,
    MaintenanceProgress, MaintenanceStep,
};
use crate::types::MemoryId;

// ── Consolidation policy ─────────────────────────────────────────────────────

/// Policy controlling when and how consolidation runs.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ConsolidationPolicy {
    /// Minimum number of memories before consolidation is eligible.
    pub minimum_candidates: usize,
    /// Maximum memories to process in one bounded run.
    pub batch_size: usize,
    /// Similarity threshold (0..1000) above which memories are merged.
    pub similarity_threshold: u16,
    /// Whether to merge duplicate-fingerprint memories automatically.
    pub auto_merge_duplicates: bool,
}

impl Default for ConsolidationPolicy {
    fn default() -> Self {
        Self {
            minimum_candidates: 10,
            batch_size: 50,
            similarity_threshold: 800,
            auto_merge_duplicates: true,
        }
    }
}

// ── Consolidation action ─────────────────────────────────────────────────────

/// Actions the consolidation engine can take on a memory group.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ConsolidationAction {
    /// Preserve a deterministic episode/source-set grouping before higher-order derivation work.
    EpisodeSourceSet { grouping: EpisodeGroupingReport },
    /// Merge multiple memories into one consolidated summary.
    Merge {
        source_ids: Vec<MemoryId>,
        summary_text: String,
    },
    /// Deduplicate by keeping one and marking others as superseded.
    Deduplicate {
        keep: MemoryId,
        remove: Vec<MemoryId>,
    },
    /// Skip — memories are too dissimilar to consolidate.
    Skip { reason: &'static str },
}

// ── Consolidation summary ────────────────────────────────────────────────────

/// Operator-visible summary after a consolidation run completes.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ConsolidationSummary {
    /// Number of memory groups evaluated.
    pub groups_evaluated: u32,
    /// Number of deterministic episode/source-set groupings produced.
    pub episode_source_sets: u32,
    /// Number of merges performed.
    pub merges_performed: u32,
    /// Number of deduplication actions taken.
    pub deduplications: u32,
    /// Number of groups skipped.
    pub skipped: u32,
    /// Total memories affected.
    pub memories_affected: u32,
    /// Sample grouping logs that explain why a source set was formed.
    pub grouping_logs: Vec<String>,
}

// ── Consolidation operation ──────────────────────────────────────────────────

/// Bounded consolidation operation that can be polled by the maintenance controller.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ConsolidationRun {
    namespace: NamespaceId,
    policy: ConsolidationPolicy,
    processed: u32,
    total: u32,
    episode_source_sets: u32,
    merges: u32,
    deduplications: u32,
    skipped: u32,
    completed: bool,
    durable_token: DurableStateToken,
    grouping_logs: Vec<String>,
}

impl ConsolidationRun {
    /// Creates a new consolidation run for a namespace.
    pub fn new(namespace: NamespaceId, policy: ConsolidationPolicy, total_groups: u32) -> Self {
        Self {
            namespace,
            policy,
            processed: 0,
            total: total_groups,
            episode_source_sets: 0,
            merges: 0,
            deduplications: 0,
            skipped: 0,
            completed: false,
            durable_token: DurableStateToken(0),
            grouping_logs: Vec::new(),
        }
    }
}

impl MaintenanceOperation for ConsolidationRun {
    type Summary = ConsolidationSummary;

    fn poll_step(&mut self) -> MaintenanceStep<Self::Summary> {
        if self.completed || self.processed >= self.total {
            self.completed = true;
            return MaintenanceStep::Completed(ConsolidationSummary {
                groups_evaluated: self.processed,
                episode_source_sets: self.episode_source_sets,
                merges_performed: self.merges,
                deduplications: self.deduplications,
                skipped: self.skipped,
                memories_affected: self.merges * 2 + self.deduplications,
                grouping_logs: self.grouping_logs.clone(),
            });
        }

        // Simulate processing one batch.
        let batch_size = (self.policy.batch_size as u32).max(1);
        let batch_end = (self.processed + batch_size).min(self.total);
        let batch_count = batch_end - self.processed;

        // Stage 1 always materializes deterministic episode/source-set grouping before
        // later merge or deduplication work. Until higher-order derivations land, treat
        // each bounded candidate group as grouped-and-skipped while preserving inspectable logs.
        self.episode_source_sets += batch_count;
        self.skipped += batch_count;
        let start_group = self.processed + 1;
        for group_index in start_group..=batch_end {
            self.grouping_logs.push(format!(
                "namespace={} episode_source_set={} reason=bounded_grouping stage=nrem_migration",
                self.namespace.as_str(),
                group_index
            ));
        }
        self.processed = batch_end;
        self.durable_token = DurableStateToken(self.processed as u64);

        if self.processed >= self.total {
            self.completed = true;
            MaintenanceStep::Completed(ConsolidationSummary {
                groups_evaluated: self.processed,
                episode_source_sets: self.episode_source_sets,
                merges_performed: self.merges,
                deduplications: self.deduplications,
                skipped: self.skipped,
                memories_affected: self.merges * 2 + self.deduplications,
                grouping_logs: self.grouping_logs.clone(),
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

/// Canonical consolidation engine owned by the core crate.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct ConsolidationEngine;

impl ConsolidationEngine {
    /// Returns the stable component identifier.
    pub const fn component_name(&self) -> &'static str {
        "engine.consolidation"
    }

    /// Creates a bounded consolidation run for the given namespace.
    pub fn create_run(
        &self,
        namespace: NamespaceId,
        policy: ConsolidationPolicy,
        estimated_groups: u32,
    ) -> ConsolidationRun {
        ConsolidationRun::new(namespace, policy, estimated_groups)
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
    fn consolidation_run_completes_in_bounded_polls() {
        let engine = ConsolidationEngine;
        let run = engine.create_run(ns("test"), ConsolidationPolicy::default(), 20);
        let mut handle = MaintenanceJobHandle::new(run, 10);

        handle.start();
        let snap = handle.poll();

        // Should complete since batch_size(50) > total(20)
        assert!(matches!(snap.state, MaintenanceJobState::Completed(_)));
        if let MaintenanceJobState::Completed(summary) = snap.state {
            assert_eq!(summary.groups_evaluated, 20);
            assert_eq!(summary.episode_source_sets, 20);
            assert_eq!(summary.grouping_logs.len(), 20);
        }
    }

    #[test]
    fn consolidation_run_can_be_cancelled() {
        let engine = ConsolidationEngine;
        let run = engine.create_run(
            ns("test"),
            ConsolidationPolicy {
                batch_size: 5,
                ..Default::default()
            },
            100,
        );
        let mut handle = MaintenanceJobHandle::new(run, 100);

        handle.start();
        handle.poll(); // Process first batch
        let snap = handle.cancel();

        assert!(matches!(
            snap.state,
            MaintenanceJobState::CancelRequested { .. }
        ));

        let snap = handle.poll();
        assert!(matches!(snap.state, MaintenanceJobState::Cancelled(_)));
    }

    #[test]
    fn consolidation_run_makes_progress_when_batch_size_is_zero() {
        let engine = ConsolidationEngine;
        let run = engine.create_run(
            ns("test"),
            ConsolidationPolicy {
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
}
