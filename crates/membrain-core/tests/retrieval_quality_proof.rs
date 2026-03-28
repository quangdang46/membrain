use membrain_core::api::NamespaceId;
use membrain_core::embed::{
    CachedTextEmbedder, EmbedError, EmbeddingPurpose, LocalTextEmbedder,
};
use membrain_core::engine::ranking::{
    fuse_scores, RankingInput, RankingProfile, RankingResult, RerankMetadata,
};
use membrain_core::engine::recall::{RecallPlanKind, RecallTraceStage};
use membrain_core::engine::result::{
    AnsweredFrom, FreshnessMarkers, ResultBuilder, ResultReason, RetrievalExplain,
};
use membrain_core::engine::retrieval_planner::{PlannerStage, RetrievalPlan, RetrievalRequest};
use membrain_core::observability::{
    CacheEvalTrace, CacheEventLabel, CacheFamilyLabel, CacheLookupOutcome, CacheReasonLabel,
    GenerationStatusLabel,
};
use membrain_core::store::cache::{
    CacheFamily, CacheGenerationAnchors, CacheKey, CacheManager, InvalidationTrigger,
};
use membrain_core::types::{CanonicalMemoryType, MemoryId, SessionId};
use std::collections::HashSet;

fn ns(value: &str) -> NamespaceId {
    NamespaceId::new(value).expect("valid namespace")
}

#[derive(Debug, Clone)]
struct SemanticFixtureEmbedder {
    generation: String,
}

impl SemanticFixtureEmbedder {
    fn new(generation: &str) -> Self {
        Self {
            generation: generation.to_string(),
        }
    }

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
        &self.generation
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

fn lexical_overlap(query: &str, candidate: &str) -> usize {
    let query_terms: HashSet<String> = query
        .to_ascii_lowercase()
        .split_whitespace()
        .map(|term| term.trim_matches(|c: char| !c.is_alphanumeric()).to_string())
        .filter(|term| !term.is_empty())
        .collect();
    let candidate_terms: HashSet<String> = candidate
        .to_ascii_lowercase()
        .split_whitespace()
        .map(|term| term.trim_matches(|c: char| !c.is_alphanumeric()).to_string())
        .filter(|term| !term.is_empty())
        .collect();
    query_terms.intersection(&candidate_terms).count()
}

fn dot(left: &[f32], right: &[f32]) -> f32 {
    left.iter().zip(right.iter()).map(|(a, b)| a * b).sum()
}

#[test]
fn semantic_embeddings_beat_naive_compact_text_overlap_for_realistic_release_query() {
    let query = "Which release deploy fix should we roll out after the pipeline outage?";
    let lexical_winner = "release rollback checklist after outage";
    let semantic_winner = "deployment pipeline remediation for production rollout";

    let lexical_overlap_winner = lexical_overlap(query, lexical_winner);
    let lexical_overlap_semantic = lexical_overlap(query, semantic_winner);
    assert!(
        lexical_overlap_winner > lexical_overlap_semantic,
        "fixture must make naive compact-text matching prefer the wrong record"
    );

    let mut embedder = CachedTextEmbedder::new(SemanticFixtureEmbedder::new("fixture-v1"), 8);
    let query_embedding = embedder
        .get_or_embed(EmbeddingPurpose::Content, query)
        .expect("query embedding");
    let lexical_candidate = embedder
        .get_or_embed(EmbeddingPurpose::Content, lexical_winner)
        .expect("lexical candidate embedding");
    let semantic_candidate = embedder
        .get_or_embed(EmbeddingPurpose::Content, semantic_winner)
        .expect("semantic candidate embedding");

    let semantic_score_lexical = dot(&query_embedding.vector, &lexical_candidate.vector);
    let semantic_score_semantic = dot(&query_embedding.vector, &semantic_candidate.vector);
    assert!(
        semantic_score_semantic > semantic_score_lexical,
        "embedding-backed retrieval should prefer the deploy/remediation memory"
    );

    assert_eq!(query_embedding.trace.cache_event, membrain_core::embed::EmbedCacheEvent::Miss);
    assert_eq!(lexical_candidate.trace.generation, "fixture-v1");
    assert_eq!(semantic_candidate.trace.backend_kind, "semantic_fixture");
    assert!(query_embedding.trace.local_only);
    assert!(!query_embedding.trace.remote_fallback_used);
}

#[test]
fn reranking_changes_final_order_and_keeps_semantic_path_metadata_inspectable() {
    let namespace = ns("retrieval-quality/rerank");
    let mut builder = ResultBuilder::new(2, namespace.clone());

    let semantic_first = fuse_scores(
        RankingInput {
            recency: 350,
            salience: 720,
            strength: 680,
            provenance: 500,
            conflict: 500,
            confidence: 500,
        },
        RankingProfile::balanced(),
    );

    let mut reranked_winner: RankingResult = fuse_scores(
        RankingInput {
            recency: 950,
            salience: 620,
            strength: 640,
            provenance: 700,
            conflict: 500,
            confidence: 850,
        },
        RankingProfile::confidence_biased(),
    );
    reranked_winner.rerank_metadata = RerankMetadata {
        float32_rescore_limit: 12,
        candidate_cut_limit: 4,
        local_reranker_mode: membrain_core::engine::ranking::LocalRerankerMode::Bounded,
        local_reranker_applied: true,
        rerank_score_delta: 180,
    };

    builder.add(
        MemoryId(10),
        namespace.clone(),
        SessionId(1),
        CanonicalMemoryType::Event,
        "semantic shortlist winner before rerank".into(),
        &semantic_first,
        AnsweredFrom::Tier2Indexed,
    );
    builder.add(
        MemoryId(20),
        namespace.clone(),
        SessionId(1),
        CanonicalMemoryType::Event,
        "operationally safer reranked deployment fix".into(),
        &reranked_winner,
        AnsweredFrom::Tier2Indexed,
    );

    let result_set = builder.build(RetrievalExplain {
        recall_plan: RecallPlanKind::Tier2ExactThenTier3Fallback,
        route_reason: "semantic shortlist required rerank to surface the safer release fix".into(),
        tiers_consulted: vec!["tier2_semantic".into(), "tier2_rerank".into()],
        trace_stages: vec![RecallTraceStage::Tier2Exact, RecallTraceStage::Tier3Fallback],
        tier1_answered_directly: false,
        candidate_budget: 12,
        time_consumed_ms: Some(9),
        ranking_profile: "confidence_biased".into(),
        contradictions_found: 0,
        historical_context: None,
        query_by_example: None,
        result_reasons: vec![
            ResultReason {
                memory_id: Some(MemoryId(10)),
                reason_code: "semantic_shortlist_retained".into(),
                detail: "candidate survived semantic ANN shortlist but lacked the top rerank delta"
                    .into(),
            },
            ResultReason {
                memory_id: Some(MemoryId(20)),
                reason_code: "rerank_promoted".into(),
                detail: "bounded local reranker promoted the deployment fix after float32 rescore"
                    .into(),
            },
        ],
    });

    let top = result_set.top().expect("top result");
    assert_eq!(top.result.memory_id, MemoryId(20));
    assert_eq!(top.result.entry_lane.as_str(), "semantic");
    assert!(top.result.rerank_metadata.local_reranker_applied);
    assert_eq!(top.result.rerank_metadata.rerank_score_delta, 180);
    assert!(result_set.packaging_metadata.rerank_metadata.is_none());
    assert!(result_set.evidence_pack.iter().any(|item| {
        item.result.rerank_metadata.local_reranker_applied
            && item.result.rerank_metadata.candidate_cut_limit == 4
    }));
    assert!(result_set.explain.result_reasons.iter().any(|reason| {
        reason.reason_code == "rerank_promoted"
            && reason.detail.contains("bounded local reranker")
    }));
}

#[test]
fn result_set_lifecycle_markers_show_cold_and_reconsolidating_runtime_effects() {
    let namespace = ns("retrieval-quality/lifecycle");
    let mut builder = ResultBuilder::new(2, namespace.clone());

    let cold_ranked = fuse_scores(
        RankingInput {
            recency: 610,
            salience: 680,
            strength: 640,
            provenance: 620,
            conflict: 500,
            confidence: 720,
        },
        RankingProfile::balanced(),
    );
    builder.add_with_confidence(
        MemoryId(101),
        namespace.clone(),
        SessionId(1),
        CanonicalMemoryType::Event,
        "cold consolidated rollout fix".into(),
        &cold_ranked,
        AnsweredFrom::Tier3Cold,
        &membrain_core::engine::confidence::ConfidenceInputs {
            corroboration_count: 3,
            reconsolidation_count: 0,
            ticks_since_last_access: 2048,
            age_ticks: 4096,
            resolution_state: membrain_core::engine::contradiction::ResolutionState::None,
            conflict_score: 0,
            causal_parent_count: 1,
            authoritativeness: 820,
            recall_count: 1,
        },
        &membrain_core::engine::confidence::ConfidencePolicy::default(),
    );

    let volatile_ranked = fuse_scores(
        RankingInput {
            recency: 830,
            salience: 720,
            strength: 690,
            provenance: 640,
            conflict: 500,
            confidence: 610,
        },
        RankingProfile::balanced(),
    );
    builder.add_with_confidence(
        MemoryId(202),
        namespace.clone(),
        SessionId(1),
        CanonicalMemoryType::Observation,
        "fresh reconsolidating incident memory".into(),
        &volatile_ranked,
        AnsweredFrom::Tier2Indexed,
        &membrain_core::engine::confidence::ConfidenceInputs {
            corroboration_count: 1,
            reconsolidation_count: 9,
            ticks_since_last_access: 6,
            age_ticks: 24,
            resolution_state: membrain_core::engine::contradiction::ResolutionState::None,
            conflict_score: 0,
            causal_parent_count: 0,
            authoritativeness: 620,
            recall_count: 7,
        },
        &membrain_core::engine::confidence::ConfidencePolicy::default(),
    );

    let result_set = builder.build(RetrievalExplain {
        recall_plan: RecallPlanKind::Tier2ExactThenTier3Fallback,
        route_reason: "bounded runtime mixed hot semantic evidence with cold consolidated evidence".into(),
        tiers_consulted: vec!["tier2_exact".into(), "tier3_cold".into()],
        trace_stages: vec![RecallTraceStage::Tier2Exact, RecallTraceStage::Tier3Fallback],
        tier1_answered_directly: false,
        candidate_budget: 2,
        time_consumed_ms: Some(11),
        ranking_profile: "balanced".into(),
        contradictions_found: 0,
        historical_context: None,
        query_by_example: None,
        result_reasons: vec![],
    });

    let reasons = result_set.explain_result_reasons();
    assert!(reasons.iter().any(|reason| reason.reason_code == "cold_consolidated"));
    assert!(reasons
        .iter()
        .any(|reason| reason.reason_code == "reconsolidation_window_open"));

    let (freshness_markers, _conflict_markers, uncertainty_markers) = result_set.explain_markers();
    assert!(freshness_markers
        .iter()
        .any(|marker| marker.code == "lifecycle_projection"));
    assert!(!freshness_markers
        .iter()
        .any(|marker| marker.code == "fresh"));
    assert!(uncertainty_markers
        .iter()
        .any(|marker| marker.code == "reconsolidation_churn"));
    assert!(result_set.has_cold_consolidated_evidence());
    assert_eq!(result_set.freshness_markers, FreshnessMarkers {
        oldest_item_days: 0,
        newest_item_days: 0,
        volatile_items_included: true,
        stale_warning: true,
        lease_sensitive: false,
        recheck_required: false,
        as_of_tick: None,
        durable_lifecycle_state: Some("labile".to_string()),
        routing_lifecycle_state: Some("fresh".to_string()),
    });
}

#[test]
fn planner_and_cache_diagnostics_distinguish_semantic_path_from_persistence_only_fallback() {
    let namespace = ns("retrieval-quality/cache");
    let request = RetrievalRequest::hybrid(
        namespace.clone(),
        "how should we fix the release deploy regression?",
        3,
    );
    let plan = RetrievalPlan::new(request);
    assert!(plan.stages.contains(&PlannerStage::LexicalPrefilter));
    assert!(plan.stages.contains(&PlannerStage::HotSemanticSearch));
    assert!(plan.stages.contains(&PlannerStage::Float32Rescore));
    assert!(plan.stages.contains(&PlannerStage::Tier3ColdSearch));
    assert_eq!(plan.budget_for_stage(PlannerStage::Float32Rescore), Some(3));

    let mut cache = CacheManager::new(4, 2);
    let key = CacheKey {
        family: CacheFamily::AnnProbeCache,
        namespace: namespace.clone(),
        workspace_key: None,
        owner_key: None,
        request_shape_hash: Some(44),
        item_key: 7,
        generations: CacheGenerationAnchors::default(),
    };
    let admission = cache
        .ann_probe
        .admit(
            key.clone(),
            vec![MemoryId(101), MemoryId(202)],
            membrain_core::store::cache::CacheAdmissionRequest {
                policy_allowed: true,
                allow_capacity_eviction: true,
                request_shape_hash: Some(44),
            },
        );
    assert_eq!(admission.reason.as_str(), "accepted");

    let warm_lookup = cache
        .ann_probe
        .lookup(&key, &CacheGenerationAnchors::default());
    let warm_trace = membrain_core::store::cache::CacheTraceStage::from_lookup(
        "ann_probe_eval",
        &warm_lookup,
        8,
    )
    .to_observability_trace();
    assert_eq!(warm_trace.cache_event.as_str(), "hit");
    assert_eq!(warm_trace.cache_family.as_str(), "ann_probe_cache");
    assert!(warm_trace.warm_reuse);

    let invalidation = cache.handle_invalidation(InvalidationTrigger::EmbeddingChange, &namespace);
    assert!(
        invalidation
            .maintenance_events
            .iter()
            .any(|event| event.reason == Some(membrain_core::store::cache::CacheReason::EmbeddingChanged))
    );

    cache.result.disable();
    let degraded_trace = CacheEvalTrace {
        cache_family: CacheFamilyLabel::ResultCache,
        cache_event: CacheEventLabel::Disabled,
        outcome: CacheLookupOutcome::Disabled,
        cache_reason: Some(CacheReasonLabel::EmbeddingChanged),
        warm_source: None,
        generation_status: GenerationStatusLabel::Unknown,
        candidates_before: 2,
        candidates_after: 0,
        warm_reuse: false,
    };

    let cache_summary = membrain_core::api::CacheMetricsSummary::from_cache_traces(
        vec![warm_trace, degraded_trace],
        false,
    );
    assert_eq!(cache_summary.ann_probe_hit_count, 1);
    assert_eq!(cache_summary.cache_hit_count, 1);
    assert_eq!(cache_summary.cache_bypass_count, 1);
    assert!(cache_summary.degraded_mode_served);
    assert_eq!(cache_summary.post_ann_candidates, Some(2));
    assert!(cache_summary.cache_traces.iter().any(|trace| {
        trace.cache_reason == Some(CacheReasonLabel::EmbeddingChanged)
            && trace.cache_event == CacheEventLabel::Disabled
    }));
}
