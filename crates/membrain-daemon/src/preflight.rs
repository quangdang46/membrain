use serde::{Deserialize, Serialize};

/// Preflight safeguard contract and API (`mb-23u.7.4`)
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PreflightRunRequest {
    pub namespace: String,
    pub original_query: String,
    pub proposed_action: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PreflightExplainResponse {
    pub allowed: bool,
    pub preflight_state: String,
    pub blocked_reasons: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub blocked_reason: Option<String>,
    pub required_overrides: Vec<String>,
    pub policy_context: String,
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
    pub blocked_reasons: Vec<String>,
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
    fn preflight_explain_response_preserves_blocked_reason_and_required_overrides() {
        let response = PreflightExplainResponse {
            allowed: false,
            preflight_state: "blocked".to_string(),
            blocked_reasons: vec!["policy_denied".to_string()],
            blocked_reason: Some("policy denied destructive action".to_string()),
            required_overrides: vec!["human_confirmation".to_string(), "audit_ticket".to_string()],
            policy_context: "destructive maintenance command".to_string(),
            request_id: Some("req-42".to_string()),
            preflight_id: Some("preflight-42".to_string()),
        };

        let value = serde_json::to_value(&response).unwrap();
        assert_eq!(
            value,
            json!({
                "allowed": false,
                "preflight_state": "blocked",
                "blocked_reasons": ["policy_denied"],
                "blocked_reason": "policy denied destructive action",
                "required_overrides": ["human_confirmation", "audit_ticket"],
                "policy_context": "destructive maintenance command",
                "request_id": "req-42",
                "preflight_id": "preflight-42"
            })
        );

        let decoded: PreflightExplainResponse = serde_json::from_value(value).unwrap();
        assert!(!decoded.allowed);
        assert_eq!(decoded.preflight_state, "blocked");
        assert_eq!(decoded.blocked_reasons, vec!["policy_denied"]);
        assert_eq!(
            decoded.blocked_reason.as_deref(),
            Some("policy denied destructive action")
        );
        assert_eq!(
            decoded.required_overrides,
            vec!["human_confirmation", "audit_ticket"]
        );
        assert_eq!(decoded.policy_context, "destructive maintenance command");
        assert_eq!(decoded.request_id.as_deref(), Some("req-42"));
        assert_eq!(decoded.preflight_id.as_deref(), Some("preflight-42"));
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
            blocked_reasons: Vec::new(),
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
                "blocked_reasons": [],
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
        assert_eq!(decoded_outcome.preflight_state, "ready");
        assert!(decoded_outcome.blocked_reasons.is_empty());
        assert_eq!(decoded_outcome.request_id.as_deref(), Some("req-42"));
        assert_eq!(
            decoded_outcome.preflight_id.as_deref(),
            Some("preflight-42")
        );
        assert_eq!(decoded_outcome.execution_id.as_deref(), Some("exec-42"));
        assert!(decoded_outcome.degraded);
        assert_eq!(
            decoded_outcome.confirmation_reason.as_deref(),
            Some("operator approved fallback mode")
        );
    }

    #[test]
    fn preflight_responses_omit_missing_optional_fields_but_still_round_trip() {
        let explain = PreflightExplainResponse {
            allowed: true,
            preflight_state: "preview_only".to_string(),
            blocked_reasons: Vec::new(),
            blocked_reason: None,
            required_overrides: Vec::new(),
            policy_context: "safe read-only preview".to_string(),
            request_id: None,
            preflight_id: None,
        };
        let outcome = PreflightOutcome {
            success: false,
            preflight_state: "blocked".to_string(),
            blocked_reasons: vec!["legal_hold".to_string()],
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
        assert!(outcome_value.get("request_id").is_none());
        assert!(outcome_value.get("preflight_id").is_none());
        assert!(outcome_value.get("execution_id").is_none());
        assert!(outcome_value.get("confirmation_reason").is_none());

        let decoded_explain: PreflightExplainResponse =
            serde_json::from_value(explain_value).unwrap();
        let decoded_outcome: PreflightOutcome = serde_json::from_value(outcome_value).unwrap();

        assert!(decoded_explain.blocked_reason.is_none());
        assert!(decoded_explain.request_id.is_none());
        assert!(decoded_explain.preflight_id.is_none());
        assert!(decoded_outcome.request_id.is_none());
        assert!(decoded_outcome.preflight_id.is_none());
        assert!(decoded_outcome.execution_id.is_none());
        assert!(decoded_outcome.confirmation_reason.is_none());
        assert_eq!(decoded_outcome.blocked_reasons, vec!["legal_hold"]);
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
            "blocked_reasons": ["confirmation_required"],
            "required_overrides": ["human_confirmation"],
            "policy_context": "namespace tenant.alpha preflight safeguard evaluation",
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
        assert!(allow_error.to_string().contains("unknown field `unexpected`"));

        let outcome_error = serde_json::from_value::<PreflightOutcome>(json!({
            "success": true,
            "preflight_state": "ready",
            "blocked_reasons": [],
            "degraded": false,
            "unexpected": true
        }))
        .unwrap_err();
        assert!(outcome_error
            .to_string()
            .contains("unknown field `unexpected`"));
    }
}
