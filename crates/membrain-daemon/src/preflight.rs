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
