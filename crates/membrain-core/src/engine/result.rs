//! Retrieval result envelope, explainability, and packaging contract.
//!
//! Owns the canonical retrieval result shape that `recall → ranking → packaging`
//! produces before any wrapper formats it for CLI, daemon, or MCP output.

use crate::api::NamespaceId;
use crate::engine::contradiction::ContradictionExplain;
use crate::engine::ranking::{RankingExplain, RankingResult, ScoreFamily};
use crate::engine::recall::{RecallPlan, RecallPlanKind, RecallRouteSummary, Tier1PlanTrace};
use crate::types::{CanonicalMemoryType, MemoryId, SessionId};

// ── Result envelope ──────────────────────────────────────────────────────────

/// One ranked retrieval result ready for packaging into a response envelope.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RetrievalResult {
    /// Durable memory identity.
    pub memory_id: MemoryId,
    /// Namespace the memory belongs to.
    pub namespace: NamespaceId,
    /// Session the memory was ingested in.
    pub session_id: SessionId,
    /// Canonical memory family.
    pub memory_type: CanonicalMemoryType,
    /// Human-readable compact text.
    pub compact_text: String,
    /// Final ranking score (0..1000).
    pub score: u16,
    /// Full ranking explain payload.
    pub ranking_explain: RankingExplain,
    /// Contradiction explanations attached to this result.
    pub contradiction_explains: Vec<ContradictionExplain>,
    /// Tier that answered the query.
    pub answered_from: AnsweredFrom,
}

/// Which storage tier serviced this result.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum AnsweredFrom {
    Tier1Hot,
    Tier2Indexed,
    Tier3Cold,
}

impl AnsweredFrom {
    /// Stable machine-readable name.
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Tier1Hot => "tier1_hot",
            Self::Tier2Indexed => "tier2_indexed",
            Self::Tier3Cold => "tier3_cold",
        }
    }
}

// ── Result set ───────────────────────────────────────────────────────────────

/// Packaged result set returned by the retrieval pipeline.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RetrievalResultSet {
    /// Ordered results (highest score first).
    pub results: Vec<RetrievalResult>,
    /// Explain summary for the entire retrieval operation.
    pub explain: RetrievalExplain,
    /// Whether the result set was truncated by budget or time limits.
    pub truncated: bool,
    /// Total candidates considered before ranking.
    pub total_candidates: usize,
}

impl RetrievalResultSet {
    /// Builds an empty result set (no matches found).
    pub fn empty(explain: RetrievalExplain) -> Self {
        Self {
            results: Vec::new(),
            explain,
            truncated: false,
            total_candidates: 0,
        }
    }

    /// Returns the top result if any matches were found.
    pub fn top(&self) -> Option<&RetrievalResult> {
        self.results.first()
    }

    /// Returns how many results were returned.
    pub fn count(&self) -> usize {
        self.results.len()
    }

    /// Returns whether any results had contradictions.
    pub fn has_contradictions(&self) -> bool {
        self.results
            .iter()
            .any(|r| !r.contradiction_explains.is_empty())
    }
}

// ── Retrieval explain ────────────────────────────────────────────────────────

/// Top-level explain payload for the full retrieval operation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RetrievalExplain {
    /// Route plan chosen by the recall engine.
    pub recall_plan: RecallPlanKind,
    /// Route reason.
    pub route_reason: &'static str,
    /// Tiers consulted during retrieval.
    pub tiers_consulted: Vec<&'static str>,
    /// Whether Tier1 answered directly.
    pub tier1_answered_directly: bool,
    /// Candidate budget used.
    pub candidate_budget: usize,
    /// Time budget consumed (ms), if known.
    pub time_consumed_ms: Option<u32>,
    /// Ranking profile applied.
    pub ranking_profile: &'static str,
    /// Number of contradictions encountered.
    pub contradictions_found: usize,
}

impl RetrievalExplain {
    /// Builds an explain payload from a recall plan and ranking profile name.
    pub fn from_plan(plan: &RecallPlan, ranking_profile: &'static str) -> Self {
        let tiers_consulted: Vec<&'static str> = plan
            .route_summary
            .trace_stages
            .iter()
            .map(|stage| match stage {
                crate::engine::recall::RecallTraceStage::Tier1ExactHandle => "tier1_exact",
                crate::engine::recall::RecallTraceStage::Tier1RecentWindow => "tier1_recent",
                crate::engine::recall::RecallTraceStage::Tier2Exact => "tier2_exact",
                crate::engine::recall::RecallTraceStage::Tier3Fallback => "tier3_fallback",
            })
            .collect();

        Self {
            recall_plan: plan.kind,
            route_reason: plan.route_summary.reason,
            tiers_consulted,
            tier1_answered_directly: plan.route_summary.tier1_answers_directly,
            candidate_budget: plan.tier1_candidate_budget,
            time_consumed_ms: None,
            ranking_profile,
            contradictions_found: 0,
        }
    }
}

// ── Result builder ───────────────────────────────────────────────────────────

/// Builder for assembling retrieval results from ranked candidates.
#[derive(Debug, Clone)]
pub struct ResultBuilder {
    results: Vec<RetrievalResult>,
    max_results: usize,
    total_candidates: usize,
}

impl ResultBuilder {
    /// Creates a new builder with a maximum result count.
    pub fn new(max_results: usize) -> Self {
        Self {
            results: Vec::new(),
            max_results,
            total_candidates: 0,
        }
    }

    /// Adds a candidate to the result set.
    pub fn add(
        &mut self,
        memory_id: MemoryId,
        namespace: NamespaceId,
        session_id: SessionId,
        memory_type: CanonicalMemoryType,
        compact_text: String,
        ranking_result: &RankingResult,
        answered_from: AnsweredFrom,
    ) {
        self.total_candidates += 1;

        let result = RetrievalResult {
            memory_id,
            namespace,
            session_id,
            memory_type,
            compact_text,
            score: ranking_result.final_score,
            ranking_explain: RankingExplain::from_result(ranking_result),
            contradiction_explains: ranking_result.contradiction_explains.clone(),
            answered_from,
        };

        self.results.push(result);
    }

    /// Builds the final result set, sorting by score and truncating.
    pub fn build(mut self, explain: RetrievalExplain) -> RetrievalResultSet {
        self.results.sort_by(|a, b| b.score.cmp(&a.score));
        let truncated = self.results.len() > self.max_results;
        self.results.truncate(self.max_results);

        RetrievalResultSet {
            results: self.results,
            explain,
            truncated,
            total_candidates: self.total_candidates,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::engine::ranking::{fuse_scores, RankingInput, RankingProfile};

    fn ns(s: &str) -> NamespaceId {
        NamespaceId::new(s).unwrap()
    }

    #[test]
    fn result_builder_sorts_by_score_and_truncates() {
        let mut builder = ResultBuilder::new(2);

        let low = fuse_scores(
            RankingInput {
                recency: 100,
                salience: 100,
                relevance: 100,
                conflict: 500,
                access: 100,
            },
            RankingProfile::balanced(),
        );
        let mid = fuse_scores(
            RankingInput {
                recency: 500,
                salience: 500,
                relevance: 500,
                conflict: 500,
                access: 500,
            },
            RankingProfile::balanced(),
        );
        let high = fuse_scores(
            RankingInput {
                recency: 900,
                salience: 900,
                relevance: 900,
                conflict: 500,
                access: 900,
            },
            RankingProfile::balanced(),
        );

        builder.add(
            MemoryId(1), ns("test"), SessionId(1),
            CanonicalMemoryType::Event, "low".into(), &low, AnsweredFrom::Tier1Hot,
        );
        builder.add(
            MemoryId(2), ns("test"), SessionId(1),
            CanonicalMemoryType::Observation, "high".into(), &high, AnsweredFrom::Tier1Hot,
        );
        builder.add(
            MemoryId(3), ns("test"), SessionId(1),
            CanonicalMemoryType::ToolOutcome, "mid".into(), &mid, AnsweredFrom::Tier2Indexed,
        );

        let explain = RetrievalExplain {
            recall_plan: RecallPlanKind::Tier2ExactThenTier3Fallback,
            route_reason: "test",
            tiers_consulted: vec!["tier1_exact"],
            tier1_answered_directly: false,
            candidate_budget: 10,
            time_consumed_ms: None,
            ranking_profile: "balanced",
            contradictions_found: 0,
        };

        let result_set = builder.build(explain);

        assert_eq!(result_set.count(), 2);
        assert!(result_set.truncated);
        assert_eq!(result_set.total_candidates, 3);
        // Highest score first
        assert!(result_set.results[0].score >= result_set.results[1].score);
        assert_eq!(result_set.results[0].memory_id, MemoryId(2));
    }

    #[test]
    fn empty_result_set() {
        let explain = RetrievalExplain {
            recall_plan: RecallPlanKind::ExactIdTier1,
            route_reason: "no match",
            tiers_consulted: vec!["tier1_exact"],
            tier1_answered_directly: true,
            candidate_budget: 10,
            time_consumed_ms: Some(5),
            ranking_profile: "balanced",
            contradictions_found: 0,
        };

        let result_set = RetrievalResultSet::empty(explain);
        assert_eq!(result_set.count(), 0);
        assert!(!result_set.truncated);
        assert!(!result_set.has_contradictions());
        assert!(result_set.top().is_none());
    }

    #[test]
    fn answered_from_names() {
        assert_eq!(AnsweredFrom::Tier1Hot.as_str(), "tier1_hot");
        assert_eq!(AnsweredFrom::Tier2Indexed.as_str(), "tier2_indexed");
        assert_eq!(AnsweredFrom::Tier3Cold.as_str(), "tier3_cold");
    }
}
