use crate::api::NamespaceId;
use crate::engine::contradiction::ContradictionRecord;
use crate::observability::{AuditEventCategory, AuditEventKind};
use crate::policy::{OperationClass, PolicyGateway, SafeguardRequest};
use crate::store::AuditLogStoreApi;
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

    /// Records contradiction archive or legal-hold policy decisions without dropping the durable row.
    pub fn record_contradiction_retention(
        &self,
        log: &mut AppendOnlyAuditLog,
        contradiction: &ContradictionRecord,
        policy: &impl PolicyGateway,
        request_id: &str,
    ) -> AuditLogEntry {
        let safeguard = policy.evaluate_safeguard(SafeguardRequest {
            operation_class: OperationClass::ContradictionArchive,
            preview_only: false,
            namespace_bound: true,
            policy_allowed: !contradiction.legal_hold,
            requires_confirmation: contradiction.archived,
            local_confirmation: !contradiction.archived || contradiction.legal_hold,
            force_allowed: contradiction.legal_hold,
            generation_bound: Some(contradiction.id.0),
            generation_matches: true,
            snapshot_required: contradiction.archived,
            snapshot_available: !contradiction.authoritative_evidence || contradiction.legal_hold,
            maintenance_window_required: false,
            maintenance_window_active: true,
            dependencies_ready: true,
            scope_precise: true,
            authoritative_input_readable: true,
            confidence_ready: true,
            can_degrade: false,
            legal_hold: contradiction.legal_hold,
        });

        let detail = contradiction.retention_reason.clone().unwrap_or_else(|| {
            if contradiction.legal_hold {
                "contradiction retained under legal hold".to_string()
            } else if contradiction.archived {
                "contradiction archived after supersession".to_string()
            } else {
                "contradiction retained as durable evidence".to_string()
            }
        });

        let entry = AuditLogEntry::new(
            AuditEventCategory::Archive,
            AuditEventKind::ArchiveRecorded,
            contradiction.namespace.clone(),
            "contradiction_policy",
            detail,
        )
        .with_memory_id(
            contradiction
                .preferred_memory
                .unwrap_or(contradiction.memory_a),
        )
        .with_request_id(request_id)
        .with_related_run(
            safeguard
                .audit
                .related_run
                .unwrap_or("contradiction-archive-run"),
        );

        let entry = if safeguard.policy_summary.decision == crate::policy::PolicyDecision::Deny {
            entry.with_redaction()
        } else {
            entry
        };

        log.append(entry)
    }
}

impl AuditLogStoreApi for AuditLogStore {
    fn component_name(&self) -> &'static str {
        "store.audit"
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
    /// Optional request-scoped correlation handle for cross-surface audit parity.
    pub request_id: Option<String>,
    /// Optional maintenance, migration, or incident correlation handle.
    pub related_run: Option<String>,
    /// Whether policy redaction affected the visible entry.
    pub redacted: bool,
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
            request_id: None,
            related_run: None,
            redacted: false,
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

    /// Attaches an optional request-scoped correlation handle.
    pub fn with_request_id(mut self, request_id: impl Into<String>) -> Self {
        self.request_id = Some(request_id.into());
        self
    }

    /// Attaches an optional maintenance, migration, or incident correlation handle.
    pub fn with_related_run(mut self, related_run: impl Into<String>) -> Self {
        self.related_run = Some(related_run.into());
        self
    }

    /// Marks the entry as policy-redacted while preserving the row.
    pub fn with_redaction(mut self) -> Self {
        self.redacted = true;
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
        entry.category = entry.kind.category();
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

    /// Returns retained audit rows for one memory in append order.
    pub fn entries_for_memory(&self, memory_id: MemoryId) -> Vec<AuditLogEntry> {
        self.entries
            .iter()
            .filter(|entry| entry.memory_id == Some(memory_id))
            .cloned()
            .collect()
    }

    /// Returns retained audit rows for one request correlation handle in append order.
    pub fn entries_for_request_id(&self, request_id: &str) -> Vec<AuditLogEntry> {
        self.entries
            .iter()
            .filter(|entry| entry.request_id.as_deref() == Some(request_id))
            .cloned()
            .collect()
    }

    /// Returns retained audit rows for one related run correlation handle in append order.
    pub fn entries_for_related_run(&self, related_run: &str) -> Vec<AuditLogEntry> {
        self.entries
            .iter()
            .filter(|entry| entry.related_run.as_deref() == Some(related_run))
            .cloned()
            .collect()
    }

    /// Returns retained audit rows for one event kind in append order.
    pub fn entries_for_kind(&self, kind: AuditEventKind) -> Vec<AuditLogEntry> {
        self.entries
            .iter()
            .filter(|entry| entry.kind == kind)
            .cloned()
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::{AppendOnlyAuditLog, AuditLogEntry, AuditLogStore};
    use crate::api::NamespaceId;
    use crate::engine::contradiction::{
        ContradictionId, ContradictionKind, ContradictionRecord, PreferredAnswerState,
        ResolutionState,
    };
    use crate::observability::{AuditEventCategory, AuditEventKind};
    use crate::policy::PolicyModule;
    use crate::types::{MemoryId, SessionId};

    fn team_alpha() -> NamespaceId {
        NamespaceId::new("team.alpha").expect("team.alpha should be a valid namespace id")
    }

    #[test]
    fn audit_log_appends_in_monotonic_order() {
        let mut log = AppendOnlyAuditLog::new(8);
        let namespace = team_alpha();

        let first = log.append(
            AuditLogEntry::new(
                AuditEventCategory::Encode,
                AuditEventKind::EncodeAccepted,
                namespace.clone(),
                "encode_engine",
                "accepted encode candidate",
            )
            .with_memory_id(MemoryId(11))
            .with_session_id(SessionId(3))
            .with_request_id("req-encode-11"),
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
        assert_eq!(first.request_id.as_deref(), Some("req-encode-11"));
        assert_eq!(log.len(), 2);
        assert_eq!(log.entries()[0].kind, AuditEventKind::EncodeAccepted);
        assert_eq!(log.entries()[1].kind, AuditEventKind::PolicyDenied);
    }

    #[test]
    fn audit_log_retains_representative_encode_policy_and_maintenance_events() {
        let mut log = AuditLogStore.new_log(8);
        let namespace = team_alpha();

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
    fn audit_log_filters_by_memory_kind_and_redaction_metadata() {
        let mut log = AppendOnlyAuditLog::new(8);
        let namespace = team_alpha();

        log.append(
            AuditLogEntry::new(
                AuditEventCategory::Policy,
                AuditEventKind::PolicyRedacted,
                namespace.clone(),
                "policy_module",
                "redacted actor for external view",
            )
            .with_memory_id(MemoryId(41))
            .with_request_id("req-policy-41")
            .with_related_run("incident-2026-03-20")
            .with_redaction(),
        );
        log.append(
            AuditLogEntry::new(
                AuditEventCategory::Maintenance,
                AuditEventKind::MaintenanceMigrationApplied,
                namespace.clone(),
                "migration_runner",
                "applied sqlite migration for audit-log rows",
            )
            .with_memory_id(MemoryId(41))
            .with_related_run("migration-0042"),
        );
        log.append(AuditLogEntry::new(
            AuditEventCategory::Archive,
            AuditEventKind::ArchiveRecorded,
            namespace,
            "cold_store",
            "archived superseded evidence",
        ));

        let memory_entries = log.entries_for_memory(MemoryId(41));
        let request_entries = log.entries_for_request_id("req-policy-41");
        let incident_entries = log.entries_for_related_run("incident-2026-03-20");
        let migration_entries = log.entries_for_kind(AuditEventKind::MaintenanceMigrationApplied);
        let migration_run_entries = log.entries_for_related_run("migration-0042");

        assert_eq!(memory_entries.len(), 2);
        assert_eq!(request_entries.len(), 1);
        assert_eq!(incident_entries.len(), 1);
        assert!(memory_entries[0].redacted);
        assert_eq!(
            memory_entries[0].request_id.as_deref(),
            Some("req-policy-41")
        );
        assert_eq!(
            memory_entries[0].related_run.as_deref(),
            Some("incident-2026-03-20")
        );
        assert_eq!(request_entries[0], memory_entries[0]);
        assert_eq!(incident_entries[0], memory_entries[0]);
        assert_eq!(migration_entries.len(), 1);
        assert_eq!(migration_run_entries.len(), 1);
        assert_eq!(migration_entries[0].actor_source, "migration_runner");
        assert_eq!(
            migration_entries[0].related_run.as_deref(),
            Some("migration-0042")
        );
        assert_eq!(migration_run_entries[0], migration_entries[0]);
    }

    #[test]
    fn append_recomputes_category_from_kind() {
        let mut log = AppendOnlyAuditLog::new(4);
        let namespace = team_alpha();

        let stored = log.append(AuditLogEntry::new(
            AuditEventCategory::Encode,
            AuditEventKind::PolicyDenied,
            namespace,
            "policy_module",
            "mismatched caller-provided category should not persist",
        ));

        assert_eq!(stored.category, AuditEventCategory::Policy);
        assert_eq!(log.entries()[0].category, AuditEventCategory::Policy);
    }

    #[test]
    fn audit_log_drops_oldest_rows_when_capacity_is_reached() {
        let mut log = AppendOnlyAuditLog::new(2);
        let namespace = team_alpha();

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

    #[test]
    fn contradiction_retention_audit_keeps_archive_event_and_request_correlation() {
        let store = AuditLogStore;
        let mut log = store.new_log(8);
        let namespace = team_alpha();
        let contradiction = ContradictionRecord {
            id: ContradictionId(9),
            namespace: namespace.clone(),
            memory_a: MemoryId(10),
            memory_b: MemoryId(20),
            kind: ContradictionKind::Supersession,
            resolution: ResolutionState::AuthoritativelyResolved,
            preferred_memory: Some(MemoryId(20)),
            preferred_answer_state: PreferredAnswerState::Preferred,
            confidence_signal: 960,
            resolution_reason: Some("authoritative source".to_string()),
            archived: true,
            legal_hold: true,
            authoritative_evidence: true,
            retention_reason: Some("legal hold keeps archived contradiction".to_string()),
            conflict_score: 880,
        };

        let entry = store.record_contradiction_retention(
            &mut log,
            &contradiction,
            &PolicyModule,
            "req-contradiction-9",
        );

        assert_eq!(entry.kind, AuditEventKind::ArchiveRecorded);
        assert_eq!(entry.request_id.as_deref(), Some("req-contradiction-9"));
        assert_eq!(
            entry.related_run.as_deref(),
            Some("contradiction-archive-run")
        );
        assert_eq!(entry.memory_id, Some(MemoryId(20)));
        assert!(entry.redacted);
        assert_eq!(log.entries_for_request_id("req-contradiction-9").len(), 1);
    }
}
