//! Episode formation and source-set grouping rules.
//!
//! Exposes bounded grouping logic for consolidation, defining how raw
//! memory items are clustered into episodes before higher-order derivations
//! (like summaries or facts) are produced.

use crate::api::TaskId;
use crate::graph::EntityId;
use crate::types::{MemoryId, SessionId};
use std::collections::{BTreeMap, HashMap};

type SkillCandidateBucketKey = (&'static str, Option<&'static str>, Option<&'static str>);
type SkillCandidateBuckets<'a> = BTreeMap<SkillCandidateBucketKey, Vec<&'a SourceGroup>>;

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
    pub start_timestamp_ms: u64,
    pub end_timestamp_ms: u64,
    pub time_span_ms: u64,
    pub matching_fields: Vec<&'static str>,
    pub common_goal_context: Option<&'static str>,
    pub common_tool_chain_context: Option<&'static str>,
    pub anchor_memory_id: MemoryId,
    pub terminal_memory_id: MemoryId,
}

/// Stable per-source citation preserved for grouped source sets.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EpisodeSourceCitation {
    pub memory_id: MemoryId,
    pub timestamp_ms: u64,
}

/// Explicit lineage payload preserved for grouped source sets.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EpisodeLineage {
    pub source_memory_ids: Vec<MemoryId>,
    pub continuity_keys: Vec<&'static str>,
    pub source_citations: Vec<EpisodeSourceCitation>,
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

/// Heuristics for the tentative skill-candidate pass derived from episodic clusters.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SkillCandidateHeuristics {
    pub min_source_groups: usize,
    pub min_total_memories: usize,
    pub min_supporting_continuities: usize,
    pub require_context_anchor: bool,
    pub keyword_limit: usize,
}

impl Default for SkillCandidateHeuristics {
    fn default() -> Self {
        Self {
            min_source_groups: 2,
            min_total_memories: 4,
            min_supporting_continuities: 1,
            require_context_anchor: true,
            keyword_limit: 6,
        }
    }
}

/// Tentative pattern detected from repeated episodic clusters.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SkillCandidatePattern {
    pub candidate_fixture: String,
    pub source_set_kind: SourceSetKind,
    pub source_episode_ids: Vec<EpisodeId>,
    pub source_fixtures: Vec<String>,
    pub source_memory_ids: Vec<MemoryId>,
    pub member_count: usize,
    pub continuity_keys: Vec<&'static str>,
    pub dominant_keywords: Vec<String>,
    pub goal_context: Option<&'static str>,
    pub tool_chain_context: Option<&'static str>,
    pub confidence: u16,
    pub status: &'static str,
    pub pattern_summary: String,
}

/// Explicit rejection record for a cluster that did not qualify as a tentative skill pattern.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SkillCandidateRejection {
    pub cluster_fixture: String,
    pub source_set_kind: SourceSetKind,
    pub source_episode_ids: Vec<EpisodeId>,
    pub member_count: usize,
    pub reason: &'static str,
    pub dominant_keywords: Vec<String>,
}

/// Deterministic skill-candidate analysis report for one bounded grouping pass.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SkillCandidateReport {
    pub candidates: Vec<SkillCandidatePattern>,
    pub rejections: Vec<SkillCandidateRejection>,
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

    /// Runs the first tentative skill-analysis pass over already-bounded episodic clusters.
    pub fn analyze_skill_candidates(
        &self,
        groups: &[SourceGroup],
        heuristics: &SkillCandidateHeuristics,
    ) -> SkillCandidateReport {
        let mut candidate_buckets: SkillCandidateBuckets<'_> = BTreeMap::new();
        let mut rejections = Vec::new();

        for group in groups {
            let continuity_count = group
                .lineage
                .continuity_keys
                .iter()
                .filter(|key| **key != group.explain.primary_reason)
                .count();
            let has_context_anchor = group.explain.common_goal_context.is_some()
                || group.explain.common_tool_chain_context.is_some();

            let rejection_reason = if group.members.len() < 2 {
                Some("singleton_source_group")
            } else if continuity_count < heuristics.min_supporting_continuities {
                Some("insufficient_continuity_signal")
            } else if heuristics.require_context_anchor && !has_context_anchor {
                Some("missing_context_anchor")
            } else {
                None
            };

            if let Some(reason) = rejection_reason {
                rejections.push(SkillCandidateRejection {
                    cluster_fixture: group.fixture_name.clone(),
                    source_set_kind: group.source_set_kind,
                    source_episode_ids: vec![group.episode_id],
                    member_count: group.members.len(),
                    reason,
                    dominant_keywords: Self::keywords_for_groups(
                        std::slice::from_ref(&group),
                        heuristics.keyword_limit,
                    ),
                });
                continue;
            }

            let bucket_key = (
                group.explain.primary_reason,
                group.explain.common_goal_context,
                group.explain.common_tool_chain_context,
            );
            candidate_buckets.entry(bucket_key).or_default().push(group);
        }

        let mut candidates = Vec::new();
        for ((primary_reason, goal_context, tool_chain_context), bucket_groups) in candidate_buckets
        {
            let total_memories = bucket_groups
                .iter()
                .map(|group| group.members.len())
                .sum::<usize>();
            if bucket_groups.len() < heuristics.min_source_groups {
                rejections.push(SkillCandidateRejection {
                    cluster_fixture: bucket_groups[0].fixture_name.clone(),
                    source_set_kind: bucket_groups[0].source_set_kind,
                    source_episode_ids: bucket_groups
                        .iter()
                        .map(|group| group.episode_id)
                        .collect(),
                    member_count: total_memories,
                    reason: "insufficient_cluster_repetition",
                    dominant_keywords: Self::keywords_for_groups(
                        &bucket_groups,
                        heuristics.keyword_limit,
                    ),
                });
                continue;
            }
            if total_memories < heuristics.min_total_memories {
                rejections.push(SkillCandidateRejection {
                    cluster_fixture: bucket_groups[0].fixture_name.clone(),
                    source_set_kind: bucket_groups[0].source_set_kind,
                    source_episode_ids: bucket_groups
                        .iter()
                        .map(|group| group.episode_id)
                        .collect(),
                    member_count: total_memories,
                    reason: "insufficient_member_support",
                    dominant_keywords: Self::keywords_for_groups(
                        &bucket_groups,
                        heuristics.keyword_limit,
                    ),
                });
                continue;
            }

            let source_set_kind = bucket_groups[0].source_set_kind;
            let continuity_keys = Self::shared_continuity_keys(&bucket_groups);
            let dominant_keywords =
                Self::keywords_for_groups(&bucket_groups, heuristics.keyword_limit);
            let confidence = Self::skill_candidate_confidence(
                &bucket_groups,
                continuity_keys.len(),
                total_memories,
            );
            let source_episode_ids = bucket_groups
                .iter()
                .map(|group| group.episode_id)
                .collect::<Vec<_>>();
            let source_fixtures = bucket_groups
                .iter()
                .map(|group| group.fixture_name.clone())
                .collect::<Vec<_>>();
            let mut source_memory_ids = bucket_groups
                .iter()
                .flat_map(|group| group.lineage.source_memory_ids.iter().copied())
                .collect::<Vec<_>>();
            source_memory_ids.sort_by_key(|id| id.0);
            source_memory_ids.dedup_by_key(|id| id.0);
            let candidate_fixture = format!(
                "skill_candidate_{}_{}_{}_{}",
                primary_reason,
                goal_context.unwrap_or("unguided").replace(' ', "_"),
                bucket_groups[0].episode_id.0,
                bucket_groups.last().unwrap().episode_id.0
            );
            let pattern_summary = format!(
                "tentative_pattern(kind={} episodes={} members={} goal_context={} tool_chain={} continuity={} keywords={})",
                source_set_kind.as_str(),
                source_episode_ids
                    .iter()
                    .map(|id| id.0.to_string())
                    .collect::<Vec<_>>()
                    .join(","),
                total_memories,
                goal_context.unwrap_or("absent"),
                tool_chain_context.unwrap_or("absent"),
                continuity_keys.join(","),
                dominant_keywords.join(",")
            );
            candidates.push(SkillCandidatePattern {
                candidate_fixture,
                source_set_kind,
                source_episode_ids,
                source_fixtures,
                source_memory_ids,
                member_count: total_memories,
                continuity_keys,
                dominant_keywords,
                goal_context,
                tool_chain_context,
                confidence,
                status: "tentative",
                pattern_summary,
            });
        }

        candidates.sort_by(|left, right| {
            right
                .confidence
                .cmp(&left.confidence)
                .then_with(|| left.candidate_fixture.cmp(&right.candidate_fixture))
        });
        rejections.sort_by(|left, right| left.cluster_fixture.cmp(&right.cluster_fixture));
        let sample_logs = candidates
            .iter()
            .map(Self::skill_candidate_log_line)
            .chain(rejections.iter().map(Self::skill_rejection_log_line))
            .collect();

        SkillCandidateReport {
            candidates,
            rejections,
            heuristics_summary: Self::skill_candidate_heuristics_summary(heuristics),
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
        let source_citations = members
            .iter()
            .map(|candidate| EpisodeSourceCitation {
                memory_id: candidate.memory_id,
                timestamp_ms: candidate.timestamp_ms,
            })
            .collect();

        SourceGroup {
            episode_id,
            fixture_name,
            source_set_kind,
            members: members.iter().map(|c| c.memory_id).collect(),
            lineage: EpisodeLineage {
                source_memory_ids,
                continuity_keys,
                source_citations,
            },
            explain: EpisodeFormationExplain {
                primary_reason,
                start_timestamp_ms: first.timestamp_ms,
                end_timestamp_ms: last.timestamp_ms,
                time_span_ms,
                matching_fields,
                common_goal_context: members
                    .iter()
                    .all(|m| m.goal_context == first.goal_context && m.goal_context.is_some())
                    .then_some(first.goal_context)
                    .flatten(),
                common_tool_chain_context: members
                    .iter()
                    .all(|m| {
                        m.tool_chain_context == first.tool_chain_context
                            && m.tool_chain_context.is_some()
                    })
                    .then_some(first.tool_chain_context)
                    .flatten(),
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

    fn skill_candidate_heuristics_summary(heuristics: &SkillCandidateHeuristics) -> String {
        format!(
            "min_source_groups={} min_total_memories={} min_supporting_continuities={} require_context_anchor={} keyword_limit={}",
            heuristics.min_source_groups,
            heuristics.min_total_memories,
            heuristics.min_supporting_continuities,
            heuristics.require_context_anchor,
            heuristics.keyword_limit,
        )
    }

    fn shared_continuity_keys(groups: &[&SourceGroup]) -> Vec<&'static str> {
        let Some(first) = groups.first() else {
            return Vec::new();
        };

        let mut shared = vec![first.explain.primary_reason];
        if first.explain.common_goal_context.is_some()
            && groups
                .iter()
                .all(|group| group.explain.common_goal_context == first.explain.common_goal_context)
        {
            shared.push("goal_context");
        }
        if first.explain.common_tool_chain_context.is_some()
            && groups.iter().all(|group| {
                group.explain.common_tool_chain_context == first.explain.common_tool_chain_context
            })
        {
            shared.push("tool_chain_context");
        }
        if groups
            .iter()
            .all(|group| group.explain.matching_fields.contains(&"entity_overlap"))
        {
            shared.push("entity_overlap");
        }
        if groups
            .iter()
            .all(|group| group.explain.matching_fields.contains(&"failure_retry"))
        {
            shared.push("failure_retry");
        }
        shared
    }

    fn keywords_for_groups(groups: &[&SourceGroup], keyword_limit: usize) -> Vec<String> {
        let mut counts: HashMap<String, usize> = HashMap::new();
        for group in groups {
            for raw in [
                Some(group.source_set_kind.as_str()),
                group.explain.common_goal_context,
                group.explain.common_tool_chain_context,
            ]
            .into_iter()
            .flatten()
            {
                for token in raw.split(|ch: char| !ch.is_ascii_alphanumeric()) {
                    let token = token.to_ascii_lowercase();
                    if token.len() >= 4 {
                        *counts.entry(token).or_default() += 1;
                    }
                }
            }
        }

        let mut ranked = counts.into_iter().collect::<Vec<_>>();
        ranked.sort_by(|left, right| right.1.cmp(&left.1).then_with(|| left.0.cmp(&right.0)));
        ranked
            .into_iter()
            .take(keyword_limit.max(1))
            .map(|(token, _)| token)
            .collect()
    }

    fn skill_candidate_confidence(
        groups: &[&SourceGroup],
        shared_continuities: usize,
        total_memories: usize,
    ) -> u16 {
        let base = 420u16;
        let repetition_bonus = ((groups.len().saturating_sub(1)) as u16).saturating_mul(110);
        let support_bonus = (total_memories.saturating_sub(2) as u16).saturating_mul(35);
        let continuity_bonus = (shared_continuities as u16).saturating_mul(40);
        base.saturating_add(repetition_bonus)
            .saturating_add(support_bonus)
            .saturating_add(continuity_bonus)
            .min(950)
    }

    fn skill_candidate_log_line(candidate: &SkillCandidatePattern) -> String {
        format!(
            "candidate={} kind={} episodes={:?} members={} status={} confidence={} goal_context={} tool_chain={} continuity={:?} keywords={:?}",
            candidate.candidate_fixture,
            candidate.source_set_kind.as_str(),
            candidate
                .source_episode_ids
                .iter()
                .map(|id| id.0)
                .collect::<Vec<_>>(),
            candidate.member_count,
            candidate.status,
            candidate.confidence,
            candidate.goal_context.unwrap_or("absent"),
            candidate.tool_chain_context.unwrap_or("absent"),
            candidate.continuity_keys,
            candidate.dominant_keywords,
        )
    }

    fn skill_rejection_log_line(rejection: &SkillCandidateRejection) -> String {
        format!(
            "rejected_fixture={} kind={} episodes={:?} members={} reason={} keywords={:?}",
            rejection.cluster_fixture,
            rejection.source_set_kind.as_str(),
            rejection
                .source_episode_ids
                .iter()
                .map(|id| id.0)
                .collect::<Vec<_>>(),
            rejection.member_count,
            rejection.reason,
            rejection.dominant_keywords,
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
        assert_eq!(groups[0].explain.start_timestamp_ms, 1000);
        assert_eq!(groups[0].explain.end_timestamp_ms, 1000);
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
            report.groups[0].lineage.source_citations,
            vec![
                EpisodeSourceCitation {
                    memory_id: MemoryId(1),
                    timestamp_ms: 1000,
                },
                EpisodeSourceCitation {
                    memory_id: MemoryId(2),
                    timestamp_ms: 1100,
                }
            ]
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

    #[test]
    fn skill_candidate_analysis_detects_repeated_context_anchored_patterns() {
        let engine = EpisodeGroupingModule;
        let mut first = cand(1, 1_000, Some("task-A"), Some(10), vec![100, 101]);
        first.goal_context = Some("capture source-set continuity");
        first.tool_chain_context = Some("maintenance.compactor");
        let mut second = cand(2, 1_100, Some("task-A"), Some(10), vec![101, 102]);
        second.goal_context = Some("capture source-set continuity");
        second.tool_chain_context = Some("maintenance.compactor");
        let mut third = cand(3, 4_000_000, Some("task-B"), Some(11), vec![200, 201]);
        third.goal_context = Some("capture source-set continuity");
        third.tool_chain_context = Some("maintenance.compactor");
        let mut fourth = cand(4, 4_000_100, Some("task-B"), Some(11), vec![201, 202]);
        fourth.goal_context = Some("capture source-set continuity");
        fourth.tool_chain_context = Some("maintenance.compactor");

        let groups = engine.form_episodes(
            &GroupingHeuristics::default(),
            &[first, second, third, fourth],
        );
        let report = engine.analyze_skill_candidates(&groups, &SkillCandidateHeuristics::default());

        assert_eq!(groups.len(), 2);
        assert_eq!(report.candidates.len(), 1);
        assert!(report.rejections.is_empty());
        assert_eq!(
            report.heuristics_summary,
            "min_source_groups=2 min_total_memories=4 min_supporting_continuities=1 require_context_anchor=true keyword_limit=6"
        );
        assert_eq!(
            report.candidates[0].candidate_fixture,
            "skill_candidate_session_cluster_capture_source-set_continuity_1_2"
        );
        assert_eq!(
            report.candidates[0].source_set_kind,
            SourceSetKind::SessionCluster
        );
        assert_eq!(
            report.candidates[0].source_episode_ids,
            vec![EpisodeId(1), EpisodeId(2)]
        );
        assert_eq!(
            report.candidates[0].source_memory_ids,
            vec![MemoryId(1), MemoryId(2), MemoryId(3), MemoryId(4)]
        );
        assert_eq!(report.candidates[0].member_count, 4);
        assert_eq!(
            report.candidates[0].continuity_keys,
            vec![
                "session_cluster",
                "goal_context",
                "tool_chain_context",
                "entity_overlap"
            ]
        );
        assert_eq!(
            report.candidates[0].goal_context,
            Some("capture source-set continuity")
        );
        assert_eq!(
            report.candidates[0].tool_chain_context,
            Some("maintenance.compactor")
        );
        assert_eq!(report.candidates[0].status, "tentative");
        assert_eq!(report.candidates[0].confidence, 760);
        assert!(report.candidates[0]
            .dominant_keywords
            .contains(&"capture".to_string()));
        assert!(report.candidates[0]
            .dominant_keywords
            .contains(&"continuity".to_string()));
        assert!(report.candidates[0]
            .dominant_keywords
            .contains(&"maintenance".to_string()));
        assert!(report.sample_logs[0].contains(
            "candidate=skill_candidate_session_cluster_capture_source-set_continuity_1_2"
        ));
    }

    #[test]
    fn skill_candidate_analysis_rejects_clusters_without_repetition_or_context_anchor() {
        let engine = EpisodeGroupingModule;
        let mut first = cand(1, 1_000, Some("task-A"), None, vec![100, 101]);
        first.goal_context = Some("capture source-set continuity");
        let mut second = cand(2, 1_100, Some("task-A"), None, vec![101, 102]);
        second.goal_context = Some("capture source-set continuity");
        let third = cand(3, 4_000_000, Some("task-B"), None, vec![300, 301]);
        let fourth = cand(4, 4_000_100, Some("task-B"), None, vec![301, 302]);

        let groups = engine.form_episodes(
            &GroupingHeuristics::default(),
            &[first, second, third, fourth],
        );
        let report = engine.analyze_skill_candidates(&groups, &SkillCandidateHeuristics::default());

        assert!(report.candidates.is_empty());
        assert_eq!(report.rejections.len(), 2);
        assert!(report
            .rejections
            .iter()
            .any(|rejection| rejection.reason == "insufficient_cluster_repetition"));
        assert!(report
            .rejections
            .iter()
            .any(|rejection| rejection.reason == "missing_context_anchor"));
        assert!(report
            .sample_logs
            .iter()
            .any(|log| log.contains("reason=insufficient_cluster_repetition")));
        assert!(report
            .sample_logs
            .iter()
            .any(|log| log.contains("reason=missing_context_anchor")));
    }
}
