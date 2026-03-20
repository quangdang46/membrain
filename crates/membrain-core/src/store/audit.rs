use crate::api::NamespaceId;
use crate::observability::{AuditEventCategory, AuditEventKind};
use crate::types::{MemoryId, SessionId};
use std::collections::VecDeque;

/// Canonical append-only audit log boundary owned by `membrain-core` storage.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct AuditLogStore;

impl AuditLogStore {
    /// Builds a bounded append-only audit log with a hard row cap.
    pub fn new_log(&self, capacity: usize) -> AppendOnlyAuditLog {
        AppendOnlyAuditLog::new(capacity)
    }
}

/// One durable audit-log entry preserved in append order.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AuditLogEntry {
    /// Monotonic sequence assigned at append time.
    pub sequence: u64,
    /// Canonical event category for high-level filtering.
    pub category: AuditEventCategory,
    /// Canonical event kind for exact filtering.
    pub kind: AuditEventKind,
    /// Namespace the event belongs to.
    pub namespace: NamespaceId,
    /// Optional memory identity linked to the event.
    pub memory_id: Option<MemoryId>,
    /// Optional session identity linked to the event.
    pub session_id: Option<SessionId>,
    /// Stable machine-readable actor source for correlation.
    pub actor_source: &'static str,
    /// Human-readable detail preserved for explain and export surfaces.
    pub detail: String,
}

impl AuditLogEntry {
    /// Builds a new audit entry without assigning a sequence yet.
    pub fn new(
        category: AuditEventCategory,
        kind: AuditEventKind,
        namespace: NamespaceId,
        actor_source: &'static str,
        detail: impl Into<String>,
    ) -> Self {
        Self {
            sequence: 0,
            category,
            kind,
            namespace,
            memory_id: None,
            session_id: None,
            actor_source,
            detail: detail.into(),
        }
    }

    /// Attaches an optional memory correlation id.
    pub fn with_memory_id(mut self, memory_id: MemoryId) -> Self {
        self.memory_id = Some(memory_id);
        self
    }

    /// Attaches an optional session correlation id.
    pub fn with_session_id(mut self, session_id: SessionId) -> Self {
        self.session_id = Some(session_id);
        self
    }
}

/// Bounded append-only audit log preserving insertion order.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AppendOnlyAuditLog {
    capacity: usize,
    next_sequence: u64,
    entries: VecDeque<AuditLogEntry>,
}

impl AppendOnlyAuditLog {
    /// Canonical default cap for append-only audit rows.
    pub const DEFAULT_CAPACITY: usize = 200_000;

    /// Builds a new append-only audit log with a hard row cap.
    pub fn new(capacity: usize) -> Self {
        Self {
            capacity: capacity.max(1),
            next_sequence: 1,
            entries: VecDeque::new(),
        }
    }

    /// Returns the number of retained audit rows.
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    /// Returns whether the audit log is empty.
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    /// Returns the current hard cap.
    pub fn capacity(&self) -> usize {
        self.capacity
    }

    /// Appends one audit entry and returns the stored row.
    pub fn append(&mut self, mut entry: AuditLogEntry) -> AuditLogEntry {
        entry.sequence = self.next_sequence;
        self.next_sequence += 1;

        if self.entries.len() >= self.capacity {
            self.entries.pop_front();
        }

        self.entries.push_back(entry.clone());
        entry
    }

    /// Returns retained audit rows in append order.
    pub fn entries(&self) -> Vec<AuditLogEntry> {
        self.entries.iter().cloned().collect()
    }

    /// Returns retained audit rows for the requested category in append order.
    pub fn entries_for_category(&self, category: AuditEventCategory) -> Vec<AuditLogEntry> {
        self.entries
            .iter()
            .filter(|entry| entry.category == category)
            .cloned()
            .collect()
    }

    /// Returns retained audit rows for the requested namespace in append order.
    pub fn entries_for_namespace(&self, namespace: &NamespaceId) -> Vec<AuditLogEntry> {
        self.entries
            .iter()
            .filter(|entry| &entry.namespace == namespace)
            .cloned()
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::{AppendOnlyAuditLog, AuditLogEntry, AuditLogStore};
    use crate::api::NamespaceId;
    use crate::observability::{AuditEventCategory, AuditEventKind};
    use crate::types::{MemoryId, SessionId};

    #[test]
    fn audit_log_appends_in_monotonic_order() {
        let mut log = AppendOnlyAuditLog::new(8);
        let namespace = NamespaceId::new("team.alpha").unwrap();

        let first = log.append(
            AuditLogEntry::new(
                AuditEventCategory::Encode,
                AuditEventKind::EncodeAccepted,
                namespace.clone(),
                "encode_engine",
                "accepted encode candidate",
            )
            .with_memory_id(MemoryId(11))
            .with_session_id(SessionId(3)),
        );
        let second = log.append(AuditLogEntry::new(
            AuditEventCategory::Policy,
            AuditEventKind::PolicyDenied,
            namespace,
            "policy_module",
            "policy denied widened read",
        ));

        assert_eq!(first.sequence, 1);
        assert_eq!(second.sequence, 2);
        assert_eq!(log.len(), 2);
        assert_eq!(log.entries()[0].kind, AuditEventKind::EncodeAccepted);
        assert_eq!(log.entries()[1].kind, AuditEventKind::PolicyDenied);
    }

    #[test]
    fn audit_log_retains_representative_encode_policy_and_maintenance_events() {
        let mut log = AuditLogStore.new_log(8);
        let namespace = NamespaceId::new("team.alpha").unwrap();

        log.append(
            AuditLogEntry::new(
                AuditEventCategory::Encode,
                AuditEventKind::EncodeAccepted,
                namespace.clone(),
                "encode_engine",
                "encoded memory into durable flow",
            )
            .with_memory_id(MemoryId(21))
            .with_session_id(SessionId(5)),
        );
        log.append(AuditLogEntry::new(
            AuditEventCategory::Policy,
            AuditEventKind::PolicyDenied,
            namespace.clone(),
            "policy_module",
            "blocked public widening without approval",
        ));
        log.append(AuditLogEntry::new(
            AuditEventCategory::Maintenance,
            AuditEventKind::MaintenanceRepairStarted,
            namespace.clone(),
            "repair_engine",
            "started repair run for stale derived state",
        ));

        let entries = log.entries_for_namespace(&namespace);
        assert_eq!(entries.len(), 3);
        assert_eq!(entries[0].category, AuditEventCategory::Encode);
        assert_eq!(entries[1].category, AuditEventCategory::Policy);
        assert_eq!(entries[2].category, AuditEventCategory::Maintenance);
        assert_eq!(entries[0].memory_id, Some(MemoryId(21)));
        assert_eq!(entries[0].session_id, Some(SessionId(5)));
    }

    #[test]
    fn audit_log_drops_oldest_rows_when_capacity_is_reached() {
        let mut log = AppendOnlyAuditLog::new(2);
        let namespace = NamespaceId::new("team.alpha").unwrap();

        log.append(AuditLogEntry::new(
            AuditEventCategory::Encode,
            AuditEventKind::EncodeAccepted,
            namespace.clone(),
            "encode_engine",
            "first",
        ));
        log.append(AuditLogEntry::new(
            AuditEventCategory::Recall,
            AuditEventKind::RecallServed,
            namespace.clone(),
            "recall_engine",
            "second",
        ));
        log.append(AuditLogEntry::new(
            AuditEventCategory::Archive,
            AuditEventKind::ArchiveRecorded,
            namespace,
            "cold_store",
            "third",
        ));

        let entries = log.entries();
        assert_eq!(entries.len(), 2);
        assert_eq!(entries[0].sequence, 2);
        assert_eq!(entries[0].kind, AuditEventKind::RecallServed);
        assert_eq!(entries[1].sequence, 3);
        assert_eq!(entries[1].kind, AuditEventKind::ArchiveRecorded);
    }
}
