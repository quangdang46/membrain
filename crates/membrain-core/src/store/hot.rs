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
    exact: HashMap<MemoryId, Tier1HotRecord>,
    recent: VecDeque<MemoryId>,
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
        let id = record.memory_id;
        if self.exact.contains_key(&id) {
            self.exact.insert(id, record);
            self.touch_recent(id);
            return;
        }

        if self.exact.len() >= self.capacity {
            if let Some(evicted) = self.recent.pop_front() {
                self.exact.remove(&evicted);
            }
        }

        self.exact.insert(id, record);
        self.recent.push_back(id);
    }

    /// Performs a bounded Tier1 exact lookup by stable memory id.
    pub fn exact_lookup(&self, memory_id: MemoryId) -> Tier1ExactLookup {
        let record = self.exact.get(&memory_id).cloned();
        Tier1ExactLookup {
            trace: Tier1LookupTrace {
                lane: Tier1LookupLane::ExactHandle,
                outcome: if record.is_some() {
                    Tier1LookupOutcome::Hit
                } else {
                    Tier1LookupOutcome::Miss
                },
                recent_candidates_inspected: 0,
                session_window_hit: false,
                payload_fetch_count: 0,
            },
            record,
        }
    }

    /// Performs a bounded recent-window scan for one session.
    pub fn recent_for_session(&self, session_id: SessionId, limit: usize) -> Tier1RecentLookup {
        let bounded_limit = limit.min(self.capacity);
        let mut inspected = 0usize;
        let mut records = Vec::new();

        for memory_id in self.recent.iter().rev() {
            let Some(record) = self.exact.get(memory_id) else {
                continue;
            };
            inspected += 1;
            if record.session_id == session_id {
                records.push(record.clone());
                if records.len() >= bounded_limit {
                    break;
                }
            }
        }

        Tier1RecentLookup {
            trace: Tier1LookupTrace {
                lane: Tier1LookupLane::RecentWindow,
                outcome: if records.is_empty() {
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

    fn touch_recent(&mut self, memory_id: MemoryId) {
        if let Some(position) = self
            .recent
            .iter()
            .position(|candidate| candidate == &memory_id)
        {
            self.recent.remove(position);
        }
        self.recent.push_back(memory_id);
    }
}
