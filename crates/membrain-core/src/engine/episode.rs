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

/// Explain payload showing why an episode was formed.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EpisodeFormationExplain {
    pub primary_reason: &'static str,
    pub time_span_ms: u64,
    pub matching_fields: Vec<&'static str>,
}

/// A formed episode or source set.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SourceGroup {
    pub episode_id: EpisodeId,
    pub members: Vec<MemoryId>,
    pub explain: EpisodeFormationExplain,
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
            let mut matched_fields = Vec::new();

            let time_gap = candidate.timestamp_ms.saturating_sub(last.timestamp_ms);
            let time_span = candidate.timestamp_ms.saturating_sub(group_start_time);

            if time_span > heuristics.max_episode_span_ms {
                // Hard limit, definitely break episode, skipping other heuristics
            } else if time_gap <= heuristics.temporal_proximity_ms {
                join = true;
                matched_fields.push("temporal_proximity");
            } else if heuristics.honor_session_bounds && candidate.session_id.is_some() && candidate.session_id == last.session_id {
                join = true;
                matched_fields.push("session_id");
            } else if heuristics.honor_task_bounds && candidate.task_id.is_some() && candidate.task_id == last.task_id {
                join = true;
                matched_fields.push("task_id");
            } else if candidate.goal_context.is_some() && candidate.goal_context == last.goal_context {
                join = true;
                matched_fields.push("goal_context");
            } else if candidate.tool_chain_context.is_some() && candidate.tool_chain_context == last.tool_chain_context {
                join = true;
                matched_fields.push("tool_chain_continuity");
            } else if candidate.failure_retry_flag {
                join = true;
                matched_fields.push("failure_retry_continuity");
            }

            if heuristics.require_entity_overlap && join {
                let has_overlap = candidate.entities.iter().any(|e| last.entities.contains(e));
                if !has_overlap {
                    join = false;
                } else {
                    matched_fields.push("entity_overlap");
                }
            }

            if join {
                current_group.push(candidate.clone());
            } else {
                groups.push(Self::finalize_group(EpisodeId(next_id), &current_group, matched_fields.clone()));
                next_id += 1;
                current_group.clear();
                current_group.push(candidate.clone());
                group_start_time = candidate.timestamp_ms;
            }
        }

        if !current_group.is_empty() {
            groups.push(Self::finalize_group(EpisodeId(next_id), &current_group, vec![]));
        }

        groups
    }

    fn finalize_group(
        episode_id: EpisodeId,
        members: &[EpisodeCandidate],
        mut matching_fields: Vec<&'static str>,
    ) -> SourceGroup {
        let first = members.first().unwrap();
        let last = members.last().unwrap();
        let time_span_ms = last.timestamp_ms.saturating_sub(first.timestamp_ms);
        
        let primary_reason = if members.len() == 1 {
            "singleton"
        } else if members.iter().all(|m| m.session_id == first.session_id && m.session_id.is_some()) {
            "shared_session"
        } else if members.iter().all(|m| m.task_id == first.task_id && m.task_id.is_some()) {
            "shared_task"
        } else {
            "temporal_proximity"
        };
        
        if matching_fields.is_empty() {
            matching_fields.push(primary_reason);
        }

        SourceGroup {
            episode_id,
            members: members.iter().map(|c| c.memory_id).collect(),
            explain: EpisodeFormationExplain {
                primary_reason,
                time_span_ms,
                matching_fields,
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn cand(id: u64, time: u64, tsk: Option<&str>, sess: Option<u64>, ents: Vec<u64>) -> EpisodeCandidate {
        EpisodeCandidate {
            memory_id: MemoryId(id),
            timestamp_ms: time,
            session_id: sess.map(SessionId),
            task_id: tsk.map(|s| TaskId::new(s)),
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
        assert_eq!(groups[0].explain.primary_reason, "singleton");
        assert_eq!(groups[0].explain.time_span_ms, 0);
    }

    #[test]
    fn groups_by_temporal_proximity() {
        let engine = EpisodeGroupingModule;
        // Times in ms (diff is 5 mins = 300,000 ms)
        let cands = vec![
            cand(1, 1000, None, None, vec![]),
            cand(2, 60_000, None, None, vec![]), // 59s later
            cand(3, 1_000_000, None, None, vec![]), // >> 10 mins later
            cand(4, 1_050_000, None, None, vec![]), // 50s after 3
        ];
        
        let groups = engine.form_episodes(&GroupingHeuristics::default(), &cands);
        assert_eq!(groups.len(), 2);
        
        // Group 1: 1 and 2
        assert_eq!(groups[0].members, vec![MemoryId(1), MemoryId(2)]);
        assert_eq!(groups[0].explain.primary_reason, "temporal_proximity");
        
        // Group 2: 3 and 4
        assert_eq!(groups[1].members, vec![MemoryId(3), MemoryId(4)]);
        assert_eq!(groups[1].explain.primary_reason, "temporal_proximity");
    }

    #[test]
    fn groups_by_shared_task_even_if_temporal_gap_large() {
        let engine = EpisodeGroupingModule;
        let mut heuristics = GroupingHeuristics::default();
        heuristics.temporal_proximity_ms = 1000; // very short temporal bound
        
        let cands = vec![
            cand(1, 1000, Some("task-A"), None, vec![]),
            cand(2, 10_000, Some("task-A"), None, vec![]), // 9s later, > 1s threshold but same task
            cand(3, 20_000, Some("task-B"), None, vec![]), // diff task
        ];
        
        let groups = engine.form_episodes(&heuristics, &cands);
        assert_eq!(groups.len(), 2);
        
        // Group 1: 1 and 2
        assert_eq!(groups[0].members, vec![MemoryId(1), MemoryId(2)]);
        assert_eq!(groups[0].explain.primary_reason, "shared_task");
        
        // Group 2: 3
        assert_eq!(groups[1].members, vec![MemoryId(3)]);
        assert_eq!(groups[1].explain.primary_reason, "singleton");
    }

    #[test]
    fn breaks_episode_on_hard_span_limit() {
        let engine = EpisodeGroupingModule;
        let mut heuristics = GroupingHeuristics::default();
        heuristics.max_episode_span_ms = 1_000_000; 
        
        let cands = vec![
            cand(1, 100_000, Some("task-A"), None, vec![]),
            cand(2, 600_000, Some("task-A"), None, vec![]), 
            cand(3, 1_200_000, Some("task-A"), None, vec![]), // Over span limit relative to cand 1
        ];
        
        let groups = engine.form_episodes(&heuristics, &cands);
        // Candidate 3 is forced into a new group even though it shares a task
        assert_eq!(groups.len(), 2);
        assert_eq!(groups[0].members, vec![MemoryId(1), MemoryId(2)]);
        assert_eq!(groups[1].members, vec![MemoryId(3)]);
    }

    #[test]
    fn honors_entity_overlap_constraint() {
        let engine = EpisodeGroupingModule;
        let mut heuristics = GroupingHeuristics::default();
        heuristics.require_entity_overlap = true;
        
        let cands = vec![
            cand(1, 1000, None, None, vec![100, 101]),
            cand(2, 1100, None, None, vec![101, 102]), // Temporal proximity, overlap on 101
            cand(3, 1200, None, None, vec![999]),      // Temporal proximity but NO overlap
        ];
        
        let groups = engine.form_episodes(&heuristics, &cands);
        assert_eq!(groups.len(), 2);
        
        // 1 & 2 grouped because of overlap.
        assert_eq!(groups[0].members, vec![MemoryId(1), MemoryId(2)]);
        
        // 3 broken off because no entity overlap with 2
        assert_eq!(groups[1].members, vec![MemoryId(3)]);
    }
}
