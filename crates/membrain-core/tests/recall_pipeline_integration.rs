use membrain_core::api::NamespaceId;
use membrain_core::engine::ranking::{fuse_scores, RankingInput, RankingProfile};
use membrain_core::engine::recall::{RecallRequest, RecallRuntime, RecallTraceStage};
use membrain_core::engine::result::{
    AnsweredFrom, EvidenceRole, PayloadState, ResultBuilder, ResultReason, RetrievalExplain,
    RetrievalResultSet,
};
use membrain_core::engine::retrieval_planner::{PrimaryCue, RetrievalPlanTrace, RetrievalRequest};
use membrain_core::graph::{
    BoundedExpansionPlanner, CutoffReason, EntityId, EntityKind, ExpansionConstraints, GraphEdge,
    GraphEntity, RelationKind,
};
use membrain_core::observability::{
    CacheEvalTrace, CacheEventLabel, CacheFamilyLabel, CacheLookupOutcome, CacheReasonLabel,
    GenerationStatusLabel, WarmSourceLabel,
};
use membrain_core::types::{
    CanonicalMemoryType, MemoryId, RawEncodeInput, RawIntakeKind, SessionId,
};
use membrain_core::BrainStore;

fn ns(s: &str) -> NamespaceId {
    NamespaceId::new(s).unwrap()
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
            query_by_example: None,
            result_reasons: vec![],
        },
        500,
    );

    assert_eq!(result_set.count(), 0);
    assert_eq!(result_set.outcome_class, membrain_core::observability::OutcomeClass::Preview);
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
