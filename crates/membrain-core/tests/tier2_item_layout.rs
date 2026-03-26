use membrain_core::api::{AgentId, NamespaceId, WorkspaceId};
use membrain_core::engine::compression::CompressionPolicy;
use membrain_core::engine::lease::{
    FreshnessState, LeaseAction, LeaseMetadata, LeasePolicy, LeaseScanItem, LeaseScanner,
};
use membrain_core::migrate::DurableSchemaObject;
use membrain_core::store::tier2::Tier2Store;
use membrain_core::store::Tier2StoreApi;
use membrain_core::types::{
    CompressionMetadata, LandmarkMetadata, MemoryId, NormalizedMemoryEnvelope, RawEncodeInput,
    RawIntakeKind, SessionId, SharingMetadata,
};
use membrain_core::{BrainStore, RuntimeConfig};

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

fn shared_envelope(raw_text: &str, compact_text: &str) -> NormalizedMemoryEnvelope {
    let mut envelope = normalized_event(raw_text, compact_text);
    envelope.sharing = SharingMetadata::new(membrain_core::policy::SharingVisibility::Shared)
        .with_workspace_id(WorkspaceId::new("ws.alpha"))
        .with_agent_id(AgentId::new("agent.writer"));
    envelope
}

fn sample_confidence_inputs() -> membrain_core::engine::confidence::ConfidenceInputs {
    membrain_core::engine::confidence::ConfidenceInputs {
        corroboration_count: 3,
        reconsolidation_count: 2,
        ticks_since_last_access: 42,
        age_ticks: 84,
        resolution_state: membrain_core::engine::contradiction::ResolutionState::None,
        conflict_score: 0,
        causal_parent_count: 2,
        authoritativeness: 720,
        recall_count: 4,
    }
}

fn sample_confidence_output() -> membrain_core::engine::confidence::ConfidenceOutput {
    membrain_core::engine::confidence::ConfidenceEngine.compute(
        &sample_confidence_inputs(),
        &membrain_core::engine::confidence::ConfidencePolicy::default(),
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
        affect: None,
        landmark,
        observation_source: observation_source.map(str::to_string),
        observation_chunk_id: observation_chunk_id.map(str::to_string),
        has_causal_parents: false,
        has_causal_children: false,
        compression: CompressionMetadata::default(),
        sharing: SharingMetadata::default(),
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
        None,
        None,
    );

    assert_eq!(layout.metadata.namespace, namespace);
    assert_eq!(layout.metadata.memory_id, MemoryId(41));
    assert_eq!(layout.metadata.session_id, SessionId(7));
    assert_eq!(layout.metadata.visibility.as_str(), "private");
    assert_eq!(layout.metadata.workspace_id, None);
    assert_eq!(layout.metadata.agent_id, None);
    assert_eq!(layout.metadata.compact_text, "compact prefilter text");
    assert_eq!(layout.metadata.lease.lease_policy, LeasePolicy::Normal);
    assert_eq!(layout.metadata.lease.freshness_state, FreshnessState::Fresh);
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
    assert_eq!(
        layout.payload_hydration_path(),
        "tier2://team.alpha/payload/0029/41"
    );
    assert_eq!(
        layout.payload.payload_locator.hydration_path(),
        layout.payload_hydration_path()
    );
    assert_eq!(layout.payload.raw_size_bytes, envelope.raw_text.len());
    assert!(layout.payload_size_matches_raw_body());
    assert_eq!(
        layout.payload.raw_text,
        "full payload with more detail than the compact prefilter text"
    );
}

#[test]
fn tier2_prefilter_view_exposes_namespace_safe_metadata_fields() {
    let store = Tier2Store;
    let namespace = NamespaceId::new("team.alpha").unwrap();
    let mut envelope = shared_envelope("raw source evidence", "compact summary");
    envelope.has_causal_parents = true;
    envelope.has_causal_children = true;
    let confidence_inputs = sample_confidence_inputs();
    let confidence_output = sample_confidence_output();

    let layout = store.layout_item(
        namespace,
        MemoryId(99),
        SessionId(13),
        12345,
        &envelope,
        Some(confidence_inputs.clone()),
        Some(confidence_output.clone()),
    );
    let prefilter = layout.prefilter_view();
    let trace = layout.prefilter_trace();

    assert_eq!(prefilter.namespace.as_str(), "team.alpha");
    assert_eq!(prefilter.memory_id, MemoryId(99));
    assert_eq!(prefilter.session_id, SessionId(13));
    assert_eq!(prefilter.compact_text, "compact summary");
    assert_eq!(prefilter.fingerprint, 12345);
    assert_eq!(prefilter.payload_size_bytes, "raw source evidence".len());
    assert_eq!(prefilter.visibility().as_str(), "shared");
    assert_eq!(prefilter.lease_policy(), LeasePolicy::Normal);
    assert_eq!(prefilter.freshness_state(), FreshnessState::Fresh);
    assert!(prefilter.has_causal_parents);
    assert!(prefilter.has_causal_children);
    assert_eq!(prefilter.workspace_id(), Some("ws.alpha"));
    assert_eq!(prefilter.agent_id(), Some("agent.writer"));
    assert_eq!(prefilter.confidence_inputs, Some(&confidence_inputs));
    assert_eq!(prefilter.confidence_output, Some(&confidence_output));
    assert_eq!(prefilter.payload_locator, layout.metadata.payload_locator);
    assert!(layout.prefilter_stays_metadata_only());
    assert_eq!(trace.metadata_candidate_count, 1);
    assert_eq!(trace.payload_fetch_count, 0);
    assert!(layout.payload_size_matches_raw_body());
}

#[test]
fn tier2_metadata_index_key_matches_namespace_safe_identity_fields() {
    let store = Tier2Store;
    let namespace = NamespaceId::new("team.alpha").unwrap();
    let mut envelope = shared_envelope("raw source evidence", "compact summary");
    envelope.has_causal_parents = true;
    let confidence_inputs = sample_confidence_inputs();
    let confidence_output = sample_confidence_output();

    let layout = store.layout_item(
        namespace,
        MemoryId(99),
        SessionId(13),
        12345,
        &envelope,
        Some(confidence_inputs.clone()),
        Some(confidence_output.clone()),
    );
    let key = layout.metadata_index_key();

    assert_eq!(key.namespace.as_str(), layout.metadata.namespace.as_str());
    assert_eq!(key.memory_id, layout.metadata.memory_id);
    assert_eq!(key.session_id, layout.metadata.session_id);
    assert_eq!(key.memory_type, layout.metadata.memory_type);
    assert_eq!(key.route_family, layout.metadata.route_family);
    assert_eq!(key.fingerprint, layout.metadata.fingerprint);
    assert_eq!(key.compact_text, layout.metadata.compact_text);
    assert_eq!(key.visibility().as_str(), "shared");
    assert_eq!(key.lease_policy(), LeasePolicy::Normal);
    assert_eq!(key.freshness_state(), FreshnessState::Fresh);
    assert!(key.has_causal_parents);
    assert!(!key.has_causal_children);
    assert_eq!(key.workspace_id(), Some("ws.alpha"));
    assert_eq!(key.agent_id(), Some("agent.writer"));
    assert_eq!(key.confidence_inputs, Some(&confidence_inputs));
    assert_eq!(key.confidence_output, Some(&confidence_output));
    assert_eq!(
        key.normalization_generation,
        layout.metadata.normalization_generation
    );
    assert_eq!(key.payload_locator, layout.metadata.payload_locator);
    assert_eq!(
        key.payload_locator.hydration_path(),
        "tier2://team.alpha/payload/0063/99"
    );
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
        None,
        None,
    );
    let beta = store.layout_item(
        NamespaceId::new("team.beta").unwrap(),
        MemoryId(99),
        SessionId(13),
        12345,
        &envelope,
        None,
        None,
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
fn tier2_payload_hydration_path_escapes_namespace_separators() {
    let store = Tier2Store;
    let namespace = NamespaceId::new("team/alpha").unwrap();
    let envelope = normalized_event("raw source evidence", "compact summary");

    let layout = store.layout_item(
        namespace,
        MemoryId(41),
        SessionId(7),
        991,
        &envelope,
        None,
        None,
    );

    assert_eq!(
        layout.payload.payload_locator.namespace.as_str(),
        "team/alpha"
    );
    assert_eq!(
        layout.payload_hydration_path(),
        "tier2://team%2Falpha/payload/0029/41"
    );
    assert_eq!(
        layout.payload.payload_locator.hydration_path(),
        layout.payload_hydration_path()
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
        None,
        None,
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
fn lease_scanner_reports_stale_action_critical_transitions_without_full_scan() {
    let scanner = LeaseScanner;
    let items = vec![
        LeaseScanItem {
            memory_id: MemoryId(11),
            lease: LeaseMetadata::new(LeasePolicy::Normal, 0),
            action_critical: false,
        },
        LeaseScanItem {
            memory_id: MemoryId(12),
            lease: LeaseMetadata::new(LeasePolicy::Volatile, 0),
            action_critical: true,
        },
        LeaseScanItem {
            memory_id: MemoryId(13),
            lease: LeaseMetadata::pinned(0),
            action_critical: true,
        },
    ];

    let report = scanner.scan(&items, 400, 2);

    assert_eq!(report.scanned_items, 2);
    assert_eq!(report.transitioned_items, 2);
    assert_eq!(report.recheck_required_items, 1);
    assert_eq!(report.withheld_items, 1);
    assert_eq!(report.queue_report.queue_family, "lease_scanner");
    assert_eq!(report.queue_report.queue_depth_before, 3);
    assert_eq!(report.queue_report.queue_depth_after, 1);
    assert!(report.queue_report.partial_run);
    assert_eq!(report.transitions[0].next_state, FreshnessState::Stale);
    assert_eq!(report.transitions[0].action, LeaseAction::LowerConfidence);
    assert_eq!(
        report.transitions[1].next_state,
        FreshnessState::RecheckRequired
    );
    assert_eq!(report.transitions[1].action, LeaseAction::Withhold);
    assert_eq!(report.transitions[1].confidence_cap, Some(250));
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
            era_started_at_tick: Some(88),
            detection_score: 903,
            detection_reason: Some(
                "arousal=0.91 novelty=0.83 recent_similarity=0.31 gap_ticks=88 crossed landmark thresholds"
                    .to_string(),
            ),
        },
    );

    let layout = store.layout_item(
        NamespaceId::new("team.alpha").unwrap(),
        MemoryId(77),
        SessionId(4),
        123,
        &envelope,
        None,
        None,
    );

    let prefilter = layout.prefilter_view();
    let index_key = layout.metadata_index_key();
    let landmark_record = layout.landmark_record();

    assert!(layout.metadata.landmark.is_landmark);
    assert_eq!(layout.metadata.landmark, envelope.landmark);
    assert_eq!(prefilter.landmark, &envelope.landmark);
    assert_eq!(index_key.landmark, &envelope.landmark);
    assert_eq!(landmark_record.namespace.as_str(), "team.alpha");
    assert_eq!(landmark_record.memory_id, MemoryId(77));
    assert!(landmark_record.is_landmark);
    assert_eq!(
        landmark_record.landmark_label.as_deref(),
        Some("project launch deadline was moved")
    );
    assert_eq!(
        landmark_record.landmark_label(),
        Some("project launch deadline was moved")
    );
    assert_eq!(
        landmark_record.era_id.as_deref(),
        Some("era-projectlaunc-0088")
    );
    assert_eq!(landmark_record.era_id(), Some("era-projectlaunc-0088"));
    assert_eq!(landmark_record.era_started_at_tick, Some(88));
    assert_eq!(landmark_record.era_started_at_tick(), Some(88));
    assert_eq!(landmark_record.detection_score, 903);
    assert_eq!(landmark_record.landmark_detection_score(), 903);
    assert!(landmark_record
        .detection_reason
        .as_deref()
        .is_some_and(|reason| reason.contains("crossed landmark thresholds")));
    assert!(landmark_record
        .landmark_detection_reason()
        .is_some_and(|reason| reason.contains("crossed landmark thresholds")));
    assert_eq!(
        prefilter.landmark_label(),
        Some("project launch deadline was moved")
    );
    assert_eq!(prefilter.era_id(), Some("era-projectlaunc-0088"));
    assert_eq!(prefilter.era_started_at_tick(), Some(88));
    assert_eq!(prefilter.landmark_detection_score(), 903);
    assert!(prefilter
        .landmark_detection_reason()
        .is_some_and(|reason| reason.contains("crossed landmark thresholds")));
    assert_eq!(
        index_key.landmark_label(),
        Some("project launch deadline was moved")
    );
    assert_eq!(index_key.era_id(), Some("era-projectlaunc-0088"));
    assert_eq!(index_key.era_started_at_tick(), Some(88));
    assert_eq!(index_key.landmark_detection_score(), 903);
    assert!(index_key
        .landmark_detection_reason()
        .is_some_and(|reason| reason.contains("crossed landmark thresholds")));
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
        None,
        None,
    );
    let prefilter = layout.prefilter_view();
    let index_key = layout.metadata_index_key();
    let landmark_record = layout.landmark_record();

    assert_eq!(layout.metadata.landmark, LandmarkMetadata::non_landmark());
    assert_eq!(prefilter.landmark, &LandmarkMetadata::non_landmark());
    assert_eq!(index_key.landmark, &LandmarkMetadata::non_landmark());
    assert_eq!(landmark_record.namespace.as_str(), "team.alpha");
    assert_eq!(landmark_record.memory_id, MemoryId(78));
    assert!(!landmark_record.is_landmark);
    assert_eq!(landmark_record.landmark_label, None);
    assert_eq!(landmark_record.landmark_label(), None);
    assert_eq!(landmark_record.era_id, None);
    assert_eq!(landmark_record.era_id(), None);
    assert_eq!(landmark_record.era_started_at_tick, None);
    assert_eq!(landmark_record.era_started_at_tick(), None);
    assert_eq!(landmark_record.detection_score, 0);
    assert_eq!(landmark_record.landmark_detection_score(), 0);
    assert_eq!(landmark_record.detection_reason, None);
    assert_eq!(landmark_record.landmark_detection_reason(), None);
    assert_eq!(prefilter.landmark_label(), None);
    assert_eq!(prefilter.era_id(), None);
    assert_eq!(index_key.landmark_label(), None);
    assert_eq!(index_key.era_id(), None);
}

#[test]
fn tier2_metadata_preserves_active_era_for_non_landmarks() {
    let store = Tier2Store;
    let envelope = normalized_event_with_landmark(
        "routine follow-up note",
        "routine follow-up note",
        LandmarkMetadata {
            era_id: Some("era-launch-0088".to_string()),
            ..LandmarkMetadata::non_landmark()
        },
    );

    let layout = store.layout_item(
        NamespaceId::new("team.alpha").unwrap(),
        MemoryId(79),
        SessionId(6),
        125,
        &envelope,
        None,
        None,
    );
    let prefilter = layout.prefilter_view();
    let index_key = layout.metadata_index_key();
    let landmark_record = layout.landmark_record();

    assert!(!layout.metadata.landmark.is_landmark);
    assert_eq!(
        layout.metadata.landmark.era_id.as_deref(),
        Some("era-launch-0088")
    );
    assert_eq!(landmark_record.era_id(), Some("era-launch-0088"));
    assert_eq!(landmark_record.era_started_at_tick(), None);
    assert_eq!(prefilter.era_id(), Some("era-launch-0088"));
    assert_eq!(index_key.era_id(), Some("era-launch-0088"));
    assert_eq!(landmark_record.landmark_detection_score(), 0);
    assert_eq!(landmark_record.landmark_detection_reason(), None);
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
        None,
        None,
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
fn tier2_observation_layout_keeps_provenance_in_metadata_and_raw_body_detached() {
    let store = Tier2Store;
    let raw_text = "file watcher noticed a new artifact";
    let compact_text = "new artifact observed";
    let envelope = normalized_observation(
        raw_text,
        compact_text,
        "passive_observation",
        "obs-0000000000000042",
    );

    let layout = store.layout_item(
        NamespaceId::new("team.alpha").unwrap(),
        MemoryId(81),
        SessionId(9),
        127,
        &envelope,
        None,
        None,
    );
    let payload = layout.payload_record();

    assert_eq!(layout.metadata.payload_size_bytes, raw_text.len());
    assert_eq!(layout.metadata.payload_size_bytes, payload.raw_size_bytes);
    assert_eq!(layout.metadata.payload_locator, payload.payload_locator);
    assert_eq!(
        layout.metadata.observation_source.as_deref(),
        Some("passive_observation")
    );
    assert_eq!(
        layout.metadata.observation_chunk_id.as_deref(),
        Some("obs-0000000000000042")
    );
    assert_eq!(payload.raw_text, raw_text);
    assert_ne!(payload.raw_text, compact_text);
    assert!(layout.prefilter_stays_metadata_only());
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
            era_started_at_tick: Some(1),
            detection_score: 812,
            detection_reason: Some("manual landmark fixture".to_string()),
        },
    );

    let layout = store.layout_item(
        NamespaceId::new("team.alpha").unwrap(),
        MemoryId(80),
        SessionId(8),
        126,
        &envelope,
        None,
        None,
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
    assert_eq!(prefilter.landmark_label(), Some("launch milestone"));
    assert_eq!(prefilter.era_id(), Some("era-launch-milestone-0001"));
    assert_eq!(prefilter.era_started_at_tick(), Some(1));
    assert_eq!(prefilter.landmark_detection_score(), 812);
    assert_eq!(
        prefilter.landmark_detection_reason(),
        Some("manual landmark fixture")
    );
    assert_eq!(prefilter.payload_locator, layout.metadata.payload_locator);

    let key = layout.metadata_index_key();
    assert_eq!(key.landmark, &envelope.landmark);
    assert_eq!(key.landmark_label(), Some("launch milestone"));
    assert_eq!(key.era_id(), Some("era-launch-milestone-0001"));
    assert_eq!(key.era_started_at_tick(), Some(1));
    assert_eq!(key.landmark_detection_score(), 812);
    assert_eq!(
        key.landmark_detection_reason(),
        Some("manual landmark fixture")
    );
}

#[test]
fn compression_lineage_surfaces_through_public_brain_store_and_tier2_metadata_views() {
    let mut store = BrainStore::new(RuntimeConfig::default());
    let namespace = NamespaceId::new("tests/compression-public").unwrap();
    let applied = store.apply_compression_pass(
        namespace.clone(),
        CompressionPolicy {
            min_episode_count: 3,
            ..CompressionPolicy::default()
        },
        3,
        false,
    );

    let artifact = applied.schema_artifact.expect("schema artifact");
    let schema_layout = store
        .compression_memory_layout(artifact.schema_memory_id)
        .expect("schema layout persisted");
    let schema_prefilter = schema_layout.prefilter_view();
    let schema_key = schema_layout.metadata_index_key();

    assert_eq!(schema_prefilter.compressed_into(), None);
    assert_eq!(
        schema_prefilter.compression_source_memory_ids(),
        artifact.source_memory_ids.as_slice()
    );
    assert_eq!(schema_key.compressed_into(), None);
    assert_eq!(
        schema_key.compression_source_memory_ids(),
        artifact.source_memory_ids.as_slice()
    );
    assert_eq!(
        schema_prefilter.compression_tick(),
        schema_key.compression_tick()
    );
    assert!(schema_prefilter.compression_tick().is_some());

    let source_memory_id = artifact.source_memory_ids[0];
    let source_layout = store
        .compression_memory_layout(source_memory_id)
        .expect("source layout persisted");
    let source_prefilter = source_layout.prefilter_view();
    let source_key = source_layout.metadata_index_key();

    assert_eq!(
        source_prefilter.compressed_into(),
        Some(artifact.schema_memory_id)
    );
    assert_eq!(
        source_key.compressed_into(),
        Some(artifact.schema_memory_id)
    );
    assert!(source_prefilter.compression_source_memory_ids().is_empty());
    assert!(source_key.compression_source_memory_ids().is_empty());
    assert_eq!(
        source_prefilter.compression_tick(),
        source_key.compression_tick()
    );
    assert!(source_prefilter.compression_tick().is_some());
}

#[test]
fn compression_log_entries_filter_by_namespace_and_since_tick() {
    let mut store = BrainStore::new(RuntimeConfig::default());
    let alpha = NamespaceId::new("tests/compression-alpha").unwrap();
    let beta = NamespaceId::new("tests/compression-beta").unwrap();

    let alpha_applied = store.apply_compression_pass(
        alpha.clone(),
        CompressionPolicy {
            min_episode_count: 3,
            ..CompressionPolicy::default()
        },
        3,
        false,
    );
    let beta_applied = store.apply_compression_pass(
        beta.clone(),
        CompressionPolicy {
            min_episode_count: 3,
            ..CompressionPolicy::default()
        },
        3,
        false,
    );

    let alpha_entry = alpha_applied
        .compression_log_entry
        .expect("alpha compression log entry");
    let beta_entry = beta_applied
        .compression_log_entry
        .expect("beta compression log entry");

    let alpha_entries = store.compression_log_entries(alpha.clone(), None);
    assert_eq!(alpha_entries, vec![alpha_entry.clone()]);

    let beta_entries = store.compression_log_entries(beta.clone(), None);
    assert_eq!(beta_entries, vec![beta_entry.clone()]);

    assert!(store
        .compression_log_entries(alpha, Some(alpha_entry.tick.saturating_add(1)))
        .is_empty());
    assert_eq!(
        store.compression_log_entries(beta, Some(beta_entry.tick)),
        vec![beta_entry]
    );
}

#[test]
fn tier2_store_schema_objects_match_split_durable_layout_contract() {
    let store = Tier2Store;

    assert_eq!(
        store.authoritative_schema_objects(),
        vec![
            DurableSchemaObject::MemoryItemsTable,
            DurableSchemaObject::MemoryPayloadsTable,
            DurableSchemaObject::MemoryLineageEdgesTable,
            DurableSchemaObject::CausalLinksTable,
            DurableSchemaObject::MemoryEntityRefsTable,
            DurableSchemaObject::MemoryRelationRefsTable,
            DurableSchemaObject::MemoryTagsTable,
            DurableSchemaObject::ConflictRecordsTable,
            DurableSchemaObject::DurableMemoryRecords,
            DurableSchemaObject::SnapshotMetadataTable,
            DurableSchemaObject::CompressionLogTable,
            DurableSchemaObject::LandmarksTable,
        ]
    );
}
