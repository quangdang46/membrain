use membrain_core::api::{FieldPresence, NamespaceId, PolicyFilterSummary};
use membrain_core::engine::recall::{
    RecallEngine, RecallPlanKind, RecallRequest, RecallRuntime, RecallTraceStage,
};
use membrain_core::engine::result::{
    FreshnessMarkers, OmissionSummary, PackagingMetadata, PolicySummary, ProvenanceSummary,
    RetrievalExplain, RetrievalResultSet,
};
use membrain_core::engine::retrieval_planner::{
    ColdTierPolicy, ExplainLevel, GraphMode, QueryPath, RetrievalEffort, RetrievalPlanTrace,
    RetrievalRequest, RetrievalRequestValidationError,
};
use membrain_core::observability::{OutcomeClass, Tier1LookupLane, Tier1LookupOutcome, TraceStage};
use membrain_core::store::hot::Tier1HotMetadataStore;
use membrain_core::types::{
    CanonicalMemoryType, FastPathRouteFamily, MemoryId, SessionId, Tier1HotRecord,
};
use membrain_core::RuntimeConfig;

fn seed_record(memory_id: u64, session_id: u64, compact_text: &str) -> Tier1HotRecord {
    Tier1HotRecord::metadata_only(
        NamespaceId::new("tests/tier1").unwrap(),
        MemoryId(memory_id),
        SessionId(session_id),
        CanonicalMemoryType::Event,
        FastPathRouteFamily::Event,
        compact_text,
        memory_id * 10,
        500,
        4_096,
    )
}

fn temporal_snapshot_trace_request() -> RetrievalRequest {
    let namespace = NamespaceId::new("team.gamma").unwrap();
    RetrievalRequest {
        query_path: QueryPath::Temporal,
        ..RetrievalRequest::hybrid(namespace, "recent deploys", 8)
            .with_context_text("incident triage")
            .with_session(SessionId(33))
            .with_as_of_tick_range(40, 75)
            .with_snapshot_name("incident_baseline")
            .with_era_id("era-incident-baseline")
            .with_min_confidence(650)
            .with_min_strength(500)
            .with_show_decaying(true)
            .with_include_public(true)
            .with_graph_mode(GraphMode::Expand)
            .with_cold_tier_policy(ColdTierPolicy::Allow)
            .with_explain_level(ExplainLevel::Full)
            .with_effort(RetrievalEffort::High)
            .with_token_budget(256)
            .with_time_budget_ms(1200)
            .with_budget(77)
    }
}

fn recent_tier1_result_set() -> RetrievalResultSet {
    RetrievalResultSet {
        outcome_class: OutcomeClass::Accepted,
        evidence_pack: Vec::new(),
        action_pack: None,
        deferred_payloads: Vec::new(),
        explain: RetrievalExplain {
            recall_plan: RecallPlanKind::RecentTier1ThenTier2Exact,
            route_reason:
                "small lookup for active session can stay on hot recent window before durable fallback"
                    .to_string(),
            tiers_consulted: vec!["tier1_recent".to_string(), "tier2_exact".to_string()],
            trace_stages: vec![RecallTraceStage::Tier1RecentWindow, RecallTraceStage::Tier2Exact],
            tier1_answered_directly: false,
            candidate_budget: 8,
            time_consumed_ms: Some(12),
            ranking_profile: "balanced".to_string(),
            contradictions_found: 0,
            historical_context: None,
            query_by_example: None,
            result_reasons: Vec::new(),
        },
        policy_summary: PolicySummary {
            namespace_applied: NamespaceId::new("team.gamma").unwrap(),
            outcome_class: OutcomeClass::Accepted,
            redactions_applied: false,
            restrictions_active: Vec::new(),
            filters: vec![PolicyFilterSummary::new(
                "team.gamma",
                "namespace",
                OutcomeClass::Accepted,
                "not_blocked",
                FieldPresence::Present("same_namespace".to_string()),
                FieldPresence::Absent,
                Vec::new(),
            )],
        },
        provenance_summary: ProvenanceSummary {
            source_kind: "retrieval_pipeline".to_string(),
            source_reference: "result_set".to_string(),
            source_agent: "core_engine".to_string(),
            original_namespace: NamespaceId::new("team.gamma").unwrap(),
            derived_from: None,
            lineage_ancestors: vec![MemoryId(11), MemoryId(17)],
            relation_to_seed: None,
            graph_seed: None,
        },
        omitted_summary: OmissionSummary {
            policy_redacted: 0,
            threshold_dropped: 0,
            dedup_dropped: 0,
            budget_capped: 0,
            duplicate_collapsed: 0,
            low_confidence_suppressed: 0,
            stale_bypassed: 0,
            confidence_filtered: 0,
        },
        freshness_markers: FreshnessMarkers {
            oldest_item_days: 1,
            newest_item_days: 0,
            volatile_items_included: false,
            stale_warning: false,
            lease_sensitive: false,
            recheck_required: false,
            as_of_tick: Some(42),
            durable_lifecycle_state: None,
            routing_lifecycle_state: None,
        },
        packaging_metadata: PackagingMetadata {
            result_budget: 5,
            token_budget: Some(256),
            graph_assistance: "none".to_string(),
            degraded_summary: None,
            packaging_mode: "evidence_only".to_string(),
            rerank_metadata: None,
        },
        output_mode: membrain_core::engine::result::DualOutputMode::Balanced,
        truncated: false,
        total_candidates: 8,
    }
}

#[test]
fn exact_lookup_respects_zero_candidate_budget() {
    let namespace = NamespaceId::new("tests/tier1").unwrap();
    let mut store = Tier1HotMetadataStore::new(4);
    store.seed(seed_record(1, 10, "recent note"));

    let exact = store.exact_lookup_with_budget(&namespace, MemoryId(1), 0);

    assert!(exact.record.is_none());
    assert_eq!(exact.trace.lane, Tier1LookupLane::ExactHandle);
    assert_eq!(exact.trace.outcome, Tier1LookupOutcome::Bypass);
    assert_eq!(exact.trace.recent_candidates_inspected, 0);
    assert_eq!(exact.trace.payload_fetch_count, 0);
}

#[test]
fn exact_lookup_reports_one_candidate_when_budget_allows_probe() {
    let namespace = NamespaceId::new("tests/tier1").unwrap();
    let mut store = Tier1HotMetadataStore::new(4);
    store.seed(seed_record(1, 10, "recent note"));

    let exact = store.exact_lookup_with_budget(&namespace, MemoryId(1), 1);

    assert_eq!(exact.trace.outcome, Tier1LookupOutcome::Hit);
    assert_eq!(exact.trace.recent_candidates_inspected, 1);
    assert_eq!(exact.trace.payload_fetch_count, 0);
}

#[test]
fn recent_lookup_respects_candidate_budget_before_finding_session_hit() {
    let namespace = NamespaceId::new("tests/tier1").unwrap();
    let mut store = Tier1HotMetadataStore::new(5);
    store.seed(seed_record(1, 10, "session-a old"));
    store.seed(seed_record(2, 10, "session-a new"));
    store.seed(seed_record(3, 20, "session-b newest"));

    let recent = store.recent_for_session_with_budget(&namespace, SessionId(10), 2, 1);

    assert!(recent.records.is_empty());
    assert_eq!(recent.trace.lane, Tier1LookupLane::RecentWindow);
    assert_eq!(recent.trace.outcome, Tier1LookupOutcome::Miss);
    assert_eq!(recent.trace.recent_candidates_inspected, 1);
    assert!(!recent.trace.session_window_hit);
    assert_eq!(recent.trace.payload_fetch_count, 0);
}

#[test]
fn recent_lookup_zero_budget_bypasses_tier1_scan() {
    let namespace = NamespaceId::new("tests/tier1").unwrap();
    let mut store = Tier1HotMetadataStore::new(5);
    store.seed(seed_record(1, 10, "session-a old"));
    store.seed(seed_record(2, 10, "session-a new"));

    let recent = store.recent_for_session_with_budget(&namespace, SessionId(10), 2, 0);

    assert!(recent.records.is_empty());
    assert_eq!(recent.trace.lane, Tier1LookupLane::RecentWindow);
    assert_eq!(recent.trace.outcome, Tier1LookupOutcome::Bypass);
    assert_eq!(recent.trace.recent_candidates_inspected, 0);
    assert!(!recent.trace.session_window_hit);
    assert_eq!(recent.trace.payload_fetch_count, 0);
}

#[test]
fn planner_trace_names_direct_tier1_route_and_latency_evidence() {
    let engine = RecallEngine;

    let plan = engine.plan_recall(RecallRequest::exact(MemoryId(7)), RuntimeConfig::default());

    assert_eq!(plan.trace.route_name, "tier1.exact_handle");
    assert!(plan.trace.tier1_answered_directly);
    assert!(plan.trace.stayed_within_latency_budget);
    assert_eq!(
        plan.trace.candidate_budget,
        RuntimeConfig::default().tier1_candidate_budget
    );
    assert_eq!(plan.trace.pre_tier1_candidates, 1);
    assert_eq!(plan.trace.post_tier1_candidates, 1);
}

#[test]
fn tier1_trace_consumers_can_rely_on_stable_machine_labels() {
    assert_eq!(Tier1LookupLane::ExactHandle.as_str(), "exact_handle");
    assert_eq!(Tier1LookupLane::RecentWindow.as_str(), "recent_window");
    assert_eq!(Tier1LookupOutcome::Hit.as_str(), "hit");
    assert_eq!(Tier1LookupOutcome::Miss.as_str(), "miss");
    assert_eq!(Tier1LookupOutcome::Bypass.as_str(), "bypass");
    assert_eq!(Tier1LookupOutcome::StaleBypass.as_str(), "stale_bypass");

    let engine = RecallEngine;
    let direct_plan =
        engine.plan_recall(RecallRequest::exact(MemoryId(7)), RuntimeConfig::default());
    let fallback_plan = engine.plan_recall(
        RecallRequest::small_session_lookup(SessionId(11)),
        RuntimeConfig::default(),
    );

    assert_eq!(
        direct_plan.trace.route_name,
        format!("tier1.{}", Tier1LookupLane::ExactHandle.as_str())
    );
    assert_eq!(
        fallback_plan.trace.route_name,
        format!(
            "tier1.{}_then_tier2_exact",
            Tier1LookupLane::RecentWindow.as_str()
        )
    );
}

#[test]
fn planner_trace_names_fallback_route_and_preserves_candidate_budget_evidence() {
    let engine = RecallEngine;

    let recent_plan = engine.plan_recall(
        RecallRequest::small_session_lookup(SessionId(11)),
        RuntimeConfig::default(),
    );
    let fallback_plan = engine.plan_recall(
        RecallRequest {
            exact_memory_id: None,
            session_id: Some(SessionId(12)),
            small_lookup: false,
            graph_expansion: false,
            predictive_preroll: false,
        },
        RuntimeConfig::default(),
    );

    assert_eq!(
        recent_plan.trace.route_name,
        "tier1.recent_window_then_tier2_exact"
    );
    assert!(!recent_plan.trace.tier1_answered_directly);
    assert!(recent_plan.trace.stayed_within_latency_budget);
    assert_eq!(
        recent_plan.trace.candidate_budget,
        RuntimeConfig::default().tier1_candidate_budget,
    );
    assert_eq!(
        recent_plan.trace.pre_tier1_candidates,
        RuntimeConfig::default().tier1_candidate_budget,
    );
    assert_eq!(
        recent_plan.trace.post_tier1_candidates,
        RuntimeConfig::default().tier1_candidate_budget,
    );

    assert_eq!(
        fallback_plan.trace.route_name,
        "tier2.exact_then_tier3_fallback"
    );
    assert!(!fallback_plan.trace.tier1_answered_directly);
    assert!(fallback_plan.trace.stayed_within_latency_budget);
    assert_eq!(fallback_plan.trace.pre_tier1_candidates, 0);
    assert_eq!(fallback_plan.trace.post_tier1_candidates, 0);
}

#[test]
fn tier1_trace_consumer_budget_evidence_stays_explicit_across_routes() {
    let engine = RecallEngine;
    let config = RuntimeConfig::default();

    let exact_plan = engine.plan_recall(RecallRequest::exact(MemoryId(7)), config);
    let recent_plan =
        engine.plan_recall(RecallRequest::small_session_lookup(SessionId(11)), config);
    let fallback_plan = engine.plan_recall(
        RecallRequest {
            exact_memory_id: None,
            session_id: Some(SessionId(12)),
            small_lookup: false,
            graph_expansion: false,
            predictive_preroll: false,
        },
        config,
    );

    assert_eq!(
        exact_plan.tier1_candidate_budget,
        config.tier1_candidate_budget
    );
    assert_eq!(
        exact_plan.trace.candidate_budget,
        exact_plan.tier1_candidate_budget
    );
    assert_eq!(exact_plan.trace.pre_tier1_candidates, 1);
    assert_eq!(exact_plan.trace.post_tier1_candidates, 1);

    assert_eq!(
        recent_plan.tier1_candidate_budget,
        config.tier1_candidate_budget
    );
    assert_eq!(
        recent_plan.trace.candidate_budget,
        recent_plan.tier1_candidate_budget
    );
    assert_eq!(
        recent_plan.trace.pre_tier1_candidates,
        recent_plan.tier1_candidate_budget
    );
    assert_eq!(
        recent_plan.trace.post_tier1_candidates,
        recent_plan.tier1_candidate_budget
    );

    assert_eq!(
        fallback_plan.tier1_candidate_budget,
        config.tier1_candidate_budget
    );
    assert_eq!(
        fallback_plan.trace.candidate_budget,
        fallback_plan.tier1_candidate_budget
    );
    assert_eq!(fallback_plan.trace.pre_tier1_candidates, 0);
    assert_eq!(fallback_plan.trace.post_tier1_candidates, 0);
}

#[test]
fn retrieval_planner_trace_preserves_temporal_tick_window_and_snapshot_anchor() {
    let request = temporal_snapshot_trace_request();
    let trace = RetrievalPlanTrace::new(&request);

    assert!(trace.lexical_query.is_none());
    assert_eq!(trace.context_text.as_deref(), Some("incident triage"));
    assert_eq!(trace.as_of_tick, Some(75));
    assert_eq!(trace.era_id.as_deref(), Some("era-incident-baseline"));
    assert_eq!(trace.min_confidence, Some(650));
    assert_eq!(trace.min_strength, Some(500));
    assert!(trace.show_decaying);
    assert!(trace.include_public);
    assert_eq!(trace.graph_mode, GraphMode::Expand);
    assert_eq!(trace.cold_tier_policy, ColdTierPolicy::Allow);
    assert_eq!(trace.explain_level, ExplainLevel::Full);
    assert_eq!(trace.effort, RetrievalEffort::High);
    assert_eq!(trace.token_budget, Some(256));
    assert_eq!(trace.time_budget_ms, Some(1200));
    assert_eq!(trace.snapshot_name.as_deref(), Some("incident_baseline"));
    let historical = trace.historical_lookup.as_ref().expect("historical lookup");
    assert_eq!(historical.window_kind.as_str(), "tick_window");
    assert_eq!(
        historical.selection_reason,
        "explicit_snapshot_and_tick_window"
    );
    assert_eq!(historical.selected_tick_window, Some((40, 75)));
    assert_eq!(historical.snapshot_as_of_tick, Some(75));
    let temporal = trace.temporal_query.expect("temporal query");
    assert_eq!(temporal.session_filter, Some(SessionId(33)));
    assert_eq!(temporal.tick_range, Some((40, 75)));
    assert_eq!(temporal.candidate_budget, 77);
}

#[test]
fn exact_id_trace_rejects_query_by_example_seed_mix() {
    let namespace = NamespaceId::new("team.gamma").unwrap();
    let request =
        RetrievalRequest::exact_id(namespace, MemoryId(91)).with_like_memory(MemoryId(17));

    let error = request.normalize_query_by_example().unwrap_err();

    assert_eq!(
        error,
        RetrievalRequestValidationError::ExactIdWithExampleCue
    );
    assert_eq!(error.as_str(), "exact_id_with_example_cue");
}

#[test]
fn retrieval_result_envelope_projects_stable_route_policy_and_provenance_labels() {
    let result_set = recent_tier1_result_set();

    let (route, stages) = result_set.explain_route();
    let reasons = result_set.explain_result_reasons();
    let (policy, provenance) = result_set.explain_policy_and_provenance();

    assert_eq!(route.route_family, "recent_tier1_then_tier2_exact");
    assert_eq!(route.route_reason, "small_session_lookup");
    assert!(route.tier1_consulted_first);
    assert_eq!(route.pre_route_candidate_count, Some(8));
    assert_eq!(route.post_route_candidate_count, Some(0));
    assert!(!route.predictive_triggered);
    assert_eq!(route.predictive_skip_reason, None);
    assert_eq!(route.predictive_fallback_behavior, None);
    assert_eq!(
        stages,
        vec![
            TraceStage::Tier1RecentWindow,
            TraceStage::Tier2Exact,
            TraceStage::PolicyGate,
            TraceStage::Packaging,
        ]
    );

    assert_eq!(policy.effective_namespace, "team.gamma");
    assert_eq!(policy.policy_family, "namespace");
    assert_eq!(policy.blocked_stage, "not_blocked");
    assert_eq!(policy.sharing_scope, "same_namespace");

    assert_eq!(provenance.source_kind, "retrieval_pipeline");
    assert_eq!(provenance.source_reference, "result_set");
    assert_eq!(provenance.lineage_ancestors, vec![11, 17]);
    assert!(reasons.is_empty());
}

#[test]
fn retrieval_result_json_preserves_stale_bypass_omission_counts() {
    let mut result_set = recent_tier1_result_set();
    result_set.omitted_summary.stale_bypassed = 2;

    let encoded = result_set.to_json().unwrap();
    let decoded = RetrievalResultSet::from_json(&encoded).unwrap();

    assert_eq!(decoded.omitted_summary.stale_bypassed, 2);
    assert_eq!(
        decoded.explain.trace_stages,
        result_set.explain.trace_stages
    );
}

#[test]
fn explain_route_appends_packaging_stage_after_policy_gate_once() {
    let mut result_set = recent_tier1_result_set();
    result_set.explain.trace_stages = vec![RecallTraceStage::Tier1RecentWindow];

    let (_, stages) = result_set.explain_route();

    assert_eq!(
        stages,
        vec![
            TraceStage::Tier1RecentWindow,
            TraceStage::PolicyGate,
            TraceStage::Packaging,
        ]
    );
    assert_eq!(
        stages
            .iter()
            .filter(|stage| **stage == TraceStage::Packaging)
            .count(),
        1
    );
}

#[test]
fn explain_route_keeps_existing_policy_gate_without_duplication() {
    let mut result_set = recent_tier1_result_set();
    result_set.explain.trace_stages = vec![
        RecallTraceStage::Tier1RecentWindow,
        RecallTraceStage::Tier2Exact,
    ];

    let (_, stages) = result_set.explain_route();

    assert_eq!(
        stages,
        vec![
            TraceStage::Tier1RecentWindow,
            TraceStage::Tier2Exact,
            TraceStage::PolicyGate,
            TraceStage::Packaging,
        ]
    );
    assert_eq!(
        stages
            .iter()
            .filter(|stage| **stage == TraceStage::PolicyGate)
            .count(),
        1
    );
    assert_eq!(
        stages
            .iter()
            .filter(|stage| **stage == TraceStage::Packaging)
            .count(),
        1
    );
}
