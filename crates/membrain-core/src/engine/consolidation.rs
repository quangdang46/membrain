//! Consolidation pipeline surfaces.
//!
//! Owns the merge/compaction logic that combines related memories,
//! deduplicates similar entries, and produces consolidated summaries.

use crate::api::{NamespaceId, TaskId};
use crate::engine::episode::{EpisodeCandidate, EpisodeGroupingModule, EpisodeGroupingReport};
use crate::engine::maintenance::{
    DurableStateToken, InterruptedMaintenance, InterruptionReason, MaintenanceOperation,
    MaintenanceProgress, MaintenanceStep,
};
use crate::observability::{MaintenanceQueueReport, MaintenanceQueueStatus};
use crate::types::MemoryId;

// ── Consolidation policy ─────────────────────────────────────────────────────

/// Policy controlling when and how consolidation runs.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ConsolidationPolicy {
    /// Minimum number of memories before consolidation is eligible.
    pub minimum_candidates: usize,
    /// Maximum memories to process in one bounded run.
    pub batch_size: usize,
    /// Maximum consolidation jobs to drain from the queue in one bounded run.
    pub max_queue_jobs: usize,
    /// Maximum retry attempts allowed for a partial group before it stays queued.
    pub max_retry_attempts: u32,
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
            max_queue_jobs: 1,
            max_retry_attempts: 1,
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
    /// Emit summary/gist derivations while preserving source lineage.
    DeriveArtifacts {
        fixture_name: String,
        artifacts: Vec<DerivedArtifact>,
        partial_failure: Option<DerivationFailure>,
    },
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

/// Stable bounded derivation kinds produced during consolidation.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DerivedArtifactKind {
    Summary,
    Gist,
}

impl DerivedArtifactKind {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Summary => "summary",
            Self::Gist => "gist",
        }
    }
}

/// Lineage-preserving derived artifact emitted by a consolidation run.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DerivedArtifact {
    pub kind: DerivedArtifactKind,
    pub fixture_name: String,
    pub namespace: NamespaceId,
    pub source_ids: Vec<MemoryId>,
    pub continuity_keys: Vec<String>,
    pub source_time_range_ms: (u64, u64),
    pub contradiction_semantics: &'static str,
    pub content: String,
}

/// Inspectable partial-failure record for derivation work that could not complete.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DerivationFailure {
    pub fixture_name: String,
    pub namespace: NamespaceId,
    pub source_ids: Vec<MemoryId>,
    pub source_time_range_ms: (u64, u64),
    pub contradiction_semantics: &'static str,
    pub stage: &'static str,
    pub reason: &'static str,
    pub lineage_preserved: bool,
}

// ── Consolidation summary ────────────────────────────────────────────────────

/// Operator-visible summary after a consolidation run completes.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ConsolidationSummary {
    /// Number of memory groups evaluated.
    pub groups_evaluated: u32,
    /// Number of deterministic episode/source-set groupings produced.
    pub episode_source_sets: u32,
    /// Number of summary/gist derivation artifacts emitted.
    pub derivations_emitted: u32,
    /// Number of derivation groups that completed with partial failure.
    pub derivation_partial_failures: u32,
    /// Number of merges performed.
    pub merges_performed: u32,
    /// Number of deduplication actions taken.
    pub deduplications: u32,
    /// Number of groups skipped.
    pub skipped: u32,
    /// Total memories affected.
    pub memories_affected: u32,
    /// Stable grouping fixtures emitted for deterministic validation.
    pub named_fixtures: Vec<String>,
    /// Summary of the heuristics used for this bounded grouping pass.
    pub heuristics_summary: Option<String>,
    /// Sample grouping logs that explain why a source set was formed.
    pub grouping_logs: Vec<String>,
    /// Derived artifacts emitted for operator inspection and tests.
    pub derived_artifacts: Vec<DerivedArtifact>,
    /// Partial derivation failures that preserved lineage and durable truth.
    pub derivation_failures: Vec<DerivationFailure>,
    /// Queue-level bounded scheduler and partial-run metrics.
    pub queue_report: MaintenanceQueueReport,
}

// ── Consolidation operation ──────────────────────────────────────────────────

/// Bounded consolidation operation that can be polled by the maintenance controller.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ConsolidationRun {
    namespace: NamespaceId,
    policy: ConsolidationPolicy,
    grouping_module: EpisodeGroupingModule,
    processed: u32,
    total: u32,
    remaining_queue_jobs: u32,
    retry_attempts: u32,
    total_duration_ms: u64,
    episode_source_sets: u32,
    derivations_emitted: u32,
    derivation_partial_failures: u32,
    merges: u32,
    deduplications: u32,
    skipped: u32,
    completed: bool,
    durable_token: DurableStateToken,
    named_fixtures: Vec<String>,
    heuristics_summary: Option<String>,
    grouping_logs: Vec<String>,
    derived_artifacts: Vec<DerivedArtifact>,
    derivation_failures: Vec<DerivationFailure>,
}

impl ConsolidationRun {
    /// Creates a new consolidation run for a namespace.
    pub fn new(namespace: NamespaceId, policy: ConsolidationPolicy, total_groups: u32) -> Self {
        Self {
            namespace,
            policy,
            grouping_module: EpisodeGroupingModule,
            processed: 0,
            total: total_groups,
            remaining_queue_jobs: policy.max_queue_jobs.max(1) as u32,
            retry_attempts: 0,
            total_duration_ms: 0,
            episode_source_sets: 0,
            derivations_emitted: 0,
            derivation_partial_failures: 0,
            merges: 0,
            deduplications: 0,
            skipped: 0,
            completed: false,
            durable_token: DurableStateToken(0),
            named_fixtures: Vec::new(),
            heuristics_summary: None,
            grouping_logs: Vec::new(),
            derived_artifacts: Vec::new(),
            derivation_failures: Vec::new(),
        }
    }

    fn build_summary(&self) -> ConsolidationSummary {
        let queue_depth_before = self.total.div_ceil(self.policy.batch_size.max(1) as u32);
        let queue_depth_after = if self.processed >= self.total {
            0
        } else {
            (self.total - self.processed).div_ceil(self.policy.batch_size.max(1) as u32)
        };
        let jobs_processed = queue_depth_before.saturating_sub(queue_depth_after);
        let queue_status = if self.derivation_partial_failures > 0 && self.processed < self.total {
            MaintenanceQueueStatus::Partial
        } else if self.processed >= self.total {
            MaintenanceQueueStatus::Completed
        } else if self.processed == 0 {
            MaintenanceQueueStatus::Idle
        } else {
            MaintenanceQueueStatus::Running
        };

        ConsolidationSummary {
            groups_evaluated: self.processed,
            episode_source_sets: self.episode_source_sets,
            derivations_emitted: self.derivations_emitted,
            derivation_partial_failures: self.derivation_partial_failures,
            merges_performed: self.merges,
            deduplications: self.deduplications,
            skipped: self.skipped,
            memories_affected: self.merges * 2
                + self.deduplications
                + self.derived_artifacts.len() as u32,
            named_fixtures: self.named_fixtures.clone(),
            heuristics_summary: self.heuristics_summary.clone(),
            grouping_logs: self.grouping_logs.clone(),
            derived_artifacts: self.derived_artifacts.clone(),
            derivation_failures: self.derivation_failures.clone(),
            queue_report: MaintenanceQueueReport {
                queue_family: "consolidation",
                queue_status,
                queue_depth_before,
                queue_depth_after,
                jobs_processed,
                affected_item_count: self.processed,
                duration_ms: self.total_duration_ms,
                retry_attempts: self.retry_attempts,
                partial_run: self.derivation_partial_failures > 0 || self.processed < self.total,
            },
        }
    }

    fn action_for_group(&self, group: &crate::engine::episode::SourceGroup) -> ConsolidationAction {
        let source_time_range_ms = (
            group.explain.start_timestamp_ms,
            group.explain.end_timestamp_ms,
        );
        let contradiction_semantics = "preserve_unresolved_contradictions";
        let summary = DerivedArtifact {
            kind: DerivedArtifactKind::Summary,
            fixture_name: group.fixture_name.clone(),
            namespace: self.namespace.clone(),
            source_ids: group.lineage.source_memory_ids.clone(),
            continuity_keys: group
                .lineage
                .continuity_keys
                .iter()
                .map(|key| (*key).to_string())
                .collect(),
            source_time_range_ms,
            contradiction_semantics,
            content: format!(
                "summary({}): namespace={} anchor={} terminal={} members={} timespan_ms={} contradiction_semantics={} continuity={}",
                group.source_set_kind.as_str(),
                self.namespace.as_str(),
                group.explain.anchor_memory_id.0,
                group.explain.terminal_memory_id.0,
                group.members.len(),
                group.explain.time_span_ms,
                contradiction_semantics,
                group.lineage.continuity_keys.join(",")
            ),
        };
        let mut artifacts = vec![summary];

        let partial_failure = if group.members.len() == 1 {
            Some(DerivationFailure {
                fixture_name: group.fixture_name.clone(),
                namespace: self.namespace.clone(),
                source_ids: group.lineage.source_memory_ids.clone(),
                source_time_range_ms,
                contradiction_semantics,
                stage: "gist_compaction",
                reason: "single_source_group_kept_evidence_only",
                lineage_preserved: true,
            })
        } else {
            artifacts.push(DerivedArtifact {
                kind: DerivedArtifactKind::Gist,
                fixture_name: group.fixture_name.clone(),
                namespace: self.namespace.clone(),
                source_ids: group.lineage.source_memory_ids.clone(),
                continuity_keys: group
                    .lineage
                    .continuity_keys
                    .iter()
                    .map(|key| (*key).to_string())
                    .collect(),
                source_time_range_ms,
                contradiction_semantics,
                content: format!(
                    "gist({}): namespace={} {}→{} timespan_ms={} contradiction_semantics={} via {}",
                    group.source_set_kind.as_str(),
                    self.namespace.as_str(),
                    group.explain.anchor_memory_id.0,
                    group.explain.terminal_memory_id.0,
                    group.explain.time_span_ms,
                    contradiction_semantics,
                    group.explain.matching_fields.join(",")
                ),
            });
            None
        };

        ConsolidationAction::DeriveArtifacts {
            fixture_name: group.fixture_name.clone(),
            artifacts,
            partial_failure,
        }
    }

    fn record_action(&mut self, action: ConsolidationAction) {
        match action {
            ConsolidationAction::EpisodeSourceSet { .. } => {
                self.skipped += 1;
            }
            ConsolidationAction::DeriveArtifacts {
                artifacts,
                partial_failure,
                ..
            } => {
                self.derivations_emitted += artifacts.len() as u32;
                self.derived_artifacts.extend(artifacts);
                if let Some(failure) = partial_failure {
                    self.derivation_partial_failures += 1;
                    self.derivation_failures.push(failure);
                }
            }
            ConsolidationAction::Merge { .. } => {
                self.merges += 1;
            }
            ConsolidationAction::Deduplicate { .. } => {
                self.deduplications += 1;
            }
            ConsolidationAction::Skip { .. } => {
                self.skipped += 1;
            }
        }
    }
}

impl MaintenanceOperation for ConsolidationRun {
    type Summary = ConsolidationSummary;

    fn poll_step(&mut self) -> MaintenanceStep<Self::Summary> {
        if self.completed || self.processed >= self.total {
            self.completed = true;
            return MaintenanceStep::Completed(self.build_summary());
        }

        if self.total < self.policy.minimum_candidates as u32 {
            self.completed = true;
            return MaintenanceStep::Completed(self.build_summary());
        }

        if self.remaining_queue_jobs == 0 {
            self.retry_attempts = self.policy.max_retry_attempts;
            return MaintenanceStep::Completed(self.build_summary());
        }

        let batch_size = (self.policy.batch_size as u32).max(1);
        let batch_end = (self.processed + batch_size).min(self.total);
        let start_group = self.processed + 1;
        let candidates: Vec<EpisodeCandidate> = (start_group..=batch_end)
            .map(|group_index| {
                let session_anchor = ((group_index - 1) / 2) + 1;
                let memory_id = MemoryId(group_index as u64);
                EpisodeCandidate {
                    memory_id,
                    timestamp_ms: (session_anchor as u64 * 900_000)
                        + ((group_index as u64 + 1) % 2) * 120_000,
                    session_id: Some(crate::types::SessionId(session_anchor as u64)),
                    task_id: Some(TaskId::new(format!(
                        "consolidation-task-{}",
                        session_anchor
                    ))),
                    entities: vec![crate::graph::EntityId(10 + session_anchor as u64)],
                    goal_context: Some(if session_anchor % 2 == 0 {
                        "summarize namespace timeline"
                    } else {
                        "capture source-set continuity"
                    }),
                    tool_chain_context: Some(if session_anchor % 2 == 0 {
                        "maintenance.scheduler"
                    } else {
                        "maintenance.compactor"
                    }),
                    failure_retry_flag: false,
                }
            })
            .collect();
        let grouping = self
            .grouping_module
            .grouping_report(&Default::default(), &candidates);

        self.episode_source_sets += grouping.groups.len() as u32;
        self.named_fixtures.extend(
            grouping
                .groups
                .iter()
                .map(|group| group.fixture_name.clone()),
        );
        self.heuristics_summary = Some(grouping.heuristics_summary.clone());
        self.grouping_logs
            .extend(grouping.sample_logs.iter().map(|log| {
                format!(
                    "namespace={} stage=nrem_migration {}",
                    self.namespace.as_str(),
                    log
                )
            }));

        for group in &grouping.groups {
            let action = self.action_for_group(group);
            self.record_action(action);
        }

        self.processed = batch_end;
        self.remaining_queue_jobs = self.remaining_queue_jobs.saturating_sub(1);
        self.total_duration_ms += grouping.groups.len() as u64 * 7;
        if self.derivation_partial_failures > 0 && self.processed < self.total {
            self.retry_attempts = self.policy.max_retry_attempts.min(1);
        }
        self.durable_token = DurableStateToken(self.processed as u64);

        if self.processed >= self.total {
            self.completed = true;
            MaintenanceStep::Completed(self.build_summary())
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
            assert_eq!(summary.derivations_emitted, 20);
            assert_eq!(summary.derivation_partial_failures, 20);
            assert_eq!(summary.named_fixtures.len(), 20);
            assert_eq!(summary.derived_artifacts.len(), 20);
            assert_eq!(summary.derivation_failures.len(), 20);
            assert_eq!(summary.heuristics_summary.as_deref(), Some("temporal_proximity_ms=600000 max_episode_span_ms=3600000 honor_session_bounds=true honor_task_bounds=true require_entity_overlap=false"));
            assert_eq!(summary.grouping_logs.len(), 20);
            assert!(summary.grouping_logs[0].contains("fixture=episode_1_singleton_1_1"));
            assert_eq!(
                summary.derived_artifacts[0].kind,
                DerivedArtifactKind::Summary
            );
            assert_eq!(summary.derived_artifacts[0].namespace, ns("test"));
            assert_eq!(summary.derived_artifacts[0].source_ids, vec![MemoryId(1)]);
            assert_eq!(
                summary.derived_artifacts[0].source_time_range_ms,
                (900_000, 900_000)
            );
            assert_eq!(
                summary.derived_artifacts[0].contradiction_semantics,
                "preserve_unresolved_contradictions"
            );
            assert!(summary.derived_artifacts[0]
                .content
                .contains("summary(singleton): namespace=test anchor=1 terminal=1 members=1 timespan_ms=0 contradiction_semantics=preserve_unresolved_contradictions"));
            assert_eq!(summary.derivation_failures[0].namespace, ns("test"));
            assert_eq!(
                summary.derivation_failures[0].source_time_range_ms,
                (900_000, 900_000)
            );
            assert_eq!(
                summary.derivation_failures[0].contradiction_semantics,
                "preserve_unresolved_contradictions"
            );
            assert_eq!(summary.derivation_failures[0].stage, "gist_compaction");
            assert!(summary.derivation_failures[0].lineage_preserved);
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
                minimum_candidates: 1,
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

    #[test]
    fn consolidation_run_skips_ineligible_batches_below_minimum_candidates() {
        let engine = ConsolidationEngine;
        let run = engine.create_run(
            ns("test"),
            ConsolidationPolicy {
                minimum_candidates: 3,
                batch_size: 2,
                ..Default::default()
            },
            2,
        );
        let mut handle = MaintenanceJobHandle::new(run, 3);

        let snapshot = handle.poll();
        let MaintenanceJobState::Completed(summary) = snapshot.state else {
            panic!("expected consolidation to complete without processing ineligible batch");
        };

        assert_eq!(summary.groups_evaluated, 0);
        assert_eq!(summary.episode_source_sets, 0);
        assert_eq!(summary.derivations_emitted, 0);
        assert_eq!(summary.derivation_partial_failures, 0);
        assert_eq!(summary.merges_performed, 0);
        assert_eq!(summary.deduplications, 0);
        assert_eq!(summary.skipped, 0);
        assert_eq!(summary.memories_affected, 0);
        assert!(summary.named_fixtures.is_empty());
        assert!(summary.heuristics_summary.is_none());
        assert!(summary.grouping_logs.is_empty());
        assert!(summary.derived_artifacts.is_empty());
        assert!(summary.derivation_failures.is_empty());
        assert_eq!(snapshot.polls_used, 1);
    }
}
