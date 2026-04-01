use crate::api::{
    AgentId, CacheMetricsSummary, ConflictMarker, FreshnessMarker, GraphExpansionSummary,
    NamespaceId, PassiveObservationInspectSummary, RequestId, ResponseContext,
    TraceOmissionSummary, TracePolicySummary, TraceProvenanceSummary, TraceScoreComponent,
    TraceStage, UncertaintyMarker, WorkspaceId,
};
use crate::engine::encode::PreparedEncodeCandidate;
use crate::engine::repair::RepairTarget;
use crate::engine::result::{ResultReason as RetrievalResultReason, RetrievalResultSet};
use crate::engine::semantic_retrieval::SemanticRetrievalResult;
use crate::graph::{CausalEvidenceAttribution, CausalEvidenceKind, CausalLink, CausalLinkType};
use crate::policy::SharingVisibility;
use crate::reasoning::QueryRewriteOutcome;
use crate::types::{
    AffectSignals, CanonicalMemoryType, FastPathRouteFamily, MemoryId, RawEncodeInput,
    RawIntakeKind, SessionId,
};
use crate::BrainStore;

type ResponseTraceBundle = (
    crate::api::RouteSummary,
    Vec<TraceStage>,
    Vec<crate::api::ResultReason>,
    TraceOmissionSummary,
    GraphExpansionSummary,
    Vec<TraceScoreComponent>,
    TracePolicySummary,
    TraceProvenanceSummary,
    Vec<FreshnessMarker>,
    Vec<ConflictMarker>,
    Vec<UncertaintyMarker>,
);

/// Shared in-memory record view used by transport operation helpers.
#[derive(Debug, Clone, PartialEq)]
pub struct OperationMemoryRecord {
    pub memory_id: MemoryId,
    pub namespace: NamespaceId,
    pub session_id: SessionId,
    pub memory_type: CanonicalMemoryType,
    pub route_family: FastPathRouteFamily,
    pub compact_text: String,
    pub provisional_salience: u16,
    pub affect: Option<AffectSignals>,
    pub fingerprint: u64,
    pub payload_size_bytes: usize,
    pub payload_state: String,
    pub visibility: SharingVisibility,
    pub is_landmark: bool,
    pub landmark_label: Option<String>,
    pub era_id: Option<String>,
    pub passive_observation: Option<PassiveObservationInspectSummary>,
    pub causal_parents: Vec<MemoryId>,
    pub causal_link_type: Option<CausalLinkType>,
    pub agent_id: Option<String>,
}

/// Shared encode request normalized before transport-specific persistence.
#[derive(Debug, Clone, PartialEq)]
pub struct EncodeOperationRequest {
    pub namespace: NamespaceId,
    pub memory_id: MemoryId,
    pub session_id: SessionId,
    pub content: String,
    pub intake_kind: RawIntakeKind,
    pub active_era_id: Option<String>,
    pub affect: Option<AffectSignals>,
    pub visibility: SharingVisibility,
    pub workspace_id: Option<String>,
    pub agent_id: Option<String>,
    pub payload_state: String,
}

/// Shared encode preparation result reused by CLI and daemon wrappers.
#[derive(Debug, Clone, PartialEq)]
pub struct PreparedOperationCandidate {
    pub prepared: PreparedEncodeCandidate,
    pub memory: OperationMemoryRecord,
}

/// Shared inspect surface derived from one in-memory operation record.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OperationInspectRecord {
    pub outcome: &'static str,
    pub memory_id: u64,
    pub namespace: String,
    pub memory_type: &'static str,
    pub route_family: &'static str,
    pub compact_text: String,
    pub provisional_salience: u16,
    pub fingerprint: u64,
    pub payload_size_bytes: usize,
    pub payload_state: String,
    pub visibility: SharingVisibility,
    pub is_landmark: bool,
    pub landmark_label: Option<String>,
    pub era_id: Option<String>,
    pub session_id: u64,
    pub passive_observation: Option<PassiveObservationInspectSummary>,
}

/// Shared transport operation helpers modeled after Claude Code-style reusable runtimes.
pub struct OperationEngine;

impl OperationEngine {
    /// Prepares one encode candidate with shared normalization, provenance, and visibility wiring.
    pub fn prepare_encode_candidate(
        store: &BrainStore,
        request: EncodeOperationRequest,
    ) -> PreparedOperationCandidate {
        let mut input = request
            .active_era_id
            .clone()
            .map(|era_id| {
                RawEncodeInput::new(request.intake_kind, request.content.as_str())
                    .with_active_era_id(era_id)
            })
            .unwrap_or_else(|| RawEncodeInput::new(request.intake_kind, request.content.as_str()));
        if let Some(affect) = request.affect {
            input = input.with_affect_signals(affect);
        }

        let mut prepared = store.encode_engine().prepare_fast_path(input);
        prepared.normalized.sharing.visibility = request.visibility;
        if let Some(workspace_id) = request.workspace_id.as_deref() {
            prepared.normalized.sharing.workspace_id = Some(WorkspaceId::new(workspace_id));
        }
        if let Some(agent_id) = request.agent_id.as_deref() {
            prepared.normalized.sharing.agent_id = Some(AgentId::new(agent_id));
        }

        let passive_observation = if prepared.captured_as_observation {
            Some(PassiveObservationInspectSummary::from_encode(
                &prepared.passive_observation_inspect,
            ))
        } else {
            None
        };

        let memory = OperationMemoryRecord {
            memory_id: request.memory_id,
            namespace: request.namespace,
            session_id: request.session_id,
            memory_type: prepared.normalized.memory_type,
            route_family: prepared.classification.route_family,
            compact_text: prepared.normalized.compact_text.clone(),
            provisional_salience: prepared.provisional_salience,
            affect: prepared.normalized.affect,
            fingerprint: prepared.fingerprint,
            payload_size_bytes: prepared.normalized.payload_size_bytes,
            payload_state: request.payload_state,
            visibility: prepared.normalized.sharing.visibility,
            is_landmark: prepared.normalized.landmark.is_landmark,
            landmark_label: prepared.normalized.landmark.landmark_label.clone(),
            era_id: prepared.normalized.landmark.era_id.clone(),
            passive_observation,
            causal_parents: Vec::new(),
            causal_link_type: None,
            agent_id: prepared
                .normalized
                .sharing
                .agent_id
                .as_ref()
                .map(|agent_id| agent_id.as_str().to_string()),
        };

        PreparedOperationCandidate { prepared, memory }
    }

    /// Builds a shared inspect record for one visible memory.
    pub fn inspect_record(
        records: &[OperationMemoryRecord],
        namespace: &NamespaceId,
        memory_id: MemoryId,
    ) -> Result<OperationInspectRecord, String> {
        let record = records
            .iter()
            .find(|record| record.namespace == *namespace && record.memory_id == memory_id)
            .ok_or_else(|| {
                format!(
                    "memory {} not found in namespace '{}'",
                    memory_id.0,
                    namespace.as_str()
                )
            })?;
        Ok(OperationInspectRecord {
            outcome: "accepted",
            memory_id: record.memory_id.0,
            namespace: record.namespace.as_str().to_string(),
            memory_type: record.memory_type.as_str(),
            route_family: record.route_family.as_str(),
            compact_text: record.compact_text.clone(),
            provisional_salience: record.provisional_salience,
            fingerprint: record.fingerprint,
            payload_size_bytes: record.payload_size_bytes,
            payload_state: record.payload_state.clone(),
            visibility: record.visibility,
            is_landmark: record.is_landmark,
            landmark_label: record.landmark_label.clone(),
            era_id: record.era_id.clone(),
            session_id: record.session_id.0,
            passive_observation: record.passive_observation.clone(),
        })
    }

    /// Builds shared causal links from transport-local memory records.
    pub fn causal_links_for_records(
        records: &[OperationMemoryRecord],
        namespace: &NamespaceId,
    ) -> Vec<CausalLink> {
        records
            .iter()
            .filter(|record| record.namespace == *namespace)
            .flat_map(|record| {
                record
                    .causal_parents
                    .iter()
                    .copied()
                    .map(move |parent_id| CausalLink {
                        src_memory_id: parent_id,
                        dst_memory_id: record.memory_id,
                        link_type: record.causal_link_type.unwrap_or(CausalLinkType::Derived),
                        created_at_ms: record.memory_id.0,
                        agent_id: record.agent_id.clone(),
                        evidence: vec![CausalEvidenceAttribution {
                            evidence_kind: CausalEvidenceKind::DurableMemory,
                            source_ref: format!(
                                "memory://{}/{}",
                                record.namespace.as_str(),
                                parent_id.0
                            ),
                            supporting_memory_ids: vec![parent_id],
                            confidence: 800,
                        }],
                    })
            })
            .collect()
    }

    /// Maps canonical retrieval result sets into shared response traces.
    pub fn response_trace_for_result_set(result_set: &RetrievalResultSet) -> ResponseTraceBundle {
        let route_summary = crate::api::RouteSummary::from_result_set(result_set);
        let trace_stages = result_set
            .explain
            .trace_stages
            .iter()
            .copied()
            .map(TraceStage::from_recall)
            .chain([TraceStage::PolicyGate, TraceStage::Packaging])
            .collect();
        let result_reasons = result_set
            .explain
            .result_reasons
            .iter()
            .map(crate::api::ResultReason::from_result_reason)
            .collect();
        let omitted_summary = TraceOmissionSummary::from_result_set(result_set);
        let graph_expansion = GraphExpansionSummary::from_result_set(result_set);
        let policy_summary = TracePolicySummary::from_result_set(result_set);
        let provenance_summary = TraceProvenanceSummary::from_result_set(result_set);
        let (freshness_markers, conflict_markers, uncertainty_markers) =
            result_set.explain_markers();
        let freshness_markers = freshness_markers
            .into_iter()
            .map(|marker| FreshnessMarker {
                code: marker.code,
                detail: marker.detail,
            })
            .collect();
        let conflict_markers = conflict_markers
            .into_iter()
            .map(|marker| ConflictMarker {
                code: marker.code,
                detail: marker.detail,
            })
            .collect();
        let uncertainty_markers = uncertainty_markers
            .into_iter()
            .map(|marker| UncertaintyMarker {
                code: marker.code,
                detail: marker.detail,
            })
            .collect();
        let score_components = result_set
            .top()
            .map(|top| {
                top.result
                    .score_summary
                    .signal_breakdown
                    .iter()
                    .map(
                        |(signal_family, raw_value, weight, _)| TraceScoreComponent {
                            signal_family: match signal_family.as_str() {
                                "recency" => "recency",
                                "salience" => "salience",
                                "strength" => "strength",
                                "provenance" => "provenance",
                                "conflict_adjustment" => "conflict_adjustment",
                                "confidence" => "confidence",
                                _ => "custom",
                            },
                            raw_value: *raw_value,
                            weight: *weight,
                        },
                    )
                    .collect()
            })
            .unwrap_or_default();

        (
            route_summary,
            trace_stages,
            result_reasons,
            omitted_summary,
            graph_expansion,
            score_components,
            policy_summary,
            provenance_summary,
            freshness_markers,
            conflict_markers,
            uncertainty_markers,
        )
    }

    /// Wraps one canonical retrieval result set in the shared response envelope.
    pub fn response_from_result_set(
        namespace: &NamespaceId,
        request_id: RequestId,
        result_set: RetrievalResultSet,
    ) -> ResponseContext<RetrievalResultSet> {
        let partial_success = matches!(
            result_set.outcome_class,
            crate::observability::OutcomeClass::Partial
        ) || result_set.truncated;
        let (
            route_summary,
            trace_stages,
            result_reasons,
            omitted_summary,
            graph_expansion,
            score_components,
            policy_summary,
            provenance_summary,
            freshness_markers,
            conflict_markers,
            uncertainty_markers,
        ) = Self::response_trace_for_result_set(&result_set);
        let policy_filters = policy_summary.filters.clone();
        let mut response = ResponseContext::success(namespace.clone(), request_id, result_set)
            .with_trace_schema(
                route_summary,
                trace_stages,
                result_reasons,
                omitted_summary,
                graph_expansion,
                CacheMetricsSummary::from_cache_traces(Vec::new(), false),
                score_components,
                policy_summary,
                provenance_summary,
                freshness_markers,
                conflict_markers,
                uncertainty_markers,
            );
        if !policy_filters.is_empty() {
            response = response.with_policy_filters(policy_filters);
        }
        if partial_success {
            response = response.with_partial_success();
        }
        response
    }

    /// Appends one shared semantic-trace reason to a retrieval result set.
    pub fn append_semantic_trace(
        result_set: &mut RetrievalResultSet,
        semantic_result: &SemanticRetrievalResult,
        reason_code: &str,
        include_embedding_details: bool,
    ) {
        result_set
            .explain
            .result_reasons
            .push(Self::semantic_trace_reason(
                semantic_result,
                reason_code,
                include_embedding_details,
            ));
    }

    /// Builds one shared semantic-trace reason for transport wrappers.
    pub fn semantic_trace_reason(
        semantic_result: &SemanticRetrievalResult,
        reason_code: &str,
        include_embedding_details: bool,
    ) -> RetrievalResultReason {
        let mut details = vec![format!(
            "shared semantic executor used lexical prefilter over {} namespace candidate(s), produced {} prefilter candidate(s), returned {} semantic candidate(s), and enforced bounded result_limit={}",
            semantic_result.trace.namespace_candidate_count,
            semantic_result.trace.lexical_prefilter_count,
            semantic_result.trace.semantic_candidate_count,
            semantic_result.trace.result_limit,
        )];
        if semantic_result.trace.bounded_shortlist_truncated {
            details.push(
                "bounded shortlist truncated lower-ranked candidates before hydration to preserve the declared runtime budget"
                    .to_string(),
            );
        }
        if let Some(reason) = semantic_result.trace.degraded_reason.as_deref() {
            details.push(format!("degraded_reason={reason}"));
        }
        if include_embedding_details {
            if let Some(query_trace) = semantic_result.trace.query_trace.as_ref() {
                details.push(format!(
                    "query_embedding={} generation={} dims={}",
                    query_trace.backend_kind, query_trace.generation, query_trace.dimensions
                ));
            }
            if let Some(batch_trace) = semantic_result.trace.batch_trace.as_ref() {
                details.push(format!(
                    "batch_embedding={} generation={} count={} dims={}",
                    batch_trace.backend_kind,
                    batch_trace.generation,
                    batch_trace.batch_size,
                    batch_trace.dimensions
                ));
            }
        }
        RetrievalResultReason {
            memory_id: None,
            reason_code: reason_code.to_string(),
            detail: details.join("; "),
        }
    }

    /// Appends one shared reasoning-trace reason to a retrieval result set.
    pub fn append_reasoning_trace(
        result_set: &mut RetrievalResultSet,
        reasoning: &QueryRewriteOutcome,
        reason_code: &str,
    ) {
        result_set
            .explain
            .result_reasons
            .push(Self::reasoning_trace_reason(reasoning, reason_code));
    }

    /// Builds one shared reasoning-trace reason for transport wrappers.
    pub fn reasoning_trace_reason(
        reasoning: &QueryRewriteOutcome,
        reason_code: &str,
    ) -> RetrievalResultReason {
        let mut details = vec![reasoning.trace_note.clone()];
        if !reasoning.rewritten_queries.is_empty() {
            details.push(format!(
                "rewritten_queries={}",
                reasoning.rewritten_queries.join(" | ")
            ));
        }
        if let Some(reason) = reasoning.degraded_reason.as_deref() {
            details.push(format!("degraded_reason={reason}"));
        }
        RetrievalResultReason {
            memory_id: None,
            reason_code: reason_code.to_string(),
            detail: details.join("; "),
        }
    }

    /// Resolves a stable maintenance action name into bounded repair targets.
    pub fn resolve_maintenance_targets(action: &str) -> Result<Vec<RepairTarget>, String> {
        match action {
            "repair" | "repair_all" => Ok(vec![
                RepairTarget::LexicalIndex,
                RepairTarget::MetadataIndex,
                RepairTarget::SemanticHotIndex,
                RepairTarget::SemanticColdIndex,
                RepairTarget::GraphConsistency,
                RepairTarget::CacheWarmState,
                RepairTarget::EngramIndex,
            ]),
            "repair_index" | "repair_indexes" => Ok(vec![
                RepairTarget::LexicalIndex,
                RepairTarget::MetadataIndex,
                RepairTarget::SemanticHotIndex,
                RepairTarget::SemanticColdIndex,
            ]),
            "repair_metadata" => Ok(vec![RepairTarget::MetadataIndex]),
            "repair_graph" => Ok(vec![RepairTarget::GraphConsistency]),
            "repair_lineage" => Ok(vec![RepairTarget::EngramIndex]),
            "repair_cache" => Ok(vec![RepairTarget::CacheWarmState]),
            other => Err(format!(
                "unknown maintenance action '{other}'. Available: repair, repair_index, repair_metadata, repair_graph, repair_lineage, repair_cache"
            )),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{EncodeOperationRequest, OperationEngine, OperationMemoryRecord};
    use crate::api::NamespaceId;
    use crate::config::RuntimeConfig;
    use crate::engine::recall::{RecallEngine, RecallRequest, RecallRuntime};
    use crate::engine::result::{RetrievalExplain, RetrievalResultSet};
    use crate::graph::CausalLinkType;
    use crate::observability::OutcomeClass;
    use crate::policy::SharingVisibility;
    use crate::types::{MemoryId, RawIntakeKind, SessionId};
    use crate::BrainStore;

    #[test]
    fn prepare_encode_candidate_preserves_visibility_and_passive_summary(
    ) -> Result<(), crate::api::ContextValidationError> {
        let store = BrainStore::new(RuntimeConfig::default());
        let prepared = OperationEngine::prepare_encode_candidate(
            &store,
            EncodeOperationRequest {
                namespace: NamespaceId::new("team.alpha")?,
                memory_id: MemoryId(7),
                session_id: SessionId(3),
                content: "watch the heartbeat and retry if needed".to_string(),
                intake_kind: RawIntakeKind::Observation,
                active_era_id: Some("incident-7".to_string()),
                affect: None,
                visibility: SharingVisibility::Shared,
                workspace_id: Some("workspace-1".to_string()),
                agent_id: Some("agent-1".to_string()),
                payload_state: "metadata_only".to_string(),
            },
        );

        assert_eq!(prepared.memory.visibility, SharingVisibility::Shared);
        assert_eq!(prepared.memory.era_id.as_deref(), Some("incident-7"));
        assert_eq!(prepared.memory.agent_id.as_deref(), Some("agent-1"));
        assert_eq!(prepared.memory.payload_state, "metadata_only");
        Ok(())
    }

    #[test]
    fn causal_links_filter_namespace_and_keep_agent_identity(
    ) -> Result<(), crate::api::ContextValidationError> {
        let namespace = NamespaceId::new("team.alpha")?;
        let other_namespace = NamespaceId::new("team.beta")?;
        let links = OperationEngine::causal_links_for_records(
            &[
                OperationMemoryRecord {
                    memory_id: MemoryId(11),
                    namespace: namespace.clone(),
                    session_id: SessionId(1),
                    memory_type: crate::types::CanonicalMemoryType::Event,
                    route_family: crate::types::FastPathRouteFamily::Event,
                    compact_text: "alpha".to_string(),
                    provisional_salience: 700,
                    affect: None,
                    fingerprint: 11,
                    payload_size_bytes: 5,
                    payload_state: "metadata_only".to_string(),
                    visibility: SharingVisibility::Private,
                    is_landmark: false,
                    landmark_label: None,
                    era_id: None,
                    passive_observation: None,
                    causal_parents: vec![MemoryId(2)],
                    causal_link_type: Some(CausalLinkType::Derived),
                    agent_id: Some("agent-1".to_string()),
                },
                OperationMemoryRecord {
                    memory_id: MemoryId(12),
                    namespace: other_namespace,
                    session_id: SessionId(1),
                    memory_type: crate::types::CanonicalMemoryType::Event,
                    route_family: crate::types::FastPathRouteFamily::Event,
                    compact_text: "beta".to_string(),
                    provisional_salience: 700,
                    affect: None,
                    fingerprint: 12,
                    payload_size_bytes: 4,
                    payload_state: "metadata_only".to_string(),
                    visibility: SharingVisibility::Private,
                    is_landmark: false,
                    landmark_label: None,
                    era_id: None,
                    passive_observation: None,
                    causal_parents: vec![MemoryId(3)],
                    causal_link_type: Some(CausalLinkType::Derived),
                    agent_id: Some("agent-2".to_string()),
                },
            ],
            &namespace,
        );

        assert_eq!(links.len(), 1);
        assert_eq!(links[0].dst_memory_id, MemoryId(11));
        assert_eq!(links[0].agent_id.as_deref(), Some("agent-1"));
        Ok(())
    }

    #[test]
    fn response_from_result_set_marks_partial_for_degraded_or_truncated_results(
    ) -> Result<(), crate::api::ContextValidationError> {
        let namespace = NamespaceId::new("team.alpha")?;
        let plan = RecallEngine.plan_recall(
            RecallRequest::small_session_lookup(SessionId(1)),
            RuntimeConfig::default(),
        );
        let mut result_set = RetrievalResultSet::empty(
            RetrievalExplain::from_plan(&plan, "balanced"),
            namespace.clone(),
        );
        result_set.outcome_class = OutcomeClass::Partial;
        result_set.truncated = true;

        let response = OperationEngine::response_from_result_set(
            &namespace,
            crate::api::RequestId::new("req-1")?,
            result_set,
        );

        assert!(response.partial_success);
        assert!(response.route_summary.is_some());
        assert!(response.explain_trace.is_some());
        Ok(())
    }

    #[test]
    fn resolve_maintenance_targets_supports_aliases() -> Result<(), String> {
        let canonical = OperationEngine::resolve_maintenance_targets("repair_index")?;
        let alias = OperationEngine::resolve_maintenance_targets("repair_indexes")?;

        assert_eq!(canonical, alias);
        assert!(OperationEngine::resolve_maintenance_targets("unknown").is_err());
        Ok(())
    }
}
