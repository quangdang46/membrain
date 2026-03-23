//! Consolidation pipeline surfaces.
//!
//! Owns the merge/compaction logic that combines related memories,
//! deduplicates similar entries, and produces consolidated summaries.

use crate::api::NamespaceId;
use crate::engine::episode::{EpisodeCandidate, EpisodeGroupingModule, EpisodeGroupingReport};
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
    pub source_ids: Vec<MemoryId>,
    pub continuity_keys: Vec<String>,
    pub content: String,
}

/// Inspectable partial-failure record for derivation work that could not complete.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DerivationFailure {
    pub fixture_name: String,
    pub source_ids: Vec<MemoryId>,
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
        }
    }

    fn action_for_group(&self, group: &crate::engine::episode::SourceGroup) -> ConsolidationAction {
        let summary = DerivedArtifact {
            kind: DerivedArtifactKind::Summary,
            fixture_name: group.fixture_name.clone(),
            source_ids: group.lineage.source_memory_ids.clone(),
            continuity_keys: group
                .lineage
                .continuity_keys
                .iter()
                .map(|key| (*key).to_string())
                .collect(),
            content: format!(
                "summary({}): anchor={} terminal={} members={} continuity={}",
                group.source_set_kind.as_str(),
                group.explain.anchor_memory_id.0,
                group.explain.terminal_memory_id.0,
                group.members.len(),
                group.lineage.continuity_keys.join(",")
            ),
        };
        let mut artifacts = vec![summary];

        let partial_failure = if group.members.len() == 1 {
            Some(DerivationFailure {
                fixture_name: group.fixture_name.clone(),
                source_ids: group.lineage.source_memory_ids.clone(),
                stage: "gist_compaction",
                reason: "single_source_group_kept_evidence_only",
                lineage_preserved: true,
            })
        } else {
            artifacts.push(DerivedArtifact {
                kind: DerivedArtifactKind::Gist,
                fixture_name: group.fixture_name.clone(),
                source_ids: group.lineage.source_memory_ids.clone(),
                continuity_keys: group
                    .lineage
                    .continuity_keys
                    .iter()
                    .map(|key| (*key).to_string())
                    .collect(),
                content: format!(
                    "gist({}): {}→{} via {}",
                    group.source_set_kind.as_str(),
                    group.explain.anchor_memory_id.0,
                    group.explain.terminal_memory_id.0,
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

        let batch_size = (self.policy.batch_size as u32).max(1);
        let batch_end = (self.processed + batch_size).min(self.total);
        let start_group = self.processed + 1;
        let candidates: Vec<EpisodeCandidate> = (start_group..=batch_end)
            .map(|group_index| EpisodeCandidate {
                memory_id: MemoryId(group_index as u64),
                timestamp_ms: group_index as u64 * 700_000,
                session_id: None,
                task_id: None,
                entities: Vec::new(),
                goal_context: None,
                tool_chain_context: None,
                failure_retry_flag: false,
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
            assert_eq!(summary.derived_artifacts[0].source_ids, vec![MemoryId(1)]);
            assert!(summary.derived_artifacts[0]
                .content
                .contains("summary(singleton): anchor=1 terminal=1 members=1"));
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
