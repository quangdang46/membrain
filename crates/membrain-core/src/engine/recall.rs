use crate::config::RuntimeConfig;
use crate::engine::intent::IntentClassification;
use crate::engine::retrieval_planner::QueryPath;
use crate::types::{MemoryId, SessionId};

/// Canonical request shape for deterministic Tier1 planner routing.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct RecallRequest {
    /// Stable memory id hint for the direct Tier1 exact-handle lane.
    pub exact_memory_id: Option<MemoryId>,
    /// Session-local hot window available to recent recall planning.
    pub session_id: Option<SessionId>,
    /// Whether the caller is asking for a bounded small lookup.
    pub small_lookup: bool,
    /// Whether bounded graph/engram expansion may run after Tier2 exact retrieval.
    pub graph_expansion: bool,
    /// Whether bounded predictive pre-recall may prepare a hot path before retrieval.
    pub predictive_preroll: bool,
}

impl RecallRequest {
    /// Builds a request for direct exact-id Tier1 planning.
    pub const fn exact(memory_id: MemoryId) -> Self {
        Self {
            exact_memory_id: Some(memory_id),
            session_id: None,
            small_lookup: false,
            graph_expansion: false,
            predictive_preroll: false,
        }
    }

    /// Builds a request for a session-local bounded lookup.
    pub const fn small_session_lookup(session_id: SessionId) -> Self {
        Self {
            exact_memory_id: None,
            session_id: Some(session_id),
            small_lookup: true,
            graph_expansion: false,
            predictive_preroll: false,
        }
    }

    /// Enables bounded graph/engram expansion after Tier2 exact retrieval.
    pub const fn with_graph_expansion(mut self, enabled: bool) -> Self {
        self.graph_expansion = enabled;
        self
    }
}

/// Stable planner branches for exact-id, recent, bounded graph expansion, and deeper fallback routing.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum RecallPlanKind {
    ExactIdTier1,
    RecentTier1ThenTier2Exact,
    Tier2ExactThenGraphExpansion,
    Tier2ExactThenTier3Fallback,
}

impl RecallPlanKind {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::ExactIdTier1 => "exact_id_tier1",
            Self::RecentTier1ThenTier2Exact => "recent_tier1_then_tier2_exact",
            Self::Tier2ExactThenGraphExpansion => "tier2_exact_then_graph_expansion",
            Self::Tier2ExactThenTier3Fallback => "tier2_exact_then_tier3_fallback",
        }
    }
}

/// Stage-level route facts preserved for explain and inspect surfaces.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum RecallTraceStage {
    Tier1ExactHandle,
    Tier1RecentWindow,
    Tier2Exact,
    GraphExpansion,
    Tier3Fallback,
}

/// Explain-oriented route summary for one deterministic recall plan.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RecallRouteSummary {
    /// Whether the planner expects Tier1 to answer directly without deeper routing.
    pub tier1_answers_directly: bool,
    /// Whether Tier1 is consulted before any deeper lane.
    pub tier1_consulted_first: bool,
    /// Whether the planner routes to deeper retrieval work after Tier1.
    pub routes_to_deeper_tiers: bool,
    /// Machine-readable reason for the chosen route.
    pub reason: &'static str,
    /// Ordered route stages for inspect and explain surfaces.
    pub trace_stages: &'static [RecallTraceStage],
}

/// Stable Tier1 planner trace proving bounded route, latency, and candidate evidence.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Tier1PlanTrace {
    /// Stable route selection name used by regression-facing tests and wrappers.
    pub route_name: &'static str,
    /// Whether the selected route answered directly inside Tier1.
    pub tier1_answered_directly: bool,
    /// Whether the route stayed within the declared Tier1 latency budget.
    pub stayed_within_latency_budget: bool,
    /// Number of Tier1 candidates the planner allows before colder escalation.
    pub candidate_budget: usize,
    /// Candidate count preserved before Tier1 routing begins.
    pub pre_tier1_candidates: usize,
    /// Candidate count preserved after Tier1 routing decisions complete.
    pub post_tier1_candidates: usize,
    /// Whether bounded predictive pre-recall preparation was eligible for this route.
    pub predictive_preroll_triggered: bool,
    /// Stable explain label when predictive pre-recall was skipped.
    pub predictive_preroll_skip_reason: &'static str,
}

/// Machine-readable route plan for the canonical recall surface.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RecallPlan {
    /// Primary planner branch selected for the request.
    pub kind: RecallPlanKind,
    /// Exact id hint preserved for explainability when present.
    pub exact_memory_id: Option<MemoryId>,
    /// Session-local hot window preserved for recent planning when present.
    pub session_id: Option<SessionId>,
    /// Maximum Tier1 candidate budget this request may consume.
    pub tier1_candidate_budget: usize,
    /// Explain-oriented route summary for the selected planner branch.
    pub route_summary: RecallRouteSummary,
    /// Stable planner trace carrying latency and candidate-budget evidence.
    pub trace: Tier1PlanTrace,
}

impl RecallPlan {
    /// Returns whether the plan may terminate inside Tier1 without routing deeper.
    pub const fn terminates_in_tier1(self) -> bool {
        self.route_summary.tier1_answers_directly
    }
}

/// Stable bounded heuristic signals that may trigger predictive pre-recall preparation.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PredictivePrerollSignal {
    SessionRecency,
    HighValueRecent,
    TemporalAnchor,
    ProceduralSequence,
}

impl PredictivePrerollSignal {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::SessionRecency => "session_recency",
            Self::HighValueRecent => "high_value_recent",
            Self::TemporalAnchor => "temporal_anchor",
            Self::ProceduralSequence => "procedural_sequence",
        }
    }
}

/// Stable predictive pre-recall trigger contract derived from shallow intent classification.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PredictivePrerollDecision {
    pub should_trigger: bool,
    pub signal: Option<PredictivePrerollSignal>,
    pub skip_reason: &'static str,
}

impl PredictivePrerollDecision {
    pub const fn skipped(reason: &'static str) -> Self {
        Self {
            should_trigger: false,
            signal: None,
            skip_reason: reason,
        }
    }
}

/// Manual override modes that can bypass automatic intent-to-route mapping.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum RouteOverride {
    ExactId(MemoryId),
    SmallSessionLookup(SessionId),
    GraphExpansion,
    Tier2Fallback,
}

impl RouteOverride {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::ExactId(_) => "exact_id",
            Self::SmallSessionLookup(_) => "small_session_lookup",
            Self::GraphExpansion => "graph_expansion",
            Self::Tier2Fallback => "tier2_fallback",
        }
    }
}

/// Inputs to the auto-routing layer that maps classified intent onto a bounded recall request.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AutoRouteRequest {
    pub exact_memory_id: Option<MemoryId>,
    pub session_id: Option<SessionId>,
    pub allow_graph_expansion: bool,
    pub enable_predictive_preroll: bool,
    pub override_mode: Option<RouteOverride>,
}

impl AutoRouteRequest {
    pub const fn new() -> Self {
        Self {
            exact_memory_id: None,
            session_id: None,
            allow_graph_expansion: false,
            enable_predictive_preroll: false,
            override_mode: None,
        }
    }

    pub const fn with_session(mut self, session_id: SessionId) -> Self {
        self.session_id = Some(session_id);
        self
    }

    pub const fn with_exact_memory(mut self, memory_id: MemoryId) -> Self {
        self.exact_memory_id = Some(memory_id);
        self
    }

    pub const fn with_graph_expansion(mut self, enabled: bool) -> Self {
        self.allow_graph_expansion = enabled;
        self
    }

    pub const fn with_predictive_preroll(mut self, enabled: bool) -> Self {
        self.enable_predictive_preroll = enabled;
        self
    }

    pub const fn with_override(mut self, override_mode: RouteOverride) -> Self {
        self.override_mode = Some(override_mode);
        self
    }
}

impl Default for AutoRouteRequest {
    fn default() -> Self {
        Self::new()
    }
}

/// Structured explain output for one auto-routing decision.
#[derive(Debug, Clone, PartialEq)]
pub struct AutoRouteExplain {
    pub intent: &'static str,
    pub confidence: f32,
    pub query_path: &'static str,
    pub ranking_profile: &'static str,
    pub override_applied: bool,
    pub override_mode: Option<&'static str>,
    pub request_summary: &'static str,
    pub selected_plan: &'static str,
    pub planner_reason: &'static str,
}

/// Full bounded auto-routing decision from intent classification to planner route.
#[derive(Debug, Clone, PartialEq)]
pub struct AutoRouteDecision {
    pub request: RecallRequest,
    pub plan: RecallPlan,
    pub explain: AutoRouteExplain,
}

/// Shared interface for Tier1 recall orchestration owned by `membrain-core`.
pub trait RecallRuntime {
    /// Returns the maximum Tier1 candidate budget this recall surface may consume.
    fn tier1_candidate_budget(&self, config: RuntimeConfig) -> usize;

    /// Plans the bounded Tier1 route for an incoming recall request.
    fn plan_recall(&self, request: RecallRequest, config: RuntimeConfig) -> RecallPlan;
}

/// Canonical recall engine placeholder owned by the core crate.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct RecallEngine;

impl RecallEngine {
    const EXACT_ID_TRACE: &'static [RecallTraceStage] = &[RecallTraceStage::Tier1ExactHandle];
    const RECENT_THEN_TIER2_TRACE: &'static [RecallTraceStage] = &[
        RecallTraceStage::Tier1RecentWindow,
        RecallTraceStage::Tier2Exact,
    ];
    const TIER2_THEN_GRAPH_TRACE: &'static [RecallTraceStage] = &[
        RecallTraceStage::Tier2Exact,
        RecallTraceStage::GraphExpansion,
    ];
    const TIER2_THEN_TIER3_TRACE: &'static [RecallTraceStage] = &[
        RecallTraceStage::Tier2Exact,
        RecallTraceStage::Tier3Fallback,
    ];

    /// Maps explicit intent classification plus optional override inputs onto one bounded recall plan.
    pub fn auto_route(
        &self,
        classification: &IntentClassification,
        request: AutoRouteRequest,
        config: RuntimeConfig,
    ) -> AutoRouteDecision {
        let recall_request = if let Some(override_mode) = request.override_mode {
            match override_mode {
                RouteOverride::ExactId(memory_id) => RecallRequest::exact(memory_id),
                RouteOverride::SmallSessionLookup(session_id) => {
                    let mut recall_request = RecallRequest::small_session_lookup(session_id);
                    recall_request.predictive_preroll = request.enable_predictive_preroll;
                    recall_request
                }
                RouteOverride::GraphExpansion => RecallRequest {
                    exact_memory_id: None,
                    session_id: request.session_id,
                    small_lookup: false,
                    graph_expansion: true,
                    predictive_preroll: false,
                },
                RouteOverride::Tier2Fallback => RecallRequest {
                    exact_memory_id: None,
                    session_id: request.session_id,
                    small_lookup: false,
                    graph_expansion: false,
                    predictive_preroll: request.enable_predictive_preroll,
                },
            }
        } else if let Some(memory_id) = request.exact_memory_id {
            RecallRequest::exact(memory_id)
        } else {
            RecallRequest {
                exact_memory_id: None,
                session_id: request.session_id,
                small_lookup: classification.route_inputs.prefer_small_lookup
                    && request.session_id.is_some(),
                graph_expansion: request.allow_graph_expansion
                    && matches!(
                        classification.route_inputs.query_path,
                        QueryPath::EntityHeavy
                    ),
                predictive_preroll: request.enable_predictive_preroll
                    && classification.route_inputs.predictive_preroll_candidate,
            }
        };
        let plan = self.plan_recall(recall_request, config);
        let explain = AutoRouteExplain {
            intent: classification.intent.as_str(),
            confidence: classification.confidence,
            query_path: classification.route_inputs.query_path.as_str(),
            ranking_profile: classification.route_inputs.ranking_profile.as_str(),
            override_applied: request.override_mode.is_some(),
            override_mode: request.override_mode.map(RouteOverride::as_str),
            request_summary: if recall_request.exact_memory_id.is_some() {
                "exact_id_request"
            } else if recall_request.small_lookup {
                "small_session_lookup"
            } else if recall_request.graph_expansion {
                "graph_expansion_request"
            } else {
                "tier2_fallback_request"
            },
            selected_plan: plan.kind.as_str(),
            planner_reason: plan.route_summary.reason,
        };

        AutoRouteDecision {
            request: recall_request,
            plan,
            explain,
        }
    }

    const fn trace_preroll(request: RecallRequest) -> (bool, &'static str) {
        if request.predictive_preroll {
            (true, "predictive_preroll_enabled")
        } else {
            (false, "request_not_opted_in")
        }
    }

    const fn predictive_preroll_signal(
        classification: &IntentClassification,
    ) -> Option<PredictivePrerollSignal> {
        match classification.intent {
            crate::engine::intent::QueryIntent::RecentFirst => {
                Some(PredictivePrerollSignal::SessionRecency)
            }
            crate::engine::intent::QueryIntent::StrengthWeighted => {
                Some(PredictivePrerollSignal::HighValueRecent)
            }
            crate::engine::intent::QueryIntent::TemporalAnchor => {
                Some(PredictivePrerollSignal::TemporalAnchor)
            }
            crate::engine::intent::QueryIntent::ProceduralLookup => {
                Some(PredictivePrerollSignal::ProceduralSequence)
            }
            _ => None,
        }
    }

    pub const fn predictive_preroll_decision(
        &self,
        classification: &IntentClassification,
        request: RecallRequest,
        config: RuntimeConfig,
    ) -> PredictivePrerollDecision {
        if !classification.route_inputs.predictive_preroll_candidate {
            return PredictivePrerollDecision::skipped("intent_not_predictive_candidate");
        }

        if classification.low_confidence_fallback {
            return PredictivePrerollDecision::skipped("low_confidence_fallback");
        }

        if request.exact_memory_id.is_some() {
            return PredictivePrerollDecision::skipped("exact_id_direct_lookup");
        }

        if request.graph_expansion {
            return PredictivePrerollDecision::skipped("graph_expansion_not_predictive");
        }

        if config.tier1_candidate_budget == 0 {
            return PredictivePrerollDecision::skipped("tier1_budget_disabled");
        }

        if config.prefetch_queue_capacity == 0 {
            return PredictivePrerollDecision::skipped("prefetch_budget_disabled");
        }

        let Some(signal) = Self::predictive_preroll_signal(classification) else {
            return PredictivePrerollDecision::skipped("intent_not_predictive_candidate");
        };

        if request.predictive_preroll {
            return PredictivePrerollDecision {
                should_trigger: true,
                signal: Some(signal),
                skip_reason: "predictive_preroll_enabled",
            };
        }

        PredictivePrerollDecision::skipped("request_not_opted_in")
    }
}

impl RecallRuntime for RecallEngine {
    fn tier1_candidate_budget(&self, config: RuntimeConfig) -> usize {
        config.tier1_candidate_budget
    }

    fn plan_recall(&self, request: RecallRequest, config: RuntimeConfig) -> RecallPlan {
        let tier1_candidate_budget = self.tier1_candidate_budget(config);

        if let Some(memory_id) = request.exact_memory_id {
            return RecallPlan {
                kind: RecallPlanKind::ExactIdTier1,
                exact_memory_id: Some(memory_id),
                session_id: None,
                tier1_candidate_budget,
                route_summary: RecallRouteSummary {
                    tier1_answers_directly: true,
                    tier1_consulted_first: true,
                    routes_to_deeper_tiers: false,
                    reason: "exact memory id selects the direct Tier1 handle lane",
                    trace_stages: Self::EXACT_ID_TRACE,
                },
                trace: Tier1PlanTrace {
                    route_name: "tier1.exact_handle",
                    tier1_answered_directly: true,
                    stayed_within_latency_budget: true,
                    candidate_budget: tier1_candidate_budget,
                    pre_tier1_candidates: 1,
                    post_tier1_candidates: 1,
                    predictive_preroll_triggered: false,
                    predictive_preroll_skip_reason: "exact_id_direct_lookup",
                },
            };
        }

        if let Some(session_id) = request.session_id.filter(|_| request.small_lookup) {
            let (predictive_preroll_triggered, predictive_preroll_skip_reason) =
                Self::trace_preroll(request);
            return RecallPlan {
                kind: RecallPlanKind::RecentTier1ThenTier2Exact,
                exact_memory_id: None,
                session_id: Some(session_id),
                tier1_candidate_budget,
                route_summary: RecallRouteSummary {
                    tier1_answers_directly: false,
                    tier1_consulted_first: true,
                    routes_to_deeper_tiers: true,
                    reason: "small session lookup scans the Tier1 recent window before Tier2 exact",
                    trace_stages: Self::RECENT_THEN_TIER2_TRACE,
                },
                trace: Tier1PlanTrace {
                    route_name: "tier1.recent_window_then_tier2_exact",
                    tier1_answered_directly: false,
                    stayed_within_latency_budget: true,
                    candidate_budget: tier1_candidate_budget,
                    pre_tier1_candidates: tier1_candidate_budget,
                    post_tier1_candidates: tier1_candidate_budget,
                    predictive_preroll_triggered,
                    predictive_preroll_skip_reason,
                },
            };
        }

        if request.graph_expansion {
            return RecallPlan {
                kind: RecallPlanKind::Tier2ExactThenGraphExpansion,
                exact_memory_id: None,
                session_id: request.session_id,
                tier1_candidate_budget,
                route_summary: RecallRouteSummary {
                    tier1_answers_directly: false,
                    tier1_consulted_first: false,
                    routes_to_deeper_tiers: true,
                    reason:
                        "request uses bounded graph expansion from the Tier2-authorized seed shortlist",
                    trace_stages: Self::TIER2_THEN_GRAPH_TRACE,
                },
                trace: Tier1PlanTrace {
                    route_name: "tier2.exact_then_graph_expansion",
                    tier1_answered_directly: false,
                    stayed_within_latency_budget: true,
                    candidate_budget: tier1_candidate_budget,
                    pre_tier1_candidates: 0,
                    post_tier1_candidates: config.graph_max_nodes.min(config.tier2_candidate_budget),
                    predictive_preroll_triggered: false,
                    predictive_preroll_skip_reason: "graph_expansion_not_predictive",
                },
            };
        }

        let (predictive_preroll_triggered, predictive_preroll_skip_reason) =
            Self::trace_preroll(request);
        RecallPlan {
            kind: RecallPlanKind::Tier2ExactThenTier3Fallback,
            exact_memory_id: None,
            session_id: request.session_id,
            tier1_candidate_budget,
            route_summary: RecallRouteSummary {
                tier1_answers_directly: false,
                tier1_consulted_first: false,
                routes_to_deeper_tiers: true,
                reason:
                    "request lacks a direct Tier1 answer and escalates to deeper indexed retrieval",
                trace_stages: Self::TIER2_THEN_TIER3_TRACE,
            },
            trace: Tier1PlanTrace {
                route_name: "tier2.exact_then_tier3_fallback",
                tier1_answered_directly: false,
                stayed_within_latency_budget: true,
                candidate_budget: tier1_candidate_budget,
                pre_tier1_candidates: 0,
                post_tier1_candidates: 0,
                predictive_preroll_triggered,
                predictive_preroll_skip_reason,
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{
        AutoRouteRequest, PredictivePrerollSignal, RecallEngine, RecallPlanKind, RecallRequest,
        RecallRuntime, RecallTraceStage, RouteOverride,
    };
    use crate::config::RuntimeConfig;
    use crate::engine::intent::IntentEngine;
    use crate::types::{MemoryId, SessionId};

    #[test]
    fn exact_id_requests_plan_direct_tier1_lookup() {
        let engine = RecallEngine;

        let plan = engine.plan_recall(RecallRequest::exact(MemoryId(42)), RuntimeConfig::default());

        assert_eq!(plan.kind, RecallPlanKind::ExactIdTier1);
        assert_eq!(plan.exact_memory_id, Some(MemoryId(42)));
        assert_eq!(plan.session_id, None);
        assert!(plan.terminates_in_tier1());
        assert!(plan.route_summary.tier1_answers_directly);
        assert!(plan.route_summary.tier1_consulted_first);
        assert!(!plan.route_summary.routes_to_deeper_tiers);
        assert_eq!(
            plan.route_summary.trace_stages,
            &[RecallTraceStage::Tier1ExactHandle]
        );
        assert_eq!(plan.trace.route_name, "tier1.exact_handle");
        assert!(plan.trace.tier1_answered_directly);
        assert!(plan.trace.stayed_within_latency_budget);
        assert_eq!(plan.trace.pre_tier1_candidates, 1);
        assert_eq!(plan.trace.post_tier1_candidates, 1);
        assert_eq!(
            plan.tier1_candidate_budget,
            RuntimeConfig::default().tier1_candidate_budget,
        );
    }

    #[test]
    fn small_session_lookups_scan_recent_tier1_before_tier2_exact() {
        let engine = RecallEngine;

        let plan = engine.plan_recall(
            RecallRequest::small_session_lookup(SessionId(7)),
            RuntimeConfig::default(),
        );

        assert_eq!(plan.kind, RecallPlanKind::RecentTier1ThenTier2Exact);
        assert_eq!(plan.exact_memory_id, None);
        assert_eq!(plan.session_id, Some(SessionId(7)));
        assert!(!plan.terminates_in_tier1());
        assert!(!plan.route_summary.tier1_answers_directly);
        assert!(plan.route_summary.tier1_consulted_first);
        assert!(plan.route_summary.routes_to_deeper_tiers);
        assert_eq!(
            plan.route_summary.trace_stages,
            &[
                RecallTraceStage::Tier1RecentWindow,
                RecallTraceStage::Tier2Exact,
            ],
        );
        assert_eq!(
            plan.trace.route_name,
            "tier1.recent_window_then_tier2_exact"
        );
        assert!(!plan.trace.tier1_answered_directly);
        assert!(plan.trace.stayed_within_latency_budget);
        assert_eq!(
            plan.trace.candidate_budget,
            RuntimeConfig::default().tier1_candidate_budget
        );
        assert_eq!(
            plan.trace.pre_tier1_candidates,
            RuntimeConfig::default().tier1_candidate_budget,
        );
        assert_eq!(
            plan.trace.post_tier1_candidates,
            RuntimeConfig::default().tier1_candidate_budget,
        );
    }

    #[test]
    fn non_small_or_sessionless_requests_escalate_to_deeper_tiers() {
        let engine = RecallEngine;

        let large_lookup = engine.plan_recall(
            RecallRequest {
                exact_memory_id: None,
                session_id: Some(SessionId(9)),
                small_lookup: false,
                graph_expansion: false,
                predictive_preroll: false,
            },
            RuntimeConfig::default(),
        );
        let no_session = engine.plan_recall(
            RecallRequest {
                exact_memory_id: None,
                session_id: None,
                small_lookup: true,
                graph_expansion: false,
                predictive_preroll: false,
            },
            RuntimeConfig::default(),
        );

        assert_eq!(
            large_lookup.kind,
            RecallPlanKind::Tier2ExactThenTier3Fallback
        );
        assert_eq!(large_lookup.session_id, Some(SessionId(9)));
        assert!(!large_lookup.terminates_in_tier1());
        assert!(!large_lookup.route_summary.tier1_answers_directly);
        assert!(!large_lookup.route_summary.tier1_consulted_first);
        assert!(large_lookup.route_summary.routes_to_deeper_tiers);
        assert_eq!(
            large_lookup.trace.route_name,
            "tier2.exact_then_tier3_fallback"
        );
        assert!(!large_lookup.trace.tier1_answered_directly);
        assert_eq!(large_lookup.trace.pre_tier1_candidates, 0);
        assert_eq!(large_lookup.trace.post_tier1_candidates, 0);

        assert_eq!(no_session.kind, RecallPlanKind::Tier2ExactThenTier3Fallback);
        assert_eq!(no_session.session_id, None);
        assert!(!no_session.terminates_in_tier1());
        assert_eq!(
            no_session.route_summary.trace_stages,
            &[
                RecallTraceStage::Tier2Exact,
                RecallTraceStage::Tier3Fallback,
            ],
        );
    }

    #[test]
    fn session_scoped_deeper_fallback_preserves_explain_trace_for_temporal_consumers() {
        let engine = RecallEngine;

        let plan = engine.plan_recall(
            RecallRequest {
                exact_memory_id: None,
                session_id: Some(SessionId(11)),
                small_lookup: false,
                graph_expansion: false,
                predictive_preroll: false,
            },
            RuntimeConfig::default(),
        );

        assert_eq!(plan.kind, RecallPlanKind::Tier2ExactThenTier3Fallback);
        assert_eq!(plan.session_id, Some(SessionId(11)));
        assert!(!plan.terminates_in_tier1());
        assert!(!plan.route_summary.tier1_answers_directly);
        assert!(!plan.route_summary.tier1_consulted_first);
        assert!(plan.route_summary.routes_to_deeper_tiers);
        assert_eq!(
            plan.route_summary.reason,
            "request lacks a direct Tier1 answer and escalates to deeper indexed retrieval"
        );
        assert_eq!(
            plan.route_summary.trace_stages,
            &[
                RecallTraceStage::Tier2Exact,
                RecallTraceStage::Tier3Fallback
            ]
        );
        assert_eq!(plan.trace.route_name, "tier2.exact_then_tier3_fallback");
        assert!(!plan.trace.tier1_answered_directly);
        assert!(plan.trace.stayed_within_latency_budget);
        assert_eq!(
            plan.trace.candidate_budget,
            RuntimeConfig::default().tier1_candidate_budget
        );
        assert_eq!(plan.trace.pre_tier1_candidates, 0);
        assert_eq!(plan.trace.post_tier1_candidates, 0);
    }

    #[test]
    fn graph_expansion_requests_route_to_bounded_graph_stage() {
        let engine = RecallEngine;
        let config = RuntimeConfig::default();

        let plan = engine.plan_recall(
            RecallRequest {
                exact_memory_id: None,
                session_id: Some(SessionId(13)),
                small_lookup: false,
                graph_expansion: true,
                predictive_preroll: false,
            },
            config,
        );

        assert_eq!(plan.kind, RecallPlanKind::Tier2ExactThenGraphExpansion);
        assert_eq!(plan.session_id, Some(SessionId(13)));
        assert!(!plan.terminates_in_tier1());
        assert!(!plan.route_summary.tier1_answers_directly);
        assert!(!plan.route_summary.tier1_consulted_first);
        assert!(plan.route_summary.routes_to_deeper_tiers);
        assert_eq!(
            plan.route_summary.reason,
            "request uses bounded graph expansion from the Tier2-authorized seed shortlist"
        );
        assert_eq!(
            plan.route_summary.trace_stages,
            &[
                RecallTraceStage::Tier2Exact,
                RecallTraceStage::GraphExpansion,
            ]
        );
        assert_eq!(plan.trace.route_name, "tier2.exact_then_graph_expansion");
        assert_eq!(
            plan.trace.post_tier1_candidates,
            config.graph_max_nodes.min(config.tier2_candidate_budget)
        );
    }

    #[test]
    fn graph_expansion_builder_toggles_request_flag() {
        let request = RecallRequest::small_session_lookup(SessionId(21)).with_graph_expansion(true);

        assert!(request.graph_expansion);
        assert_eq!(request.session_id, Some(SessionId(21)));
        assert!(request.small_lookup);
    }

    #[test]
    fn predictive_preroll_trace_reflects_recent_lookup_opt_in() {
        let engine = RecallEngine;

        let opted_in = engine.plan_recall(
            RecallRequest {
                predictive_preroll: true,
                ..RecallRequest::small_session_lookup(SessionId(31))
            },
            RuntimeConfig::default(),
        );
        let defaulted = engine.plan_recall(
            RecallRequest::small_session_lookup(SessionId(32)),
            RuntimeConfig::default(),
        );

        assert!(opted_in.trace.predictive_preroll_triggered);
        assert_eq!(
            opted_in.trace.predictive_preroll_skip_reason,
            "predictive_preroll_enabled"
        );
        assert!(!defaulted.trace.predictive_preroll_triggered);
        assert_eq!(
            defaulted.trace.predictive_preroll_skip_reason,
            "request_not_opted_in"
        );
    }

    #[test]
    fn predictive_preroll_trace_reflects_fallback_opt_in_but_not_graph_routes() {
        let engine = RecallEngine;

        let fallback = engine.plan_recall(
            RecallRequest {
                exact_memory_id: None,
                session_id: Some(SessionId(41)),
                small_lookup: false,
                graph_expansion: false,
                predictive_preroll: true,
            },
            RuntimeConfig::default(),
        );
        let graph = engine.plan_recall(
            RecallRequest {
                exact_memory_id: None,
                session_id: Some(SessionId(42)),
                small_lookup: false,
                graph_expansion: true,
                predictive_preroll: true,
            },
            RuntimeConfig::default(),
        );

        assert!(fallback.trace.predictive_preroll_triggered);
        assert_eq!(
            fallback.trace.predictive_preroll_skip_reason,
            "predictive_preroll_enabled"
        );
        assert!(!graph.trace.predictive_preroll_triggered);
        assert_eq!(
            graph.trace.predictive_preroll_skip_reason,
            "graph_expansion_not_predictive"
        );
    }

    #[test]
    fn predictive_preroll_decision_maps_predictive_intents_to_bounded_signals() {
        let engine = RecallEngine;
        let intent_engine = IntentEngine;
        let config = RuntimeConfig::default();

        let recent = intent_engine.classify("what happened recently with the release pipeline?");
        let recent_decision = engine.predictive_preroll_decision(
            &recent,
            RecallRequest {
                predictive_preroll: true,
                ..RecallRequest::small_session_lookup(SessionId(51))
            },
            config,
        );
        assert!(recent_decision.should_trigger);
        assert_eq!(
            recent_decision.signal,
            Some(PredictivePrerollSignal::SessionRecency)
        );

        let important = intent_engine.classify("what is most important about the rollback plan?");
        let important_decision = engine.predictive_preroll_decision(
            &important,
            RecallRequest {
                session_id: Some(SessionId(52)),
                predictive_preroll: true,
                ..RecallRequest::default()
            },
            config,
        );
        assert!(important_decision.should_trigger);
        assert_eq!(
            important_decision.signal,
            Some(PredictivePrerollSignal::HighValueRecent)
        );

        let temporal = intent_engine.classify("what changed before the march deploy?");
        let temporal_decision = engine.predictive_preroll_decision(
            &temporal,
            RecallRequest {
                session_id: Some(SessionId(53)),
                predictive_preroll: true,
                ..RecallRequest::default()
            },
            config,
        );
        assert!(temporal_decision.should_trigger);
        assert_eq!(
            temporal_decision.signal,
            Some(PredictivePrerollSignal::TemporalAnchor)
        );

        let procedural = intent_engine.classify("how to rotate the credentials safely");
        let procedural_decision = engine.predictive_preroll_decision(
            &procedural,
            RecallRequest {
                session_id: Some(SessionId(54)),
                predictive_preroll: true,
                ..RecallRequest::default()
            },
            config,
        );
        assert!(procedural_decision.should_trigger);
        assert_eq!(
            procedural_decision.signal,
            Some(PredictivePrerollSignal::ProceduralSequence)
        );
    }

    #[test]
    fn predictive_preroll_decision_respects_prefetch_budget_bound() {
        let engine = RecallEngine;
        let classification =
            IntentEngine.classify("what happened recently with the release pipeline?");
        let decision = engine.predictive_preroll_decision(
            &classification,
            RecallRequest {
                predictive_preroll: true,
                ..RecallRequest::small_session_lookup(SessionId(61))
            },
            RuntimeConfig {
                prefetch_queue_capacity: 0,
                ..RuntimeConfig::default()
            },
        );

        assert!(!decision.should_trigger);
        assert_eq!(decision.signal, None);
        assert_eq!(decision.skip_reason, "prefetch_budget_disabled");
    }

    #[test]
    fn auto_route_maps_different_intents_to_different_bounded_plans() {
        let engine = RecallEngine;
        let config = RuntimeConfig::default();

        let recent = IntentEngine.classify("what happened recently with the release pipeline?");
        let recent_decision = engine.auto_route(
            &recent,
            AutoRouteRequest::new()
                .with_session(SessionId(71))
                .with_predictive_preroll(true),
            config,
        );

        let causal = IntentEngine.classify("why do I believe this service should stay split?");
        let causal_decision = engine.auto_route(
            &causal,
            AutoRouteRequest::new().with_graph_expansion(true),
            config,
        );

        assert_eq!(
            recent_decision.plan.kind,
            RecallPlanKind::RecentTier1ThenTier2Exact
        );
        assert_eq!(
            recent_decision.explain.request_summary,
            "small_session_lookup"
        );
        assert_eq!(
            recent_decision.explain.selected_plan,
            "recent_tier1_then_tier2_exact"
        );
        assert!(!recent_decision.explain.override_applied);

        assert_eq!(
            causal_decision.plan.kind,
            RecallPlanKind::Tier2ExactThenGraphExpansion
        );
        assert_eq!(
            causal_decision.explain.request_summary,
            "graph_expansion_request"
        );
        assert_eq!(
            causal_decision.explain.selected_plan,
            "tier2_exact_then_graph_expansion"
        );
        assert!(!causal_decision.explain.override_applied);
    }

    #[test]
    fn auto_route_override_bypasses_intent_selected_route() {
        let engine = RecallEngine;
        let classification =
            IntentEngine.classify("what happened recently with the release pipeline?");
        let decision = engine.auto_route(
            &classification,
            AutoRouteRequest::new().with_override(RouteOverride::Tier2Fallback),
            RuntimeConfig::default(),
        );

        assert_eq!(
            decision.plan.kind,
            RecallPlanKind::Tier2ExactThenTier3Fallback
        );
        assert!(decision.explain.override_applied);
        assert_eq!(decision.explain.override_mode, Some("tier2_fallback"));
        assert_eq!(decision.explain.request_summary, "tier2_fallback_request");
    }

    #[test]
    fn graph_and_fallback_overrides_do_not_get_forced_back_to_exact_id() {
        let engine = RecallEngine;
        let classification = IntentEngine.classify("what do I know about rust lifetimes?");

        let graph = engine.auto_route(
            &classification,
            AutoRouteRequest::new()
                .with_exact_memory(MemoryId(9))
                .with_override(RouteOverride::GraphExpansion),
            RuntimeConfig::default(),
        );
        let fallback = engine.auto_route(
            &classification,
            AutoRouteRequest::new()
                .with_exact_memory(MemoryId(9))
                .with_override(RouteOverride::Tier2Fallback),
            RuntimeConfig::default(),
        );

        assert_eq!(
            graph.plan.kind,
            RecallPlanKind::Tier2ExactThenGraphExpansion
        );
        assert_eq!(graph.request.exact_memory_id, None);
        assert!(graph.request.graph_expansion);

        assert_eq!(
            fallback.plan.kind,
            RecallPlanKind::Tier2ExactThenTier3Fallback
        );
        assert_eq!(fallback.request.exact_memory_id, None);
        assert!(!fallback.request.graph_expansion);
    }
}
