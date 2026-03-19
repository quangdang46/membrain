use crate::observability::OutcomeClass;

/// Effective policy decision shared across core APIs and wrappers.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PolicyDecision {
    /// The request may proceed to bounded planning or retrieval.
    Allow,
    /// The request must stop before expensive work begins.
    Deny,
}

/// Canonical ingest modes that remain inspectable across intake surfaces.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IngestMode {
    Active,
    PassiveObservation,
}

/// Machine-readable passive-observation write outcome.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PassiveObservationDecision {
    Capture,
    Suppress,
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

/// Shared safeguard classes for risky or high-blast-radius operations.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OperationClass {
    ReadOnlyAssessment,
    DerivedSurfaceMutation,
    AuthoritativeRewrite,
    IrreversibleMutation,
}

/// Shared preflight readiness states preserved across interfaces.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PreflightState {
    Ready,
    Blocked,
    MissingData,
    StaleKnowledge,
}

/// Machine-readable per-check status preserved in safeguard outputs.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PreflightCheckStatus {
    Passed,
    Blocked,
    Degraded,
    Rejected,
}

/// Stable blocked or rejected reason codes for preflight and safeguard flows.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SafeguardReasonCode {
    ConfirmationRequired,
    StalePreflight,
    GenerationMismatch,
    SnapshotRequired,
    MaintenanceWindowRequired,
    DependencyPending,
    ScopeAmbiguous,
    AuthoritativeInputUnreadable,
    ConfidenceTooLow,
    PolicyDenied,
    LegalHold,
}

/// Recovery posture for risky operations.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ReversibilityKind {
    RepairableFromDurableTruth,
    RollbackViaSnapshot,
    PartiallyReversible,
    Irreversible,
}

/// Shared readiness check result preserved in the safeguard object.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PreflightCheck {
    pub check_name: &'static str,
    pub status: PreflightCheckStatus,
    pub reason_codes: Vec<SafeguardReasonCode>,
    pub checked_scope: &'static str,
}

/// Shared write-gating request for passive observation.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ObservationWriteRequest {
    pub ingest_mode: IngestMode,
    pub namespace_bound: bool,
    pub policy_allowed: bool,
    pub duplicate_hint: bool,
}

/// Machine-readable passive-observation gate result.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ObservationWriteOutcome {
    pub decision: PassiveObservationDecision,
    pub policy_summary: PolicySummary,
    pub captured_as_observation: bool,
}

/// Shared confirmation state for previewed risky operations.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ConfirmationState {
    pub required: bool,
    pub force_allowed: bool,
    pub confirmed: bool,
    pub generation_bound: Option<u64>,
}

/// Shared safeguard payload reused by preview, blocked, degraded, rejected, and accepted flows.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SafeguardOutcome {
    pub outcome_class: OutcomeClass,
    pub preflight_state: PreflightState,
    pub operation_class: OperationClass,
    pub blocked_reasons: Vec<SafeguardReasonCode>,
    pub preflight_checks: Vec<PreflightCheck>,
    pub reversibility: ReversibilityKind,
    pub confirmation: ConfirmationState,
    pub policy_summary: PolicySummary,
}

/// Input facts evaluated by the shared preflight safeguard machine.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SafeguardRequest {
    pub operation_class: OperationClass,
    pub preview_only: bool,
    pub namespace_bound: bool,
    pub policy_allowed: bool,
    pub requires_confirmation: bool,
    pub local_confirmation: bool,
    pub force_allowed: bool,
    pub generation_bound: Option<u64>,
    pub generation_matches: bool,
    pub snapshot_required: bool,
    pub snapshot_available: bool,
    pub maintenance_window_required: bool,
    pub maintenance_window_active: bool,
    pub dependencies_ready: bool,
    pub scope_precise: bool,
    pub authoritative_input_readable: bool,
    pub confidence_ready: bool,
    pub can_degrade: bool,
    pub legal_hold: bool,
}

impl SafeguardRequest {
    /// Builds a baseline request with all readiness checks passing.
    pub const fn ready(operation_class: OperationClass) -> Self {
        Self {
            operation_class,
            preview_only: false,
            namespace_bound: true,
            policy_allowed: true,
            requires_confirmation: false,
            local_confirmation: false,
            force_allowed: false,
            generation_bound: None,
            generation_matches: true,
            snapshot_required: false,
            snapshot_available: true,
            maintenance_window_required: false,
            maintenance_window_active: true,
            dependencies_ready: true,
            scope_precise: true,
            authoritative_input_readable: true,
            confidence_ready: true,
            can_degrade: false,
            legal_hold: false,
        }
    }
}

/// Stable core policy boundary that wrappers call instead of reimplementing policy behavior.
pub trait PolicyGateway {
    /// Evaluates the namespace gate before expensive work starts.
    fn evaluate_namespace(&self, namespace_bound: bool) -> PolicySummary;

    /// Evaluates passive-observation write gating before intake persists anything.
    fn evaluate_observation_write(&self, request: ObservationWriteRequest) -> ObservationWriteOutcome;

    /// Evaluates the shared safeguard and preflight contract for risky operations.
    fn evaluate_safeguard(&self, request: SafeguardRequest) -> SafeguardOutcome;
}

/// Minimal shared policy module owned by `membrain-core`.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct PolicyModule;

impl PolicyModule {
    fn reversibility_for(operation_class: OperationClass) -> ReversibilityKind {
        match operation_class {
            OperationClass::ReadOnlyAssessment => ReversibilityKind::RepairableFromDurableTruth,
            OperationClass::DerivedSurfaceMutation => ReversibilityKind::RepairableFromDurableTruth,
            OperationClass::AuthoritativeRewrite => ReversibilityKind::RollbackViaSnapshot,
            OperationClass::IrreversibleMutation => ReversibilityKind::Irreversible,
        }
    }

    fn confirmation_state(request: SafeguardRequest) -> ConfirmationState {
        ConfirmationState {
            required: request.requires_confirmation,
            force_allowed: request.force_allowed,
            confirmed: request.local_confirmation,
            generation_bound: request.generation_bound,
        }
    }

    fn push_check(
        checks: &mut Vec<PreflightCheck>,
        check_name: &'static str,
        status: PreflightCheckStatus,
        checked_scope: &'static str,
        reason_codes: Vec<SafeguardReasonCode>,
    ) {
        checks.push(PreflightCheck {
            check_name,
            status,
            checked_scope,
            reason_codes,
        });
    }
}

impl PolicyGateway for PolicyModule {
    fn evaluate_namespace(&self, namespace_bound: bool) -> PolicySummary {
        if namespace_bound {
            PolicySummary::allow(true)
        } else {
            PolicySummary::deny(false)
        }
    }

    fn evaluate_observation_write(&self, request: ObservationWriteRequest) -> ObservationWriteOutcome {
        let policy_summary = if request.namespace_bound && request.policy_allowed {
            PolicySummary::allow(request.namespace_bound)
        } else {
            PolicySummary::deny(request.namespace_bound)
        };

        let decision = match request.ingest_mode {
            IngestMode::Active => {
                if !request.namespace_bound || !request.policy_allowed {
                    PassiveObservationDecision::Deny
                } else {
                    PassiveObservationDecision::Capture
                }
            }
            IngestMode::PassiveObservation => {
                if !request.namespace_bound || !request.policy_allowed {
                    PassiveObservationDecision::Deny
                } else if request.duplicate_hint {
                    PassiveObservationDecision::Suppress
                } else {
                    PassiveObservationDecision::Capture
                }
            }
        };

        ObservationWriteOutcome {
            decision,
            policy_summary,
            captured_as_observation: matches!(
                (request.ingest_mode, decision),
                (IngestMode::PassiveObservation, PassiveObservationDecision::Capture)
            ),
        }
    }

    fn evaluate_safeguard(&self, request: SafeguardRequest) -> SafeguardOutcome {
        let policy_summary = if request.namespace_bound && request.policy_allowed && !request.legal_hold {
            PolicySummary::allow(request.namespace_bound)
        } else {
            PolicySummary::deny(request.namespace_bound)
        };

        let confirmation = Self::confirmation_state(request);
        let reversibility = Self::reversibility_for(request.operation_class);
        let mut blocked_reasons = Vec::new();
        let mut preflight_checks = Vec::new();

        let policy_reason = if request.legal_hold {
            Some(SafeguardReasonCode::LegalHold)
        } else if !request.policy_allowed || !request.namespace_bound {
            Some(SafeguardReasonCode::PolicyDenied)
        } else {
            None
        };
        Self::push_check(
            &mut preflight_checks,
            "policy",
            if policy_reason.is_some() {
                PreflightCheckStatus::Rejected
            } else {
                PreflightCheckStatus::Passed
            },
            "effective_namespace",
            policy_reason.into_iter().collect(),
        );

        let generation_reason = if !request.generation_matches {
            Some(if request.generation_bound.is_some() {
                SafeguardReasonCode::GenerationMismatch
            } else {
                SafeguardReasonCode::StalePreflight
            })
        } else {
            None
        };
        Self::push_check(
            &mut preflight_checks,
            "generation",
            if generation_reason.is_some() {
                PreflightCheckStatus::Blocked
            } else {
                PreflightCheckStatus::Passed
            },
            "preview_generation",
            generation_reason.into_iter().collect(),
        );

        let input_reason = if request.snapshot_required && !request.snapshot_available {
            Some(SafeguardReasonCode::SnapshotRequired)
        } else if !request.scope_precise {
            Some(SafeguardReasonCode::ScopeAmbiguous)
        } else {
            None
        };
        Self::push_check(
            &mut preflight_checks,
            "required_input",
            if input_reason.is_some() {
                PreflightCheckStatus::Blocked
            } else {
                PreflightCheckStatus::Passed
            },
            "request_scope",
            input_reason.into_iter().collect(),
        );

        let dependency_reason = if request.maintenance_window_required && !request.maintenance_window_active {
            Some(SafeguardReasonCode::MaintenanceWindowRequired)
        } else if !request.dependencies_ready {
            Some(SafeguardReasonCode::DependencyPending)
        } else {
            None
        };
        Self::push_check(
            &mut preflight_checks,
            "dependency",
            if dependency_reason.is_some() {
                PreflightCheckStatus::Blocked
            } else {
                PreflightCheckStatus::Passed
            },
            "operation_dependencies",
            dependency_reason.into_iter().collect(),
        );

        let confidence_reason = (!request.confidence_ready)
            .then_some(SafeguardReasonCode::ConfidenceTooLow);
        Self::push_check(
            &mut preflight_checks,
            "confidence",
            if confidence_reason.is_some() {
                if request.can_degrade {
                    PreflightCheckStatus::Degraded
                } else {
                    PreflightCheckStatus::Blocked
                }
            } else {
                PreflightCheckStatus::Passed
            },
            "confidence_window",
            confidence_reason.into_iter().collect(),
        );

        let readable_reason = (!request.authoritative_input_readable)
            .then_some(SafeguardReasonCode::AuthoritativeInputUnreadable);
        Self::push_check(
            &mut preflight_checks,
            "authoritative_input",
            if readable_reason.is_some() {
                if request.can_degrade {
                    PreflightCheckStatus::Degraded
                } else {
                    PreflightCheckStatus::Blocked
                }
            } else {
                PreflightCheckStatus::Passed
            },
            "authoritative_inputs",
            readable_reason.into_iter().collect(),
        );

        if let Some(reason) = policy_reason {
            blocked_reasons.push(reason);
            return SafeguardOutcome {
                outcome_class: OutcomeClass::Rejected,
                preflight_state: PreflightState::Blocked,
                operation_class: request.operation_class,
                blocked_reasons,
                preflight_checks,
                reversibility,
                confirmation,
                policy_summary,
            };
        }

        if let Some(reason) = generation_reason {
            blocked_reasons.push(reason);
        }
        if let Some(reason) = input_reason {
            blocked_reasons.push(reason);
        }
        if let Some(reason) = dependency_reason {
            blocked_reasons.push(reason);
        }
        if !request.confidence_ready && !request.can_degrade {
            blocked_reasons.push(SafeguardReasonCode::ConfidenceTooLow);
        }
        if !request.authoritative_input_readable && !request.can_degrade {
            blocked_reasons.push(SafeguardReasonCode::AuthoritativeInputUnreadable);
        }

        let preflight_state = if !request.generation_matches {
            PreflightState::StaleKnowledge
        } else if (request.snapshot_required && !request.snapshot_available) || !request.scope_precise {
            PreflightState::MissingData
        } else if blocked_reasons.is_empty() {
            PreflightState::Ready
        } else {
            PreflightState::Blocked
        };

        if request.preview_only {
            return SafeguardOutcome {
                outcome_class: OutcomeClass::Preview,
                preflight_state,
                operation_class: request.operation_class,
                blocked_reasons,
                preflight_checks,
                reversibility,
                confirmation,
                policy_summary,
            };
        }

        if request.requires_confirmation && !request.local_confirmation {
            blocked_reasons.push(SafeguardReasonCode::ConfirmationRequired);
        }

        if !blocked_reasons.is_empty() {
            return SafeguardOutcome {
                outcome_class: OutcomeClass::Blocked,
                preflight_state: if request.requires_confirmation && !request.local_confirmation {
                    PreflightState::Blocked
                } else {
                    preflight_state
                },
                operation_class: request.operation_class,
                blocked_reasons,
                preflight_checks,
                reversibility,
                confirmation,
                policy_summary,
            };
        }

        let outcome_class = if !request.confidence_ready || !request.authoritative_input_readable {
            OutcomeClass::Degraded
        } else {
            OutcomeClass::Accepted
        };

        SafeguardOutcome {
            outcome_class,
            preflight_state: PreflightState::Ready,
            operation_class: request.operation_class,
            blocked_reasons,
            preflight_checks,
            reversibility,
            confirmation,
            policy_summary,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{
        IngestMode, ObservationWriteRequest, OperationClass, PassiveObservationDecision,
        PolicyDecision, PolicyGateway, PolicyModule, PreflightCheckStatus, PreflightState,
        ReversibilityKind, SafeguardReasonCode, SafeguardRequest,
    };
    use crate::observability::OutcomeClass;

    #[test]
    fn passive_observation_captures_when_policy_allows_and_no_duplicate_hint_exists() {
        let gateway = PolicyModule;
        let outcome = gateway.evaluate_observation_write(ObservationWriteRequest {
            ingest_mode: IngestMode::PassiveObservation,
            namespace_bound: true,
            policy_allowed: true,
            duplicate_hint: false,
        });

        assert_eq!(outcome.decision, PassiveObservationDecision::Capture);
        assert_eq!(outcome.policy_summary.decision, PolicyDecision::Allow);
        assert!(outcome.captured_as_observation);
    }

    #[test]
    fn passive_observation_suppresses_duplicate_hints_without_policy_denial() {
        let gateway = PolicyModule;
        let outcome = gateway.evaluate_observation_write(ObservationWriteRequest {
            ingest_mode: IngestMode::PassiveObservation,
            namespace_bound: true,
            policy_allowed: true,
            duplicate_hint: true,
        });

        assert_eq!(outcome.decision, PassiveObservationDecision::Suppress);
        assert_eq!(outcome.policy_summary.decision, PolicyDecision::Allow);
        assert!(!outcome.captured_as_observation);
    }

    #[test]
    fn passive_observation_denial_stops_before_capture() {
        let gateway = PolicyModule;
        let outcome = gateway.evaluate_observation_write(ObservationWriteRequest {
            ingest_mode: IngestMode::PassiveObservation,
            namespace_bound: false,
            policy_allowed: false,
            duplicate_hint: false,
        });

        assert_eq!(outcome.decision, PassiveObservationDecision::Deny);
        assert_eq!(outcome.policy_summary.decision, PolicyDecision::Deny);
        assert!(!outcome.captured_as_observation);
    }

    #[test]
    fn active_ingest_denial_stops_before_capture() {
        let gateway = PolicyModule;
        let outcome = gateway.evaluate_observation_write(ObservationWriteRequest {
            ingest_mode: IngestMode::Active,
            namespace_bound: false,
            policy_allowed: false,
            duplicate_hint: false,
        });

        assert_eq!(outcome.decision, PassiveObservationDecision::Deny);
        assert_eq!(outcome.policy_summary.decision, PolicyDecision::Deny);
        assert!(!outcome.captured_as_observation);
    }

    #[test]
    fn preview_only_returns_preview_outcome() {
        let gateway = PolicyModule;
        let mut request = SafeguardRequest::ready(OperationClass::AuthoritativeRewrite);
        request.preview_only = true;
        request.requires_confirmation = true;

        let outcome = gateway.evaluate_safeguard(request);

        assert_eq!(outcome.outcome_class, OutcomeClass::Preview);
        assert_eq!(outcome.preflight_state, PreflightState::Ready);
        assert!(outcome.blocked_reasons.is_empty());
        assert!(outcome.confirmation.required);
        assert_eq!(outcome.reversibility, ReversibilityKind::RollbackViaSnapshot);
    }

    #[test]
    fn apply_without_required_confirmation_blocks() {
        let gateway = PolicyModule;
        let mut request = SafeguardRequest::ready(OperationClass::AuthoritativeRewrite);
        request.requires_confirmation = true;

        let outcome = gateway.evaluate_safeguard(request);

        assert_eq!(outcome.outcome_class, OutcomeClass::Blocked);
        assert_eq!(outcome.preflight_state, PreflightState::Blocked);
        assert!(outcome
            .blocked_reasons
            .contains(&SafeguardReasonCode::ConfirmationRequired));
    }

    #[test]
    fn force_confirmed_apply_stays_accepted_when_other_checks_pass() {
        let gateway = PolicyModule;
        let mut request = SafeguardRequest::ready(OperationClass::AuthoritativeRewrite);
        request.requires_confirmation = true;
        request.local_confirmation = true;
        request.force_allowed = true;
        request.generation_bound = Some(7);

        let outcome = gateway.evaluate_safeguard(request);

        assert_eq!(outcome.outcome_class, OutcomeClass::Accepted);
        assert_eq!(outcome.preflight_state, PreflightState::Ready);
        assert!(outcome.confirmation.confirmed);
        assert!(outcome.confirmation.force_allowed);
        assert_eq!(outcome.confirmation.generation_bound, Some(7));
    }

    #[test]
    fn degraded_outcome_preserves_degraded_check_status() {
        let gateway = PolicyModule;
        let mut request = SafeguardRequest::ready(OperationClass::DerivedSurfaceMutation);
        request.confidence_ready = false;
        request.can_degrade = true;

        let outcome = gateway.evaluate_safeguard(request);

        assert_eq!(outcome.outcome_class, OutcomeClass::Degraded);
        assert!(outcome.blocked_reasons.is_empty());
        assert!(outcome.preflight_checks.iter().any(|check| {
            check.check_name == "confidence" && check.status == PreflightCheckStatus::Degraded
        }));
    }

    #[test]
    fn policy_denial_is_rejected_even_with_local_confirmation() {
        let gateway = PolicyModule;
        let mut request = SafeguardRequest::ready(OperationClass::IrreversibleMutation);
        request.requires_confirmation = true;
        request.local_confirmation = true;
        request.policy_allowed = false;

        let outcome = gateway.evaluate_safeguard(request);

        assert_eq!(outcome.outcome_class, OutcomeClass::Rejected);
        assert_eq!(outcome.policy_summary.decision, PolicyDecision::Deny);
        assert!(outcome
            .blocked_reasons
            .contains(&SafeguardReasonCode::PolicyDenied));
    }

    #[test]
    fn stale_preflight_blocks_when_generation_anchor_is_missing() {
        let gateway = PolicyModule;
        let mut request = SafeguardRequest::ready(OperationClass::AuthoritativeRewrite);
        request.generation_matches = false;

        let outcome = gateway.evaluate_safeguard(request);

        assert_eq!(outcome.outcome_class, OutcomeClass::Blocked);
        assert_eq!(outcome.preflight_state, PreflightState::StaleKnowledge);
        assert!(outcome
            .blocked_reasons
            .contains(&SafeguardReasonCode::StalePreflight));
    }

    #[test]
    fn generation_mismatch_blocks_when_bound_generation_moves() {
        let gateway = PolicyModule;
        let mut request = SafeguardRequest::ready(OperationClass::AuthoritativeRewrite);
        request.generation_bound = Some(11);
        request.generation_matches = false;

        let outcome = gateway.evaluate_safeguard(request);

        assert_eq!(outcome.outcome_class, OutcomeClass::Blocked);
        assert!(outcome
            .blocked_reasons
            .contains(&SafeguardReasonCode::GenerationMismatch));
    }

    #[test]
    fn snapshot_requirement_surfaces_missing_data() {
        let gateway = PolicyModule;
        let mut request = SafeguardRequest::ready(OperationClass::AuthoritativeRewrite);
        request.snapshot_required = true;
        request.snapshot_available = false;

        let outcome = gateway.evaluate_safeguard(request);

        assert_eq!(outcome.outcome_class, OutcomeClass::Blocked);
        assert_eq!(outcome.preflight_state, PreflightState::MissingData);
        assert!(outcome
            .blocked_reasons
            .contains(&SafeguardReasonCode::SnapshotRequired));
    }

    #[test]
    fn legal_hold_is_rejected_like_terminal_governance_denial() {
        let gateway = PolicyModule;
        let mut request = SafeguardRequest::ready(OperationClass::IrreversibleMutation);
        request.local_confirmation = true;
        request.legal_hold = true;

        let outcome = gateway.evaluate_safeguard(request);

        assert_eq!(outcome.outcome_class, OutcomeClass::Rejected);
        assert!(outcome
            .blocked_reasons
            .contains(&SafeguardReasonCode::LegalHold));
    }
}
