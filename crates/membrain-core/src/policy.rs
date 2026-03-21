use crate::observability::OutcomeClass;

/// Effective policy decision shared across core APIs and wrappers.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize)]
pub enum PolicyDecision {
    /// The request may proceed to bounded planning or retrieval.
    Allow,
    /// The request must stop before expensive work begins.
    Deny,
}

/// Canonical ingest modes that remain inspectable across intake surfaces.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IngestMode {
    Active,
    PassiveObservation,
}

/// Machine-readable passive-observation write outcome.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PassiveObservationDecision {
    Capture,
    Suppress,
    Deny,
}

impl PassiveObservationDecision {
    /// Returns the stable machine-readable passive-observation decision label.
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Capture => "capture",
            Self::Suppress => "suppress",
            Self::Deny => "deny",
        }
    }
}

/// Canonical visibility for namespace-aware sharing.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, serde::Serialize)]
pub enum SharingVisibility {
    #[default]
    Private,
    Shared,
    Public,
}

impl SharingVisibility {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Private => "private",
            Self::Shared => "shared",
            Self::Public => "public",
        }
    }
}

/// Machine-readable access scope granted after sharing mediation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize)]
pub enum SharingScope {
    NamespaceOnly,
    ApprovedShared,
    ApprovedPublic,
}

impl SharingScope {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::NamespaceOnly => "namespace_only",
            Self::ApprovedShared => "approved_shared",
            Self::ApprovedPublic => "approved_public",
        }
    }
}

/// Stable denial reasons for namespace-aware sharing access.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize)]
pub enum SharingDenialReason {
    NamespaceIsolation,
    ApprovedScopeRequired,
    VisibilityNotShareable,
    WorkspaceAclDenied,
    AgentAclDenied,
    SessionVisibilityDenied,
    LegalHold,
}

impl SharingDenialReason {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::NamespaceIsolation => "namespace_isolation",
            Self::ApprovedScopeRequired => "approved_scope_required",
            Self::VisibilityNotShareable => "visibility_not_shareable",
            Self::WorkspaceAclDenied => "workspace_acl_denied",
            Self::AgentAclDenied => "agent_acl_denied",
            Self::SessionVisibilityDenied => "session_visibility_denied",
            Self::LegalHold => "legal_hold",
        }
    }
}

/// Final mediation outcome for namespace-aware sharing access.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize)]
pub enum SharingAccessDecision {
    Allow,
    Redact,
    Deny,
}

/// Machine-readable summary of the policy gate that fired before expensive work.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize)]
pub struct PolicySummary {
    /// Final allow/deny decision for the preflight gate.
    pub decision: PolicyDecision,
    /// Whether namespace binding succeeded before the request was evaluated.
    pub namespace_bound: bool,
    /// Canonical outcome class wrappers should preserve in traces and envelopes.
    pub outcome_class: OutcomeClass,
}

impl PolicySummary {
    /// Builds an allow summary for a namespace-bound request.
    pub const fn allow(namespace_bound: bool) -> Self {
        Self {
            decision: PolicyDecision::Allow,
            namespace_bound,
            outcome_class: OutcomeClass::Accepted,
        }
    }

    /// Builds a deny summary for a request that must stop before expensive work.
    pub const fn deny(namespace_bound: bool) -> Self {
        Self {
            decision: PolicyDecision::Deny,
            namespace_bound,
            outcome_class: OutcomeClass::Rejected,
        }
    }
}

/// Shared safeguard classes for risky or high-blast-radius operations.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize)]
pub enum OperationClass {
    ReadOnlyAssessment,
    DerivedSurfaceMutation,
    AuthoritativeRewrite,
    IrreversibleMutation,
    ContradictionArchive,
}

/// Shared preflight readiness states preserved across interfaces.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize)]
pub enum PreflightState {
    Ready,
    Blocked,
    MissingData,
    StaleKnowledge,
}

/// Machine-readable per-check status preserved in safeguard outputs.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize)]
pub enum PreflightCheckStatus {
    Passed,
    Blocked,
    Degraded,
    Rejected,
}

/// Stable blocked or rejected reason codes for preflight and safeguard flows.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize)]
pub enum SafeguardReasonCode {
    ConfirmationRequired,
    StalePreflight,
    GenerationMismatch,
    SnapshotRequired,
    MaintenanceWindowRequired,
    DependencyPending,
    ScopeAmbiguous,
    AuthoritativeInputUnreadable,
    ConfidenceTooLow,
    PolicyDenied,
    LegalHold,
}

/// Recovery posture for risky operations.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize)]
pub enum ReversibilityKind {
    RepairableFromDurableTruth,
    RollbackViaSnapshot,
    PartiallyReversible,
    Irreversible,
}

/// Shared readiness check result preserved in the safeguard object.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
pub struct PreflightCheck {
    pub check_name: &'static str,
    pub status: PreflightCheckStatus,
    pub reason_codes: Vec<SafeguardReasonCode>,
    pub checked_scope: &'static str,
}

/// Shared write-gating request for passive observation.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ObservationWriteRequest {
    pub ingest_mode: IngestMode,
    pub namespace_bound: bool,
    pub policy_allowed: bool,
    pub duplicate_hint: bool,
}

/// Machine-readable passive-observation gate result.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ObservationWriteOutcome {
    pub decision: PassiveObservationDecision,
    pub policy_summary: PolicySummary,
    pub captured_as_observation: bool,
}

/// Input facts for namespace-aware sharing mediation.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SharingAccessRequest {
    pub same_namespace: bool,
    pub include_public: bool,
    pub visibility: SharingVisibility,
    pub workspace_acl_allowed: bool,
    pub agent_acl_allowed: bool,
    pub session_visibility_allowed: bool,
    pub legal_hold: bool,
}

/// Machine-readable outcome for namespace-aware sharing mediation.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
pub struct SharingAccessOutcome {
    pub decision: SharingAccessDecision,
    pub policy_summary: PolicySummary,
    pub sharing_scope: Option<SharingScope>,
    pub denial_reasons: Vec<SharingDenialReason>,
    pub redaction_fields: Vec<&'static str>,
}

/// Shared confirmation state for previewed risky operations.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize)]
pub struct ConfirmationState {
    pub required: bool,
    pub force_allowed: bool,
    pub confirmed: bool,
    pub generation_bound: Option<u64>,
}

/// Shared confidence threshold metadata for high-stakes or action-oriented guidance.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
pub struct ConfidenceConstraint {
    pub minimum_level: &'static str,
    pub change_my_mind_conditions: Vec<&'static str>,
}

/// Stable audit correlation payload for safeguard preview, apply, and rollback review.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
pub struct SafeguardAudit {
    pub event_kind: &'static str,
    pub actor_source: &'static str,
    pub request_id: &'static str,
    pub preview_id: Option<&'static str>,
    pub related_run: Option<&'static str>,
    pub scope_handle: &'static str,
}

/// Shared safeguard payload reused by preview, blocked, degraded, rejected, and accepted flows.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
pub struct SafeguardOutcome {
    pub outcome_class: OutcomeClass,
    pub preflight_state: PreflightState,
    pub operation_class: OperationClass,
    pub affected_scope: &'static str,
    pub impact_summary: &'static str,
    pub blocked_reasons: Vec<SafeguardReasonCode>,
    pub preflight_checks: Vec<PreflightCheck>,
    pub check_results: Vec<PreflightCheck>,
    pub warnings: Vec<&'static str>,
    pub confidence_constraints: Option<ConfidenceConstraint>,
    pub reversibility: ReversibilityKind,
    pub confirmation: ConfirmationState,
    pub audit: SafeguardAudit,
    pub policy_summary: PolicySummary,
}

/// Input facts evaluated by the shared preflight safeguard machine.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SafeguardRequest {
    pub operation_class: OperationClass,
    pub preview_only: bool,
    pub namespace_bound: bool,
    pub policy_allowed: bool,
    pub requires_confirmation: bool,
    pub local_confirmation: bool,
    pub force_allowed: bool,
    pub generation_bound: Option<u64>,
    pub generation_matches: bool,
    pub snapshot_required: bool,
    pub snapshot_available: bool,
    pub maintenance_window_required: bool,
    pub maintenance_window_active: bool,
    pub dependencies_ready: bool,
    pub scope_precise: bool,
    pub authoritative_input_readable: bool,
    pub confidence_ready: bool,
    pub can_degrade: bool,
    pub legal_hold: bool,
}

impl SafeguardRequest {
    /// Builds a baseline request with all readiness checks passing.
    pub const fn ready(operation_class: OperationClass) -> Self {
        Self {
            operation_class,
            preview_only: false,
            namespace_bound: true,
            policy_allowed: true,
            requires_confirmation: false,
            local_confirmation: false,
            force_allowed: false,
            generation_bound: None,
            generation_matches: true,
            snapshot_required: false,
            snapshot_available: true,
            maintenance_window_required: false,
            maintenance_window_active: true,
            dependencies_ready: true,
            scope_precise: true,
            authoritative_input_readable: true,
            confidence_ready: true,
            can_degrade: false,
            legal_hold: false,
        }
    }
}

/// Stable core policy boundary that wrappers call instead of reimplementing policy behavior.
pub trait PolicyGateway {
    /// Evaluates the namespace gate before expensive work starts.
    fn evaluate_namespace(&self, namespace_bound: bool) -> PolicySummary;

    /// Evaluates namespace-aware sharing access before candidate generation or packaging.
    fn evaluate_sharing_access(&self, request: SharingAccessRequest) -> SharingAccessOutcome;

    /// Evaluates passive-observation write gating before intake persists anything.
    fn evaluate_observation_write(
        &self,
        request: ObservationWriteRequest,
    ) -> ObservationWriteOutcome;

    /// Evaluates the shared safeguard and preflight contract for risky operations.
    fn evaluate_safeguard(&self, request: SafeguardRequest) -> SafeguardOutcome;
}

/// Minimal shared policy module owned by `membrain-core`.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct PolicyModule;

impl PolicyModule {
    fn reversibility_for(operation_class: OperationClass) -> ReversibilityKind {
        match operation_class {
            OperationClass::ReadOnlyAssessment => ReversibilityKind::RepairableFromDurableTruth,
            OperationClass::DerivedSurfaceMutation => ReversibilityKind::RepairableFromDurableTruth,
            OperationClass::AuthoritativeRewrite => ReversibilityKind::RollbackViaSnapshot,
            OperationClass::IrreversibleMutation => ReversibilityKind::Irreversible,
            OperationClass::ContradictionArchive => ReversibilityKind::RollbackViaSnapshot,
        }
    }

    fn sharing_scope(request: SharingAccessRequest) -> Option<SharingScope> {
        if request.same_namespace {
            Some(SharingScope::NamespaceOnly)
        } else {
            match request.visibility {
                SharingVisibility::Public if request.include_public => {
                    Some(SharingScope::ApprovedPublic)
                }
                SharingVisibility::Shared => Some(SharingScope::ApprovedShared),
                _ => None,
            }
        }
    }

    fn affected_scope(request: SafeguardRequest) -> &'static str {
        if request.namespace_bound {
            "effective_namespace"
        } else {
            "unbound_namespace"
        }
    }

    fn impact_summary(request: SafeguardRequest) -> &'static str {
        match request.operation_class {
            OperationClass::ReadOnlyAssessment => "read_only_assessment",
            OperationClass::DerivedSurfaceMutation => {
                if request.can_degrade {
                    "derived_surface_mutation_with_degraded_fallback"
                } else {
                    "derived_surface_mutation"
                }
            }
            OperationClass::AuthoritativeRewrite => {
                if request.maintenance_window_required {
                    "authoritative_rewrite_requires_window"
                } else {
                    "authoritative_rewrite"
                }
            }
            OperationClass::IrreversibleMutation => "irreversible_mutation",
            OperationClass::ContradictionArchive => "contradiction_archive",
        }
    }

    fn warning_messages(request: SafeguardRequest) -> Vec<&'static str> {
        let mut warnings = Vec::new();
        if !request.confidence_ready {
            warnings.push("low_confidence");
        }
        if !request.authoritative_input_readable {
            warnings.push("authoritative_input_unreadable");
        }
        if !request.generation_matches {
            warnings.push("stale_generation");
        }
        warnings
    }

    fn confidence_constraints(request: SafeguardRequest) -> Option<ConfidenceConstraint> {
        (!request.confidence_ready).then_some(ConfidenceConstraint {
            minimum_level: "high",
            change_my_mind_conditions: vec![
                "fresh_authoritative_inputs",
                "resolved_policy_scope",
                "stable_generation_anchor",
            ],
        })
    }

    fn audit(request: SafeguardRequest) -> SafeguardAudit {
        SafeguardAudit {
            event_kind: "safeguard_evaluation",
            actor_source: "core_policy",
            request_id: "policy-eval",
            preview_id: request.preview_only.then_some("preview"),
            related_run: match request.operation_class {
                OperationClass::ReadOnlyAssessment => None,
                OperationClass::DerivedSurfaceMutation => Some("derived-surface-run"),
                OperationClass::AuthoritativeRewrite => Some("authoritative-rewrite-run"),
                OperationClass::IrreversibleMutation => Some("irreversible-mutation-run"),
                OperationClass::ContradictionArchive => Some("contradiction-archive-run"),
            },
            scope_handle: Self::affected_scope(request),
        }
    }

    fn confirmation_state(request: SafeguardRequest) -> ConfirmationState {
        ConfirmationState {
            required: request.requires_confirmation,
            force_allowed: request.force_allowed,
            confirmed: request.local_confirmation,
            generation_bound: request.generation_bound,
        }
    }

    fn push_check(
        checks: &mut Vec<PreflightCheck>,
        check_name: &'static str,
        status: PreflightCheckStatus,
        checked_scope: &'static str,
        reason_codes: Vec<SafeguardReasonCode>,
    ) {
        checks.push(PreflightCheck {
            check_name,
            status,
            checked_scope,
            reason_codes,
        });
    }
}

impl PolicyGateway for PolicyModule {
    fn evaluate_namespace(&self, namespace_bound: bool) -> PolicySummary {
        if namespace_bound {
            PolicySummary::allow(true)
        } else {
            PolicySummary::deny(false)
        }
    }

    fn evaluate_sharing_access(&self, request: SharingAccessRequest) -> SharingAccessOutcome {
        let mut denial_reasons = Vec::new();

        if request.legal_hold && !request.same_namespace {
            denial_reasons.push(SharingDenialReason::LegalHold);
        }
        if !request.workspace_acl_allowed {
            denial_reasons.push(SharingDenialReason::WorkspaceAclDenied);
        }
        if !request.agent_acl_allowed {
            denial_reasons.push(SharingDenialReason::AgentAclDenied);
        }
        if !request.session_visibility_allowed {
            denial_reasons.push(SharingDenialReason::SessionVisibilityDenied);
        }

        let sharing_scope = Self::sharing_scope(request);
        if !request.same_namespace && sharing_scope.is_none() {
            denial_reasons.push(SharingDenialReason::NamespaceIsolation);
            denial_reasons.push(match request.visibility {
                SharingVisibility::Private => SharingDenialReason::VisibilityNotShareable,
                SharingVisibility::Public => SharingDenialReason::ApprovedScopeRequired,
                SharingVisibility::Shared => SharingDenialReason::NamespaceIsolation,
            });
        }

        let policy_summary = if denial_reasons.is_empty() {
            PolicySummary::allow(true)
        } else {
            PolicySummary::deny(true)
        };

        if !denial_reasons.is_empty() {
            let redaction_fields = if request.same_namespace {
                vec!["sharing_scope"]
            } else {
                vec!["memory_id", "sharing_scope", "workspace_id", "session_id"]
            };
            return SharingAccessOutcome {
                decision: SharingAccessDecision::Deny,
                policy_summary,
                sharing_scope: None,
                denial_reasons,
                redaction_fields,
            };
        }

        let Some(sharing_scope) = sharing_scope else {
            return SharingAccessOutcome {
                decision: SharingAccessDecision::Deny,
                policy_summary: PolicySummary::deny(true),
                sharing_scope: None,
                denial_reasons: vec![SharingDenialReason::NamespaceIsolation],
                redaction_fields: vec!["memory_id", "sharing_scope", "workspace_id", "session_id"],
            };
        };

        let redaction_fields = if request.same_namespace {
            Vec::new()
        } else {
            vec!["workspace_id", "session_id"]
        };
        let decision = if redaction_fields.is_empty() {
            SharingAccessDecision::Allow
        } else {
            SharingAccessDecision::Redact
        };

        SharingAccessOutcome {
            decision,
            policy_summary,
            sharing_scope: Some(sharing_scope),
            denial_reasons,
            redaction_fields,
        }
    }

    fn evaluate_observation_write(
        &self,
        request: ObservationWriteRequest,
    ) -> ObservationWriteOutcome {
        let policy_summary = if request.namespace_bound && request.policy_allowed {
            PolicySummary::allow(request.namespace_bound)
        } else {
            PolicySummary::deny(request.namespace_bound)
        };

        let decision = match request.ingest_mode {
            IngestMode::Active => {
                if !request.namespace_bound || !request.policy_allowed {
                    PassiveObservationDecision::Deny
                } else {
                    PassiveObservationDecision::Capture
                }
            }
            IngestMode::PassiveObservation => {
                if !request.namespace_bound || !request.policy_allowed {
                    PassiveObservationDecision::Deny
                } else if request.duplicate_hint {
                    PassiveObservationDecision::Suppress
                } else {
                    PassiveObservationDecision::Capture
                }
            }
        };

        ObservationWriteOutcome {
            decision,
            policy_summary,
            captured_as_observation: matches!(
                (request.ingest_mode, decision),
                (
                    IngestMode::PassiveObservation,
                    PassiveObservationDecision::Capture
                )
            ),
        }
    }

    fn evaluate_safeguard(&self, request: SafeguardRequest) -> SafeguardOutcome {
        let policy_summary =
            if request.namespace_bound && request.policy_allowed && !request.legal_hold {
                PolicySummary::allow(request.namespace_bound)
            } else {
                PolicySummary::deny(request.namespace_bound)
            };

        let confirmation = Self::confirmation_state(request);
        let reversibility = Self::reversibility_for(request.operation_class);
        let mut blocked_reasons = Vec::new();
        let mut preflight_checks = Vec::new();

        let policy_reason = if request.legal_hold {
            Some(SafeguardReasonCode::LegalHold)
        } else if !request.policy_allowed || !request.namespace_bound {
            Some(SafeguardReasonCode::PolicyDenied)
        } else {
            None
        };
        Self::push_check(
            &mut preflight_checks,
            "policy",
            if policy_reason.is_some() {
                PreflightCheckStatus::Rejected
            } else {
                PreflightCheckStatus::Passed
            },
            "effective_namespace",
            policy_reason.into_iter().collect(),
        );

        let generation_reason = if !request.generation_matches {
            Some(if request.generation_bound.is_some() {
                SafeguardReasonCode::GenerationMismatch
            } else {
                SafeguardReasonCode::StalePreflight
            })
        } else {
            None
        };
        Self::push_check(
            &mut preflight_checks,
            "generation",
            if generation_reason.is_some() {
                PreflightCheckStatus::Blocked
            } else {
                PreflightCheckStatus::Passed
            },
            "preview_generation",
            generation_reason.into_iter().collect(),
        );

        let input_reason = if request.snapshot_required && !request.snapshot_available {
            Some(SafeguardReasonCode::SnapshotRequired)
        } else if !request.scope_precise {
            Some(SafeguardReasonCode::ScopeAmbiguous)
        } else {
            None
        };
        Self::push_check(
            &mut preflight_checks,
            "required_input",
            if input_reason.is_some() {
                PreflightCheckStatus::Blocked
            } else {
                PreflightCheckStatus::Passed
            },
            "request_scope",
            input_reason.into_iter().collect(),
        );

        let dependency_reason =
            if request.maintenance_window_required && !request.maintenance_window_active {
                Some(SafeguardReasonCode::MaintenanceWindowRequired)
            } else if !request.dependencies_ready {
                Some(SafeguardReasonCode::DependencyPending)
            } else {
                None
            };
        Self::push_check(
            &mut preflight_checks,
            "dependency",
            if dependency_reason.is_some() {
                PreflightCheckStatus::Blocked
            } else {
                PreflightCheckStatus::Passed
            },
            "operation_dependencies",
            dependency_reason.into_iter().collect(),
        );

        let contradiction_archive_reason = if request.operation_class
            == OperationClass::ContradictionArchive
            && !request.snapshot_available
        {
            Some(SafeguardReasonCode::SnapshotRequired)
        } else if request.operation_class == OperationClass::ContradictionArchive
            && !request.scope_precise
        {
            Some(SafeguardReasonCode::ScopeAmbiguous)
        } else if request.operation_class == OperationClass::ContradictionArchive
            && request.legal_hold
        {
            Some(SafeguardReasonCode::LegalHold)
        } else {
            None
        };
        Self::push_check(
            &mut preflight_checks,
            "contradiction_archive",
            if contradiction_archive_reason.is_some() {
                PreflightCheckStatus::Blocked
            } else {
                PreflightCheckStatus::Passed
            },
            "contradiction_records",
            contradiction_archive_reason.into_iter().collect(),
        );

        let confidence_reason =
            (!request.confidence_ready).then_some(SafeguardReasonCode::ConfidenceTooLow);
        Self::push_check(
            &mut preflight_checks,
            "confidence",
            if confidence_reason.is_some() {
                if request.can_degrade {
                    PreflightCheckStatus::Degraded
                } else {
                    PreflightCheckStatus::Blocked
                }
            } else {
                PreflightCheckStatus::Passed
            },
            "confidence_window",
            confidence_reason.into_iter().collect(),
        );

        let readable_reason = (!request.authoritative_input_readable)
            .then_some(SafeguardReasonCode::AuthoritativeInputUnreadable);
        Self::push_check(
            &mut preflight_checks,
            "authoritative_input",
            if readable_reason.is_some() {
                if request.can_degrade {
                    PreflightCheckStatus::Degraded
                } else {
                    PreflightCheckStatus::Blocked
                }
            } else {
                PreflightCheckStatus::Passed
            },
            "authoritative_inputs",
            readable_reason.into_iter().collect(),
        );

        if let Some(reason) = policy_reason {
            blocked_reasons.push(reason);
            return SafeguardOutcome {
                outcome_class: OutcomeClass::Rejected,
                preflight_state: PreflightState::Blocked,
                operation_class: request.operation_class,
                affected_scope: Self::affected_scope(request),
                impact_summary: Self::impact_summary(request),
                blocked_reasons,
                preflight_checks: preflight_checks.clone(),
                check_results: preflight_checks,
                warnings: Self::warning_messages(request),
                confidence_constraints: Self::confidence_constraints(request),
                reversibility,
                confirmation,
                audit: Self::audit(request),
                policy_summary,
            };
        }

        if let Some(reason) = generation_reason {
            blocked_reasons.push(reason);
        }
        if let Some(reason) = input_reason {
            blocked_reasons.push(reason);
        }
        if let Some(reason) = dependency_reason {
            blocked_reasons.push(reason);
        }
        if let Some(reason) = contradiction_archive_reason {
            blocked_reasons.push(reason);
        }
        if !request.confidence_ready && !request.can_degrade {
            blocked_reasons.push(SafeguardReasonCode::ConfidenceTooLow);
        }
        if !request.authoritative_input_readable && !request.can_degrade {
            blocked_reasons.push(SafeguardReasonCode::AuthoritativeInputUnreadable);
        }

        let preflight_state = if !request.generation_matches {
            PreflightState::StaleKnowledge
        } else if (request.snapshot_required && !request.snapshot_available)
            || !request.scope_precise
        {
            PreflightState::MissingData
        } else if blocked_reasons.is_empty() {
            PreflightState::Ready
        } else {
            PreflightState::Blocked
        };

        if request.preview_only {
            return SafeguardOutcome {
                outcome_class: OutcomeClass::Preview,
                preflight_state,
                operation_class: request.operation_class,
                affected_scope: Self::affected_scope(request),
                impact_summary: Self::impact_summary(request),
                blocked_reasons,
                preflight_checks: preflight_checks.clone(),
                check_results: preflight_checks,
                warnings: Self::warning_messages(request),
                confidence_constraints: Self::confidence_constraints(request),
                reversibility,
                confirmation,
                audit: Self::audit(request),
                policy_summary,
            };
        }

        if request.requires_confirmation && !request.local_confirmation {
            blocked_reasons.push(SafeguardReasonCode::ConfirmationRequired);
        }

        if !blocked_reasons.is_empty() {
            return SafeguardOutcome {
                outcome_class: OutcomeClass::Blocked,
                preflight_state: if request.requires_confirmation && !request.local_confirmation {
                    PreflightState::Blocked
                } else {
                    preflight_state
                },
                operation_class: request.operation_class,
                affected_scope: Self::affected_scope(request),
                impact_summary: Self::impact_summary(request),
                blocked_reasons,
                preflight_checks: preflight_checks.clone(),
                check_results: preflight_checks,
                warnings: Self::warning_messages(request),
                confidence_constraints: Self::confidence_constraints(request),
                reversibility,
                confirmation,
                audit: Self::audit(request),
                policy_summary,
            };
        }

        let outcome_class = if !request.confidence_ready || !request.authoritative_input_readable {
            OutcomeClass::Degraded
        } else {
            OutcomeClass::Accepted
        };

        SafeguardOutcome {
            outcome_class,
            preflight_state: PreflightState::Ready,
            operation_class: request.operation_class,
            affected_scope: Self::affected_scope(request),
            impact_summary: Self::impact_summary(request),
            blocked_reasons,
            preflight_checks: preflight_checks.clone(),
            check_results: preflight_checks,
            warnings: Self::warning_messages(request),
            confidence_constraints: Self::confidence_constraints(request),
            reversibility,
            confirmation,
            audit: Self::audit(request),
            policy_summary,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{
        IngestMode, ObservationWriteRequest, OperationClass, PassiveObservationDecision,
        PolicyDecision, PolicyGateway, PolicyModule, PreflightCheckStatus, PreflightState,
        ReversibilityKind, SafeguardReasonCode, SafeguardRequest, SharingAccessDecision,
        SharingAccessRequest, SharingDenialReason, SharingVisibility,
    };
    use crate::observability::OutcomeClass;

    #[test]
    fn sharing_access_allows_same_namespace_private_scope() {
        let gateway = PolicyModule;
        let outcome = gateway.evaluate_sharing_access(SharingAccessRequest {
            same_namespace: true,
            include_public: false,
            visibility: SharingVisibility::Private,
            workspace_acl_allowed: true,
            agent_acl_allowed: true,
            session_visibility_allowed: true,
            legal_hold: false,
        });

        assert_eq!(outcome.decision, SharingAccessDecision::Allow);
        assert_eq!(outcome.sharing_scope.unwrap().as_str(), "namespace_only");
        assert!(outcome.denial_reasons.is_empty());
        assert!(outcome.redaction_fields.is_empty());
    }

    #[test]
    fn sharing_access_denies_instead_of_panicking_when_allow_path_lacks_scope() {
        let gateway = PolicyModule;
        let outcome = gateway.evaluate_sharing_access(SharingAccessRequest {
            same_namespace: false,
            include_public: false,
            visibility: SharingVisibility::Public,
            workspace_acl_allowed: true,
            agent_acl_allowed: true,
            session_visibility_allowed: true,
            legal_hold: false,
        });

        assert_eq!(outcome.decision, SharingAccessDecision::Deny);
        assert_eq!(outcome.policy_summary.decision, PolicyDecision::Deny);
        assert_eq!(outcome.sharing_scope, None);
        assert_eq!(
            outcome.denial_reasons,
            vec![
                SharingDenialReason::NamespaceIsolation,
                SharingDenialReason::ApprovedScopeRequired,
            ]
        );
        assert_eq!(
            outcome.redaction_fields,
            vec!["memory_id", "sharing_scope", "workspace_id", "session_id"]
        );
    }

    #[test]
    fn sharing_access_denies_private_cross_namespace_visibility() {
        let gateway = PolicyModule;
        let outcome = gateway.evaluate_sharing_access(SharingAccessRequest {
            same_namespace: false,
            include_public: false,
            visibility: SharingVisibility::Private,
            workspace_acl_allowed: true,
            agent_acl_allowed: true,
            session_visibility_allowed: true,
            legal_hold: false,
        });

        assert_eq!(outcome.decision, SharingAccessDecision::Deny);
        assert!(outcome
            .denial_reasons
            .iter()
            .any(|reason| reason.as_str() == "namespace_isolation"));
        assert!(outcome
            .denial_reasons
            .iter()
            .any(|reason| reason.as_str() == "visibility_not_shareable"));
        assert_eq!(
            outcome.redaction_fields,
            vec!["memory_id", "sharing_scope", "workspace_id", "session_id"]
        );
    }

    #[test]
    fn sharing_access_redacts_shared_cross_namespace_visibility() {
        let gateway = PolicyModule;
        let outcome = gateway.evaluate_sharing_access(SharingAccessRequest {
            same_namespace: false,
            include_public: false,
            visibility: SharingVisibility::Shared,
            workspace_acl_allowed: true,
            agent_acl_allowed: true,
            session_visibility_allowed: true,
            legal_hold: false,
        });

        assert_eq!(outcome.decision, SharingAccessDecision::Redact);
        assert_eq!(outcome.sharing_scope.unwrap().as_str(), "approved_shared");
        assert_eq!(outcome.redaction_fields, vec!["workspace_id", "session_id"]);
    }

    #[test]
    fn sharing_access_requires_include_public_for_public_widening() {
        let gateway = PolicyModule;
        let outcome = gateway.evaluate_sharing_access(SharingAccessRequest {
            same_namespace: false,
            include_public: false,
            visibility: SharingVisibility::Public,
            workspace_acl_allowed: true,
            agent_acl_allowed: true,
            session_visibility_allowed: true,
            legal_hold: false,
        });

        assert_eq!(outcome.decision, SharingAccessDecision::Deny);
        assert!(outcome
            .denial_reasons
            .iter()
            .any(|reason| reason.as_str() == "approved_scope_required"));
    }

    #[test]
    fn sharing_access_denies_when_acl_or_visibility_guards_fail() {
        let gateway = PolicyModule;
        let outcome = gateway.evaluate_sharing_access(SharingAccessRequest {
            same_namespace: false,
            include_public: true,
            visibility: SharingVisibility::Public,
            workspace_acl_allowed: false,
            agent_acl_allowed: false,
            session_visibility_allowed: false,
            legal_hold: true,
        });

        assert_eq!(outcome.decision, SharingAccessDecision::Deny);
        assert!(outcome
            .denial_reasons
            .iter()
            .any(|reason| reason.as_str() == "workspace_acl_denied"));
        assert!(outcome
            .denial_reasons
            .iter()
            .any(|reason| reason.as_str() == "agent_acl_denied"));
        assert!(outcome
            .denial_reasons
            .iter()
            .any(|reason| reason.as_str() == "session_visibility_denied"));
        assert!(outcome
            .denial_reasons
            .iter()
            .any(|reason| reason.as_str() == "legal_hold"));
    }

    #[test]
    fn sharing_access_allows_same_namespace_private_reads_during_legal_hold() {
        let gateway = PolicyModule;
        let outcome = gateway.evaluate_sharing_access(SharingAccessRequest {
            same_namespace: true,
            include_public: false,
            visibility: SharingVisibility::Private,
            workspace_acl_allowed: true,
            agent_acl_allowed: true,
            session_visibility_allowed: true,
            legal_hold: true,
        });

        assert_eq!(outcome.decision, SharingAccessDecision::Allow);
        assert_eq!(outcome.sharing_scope.unwrap().as_str(), "namespace_only");
        assert!(outcome.denial_reasons.is_empty());
        assert!(outcome.redaction_fields.is_empty());
    }

    #[test]
    fn passive_observation_captures_when_policy_allows_and_no_duplicate_hint_exists() {
        let gateway = PolicyModule;
        let outcome = gateway.evaluate_observation_write(ObservationWriteRequest {
            ingest_mode: IngestMode::PassiveObservation,
            namespace_bound: true,
            policy_allowed: true,
            duplicate_hint: false,
        });

        assert_eq!(outcome.decision, PassiveObservationDecision::Capture);
        assert_eq!(outcome.policy_summary.decision, PolicyDecision::Allow);
        assert!(outcome.captured_as_observation);
    }

    #[test]
    fn passive_observation_suppresses_duplicate_hints_without_policy_denial() {
        let gateway = PolicyModule;
        let outcome = gateway.evaluate_observation_write(ObservationWriteRequest {
            ingest_mode: IngestMode::PassiveObservation,
            namespace_bound: true,
            policy_allowed: true,
            duplicate_hint: true,
        });

        assert_eq!(outcome.decision, PassiveObservationDecision::Suppress);
        assert_eq!(outcome.policy_summary.decision, PolicyDecision::Allow);
        assert!(!outcome.captured_as_observation);
    }

    #[test]
    fn passive_observation_denial_stops_before_capture() {
        let gateway = PolicyModule;
        let outcome = gateway.evaluate_observation_write(ObservationWriteRequest {
            ingest_mode: IngestMode::PassiveObservation,
            namespace_bound: false,
            policy_allowed: false,
            duplicate_hint: false,
        });

        assert_eq!(outcome.decision, PassiveObservationDecision::Deny);
        assert_eq!(outcome.policy_summary.decision, PolicyDecision::Deny);
        assert!(!outcome.captured_as_observation);
    }

    #[test]
    fn active_ingest_denial_stops_before_capture() {
        let gateway = PolicyModule;
        let outcome = gateway.evaluate_observation_write(ObservationWriteRequest {
            ingest_mode: IngestMode::Active,
            namespace_bound: false,
            policy_allowed: false,
            duplicate_hint: false,
        });

        assert_eq!(outcome.decision, PassiveObservationDecision::Deny);
        assert_eq!(outcome.policy_summary.decision, PolicyDecision::Deny);
        assert!(!outcome.captured_as_observation);
    }

    #[test]
    fn preview_only_returns_preview_outcome() {
        let gateway = PolicyModule;
        let mut request = SafeguardRequest::ready(OperationClass::AuthoritativeRewrite);
        request.preview_only = true;
        request.requires_confirmation = true;

        let outcome = gateway.evaluate_safeguard(request);

        assert_eq!(outcome.outcome_class, OutcomeClass::Preview);
        assert_eq!(outcome.preflight_state, PreflightState::Ready);
        assert!(outcome.blocked_reasons.is_empty());
        assert!(outcome.confirmation.required);
        assert_eq!(
            outcome.reversibility,
            ReversibilityKind::RollbackViaSnapshot
        );
    }

    #[test]
    fn apply_without_required_confirmation_blocks() {
        let gateway = PolicyModule;
        let mut request = SafeguardRequest::ready(OperationClass::AuthoritativeRewrite);
        request.requires_confirmation = true;

        let outcome = gateway.evaluate_safeguard(request);

        assert_eq!(outcome.outcome_class, OutcomeClass::Blocked);
        assert_eq!(outcome.preflight_state, PreflightState::Blocked);
        assert!(outcome
            .blocked_reasons
            .contains(&SafeguardReasonCode::ConfirmationRequired));
    }

    #[test]
    fn force_confirmed_apply_stays_accepted_when_other_checks_pass() {
        let gateway = PolicyModule;
        let mut request = SafeguardRequest::ready(OperationClass::AuthoritativeRewrite);
        request.requires_confirmation = true;
        request.local_confirmation = true;
        request.force_allowed = true;
        request.generation_bound = Some(7);

        let outcome = gateway.evaluate_safeguard(request);

        assert_eq!(outcome.outcome_class, OutcomeClass::Accepted);
        assert_eq!(outcome.preflight_state, PreflightState::Ready);
        assert!(outcome.confirmation.confirmed);
        assert!(outcome.confirmation.force_allowed);
        assert_eq!(outcome.confirmation.generation_bound, Some(7));
    }

    #[test]
    fn degraded_outcome_preserves_degraded_check_status() {
        let gateway = PolicyModule;
        let mut request = SafeguardRequest::ready(OperationClass::DerivedSurfaceMutation);
        request.confidence_ready = false;
        request.can_degrade = true;

        let outcome = gateway.evaluate_safeguard(request);

        assert_eq!(outcome.outcome_class, OutcomeClass::Degraded);
        assert!(outcome.blocked_reasons.is_empty());
        assert_eq!(outcome.affected_scope, "effective_namespace");
        assert_eq!(
            outcome.impact_summary,
            "derived_surface_mutation_with_degraded_fallback"
        );
        assert_eq!(outcome.warnings, vec!["low_confidence"]);
        assert_eq!(outcome.preflight_checks, outcome.check_results);
        assert_eq!(outcome.audit.related_run, Some("derived-surface-run"));
        assert_eq!(
            outcome.confidence_constraints,
            Some(super::ConfidenceConstraint {
                minimum_level: "high",
                change_my_mind_conditions: vec![
                    "fresh_authoritative_inputs",
                    "resolved_policy_scope",
                    "stable_generation_anchor",
                ],
            })
        );
        assert!(outcome.preflight_checks.iter().any(|check| {
            check.check_name == "confidence" && check.status == PreflightCheckStatus::Degraded
        }));
    }

    #[test]
    fn policy_denial_is_rejected_even_with_local_confirmation() {
        let gateway = PolicyModule;
        let mut request = SafeguardRequest::ready(OperationClass::IrreversibleMutation);
        request.requires_confirmation = true;
        request.local_confirmation = true;
        request.policy_allowed = false;

        let outcome = gateway.evaluate_safeguard(request);

        assert_eq!(outcome.outcome_class, OutcomeClass::Rejected);
        assert_eq!(outcome.policy_summary.decision, PolicyDecision::Deny);
        assert!(outcome
            .blocked_reasons
            .contains(&SafeguardReasonCode::PolicyDenied));
    }

    #[test]
    fn stale_preflight_blocks_when_generation_anchor_is_missing() {
        let gateway = PolicyModule;
        let mut request = SafeguardRequest::ready(OperationClass::AuthoritativeRewrite);
        request.generation_matches = false;

        let outcome = gateway.evaluate_safeguard(request);

        assert_eq!(outcome.outcome_class, OutcomeClass::Blocked);
        assert_eq!(outcome.preflight_state, PreflightState::StaleKnowledge);
        assert!(outcome
            .blocked_reasons
            .contains(&SafeguardReasonCode::StalePreflight));
    }

    #[test]
    fn generation_mismatch_blocks_when_bound_generation_moves() {
        let gateway = PolicyModule;
        let mut request = SafeguardRequest::ready(OperationClass::AuthoritativeRewrite);
        request.generation_bound = Some(11);
        request.generation_matches = false;

        let outcome = gateway.evaluate_safeguard(request);

        assert_eq!(outcome.outcome_class, OutcomeClass::Blocked);
        assert!(outcome
            .blocked_reasons
            .contains(&SafeguardReasonCode::GenerationMismatch));
    }

    #[test]
    fn snapshot_requirement_surfaces_missing_data() {
        let gateway = PolicyModule;
        let mut request = SafeguardRequest::ready(OperationClass::AuthoritativeRewrite);
        request.snapshot_required = true;
        request.snapshot_available = false;

        let outcome = gateway.evaluate_safeguard(request);

        assert_eq!(outcome.outcome_class, OutcomeClass::Blocked);
        assert_eq!(outcome.preflight_state, PreflightState::MissingData);
        assert!(outcome
            .blocked_reasons
            .contains(&SafeguardReasonCode::SnapshotRequired));
    }

    #[test]
    fn legal_hold_is_rejected_like_terminal_governance_denial() {
        let gateway = PolicyModule;
        let mut request = SafeguardRequest::ready(OperationClass::IrreversibleMutation);
        request.local_confirmation = true;
        request.legal_hold = true;

        let outcome = gateway.evaluate_safeguard(request);

        assert_eq!(outcome.outcome_class, OutcomeClass::Rejected);
        assert!(outcome
            .blocked_reasons
            .contains(&SafeguardReasonCode::LegalHold));
    }

    #[test]
    fn contradiction_archive_requires_snapshot_for_authoritative_archive() {
        let gateway = PolicyModule;
        let mut request = SafeguardRequest::ready(OperationClass::ContradictionArchive);
        request.snapshot_available = false;

        let outcome = gateway.evaluate_safeguard(request);

        assert_eq!(outcome.outcome_class, OutcomeClass::Blocked);
        assert!(outcome
            .blocked_reasons
            .contains(&SafeguardReasonCode::SnapshotRequired));
        assert!(outcome.preflight_checks.iter().any(|check| {
            check.check_name == "contradiction_archive"
                && check
                    .reason_codes
                    .contains(&SafeguardReasonCode::SnapshotRequired)
        }));
    }

    #[test]
    fn contradiction_archive_rejects_legal_hold() {
        let gateway = PolicyModule;
        let mut request = SafeguardRequest::ready(OperationClass::ContradictionArchive);
        request.legal_hold = true;

        let outcome = gateway.evaluate_safeguard(request);

        assert_eq!(outcome.outcome_class, OutcomeClass::Rejected);
        assert!(outcome
            .blocked_reasons
            .contains(&SafeguardReasonCode::LegalHold));
    }
}
