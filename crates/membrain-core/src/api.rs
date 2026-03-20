use crate::observability::OutcomeClass;
use crate::engine::encode::PassiveObservationInspect;
use crate::policy::{
    PolicyGateway, PolicySummary, SafeguardOutcome as PolicySafeguardOutcome, SharingAccessOutcome,
    SharingAccessRequest, SharingVisibility,
};
use crate::types::{MemoryId, SessionId};

/// Stable namespace identifier carried by every core request and response envelope.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct NamespaceId(String);

impl NamespaceId {
    /// Builds a validated namespace identifier.
    pub fn new(raw: impl Into<String>) -> Result<Self, ContextValidationError> {
        let raw = raw.into();
        if raw.is_empty() {
            return Err(ContextValidationError::MissingNamespace);
        }
        if raw
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_' | b'.' | b'/'))
        {
            Ok(Self(raw))
        } else {
            Err(ContextValidationError::MalformedNamespace)
        }
    }

    /// Returns the machine-readable namespace string.
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

/// Stable idempotency and trace identifier carried by all interface wrappers.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct RequestId(String);

impl RequestId {
    /// Builds a validated request identifier.
    pub fn new(raw: impl Into<String>) -> Result<Self, ContextValidationError> {
        let raw = raw.into();
        if raw.trim().is_empty() {
            Err(ContextValidationError::MissingRequestId)
        } else {
            Ok(Self(raw))
        }
    }

    /// Returns the machine-readable request identifier.
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

/// Stable workspace identifier preserved in shared request envelopes.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct WorkspaceId(String);

impl WorkspaceId {
    /// Builds a new workspace identifier.
    pub fn new(raw: impl Into<String>) -> Self {
        Self(raw.into())
    }
}

/// Stable agent identifier preserved in shared request envelopes.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct AgentId(String);

impl AgentId {
    /// Builds a new agent identifier.
    pub fn new(raw: impl Into<String>) -> Self {
        Self(raw.into())
    }
}

/// Stable task or governing work-item identifier preserved in request envelopes.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct TaskId(String);

impl TaskId {
    /// Builds a new task identifier.
    pub fn new(raw: impl Into<String>) -> Self {
        Self(raw.into())
    }
}

/// Shared policy hints carried by wrappers before core policy evaluation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct PolicyContext {
    /// Whether approved public/shareable widening was requested.
    pub include_public: bool,
    /// Visibility attached to the targeted memory or result set.
    pub sharing_visibility: SharingVisibility,
    /// Whether the transport already bound the caller identity deterministically.
    pub caller_identity_bound: bool,
    /// Whether workspace ACL permits the requested read or write.
    pub workspace_acl_allowed: bool,
    /// Whether agent ACL permits the requested read or write.
    pub agent_acl_allowed: bool,
    /// Whether session visibility permits the requested read or write.
    pub session_visibility_allowed: bool,
    /// Whether legal hold forbids widened sharing for this request.
    pub legal_hold: bool,
}

/// Shared request envelope reused across CLI, daemon, and MCP surfaces.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RequestContext {
    /// Requested namespace scope supplied by the caller when known.
    pub namespace: Option<NamespaceId>,
    /// Optional workspace identifier preserved for later routing or policy checks.
    pub workspace_id: Option<WorkspaceId>,
    /// Optional calling agent identity preserved for later routing or policy checks.
    pub agent_id: Option<AgentId>,
    /// Optional active session binding preserved for hot-path lookups.
    pub session_id: Option<SessionId>,
    /// Optional active task or governing work-item handle.
    pub task_id: Option<TaskId>,
    /// Stable idempotency and tracing key.
    pub request_id: RequestId,
    /// Shared policy hints passed into the core policy gateway.
    pub policy_context: PolicyContext,
    /// Optional request-path time budget in milliseconds.
    pub time_budget_ms: Option<u32>,
}

impl RequestContext {
    /// Binds one effective namespace using either the explicit request value or one deterministic default.
    pub fn bind_namespace(
        &self,
        deterministic_default: Option<NamespaceId>,
    ) -> Result<BoundRequestContext, ContextValidationError> {
        let namespace = match (&self.namespace, deterministic_default) {
            (Some(namespace), _) => namespace.clone(),
            (None, Some(namespace)) => namespace,
            (None, None) => return Err(ContextValidationError::MissingNamespace),
        };

        Ok(BoundRequestContext {
            request: self.clone(),
            namespace,
        })
    }
}

/// Request envelope after deterministic effective-namespace binding.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BoundRequestContext {
    request: RequestContext,
    namespace: NamespaceId,
}

impl BoundRequestContext {
    /// Returns the effective namespace for this request.
    pub fn namespace(&self) -> &NamespaceId {
        &self.namespace
    }

    /// Returns the original request envelope.
    pub fn request(&self) -> &RequestContext {
        &self.request
    }

    /// Evaluates the shared namespace gate before expensive work begins.
    pub fn evaluate_policy(&self, gateway: &impl PolicyGateway) -> PolicySummary {
        if self.request.policy_context.caller_identity_bound {
            gateway.evaluate_namespace(true)
        } else {
            PolicySummary::deny(true)
        }
    }

    /// Evaluates namespace-aware sharing access before candidate generation or packaging.
    pub fn evaluate_sharing_access(&self, gateway: &impl PolicyGateway) -> SharingAccessOutcome {
        gateway.evaluate_sharing_access(SharingAccessRequest {
            same_namespace: true,
            include_public: self.request.policy_context.include_public,
            visibility: self.request.policy_context.sharing_visibility,
            workspace_acl_allowed: self.request.policy_context.workspace_acl_allowed,
            agent_acl_allowed: self.request.policy_context.agent_acl_allowed,
            session_visibility_allowed: self.request.policy_context.session_visibility_allowed,
            legal_hold: self.request.policy_context.legal_hold,
        })
    }

    /// Evaluates widened cross-namespace sharing access for an explicitly targeted memory.
    pub fn evaluate_cross_namespace_sharing_access(
        &self,
        gateway: &impl PolicyGateway,
        memory_namespace: &NamespaceId,
    ) -> SharingAccessOutcome {
        gateway.evaluate_sharing_access(SharingAccessRequest {
            same_namespace: self.namespace == *memory_namespace,
            include_public: self.request.policy_context.include_public,
            visibility: self.request.policy_context.sharing_visibility,
            workspace_acl_allowed: self.request.policy_context.workspace_acl_allowed,
            agent_acl_allowed: self.request.policy_context.agent_acl_allowed,
            session_visibility_allowed: self.request.policy_context.session_visibility_allowed,
            legal_hold: self.request.policy_context.legal_hold,
        })
    }
}

/// Machine-readable failure family for shared core response envelopes.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ErrorKind {
    ValidationFailure,
    PolicyDenied,
    UnsupportedFeature,
    TransientFailure,
    TimeoutFailure,
    CorruptionFailure,
    InternalFailure,
}

impl ErrorKind {
    /// Returns whether the caller may safely retry this failure family.
    pub const fn retryable(self) -> bool {
        matches!(self, Self::TransientFailure | Self::TimeoutFailure)
    }

    /// Returns the stable machine-readable name for this failure family.
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::ValidationFailure => "validation_failure",
            Self::PolicyDenied => "policy_denied",
            Self::UnsupportedFeature => "unsupported_feature",
            Self::TransientFailure => "transient_failure",
            Self::TimeoutFailure => "timeout_failure",
            Self::CorruptionFailure => "corruption_failure",
            Self::InternalFailure => "internal_failure",
        }
    }

    /// Returns the canonical first recovery step for this failure family.
    pub const fn primary_remediation(self) -> RemediationStep {
        match self {
            Self::ValidationFailure => RemediationStep::FixRequest,
            Self::PolicyDenied => RemediationStep::ChangeScope,
            Self::UnsupportedFeature => RemediationStep::CheckHealth,
            Self::TransientFailure => RemediationStep::RetryWithBackoff,
            Self::TimeoutFailure => RemediationStep::RetryWithHigherBudget,
            Self::CorruptionFailure => RemediationStep::RunDoctor,
            Self::InternalFailure => RemediationStep::InspectState,
        }
    }
}

/// Stable machine-readable next-step hint shared across interfaces.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RemediationStep {
    FixRequest,
    ChangeScope,
    CheckHealth,
    RetryWithBackoff,
    RetryWithHigherBudget,
    RunDoctor,
    RunRepair,
    InspectState,
}

impl RemediationStep {
    /// Returns the stable machine-readable name for this next-step hint.
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::FixRequest => "fix_request",
            Self::ChangeScope => "change_scope",
            Self::CheckHealth => "check_health",
            Self::RetryWithBackoff => "retry_with_backoff",
            Self::RetryWithHigherBudget => "retry_with_higher_budget",
            Self::RunDoctor => "run_doctor",
            Self::RunRepair => "run_repair",
            Self::InspectState => "inspect_state",
        }
    }
}

/// Shared machine-readable remediation payload for failed or degraded responses.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RemediationHint {
    pub summary: String,
    pub next_steps: Vec<RemediationStep>,
}

impl RemediationHint {
    /// Builds a remediation payload from one summary and ordered next steps.
    pub fn new(summary: impl Into<String>, next_steps: Vec<RemediationStep>) -> Self {
        Self {
            summary: summary.into(),
            next_steps,
        }
    }

    /// Builds the canonical minimal remediation payload for one failure family.
    pub fn for_error(error_kind: ErrorKind, summary: impl Into<String>) -> Self {
        let mut next_steps = vec![error_kind.primary_remediation()];
        if matches!(error_kind, ErrorKind::CorruptionFailure) {
            next_steps.push(RemediationStep::RunRepair);
        }
        Self::new(summary, next_steps)
    }

    /// Returns the ordered stable machine-readable step names.
    pub fn step_names(&self) -> Vec<&'static str> {
        self.next_steps.iter().map(|step| step.as_str()).collect()
    }
}

/// Stable availability posture shared across CLI, daemon, and MCP responses.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AvailabilityPosture {
    Full,
    Degraded,
    ReadOnly,
    Offline,
}

impl AvailabilityPosture {
    /// Returns the stable machine-readable posture string.
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Full => "full",
            Self::Degraded => "degraded",
            Self::ReadOnly => "read_only",
            Self::Offline => "offline",
        }
    }
}

/// Machine-readable degraded-mode reason preserved across interfaces.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AvailabilityReason {
    GraphUnavailable,
    IndexBypassed,
    CacheInvalidated,
    RepairInFlight,
    AuthoritativeInputUnreadable,
}

impl AvailabilityReason {
    /// Returns the stable machine-readable degraded-reason string.
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::GraphUnavailable => "graph_unavailable",
            Self::IndexBypassed => "index_bypassed",
            Self::CacheInvalidated => "cache_invalidated",
            Self::RepairInFlight => "repair_in_flight",
            Self::AuthoritativeInputUnreadable => "authoritative_input_unreadable",
        }
    }
}

/// Shared availability summary for degraded or read-only service posture.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AvailabilitySummary {
    pub posture: AvailabilityPosture,
    pub query_capabilities: Vec<&'static str>,
    pub mutation_capabilities: Vec<&'static str>,
    pub degraded_reasons: Vec<AvailabilityReason>,
    pub recovery_conditions: Vec<RemediationStep>,
}

/// Shared absent-vs-redacted field marker for cross-interface response parity.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FieldPresence<T> {
    Present(T),
    Absent,
    Redacted,
}

impl<T> FieldPresence<T> {
    /// Returns true when the field value is present and visible.
    pub const fn is_present(&self) -> bool {
        matches!(self, Self::Present(_))
    }

    /// Returns the stable machine-readable state for this field.
    pub const fn state_name(&self) -> &'static str {
        match self {
            Self::Present(_) => "present",
            Self::Absent => "absent",
            Self::Redacted => "redacted",
        }
    }
}

/// Shared machine-readable summary of policy shaping applied to a visible outcome.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PolicyFilterSummary {
    pub effective_namespace: String,
    pub policy_family: &'static str,
    pub outcome_class: OutcomeClass,
    pub blocked_stage: &'static str,
    pub sharing_scope: FieldPresence<&'static str>,
    pub retention_marker: FieldPresence<&'static str>,
    pub redaction_fields: Vec<&'static str>,
}

impl PolicyFilterSummary {
    /// Builds a new machine-readable policy filter summary.
    pub fn new(
        effective_namespace: impl Into<String>,
        policy_family: &'static str,
        outcome_class: OutcomeClass,
        blocked_stage: &'static str,
        sharing_scope: FieldPresence<&'static str>,
        retention_marker: FieldPresence<&'static str>,
        redaction_fields: Vec<&'static str>,
    ) -> Self {
        Self {
            effective_namespace: effective_namespace.into(),
            policy_family,
            outcome_class,
            blocked_stage,
            sharing_scope,
            retention_marker,
            redaction_fields,
        }
    }
}

impl AvailabilitySummary {
    /// Builds a new machine-readable availability summary.
    pub fn new(
        posture: AvailabilityPosture,
        query_capabilities: Vec<&'static str>,
        mutation_capabilities: Vec<&'static str>,
        degraded_reasons: Vec<AvailabilityReason>,
        recovery_conditions: Vec<RemediationStep>,
    ) -> Self {
        Self {
            posture,
            query_capabilities,
            mutation_capabilities,
            degraded_reasons,
            recovery_conditions,
        }
    }

    /// Builds the canonical degraded availability summary.
    pub fn degraded(
        query_capabilities: Vec<&'static str>,
        mutation_capabilities: Vec<&'static str>,
        degraded_reasons: Vec<AvailabilityReason>,
        recovery_conditions: Vec<RemediationStep>,
    ) -> Self {
        Self::new(
            AvailabilityPosture::Degraded,
            query_capabilities,
            mutation_capabilities,
            degraded_reasons,
            recovery_conditions,
        )
    }

    /// Returns the stable machine-readable degraded-reason names.
    pub fn reason_names(&self) -> Vec<&'static str> {
        self.degraded_reasons
            .iter()
            .map(|reason| reason.as_str())
            .collect()
    }

    /// Returns the stable machine-readable recovery-condition names.
    pub fn recovery_condition_names(&self) -> Vec<&'static str> {
        self.recovery_conditions
            .iter()
            .map(|step| step.as_str())
            .collect()
    }
}

/// Shared validation failures for request-envelope and namespace binding rules.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ContextValidationError {
    MissingNamespace,
    MalformedNamespace,
    MissingRequestId,
}

impl ContextValidationError {
    /// Maps context-validation failures into the shared error taxonomy.
    pub const fn error_kind(self) -> ErrorKind {
        ErrorKind::ValidationFailure
    }
}

impl std::fmt::Display for ContextValidationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::MissingNamespace => write!(f, "missing namespace"),
            Self::MalformedNamespace => write!(f, "malformed namespace"),
            Self::MissingRequestId => write!(f, "missing request id"),
        }
    }
}

impl std::error::Error for ContextValidationError {}

/// Shared warning payload for non-fatal response annotations.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResponseWarning {
    pub code: &'static str,
    pub detail: String,
}

impl ResponseWarning {
    /// Builds a new machine-readable warning.
    pub fn new(code: &'static str, detail: impl Into<String>) -> Self {
        Self {
            code,
            detail: detail.into(),
        }
    }
}

/// Machine-readable top-level route summary preserved across interfaces.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RouteSummary {
    pub route_family: &'static str,
    pub route_reason: &'static str,
    pub tier1_consulted_first: bool,
    pub routes_to_deeper_tiers: bool,
}

/// Stable trace-stage vocabulary for cross-surface explain payloads.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TraceStage {
    Tier1ExactHandle,
    Tier1RecentWindow,
    Tier2Exact,
    Tier3Fallback,
    PolicyGate,
    Packaging,
}

impl TraceStage {
    /// Returns the stable machine-readable stage name.
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Tier1ExactHandle => "tier1_exact_handle",
            Self::Tier1RecentWindow => "tier1_recent_window",
            Self::Tier2Exact => "tier2_exact",
            Self::Tier3Fallback => "tier3_fallback",
            Self::PolicyGate => "policy_gate",
            Self::Packaging => "packaging",
        }
    }
    /// Maps a retrieval trace stage into the shared cross-surface stage vocabulary.
    pub const fn from_recall(stage: crate::engine::recall::RecallTraceStage) -> Self {
        match stage {
            crate::engine::recall::RecallTraceStage::Tier1ExactHandle => Self::Tier1ExactHandle,
            crate::engine::recall::RecallTraceStage::Tier1RecentWindow => Self::Tier1RecentWindow,
            crate::engine::recall::RecallTraceStage::Tier2Exact => Self::Tier2Exact,
            crate::engine::recall::RecallTraceStage::Tier3Fallback => Self::Tier3Fallback,
        }
    }
}

/// Machine-readable reason describing why an item appeared or was omitted.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResultReason {
    pub memory_id: Option<MemoryId>,
    pub reason_code: &'static str,
    pub detail: &'static str,
}

/// Shared policy summary for explain surfaces.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TracePolicySummary {
    pub effective_namespace: String,
    pub policy_family: &'static str,
    pub outcome_class: OutcomeClass,
    pub blocked_stage: &'static str,
    pub redaction_fields: Vec<&'static str>,
    pub retention_state: FieldPresence<&'static str>,
    pub sharing_scope: FieldPresence<&'static str>,
}

/// Shared provenance summary for explain surfaces.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TraceProvenanceSummary {
    pub source_kind: &'static str,
    pub source_reference: &'static str,
    pub lineage_ancestors: Vec<MemoryId>,
}

/// Shared inspect summary for passive-observation provenance and retention semantics.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PassiveObservationInspectSummary {
    pub source_kind: &'static str,
    pub write_decision: &'static str,
    pub captured_as_observation: bool,
    pub observation_source: FieldPresence<String>,
    pub observation_chunk_id: FieldPresence<String>,
    pub retention_marker: FieldPresence<&'static str>,
}

impl PassiveObservationInspectSummary {
    /// Builds an inspect summary from encode-side passive-observation facts.
    pub fn from_encode(inspect: &PassiveObservationInspect) -> Self {
        Self {
            source_kind: inspect.source_kind,
            write_decision: inspect.write_decision,
            captured_as_observation: inspect.captured_as_observation,
            observation_source: inspect
                .observation_source
                .clone()
                .map(FieldPresence::Present)
                .unwrap_or(FieldPresence::Absent),
            observation_chunk_id: inspect
                .observation_chunk_id
                .clone()
                .map(FieldPresence::Present)
                .unwrap_or(FieldPresence::Absent),
            retention_marker: if inspect.retention_marker == "absent" {
                FieldPresence::Absent
            } else {
                FieldPresence::Present(inspect.retention_marker)
            },
        }
    }
}

/// Shared freshness marker for explain surfaces.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FreshnessMarker {
    pub code: &'static str,
    pub detail: &'static str,
}

/// Shared conflict marker for explain surfaces.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ConflictMarker {
    pub code: &'static str,
    pub detail: &'static str,
}

/// Shared uncertainty marker for explain surfaces.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UncertaintyMarker {
    pub code: &'static str,
    pub detail: &'static str,
}

/// Shared response envelope reused across CLI, daemon, and MCP wrappers.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResponseContext<T> {
    pub ok: bool,
    pub request_id: RequestId,
    pub namespace: NamespaceId,
    pub result: Option<T>,
    pub route_summary: Option<RouteSummary>,
    pub trace_stages: Vec<TraceStage>,
    pub result_reasons: Vec<ResultReason>,
    pub outcome_class: OutcomeClass,
    pub error_kind: Option<ErrorKind>,
    pub retryable: bool,
    pub partial_success: bool,
    pub remediation: Option<RemediationHint>,
    pub availability: Option<AvailabilitySummary>,
    pub policy_filters_applied: Vec<PolicyFilterSummary>,
    pub policy_summary: Option<TracePolicySummary>,
    pub provenance_summary: Option<TraceProvenanceSummary>,
    pub passive_observation: Option<PassiveObservationInspectSummary>,
    pub freshness_markers: Vec<FreshnessMarker>,
    pub conflict_markers: Vec<ConflictMarker>,
    pub uncertainty_markers: Vec<UncertaintyMarker>,
    pub safeguard: Option<PolicySafeguardOutcome>,
    pub warnings: Vec<ResponseWarning>,
}

impl<T> ResponseContext<T> {
    /// Builds a successful shared response envelope.
    pub fn success(namespace: NamespaceId, request_id: RequestId, result: T) -> Self {
        Self {
            ok: true,
            request_id,
            namespace,
            result: Some(result),
            route_summary: None,
            trace_stages: Vec::new(),
            result_reasons: Vec::new(),
            outcome_class: OutcomeClass::Accepted,
            error_kind: None,
            retryable: false,
            partial_success: false,
            remediation: None,
            availability: None,
            policy_filters_applied: Vec::new(),
            policy_summary: None,
            provenance_summary: None,
            passive_observation: None,
            freshness_markers: Vec::new(),
            conflict_markers: Vec::new(),
            uncertainty_markers: Vec::new(),
            safeguard: None,
            warnings: Vec::new(),
        }
    }

    /// Attaches shared explain-trace schema fields to this response.
    pub fn with_trace_schema(
        mut self,
        route_summary: RouteSummary,
        trace_stages: Vec<TraceStage>,
        result_reasons: Vec<ResultReason>,
        policy_summary: TracePolicySummary,
        provenance_summary: TraceProvenanceSummary,
        freshness_markers: Vec<FreshnessMarker>,
        conflict_markers: Vec<ConflictMarker>,
        uncertainty_markers: Vec<UncertaintyMarker>,
    ) -> Self {
        self.route_summary = Some(route_summary);
        self.trace_stages = trace_stages;
        self.result_reasons = result_reasons;
        self.policy_summary = Some(policy_summary);
        self.provenance_summary = Some(provenance_summary);
        self.freshness_markers = freshness_markers;
        self.conflict_markers = conflict_markers;
        self.uncertainty_markers = uncertainty_markers;
        self
    }

    /// Attaches passive-observation inspect facts to this response.
    pub fn with_passive_observation(
        mut self,
        passive_observation: PassiveObservationInspectSummary,
    ) -> Self {
        self.passive_observation = Some(passive_observation);
        self
    }

    /// Builds a failed shared response envelope.
    pub fn failure(
        namespace: NamespaceId,
        request_id: RequestId,
        error_kind: ErrorKind,
        warnings: Vec<ResponseWarning>,
    ) -> Self {
        Self {
            ok: false,
            request_id,
            namespace,
            result: None,
            route_summary: None,
            trace_stages: Vec::new(),
            result_reasons: Vec::new(),
            outcome_class: OutcomeClass::Rejected,
            retryable: error_kind.retryable(),
            error_kind: Some(error_kind),
            partial_success: false,
            remediation: Some(RemediationHint::for_error(
                error_kind,
                format!("{}", error_kind.as_str()),
            )),
            availability: None,
            policy_filters_applied: Vec::new(),
            policy_summary: None,
            provenance_summary: None,
            passive_observation: None,
            freshness_markers: Vec::new(),
            conflict_markers: Vec::new(),
            uncertainty_markers: Vec::new(),
            safeguard: None,
            warnings,
        }
    }

    /// Marks a successful response as partial without changing the result payload.
    pub fn with_partial_success(mut self) -> Self {
        self.partial_success = true;
        self.outcome_class = OutcomeClass::Partial;
        self
    }

    /// Attaches machine-readable remediation to this response.
    pub fn with_remediation(mut self, remediation: RemediationHint) -> Self {
        self.remediation = Some(remediation);
        self
    }

    /// Attaches machine-readable availability posture to this response.
    pub fn with_availability(mut self, availability: AvailabilitySummary) -> Self {
        self.outcome_class = match availability.posture {
            AvailabilityPosture::Full => self.outcome_class,
            AvailabilityPosture::Degraded
            | AvailabilityPosture::ReadOnly
            | AvailabilityPosture::Offline => OutcomeClass::Degraded,
        };
        self.availability = Some(availability);
        self
    }

    /// Attaches machine-readable policy shaping summaries to this response.
    pub fn with_policy_filters(mut self, policy_filters_applied: Vec<PolicyFilterSummary>) -> Self {
        self.policy_filters_applied = policy_filters_applied;
        self
    }

    /// Attaches a machine-readable safeguard summary to this response.
    pub fn with_safeguard(mut self, safeguard: PolicySafeguardOutcome) -> Self {
        self.outcome_class = safeguard.outcome_class;
        self.safeguard = Some(safeguard);
        self
    }
}

/// Stable shared API boundary exposed by the core crate.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct ApiModule;

impl ApiModule {
    /// Returns the stable component identifier for this shared API surface.
    pub const fn component_name(&self) -> &'static str {
        "api"
    }
}

#[cfg(test)]
mod tests {
    use super::{
        ApiModule, AvailabilityPosture, AvailabilityReason, AvailabilitySummary, ConflictMarker,
        ContextValidationError, ErrorKind, FieldPresence, FreshnessMarker, NamespaceId,
        PolicyContext, PolicyFilterSummary, RemediationHint, RemediationStep, RequestContext,
        RequestId, ResponseContext, ResponseWarning, ResultReason, RouteSummary,
        PassiveObservationInspectSummary, TracePolicySummary, TraceProvenanceSummary,
        TraceStage, UncertaintyMarker,
    };
    use crate::observability::OutcomeClass;
    use crate::policy::{
        ConfidenceConstraint, ConfirmationState, OperationClass, PolicyDecision, PolicyGateway,
        PolicyModule, PreflightState, ReversibilityKind, SafeguardAudit,
        SafeguardOutcome as PolicySafeguardOutcome, SafeguardReasonCode, SafeguardRequest,
        SharingAccessDecision, SharingVisibility,
    };
    use crate::types::SessionId;

    #[test]
    fn namespace_binding_accepts_explicit_namespace() {
        let request = RequestContext {
            namespace: Some(NamespaceId::new("team.alpha").unwrap()),
            workspace_id: None,
            agent_id: None,
            session_id: Some(SessionId(7)),
            task_id: None,
            request_id: RequestId::new("req-1").unwrap(),
            policy_context: PolicyContext {
                include_public: false,
                sharing_visibility: SharingVisibility::Private,
                caller_identity_bound: true,
                workspace_acl_allowed: true,
                agent_acl_allowed: true,
                session_visibility_allowed: true,
                legal_hold: false,
            },
            time_budget_ms: Some(50),
        };

        let bound = request.bind_namespace(None).unwrap();
        let policy = bound.evaluate_policy(&PolicyModule);

        assert_eq!(bound.namespace().as_str(), "team.alpha");
        assert_eq!(bound.request().session_id, Some(SessionId(7)));
        assert_eq!(policy.decision, PolicyDecision::Allow);
    }

    #[test]
    fn namespace_policy_denies_when_caller_identity_is_not_bound() {
        let request = RequestContext {
            namespace: Some(NamespaceId::new("team.alpha").unwrap()),
            workspace_id: None,
            agent_id: None,
            session_id: None,
            task_id: None,
            request_id: RequestId::new("req-1b").unwrap(),
            policy_context: PolicyContext {
                include_public: false,
                sharing_visibility: SharingVisibility::Private,
                caller_identity_bound: false,
                workspace_acl_allowed: true,
                agent_acl_allowed: true,
                session_visibility_allowed: true,
                legal_hold: false,
            },
            time_budget_ms: None,
        };

        let bound = request.bind_namespace(None).unwrap();
        let policy = bound.evaluate_policy(&PolicyModule);

        assert_eq!(policy.decision, PolicyDecision::Deny);
        assert!(policy.namespace_bound);
        assert_eq!(policy.outcome_class, OutcomeClass::Rejected);
    }

    #[test]
    fn namespace_binding_uses_deterministic_default_when_omitted() {
        let request = RequestContext {
            namespace: None,
            workspace_id: None,
            agent_id: None,
            session_id: None,
            task_id: None,
            request_id: RequestId::new("req-2").unwrap(),
            policy_context: PolicyContext {
                include_public: true,
                sharing_visibility: SharingVisibility::Public,
                caller_identity_bound: true,
                workspace_acl_allowed: true,
                agent_acl_allowed: true,
                session_visibility_allowed: true,
                legal_hold: false,
            },
            time_budget_ms: None,
        };

        let bound = request
            .bind_namespace(Some(NamespaceId::new("default/ns").unwrap()))
            .unwrap();

        assert_eq!(bound.namespace().as_str(), "default/ns");
        assert!(bound.request().policy_context.include_public);
        assert!(bound.request().policy_context.caller_identity_bound);
    }

    #[test]
    fn sharing_access_allows_same_namespace_private_reads() {
        let request = RequestContext {
            namespace: Some(NamespaceId::new("team.alpha").unwrap()),
            workspace_id: None,
            agent_id: None,
            session_id: None,
            task_id: None,
            request_id: RequestId::new("req-share-1").unwrap(),
            policy_context: PolicyContext {
                include_public: false,
                sharing_visibility: SharingVisibility::Private,
                caller_identity_bound: true,
                workspace_acl_allowed: true,
                agent_acl_allowed: true,
                session_visibility_allowed: true,
                legal_hold: false,
            },
            time_budget_ms: None,
        };

        let bound = request.bind_namespace(None).unwrap();
        let outcome = bound.evaluate_sharing_access(&PolicyModule);

        assert_eq!(outcome.decision, SharingAccessDecision::Allow);
        assert_eq!(outcome.sharing_scope.unwrap().as_str(), "namespace_only");
        assert!(outcome.redaction_fields.is_empty());
        assert!(outcome.denial_reasons.is_empty());
    }

    #[test]
    fn sharing_access_denies_cross_namespace_private_reads_without_leaking_scope() {
        let request = RequestContext {
            namespace: Some(NamespaceId::new("team.alpha").unwrap()),
            workspace_id: None,
            agent_id: None,
            session_id: None,
            task_id: None,
            request_id: RequestId::new("req-share-2").unwrap(),
            policy_context: PolicyContext {
                include_public: false,
                sharing_visibility: SharingVisibility::Private,
                caller_identity_bound: true,
                workspace_acl_allowed: true,
                agent_acl_allowed: true,
                session_visibility_allowed: true,
                legal_hold: false,
            },
            time_budget_ms: None,
        };

        let bound = request.bind_namespace(None).unwrap();
        let outcome = bound.evaluate_cross_namespace_sharing_access(
            &PolicyModule,
            &NamespaceId::new("team.beta").unwrap(),
        );

        assert_eq!(outcome.decision, SharingAccessDecision::Deny);
        assert_eq!(outcome.policy_summary.decision, PolicyDecision::Deny);
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
    fn sharing_access_redacts_cross_namespace_shared_reads() {
        let request = RequestContext {
            namespace: Some(NamespaceId::new("team.alpha").unwrap()),
            workspace_id: None,
            agent_id: None,
            session_id: None,
            task_id: None,
            request_id: RequestId::new("req-share-3").unwrap(),
            policy_context: PolicyContext {
                include_public: false,
                sharing_visibility: SharingVisibility::Shared,
                caller_identity_bound: true,
                workspace_acl_allowed: true,
                agent_acl_allowed: true,
                session_visibility_allowed: true,
                legal_hold: false,
            },
            time_budget_ms: None,
        };

        let bound = request.bind_namespace(None).unwrap();
        let outcome = bound.evaluate_cross_namespace_sharing_access(
            &PolicyModule,
            &NamespaceId::new("team.beta").unwrap(),
        );

        assert_eq!(outcome.decision, SharingAccessDecision::Redact);
        assert_eq!(outcome.sharing_scope.unwrap().as_str(), "approved_shared");
        assert_eq!(outcome.redaction_fields, vec!["workspace_id", "session_id"]);
    }

    #[test]
    fn sharing_access_denies_public_widening_without_include_public() {
        let request = RequestContext {
            namespace: Some(NamespaceId::new("team.alpha").unwrap()),
            workspace_id: None,
            agent_id: None,
            session_id: None,
            task_id: None,
            request_id: RequestId::new("req-share-4").unwrap(),
            policy_context: PolicyContext {
                include_public: false,
                sharing_visibility: SharingVisibility::Public,
                caller_identity_bound: true,
                workspace_acl_allowed: true,
                agent_acl_allowed: true,
                session_visibility_allowed: true,
                legal_hold: false,
            },
            time_budget_ms: None,
        };

        let bound = request.bind_namespace(None).unwrap();
        let outcome = bound.evaluate_cross_namespace_sharing_access(
            &PolicyModule,
            &NamespaceId::new("team.beta").unwrap(),
        );

        assert_eq!(outcome.decision, SharingAccessDecision::Deny);
        assert!(outcome
            .denial_reasons
            .iter()
            .any(|reason| reason.as_str() == "approved_scope_required"));
    }

    #[test]
    fn sharing_access_allows_same_namespace_private_reads_during_legal_hold() {
        let request = RequestContext {
            namespace: Some(NamespaceId::new("team.alpha").unwrap()),
            workspace_id: None,
            agent_id: None,
            session_id: None,
            task_id: None,
            request_id: RequestId::new("req-share-legal-hold").unwrap(),
            policy_context: PolicyContext {
                include_public: false,
                sharing_visibility: SharingVisibility::Private,
                caller_identity_bound: true,
                workspace_acl_allowed: true,
                agent_acl_allowed: true,
                session_visibility_allowed: true,
                legal_hold: true,
            },
            time_budget_ms: None,
        };

        let bound = request.bind_namespace(None).unwrap();
        let outcome = bound.evaluate_sharing_access(&PolicyModule);

        assert_eq!(outcome.decision, SharingAccessDecision::Allow);
        assert_eq!(outcome.sharing_scope.unwrap().as_str(), "namespace_only");
        assert!(outcome.denial_reasons.is_empty());
        assert!(outcome.redaction_fields.is_empty());
    }

    #[test]
    fn namespace_binding_rejects_missing_namespace_without_default() {
        let request = RequestContext {
            namespace: None,
            workspace_id: None,
            agent_id: None,
            session_id: None,
            task_id: None,
            request_id: RequestId::new("req-3").unwrap(),
            policy_context: PolicyContext {
                include_public: false,
                sharing_visibility: SharingVisibility::Private,
                caller_identity_bound: false,
                workspace_acl_allowed: true,
                agent_acl_allowed: true,
                session_visibility_allowed: true,
                legal_hold: false,
            },
            time_budget_ms: None,
        };

        let error = request.bind_namespace(None).unwrap_err();
        assert_eq!(error, ContextValidationError::MissingNamespace);
        assert_eq!(error.error_kind(), ErrorKind::ValidationFailure);
    }

    #[test]
    fn namespace_validation_rejects_malformed_names() {
        let error = NamespaceId::new("bad namespace").unwrap_err();
        assert_eq!(error, ContextValidationError::MalformedNamespace);
    }

    #[test]
    fn response_context_preserves_retryability_and_partial_success() {
        let namespace = NamespaceId::new("team.alpha").unwrap();
        let request_id = RequestId::new("req-4").unwrap();

        let availability = AvailabilitySummary::degraded(
            vec!["recall", "inspect"],
            vec!["preview_only"],
            vec![AvailabilityReason::RepairInFlight],
            vec![RemediationStep::RunDoctor, RemediationStep::RunRepair],
        );
        let safeguard = PolicySafeguardOutcome {
            outcome_class: OutcomeClass::Blocked,
            preflight_state: PreflightState::Blocked,
            operation_class: OperationClass::AuthoritativeRewrite,
            affected_scope: "effective_namespace",
            impact_summary: "authoritative_rewrite_requires_window",
            blocked_reasons: vec![
                SafeguardReasonCode::ConfirmationRequired,
                SafeguardReasonCode::SnapshotRequired,
            ],
            preflight_checks: Vec::new(),
            check_results: Vec::new(),
            warnings: vec!["stale_generation"],
            confidence_constraints: Some(ConfidenceConstraint {
                minimum_level: "high",
                change_my_mind_conditions: vec!["fresh_authoritative_inputs"],
            }),
            reversibility: ReversibilityKind::RollbackViaSnapshot,
            confirmation: ConfirmationState {
                required: true,
                force_allowed: false,
                confirmed: false,
                generation_bound: None,
            },
            audit: SafeguardAudit {
                event_kind: "safeguard_evaluation",
                actor_source: "core_policy",
                request_id: "policy-eval",
                preview_id: None,
                related_run: Some("authoritative-rewrite-run"),
                scope_handle: "effective_namespace",
            },
            policy_summary: PolicyModule.evaluate_namespace(true),
        };
        let success = ResponseContext::success(namespace.clone(), request_id.clone(), 7u8)
            .with_partial_success()
            .with_availability(availability.clone())
            .with_policy_filters(vec![PolicyFilterSummary::new(
                "team.alpha",
                "retention",
                OutcomeClass::Accepted,
                "packaging",
                FieldPresence::Redacted,
                FieldPresence::Absent,
                vec!["raw_text"],
            )])
            .with_safeguard(safeguard.clone());
        assert!(success.ok);
        assert!(success.partial_success);
        assert_eq!(success.result, Some(7));
        assert_eq!(success.outcome_class, OutcomeClass::Blocked);
        assert_eq!(success.error_kind, None);
        assert_eq!(success.availability, Some(availability));
        assert_eq!(success.policy_filters_applied.len(), 1);
        assert_eq!(
            success.policy_filters_applied[0].effective_namespace,
            "team.alpha"
        );
        assert_eq!(success.policy_filters_applied[0].policy_family, "retention");
        assert_eq!(
            success.policy_filters_applied[0].outcome_class,
            OutcomeClass::Accepted
        );
        assert_eq!(success.policy_filters_applied[0].blocked_stage, "packaging");
        assert_eq!(success.safeguard, Some(safeguard));
        assert_eq!(
            success.safeguard.as_ref().unwrap().outcome_class,
            OutcomeClass::Blocked,
        );
        assert_eq!(
            success.safeguard.as_ref().unwrap().affected_scope,
            "effective_namespace",
        );
        assert_eq!(
            success.safeguard.as_ref().unwrap().impact_summary,
            "authoritative_rewrite_requires_window",
        );
        assert_eq!(
            success.safeguard.as_ref().unwrap().warnings,
            vec!["stale_generation"],
        );
        assert_eq!(
            success.safeguard.as_ref().unwrap().audit.scope_handle,
            "effective_namespace",
        );
        assert_eq!(
            success.policy_filters_applied[0].sharing_scope.state_name(),
            "redacted",
        );
        assert_eq!(
            success.policy_filters_applied[0]
                .retention_marker
                .state_name(),
            "absent",
        );

        let failure = ResponseContext::<u8>::failure(
            namespace,
            request_id,
            ErrorKind::TimeoutFailure,
            vec![ResponseWarning::new("budget", "time budget exhausted")],
        );
        assert!(!failure.ok);
        assert_eq!(failure.outcome_class, OutcomeClass::Rejected);
        assert_eq!(failure.error_kind, Some(ErrorKind::TimeoutFailure));
        assert!(failure.retryable);
        assert_eq!(failure.warnings.len(), 1);
        assert_eq!(
            failure.remediation,
            Some(RemediationHint::new(
                "timeout_failure",
                vec![RemediationStep::RetryWithHigherBudget],
            )),
        );
        assert_eq!(failure.safeguard, None);
    }

    #[test]
    fn trace_schema_fields_attach_with_stable_machine_names() {
        let response = ResponseContext::success(
            NamespaceId::new("team.gamma").unwrap(),
            RequestId::new("req-trace").unwrap(),
            11u8,
        )
        .with_trace_schema(
            RouteSummary {
                route_family: "tiered_recall",
                route_reason: "bounded tier1 then tier2",
                tier1_consulted_first: true,
                routes_to_deeper_tiers: true,
            },
            vec![
                TraceStage::PolicyGate,
                TraceStage::Tier1RecentWindow,
                TraceStage::Tier2Exact,
                TraceStage::Packaging,
            ],
            vec![ResultReason {
                memory_id: None,
                reason_code: "tier2_exact_match",
                detail: "candidate survived bounded ranking",
            }],
            TracePolicySummary {
                effective_namespace: "team.gamma".into(),
                policy_family: "namespace",
                outcome_class: OutcomeClass::Accepted,
                blocked_stage: "not_blocked",
                redaction_fields: vec!["raw_text"],
                retention_state: FieldPresence::Absent,
                sharing_scope: FieldPresence::Present("same_namespace"),
            },
            TraceProvenanceSummary {
                source_kind: "memory",
                source_reference: "memory_id",
                lineage_ancestors: Vec::new(),
            },
            vec![FreshnessMarker {
                code: "fresh",
                detail: "item is recent enough for default packaging",
            }],
            vec![ConflictMarker {
                code: "no_open_conflict",
                detail: "no contradiction siblings were surfaced",
            }],
            vec![UncertaintyMarker {
                code: "low_uncertainty",
                detail: "bounded evidence had low uncertainty",
            }],
        )
        .with_passive_observation(PassiveObservationInspectSummary {
            source_kind: "observation",
            write_decision: "capture",
            captured_as_observation: true,
            observation_source: FieldPresence::Present("passive_observation".into()),
            observation_chunk_id: FieldPresence::Present("obs-0000000000000042".into()),
            retention_marker: FieldPresence::Present("volatile_observation"),
        });

        assert_eq!(
            response.route_summary.as_ref().unwrap().route_family,
            "tiered_recall"
        );
        assert_eq!(response.trace_stages.len(), 4);
        assert_eq!(response.trace_stages[0].as_str(), "policy_gate");
        assert_eq!(response.trace_stages[1].as_str(), "tier1_recent_window");
        assert_eq!(response.trace_stages[2].as_str(), "tier2_exact");
        assert_eq!(response.trace_stages[3].as_str(), "packaging");
        assert_eq!(response.result_reasons[0].reason_code, "tier2_exact_match");
        assert_eq!(
            response
                .policy_summary
                .as_ref()
                .unwrap()
                .effective_namespace,
            "team.gamma"
        );
        assert_eq!(
            response
                .policy_summary
                .as_ref()
                .unwrap()
                .sharing_scope
                .state_name(),
            "present"
        );
        assert_eq!(
            response
                .policy_summary
                .as_ref()
                .unwrap()
                .retention_state
                .state_name(),
            "absent"
        );
        assert_eq!(
            response
                .provenance_summary
                .as_ref()
                .unwrap()
                .source_reference,
            "memory_id"
        );
        assert_eq!(response.freshness_markers[0].code, "fresh");
        assert_eq!(response.conflict_markers[0].code, "no_open_conflict");
        assert_eq!(response.uncertainty_markers[0].code, "low_uncertainty");
        assert_eq!(
            response
                .passive_observation
                .as_ref()
                .unwrap()
                .observation_source
                .state_name(),
            "present"
        );
        assert_eq!(
            response
                .passive_observation
                .as_ref()
                .unwrap()
                .observation_chunk_id
                .state_name(),
            "present"
        );
        assert_eq!(
            response
                .passive_observation
                .as_ref()
                .unwrap()
                .retention_marker
                .state_name(),
            "present"
        );
        assert_eq!(
            response
                .passive_observation
                .as_ref()
                .unwrap()
                .write_decision,
            "capture"
        );
    }

    #[test]
    fn remediation_and_availability_use_canonical_machine_names() {
        let remediation = RemediationHint::for_error(ErrorKind::CorruptionFailure, "corruption");
        assert_eq!(remediation.summary, "corruption");
        assert_eq!(
            remediation.next_steps,
            vec![RemediationStep::RunDoctor, RemediationStep::RunRepair],
        );
        assert_eq!(remediation.step_names(), vec!["run_doctor", "run_repair"],);
        assert_eq!(
            ErrorKind::PolicyDenied.primary_remediation(),
            RemediationStep::ChangeScope
        );
        assert_eq!(AvailabilityPosture::ReadOnly.as_str(), "read_only");

        let availability = AvailabilitySummary::degraded(
            vec!["recall"],
            vec!["preview_only"],
            vec![
                AvailabilityReason::AuthoritativeInputUnreadable,
                AvailabilityReason::CacheInvalidated,
            ],
            vec![RemediationStep::RunDoctor, RemediationStep::InspectState],
        );
        assert_eq!(availability.posture, AvailabilityPosture::Degraded);
        assert_eq!(
            availability.reason_names(),
            vec!["authoritative_input_unreadable", "cache_invalidated"],
        );
        assert_eq!(
            availability.recovery_condition_names(),
            vec!["run_doctor", "inspect_state"],
        );

        let degraded = ResponseContext::success(
            NamespaceId::new("team.beta").unwrap(),
            RequestId::new("req-5a").unwrap(),
            9u8,
        )
        .with_availability(availability);
        assert!(degraded.ok);
        assert_eq!(degraded.error_kind, None);
        assert_eq!(degraded.outcome_class, OutcomeClass::Degraded);
    }

    #[test]
    fn safeguard_blocked_readiness_stays_distinct_from_policy_denied_failure() {
        let namespace = NamespaceId::new("team.alpha").unwrap();
        let request_id = RequestId::new("req-5").unwrap();
        let gateway = PolicyModule;
        let mut request = SafeguardRequest::ready(OperationClass::AuthoritativeRewrite);
        request.requires_confirmation = true;

        let blocked = gateway.evaluate_safeguard(request);
        assert_eq!(blocked.outcome_class, OutcomeClass::Blocked);
        assert_eq!(blocked.preflight_state, PreflightState::Blocked);
        assert!(blocked
            .blocked_reasons
            .contains(&SafeguardReasonCode::ConfirmationRequired));

        let blocked_response =
            ResponseContext::success(namespace.clone(), request_id.clone(), "preview")
                .with_safeguard(blocked);
        assert!(blocked_response.ok);
        assert_eq!(blocked_response.error_kind, None);
        assert_eq!(blocked_response.outcome_class, OutcomeClass::Blocked);
        assert!(blocked_response.partial_success == false);
        assert!(blocked_response
            .safeguard
            .as_ref()
            .unwrap()
            .blocked_reasons
            .contains(&SafeguardReasonCode::ConfirmationRequired));

        let denied_response = ResponseContext::<&str>::failure(
            namespace,
            request_id,
            ErrorKind::PolicyDenied,
            vec![ResponseWarning::new("policy", "namespace policy denied")],
        );
        assert!(!denied_response.ok);
        assert_eq!(denied_response.outcome_class, OutcomeClass::Rejected);
        assert_eq!(denied_response.error_kind, Some(ErrorKind::PolicyDenied));
        assert!(denied_response.safeguard.is_none());
        assert_eq!(
            denied_response.remediation,
            Some(RemediationHint::new(
                "policy_denied",
                vec![RemediationStep::ChangeScope],
            )),
        );
    }

    #[test]
    fn api_module_reports_stable_component_name() {
        let api = ApiModule;
        assert_eq!(api.component_name(), "api");
    }
}
