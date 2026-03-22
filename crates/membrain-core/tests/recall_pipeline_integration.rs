use membrain_core::api::NamespaceId;
use membrain_core::engine::ranking::{fuse_scores, RankingInput, RankingProfile};
use membrain_core::engine::recall::{RecallRequest, RecallRuntime, RecallTraceStage};
use membrain_core::engine::result::{
    AnsweredFrom, EvidenceRole, PayloadState, ResultBuilder, RetrievalExplain, RetrievalResultSet,
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
        result_reasons: vec![membrain_core::engine::result::ResultReason {
            memory_id: None,
            reason_code: "no_match".to_string(),
            detail: "no candidates found".to_string(),
        }],
    };

    let result_set = RetrievalResultSet::empty(explain, ns("test"));

    assert_eq!(result_set.count(), 0);
    assert!(!result_set.truncated);
    assert!(result_set.top().is_none());
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
