use serde::{Deserialize, Serialize};

/// Canonical MCP Request Envelopes
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "method", content = "params")]
pub enum McpRequest {
    #[serde(rename = "encode")]
    Encode(EncodeParams),
    #[serde(rename = "recall")]
    Recall(RecallParams),
    #[serde(rename = "inspect")]
    Inspect(InspectParams),
    #[serde(rename = "explain")]
    Explain(ExplainParams),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EncodeParams {
    pub content: String,
    pub namespace: String,
    pub payload_type: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecallParams {
    pub query: String,
    pub namespace: String,
    pub limit: Option<usize>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InspectParams {
    pub id: u64,
    pub namespace: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExplainParams {
    pub query: String,
    pub namespace: String,
}

/// Canonical MCP Response Envelope
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpResponse {
    pub status: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub payload: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<McpError>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpError {
    pub code: String,
    pub message: String,
    pub is_policy_denial: bool,
}

/// Extensions for MCP Resources (`mb-23u.7.2`)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpResource {
    pub uri: String,
    pub name: String,
    pub mime_type: String,
    pub description: Option<String>,
}
