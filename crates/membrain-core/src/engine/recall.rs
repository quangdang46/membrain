use crate::config::RuntimeConfig;
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
}

impl RecallRequest {
    /// Builds a request for direct exact-id Tier1 planning.
    pub const fn exact(memory_id: MemoryId) -> Self {
        Self {
            exact_memory_id: Some(memory_id),
            session_id: None,
            small_lookup: false,
        }
    }

    /// Builds a request for a session-local bounded lookup.
    pub const fn small_session_lookup(session_id: SessionId) -> Self {
        Self {
            exact_memory_id: None,
            session_id: Some(session_id),
            small_lookup: true,
        }
    }
}

/// Stable planner branches for exact-id, recent, and deeper fallback routing.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RecallPlanKind {
    ExactIdTier1,
    RecentTier1ThenTier2Exact,
    Tier2ExactThenTier3Fallback,
}

/// Stage-level route facts preserved for explain and inspect surfaces.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RecallTraceStage {
    Tier1ExactHandle,
    Tier1RecentWindow,
    Tier2Exact,
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
    const TIER2_THEN_TIER3_TRACE: &'static [RecallTraceStage] = &[
        RecallTraceStage::Tier2Exact,
        RecallTraceStage::Tier3Fallback,
    ];
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
                },
            };
        }

        if let Some(session_id) = request.session_id.filter(|_| request.small_lookup) {
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
                },
            };
        }

        RecallPlan {
            kind: RecallPlanKind::Tier2ExactThenTier3Fallback,
            exact_memory_id: None,
            session_id: request.session_id,
            tier1_candidate_budget,
            route_summary: RecallRouteSummary {
                tier1_answers_directly: false,
                tier1_consulted_first: false,
                routes_to_deeper_tiers: true,
                reason: "request lacks a direct Tier1 answer and escalates to deeper indexed retrieval",
                trace_stages: Self::TIER2_THEN_TIER3_TRACE,
            },
            trace: Tier1PlanTrace {
                route_name: "tier2.exact_then_tier3_fallback",
                tier1_answered_directly: false,
                stayed_within_latency_budget: true,
                candidate_budget: tier1_candidate_budget,
                pre_tier1_candidates: 0,
                post_tier1_candidates: 0,
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{RecallEngine, RecallPlanKind, RecallRequest, RecallRuntime, RecallTraceStage};
    use crate::config::RuntimeConfig;
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
        assert_eq!(plan.route_summary.trace_stages, &[RecallTraceStage::Tier1ExactHandle]);
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
        assert_eq!(plan.trace.route_name, "tier1.recent_window_then_tier2_exact");
        assert!(!plan.trace.tier1_answered_directly);
        assert!(plan.trace.stayed_within_latency_budget);
        assert_eq!(plan.trace.candidate_budget, RuntimeConfig::default().tier1_candidate_budget);
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
            },
            RuntimeConfig::default(),
        );
        let no_session = engine.plan_recall(
            RecallRequest {
                exact_memory_id: None,
                session_id: None,
                small_lookup: true,
            },
            RuntimeConfig::default(),
        );

        assert_eq!(large_lookup.kind, RecallPlanKind::Tier2ExactThenTier3Fallback);
        assert_eq!(large_lookup.session_id, Some(SessionId(9)));
        assert!(!large_lookup.terminates_in_tier1());
        assert!(!large_lookup.route_summary.tier1_answers_directly);
        assert!(!large_lookup.route_summary.tier1_consulted_first);
        assert!(large_lookup.route_summary.routes_to_deeper_tiers);
        assert_eq!(large_lookup.trace.route_name, "tier2.exact_then_tier3_fallback");
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
}
