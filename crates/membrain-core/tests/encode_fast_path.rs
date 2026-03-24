use membrain_core::api::NamespaceId;
use membrain_core::brain_store::BrainStore;
use membrain_core::engine::contradiction::{ContradictionKind, ContradictionStore};
use membrain_core::engine::encode::{EncodeEngine, EncodeRuntime, EncodeWriteBranch};
use membrain_core::observability::EncodeFastPathStage;
use membrain_core::policy::{IngestMode, PassiveObservationDecision};
use membrain_core::types::{
    CanonicalMemoryType, FastPathRouteFamily, LandmarkMetadata, LandmarkSignals, MemoryId,
    RawEncodeInput, RawIntakeKind,
};
use membrain_core::RuntimeConfig;

fn test_engine() -> EncodeEngine {
    EncodeEngine::new(RuntimeConfig::default())
}

#[test]
fn normalization_and_fingerprint_are_stable_for_equivalent_whitespace() {
    let engine = test_engine();

    let first = engine.prepare_fast_path(RawEncodeInput::new(
        RawIntakeKind::Event,
        "  hello   world  ",
    ));
    let second = engine.prepare_fast_path(RawEncodeInput::new(RawIntakeKind::Event, "hello world"));

    assert_eq!(first.normalized.compact_text, "hello world");
    assert_eq!(
        first.normalized.compact_text,
        second.normalized.compact_text
    );
    assert_eq!(first.fingerprint, second.fingerprint);
    assert_eq!(
        first.trace.stages,
        [
            EncodeFastPathStage::Normalize,
            EncodeFastPathStage::Fingerprint,
            EncodeFastPathStage::ShallowClassify,
            EncodeFastPathStage::ProvisionalSalience,
            EncodeFastPathStage::LandmarkTagging,
        ]
    );
    assert_eq!(first.trace.duplicate_hint_candidate_count, 0);
    assert_eq!(first.normalized.landmark, LandmarkMetadata::non_landmark());
    assert_eq!(first.trace.landmark, LandmarkMetadata::non_landmark());
    assert_eq!(first.trace.landmark_signals, None);
    assert_eq!(first.write_decision, PassiveObservationDecision::Capture);
    assert!(!first.captured_as_observation);
    assert!(first.trace.stayed_within_latency_budget);
}

#[test]
fn shallow_classification_tracks_the_canonical_memory_family() {
    let engine = test_engine();

    let observation = engine.prepare_fast_path(RawEncodeInput::new(
        RawIntakeKind::Observation,
        "file watcher noticed a new artifact",
    ));
    let tool_outcome = engine.prepare_fast_path(RawEncodeInput::new(
        RawIntakeKind::ToolOutcome,
        "cargo test completed successfully",
    ));

    assert_eq!(
        observation.normalized.memory_type,
        CanonicalMemoryType::Observation
    );
    assert_eq!(
        observation.classification.route_family,
        FastPathRouteFamily::Observation
    );
    assert_eq!(
        tool_outcome.normalized.memory_type,
        CanonicalMemoryType::ToolOutcome
    );
    assert_eq!(
        tool_outcome.classification.route_family,
        FastPathRouteFamily::ToolOutcome
    );
}

#[test]
fn provisional_salience_stays_bounded_and_inspectable() {
    let engine = test_engine();

    let event = engine.prepare_fast_path(RawEncodeInput::new(RawIntakeKind::Event, "ping"));
    let preference = engine.prepare_fast_path(RawEncodeInput::new(
        RawIntakeKind::UserPreference,
        "always use rustls for local-only network features",
    ));

    assert!(event.provisional_salience <= 1_000);
    assert!(preference.provisional_salience <= 1_000);
    assert!(preference.provisional_salience > event.provisional_salience);
    assert_eq!(
        preference.trace.provisional_salience,
        preference.provisional_salience
    );
    assert_eq!(
        preference.trace.route_family,
        FastPathRouteFamily::UserPreference
    );
}

#[test]
fn passive_observation_capture_stays_explicit_when_policy_allows() {
    let engine = test_engine();

    let observed = engine.prepare_ingest_candidate(
        RawEncodeInput::new(RawIntakeKind::Observation, "fresh passive signal"),
        IngestMode::PassiveObservation,
        true,
        false,
    );

    assert_eq!(observed.write_decision, PassiveObservationDecision::Capture);
    assert!(observed.captured_as_observation);
    assert_eq!(
        observed.passive_observation_inspect.source_kind,
        RawIntakeKind::Observation.as_str()
    );
    assert_eq!(
        observed.passive_observation_inspect.write_decision,
        "capture"
    );
    assert_eq!(
        observed
            .passive_observation_inspect
            .observation_source
            .as_deref(),
        Some("passive_observation")
    );
    assert!(observed
        .passive_observation_inspect
        .observation_chunk_id
        .as_deref()
        .is_some_and(|chunk| chunk.starts_with("obs-")));
    assert_eq!(
        observed.passive_observation_inspect.retention_marker,
        "volatile_observation"
    );
}

#[test]
fn passive_observation_denial_blocks_capture_when_namespace_is_not_bound() {
    let engine = test_engine();

    let denied = engine.prepare_ingest_candidate(
        RawEncodeInput::new(RawIntakeKind::Observation, "denied passive observation"),
        IngestMode::PassiveObservation,
        false,
        false,
    );

    assert_eq!(denied.write_decision, PassiveObservationDecision::Deny);
    assert!(!denied.captured_as_observation);
    assert_eq!(denied.passive_observation_inspect.write_decision, "deny");
    assert_eq!(
        denied.passive_observation_inspect.retention_marker,
        "volatile_observation"
    );
}

#[test]
fn passive_observation_duplicate_hints_are_suppressed() {
    let engine = test_engine();

    let suppressed = engine.prepare_ingest_candidate(
        RawEncodeInput::new(RawIntakeKind::Observation, "passive duplicate hint"),
        IngestMode::PassiveObservation,
        true,
        true,
    );

    assert_eq!(
        suppressed.write_decision,
        PassiveObservationDecision::Suppress
    );
    assert_eq!(suppressed.trace.duplicate_hint_candidate_count, 1);
    assert!(!suppressed.captured_as_observation);
    assert_eq!(
        suppressed.passive_observation_inspect.write_decision,
        "suppress"
    );
    assert_eq!(
        suppressed.passive_observation_inspect.retention_marker,
        "volatile_observation"
    );
}

#[test]
fn encode_runtime_trait_delegates_to_the_inherent_fast_path() {
    let engine = test_engine();
    let runtime: &dyn EncodeRuntime = &engine;

    let prepared = runtime.prepare_fast_path(RawEncodeInput::new(
        RawIntakeKind::Event,
        "trait dispatched encode",
    ));

    assert_eq!(prepared.normalized.compact_text, "trait dispatched encode");
    assert_eq!(prepared.write_decision, PassiveObservationDecision::Capture);
    assert!(!prepared.captured_as_observation);
}

#[test]
fn qualified_landmark_signals_stay_visible_in_fast_path_trace_for_temporal_consumers() {
    let engine = test_engine();
    let signals = LandmarkSignals::new(0.93, 0.86, 0.22, 144);

    let prepared = engine.prepare_fast_path(
        RawEncodeInput::new(
            RawIntakeKind::Event,
            "quarter closed after launch milestone and contract renewal",
        )
        .with_landmark_signals(signals),
    );

    assert!(prepared.normalized.landmark.is_landmark);
    assert_eq!(prepared.trace.landmark_signals, Some(signals));
    assert_eq!(prepared.trace.landmark, prepared.normalized.landmark);
    assert_eq!(
        prepared.normalized.landmark.landmark_label.as_deref(),
        Some("quarter closed after launch milestone and contract renewal")
    );
    assert_eq!(
        prepared.normalized.landmark.era_id.as_deref(),
        Some("era-quarterclose-0144")
    );
    assert_eq!(prepared.normalized.landmark, prepared.trace.landmark);
    assert_eq!(prepared.normalized.landmark.era_started_at_tick, Some(144));
    assert!(prepared.normalized.landmark.detection_score >= 800);
    assert!(prepared
        .normalized
        .landmark
        .detection_reason
        .as_deref()
        .is_some_and(|reason| {
            reason.contains("crossed landmark thresholds")
                && reason.contains("arousal>=0.70")
                && reason.contains("novelty>=0.75")
                && reason.contains("recent_similarity<=0.85")
                && reason.contains("gap_ticks>=50")
        }));
    assert_eq!(
        prepared.classification.route_family,
        FastPathRouteFamily::Event
    );
    assert_eq!(prepared.write_decision, PassiveObservationDecision::Capture);
    assert!(!prepared.captured_as_observation);
    assert!(prepared.trace.stayed_within_latency_budget);
}

#[test]
fn landmark_era_slug_falls_back_when_text_has_no_ascii_alphanumerics() {
    let engine = test_engine();
    let prepared = engine.prepare_fast_path(
        RawEncodeInput::new(RawIntakeKind::Event, "   !!! ###   ")
            .with_landmark_signals(LandmarkSignals::new(0.95, 0.92, 0.10, 88)),
    );

    assert!(prepared.normalized.landmark.is_landmark);
    assert_eq!(
        prepared.normalized.landmark.landmark_label.as_deref(),
        Some("!!! ###")
    );
    assert_eq!(
        prepared.normalized.landmark.era_id.as_deref(),
        Some("era-landmark-0088")
    );
    assert_eq!(prepared.trace.landmark, prepared.normalized.landmark);
    assert_eq!(prepared.normalized.landmark.era_started_at_tick, Some(88));
    assert!(prepared.normalized.landmark.detection_score >= 850);
    assert!(prepared
        .normalized
        .landmark
        .detection_reason
        .as_deref()
        .is_some_and(|reason| {
            reason.contains("crossed landmark thresholds")
                && reason.contains("arousal>=0.70")
                && reason.contains("novelty>=0.75")
                && reason.contains("recent_similarity<=0.85")
                && reason.contains("gap_ticks>=50")
        }));
}

#[test]
fn contradiction_branching_records_an_explicit_artifact_instead_of_overwrite() {
    let mut store = BrainStore::default();
    let namespace = NamespaceId::new("tests/contradictions").unwrap();

    let outcome = store
        .record_encode_contradiction(
            namespace.clone(),
            MemoryId(11),
            MemoryId(22),
            ContradictionKind::Revision,
            640,
        )
        .unwrap();

    assert_eq!(outcome.branch, EncodeWriteBranch::ContradictionRecorded);
    assert_eq!(outcome.existing_memory, MemoryId(11));
    assert_eq!(outcome.incoming_memory, MemoryId(22));
    assert_eq!(outcome.kind, ContradictionKind::Revision);

    let explains = store
        .contradiction_engine()
        .explain_for_memory(MemoryId(11));
    assert_eq!(explains.len(), 1);
    assert_eq!(explains[0].contradiction_id.0, outcome.contradiction_id.0);
    assert_eq!(explains[0].conflicting_memory, MemoryId(22));
}

#[test]
fn contradiction_branching_rejects_duplicate_pair_records() {
    let mut store = BrainStore::default();
    let namespace = NamespaceId::new("tests/contradictions").unwrap();

    store
        .record_encode_contradiction(
            namespace.clone(),
            MemoryId(7),
            MemoryId(8),
            ContradictionKind::Duplicate,
            900,
        )
        .unwrap();

    let duplicate = store.record_encode_contradiction(
        namespace,
        MemoryId(8),
        MemoryId(7),
        ContradictionKind::Revision,
        750,
    );

    assert!(duplicate.is_err());
    assert_eq!(
        store
            .contradiction_engine()
            .find_by_memory(MemoryId(7))
            .len(),
        1
    );
}
