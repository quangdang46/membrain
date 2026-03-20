use serde::{Deserialize, Serialize};

/// Preflight safeguard contract and API (`mb-23u.7.4`)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PreflightRunRequest {
    pub namespace: String,
    pub original_query: String,
    pub proposed_action: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PreflightExplainResponse {
    pub allowed: bool,
    pub blocked_reason: Option<String>,
    pub required_overrides: Vec<String>,
    pub policy_context: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PreflightAllowRequest {
    pub namespace: String,
    pub authorization_token: String,
    pub bypass_flags: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PreflightOutcome {
    pub success: bool,
    pub execution_id: Option<String>,
    pub degraded: bool,
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
            blocked_reason: Some("policy denied destructive action".to_string()),
            required_overrides: vec!["human_confirmation".to_string(), "audit_ticket".to_string()],
            policy_context: "destructive maintenance command".to_string(),
        };

        let value = serde_json::to_value(&response).unwrap();
        assert_eq!(
            value,
            json!({
                "allowed": false,
                "blocked_reason": "policy denied destructive action",
                "required_overrides": ["human_confirmation", "audit_ticket"],
                "policy_context": "destructive maintenance command"
            })
        );

        let decoded: PreflightExplainResponse = serde_json::from_value(value).unwrap();
        assert!(!decoded.allowed);
        assert_eq!(
            decoded.blocked_reason.as_deref(),
            Some("policy denied destructive action")
        );
        assert_eq!(
            decoded.required_overrides,
            vec!["human_confirmation", "audit_ticket"]
        );
        assert_eq!(decoded.policy_context, "destructive maintenance command");
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
        assert_eq!(decoded_outcome.execution_id.as_deref(), Some("exec-42"));
        assert!(decoded_outcome.degraded);
        assert_eq!(
            decoded_outcome.confirmation_reason.as_deref(),
            Some("operator approved fallback mode")
        );
    }
}
