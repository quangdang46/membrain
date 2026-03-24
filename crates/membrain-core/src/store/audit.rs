use crate::api::NamespaceId;
use crate::engine::contradiction::ContradictionRecord;
use crate::observability::{AuditEventCategory, AuditEventKind};
use crate::policy::{OperationClass, PolicyGateway, SafeguardRequest};
use crate::store::AuditLogStoreApi;
use crate::types::{MemoryId, SessionId};
use std::collections::VecDeque;

/// Exportable audit taxonomy row pairing a stable category with one event kind.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct AuditTaxonomyRow {
    pub category: AuditEventCategory,
    pub kind: AuditEventKind,
    pub category_name: &'static str,
    pub kind_name: &'static str,
}

impl AuditTaxonomyRow {
    /// Builds one stable taxonomy row for export and regression checks.
    pub const fn new(kind: AuditEventKind) -> Self {
        let category = kind.category();
        Self {
            category,
            kind,
            category_name: category.as_str(),
            kind_name: kind.as_str(),
        }
    }
}

/// Canonical append-only audit log boundary owned by `membrain-core` storage.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct AuditLogStore;

impl AuditLogStore {
    /// Returns the stable high-level audit categories accepted by the append-only log.
    pub const fn categories(&self) -> &'static [AuditEventCategory] {
        &[
            AuditEventCategory::Encode,
            AuditEventCategory::Recall,
            AuditEventCategory::Policy,
            AuditEventCategory::Maintenance,
            AuditEventCategory::Archive,
        ]
    }

    /// Returns the stable audit event taxonomy accepted by the append-only log.
    pub const fn event_kinds(&self) -> &'static [AuditEventKind] {
        &[
            AuditEventKind::EncodeAccepted,
            AuditEventKind::EncodeRejected,
            AuditEventKind::RecallServed,
            AuditEventKind::RecallDenied,
            AuditEventKind::PolicyDenied,
            AuditEventKind::PolicyRedacted,
            AuditEventKind::ApprovedSharing,
            AuditEventKind::MaintenanceRepairStarted,
            AuditEventKind::MaintenanceRepairCompleted,
            AuditEventKind::MaintenanceRepairDegraded,
            AuditEventKind::MaintenanceRepairRollbackTriggered,
            AuditEventKind::MaintenanceRepairRollbackCompleted,
            AuditEventKind::MaintenanceMigrationApplied,
            AuditEventKind::MaintenanceCompactionApplied,
            AuditEventKind::MaintenanceConsolidationStarted,
            AuditEventKind::MaintenanceConsolidationCompleted,
            AuditEventKind::MaintenanceConsolidationPartial,
            AuditEventKind::MaintenanceReconsolidationApplied,
            AuditEventKind::MaintenanceReconsolidationDiscarded,
            AuditEventKind::MaintenanceReconsolidationDeferred,
            AuditEventKind::MaintenanceReconsolidationBlocked,
            AuditEventKind::IncidentRecorded,
            AuditEventKind::ArchiveRecorded,
        ]
    }

    /// Returns the stable exportable taxonomy rows accepted by the append-only log.
    pub fn taxonomy(&self) -> Vec<AuditTaxonomyRow> {
        self.event_kinds()
            .iter()
            .copied()
            .map(AuditTaxonomyRow::new)
            .collect()
    }

    /// Builds a bounded append-only audit log with a hard row cap.
    pub fn new_log(&self, capacity: usize) -> AppendOnlyAuditLog {
        AppendOnlyAuditLog::new(capacity)
    }

    /// Builds a bounded append-only audit log with the canonical default row cap.
    pub fn new_default_log(&self) -> AppendOnlyAuditLog {
        self.new_log(AppendOnlyAuditLog::DEFAULT_CAPACITY)
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
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
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

/// Bounded filter used to inspect one audit-history slice.
#[derive(Debug, Clone, Default, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct AuditLogFilter {
    pub namespace: Option<NamespaceId>,
    pub memory_id: Option<MemoryId>,
    pub session_id: Option<SessionId>,
    pub category: Option<AuditEventCategory>,
    pub kind: Option<AuditEventKind>,
    pub request_id: Option<String>,
    pub related_run: Option<String>,
    pub min_sequence: Option<u64>,
    pub max_sequence: Option<u64>,
    pub redacted: Option<bool>,
}

impl AuditLogFilter {
    /// Returns whether the entry matches the configured slice constraints.
    pub fn matches(&self, entry: &AuditLogEntry) -> bool {
        self.namespace
            .as_ref()
            .is_none_or(|namespace| &entry.namespace == namespace)
            && self
                .memory_id
                .is_none_or(|memory_id| entry.memory_id == Some(memory_id))
            && self
                .session_id
                .is_none_or(|session_id| entry.session_id == Some(session_id))
            && self
                .category
                .is_none_or(|category| entry.category == category)
            && self.kind.is_none_or(|kind| entry.kind == kind)
            && self
                .request_id
                .as_deref()
                .is_none_or(|request_id| entry.request_id.as_deref() == Some(request_id))
            && self
                .related_run
                .as_deref()
                .is_none_or(|related_run| entry.related_run.as_deref() == Some(related_run))
            && self
                .min_sequence
                .is_none_or(|min_sequence| entry.sequence >= min_sequence)
            && self
                .max_sequence
                .is_none_or(|max_sequence| entry.sequence <= max_sequence)
            && self
                .redacted
                .is_none_or(|redacted| entry.redacted == redacted)
    }
}

/// Structured bounded export for one audit-history slice.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
pub struct AuditLogSlice {
    pub rows: Vec<AuditLogEntry>,
    pub total_matches: usize,
    pub truncated: bool,
}

impl AuditLogSlice {
    /// Returns how many rows are included in the exported slice.
    pub fn returned_rows(&self) -> usize {
        self.rows.len()
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

    /// Returns the next monotonic sequence that will be assigned on append.
    pub fn next_sequence(&self) -> u64 {
        self.next_sequence
    }

    /// Returns the last retained sequence, if any rows are present.
    pub fn last_sequence(&self) -> Option<u64> {
        self.entries.back().map(|entry| entry.sequence)
    }

    /// Returns the canonical row sequence range retained in this log.
    pub fn retained_sequence_range(&self) -> Option<std::ops::RangeInclusive<u64>> {
        Some(self.entries.front()?.sequence..=self.entries.back()?.sequence)
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

    /// Returns retained audit rows for one session in append order.
    pub fn entries_for_session(&self, session_id: SessionId) -> Vec<AuditLogEntry> {
        self.entries
            .iter()
            .filter(|entry| entry.session_id == Some(session_id))
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

    /// Returns one bounded, structured audit-history slice after applying the requested filters.
    pub fn slice(&self, filter: &AuditLogFilter, limit: Option<usize>) -> AuditLogSlice {
        let total_matches = self
            .entries
            .iter()
            .filter(|entry| filter.matches(entry))
            .count();

        let mut rows: Vec<_> = self
            .entries
            .iter()
            .filter(|entry| filter.matches(entry))
            .cloned()
            .collect();

        if let Some(limit) = limit {
            if rows.len() > limit {
                rows = rows.split_off(rows.len() - limit);
            }
        }

        AuditLogSlice {
            truncated: rows.len() < total_matches,
            rows,
            total_matches,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{
        AppendOnlyAuditLog, AuditLogEntry, AuditLogFilter, AuditLogSlice, AuditLogStore,
        AuditTaxonomyRow,
    };
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
    fn audit_store_exposes_canonical_taxonomy_and_default_capacity_log() {
        let store = AuditLogStore;
        let categories = store.categories();
        let kinds = store.event_kinds();
        let taxonomy = store.taxonomy();
        let log = store.new_default_log();

        assert_eq!(categories.len(), 5);
        assert_eq!(categories[0], AuditEventCategory::Encode);
        assert_eq!(categories[4], AuditEventCategory::Archive);
        assert!(kinds.contains(&AuditEventKind::EncodeAccepted));
        assert!(kinds.contains(&AuditEventKind::PolicyDenied));
        assert!(kinds.contains(&AuditEventKind::ApprovedSharing));
        assert!(kinds.contains(&AuditEventKind::MaintenanceConsolidationPartial));
        assert!(kinds.contains(&AuditEventKind::MaintenanceReconsolidationApplied));
        assert!(kinds.contains(&AuditEventKind::MaintenanceReconsolidationDiscarded));
        assert!(kinds.contains(&AuditEventKind::MaintenanceReconsolidationDeferred));
        assert!(kinds.contains(&AuditEventKind::MaintenanceReconsolidationBlocked));
        assert!(kinds.contains(&AuditEventKind::ArchiveRecorded));
        assert_eq!(taxonomy.len(), kinds.len());
        assert_eq!(
            taxonomy[0],
            AuditTaxonomyRow::new(AuditEventKind::EncodeAccepted)
        );
        assert_eq!(taxonomy[0].category_name, "encode");
        assert_eq!(taxonomy[0].kind_name, "encode_accepted");
        assert_eq!(
            taxonomy.last(),
            Some(&AuditTaxonomyRow::new(AuditEventKind::ArchiveRecorded))
        );
        assert_eq!(log.capacity(), AppendOnlyAuditLog::DEFAULT_CAPACITY);
        assert_eq!(log.next_sequence(), 1);
        assert_eq!(log.last_sequence(), None);
        assert_eq!(log.retained_sequence_range(), None);
    }

    #[test]
    fn audit_taxonomy_rows_remain_category_consistent_and_machine_readable() {
        let taxonomy = AuditLogStore.taxonomy();

        assert!(taxonomy
            .iter()
            .all(|row| row.category == row.kind.category()));
        assert!(taxonomy
            .iter()
            .all(|row| row.category_name == row.category.as_str()));
        assert!(taxonomy
            .iter()
            .all(|row| row.kind_name == row.kind.as_str()));
        assert!(taxonomy
            .iter()
            .any(|row| row.kind_name == "approved_sharing"));
        assert!(taxonomy
            .iter()
            .any(|row| row.kind_name == "maintenance_migration_applied"));
        assert!(taxonomy
            .iter()
            .any(|row| row.kind_name == "maintenance_reconsolidation_applied"));
        assert!(taxonomy
            .iter()
            .any(|row| row.kind_name == "maintenance_reconsolidation_discarded"));
        assert!(taxonomy
            .iter()
            .any(|row| row.kind_name == "maintenance_reconsolidation_deferred"));
        assert!(taxonomy
            .iter()
            .any(|row| row.kind_name == "maintenance_reconsolidation_blocked"));
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
        assert_eq!(log.next_sequence(), 3);
        assert_eq!(log.last_sequence(), Some(2));
        assert_eq!(log.retained_sequence_range(), Some(1..=2));
        assert_eq!(log.entries()[0].kind, AuditEventKind::EncodeAccepted);
        assert_eq!(log.entries()[1].kind, AuditEventKind::PolicyDenied);
    }

    #[test]
    fn audit_log_retains_representative_encode_policy_and_maintenance_events() {
        let store = AuditLogStore;
        let taxonomy = store.taxonomy();
        let mut log = store.new_log(8);
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
        assert!(taxonomy
            .iter()
            .any(|row| row.kind == entries[0].kind && row.category == entries[0].category));
        assert!(taxonomy
            .iter()
            .any(|row| row.kind == entries[1].kind && row.category == entries[1].category));
        assert!(taxonomy
            .iter()
            .any(|row| row.kind == entries[2].kind && row.category == entries[2].category));
        assert_eq!(entries[0].memory_id, Some(MemoryId(21)));
        assert_eq!(entries[0].session_id, Some(SessionId(5)));
    }

    #[test]
    fn audit_log_retains_degraded_and_rollback_maintenance_events() {
        let mut log = AuditLogStore.new_log(8);
        let namespace = team_alpha();

        log.append(
            AuditLogEntry::new(
                AuditEventCategory::Maintenance,
                AuditEventKind::MaintenanceRepairDegraded,
                namespace.clone(),
                "repair_engine",
                "repair run left cache warm state degraded",
            )
            .with_request_id("req-repair-degraded")
            .with_related_run("repair-run-17"),
        );
        log.append(
            AuditLogEntry::new(
                AuditEventCategory::Maintenance,
                AuditEventKind::MaintenanceRepairRollbackTriggered,
                namespace.clone(),
                "repair_engine",
                "rollback required after derived-state mismatch",
            )
            .with_request_id("req-repair-rollback")
            .with_related_run("repair-run-17"),
        );
        log.append(
            AuditLogEntry::new(
                AuditEventCategory::Maintenance,
                AuditEventKind::MaintenanceRepairRollbackCompleted,
                namespace.clone(),
                "repair_engine",
                "rollback completed and durable generation restored",
            )
            .with_related_run("repair-run-17"),
        );

        let namespace_entries = log.entries_for_namespace(&namespace);
        let rollback_entries = log.entries_for_related_run("repair-run-17");
        let degraded_entries = log.entries_for_kind(AuditEventKind::MaintenanceRepairDegraded);
        let rollback_trigger_entries =
            log.entries_for_kind(AuditEventKind::MaintenanceRepairRollbackTriggered);
        let rollback_completed_entries =
            log.entries_for_kind(AuditEventKind::MaintenanceRepairRollbackCompleted);

        assert_eq!(namespace_entries.len(), 3);
        assert!(namespace_entries
            .iter()
            .all(|entry| entry.category == AuditEventCategory::Maintenance));
        assert_eq!(rollback_entries.len(), 3);
        assert_eq!(degraded_entries.len(), 1);
        assert_eq!(rollback_trigger_entries.len(), 1);
        assert_eq!(rollback_completed_entries.len(), 1);
        assert_eq!(
            degraded_entries[0].request_id.as_deref(),
            Some("req-repair-degraded")
        );
        assert_eq!(
            rollback_trigger_entries[0].request_id.as_deref(),
            Some("req-repair-rollback")
        );
        assert_eq!(
            rollback_completed_entries[0].related_run.as_deref(),
            Some("repair-run-17")
        );
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
            .with_session_id(SessionId(12))
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
        let session_entries = log.entries_for_session(SessionId(12));
        let request_entries = log.entries_for_request_id("req-policy-41");
        let incident_entries = log.entries_for_related_run("incident-2026-03-20");
        let migration_entries = log.entries_for_kind(AuditEventKind::MaintenanceMigrationApplied);
        let migration_run_entries = log.entries_for_related_run("migration-0042");

        assert_eq!(memory_entries.len(), 2);
        assert_eq!(session_entries.len(), 1);
        assert_eq!(request_entries.len(), 1);
        assert_eq!(incident_entries.len(), 1);
        assert!(memory_entries[0].redacted);
        assert_eq!(session_entries[0], memory_entries[0]);
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
        assert_eq!(log.next_sequence(), 4);
        assert_eq!(log.last_sequence(), Some(3));
        assert_eq!(log.retained_sequence_range(), Some(2..=3));
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

    #[test]
    fn contradiction_retention_audit_uses_stable_fallback_detail_without_redaction() {
        let store = AuditLogStore;
        let mut log = store.new_log(8);
        let namespace = team_alpha();
        let contradiction = ContradictionRecord {
            id: ContradictionId(10),
            namespace,
            memory_a: MemoryId(30),
            memory_b: MemoryId(40),
            kind: ContradictionKind::Supersession,
            resolution: ResolutionState::AuthoritativelyResolved,
            preferred_memory: None,
            preferred_answer_state: PreferredAnswerState::Unset,
            confidence_signal: 910,
            resolution_reason: None,
            archived: true,
            legal_hold: false,
            authoritative_evidence: true,
            retention_reason: None,
            conflict_score: 840,
        };

        let entry = store.record_contradiction_retention(
            &mut log,
            &contradiction,
            &PolicyModule,
            "req-contradiction-10",
        );

        assert_eq!(entry.kind, AuditEventKind::ArchiveRecorded);
        assert_eq!(entry.detail, "contradiction archived after supersession");
        assert_eq!(entry.memory_id, Some(MemoryId(30)));
        assert_eq!(entry.request_id.as_deref(), Some("req-contradiction-10"));
        assert_eq!(
            entry.related_run.as_deref(),
            Some("contradiction-archive-run")
        );
        assert!(!entry.redacted);
        assert_eq!(
            log.entries_for_request_id("req-contradiction-10"),
            vec![entry]
        );
    }

    #[test]
    fn audit_log_slice_filters_and_limits_rows() {
        let mut log = AppendOnlyAuditLog::new(8);
        let namespace = team_alpha();

        log.append(
            AuditLogEntry::new(
                AuditEventCategory::Encode,
                AuditEventKind::EncodeAccepted,
                namespace.clone(),
                "encode_engine",
                "encoded memory into durable flow",
            )
            .with_memory_id(MemoryId(51))
            .with_session_id(SessionId(4))
            .with_request_id("req-encode-51"),
        );
        log.append(
            AuditLogEntry::new(
                AuditEventCategory::Policy,
                AuditEventKind::PolicyRedacted,
                namespace.clone(),
                "policy_module",
                "redacted sensitive actor fields",
            )
            .with_memory_id(MemoryId(51))
            .with_request_id("req-policy-51")
            .with_related_run("incident-2026-03-23")
            .with_redaction(),
        );
        log.append(
            AuditLogEntry::new(
                AuditEventCategory::Maintenance,
                AuditEventKind::MaintenanceMigrationApplied,
                namespace.clone(),
                "migration_runner",
                "applied audit-log schema migration",
            )
            .with_memory_id(MemoryId(51))
            .with_related_run("migration-0043"),
        );

        let filter = AuditLogFilter {
            namespace: Some(namespace.clone()),
            memory_id: Some(MemoryId(51)),
            min_sequence: Some(2),
            ..AuditLogFilter::default()
        };
        let slice = log.slice(&filter, Some(1));

        assert_eq!(slice.total_matches, 2);
        assert!(slice.truncated);
        assert_eq!(slice.returned_rows(), 1);
        assert_eq!(
            slice.rows[0].kind,
            AuditEventKind::MaintenanceMigrationApplied
        );
    }

    #[test]
    fn audit_log_slice_preserves_export_ready_rows_and_metadata() {
        let mut log = AppendOnlyAuditLog::new(8);
        let namespace = team_alpha();

        log.append(
            AuditLogEntry::new(
                AuditEventCategory::Encode,
                AuditEventKind::EncodeAccepted,
                namespace.clone(),
                "encode_engine",
                "encoded durable memory",
            )
            .with_memory_id(MemoryId(61))
            .with_session_id(SessionId(7))
            .with_request_id("req-encode-61"),
        );
        log.append(
            AuditLogEntry::new(
                AuditEventCategory::Policy,
                AuditEventKind::PolicyRedacted,
                namespace.clone(),
                "policy_module",
                "redacted actor fields for export",
            )
            .with_memory_id(MemoryId(61))
            .with_request_id("req-policy-61")
            .with_related_run("incident-2026-03-23")
            .with_redaction(),
        );

        let slice = log.slice(
            &AuditLogFilter {
                namespace: Some(namespace),
                memory_id: Some(MemoryId(61)),
                ..AuditLogFilter::default()
            },
            Some(8),
        );

        assert_eq!(slice.total_matches, 2);
        assert!(!slice.truncated);
        assert_eq!(slice.rows.len(), 2);
        assert_eq!(slice.rows[0].kind, AuditEventKind::EncodeAccepted);
        assert_eq!(slice.rows[1].kind, AuditEventKind::PolicyRedacted);
        assert_eq!(slice.rows[0].request_id.as_deref(), Some("req-encode-61"));
        assert_eq!(
            slice.rows[1].related_run.as_deref(),
            Some("incident-2026-03-23")
        );
        assert!(slice.rows[1].redacted);
    }

    #[test]
    fn audit_log_slice_can_filter_by_request_id_kind_and_redaction() {
        let mut log = AppendOnlyAuditLog::new(8);
        let namespace = team_alpha();

        log.append(
            AuditLogEntry::new(
                AuditEventCategory::Policy,
                AuditEventKind::PolicyDenied,
                namespace.clone(),
                "policy_module",
                "denied namespace widening",
            )
            .with_request_id("req-policy-denied"),
        );
        log.append(
            AuditLogEntry::new(
                AuditEventCategory::Policy,
                AuditEventKind::PolicyRedacted,
                namespace,
                "policy_module",
                "redacted actor metadata",
            )
            .with_request_id("req-policy-redacted")
            .with_redaction(),
        );

        let filter = AuditLogFilter {
            kind: Some(AuditEventKind::PolicyRedacted),
            request_id: Some("req-policy-redacted".to_string()),
            redacted: Some(true),
            ..AuditLogFilter::default()
        };
        let AuditLogSlice {
            rows,
            total_matches,
            truncated,
        } = log.slice(&filter, Some(8));

        assert_eq!(total_matches, 1);
        assert!(!truncated);
        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0].kind, AuditEventKind::PolicyRedacted);
        assert_eq!(rows[0].request_id.as_deref(), Some("req-policy-redacted"));
        assert!(rows[0].redacted);
    }

    #[test]
    fn audit_taxonomy_rows_and_filters_serialize_machine_readable_payloads() {
        let taxonomy = AuditLogStore.taxonomy();
        let filter = AuditLogFilter {
            namespace: Some(team_alpha()),
            memory_id: Some(MemoryId(88)),
            session_id: Some(SessionId(13)),
            category: Some(AuditEventCategory::Maintenance),
            kind: Some(AuditEventKind::MaintenanceRepairRollbackCompleted),
            request_id: Some("req-repair-88".to_string()),
            related_run: Some("repair-run-88".to_string()),
            min_sequence: Some(7),
            max_sequence: Some(9),
            redacted: Some(true),
        };

        let taxonomy_json = serde_json::to_value(&taxonomy).expect("taxonomy should serialize");
        let filter_json = serde_json::to_value(&filter).expect("filter should serialize");

        assert!(taxonomy_json.as_array().is_some());
        assert!(taxonomy_json
            .as_array()
            .expect("taxonomy should be an array")
            .iter()
            .any(|row| {
                row["category"] == "maintenance"
                    && row["kind"] == "maintenance_repair_rollback_completed"
                    && row["category_name"] == "maintenance"
                    && row["kind_name"] == "maintenance_repair_rollback_completed"
            }));
        assert_eq!(filter_json["namespace"], "team.alpha");
        assert_eq!(filter_json["memory_id"], 88);
        assert_eq!(filter_json["session_id"], 13);
        assert_eq!(filter_json["category"], "maintenance");
        assert_eq!(filter_json["kind"], "maintenance_repair_rollback_completed");
        assert_eq!(filter_json["request_id"], "req-repair-88");
        assert_eq!(filter_json["related_run"], "repair-run-88");
        assert_eq!(filter_json["min_sequence"], 7);
        assert_eq!(filter_json["max_sequence"], 9);
        assert_eq!(filter_json["redacted"], true);
    }
}
