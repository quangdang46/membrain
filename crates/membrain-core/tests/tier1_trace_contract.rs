use membrain_core::api::NamespaceId;
use membrain_core::engine::recall::{RecallEngine, RecallRequest, RecallRuntime};
use membrain_core::observability::{Tier1LookupLane, Tier1LookupOutcome};
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
    assert_eq!(plan.trace.candidate_budget, RuntimeConfig::default().tier1_candidate_budget);
    assert_eq!(plan.trace.pre_tier1_candidates, 1);
    assert_eq!(plan.trace.post_tier1_candidates, 1);
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

    assert_eq!(fallback_plan.trace.route_name, "tier2.exact_then_tier3_fallback");
    assert!(!fallback_plan.trace.tier1_answered_directly);
    assert!(fallback_plan.trace.stayed_within_latency_budget);
    assert_eq!(fallback_plan.trace.pre_tier1_candidates, 0);
    assert_eq!(fallback_plan.trace.post_tier1_candidates, 0);
}
