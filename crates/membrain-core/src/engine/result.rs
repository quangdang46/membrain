//! Retrieval result envelope, explainability, and packaging contract.
//!
//! Owns the canonical retrieval result shape that `recall → ranking → packaging`
//! produces before any wrapper formats it for CLI, daemon, or MCP output.

use crate::api::{FieldPresence, NamespaceId, PolicyFilterSummary};
use crate::brain_store::PreparedTier2Layout;
use crate::engine::contradiction::{ContradictionExplain, ResolutionState};
use crate::engine::ranking::{RankingExplain, RankingResult, RerankMetadata};
use crate::engine::recall::{RecallPlan, RecallPlanKind, RecallTraceStage};
use crate::graph::{EntityId, RelationKind};
use crate::observability::OutcomeClass;
use crate::observability::{
    ExplainResultReason, ObservabilityModule, TracePolicySummary, TraceProvenanceSummary,
};
use crate::types::{CanonicalMemoryType, MemoryId, SessionId};
use serde::{Deserialize, Serialize};

// ── Result envelope ──────────────────────────────────────────────────────────

/// Which retrieval role an evidence item plays inside the bounded result set.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum EvidenceRole {
    Primary,
    Supporting,
}

impl EvidenceRole {
    /// Stable machine-readable name.
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Primary => "primary",
            Self::Supporting => "supporting",
        }
    }
}

/// Inline vs deferred payload state preserved across wrappers.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum PayloadState {
    Inline,
    PreviewOnly,
    Deferred,
    Redacted,
}

impl PayloadState {
    /// Stable machine-readable name.
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Inline => "inline",
            Self::PreviewOnly => "preview_only",
            Self::Deferred => "deferred",
            Self::Redacted => "redacted",
        }
    }
}

/// Lane by which an item entered the bounded shortlist.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum EntryLane {
    Exact,
    Recent,
    Lexical,
    Semantic,
    Graph,
    ColdFallback,
}

impl EntryLane {
    /// Stable machine-readable name.
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Exact => "exact",
            Self::Recent => "recent",
            Self::Lexical => "lexical",
            Self::Semantic => "semantic",
            Self::Graph => "graph",
            Self::ColdFallback => "cold_fallback",
        }
    }
}

/// Bounded score decomposition preserved for packaging and inspect surfaces.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ScoreSummary {
    pub final_score: u16,
    pub total_weighted_score: u32,
    pub signal_breakdown: Vec<(String, u16, u8, u32)>,
    pub profile: String,
}

impl ScoreSummary {
    /// Builds a bounded score summary from the ranking explain payload.
    pub fn from_ranking_explain(explain: &RankingExplain) -> Self {
        Self {
            final_score: explain.final_score,
            total_weighted_score: explain.total_weighted_score,
            signal_breakdown: explain
                .signal_breakdown
                .iter()
                .map(|(family, raw_value, weight, weighted_value)| {
                    (
                        family.as_str().to_string(),
                        *raw_value,
                        *weight,
                        *weighted_value,
                    )
                })
                .collect(),
            profile: explain.profile.clone(),
        }
    }
}

/// Bounded uncertainty markers attached to returned evidence.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct UncertaintyMarkers {
    pub confidence: u16,
    pub uncertainty_score: u16,
    pub freshness_uncertainty: Option<u16>,
    pub contradiction_uncertainty: Option<u16>,
    pub missing_evidence_uncertainty: Option<u16>,
}

/// Machine-readable conflict summary preserved for every returned item.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ConflictMarkers {
    pub conflict_state: ResolutionState,
    pub conflict_record_ids: Vec<u64>,
    pub belief_chain_id: Option<u64>,
    pub superseded_by: Option<MemoryId>,
    pub contradiction_lineage_pairs: Vec<[MemoryId; 2]>,
    pub resolution_reasons: Vec<String>,
    pub audit_visible_count: usize,
    pub omitted_conflict_siblings: usize,
}

/// One ranked retrieval result ready for packaging into a response envelope.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
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
    /// Primary/supporting role inside the evidence pack.
    pub role: EvidenceRole,
    /// Lane by which the item entered the bounded shortlist.
    pub entry_lane: EntryLane,
    /// Inline vs deferred payload state.
    pub payload_state: PayloadState,
    /// Final ranking score (0..1000).
    pub score: u16,
    /// Bounded score decomposition used for packaging and inspect surfaces.
    pub score_summary: ScoreSummary,
    /// Full ranking explain payload.
    pub ranking_explain: RankingExplain,
    /// Contradiction explanations attached to this result.
    pub contradiction_explains: Vec<ContradictionExplain>,
    /// Conflict-state summary required by inspect/explain surfaces.
    pub conflict_markers: ConflictMarkers,
    /// Uncertainty contribution summary for the item.
    pub uncertainty_markers: UncertaintyMarkers,
    /// Tier that answered the query.
    pub answered_from: AnsweredFrom,
    /// Bounded rerank metadata preserved for inspect/explain surfaces.
    pub rerank_metadata: RerankMetadata,
}

/// Which storage tier serviced this result.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
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

/// Omitted result summary (filtered by policy, threshold, or bounded packaging rules).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct OmissionSummary {
    pub policy_redacted: usize,
    pub threshold_dropped: usize,
    pub dedup_dropped: usize,
    pub budget_capped: usize,
    pub duplicate_collapsed: usize,
    pub low_confidence_suppressed: usize,
    pub stale_bypassed: usize,
}

/// Policy evaluation summary.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PolicySummary {
    pub namespace_applied: NamespaceId,
    pub outcome_class: OutcomeClass,
    pub redactions_applied: bool,
    pub restrictions_active: Vec<String>,
    pub filters: Vec<PolicyFilterSummary>,
}

/// Freshness markers for the result set.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FreshnessMarkers {
    pub oldest_item_days: u32,
    pub newest_item_days: u32,
    pub volatile_items_included: bool,
    pub stale_warning: bool,
    pub as_of_tick: Option<u64>,
}

/// Provenance trace for an item.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProvenanceSummary {
    pub source_kind: String,
    pub source_reference: String,
    pub source_agent: String,
    pub original_namespace: NamespaceId,
    pub derived_from: Option<MemoryId>,
    pub lineage_ancestors: Vec<MemoryId>,
    pub relation_to_seed: Option<RelationKind>,
    pub graph_seed: Option<EntityId>,
}

/// Bounded evidence item in the evidence pack.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct EvidenceItem {
    pub result: RetrievalResult,
    pub provenance_summary: ProvenanceSummary,
    pub freshness_markers: FreshnessMarkers,
    pub omitted_fields: Vec<String>,
}

/// Action artifact linking back to evidence.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ActionArtifact {
    pub action_type: String,
    pub suggestion: String,
    pub supporting_evidence: Vec<MemoryId>,
    pub confidence_score: u16,
}

/// Handle for a payload intentionally deferred past the final bounded cut.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DeferredPayload {
    pub memory_id: MemoryId,
    pub payload_state: PayloadState,
    pub reason: String,
    pub hydration_path: String,
}

/// Packaging facts preserved for downstream consumers.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PackagingMetadata {
    pub result_budget: usize,
    pub token_budget: Option<usize>,
    pub graph_assistance: String,
    pub degraded_summary: Option<String>,
    pub packaging_mode: String,
    pub rerank_metadata: Option<RerankMetadata>,
}

/// Operating mode for dual memory output.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum DualOutputMode {
    Strict,
    Balanced,
    Fast,
}

// ── Result set ───────────────────────────────────────────────────────────────

/// Packaged result set returned by the retrieval pipeline.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RetrievalResultSet {
    /// Final outcome class for the bounded retrieval path.
    pub outcome_class: OutcomeClass,
    /// Bounded evidence items (what the system knows).
    pub evidence_pack: Vec<EvidenceItem>,
    /// Derived action suggestions (what the system suggests doing).
    pub action_pack: Option<Vec<ActionArtifact>>,
    /// Payloads intentionally left deferred after the final cut.
    pub deferred_payloads: Vec<DeferredPayload>,
    /// Explain summary for the entire retrieval operation.
    pub explain: RetrievalExplain,
    /// Policy evaluation context.
    pub policy_summary: PolicySummary,
    /// Shared provenance summary for the packaged result set.
    pub provenance_summary: ProvenanceSummary,
    /// Omission statistics.
    pub omitted_summary: OmissionSummary,
    /// Freshness state.
    pub freshness_markers: FreshnessMarkers,
    /// Packaging facts preserved across transport wrappers.
    pub packaging_metadata: PackagingMetadata,
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
            outcome_class: OutcomeClass::Accepted,
            evidence_pack: Vec::new(),
            action_pack: None,
            deferred_payloads: Vec::new(),
            explain,
            policy_summary: PolicySummary {
                namespace_applied: namespace.clone(),
                outcome_class: OutcomeClass::Accepted,
                redactions_applied: false,
                restrictions_active: Vec::new(),
                filters: Vec::new(),
            },
            provenance_summary: ProvenanceSummary {
                source_kind: "retrieval_pipeline".to_string(),
                source_reference: "result_set".to_string(),
                source_agent: "core_engine".to_string(),
                original_namespace: namespace,
                derived_from: None,
                lineage_ancestors: Vec::new(),
                relation_to_seed: None,
                graph_seed: None,
            },
            omitted_summary: OmissionSummary {
                policy_redacted: 0,
                threshold_dropped: 0,
                dedup_dropped: 0,
                budget_capped: 0,
                duplicate_collapsed: 0,
                low_confidence_suppressed: 0,
                stale_bypassed: 0,
            },
            freshness_markers: FreshnessMarkers {
                oldest_item_days: 0,
                newest_item_days: 0,
                volatile_items_included: false,
                stale_warning: false,
                as_of_tick: None,
            },
            packaging_metadata: PackagingMetadata {
                result_budget: 0,
                token_budget: None,
                graph_assistance: "none".to_string(),
                degraded_summary: None,
                packaging_mode: "evidence_only".to_string(),
                rerank_metadata: None,
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

    /// Encodes the canonical retrieval result set to transport-stable JSON.
    pub fn to_json(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string(self)
    }

    /// Decodes transport JSON back into the canonical result set.
    pub fn from_json(json: &str) -> Result<Self, serde_json::Error> {
        serde_json::from_str(json)
    }
}

// ── Retrieval explain ────────────────────────────────────────────────────────

/// Why an item appeared or an alternative was omitted.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ResultReason {
    pub memory_id: Option<MemoryId>,
    pub reason_code: String,
    pub detail: String,
}

/// Top-level explain payload for the full retrieval operation.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RetrievalExplain {
    /// Route plan chosen by the recall engine.
    pub recall_plan: RecallPlanKind,
    /// Route reason.
    pub route_reason: String,
    /// Tiers consulted during retrieval.
    pub tiers_consulted: Vec<String>,
    /// Stable trace stages preserved for explain surfaces.
    pub trace_stages: Vec<RecallTraceStage>,
    /// Whether Tier1 answered directly.
    pub tier1_answered_directly: bool,
    /// Candidate budget used.
    pub candidate_budget: usize,
    /// Time budget consumed (ms), if known.
    pub time_consumed_ms: Option<u32>,
    /// Ranking profile applied.
    pub ranking_profile: String,
    /// Number of contradictions encountered.
    pub contradictions_found: usize,
    /// Why concrete results appeared or alternatives were omitted.
    pub result_reasons: Vec<ResultReason>,
}

impl RetrievalResultSet {
    /// Builds the shared route summary and trace stages from the canonical envelope.
    pub fn explain_route(
        &self,
    ) -> (
        crate::observability::RouteSummary,
        Vec<crate::observability::TraceStage>,
    ) {
        ObservabilityModule.explain_route(self)
    }

    /// Builds the shared result-reason family from the canonical envelope.
    pub fn explain_result_reasons(&self) -> Vec<ExplainResultReason> {
        ObservabilityModule.explain_result_reasons(self)
    }

    /// Builds the shared policy and provenance summaries from the canonical envelope.
    pub fn explain_policy_and_provenance(&self) -> (TracePolicySummary, TraceProvenanceSummary) {
        ObservabilityModule.explain_policy_and_provenance(self)
    }

    /// Builds the shared freshness, conflict, and uncertainty marker families.
    pub fn explain_markers(
        &self,
    ) -> (
        Vec<crate::observability::FreshnessMarker>,
        Vec<crate::observability::ConflictMarker>,
        Vec<crate::observability::UncertaintyMarker>,
    ) {
        ObservabilityModule.explain_markers(self)
    }
}

impl RetrievalExplain {
    /// Builds an explain payload from a recall plan and ranking profile name.
    pub fn from_plan(plan: &RecallPlan, ranking_profile: &'static str) -> Self {
        let tiers_consulted: Vec<String> = plan
            .route_summary
            .trace_stages
            .iter()
            .map(|stage| {
                match stage {
                    crate::engine::recall::RecallTraceStage::Tier1ExactHandle => "tier1_exact",
                    crate::engine::recall::RecallTraceStage::Tier1RecentWindow => "tier1_recent",
                    crate::engine::recall::RecallTraceStage::Tier2Exact => "tier2_exact",
                    crate::engine::recall::RecallTraceStage::Tier3Fallback => "tier3_fallback",
                }
                .to_string()
            })
            .collect();

        Self {
            recall_plan: plan.kind,
            route_reason: plan.route_summary.reason.to_string(),
            tiers_consulted,
            trace_stages: plan.route_summary.trace_stages.to_vec(),
            tier1_answered_directly: plan.route_summary.tier1_answers_directly,
            candidate_budget: plan.tier1_candidate_budget,
            time_consumed_ms: None,
            ranking_profile: ranking_profile.to_string(),
            contradictions_found: 0,
            result_reasons: Vec::new(),
        }
    }

    /// Appends bounded temporal-landmark explain reasons derived from a prepared Tier2 layout.
    pub fn push_temporal_landmark_reasons_from_prepared_layout(
        &mut self,
        prepared: &PreparedTier2Layout,
    ) {
        let memory_id = Some(prepared.layout.metadata.memory_id);
        let landmark = &prepared.layout.metadata.landmark;
        let metadata_detail = format!(
            "tier2 prefilter kept {} metadata candidate(s) and fetched {} payload(s) before the final cut",
            prepared.prefilter_trace.metadata_candidate_count,
            prepared.prefilter_trace.payload_fetch_count,
        );
        self.result_reasons.push(ResultReason {
            memory_id,
            reason_code: "temporal_prefilter_metadata_only".to_string(),
            detail: metadata_detail,
        });
        self.result_reasons.push(ResultReason {
            memory_id,
            reason_code: "temporal_payload_deferred".to_string(),
            detail: format!(
                "heavyweight Tier2 payload remained deferred until hydration path {}",
                prepared.layout.payload_hydration_path()
            ),
        });

        if landmark.is_landmark {
            let mut detail = match (&landmark.landmark_label, &landmark.era_id) {
                (Some(label), Some(era_id)) => {
                    format!("landmark \"{label}\" opened era \"{era_id}\"")
                }
                (Some(label), None) => {
                    format!("landmark \"{label}\" remained active without opening a new era")
                }
                (None, Some(era_id)) => format!("landmark opened era \"{era_id}\""),
                (None, None) => "memory remained an unlabeled landmark".to_string(),
            };
            if prepared.prefilter_stays_metadata_only() {
                detail.push_str(" while staying on metadata-only Tier2 planning");
            }
            self.result_reasons.push(ResultReason {
                memory_id,
                reason_code: "temporal_landmark_selected".to_string(),
                detail,
            });
        } else {
            self.result_reasons.push(ResultReason {
                memory_id,
                reason_code: "temporal_landmark_not_selected".to_string(),
                detail: "memory stayed recallable without landmark promotion or era creation"
                    .to_string(),
            });
        }
    }
}

// ── Result builder ───────────────────────────────────────────────────────────

/// Builder for assembling retrieval results from ranked candidates.
#[derive(Debug, Clone)]
pub struct ResultBuilder {
    evidence_pack: Vec<EvidenceItem>,
    deferred_payloads: Vec<DeferredPayload>,
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
            deferred_payloads: Vec::new(),
            action_pack: None,
            max_results,
            total_candidates: 0,
            namespace_applied,
        }
    }

    /// Adds a candidate to the result set.
    #[allow(clippy::too_many_arguments)]
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

        let ranking_explain = RankingExplain::from_result(ranking_result);
        let result = RetrievalResult {
            memory_id,
            namespace: namespace.clone(),
            session_id,
            memory_type,
            compact_text,
            role: EvidenceRole::Primary,
            entry_lane: match answered_from {
                AnsweredFrom::Tier1Hot => EntryLane::Exact,
                AnsweredFrom::Tier2Indexed => EntryLane::Semantic,
                AnsweredFrom::Tier3Cold => EntryLane::ColdFallback,
            },
            payload_state: PayloadState::Inline,
            score: ranking_result.final_score,
            score_summary: ScoreSummary::from_ranking_explain(&ranking_explain),
            ranking_explain,
            contradiction_explains: ranking_result.contradiction_explains.clone(),
            conflict_markers: ConflictMarkers {
                conflict_state: ranking_result
                    .contradiction_explains
                    .iter()
                    .map(|explain| explain.resolution)
                    .find(|state| *state != ResolutionState::None)
                    .unwrap_or(ResolutionState::None),
                conflict_record_ids: ranking_result
                    .contradiction_explains
                    .iter()
                    .map(|explain| explain.contradiction_id.0)
                    .collect(),
                belief_chain_id: None,
                superseded_by: ranking_result
                    .contradiction_explains
                    .iter()
                    .find_map(|explain| {
                        (explain.superseded_memory == Some(memory_id)).then_some(
                            explain
                                .preferred_memory
                                .unwrap_or(explain.conflicting_memory),
                        )
                    }),
                contradiction_lineage_pairs: ranking_result
                    .contradiction_explains
                    .iter()
                    .map(|explain| explain.lineage_pair)
                    .collect(),
                resolution_reasons: ranking_result
                    .contradiction_explains
                    .iter()
                    .filter_map(|explain| explain.resolution_reason.clone())
                    .collect(),
                audit_visible_count: ranking_result
                    .contradiction_explains
                    .iter()
                    .filter(|explain| explain.audit_visible)
                    .count(),
                omitted_conflict_siblings: 0,
            },
            uncertainty_markers: UncertaintyMarkers {
                confidence: ranking_result.final_score,
                uncertainty_score: 1000u16.saturating_sub(ranking_result.final_score),
                freshness_uncertainty: None,
                contradiction_uncertainty: ranking_result
                    .contradiction_explains
                    .iter()
                    .map(|explain| 1000u16.saturating_sub(explain.confidence_signal))
                    .max(),
                missing_evidence_uncertainty: None,
            },
            answered_from,
            rerank_metadata: ranking_result.rerank_metadata.clone(),
        };

        let provenance_summary = ProvenanceSummary {
            source_kind: "memory".to_string(),
            source_reference: "memory_id".to_string(),
            source_agent: "core_engine".to_string(),
            original_namespace: namespace,
            derived_from: None,
            lineage_ancestors: Vec::new(),
            relation_to_seed: None,
            graph_seed: None,
        };

        self.evidence_pack.push(EvidenceItem {
            result,
            provenance_summary,
            freshness_markers: FreshnessMarkers {
                oldest_item_days: 0,
                newest_item_days: 0,
                volatile_items_included: false,
                stale_warning: false,
                as_of_tick: None,
            },
            omitted_fields: Vec::new(),
        });
    }

    /// Builds the final result set, sorting by score and truncating.
    pub fn build(mut self, explain: RetrievalExplain) -> RetrievalResultSet {
        self.evidence_pack
            .sort_by(|a, b| b.result.score.cmp(&a.result.score));
        let truncated = self.evidence_pack.len() > self.max_results;
        let outcome_class = if truncated {
            OutcomeClass::Partial
        } else {
            OutcomeClass::Accepted
        };
        let packaging_mode = if self.action_pack.is_some() {
            "evidence_plus_action"
        } else {
            "evidence_only"
        };
        self.evidence_pack.truncate(self.max_results);
        let rerank_metadata = self.evidence_pack.first().and_then(|first| {
            self.evidence_pack
                .iter()
                .all(|item| item.result.rerank_metadata == first.result.rerank_metadata)
                .then(|| first.result.rerank_metadata.clone())
        });

        let contradictions_found = self
            .evidence_pack
            .iter()
            .map(|item| item.result.contradiction_explains.len())
            .sum();

        RetrievalResultSet {
            outcome_class,
            evidence_pack: self.evidence_pack,
            action_pack: self.action_pack,
            deferred_payloads: self.deferred_payloads,
            explain: RetrievalExplain {
                contradictions_found,
                ..explain
            },
            policy_summary: PolicySummary {
                namespace_applied: self.namespace_applied.clone(),
                outcome_class,
                redactions_applied: false,
                restrictions_active: Vec::new(),
                filters: vec![PolicyFilterSummary::new(
                    self.namespace_applied.as_str(),
                    "namespace",
                    OutcomeClass::Accepted,
                    "none",
                    FieldPresence::Present("namespace_bound".to_string()),
                    FieldPresence::Absent,
                    Vec::new(),
                )],
            },
            provenance_summary: ProvenanceSummary {
                source_kind: "retrieval_pipeline".to_string(),
                source_reference: "result_builder".to_string(),
                source_agent: "core_engine".to_string(),
                original_namespace: self.namespace_applied.clone(),
                derived_from: None,
                lineage_ancestors: Vec::new(),
                relation_to_seed: None,
                graph_seed: None,
            },
            omitted_summary: OmissionSummary {
                policy_redacted: 0,
                threshold_dropped: 0,
                dedup_dropped: 0,
                budget_capped: usize::from(truncated),
                duplicate_collapsed: 0,
                low_confidence_suppressed: 0,
                stale_bypassed: 0,
            },
            freshness_markers: FreshnessMarkers {
                oldest_item_days: 0,
                newest_item_days: 0,
                volatile_items_included: false,
                stale_warning: false,
                as_of_tick: None,
            },
            packaging_metadata: PackagingMetadata {
                result_budget: self.max_results,
                token_budget: None,
                graph_assistance: "none".to_string(),
                degraded_summary: None,
                packaging_mode: packaging_mode.to_string(),
                rerank_metadata,
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
    use crate::brain_store::BrainStore;
    use crate::config::RuntimeConfig;
    use crate::engine::contradiction::{
        ContradictionExplain, ContradictionId, ContradictionKind, PreferredAnswerState,
    };
    use crate::engine::ranking::{fuse_scores, RankingInput, RankingProfile};
    use crate::engine::recall::{RecallEngine, RecallRequest, RecallRuntime};
    use crate::types::{LandmarkSignals, RawEncodeInput, RawIntakeKind};

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
                strength: 100,
                provenance: 100,
                conflict: 500,
            },
            RankingProfile::balanced(),
        );
        let mid = fuse_scores(
            RankingInput {
                recency: 500,
                salience: 500,
                strength: 500,
                provenance: 500,
                conflict: 500,
            },
            RankingProfile::balanced(),
        );
        let high = fuse_scores(
            RankingInput {
                recency: 900,
                salience: 900,
                strength: 900,
                provenance: 900,
                conflict: 500,
            },
            RankingProfile::balanced(),
        );

        builder.add(
            MemoryId(1),
            ns("test"),
            SessionId(1),
            CanonicalMemoryType::Event,
            "low".into(),
            &low,
            AnsweredFrom::Tier1Hot,
        );
        builder.add(
            MemoryId(2),
            ns("test"),
            SessionId(1),
            CanonicalMemoryType::Observation,
            "high".into(),
            &high,
            AnsweredFrom::Tier1Hot,
        );
        builder.add(
            MemoryId(3),
            ns("test"),
            SessionId(1),
            CanonicalMemoryType::ToolOutcome,
            "mid".into(),
            &mid,
            AnsweredFrom::Tier2Indexed,
        );

        let explain = RetrievalExplain {
            recall_plan: RecallPlanKind::Tier2ExactThenTier3Fallback,
            route_reason: "test".to_string(),
            tiers_consulted: vec!["tier1_exact".to_string()],
            trace_stages: vec![RecallTraceStage::Tier1ExactHandle],
            tier1_answered_directly: false,
            candidate_budget: 10,
            time_consumed_ms: None,
            ranking_profile: "balanced".to_string(),
            contradictions_found: 0,
            result_reasons: vec![ResultReason {
                memory_id: Some(MemoryId(2)),
                reason_code: "score_kept".to_string(),
                detail: "top-ranked result stayed within the bounded result cap".to_string(),
            }],
        };

        let result_set = builder.build(explain);

        assert_eq!(result_set.count(), 2);
        assert!(result_set.truncated);
        assert_eq!(result_set.outcome_class, OutcomeClass::Partial);
        assert_eq!(
            result_set.policy_summary.outcome_class,
            OutcomeClass::Partial
        );
        assert_eq!(result_set.total_candidates, 3);
        assert_eq!(result_set.omitted_summary.budget_capped, 1);
        assert_eq!(result_set.packaging_metadata.result_budget, 2);
        assert_eq!(
            result_set.packaging_metadata.packaging_mode,
            "evidence_only"
        );
        assert_eq!(
            result_set
                .packaging_metadata
                .rerank_metadata
                .as_ref()
                .map(|metadata| metadata.local_reranker_mode.as_str()),
            Some("disabled")
        );
        assert!(result_set.deferred_payloads.is_empty());
        // Highest score first
        assert!(
            result_set.evidence_pack[0].result.score >= result_set.evidence_pack[1].result.score
        );
        assert_eq!(result_set.evidence_pack[0].result.memory_id, MemoryId(2));
        assert_eq!(
            result_set.evidence_pack[0].result.entry_lane.as_str(),
            "exact"
        );
        assert_eq!(
            result_set.evidence_pack[0].result.score_summary.final_score,
            result_set.evidence_pack[0].result.score
        );
        assert_eq!(
            result_set.evidence_pack[0]
                .result
                .score_summary
                .total_weighted_score,
            result_set.evidence_pack[0]
                .result
                .ranking_explain
                .total_weighted_score
        );
        assert!(result_set.evidence_pack[0]
            .result
            .score_summary
            .signal_breakdown
            .iter()
            .all(|(_, _, _, weighted_value)| *weighted_value <= 1000));
        assert_eq!(
            result_set.policy_summary.filters[0].effective_namespace,
            "test".to_string()
        );
        assert_eq!(
            result_set.evidence_pack[0]
                .result
                .conflict_markers
                .conflict_state,
            ResolutionState::None
        );
        assert!(result_set.evidence_pack[0]
            .result
            .conflict_markers
            .conflict_record_ids
            .is_empty());
        assert!(result_set.evidence_pack[0]
            .result
            .conflict_markers
            .contradiction_lineage_pairs
            .is_empty());
        assert!(result_set.evidence_pack[0]
            .result
            .conflict_markers
            .resolution_reasons
            .is_empty());
        assert_eq!(
            result_set.evidence_pack[0]
                .result
                .conflict_markers
                .audit_visible_count,
            0
        );
        assert_eq!(
            result_set.evidence_pack[0]
                .result
                .uncertainty_markers
                .contradiction_uncertainty,
            None
        );
        assert_eq!(result_set.explain.contradictions_found, 0);
    }

    #[test]
    fn empty_result_set() {
        let explain = RetrievalExplain {
            recall_plan: RecallPlanKind::ExactIdTier1,
            route_reason: "no match".to_string(),
            tiers_consulted: vec!["tier1_exact".to_string()],
            trace_stages: vec![RecallTraceStage::Tier1ExactHandle],
            tier1_answered_directly: true,
            candidate_budget: 10,
            time_consumed_ms: Some(5),
            ranking_profile: "balanced".to_string(),
            contradictions_found: 0,
            result_reasons: vec![ResultReason {
                memory_id: None,
                reason_code: "no_match".to_string(),
                detail: "the bounded route produced no candidates".to_string(),
            }],
        };

        let result_set = RetrievalResultSet::empty(explain, ns("test"));
        assert_eq!(result_set.count(), 0);
        assert!(!result_set.truncated);
        assert!(!result_set.has_contradictions());
        assert!(result_set.top().is_none());
        assert!(result_set.deferred_payloads.is_empty());
        assert_eq!(
            result_set.packaging_metadata.packaging_mode,
            "evidence_only"
        );
        assert_eq!(result_set.omitted_summary.budget_capped, 0);
    }

    #[test]
    fn result_builder_suppresses_pack_level_rerank_metadata_when_retained_items_disagree() {
        let mut builder = ResultBuilder::new(2, ns("test"));
        let mut top_ranked = fuse_scores(
            RankingInput {
                recency: 900,
                salience: 850,
                strength: 800,
                provenance: 750,
                conflict: 0,
            },
            RankingProfile::balanced(),
        );
        top_ranked.rerank_metadata = crate::engine::ranking::RerankMetadata {
            float32_rescore_limit: 8,
            candidate_cut_limit: 4,
            local_reranker_mode: crate::engine::ranking::LocalRerankerMode::Bounded,
            local_reranker_applied: true,
            rerank_score_delta: 35,
        };

        let mut second_ranked = fuse_scores(
            RankingInput {
                recency: 820,
                salience: 810,
                strength: 790,
                provenance: 780,
                conflict: 0,
            },
            RankingProfile::balanced(),
        );
        second_ranked.rerank_metadata = crate::engine::ranking::RerankMetadata::float32_only(5, 5);

        builder.add(
            MemoryId(1),
            ns("test"),
            SessionId(1),
            CanonicalMemoryType::Event,
            "top ranked result".to_string(),
            &top_ranked,
            AnsweredFrom::Tier1Hot,
        );
        builder.add(
            MemoryId(2),
            ns("test"),
            SessionId(1),
            CanonicalMemoryType::Event,
            "second ranked result".to_string(),
            &second_ranked,
            AnsweredFrom::Tier2Indexed,
        );

        let plan = RecallEngine.plan_recall(
            RecallRequest::small_session_lookup(SessionId(1)),
            RuntimeConfig::default(),
        );
        let result_set = builder.build(RetrievalExplain::from_plan(&plan, "balanced"));

        assert_eq!(result_set.count(), 2);
        assert!(result_set.packaging_metadata.rerank_metadata.is_none());
        assert!(
            result_set.evidence_pack[0]
                .result
                .rerank_metadata
                .local_reranker_applied
        );
        assert_eq!(
            result_set.evidence_pack[1]
                .result
                .rerank_metadata
                .local_reranker_mode
                .as_str(),
            "disabled"
        );
    }

    #[test]
    fn result_builder_preserves_contradiction_audit_and_lineage_markers() {
        let mut builder = ResultBuilder::new(1, ns("test"));
        let mut ranked = fuse_scores(
            RankingInput {
                recency: 650,
                salience: 700,
                strength: 760,
                provenance: 900,
                conflict: 280,
            },
            RankingProfile::balanced(),
        );
        ranked.contradiction_explains = vec![ContradictionExplain {
            contradiction_id: ContradictionId(9),
            kind: ContradictionKind::Supersession,
            resolution: ResolutionState::ManuallyResolved,
            preferred_answer_state: PreferredAnswerState::Preferred,
            preferred_memory: Some(MemoryId(44)),
            confidence_signal: 830,
            conflicting_memory: MemoryId(12),
            lineage_pair: [MemoryId(12), MemoryId(44)],
            result_is_preferred: true,
            superseded_memory: Some(MemoryId(12)),
            resolution_reason: Some("manual review".to_string()),
            active_contradiction: false,
            archived: false,
            legal_hold: false,
            authoritative_evidence: true,
            retention_reason: None,
            audit_visible: true,
        }];

        builder.add(
            MemoryId(44),
            ns("test"),
            SessionId(3),
            CanonicalMemoryType::Event,
            "resolved contradiction".into(),
            &ranked,
            AnsweredFrom::Tier2Indexed,
        );

        let result_set = builder.build(RetrievalExplain {
            recall_plan: RecallPlanKind::Tier2ExactThenTier3Fallback,
            route_reason: "contradiction explain".to_string(),
            tiers_consulted: vec!["tier2_exact".to_string()],
            trace_stages: vec![RecallTraceStage::Tier2Exact],
            tier1_answered_directly: false,
            candidate_budget: 6,
            time_consumed_ms: Some(9),
            ranking_profile: "balanced".to_string(),
            contradictions_found: 0,
            result_reasons: vec![ResultReason {
                memory_id: Some(MemoryId(44)),
                reason_code: "contradiction_selected".to_string(),
                detail: "manual review kept the preferred branch visible".to_string(),
            }],
        });

        let markers = &result_set.evidence_pack[0].result.conflict_markers;
        assert_eq!(result_set.explain.contradictions_found, 1);
        assert_eq!(markers.conflict_state, ResolutionState::ManuallyResolved);
        assert_eq!(markers.conflict_record_ids, vec![9]);
        assert_eq!(markers.superseded_by, None);
        assert_eq!(
            markers.contradiction_lineage_pairs,
            vec![[MemoryId(12), MemoryId(44)]]
        );
        assert_eq!(
            markers.resolution_reasons,
            vec!["manual review".to_string()]
        );
        assert_eq!(markers.audit_visible_count, 1);
        assert_eq!(
            result_set.evidence_pack[0]
                .result
                .uncertainty_markers
                .contradiction_uncertainty,
            Some(170)
        );
    }

    #[test]
    fn result_builder_counts_legal_hold_contradictions_as_audit_visible() {
        let mut builder = ResultBuilder::new(1, ns("test"));
        let mut ranked = fuse_scores(
            RankingInput {
                recency: 650,
                salience: 700,
                strength: 760,
                provenance: 900,
                conflict: 280,
            },
            RankingProfile::balanced(),
        );
        ranked.contradiction_explains = vec![ContradictionExplain {
            contradiction_id: ContradictionId(13),
            kind: ContradictionKind::Supersession,
            resolution: ResolutionState::AuthoritativelyResolved,
            preferred_answer_state: PreferredAnswerState::Preferred,
            preferred_memory: Some(MemoryId(44)),
            confidence_signal: 980,
            conflicting_memory: MemoryId(12),
            lineage_pair: [MemoryId(12), MemoryId(44)],
            result_is_preferred: true,
            superseded_memory: Some(MemoryId(12)),
            resolution_reason: Some("authoritative archive".to_string()),
            active_contradiction: false,
            archived: true,
            legal_hold: true,
            authoritative_evidence: true,
            retention_reason: Some("legal hold keeps archived contradiction".to_string()),
            audit_visible: true,
        }];

        builder.add(
            MemoryId(44),
            ns("test"),
            SessionId(3),
            CanonicalMemoryType::Event,
            "archived contradiction under legal hold".into(),
            &ranked,
            AnsweredFrom::Tier2Indexed,
        );

        let result_set = builder.build(RetrievalExplain {
            recall_plan: RecallPlanKind::Tier2ExactThenTier3Fallback,
            route_reason: "contradiction archive explain".to_string(),
            tiers_consulted: vec!["tier2_exact".to_string()],
            trace_stages: vec![RecallTraceStage::Tier2Exact],
            tier1_answered_directly: false,
            candidate_budget: 6,
            time_consumed_ms: Some(9),
            ranking_profile: "balanced".to_string(),
            contradictions_found: 0,
            result_reasons: vec![ResultReason {
                memory_id: Some(MemoryId(44)),
                reason_code: "contradiction_retained_under_legal_hold".to_string(),
                detail: "legal hold keeps archived authoritative evidence visible".to_string(),
            }],
        });

        let markers = &result_set.evidence_pack[0].result.conflict_markers;
        assert_eq!(markers.audit_visible_count, 1);
        assert_eq!(
            markers.resolution_reasons,
            vec!["authoritative archive".to_string()]
        );
    }

    #[test]
    fn retrieval_result_set_round_trips_through_transport_json() {
        let mut builder = ResultBuilder::new(1, ns("test"));
        let ranked = fuse_scores(
            RankingInput {
                recency: 700,
                salience: 650,
                strength: 800,
                provenance: 600,
                conflict: 500,
            },
            RankingProfile::balanced(),
        );

        builder.add(
            MemoryId(7),
            ns("test"),
            SessionId(3),
            CanonicalMemoryType::Event,
            "round trip".into(),
            &ranked,
            AnsweredFrom::Tier2Indexed,
        );
        builder.deferred_payloads.push(DeferredPayload {
            memory_id: MemoryId(7),
            payload_state: PayloadState::Deferred,
            reason: "tier2 payload intentionally deferred past bounded result packaging"
                .to_string(),
            hydration_path: "tier2://test/payload/0007/7".to_string(),
        });

        let original = builder.build(RetrievalExplain {
            recall_plan: RecallPlanKind::Tier2ExactThenTier3Fallback,
            route_reason: "tier2 path".to_string(),
            tiers_consulted: vec!["tier1_recent".to_string(), "tier2_exact".to_string()],
            trace_stages: vec![
                RecallTraceStage::Tier1RecentWindow,
                RecallTraceStage::Tier2Exact,
            ],
            tier1_answered_directly: false,
            candidate_budget: 8,
            time_consumed_ms: Some(12),
            ranking_profile: "balanced".to_string(),
            contradictions_found: 0,
            result_reasons: vec![ResultReason {
                memory_id: Some(MemoryId(7)),
                reason_code: "tier2_exact_match".to_string(),
                detail: "candidate survived bounded ranking and packaging".to_string(),
            }],
        });

        let json = original.to_json().unwrap();
        let decoded = RetrievalResultSet::from_json(&json).unwrap();

        assert_eq!(decoded, original);
        assert_eq!(decoded.count(), 1);
        assert_eq!(decoded.top().unwrap().result.memory_id, MemoryId(7));
        assert_eq!(decoded.deferred_payloads.len(), 1);
        assert_eq!(decoded.deferred_payloads[0].memory_id, MemoryId(7));
        assert_eq!(
            decoded.deferred_payloads[0].payload_state,
            PayloadState::Deferred
        );
        assert_eq!(
            decoded.deferred_payloads[0].hydration_path,
            "tier2://test/payload/0007/7"
        );
        assert_eq!(
            decoded.explain.result_reasons[0].reason_code,
            "tier2_exact_match"
        );
    }

    #[test]
    fn result_builder_marks_losing_memory_with_superseding_winner() {
        let mut builder = ResultBuilder::new(1, ns("test"));
        let mut ranked = fuse_scores(
            RankingInput {
                recency: 620,
                salience: 700,
                strength: 710,
                provenance: 900,
                conflict: 260,
            },
            RankingProfile::balanced(),
        );
        ranked.contradiction_explains = vec![ContradictionExplain {
            contradiction_id: ContradictionId(11),
            kind: ContradictionKind::Supersession,
            resolution: ResolutionState::ManuallyResolved,
            preferred_answer_state: PreferredAnswerState::Preferred,
            preferred_memory: Some(MemoryId(44)),
            confidence_signal: 830,
            conflicting_memory: MemoryId(44),
            lineage_pair: [MemoryId(12), MemoryId(44)],
            result_is_preferred: false,
            superseded_memory: Some(MemoryId(12)),
            resolution_reason: Some("manual review".to_string()),
            active_contradiction: false,
            archived: false,
            legal_hold: false,
            authoritative_evidence: true,
            retention_reason: None,
            audit_visible: true,
        }];

        builder.add(
            MemoryId(12),
            ns("test"),
            SessionId(3),
            CanonicalMemoryType::Event,
            "superseded contradiction".into(),
            &ranked,
            AnsweredFrom::Tier2Indexed,
        );

        let result_set = builder.build(RetrievalExplain {
            recall_plan: RecallPlanKind::Tier2ExactThenTier3Fallback,
            route_reason: "contradiction explain".to_string(),
            tiers_consulted: vec!["tier2_exact".to_string()],
            trace_stages: vec![RecallTraceStage::Tier2Exact],
            tier1_answered_directly: false,
            candidate_budget: 6,
            time_consumed_ms: Some(9),
            ranking_profile: "balanced".to_string(),
            contradictions_found: 0,
            result_reasons: vec![ResultReason {
                memory_id: Some(MemoryId(12)),
                reason_code: "contradiction_visible".to_string(),
                detail: "manual review kept the losing branch inspectable".to_string(),
            }],
        });

        let markers = &result_set.evidence_pack[0].result.conflict_markers;
        assert_eq!(markers.superseded_by, Some(MemoryId(44)));
        assert_eq!(markers.conflict_record_ids, vec![11]);
        assert_eq!(
            markers.contradiction_lineage_pairs,
            vec![[MemoryId(12), MemoryId(44)]]
        );
    }

    #[test]
    fn retrieval_explain_adds_temporal_landmark_reasons_from_prepared_layout() {
        let store = BrainStore::new(RuntimeConfig::default());
        let prepared = store.prepare_tier2_layout_with_trace_from_encode(
            ns("timeline"),
            MemoryId(21),
            SessionId(8),
            RawEncodeInput::new(RawIntakeKind::Event, "Launch day")
                .with_landmark_signals(LandmarkSignals::new(0.95, 0.91, 0.12, 88)),
        );
        let mut explain = RetrievalExplain {
            recall_plan: RecallPlanKind::Tier2ExactThenTier3Fallback,
            route_reason: "temporal lookup".to_string(),
            tiers_consulted: vec!["tier2_exact".to_string()],
            trace_stages: vec![RecallTraceStage::Tier2Exact],
            tier1_answered_directly: false,
            candidate_budget: 4,
            time_consumed_ms: None,
            ranking_profile: "balanced".to_string(),
            contradictions_found: 0,
            result_reasons: Vec::new(),
        };

        explain.push_temporal_landmark_reasons_from_prepared_layout(&prepared);

        assert_eq!(explain.result_reasons.len(), 3);
        assert_eq!(explain.result_reasons[0].memory_id, Some(MemoryId(21)));
        assert_eq!(
            explain.result_reasons[0].reason_code,
            "temporal_prefilter_metadata_only"
        );
        assert!(explain.result_reasons[0]
            .detail
            .contains("fetched 0 payload(s)"));
        assert_eq!(
            explain.result_reasons[1].reason_code,
            "temporal_payload_deferred"
        );
        assert!(explain.result_reasons[1]
            .detail
            .contains("tier2://timeline/payload/0015/21"));
        assert_eq!(
            explain.result_reasons[2].reason_code,
            "temporal_landmark_selected"
        );
        assert!(explain.result_reasons[2]
            .detail
            .contains("metadata-only Tier2 planning"));
        assert!(explain.result_reasons[2].detail.contains("launch"));
    }

    #[test]
    fn retrieval_explain_marks_non_landmarks_without_era_creation() {
        let store = BrainStore::new(RuntimeConfig::default());
        let prepared = store.prepare_tier2_layout_with_trace_from_encode(
            ns("timeline"),
            MemoryId(34),
            SessionId(9),
            RawEncodeInput::new(RawIntakeKind::Observation, "Routine checkin"),
        );
        let mut explain = RetrievalExplain {
            recall_plan: RecallPlanKind::Tier2ExactThenTier3Fallback,
            route_reason: "temporal lookup".to_string(),
            tiers_consulted: vec!["tier2_exact".to_string()],
            trace_stages: vec![RecallTraceStage::Tier2Exact],
            tier1_answered_directly: false,
            candidate_budget: 4,
            time_consumed_ms: None,
            ranking_profile: "balanced".to_string(),
            contradictions_found: 0,
            result_reasons: Vec::new(),
        };

        explain.push_temporal_landmark_reasons_from_prepared_layout(&prepared);

        assert_eq!(explain.result_reasons.len(), 3);
        assert_eq!(
            explain.result_reasons[1].reason_code,
            "temporal_payload_deferred"
        );
        assert!(explain.result_reasons[1]
            .detail
            .contains("tier2://timeline/payload/0022/34"));
        assert_eq!(
            explain.result_reasons[2].reason_code,
            "temporal_landmark_not_selected"
        );
        assert!(explain.result_reasons[2]
            .detail
            .contains("without landmark promotion or era creation"));
    }

    #[test]
    fn answered_from_names() {
        assert_eq!(AnsweredFrom::Tier1Hot.as_str(), "tier1_hot");
        assert_eq!(AnsweredFrom::Tier2Indexed.as_str(), "tier2_indexed");
        assert_eq!(AnsweredFrom::Tier3Cold.as_str(), "tier3_cold");
    }
}
