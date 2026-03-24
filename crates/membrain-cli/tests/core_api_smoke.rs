use membrain_core::api::{
    AvailabilityPosture, AvailabilityReason, AvailabilitySummary, ErrorKind, NamespaceId,
    PassiveObservationInspectSummary, PolicyContext, RemediationStep, RequestContext, RequestId,
    ResponseContext,
};
use membrain_core::engine::encode::EncodeRuntime;
use membrain_core::engine::intent::{IntentEngine, QueryIntent};
use membrain_core::engine::maintenance::{
    MaintenanceController, MaintenanceJobHandle, MaintenanceJobState,
};
use membrain_core::engine::ranking::RankingRuntime;
use membrain_core::engine::recall::{RecallRuntime, RecallTraceStage};
use membrain_core::engine::repair::{IndexRepairEntrypoint, RepairTarget};
use membrain_core::engine::result::RetrievalExplain;
use membrain_core::engine::retrieval_planner::{
    PrimaryCue, RetrievalPlanTrace, RetrievalRequest, RetrievalRequestValidationError,
};
use membrain_core::observability::OutcomeClass;
use membrain_core::policy::{
    PolicyDecision, PolicyGateway, PolicyModule, SharingAccessDecision, SharingVisibility,
};
use membrain_core::store::hot::Tier1HotMetadataStore;
use membrain_core::store::{ColdStoreApi, HotStoreApi, Tier2StoreApi};
use membrain_core::types::{
    CanonicalMemoryType, FastPathRouteFamily, MemoryId, RawEncodeInput, RawIntakeKind, SessionId,
    Tier1HotRecord, Tier1PayloadState, WorkingMemoryId, WorkingMemoryItem,
};
use membrain_core::{BrainStore, RuntimeConfig};

use membrain_daemon::mcp::{McpError, McpResponse};
use membrain_daemon::rpc::{JsonRpcResponse, RuntimeMethodRequest};
use serde_json::json;

#[test]
fn cli_depends_on_core_api() {
    let _: membrain_core::CoreApiVersion = membrain_cli::core_api_version();
}

fn requires_policy<E: PolicyGateway>(_surface: &E) {}
fn requires_encode<E: EncodeRuntime>(_surface: &E) {}
fn requires_intent(_surface: &IntentEngine) {}
fn requires_recall<E: RecallRuntime>(_surface: &E) {}
fn requires_ranking<E: RankingRuntime>(_surface: &E) {}
fn requires_hot_store<E: HotStoreApi>(_surface: &E) {}
fn requires_tier2_store<E: Tier2StoreApi>(_surface: &E) {}
fn requires_cold_store<E: ColdStoreApi>(_surface: &E) {}

#[test]
fn cli_surfaces_same_error_taxonomy_as_daemon_wrappers() {
    let rpc_response =
        JsonRpcResponse::error(Some(json!("req-1")), -32602, "missing namespace", None);
    assert_eq!(rpc_response.error.as_ref().unwrap().code, -32602);
    assert_eq!(
        rpc_response.error.as_ref().unwrap().message,
        "missing namespace"
    );

    let parsed = RuntimeMethodRequest {
        jsonrpc: "1.0".to_string(),
        method: "status".to_string(),
        params: json!({}),
        id: Some(json!("req-2")),
    }
    .parse_method()
    .unwrap_err();
    assert_eq!(parsed.code, -32600);
    assert_eq!(parsed.message, "unsupported jsonrpc version");
    assert_eq!(parsed.data, Some(json!({"expected":"2.0","actual":"1.0"})));

    let mcp_response = McpResponse::failure(McpError {
        code: "policy_denied".to_string(),
        message: "namespace isolation prevents export".to_string(),
        is_policy_denial: true,
    });
    assert_eq!(mcp_response.status, "error");
    assert!(mcp_response.retrieval.is_none());
    assert!(mcp_response.payload.is_none());
    let mcp_error = mcp_response.error.as_ref().unwrap();
    assert_eq!(mcp_error.code, "policy_denied");
    assert_eq!(mcp_error.message, "namespace isolation prevents export");
    assert!(mcp_error.is_policy_denial);
}

#[test]
fn cli_depends_on_shared_core_boundaries() {
    let store = BrainStore::new(RuntimeConfig::default());

    requires_policy(store.policy());
    requires_encode(store.encode_engine());
    requires_intent(store.intent_engine());
    requires_recall(store.recall_engine());
    requires_ranking(store.ranking_engine());
    requires_hot_store(store.hot_store());
    requires_tier2_store(store.tier2_store());
    requires_cold_store(store.cold_store());

    let summary = store.policy().evaluate_namespace(true);
    assert_eq!(summary.decision, PolicyDecision::Allow);
    assert!(summary.namespace_bound);
    assert_eq!(summary.outcome_class, OutcomeClass::Accepted);
    assert_eq!(
        store.encode_engine().tier1_candidate_budget(store.config()),
        store.config().tier1_candidate_budget,
    );
    assert_eq!(
        store.recall_engine().tier1_candidate_budget(store.config()),
        store.config().tier1_candidate_budget,
    );
    assert!(store.ranking_engine().packages_explainable_results());
    assert_eq!(store.observability().component_name(), "observability");
    assert_eq!(store.hot_store().component_name(), "store.hot");
    assert_eq!(store.tier2_store().component_name(), "store.tier2");
    assert_eq!(store.cold_store().component_name(), "store.cold");
    assert_eq!(store.graph().component_name(), "graph");
    assert_eq!(store.index().component_name(), "index");
    assert_eq!(store.embed().component_name(), "embed");
    assert_eq!(store.migrate().component_name(), "migrate");
    assert_eq!(store.repair_engine().component_name(), "engine.repair");
}

#[test]
fn cli_can_drive_working_memory_admission_through_core_encode_surface(
) -> Result<(), membrain_core::engine::encode::WorkingMemoryError> {
    let mut store = BrainStore::new(RuntimeConfig::default());
    let incoming = WorkingMemoryItem::new(WorkingMemoryId(99), 400);

    let admission = store
        .encode_engine_mut()
        .working_memory_mut()
        .admit(incoming.clone())?;

    assert_eq!(admission.item, incoming);
    assert_eq!(admission.trace.slot_pressure, 0); // pre-decision slot count (empty before first admit)
    assert_eq!(store.encode_engine().working_memory().slots(), &[incoming]);
    Ok(())
}

#[test]
fn cli_can_prepare_the_synchronous_encode_fast_path_through_core() {
    let store = BrainStore::new(RuntimeConfig::default());

    let prepared = store.encode_engine().prepare_fast_path(RawEncodeInput::new(
        RawIntakeKind::Event,
        "  hello   world  ",
    ));

    assert_eq!(prepared.normalized.memory_type, CanonicalMemoryType::Event);
    assert_eq!(prepared.normalized.compact_text, "hello world");
    assert_eq!(
        prepared.classification.route_family,
        FastPathRouteFamily::Event
    );
    assert_eq!(prepared.trace.duplicate_hint_candidate_count, 0);
    assert!(prepared.trace.stayed_within_latency_budget);
}

#[test]
fn cli_can_exercise_recall_route_explain_contract_through_core() {
    let store = BrainStore::new(RuntimeConfig::default());

    let exact_plan = store.recall_engine().plan_recall(
        membrain_core::engine::recall::RecallRequest::exact(MemoryId(42)),
        store.config(),
    );
    let small_lookup_plan = store.recall_engine().plan_recall(
        membrain_core::engine::recall::RecallRequest::small_session_lookup(SessionId(7)),
        store.config(),
    );

    assert!(exact_plan.terminates_in_tier1());
    assert!(exact_plan.route_summary.tier1_answers_directly);
    assert!(exact_plan.route_summary.tier1_consulted_first);
    assert!(!exact_plan.route_summary.routes_to_deeper_tiers);
    assert_eq!(
        exact_plan.route_summary.reason,
        "exact memory id selects the direct Tier1 handle lane"
    );
    assert_eq!(
        exact_plan.route_summary.trace_stages,
        &[RecallTraceStage::Tier1ExactHandle]
    );

    assert!(!small_lookup_plan.terminates_in_tier1());
    assert!(!small_lookup_plan.route_summary.tier1_answers_directly);
    assert!(small_lookup_plan.route_summary.tier1_consulted_first);
    assert!(small_lookup_plan.route_summary.routes_to_deeper_tiers);
    assert_eq!(
        small_lookup_plan.route_summary.reason,
        "small session lookup scans the Tier1 recent window before Tier2 exact"
    );
    assert_eq!(
        small_lookup_plan.route_summary.trace_stages,
        &[
            RecallTraceStage::Tier1RecentWindow,
            RecallTraceStage::Tier2Exact,
        ]
    );
}

#[test]
fn cli_can_exercise_intent_taxonomy_and_classification_logs_through_core() {
    let store = BrainStore::new(RuntimeConfig::default());

    let procedural = store
        .intent_engine()
        .classify("how to deploy the service after the last incident?");
    let fallback = store.intent_engine().classify("rust lifetime notes");
    let log = procedural.log_record();

    assert_eq!(procedural.intent, QueryIntent::ProceduralLookup);
    assert_eq!(procedural.route_inputs.query_path.as_str(), "entity_heavy");
    assert_eq!(procedural.route_inputs.ranking_profile.as_str(), "balanced");
    assert!(procedural.route_inputs.prefer_small_lookup);
    assert!(
        procedural
            .route_inputs
            .prefer_preview_only_on_low_confidence
    );
    assert!(procedural.route_inputs.high_stakes);
    assert_eq!(log.intent, "procedural_lookup");
    assert_eq!(log.query_path, "entity_heavy");
    assert!(log.matched_patterns.contains(&"how to"));
    assert_eq!(fallback.intent, QueryIntent::SemanticBroad);
    assert!(fallback.low_confidence_fallback);
    assert_eq!(
        fallback.log_record().matched_patterns,
        vec!["default_semantic_broad"]
    );
}

#[test]
fn cli_can_drive_tier1_hot_metadata_store_session_windows() {
    let store = BrainStore::new(RuntimeConfig::default());
    let namespace = NamespaceId::new("cli.team").unwrap();
    let mut hot: Tier1HotMetadataStore = store.hot_store().new_metadata_store(3);

    hot.seed(Tier1HotRecord::metadata_only(
        namespace.clone(),
        MemoryId(1),
        SessionId(7),
        CanonicalMemoryType::Event,
        FastPathRouteFamily::Event,
        "older",
        11,
        300,
        16_384,
    ));
    hot.seed(Tier1HotRecord::metadata_only(
        namespace.clone(),
        MemoryId(2),
        SessionId(7),
        CanonicalMemoryType::Event,
        FastPathRouteFamily::Event,
        "newer",
        22,
        450,
        16_384,
    ));

    let recent = hot.recent_for_session(&namespace, SessionId(7), 2);
    let exact = hot.exact_lookup(&namespace, MemoryId(2));

    assert_eq!(recent.records.len(), 2);
    assert_eq!(recent.records[0].memory_id, MemoryId(2));
    assert_eq!(
        recent.records[0].payload_state,
        Tier1PayloadState::MetadataOnly
    );
    assert_eq!(recent.trace.payload_fetch_count, 0);
    assert!(recent.trace.session_window_hit);
    assert_eq!(exact.trace.payload_fetch_count, 0);
}

#[test]
fn cli_zero_limit_recent_windows_stay_empty() {
    let store = BrainStore::new(RuntimeConfig::default());
    let namespace = NamespaceId::new("cli.team").unwrap();
    let mut hot: Tier1HotMetadataStore = store.hot_store().new_metadata_store(3);

    hot.seed(Tier1HotRecord::metadata_only(
        namespace.clone(),
        MemoryId(1),
        SessionId(7),
        CanonicalMemoryType::Event,
        FastPathRouteFamily::Event,
        "only record",
        11,
        300,
        16_384,
    ));

    let recent = hot.recent_for_session(&namespace, SessionId(7), 0);

    assert!(recent.records.is_empty());
    assert_eq!(recent.trace.payload_fetch_count, 0);
    assert_eq!(recent.trace.recent_candidates_inspected, 0);
    assert!(!recent.trace.session_window_hit);
}

#[test]
fn cli_can_surface_passive_observation_inspect_provenance_and_retention() {
    let store = BrainStore::new(RuntimeConfig::default());
    let observed = store.encode_engine().prepare_ingest_candidate(
        RawEncodeInput::new(RawIntakeKind::Observation, "watcher noticed a file change"),
        membrain_core::policy::IngestMode::PassiveObservation,
        true,
        false,
    );

    let response = ResponseContext::success(
        NamespaceId::new("cli.team").unwrap(),
        RequestId::new("req-passive-observation").unwrap(),
        1u8,
    )
    .with_passive_observation(PassiveObservationInspectSummary::from_encode(
        &observed.passive_observation_inspect,
    ));

    let passive = response.passive_observation.as_ref().unwrap();
    assert_eq!(passive.source_kind, "observation");
    assert_eq!(passive.write_decision, "capture");
    assert!(passive.captured_as_observation);
    assert_eq!(passive.observation_source.state_name(), "present");
    assert_eq!(passive.observation_chunk_id.state_name(), "present");
    assert_eq!(passive.retention_marker.state_name(), "present");
}

#[test]
fn cli_can_observe_query_by_example_normalization_and_seed_order() {
    let namespace = NamespaceId::new("cli.team").unwrap();
    let normalized = RetrievalRequest::hybrid(namespace, "  example cue  ", 4)
        .with_like_memory(MemoryId(21))
        .with_unlike_memory(MemoryId(22))
        .normalize_query_by_example()
        .unwrap();

    assert_eq!(normalized.primary_cue, PrimaryCue::QueryText);
    assert!(normalized.uses_query_text_as_primary_cue());
    assert!(normalized.has_example_seeds());
    assert_eq!(normalized.seed_descriptors(), vec!["like:21", "unlike:22"]);
    assert_eq!(normalized.seed_polarities(), vec!["like", "unlike"]);
    assert_eq!(
        normalized.seed_memory_ids(),
        vec![MemoryId(21), MemoryId(22)]
    );
    assert_eq!(
        normalized.normalized_query_text.as_deref(),
        Some("example cue")
    );
}

#[test]
fn cli_treats_whitespace_only_query_text_as_absent_for_query_by_example() {
    let namespace = NamespaceId::new("cli.team").unwrap();
    let request = RetrievalRequest::hybrid(namespace, "   ", 4).with_like_memory(MemoryId(21));
    let normalized = request.normalize_query_by_example().unwrap();

    assert_eq!(normalized.normalized_query_text, None);
    assert_eq!(normalized.primary_cue, PrimaryCue::LikeId);
    assert_eq!(normalized.seed_descriptors(), vec!["like:21"]);
    assert!(!request.requires_lexical_prefilter());
    assert!(request.requires_semantic_search());
}

#[test]
fn cli_rejects_exact_id_plus_query_by_example_mix() {
    let namespace = NamespaceId::new("cli.team").unwrap();
    let request =
        RetrievalRequest::exact_id(namespace, MemoryId(44)).with_unlike_memory(MemoryId(21));
    let error = request.normalize_query_by_example().unwrap_err();

    assert_eq!(
        error,
        RetrievalRequestValidationError::ExactIdWithExampleCue
    );
    assert_eq!(error.as_str(), "exact_id_with_example_cue");
}

#[test]
fn cli_rejects_duplicate_query_by_example_cues() {
    let namespace = NamespaceId::new("cli.team").unwrap();
    let request = RetrievalRequest::hybrid(namespace, "release blocker", 4)
        .with_like_memory(MemoryId(21))
        .with_unlike_memory(MemoryId(21));
    let error = request.normalize_query_by_example().unwrap_err();

    assert_eq!(
        error,
        RetrievalRequestValidationError::DuplicateExampleCue(MemoryId(21))
    );
    assert_eq!(error.as_str(), "duplicate_example_cue");
}

#[test]
fn cli_query_by_example_explain_names_source_mode_and_materialization_gaps() {
    let namespace = NamespaceId::new("cli.team").unwrap();
    let request = RetrievalRequest::hybrid(namespace, "  release blocker  ", 4)
        .with_like_memory(MemoryId(21))
        .with_unlike_memory(MemoryId(34));
    let normalization = request.normalize_query_by_example().unwrap();
    let mut trace = RetrievalPlanTrace::new(&request);
    trace.set_query_by_example_materialization(&normalization, &[MemoryId(21)]);
    trace.set_final_candidates(3);

    let mut explain = RetrievalExplain::from_plan(
        &membrain_core::engine::recall::RecallEngine.plan_recall(
            membrain_core::engine::recall::RecallRequest::small_session_lookup(SessionId(9)),
            RuntimeConfig::default(),
        ),
        "balanced",
    );
    explain.set_query_by_example_trace(&trace);

    let query_by_example = explain.query_by_example.expect("query-by-example trace");
    assert_eq!(query_by_example.primary_cue, "query_text");
    assert_eq!(
        query_by_example.requested_seed_descriptors,
        vec!["like:21", "unlike:34"]
    );
    assert_eq!(
        query_by_example.materialized_seed_descriptors,
        vec!["like:21"]
    );
    assert_eq!(query_by_example.missing_seed_descriptors, vec!["unlike:34"]);
    assert_eq!(query_by_example.expanded_candidate_count, 3);
    assert_eq!(
        query_by_example.influence_summary,
        "primary cue query_text expanded 3 candidate(s) from 2 requested seed(s); 1 seed(s) materialized and 1 seed(s) remained unavailable"
    );
    assert!(explain.result_reasons.iter().any(|reason| {
        reason.reason_code == "query_by_example_seed_materialized"
            && reason.detail == "seed like:21 materialized from stored evidence"
    }));
    assert!(explain.result_reasons.iter().any(|reason| {
        reason.reason_code == "query_by_example_seed_missing"
            && reason.detail == "seed unlike:34 was requested but not available for expansion"
    }));
    assert!(explain.result_reasons.iter().any(|reason| {
        reason.reason_code == "query_by_example_candidate_expansion"
            && reason.detail.contains("primary cue query_text expanded 3 candidate(s)")
    }));
}

#[test]
fn cli_keeps_non_observation_inspect_fields_explicitly_absent() {
    let store = BrainStore::new(RuntimeConfig::default());
    let active = store.encode_engine().prepare_fast_path(RawEncodeInput::new(
        RawIntakeKind::Event,
        "ordinary active intake",
    ));

    let response = ResponseContext::success(
        NamespaceId::new("cli.team").unwrap(),
        RequestId::new("req-active-ingest").unwrap(),
        1u8,
    )
    .with_passive_observation(PassiveObservationInspectSummary::from_encode(
        &active.passive_observation_inspect,
    ));

    let passive = response.passive_observation.as_ref().unwrap();
    assert_eq!(passive.source_kind, "event");
    assert_eq!(passive.write_decision, "capture");
    assert!(!passive.captured_as_observation);
    assert_eq!(passive.observation_source.state_name(), "absent");
    assert_eq!(passive.observation_chunk_id.state_name(), "absent");
    assert_eq!(passive.retention_marker.state_name(), "absent");
}

#[test]
fn cli_can_surface_repair_reports_through_shared_core_repair_engine() {
    let store = BrainStore::new(RuntimeConfig::default());
    let namespace = NamespaceId::new("cli.team").unwrap();
    let run = store.repair_engine().create_targeted(
        namespace,
        vec![RepairTarget::LexicalIndex, RepairTarget::MetadataIndex],
        IndexRepairEntrypoint::RebuildIfNeeded,
    );
    let mut handle = MaintenanceJobHandle::new(run, 8);

    handle.start();
    let mut completed_summary = None;
    for _ in 0..8 {
        let snapshot = handle.poll();
        match snapshot.state {
            MaintenanceJobState::Completed(summary) => {
                completed_summary = Some(summary);
                break;
            }
            MaintenanceJobState::Running { .. } => continue,
            _ => break,
        }
    }

    let summary = completed_summary.expect("repair run should complete within bounded polls");
    assert_eq!(summary.targets_checked, 2);
    assert_eq!(summary.rebuilt, 2);
    assert!(summary
        .results
        .iter()
        .all(|result| result.verification_passed));
    assert!(summary.results.iter().all(|result| {
        result.rebuild_entrypoint == Some(IndexRepairEntrypoint::RebuildIfNeeded)
    }));
    assert_eq!(summary.results[0].target, RepairTarget::LexicalIndex);
    assert_eq!(summary.results[1].target, RepairTarget::MetadataIndex);
    assert_eq!(
        summary.results[0].rebuilt_outputs,
        vec!["fts5_lexical_projection", "lexical_lookup_table"]
    );
    assert_eq!(
        summary.results[1].rebuilt_outputs,
        vec!["tier2_metadata_projection", "namespace_lookup_table"]
    );
}

// ── CLI parity and denial end-to-end scripts ─────────────────────────────────
//
// These tests verify that CLI-facing encode, recall, inspect, explain, and
// admin surfaces produce deterministic outputs that match the canonical
// retrieval, policy, and error contracts without relying on ambient time
// or flaky wall-clock assumptions.

#[test]
fn cli_parity_encode_recall_inspect_pipeline() {
    let store = BrainStore::new(RuntimeConfig::default());
    let namespace = NamespaceId::new("parity.test").expect("valid namespace");
    let request_id = RequestId::new("req-pipeline-1").expect("valid request id");

    // Step 1: Encode — run the fast path and verify deterministic output
    let prepared = store.encode_engine().prepare_fast_path(RawEncodeInput::new(
        RawIntakeKind::Event,
        "  parity test  encode  recall  inspect  ",
    ));
    assert_eq!(prepared.normalized.memory_type, CanonicalMemoryType::Event);
    assert_eq!(
        prepared.normalized.compact_text,
        "parity test encode recall inspect"
    );
    assert_eq!(
        prepared.classification.route_family,
        FastPathRouteFamily::Event
    );
    assert!(prepared.trace.stayed_within_latency_budget);

    // Step 2: Build a successful response envelope and verify parity fields
    let response = ResponseContext::success(namespace.clone(), request_id.clone(), 1u8);
    assert!(response.ok);
    assert_eq!(response.request_id.as_str(), "req-pipeline-1");
    assert_eq!(response.namespace.as_str(), "parity.test");
    assert_eq!(response.outcome_class, OutcomeClass::Accepted);
    assert!(response.error_kind.is_none());
    assert!(!response.retryable);
    assert!(!response.partial_success);
    assert!(response.remediation.is_none());
    assert!(response.availability.is_none());
    assert!(response.warnings.is_empty());

    // Step 3: Recall route planning — verify the plan is deterministic
    let plan = store.recall_engine().plan_recall(
        membrain_core::engine::recall::RecallRequest::exact(MemoryId(1)),
        store.config(),
    );
    assert!(plan.terminates_in_tier1());
    assert_eq!(
        plan.route_summary.trace_stages,
        &[RecallTraceStage::Tier1ExactHandle]
    );

    // Step 4: Inspect parity — the passive observation fields must be explicitly
    // absent for non-observation intake, not silently missing
    let active = store.encode_engine().prepare_fast_path(RawEncodeInput::new(
        RawIntakeKind::Event,
        "active intake for parity",
    ));
    let inspect_response = ResponseContext::success(
        namespace.clone(),
        RequestId::new("req-inspect-parity").unwrap(),
        1u8,
    )
    .with_passive_observation(PassiveObservationInspectSummary::from_encode(
        &active.passive_observation_inspect,
    ));
    let passive = inspect_response.passive_observation.as_ref().unwrap();
    assert!(!passive.captured_as_observation);
    assert_eq!(passive.observation_source.state_name(), "absent");
    assert_eq!(passive.observation_chunk_id.state_name(), "absent");
}

#[test]
fn cli_parity_denial_scenarios() {
    let namespace = NamespaceId::new("parity.denial").unwrap();

    // Denial 1: Missing namespace fails at binding time
    let request_no_ns = RequestContext {
        namespace: None,
        workspace_id: None,
        agent_id: None,
        session_id: None,
        task_id: None,
        request_id: RequestId::new("req-denial-1").unwrap(),
        policy_context: PolicyContext {
            include_public: false,
            sharing_visibility: SharingVisibility::Private,
            caller_identity_bound: true,
            workspace_acl_allowed: true,
            agent_acl_allowed: true,
            session_visibility_allowed: true,
            legal_hold: false,
        },
        time_budget_ms: None,
    };
    let bind_result = request_no_ns.bind_namespace(None);
    assert!(bind_result.is_err());
    let err = bind_result.unwrap_err();
    assert_eq!(err.error_kind(), ErrorKind::ValidationFailure);

    // Denial 2: Unbound caller identity is denied by policy
    let request_unbound = RequestContext {
        namespace: Some(namespace.clone()),
        workspace_id: None,
        agent_id: None,
        session_id: None,
        task_id: None,
        request_id: RequestId::new("req-denial-2").unwrap(),
        policy_context: PolicyContext {
            include_public: false,
            sharing_visibility: SharingVisibility::Private,
            caller_identity_bound: false,
            workspace_acl_allowed: true,
            agent_acl_allowed: true,
            session_visibility_allowed: true,
            legal_hold: false,
        },
        time_budget_ms: None,
    };
    let bound = request_unbound.bind_namespace(None).unwrap();
    let policy = bound.evaluate_policy(&PolicyModule);
    assert_eq!(policy.decision, PolicyDecision::Deny);
    assert_eq!(policy.outcome_class, OutcomeClass::Rejected);

    // Denial 3: Failed response envelope preserves error taxonomy
    let failure = ResponseContext::<()>::failure(
        namespace.clone(),
        RequestId::new("req-denial-3").unwrap(),
        ErrorKind::PolicyDenied,
        vec![],
    );
    assert!(!failure.ok);
    assert_eq!(failure.outcome_class, OutcomeClass::Rejected);
    assert_eq!(failure.error_kind, Some(ErrorKind::PolicyDenied));
    assert!(!failure.retryable);
    let remediation = failure.remediation.as_ref().unwrap();
    assert!(remediation.step_names().contains(&"change_scope"));
}

#[test]
fn cli_parity_cross_namespace_sharing_denial_and_redaction() {
    let request = RequestContext {
        namespace: Some(NamespaceId::new("parity.ns1").unwrap()),
        workspace_id: None,
        agent_id: None,
        session_id: None,
        task_id: None,
        request_id: RequestId::new("req-cross-ns").unwrap(),
        policy_context: PolicyContext {
            include_public: false,
            sharing_visibility: SharingVisibility::Private,
            caller_identity_bound: true,
            workspace_acl_allowed: true,
            agent_acl_allowed: true,
            session_visibility_allowed: true,
            legal_hold: false,
        },
        time_budget_ms: None,
    };
    let bound = request.bind_namespace(None).unwrap();
    let outcome = bound.evaluate_cross_namespace_sharing_access(
        &PolicyModule,
        &NamespaceId::new("parity.ns2").unwrap(),
    );
    assert_eq!(outcome.decision, SharingAccessDecision::Deny);
    assert!(outcome
        .denial_reasons
        .iter()
        .any(|r| r.as_str() == "namespace_isolation"));
    assert!(outcome.redaction_fields.contains(&"memory_id"));
    assert!(outcome.redaction_fields.contains(&"session_id"));
}

#[test]
fn cli_parity_partial_success_and_degraded_outcomes() {
    let namespace = NamespaceId::new("parity.partial").unwrap();
    let request_id = RequestId::new("req-partial").unwrap();

    // Partial success outcome
    let partial = ResponseContext::success(namespace.clone(), request_id.clone(), 42u8)
        .with_partial_success();
    assert!(partial.ok);
    assert!(partial.partial_success);
    assert_eq!(partial.outcome_class, OutcomeClass::Partial);

    // Degraded availability outcome
    let degraded = ResponseContext::success(namespace.clone(), request_id.clone(), 42u8)
        .with_availability(AvailabilitySummary::degraded(
            vec!["recall"],
            vec!["encode"],
            vec![AvailabilityReason::RepairInFlight],
            vec![RemediationStep::CheckHealth],
        ));
    assert!(degraded.ok);
    assert_eq!(degraded.outcome_class, OutcomeClass::Degraded);
    let avail = degraded.availability.as_ref().unwrap();
    assert_eq!(avail.posture, AvailabilityPosture::Degraded);
    assert!(avail
        .degraded_reasons
        .contains(&AvailabilityReason::RepairInFlight));
}

#[test]
fn cli_parity_repair_engine_produces_deterministic_results() {
    let store = BrainStore::new(RuntimeConfig::default());
    let namespace = NamespaceId::new("parity.repair").unwrap();

    // Run repair twice and verify identical results
    for _ in 0..2 {
        let run = store.repair_engine().create_targeted(
            namespace.clone(),
            vec![RepairTarget::LexicalIndex, RepairTarget::MetadataIndex],
            IndexRepairEntrypoint::RebuildIfNeeded,
        );
        let mut handle = MaintenanceJobHandle::new(run, 8);
        handle.start();
        let mut completed = false;
        for _ in 0..8 {
            let snapshot = handle.poll();
            if let MaintenanceJobState::Completed(summary) = snapshot.state {
                assert_eq!(summary.targets_checked, 2);
                assert_eq!(summary.rebuilt, 2);
                assert!(summary.results.iter().all(|r| r.verification_passed));
                assert_eq!(summary.results[0].target, RepairTarget::LexicalIndex);
                assert_eq!(summary.results[1].target, RepairTarget::MetadataIndex);
                completed = true;
                break;
            }
        }
        assert!(completed, "repair should complete within bounded polls");
    }
}
