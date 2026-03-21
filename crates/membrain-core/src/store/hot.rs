use crate::api::NamespaceId;
use crate::observability::{Tier1LookupLane, Tier1LookupOutcome, Tier1LookupTrace};
use crate::store::HotStoreApi;
use crate::types::{MemoryId, SessionId, Tier1HotRecord};
use std::collections::{HashMap, VecDeque};

/// Namespace-aware Tier1 metadata store boundary owned by `membrain-core`.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct HotStore;

impl HotStore {
    /// Builds a bounded in-process Tier1 metadata store for exact and session-local lookups.
    pub fn new_metadata_store(&self, capacity: usize) -> Tier1HotMetadataStore {
        Tier1HotMetadataStore::new(capacity)
    }
}

impl HotStoreApi for HotStore {
    fn component_name(&self) -> &'static str {
        "store.hot"
    }
}

/// Exact Tier1 lookup result with machine-readable routing evidence.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Tier1ExactLookup {
    pub record: Option<Tier1HotRecord>,
    pub trace: Tier1LookupTrace,
}

/// Session-window Tier1 lookup result with machine-readable routing evidence.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Tier1RecentLookup {
    pub records: Vec<Tier1HotRecord>,
    pub trace: Tier1LookupTrace,
}

/// Bounded Tier1 hot metadata store for exact and recent lookups.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Tier1HotMetadataStore {
    capacity: usize,
    exact: HashMap<(NamespaceId, MemoryId), Tier1HotRecord>,
    recent: VecDeque<(NamespaceId, MemoryId)>,
}

impl Tier1HotMetadataStore {
    /// Builds a new bounded Tier1 metadata store.
    pub fn new(capacity: usize) -> Self {
        Self {
            capacity: capacity.max(1),
            exact: HashMap::new(),
            recent: VecDeque::new(),
        }
    }

    /// Returns the number of resident Tier1 metadata entries.
    pub fn len(&self) -> usize {
        self.exact.len()
    }

    /// Returns whether the Tier1 metadata store is empty.
    pub fn is_empty(&self) -> bool {
        self.exact.is_empty()
    }

    /// Seeds or refreshes one hot metadata record without loading a heavyweight payload.
    pub fn seed(&mut self, record: Tier1HotRecord) {
        let key = (record.namespace.clone(), record.memory_id);
        if self.exact.contains_key(&key) {
            self.exact.insert(key.clone(), record);
            self.touch_recent(key);
            return;
        }

        while self.exact.len() >= self.capacity {
            let Some(evicted) = self.recent.pop_front() else {
                break;
            };
            if self.exact.remove(&evicted).is_some() {
                break;
            }
        }

        self.exact.insert(key.clone(), record);
        self.recent.push_back(key);
    }

    /// Performs a bounded Tier1 exact lookup by stable namespace and memory id.
    pub fn exact_lookup(
        &mut self,
        namespace: &NamespaceId,
        memory_id: MemoryId,
    ) -> Tier1ExactLookup {
        self.exact_lookup_with_budget(namespace, memory_id, 1)
    }

    /// Performs a bounded Tier1 exact lookup by stable namespace and memory id with an explicit candidate budget.
    pub fn exact_lookup_with_budget(
        &mut self,
        namespace: &NamespaceId,
        memory_id: MemoryId,
        candidate_budget: usize,
    ) -> Tier1ExactLookup {
        if candidate_budget == 0 {
            return Tier1ExactLookup {
                record: None,
                trace: Tier1LookupTrace {
                    lane: Tier1LookupLane::ExactHandle,
                    outcome: Tier1LookupOutcome::Bypass,
                    recent_candidates_inspected: 0,
                    session_window_hit: false,
                    payload_fetch_count: 0,
                },
            };
        }

        let key = (namespace.clone(), memory_id);
        let record = self.exact.get(&key).cloned();
        if record.is_some() {
            self.touch_recent(key);
        }
        Tier1ExactLookup {
            trace: Tier1LookupTrace {
                lane: Tier1LookupLane::ExactHandle,
                outcome: if record.is_some() {
                    Tier1LookupOutcome::Hit
                } else {
                    Tier1LookupOutcome::Miss
                },
                recent_candidates_inspected: 1,
                session_window_hit: false,
                payload_fetch_count: 0,
            },
            record,
        }
    }

    /// Performs a bounded recent-window scan for one namespace-local session.
    pub fn recent_for_session(
        &self,
        namespace: &NamespaceId,
        session_id: SessionId,
        limit: usize,
    ) -> Tier1RecentLookup {
        self.recent_for_session_with_budget(namespace, session_id, limit, self.capacity)
    }

    /// Performs a bounded recent-window scan for one namespace-local session with an explicit candidate budget.
    pub fn recent_for_session_with_budget(
        &self,
        namespace: &NamespaceId,
        session_id: SessionId,
        limit: usize,
        candidate_budget: usize,
    ) -> Tier1RecentLookup {
        let bounded_limit = limit.min(self.capacity).min(candidate_budget);
        let mut inspected = 0usize;
        let mut records = Vec::new();
        let mut stale_candidates_skipped = 0usize;

        if bounded_limit == 0 {
            return Tier1RecentLookup {
                trace: Tier1LookupTrace {
                    lane: Tier1LookupLane::RecentWindow,
                    outcome: Tier1LookupOutcome::Bypass,
                    recent_candidates_inspected: 0,
                    session_window_hit: false,
                    payload_fetch_count: 0,
                },
                records,
            };
        }

        for key in self.recent.iter().rev() {
            if inspected >= candidate_budget {
                break;
            }

            let Some(record) = self.exact.get(key) else {
                stale_candidates_skipped += 1;
                continue;
            };
            inspected += 1;
            if &record.namespace == namespace && record.session_id == session_id {
                records.push(record.clone());
                if records.len() >= bounded_limit {
                    break;
                }
            }
        }

        Tier1RecentLookup {
            trace: Tier1LookupTrace {
                lane: Tier1LookupLane::RecentWindow,
                outcome: if records.is_empty() && inspected == 0 && stale_candidates_skipped > 0 {
                    Tier1LookupOutcome::StaleBypass
                } else if records.is_empty() {
                    Tier1LookupOutcome::Miss
                } else {
                    Tier1LookupOutcome::Hit
                },
                recent_candidates_inspected: inspected,
                session_window_hit: !records.is_empty(),
                payload_fetch_count: 0,
            },
            records,
        }
    }

    fn touch_recent(&mut self, key: (NamespaceId, MemoryId)) {
        if let Some(position) = self.recent.iter().position(|candidate| candidate == &key) {
            self.recent.remove(position);
        }
        self.recent.push_back(key);
    }
}

#[cfg(test)]
impl Tier1HotMetadataStore {
    fn inject_stale_recent_reference(&mut self, namespace: &str, memory_id: u64) {
        self.recent
            .push_back((NamespaceId::new(namespace).unwrap(), MemoryId(memory_id)));
    }
}

#[cfg(test)]
mod tests {
    use super::Tier1HotMetadataStore;
    use crate::api::NamespaceId;
    use crate::observability::Tier1LookupOutcome;
    use crate::types::{
        CanonicalMemoryType, FastPathRouteFamily, MemoryId, SessionId, Tier1HotRecord,
    };

    fn seed_record(
        namespace: &str,
        memory_id: u64,
        session_id: u64,
        compact_text: &str,
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
            4_096,
        )
    }

    #[test]
    fn exact_lookup_refreshes_recency_before_eviction() {
        let namespace = NamespaceId::new("team.alpha").unwrap();
        let mut store = Tier1HotMetadataStore::new(2);
        store.seed(seed_record("team.alpha", 1, 10, "older"));
        store.seed(seed_record("team.alpha", 2, 10, "newer"));

        let exact = store.exact_lookup(&namespace, MemoryId(1));
        store.seed(seed_record("team.alpha", 3, 10, "newest"));

        assert_eq!(
            exact.record.as_ref().map(|record| record.memory_id),
            Some(MemoryId(1))
        );
        assert!(store.exact_lookup(&namespace, MemoryId(1)).record.is_some());
        assert!(store.exact_lookup(&namespace, MemoryId(2)).record.is_none());
        assert!(store.exact_lookup(&namespace, MemoryId(3)).record.is_some());
    }

    #[test]
    fn default_recent_lookup_scans_past_interleaved_foreign_session_entries() {
        let namespace = NamespaceId::new("team.alpha").unwrap();
        let mut store = Tier1HotMetadataStore::new(4);
        store.seed(seed_record("team.alpha", 1, 10, "target older"));
        store.seed(seed_record("team.alpha", 2, 20, "foreign newest"));
        store.seed(seed_record("team.alpha", 3, 10, "target newer"));
        store.seed(seed_record("team.alpha", 4, 30, "foreign newest-most"));

        let recent = store.recent_for_session(&namespace, SessionId(10), 2);

        assert_eq!(recent.records.len(), 2);
        assert_eq!(recent.records[0].memory_id, MemoryId(3));
        assert_eq!(recent.records[1].memory_id, MemoryId(1));
        assert_eq!(recent.trace.recent_candidates_inspected, 4);
        assert!(recent.trace.session_window_hit);
    }

    #[test]
    fn exact_lookup_does_not_cross_namespace_on_colliding_memory_ids() {
        let namespace = NamespaceId::new("team.alpha").unwrap();
        let mut store = Tier1HotMetadataStore::new(3);
        store.seed(seed_record("team.alpha", 7, 10, "alpha record"));
        store.seed(seed_record("team.beta", 7, 10, "beta record"));

        let exact = store.exact_lookup(&namespace, MemoryId(7));

        assert_eq!(
            exact
                .record
                .as_ref()
                .map(|record| record.compact_text.as_str()),
            Some("alpha record")
        );
        assert_eq!(
            exact
                .record
                .as_ref()
                .map(|record| record.namespace.as_str()),
            Some("team.alpha")
        );
    }

    #[test]
    fn recent_lookup_only_returns_records_for_the_requested_namespace() {
        let namespace = NamespaceId::new("team.alpha").unwrap();
        let mut store = Tier1HotMetadataStore::new(4);
        store.seed(seed_record("team.alpha", 1, 10, "alpha older"));
        store.seed(seed_record("team.beta", 2, 10, "beta newer"));
        store.seed(seed_record("team.alpha", 3, 10, "alpha newest"));
        store.seed(seed_record("team.beta", 4, 10, "beta newest-most"));

        let recent = store.recent_for_session(&namespace, SessionId(10), 2);

        assert_eq!(recent.records.len(), 2);
        assert!(recent
            .records
            .iter()
            .all(|record| record.namespace == namespace));
        assert_eq!(recent.records[0].memory_id, MemoryId(3));
        assert_eq!(recent.records[1].memory_id, MemoryId(1));
        assert_eq!(recent.trace.recent_candidates_inspected, 4);
    }

    #[test]
    fn recent_lookup_reports_stale_bypass_when_only_stale_recency_entries_remain() {
        let namespace = NamespaceId::new("team.alpha").unwrap();
        let mut store = Tier1HotMetadataStore::new(2);
        store.seed(seed_record("team.alpha", 1, 10, "older"));
        store.seed(seed_record("team.alpha", 2, 10, "newer"));
        store.seed(seed_record("team.alpha", 3, 20, "evicts older"));

        store.inject_stale_recent_reference("team.alpha", 99);

        let recent = store.recent_for_session(&namespace, SessionId(30), 1);

        assert!(recent.records.is_empty());
        assert_eq!(recent.trace.outcome, Tier1LookupOutcome::StaleBypass);
        assert!(!recent.trace.session_window_hit);
        assert_eq!(recent.trace.recent_candidates_inspected, 2);
    }

    #[test]
    fn recent_lookup_reports_miss_when_stale_entries_exist_but_live_candidates_were_checked() {
        let namespace = NamespaceId::new("team.alpha").unwrap();
        let mut store = Tier1HotMetadataStore::new(2);
        store.seed(seed_record("team.alpha", 1, 10, "older"));
        store.seed(seed_record("team.alpha", 2, 20, "newer foreign session"));
        store.seed(seed_record("team.alpha", 3, 20, "evicts older"));

        store.inject_stale_recent_reference("team.alpha", 99);

        let recent = store.recent_for_session(&namespace, SessionId(30), 1);

        assert!(recent.records.is_empty());
        assert_eq!(recent.trace.outcome, Tier1LookupOutcome::Miss);
        assert!(!recent.trace.session_window_hit);
        assert_eq!(recent.trace.recent_candidates_inspected, 2);
    }

    #[test]
    fn recent_lookup_keeps_hit_outcome_when_stale_entries_exist_but_valid_records_answer() {
        let namespace = NamespaceId::new("team.alpha").unwrap();
        let mut store = Tier1HotMetadataStore::new(2);
        store.seed(seed_record("team.alpha", 1, 10, "older"));
        store.seed(seed_record("team.alpha", 2, 10, "newer"));
        store.seed(seed_record("team.alpha", 3, 10, "evicts older"));

        store.inject_stale_recent_reference("team.alpha", 99);

        let recent = store.recent_for_session(&namespace, SessionId(10), 1);

        assert_eq!(recent.records.len(), 1);
        assert_eq!(recent.records[0].memory_id, MemoryId(3));
        assert_eq!(recent.trace.outcome, Tier1LookupOutcome::Hit);
        assert!(recent.trace.session_window_hit);
    }
}
