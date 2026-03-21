use membrain_core::api::{
    NamespaceId, PassiveObservationInspectSummary, RequestId, ResponseContext,
};
use membrain_core::engine::encode::EncodeRuntime;
use membrain_core::engine::intent::{IntentEngine, QueryIntent};
use membrain_core::engine::maintenance::{
    MaintenanceController, MaintenanceJobHandle, MaintenanceJobState,
};
use membrain_core::engine::ranking::RankingRuntime;
use membrain_core::engine::recall::{RecallRuntime, RecallTraceStage};
use membrain_core::engine::repair::{IndexRepairEntrypoint, RepairTarget};
use membrain_core::engine::retrieval_planner::{PrimaryCue, RetrievalRequest};
use membrain_core::observability::OutcomeClass;
use membrain_core::policy::{PolicyDecision, PolicyGateway};
use membrain_core::store::hot::Tier1HotMetadataStore;
use membrain_core::store::{ColdStoreApi, HotStoreApi, Tier2StoreApi};
use membrain_core::types::{
    CanonicalMemoryType, FastPathRouteFamily, MemoryId, RawEncodeInput, RawIntakeKind, SessionId,
    Tier1HotRecord, Tier1PayloadState, WorkingMemoryId, WorkingMemoryItem,
};
use membrain_core::{BrainStore, RuntimeConfig};

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
