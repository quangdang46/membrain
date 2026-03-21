use membrain_core::api::{NamespaceId, RequestId};
use membrain_core::engine::result::RetrievalResultSet;
use membrain_core::observability::OutcomeClass;
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
#[serde(deny_unknown_fields)]
pub struct EncodeParams {
    pub content: String,
    pub namespace: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub payload_type: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RecallParams {
    pub query: String,
    pub namespace: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub limit: Option<usize>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct InspectParams {
    pub id: u64,
    pub namespace: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ExplainParams {
    pub query: String,
    pub namespace: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub limit: Option<usize>,
}

/// Canonical MCP Response Envelope.
///
/// Retrieval-facing tools should prefer the explicit `retrieval` payload so MCP preserves the
/// same stable envelope family as CLI JSON and daemon/JSON-RPC instead of hiding it behind an
/// untyped blob.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct McpResponse {
    pub status: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub retrieval: Option<McpRetrievalPayload>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub payload: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<McpError>,
}

impl McpResponse {
    pub fn success(payload: serde_json::Value) -> Self {
        Self {
            status: "ok".to_string(),
            retrieval: None,
            payload: Some(payload),
            error: None,
        }
    }

    pub fn retrieval_success(retrieval: McpRetrievalPayload) -> Self {
        Self {
            status: "ok".to_string(),
            retrieval: Some(retrieval),
            payload: None,
            error: None,
        }
    }

    pub fn failure(error: McpError) -> Self {
        Self {
            status: "error".to_string(),
            retrieval: None,
            payload: None,
            error: Some(error),
        }
    }
}

/// MCP retrieval payload preserving the canonical retrieval-result envelope families.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct McpRetrievalPayload {
    pub request_id: RequestId,
    pub namespace: NamespaceId,
    pub outcome_class: OutcomeClass,
    pub partial_success: bool,
    pub explain_trace: McpExplainTrace,
    pub result: RetrievalResultSet,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct McpExplainTrace {
    pub route_summary: serde_json::Value,
    pub trace_stages: Vec<String>,
    pub result_reasons: serde_json::Value,
    pub omitted_summary: serde_json::Value,
    pub policy_summary: serde_json::Value,
    pub provenance_summary: serde_json::Value,
    pub freshness_markers: serde_json::Value,
    pub conflict_markers: serde_json::Value,
    pub uncertainty_markers: serde_json::Value,
}

impl McpRetrievalPayload {
    pub fn from_result(
        request_id: RequestId,
        namespace: NamespaceId,
        partial_success: bool,
        result: RetrievalResultSet,
    ) -> Result<Self, serde_json::Error> {
        let partial_success = partial_success
            || matches!(result.outcome_class, OutcomeClass::Partial)
            || result.truncated;
        let outcome_class = result.outcome_class;
        let (route_summary, trace_stages) = result.explain_route();
        let result_reasons = result.explain_result_reasons();
        let (policy_summary, provenance_summary) = result.explain_policy_and_provenance();
        let (freshness_markers, conflict_markers, uncertainty_markers) = result.explain_markers();
        let explain_trace = McpExplainTrace {
            route_summary: serde_json::to_value(&route_summary)?,
            trace_stages: trace_stages
                .into_iter()
                .map(|stage| stage.as_str().to_string())
                .collect(),
            result_reasons: serde_json::to_value(&result_reasons)?,
            omitted_summary: serde_json::to_value(&result.omitted_summary)?,
            policy_summary: serde_json::to_value(&policy_summary)?,
            provenance_summary: serde_json::to_value(&provenance_summary)?,
            freshness_markers: serde_json::to_value(&freshness_markers)?,
            conflict_markers: serde_json::to_value(&conflict_markers)?,
            uncertainty_markers: serde_json::to_value(&uncertainty_markers)?,
        };

        Ok(Self {
            request_id,
            namespace,
            outcome_class,
            partial_success,
            explain_trace,
            result,
        })
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
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

#[cfg(test)]
mod tests {
    use super::{
        EncodeParams, ExplainParams, InspectParams, McpError, McpRequest, McpResponse,
        McpRetrievalPayload, RecallParams,
    };
    use membrain_core::api::{NamespaceId, RequestId};
    use membrain_core::engine::recall::RecallPlanKind;
    use membrain_core::engine::result::RetrievalExplain;
    use membrain_core::engine::result::RetrievalResultSet;
    use membrain_core::engine::result::{DualOutputMode, PackagingMetadata};
    use membrain_core::observability::OutcomeClass;

    fn sample_result_set() -> RetrievalResultSet {
        RetrievalResultSet {
            outcome_class: OutcomeClass::Accepted,
            evidence_pack: Vec::new(),
            action_pack: None,
            deferred_payloads: Vec::new(),
            explain: RetrievalExplain {
                recall_plan: RecallPlanKind::ExactIdTier1,
                route_reason: "tier1 exact route".to_string(),
                tiers_consulted: vec!["tier1_exact".to_string()],
                trace_stages: Vec::new(),
                tier1_answered_directly: true,
                candidate_budget: 8,
                time_consumed_ms: Some(1),
                ranking_profile: "balanced".to_string(),
                contradictions_found: 0,
                result_reasons: Vec::new(),
            },
            policy_summary: membrain_core::engine::result::PolicySummary {
                namespace_applied: NamespaceId::new("mcp.team").unwrap(),
                outcome_class: OutcomeClass::Accepted,
                redactions_applied: false,
                restrictions_active: Vec::new(),
                filters: Vec::new(),
            },
            provenance_summary: membrain_core::engine::result::ProvenanceSummary {
                source_kind: "retrieval_pipeline".to_string(),
                source_reference: "result_set".to_string(),
                source_agent: "mcp".to_string(),
                original_namespace: NamespaceId::new("mcp.team").unwrap(),
                derived_from: None,
                lineage_ancestors: Vec::new(),
                relation_to_seed: None,
                graph_seed: None,
            },
            omitted_summary: membrain_core::engine::result::OmissionSummary {
                policy_redacted: 0,
                threshold_dropped: 0,
                dedup_dropped: 0,
                budget_capped: 0,
                duplicate_collapsed: 0,
                low_confidence_suppressed: 0,
                stale_bypassed: 0,
            },
            freshness_markers: membrain_core::engine::result::FreshnessMarkers {
                oldest_item_days: 0,
                newest_item_days: 0,
                volatile_items_included: false,
                stale_warning: false,
                as_of_tick: None,
            },
            packaging_metadata: PackagingMetadata {
                result_budget: 8,
                token_budget: Some(256),
                graph_assistance: "none".to_string(),
                degraded_summary: None,
                packaging_mode: "bounded".to_string(),
                rerank_metadata: None,
            },
            output_mode: DualOutputMode::Balanced,
            truncated: false,
            total_candidates: 0,
        }
    }

    #[test]
    fn retrieval_payload_preserves_canonical_result_families() {
        let payload = McpRetrievalPayload::from_result(
            RequestId::new("req-1").unwrap(),
            NamespaceId::new("mcp.team").unwrap(),
            false,
            sample_result_set(),
        )
        .unwrap();

        let json = serde_json::to_value(&payload).unwrap();
        assert_eq!(json["request_id"], "req-1");
        assert_eq!(json["namespace"], "mcp.team");
        assert_eq!(json["outcome_class"], "Accepted");
        assert!(json["result"].get("evidence_pack").is_some());
        assert!(json["result"].get("action_pack").is_some());
        assert!(json["result"].get("deferred_payloads").is_some());
        assert!(json["result"].get("omitted_summary").is_some());
        assert!(json["result"].get("policy_summary").is_some());
        assert!(json["result"].get("provenance_summary").is_some());
        assert!(json["result"].get("freshness_markers").is_some());
        assert!(json["result"].get("packaging_metadata").is_some());
        assert!(json["result"].get("explain").is_some());
        assert_eq!(json["explain_trace"]["route_summary"]["route_family"], "exact_id_tier1");
        assert!(json["explain_trace"].get("omitted_summary").is_some());
        assert!(json["explain_trace"].get("policy_summary").is_some());
        assert!(json["explain_trace"].get("provenance_summary").is_some());
        assert!(json["explain_trace"].get("freshness_markers").is_some());
        assert!(json["explain_trace"].get("conflict_markers").is_some());
        assert!(json["explain_trace"].get("uncertainty_markers").is_some());
    }

    #[test]
    fn retrieval_payload_marks_partial_success_without_mutating_result() {
        let payload = McpRetrievalPayload::from_result(
            RequestId::new("req-2").unwrap(),
            NamespaceId::new("mcp.team").unwrap(),
            true,
            sample_result_set(),
        )
        .unwrap();

        assert_eq!(payload.outcome_class, OutcomeClass::Accepted);
        assert_eq!(payload.result.outcome_class, OutcomeClass::Accepted);
        assert_eq!(payload.explain_trace.omitted_summary["budget_capped"], 0);
        assert_eq!(payload.explain_trace.policy_summary["effective_namespace"], "mcp.team");
        assert_eq!(payload.explain_trace.provenance_summary["source_reference"], "result_set");
        assert!(payload.partial_success);
    }

    #[test]
    fn retrieval_payload_derives_partial_success_from_core_result() {
        let mut result = sample_result_set();
        result.outcome_class = OutcomeClass::Partial;

        let payload = McpRetrievalPayload::from_result(
            RequestId::new("req-2b").unwrap(),
            NamespaceId::new("mcp.team").unwrap(),
            false,
            result,
        )
        .unwrap();

        assert_eq!(payload.outcome_class, OutcomeClass::Partial);
        assert_eq!(payload.result.outcome_class, OutcomeClass::Partial);
        assert!(payload.partial_success);
    }

    #[test]
    fn retrieval_success_uses_typed_transport_slot() {
        let response = McpResponse::retrieval_success(
            McpRetrievalPayload::from_result(
                RequestId::new("req-3").unwrap(),
                NamespaceId::new("mcp.team").unwrap(),
                false,
                sample_result_set(),
            )
            .unwrap(),
        );

        let json = serde_json::to_value(&response).unwrap();
        assert_eq!(json["status"], "ok");
        assert!(json.get("retrieval").is_some());
        assert!(json["retrieval"].get("explain_trace").is_some());
        assert!(json.get("payload").is_none());
        assert!(json.get("error").is_none());
    }

    #[test]
    fn retrieval_payload_derives_partial_success_from_truncation_without_inventing_outcome() {
        let mut result = sample_result_set();
        result.truncated = true;

        let payload = McpRetrievalPayload::from_result(
            RequestId::new("req-4").unwrap(),
            NamespaceId::new("mcp.team").unwrap(),
            false,
            result,
        )
        .unwrap();

        assert_eq!(payload.outcome_class, OutcomeClass::Accepted);
        assert_eq!(payload.result.outcome_class, OutcomeClass::Accepted);
        assert!(payload.partial_success);
    }

    #[test]
    fn explain_request_round_trips_optional_limit() {
        let request = McpRequest::Explain(ExplainParams {
            query: "session:7".to_string(),
            namespace: "team.alpha".to_string(),
            limit: Some(2),
        });

        let json = serde_json::to_value(&request).unwrap();
        assert_eq!(json["method"], "explain");
        assert_eq!(json["params"]["query"], "session:7");
        assert_eq!(json["params"]["namespace"], "team.alpha");
        assert_eq!(json["params"]["limit"], 2);

        let decoded: McpRequest = serde_json::from_value(json).unwrap();
        match decoded {
            McpRequest::Explain(params) => assert_eq!(params.limit, Some(2)),
            other => panic!("unexpected decoded request: {other:?}"),
        }
    }

    #[test]
    fn explain_request_omits_limit_when_not_provided() {
        let request = McpRequest::Explain(ExplainParams {
            query: "session:7".to_string(),
            namespace: "team.alpha".to_string(),
            limit: None,
        });

        let json = serde_json::to_value(&request).unwrap();
        assert!(json["params"].get("limit").is_none());
    }

    #[test]
    fn mcp_param_structs_reject_unknown_fields() {
        let encode_error = serde_json::from_value::<EncodeParams>(serde_json::json!({
            "content": "hello",
            "namespace": "team.alpha",
            "unexpected": true
        }))
        .unwrap_err();
        assert!(encode_error.to_string().contains("unknown field `unexpected`"));

        let recall_error = serde_json::from_value::<RecallParams>(serde_json::json!({
            "query": "session:7",
            "namespace": "team.alpha",
            "limit": 3,
            "unexpected": true
        }))
        .unwrap_err();
        assert!(recall_error.to_string().contains("unknown field `unexpected`"));

        let inspect_error = serde_json::from_value::<InspectParams>(serde_json::json!({
            "id": 7,
            "namespace": "team.alpha",
            "unexpected": true
        }))
        .unwrap_err();
        assert!(inspect_error.to_string().contains("unknown field `unexpected`"));

        let explain_error = serde_json::from_value::<ExplainParams>(serde_json::json!({
            "query": "session:7",
            "namespace": "team.alpha",
            "limit": 2,
            "unexpected": true
        }))
        .unwrap_err();
        assert!(explain_error.to_string().contains("unknown field `unexpected`"));
    }

    #[test]
    fn failure_response_preserves_policy_denial_metadata() {
        let response = McpResponse::failure(McpError {
            code: "policy_denied".to_string(),
            message: "namespace isolation prevents export".to_string(),
            is_policy_denial: true,
        });

        let json = serde_json::to_value(&response).unwrap();
        assert_eq!(json["status"], "error");
        assert!(json.get("retrieval").is_none());
        assert!(json.get("payload").is_none());
        assert_eq!(json["error"]["code"], "policy_denied");
        assert_eq!(
            json["error"]["message"],
            "namespace isolation prevents export"
        );
        assert_eq!(json["error"]["is_policy_denial"], true);
    }

    #[test]
    fn mcp_response_rejects_unknown_top_level_fields() {
        let error = serde_json::from_value::<McpResponse>(serde_json::json!({
            "status": "ok",
            "retrieval": {
                "request_id": "req-5",
                "namespace": "mcp.team",
                "outcome_class": "Accepted",
                "partial_success": false,
                "explain_trace": {
                    "route_summary": {"route_family": "exact_id_tier1"},
                    "trace_stages": ["tier1_exact"],
                    "result_reasons": [],
                    "omitted_summary": {"budget_capped": 0},
                    "policy_summary": {"effective_namespace": "mcp.team"},
                    "provenance_summary": {"source_reference": "result_set"},
                    "freshness_markers": [],
                    "conflict_markers": [],
                    "uncertainty_markers": []
                },
                "result": serde_json::to_value(sample_result_set()).unwrap()
            },
            "unexpected": true
        }))
        .unwrap_err();

        assert!(error.to_string().contains("unknown field `unexpected`"));
    }

    #[test]
    fn mcp_response_rejects_unknown_retrieval_fields() {
        let error = serde_json::from_value::<McpResponse>(serde_json::json!({
            "status": "ok",
            "retrieval": {
                "request_id": "req-6",
                "namespace": "mcp.team",
                "outcome_class": "Accepted",
                "partial_success": false,
                "explain_trace": {
                    "route_summary": {"route_family": "exact_id_tier1"},
                    "trace_stages": ["tier1_exact"],
                    "result_reasons": [],
                    "omitted_summary": {"budget_capped": 0},
                    "policy_summary": {"effective_namespace": "mcp.team"},
                    "provenance_summary": {"source_reference": "result_set"},
                    "freshness_markers": [],
                    "conflict_markers": [],
                    "uncertainty_markers": []
                },
                "result": serde_json::to_value(sample_result_set()).unwrap(),
                "unexpected": true
            }
        }))
        .unwrap_err();

        assert!(error.to_string().contains("unknown field `unexpected`"));
    }

    #[test]
    fn mcp_response_rejects_unknown_error_fields() {
        let error = serde_json::from_value::<McpResponse>(serde_json::json!({
            "status": "error",
            "error": {
                "code": "policy_denied",
                "message": "namespace isolation prevents export",
                "is_policy_denial": true,
                "unexpected": true
            }
        }))
        .unwrap_err();

        assert!(error.to_string().contains("unknown field `unexpected`"));
    }
}
