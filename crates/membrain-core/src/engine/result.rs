//! Retrieval result envelope, explainability, and packaging contract.
//!
//! Owns the canonical retrieval result shape that `recall → ranking → packaging`
//! produces before any wrapper formats it for CLI, daemon, or MCP output.

use crate::api::NamespaceId;
use crate::engine::contradiction::ContradictionExplain;
use crate::engine::ranking::{RankingExplain, RankingResult};
use crate::engine::recall::{RecallPlan, RecallPlanKind};
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

// ── Packaging & Sub-Summaries ────────────────────────────────────────────────

/// Omitted result summary (filtered by policy or threshold).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OmissionSummary {
    pub policy_redacted: usize,
    pub threshold_dropped: usize,
    pub dedup_dropped: usize,
}

/// Policy evaluation summary.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PolicySummary {
    pub namespace_applied: NamespaceId,
    pub redactions_applied: bool,
    pub restrictions_active: Vec<&'static str>,
}

/// Freshness markers for the result set.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FreshnessMarkers {
    pub oldest_item_days: u32,
    pub newest_item_days: u32,
    pub volatile_items_included: bool,
}

/// Provenance trace for an item.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProvenanceSummary {
    pub source_agent: &'static str,
    pub original_namespace: NamespaceId,
    pub derived_from: Option<MemoryId>,
}

/// Bounded evidence item in the evidence pack.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EvidenceItem {
    pub result: RetrievalResult,
    pub provenance: ProvenanceSummary,
    pub omitted_fields: Vec<&'static str>,
}

/// Action artifact linking back to evidence.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ActionArtifact {
    pub action_type: &'static str,
    pub suggestion: String,
    pub supporting_evidence: Vec<MemoryId>,
    pub confidence_score: u16,
}

/// Operating mode for dual memory output.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum DualOutputMode {
    Strict,
    Balanced,
    Fast,
}

// ── Result set ───────────────────────────────────────────────────────────────

/// Packaged result set returned by the retrieval pipeline.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RetrievalResultSet {
    /// Bounded evidence items (what the system knows).
    pub evidence_pack: Vec<EvidenceItem>,
    /// Derived action suggestions (what the system suggests doing).
    pub action_pack: Option<Vec<ActionArtifact>>,
    /// Explain summary for the entire retrieval operation.
    pub explain: RetrievalExplain,
    /// Policy evaluation context.
    pub policy_summary: PolicySummary,
    /// Omission statistics.
    pub omitted_summary: OmissionSummary,
    /// Freshness state.
    pub freshness_markers: FreshnessMarkers,
    /// Output mode applied.
    pub output_mode: DualOutputMode,
    /// Whether the result set was truncated by budget or time limits.
    pub truncated: bool,
    /// Total candidates considered before ranking.
    pub total_candidates: usize,
}


impl RetrievalResultSet {
    /// Builds an empty result set (no matches found).
    pub fn empty(explain: RetrievalExplain, namespace: NamespaceId) -> Self {
        Self {
            evidence_pack: Vec::new(),
            action_pack: None,
            explain,
            policy_summary: PolicySummary {
                namespace_applied: namespace,
                redactions_applied: false,
                restrictions_active: Vec::new(),
            },
            omitted_summary: OmissionSummary {
                policy_redacted: 0,
                threshold_dropped: 0,
                dedup_dropped: 0,
            },
            freshness_markers: FreshnessMarkers {
                oldest_item_days: 0,
                newest_item_days: 0,
                volatile_items_included: false,
            },
            output_mode: DualOutputMode::Balanced,
            truncated: false,
            total_candidates: 0,
        }
    }

    /// Returns the top evidence item if any exist.
    pub fn top(&self) -> Option<&EvidenceItem> {
        self.evidence_pack.first()
    }

    /// Returns how many evidence items were returned.
    pub fn count(&self) -> usize {
        self.evidence_pack.len()
    }

    /// Returns whether any evidence items had contradictions.
    pub fn has_contradictions(&self) -> bool {
        self.evidence_pack
            .iter()
            .any(|e| !e.result.contradiction_explains.is_empty())
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
    evidence_pack: Vec<EvidenceItem>,
    pub action_pack: Option<Vec<ActionArtifact>>,
    max_results: usize,
    total_candidates: usize,
    namespace_applied: NamespaceId,
}

impl ResultBuilder {
    /// Creates a new builder with a maximum result count.
    pub fn new(max_results: usize, namespace_applied: NamespaceId) -> Self {
        Self {
            evidence_pack: Vec::new(),
            action_pack: None,
            max_results,
            total_candidates: 0,
            namespace_applied,
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
            namespace: namespace.clone(),
            session_id,
            memory_type,
            compact_text,
            score: ranking_result.final_score,
            ranking_explain: RankingExplain::from_result(ranking_result),
            contradiction_explains: ranking_result.contradiction_explains.clone(),
            answered_from,
        };
        
        // Basic provenance for now
        let provenance = ProvenanceSummary {
            source_agent: "core_engine",
            original_namespace: namespace,
            derived_from: None,
        };

        self.evidence_pack.push(EvidenceItem {
            result,
            provenance,
            omitted_fields: Vec::new(),
        });
    }

    /// Builds the final result set, sorting by score and truncating.
    pub fn build(mut self, explain: RetrievalExplain) -> RetrievalResultSet {
        self.evidence_pack.sort_by(|a, b| b.result.score.cmp(&a.result.score));
        let truncated = self.evidence_pack.len() > self.max_results;
        self.evidence_pack.truncate(self.max_results);

        RetrievalResultSet {
            evidence_pack: self.evidence_pack,
            action_pack: self.action_pack,
            explain,
            policy_summary: PolicySummary {
                namespace_applied: self.namespace_applied,
                redactions_applied: false,
                restrictions_active: Vec::new(),
            },
            omitted_summary: OmissionSummary {
                policy_redacted: 0,
                threshold_dropped: 0,
                dedup_dropped: 0,
            },
            freshness_markers: FreshnessMarkers {
                oldest_item_days: 0,
                newest_item_days: 0,
                volatile_items_included: false,
            },
            output_mode: DualOutputMode::Balanced,
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
        let mut builder = ResultBuilder::new(2, ns("test"));

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
        assert!(result_set.evidence_pack[0].result.score >= result_set.evidence_pack[1].result.score);
        assert_eq!(result_set.evidence_pack[0].result.memory_id, MemoryId(2));
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

        let result_set = RetrievalResultSet::empty(explain, ns("test"));
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
