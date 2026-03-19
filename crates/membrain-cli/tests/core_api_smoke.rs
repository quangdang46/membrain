use membrain_core::engine::encode::EncodeRuntime;
use membrain_core::engine::ranking::RankingRuntime;
use membrain_core::engine::recall::RecallRuntime;
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
    assert_eq!(admission.trace.slot_pressure, 1);
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
fn cli_can_drive_tier1_hot_metadata_store_session_windows() {
    let store = BrainStore::new(RuntimeConfig::default());
    let mut hot: Tier1HotMetadataStore = store.hot_store().new_metadata_store(3);

    hot.seed(Tier1HotRecord::metadata_only(
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
        MemoryId(2),
        SessionId(7),
        CanonicalMemoryType::Event,
        FastPathRouteFamily::Event,
        "newer",
        22,
        450,
        16_384,
    ));

    let recent = hot.recent_for_session(SessionId(7), 2);
    let exact = hot.exact_lookup(MemoryId(2));

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
