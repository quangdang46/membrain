//! Episode formation and source-set grouping rules.
//!
//! Exposes bounded grouping logic for consolidation, defining how raw
//! memory items are clustered into episodes before higher-order derivations
//! (like summaries or facts) are produced.

use crate::api::TaskId;
use crate::graph::EntityId;
use crate::types::{MemoryId, SessionId};

/// Stable identifier for a grouped source set or episode.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct EpisodeId(pub u64);

/// Explicit heuristics used to determine if two items belong in the same episode.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GroupingHeuristics {
    /// Maximum time gap (in milliseconds) allowed between adjacent events in an episode.
    pub temporal_proximity_ms: u64,
    /// Absolute maximum span (in milliseconds) an episode can run.
    pub max_episode_span_ms: u64,
    /// Whether a shared session identifier strongly binds events together.
    pub honor_session_bounds: bool,
    /// Whether a shared task identifier strongly binds events together.
    pub honor_task_bounds: bool,
    /// Minimum entity overlap required if time/session are weak bounds.
    pub require_entity_overlap: bool,
}

impl Default for GroupingHeuristics {
    fn default() -> Self {
        Self {
            temporal_proximity_ms: 10 * 60 * 1000, // 10 minutes
            max_episode_span_ms: 60 * 60 * 1000,   // 1 hour
            honor_session_bounds: true,
            honor_task_bounds: true,
            require_entity_overlap: false,
        }
    }
}

/// Input metadata required to evaluate grouping constraints.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EpisodeCandidate {
    pub memory_id: MemoryId,
    pub timestamp_ms: u64,
    pub session_id: Option<SessionId>,
    pub task_id: Option<TaskId>,
    pub entities: Vec<EntityId>,
    pub goal_context: Option<&'static str>,
    pub tool_chain_context: Option<&'static str>,
    pub failure_retry_flag: bool,
}

/// Stable classification for a formed source set.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SourceSetKind {
    Singleton,
    TemporalCluster,
    SessionCluster,
    TaskCluster,
    GoalCluster,
    ToolChainCluster,
    RetryCluster,
}

impl SourceSetKind {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Singleton => "singleton",
            Self::TemporalCluster => "temporal_cluster",
            Self::SessionCluster => "session_cluster",
            Self::TaskCluster => "task_cluster",
            Self::GoalCluster => "goal_cluster",
            Self::ToolChainCluster => "tool_chain_cluster",
            Self::RetryCluster => "retry_cluster",
        }
    }
}

/// Explain payload showing why an episode was formed.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EpisodeFormationExplain {
    pub primary_reason: &'static str,
    pub time_span_ms: u64,
    pub matching_fields: Vec<&'static str>,
    pub anchor_memory_id: MemoryId,
    pub terminal_memory_id: MemoryId,
}

/// Explicit lineage payload preserved for grouped source sets.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EpisodeLineage {
    pub source_memory_ids: Vec<MemoryId>,
    pub continuity_keys: Vec<&'static str>,
}

/// A formed episode or source set.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SourceGroup {
    pub episode_id: EpisodeId,
    pub fixture_name: String,
    pub source_set_kind: SourceSetKind,
    pub members: Vec<MemoryId>,
    pub lineage: EpisodeLineage,
    pub explain: EpisodeFormationExplain,
}

/// Bounded, deterministic report for one grouping pass.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EpisodeGroupingReport {
    pub groups: Vec<SourceGroup>,
    pub heuristics_summary: String,
    pub sample_logs: Vec<String>,
}

/// Stable grouping boundary owned by `membrain-core`.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct EpisodeGroupingModule;

impl EpisodeGroupingModule {
    /// Returns the stable component identifier.
    pub const fn component_name(&self) -> &'static str {
        "engine.episode_grouping"
    }

    /// Clusters a linear sequence of candidates into episodes.
    ///
    /// The input `candidates` are assumed to be sorted chronologically.
    pub fn form_episodes(
        &self,
        heuristics: &GroupingHeuristics,
        candidates: &[EpisodeCandidate],
    ) -> Vec<SourceGroup> {
        let mut groups = Vec::new();
        if candidates.is_empty() {
            return groups;
        }

        let mut current_group = vec![candidates[0].clone()];
        let mut group_start_time = candidates[0].timestamp_ms;
        let mut next_id = 1;

        for candidate in candidates.iter().skip(1) {
            let last = current_group.last().unwrap();
            let mut join = false;

            let time_gap = candidate.timestamp_ms.saturating_sub(last.timestamp_ms);
            let time_span = candidate.timestamp_ms.saturating_sub(group_start_time);

            if time_span <= heuristics.max_episode_span_ms {
                join = time_gap <= heuristics.temporal_proximity_ms
                    || (heuristics.honor_session_bounds
                        && candidate.session_id.is_some()
                        && candidate.session_id == last.session_id)
                    || (heuristics.honor_task_bounds
                        && candidate.task_id.is_some()
                        && candidate.task_id == last.task_id)
                    || (candidate.goal_context.is_some()
                        && candidate.goal_context == last.goal_context)
                    || (candidate.tool_chain_context.is_some()
                        && candidate.tool_chain_context == last.tool_chain_context)
                    || (candidate.failure_retry_flag && last.failure_retry_flag);
            }

            if heuristics.require_entity_overlap && join {
                let has_overlap = candidate.entities.iter().any(|e| last.entities.contains(e));
                if !has_overlap {
                    join = false;
                }
            }

            if join {
                current_group.push(candidate.clone());
            } else {
                groups.push(Self::finalize_group(EpisodeId(next_id), &current_group));
                next_id += 1;
                current_group.clear();
                current_group.push(candidate.clone());
                group_start_time = candidate.timestamp_ms;
            }
        }

        if !current_group.is_empty() {
            groups.push(Self::finalize_group(EpisodeId(next_id), &current_group));
        }

        groups
    }

    /// Produces a deterministic report that can be attached to maintenance and inspect surfaces.
    pub fn grouping_report(
        &self,
        heuristics: &GroupingHeuristics,
        candidates: &[EpisodeCandidate],
    ) -> EpisodeGroupingReport {
        let groups = self.form_episodes(heuristics, candidates);
        let sample_logs = groups.iter().map(Self::sample_log_line).collect();
        EpisodeGroupingReport {
            groups,
            heuristics_summary: Self::heuristics_summary(heuristics),
            sample_logs,
        }
    }

    fn finalize_group(episode_id: EpisodeId, members: &[EpisodeCandidate]) -> SourceGroup {
        let first = members.first().unwrap();
        let last = members.last().unwrap();
        let time_span_ms = last.timestamp_ms.saturating_sub(first.timestamp_ms);
        let source_set_kind = Self::classify_group_kind(members);
        let primary_reason = source_set_kind.as_str();

        let mut matching_fields = vec![primary_reason];
        if members.len() > 1
            && members
                .iter()
                .all(|m| m.session_id == first.session_id && m.session_id.is_some())
        {
            matching_fields.push("session_id");
        }
        if members.len() > 1
            && members
                .iter()
                .all(|m| m.task_id == first.task_id && m.task_id.is_some())
        {
            matching_fields.push("task_id");
        }
        if members.len() > 1
            && members
                .iter()
                .all(|m| m.goal_context == first.goal_context && m.goal_context.is_some())
        {
            matching_fields.push("goal_context");
        }
        if members.len() > 1
            && members.iter().all(|m| {
                m.tool_chain_context == first.tool_chain_context && m.tool_chain_context.is_some()
            })
        {
            matching_fields.push("tool_chain_context");
        }
        if members.len() > 1 && members.iter().all(|m| m.failure_retry_flag) {
            matching_fields.push("failure_retry");
        }

        if members.len() > 1 {
            let has_entity_overlap = members.windows(2).all(|pair| {
                pair[0]
                    .entities
                    .iter()
                    .any(|entity| pair[1].entities.contains(entity))
            });
            if has_entity_overlap {
                matching_fields.push("entity_overlap");
            }
        }

        let fixture_name = format!(
            "episode_{}_{}_{}_{}",
            episode_id.0,
            source_set_kind.as_str(),
            first.memory_id.0,
            last.memory_id.0
        );
        let continuity_keys = matching_fields.clone();
        let source_memory_ids = members.iter().map(|c| c.memory_id).collect();

        SourceGroup {
            episode_id,
            fixture_name,
            source_set_kind,
            members: members.iter().map(|c| c.memory_id).collect(),
            lineage: EpisodeLineage {
                source_memory_ids,
                continuity_keys,
            },
            explain: EpisodeFormationExplain {
                primary_reason,
                time_span_ms,
                matching_fields,
                anchor_memory_id: first.memory_id,
                terminal_memory_id: last.memory_id,
            },
        }
    }

    fn classify_group_kind(members: &[EpisodeCandidate]) -> SourceSetKind {
        let first = members.first().unwrap();
        if members.len() == 1 {
            SourceSetKind::Singleton
        } else if members
            .iter()
            .all(|m| m.session_id == first.session_id && m.session_id.is_some())
        {
            SourceSetKind::SessionCluster
        } else if members
            .iter()
            .all(|m| m.task_id == first.task_id && m.task_id.is_some())
        {
            SourceSetKind::TaskCluster
        } else if members
            .iter()
            .all(|m| m.goal_context == first.goal_context && m.goal_context.is_some())
        {
            SourceSetKind::GoalCluster
        } else if members.iter().all(|m| {
            m.tool_chain_context == first.tool_chain_context && m.tool_chain_context.is_some()
        }) {
            SourceSetKind::ToolChainCluster
        } else if members.iter().all(|m| m.failure_retry_flag) {
            SourceSetKind::RetryCluster
        } else {
            SourceSetKind::TemporalCluster
        }
    }

    fn heuristics_summary(heuristics: &GroupingHeuristics) -> String {
        format!(
            "temporal_proximity_ms={} max_episode_span_ms={} honor_session_bounds={} honor_task_bounds={} require_entity_overlap={}",
            heuristics.temporal_proximity_ms,
            heuristics.max_episode_span_ms,
            heuristics.honor_session_bounds,
            heuristics.honor_task_bounds,
            heuristics.require_entity_overlap,
        )
    }

    fn sample_log_line(group: &SourceGroup) -> String {
        format!(
            "fixture={} episode={} kind={} anchor={} terminal={} members={:?} lineage={:?} reason={} fields={:?}",
            group.fixture_name,
            group.episode_id.0,
            group.source_set_kind.as_str(),
            group.explain.anchor_memory_id.0,
            group.explain.terminal_memory_id.0,
            group.members.iter().map(|id| id.0).collect::<Vec<_>>(),
            group.lineage
                .source_memory_ids
                .iter()
                .map(|id| id.0)
                .collect::<Vec<_>>(),
            group.explain.primary_reason,
            group.explain.matching_fields,
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn cand(
        id: u64,
        time: u64,
        tsk: Option<&str>,
        sess: Option<u64>,
        ents: Vec<u64>,
    ) -> EpisodeCandidate {
        EpisodeCandidate {
            memory_id: MemoryId(id),
            timestamp_ms: time,
            session_id: sess.map(SessionId),
            task_id: tsk.map(TaskId::new),
            entities: ents.into_iter().map(EntityId).collect(),
            goal_context: None,
            tool_chain_context: None,
            failure_retry_flag: false,
        }
    }

    #[test]
    fn single_item_forms_singleton_episode() {
        let engine = EpisodeGroupingModule;
        let cands = vec![cand(1, 1000, None, None, vec![])];

        let groups = engine.form_episodes(&GroupingHeuristics::default(), &cands);
        assert_eq!(groups.len(), 1);
        assert_eq!(groups[0].members.len(), 1);
        assert_eq!(groups[0].source_set_kind, SourceSetKind::Singleton);
        assert_eq!(groups[0].explain.primary_reason, "singleton");
        assert_eq!(groups[0].explain.time_span_ms, 0);
        assert_eq!(groups[0].explain.anchor_memory_id, MemoryId(1));
        assert_eq!(groups[0].explain.terminal_memory_id, MemoryId(1));
    }

    #[test]
    fn groups_by_temporal_proximity() {
        let engine = EpisodeGroupingModule;
        let cands = vec![
            cand(1, 1000, None, None, vec![]),
            cand(2, 60_000, None, None, vec![]),
            cand(3, 1_000_000, None, None, vec![]),
            cand(4, 1_050_000, None, None, vec![]),
        ];

        let groups = engine.form_episodes(&GroupingHeuristics::default(), &cands);
        assert_eq!(groups.len(), 2);
        assert_eq!(groups[0].members, vec![MemoryId(1), MemoryId(2)]);
        assert_eq!(groups[0].source_set_kind, SourceSetKind::TemporalCluster);
        assert_eq!(groups[0].explain.primary_reason, "temporal_cluster");
        assert_eq!(groups[1].members, vec![MemoryId(3), MemoryId(4)]);
        assert_eq!(groups[1].source_set_kind, SourceSetKind::TemporalCluster);
        assert_eq!(groups[1].explain.primary_reason, "temporal_cluster");
    }

    #[test]
    fn groups_by_shared_task_even_if_temporal_gap_large() {
        let engine = EpisodeGroupingModule;
        let heuristics = GroupingHeuristics {
            temporal_proximity_ms: 1000,
            ..Default::default()
        };

        let cands = vec![
            cand(1, 1000, Some("task-A"), None, vec![]),
            cand(2, 10_000, Some("task-A"), None, vec![]),
            cand(3, 20_000, Some("task-B"), None, vec![]),
        ];

        let groups = engine.form_episodes(&heuristics, &cands);
        assert_eq!(groups.len(), 2);
        assert_eq!(groups[0].members, vec![MemoryId(1), MemoryId(2)]);
        assert_eq!(groups[0].source_set_kind, SourceSetKind::TaskCluster);
        assert_eq!(groups[0].explain.primary_reason, "task_cluster");
        assert_eq!(groups[1].members, vec![MemoryId(3)]);
        assert_eq!(groups[1].explain.primary_reason, "singleton");
    }

    #[test]
    fn breaks_episode_on_hard_span_limit() {
        let engine = EpisodeGroupingModule;
        let heuristics = GroupingHeuristics {
            max_episode_span_ms: 1_000_000,
            ..Default::default()
        };

        let cands = vec![
            cand(1, 100_000, Some("task-A"), None, vec![]),
            cand(2, 600_000, Some("task-A"), None, vec![]),
            cand(3, 1_200_000, Some("task-A"), None, vec![]),
        ];

        let groups = engine.form_episodes(&heuristics, &cands);
        assert_eq!(groups.len(), 2);
        assert_eq!(groups[0].members, vec![MemoryId(1), MemoryId(2)]);
        assert_eq!(groups[1].members, vec![MemoryId(3)]);
    }

    #[test]
    fn honors_entity_overlap_constraint() {
        let engine = EpisodeGroupingModule;
        let heuristics = GroupingHeuristics {
            require_entity_overlap: true,
            ..Default::default()
        };

        let cands = vec![
            cand(1, 1000, None, None, vec![100, 101]),
            cand(2, 1100, None, None, vec![101, 102]),
            cand(3, 1200, None, None, vec![999]),
        ];

        let groups = engine.form_episodes(&heuristics, &cands);
        assert_eq!(groups.len(), 2);
        assert_eq!(groups[0].members, vec![MemoryId(1), MemoryId(2)]);
        assert!(groups[0]
            .explain
            .matching_fields
            .contains(&"entity_overlap"));
        assert_eq!(groups[1].members, vec![MemoryId(3)]);
        assert_eq!(groups[1].explain.matching_fields, vec!["singleton"]);
    }

    #[test]
    fn singleton_after_overlap_rejection_does_not_keep_temporal_reason() {
        let engine = EpisodeGroupingModule;
        let heuristics = GroupingHeuristics {
            require_entity_overlap: true,
            ..Default::default()
        };

        let cands = vec![
            cand(1, 1000, None, None, vec![10]),
            cand(2, 1100, None, None, vec![20]),
        ];

        let groups = engine.form_episodes(&heuristics, &cands);
        assert_eq!(groups.len(), 2);
        assert_eq!(groups[0].explain.primary_reason, "singleton");
        assert_eq!(groups[0].explain.matching_fields, vec!["singleton"]);
        assert_eq!(groups[1].explain.primary_reason, "singleton");
        assert_eq!(groups[1].explain.matching_fields, vec!["singleton"]);
    }

    #[test]
    fn grouping_report_emits_deterministic_sample_logs() {
        let engine = EpisodeGroupingModule;
        let cands = vec![
            cand(1, 1000, Some("task-A"), None, vec![10]),
            cand(2, 1100, Some("task-A"), None, vec![10, 20]),
        ];

        let report = engine.grouping_report(&GroupingHeuristics::default(), &cands);
        assert_eq!(report.groups.len(), 1);
        assert_eq!(report.sample_logs.len(), 1);
        assert_eq!(
            report.heuristics_summary,
            "temporal_proximity_ms=600000 max_episode_span_ms=3600000 honor_session_bounds=true honor_task_bounds=true require_entity_overlap=false"
        );
        assert_eq!(report.groups[0].fixture_name, "episode_1_task_cluster_1_2");
        assert_eq!(
            report.groups[0].lineage.source_memory_ids,
            vec![MemoryId(1), MemoryId(2)]
        );
        assert_eq!(
            report.groups[0].lineage.continuity_keys,
            vec!["task_cluster", "task_id", "entity_overlap"]
        );
        assert_eq!(
            report.sample_logs[0],
            "fixture=episode_1_task_cluster_1_2 episode=1 kind=task_cluster anchor=1 terminal=2 members=[1, 2] lineage=[1, 2] reason=task_cluster fields=[\"task_cluster\", \"task_id\", \"entity_overlap\"]"
        );
    }

    #[test]
    fn failure_retry_requires_continuity_across_group() {
        let engine = EpisodeGroupingModule;
        let heuristics = GroupingHeuristics {
            temporal_proximity_ms: 1_000,
            ..Default::default()
        };
        let mut first = cand(1, 1_000, None, None, vec![]);
        first.failure_retry_flag = true;
        let mut second = cand(2, 20_000, None, None, vec![]);
        second.failure_retry_flag = true;
        let third = cand(3, 40_000, None, None, vec![]);

        let groups = engine.form_episodes(&heuristics, &[first, second, third]);
        assert_eq!(groups.len(), 2);
        assert_eq!(groups[0].source_set_kind, SourceSetKind::RetryCluster);
        assert_eq!(groups[0].explain.primary_reason, "retry_cluster");
        assert!(groups[0].explain.matching_fields.contains(&"failure_retry"));
        assert_eq!(groups[1].source_set_kind, SourceSetKind::Singleton);
    }
}
