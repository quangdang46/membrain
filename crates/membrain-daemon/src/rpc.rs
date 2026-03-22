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
                    })
                }
                Some(value) => Some(value),
                None => {
                    return Err(JsonRpcError {
                        code: -32602,
                        message: "result_budget must be a positive integer".to_string(),
                        data: None,
                    })
                }
            }
        }
        Some(_) => {
            return Err(JsonRpcError {
                code: -32602,
                message: "result_budget must be a positive integer".to_string(),
                data: None,
            })
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
            "encode" => {
                reject_unknown_fields(&self.params, &["content", "namespace", "memory_type"])?;
                let content = parse_required_string(&self.params, "content")?;
                let namespace = parse_required_string(&self.params, "namespace")?;
                let memory_type = self
                    .params
                    .get("memory_type")
                    .and_then(Value::as_str)
                    .map(ToOwned::to_owned);
                Ok(RuntimeRequest::Encode {
                    content,
                    namespace,
                    memory_type,
                })
            }
            "recall" => {
                reject_unknown_fields(
                    &self.params,
                    &[
                        "query",
                        "query_text",
                        "namespace",
                        "limit",
                        "result_budget",
                        "context_text",
                        "effort",
                        "include_public",
                        "like_id",
                        "unlike_id",
                        "as_of_tick",
                        "at_snapshot",
                        "min_confidence",
                    ],
                )?;
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
                let result_budget = parse_optional_budget(&self.params)?;
                let context_text = parse_optional_string(&self.params, "context_text")?;
                let effort = parse_optional_string(&self.params, "effort")?;
                let include_public = parse_optional_bool(&self.params, "include_public")?;
                let like_id = parse_optional_positive_u64(&self.params, "like_id")?;
                let unlike_id = parse_optional_positive_u64(&self.params, "unlike_id")?;
                let as_of_tick = parse_optional_u64(&self.params, "as_of_tick")?;
                let at_snapshot = parse_optional_string(&self.params, "at_snapshot")?;
                let min_confidence = parse_optional_f64(&self.params, "min_confidence")?;

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
                    result_budget,
                    context_text,
                    effort,
                    include_public,
                    like_id,
                    unlike_id,
                    as_of_tick,
                    at_snapshot,
                    min_confidence,
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
                reject_unknown_fields(&self.params, &["id", "namespace_id"])?;
                let id = parse_required_u64(&self.params, "id")?;
                let namespace_id = parse_required_string(&self.params, "namespace_id")?;
                Ok(RuntimeRequest::Share { id, namespace_id })
            }
            "unshare" => {
                reject_unknown_fields(&self.params, &["id", "namespace"])?;
                let id = parse_required_u64(&self.params, "id")?;
                let namespace = parse_required_string(&self.params, "namespace")?;
                Ok(RuntimeRequest::Unshare { id, namespace })
            }
            "link" => {
                reject_unknown_fields(
                    &self.params,
                    &["source_id", "target_id", "namespace", "link_type"],
                )?;
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
                })
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
pub enum RuntimeRequest {
    Ping,
    Status,
    Doctor,
    Encode {
        content: String,
        namespace: String,
        memory_type: Option<String>,
    },
    Recall {
        query_text: Option<String>,
        namespace: String,
        result_budget: Option<usize>,
        context_text: Option<String>,
        effort: Option<String>,
        include_public: Option<bool>,
        like_id: Option<u64>,
        unlike_id: Option<u64>,
        as_of_tick: Option<u64>,
        at_snapshot: Option<String>,
        min_confidence: Option<f64>,
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
    },
    Unshare {
        id: u64,
        namespace: String,
    },
    Link {
        source_id: u64,
        target_id: u64,
        namespace: String,
        link_type: Option<String>,
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
                query_text: Some("memory:42".to_string()),
                namespace: "team.alpha".to_string(),
                result_budget: Some(3),
                context_text: None,
                effort: None,
                include_public: None,
                like_id: None,
                unlike_id: None,
                as_of_tick: None,
                at_snapshot: None,
                min_confidence: None,
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
    fn parse_method_recall_accepts_query_by_example_and_richer_contract_fields() {
        let request = RuntimeMethodRequest {
            jsonrpc: "2.0".to_string(),
            method: "recall".to_string(),
            params: json!({
                "query_text": "session:7",
                "namespace": "team.alpha",
                "result_budget": 4,
                "context_text": "triaging recall drift",
                "effort": "high",
                "include_public": true,
                "like_id": 11,
                "unlike_id": 17,
                "as_of_tick": 42,
                "min_confidence": 0.8
            }),
            id: Some(json!(18)),
        };

        assert_eq!(
            request.parse_method().unwrap(),
            RuntimeRequest::Recall {
                query_text: Some("session:7".to_string()),
                namespace: "team.alpha".to_string(),
                result_budget: Some(4),
                context_text: Some("triaging recall drift".to_string()),
                effort: Some("high".to_string()),
                include_public: Some(true),
                like_id: Some(11),
                unlike_id: Some(17),
                as_of_tick: Some(42),
                at_snapshot: None,
                min_confidence: Some(0.8),
            }
        );

        let example_only = RuntimeMethodRequest {
            jsonrpc: "2.0".to_string(),
            method: "recall".to_string(),
            params: json!({
                "namespace": "team.alpha",
                "like_id": 99,
                "result_budget": 2
            }),
            id: Some(json!(19)),
        };

        assert_eq!(
            example_only.parse_method().unwrap(),
            RuntimeRequest::Recall {
                query_text: None,
                namespace: "team.alpha".to_string(),
                result_budget: Some(2),
                context_text: None,
                effort: None,
                include_public: None,
                like_id: Some(99),
                unlike_id: None,
                as_of_tick: None,
                at_snapshot: None,
                min_confidence: None,
            }
        );
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
    }

    #[test]
    fn parse_method_encode_accepts_required_and_optional_fields() {
        let request = RuntimeMethodRequest {
            jsonrpc: "2.0".to_string(),
            method: "encode".to_string(),
            params: json!({
                "content": "user prefers dark mode",
                "namespace": "team.alpha",
                "memory_type": "user_preference"
            }),
            id: Some(json!(30)),
        };

        match request.parse_method().unwrap() {
            RuntimeRequest::Encode {
                content,
                namespace,
                memory_type,
            } => {
                assert_eq!(content, "user prefers dark mode");
                assert_eq!(namespace, "team.alpha");
                assert_eq!(memory_type, Some("user_preference".to_string()));
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
            RuntimeRequest::Encode { memory_type, .. } => assert!(memory_type.is_none()),
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
            RuntimeRequest::Share { id, namespace_id } => {
                assert_eq!(id, 42);
                assert_eq!(namespace_id, "team.beta");
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
            RuntimeRequest::Unshare { id, namespace } => {
                assert_eq!(id, 42);
                assert_eq!(namespace, "team.alpha");
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
            } => {
                assert_eq!(source_id, 42);
                assert_eq!(target_id, 99);
                assert_eq!(namespace, "team.alpha");
                assert_eq!(link_type, Some("supports".to_string()));
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
