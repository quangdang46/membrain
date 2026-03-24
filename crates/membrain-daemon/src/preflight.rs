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
