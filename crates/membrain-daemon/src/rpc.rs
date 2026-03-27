use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

/// Lightweight JSON-RPC envelope.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JsonRpcRequest {
    pub jsonrpc: String,
    pub method: String,
    pub params: Option<serde_json::Value>,
    pub id: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JsonRpcResponse {
    pub jsonrpc: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub result: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<JsonRpcError>,
    pub id: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JsonRpcError {
    pub code: i32,
    pub message: String,
    pub data: Option<serde_json::Value>,
}

impl JsonRpcResponse {
    pub fn success(id: Option<Value>, result: Value) -> Self {
        Self {
            jsonrpc: "2.0".to_string(),
            result: Some(result),
            error: None,
            id,
        }
    }

    pub fn error(
        id: Option<Value>,
        code: i32,
        message: impl Into<String>,
        data: Option<Value>,
    ) -> Self {
        Self {
            jsonrpc: "2.0".to_string(),
            result: None,
            error: Some(JsonRpcError {
                code,
                message: message.into(),
                data,
            }),
            id,
        }
    }
}

const COMMON_ENVELOPE_FIELDS: &[&str] = &[
    "request_id",
    "workspace_id",
    "agent_id",
    "session_id",
    "task_id",
    "time_budget_ms",
    "policy_context",
];

const fn default_true() -> bool {
    true
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct RuntimePolicyContextHint {
    #[serde(default)]
    pub include_public: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sharing_visibility: Option<String>,
    #[serde(default = "default_true")]
    pub caller_identity_bound: bool,
    #[serde(default = "default_true")]
    pub workspace_acl_allowed: bool,
    #[serde(default = "default_true")]
    pub agent_acl_allowed: bool,
    #[serde(default = "default_true")]
    pub session_visibility_allowed: bool,
    #[serde(default)]
    pub legal_hold: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
#[serde(deny_unknown_fields)]
pub struct RuntimeCommonFields {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub request_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub workspace_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub agent_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub session_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub task_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub time_budget_ms: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub policy_context: Option<RuntimePolicyContextHint>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum RuntimePosture {
    Full,
    Degraded,
    ReadOnly,
    Offline,
}

impl RuntimePosture {
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::Full => "full",
            Self::Degraded => "degraded",
            Self::ReadOnly => "read_only",
            Self::Offline => "offline",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum RuntimeAuthorityMode {
    UnixSocketDaemon,
    StdioFacade,
}

impl RuntimeAuthorityMode {
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::UnixSocketDaemon => "unix_socket_daemon",
            Self::StdioFacade => "stdio_facade",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct RuntimeMetrics {
    pub queue_depth: usize,
    pub active_requests: usize,
    pub background_jobs: usize,
    pub cancelled_requests: usize,
    pub maintenance_runs: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum RuntimeEmbedderState {
    NotLoaded,
    Loaded,
    Warm,
    Degraded,
    Unavailable,
}

impl RuntimeEmbedderState {
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::NotLoaded => "not_loaded",
            Self::Loaded => "loaded",
            Self::Warm => "warm",
            Self::Degraded => "degraded",
            Self::Unavailable => "unavailable",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct RuntimeEmbedderStatus {
    pub state: RuntimeEmbedderState,
    pub backend_kind: String,
    pub generation: String,
    pub dimensions: usize,
    pub loads: u64,
    pub requests: u64,
    pub cache_hits: u64,
    pub cache_misses: u64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub last_error: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct RuntimeStatus {
    pub posture: RuntimePosture,
    pub authority_mode: RuntimeAuthorityMode,
    pub authoritative_runtime: bool,
    pub maintenance_active: bool,
    pub warm_runtime_guarantees: Vec<String>,
    pub degraded_reasons: Vec<String>,
    pub metrics: RuntimeMetrics,
    pub embedder: RuntimeEmbedderStatus,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct RuntimeDoctorIndex {
    pub family: &'static str,
    pub health: &'static str,
    pub usable: bool,
    pub entry_count: usize,
    pub generation: &'static str,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct RuntimeRepairReport {
    pub target: &'static str,
    pub status: &'static str,
    pub verification_passed: bool,
    pub rebuild_entrypoint: Option<&'static str>,
    pub rebuilt_outputs: Vec<&'static str>,
    pub durable_sources: Vec<&'static str>,
    pub verification_artifact_name: &'static str,
    pub parity_check: &'static str,
    pub authoritative_rows: u64,
    pub derived_rows: u64,
    pub authoritative_generation: &'static str,
    pub derived_generation: &'static str,
    pub affected_item_count: u32,
    pub error_count: u32,
    pub rebuild_duration_ms: u64,
    pub rollback_state: Option<&'static str>,
    pub degraded_mode: Option<&'static str>,
    pub rollback_trigger: Option<&'static str>,
    pub remediation_steps: Vec<&'static str>,
    pub queue_depth_before: u32,
    pub queue_depth_after: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct RuntimeAvailability {
    pub posture: &'static str,
    pub query_capabilities: Vec<&'static str>,
    pub mutation_capabilities: Vec<&'static str>,
    pub degraded_reasons: Vec<&'static str>,
    pub recovery_conditions: Vec<&'static str>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct RuntimeRemediation {
    pub summary: String,
    pub next_steps: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct RuntimeDoctorCheck {
    pub name: &'static str,
    pub surface_kind: &'static str,
    pub status: &'static str,
    pub severity: &'static str,
    pub affected_scope: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub degraded_impact: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub remediation: Option<RuntimeRemediation>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct RuntimeDoctorSummary {
    pub ok_checks: usize,
    pub warn_checks: usize,
    pub fail_checks: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct RuntimeDoctorRunbookHint {
    pub runbook_id: &'static str,
    pub source_doc: &'static str,
    pub section: &'static str,
    pub reason: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct RuntimeDoctorReport {
    pub status: &'static str,
    pub action: &'static str,
    pub posture: RuntimePosture,
    pub degraded_reasons: Vec<String>,
    pub metrics: RuntimeMetrics,
    pub summary: RuntimeDoctorSummary,
    pub indexes: Vec<RuntimeDoctorIndex>,
    pub repair_engine_component: &'static str,
    pub repair_reports: Vec<RuntimeRepairReport>,
    pub checks: Vec<RuntimeDoctorCheck>,
    pub runbook_hints: Vec<RuntimeDoctorRunbookHint>,
    pub warnings: Vec<&'static str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error_kind: Option<&'static str>,
    pub retryable: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub remediation: Option<RuntimeRemediation>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub availability: Option<RuntimeAvailability>,
    pub health: Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RuntimeMaintenanceAccepted {
    pub maintenance_id: u64,
    pub polls_budget: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RuntimeBusy {
    pub queue_depth: usize,
    pub max_queue_depth: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RuntimeCancelled {
    pub reason: &'static str,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RuntimeMethodRequest {
    pub jsonrpc: String,
    pub method: String,
    #[serde(default)]
    pub params: Value,
    pub id: Option<Value>,
}

fn parse_optional_positive_usize(
    params: &Value,
    field: &'static str,
) -> Result<Option<usize>, JsonRpcError> {
    match params.get(field) {
        None | Some(Value::Null) => Ok(None),
        Some(Value::Number(value)) => {
            let parsed = value.as_u64().and_then(|value| usize::try_from(value).ok());
            match parsed {
                Some(0) => Err(JsonRpcError {
                    code: -32602,
                    message: format!("{field} must be at least 1"),
                    data: None,
                }),
                Some(limit) => Ok(Some(limit)),
                None => Err(JsonRpcError {
                    code: -32602,
                    message: format!("{field} must be a positive integer"),
                    data: None,
                }),
            }
        }
        Some(_) => Err(JsonRpcError {
            code: -32602,
            message: format!("{field} must be a positive integer"),
            data: None,
        }),
    }
}

fn parse_optional_limit(params: &Value) -> Result<Option<usize>, JsonRpcError> {
    parse_optional_positive_usize(params, "limit")
}

fn parse_session_handle(params: &Value) -> Result<Option<String>, JsonRpcError> {
    match params.get("session_id") {
        None | Some(Value::Null) => Ok(None),
        Some(Value::String(value)) => Ok(Some(value.clone())),
        Some(Value::Number(value)) => Ok(Some(value.to_string())),
        Some(_) => Err(JsonRpcError {
            code: -32602,
            message: "session_id must be a string or positive integer".to_string(),
            data: None,
        }),
    }
}

fn parse_required_u64(params: &Value, field: &'static str) -> Result<u64, JsonRpcError> {
    match params.get(field) {
        None | Some(Value::Null) => Err(JsonRpcError {
            code: -32602,
            message: format!("missing {field}"),
            data: None,
        }),
        Some(Value::Number(value)) => match value.as_u64() {
            Some(0) => Err(JsonRpcError {
                code: -32602,
                message: format!("{field} must be at least 1"),
                data: None,
            }),
            Some(value) => Ok(value),
            None => Err(JsonRpcError {
                code: -32602,
                message: format!("{field} must be a positive integer"),
                data: None,
            }),
        },
        Some(_) => Err(JsonRpcError {
            code: -32602,
            message: format!("{field} must be a positive integer"),
            data: None,
        }),
    }
}

fn parse_optional_u32(params: &Value, field: &'static str) -> Result<Option<u32>, JsonRpcError> {
    match params.get(field) {
        None | Some(Value::Null) => Ok(None),
        Some(Value::Number(value)) => {
            match value.as_u64().and_then(|value| u32::try_from(value).ok()) {
                Some(0) => Err(JsonRpcError {
                    code: -32602,
                    message: format!("{field} must be at least 1"),
                    data: None,
                }),
                Some(value) => Ok(Some(value)),
                None => Err(JsonRpcError {
                    code: -32602,
                    message: format!("{field} must be a positive integer"),
                    data: None,
                }),
            }
        }
        Some(_) => Err(JsonRpcError {
            code: -32602,
            message: format!("{field} must be a positive integer"),
            data: None,
        }),
    }
}

fn parse_optional_u64(params: &Value, field: &'static str) -> Result<Option<u64>, JsonRpcError> {
    match params.get(field) {
        None | Some(Value::Null) => Ok(None),
        Some(Value::Number(value)) => value.as_u64().map(Some).ok_or_else(|| JsonRpcError {
            code: -32602,
            message: format!("{field} must be a non-negative integer"),
            data: None,
        }),
        Some(_) => Err(JsonRpcError {
            code: -32602,
            message: format!("{field} must be a non-negative integer"),
            data: None,
        }),
    }
}

fn parse_optional_positive_u64(
    params: &Value,
    field: &'static str,
) -> Result<Option<u64>, JsonRpcError> {
    match params.get(field) {
        None | Some(Value::Null) => Ok(None),
        Some(Value::Number(value)) => match value.as_u64() {
            Some(0) => Err(JsonRpcError {
                code: -32602,
                message: format!("{field} must be at least 1"),
                data: None,
            }),
            Some(value) => Ok(Some(value)),
            None => Err(JsonRpcError {
                code: -32602,
                message: format!("{field} must be a positive integer"),
                data: None,
            }),
        },
        Some(_) => Err(JsonRpcError {
            code: -32602,
            message: format!("{field} must be a positive integer"),
            data: None,
        }),
    }
}

fn parse_optional_string(
    params: &Value,
    field: &'static str,
) -> Result<Option<String>, JsonRpcError> {
    match params.get(field) {
        None | Some(Value::Null) => Ok(None),
        Some(Value::String(value)) => Ok(Some(value.clone())),
        Some(_) => Err(JsonRpcError {
            code: -32602,
            message: format!("{field} must be a string"),
            data: None,
        }),
    }
}

fn parse_optional_bool(params: &Value, field: &'static str) -> Result<Option<bool>, JsonRpcError> {
    match params.get(field) {
        None | Some(Value::Null) => Ok(None),
        Some(Value::Bool(value)) => Ok(Some(*value)),
        Some(_) => Err(JsonRpcError {
            code: -32602,
            message: format!("{field} must be a boolean"),
            data: None,
        }),
    }
}

fn parse_optional_f64(params: &Value, field: &'static str) -> Result<Option<f64>, JsonRpcError> {
    match params.get(field) {
        None | Some(Value::Null) => Ok(None),
        Some(Value::Number(value)) => value.as_f64().map(Some).ok_or_else(|| JsonRpcError {
            code: -32602,
            message: format!("{field} must be a number"),
            data: None,
        }),
        Some(_) => Err(JsonRpcError {
            code: -32602,
            message: format!("{field} must be a number"),
            data: None,
        }),
    }
}

fn parse_optional_budget(params: &Value) -> Result<Option<usize>, JsonRpcError> {
    let limit = parse_optional_limit(params)?;
    let result_budget = match params.get("result_budget") {
        None | Some(Value::Null) => None,
        Some(Value::Number(value)) => {
            let parsed = value.as_u64().and_then(|value| usize::try_from(value).ok());
            match parsed {
                Some(0) => {
                    return Err(JsonRpcError {
                        code: -32602,
                        message: "result_budget must be at least 1".to_string(),
                        data: None,
                    });
                }
                Some(value) => Some(value),
                None => {
                    return Err(JsonRpcError {
                        code: -32602,
                        message: "result_budget must be a positive integer".to_string(),
                        data: None,
                    });
                }
            }
        }
        Some(_) => {
            return Err(JsonRpcError {
                code: -32602,
                message: "result_budget must be a positive integer".to_string(),
                data: None,
            });
        }
    };

    match (limit, result_budget) {
        (Some(limit), Some(result_budget)) if limit != result_budget => Err(JsonRpcError {
            code: -32602,
            message: "limit and result_budget must match when both are provided".to_string(),
            data: None,
        }),
        (Some(limit), _) => Ok(Some(limit)),
        (_, Some(result_budget)) => Ok(Some(result_budget)),
        (None, None) => Ok(None),
    }
}

fn parse_required_string(params: &Value, field: &'static str) -> Result<String, JsonRpcError> {
    params
        .get(field)
        .and_then(Value::as_str)
        .map(ToOwned::to_owned)
        .ok_or_else(|| JsonRpcError {
            code: -32602,
            message: format!("missing {field}"),
            data: None,
        })
}

fn parse_string_array(params: &Value, field: &'static str) -> Result<Vec<String>, JsonRpcError> {
    match params.get(field) {
        None | Some(Value::Null) => Ok(Vec::new()),
        Some(Value::Array(items)) => items
            .iter()
            .map(|item| {
                item.as_str()
                    .map(ToOwned::to_owned)
                    .ok_or_else(|| JsonRpcError {
                        code: -32602,
                        message: format!("{field} must be an array of strings"),
                        data: None,
                    })
            })
            .collect(),
        Some(_) => Err(JsonRpcError {
            code: -32602,
            message: format!("{field} must be an array of strings"),
            data: None,
        }),
    }
}

fn reject_unknown_fields(params: &Value, allowed: &[&str]) -> Result<(), JsonRpcError> {
    let Some(object) = params.as_object() else {
        return Ok(());
    };

    if let Some(field) = object
        .keys()
        .find(|field| !allowed.contains(&field.as_str()))
    {
        return Err(JsonRpcError {
            code: -32602,
            message: format!("unknown field {field}"),
            data: None,
        });
    }

    Ok(())
}

fn parse_policy_context(params: &Value) -> Result<Option<RuntimePolicyContextHint>, JsonRpcError> {
    match params.get("policy_context") {
        None | Some(Value::Null) => Ok(None),
        Some(value @ Value::Object(_)) => {
            serde_json::from_value(value.clone())
                .map(Some)
                .map_err(|err| JsonRpcError {
                    code: -32602,
                    message: err.to_string(),
                    data: None,
                })
        }
        Some(_) => Err(JsonRpcError {
            code: -32602,
            message: "policy_context must be a JSON object".to_string(),
            data: None,
        }),
    }
}

fn parse_common_fields(params: &Value) -> Result<RuntimeCommonFields, JsonRpcError> {
    Ok(RuntimeCommonFields {
        request_id: parse_optional_string(params, "request_id")?,
        workspace_id: parse_optional_string(params, "workspace_id")?,
        agent_id: parse_optional_string(params, "agent_id")?,
        session_id: parse_session_handle(params)?,
        task_id: parse_optional_string(params, "task_id")?,
        time_budget_ms: parse_optional_u32(params, "time_budget_ms")?,
        policy_context: parse_policy_context(params)?,
    })
}

impl RuntimeMethodRequest {
    pub fn parse_method(&self) -> Result<RuntimeRequest, JsonRpcError> {
        if self.jsonrpc != "2.0" {
            return Err(JsonRpcError {
                code: -32600,
                message: "unsupported jsonrpc version".to_string(),
                data: Some(json!({ "expected": "2.0", "actual": self.jsonrpc })),
            });
        }

        if !self.params.is_object() && !self.params.is_null() {
            return Err(JsonRpcError {
                code: -32602,
                message: "params must be a JSON object".to_string(),
                data: None,
            });
        }

        match self.method.as_str() {
            "ping" => Ok(RuntimeRequest::Ping),
            "status" => Ok(RuntimeRequest::Status),
            "doctor" => {
                reject_unknown_fields(&self.params, COMMON_ENVELOPE_FIELDS)?;
                let _common = parse_common_fields(&self.params)?;
                let _ = _common;
                Ok(RuntimeRequest::Doctor)
            }
            "health" => {
                reject_unknown_fields(&self.params, COMMON_ENVELOPE_FIELDS)?;
                let _common = parse_common_fields(&self.params)?;
                let _ = _common;
                Ok(RuntimeRequest::Health)
            }
            "encode" => {
                let mut allowed = vec![
                    "content",
                    "namespace",
                    "memory_type",
                    "visibility",
                    "emotional_annotations",
                ];
                allowed.extend_from_slice(COMMON_ENVELOPE_FIELDS);
                reject_unknown_fields(&self.params, &allowed)?;
                let common = parse_common_fields(&self.params)?;
                let content = parse_required_string(&self.params, "content")?;
                let namespace = parse_required_string(&self.params, "namespace")?;
                let memory_type = self
                    .params
                    .get("memory_type")
                    .and_then(Value::as_str)
                    .map(ToOwned::to_owned);
                let visibility = parse_optional_string(&self.params, "visibility")?;
                let emotional_annotations = self.params.get("emotional_annotations").cloned();
                Ok(RuntimeRequest::Encode {
                    content,
                    namespace,
                    memory_type,
                    visibility,
                    emotional_annotations,
                    common,
                })
            }
            "observe" => {
                let mut allowed = vec![
                    "content",
                    "namespace",
                    "context",
                    "chunk_size",
                    "source_label",
                    "topic_threshold",
                    "min_chunk_size",
                    "dry_run",
                ];
                allowed.extend_from_slice(COMMON_ENVELOPE_FIELDS);
                reject_unknown_fields(&self.params, &allowed)?;
                let common = parse_common_fields(&self.params)?;
                let content = parse_required_string(&self.params, "content")?;
                let namespace = parse_required_string(&self.params, "namespace")?;
                let context = parse_optional_string(&self.params, "context")?;
                let chunk_size = parse_optional_positive_usize(&self.params, "chunk_size")?;
                let source_label = parse_optional_string(&self.params, "source_label")?;
                let topic_threshold = parse_optional_f64(&self.params, "topic_threshold")?;
                let min_chunk_size = parse_optional_positive_usize(&self.params, "min_chunk_size")?;
                let dry_run = parse_optional_bool(&self.params, "dry_run")?;
                Ok(RuntimeRequest::Observe {
                    content,
                    namespace,
                    context,
                    chunk_size,
                    source_label,
                    topic_threshold: topic_threshold.map(|value| value as f32),
                    min_chunk_size,
                    dry_run,
                    common,
                })
            }
            "skills" => {
                let mut allowed = vec!["namespace", "extract"];
                allowed.extend_from_slice(COMMON_ENVELOPE_FIELDS);
                reject_unknown_fields(&self.params, &allowed)?;
                let common = parse_common_fields(&self.params)?;
                let namespace = parse_required_string(&self.params, "namespace")?;
                let extract = parse_optional_bool(&self.params, "extract")?;
                Ok(RuntimeRequest::Skills {
                    namespace,
                    extract,
                    common,
                })
            }
            "procedures" => {
                let mut allowed = vec![
                    "namespace",
                    "promote",
                    "rollback",
                    "note",
                    "approved_by",
                    "public",
                ];
                allowed.extend_from_slice(COMMON_ENVELOPE_FIELDS);
                reject_unknown_fields(&self.params, &allowed)?;
                let common = parse_common_fields(&self.params)?;
                let namespace = parse_required_string(&self.params, "namespace")?;
                let promote = parse_optional_string(&self.params, "promote")?;
                let rollback = parse_optional_string(&self.params, "rollback")?;
                let note = parse_optional_string(&self.params, "note")?;
                let approved_by = parse_optional_string(&self.params, "approved_by")?;
                let public = parse_optional_bool(&self.params, "public")?;
                Ok(RuntimeRequest::Procedures {
                    namespace,
                    promote,
                    rollback,
                    note,
                    approved_by,
                    public,
                    common,
                })
            }
            "recall" => {
                let recall_allowed = [
                    "query",
                    "query_text",
                    "namespace",
                    "mode",
                    "limit",
                    "result_budget",
                    "token_budget",
                    "context_text",
                    "effort",
                    "include_public",
                    "like_id",
                    "unlike_id",
                    "graph_mode",
                    "cold_tier",
                    "memory_kinds",
                    "era_id",
                    "as_of_tick",
                    "at_snapshot",
                    "min_strength",
                    "min_confidence",
                    "show_decaying",
                    "mood_congruent",
                ];
                let mut allowed = recall_allowed.to_vec();
                allowed.extend_from_slice(COMMON_ENVELOPE_FIELDS);
                reject_unknown_fields(&self.params, &allowed)?;
                let _common = parse_common_fields(&self.params)?;
                let query = parse_optional_string(&self.params, "query")?;
                let query_text = parse_optional_string(&self.params, "query_text")?;
                let namespace = self
                    .params
                    .get("namespace")
                    .and_then(Value::as_str)
                    .ok_or_else(|| JsonRpcError {
                        code: -32602,
                        message: "missing namespace".to_string(),
                        data: None,
                    })?;
                let mode = parse_optional_string(&self.params, "mode")?;
                let result_budget = parse_optional_budget(&self.params)?;
                let token_budget = parse_optional_positive_usize(&self.params, "token_budget")?;
                let time_budget_ms = parse_optional_u32(&self.params, "time_budget_ms")?;
                let context_text = parse_optional_string(&self.params, "context_text")?;
                let effort = parse_optional_string(&self.params, "effort")?;
                let include_public = parse_optional_bool(&self.params, "include_public")?;
                let like_id = parse_optional_positive_u64(&self.params, "like_id")?;
                let unlike_id = parse_optional_positive_u64(&self.params, "unlike_id")?;
                let graph_mode = parse_optional_string(&self.params, "graph_mode")?;
                let cold_tier = parse_optional_bool(&self.params, "cold_tier")?;
                let workspace_id = parse_optional_string(&self.params, "workspace_id")?;
                let agent_id = parse_optional_string(&self.params, "agent_id")?;
                let session_id = parse_session_handle(&self.params)?;
                let task_id = parse_optional_string(&self.params, "task_id")?;
                let memory_kinds = match self.params.get("memory_kinds") {
                    None | Some(Value::Null) => None,
                    Some(_) => Some(parse_string_array(&self.params, "memory_kinds")?),
                };
                let era_id = parse_optional_string(&self.params, "era_id")?;
                let as_of_tick = parse_optional_u64(&self.params, "as_of_tick")?;
                let at_snapshot = parse_optional_string(&self.params, "at_snapshot")?;
                let min_strength =
                    parse_optional_u32(&self.params, "min_strength")?.map(|value| value as u16);
                let min_confidence = parse_optional_f64(&self.params, "min_confidence")?;
                let show_decaying = parse_optional_bool(&self.params, "show_decaying")?;
                let mood_congruent = parse_optional_bool(&self.params, "mood_congruent")?;

                if as_of_tick.is_some() && at_snapshot.is_some() {
                    return Err(JsonRpcError {
                        code: -32602,
                        message: "as_of_tick and at_snapshot cannot be combined".to_string(),
                        data: None,
                    });
                }

                let primary_query = query_text.or(query);
                if primary_query.is_none() && like_id.is_none() && unlike_id.is_none() {
                    return Err(JsonRpcError {
                        code: -32602,
                        message: "missing query or query-by-example cue".to_string(),
                        data: None,
                    });
                }

                Ok(RuntimeRequest::Recall {
                    query_text: primary_query,
                    namespace: namespace.to_string(),
                    mode,
                    result_budget,
                    token_budget,
                    time_budget_ms,
                    context_text,
                    effort,
                    include_public,
                    like_id,
                    unlike_id,
                    graph_mode,
                    cold_tier,
                    workspace_id,
                    agent_id,
                    session_id,
                    task_id,
                    memory_kinds,
                    era_id,
                    as_of_tick,
                    at_snapshot,
                    min_strength,
                    min_confidence,
                    show_decaying,
                    mood_congruent,
                    common: _common,
                })
            }
            "context_budget" => {
                let allowed = [
                    "token_budget",
                    "namespace",
                    "current_context",
                    "working_memory_ids",
                    "format",
                    "mood_congruent",
                ];
                let mut allowed_fields = allowed.to_vec();
                allowed_fields.extend_from_slice(COMMON_ENVELOPE_FIELDS);
                reject_unknown_fields(&self.params, &allowed_fields)?;
                let common = parse_common_fields(&self.params)?;
                let token_budget = parse_required_u64(&self.params, "token_budget")? as usize;
                let namespace = parse_required_string(&self.params, "namespace")?;
                let current_context = parse_optional_string(&self.params, "current_context")?;
                let format = parse_optional_string(&self.params, "format")?;
                let mood_congruent = parse_optional_bool(&self.params, "mood_congruent")?;
                let working_memory_ids = match self.params.get("working_memory_ids") {
                    None | Some(Value::Null) => None,
                    Some(Value::Array(values)) => Some(
                        values
                            .iter()
                            .map(|value| {
                                value.as_u64().ok_or_else(|| JsonRpcError {
                                    code: -32602,
                                    message:
                                        "working_memory_ids must be an array of positive integers"
                                            .to_string(),
                                    data: None,
                                })
                            })
                            .collect::<Result<Vec<_>, _>>()?,
                    ),
                    Some(_) => {
                        return Err(JsonRpcError {
                            code: -32602,
                            message: "working_memory_ids must be an array of positive integers"
                                .to_string(),
                            data: None,
                        });
                    }
                };
                Ok(RuntimeRequest::ContextBudget {
                    token_budget,
                    namespace,
                    current_context,
                    working_memory_ids,
                    format,
                    mood_congruent,
                    common,
                })
            }
            "goal_state" => {
                let mut allowed = vec!["namespace", "task_id"];
                allowed.extend_from_slice(COMMON_ENVELOPE_FIELDS);
                reject_unknown_fields(&self.params, &allowed)?;
                let common = parse_common_fields(&self.params)?;
                let namespace = parse_required_string(&self.params, "namespace")?;
                let task_id = parse_optional_string(&self.params, "task_id")?;
                Ok(RuntimeRequest::GoalState {
                    namespace,
                    task_id,
                    common,
                })
            }
            "goal_pause" => {
                let mut allowed = vec!["namespace", "task_id", "note"];
                allowed.extend_from_slice(COMMON_ENVELOPE_FIELDS);
                reject_unknown_fields(&self.params, &allowed)?;
                let common = parse_common_fields(&self.params)?;
                let namespace = parse_required_string(&self.params, "namespace")?;
                let task_id = parse_optional_string(&self.params, "task_id")?;
                let note = parse_optional_string(&self.params, "note")?;
                Ok(RuntimeRequest::GoalPause {
                    namespace,
                    task_id,
                    note,
                    common,
                })
            }
            "goal_pin" => {
                let mut allowed = vec!["namespace", "task_id", "memory_id"];
                allowed.extend_from_slice(COMMON_ENVELOPE_FIELDS);
                reject_unknown_fields(&self.params, &allowed)?;
                let common = parse_common_fields(&self.params)?;
                let namespace = parse_required_string(&self.params, "namespace")?;
                let task_id = parse_optional_string(&self.params, "task_id")?;
                let memory_id = parse_required_u64(&self.params, "memory_id")?;
                Ok(RuntimeRequest::GoalPin {
                    namespace,
                    task_id,
                    memory_id,
                    common,
                })
            }
            "goal_dismiss" => {
                let mut allowed = vec!["namespace", "task_id", "memory_id"];
                allowed.extend_from_slice(COMMON_ENVELOPE_FIELDS);
                reject_unknown_fields(&self.params, &allowed)?;
                let common = parse_common_fields(&self.params)?;
                let namespace = parse_required_string(&self.params, "namespace")?;
                let task_id = parse_optional_string(&self.params, "task_id")?;
                let memory_id = parse_required_u64(&self.params, "memory_id")?;
                Ok(RuntimeRequest::GoalDismiss {
                    namespace,
                    task_id,
                    memory_id,
                    common,
                })
            }
            "goal_snapshot" => {
                let mut allowed = vec!["namespace", "task_id", "note"];
                allowed.extend_from_slice(COMMON_ENVELOPE_FIELDS);
                reject_unknown_fields(&self.params, &allowed)?;
                let common = parse_common_fields(&self.params)?;
                let namespace = parse_required_string(&self.params, "namespace")?;
                let task_id = parse_optional_string(&self.params, "task_id")?;
                let note = parse_optional_string(&self.params, "note")?;
                Ok(RuntimeRequest::GoalSnapshot {
                    namespace,
                    task_id,
                    note,
                    common,
                })
            }
            "goal_resume" => {
                let mut allowed = vec!["namespace", "task_id"];
                allowed.extend_from_slice(COMMON_ENVELOPE_FIELDS);
                reject_unknown_fields(&self.params, &allowed)?;
                let common = parse_common_fields(&self.params)?;
                let namespace = parse_required_string(&self.params, "namespace")?;
                let task_id = parse_optional_string(&self.params, "task_id")?;
                Ok(RuntimeRequest::GoalResume {
                    namespace,
                    task_id,
                    common,
                })
            }
            "goal_abandon" => {
                let mut allowed = vec!["namespace", "task_id", "reason"];
                allowed.extend_from_slice(COMMON_ENVELOPE_FIELDS);
                reject_unknown_fields(&self.params, &allowed)?;
                let common = parse_common_fields(&self.params)?;
                let namespace = parse_required_string(&self.params, "namespace")?;
                let task_id = parse_optional_string(&self.params, "task_id")?;
                let reason = parse_optional_string(&self.params, "reason")?;
                Ok(RuntimeRequest::GoalAbandon {
                    namespace,
                    task_id,
                    reason,
                    common,
                })
            }
            "inspect" => {
                let mut allowed = vec!["id", "namespace"];
                allowed.extend_from_slice(COMMON_ENVELOPE_FIELDS);
                reject_unknown_fields(&self.params, &allowed)?;
                let _common = parse_common_fields(&self.params)?;
                let id = parse_required_u64(&self.params, "id")?;
                let namespace = self
                    .params
                    .get("namespace")
                    .and_then(Value::as_str)
                    .ok_or_else(|| JsonRpcError {
                        code: -32602,
                        message: "missing namespace".to_string(),
                        data: None,
                    })?;
                Ok(RuntimeRequest::Inspect {
                    id,
                    namespace: namespace.to_string(),
                    common: _common,
                })
            }
            "explain" => {
                let mut allowed = vec!["query", "namespace", "limit", "depth"];
                allowed.extend_from_slice(COMMON_ENVELOPE_FIELDS);
                reject_unknown_fields(&self.params, &allowed)?;
                let _common = parse_common_fields(&self.params)?;
                let query = self
                    .params
                    .get("query")
                    .and_then(Value::as_str)
                    .ok_or_else(|| JsonRpcError {
                        code: -32602,
                        message: "missing query".to_string(),
                        data: None,
                    })?;
                let namespace = self
                    .params
                    .get("namespace")
                    .and_then(Value::as_str)
                    .ok_or_else(|| JsonRpcError {
                        code: -32602,
                        message: "missing namespace".to_string(),
                        data: None,
                    })?;
                let limit = parse_optional_limit(&self.params)?;
                let depth = parse_optional_positive_usize(&self.params, "depth")?;
                Ok(RuntimeRequest::Explain {
                    query: query.to_string(),
                    namespace: namespace.to_string(),
                    limit,
                    depth,
                    common: _common,
                })
            }
            "why" => {
                let mut allowed = vec!["query", "id", "namespace", "limit", "depth"];
                allowed.extend_from_slice(COMMON_ENVELOPE_FIELDS);
                reject_unknown_fields(&self.params, &allowed)?;
                let common = parse_common_fields(&self.params)?;
                let query = self
                    .params
                    .get("query")
                    .and_then(Value::as_str)
                    .map(ToOwned::to_owned)
                    .or_else(|| {
                        self.params
                            .get("id")
                            .and_then(Value::as_u64)
                            .map(|id| id.to_string())
                    })
                    .ok_or_else(|| JsonRpcError {
                        code: -32602,
                        message: "missing query".to_string(),
                        data: None,
                    })?;
                let namespace = self
                    .params
                    .get("namespace")
                    .and_then(Value::as_str)
                    .ok_or_else(|| JsonRpcError {
                        code: -32602,
                        message: "missing namespace".to_string(),
                        data: None,
                    })?;
                let limit = parse_optional_limit(&self.params)?;
                let depth = parse_optional_positive_usize(&self.params, "depth")?;
                Ok(RuntimeRequest::Explain {
                    query,
                    namespace: namespace.to_string(),
                    limit,
                    depth,
                    common,
                })
            }
            "audit" => {
                let mut allowed = vec!["namespace", "memory_id", "since_tick", "op", "limit"];
                allowed.extend_from_slice(COMMON_ENVELOPE_FIELDS);
                reject_unknown_fields(&self.params, &allowed)?;
                let common = parse_common_fields(&self.params)?;
                let namespace = parse_required_string(&self.params, "namespace")?;
                let memory_id = parse_optional_positive_u64(&self.params, "memory_id")?;
                let since_tick = parse_optional_u64(&self.params, "since_tick")?;
                let op = parse_optional_string(&self.params, "op")?;
                let limit = parse_optional_limit(&self.params)?;
                Ok(RuntimeRequest::Audit {
                    namespace,
                    memory_id,
                    since_tick,
                    op,
                    limit,
                    common,
                })
            }
            "mood_history" => {
                let mut allowed = vec!["namespace", "since_tick"];
                allowed.extend_from_slice(COMMON_ENVELOPE_FIELDS);
                reject_unknown_fields(&self.params, &allowed)?;
                let common = parse_common_fields(&self.params)?;
                let namespace = parse_required_string(&self.params, "namespace")?;
                let since_tick = parse_optional_u64(&self.params, "since_tick")?;
                Ok(RuntimeRequest::MoodHistory {
                    namespace,
                    since_tick,
                    common,
                })
            }
            "hot_paths" => {
                let mut allowed = vec!["namespace", "top_n"];
                allowed.extend_from_slice(COMMON_ENVELOPE_FIELDS);
                reject_unknown_fields(&self.params, &allowed)?;
                let common = parse_common_fields(&self.params)?;
                let namespace = parse_required_string(&self.params, "namespace")?;
                let top_n = parse_optional_positive_usize(&self.params, "top_n")?;
                Ok(RuntimeRequest::HotPaths {
                    namespace,
                    top_n,
                    common,
                })
            }
            "dead_zones" => {
                let mut allowed = vec!["namespace", "min_age_ticks"];
                allowed.extend_from_slice(COMMON_ENVELOPE_FIELDS);
                reject_unknown_fields(&self.params, &allowed)?;
                let common = parse_common_fields(&self.params)?;
                let namespace = parse_required_string(&self.params, "namespace")?;
                let min_age_ticks = parse_optional_u64(&self.params, "min_age_ticks")?;
                Ok(RuntimeRequest::DeadZones {
                    namespace,
                    min_age_ticks,
                    common,
                })
            }
            "compress" => {
                let mut allowed = vec!["namespace", "dry_run"];
                allowed.extend_from_slice(COMMON_ENVELOPE_FIELDS);
                reject_unknown_fields(&self.params, &allowed)?;
                let common = parse_common_fields(&self.params)?;
                let namespace = parse_required_string(&self.params, "namespace")?;
                let dry_run = parse_optional_bool(&self.params, "dry_run")?.unwrap_or(false);
                Ok(RuntimeRequest::Compress {
                    namespace,
                    dry_run,
                    common,
                })
            }
            "schemas" => {
                let mut allowed = vec!["namespace", "top_n"];
                allowed.extend_from_slice(COMMON_ENVELOPE_FIELDS);
                reject_unknown_fields(&self.params, &allowed)?;
                let common = parse_common_fields(&self.params)?;
                let namespace = parse_required_string(&self.params, "namespace")?;
                let top_n = parse_optional_positive_usize(&self.params, "top_n")?;
                Ok(RuntimeRequest::Schemas {
                    namespace,
                    top_n,
                    common,
                })
            }
            "invalidate" => {
                let mut allowed = vec!["id", "namespace", "dry_run"];
                allowed.extend_from_slice(COMMON_ENVELOPE_FIELDS);
                reject_unknown_fields(&self.params, &allowed)?;
                let common = parse_common_fields(&self.params)?;
                let id = parse_required_u64(&self.params, "id")?;
                let namespace = parse_required_string(&self.params, "namespace")?;
                let dry_run = self
                    .params
                    .get("dry_run")
                    .and_then(Value::as_bool)
                    .unwrap_or(false);
                Ok(RuntimeRequest::Invalidate {
                    id,
                    namespace,
                    dry_run,
                    common,
                })
            }
            "preflight.run" => {
                reject_unknown_fields(
                    &self.params,
                    &["namespace", "original_query", "proposed_action"],
                )?;
                Ok(RuntimeRequest::PreflightRun {
                    namespace: parse_required_string(&self.params, "namespace")?,
                    original_query: parse_required_string(&self.params, "original_query")?,
                    proposed_action: parse_required_string(&self.params, "proposed_action")?,
                })
            }
            "preflight.explain" => {
                reject_unknown_fields(
                    &self.params,
                    &["namespace", "original_query", "proposed_action"],
                )?;
                Ok(RuntimeRequest::PreflightExplain {
                    namespace: parse_required_string(&self.params, "namespace")?,
                    original_query: parse_required_string(&self.params, "original_query")?,
                    proposed_action: parse_required_string(&self.params, "proposed_action")?,
                })
            }
            "preflight.allow" => {
                reject_unknown_fields(
                    &self.params,
                    &[
                        "namespace",
                        "original_query",
                        "proposed_action",
                        "authorization_token",
                        "bypass_flags",
                    ],
                )?;
                Ok(RuntimeRequest::PreflightAllow {
                    namespace: parse_required_string(&self.params, "namespace")?,
                    original_query: parse_required_string(&self.params, "original_query")?,
                    proposed_action: parse_required_string(&self.params, "proposed_action")?,
                    authorization_token: parse_required_string(
                        &self.params,
                        "authorization_token",
                    )?,
                    bypass_flags: parse_string_array(&self.params, "bypass_flags")?,
                })
            }
            "resources.list" => {
                reject_unknown_fields(&self.params, COMMON_ENVELOPE_FIELDS)?;
                let _common = parse_common_fields(&self.params)?;
                let _ = _common;
                Ok(RuntimeRequest::ResourcesList)
            }
            "resource.read" => {
                let mut allowed = vec!["uri"];
                allowed.extend_from_slice(COMMON_ENVELOPE_FIELDS);
                reject_unknown_fields(&self.params, &allowed)?;
                let _common = parse_common_fields(&self.params)?;
                let _ = _common;
                Ok(RuntimeRequest::ResourceRead {
                    uri: parse_required_string(&self.params, "uri")?,
                })
            }
            "streams.list" => {
                reject_unknown_fields(&self.params, COMMON_ENVELOPE_FIELDS)?;
                let _common = parse_common_fields(&self.params)?;
                let _ = _common;
                Ok(RuntimeRequest::StreamsList)
            }
            "sleep" => {
                reject_unknown_fields(&self.params, &["millis"])?;
                let millis = self
                    .params
                    .get("millis")
                    .and_then(Value::as_u64)
                    .unwrap_or(0);
                Ok(RuntimeRequest::Sleep { millis })
            }
            "set_posture" => {
                reject_unknown_fields(&self.params, &["posture", "reasons"])?;
                let posture = self
                    .params
                    .get("posture")
                    .and_then(Value::as_str)
                    .ok_or_else(|| JsonRpcError {
                        code: -32602,
                        message: "missing posture".to_string(),
                        data: None,
                    })?;
                let reasons = match self.params.get("reasons") {
                    None | Some(Value::Null) => Vec::new(),
                    Some(Value::Array(items)) => items
                        .iter()
                        .map(|item| {
                            item.as_str()
                                .map(ToOwned::to_owned)
                                .ok_or_else(|| JsonRpcError {
                                    code: -32602,
                                    message: "reasons must be an array of strings".to_string(),
                                    data: None,
                                })
                        })
                        .collect::<Result<Vec<_>, _>>()?,
                    Some(_) => {
                        return Err(JsonRpcError {
                            code: -32602,
                            message: "reasons must be an array of strings".to_string(),
                            data: None,
                        });
                    }
                };
                Ok(RuntimeRequest::SetPosture {
                    posture: posture.to_string(),
                    reasons,
                })
            }
            "run_maintenance" => {
                reject_unknown_fields(&self.params, &["polls_budget", "step_delay_ms"])?;
                let polls_budget = parse_optional_u32(&self.params, "polls_budget")?;
                let step_delay_ms =
                    parse_optional_u32(&self.params, "step_delay_ms")?.map(u64::from);
                Ok(RuntimeRequest::RunMaintenance {
                    polls_budget,
                    step_delay_ms,
                })
            }
            "shutdown" => Ok(RuntimeRequest::Shutdown),
            "forget" => {
                reject_unknown_fields(&self.params, &["id", "namespace", "mode", "reason"])?;
                let id = parse_required_u64(&self.params, "id")?;
                let namespace = parse_required_string(&self.params, "namespace")?;
                let mode = self
                    .params
                    .get("mode")
                    .and_then(Value::as_str)
                    .map(ToOwned::to_owned);
                let reason = self
                    .params
                    .get("reason")
                    .and_then(Value::as_str)
                    .map(ToOwned::to_owned);
                Ok(RuntimeRequest::Forget {
                    id,
                    namespace,
                    mode,
                    reason,
                })
            }
            "pin" => {
                reject_unknown_fields(&self.params, &["id", "namespace", "reason"])?;
                let id = parse_required_u64(&self.params, "id")?;
                let namespace = parse_required_string(&self.params, "namespace")?;
                let reason = self
                    .params
                    .get("reason")
                    .and_then(Value::as_str)
                    .map(ToOwned::to_owned);
                Ok(RuntimeRequest::Pin {
                    id,
                    namespace,
                    reason,
                })
            }
            "consolidate" => {
                reject_unknown_fields(&self.params, &["namespace", "scope"])?;
                let namespace = parse_required_string(&self.params, "namespace")?;
                let scope = self
                    .params
                    .get("scope")
                    .and_then(Value::as_str)
                    .map(ToOwned::to_owned);
                Ok(RuntimeRequest::Consolidate { namespace, scope })
            }
            "share" => {
                let mut allowed = vec!["id", "namespace_id"];
                allowed.extend_from_slice(COMMON_ENVELOPE_FIELDS);
                reject_unknown_fields(&self.params, &allowed)?;
                let _common = parse_common_fields(&self.params)?;
                let id = parse_required_u64(&self.params, "id")?;
                let namespace_id = parse_required_string(&self.params, "namespace_id")?;
                Ok(RuntimeRequest::Share {
                    id,
                    namespace_id,
                    common: _common,
                })
            }
            "unshare" => {
                let mut allowed = vec!["id", "namespace"];
                allowed.extend_from_slice(COMMON_ENVELOPE_FIELDS);
                reject_unknown_fields(&self.params, &allowed)?;
                let _common = parse_common_fields(&self.params)?;
                let id = parse_required_u64(&self.params, "id")?;
                let namespace = parse_required_string(&self.params, "namespace")?;
                Ok(RuntimeRequest::Unshare {
                    id,
                    namespace,
                    common: _common,
                })
            }
            "fork" => {
                let mut allowed = vec!["name", "namespace", "parent_namespace", "inherit", "note"];
                allowed.extend_from_slice(COMMON_ENVELOPE_FIELDS);
                reject_unknown_fields(&self.params, &allowed)?;
                let common = parse_common_fields(&self.params)?;
                let name = parse_required_string(&self.params, "name")?;
                let namespace = parse_required_string(&self.params, "namespace")?;
                let parent_namespace = parse_optional_string(&self.params, "parent_namespace")?;
                let inherit = parse_optional_string(&self.params, "inherit")?;
                let note = parse_optional_string(&self.params, "note")?;
                Ok(RuntimeRequest::Fork {
                    name,
                    namespace,
                    parent_namespace,
                    inherit,
                    note,
                    common,
                })
            }
            "merge_fork" => {
                let mut allowed = vec![
                    "fork_name",
                    "target_namespace",
                    "conflict_strategy",
                    "dry_run",
                ];
                allowed.extend_from_slice(COMMON_ENVELOPE_FIELDS);
                reject_unknown_fields(&self.params, &allowed)?;
                let common = parse_common_fields(&self.params)?;
                let fork_name = parse_required_string(&self.params, "fork_name")?;
                let target_namespace = parse_required_string(&self.params, "target_namespace")?;
                let conflict_strategy = parse_optional_string(&self.params, "conflict_strategy")?;
                let dry_run = parse_optional_bool(&self.params, "dry_run")?.unwrap_or(false);
                Ok(RuntimeRequest::MergeFork {
                    fork_name,
                    target_namespace,
                    conflict_strategy,
                    dry_run,
                    common,
                })
            }
            "link" => {
                let mut allowed = vec!["source_id", "target_id", "namespace", "link_type"];
                allowed.extend_from_slice(COMMON_ENVELOPE_FIELDS);
                reject_unknown_fields(&self.params, &allowed)?;
                let _common = parse_common_fields(&self.params)?;
                let source_id = parse_required_u64(&self.params, "source_id")?;
                let target_id = parse_required_u64(&self.params, "target_id")?;
                let namespace = parse_required_string(&self.params, "namespace")?;
                let link_type = self
                    .params
                    .get("link_type")
                    .and_then(Value::as_str)
                    .map(ToOwned::to_owned);
                Ok(RuntimeRequest::Link {
                    source_id,
                    target_id,
                    namespace,
                    link_type,
                    common: _common,
                })
            }

            // -- MCP Protocol methods ------------------------------------------------
            "initialize" => {
                let protocol_version = self
                    .params
                    .get("protocolVersion")
                    .and_then(Value::as_str)
                    .map(ToOwned::to_owned)
                    .unwrap_or_else(|| "2024-11-05".to_string());
                let capabilities = self
                    .params
                    .get("capabilities")
                    .cloned()
                    .unwrap_or(json!({}));
                let client_info = self.params.get("clientInfo").cloned().unwrap_or(json!({}));
                Ok(RuntimeRequest::McpInitialize {
                    protocol_version,
                    capabilities,
                    client_info,
                })
            }
            "notifications/initialized" => Ok(RuntimeRequest::McpInitialized),
            "tools/list" => Ok(RuntimeRequest::McpToolsList),
            "tools/call" => {
                let name = parse_required_string(&self.params, "name")?;
                let arguments = self.params.get("arguments").cloned().unwrap_or(json!({}));
                Ok(RuntimeRequest::McpToolsCall { name, arguments })
            }
            "resources/list" => Ok(RuntimeRequest::McpResourcesList),
            "resources/read" => {
                let uri = parse_required_string(&self.params, "uri")?;
                Ok(RuntimeRequest::McpResourcesRead { uri })
            }
            "prompts/list" => Ok(RuntimeRequest::McpPromptsList),
            "prompts/get" => {
                let name = parse_required_string(&self.params, "name")?;
                let arguments = self.params.get("arguments").cloned().unwrap_or(json!({}));
                Ok(RuntimeRequest::McpPromptsGet { name, arguments })
            }

            _ => Err(JsonRpcError {
                code: -32601,
                message: format!("unknown method '{}'", self.method),
                data: None,
            }),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
#[allow(clippy::large_enum_variant)]
pub enum RuntimeRequest {
    Ping,
    Status,
    Doctor,
    Health,
    Encode {
        content: String,
        namespace: String,
        memory_type: Option<String>,
        visibility: Option<String>,
        emotional_annotations: Option<serde_json::Value>,
        common: RuntimeCommonFields,
    },
    Observe {
        content: String,
        namespace: String,
        context: Option<String>,
        chunk_size: Option<usize>,
        source_label: Option<String>,
        topic_threshold: Option<f32>,
        min_chunk_size: Option<usize>,
        dry_run: Option<bool>,
        common: RuntimeCommonFields,
    },
    Skills {
        namespace: String,
        extract: Option<bool>,
        common: RuntimeCommonFields,
    },
    Procedures {
        namespace: String,
        promote: Option<String>,
        rollback: Option<String>,
        note: Option<String>,
        approved_by: Option<String>,
        public: Option<bool>,
        common: RuntimeCommonFields,
    },
    Recall {
        query_text: Option<String>,
        namespace: String,
        mode: Option<String>,
        result_budget: Option<usize>,
        token_budget: Option<usize>,
        time_budget_ms: Option<u32>,
        context_text: Option<String>,
        effort: Option<String>,
        include_public: Option<bool>,
        like_id: Option<u64>,
        unlike_id: Option<u64>,
        graph_mode: Option<String>,
        cold_tier: Option<bool>,
        workspace_id: Option<String>,
        agent_id: Option<String>,
        session_id: Option<String>,
        task_id: Option<String>,
        memory_kinds: Option<Vec<String>>,
        era_id: Option<String>,
        as_of_tick: Option<u64>,
        at_snapshot: Option<String>,
        min_strength: Option<u16>,
        min_confidence: Option<f64>,
        show_decaying: Option<bool>,
        mood_congruent: Option<bool>,
        common: RuntimeCommonFields,
    },
    ContextBudget {
        token_budget: usize,
        namespace: String,
        current_context: Option<String>,
        working_memory_ids: Option<Vec<u64>>,
        format: Option<String>,
        mood_congruent: Option<bool>,
        common: RuntimeCommonFields,
    },
    GoalState {
        namespace: String,
        task_id: Option<String>,
        common: RuntimeCommonFields,
    },
    GoalPause {
        namespace: String,
        task_id: Option<String>,
        note: Option<String>,
        common: RuntimeCommonFields,
    },
    GoalPin {
        namespace: String,
        task_id: Option<String>,
        memory_id: u64,
        common: RuntimeCommonFields,
    },
    GoalDismiss {
        namespace: String,
        task_id: Option<String>,
        memory_id: u64,
        common: RuntimeCommonFields,
    },
    GoalSnapshot {
        namespace: String,
        task_id: Option<String>,
        note: Option<String>,
        common: RuntimeCommonFields,
    },
    GoalResume {
        namespace: String,
        task_id: Option<String>,
        common: RuntimeCommonFields,
    },
    GoalAbandon {
        namespace: String,
        task_id: Option<String>,
        reason: Option<String>,
        common: RuntimeCommonFields,
    },
    Inspect {
        id: u64,
        namespace: String,
        common: RuntimeCommonFields,
    },
    Explain {
        query: String,
        namespace: String,
        limit: Option<usize>,
        depth: Option<usize>,
        common: RuntimeCommonFields,
    },
    Audit {
        namespace: String,
        memory_id: Option<u64>,
        since_tick: Option<u64>,
        op: Option<String>,
        limit: Option<usize>,
        common: RuntimeCommonFields,
    },
    MoodHistory {
        namespace: String,
        since_tick: Option<u64>,
        common: RuntimeCommonFields,
    },
    HotPaths {
        namespace: String,
        top_n: Option<usize>,
        common: RuntimeCommonFields,
    },
    DeadZones {
        namespace: String,
        min_age_ticks: Option<u64>,
        common: RuntimeCommonFields,
    },
    Compress {
        namespace: String,
        dry_run: bool,
        common: RuntimeCommonFields,
    },
    Schemas {
        namespace: String,
        top_n: Option<usize>,
        common: RuntimeCommonFields,
    },
    Invalidate {
        id: u64,
        namespace: String,
        dry_run: bool,
        common: RuntimeCommonFields,
    },
    Forget {
        id: u64,
        namespace: String,
        mode: Option<String>,
        reason: Option<String>,
    },
    Pin {
        id: u64,
        namespace: String,
        reason: Option<String>,
    },
    Consolidate {
        namespace: String,
        scope: Option<String>,
    },
    Share {
        id: u64,
        namespace_id: String,
        common: RuntimeCommonFields,
    },
    Unshare {
        id: u64,
        namespace: String,
        common: RuntimeCommonFields,
    },
    Fork {
        name: String,
        namespace: String,
        parent_namespace: Option<String>,
        inherit: Option<String>,
        note: Option<String>,
        common: RuntimeCommonFields,
    },
    MergeFork {
        fork_name: String,
        target_namespace: String,
        conflict_strategy: Option<String>,
        dry_run: bool,
        common: RuntimeCommonFields,
    },
    Link {
        source_id: u64,
        target_id: u64,
        namespace: String,
        link_type: Option<String>,
        common: RuntimeCommonFields,
    },
    PreflightRun {
        namespace: String,
        original_query: String,
        proposed_action: String,
    },
    PreflightExplain {
        namespace: String,
        original_query: String,
        proposed_action: String,
    },
    PreflightAllow {
        namespace: String,
        original_query: String,
        proposed_action: String,
        authorization_token: String,
        bypass_flags: Vec<String>,
    },
    ResourcesList,
    ResourceRead {
        uri: String,
    },
    StreamsList,
    Sleep {
        millis: u64,
    },
    SetPosture {
        posture: String,
        reasons: Vec<String>,
    },
    RunMaintenance {
        polls_budget: Option<u32>,
        step_delay_ms: Option<u64>,
    },
    Shutdown,

    // -- MCP Protocol -------------------------------------------------------
    McpInitialize {
        protocol_version: String,
        capabilities: serde_json::Value,
        client_info: serde_json::Value,
    },
    McpInitialized,
    McpToolsList,
    McpToolsCall {
        name: String,
        arguments: serde_json::Value,
    },
    McpResourcesList,
    McpResourcesRead {
        uri: String,
    },
    McpPromptsList,
    McpPromptsGet {
        name: String,
        arguments: serde_json::Value,
    },
}

pub fn busy_payload(queue_depth: usize, max_queue_depth: usize) -> Value {
    json!(RuntimeBusy {
        queue_depth,
        max_queue_depth,
    })
}

pub fn cancelled_payload() -> Value {
    json!(RuntimeCancelled {
        reason: "server_shutdown",
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn sample_common_fields() -> RuntimeCommonFields {
        RuntimeCommonFields {
            request_id: Some("req-common".to_string()),
            workspace_id: Some("ws-7".to_string()),
            agent_id: Some("agent-3".to_string()),
            session_id: Some("session-9".to_string()),
            task_id: Some("task-2".to_string()),
            time_budget_ms: Some(75),
            policy_context: Some(RuntimePolicyContextHint {
                include_public: true,
                sharing_visibility: Some("public".to_string()),
                caller_identity_bound: true,
                workspace_acl_allowed: true,
                agent_acl_allowed: true,
                session_visibility_allowed: true,
                legal_hold: false,
            }),
        }
    }

    #[test]
    fn parse_method_rejects_non_jsonrpc_2_0_requests() {
        let request = RuntimeMethodRequest {
            jsonrpc: "1.0".to_string(),
            method: "status".to_string(),
            params: json!({}),
            id: Some(json!(1)),
        };

        let error = request.parse_method().unwrap_err();
        assert_eq!(error.code, -32600);
        assert_eq!(error.message, "unsupported jsonrpc version");
        assert_eq!(
            error.data,
            Some(json!({ "expected": "2.0", "actual": "1.0" }))
        );
    }

    #[test]
    fn parse_method_rejects_non_object_params() {
        let request = RuntimeMethodRequest {
            jsonrpc: "2.0".to_string(),
            method: "sleep".to_string(),
            params: json!([100]),
            id: Some(json!(1)),
        };

        let error = request.parse_method().unwrap_err();
        assert_eq!(error.code, -32602);
        assert_eq!(error.message, "params must be a JSON object");
        assert_eq!(error.data, None);
    }

    #[test]
    fn parse_method_accepts_named_run_maintenance_params() {
        let request = RuntimeMethodRequest {
            jsonrpc: "2.0".to_string(),
            method: "run_maintenance".to_string(),
            params: json!({ "polls_budget": 7, "step_delay_ms": 15 }),
            id: Some(json!(1)),
        };

        assert_eq!(
            request.parse_method().unwrap(),
            RuntimeRequest::RunMaintenance {
                polls_budget: Some(7),
                step_delay_ms: Some(15),
            }
        );
    }

    #[test]
    fn parse_method_rejects_zero_or_fractional_run_maintenance_polls_budget() {
        let zero_budget = RuntimeMethodRequest {
            jsonrpc: "2.0".to_string(),
            method: "run_maintenance".to_string(),
            params: json!({ "polls_budget": 0, "step_delay_ms": 15 }),
            id: Some(json!(2)),
        };
        let error = zero_budget.parse_method().unwrap_err();
        assert_eq!(error.code, -32602);
        assert_eq!(error.message, "polls_budget must be at least 1");

        let fractional_budget = RuntimeMethodRequest {
            jsonrpc: "2.0".to_string(),
            method: "run_maintenance".to_string(),
            params: json!({ "polls_budget": 1.5, "step_delay_ms": 15 }),
            id: Some(json!(3)),
        };
        let error = fractional_budget.parse_method().unwrap_err();
        assert_eq!(error.code, -32602);
        assert_eq!(error.message, "polls_budget must be a positive integer");
    }

    #[test]
    fn parse_method_rejects_zero_or_fractional_run_maintenance_step_delay() {
        let zero_step_delay = RuntimeMethodRequest {
            jsonrpc: "2.0".to_string(),
            method: "run_maintenance".to_string(),
            params: json!({ "polls_budget": 4, "step_delay_ms": 0 }),
            id: Some(json!(4)),
        };
        let error = zero_step_delay.parse_method().unwrap_err();
        assert_eq!(error.code, -32602);
        assert_eq!(error.message, "step_delay_ms must be at least 1");

        let fractional_step_delay = RuntimeMethodRequest {
            jsonrpc: "2.0".to_string(),
            method: "run_maintenance".to_string(),
            params: json!({ "polls_budget": 4, "step_delay_ms": 1.5 }),
            id: Some(json!(5)),
        };
        let error = fractional_step_delay.parse_method().unwrap_err();
        assert_eq!(error.code, -32602);
        assert_eq!(error.message, "step_delay_ms must be a positive integer");
    }

    #[test]
    fn parse_method_rejects_non_numeric_run_maintenance_fields() {
        let string_budget = RuntimeMethodRequest {
            jsonrpc: "2.0".to_string(),
            method: "run_maintenance".to_string(),
            params: json!({ "polls_budget": "4", "step_delay_ms": 15 }),
            id: Some(json!(6)),
        };
        let error = string_budget.parse_method().unwrap_err();
        assert_eq!(error.code, -32602);
        assert_eq!(error.message, "polls_budget must be a positive integer");

        let string_step_delay = RuntimeMethodRequest {
            jsonrpc: "2.0".to_string(),
            method: "run_maintenance".to_string(),
            params: json!({ "polls_budget": 4, "step_delay_ms": "15" }),
            id: Some(json!(7)),
        };
        let error = string_step_delay.parse_method().unwrap_err();
        assert_eq!(error.code, -32602);
        assert_eq!(error.message, "step_delay_ms must be a positive integer");
    }

    #[test]
    fn parse_method_run_maintenance_rejects_unknown_fields() {
        let request = RuntimeMethodRequest {
            jsonrpc: "2.0".to_string(),
            method: "run_maintenance".to_string(),
            params: json!({
                "polls_budget": 4,
                "step_delay_ms": 15,
                "unexpected": true
            }),
            id: Some(json!(8)),
        };

        let error = request.parse_method().unwrap_err();
        assert_eq!(error.code, -32602);
        assert_eq!(error.message, "unknown field unexpected");
    }

    #[test]
    fn parse_method_accepts_doctor_without_params() {
        let request = RuntimeMethodRequest {
            jsonrpc: "2.0".to_string(),
            method: "doctor".to_string(),
            params: json!({}),
            id: Some(json!(101)),
        };

        assert_eq!(request.parse_method().unwrap(), RuntimeRequest::Doctor);
    }

    #[test]
    fn parse_method_accepts_mood_history_params() {
        let request = RuntimeMethodRequest {
            jsonrpc: "2.0".to_string(),
            method: "mood_history".to_string(),
            params: json!({ "namespace": "team.alpha", "since_tick": 12 }),
            id: Some(json!(17)),
        };

        assert_eq!(
            request.parse_method().unwrap(),
            RuntimeRequest::MoodHistory {
                namespace: "team.alpha".to_string(),
                since_tick: Some(12),
                common: RuntimeCommonFields::default(),
            }
        );
    }

    #[test]
    fn parse_method_accepts_compress_params() {
        let request = RuntimeMethodRequest {
            jsonrpc: "2.0".to_string(),
            method: "compress".to_string(),
            params: json!({ "namespace": "team.alpha", "dry_run": true }),
            id: Some(json!(18)),
        };

        assert_eq!(
            request.parse_method().unwrap(),
            RuntimeRequest::Compress {
                namespace: "team.alpha".to_string(),
                dry_run: true,
                common: RuntimeCommonFields::default(),
            }
        );
    }

    #[test]
    fn parse_method_accepts_schemas_params() {
        let request = RuntimeMethodRequest {
            jsonrpc: "2.0".to_string(),
            method: "schemas".to_string(),
            params: json!({ "namespace": "team.alpha", "top_n": 5 }),
            id: Some(json!(19)),
        };

        assert_eq!(
            request.parse_method().unwrap(),
            RuntimeRequest::Schemas {
                namespace: "team.alpha".to_string(),
                top_n: Some(5),
                common: RuntimeCommonFields::default(),
            }
        );
    }

    #[test]
    fn parse_method_accepts_named_recall_params() {
        let request = RuntimeMethodRequest {
            jsonrpc: "2.0".to_string(),
            method: "recall".to_string(),
            params: json!({ "query": "memory:42", "namespace": "team.alpha", "limit": 3 }),
            id: Some(json!(1)),
        };

        assert_eq!(
            request.parse_method().unwrap(),
            RuntimeRequest::Recall {
                query_text: Some("memory:42".to_string()),
                namespace: "team.alpha".to_string(),
                mode: None,
                result_budget: Some(3),
                token_budget: None,
                time_budget_ms: None,
                context_text: None,
                effort: None,
                include_public: None,
                like_id: None,
                unlike_id: None,
                graph_mode: None,
                cold_tier: None,
                common: RuntimeCommonFields::default(),
                workspace_id: None,
                agent_id: None,
                session_id: None,
                task_id: None,
                memory_kinds: None,
                era_id: None,
                as_of_tick: None,
                at_snapshot: None,
                min_strength: None,
                min_confidence: None,
                show_decaying: None,
                mood_congruent: None,
            }
        );
    }

    #[test]
    fn parse_method_recall_requires_query_or_example_cue_and_namespace() {
        let missing_query = RuntimeMethodRequest {
            jsonrpc: "2.0".to_string(),
            method: "recall".to_string(),
            params: json!({ "namespace": "team.alpha" }),
            id: Some(json!(1)),
        };
        let error = missing_query.parse_method().unwrap_err();
        assert_eq!(error.code, -32602);
        assert_eq!(error.message, "missing query or query-by-example cue");

        let missing_namespace = RuntimeMethodRequest {
            jsonrpc: "2.0".to_string(),
            method: "recall".to_string(),
            params: json!({ "query": "memory:42" }),
            id: Some(json!(2)),
        };
        let error = missing_namespace.parse_method().unwrap_err();
        assert_eq!(error.code, -32602);
        assert_eq!(error.message, "missing namespace");
    }

    #[test]
    fn parse_method_accepts_context_budget_params() {
        let request = RuntimeMethodRequest {
            jsonrpc: "2.0".to_string(),
            method: "context_budget".to_string(),
            params: json!({
                "token_budget": 256,
                "namespace": "team.alpha",
                "current_context": "debugging session",
                "working_memory_ids": [7, 8],
                "format": "markdown",
                "mood_congruent": true
            }),
            id: Some(json!(9)),
        };

        assert_eq!(
            request.parse_method().unwrap(),
            RuntimeRequest::ContextBudget {
                token_budget: 256,
                namespace: "team.alpha".to_string(),
                current_context: Some("debugging session".to_string()),
                working_memory_ids: Some(vec![7, 8]),
                format: Some("markdown".to_string()),
                mood_congruent: Some(true),
                common: RuntimeCommonFields::default(),
            }
        );
    }

    #[test]
    fn parse_method_context_budget_rejects_missing_namespace_and_bad_working_memory_ids() {
        let missing_namespace = RuntimeMethodRequest {
            jsonrpc: "2.0".to_string(),
            method: "context_budget".to_string(),
            params: json!({ "token_budget": 64 }),
            id: Some(json!(10)),
        };
        let error = missing_namespace.parse_method().unwrap_err();
        assert_eq!(error.code, -32602);
        assert_eq!(error.message, "missing namespace");

        let bad_ids = RuntimeMethodRequest {
            jsonrpc: "2.0".to_string(),
            method: "context_budget".to_string(),
            params: json!({
                "token_budget": 64,
                "namespace": "team.alpha",
                "working_memory_ids": [1, "two"]
            }),
            id: Some(json!(11)),
        };
        let error = bad_ids.parse_method().unwrap_err();
        assert_eq!(error.code, -32602);
        assert_eq!(
            error.message,
            "working_memory_ids must be an array of positive integers"
        );
    }

    #[test]
    fn parse_method_recall_rejects_invalid_budgets_and_conflicting_history_fields() {
        let zero_limit = RuntimeMethodRequest {
            jsonrpc: "2.0".to_string(),
            method: "recall".to_string(),
            params: json!({ "query": "memory:42", "namespace": "team.alpha", "limit": 0 }),
            id: Some(json!(3)),
        };
        let error = zero_limit.parse_method().unwrap_err();
        assert_eq!(error.code, -32602);
        assert_eq!(error.message, "limit must be at least 1");

        let fractional_limit = RuntimeMethodRequest {
            jsonrpc: "2.0".to_string(),
            method: "recall".to_string(),
            params: json!({ "query": "memory:42", "namespace": "team.alpha", "limit": 1.5 }),
            id: Some(json!(4)),
        };
        let error = fractional_limit.parse_method().unwrap_err();
        assert_eq!(error.code, -32602);
        assert_eq!(error.message, "limit must be a positive integer");

        let mismatched_budget = RuntimeMethodRequest {
            jsonrpc: "2.0".to_string(),
            method: "recall".to_string(),
            params: json!({
                "query": "memory:42",
                "namespace": "team.alpha",
                "limit": 2,
                "result_budget": 3
            }),
            id: Some(json!(5)),
        };
        let error = mismatched_budget.parse_method().unwrap_err();
        assert_eq!(error.code, -32602);
        assert_eq!(
            error.message,
            "limit and result_budget must match when both are provided"
        );

        let conflicting_history = RuntimeMethodRequest {
            jsonrpc: "2.0".to_string(),
            method: "recall".to_string(),
            params: json!({
                "query": "memory:42",
                "namespace": "team.alpha",
                "as_of_tick": 42,
                "at_snapshot": "before-refactor"
            }),
            id: Some(json!(6)),
        };
        let error = conflicting_history.parse_method().unwrap_err();
        assert_eq!(error.code, -32602);
        assert_eq!(
            error.message,
            "as_of_tick and at_snapshot cannot be combined"
        );
    }

    #[test]
    fn parse_method_accepts_named_inspect_params() {
        let request = RuntimeMethodRequest {
            jsonrpc: "2.0".to_string(),
            method: "inspect".to_string(),
            params: json!({ "id": 42, "namespace": "team.alpha" }),
            id: Some(json!(5)),
        };

        assert_eq!(
            request.parse_method().unwrap(),
            RuntimeRequest::Inspect {
                id: 42,
                namespace: "team.alpha".to_string(),
                common: RuntimeCommonFields::default(),
            }
        );
    }

    #[test]
    fn parse_method_inspect_requires_id_and_namespace() {
        let missing_id = RuntimeMethodRequest {
            jsonrpc: "2.0".to_string(),
            method: "inspect".to_string(),
            params: json!({ "namespace": "team.alpha" }),
            id: Some(json!(6)),
        };
        let error = missing_id.parse_method().unwrap_err();
        assert_eq!(error.code, -32602);
        assert_eq!(error.message, "missing id");

        let zero_id = RuntimeMethodRequest {
            jsonrpc: "2.0".to_string(),
            method: "inspect".to_string(),
            params: json!({ "id": 0, "namespace": "team.alpha" }),
            id: Some(json!(61)),
        };
        let error = zero_id.parse_method().unwrap_err();
        assert_eq!(error.code, -32602);
        assert_eq!(error.message, "id must be at least 1");

        let fractional_id = RuntimeMethodRequest {
            jsonrpc: "2.0".to_string(),
            method: "inspect".to_string(),
            params: json!({ "id": 1.5, "namespace": "team.alpha" }),
            id: Some(json!(62)),
        };
        let error = fractional_id.parse_method().unwrap_err();
        assert_eq!(error.code, -32602);
        assert_eq!(error.message, "id must be a positive integer");

        let missing_namespace = RuntimeMethodRequest {
            jsonrpc: "2.0".to_string(),
            method: "inspect".to_string(),
            params: json!({ "id": 42 }),
            id: Some(json!(7)),
        };
        let error = missing_namespace.parse_method().unwrap_err();
        assert_eq!(error.code, -32602);
        assert_eq!(error.message, "missing namespace");
    }

    #[test]
    fn parse_method_accepts_named_explain_params() {
        let request = RuntimeMethodRequest {
            jsonrpc: "2.0".to_string(),
            method: "explain".to_string(),
            params: json!({ "query": "memory:42", "namespace": "team.alpha", "limit": 2 }),
            id: Some(json!(8)),
        };

        assert_eq!(
            request.parse_method().unwrap(),
            RuntimeRequest::Explain {
                query: "memory:42".to_string(),
                namespace: "team.alpha".to_string(),
                limit: Some(2),
                depth: None,
                common: RuntimeCommonFields::default(),
            }
        );
    }

    #[test]
    fn parse_method_accepts_why_alias_and_optional_depth() {
        let request = RuntimeMethodRequest {
            jsonrpc: "2.0".to_string(),
            method: "why".to_string(),
            params: json!({ "id": 42, "namespace": "team.alpha", "depth": 3 }),
            id: Some(json!(81)),
        };

        assert_eq!(
            request.parse_method().unwrap(),
            RuntimeRequest::Explain {
                query: "42".to_string(),
                namespace: "team.alpha".to_string(),
                limit: None,
                depth: Some(3),
                common: RuntimeCommonFields::default(),
            }
        );
    }

    #[test]
    fn parse_method_accepts_invalidate_with_optional_dry_run() {
        let request = RuntimeMethodRequest {
            jsonrpc: "2.0".to_string(),
            method: "invalidate".to_string(),
            params: json!({ "id": 42, "namespace": "team.alpha", "dry_run": true }),
            id: Some(json!(82)),
        };

        assert_eq!(
            request.parse_method().unwrap(),
            RuntimeRequest::Invalidate {
                id: 42,
                namespace: "team.alpha".to_string(),
                dry_run: true,
                common: RuntimeCommonFields::default(),
            }
        );
    }

    #[test]
    fn parse_method_explain_reuses_limit_validation() {
        let zero_limit = RuntimeMethodRequest {
            jsonrpc: "2.0".to_string(),
            method: "explain".to_string(),
            params: json!({ "query": "memory:42", "namespace": "team.alpha", "limit": 0 }),
            id: Some(json!(9)),
        };
        let error = zero_limit.parse_method().unwrap_err();
        assert_eq!(error.code, -32602);
        assert_eq!(error.message, "limit must be at least 1");
    }

    #[test]
    fn parse_method_accepts_preflight_methods_with_canonical_fields() {
        let run = RuntimeMethodRequest {
            jsonrpc: "2.0".to_string(),
            method: "preflight.run".to_string(),
            params: json!({
                "namespace": "team.alpha",
                "original_query": "delete prior audit events",
                "proposed_action": "purge namespace audit history"
            }),
            id: Some(json!(10)),
        };
        assert_eq!(
            run.parse_method().unwrap(),
            RuntimeRequest::PreflightRun {
                namespace: "team.alpha".to_string(),
                original_query: "delete prior audit events".to_string(),
                proposed_action: "purge namespace audit history".to_string(),
            }
        );

        let explain = RuntimeMethodRequest {
            jsonrpc: "2.0".to_string(),
            method: "preflight.explain".to_string(),
            params: json!({
                "namespace": "team.alpha",
                "original_query": "delete prior audit events",
                "proposed_action": "purge namespace audit history"
            }),
            id: Some(json!(11)),
        };
        assert_eq!(
            explain.parse_method().unwrap(),
            RuntimeRequest::PreflightExplain {
                namespace: "team.alpha".to_string(),
                original_query: "delete prior audit events".to_string(),
                proposed_action: "purge namespace audit history".to_string(),
            }
        );

        let allow = RuntimeMethodRequest {
            jsonrpc: "2.0".to_string(),
            method: "preflight.allow".to_string(),
            params: json!({
                "namespace": "team.alpha",
                "original_query": "delete prior audit events",
                "proposed_action": "purge namespace audit history",
                "authorization_token": "token-123",
                "bypass_flags": ["manual_override"]
            }),
            id: Some(json!(12)),
        };
        assert_eq!(
            allow.parse_method().unwrap(),
            RuntimeRequest::PreflightAllow {
                namespace: "team.alpha".to_string(),
                original_query: "delete prior audit events".to_string(),
                proposed_action: "purge namespace audit history".to_string(),
                authorization_token: "token-123".to_string(),
                bypass_flags: vec!["manual_override".to_string()],
            }
        );

        let resources = RuntimeMethodRequest {
            jsonrpc: "2.0".to_string(),
            method: "resources.list".to_string(),
            params: json!({}),
            id: Some(json!(13)),
        };
        assert_eq!(
            resources.parse_method().unwrap(),
            RuntimeRequest::ResourcesList
        );

        let resource_read = RuntimeMethodRequest {
            jsonrpc: "2.0".to_string(),
            method: "resource.read".to_string(),
            params: json!({ "uri": "membrain://inspect/team.alpha/42" }),
            id: Some(json!(14)),
        };
        assert_eq!(
            resource_read.parse_method().unwrap(),
            RuntimeRequest::ResourceRead {
                uri: "membrain://inspect/team.alpha/42".to_string(),
            }
        );

        let streams = RuntimeMethodRequest {
            jsonrpc: "2.0".to_string(),
            method: "streams.list".to_string(),
            params: json!({}),
            id: Some(json!(15)),
        };
        assert_eq!(streams.parse_method().unwrap(), RuntimeRequest::StreamsList);
    }

    #[test]
    fn parse_method_preflight_rejects_missing_fields_and_non_string_bypass_flags() {
        let missing_action = RuntimeMethodRequest {
            jsonrpc: "2.0".to_string(),
            method: "preflight.run".to_string(),
            params: json!({
                "namespace": "team.alpha",
                "original_query": "delete prior audit events"
            }),
            id: Some(json!(13)),
        };
        let error = missing_action.parse_method().unwrap_err();
        assert_eq!(error.code, -32602);
        assert_eq!(error.message, "missing proposed_action");

        let bad_flags = RuntimeMethodRequest {
            jsonrpc: "2.0".to_string(),
            method: "preflight.allow".to_string(),
            params: json!({
                "namespace": "team.alpha",
                "original_query": "delete prior audit events",
                "proposed_action": "purge namespace audit history",
                "authorization_token": "token-123",
                "bypass_flags": ["manual_override", 7]
            }),
            id: Some(json!(14)),
        };
        let error = bad_flags.parse_method().unwrap_err();
        assert_eq!(error.code, -32602);
        assert_eq!(error.message, "bypass_flags must be an array of strings");
    }

    #[test]
    fn parse_method_preflight_rejects_unknown_fields() {
        let run = RuntimeMethodRequest {
            jsonrpc: "2.0".to_string(),
            method: "preflight.run".to_string(),
            params: json!({
                "namespace": "team.alpha",
                "original_query": "delete prior audit events",
                "proposed_action": "purge namespace audit history",
                "unexpected": true
            }),
            id: Some(json!(15)),
        };
        let error = run.parse_method().unwrap_err();
        assert_eq!(error.code, -32602);
        assert_eq!(error.message, "unknown field unexpected");

        let explain = RuntimeMethodRequest {
            jsonrpc: "2.0".to_string(),
            method: "preflight.explain".to_string(),
            params: json!({
                "namespace": "team.alpha",
                "original_query": "delete prior audit events",
                "proposed_action": "purge namespace audit history",
                "unexpected": true
            }),
            id: Some(json!(16)),
        };
        let error = explain.parse_method().unwrap_err();
        assert_eq!(error.code, -32602);
        assert_eq!(error.message, "unknown field unexpected");

        let allow = RuntimeMethodRequest {
            jsonrpc: "2.0".to_string(),
            method: "preflight.allow".to_string(),
            params: json!({
                "namespace": "team.alpha",
                "original_query": "delete prior audit events",
                "proposed_action": "purge namespace audit history",
                "authorization_token": "token-123",
                "bypass_flags": ["manual_override"],
                "unexpected": true
            }),
            id: Some(json!(17)),
        };
        let error = allow.parse_method().unwrap_err();
        assert_eq!(error.code, -32602);
        assert_eq!(error.message, "unknown field unexpected");

        let resources = RuntimeMethodRequest {
            jsonrpc: "2.0".to_string(),
            method: "resources.list".to_string(),
            params: json!({ "unexpected": true }),
            id: Some(json!(18)),
        };
        let error = resources.parse_method().unwrap_err();
        assert_eq!(error.code, -32602);
        assert_eq!(error.message, "unknown field unexpected");

        let resource_read = RuntimeMethodRequest {
            jsonrpc: "2.0".to_string(),
            method: "resource.read".to_string(),
            params: json!({
                "uri": "membrain://inspect/team.alpha/42",
                "unexpected": true
            }),
            id: Some(json!(19)),
        };
        let error = resource_read.parse_method().unwrap_err();
        assert_eq!(error.code, -32602);
        assert_eq!(error.message, "unknown field unexpected");

        let streams = RuntimeMethodRequest {
            jsonrpc: "2.0".to_string(),
            method: "streams.list".to_string(),
            params: json!({ "unexpected": true }),
            id: Some(json!(20)),
        };
        let error = streams.parse_method().unwrap_err();
        assert_eq!(error.code, -32602);
        assert_eq!(error.message, "unknown field unexpected");
    }

    #[test]
    fn parse_method_common_fields_round_trip_through_helper() {
        let params = json!({
            "request_id": "req-common",
            "workspace_id": "ws-7",
            "agent_id": "agent-3",
            "session_id": "session-9",
            "task_id": "task-2",
            "time_budget_ms": 75,
            "policy_context": {
                "include_public": true,
                "sharing_visibility": "public"
            }
        });

        assert_eq!(
            parse_common_fields(&params).unwrap(),
            sample_common_fields()
        );
        assert_eq!(
            parse_session_handle(&json!({"session_id": 9})).unwrap(),
            Some("9".to_string())
        );
    }

    #[test]
    fn parse_method_common_fields_reject_invalid_policy_context_shapes() {
        let error = parse_common_fields(&json!({"policy_context": true})).unwrap_err();
        assert_eq!(error.code, -32602);
        assert_eq!(error.message, "policy_context must be a JSON object");

        let error = parse_common_fields(&json!({
            "policy_context": {"include_public": true, "unexpected": true}
        }))
        .unwrap_err();
        assert!(error.message.contains("unknown field"));
        assert!(error.message.contains("unexpected"));
    }

    #[test]
    fn parse_method_recall_accepts_query_by_example_and_richer_contract_fields() {
        let request = RuntimeMethodRequest {
            jsonrpc: "2.0".to_string(),
            method: "recall".to_string(),
            params: json!({
                "query_text": "session:7",
                "namespace": "team.alpha",
                "mode": "semantic",
                "result_budget": 4,
                "token_budget": 256,
                "time_budget_ms": 75,
                "context_text": "triaging recall drift",
                "effort": "high",
                "include_public": true,
                "like_id": 11,
                "unlike_id": 17,
                "graph_mode": "expand",
                "cold_tier": true,
                "workspace_id": "ws-7",
                "agent_id": "agent-3",
                "session_id": "session-9",
                "task_id": "task-2",
                "request_id": "req-recall-18",
                "policy_context": {
                    "include_public": true,
                    "sharing_visibility": "public"
                },
                "memory_kinds": ["user_preference", "session_note"],
                "era_id": "incident-2026",
                "as_of_tick": 42,
                "min_strength": 200,
                "min_confidence": 0.8,
                "show_decaying": true,
                "mood_congruent": true
            }),
            id: Some(json!(18)),
        };

        assert_eq!(
            request.parse_method().unwrap(),
            RuntimeRequest::Recall {
                query_text: Some("session:7".to_string()),
                namespace: "team.alpha".to_string(),
                mode: Some("semantic".to_string()),
                result_budget: Some(4),
                token_budget: Some(256),
                time_budget_ms: Some(75),
                context_text: Some("triaging recall drift".to_string()),
                effort: Some("high".to_string()),
                include_public: Some(true),
                like_id: Some(11),
                unlike_id: Some(17),
                graph_mode: Some("expand".to_string()),
                cold_tier: Some(true),
                workspace_id: Some("ws-7".to_string()),
                agent_id: Some("agent-3".to_string()),
                session_id: Some("session-9".to_string()),
                task_id: Some("task-2".to_string()),
                memory_kinds: Some(vec![
                    "user_preference".to_string(),
                    "session_note".to_string()
                ]),
                era_id: Some("incident-2026".to_string()),
                as_of_tick: Some(42),
                at_snapshot: None,
                min_strength: Some(200),
                min_confidence: Some(0.8),
                show_decaying: Some(true),
                mood_congruent: Some(true),
                common: RuntimeCommonFields {
                    request_id: Some("req-recall-18".to_string()),
                    workspace_id: Some("ws-7".to_string()),
                    agent_id: Some("agent-3".to_string()),
                    session_id: Some("session-9".to_string()),
                    task_id: Some("task-2".to_string()),
                    time_budget_ms: Some(75),
                    policy_context: Some(RuntimePolicyContextHint {
                        include_public: true,
                        sharing_visibility: Some("public".to_string()),
                        caller_identity_bound: true,
                        workspace_acl_allowed: true,
                        agent_acl_allowed: true,
                        session_visibility_allowed: true,
                        legal_hold: false,
                    }),
                },
            }
        );

        let example_only = RuntimeMethodRequest {
            jsonrpc: "2.0".to_string(),
            method: "recall".to_string(),
            params: json!({
                "namespace": "team.alpha",
                "like_id": 99,
                "result_budget": 2,
                "session_id": 9
            }),
            id: Some(json!(19)),
        };

        assert_eq!(
            example_only.parse_method().unwrap(),
            RuntimeRequest::Recall {
                query_text: None,
                namespace: "team.alpha".to_string(),
                mode: None,
                result_budget: Some(2),
                token_budget: None,
                time_budget_ms: None,
                context_text: None,
                effort: None,
                include_public: None,
                like_id: Some(99),
                unlike_id: None,
                graph_mode: None,
                cold_tier: None,
                workspace_id: None,
                agent_id: None,
                session_id: Some("9".to_string()),
                task_id: None,
                memory_kinds: None,
                era_id: None,
                as_of_tick: None,
                at_snapshot: None,
                min_strength: None,
                min_confidence: None,
                show_decaying: None,
                mood_congruent: None,
                common: RuntimeCommonFields {
                    request_id: None,
                    workspace_id: None,
                    agent_id: None,
                    session_id: Some("9".to_string()),
                    task_id: None,
                    time_budget_ms: None,
                    policy_context: None,
                },
            }
        );
    }

    #[test]
    fn parse_method_recall_rejects_invalid_extended_contract_fields() {
        let invalid_memory_kinds = RuntimeMethodRequest {
            jsonrpc: "2.0".to_string(),
            method: "recall".to_string(),
            params: json!({
                "query_text": "memory:42",
                "namespace": "team.alpha",
                "memory_kinds": ["user_preference", 7]
            }),
            id: Some(json!(20)),
        };
        let error = invalid_memory_kinds.parse_method().unwrap_err();
        assert_eq!(error.code, -32602);
        assert_eq!(error.message, "memory_kinds must be an array of strings");

        let invalid_token_budget = RuntimeMethodRequest {
            jsonrpc: "2.0".to_string(),
            method: "recall".to_string(),
            params: json!({
                "query_text": "memory:42",
                "namespace": "team.alpha",
                "token_budget": 0
            }),
            id: Some(json!(21)),
        };
        let error = invalid_token_budget.parse_method().unwrap_err();
        assert_eq!(error.code, -32602);
        assert_eq!(error.message, "token_budget must be at least 1");

        let invalid_min_strength = RuntimeMethodRequest {
            jsonrpc: "2.0".to_string(),
            method: "recall".to_string(),
            params: json!({
                "query_text": "memory:42",
                "namespace": "team.alpha",
                "min_strength": 0
            }),
            id: Some(json!(22)),
        };
        let error = invalid_min_strength.parse_method().unwrap_err();
        assert_eq!(error.code, -32602);
        assert_eq!(error.message, "min_strength must be at least 1");
    }

    #[test]
    fn parse_method_retrieval_methods_reject_unknown_fields() {
        let recall = RuntimeMethodRequest {
            jsonrpc: "2.0".to_string(),
            method: "recall".to_string(),
            params: json!({
                "query_text": "memory:42",
                "namespace": "team.alpha",
                "result_budget": 3,
                "unexpected": true
            }),
            id: Some(json!(18)),
        };
        let error = recall.parse_method().unwrap_err();
        assert_eq!(error.code, -32602);
        assert_eq!(error.message, "unknown field unexpected");

        let inspect = RuntimeMethodRequest {
            jsonrpc: "2.0".to_string(),
            method: "inspect".to_string(),
            params: json!({
                "id": 42,
                "namespace": "team.alpha",
                "unexpected": true
            }),
            id: Some(json!(19)),
        };
        let error = inspect.parse_method().unwrap_err();
        assert_eq!(error.code, -32602);
        assert_eq!(error.message, "unknown field unexpected");

        let explain = RuntimeMethodRequest {
            jsonrpc: "2.0".to_string(),
            method: "explain".to_string(),
            params: json!({
                "query": "memory:42",
                "namespace": "team.alpha",
                "limit": 2,
                "unexpected": true
            }),
            id: Some(json!(20)),
        };
        let error = explain.parse_method().unwrap_err();
        assert_eq!(error.code, -32602);
        assert_eq!(error.message, "unknown field unexpected");

        let goal_state = RuntimeMethodRequest {
            jsonrpc: "2.0".to_string(),
            method: "goal_state".to_string(),
            params: json!({
                "namespace": "team.alpha",
                "task_id": "task-42",
                "unexpected": true
            }),
            id: Some(json!(21)),
        };
        let error = goal_state.parse_method().unwrap_err();
        assert_eq!(error.code, -32602);
        assert_eq!(error.message, "unknown field unexpected");
    }

    #[test]
    fn parse_method_retrieval_methods_accept_common_envelope_fields() {
        let common = sample_common_fields();
        let recall = RuntimeMethodRequest {
            jsonrpc: "2.0".to_string(),
            method: "recall".to_string(),
            params: json!({
                "query_text": "memory:42",
                "namespace": "team.alpha",
                "result_budget": 3,
                "request_id": common.request_id,
                "workspace_id": common.workspace_id,
                "agent_id": common.agent_id,
                "session_id": common.session_id,
                "task_id": common.task_id,
                "time_budget_ms": common.time_budget_ms,
                "policy_context": common.policy_context,
            }),
            id: Some(json!(181)),
        };
        match recall.parse_method().unwrap() {
            RuntimeRequest::Recall { common, .. } => assert_eq!(common, sample_common_fields()),
            _ => std::process::abort(),
        }

        let inspect = RuntimeMethodRequest {
            jsonrpc: "2.0".to_string(),
            method: "inspect".to_string(),
            params: json!({
                "id": 42,
                "namespace": "team.alpha",
                "request_id": "req-common",
                "policy_context": {"include_public": false}
            }),
            id: Some(json!(182)),
        };
        match inspect.parse_method().unwrap() {
            RuntimeRequest::Inspect { common, .. } => {
                assert_eq!(common.request_id.as_deref(), Some("req-common"));
                assert_eq!(common.policy_context.unwrap().include_public, false);
            }
            _ => std::process::abort(),
        }

        let explain = RuntimeMethodRequest {
            jsonrpc: "2.0".to_string(),
            method: "explain".to_string(),
            params: json!({
                "query": "memory:42",
                "namespace": "team.alpha",
                "limit": 2,
                "agent_id": "agent-7"
            }),
            id: Some(json!(183)),
        };
        match explain.parse_method().unwrap() {
            RuntimeRequest::Explain { common, .. } => {
                assert_eq!(common.agent_id.as_deref(), Some("agent-7"));
            }
            _ => std::process::abort(),
        }

        let goal_state = RuntimeMethodRequest {
            jsonrpc: "2.0".to_string(),
            method: "goal_state".to_string(),
            params: json!({
                "namespace": "team.alpha",
                "task_id": "task-99",
                "request_id": "req-goal",
                "agent_id": "agent-11"
            }),
            id: Some(json!(184)),
        };
        match goal_state.parse_method().unwrap() {
            RuntimeRequest::GoalState {
                namespace,
                task_id,
                common,
            } => {
                assert_eq!(namespace, "team.alpha");
                assert_eq!(task_id.as_deref(), Some("task-99"));
                assert_eq!(common.request_id.as_deref(), Some("req-goal"));
                assert_eq!(common.agent_id.as_deref(), Some("agent-11"));
            }
            _ => std::process::abort(),
        }

        let goal_pin = RuntimeMethodRequest {
            jsonrpc: "2.0".to_string(),
            method: "goal_pin".to_string(),
            params: json!({
                "namespace": "team.alpha",
                "task_id": "task-99",
                "memory_id": 7,
                "request_id": "req-goal-pin"
            }),
            id: Some(json!(185)),
        };
        match goal_pin.parse_method().unwrap() {
            RuntimeRequest::GoalPin {
                namespace,
                task_id,
                memory_id,
                common,
            } => {
                assert_eq!(namespace, "team.alpha");
                assert_eq!(task_id.as_deref(), Some("task-99"));
                assert_eq!(memory_id, 7);
                assert_eq!(common.request_id.as_deref(), Some("req-goal-pin"));
            }
            _ => std::process::abort(),
        }

        let goal_snapshot = RuntimeMethodRequest {
            jsonrpc: "2.0".to_string(),
            method: "goal_snapshot".to_string(),
            params: json!({
                "namespace": "team.alpha",
                "task_id": "task-99",
                "note": "handoff",
                "agent_id": "agent-12"
            }),
            id: Some(json!(186)),
        };
        match goal_snapshot.parse_method().unwrap() {
            RuntimeRequest::GoalSnapshot {
                namespace,
                task_id,
                note,
                common,
            } => {
                assert_eq!(namespace, "team.alpha");
                assert_eq!(task_id.as_deref(), Some("task-99"));
                assert_eq!(note.as_deref(), Some("handoff"));
                assert_eq!(common.agent_id.as_deref(), Some("agent-12"));
            }
            _ => std::process::abort(),
        }
    }

    #[test]
    fn parse_method_observe_accepts_required_and_optional_fields() {
        let request = RuntimeMethodRequest {
            jsonrpc: "2.0".to_string(),
            method: "observe".to_string(),
            params: json!({
                "content": "streamed content",
                "namespace": "team.alpha",
                "context": "coding session",
                "chunk_size": 120,
                "source_label": "stdin:test",
                "topic_threshold": 0.4,
                "min_chunk_size": 24,
                "dry_run": true,
                "request_id": "req-observe"
            }),
            id: Some(json!(29)),
        };

        match request.parse_method().unwrap() {
            RuntimeRequest::Observe {
                content,
                namespace,
                context,
                chunk_size,
                source_label,
                topic_threshold,
                min_chunk_size,
                dry_run,
                common,
            } => {
                assert_eq!(content, "streamed content");
                assert_eq!(namespace, "team.alpha");
                assert_eq!(context.as_deref(), Some("coding session"));
                assert_eq!(chunk_size, Some(120));
                assert_eq!(source_label.as_deref(), Some("stdin:test"));
                assert_eq!(topic_threshold, Some(0.4));
                assert_eq!(min_chunk_size, Some(24));
                assert_eq!(dry_run, Some(true));
                assert_eq!(common.request_id.as_deref(), Some("req-observe"));
            }
            _ => std::process::abort(),
        }
    }

    #[test]
    fn parse_method_skills_accepts_namespace_extract_and_common_fields() {
        let request = RuntimeMethodRequest {
            jsonrpc: "2.0".to_string(),
            method: "skills".to_string(),
            params: json!({
                "namespace": "team.alpha",
                "extract": true,
                "request_id": "req-skills"
            }),
            id: Some(json!(291)),
        };

        match request.parse_method().unwrap() {
            RuntimeRequest::Skills {
                namespace,
                extract,
                common,
            } => {
                assert_eq!(namespace, "team.alpha");
                assert_eq!(extract, Some(true));
                assert_eq!(common.request_id.as_deref(), Some("req-skills"));
            }
            _ => std::process::abort(),
        }
    }

    #[test]
    fn parse_method_procedures_accepts_mutation_and_common_fields() {
        let request = RuntimeMethodRequest {
            jsonrpc: "2.0".to_string(),
            method: "procedures".to_string(),
            params: json!({
                "namespace": "team.alpha",
                "promote": "procedural://team.alpha/000000000000002a",
                "note": "approved",
                "approved_by": "daemon.user",
                "public": true,
                "request_id": "req-procedures"
            }),
            id: Some(json!(292)),
        };

        match request.parse_method().unwrap() {
            RuntimeRequest::Procedures {
                namespace,
                promote,
                note,
                approved_by,
                public,
                common,
                ..
            } => {
                assert_eq!(namespace, "team.alpha");
                assert_eq!(
                    promote.as_deref(),
                    Some("procedural://team.alpha/000000000000002a")
                );
                assert_eq!(note.as_deref(), Some("approved"));
                assert_eq!(approved_by.as_deref(), Some("daemon.user"));
                assert_eq!(public, Some(true));
                assert_eq!(common.request_id.as_deref(), Some("req-procedures"));
            }
            _ => std::process::abort(),
        }
    }

    #[test]
    fn parse_method_encode_accepts_required_and_optional_fields() {
        let request = RuntimeMethodRequest {
            jsonrpc: "2.0".to_string(),
            method: "encode".to_string(),
            params: json!({
                "content": "user prefers dark mode",
                "namespace": "team.alpha",
                "memory_type": "user_preference",
                "visibility": "shared",
                "request_id": "req-encode"
            }),
            id: Some(json!(30)),
        };

        match request.parse_method().unwrap() {
            RuntimeRequest::Encode {
                content,
                namespace,
                memory_type,
                visibility,
                emotional_annotations: _,
                common,
            } => {
                assert_eq!(content, "user prefers dark mode");
                assert_eq!(namespace, "team.alpha");
                assert_eq!(memory_type, Some("user_preference".to_string()));
                assert_eq!(visibility, Some("shared".to_string()));
                assert_eq!(common.request_id.as_deref(), Some("req-encode"));
            }
            _ => std::process::abort(),
        }

        // Without optional memory_type
        let minimal = RuntimeMethodRequest {
            jsonrpc: "2.0".to_string(),
            method: "encode".to_string(),
            params: json!({
                "content": "hello world",
                "namespace": "team.alpha"
            }),
            id: Some(json!(31)),
        };

        match minimal.parse_method().unwrap() {
            RuntimeRequest::Encode {
                memory_type,
                visibility,
                common,
                ..
            } => {
                assert!(memory_type.is_none());
                assert!(visibility.is_none());
                assert_eq!(common, RuntimeCommonFields::default());
            }
            _ => std::process::abort(),
        }
    }

    #[test]
    fn parse_method_encode_rejects_missing_required_fields() {
        let missing_content = RuntimeMethodRequest {
            jsonrpc: "2.0".to_string(),
            method: "encode".to_string(),
            params: json!({"namespace": "team.alpha"}),
            id: Some(json!(32)),
        };
        let error = missing_content.parse_method().unwrap_err();
        assert_eq!(error.code, -32602);
        assert_eq!(error.message, "missing content");

        let missing_ns = RuntimeMethodRequest {
            jsonrpc: "2.0".to_string(),
            method: "encode".to_string(),
            params: json!({"content": "hello"}),
            id: Some(json!(33)),
        };
        let error = missing_ns.parse_method().unwrap_err();
        assert_eq!(error.code, -32602);
        assert_eq!(error.message, "missing namespace");
    }

    #[test]
    fn parse_method_forget_accepts_canonical_fields() {
        let request = RuntimeMethodRequest {
            jsonrpc: "2.0".to_string(),
            method: "forget".to_string(),
            params: json!({
                "id": 42,
                "namespace": "team.alpha",
                "mode": "archive",
                "reason": "stale data"
            }),
            id: Some(json!(34)),
        };

        match request.parse_method().unwrap() {
            RuntimeRequest::Forget {
                id,
                namespace,
                mode,
                reason,
            } => {
                assert_eq!(id, 42);
                assert_eq!(namespace, "team.alpha");
                assert_eq!(mode, Some("archive".to_string()));
                assert_eq!(reason, Some("stale data".to_string()));
            }
            _ => std::process::abort(),
        }
    }

    #[test]
    fn parse_method_pin_accepts_canonical_fields() {
        let request = RuntimeMethodRequest {
            jsonrpc: "2.0".to_string(),
            method: "pin".to_string(),
            params: json!({
                "id": 42,
                "namespace": "team.alpha",
                "reason": "critical reference"
            }),
            id: Some(json!(35)),
        };

        match request.parse_method().unwrap() {
            RuntimeRequest::Pin {
                id,
                namespace,
                reason,
            } => {
                assert_eq!(id, 42);
                assert_eq!(namespace, "team.alpha");
                assert_eq!(reason, Some("critical reference".to_string()));
            }
            _ => std::process::abort(),
        }
    }

    #[test]
    fn parse_method_consolidate_accepts_canonical_fields() {
        let request = RuntimeMethodRequest {
            jsonrpc: "2.0".to_string(),
            method: "consolidate".to_string(),
            params: json!({
                "namespace": "team.alpha",
                "scope": "session"
            }),
            id: Some(json!(36)),
        };

        match request.parse_method().unwrap() {
            RuntimeRequest::Consolidate { namespace, scope } => {
                assert_eq!(namespace, "team.alpha");
                assert_eq!(scope, Some("session".to_string()));
            }
            _ => std::process::abort(),
        }
    }

    #[test]
    fn parse_method_share_and_unshare_accept_canonical_fields() {
        let share = RuntimeMethodRequest {
            jsonrpc: "2.0".to_string(),
            method: "share".to_string(),
            params: json!({
                "id": 42,
                "namespace_id": "team.beta"
            }),
            id: Some(json!(37)),
        };

        match share.parse_method().unwrap() {
            RuntimeRequest::Share {
                id,
                namespace_id,
                common,
            } => {
                assert_eq!(id, 42);
                assert_eq!(namespace_id, "team.beta");
                assert_eq!(common, RuntimeCommonFields::default());
            }
            _ => std::process::abort(),
        }

        let unshare = RuntimeMethodRequest {
            jsonrpc: "2.0".to_string(),
            method: "unshare".to_string(),
            params: json!({
                "id": 42,
                "namespace": "team.alpha"
            }),
            id: Some(json!(38)),
        };

        match unshare.parse_method().unwrap() {
            RuntimeRequest::Unshare {
                id,
                namespace,
                common,
            } => {
                assert_eq!(id, 42);
                assert_eq!(namespace, "team.alpha");
                assert_eq!(common, RuntimeCommonFields::default());
            }
            _ => std::process::abort(),
        }
    }

    #[test]
    fn parse_method_fork_and_merge_fork_accept_canonical_fields() {
        let fork = RuntimeMethodRequest {
            jsonrpc: "2.0".to_string(),
            method: "fork".to_string(),
            params: json!({
                "name": "agent-specialist",
                "namespace": "team.alpha",
                "inherit": "shared",
                "note": "testing"
            }),
            id: Some(json!(39)),
        };
        match fork.parse_method().unwrap() {
            RuntimeRequest::Fork {
                name,
                namespace,
                parent_namespace,
                inherit,
                note,
                common,
            } => {
                assert_eq!(name, "agent-specialist");
                assert_eq!(namespace, "team.alpha");
                assert_eq!(parent_namespace, None);
                assert_eq!(inherit.as_deref(), Some("shared"));
                assert_eq!(note.as_deref(), Some("testing"));
                assert_eq!(common, RuntimeCommonFields::default());
            }
            _ => std::process::abort(),
        }

        let merge = RuntimeMethodRequest {
            jsonrpc: "2.0".to_string(),
            method: "merge_fork".to_string(),
            params: json!({
                "fork_name": "agent-specialist",
                "target_namespace": "team.alpha",
                "conflict_strategy": "recency-wins",
                "dry_run": true
            }),
            id: Some(json!(40)),
        };
        match merge.parse_method().unwrap() {
            RuntimeRequest::MergeFork {
                fork_name,
                target_namespace,
                conflict_strategy,
                dry_run,
                common,
            } => {
                assert_eq!(fork_name, "agent-specialist");
                assert_eq!(target_namespace, "team.alpha");
                assert_eq!(conflict_strategy.as_deref(), Some("recency-wins"));
                assert!(dry_run);
                assert_eq!(common, RuntimeCommonFields::default());
            }
            _ => std::process::abort(),
        }
    }

    #[test]
    fn parse_method_share_and_unshare_reject_unknown_fields() {
        let share = RuntimeMethodRequest {
            jsonrpc: "2.0".to_string(),
            method: "share".to_string(),
            params: json!({
                "id": 42,
                "namespace_id": "team.beta",
                "unexpected": true
            }),
            id: Some(json!(39)),
        };
        let error = share.parse_method().unwrap_err();
        assert_eq!(error.code, -32602);
        assert_eq!(error.message, "unknown field unexpected");

        let unshare = RuntimeMethodRequest {
            jsonrpc: "2.0".to_string(),
            method: "unshare".to_string(),
            params: json!({
                "id": 42,
                "namespace": "team.alpha",
                "unexpected": true
            }),
            id: Some(json!(40)),
        };
        let error = unshare.parse_method().unwrap_err();
        assert_eq!(error.code, -32602);
        assert_eq!(error.message, "unknown field unexpected");
    }

    #[test]
    fn parse_method_mutation_methods_accept_common_envelope_fields() {
        let common = sample_common_fields();
        let share = RuntimeMethodRequest {
            jsonrpc: "2.0".to_string(),
            method: "share".to_string(),
            params: json!({
                "id": 42,
                "namespace_id": "team.beta",
                "request_id": common.request_id,
                "workspace_id": common.workspace_id,
                "agent_id": common.agent_id,
                "session_id": common.session_id,
                "task_id": common.task_id,
                "time_budget_ms": common.time_budget_ms,
                "policy_context": common.policy_context,
            }),
            id: Some(json!(391)),
        };
        match share.parse_method().unwrap() {
            RuntimeRequest::Share { common, .. } => assert_eq!(common, sample_common_fields()),
            _ => std::process::abort(),
        }

        let unshare = RuntimeMethodRequest {
            jsonrpc: "2.0".to_string(),
            method: "unshare".to_string(),
            params: json!({
                "id": 42,
                "namespace": "team.alpha",
                "request_id": "req-unshare",
                "policy_context": {"include_public": false, "sharing_visibility": "private"}
            }),
            id: Some(json!(392)),
        };
        match unshare.parse_method().unwrap() {
            RuntimeRequest::Unshare { common, .. } => {
                assert_eq!(common.request_id.as_deref(), Some("req-unshare"));
                assert_eq!(
                    common.policy_context.unwrap().sharing_visibility.as_deref(),
                    Some("private")
                );
            }
            _ => std::process::abort(),
        }
    }

    #[test]
    fn parse_method_link_accepts_canonical_fields() {
        let request = RuntimeMethodRequest {
            jsonrpc: "2.0".to_string(),
            method: "link".to_string(),
            params: json!({
                "source_id": 42,
                "target_id": 99,
                "namespace": "team.alpha",
                "link_type": "supports"
            }),
            id: Some(json!(39)),
        };

        match request.parse_method().unwrap() {
            RuntimeRequest::Link {
                source_id,
                target_id,
                namespace,
                link_type,
                common,
            } => {
                assert_eq!(source_id, 42);
                assert_eq!(target_id, 99);
                assert_eq!(namespace, "team.alpha");
                assert_eq!(link_type, Some("supports".to_string()));
                assert_eq!(common, RuntimeCommonFields::default());
            }
            _ => std::process::abort(),
        }
    }

    #[test]
    fn parse_method_link_accepts_common_envelope_fields() {
        let request = RuntimeMethodRequest {
            jsonrpc: "2.0".to_string(),
            method: "link".to_string(),
            params: json!({
                "source_id": 42,
                "target_id": 99,
                "namespace": "team.alpha",
                "link_type": "supports",
                "request_id": "req-link",
                "task_id": "task-99"
            }),
            id: Some(json!(399)),
        };

        match request.parse_method().unwrap() {
            RuntimeRequest::Link { common, .. } => {
                assert_eq!(common.request_id.as_deref(), Some("req-link"));
                assert_eq!(common.task_id.as_deref(), Some("task-99"));
            }
            _ => std::process::abort(),
        }
    }

    #[test]
    fn mutation_methods_reject_unknown_fields() {
        let forget = RuntimeMethodRequest {
            jsonrpc: "2.0".to_string(),
            method: "forget".to_string(),
            params: json!({
                "id": 42,
                "namespace": "team.alpha",
                "unexpected": true
            }),
            id: Some(json!(40)),
        };
        let error = forget.parse_method().unwrap_err();
        assert_eq!(error.code, -32602);
        assert_eq!(error.message, "unknown field unexpected");

        let pin = RuntimeMethodRequest {
            jsonrpc: "2.0".to_string(),
            method: "pin".to_string(),
            params: json!({
                "id": 42,
                "namespace": "team.alpha",
                "unexpected": true
            }),
            id: Some(json!(41)),
        };
        let error = pin.parse_method().unwrap_err();
        assert_eq!(error.code, -32602);

        let encode = RuntimeMethodRequest {
            jsonrpc: "2.0".to_string(),
            method: "encode".to_string(),
            params: json!({
                "content": "hello",
                "namespace": "team.alpha",
                "unexpected": true
            }),
            id: Some(json!(42)),
        };
        let error = encode.parse_method().unwrap_err();
        assert_eq!(error.code, -32602);

        let encode_visibility = RuntimeMethodRequest {
            jsonrpc: "2.0".to_string(),
            method: "encode".to_string(),
            params: json!({
                "content": "hello",
                "namespace": "team.alpha",
                "visibility": "shared",
                "request_id": "req-encode-visibility"
            }),
            id: Some(json!(420)),
        };
        match encode_visibility.parse_method().unwrap() {
            RuntimeRequest::Encode {
                visibility, common, ..
            } => {
                assert_eq!(visibility.as_deref(), Some("shared"));
                assert_eq!(common.request_id.as_deref(), Some("req-encode-visibility"));
            }
            _ => std::process::abort(),
        }

        let link = RuntimeMethodRequest {
            jsonrpc: "2.0".to_string(),
            method: "link".to_string(),
            params: json!({
                "source_id": 1,
                "target_id": 2,
                "namespace": "team.alpha",
                "unexpected": true
            }),
            id: Some(json!(43)),
        };
        let error = link.parse_method().unwrap_err();
        assert_eq!(error.code, -32602);
    }
}
