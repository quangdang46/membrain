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
            "recall" => {
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
                let limit = match self.params.get("limit") {
                    None | Some(Value::Null) => None,
                    Some(Value::Number(value)) => {
                        let parsed = value.as_u64().and_then(|value| usize::try_from(value).ok());
                        match parsed {
                            Some(0) => {
                                return Err(JsonRpcError {
                                    code: -32602,
                                    message: "limit must be at least 1".to_string(),
                                    data: None,
                                });
                            }
                            Some(limit) => Some(limit),
                            None => {
                                return Err(JsonRpcError {
                                    code: -32602,
                                    message: "limit must be a positive integer".to_string(),
                                    data: None,
                                });
                            }
                        }
                    }
                    Some(_) => {
                        return Err(JsonRpcError {
                            code: -32602,
                            message: "limit must be a positive integer".to_string(),
                            data: None,
                        });
                    }
                };
                Ok(RuntimeRequest::Recall {
                    query: query.to_string(),
                    namespace: namespace.to_string(),
                    limit,
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
                            item.as_str().map(ToOwned::to_owned).ok_or_else(|| JsonRpcError {
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
                let polls_budget = self
                    .params
                    .get("polls_budget")
                    .and_then(Value::as_u64)
                    .and_then(|value| u32::try_from(value).ok());
                let step_delay_ms = self.params.get("step_delay_ms").and_then(Value::as_u64);
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
    Recall {
        query: String,
        namespace: String,
        limit: Option<usize>,
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
}
