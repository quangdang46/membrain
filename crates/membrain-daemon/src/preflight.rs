use membrain_core::api::NamespaceId;
use membrain_core::observability::OutcomeClass;
use membrain_core::policy::PolicyGateway;
use membrain_core::policy::{
    OperationClass, PolicyDecision, PolicyModule, PreflightCheck, PreflightCheckStatus,
    PreflightState, SafeguardOutcome, SafeguardReasonCode, SafeguardRequest,
};
use serde::{Deserialize, Serialize};

/// Preflight safeguard contract and API (`mb-23u.7.4`)
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PreflightRunRequest {
    pub namespace: String,
    pub original_query: String,
    pub proposed_action: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct PreflightCheckView {
    pub check_name: String,
    pub status: String,
    pub reason_codes: Vec<String>,
    pub checked_scope: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct ConfirmationView {
    pub required: bool,
    pub force_allowed: bool,
    pub confirmed: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub generation_bound: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct AuditView {
    pub event_kind: String,
    pub actor_source: String,
    pub request_id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub preview_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub related_run: Option<String>,
    pub scope_handle: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct PolicySummaryView {
    pub decision: String,
    pub namespace_bound: bool,
    pub outcome_class: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PreflightExplainResponse {
    pub allowed: bool,
    pub preflight_state: String,
    pub preflight_outcome: String,
    pub blocked_reasons: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub blocked_reason: Option<String>,
    pub required_overrides: Vec<String>,
    pub policy_context: String,
    pub check_results: Vec<PreflightCheckView>,
    pub confirmation: ConfirmationView,
    pub audit: AuditView,
    pub policy_summary: PolicySummaryView,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub request_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub preflight_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PreflightAllowRequest {
    pub namespace: String,
    pub original_query: String,
    pub proposed_action: String,
    pub authorization_token: String,
    pub bypass_flags: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PreflightOutcome {
    pub success: bool,
    pub preflight_state: String,
    pub preflight_outcome: String,
    pub outcome_class: String,
    pub blocked_reasons: Vec<String>,
    pub check_results: Vec<PreflightCheckView>,
    pub confirmation: ConfirmationView,
    pub audit: AuditView,
    pub policy_summary: PolicySummaryView,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub request_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub preflight_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub execution_id: Option<String>,
    pub degraded: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub confirmation_reason: Option<String>,
}

#[derive(Debug, Clone)]
pub struct EvaluatedPreflight {
    pub request_id: String,
    pub preflight_id: String,
    pub execution_id: Option<String>,
    pub outcome: SafeguardOutcome,
}

pub fn validate_preflight_namespace(namespace: &str) -> Result<(), String> {
    NamespaceId::new(namespace)
        .map(|_| ())
        .map_err(|err| err.to_string())
}

pub fn preflight_request(
    namespace: &str,
    original_query: &str,
    proposed_action: &str,
    local_confirmation: bool,
    confirmed_generation: Option<u64>,
) -> SafeguardRequest {
    let namespace_bound = NamespaceId::new(namespace).is_ok();
    let query = format!("{original_query} {proposed_action}").to_ascii_lowercase();
    let is_destructive = query.contains("delete") || query.contains("purge");
    let legal_hold = query.contains("legal hold");
    let snapshot_required = query.contains("snapshot") || query.contains("archive");
    let stale = query.contains("stale");
    let missing_scope = query.contains("ambiguous") || query.contains("all namespaces");
    let maintenance_window_required = query.contains("rewrite") || query.contains("reindex");
    let maintenance_window_active = !query.contains("window closed");
    let dependencies_ready = !query.contains("dependency pending");
    let confidence_ready = !query.contains("low confidence");
    let authoritative_input_readable = !query.contains("input unreadable");
    let can_degrade = query.contains("degraded") || query.contains("fallback");

    SafeguardRequest {
        operation_class: if legal_hold || query.contains("contradiction") {
            OperationClass::ContradictionArchive
        } else if is_destructive {
            OperationClass::IrreversibleMutation
        } else if maintenance_window_required {
            OperationClass::AuthoritativeRewrite
        } else {
            OperationClass::ReadOnlyAssessment
        },
        preview_only: false,
        namespace_bound,
        policy_allowed: namespace_bound && !legal_hold,
        requires_confirmation: is_destructive,
        local_confirmation,
        force_allowed: is_destructive,
        generation_bound: confirmed_generation,
        generation_matches: !stale,
        snapshot_required,
        snapshot_available: !query.contains("snapshot missing"),
        maintenance_window_required,
        maintenance_window_active,
        dependencies_ready,
        scope_precise: !missing_scope,
        authoritative_input_readable,
        confidence_ready,
        can_degrade,
        legal_hold,
    }
}

#[allow(clippy::too_many_arguments)]
pub fn evaluate_preflight(
    namespace: &str,
    original_query: &str,
    proposed_action: &str,
    request_correlation_id: u64,
    local_confirmation: bool,
    preview_only: bool,
    actor_source: &'static str,
    request_prefix: &str,
) -> Result<EvaluatedPreflight, String> {
    validate_preflight_namespace(namespace)?;
    let _request = PreflightRunRequest {
        namespace: namespace.to_string(),
        original_query: original_query.to_string(),
        proposed_action: proposed_action.to_string(),
    };
    let request_id = if preview_only {
        format!("{request_prefix}-preflight-explain-{request_correlation_id}")
    } else if local_confirmation {
        format!("{request_prefix}-preflight-allow-{request_correlation_id}")
    } else {
        format!("{request_prefix}-preflight-run-{request_correlation_id}")
    };
    let preflight_id = format!("preflight-{request_correlation_id}");
    let mut request = preflight_request(
        namespace,
        original_query,
        proposed_action,
        local_confirmation,
        Some(request_correlation_id),
    );
    request.preview_only = preview_only;
    let mut outcome = PolicyModule.evaluate_safeguard(request);
    outcome.audit.actor_source = actor_source;
    outcome.audit.request_id = Box::leak(request_id.clone().into_boxed_str());
    outcome.audit.preview_id = Some(Box::leak(preflight_id.clone().into_boxed_str()));
    Ok(EvaluatedPreflight {
        request_id,
        preflight_id,
        execution_id: (!preview_only
            && matches!(
                outcome.outcome_class,
                OutcomeClass::Accepted | OutcomeClass::Degraded
            ))
        .then(|| format!("exec-{request_correlation_id}")),
        outcome,
    })
}

pub fn check_status_label(status: PreflightCheckStatus) -> String {
    match status {
        PreflightCheckStatus::Passed => "passed",
        PreflightCheckStatus::Blocked => "blocked",
        PreflightCheckStatus::Degraded => "degraded",
        PreflightCheckStatus::Rejected => "rejected",
    }
    .to_string()
}

pub fn reason_code_label(reason: SafeguardReasonCode) -> String {
    reason.as_str().to_string()
}

pub fn check_view(check: &PreflightCheck) -> PreflightCheckView {
    PreflightCheckView {
        check_name: check.check_name.to_string(),
        status: check_status_label(check.status),
        reason_codes: check
            .reason_codes
            .iter()
            .copied()
            .map(reason_code_label)
            .collect(),
        checked_scope: check.checked_scope.to_string(),
    }
}

pub fn preflight_state_label(state: PreflightState) -> &'static str {
    match state {
        PreflightState::Ready => "ready",
        PreflightState::Blocked => "blocked",
        PreflightState::MissingData => "missing_data",
        PreflightState::StaleKnowledge => "stale_knowledge",
    }
}

pub fn policy_decision_label(decision: PolicyDecision) -> &'static str {
    decision.as_str()
}

pub fn preflight_outcome_label(
    outcome: &SafeguardOutcome,
    local_confirmation: bool,
) -> &'static str {
    if local_confirmation
        && matches!(
            outcome.outcome_class,
            OutcomeClass::Accepted | OutcomeClass::Degraded
        )
    {
        "force_confirmed"
    } else {
        match outcome.outcome_class {
            OutcomeClass::Preview => "preview_only",
            OutcomeClass::Blocked => "blocked",
            OutcomeClass::Degraded => "degraded",
            OutcomeClass::Accepted => "ready",
            OutcomeClass::Rejected => "blocked",
            OutcomeClass::Partial => "degraded",
        }
    }
}

pub fn confirmation_reason(outcome: &SafeguardOutcome, local_confirmation: bool) -> Option<String> {
    (local_confirmation
        && matches!(
            outcome.outcome_class,
            OutcomeClass::Accepted | OutcomeClass::Degraded
        ))
    .then_some("operator confirmed exact previewed scope".to_string())
}

pub fn blocked_reason_message(
    outcome: &SafeguardOutcome,
    confirmation_pending: bool,
) -> Option<String> {
    if confirmation_pending {
        return Some(
            SafeguardReasonCode::ConfirmationRequired
                .operator_message()
                .to_string(),
        );
    }

    outcome.operator_message()
}

pub fn required_overrides(outcome: &SafeguardOutcome) -> Vec<String> {
    let mut overrides = Vec::new();
    if outcome
        .blocked_reasons
        .contains(&SafeguardReasonCode::ConfirmationRequired)
    {
        overrides.push("human_confirmation".to_string());
    }
    overrides
}

pub fn policy_context(namespace: &str, outcome: &SafeguardOutcome) -> String {
    let action = match outcome.operation_class {
        OperationClass::ReadOnlyAssessment => "read-only assessment",
        OperationClass::DerivedSurfaceMutation => "derived surface mutation",
        OperationClass::AuthoritativeRewrite => "authoritative rewrite",
        OperationClass::IrreversibleMutation => "irreversible mutation",
        OperationClass::ContradictionArchive => "contradiction archive",
    };
    format!(
        "namespace {namespace} preflight safeguard evaluation ({action}; {})",
        outcome.impact_summary
    )
}

pub fn to_preflight_outcome(
    evaluated: EvaluatedPreflight,
    local_confirmation: bool,
) -> PreflightOutcome {
    let outcome = evaluated.outcome;
    let blocked_reasons = outcome
        .blocked_reasons
        .iter()
        .copied()
        .map(reason_code_label)
        .collect::<Vec<_>>();
    let outcome_class = outcome.outcome_class.as_str().to_string();
    let degraded = matches!(outcome.outcome_class, OutcomeClass::Degraded);
    let success = matches!(
        outcome.outcome_class,
        OutcomeClass::Accepted | OutcomeClass::Degraded
    );
    PreflightOutcome {
        success,
        preflight_state: preflight_state_label(outcome.preflight_state).to_string(),
        preflight_outcome: preflight_outcome_label(&outcome, local_confirmation).to_string(),
        outcome_class,
        blocked_reasons,
        check_results: outcome.check_results.iter().map(check_view).collect(),
        confirmation: ConfirmationView {
            required: outcome.confirmation.required,
            force_allowed: outcome.confirmation.force_allowed,
            confirmed: outcome.confirmation.confirmed,
            generation_bound: outcome.confirmation.generation_bound,
        },
        audit: AuditView {
            event_kind: outcome.audit.event_kind.to_string(),
            actor_source: outcome.audit.actor_source.to_string(),
            request_id: outcome.audit.request_id.to_string(),
            preview_id: outcome.audit.preview_id.map(str::to_string),
            related_run: outcome.audit.related_run.map(str::to_string),
            scope_handle: outcome.audit.scope_handle.to_string(),
        },
        policy_summary: PolicySummaryView {
            decision: policy_decision_label(outcome.policy_summary.decision).to_string(),
            namespace_bound: outcome.policy_summary.namespace_bound,
            outcome_class: outcome.policy_summary.outcome_class.as_str().to_string(),
        },
        request_id: Some(evaluated.request_id),
        preflight_id: Some(evaluated.preflight_id),
        execution_id: evaluated.execution_id,
        degraded,
        confirmation_reason: confirmation_reason(&outcome, local_confirmation),
    }
}

pub fn to_preflight_explain_response(
    namespace: &str,
    evaluated: EvaluatedPreflight,
) -> PreflightExplainResponse {
    let outcome = evaluated.outcome;
    let mut blocked_reasons = outcome
        .blocked_reasons
        .iter()
        .copied()
        .map(reason_code_label)
        .collect::<Vec<_>>();
    if outcome.confirmation.required && !outcome.confirmation.confirmed {
        blocked_reasons.push("confirmation_required".to_string());
    }
    let allowed = false;
    let preflight_state = if blocked_reasons.is_empty() {
        preflight_state_label(outcome.preflight_state).to_string()
    } else {
        "blocked".to_string()
    };
    PreflightExplainResponse {
        allowed,
        preflight_state,
        preflight_outcome: preflight_outcome_label(&outcome, false).to_string(),
        blocked_reasons: blocked_reasons.clone(),
        blocked_reason: blocked_reason_message(
            &outcome,
            outcome.confirmation.required && !outcome.confirmation.confirmed,
        ),
        required_overrides: if outcome.confirmation.required && !outcome.confirmation.confirmed {
            vec!["human_confirmation".to_string()]
        } else {
            required_overrides(&outcome)
        },
        policy_context: policy_context(namespace, &outcome),
        check_results: outcome.check_results.iter().map(check_view).collect(),
        confirmation: ConfirmationView {
            required: outcome.confirmation.required,
            force_allowed: outcome.confirmation.force_allowed,
            confirmed: outcome.confirmation.confirmed,
            generation_bound: outcome.confirmation.generation_bound,
        },
        audit: AuditView {
            event_kind: outcome.audit.event_kind.to_string(),
            actor_source: outcome.audit.actor_source.to_string(),
            request_id: outcome.audit.request_id.to_string(),
            preview_id: outcome.audit.preview_id.map(str::to_string),
            related_run: outcome.audit.related_run.map(str::to_string),
            scope_handle: outcome.audit.scope_handle.to_string(),
        },
        policy_summary: PolicySummaryView {
            decision: policy_decision_label(outcome.policy_summary.decision).to_string(),
            namespace_bound: outcome.policy_summary.namespace_bound,
            outcome_class: outcome.policy_summary.outcome_class.as_str().to_string(),
        },
        request_id: Some(evaluated.request_id),
        preflight_id: Some(evaluated.preflight_id),
    }
}

#[allow(clippy::too_many_arguments)]
pub fn preflight_allow(
    namespace: &str,
    original_query: &str,
    proposed_action: &str,
    authorization_token: &str,
    bypass_flags: &[String],
    request_correlation_id: u64,
    actor_source: &'static str,
    request_prefix: &str,
) -> Result<PreflightOutcome, String> {
    validate_preflight_namespace(namespace)?;
    let _request = PreflightAllowRequest {
        namespace: namespace.to_string(),
        original_query: original_query.to_string(),
        proposed_action: proposed_action.to_string(),
        authorization_token: authorization_token.to_string(),
        bypass_flags: bypass_flags.to_vec(),
    };
    let confirmed = authorization_token.starts_with("allow-")
        && bypass_flags.iter().any(|flag| flag == "manual_override");
    let evaluated = evaluate_preflight(
        namespace,
        original_query,
        proposed_action,
        request_correlation_id,
        confirmed,
        false,
        actor_source,
        request_prefix,
    )?;
    Ok(to_preflight_outcome(evaluated, confirmed))
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn sample_check() -> PreflightCheckView {
        PreflightCheckView {
            check_name: "policy".to_string(),
            status: "passed".to_string(),
            reason_codes: Vec::new(),
            checked_scope: "effective_namespace".to_string(),
        }
    }

    fn sample_confirmation() -> ConfirmationView {
        ConfirmationView {
            required: true,
            force_allowed: true,
            confirmed: false,
            generation_bound: Some(42),
        }
    }

    fn sample_audit() -> AuditView {
        AuditView {
            event_kind: "safeguard_evaluation".to_string(),
            actor_source: "daemon_jsonrpc".to_string(),
            request_id: "daemon-preflight-run-42".to_string(),
            preview_id: Some("preflight-42".to_string()),
            related_run: Some("irreversible-mutation-run".to_string()),
            scope_handle: "effective_namespace".to_string(),
        }
    }

    fn sample_policy_summary() -> PolicySummaryView {
        PolicySummaryView {
            decision: "allow".to_string(),
            namespace_bound: true,
            outcome_class: "accepted".to_string(),
        }
    }

    fn blocked_check(reason_code: &str, checked_scope: &str) -> PreflightCheckView {
        PreflightCheckView {
            check_name: reason_code.to_string(),
            status: "blocked".to_string(),
            reason_codes: vec![reason_code.to_string()],
            checked_scope: checked_scope.to_string(),
        }
    }

    fn blocked_policy_summary(reason_code: &str) -> PolicySummaryView {
        PolicySummaryView {
            decision: if reason_code == "policy_denied" {
                "deny".to_string()
            } else {
                "allow".to_string()
            },
            namespace_bound: true,
            outcome_class: "blocked".to_string(),
        }
    }

    fn blocked_explain_fixture(
        reason_code: &str,
        blocked_reason: &str,
        checked_scope: &str,
    ) -> PreflightExplainResponse {
        PreflightExplainResponse {
            allowed: false,
            preflight_state: "blocked".to_string(),
            preflight_outcome: "blocked".to_string(),
            blocked_reasons: vec![reason_code.to_string()],
            blocked_reason: Some(blocked_reason.to_string()),
            required_overrides: Vec::new(),
            policy_context: format!(
                "force-confirm remains blocked for {reason_code} on {checked_scope}"
            ),
            check_results: vec![blocked_check(reason_code, checked_scope)],
            confirmation: ConfirmationView {
                required: true,
                force_allowed: true,
                confirmed: true,
                generation_bound: Some(42),
            },
            audit: AuditView {
                request_id: format!("daemon-preflight-explain-{reason_code}"),
                preview_id: Some(format!("preflight-{reason_code}")),
                related_run: Some(format!("blocked-run-{reason_code}")),
                ..sample_audit()
            },
            policy_summary: blocked_policy_summary(reason_code),
            request_id: Some(format!("req-{reason_code}")),
            preflight_id: Some(format!("preflight-{reason_code}")),
        }
    }

    fn blocked_outcome_fixture(
        reason_code: &str,
        _blocked_reason: &str,
        checked_scope: &str,
    ) -> PreflightOutcome {
        PreflightOutcome {
            success: false,
            preflight_state: "blocked".to_string(),
            preflight_outcome: "blocked".to_string(),
            outcome_class: "blocked".to_string(),
            blocked_reasons: vec![reason_code.to_string()],
            check_results: vec![blocked_check(reason_code, checked_scope)],
            confirmation: ConfirmationView {
                required: true,
                force_allowed: true,
                confirmed: true,
                generation_bound: Some(42),
            },
            audit: AuditView {
                request_id: format!("daemon-preflight-allow-{reason_code}"),
                preview_id: Some(format!("preflight-{reason_code}")),
                related_run: Some(format!("blocked-run-{reason_code}")),
                ..sample_audit()
            },
            policy_summary: blocked_policy_summary(reason_code),
            request_id: Some(format!("req-{reason_code}")),
            preflight_id: Some(format!("preflight-{reason_code}")),
            execution_id: None,
            degraded: false,
            confirmation_reason: None,
        }
    }

    #[test]
    fn preflight_run_request_round_trips_with_canonical_fields() {
        let request = PreflightRunRequest {
            namespace: "tenant.alpha".to_string(),
            original_query: "delete all prior audit events".to_string(),
            proposed_action: "purge namespace audit history".to_string(),
        };

        let value = serde_json::to_value(&request).unwrap();
        assert_eq!(
            value,
            json!({
                "namespace": "tenant.alpha",
                "original_query": "delete all prior audit events",
                "proposed_action": "purge namespace audit history"
            })
        );

        let decoded: PreflightRunRequest = serde_json::from_value(value).unwrap();
        assert_eq!(decoded.namespace, "tenant.alpha");
        assert_eq!(decoded.original_query, "delete all prior audit events");
        assert_eq!(decoded.proposed_action, "purge namespace audit history");
    }

    #[test]
    fn preflight_explain_response_preserves_parity_fields() {
        let response = PreflightExplainResponse {
            allowed: false,
            preflight_state: "blocked".to_string(),
            preflight_outcome: "blocked".to_string(),
            blocked_reasons: vec!["policy_denied".to_string()],
            blocked_reason: Some("policy denied destructive action".to_string()),
            required_overrides: vec!["human_confirmation".to_string(), "audit_ticket".to_string()],
            policy_context: "destructive maintenance command".to_string(),
            check_results: vec![sample_check()],
            confirmation: sample_confirmation(),
            audit: sample_audit(),
            policy_summary: sample_policy_summary(),
            request_id: Some("req-42".to_string()),
            preflight_id: Some("preflight-42".to_string()),
        };

        let value = serde_json::to_value(&response).unwrap();
        assert_eq!(
            value,
            json!({
                "allowed": false,
                "preflight_state": "blocked",
                "preflight_outcome": "blocked",
                "blocked_reasons": ["policy_denied"],
                "blocked_reason": "policy denied destructive action",
                "required_overrides": ["human_confirmation", "audit_ticket"],
                "policy_context": "destructive maintenance command",
                "check_results": [{
                    "check_name": "policy",
                    "status": "passed",
                    "reason_codes": [],
                    "checked_scope": "effective_namespace"
                }],
                "confirmation": {
                    "required": true,
                    "force_allowed": true,
                    "confirmed": false,
                    "generation_bound": 42
                },
                "audit": {
                    "event_kind": "safeguard_evaluation",
                    "actor_source": "daemon_jsonrpc",
                    "request_id": "daemon-preflight-run-42",
                    "preview_id": "preflight-42",
                    "related_run": "irreversible-mutation-run",
                    "scope_handle": "effective_namespace"
                },
                "policy_summary": {
                    "decision": "allow",
                    "namespace_bound": true,
                    "outcome_class": "accepted"
                },
                "request_id": "req-42",
                "preflight_id": "preflight-42"
            })
        );

        let decoded: PreflightExplainResponse = serde_json::from_value(value).unwrap();
        assert!(!decoded.allowed);
        assert_eq!(decoded.preflight_outcome, "blocked");
        assert_eq!(decoded.check_results, vec![sample_check()]);
        assert_eq!(decoded.confirmation, sample_confirmation());
        assert_eq!(decoded.audit, sample_audit());
        assert_eq!(decoded.policy_summary, sample_policy_summary());
    }

    #[test]
    fn preflight_allow_and_outcome_round_trip_optional_fields() {
        let allow_request = PreflightAllowRequest {
            namespace: "tenant.alpha".to_string(),
            original_query: "delete stale archive".to_string(),
            proposed_action: "hard_delete".to_string(),
            authorization_token: "token-123".to_string(),
            bypass_flags: vec!["manual_override".to_string()],
        };
        let outcome = PreflightOutcome {
            success: true,
            preflight_state: "ready".to_string(),
            preflight_outcome: "force_confirmed".to_string(),
            outcome_class: "accepted".to_string(),
            blocked_reasons: Vec::new(),
            check_results: vec![sample_check()],
            confirmation: ConfirmationView {
                confirmed: true,
                ..sample_confirmation()
            },
            audit: sample_audit(),
            policy_summary: sample_policy_summary(),
            request_id: Some("req-42".to_string()),
            preflight_id: Some("preflight-42".to_string()),
            execution_id: Some("exec-42".to_string()),
            degraded: true,
            confirmation_reason: Some("operator approved fallback mode".to_string()),
        };

        let allow_value = serde_json::to_value(&allow_request).unwrap();
        assert_eq!(
            allow_value,
            json!({
                "namespace": "tenant.alpha",
                "original_query": "delete stale archive",
                "proposed_action": "hard_delete",
                "authorization_token": "token-123",
                "bypass_flags": ["manual_override"]
            })
        );
        let outcome_value = serde_json::to_value(&outcome).unwrap();
        assert_eq!(
            outcome_value,
            json!({
                "success": true,
                "preflight_state": "ready",
                "preflight_outcome": "force_confirmed",
                "outcome_class": "accepted",
                "blocked_reasons": [],
                "check_results": [{
                    "check_name": "policy",
                    "status": "passed",
                    "reason_codes": [],
                    "checked_scope": "effective_namespace"
                }],
                "confirmation": {
                    "required": true,
                    "force_allowed": true,
                    "confirmed": true,
                    "generation_bound": 42
                },
                "audit": {
                    "event_kind": "safeguard_evaluation",
                    "actor_source": "daemon_jsonrpc",
                    "request_id": "daemon-preflight-run-42",
                    "preview_id": "preflight-42",
                    "related_run": "irreversible-mutation-run",
                    "scope_handle": "effective_namespace"
                },
                "policy_summary": {
                    "decision": "allow",
                    "namespace_bound": true,
                    "outcome_class": "accepted"
                },
                "request_id": "req-42",
                "preflight_id": "preflight-42",
                "execution_id": "exec-42",
                "degraded": true,
                "confirmation_reason": "operator approved fallback mode"
            })
        );

        let decoded_allow: PreflightAllowRequest = serde_json::from_value(allow_value).unwrap();
        let decoded_outcome: PreflightOutcome = serde_json::from_value(outcome_value).unwrap();

        assert_eq!(decoded_allow.namespace, "tenant.alpha");
        assert_eq!(decoded_allow.original_query, "delete stale archive");
        assert_eq!(decoded_allow.proposed_action, "hard_delete");
        assert_eq!(decoded_allow.authorization_token, "token-123");
        assert_eq!(decoded_allow.bypass_flags, vec!["manual_override"]);
        assert!(decoded_outcome.success);
        assert_eq!(decoded_outcome.preflight_outcome, "force_confirmed");
        assert_eq!(decoded_outcome.outcome_class, "accepted");
        assert_eq!(decoded_outcome.check_results, vec![sample_check()]);
        assert!(decoded_outcome.confirmation.confirmed);
    }

    #[test]
    fn preflight_responses_omit_missing_optional_fields_but_still_round_trip() {
        let explain = PreflightExplainResponse {
            allowed: false,
            preflight_state: "ready".to_string(),
            preflight_outcome: "preview_only".to_string(),
            blocked_reasons: Vec::new(),
            blocked_reason: None,
            required_overrides: Vec::new(),
            policy_context: "safe read-only preview".to_string(),
            check_results: vec![sample_check()],
            confirmation: sample_confirmation(),
            audit: AuditView {
                preview_id: None,
                related_run: None,
                ..sample_audit()
            },
            policy_summary: sample_policy_summary(),
            request_id: None,
            preflight_id: None,
        };
        let outcome = PreflightOutcome {
            success: false,
            preflight_state: "blocked".to_string(),
            preflight_outcome: "blocked".to_string(),
            outcome_class: "blocked".to_string(),
            blocked_reasons: vec!["legal_hold".to_string()],
            check_results: vec![sample_check()],
            confirmation: sample_confirmation(),
            audit: AuditView {
                preview_id: None,
                related_run: None,
                ..sample_audit()
            },
            policy_summary: sample_policy_summary(),
            request_id: None,
            preflight_id: None,
            execution_id: None,
            degraded: false,
            confirmation_reason: None,
        };

        let explain_value = serde_json::to_value(&explain).unwrap();
        let outcome_value = serde_json::to_value(&outcome).unwrap();

        assert!(explain_value.get("blocked_reason").is_none());
        assert!(explain_value.get("request_id").is_none());
        assert!(explain_value.get("preflight_id").is_none());
        assert!(explain_value["audit"].get("preview_id").is_none());
        assert!(outcome_value.get("request_id").is_none());
        assert!(outcome_value.get("preflight_id").is_none());
        assert!(outcome_value.get("execution_id").is_none());
        assert!(outcome_value.get("confirmation_reason").is_none());
        assert!(outcome_value["audit"].get("preview_id").is_none());

        let decoded_explain: PreflightExplainResponse =
            serde_json::from_value(explain_value).unwrap();
        let decoded_outcome: PreflightOutcome = serde_json::from_value(outcome_value).unwrap();

        assert_eq!(decoded_explain.preflight_outcome, "preview_only");
        assert!(decoded_explain.request_id.is_none());
        assert!(decoded_outcome.preflight_id.is_none());
        assert!(decoded_outcome.execution_id.is_none());
        assert_eq!(decoded_outcome.blocked_reasons, vec!["legal_hold"]);
    }

    #[test]
    fn preflight_contract_round_trips_missing_data_and_degraded_states() {
        let explain = PreflightExplainResponse {
            allowed: false,
            preflight_state: "missing_data".to_string(),
            preflight_outcome: "blocked".to_string(),
            blocked_reasons: vec!["snapshot_required".to_string()],
            blocked_reason: Some("snapshot is required before this action can proceed".to_string()),
            required_overrides: Vec::new(),
            policy_context: "destructive maintenance command".to_string(),
            check_results: vec![PreflightCheckView {
                check_name: "required_input".to_string(),
                status: "blocked".to_string(),
                reason_codes: vec!["snapshot_required".to_string()],
                checked_scope: "effective_namespace".to_string(),
            }],
            confirmation: sample_confirmation(),
            audit: sample_audit(),
            policy_summary: PolicySummaryView {
                decision: "allow".to_string(),
                namespace_bound: true,
                outcome_class: "blocked".to_string(),
            },
            request_id: Some("req-missing-data".to_string()),
            preflight_id: Some("preflight-missing-data".to_string()),
        };
        let outcome = PreflightOutcome {
            success: true,
            preflight_state: "stale_knowledge".to_string(),
            preflight_outcome: "degraded".to_string(),
            outcome_class: "degraded".to_string(),
            blocked_reasons: vec!["authoritative_input_unreadable".to_string()],
            check_results: vec![PreflightCheckView {
                check_name: "freshness".to_string(),
                status: "degraded".to_string(),
                reason_codes: vec!["authoritative_input_unreadable".to_string()],
                checked_scope: "effective_namespace".to_string(),
            }],
            confirmation: ConfirmationView {
                required: false,
                force_allowed: false,
                confirmed: false,
                generation_bound: Some(7),
            },
            audit: sample_audit(),
            policy_summary: PolicySummaryView {
                decision: "allow".to_string(),
                namespace_bound: true,
                outcome_class: "degraded".to_string(),
            },
            request_id: Some("req-stale-knowledge".to_string()),
            preflight_id: Some("preflight-stale-knowledge".to_string()),
            execution_id: Some("exec-stale-knowledge".to_string()),
            degraded: true,
            confirmation_reason: None,
        };

        let explain_value = serde_json::to_value(&explain).unwrap();
        let outcome_value = serde_json::to_value(&outcome).unwrap();
        let decoded_explain: PreflightExplainResponse =
            serde_json::from_value(explain_value.clone()).unwrap();
        let decoded_outcome: PreflightOutcome =
            serde_json::from_value(outcome_value.clone()).unwrap();

        assert_eq!(explain_value["preflight_state"], json!("missing_data"));
        assert_eq!(
            explain_value["blocked_reasons"],
            json!(["snapshot_required"])
        );
        assert_eq!(decoded_explain.preflight_state, "missing_data");
        assert_eq!(decoded_explain.check_results[0].status, "blocked");

        assert_eq!(outcome_value["preflight_state"], json!("stale_knowledge"));
        assert_eq!(outcome_value["preflight_outcome"], json!("degraded"));
        assert_eq!(outcome_value["outcome_class"], json!("degraded"));
        assert_eq!(decoded_outcome.preflight_state, "stale_knowledge");
        assert_eq!(decoded_outcome.preflight_outcome, "degraded");
        assert_eq!(decoded_outcome.policy_summary.outcome_class, "degraded");
        assert!(decoded_outcome.degraded);
    }

    #[test]
    fn force_confirm_blockers_preserve_blocked_serialization_contract() {
        let cases = [
            (
                "policy_denied",
                "policy denied the requested action",
                "effective_namespace",
                json!(true),
            ),
            (
                "scope_ambiguous",
                "requested scope is ambiguous",
                "requested_scope",
                json!(true),
            ),
            (
                "snapshot_required",
                "snapshot is required before this action can proceed",
                "effective_namespace",
                json!(true),
            ),
            (
                "legal_hold",
                "legal hold blocks the requested action",
                "effective_namespace",
                json!(true),
            ),
        ];

        for (reason_code, blocked_reason, checked_scope, namespace_bound) in cases {
            let explain = blocked_explain_fixture(reason_code, blocked_reason, checked_scope);
            let outcome = blocked_outcome_fixture(reason_code, blocked_reason, checked_scope);

            let explain_value = serde_json::to_value(&explain).unwrap();
            let outcome_value = serde_json::to_value(&outcome).unwrap();

            assert_eq!(explain_value["allowed"], json!(false), "{reason_code}");
            assert_eq!(
                explain_value["preflight_state"],
                json!("blocked"),
                "{reason_code}"
            );
            assert_eq!(
                explain_value["preflight_outcome"],
                json!("blocked"),
                "{reason_code}"
            );
            assert_eq!(
                explain_value["blocked_reasons"],
                json!([reason_code]),
                "{reason_code}"
            );
            assert_eq!(
                explain_value["blocked_reason"],
                json!(blocked_reason),
                "{reason_code}"
            );
            assert_eq!(
                explain_value["required_overrides"],
                json!([]),
                "{reason_code}"
            );
            assert_eq!(
                explain_value["check_results"][0]["status"],
                json!("blocked"),
                "{reason_code}"
            );
            assert_eq!(
                explain_value["check_results"][0]["reason_codes"],
                json!([reason_code]),
                "{reason_code}"
            );
            assert_eq!(
                explain_value["check_results"][0]["checked_scope"],
                json!(checked_scope),
                "{reason_code}"
            );
            assert_eq!(
                explain_value["confirmation"]["required"],
                json!(true),
                "{reason_code}"
            );
            assert_eq!(
                explain_value["confirmation"]["force_allowed"],
                json!(true),
                "{reason_code}"
            );
            assert_eq!(
                explain_value["confirmation"]["confirmed"],
                json!(true),
                "{reason_code}"
            );
            assert_eq!(
                explain_value["policy_summary"]["decision"],
                if reason_code == "policy_denied" {
                    json!("deny")
                } else {
                    json!("allow")
                },
                "{reason_code}"
            );
            assert_eq!(
                explain_value["policy_summary"]["namespace_bound"], namespace_bound,
                "{reason_code}"
            );
            assert_eq!(
                explain_value["policy_summary"]["outcome_class"],
                json!("blocked"),
                "{reason_code}"
            );

            assert_eq!(outcome_value["success"], json!(false), "{reason_code}");
            assert_eq!(
                outcome_value["preflight_state"],
                json!("blocked"),
                "{reason_code}"
            );
            assert_eq!(
                outcome_value["preflight_outcome"],
                json!("blocked"),
                "{reason_code}"
            );
            assert_eq!(
                outcome_value["outcome_class"],
                json!("blocked"),
                "{reason_code}"
            );
            assert_eq!(
                outcome_value["blocked_reasons"],
                json!([reason_code]),
                "{reason_code}"
            );
            assert_eq!(
                outcome_value["check_results"][0]["status"],
                json!("blocked"),
                "{reason_code}"
            );
            assert_eq!(
                outcome_value["check_results"][0]["reason_codes"],
                json!([reason_code]),
                "{reason_code}"
            );
            assert_eq!(
                outcome_value["check_results"][0]["checked_scope"],
                json!(checked_scope),
                "{reason_code}"
            );
            assert_eq!(
                outcome_value["confirmation"]["required"],
                json!(true),
                "{reason_code}"
            );
            assert_eq!(
                outcome_value["confirmation"]["force_allowed"],
                json!(true),
                "{reason_code}"
            );
            assert_eq!(
                outcome_value["confirmation"]["confirmed"],
                json!(true),
                "{reason_code}"
            );
            assert_eq!(
                outcome_value["policy_summary"]["decision"],
                if reason_code == "policy_denied" {
                    json!("deny")
                } else {
                    json!("allow")
                },
                "{reason_code}"
            );
            assert_eq!(
                outcome_value["policy_summary"]["namespace_bound"], namespace_bound,
                "{reason_code}"
            );
            assert_eq!(
                outcome_value["policy_summary"]["outcome_class"],
                json!("blocked"),
                "{reason_code}"
            );
            assert!(outcome_value.get("execution_id").is_none(), "{reason_code}");
            assert!(
                outcome_value.get("confirmation_reason").is_none(),
                "{reason_code}"
            );

            let decoded_explain: PreflightExplainResponse =
                serde_json::from_value(explain_value).unwrap();
            let decoded_outcome: PreflightOutcome = serde_json::from_value(outcome_value).unwrap();
            assert!(!decoded_explain.allowed, "{reason_code}");
            assert_eq!(
                decoded_explain.blocked_reasons,
                vec![reason_code],
                "{reason_code}"
            );
            assert_eq!(
                decoded_explain.blocked_reason.as_deref(),
                Some(blocked_reason),
                "{reason_code}"
            );
            assert_eq!(
                decoded_outcome.preflight_outcome, "blocked",
                "{reason_code}"
            );
            assert_eq!(
                decoded_outcome.blocked_reasons,
                vec![reason_code],
                "{reason_code}"
            );
            assert!(decoded_outcome.confirmation.confirmed, "{reason_code}");
            assert_eq!(
                decoded_outcome.policy_summary.outcome_class, "blocked",
                "{reason_code}"
            );
        }
    }

    #[test]
    fn preflight_run_request_rejects_unknown_fields() {
        let error = serde_json::from_value::<PreflightRunRequest>(json!({
            "namespace": "tenant.alpha",
            "original_query": "delete all prior audit events",
            "proposed_action": "purge namespace audit history",
            "unexpected": true
        }))
        .unwrap_err();

        assert!(error.to_string().contains("unknown field `unexpected`"));
    }

    #[test]
    fn preflight_responses_and_allow_requests_reject_unknown_fields() {
        let explain_error = serde_json::from_value::<PreflightExplainResponse>(json!({
            "allowed": false,
            "preflight_state": "blocked",
            "preflight_outcome": "blocked",
            "blocked_reasons": ["confirmation_required"],
            "required_overrides": ["human_confirmation"],
            "policy_context": "namespace tenant.alpha preflight safeguard evaluation",
            "check_results": [],
            "confirmation": {
                "required": true,
                "force_allowed": true,
                "confirmed": false
            },
            "audit": {
                "event_kind": "safeguard_evaluation",
                "actor_source": "daemon_jsonrpc",
                "request_id": "daemon-preflight-run-42",
                "scope_handle": "effective_namespace"
            },
            "policy_summary": {
                "decision": "allow",
                "namespace_bound": true,
                "outcome_class": "accepted"
            },
            "unexpected": true
        }))
        .unwrap_err();
        assert!(explain_error
            .to_string()
            .contains("unknown field `unexpected`"));

        let allow_error = serde_json::from_value::<PreflightAllowRequest>(json!({
            "namespace": "tenant.alpha",
            "original_query": "delete stale archive",
            "proposed_action": "hard_delete",
            "authorization_token": "allow-123",
            "bypass_flags": ["manual_override"],
            "unexpected": true
        }))
        .unwrap_err();
        assert!(allow_error
            .to_string()
            .contains("unknown field `unexpected`"));

        let outcome_error = serde_json::from_value::<PreflightOutcome>(json!({
            "success": true,
            "preflight_state": "ready",
            "preflight_outcome": "force_confirmed",
            "outcome_class": "accepted",
            "blocked_reasons": [],
            "check_results": [],
            "confirmation": {
                "required": true,
                "force_allowed": true,
                "confirmed": true
            },
            "audit": {
                "event_kind": "safeguard_evaluation",
                "actor_source": "daemon_jsonrpc",
                "request_id": "daemon-preflight-run-42",
                "scope_handle": "effective_namespace"
            },
            "policy_summary": {
                "decision": "allow",
                "namespace_bound": true,
                "outcome_class": "accepted"
            },
            "degraded": false,
            "unexpected": true
        }))
        .unwrap_err();
        assert!(outcome_error
            .to_string()
            .contains("unknown field `unexpected`"));
    }
}
