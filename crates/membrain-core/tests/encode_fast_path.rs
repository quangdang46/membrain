use membrain_core::engine::encode::EncodeEngine;
use membrain_core::observability::EncodeFastPathStage;
use membrain_core::types::{
    CanonicalMemoryType, FastPathRouteFamily, RawEncodeInput, RawIntakeKind,
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
        ]
    );
    assert_eq!(first.trace.duplicate_hint_candidate_count, 0);
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
