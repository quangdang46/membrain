use crate::observability::OutcomeClass;

/// Effective policy decision shared across core APIs and wrappers.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PolicyDecision {
    /// The request may proceed to bounded planning or retrieval.
    Allow,
    /// The request must stop before expensive work begins.
    Deny,
}

/// Machine-readable summary of the policy gate that fired before expensive work.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PolicySummary {
    /// Final allow/deny decision for the preflight gate.
    pub decision: PolicyDecision,
    /// Whether namespace binding succeeded before the request was evaluated.
    pub namespace_bound: bool,
    /// Canonical outcome class wrappers should preserve in traces and envelopes.
    pub outcome_class: OutcomeClass,
}

impl PolicySummary {
    /// Builds an allow summary for a namespace-bound request.
    pub const fn allow(namespace_bound: bool) -> Self {
        Self {
            decision: PolicyDecision::Allow,
            namespace_bound,
            outcome_class: OutcomeClass::Accepted,
        }
    }

    /// Builds a deny summary for a request that must stop before expensive work.
    pub const fn deny(namespace_bound: bool) -> Self {
        Self {
            decision: PolicyDecision::Deny,
            namespace_bound,
            outcome_class: OutcomeClass::Rejected,
        }
    }
}

/// Stable core policy boundary that wrappers call instead of reimplementing policy behavior.
pub trait PolicyGateway {
    /// Evaluates the namespace gate before expensive work starts.
    fn evaluate_namespace(&self, namespace_bound: bool) -> PolicySummary;
}

/// Minimal shared policy module owned by `membrain-core`.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct PolicyModule;

impl PolicyGateway for PolicyModule {
    fn evaluate_namespace(&self, namespace_bound: bool) -> PolicySummary {
        if namespace_bound {
            PolicySummary::allow(true)
        } else {
            PolicySummary::deny(false)
        }
    }
}
