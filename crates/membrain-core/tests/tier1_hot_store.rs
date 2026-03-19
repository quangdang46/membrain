use membrain_core::api::NamespaceId;
use membrain_core::observability::{Tier1LookupLane, Tier1LookupOutcome};
use membrain_core::store::hot::Tier1HotMetadataStore;
use membrain_core::types::{
    CanonicalMemoryType, FastPathRouteFamily, MemoryId, SessionId, Tier1HotRecord,
    Tier1PayloadState,
};

fn seed_record(
    namespace: &str,
    memory_id: u64,
    session_id: u64,
    compact_text: &str,
    payload_size_bytes: usize,
) -> Tier1HotRecord {
    Tier1HotRecord::metadata_only(
        NamespaceId::new(namespace).unwrap(),
        MemoryId(memory_id),
        SessionId(session_id),
        CanonicalMemoryType::Event,
        FastPathRouteFamily::Event,
        compact_text,
        memory_id * 10,
        500,
        payload_size_bytes,
    )
}

#[test]
fn exact_lookup_hits_without_fetching_large_payloads() {
    let namespace = NamespaceId::new("tests/tier1").unwrap();
    let mut store = Tier1HotMetadataStore::new(4);
    store.seed(seed_record("tests/tier1", 1, 10, "recent note", 32_768));

    let exact = store.exact_lookup(&namespace, MemoryId(1));

    assert_eq!(exact.trace.lane, Tier1LookupLane::ExactHandle);
    assert_eq!(exact.trace.outcome, Tier1LookupOutcome::Hit);
    assert_eq!(exact.trace.payload_fetch_count, 0);
    assert_eq!(
        exact.record.as_ref().map(|record| record.payload_state),
        Some(Tier1PayloadState::MetadataOnly)
    );
    assert_eq!(
        exact
            .record
            .as_ref()
            .map(|record| record.payload_size_bytes),
        Some(32_768)
    );
}

#[test]
fn recent_window_stays_session_local_and_bounded() {
    let namespace = NamespaceId::new("tests/tier1").unwrap();
    let mut store = Tier1HotMetadataStore::new(5);
    store.seed(seed_record("tests/tier1", 1, 10, "session-a old", 4_096));
    store.seed(seed_record("tests/tier1", 2, 20, "session-b", 4_096));
    store.seed(seed_record("tests/tier1", 3, 10, "session-a new", 4_096));
    store.seed(seed_record("tests/tier1", 4, 10, "session-a newest", 4_096));

    let recent = store.recent_for_session(&namespace, SessionId(10), 2);

    assert_eq!(recent.trace.lane, Tier1LookupLane::RecentWindow);
    assert_eq!(recent.trace.outcome, Tier1LookupOutcome::Hit);
    assert!(recent.trace.session_window_hit);
    assert_eq!(recent.trace.payload_fetch_count, 0);
    assert_eq!(recent.trace.recent_candidates_inspected, 2);
    assert_eq!(recent.records.len(), 2);
    assert_eq!(recent.records[0].memory_id, MemoryId(4));
    assert_eq!(recent.records[1].memory_id, MemoryId(3));
}

#[test]
fn default_recent_window_scans_past_interleaved_foreign_session_entries() {
    let namespace = NamespaceId::new("tests/tier1").unwrap();
    let mut store = Tier1HotMetadataStore::new(4);
    store.seed(seed_record("tests/tier1", 1, 10, "target older", 4_096));
    store.seed(seed_record("tests/tier1", 2, 20, "foreign newer", 4_096));
    store.seed(seed_record("tests/tier1", 3, 10, "target newest", 4_096));
    store.seed(seed_record("tests/tier1", 4, 30, "foreign newest-most", 4_096));

    let recent = store.recent_for_session(&namespace, SessionId(10), 2);

    assert_eq!(recent.trace.lane, Tier1LookupLane::RecentWindow);
    assert_eq!(recent.trace.outcome, Tier1LookupOutcome::Hit);
    assert!(recent.trace.session_window_hit);
    assert_eq!(recent.trace.payload_fetch_count, 0);
    assert_eq!(recent.trace.recent_candidates_inspected, 4);
    assert_eq!(recent.records.len(), 2);
    assert_eq!(recent.records[0].memory_id, MemoryId(3));
    assert_eq!(recent.records[1].memory_id, MemoryId(1));
}

#[test]
fn bounded_capacity_evicts_the_oldest_hot_metadata_entry() {
    let namespace = NamespaceId::new("tests/tier1").unwrap();
    let mut store = Tier1HotMetadataStore::new(2);
    store.seed(seed_record("tests/tier1", 1, 10, "oldest", 8_192));
    store.seed(seed_record("tests/tier1", 2, 10, "middle", 8_192));
    store.seed(seed_record("tests/tier1", 3, 10, "newest", 8_192));

    let oldest = store.exact_lookup(&namespace, MemoryId(1));
    let newest = store.exact_lookup(&namespace, MemoryId(3));

    assert_eq!(store.len(), 2);
    assert_eq!(oldest.trace.outcome, Tier1LookupOutcome::Miss);
    assert_eq!(newest.trace.outcome, Tier1LookupOutcome::Hit);
}

#[test]
fn exact_lookup_does_not_cross_namespace_boundaries() {
    let alpha = NamespaceId::new("tests/alpha").unwrap();
    let mut store = Tier1HotMetadataStore::new(3);
    store.seed(seed_record("tests/alpha", 9, 10, "alpha payload", 8_192));
    store.seed(seed_record("tests/beta", 9, 10, "beta payload", 8_192));

    let exact = store.exact_lookup(&alpha, MemoryId(9));

    assert_eq!(exact.trace.outcome, Tier1LookupOutcome::Hit);
    assert_eq!(exact.record.as_ref().map(|record| record.namespace.as_str()), Some("tests/alpha"));
    assert_eq!(exact.record.as_ref().map(|record| record.compact_text.as_str()), Some("alpha payload"));
}

#[test]
fn recent_window_filters_out_colliding_foreign_namespace_entries() {
    let alpha = NamespaceId::new("tests/alpha").unwrap();
    let mut store = Tier1HotMetadataStore::new(4);
    store.seed(seed_record("tests/alpha", 1, 10, "alpha older", 4_096));
    store.seed(seed_record("tests/beta", 2, 10, "beta newer", 4_096));
    store.seed(seed_record("tests/alpha", 3, 10, "alpha newest", 4_096));
    store.seed(seed_record("tests/beta", 4, 10, "beta newest-most", 4_096));

    let recent = store.recent_for_session(&alpha, SessionId(10), 2);

    assert_eq!(recent.trace.outcome, Tier1LookupOutcome::Hit);
    assert_eq!(recent.trace.recent_candidates_inspected, 4);
    assert_eq!(recent.records.len(), 2);
    assert!(recent.records.iter().all(|record| record.namespace == alpha));
    assert_eq!(recent.records[0].memory_id, MemoryId(3));
    assert_eq!(recent.records[1].memory_id, MemoryId(1));
}
