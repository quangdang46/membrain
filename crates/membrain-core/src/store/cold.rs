//! Cold/archive store boundary.
//!
//! Owns cold-tier storage for archived, demoted, and long-retention
//! memories. Provides restore surfaces for retrieving forgotten items.

use crate::api::NamespaceId;
use crate::observability::OutcomeClass;
use crate::policy::{OperationClass, PolicyGateway, SafeguardRequest};
use crate::store::ColdStoreApi;
use crate::types::MemoryId;

/// Archive state of a cold-stored memory.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ArchiveState {
    /// Archived records are retained in the cold tier, omitted from ordinary recall, but available to explicit cold-tier inspect and restore surfaces.
    Archived,
    /// Soft-forgotten: omitted from ordinary recall but available to explicit cold-tier recall and restore surfaces.
    Forgotten,
    /// Permanently expired and eligible for cleanup.
    Expired,
    /// Irreversibly deleted through the dedicated policy-governed hard-delete path.
    HardDeleted,
}

impl ArchiveState {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Archived => "archived",
            Self::Forgotten => "forgotten",
            Self::Expired => "expired",
            Self::HardDeleted => "hard_deleted",
        }
    }

    pub const fn restorable(self) -> bool {
        matches!(self, Self::Archived | Self::Forgotten)
    }

    pub const fn hard_deleted(self) -> bool {
        matches!(self, Self::HardDeleted)
    }
}

/// Why a cold record entered the archive lane.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ArchiveReason {
    UtilityForgetting,
    ExplicitArchive,
    RetentionCompaction,
}

impl ArchiveReason {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::UtilityForgetting => "utility_forgetting_archive",
            Self::ExplicitArchive => "explicit_archive",
            Self::RetentionCompaction => "retention_compaction",
        }
    }
}

/// Explicit tombstone reasons for archived or detached content.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TombstoneReason {
    PolicyRedaction,
    RetentionPurge,
    PayloadDropped,
    HardDeleted,
}

impl TombstoneReason {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::PolicyRedaction => "policy_redaction",
            Self::RetentionPurge => "retention_purge",
            Self::PayloadDropped => "payload_dropped",
            Self::HardDeleted => "hard_deleted",
        }
    }
}

/// Loss reasons that can degrade archive inspection or restore fidelity.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum LossReason {
    DetachedPayloadMissing,
    ContentHandleMissing,
    LineageMissing,
    DerivedStateStale,
}

impl LossReason {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::DetachedPayloadMissing => "detached_payload_missing",
            Self::ContentHandleMissing => "content_handle_missing",
            Self::LineageMissing => "lineage_missing",
            Self::DerivedStateStale => "derived_state_stale",
        }
    }
}

/// The surface where an explicit loss or tombstone indicator applies.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum LossSurface {
    Payload,
    Content,
    Lineage,
    DerivedState,
}

impl LossSurface {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Payload => "payload",
            Self::Content => "content",
            Self::Lineage => "lineage",
            Self::DerivedState => "derived_state",
        }
    }
}

/// Machine-readable kind of explicit restore degradation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum LossKind {
    Tombstoned,
    Unavailable,
    Stale,
}

impl LossKind {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Tombstoned => "tombstoned",
            Self::Unavailable => "unavailable",
            Self::Stale => "stale",
        }
    }
}

/// Explicit loss or tombstone marker surfaced by archive inspect and restore flows.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct LossIndicator {
    pub surface: LossSurface,
    pub kind: LossKind,
    pub reason: &'static str,
}

impl LossIndicator {
    pub const fn tombstone(surface: LossSurface, reason: TombstoneReason) -> Self {
        Self {
            surface,
            kind: LossKind::Tombstoned,
            reason: reason.as_str(),
        }
    }

    pub const fn unavailable(surface: LossSurface, reason: LossReason) -> Self {
        Self {
            surface,
            kind: LossKind::Unavailable,
            reason: reason.as_str(),
        }
    }

    pub const fn stale(surface: LossSurface, reason: LossReason) -> Self {
        Self {
            surface,
            kind: LossKind::Stale,
            reason: reason.as_str(),
        }
    }
}

/// Storage representation of the archived payload body.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ColdPayloadState {
    /// The bounded payload remains in the cold row itself.
    Inline,
    /// Payload bytes are detached behind a stable `payload_ref`.
    Detached,
    /// Payload bytes were intentionally removed and replaced by an explicit tombstone.
    Tombstoned(TombstoneReason),
    /// Payload bytes should exist but are currently unavailable or stale.
    Unavailable(LossReason),
}

impl ColdPayloadState {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Inline => "inline",
            Self::Detached => "detached",
            Self::Tombstoned(_) => "tombstoned",
            Self::Unavailable(_) => "unavailable",
        }
    }

    pub const fn primary_loss_indicator(self) -> Option<LossIndicator> {
        match self {
            Self::Inline | Self::Detached => None,
            Self::Tombstoned(reason) => {
                Some(LossIndicator::tombstone(LossSurface::Payload, reason))
            }
            Self::Unavailable(reason) => {
                Some(LossIndicator::unavailable(LossSurface::Payload, reason))
            }
        }
    }
}

/// Metadata record for one cold-stored memory.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ColdRecord {
    pub memory_id: MemoryId,
    pub namespace: NamespaceId,
    pub state: ArchiveState,
    pub compact_text: String,
    pub original_tier: &'static str,
    pub content_ref: Option<String>,
    pub payload_ref: Option<String>,
    pub archive_reason: Option<ArchiveReason>,
    pub payload_state: ColdPayloadState,
    pub loss_indicators: Vec<LossIndicator>,
    pub legal_hold: bool,
    pub deletion_authority: Option<DeletionAuthority>,
    pub requested_by: Option<&'static str>,
    pub destructive_confirmation: bool,
}

impl ColdRecord {
    /// Returns whether this record can still be surfaced to explicit cold-tier recall flows.
    pub const fn recall_visible(&self) -> bool {
        matches!(self.state, ArchiveState::Forgotten)
    }

    /// Returns whether this record may be restored back into active storage.
    pub const fn restorable(&self) -> bool {
        self.state.restorable()
    }

    /// Returns whether metadata inspect can proceed without fetching the detached payload body.
    pub const fn inspectable_without_payload_fetch(&self) -> bool {
        true
    }

    /// Returns the explicit loss or tombstone indicators that must be surfaced during restore.
    pub fn restore_loss_indicators(&self) -> Vec<LossIndicator> {
        let mut indicators = self.loss_indicators.clone();

        if let Some(indicator) = self.payload_state.primary_loss_indicator() {
            if !indicators.contains(&indicator) {
                indicators.push(indicator);
            }
        }

        if self.content_ref.is_none() {
            let indicator =
                LossIndicator::unavailable(LossSurface::Content, LossReason::ContentHandleMissing);
            if !indicators.contains(&indicator) {
                indicators.push(indicator);
            }
        }

        if matches!(self.payload_state, ColdPayloadState::Detached) && self.payload_ref.is_none() {
            let indicator = LossIndicator::unavailable(
                LossSurface::Payload,
                LossReason::DetachedPayloadMissing,
            );
            if !indicators.contains(&indicator) {
                indicators.push(indicator);
            }
        }

        indicators
    }

    /// Evaluates the restore outcome for this record without mutating storage.
    pub fn restore_result(&self) -> RestoreResult {
        if !self.restorable() {
            RestoreResult::NotRestorable
        } else if self.restore_loss_indicators().is_empty() {
            RestoreResult::Restored
        } else {
            RestoreResult::PartiallyRestored
        }
    }

    pub const fn hard_delete_eligible(&self) -> bool {
        self.restorable() || matches!(self.state, ArchiveState::Expired)
    }
}

/// Inspectable archive-storage contract for one cold record.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ArchiveStorageContract {
    pub memory_id: MemoryId,
    pub namespace: NamespaceId,
    pub archive_state: &'static str,
    pub archive_reason: Option<&'static str>,
    pub content_ref_present: bool,
    pub payload_ref_present: bool,
    pub payload_state: &'static str,
    pub recall_visible: bool,
    pub restorable: bool,
    pub inspectable_without_payload_fetch: bool,
    pub legal_hold: bool,
    pub hard_delete_eligible: bool,
    pub hard_deleted: bool,
    pub deletion_authority: Option<&'static str>,
    pub requested_by: Option<&'static str>,
    pub destructive_confirmation_required: bool,
    pub loss_indicators: Vec<LossIndicator>,
}

/// Ordered steps in the explicit restore workflow.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum RestoreStep {
    ValidateRestoreRequest,
    LoadDurableMetadata,
    RehydrateAvailablePayload,
    CommitDurableState,
    RefreshDerivedState,
}

impl RestoreStep {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::ValidateRestoreRequest => "validate_restore_request",
            Self::LoadDurableMetadata => "load_durable_metadata",
            Self::RehydrateAvailablePayload => "rehydrate_available_payload",
            Self::CommitDurableState => "commit_durable_state",
            Self::RefreshDerivedState => "refresh_derived_state",
        }
    }
}

/// Result of a restore operation.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RestoreResult {
    Restored,
    PartiallyRestored,
    NotFound,
    NotRestorable,
}

impl RestoreResult {
    /// Returns the stable machine-readable restore outcome name.
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Restored => "restored",
            Self::PartiallyRestored => "partially_restored",
            Self::NotFound => "not_found",
            Self::NotRestorable => "not_restorable",
        }
    }
}

/// Inspectable restore workflow plan derived from one cold record.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RestorePlan {
    pub memory_id: MemoryId,
    pub namespace: NamespaceId,
    pub result: RestoreResult,
    pub prior_archive_state: &'static str,
    pub target_lifecycle_state: Option<&'static str>,
    pub target_tier: Option<&'static str>,
    pub archive_reason: Option<&'static str>,
    pub payload_state: &'static str,
    pub partial: bool,
    pub loss_indicators: Vec<LossIndicator>,
    pub steps: Vec<RestoreStep>,
}

/// Stable authority classes for the dedicated irreversible deletion lane.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum DeletionAuthority {
    Policy,
    Compliance,
    RetentionExpiry,
}

impl DeletionAuthority {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Policy => "policy_authorized",
            Self::Compliance => "compliance_authorized",
            Self::RetentionExpiry => "retention_expiry",
        }
    }
}

/// Stable denial reasons for the dedicated irreversible deletion lane.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum HardDeleteDenialReason {
    PolicyDenied,
    LegalHold,
    MissingAuthority,
    MissingRequester,
    ConfirmationRequired,
    ActiveState,
}

impl HardDeleteDenialReason {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::PolicyDenied => "policy_denied",
            Self::LegalHold => "legal_hold",
            Self::MissingAuthority => "missing_authority",
            Self::MissingRequester => "missing_requester",
            Self::ConfirmationRequired => "confirmation_required",
            Self::ActiveState => "active_state_forbidden",
        }
    }
}

/// Explicit cold-tier audit artifact for a hard-delete request.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HardDeleteAuditTrail {
    pub outcome_class: OutcomeClass,
    pub request_id: &'static str,
    pub actor_source: &'static str,
    pub related_run: &'static str,
    pub detail: String,
    pub redacted: bool,
}

/// Machine-readable hard-delete decision scoped to the cold-store boundary.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HardDeletePlan {
    pub memory_id: MemoryId,
    pub namespace: NamespaceId,
    pub allowed: bool,
    pub policy_surface: &'static str,
    pub authority: Option<&'static str>,
    pub requested_by: Option<&'static str>,
    pub prior_archive_state: &'static str,
    pub resulting_archive_state: &'static str,
    pub payload_state: &'static str,
    pub loss_indicators: Vec<LossIndicator>,
    pub denial_reasons: Vec<HardDeleteDenialReason>,
    pub safeguard_related_run: Option<&'static str>,
    pub audit: HardDeleteAuditTrail,
}

/// Cold/archive store boundary owned by `membrain-core`.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct ColdStore;

impl ColdStore {
    /// Returns the stable component identifier.
    pub const fn component_name_inner(&self) -> &'static str {
        "store.cold"
    }

    fn hard_delete_detail(
        &self,
        record: &ColdRecord,
        allowed: bool,
        denial_reasons: &[HardDeleteDenialReason],
    ) -> String {
        if allowed {
            format!(
                "hard delete authorized for memory {} by {} via {}",
                record.memory_id.0,
                record.requested_by.unwrap_or("unknown_requester"),
                record
                    .deletion_authority
                    .map(DeletionAuthority::as_str)
                    .unwrap_or("missing_authority"),
            )
        } else {
            let reasons = denial_reasons
                .iter()
                .map(|reason| reason.as_str())
                .collect::<Vec<_>>()
                .join(",");
            format!(
                "hard delete denied for memory {} ({})",
                record.memory_id.0, reasons,
            )
        }
    }

    /// Returns the concrete archive-storage contract for one cold record.
    pub fn archive_contract(&self, record: &ColdRecord) -> ArchiveStorageContract {
        ArchiveStorageContract {
            memory_id: record.memory_id,
            namespace: record.namespace.clone(),
            archive_state: record.state.as_str(),
            archive_reason: record.archive_reason.map(ArchiveReason::as_str),
            content_ref_present: record.content_ref.is_some(),
            payload_ref_present: record.payload_ref.is_some(),
            payload_state: record.payload_state.as_str(),
            recall_visible: record.recall_visible(),
            restorable: record.restorable(),
            inspectable_without_payload_fetch: record.inspectable_without_payload_fetch(),
            legal_hold: record.legal_hold,
            hard_delete_eligible: record.hard_delete_eligible(),
            hard_deleted: record.state.hard_deleted(),
            deletion_authority: record.deletion_authority.map(DeletionAuthority::as_str),
            requested_by: record.requested_by,
            destructive_confirmation_required: true,
            loss_indicators: record.restore_loss_indicators(),
        }
    }

    /// Plans an explicit restore without mutating durable state.
    pub fn plan_restore(&self, record: &ColdRecord) -> RestorePlan {
        let result = record.restore_result();
        let partial = matches!(result, RestoreResult::PartiallyRestored);
        let steps = if record.restorable() {
            vec![
                RestoreStep::ValidateRestoreRequest,
                RestoreStep::LoadDurableMetadata,
                RestoreStep::RehydrateAvailablePayload,
                RestoreStep::CommitDurableState,
                RestoreStep::RefreshDerivedState,
            ]
        } else {
            vec![
                RestoreStep::ValidateRestoreRequest,
                RestoreStep::LoadDurableMetadata,
            ]
        };

        RestorePlan {
            memory_id: record.memory_id,
            namespace: record.namespace.clone(),
            result,
            prior_archive_state: record.state.as_str(),
            target_lifecycle_state: record.restorable().then_some("labile"),
            target_tier: record.restorable().then_some("tier1"),
            archive_reason: record.archive_reason.map(ArchiveReason::as_str),
            payload_state: record.payload_state.as_str(),
            partial,
            loss_indicators: record.restore_loss_indicators(),
            steps,
        }
    }

    /// Plans the dedicated irreversible hard-delete lane without mutating durable state.
    pub fn plan_hard_delete(
        &self,
        record: &ColdRecord,
        request_id: &'static str,
        policy: &impl PolicyGateway,
    ) -> HardDeletePlan {
        let mut denial_reasons = Vec::new();
        if record.legal_hold {
            denial_reasons.push(HardDeleteDenialReason::LegalHold);
        }
        if record.deletion_authority.is_none() {
            denial_reasons.push(HardDeleteDenialReason::MissingAuthority);
        }
        if record.requested_by.is_none() {
            denial_reasons.push(HardDeleteDenialReason::MissingRequester);
        }
        if !record.destructive_confirmation {
            denial_reasons.push(HardDeleteDenialReason::ConfirmationRequired);
        }
        if !record.hard_delete_eligible() {
            denial_reasons.push(HardDeleteDenialReason::ActiveState);
        }

        let safeguard = policy.evaluate_safeguard(SafeguardRequest {
            operation_class: OperationClass::IrreversibleMutation,
            preview_only: false,
            namespace_bound: true,
            policy_allowed: denial_reasons.is_empty(),
            requires_confirmation: true,
            local_confirmation: record.destructive_confirmation && denial_reasons.is_empty(),
            force_allowed: false,
            generation_bound: Some(record.memory_id.0),
            generation_matches: true,
            snapshot_required: true,
            snapshot_available: record.hard_delete_eligible(),
            maintenance_window_required: false,
            maintenance_window_active: true,
            dependencies_ready: true,
            scope_precise: true,
            authoritative_input_readable: true,
            confidence_ready: true,
            can_degrade: false,
            legal_hold: record.legal_hold,
        });

        if safeguard.policy_summary.decision != crate::policy::PolicyDecision::Allow
            && !denial_reasons.contains(&HardDeleteDenialReason::PolicyDenied)
            && !record.legal_hold
        {
            denial_reasons.push(HardDeleteDenialReason::PolicyDenied);
        }

        let allowed = denial_reasons.is_empty();
        let detail = self.hard_delete_detail(record, allowed, &denial_reasons);
        let audit = HardDeleteAuditTrail {
            outcome_class: if allowed {
                OutcomeClass::Accepted
            } else {
                safeguard.outcome_class
            },
            request_id,
            actor_source: "cold_store_hard_delete",
            related_run: safeguard
                .audit
                .related_run
                .unwrap_or("irreversible-mutation-run"),
            detail,
            redacted: !allowed,
        };

        HardDeletePlan {
            memory_id: record.memory_id,
            namespace: record.namespace.clone(),
            allowed,
            policy_surface: "hard_delete",
            authority: record.deletion_authority.map(DeletionAuthority::as_str),
            requested_by: record.requested_by,
            prior_archive_state: record.state.as_str(),
            resulting_archive_state: if allowed {
                ArchiveState::HardDeleted.as_str()
            } else {
                record.state.as_str()
            },
            payload_state: if allowed {
                ColdPayloadState::Tombstoned(TombstoneReason::HardDeleted).as_str()
            } else {
                record.payload_state.as_str()
            },
            loss_indicators: if allowed {
                vec![LossIndicator::tombstone(
                    LossSurface::Payload,
                    TombstoneReason::HardDeleted,
                )]
            } else {
                record.restore_loss_indicators()
            },
            denial_reasons,
            safeguard_related_run: safeguard.audit.related_run,
            audit,
        }
    }
}

impl ColdStoreApi for ColdStore {
    fn component_name(&self) -> &'static str {
        "store.cold"
    }
}

impl ColdStore {
    pub fn restore_plan_for(&self, record: &ColdRecord) -> RestorePlan {
        self.plan_restore(record)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn record(state: ArchiveState) -> ColdRecord {
        ColdRecord {
            memory_id: MemoryId(7),
            namespace: NamespaceId::new("test.namespace").unwrap(),
            state,
            compact_text: "cold summary".to_string(),
            original_tier: "hot",
            content_ref: Some("content://test.namespace/7".to_string()),
            payload_ref: Some("payload://test.namespace/7".to_string()),
            archive_reason: Some(ArchiveReason::UtilityForgetting),
            payload_state: ColdPayloadState::Detached,
            loss_indicators: Vec::new(),
            legal_hold: false,
            deletion_authority: None,
            requested_by: None,
            destructive_confirmation: false,
        }
    }

    #[test]
    fn archive_states_have_correct_restorability() {
        assert!(ArchiveState::Archived.restorable());
        assert!(ArchiveState::Forgotten.restorable());
        assert!(!ArchiveState::Expired.restorable());
        assert!(!ArchiveState::HardDeleted.restorable());
        assert!(ArchiveState::HardDeleted.hard_deleted());
    }

    #[test]
    fn cold_record_visibility_and_restore_contract_follow_archive_state() {
        let archived = record(ArchiveState::Archived);
        assert!(!archived.recall_visible());
        assert!(archived.restorable());
        assert_eq!(archived.restore_result(), RestoreResult::Restored);

        let forgotten = record(ArchiveState::Forgotten);
        assert!(forgotten.recall_visible());
        assert!(forgotten.restorable());
        assert_eq!(forgotten.restore_result(), RestoreResult::Restored);

        let expired = record(ArchiveState::Expired);
        assert!(!expired.recall_visible());
        assert!(!expired.restorable());
        assert_eq!(expired.restore_result(), RestoreResult::NotRestorable);
    }

    #[test]
    fn archive_contract_surfaces_concrete_storage_shape() {
        let store = ColdStore;
        let contract = store.archive_contract(&record(ArchiveState::Archived));

        assert_eq!(contract.archive_state, "archived");
        assert_eq!(contract.archive_reason, Some("utility_forgetting_archive"));
        assert_eq!(contract.payload_state, "detached");
        assert!(contract.content_ref_present);
        assert!(contract.payload_ref_present);
        assert!(contract.inspectable_without_payload_fetch);
        assert!(contract.restorable);
        assert!(!contract.legal_hold);
        assert!(contract.hard_delete_eligible);
        assert!(!contract.hard_deleted);
        assert_eq!(contract.deletion_authority, None);
        assert_eq!(contract.requested_by, None);
        assert!(contract.destructive_confirmation_required);
        assert!(contract.loss_indicators.is_empty());
    }

    #[test]
    fn restore_plan_commits_durable_state_before_refreshing_derived_surfaces() {
        let store = ColdStore;
        let plan = store.plan_restore(&record(ArchiveState::Archived));

        assert_eq!(plan.result, RestoreResult::Restored);
        assert_eq!(plan.prior_archive_state, "archived");
        assert_eq!(plan.target_lifecycle_state, Some("labile"));
        assert_eq!(plan.target_tier, Some("tier1"));
        assert!(!plan.partial);
        assert_eq!(
            plan.steps
                .iter()
                .map(|step| step.as_str())
                .collect::<Vec<_>>(),
            vec![
                "validate_restore_request",
                "load_durable_metadata",
                "rehydrate_available_payload",
                "commit_durable_state",
                "refresh_derived_state",
            ]
        );

        let commit_index = plan
            .steps
            .iter()
            .position(|step| *step == RestoreStep::CommitDurableState)
            .unwrap();
        let refresh_index = plan
            .steps
            .iter()
            .position(|step| *step == RestoreStep::RefreshDerivedState)
            .unwrap();
        assert!(commit_index < refresh_index);
    }

    #[test]
    fn tombstoned_payloads_surface_partial_restore_instead_of_fabricating_full_payload() {
        let store = ColdStore;
        let mut archived = record(ArchiveState::Archived);
        archived.payload_state = ColdPayloadState::Tombstoned(TombstoneReason::RetentionPurge);
        archived.payload_ref = None;

        let plan = store.plan_restore(&archived);

        assert_eq!(plan.result, RestoreResult::PartiallyRestored);
        assert!(plan.partial);
        assert_eq!(
            plan.loss_indicators,
            vec![LossIndicator::tombstone(
                LossSurface::Payload,
                TombstoneReason::RetentionPurge,
            )]
        );
    }

    #[test]
    fn missing_detached_payload_ref_surfaces_explicit_loss_indicator() {
        let store = ColdStore;
        let mut archived = record(ArchiveState::Archived);
        archived.payload_ref = None;

        let plan = store.plan_restore(&archived);

        assert_eq!(plan.result, RestoreResult::PartiallyRestored);
        assert!(plan.partial);
        assert_eq!(
            plan.loss_indicators,
            vec![LossIndicator::unavailable(
                LossSurface::Payload,
                LossReason::DetachedPayloadMissing,
            )]
        );
    }

    #[test]
    fn restore_result_names_are_stable() {
        assert_eq!(RestoreResult::Restored.as_str(), "restored");
        assert_eq!(
            RestoreResult::PartiallyRestored.as_str(),
            "partially_restored"
        );
        assert_eq!(RestoreResult::NotFound.as_str(), "not_found");
        assert_eq!(RestoreResult::NotRestorable.as_str(), "not_restorable");
    }

    #[test]
    fn hard_delete_denies_legal_hold_and_preserves_archive_state() {
        let store = ColdStore;
        let mut archived = record(ArchiveState::Archived);
        archived.legal_hold = true;
        archived.deletion_authority = Some(DeletionAuthority::Compliance);
        archived.requested_by = Some("agent.retention");
        archived.destructive_confirmation = true;

        let plan = store.plan_hard_delete(
            &archived,
            "req-hard-delete-hold",
            &crate::policy::PolicyModule,
        );

        assert!(!plan.allowed);
        assert_eq!(plan.policy_surface, "hard_delete");
        assert_eq!(plan.prior_archive_state, "archived");
        assert_eq!(plan.resulting_archive_state, "archived");
        assert!(plan
            .denial_reasons
            .contains(&HardDeleteDenialReason::LegalHold));
        assert_eq!(plan.audit.request_id, "req-hard-delete-hold");
        assert_eq!(plan.audit.actor_source, "cold_store_hard_delete");
        assert!(plan.audit.redacted);
    }

    #[test]
    fn hard_delete_requires_explicit_authority_requester_and_confirmation() {
        let store = ColdStore;
        let archived = record(ArchiveState::Archived);

        let plan = store.plan_hard_delete(
            &archived,
            "req-hard-delete-missing",
            &crate::policy::PolicyModule,
        );

        assert!(!plan.allowed);
        assert!(plan
            .denial_reasons
            .contains(&HardDeleteDenialReason::MissingAuthority));
        assert!(plan
            .denial_reasons
            .contains(&HardDeleteDenialReason::MissingRequester));
        assert!(plan
            .denial_reasons
            .contains(&HardDeleteDenialReason::ConfirmationRequired));
        assert_eq!(plan.resulting_archive_state, "archived");
        assert_eq!(plan.payload_state, "detached");
    }

    #[test]
    fn hard_delete_requires_non_active_archive_state() {
        let store = ColdStore;
        let mut deleted = record(ArchiveState::HardDeleted);
        deleted.deletion_authority = Some(DeletionAuthority::Policy);
        deleted.requested_by = Some("operator.cli");
        deleted.destructive_confirmation = true;

        let plan = store.plan_hard_delete(
            &deleted,
            "req-hard-delete-active",
            &crate::policy::PolicyModule,
        );

        assert!(!plan.allowed);
        assert!(plan
            .denial_reasons
            .contains(&HardDeleteDenialReason::ActiveState));
    }

    #[test]
    fn hard_delete_allowed_path_surfaces_irreversible_tombstone_and_audit() {
        let store = ColdStore;
        let mut archived = record(ArchiveState::Expired);
        archived.deletion_authority = Some(DeletionAuthority::RetentionExpiry);
        archived.requested_by = Some("retention.job");
        archived.destructive_confirmation = true;

        let plan = store.plan_hard_delete(
            &archived,
            "req-hard-delete-ok",
            &crate::policy::PolicyModule,
        );

        assert!(plan.allowed);
        assert_eq!(plan.authority, Some("retention_expiry"));
        assert_eq!(plan.requested_by, Some("retention.job"));
        assert_eq!(plan.prior_archive_state, "expired");
        assert_eq!(plan.resulting_archive_state, "hard_deleted");
        assert_eq!(plan.payload_state, "tombstoned");
        assert_eq!(
            plan.loss_indicators,
            vec![LossIndicator::tombstone(
                LossSurface::Payload,
                TombstoneReason::HardDeleted,
            )]
        );
        assert_eq!(plan.audit.outcome_class, OutcomeClass::Accepted);
        assert_eq!(plan.audit.related_run, "irreversible-mutation-run");
        assert!(!plan.audit.redacted);
        assert!(plan.audit.detail.contains("retention.job"));
    }

    #[test]
    fn archive_and_loss_labels_are_stable() {
        assert_eq!(
            ArchiveReason::UtilityForgetting.as_str(),
            "utility_forgetting_archive"
        );
        assert_eq!(
            TombstoneReason::PolicyRedaction.as_str(),
            "policy_redaction"
        );
        assert_eq!(TombstoneReason::HardDeleted.as_str(), "hard_deleted");
        assert_eq!(
            LossReason::DerivedStateStale.as_str(),
            "derived_state_stale"
        );
        assert_eq!(LossSurface::DerivedState.as_str(), "derived_state");
        assert_eq!(LossKind::Tombstoned.as_str(), "tombstoned");
        assert_eq!(ColdPayloadState::Detached.as_str(), "detached");
        assert_eq!(DeletionAuthority::Policy.as_str(), "policy_authorized");
        assert_eq!(
            HardDeleteDenialReason::ConfirmationRequired.as_str(),
            "confirmation_required"
        );
        assert_eq!(
            RestoreStep::CommitDurableState.as_str(),
            "commit_durable_state"
        );
    }
}
