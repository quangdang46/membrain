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
pub struct RuntimeMetrics {
    pub queue_depth: usize,
    pub active_requests: usize,
    pub background_jobs: usize,
    pub cancelled_requests: usize,
    pub maintenance_runs: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct RuntimeStatus {
    pub posture: RuntimePosture,
    pub degraded_reasons: Vec<String>,
    pub metrics: RuntimeMetrics,
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
pub struct RuntimeDoctorReport {
    pub status: &'static str,
    pub action: &'static str,
    pub posture: RuntimePosture,
    pub degraded_reasons: Vec<String>,
    pub metrics: RuntimeMetrics,
    pub indexes: Vec<RuntimeDoctorIndex>,
    pub warnings: Vec<&'static str>,
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

fn parse_optional_limit(params: &Value) -> Result<Option<usize>, JsonRpcError> {
    match params.get("limit") {
        None | Some(Value::Null) => Ok(None),
        Some(Value::Number(value)) => {
            let parsed = value.as_u64().and_then(|value| usize::try_from(value).ok());
            match parsed {
                Some(0) => Err(JsonRpcError {
                    code: -32602,
                    message: "limit must be at least 1".to_string(),
                    data: None,
                }),
                Some(limit) => Ok(Some(limit)),
                None => Err(JsonRpcError {
                    code: -32602,
                    message: "limit must be a positive integer".to_string(),
                    data: None,
                }),
            }
        }
        Some(_) => Err(JsonRpcError {
            code: -32602,
            message: "limit must be a positive integer".to_string(),
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
            "doctor" => Ok(RuntimeRequest::Doctor),
            "recall" => {
                reject_unknown_fields(&self.params, &["query", "namespace", "limit"])?;
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
                Ok(RuntimeRequest::Recall {
                    query: query.to_string(),
                    namespace: namespace.to_string(),
                    limit,
                })
            }
            "inspect" => {
                reject_unknown_fields(&self.params, &["id", "namespace"])?;
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
                })
            }
            "explain" => {
                reject_unknown_fields(&self.params, &["query", "namespace", "limit"])?;
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
                Ok(RuntimeRequest::Explain {
                    query: query.to_string(),
                    namespace: namespace.to_string(),
                    limit,
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
                    &["namespace", "authorization_token", "bypass_flags"],
                )?;
                Ok(RuntimeRequest::PreflightAllow {
                    namespace: parse_required_string(&self.params, "namespace")?,
                    authorization_token: parse_required_string(
                        &self.params,
                        "authorization_token",
                    )?,
                    bypass_flags: parse_string_array(&self.params, "bypass_flags")?,
                })
            }
            "sleep" => {
                let millis = self
                    .params
                    .get("millis")
                    .and_then(Value::as_u64)
                    .unwrap_or(0);
                Ok(RuntimeRequest::Sleep { millis })
            }
            "set_posture" => {
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
                let polls_budget = parse_optional_u32(&self.params, "polls_budget")?;
                let step_delay_ms =
                    parse_optional_u32(&self.params, "step_delay_ms")?.map(u64::from);
                Ok(RuntimeRequest::RunMaintenance {
                    polls_budget,
                    step_delay_ms,
                })
            }
            "shutdown" => Ok(RuntimeRequest::Shutdown),
            _ => Err(JsonRpcError {
                code: -32601,
                message: format!("unknown method '{}'", self.method),
                data: None,
            }),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RuntimeRequest {
    Ping,
    Status,
    Doctor,
    Recall {
        query: String,
        namespace: String,
        limit: Option<usize>,
    },
    Inspect {
        id: u64,
        namespace: String,
    },
    Explain {
        query: String,
        namespace: String,
        limit: Option<usize>,
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
        authorization_token: String,
        bypass_flags: Vec<String>,
    },
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
                query: "memory:42".to_string(),
                namespace: "team.alpha".to_string(),
                limit: Some(3),
            }
        );
    }

    #[test]
    fn parse_method_recall_requires_query_and_namespace() {
        let missing_query = RuntimeMethodRequest {
            jsonrpc: "2.0".to_string(),
            method: "recall".to_string(),
            params: json!({ "namespace": "team.alpha" }),
            id: Some(json!(1)),
        };
        let error = missing_query.parse_method().unwrap_err();
        assert_eq!(error.code, -32602);
        assert_eq!(error.message, "missing query");

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
    fn parse_method_recall_rejects_non_positive_or_non_integer_limits() {
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
                "authorization_token": "token-123",
                "bypass_flags": ["manual_override"]
            }),
            id: Some(json!(12)),
        };
        assert_eq!(
            allow.parse_method().unwrap(),
            RuntimeRequest::PreflightAllow {
                namespace: "team.alpha".to_string(),
                authorization_token: "token-123".to_string(),
                bypass_flags: vec!["manual_override".to_string()],
            }
        );
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
                "authorization_token": "token-123",
                "bypass_flags": ["manual_override"],
                "unexpected": true
            }),
            id: Some(json!(17)),
        };
        let error = allow.parse_method().unwrap_err();
        assert_eq!(error.code, -32602);
        assert_eq!(error.message, "unknown field unexpected");
    }

    #[test]
    fn parse_method_retrieval_methods_reject_unknown_fields() {
        let recall = RuntimeMethodRequest {
            jsonrpc: "2.0".to_string(),
            method: "recall".to_string(),
            params: json!({
                "query": "memory:42",
                "namespace": "team.alpha",
                "limit": 3,
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
    }
}
