use membrain_core::api::{FieldPresence, NamespaceId, PolicyContext, RequestContext, RequestId};
use membrain_core::embed::{CachedTextEmbedder, EmbedError, EmbeddingPurpose, LocalTextEmbedder};
use membrain_core::engine::contradiction::{
    ContradictionCandidate, ContradictionExplain, ContradictionKind, PreferredAnswerState,
    ResolutionState,
};
use membrain_core::engine::ranking::{fuse_scores, RankingInput, RankingProfile};
use membrain_core::engine::recall::{
    RecallPlanKind, RecallRequest, RecallRuntime, RecallTraceStage,
};
use membrain_core::engine::result::{
    AnsweredFrom, EvidenceRole, PayloadState, ResultBuilder, ResultReason, RetrievalExplain,
    RetrievalResultSet,
};
use membrain_core::engine::retrieval_planner::{PrimaryCue, RetrievalPlanTrace, RetrievalRequest};
use membrain_core::engine::semantic_retrieval::{
    HydratedMemoryRecord, SemanticExecutorConfig, SharedSemanticRetrievalExecutor,
};
use membrain_core::graph::{
    BoundedExpansionPlanner, CutoffReason, EntityId, EntityKind, ExpansionConstraints, GraphEdge,
    GraphEntity, RelationKind,
};
use membrain_core::health::{
    AttentionNamespaceInputs, BrainHealthInputs, BrainHealthReport, FeatureAvailabilityEntry,
};
use membrain_core::index::{IndexApi, IndexModule};
use membrain_core::observability::{
    CacheEvalTrace, CacheEventLabel, CacheFamilyLabel, CacheLookupOutcome, CacheReasonLabel,
    GenerationStatusLabel, WarmSourceLabel,
};
use membrain_core::store::cache::CacheManager;
use membrain_core::types::{
    CanonicalMemoryType, MemoryId, RawEncodeInput, RawIntakeKind, SessionId,
};
use membrain_core::BrainStore;

fn ns(s: &str) -> NamespaceId {
    NamespaceId::new(s).unwrap()
}

#[derive(Debug, Clone)]
struct SemanticFixtureEmbedder;

impl SemanticFixtureEmbedder {
    fn vector_for(&self, text: &str) -> Vec<f32> {
        let normalized = text.to_ascii_lowercase();
        let rollout = if normalized.contains("deploy")
            || normalized.contains("rollout")
            || normalized.contains("pipeline")
            || normalized.contains("production")
            || normalized.contains("remediation")
        {
            2.0
        } else {
            0.0
        };
        let rollback = if normalized.contains("rollback") || normalized.contains("checklist") {
            1.0
        } else {
            0.0
        };
        let checkout = if normalized.contains("checkout") || normalized.contains("cart") {
            1.0
        } else {
            0.0
        };
        let general_release = if normalized.contains("release") || normalized.contains("outage") {
            1.0
        } else {
            0.0
        };
        vec![rollout, rollback, checkout, general_release]
    }
}

impl LocalTextEmbedder for SemanticFixtureEmbedder {
    fn backend_kind(&self) -> &'static str {
        "semantic_fixture"
    }

    fn generation(&self) -> &str {
        "fixture-v1"
    }

    fn dimensions(&self) -> usize {
        4
    }

    fn embed_text(
        &mut self,
        _purpose: EmbeddingPurpose,
        normalized_text: &str,
    ) -> Result<Vec<f32>, EmbedError> {
        Ok(self.vector_for(normalized_text))
    }

    fn embed_texts(
        &mut self,
        _purpose: EmbeddingPurpose,
        normalized_texts: &[String],
    ) -> Result<Vec<Vec<f32>>, EmbedError> {
        Ok(normalized_texts
            .iter()
            .map(|text| self.vector_for(text))
            .collect())
    }
}

fn semantic_record(memory_id: u64, namespace: &NamespaceId, text: &str) -> HydratedMemoryRecord {
    HydratedMemoryRecord {
        memory_id: MemoryId(memory_id),
        namespace: namespace.clone(),
        session_id: SessionId(1),
        memory_type: CanonicalMemoryType::Event,
        route_family: membrain_core::types::FastPathRouteFamily::Observation,
        compact_text: text.to_string(),
        raw_text: text.to_string(),
        affect: None,
    }
}

#[test]
fn encode_prepares_fast_path_for_recall_pipeline() {
    let store = BrainStore::new(membrain_core::RuntimeConfig::default());

    let prepared = store.encode_engine().prepare_fast_path(RawEncodeInput::new(
        RawIntakeKind::Event,
        "test memory for recall pipeline",
    ));

    assert_eq!(
        prepared.normalized.compact_text,
        "test memory for recall pipeline"
    );
    assert!(prepared.trace.stayed_within_latency_budget);
    assert_eq!(
        prepared.write_decision,
        membrain_core::policy::PassiveObservationDecision::Capture
    );
}

#[test]
fn recall_plan_exact_id_terminates_in_tier1() {
    let store = BrainStore::new(membrain_core::RuntimeConfig::default());
    let config = store.config();

    let plan = store
        .recall_engine()
        .plan_recall(RecallRequest::exact(MemoryId(42)), config);

    assert!(plan.terminates_in_tier1());
    assert_eq!(
        plan.route_summary.trace_stages,
        &[RecallTraceStage::Tier1ExactHandle]
    );
}

#[test]
fn recall_plan_session_lookup_routes_to_tier2() {
    let store = BrainStore::new(membrain_core::RuntimeConfig::default());
    let config = store.config();

    let plan = store
        .recall_engine()
        .plan_recall(RecallRequest::small_session_lookup(SessionId(7)), config);

    assert!(!plan.terminates_in_tier1());
    assert!(plan.route_summary.tier1_consulted_first);
    assert!(plan.route_summary.routes_to_deeper_tiers);
}

#[test]
fn ranking_fusion_produces_scores_for_recall_results() {
    let input = RankingInput {
        recency: 800,
        salience: 600,
        strength: 900,
        provenance: 700,
        conflict: 500,
        confidence: 500,
    };

    let result = fuse_scores(input, RankingProfile::balanced());

    assert!(result.final_score > 0);
    assert_eq!(result.profile_name, "balanced");
    assert!(!result.signals.is_empty());
}

#[test]
fn result_builder_adds_and_ranks_candidates() {
    let mut builder = ResultBuilder::new(3, ns("test"));

    let high = fuse_scores(
        RankingInput {
            recency: 900,
            salience: 850,
            strength: 800,
            provenance: 750,
            conflict: 500,
            confidence: 500,
        },
        RankingProfile::balanced(),
    );

    let medium = fuse_scores(
        RankingInput {
            recency: 500,
            salience: 500,
            strength: 500,
            provenance: 500,
            conflict: 500,
            confidence: 500,
        },
        RankingProfile::balanced(),
    );

    let low = fuse_scores(
        RankingInput {
            recency: 100,
            salience: 100,
            strength: 100,
            provenance: 100,
            conflict: 500,
            confidence: 500,
        },
        RankingProfile::balanced(),
    );

    builder.add(
        MemoryId(1),
        ns("test"),
        SessionId(1),
        CanonicalMemoryType::Event,
        "low score item".into(),
        &low,
        AnsweredFrom::Tier1Hot,
    );

    builder.add(
        MemoryId(2),
        ns("test"),
        SessionId(1),
        CanonicalMemoryType::Event,
        "high score item".into(),
        &high,
        AnsweredFrom::Tier2Indexed,
    );

    builder.add(
        MemoryId(3),
        ns("test"),
        SessionId(1),
        CanonicalMemoryType::Observation,
        "medium score item".into(),
        &medium,
        AnsweredFrom::Tier1Hot,
    );

    let explain = RetrievalExplain {
        recall_plan: membrain_core::engine::recall::RecallPlanKind::Tier2ExactThenTier3Fallback,
        route_reason: "test ranking pipeline".to_string(),
        tiers_consulted: vec!["tier1_recent".to_string(), "tier2_exact".to_string()],
        trace_stages: vec![
            RecallTraceStage::Tier1RecentWindow,
            RecallTraceStage::Tier2Exact,
        ],
        tier1_answered_directly: false,
        candidate_budget: 10,
        time_consumed_ms: Some(5),
        ranking_profile: "balanced".to_string(),
        contradictions_found: 0,
        historical_context: None,
        query_by_example: None,
        result_reasons: vec![membrain_core::engine::result::ResultReason {
            memory_id: Some(MemoryId(2)),
            reason_code: "score_kept".to_string(),
            detail: "top-ranked result".to_string(),
        }],
    };

    let result_set = builder.build(explain);

    assert_eq!(result_set.count(), 3);
    assert!(!result_set.truncated);
    assert!(result_set.evidence_pack[0].result.score >= result_set.evidence_pack[1].result.score);
    assert!(result_set.evidence_pack[1].result.score >= result_set.evidence_pack[2].result.score);
}

#[test]
fn result_builder_truncates_to_budget() {
    let mut builder = ResultBuilder::new(2, ns("test"));

    for i in 0..5 {
        let score = (i as u16 + 1) * 100;
        let ranked = fuse_scores(
            RankingInput {
                recency: score,
                salience: score,
                strength: score,
                provenance: score,
                conflict: 500,
                confidence: 500,
            },
            RankingProfile::balanced(),
        );

        builder.add(
            MemoryId(i),
            ns("test"),
            SessionId(1),
            CanonicalMemoryType::Event,
            format!("item {}", i),
            &ranked,
            AnsweredFrom::Tier1Hot,
        );
    }

    let explain = RetrievalExplain {
        recall_plan: membrain_core::engine::recall::RecallPlanKind::Tier2ExactThenTier3Fallback,
        route_reason: "truncation test".to_string(),
        tiers_consulted: vec!["tier1_recent".to_string()],
        trace_stages: vec![RecallTraceStage::Tier1RecentWindow],
        tier1_answered_directly: false,
        candidate_budget: 2,
        time_consumed_ms: Some(3),
        ranking_profile: "balanced".to_string(),
        contradictions_found: 0,
        historical_context: None,
        query_by_example: None,
        result_reasons: vec![],
    };

    let result_set = builder.build(explain);

    assert_eq!(result_set.count(), 2);
    assert!(result_set.truncated);
    assert_eq!(result_set.total_candidates, 5);
}

#[test]
fn empty_result_set_has_correct_outcome_class() {
    let explain = RetrievalExplain {
        recall_plan: membrain_core::engine::recall::RecallPlanKind::ExactIdTier1,
        route_reason: "no candidates".to_string(),
        tiers_consulted: vec!["tier1_exact".to_string()],
        trace_stages: vec![RecallTraceStage::Tier1ExactHandle],
        tier1_answered_directly: true,
        candidate_budget: 10,
        time_consumed_ms: Some(1),
        ranking_profile: "balanced".to_string(),
        contradictions_found: 0,
        historical_context: None,
        query_by_example: None,
        result_reasons: vec![membrain_core::engine::result::ResultReason {
            memory_id: None,
            reason_code: "no_match".to_string(),
            detail: "no candidates found".to_string(),
        }],
    };

    let result_set = RetrievalResultSet::empty(explain, ns("test"));

    assert_eq!(
        result_set.outcome_class,
        membrain_core::observability::OutcomeClass::Preview
    );
    assert_eq!(
        result_set.policy_summary.outcome_class,
        membrain_core::observability::OutcomeClass::Preview
    );
    assert_eq!(result_set.count(), 0);
    assert!(!result_set.truncated);
    assert!(result_set.top().is_none());
}

#[test]
fn historical_trace_exposes_snapshot_selection_details() {
    let store = BrainStore::new(membrain_core::RuntimeConfig::default());
    let namespace = ns("time_travel");
    let snapshot = {
        let mut mutable = store.clone();
        mutable.capture_snapshot(
            namespace.clone(),
            "incident_baseline",
            Some("before rollback".to_string()),
            3,
            membrain_core::types::SnapshotRetentionClass::Restorable,
        )
    };
    let request = RetrievalRequest::hybrid(namespace, "rollback decision", 4)
        .with_as_of_tick_range(20, 80)
        .with_snapshot_name("incident_baseline");

    let mut trace = RetrievalPlanTrace::new(&request);
    trace.set_resolved_snapshot(snapshot.clone());
    trace.set_final_candidates(2);

    let mut explain = RetrievalExplain {
        recall_plan: membrain_core::engine::recall::RecallPlanKind::Tier2ExactThenTier3Fallback,
        route_reason: "historical retrieval".to_string(),
        tiers_consulted: vec!["tier2_exact".to_string()],
        trace_stages: vec![RecallTraceStage::Tier2Exact],
        tier1_answered_directly: false,
        candidate_budget: 4,
        time_consumed_ms: Some(2),
        ranking_profile: "balanced".to_string(),
        contradictions_found: 0,
        historical_context: None,
        query_by_example: None,
        result_reasons: vec![],
    };
    explain.set_historical_trace(&trace);

    let historical_context = explain.historical_context.as_ref().unwrap();
    assert_eq!(historical_context.window_kind, "snapshot");
    assert_eq!(
        historical_context.selection_reason,
        "snapshot_anchor_overrides_tick_window_end"
    );
    assert_eq!(
        historical_context.selected_tick_window,
        Some((0, snapshot.as_of_tick))
    );
    assert_eq!(historical_context.as_of_tick, Some(snapshot.as_of_tick));
    assert_eq!(historical_context.snapshot_id, Some(snapshot.snapshot_id));
    assert_eq!(
        historical_context.snapshot_name.as_deref(),
        Some("incident_baseline")
    );
    assert!(explain.result_reasons.iter().any(|reason| {
        reason.reason_code == "historical_window_selected"
            && reason.detail.contains("historical retrieval used snapshot")
    }));
    assert!(explain.result_reasons.iter().any(|reason| {
        reason.reason_code == "historical_snapshot_selected"
            && reason.detail.contains("incident_baseline")
    }));
}

#[test]
fn recall_trace_exposes_query_by_example_seed_selection_details() {
    let request = RetrievalRequest::hybrid(ns("test"), "  similar outage  ", 4)
        .with_like_memory(MemoryId(42))
        .with_unlike_memory(MemoryId(77));

    let normalization = request.normalize_query_by_example().unwrap();
    let mut trace = RetrievalPlanTrace::new(&request);
    trace.set_query_by_example_materialization(&normalization, &[MemoryId(42), MemoryId(99)]);
    trace.set_final_candidates(3);

    assert_eq!(
        trace.query_by_example_primary_cue,
        Some(PrimaryCue::QueryText)
    );
    assert_eq!(
        trace.query_by_example_seed_descriptors,
        vec!["like:42".to_string(), "unlike:77".to_string()]
    );
    assert_eq!(
        trace.query_by_example_materialized_seed_descriptors,
        vec!["like:42".to_string()]
    );
    assert_eq!(
        trace.query_by_example_missing_seed_descriptors,
        vec!["unlike:77".to_string()]
    );
    assert!(trace.summary().contains(
        "query_by_example: primary_cue=query_text, requested_seeds=[\"like:42\", \"unlike:77\"], materialized_seeds=[\"like:42\"], missing_seeds=[\"unlike:77\"]"
    ));

    let mut explain = RetrievalExplain {
        recall_plan: membrain_core::engine::recall::RecallPlanKind::Tier2ExactThenTier3Fallback,
        route_reason: "query by example".to_string(),
        tiers_consulted: vec!["tier2_exact".to_string()],
        trace_stages: vec![RecallTraceStage::Tier2Exact],
        tier1_answered_directly: false,
        candidate_budget: 4,
        time_consumed_ms: Some(2),
        ranking_profile: "balanced".to_string(),
        contradictions_found: 0,
        historical_context: None,
        query_by_example: None,
        result_reasons: vec![ResultReason {
            memory_id: Some(MemoryId(42)),
            reason_code: "score_kept".to_string(),
            detail: "seeded candidate survived ranking".to_string(),
        }],
    };
    explain.set_query_by_example_trace(&trace);

    let query_by_example = explain.query_by_example.as_ref().unwrap();
    assert_eq!(query_by_example.primary_cue, "query_text");
    assert_eq!(
        query_by_example.requested_seed_descriptors,
        vec!["like:42", "unlike:77"]
    );
    assert_eq!(
        query_by_example.materialized_seed_descriptors,
        vec!["like:42"]
    );
    assert_eq!(query_by_example.missing_seed_descriptors, vec!["unlike:77"]);
    assert_eq!(query_by_example.expanded_candidate_count, 3);
    assert_eq!(
        query_by_example.influence_summary,
        "primary cue query_text expanded 3 candidate(s) from 2 requested seed(s); 1 seed(s) materialized and 1 seed(s) remained unavailable"
    );
    assert!(explain.result_reasons.iter().any(|reason| {
        reason.reason_code == "query_by_example_seed_materialized"
            && reason.detail == "seed like:42 materialized from stored evidence"
    }));
    assert!(explain.result_reasons.iter().any(|reason| {
        reason.reason_code == "query_by_example_seed_missing"
            && reason.detail == "seed unlike:77 was requested but not available for expansion"
    }));
    assert!(explain.result_reasons.iter().any(|reason| {
        reason.reason_code == "query_by_example_candidate_expansion"
            && reason.detail == query_by_example.influence_summary
    }));
}

#[test]
fn confidence_filter_emits_explain_reasons_for_suppressed_candidates() {
    let mut builder = ResultBuilder::new(3, ns("confidence_filter_explain"));

    let high = fuse_scores(
        RankingInput {
            recency: 900,
            salience: 900,
            strength: 800,
            provenance: 900,
            conflict: 500,
            confidence: 900,
        },
        RankingProfile::balanced(),
    );
    let low = fuse_scores(
        RankingInput {
            recency: 200,
            salience: 200,
            strength: 200,
            provenance: 200,
            conflict: 500,
            confidence: 200,
        },
        RankingProfile::balanced(),
    );

    builder.add_with_confidence(
        MemoryId(41),
        ns("confidence_filter_explain"),
        SessionId(1),
        CanonicalMemoryType::Event,
        "high confidence".into(),
        &high,
        AnsweredFrom::Tier2Indexed,
        &membrain_core::engine::confidence::ConfidenceInputs {
            corroboration_count: 8,
            reconsolidation_count: 0,
            ticks_since_last_access: 10,
            age_ticks: 10,
            resolution_state: membrain_core::engine::contradiction::ResolutionState::None,
            conflict_score: 0,
            causal_parent_count: 4,
            authoritativeness: 950,
            recall_count: 6,
        },
        &membrain_core::engine::confidence::ConfidencePolicy::default(),
    );
    builder.add_with_confidence(
        MemoryId(42),
        ns("confidence_filter_explain"),
        SessionId(1),
        CanonicalMemoryType::Event,
        "low confidence".into(),
        &low,
        AnsweredFrom::Tier2Indexed,
        &membrain_core::engine::confidence::ConfidenceInputs {
            corroboration_count: 0,
            reconsolidation_count: 0,
            ticks_since_last_access: 1000,
            age_ticks: 1000,
            resolution_state: membrain_core::engine::contradiction::ResolutionState::Unresolved,
            conflict_score: 800,
            causal_parent_count: 0,
            authoritativeness: 100,
            recall_count: 0,
        },
        &membrain_core::engine::confidence::ConfidencePolicy::default(),
    );

    let result_set = builder.build_with_confidence_filter(
        RetrievalExplain {
            recall_plan: membrain_core::engine::recall::RecallPlanKind::Tier2ExactThenTier3Fallback,
            route_reason: "confidence filter explain".to_string(),
            tiers_consulted: vec!["tier2_exact".to_string()],
            trace_stages: vec![RecallTraceStage::Tier2Exact],
            tier1_answered_directly: false,
            candidate_budget: 3,
            time_consumed_ms: Some(7),
            ranking_profile: "balanced".to_string(),
            contradictions_found: 0,
            historical_context: None,
            query_by_example: None,
            result_reasons: vec![],
        },
        500,
    );

    assert_eq!(result_set.count(), 1);
    assert_eq!(result_set.omitted_summary.confidence_filtered, 1);
    assert_eq!(result_set.omitted_summary.low_confidence_suppressed, 1);
    assert!(result_set.explain.result_reasons.iter().any(|reason| {
        reason.reason_code == "confidence_threshold_applied"
            && reason.detail == "filtered 1 candidate(s) below min_confidence=500"
    }));
    assert!(result_set.explain.result_reasons.iter().any(|reason| {
        reason.memory_id == Some(MemoryId(42))
            && reason.reason_code == "low_confidence_suppressed"
            && reason.detail
                == "candidate suppressed because confidence fell below min_confidence=500"
    }));
    assert!(result_set.evidence_pack[0]
        .result
        .uncertainty_markers
        .confidence_interval
        .is_some());
}

#[test]
fn confidence_filter_empty_result_set_degrades_outcome_from_accepted() {
    let mut builder = ResultBuilder::new(3, ns("confidence_filter_empty"));

    let low = fuse_scores(
        RankingInput {
            recency: 200,
            salience: 200,
            strength: 200,
            provenance: 200,
            conflict: 500,
            confidence: 200,
        },
        RankingProfile::balanced(),
    );

    builder.add_with_confidence(
        MemoryId(99),
        ns("confidence_filter_empty"),
        SessionId(1),
        CanonicalMemoryType::Event,
        "filtered away".into(),
        &low,
        AnsweredFrom::Tier2Indexed,
        &membrain_core::engine::confidence::ConfidenceInputs {
            corroboration_count: 0,
            reconsolidation_count: 0,
            ticks_since_last_access: 1000,
            age_ticks: 1000,
            resolution_state: membrain_core::engine::contradiction::ResolutionState::Unresolved,
            conflict_score: 800,
            causal_parent_count: 0,
            authoritativeness: 100,
            recall_count: 0,
        },
        &membrain_core::engine::confidence::ConfidencePolicy::default(),
    );

    let result_set = builder.build_with_confidence_filter(
        RetrievalExplain {
            recall_plan: membrain_core::engine::recall::RecallPlanKind::Tier2ExactThenTier3Fallback,
            route_reason: "confidence filter emptied the result set".to_string(),
            tiers_consulted: vec!["tier2_exact".to_string()],
            trace_stages: vec![RecallTraceStage::Tier2Exact],
            tier1_answered_directly: false,
            candidate_budget: 3,
            time_consumed_ms: Some(7),
            ranking_profile: "balanced".to_string(),
            contradictions_found: 0,
            historical_context: None,
            query_by_example: None,
            result_reasons: vec![],
        },
        500,
    );

    assert_eq!(result_set.count(), 0);
    assert_eq!(
        result_set.outcome_class,
        membrain_core::observability::OutcomeClass::Preview
    );
    assert_eq!(
        result_set.policy_summary.outcome_class,
        membrain_core::observability::OutcomeClass::Preview
    );
    assert_eq!(result_set.omitted_summary.confidence_filtered, 1);
    assert!(result_set.explain.result_reasons.iter().any(|reason| {
        reason.reason_code == "confidence_threshold_applied"
            && reason.detail == "filtered 1 candidate(s) below min_confidence=500"
    }));
}

#[test]
fn explain_markers_surface_reconsolidation_churn_when_present() {
    let mut builder = ResultBuilder::new(1, ns("reconsolidation_churn_marker"));
    let ranked = fuse_scores(
        RankingInput {
            recency: 500,
            salience: 500,
            strength: 500,
            provenance: 500,
            conflict: 500,
            confidence: 800,
        },
        RankingProfile::balanced(),
    );

    builder.add_with_confidence(
        MemoryId(78),
        ns("reconsolidation_churn_marker"),
        SessionId(1),
        CanonicalMemoryType::Event,
        "retained churn-heavy confidence".into(),
        &ranked,
        AnsweredFrom::Tier2Indexed,
        &membrain_core::engine::confidence::ConfidenceInputs {
            corroboration_count: 4,
            reconsolidation_count: 16,
            ticks_since_last_access: 10,
            age_ticks: 10,
            resolution_state: membrain_core::engine::contradiction::ResolutionState::None,
            conflict_score: 0,
            causal_parent_count: 2,
            authoritativeness: 900,
            recall_count: 4,
        },
        &membrain_core::engine::confidence::ConfidencePolicy::default(),
    );

    let result_set = builder.build(RetrievalExplain {
        recall_plan: membrain_core::engine::recall::RecallPlanKind::Tier2ExactThenTier3Fallback,
        route_reason: "reconsolidation churn marker".to_string(),
        tiers_consulted: vec!["tier2_exact".to_string()],
        trace_stages: vec![RecallTraceStage::Tier2Exact],
        tier1_answered_directly: false,
        candidate_budget: 1,
        time_consumed_ms: Some(4),
        ranking_profile: "balanced".to_string(),
        contradictions_found: 0,
        historical_context: None,
        query_by_example: None,
        result_reasons: vec![],
    });

    let (_, _, uncertainty_markers) = result_set.explain_markers();
    assert_eq!(uncertainty_markers.len(), 1);
    assert_eq!(uncertainty_markers[0].code, "reconsolidation_churn");
    assert_eq!(
        uncertainty_markers[0].detail,
        "bounded evidence shows elevated reconsolidation churn and reduced reliability"
    );
}

#[test]
fn explain_markers_surface_low_confidence_when_result_is_retained() {
    let mut builder = ResultBuilder::new(1, ns("low_confidence_marker"));
    let ranked = fuse_scores(
        RankingInput {
            recency: 300,
            salience: 300,
            strength: 300,
            provenance: 200,
            conflict: 500,
            confidence: 200,
        },
        RankingProfile::balanced(),
    );

    builder.add_with_confidence(
        MemoryId(77),
        ns("low_confidence_marker"),
        SessionId(1),
        CanonicalMemoryType::Event,
        "retained low confidence".into(),
        &ranked,
        AnsweredFrom::Tier2Indexed,
        &membrain_core::engine::confidence::ConfidenceInputs {
            corroboration_count: 0,
            reconsolidation_count: 0,
            ticks_since_last_access: 1000,
            age_ticks: 1000,
            resolution_state: membrain_core::engine::contradiction::ResolutionState::Unresolved,
            conflict_score: 900,
            causal_parent_count: 0,
            authoritativeness: 100,
            recall_count: 0,
        },
        &membrain_core::engine::confidence::ConfidencePolicy::default(),
    );

    let result_set = builder.build(RetrievalExplain {
        recall_plan: membrain_core::engine::recall::RecallPlanKind::Tier2ExactThenTier3Fallback,
        route_reason: "low confidence marker".to_string(),
        tiers_consulted: vec!["tier2_exact".to_string()],
        trace_stages: vec![RecallTraceStage::Tier2Exact],
        tier1_answered_directly: false,
        candidate_budget: 1,
        time_consumed_ms: Some(4),
        ranking_profile: "balanced".to_string(),
        contradictions_found: 0,
        historical_context: None,
        query_by_example: None,
        result_reasons: vec![],
    });

    let (_, _, uncertainty_markers) = result_set.explain_markers();
    assert_eq!(uncertainty_markers.len(), 1);
    assert_eq!(uncertainty_markers[0].code, "low_confidence");
    assert_eq!(
        uncertainty_markers[0].detail,
        "bounded evidence fell below the action-oriented confidence threshold"
    );
}

#[test]
fn answered_from_tier_reports_correct_source() {
    assert_eq!(AnsweredFrom::Tier1Hot.as_str(), "tier1_hot");
    assert_eq!(AnsweredFrom::Tier2Indexed.as_str(), "tier2_indexed");
    assert_eq!(AnsweredFrom::Tier3Cold.as_str(), "tier3_cold");
}

#[test]
fn realistic_semantic_query_beats_lexical_distractor_on_normal_tier2_path() {
    let namespace = ns("semantic_normal_path");
    let query = "Which release deploy fix should we roll out after the pipeline outage?";
    let lexical_distractor = "release rollback checklist after outage";
    let semantic_winner = "production deploy pipeline remediation rollout for incident fix";

    let records = vec![
        semantic_record(10, &namespace, lexical_distractor),
        semantic_record(20, &namespace, semantic_winner),
    ];
    let mut embedder = CachedTextEmbedder::new(SemanticFixtureEmbedder, 8);
    let semantic = SharedSemanticRetrievalExecutor.execute(
        &records,
        &namespace,
        query,
        None,
        SemanticExecutorConfig::bounded(2),
        &mut embedder,
    );

    assert_eq!(semantic.candidates.len(), 2);
    assert!(semantic.trace.degraded_reason.is_none());
    assert_eq!(semantic.trace.lexical_prefilter_count, 2);
    assert_eq!(semantic.trace.semantic_candidate_count, 2);
    assert_eq!(semantic.candidates[0].record.memory_id, MemoryId(20));
    assert!(semantic.candidates[0].semantic_score > semantic.candidates[1].semantic_score);
    assert!(semantic.candidates[1].lexical_score > semantic.candidates[0].lexical_score);
    assert_eq!(
        semantic
            .trace
            .query_trace
            .as_ref()
            .expect("query trace")
            .backend_kind,
        "semantic_fixture"
    );

    let top_ranking = fuse_scores(
        RankingInput {
            recency: semantic.candidates[0].ranking_score,
            salience: semantic.candidates[0].ranking_score,
            strength: semantic.candidates[0].ranking_score,
            provenance: 700,
            conflict: 500,
            confidence: semantic.candidates[0].ranking_score,
        },
        RankingProfile::balanced(),
    );
    let runner_up_ranking = fuse_scores(
        RankingInput {
            recency: semantic.candidates[1].ranking_score,
            salience: semantic.candidates[1].ranking_score,
            strength: semantic.candidates[1].ranking_score,
            provenance: 650,
            conflict: 500,
            confidence: semantic.candidates[1].ranking_score,
        },
        RankingProfile::balanced(),
    );

    let mut builder = ResultBuilder::new(2, namespace.clone());
    builder.add(
        semantic.candidates[0].record.memory_id,
        namespace.clone(),
        SessionId(1),
        CanonicalMemoryType::Event,
        semantic.candidates[0].record.compact_text.clone(),
        &top_ranking,
        AnsweredFrom::Tier2Indexed,
    );
    builder.add(
        semantic.candidates[1].record.memory_id,
        namespace.clone(),
        SessionId(1),
        CanonicalMemoryType::Event,
        semantic.candidates[1].record.compact_text.clone(),
        &runner_up_ranking,
        AnsweredFrom::Tier2Indexed,
    );

    let result_set = builder.build(RetrievalExplain {
        recall_plan: RecallPlanKind::Tier2ExactThenTier3Fallback,
        route_reason: "normal semantic retrieval path outranked lexical distractor".to_string(),
        tiers_consulted: vec!["tier2_semantic".to_string()],
        trace_stages: vec![RecallTraceStage::Tier2Exact],
        tier1_answered_directly: false,
        candidate_budget: 2,
        time_consumed_ms: Some(3),
        ranking_profile: "balanced".to_string(),
        contradictions_found: 0,
        historical_context: None,
        query_by_example: None,
        result_reasons: vec![
            ResultReason {
                memory_id: None,
                reason_code: "semantic_executor_trace".to_string(),
                detail: "shared semantic executor used lexical prefilter over 2 namespace candidate(s), produced 2 prefilter candidate(s), returned 2 semantic candidate(s), and enforced bounded result_limit=2"
                    .to_string(),
            },
            ResultReason {
                memory_id: Some(MemoryId(20)),
                reason_code: "semantic_path_won".to_string(),
                detail: "bounded semantic scoring promoted the deployment remediation memory over the lexical distractor"
                    .to_string(),
            },
        ],
    });

    let top = result_set.top().expect("top result");
    assert_eq!(top.result.memory_id, MemoryId(20));
    assert_eq!(top.result.answered_from, AnsweredFrom::Tier2Indexed);
    assert_eq!(top.result.entry_lane.as_str(), "semantic");
    assert!(result_set.explain.result_reasons.iter().any(|reason| {
        reason.reason_code == "semantic_path_won" && reason.detail.contains("lexical distractor")
    }));

    let explain_reasons = result_set.explain_result_reasons();
    assert!(explain_reasons.iter().any(|reason| {
        reason.reason_code == "semantic_executor_trace"
            && reason.reason_family == "selection"
            && reason.route_stage == membrain_core::observability::TraceStage::Tier2Exact
            && reason
                .detail
                .contains("lexical prefilter over 2 namespace candidate(s)")
            && reason.detail.contains("returned 2 semantic candidate(s)")
            && reason.detail.contains("bounded result_limit=2")
    }));
}

#[test]
fn evidence_role_has_correct_string_representation() {
    assert_eq!(EvidenceRole::Primary.as_str(), "primary");
    assert_eq!(EvidenceRole::Supporting.as_str(), "supporting");
}

#[test]
fn payload_state_has_correct_string_representation() {
    assert_eq!(PayloadState::Inline.as_str(), "inline");
    assert_eq!(PayloadState::PreviewOnly.as_str(), "preview_only");
    assert_eq!(PayloadState::Deferred.as_str(), "deferred");
    assert_eq!(PayloadState::Redacted.as_str(), "redacted");
}

#[test]
fn encode_to_recall_round_trip_preserves_payload_policy_and_temporal_explain() {
    let store = BrainStore::new(membrain_core::RuntimeConfig::default());
    let namespace = ns("round_trip");
    let memory_id = MemoryId(21);
    let session_id = SessionId(8);
    let prepared = store.prepare_tier2_layout_with_trace_from_encode(
        namespace.clone(),
        memory_id,
        session_id,
        RawEncodeInput::new(RawIntakeKind::Event, "Launch day").with_landmark_signals(
            membrain_core::types::LandmarkSignals::new(0.95, 0.91, 0.12, 88),
        ),
    );

    let plan = store
        .recall_engine()
        .plan_recall(RecallRequest::exact(memory_id), store.config());
    let ranking = fuse_scores(
        RankingInput {
            recency: 920,
            salience: 910,
            strength: 860,
            provenance: 830,
            conflict: 500,
            confidence: 500,
        },
        RankingProfile::balanced(),
    );
    let mut builder = ResultBuilder::new(1, namespace.clone());
    builder.add(
        memory_id,
        namespace.clone(),
        session_id,
        prepared.layout.metadata.memory_type,
        prepared.layout.metadata.compact_text.clone(),
        &ranking,
        AnsweredFrom::Tier1Hot,
    );
    let mut explain = RetrievalExplain::from_plan(&plan, "balanced");
    explain.push_temporal_landmark_reasons_from_prepared_layout(&prepared);
    let result_set = builder.build(explain);
    let (policy_summary, provenance_summary) = result_set.explain_policy_and_provenance();

    assert_eq!(result_set.count(), 1);
    assert_eq!(result_set.top().unwrap().result.memory_id, memory_id);
    assert_eq!(result_set.top().unwrap().result.compact_text, "Launch day");
    assert!(result_set.deferred_payloads.is_empty());
    assert!(prepared.prefilter_stays_metadata_only());
    assert!(prepared.layout.prefilter_stays_metadata_only());
    assert_eq!(prepared.prefilter_trace.payload_fetch_count, 0);
    assert!(result_set.explain.result_reasons.iter().any(|reason| {
        reason.reason_code == "temporal_prefilter_metadata_only"
            && reason.memory_id == Some(memory_id)
    }));
    assert!(result_set.explain.result_reasons.iter().any(|reason| {
        reason.reason_code == "temporal_payload_deferred"
            && reason.detail.contains("tier2://round_trip/payload")
    }));
    assert!(result_set.explain.result_reasons.iter().any(|reason| {
        reason.reason_code == "temporal_landmark_selected" && reason.detail.contains("tick 88")
    }));
    assert_eq!(policy_summary.effective_namespace, "round_trip");
    assert_eq!(policy_summary.policy_family, "namespace");
    assert_eq!(policy_summary.sharing_scope, "custom_sharing_scope");
    assert_eq!(policy_summary.retention_state, "absent");
    assert_eq!(provenance_summary.source_reference, "result_builder");
}

#[test]
fn health_report_surfaces_batch1_metrics_and_status_rollups() {
    let mut cache = CacheManager::new(4, 4);
    cache.prefetch.submit_hint(
        ns("health_surface"),
        membrain_core::store::cache::PrefetchTrigger::SessionRecency,
        vec![MemoryId(1), MemoryId(2)],
    );
    let dropped = cache.prefetch.cancel_namespace(
        &ns("health_surface"),
        membrain_core::store::cache::PrefetchBypassReason::NamespaceNarrowed,
    );
    assert_eq!(dropped, 1);

    let report = BrainHealthReport::from_inputs(
        BrainHealthInputs {
            hot_memories: 12,
            hot_capacity: 20,
            cold_memories: 5,
            avg_strength: 0.61,
            avg_confidence: 0.74,
            low_confidence_count: 2,
            decay_rate: 0.02,
            archive_count: 3,
            lifecycle: membrain_core::health::LifecycleHealthReport {
                consolidated_to_cold_count: 5,
                reconsolidation_active_count: 1,
                forgetting_archive_count: 3,
                background_maintenance_runs: 3,
                background_maintenance_log: vec![
                    "maintenance_consolidation_completed:cold_migration=5".to_string(),
                    "maintenance_reconsolidation_applied:volatile_results=1".to_string(),
                    "maintenance_forgetting_evaluated:archived=3".to_string(),
                ],
            },
            total_engrams: 4,
            avg_cluster_size: 1.5,
            top_engrams: vec![("ops".to_string(), 2)],
            landmark_count: 1,
            unresolved_conflicts: 0,
            uncertain_count: 1,
            dream_links_total: 7,
            last_dream_tick: Some(99),
            affect_history_rows: 1,
            latest_affect_snapshot: Some((0.4, 0.9)),
            latest_affect_tick: Some(118),
            attention_namespaces: vec![AttentionNamespaceInputs {
                namespace: "health_surface".to_string(),
                recall_count: 9,
                encode_count: 3,
                working_memory_pressure: 5,
                promotion_count: 1,
                overflow_count: 1,
            }],
            total_recalls: 15,
            total_encodes: 6,
            current_tick: 120,
            daemon_uptime_ticks: 90,
            index_reports: IndexModule.health_reports(),
            availability: Some(membrain_core::api::AvailabilitySummary::degraded(
                vec!["recall", "health"],
                vec!["encode"],
                vec![membrain_core::api::AvailabilityReason::CacheInvalidated],
                vec![membrain_core::api::RemediationStep::CheckHealth],
            )),
            feature_availability: vec![
                FeatureAvailabilityEntry {
                    feature: "health".to_string(),
                    posture: membrain_core::api::AvailabilityPosture::Full,
                    note: Some("dashboard_surface_ready".to_string()),
                },
                FeatureAvailabilityEntry {
                    feature: "dream".to_string(),
                    posture: membrain_core::api::AvailabilityPosture::Degraded,
                    note: Some("offline_scheduler_only".to_string()),
                },
            ],
            previous_total_recalls: Some(10),
            previous_total_encodes: Some(5),
            previous_repair_queue_depth: None,
            previous_hot_memories: Some(10),
            previous_low_confidence_count: Some(3),
            previous_unresolved_conflicts: Some(1),
            previous_uncertain_count: Some(2),
            previous_cache_hit_count: Some(0),
            previous_cache_miss_count: Some(0),
            previous_cache_bypass_count: Some(0),
            previous_prefetch_queue_depth: Some(0),
            previous_prefetch_drop_count: Some(0),
            previous_index_stale_count: Some(0),
            previous_index_missing_count: Some(0),
            previous_index_repair_backlog_total: Some(0),
            previous_availability_posture: Some(membrain_core::api::AvailabilityPosture::Full),
        },
        &cache,
        None,
    );

    assert_eq!(report.dream_links_total, 7);
    assert_eq!(report.last_dream_tick, Some(99));
    assert_eq!(report.affect_history_rows, 1);
    assert_eq!(report.latest_affect_snapshot, Some((0.4, 0.9)));
    assert_eq!(report.latest_affect_tick, Some(118));
    assert_eq!(report.attention.hotspot_count, 1);
    assert_eq!(report.attention.total_recall_count, 9);
    assert_eq!(report.attention.hotspots[0].namespace, "health_surface");
    assert_eq!(report.attention.hotspots[0].attention_score, 163);
    assert_eq!(report.attention.hotspots[0].status, "warming");
    assert_eq!(report.feature_availability.len(), 2);
    assert!(report.feature_availability.iter().any(|feature| {
        feature.feature == "dream"
            && feature.posture == membrain_core::api::AvailabilityPosture::Degraded
            && feature.note.as_deref() == Some("offline_scheduler_only")
    }));
    assert_eq!(report.backpressure_state, Some("normal"));
    assert_eq!(
        report.availability_posture,
        Some(membrain_core::api::AvailabilityPosture::Degraded)
    );
    assert!(report
        .availability_notes
        .as_deref()
        .is_some_and(|notes| notes.contains("cache_invalidated")));
    assert!(report.subsystem_status.iter().any(|status| {
        status.subsystem == "availability"
            && status.detail.contains("posture=degraded")
            && status.reasons.contains(&"cache_invalidated")
    }));
    assert!(report.trends.iter().any(|trend| {
        trend.subsystem == "memory"
            && trend.metric == "low_confidence_count"
            && trend.direction == membrain_core::health::TrendDirection::Improving
    }));
    assert!(report.trend_summary.iter().any(|summary| {
        summary.subsystem == "availability"
            && summary
                .trends
                .iter()
                .any(|trend| trend.metric == "availability_posture")
    }));
    assert!(report.dashboard_views.iter().any(|view| {
        view.view.as_str() == "affect_trajectory"
            && view.summary.contains("rows=1")
            && view.drill_down_targets.contains(&"/mood_history")
    }));
    assert!(report
        .drill_down_paths
        .iter()
        .any(|path| path.path == "/mood_history" && path.target_ref == "mood_history"));
}

#[test]
fn contradiction_and_policy_integration_surface_redaction_and_legal_hold_visibility() {
    let mut store = BrainStore::default();
    let namespace = ns("policy_contradiction");
    store.contradiction_engine_mut().register_memory(
        namespace.clone(),
        MemoryId(10),
        101,
        "deployment target is prod-a".into(),
    );
    store.contradiction_engine_mut().register_memory(
        namespace.clone(),
        MemoryId(20),
        202,
        "deployment target is prod-b".into(),
    );
    let branch = store
        .record_encode_contradiction(
            namespace.clone(),
            MemoryId(10),
            MemoryId(20),
            ContradictionKind::Supersession,
            880,
        )
        .unwrap();
    let encode_candidate = ContradictionCandidate {
        memory_id: MemoryId(30),
        fingerprint: 202,
        compact_text: "deployment target is prod-b".into(),
        namespace: namespace.clone(),
    };
    let detect_outcome = store
        .detect_and_branch_encode(namespace.clone(), MemoryId(30), &encode_candidate)
        .unwrap();

    let cross_namespace_request = RequestContext {
        namespace: Some(namespace.clone()),
        workspace_id: None,
        agent_id: None,
        session_id: Some(SessionId(5)),
        task_id: None,
        request_id: RequestId::new("req-policy-contradiction").unwrap(),
        policy_context: PolicyContext {
            include_public: false,
            sharing_visibility: membrain_core::policy::SharingVisibility::Private,
            caller_identity_bound: true,
            workspace_acl_allowed: true,
            agent_acl_allowed: true,
            session_visibility_allowed: true,
            legal_hold: false,
        },
        time_budget_ms: None,
    };
    let sharing_outcome = cross_namespace_request
        .bind_namespace(None)
        .unwrap()
        .evaluate_cross_namespace_sharing_access(store.policy(), &ns("policy_other"));

    let mut contradiction_ranked = fuse_scores(
        RankingInput {
            recency: 780,
            salience: 770,
            strength: 740,
            provenance: 920,
            conflict: 260,
            confidence: 760,
        },
        RankingProfile::balanced(),
    );
    contradiction_ranked.contradiction_explains = vec![ContradictionExplain {
        contradiction_id: branch.contradiction_id,
        kind: ContradictionKind::Supersession,
        resolution: ResolutionState::AuthoritativelyResolved,
        preferred_answer_state: PreferredAnswerState::Preferred,
        preferred_memory: Some(MemoryId(20)),
        confidence_signal: 980,
        conflicting_memory: MemoryId(10),
        lineage_pair: [MemoryId(10), MemoryId(20)],
        result_is_preferred: true,
        superseded_memory: Some(MemoryId(10)),
        resolution_reason: Some("authoritative archive".to_string()),
        active_contradiction: false,
        archived: true,
        legal_hold: true,
        authoritative_evidence: true,
        retention_reason: Some("legal hold keeps archived contradiction".to_string()),
        audit_visible: true,
    }];

    let mut builder = ResultBuilder::new(1, namespace.clone());
    builder.add(
        MemoryId(20),
        namespace.clone(),
        SessionId(5),
        CanonicalMemoryType::Event,
        "deployment target is prod-b".into(),
        &contradiction_ranked,
        AnsweredFrom::Tier2Indexed,
    );
    let mut result_set = builder.build(RetrievalExplain {
        recall_plan: membrain_core::engine::recall::RecallPlanKind::Tier2ExactThenTier3Fallback,
        route_reason: "contradiction and policy packaging".to_string(),
        tiers_consulted: vec!["tier2_exact".to_string()],
        trace_stages: vec![RecallTraceStage::Tier2Exact],
        tier1_answered_directly: false,
        candidate_budget: 4,
        time_consumed_ms: Some(7),
        ranking_profile: "balanced".to_string(),
        contradictions_found: 0,
        historical_context: None,
        query_by_example: None,
        result_reasons: vec![
            ResultReason {
                memory_id: Some(MemoryId(20)),
                reason_code: "contradiction_retained_under_legal_hold".to_string(),
                detail: "legal hold keeps archived authoritative evidence visible".to_string(),
            },
            ResultReason {
                memory_id: None,
                reason_code: "policy_redacted".to_string(),
                detail: "1 cross-namespace item redacted by namespace policy".to_string(),
            },
        ],
    });
    result_set.policy_summary.redactions_applied = true;
    result_set.policy_summary.restrictions_active = vec![
        "cross_namespace_private".to_string(),
        "retention".to_string(),
    ];
    result_set.policy_summary.filters = vec![membrain_core::api::PolicyFilterSummary::new(
        namespace.as_str(),
        "namespace",
        membrain_core::observability::OutcomeClass::Accepted,
        "package",
        FieldPresence::Redacted,
        FieldPresence::Present("legal_hold".to_string()),
        sharing_outcome
            .redaction_fields
            .iter()
            .map(|field| (*field).to_string())
            .collect(),
    )];
    result_set.omitted_summary.policy_redacted = 1;

    let (trace_policy_summary, _) = result_set.explain_policy_and_provenance();

    assert!(detect_outcome.is_contradiction());
    assert_eq!(
        sharing_outcome.decision,
        membrain_core::policy::SharingAccessDecision::Deny
    );
    assert_eq!(
        sharing_outcome.redaction_fields,
        vec!["memory_id", "sharing_scope", "workspace_id", "session_id"]
    );
    assert_eq!(result_set.explain.contradictions_found, 1);
    assert_eq!(result_set.omitted_summary.policy_redacted, 1);
    assert!(result_set.policy_summary.redactions_applied);
    assert_eq!(
        result_set.policy_summary.restrictions_active,
        vec!["cross_namespace_private", "retention"]
    );
    assert_eq!(
        result_set.evidence_pack[0]
            .result
            .conflict_markers
            .audit_visible_count,
        1
    );
    assert_eq!(
        result_set.evidence_pack[0]
            .result
            .conflict_markers
            .resolution_reasons,
        vec!["authoritative archive".to_string()]
    );
    assert!(result_set
        .explain
        .result_reasons
        .iter()
        .any(|reason| { reason.reason_code == "contradiction_retained_under_legal_hold" }));
    assert!(result_set.explain.result_reasons.iter().any(|reason| {
        reason.reason_code == "policy_redacted" && reason.detail.contains("cross-namespace")
    }));
    assert_eq!(
        trace_policy_summary.effective_namespace,
        "policy_contradiction"
    );
    assert_eq!(trace_policy_summary.sharing_scope, "same_namespace");
    assert_eq!(trace_policy_summary.retention_state, "retained");
    assert_eq!(trace_policy_summary.redaction_fields, vec!["payload"]);
    assert_eq!(
        result_set.policy_summary.filters[0].redaction_fields,
        vec![
            "memory_id".to_string(),
            "sharing_scope".to_string(),
            "workspace_id".to_string(),
            "session_id".to_string(),
        ]
    );
}

#[test]
fn cache_metrics_preserve_per_family_hit_breakdown() {
    let summary = membrain_core::api::CacheMetricsSummary::from_cache_traces(
        vec![
            CacheEvalTrace {
                cache_family: CacheFamilyLabel::Tier2Query,
                cache_event: CacheEventLabel::Hit,
                outcome: CacheLookupOutcome::Hit,
                cache_reason: None,
                warm_source: Some(WarmSourceLabel::Tier2QueryCache),
                generation_status: GenerationStatusLabel::Valid,
                candidates_before: 6,
                candidates_after: 2,
                warm_reuse: true,
            },
            CacheEvalTrace {
                cache_family: CacheFamilyLabel::Negative,
                cache_event: CacheEventLabel::Hit,
                outcome: CacheLookupOutcome::Hit,
                cache_reason: Some(CacheReasonLabel::ColdStart),
                warm_source: None,
                generation_status: GenerationStatusLabel::Unknown,
                candidates_before: 2,
                candidates_after: 2,
                warm_reuse: false,
            },
        ],
        false,
    );

    assert_eq!(summary.cache_hit_count, 2);
    assert_eq!(summary.tier2_query_hit_count, 1);
    assert_eq!(summary.negative_cache_hit_count, 1);
    assert_eq!(summary.post_tier2_candidates, Some(2));
}

#[test]
fn graph_direct_hit_fixture_has_no_associative_expansion() {
    let planner = BoundedExpansionPlanner::new(ExpansionConstraints {
        max_depth: 2,
        max_entities: 4,
        min_strength: 50,
        follow_reverse: false,
    });

    let (neighborhood, explain) = planner.plan_bfs(EntityId(42), |_| Vec::new());

    assert_eq!(neighborhood.seed, EntityId(42));
    assert!(neighborhood.entities.is_empty());
    assert!(explain.followed_edges.is_empty());
    assert!(explain.omitted_neighbors.is_empty());
    assert!(explain.cutoff_reasons.is_empty());
}

#[test]
fn graph_partial_cue_fixture_reports_engram_seeded_expansion_details() {
    let planner = BoundedExpansionPlanner::new(ExpansionConstraints {
        max_depth: 2,
        max_entities: 4,
        min_strength: 50,
        follow_reverse: false,
    });

    let (neighborhood, explain) = planner.plan_bfs_with_additional_seeds(
        EntityId(7),
        &[EntityId(99)],
        |current| match current {
            EntityId(7) => vec![(
                GraphEdge {
                    from: current,
                    to: EntityId(8),
                    relation: RelationKind::Mentions,
                    strength: 100,
                },
                GraphEntity {
                    id: EntityId(8),
                    kind: EntityKind::Concept,
                    label: "direct-seed-neighbor".into(),
                    namespace: ns("graph"),
                    memory_id: None,
                },
            )],
            EntityId(99) => vec![(
                GraphEdge {
                    from: current,
                    to: EntityId(100),
                    relation: RelationKind::SharedTopic,
                    strength: 100,
                },
                GraphEntity {
                    id: EntityId(100),
                    kind: EntityKind::Memory,
                    label: "engram-member".into(),
                    namespace: ns("graph"),
                    memory_id: Some(MemoryId(100)),
                },
            )],
            _ => Vec::new(),
        },
    );

    assert_eq!(neighborhood.entities.len(), 2);
    assert_eq!(explain.followed_edges.len(), 2);
    assert!(explain
        .followed_edges
        .iter()
        .any(|edge| edge.via_additional_seed && edge.to == EntityId(100)));
    assert!(explain.omitted_neighbors.is_empty());
}

#[test]
fn graph_capped_associative_fixture_reports_omitted_neighbor() {
    let planner = BoundedExpansionPlanner::new(ExpansionConstraints {
        max_depth: 2,
        max_entities: 1,
        min_strength: 50,
        follow_reverse: false,
    });

    let (_neighborhood, explain) = planner.plan_bfs(EntityId(1), |current| {
        if current == EntityId(1) {
            vec![
                (
                    GraphEdge {
                        from: current,
                        to: EntityId(2),
                        relation: RelationKind::Mentions,
                        strength: 100,
                    },
                    GraphEntity {
                        id: EntityId(2),
                        kind: EntityKind::Concept,
                        label: "kept".into(),
                        namespace: ns("graph"),
                        memory_id: None,
                    },
                ),
                (
                    GraphEdge {
                        from: current,
                        to: EntityId(3),
                        relation: RelationKind::SharedTopic,
                        strength: 100,
                    },
                    GraphEntity {
                        id: EntityId(3),
                        kind: EntityKind::Memory,
                        label: "omitted".into(),
                        namespace: ns("graph"),
                        memory_id: Some(MemoryId(3)),
                    },
                ),
            ]
        } else {
            Vec::new()
        }
    });

    assert_eq!(explain.followed_edges.len(), 1);
    assert_eq!(explain.omitted_neighbors.len(), 1);
    assert_eq!(explain.omitted_neighbors[0].entity_id, EntityId(3));
    assert_eq!(
        explain.omitted_neighbors[0].reason,
        CutoffReason::MaxNodesReached(1)
    );
    assert!(explain
        .cutoff_reasons
        .contains(&CutoffReason::BudgetExhausted));
}

#[test]
fn graph_namespace_gate_fixture_reports_policy_cutoff_without_following_blocked_neighbor() {
    let planner = BoundedExpansionPlanner::new(ExpansionConstraints {
        max_depth: 2,
        max_entities: 4,
        min_strength: 50,
        follow_reverse: false,
    });

    let allowed_namespace = ns("graph");
    let blocked_namespace = ns("other-graph");
    let (neighborhood, explain) = planner.plan_bfs_with_policy(
        EntityId(10),
        &[],
        |current| {
            if current == EntityId(10) {
                vec![
                    (
                        GraphEdge {
                            from: current,
                            to: EntityId(11),
                            relation: RelationKind::Mentions,
                            strength: 100,
                        },
                        GraphEntity {
                            id: EntityId(11),
                            kind: EntityKind::Concept,
                            label: "allowed".into(),
                            namespace: allowed_namespace.clone(),
                            memory_id: None,
                        },
                    ),
                    (
                        GraphEdge {
                            from: current,
                            to: EntityId(12),
                            relation: RelationKind::SharedTopic,
                            strength: 100,
                        },
                        GraphEntity {
                            id: EntityId(12),
                            kind: EntityKind::Memory,
                            label: "blocked".into(),
                            namespace: blocked_namespace.clone(),
                            memory_id: Some(MemoryId(12)),
                        },
                    ),
                ]
            } else {
                Vec::new()
            }
        },
        |entity| entity.namespace == allowed_namespace,
    );

    assert_eq!(neighborhood.entities.len(), 1);
    assert_eq!(neighborhood.entities[0].id, EntityId(11));
    assert_eq!(explain.followed_edges.len(), 1);
    assert_eq!(explain.followed_edges[0].to, EntityId(11));
    assert_eq!(explain.omitted_neighbors.len(), 1);
    assert_eq!(explain.omitted_neighbors[0].entity_id, EntityId(12));
    assert_eq!(
        explain.omitted_neighbors[0].reason,
        CutoffReason::PolicyNamespaceBlocked
    );
    assert!(explain
        .cutoff_reasons
        .contains(&CutoffReason::PolicyNamespaceBlocked));
    assert!(!explain
        .followed_edges
        .iter()
        .any(|edge| edge.to == EntityId(12)));
}
