use membrain_core::api::NamespaceId;
use membrain_core::store::tier2::Tier2Store;
use membrain_core::types::{
    LandmarkMetadata, MemoryId, NormalizedMemoryEnvelope, RawEncodeInput, RawIntakeKind, SessionId,
};

fn normalized_event(raw_text: &str, compact_text: &str) -> NormalizedMemoryEnvelope {
    normalized_event_with_landmark(raw_text, compact_text, LandmarkMetadata::non_landmark())
}

fn normalized_event_with_landmark(
    raw_text: &str,
    compact_text: &str,
    landmark: LandmarkMetadata,
) -> NormalizedMemoryEnvelope {
    normalized_envelope(
        RawIntakeKind::Event,
        raw_text,
        compact_text,
        landmark,
        None,
        None,
    )
}

fn normalized_observation(
    raw_text: &str,
    compact_text: &str,
    observation_source: &str,
    observation_chunk_id: &str,
) -> NormalizedMemoryEnvelope {
    normalized_envelope(
        RawIntakeKind::Observation,
        raw_text,
        compact_text,
        LandmarkMetadata::non_landmark(),
        Some(observation_source),
        Some(observation_chunk_id),
    )
}

fn normalized_envelope(
    kind: RawIntakeKind,
    raw_text: &str,
    compact_text: &str,
    landmark: LandmarkMetadata,
    observation_source: Option<&str>,
    observation_chunk_id: Option<&str>,
) -> NormalizedMemoryEnvelope {
    let input = RawEncodeInput::new(kind, raw_text);
    NormalizedMemoryEnvelope {
        memory_type: input.kind.canonical_memory_type(),
        source_kind: input.kind,
        raw_text: input.raw_text,
        compact_text: compact_text.to_string(),
        normalization_generation: "norm-v1",
        payload_size_bytes: raw_text.len(),
        landmark,
        observation_source: observation_source.map(str::to_string),
        observation_chunk_id: observation_chunk_id.map(str::to_string),
    }
}

#[test]
fn tier2_layout_separates_metadata_from_raw_payload_body_and_preserves_namespace() {
    let store = Tier2Store;
    let namespace = NamespaceId::new("team.alpha").unwrap();
    let envelope = normalized_event(
        "full payload with more detail than the compact prefilter text",
        "compact prefilter text",
    );

    let layout = store.layout_item(
        namespace.clone(),
        MemoryId(41),
        SessionId(7),
        991,
        &envelope,
    );

    assert_eq!(layout.metadata.namespace, namespace);
    assert_eq!(layout.metadata.memory_id, MemoryId(41));
    assert_eq!(layout.metadata.session_id, SessionId(7));
    assert_eq!(layout.metadata.compact_text, "compact prefilter text");
    assert_eq!(
        layout.metadata.payload_size_bytes,
        envelope.payload_size_bytes
    );
    assert_eq!(layout.payload.namespace.as_str(), "team.alpha");
    assert_eq!(layout.payload.memory_id, MemoryId(41));
    assert_eq!(
        layout.metadata.payload_locator,
        layout.payload.payload_locator
    );
    assert_eq!(
        layout.payload.payload_locator.namespace.as_str(),
        "team.alpha"
    );
    assert_eq!(layout.payload.raw_size_bytes, envelope.raw_text.len());
    assert_eq!(
        layout.payload.raw_text,
        "full payload with more detail than the compact prefilter text"
    );
}

#[test]
fn tier2_prefilter_view_exposes_namespace_safe_metadata_fields() {
    let store = Tier2Store;
    let namespace = NamespaceId::new("team.alpha").unwrap();
    let envelope = normalized_event("raw source evidence", "compact summary");

    let layout = store.layout_item(namespace, MemoryId(99), SessionId(13), 12345, &envelope);
    let prefilter = layout.prefilter_view();
    let trace = layout.prefilter_trace();

    assert_eq!(prefilter.namespace.as_str(), "team.alpha");
    assert_eq!(prefilter.memory_id, MemoryId(99));
    assert_eq!(prefilter.session_id, SessionId(13));
    assert_eq!(prefilter.compact_text, "compact summary");
    assert_eq!(prefilter.fingerprint, 12345);
    assert_eq!(prefilter.payload_size_bytes, "raw source evidence".len());
    assert_eq!(prefilter.payload_locator, layout.metadata.payload_locator);
    assert!(layout.prefilter_stays_metadata_only());
    assert_eq!(trace.metadata_candidate_count, 1);
    assert_eq!(trace.payload_fetch_count, 0);
}

#[test]
fn tier2_metadata_index_key_matches_namespace_safe_identity_fields() {
    let store = Tier2Store;
    let namespace = NamespaceId::new("team.alpha").unwrap();
    let envelope = normalized_event("raw source evidence", "compact summary");

    let layout = store.layout_item(namespace, MemoryId(99), SessionId(13), 12345, &envelope);
    let key = layout.metadata_index_key();

    assert_eq!(key.namespace.as_str(), layout.metadata.namespace.as_str());
    assert_eq!(key.memory_id, layout.metadata.memory_id);
    assert_eq!(key.session_id, layout.metadata.session_id);
    assert_eq!(key.memory_type, layout.metadata.memory_type);
    assert_eq!(key.route_family, layout.metadata.route_family);
    assert_eq!(key.fingerprint, layout.metadata.fingerprint);
    assert_eq!(key.compact_text, layout.metadata.compact_text);
    assert_eq!(
        key.normalization_generation,
        layout.metadata.normalization_generation
    );
    assert_eq!(key.payload_locator, layout.metadata.payload_locator);
    assert!(layout.index_key_stays_metadata_only());
}

#[test]
fn tier2_payload_locator_changes_with_namespace_for_same_memory_id() {
    let store = Tier2Store;
    let envelope = normalized_event("raw source evidence", "compact summary");

    let alpha = store.layout_item(
        NamespaceId::new("team.alpha").unwrap(),
        MemoryId(99),
        SessionId(13),
        12345,
        &envelope,
    );
    let beta = store.layout_item(
        NamespaceId::new("team.beta").unwrap(),
        MemoryId(99),
        SessionId(13),
        12345,
        &envelope,
    );

    assert_ne!(
        alpha.payload.payload_locator.namespace,
        beta.payload.payload_locator.namespace
    );
    assert_eq!(
        alpha.payload.payload_locator.slot,
        beta.payload.payload_locator.slot
    );
    assert_eq!(
        alpha.payload.payload_locator.shard,
        beta.payload.payload_locator.shard
    );
}

#[test]
fn tier2_payload_body_stays_outside_prefilter_and_index_views() {
    let store = Tier2Store;
    let envelope = normalized_event(
        "full payload with detail that should stay detached",
        "compact summary",
    );

    let layout = store.layout_item(
        NamespaceId::new("team.alpha").unwrap(),
        MemoryId(17),
        SessionId(2),
        88,
        &envelope,
    );
    let prefilter = layout.prefilter_view();
    let key = layout.metadata_index_key();
    let payload = layout.payload_record();

    assert_eq!(prefilter.compact_text, "compact summary");
    assert_eq!(key.compact_text, "compact summary");
    assert_eq!(prefilter.payload_locator, payload.payload_locator);
    assert_eq!(key.payload_locator, payload.payload_locator);
    assert_ne!(payload.raw_text, prefilter.compact_text);
    assert_ne!(payload.raw_text, key.compact_text);
    assert_eq!(layout.prefilter_trace().payload_fetch_count, 0);
}

#[test]
fn tier2_metadata_preserves_landmark_and_era_fields_for_durable_recall() {
    let store = Tier2Store;
    let envelope = normalized_event_with_landmark(
        "project launch deadline was moved",
        "project launch deadline was moved",
        LandmarkMetadata {
            is_landmark: true,
            landmark_label: Some("project launch deadline was moved".to_string()),
            era_id: Some("era-projectlaunc-0088".to_string()),
        },
    );

    let layout = store.layout_item(
        NamespaceId::new("team.alpha").unwrap(),
        MemoryId(77),
        SessionId(4),
        123,
        &envelope,
    );

    assert!(layout.metadata.landmark.is_landmark);
    assert_eq!(layout.metadata.landmark, envelope.landmark);
    assert_eq!(layout.prefilter_view().landmark, &envelope.landmark);
    assert_eq!(layout.metadata_index_key().landmark, &envelope.landmark);
}

#[test]
fn tier2_non_landmarks_remain_explicitly_non_landmarks_in_metadata_views() {
    let store = Tier2Store;
    let envelope = normalized_event("routine standup note", "routine standup note");

    let layout = store.layout_item(
        NamespaceId::new("team.alpha").unwrap(),
        MemoryId(78),
        SessionId(5),
        124,
        &envelope,
    );

    assert_eq!(layout.metadata.landmark, LandmarkMetadata::non_landmark());
    assert_eq!(
        layout.prefilter_view().landmark,
        &LandmarkMetadata::non_landmark()
    );
    assert_eq!(
        layout.metadata_index_key().landmark,
        &LandmarkMetadata::non_landmark()
    );
}

#[test]
fn tier2_metadata_preserves_passive_observation_provenance_without_payload_fetches() {
    let store = Tier2Store;
    let envelope = normalized_observation(
        "file watcher noticed a new artifact",
        "new artifact observed",
        "passive_observation",
        "obs-0000000000000042",
    );

    let layout = store.layout_item(
        NamespaceId::new("team.alpha").unwrap(),
        MemoryId(79),
        SessionId(6),
        125,
        &envelope,
    );
    let prefilter = layout.prefilter_view();

    assert_eq!(
        layout.metadata.observation_source.as_deref(),
        Some("passive_observation")
    );
    assert_eq!(
        layout.metadata.observation_chunk_id.as_deref(),
        Some("obs-0000000000000042")
    );
    assert_eq!(prefilter.observation_source, Some("passive_observation"));
    assert_eq!(prefilter.observation_chunk_id, Some("obs-0000000000000042"));
    assert_eq!(layout.prefilter_trace().payload_fetch_count, 0);
}

#[test]
fn tier2_landmark_prefilter_trace_stays_metadata_first_for_temporal_recall_consumers() {
    let store = Tier2Store;
    let envelope = normalized_event_with_landmark(
        "quarter closed after launch milestone",
        "quarter closed after launch milestone",
        LandmarkMetadata {
            is_landmark: true,
            landmark_label: Some("launch milestone".to_string()),
            era_id: Some("era-launch-milestone-0001".to_string()),
        },
    );

    let layout = store.layout_item(
        NamespaceId::new("team.alpha").unwrap(),
        MemoryId(80),
        SessionId(8),
        126,
        &envelope,
    );
    let prefilter = layout.prefilter_view();
    let trace = layout.prefilter_trace();

    assert_eq!(
        trace.outcome,
        membrain_core::observability::Tier2PrefilterOutcome::Ready
    );
    assert_eq!(trace.metadata_candidate_count, 1);
    assert_eq!(trace.payload_fetch_count, 0);
    assert!(layout.prefilter_stays_metadata_only());
    assert!(prefilter.landmark.is_landmark);
    assert_eq!(
        prefilter.landmark.landmark_label.as_deref(),
        Some("launch milestone")
    );
    assert_eq!(
        prefilter.landmark.era_id.as_deref(),
        Some("era-launch-milestone-0001")
    );
    assert_eq!(prefilter.payload_locator, layout.metadata.payload_locator);
    assert_eq!(layout.metadata_index_key().landmark, &envelope.landmark);
}
