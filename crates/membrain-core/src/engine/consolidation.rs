//! Consolidation pipeline surfaces.
//!
//! Owns the merge/compaction logic that combines related memories,
//! deduplicates similar entries, and produces consolidated summaries.

use crate::api::{ErrorKind, NamespaceId, RemediationStep, TaskId};
use crate::engine::episode::{EpisodeCandidate, EpisodeGroupingModule, EpisodeGroupingReport};
use crate::engine::maintenance::{
    DurableStateToken, InterruptedMaintenance, InterruptionReason, MaintenanceFailureArtifact,
    MaintenanceOperation, MaintenanceProgress, MaintenanceStep,
};
use crate::graph::{EntityId, RelationKind};
use crate::observability::{MaintenanceQueueReport, MaintenanceQueueStatus};
use crate::types::MemoryId;
use std::collections::BTreeMap;

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
    /// Minimum members required before tentative skill extraction may run.
    pub min_skill_members: usize,
    /// Minimum bounded cluster-quality score required for tentative skill extraction.
    pub min_skill_quality: u16,
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
            min_skill_members: 3,
            min_skill_quality: 700,
        }
    }
}

// ── Consolidation action ─────────────────────────────────────────────────────

/// Actions the consolidation engine can take on a memory group.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ConsolidationAction {
    /// Preserve a deterministic episode/source-set grouping before higher-order derivation work.
    EpisodeSourceSet { grouping: EpisodeGroupingReport },
    /// Emit summary/gist/fact/relation derivations while preserving source lineage.
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
    Fact,
    Relation,
    Skill,
}

impl DerivedArtifactKind {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Summary => "summary",
            Self::Gist => "gist",
            Self::Fact => "fact",
            Self::Relation => "relation",
            Self::Skill => "skill",
        }
    }
}

/// Provenance envelope preserved for every consolidation derivation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DerivationProvenance {
    pub source_kind: &'static str,
    pub source_ref: String,
    pub derived_from: Vec<MemoryId>,
    pub lineage_ancestors: Vec<MemoryId>,
    pub relation_to_seed: Option<RelationKind>,
    pub graph_seed: Option<EntityId>,
}

/// Stable source citation captured for each derivation input.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DerivedSourceCitation {
    pub memory_id: MemoryId,
    pub source_ref: String,
    pub timestamp_ms: u64,
    pub evidence_kind: &'static str,
}

/// Stable explain metadata showing how one derivation was formed.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DerivationExplain {
    pub derivation_rule: &'static str,
    pub status: &'static str,
    pub confidence: u16,
    pub supporting_fields: Vec<String>,
}

/// Inspectable reflection-compiler metadata preserved on advisory guidance artifacts.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReflectionArtifactMetadata {
    pub primary_guidance: &'static str,
    pub source_outcome: &'static str,
    pub checklist_items: Vec<String>,
    pub advisory: bool,
    pub trusted_by_default: bool,
    pub release_rule: &'static str,
    pub promotion_basis: &'static str,
}

/// Lineage-preserving derived artifact emitted by a consolidation run.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DerivedArtifact {
    pub kind: DerivedArtifactKind,
    pub fixture_name: String,
    pub namespace: NamespaceId,
    pub source_ids: Vec<MemoryId>,
    pub continuity_keys: Vec<String>,
    pub source_citations: Vec<DerivedSourceCitation>,
    pub source_time_range_ms: (u64, u64),
    pub contradiction_semantics: &'static str,
    pub provenance: DerivationProvenance,
    pub explain: DerivationExplain,
    pub content: String,
    pub reflection: Option<ReflectionArtifactMetadata>,
}

/// Inspectable partial-failure record for derivation work that could not complete.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DerivationFailure {
    pub fixture_name: String,
    pub namespace: NamespaceId,
    pub source_ids: Vec<MemoryId>,
    pub continuity_keys: Vec<String>,
    pub source_citations: Vec<DerivedSourceCitation>,
    pub source_time_range_ms: (u64, u64),
    pub contradiction_semantics: &'static str,
    pub provenance: DerivationProvenance,
    pub explain: DerivationExplain,
    pub stage: &'static str,
    pub reason: &'static str,
    pub lineage_preserved: bool,
}

// ── Consolidation summary ────────────────────────────────────────────────────

/// Stable bounded compaction unit selected by the consolidation planner.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CompactionUnitKind {
    EpisodeSourceSet,
}

impl CompactionUnitKind {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::EpisodeSourceSet => "episode_source_set",
        }
    }
}

/// Inspectable unit-selection contract for one bounded compaction run.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CompactionUnitSelection {
    pub unit_kind: CompactionUnitKind,
    pub selected_units: u32,
    pub selection_rule: &'static str,
    pub durable_truth_source: &'static str,
    pub derived_outputs: Vec<&'static str>,
    pub authoritative_evidence_retained: bool,
}

/// Inspectable batching and cancellation contract for one bounded compaction run.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CompactionBatchPlan {
    pub unit_selection: CompactionUnitSelection,
    pub batch_size: u32,
    pub queue_jobs_budget: u32,
    pub planned_batches: u32,
    pub completed_batches: u32,
    pub pending_batches: u32,
    pub completed_units: u32,
    pub pending_units: u32,
    pub cancellation_checkpoint: DurableStateToken,
    pub cancellation_safety: &'static str,
}

/// Stable operator-facing compaction report emitted by one bounded compaction run.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CompactionOperatorReport {
    pub report_kind: &'static str,
    pub unit_kind: &'static str,
    pub authoritative_truth_source: &'static str,
    pub selected_units: u32,
    pub completed_units: u32,
    pub pending_units: u32,
    pub queue_jobs_budget: u32,
    pub queue_depth_before: u32,
    pub queue_depth_after: u32,
    pub degraded_mode: Option<&'static str>,
    pub rollback_trigger: Option<&'static str>,
    pub remediation_steps: Vec<RemediationStep>,
}

/// Operator-visible summary after a consolidation run completes.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ConsolidationSummary {
    /// Number of memory groups evaluated.
    pub groups_evaluated: u32,
    /// Number of deterministic episode/source-set groupings produced.
    pub episode_source_sets: u32,
    /// Number of summary/gist/fact/relation derivation artifacts emitted.
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
    /// Partial or repair-handoff artifacts emitted for queue or retry boundaries.
    pub failure_artifacts: Vec<MaintenanceFailureArtifact>,
    /// Queue-level bounded scheduler and partial-run metrics.
    pub queue_report: MaintenanceQueueReport,
    /// Inspectable compaction planning contract for this bounded run.
    pub batch_plan: CompactionBatchPlan,
    /// Stable operator-facing compaction report for degraded-mode and rollback review.
    pub operator_report: CompactionOperatorReport,
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
    failure_artifacts: Vec<MaintenanceFailureArtifact>,
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
            failure_artifacts: Vec::new(),
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
        let queue_budget_exhausted = self.processed < self.total && self.remaining_queue_jobs == 0;
        let queue_status = if self.processed >= self.total {
            MaintenanceQueueStatus::Completed
        } else if self.processed == 0 {
            MaintenanceQueueStatus::Idle
        } else if self.derivation_partial_failures > 0 || queue_budget_exhausted {
            MaintenanceQueueStatus::Partial
        } else {
            MaintenanceQueueStatus::Running
        };
        let pending_units = self.total.saturating_sub(self.processed);
        let planned_batches = queue_depth_before;
        let completed_batches = jobs_processed;
        let pending_batches = queue_depth_after;
        let mut failure_artifacts = self.failure_artifacts.clone();
        let degraded_mode = queue_budget_exhausted.then_some("continue_degraded_reads");
        let rollback_trigger = queue_budget_exhausted.then_some("verification_mismatch");
        if queue_budget_exhausted {
            failure_artifacts.push(self.repair_boundary_artifact(
                "consolidation_queue_budget_boundary",
                "scheduler_drain",
                self.processed,
                pending_units,
                ErrorKind::TimeoutFailure,
                true,
                "queue_budget_exhausted",
            ));
        }
        let remediation_steps = if queue_budget_exhausted {
            vec![
                RemediationStep::CheckHealth,
                RemediationStep::RollbackRecentChange,
                RemediationStep::RunRepair,
                RemediationStep::InspectState,
            ]
        } else {
            Vec::new()
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
            failure_artifacts,
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
            batch_plan: CompactionBatchPlan {
                unit_selection: CompactionUnitSelection {
                    unit_kind: CompactionUnitKind::EpisodeSourceSet,
                    selected_units: self.total,
                    selection_rule: "episode_source_sets_with_lineage_and_durable_citations_and_schema_compression",
                    durable_truth_source: "durable_memory_rows",
                    derived_outputs: vec!["summary", "fact", "gist", "relation", "skill", "schema"],
                    authoritative_evidence_retained: true,
                },
                batch_size: self.policy.batch_size.max(1) as u32,
                queue_jobs_budget: queue_depth_before,
                planned_batches,
                completed_batches,
                pending_batches,
                completed_units: self.processed,
                pending_units,
                cancellation_checkpoint: self.durable_token,
                cancellation_safety:
                    "interruptions preserve durable truth and resume from durable checkpoint",
            },
            operator_report: CompactionOperatorReport {
                report_kind: "compaction_run_report",
                unit_kind: CompactionUnitKind::EpisodeSourceSet.as_str(),
                authoritative_truth_source: "durable_memory_rows",
                selected_units: self.total,
                completed_units: self.processed,
                pending_units,
                queue_jobs_budget: queue_depth_before,
                queue_depth_before,
                queue_depth_after,
                degraded_mode,
                rollback_trigger,
                remediation_steps,
            },
        }
    }

    fn source_citations_for_group(
        &self,
        group: &crate::engine::episode::SourceGroup,
    ) -> Vec<DerivedSourceCitation> {
        group
            .lineage
            .source_citations
            .iter()
            .map(|citation| DerivedSourceCitation {
                memory_id: citation.memory_id,
                source_ref: format!(
                    "memory://{}/{}",
                    self.namespace.as_str(),
                    citation.memory_id.0
                ),
                timestamp_ms: citation.timestamp_ms,
                evidence_kind: "durable_memory",
            })
            .collect()
    }

    fn run_handle(&self) -> String {
        format!("consolidation://{}", self.namespace.as_str())
    }

    fn repair_boundary_artifact(
        &self,
        artifact_name: &'static str,
        attempted_edge: &'static str,
        affected_item_count: u32,
        pending_item_count: u32,
        failure_family: ErrorKind,
        retryable: bool,
        escalation_boundary: &'static str,
    ) -> MaintenanceFailureArtifact {
        MaintenanceFailureArtifact {
            artifact_name,
            object_handle: self.run_handle(),
            scope: self.namespace.as_str().to_string(),
            attempted_edge,
            affected_item_count,
            pending_item_count,
            failure_family: failure_family.as_str(),
            retryable,
            escalation_boundary,
        }
    }

    fn explain_for_group(
        group: &crate::engine::episode::SourceGroup,
        kind: DerivedArtifactKind,
        status: &'static str,
    ) -> DerivationExplain {
        let base: u16 = match kind {
            DerivedArtifactKind::Summary => 580,
            DerivedArtifactKind::Gist => 520,
            DerivedArtifactKind::Fact => 600,
            DerivedArtifactKind::Relation => 600,
            DerivedArtifactKind::Skill => 640,
        };
        let support_bonus = (group.members.len().saturating_sub(1) as u16).saturating_mul(40);
        let field_bonus = (group.explain.matching_fields.len() as u16).saturating_mul(15);
        let confidence = base
            .saturating_add(support_bonus)
            .saturating_add(field_bonus)
            .min(950);

        DerivationExplain {
            derivation_rule: match kind {
                DerivedArtifactKind::Summary => "episode_summary",
                DerivedArtifactKind::Gist => "gist_compaction",
                DerivedArtifactKind::Fact => "fact_extraction",
                DerivedArtifactKind::Relation => "relation_reinforcement",
                DerivedArtifactKind::Skill => "skill_extraction",
            },
            status,
            confidence,
            supporting_fields: group
                .explain
                .matching_fields
                .iter()
                .map(|field| (*field).to_string())
                .collect(),
        }
    }

    fn skill_quality_for_group(&self, group: &crate::engine::episode::SourceGroup) -> u16 {
        let mut score: u16 = 420;
        score = score
            .saturating_add((group.members.len().min(6) as u16).saturating_mul(55))
            .saturating_add((group.explain.matching_fields.len().min(6) as u16).saturating_mul(18));

        if matches!(
            group.source_set_kind,
            crate::engine::episode::SourceSetKind::SessionCluster
                | crate::engine::episode::SourceSetKind::TaskCluster
                | crate::engine::episode::SourceSetKind::GoalCluster
                | crate::engine::episode::SourceSetKind::ToolChainCluster
                | crate::engine::episode::SourceSetKind::RetryCluster
        ) {
            score = score.saturating_add(70);
        }

        if group.explain.common_goal_context.is_some() {
            score = score.saturating_add(45);
        }
        if group.explain.common_tool_chain_context.is_some() {
            score = score.saturating_add(35);
        }
        if group.explain.matching_fields.contains(&"entity_overlap") {
            score = score.saturating_add(30);
        }
        if group.explain.matching_fields.contains(&"failure_retry") {
            score = score.saturating_add(25);
        }

        score.min(950)
    }

    fn derive_skill_artifact(
        &self,
        group: &crate::engine::episode::SourceGroup,
        source_ids: &[MemoryId],
        continuity_keys: &[String],
        source_citations: &[DerivedSourceCitation],
        source_time_range_ms: (u64, u64),
        contradiction_semantics: &'static str,
        graph_seed: Option<EntityId>,
    ) -> Option<DerivedArtifact> {
        if group.members.len() < self.policy.min_skill_members {
            return None;
        }

        let quality_score = self.skill_quality_for_group(group);
        if quality_score < self.policy.min_skill_quality {
            return None;
        }

        let mut keyword_counts: BTreeMap<&'static str, usize> = BTreeMap::new();
        for field in &group.explain.matching_fields {
            *keyword_counts.entry(field).or_default() += 2;
        }
        if let Some(goal) = group.explain.common_goal_context {
            for token in goal.split_whitespace() {
                let normalized = token.trim_matches(|ch: char| !ch.is_ascii_alphanumeric());
                if normalized.len() >= 4 {
                    *keyword_counts.entry(normalized).or_default() += 3;
                }
            }
        }
        if let Some(tool_chain) = group.explain.common_tool_chain_context {
            for token in tool_chain.split(|ch: char| !(ch.is_ascii_alphanumeric() || ch == '.')) {
                if token.len() >= 4 {
                    *keyword_counts.entry(token).or_default() += 2;
                }
            }
        }

        let mut top_keywords = keyword_counts.into_iter().collect::<Vec<_>>();
        top_keywords.sort_by(|left, right| right.1.cmp(&left.1).then_with(|| left.0.cmp(right.0)));
        let top_keywords = top_keywords
            .into_iter()
            .take(4)
            .map(|(keyword, _)| keyword.to_string())
            .collect::<Vec<_>>();

        let context_hint = group
            .explain
            .common_goal_context
            .or(group.explain.common_tool_chain_context)
            .unwrap_or(group.explain.primary_reason);
        let successful_outcome = !group.explain.matching_fields.contains(&"failure_retry");
        let primary_guidance = if successful_outcome {
            "procedure"
        } else {
            "anti_pattern"
        };
        let outcome_label = if successful_outcome {
            "successful_episode"
        } else {
            "failed_episode"
        };
        let action_hint = if let Some(tool_chain) = group.explain.common_tool_chain_context {
            format!("repeat bounded workflow via {tool_chain}")
        } else if group.explain.matching_fields.contains(&"failure_retry") {
            "avoid reusing the failed retry path without new evidence".to_string()
        } else {
            format!(
                "repeat the shared {} pattern",
                group.source_set_kind.as_str().replace('_', " ")
            )
        };
        let checklist_items = vec![
            format!("confirm goal context: {context_hint}"),
            format!("review action trace: {action_hint}"),
            format!("inspect supporting evidence count: {}", source_ids.len()),
            format!("verify advisory status before reuse: {outcome_label}"),
        ];
        let checklist_rendered = checklist_items.join("|");
        let keywords_rendered = if top_keywords.is_empty() {
            "none".to_string()
        } else {
            top_keywords.join(",")
        };

        Some(DerivedArtifact {
            kind: DerivedArtifactKind::Skill,
            fixture_name: group.fixture_name.clone(),
            namespace: self.namespace.clone(),
            source_ids: source_ids.to_vec(),
            continuity_keys: continuity_keys.to_vec(),
            source_citations: source_citations.to_vec(),
            source_time_range_ms,
            contradiction_semantics,
            provenance: DerivationProvenance {
                source_kind: "consolidation",
                source_ref: format!(
                    "consolidation://{}/{}#skill",
                    self.namespace.as_str(),
                    group.fixture_name
                ),
                derived_from: source_ids.to_vec(),
                lineage_ancestors: source_ids.to_vec(),
                relation_to_seed: Some(RelationKind::DerivedFrom),
                graph_seed,
            },
            explain: Self::explain_for_group(group, DerivedArtifactKind::Skill, "tentative"),
            content: format!(
                "skill({}): namespace={} source_engram_id={} confidence={} member_count={} tentative=true accepted=false guidance={} source_outcome={} advisory=true trusted_by_default=false release_rule=explicit_acceptance_or_repeated_use_with_lineage action_pattern={} checklist={} keywords={} citations={}",
                group.source_set_kind.as_str(),
                self.namespace.as_str(),
                graph_seed.map_or(0, |seed| seed.0),
                quality_score,
                group.members.len(),
                primary_guidance,
                outcome_label,
                action_hint,
                checklist_rendered,
                keywords_rendered,
                source_citations
                    .iter()
                    .map(|citation| format!("{}:{}", citation.evidence_kind, citation.source_ref))
                    .collect::<Vec<_>>()
                    .join(",")
            ),
            reflection: Some(ReflectionArtifactMetadata {
                primary_guidance,
                source_outcome: outcome_label,
                checklist_items,
                advisory: true,
                trusted_by_default: false,
                release_rule: "explicit_acceptance_or_repeated_use_with_lineage",
                promotion_basis: "human_approval_or_repeated_usefulness",
            }),
        })
    }

    fn action_for_group(&self, group: &crate::engine::episode::SourceGroup) -> ConsolidationAction {
        let source_time_range_ms = (
            group.explain.start_timestamp_ms,
            group.explain.end_timestamp_ms,
        );
        let contradiction_semantics = "preserve_unresolved_contradictions";
        let source_ids = group.lineage.source_memory_ids.clone();
        let continuity_keys: Vec<String> = group
            .lineage
            .continuity_keys
            .iter()
            .map(|key| (*key).to_string())
            .collect();
        let source_citations = self.source_citations_for_group(group);
        let graph_seed = group
            .members
            .first()
            .map(|memory_id| EntityId(10 + memory_id.0.div_ceil(2)));
        let summary_provenance = DerivationProvenance {
            source_kind: "consolidation",
            source_ref: format!(
                "consolidation://{}/{}#summary",
                self.namespace.as_str(),
                group.fixture_name
            ),
            derived_from: source_ids.clone(),
            lineage_ancestors: source_ids.clone(),
            relation_to_seed: Some(RelationKind::DerivedFrom),
            graph_seed,
        };
        let summary = DerivedArtifact {
            kind: DerivedArtifactKind::Summary,
            fixture_name: group.fixture_name.clone(),
            namespace: self.namespace.clone(),
            source_ids: source_ids.clone(),
            continuity_keys: continuity_keys.clone(),
            source_citations: source_citations.clone(),
            source_time_range_ms,
            contradiction_semantics,
            provenance: summary_provenance,
            explain: Self::explain_for_group(group, DerivedArtifactKind::Summary, "complete"),
            content: format!(
                "summary({}): namespace={} anchor={} terminal={} members={} timespan_ms={} contradiction_semantics={} continuity={} citations={}",
                group.source_set_kind.as_str(),
                self.namespace.as_str(),
                group.explain.anchor_memory_id.0,
                group.explain.terminal_memory_id.0,
                group.members.len(),
                group.explain.time_span_ms,
                contradiction_semantics,
                group.lineage.continuity_keys.join(","),
                source_citations
                    .iter()
                    .map(|citation| format!("{}@{}", citation.memory_id.0, citation.timestamp_ms))
                    .collect::<Vec<_>>()
                    .join(",")
            ),
            reflection: None,
        };
        let mut artifacts = vec![summary];

        if group.members.len() > 1 {
            artifacts.push(DerivedArtifact {
                kind: DerivedArtifactKind::Fact,
                fixture_name: group.fixture_name.clone(),
                namespace: self.namespace.clone(),
                source_ids: source_ids.clone(),
                continuity_keys: continuity_keys.clone(),
                source_citations: source_citations.clone(),
                source_time_range_ms,
                contradiction_semantics,
                provenance: DerivationProvenance {
                    source_kind: "consolidation",
                    source_ref: format!(
                        "consolidation://{}/{}#fact",
                        self.namespace.as_str(),
                        group.fixture_name
                    ),
                    derived_from: source_ids.clone(),
                    lineage_ancestors: source_ids.clone(),
                    relation_to_seed: Some(RelationKind::DerivedFrom),
                    graph_seed,
                },
                explain: Self::explain_for_group(group, DerivedArtifactKind::Fact, "complete"),
                content: format!(
                    "fact({}): namespace={} canonical_claim=episode:{} members={} time_range_ms={}..{} contradiction_semantics={} lineage={} citations={} evidence_rule=durable_memory_only ",
                    group.source_set_kind.as_str(),
                    self.namespace.as_str(),
                    group.episode_id.0,
                    group.members.len(),
                    source_time_range_ms.0,
                    source_time_range_ms.1,
                    contradiction_semantics,
                    source_ids
                        .iter()
                        .map(|id| id.0.to_string())
                        .collect::<Vec<_>>()
                        .join(","),
                    source_citations
                        .iter()
                        .map(|citation| format!("{}:{}", citation.evidence_kind, citation.source_ref))
                        .collect::<Vec<_>>()
                        .join(",")
                ),
                reflection: None,
            });
        }

        let relation_signal_reinforced = group
            .explain
            .matching_fields
            .iter()
            .any(|field| *field != group.explain.primary_reason);

        let partial_failure = if group.members.len() == 1 {
            Some(DerivationFailure {
                fixture_name: group.fixture_name.clone(),
                namespace: self.namespace.clone(),
                source_ids: source_ids.clone(),
                continuity_keys: continuity_keys.clone(),
                source_citations: source_citations.clone(),
                source_time_range_ms,
                contradiction_semantics,
                provenance: DerivationProvenance {
                    source_kind: "consolidation",
                    source_ref: format!(
                        "consolidation://{}/{}#gist_failure",
                        self.namespace.as_str(),
                        group.fixture_name
                    ),
                    derived_from: source_ids.clone(),
                    lineage_ancestors: source_ids.clone(),
                    relation_to_seed: Some(RelationKind::DerivedFrom),
                    graph_seed,
                },
                explain: Self::explain_for_group(group, DerivedArtifactKind::Gist, "partial"),
                stage: "gist_compaction",
                reason: "single_source_group_kept_evidence_only",
                lineage_preserved: true,
            })
        } else {
            artifacts.push(DerivedArtifact {
                kind: DerivedArtifactKind::Gist,
                fixture_name: group.fixture_name.clone(),
                namespace: self.namespace.clone(),
                source_ids: source_ids.clone(),
                continuity_keys: continuity_keys.clone(),
                source_citations: source_citations.clone(),
                source_time_range_ms,
                contradiction_semantics,
                provenance: DerivationProvenance {
                    source_kind: "consolidation",
                    source_ref: format!(
                        "consolidation://{}/{}#gist",
                        self.namespace.as_str(),
                        group.fixture_name
                    ),
                    derived_from: source_ids.clone(),
                    lineage_ancestors: source_ids.clone(),
                    relation_to_seed: Some(RelationKind::DerivedFrom),
                    graph_seed,
                },
                explain: Self::explain_for_group(group, DerivedArtifactKind::Gist, "complete"),
                content: format!(
                    "gist({}): namespace={} {}→{} timespan_ms={} contradiction_semantics={} via {} citations={}",
                    group.source_set_kind.as_str(),
                    self.namespace.as_str(),
                    group.explain.anchor_memory_id.0,
                    group.explain.terminal_memory_id.0,
                    group.explain.time_span_ms,
                    contradiction_semantics,
                    group.explain.matching_fields.join(","),
                    source_citations
                        .iter()
                        .map(|citation| format!("{}:{}", citation.evidence_kind, citation.source_ref))
                        .collect::<Vec<_>>()
                        .join(",")
                ),
                reflection: None,
            });
            if relation_signal_reinforced {
                artifacts.push(DerivedArtifact {
                    kind: DerivedArtifactKind::Relation,
                    fixture_name: group.fixture_name.clone(),
                    namespace: self.namespace.clone(),
                    source_ids: source_ids.clone(),
                    continuity_keys: continuity_keys.clone(),
                    source_citations: source_citations.clone(),
                    source_time_range_ms,
                    contradiction_semantics,
                    provenance: DerivationProvenance {
                        source_kind: "consolidation",
                        source_ref: format!(
                            "consolidation://{}/{}#relation",
                            self.namespace.as_str(),
                            group.fixture_name
                        ),
                        derived_from: source_ids.clone(),
                        lineage_ancestors: source_ids.clone(),
                        relation_to_seed: Some(RelationKind::SharedTopic),
                        graph_seed,
                    },
                    explain: Self::explain_for_group(
                        group,
                        DerivedArtifactKind::Relation,
                        "reinforced"
                    ),
                    content: format!(
                        "relation({}): namespace={} relation={} seed={} continuity={} contradiction_semantics={} status=reinforced members={} citations={} evidence_rule=durable_memory_only",
                        group.source_set_kind.as_str(),
                        self.namespace.as_str(),
                        RelationKind::SharedTopic.as_str(),
                        graph_seed.map_or(0, |seed| seed.0),
                        continuity_keys.join(","),
                        contradiction_semantics,
                        source_ids
                            .iter()
                            .map(|id| id.0.to_string())
                            .collect::<Vec<_>>()
                            .join("→"),
                        source_citations
                            .iter()
                            .map(|citation| format!("{}:{}", citation.evidence_kind, citation.source_ref))
                            .collect::<Vec<_>>()
                            .join(",")
                    ),
                    reflection: None,
                });
                if let Some(skill) = self.derive_skill_artifact(
                    group,
                    &source_ids,
                    &continuity_keys,
                    &source_citations,
                    source_time_range_ms,
                    contradiction_semantics,
                    graph_seed,
                ) {
                    artifacts.push(skill);
                }
                None
            } else {
                Some(DerivationFailure {
                    fixture_name: group.fixture_name.clone(),
                    namespace: self.namespace.clone(),
                    source_ids: source_ids.clone(),
                    continuity_keys: continuity_keys.clone(),
                    source_citations: source_citations.clone(),
                    source_time_range_ms,
                    contradiction_semantics,
                    provenance: DerivationProvenance {
                        source_kind: "consolidation",
                        source_ref: format!(
                            "consolidation://{}/{}#relation_failure",
                            self.namespace.as_str(),
                            group.fixture_name
                        ),
                        derived_from: source_ids.clone(),
                        lineage_ancestors: source_ids.clone(),
                        relation_to_seed: Some(RelationKind::SharedTopic),
                        graph_seed,
                    },
                    explain: Self::explain_for_group(
                        group,
                        DerivedArtifactKind::Relation,
                        "partial",
                    ),
                    stage: "relation_reinforcement",
                    reason: "unresolved_relation_signal_requires_review",
                    lineage_preserved: true,
                })
            }
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

        let mut batch_partial_failures = 0u32;
        let retry_budget_available = self.retry_attempts < self.policy.max_retry_attempts;
        for group in &grouping.groups {
            let action = self.action_for_group(group);
            if let ConsolidationAction::DeriveArtifacts {
                partial_failure: Some(failure),
                ..
            } = &action
            {
                batch_partial_failures += 1;
                let retryable = retry_budget_available;
                let failure_family = if retryable {
                    ErrorKind::TransientFailure
                } else {
                    ErrorKind::InternalFailure
                };
                let escalation_boundary = if retryable {
                    "retry_budget_remaining"
                } else {
                    "retry_budget_exhausted"
                };
                self.failure_artifacts.push(self.repair_boundary_artifact(
                    "consolidation_partial_derivation",
                    failure.stage,
                    group.members.len() as u32,
                    self.total.saturating_sub(batch_end),
                    failure_family,
                    retryable,
                    escalation_boundary,
                ));
            }
            self.record_action(action);
        }

        self.processed = batch_end;
        self.remaining_queue_jobs = self.remaining_queue_jobs.saturating_sub(1);
        self.total_duration_ms += grouping.groups.len() as u64 * 7;
        if batch_partial_failures > 0 {
            self.retry_attempts = self
                .retry_attempts
                .saturating_add(batch_partial_failures)
                .min(self.policy.max_retry_attempts);
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
            artifact: Some(self.repair_boundary_artifact(
                "consolidation_interruption",
                "scheduler_drain",
                self.processed,
                self.total.saturating_sub(self.processed),
                match reason {
                    InterruptionReason::Cancelled => ErrorKind::TransientFailure,
                    InterruptionReason::TimedOut => ErrorKind::TimeoutFailure,
                },
                self.processed < self.total,
                if self.retry_attempts < self.policy.max_retry_attempts {
                    "resume_from_durable_checkpoint"
                } else {
                    "operator_repair_required"
                },
            )),
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
            assert_eq!(summary.episode_source_sets, 10);
            assert_eq!(summary.derivations_emitted, 40);
            assert_eq!(summary.derivation_partial_failures, 0);
            assert_eq!(summary.named_fixtures.len(), 10);
            assert_eq!(summary.derived_artifacts.len(), 40);
            assert_eq!(summary.derivation_failures.len(), 0);
            assert_eq!(summary.heuristics_summary.as_deref(), Some("temporal_proximity_ms=600000 max_episode_span_ms=3600000 honor_session_bounds=true honor_task_bounds=true require_entity_overlap=false"));
            assert_eq!(summary.grouping_logs.len(), 10);
            assert!(summary.grouping_logs[0].contains("fixture=episode_1_session_cluster_1_2"));
            assert_eq!(
                summary.derived_artifacts[0].kind,
                DerivedArtifactKind::Summary
            );
            assert_eq!(summary.derived_artifacts[0].namespace, ns("test"));
            assert_eq!(
                summary.derived_artifacts[0].source_ids,
                vec![MemoryId(1), MemoryId(2)]
            );
            assert_eq!(
                summary.derived_artifacts[0].source_time_range_ms,
                (900_000, 1_020_000)
            );
            assert_eq!(
                summary.derived_artifacts[0].contradiction_semantics,
                "preserve_unresolved_contradictions"
            );
            assert!(summary.derived_artifacts[0]
                .content
                .contains("summary(session_cluster): namespace=test anchor=1 terminal=2 members=2 timespan_ms=120000 contradiction_semantics=preserve_unresolved_contradictions"));
            assert_eq!(
                summary.derived_artifacts[0].continuity_keys,
                vec![
                    "session_cluster",
                    "session_id",
                    "task_id",
                    "goal_context",
                    "tool_chain_context",
                    "entity_overlap"
                ]
            );
            assert_eq!(
                summary.derived_artifacts[0].provenance.source_kind,
                "consolidation"
            );
            assert_eq!(
                summary.derived_artifacts[0].provenance.source_ref,
                "consolidation://test/episode_1_session_cluster_1_2#summary"
            );
            assert_eq!(summary.derived_artifacts[1].kind, DerivedArtifactKind::Fact);
            assert_eq!(
                summary.derived_artifacts[1].source_ids,
                vec![MemoryId(1), MemoryId(2)]
            );
            assert_eq!(
                summary.derived_artifacts[1].source_citations,
                vec![
                    DerivedSourceCitation {
                        memory_id: MemoryId(1),
                        source_ref: "memory://test/1".to_string(),
                        timestamp_ms: 900_000,
                        evidence_kind: "durable_memory",
                    },
                    DerivedSourceCitation {
                        memory_id: MemoryId(2),
                        source_ref: "memory://test/2".to_string(),
                        timestamp_ms: 1_020_000,
                        evidence_kind: "durable_memory",
                    }
                ]
            );
            assert_eq!(
                summary.derived_artifacts[1].explain.derivation_rule,
                "fact_extraction"
            );
            assert_eq!(summary.derived_artifacts[1].explain.status, "complete");
            assert!(summary.derived_artifacts[1].explain.confidence >= 700);
            assert!(summary.derived_artifacts[1].content.contains(
                "citations=durable_memory:memory://test/1,durable_memory:memory://test/2"
            ));
            assert_eq!(summary.derived_artifacts[2].kind, DerivedArtifactKind::Gist);
            assert_eq!(
                summary.derived_artifacts[2].source_ids,
                vec![MemoryId(1), MemoryId(2)]
            );
            assert_eq!(
                summary.derived_artifacts[2].explain.derivation_rule,
                "gist_compaction"
            );
            assert_eq!(summary.derived_artifacts[2].explain.status, "complete");
            assert_eq!(
                summary.derived_artifacts[3].kind,
                DerivedArtifactKind::Relation
            );
            assert_eq!(
                summary.derived_artifacts[3].provenance.relation_to_seed,
                Some(RelationKind::SharedTopic)
            );
            assert_eq!(
                summary.derived_artifacts[3].source_citations,
                vec![
                    DerivedSourceCitation {
                        memory_id: MemoryId(1),
                        source_ref: "memory://test/1".to_string(),
                        timestamp_ms: 900_000,
                        evidence_kind: "durable_memory",
                    },
                    DerivedSourceCitation {
                        memory_id: MemoryId(2),
                        source_ref: "memory://test/2".to_string(),
                        timestamp_ms: 1_020_000,
                        evidence_kind: "durable_memory",
                    }
                ]
            );
            assert_eq!(
                summary.derived_artifacts[3].explain.derivation_rule,
                "relation_reinforcement"
            );
            assert_eq!(summary.derived_artifacts[3].explain.status, "reinforced");
            assert!(summary.derived_artifacts[3].explain.confidence >= 700);
            assert!(summary.derived_artifacts[3]
                .content
                .contains("relation(session_cluster): namespace=test relation=shared_topic"));
            assert!(summary.derived_artifacts[3]
                .content
                .contains("status=reinforced"));
        }
    }

    #[test]
    fn consolidation_run_extracts_tentative_skill_when_cluster_quality_is_high() {
        let run = ConsolidationRun::new(
            ns("test"),
            ConsolidationPolicy {
                min_skill_members: 2,
                min_skill_quality: 700,
                ..Default::default()
            },
            2,
        );
        let group = crate::engine::episode::SourceGroup {
            episode_id: crate::engine::episode::EpisodeId(7),
            fixture_name: "episode_7_session_cluster_1_2".to_string(),
            source_set_kind: crate::engine::episode::SourceSetKind::SessionCluster,
            members: vec![MemoryId(1), MemoryId(2)],
            lineage: crate::engine::episode::EpisodeLineage {
                source_memory_ids: vec![MemoryId(1), MemoryId(2)],
                continuity_keys: vec![
                    "session_cluster",
                    "session_id",
                    "task_id",
                    "goal_context",
                    "tool_chain_context",
                    "entity_overlap",
                ],
                source_citations: vec![
                    crate::engine::episode::EpisodeSourceCitation {
                        memory_id: MemoryId(1),
                        timestamp_ms: 900_000,
                    },
                    crate::engine::episode::EpisodeSourceCitation {
                        memory_id: MemoryId(2),
                        timestamp_ms: 1_020_000,
                    },
                ],
            },
            explain: crate::engine::episode::EpisodeFormationExplain {
                primary_reason: "session_cluster",
                start_timestamp_ms: 900_000,
                end_timestamp_ms: 1_020_000,
                time_span_ms: 120_000,
                matching_fields: vec![
                    "session_cluster",
                    "session_id",
                    "task_id",
                    "goal_context",
                    "tool_chain_context",
                    "entity_overlap",
                ],
                common_goal_context: Some("capture source-set continuity"),
                common_tool_chain_context: Some("maintenance.compactor"),
                anchor_memory_id: MemoryId(1),
                terminal_memory_id: MemoryId(2),
            },
        };

        let ConsolidationAction::DeriveArtifacts {
            artifacts,
            partial_failure,
            ..
        } = run.action_for_group(&group)
        else {
            panic!("expected derivation action");
        };

        assert_eq!(artifacts.len(), 5);
        assert_eq!(artifacts[4].kind, DerivedArtifactKind::Skill);
        assert_eq!(artifacts[4].explain.derivation_rule, "skill_extraction");
        assert_eq!(artifacts[4].explain.status, "tentative");
        assert_eq!(
            artifacts[4].provenance.source_ref,
            "consolidation://test/episode_7_session_cluster_1_2#skill"
        );
        assert_eq!(
            artifacts[4].provenance.relation_to_seed,
            Some(RelationKind::DerivedFrom)
        );
        assert!(artifacts[4]
            .content
            .contains("skill(session_cluster): namespace=test source_engram_id=11 confidence="));
        assert!(artifacts[4]
            .content
            .contains("tentative=true accepted=false"));
        assert!(artifacts[4].content.contains("guidance=procedure"));
        assert!(artifacts[4]
            .content
            .contains("source_outcome=successful_episode"));
        assert!(artifacts[4]
            .content
            .contains("advisory=true trusted_by_default=false"));
        assert!(artifacts[4]
            .content
            .contains("release_rule=explicit_acceptance_or_repeated_use_with_lineage"));
        assert!(artifacts[4]
            .content
            .contains("action_pattern=repeat bounded workflow via maintenance.compactor"));
        assert!(artifacts[4].content.contains("checklist="));
        assert!(artifacts[4].content.contains("keywords="));
        assert!(artifacts[4].content.contains("capture"));
        assert!(artifacts[4].content.contains("continuity"));
        assert!(artifacts[4].content.contains("compactor"));
        let reflection = artifacts[4]
            .reflection
            .as_ref()
            .expect("reflection compiler metadata present");
        assert_eq!(reflection.primary_guidance, "procedure");
        assert_eq!(reflection.source_outcome, "successful_episode");
        assert!(reflection.advisory);
        assert!(!reflection.trusted_by_default);
        assert!(!reflection.checklist_items.is_empty());
        assert!(partial_failure.is_none());
    }

    #[test]
    fn consolidation_run_marks_relation_reinforcement_partial_when_signal_is_only_temporal() {
        let run = ConsolidationRun::new(ns("test"), ConsolidationPolicy::default(), 2);
        let group = crate::engine::episode::SourceGroup {
            episode_id: crate::engine::episode::EpisodeId(42),
            fixture_name: "episode_42_temporal_cluster_1_2".to_string(),
            source_set_kind: crate::engine::episode::SourceSetKind::TemporalCluster,
            members: vec![MemoryId(1), MemoryId(2)],
            lineage: crate::engine::episode::EpisodeLineage {
                source_memory_ids: vec![MemoryId(1), MemoryId(2)],
                continuity_keys: vec!["temporal_cluster"],
                source_citations: vec![
                    crate::engine::episode::EpisodeSourceCitation {
                        memory_id: MemoryId(1),
                        timestamp_ms: 900_000,
                    },
                    crate::engine::episode::EpisodeSourceCitation {
                        memory_id: MemoryId(2),
                        timestamp_ms: 1_020_000,
                    },
                ],
            },
            explain: crate::engine::episode::EpisodeFormationExplain {
                primary_reason: "temporal_cluster",
                start_timestamp_ms: 900_000,
                end_timestamp_ms: 1_020_000,
                time_span_ms: 120_000,
                matching_fields: vec!["temporal_cluster"],
                common_goal_context: None,
                common_tool_chain_context: None,
                anchor_memory_id: MemoryId(1),
                terminal_memory_id: MemoryId(2),
            },
        };

        let ConsolidationAction::DeriveArtifacts {
            artifacts,
            partial_failure,
            ..
        } = run.action_for_group(&group)
        else {
            panic!("expected derivation action");
        };

        assert_eq!(artifacts.len(), 3);
        assert_eq!(artifacts[0].kind, DerivedArtifactKind::Summary);
        assert_eq!(artifacts[1].kind, DerivedArtifactKind::Fact);
        assert_eq!(artifacts[2].kind, DerivedArtifactKind::Gist);
        assert!(artifacts
            .iter()
            .all(|artifact| artifact.kind != DerivedArtifactKind::Relation));

        let failure = partial_failure.expect("expected relation reinforcement partial failure");
        assert_eq!(failure.stage, "relation_reinforcement");
        assert_eq!(failure.reason, "unresolved_relation_signal_requires_review");
        assert_eq!(failure.explain.derivation_rule, "relation_reinforcement");
        assert_eq!(failure.explain.status, "partial");
        assert_eq!(
            failure.provenance.source_ref,
            "consolidation://test/episode_42_temporal_cluster_1_2#relation_failure"
        );
        assert_eq!(
            failure.provenance.relation_to_seed,
            Some(crate::graph::RelationKind::SharedTopic)
        );
        assert_eq!(failure.continuity_keys, vec!["temporal_cluster"]);
        assert!(failure.lineage_preserved);
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
