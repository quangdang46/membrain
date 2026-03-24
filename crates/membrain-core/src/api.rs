use crate::engine::encode::PassiveObservationInspect;
use crate::engine::result::{EntryLane, OmissionSummary, RetrievalResultSet};
use crate::graph::RelationKind;
use crate::observability::{CacheEvalTrace, OutcomeClass};
use crate::policy::{
    PolicyGateway, PolicySummary, SafeguardOutcome as PolicySafeguardOutcome, SharingAccessOutcome,
    SharingAccessRequest, SharingVisibility,
};
use crate::types::{MemoryId, SessionId};

/// Stable namespace identifier carried by every core request and response envelope.
#[derive(Debug, Clone, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
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
#[derive(Debug, Clone, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
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
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize)]
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
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize)]
pub enum RemediationStep {
    FixRequest,
    ChangeScope,
    CheckHealth,
    RetryWithBackoff,
    RetryWithHigherBudget,
    RunDoctor,
    RunRepair,
    RollbackRecentChange,
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
            Self::RollbackRecentChange => "rollback_recent_change",
            Self::InspectState => "inspect_state",
        }
    }
}

/// Shared machine-readable remediation payload for failed or degraded responses.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
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
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize)]
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
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize)]
pub enum AvailabilityReason {
    GraphUnavailable,
    IndexBypassed,
    CacheInvalidated,
    RepairInFlight,
    RepairRollbackRequired,
    RepairRollbackInProgress,
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
            Self::RepairRollbackRequired => "repair_rollback_required",
            Self::RepairRollbackInProgress => "repair_rollback_in_progress",
            Self::AuthoritativeInputUnreadable => "authoritative_input_unreadable",
        }
    }
}

/// Shared availability summary for degraded or read-only service posture.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
pub struct AvailabilitySummary {
    pub posture: AvailabilityPosture,
    pub query_capabilities: Vec<&'static str>,
    pub mutation_capabilities: Vec<&'static str>,
    pub degraded_reasons: Vec<AvailabilityReason>,
    pub recovery_conditions: Vec<RemediationStep>,
}

/// Shared absent-vs-redacted field marker for cross-interface response parity.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
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
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct PolicyFilterSummary {
    pub effective_namespace: String,
    pub policy_family: String,
    pub outcome_class: OutcomeClass,
    pub blocked_stage: String,
    pub sharing_scope: FieldPresence<String>,
    pub retention_marker: FieldPresence<String>,
    pub redaction_fields: Vec<String>,
}

impl PolicyFilterSummary {
    /// Builds a new machine-readable policy filter summary.
    pub fn new(
        effective_namespace: impl Into<String>,
        policy_family: impl Into<String>,
        outcome_class: OutcomeClass,
        blocked_stage: impl Into<String>,
        sharing_scope: FieldPresence<String>,
        retention_marker: FieldPresence<String>,
        redaction_fields: Vec<String>,
    ) -> Self {
        Self {
            effective_namespace: effective_namespace.into(),
            policy_family: policy_family.into(),
            outcome_class,
            blocked_stage: blocked_stage.into(),
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
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
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
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
pub struct RouteSummary {
    pub route_family: &'static str,
    pub route_reason: &'static str,
    pub tier1_consulted_first: bool,
    pub tier1_answered_directly: bool,
    pub routes_to_deeper_tiers: bool,
    pub candidate_budget: Option<usize>,
    pub pre_route_candidate_count: Option<usize>,
    pub post_route_candidate_count: Option<usize>,
    pub fallback_reason: Option<&'static str>,
}

/// Bounded score component preserved for explain and inspect surfaces.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
pub struct TraceScoreComponent {
    pub signal_family: &'static str,
    pub raw_value: u16,
    pub weight: u8,
}

/// Stable trace-stage vocabulary for cross-surface explain payloads.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum TraceStage {
    Tier1ExactHandle,
    Tier1RecentWindow,
    Tier2Exact,
    GraphExpansion,
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
            Self::GraphExpansion => "graph_expansion",
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
            crate::engine::recall::RecallTraceStage::GraphExpansion => Self::GraphExpansion,
            crate::engine::recall::RecallTraceStage::Tier3Fallback => Self::Tier3Fallback,
        }
    }
}

/// Machine-readable reason describing why an item appeared or was omitted.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
pub struct ResultReason {
    pub memory_id: Option<MemoryId>,
    pub reason_code: String,
    pub reason_family: String,
    pub route_stage: TraceStage,
    pub policy_filter_applied: bool,
    pub detail: String,
}

/// Shared policy summary for explain surfaces.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
pub struct TracePolicySummary {
    pub effective_namespace: String,
    pub policy_family: &'static str,
    pub outcome_class: OutcomeClass,
    pub blocked_stage: &'static str,
    pub redaction_fields: Vec<&'static str>,
    pub retention_state: FieldPresence<&'static str>,
    pub sharing_scope: FieldPresence<&'static str>,
    pub filters: Vec<PolicyFilterSummary>,
}

/// Shared provenance summary for explain surfaces.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
pub struct TraceProvenanceSummary {
    pub source_kind: String,
    pub source_reference: String,
    pub lineage_ancestors: Vec<MemoryId>,
    pub relation_to_seed: FieldPresence<&'static str>,
    pub graph_seed: FieldPresence<u64>,
}

/// Shared graph-expansion evidence for explain surfaces.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
pub struct GraphExpansionSummary {
    pub graph_assistance: &'static str,
    pub seed_memory_ids: Vec<u64>,
    pub graph_seed: FieldPresence<u64>,
    pub graph_entry_points: Vec<u64>,
    pub followed_relations: Vec<&'static str>,
    pub supporting_memory_ids: Vec<u64>,
    pub omitted_neighbor_ids: Vec<u64>,
    pub omitted_neighbor_count: usize,
    pub cutoff_reasons: Vec<&'static str>,
}

/// Shared omission summary for explain surfaces.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
pub struct TraceOmissionSummary {
    pub policy_redacted: usize,
    pub threshold_dropped: usize,
    pub dedup_dropped: usize,
    pub budget_capped: usize,
    pub duplicate_collapsed: usize,
    pub low_confidence_suppressed: usize,
    pub stale_bypassed: usize,
    pub confidence_filtered: usize,
    pub graph_omitted_neighbors: usize,
}

impl TraceOmissionSummary {
    /// Builds the shared omission summary from canonical retrieval omission state.
    pub fn from_omission(omission: &OmissionSummary, graph_omitted_neighbors: usize) -> Self {
        Self {
            policy_redacted: omission.policy_redacted,
            threshold_dropped: omission.threshold_dropped,
            dedup_dropped: omission.dedup_dropped,
            budget_capped: omission.budget_capped,
            duplicate_collapsed: omission.duplicate_collapsed,
            low_confidence_suppressed: omission.low_confidence_suppressed,
            stale_bypassed: omission.stale_bypassed,
            confidence_filtered: omission.confidence_filtered,
            graph_omitted_neighbors,
        }
    }

    /// Builds the shared omission summary from one retrieval result set.
    pub fn from_result_set(result_set: &RetrievalResultSet) -> Self {
        Self::from_omission(
            &result_set.omitted_summary,
            GraphExpansionSummary::from_result_set(result_set).omitted_neighbor_count,
        )
    }
}

impl GraphExpansionSummary {
    /// Builds the shared graph-expansion summary from one retrieval result set.
    pub fn from_result_set(result_set: &RetrievalResultSet) -> Self {
        let graph_seed = result_set.provenance_summary.graph_seed.map(|seed| seed.0);
        let mut seed_memory_ids = match result_set.provenance_summary.graph_seed {
            Some(graph_seed) => result_set
                .evidence_pack
                .iter()
                .filter_map(|item| {
                    (item.provenance_summary.graph_seed == Some(graph_seed))
                        .then_some(item.result.memory_id.0)
                })
                .collect::<Vec<_>>(),
            None => Vec::new(),
        };
        seed_memory_ids.sort_unstable();
        seed_memory_ids.dedup();

        let mut graph_entry_points = result_set
            .evidence_pack
            .iter()
            .filter_map(|item| item.provenance_summary.graph_seed.map(|seed| seed.0))
            .collect::<Vec<_>>();
        graph_entry_points.sort_unstable();
        graph_entry_points.dedup();

        let mut supporting_memory_ids = result_set
            .evidence_pack
            .iter()
            .filter(|item| matches!(item.result.entry_lane, EntryLane::Graph))
            .map(|item| item.result.memory_id.0)
            .collect::<Vec<_>>();
        supporting_memory_ids.sort_unstable();
        supporting_memory_ids.dedup();

        let mut omitted_neighbor_ids = result_set
            .deferred_payloads
            .iter()
            .filter_map(|payload| {
                payload
                    .reason
                    .contains("graph_omitted_neighbor")
                    .then_some(payload.memory_id.0)
            })
            .collect::<Vec<_>>();
        omitted_neighbor_ids.sort_unstable();
        omitted_neighbor_ids.dedup();
        let omitted_neighbor_count = omitted_neighbor_ids.len();

        let mut followed_relations = result_set
            .evidence_pack
            .iter()
            .filter_map(|item| {
                item.provenance_summary
                    .relation_to_seed
                    .map(RelationKind::as_str)
            })
            .collect::<Vec<_>>();
        followed_relations.sort_unstable();
        followed_relations.dedup();

        let cutoff_reasons = result_set
            .explain
            .result_reasons
            .iter()
            .filter_map(|reason| match reason.reason_code.as_str() {
                "graph_cutoff_max_depth" => Some("max_depth_reached"),
                "graph_cutoff_budget" => Some("budget_exhausted"),
                "graph_cutoff_policy_namespace" => Some("policy_namespace_blocked"),
                _ => None,
            })
            .collect::<Vec<_>>();

        let graph_assistance = if matches!(
            result_set.explain.recall_plan,
            crate::engine::recall::RecallPlanKind::Tier2ExactThenGraphExpansion
        ) {
            if supporting_memory_ids.is_empty() {
                "graph_considered_without_new_neighbors"
            } else {
                "graph_expanded_supporting_neighbors"
            }
        } else {
            "none"
        };

        Self {
            graph_assistance,
            seed_memory_ids,
            graph_seed: graph_seed
                .map(FieldPresence::Present)
                .unwrap_or(FieldPresence::Absent),
            graph_entry_points,
            followed_relations,
            supporting_memory_ids,
            omitted_neighbor_ids,
            omitted_neighbor_count,
            cutoff_reasons,
        }
    }
}

impl RouteSummary {
    /// Builds the shared machine-readable route summary from a canonical retrieval result set.
    pub fn from_result_set(result_set: &RetrievalResultSet) -> Self {
        let route_family = match result_set.explain.recall_plan {
            crate::engine::recall::RecallPlanKind::ExactIdTier1 => "exact_id_tier1",
            crate::engine::recall::RecallPlanKind::RecentTier1ThenTier2Exact => {
                "recent_tier1_then_tier2_exact"
            }
            crate::engine::recall::RecallPlanKind::Tier2ExactThenGraphExpansion => {
                "tier2_exact_then_graph_expansion"
            }
            crate::engine::recall::RecallPlanKind::Tier2ExactThenTier3Fallback => {
                "tier2_exact_then_tier3_fallback"
            }
        };
        let route_reason = match result_set.explain.route_reason.as_str() {
            "exact memory id provided" | "exact memory id selects the direct Tier1 handle lane" => {
                "exact_memory_id"
            }
            "small lookup for active session can stay on hot recent window before durable fallback"
            | "small session lookup scans the Tier1 recent window before Tier2 exact" => {
                "small_session_lookup"
            }
            "request uses bounded graph expansion from the Tier2-authorized seed shortlist" => {
                "bounded_graph_expansion"
            }
            "request needs broader durable retrieval before cold fallback"
            | "request lacks a direct Tier1 answer and escalates to deeper indexed retrieval" => {
                "broader_durable_retrieval"
            }
            _ => "custom_route_reason",
        };
        let fallback_reason =
            result_set
                .explain
                .trace_stages
                .last()
                .and_then(|stage| match stage {
                    crate::engine::recall::RecallTraceStage::GraphExpansion => {
                        Some("bounded_graph_expansion")
                    }
                    crate::engine::recall::RecallTraceStage::Tier3Fallback => {
                        Some("tier3_fallback")
                    }
                    crate::engine::recall::RecallTraceStage::Tier2Exact
                        if !result_set.explain.tier1_answered_directly
                            && result_set.explain.trace_stages.iter().any(|stage| {
                                matches!(
                                    stage,
                                    crate::engine::recall::RecallTraceStage::Tier1RecentWindow
                                )
                            }) =>
                    {
                        Some("tier1_recent_insufficient")
                    }
                    _ => None,
                });

        Self {
            route_family,
            route_reason,
            tier1_consulted_first: matches!(
                result_set.explain.trace_stages.first(),
                Some(
                    crate::engine::recall::RecallTraceStage::Tier1ExactHandle
                        | crate::engine::recall::RecallTraceStage::Tier1RecentWindow
                )
            ),
            tier1_answered_directly: result_set.explain.tier1_answered_directly,
            routes_to_deeper_tiers: result_set.explain.trace_stages.iter().any(|stage| {
                matches!(
                    stage,
                    crate::engine::recall::RecallTraceStage::Tier2Exact
                        | crate::engine::recall::RecallTraceStage::GraphExpansion
                        | crate::engine::recall::RecallTraceStage::Tier3Fallback
                )
            }),
            candidate_budget: Some(result_set.explain.candidate_budget),
            pre_route_candidate_count: Some(result_set.total_candidates),
            post_route_candidate_count: Some(result_set.evidence_pack.len()),
            fallback_reason,
        }
    }
}

impl ResultReason {
    /// Builds the shared explain reason shape from canonical retrieval reasoning.
    pub fn from_result_reason(reason: &crate::engine::result::ResultReason) -> Self {
        let reason_code = match reason.reason_code.as_str() {
            "score_kept" => "score_kept",
            "no_match" => "no_match",
            "tier2_exact_match" => "tier2_exact_match",
            "query_by_example_seed_materialized" => "query_by_example_seed_materialized",
            "query_by_example_seed_missing" => "query_by_example_seed_missing",
            "query_by_example_candidate_expansion" => "query_by_example_candidate_expansion",
            "temporal_prefilter_metadata_only" => "temporal_prefilter_metadata_only",
            "temporal_payload_deferred" => "temporal_payload_deferred",
            "temporal_landmark_selected" => "temporal_landmark_selected",
            "temporal_landmark_not_selected" => "temporal_landmark_not_selected",
            "contradiction_selected" => "contradiction_selected",
            "contradiction_visible" => "contradiction_visible",
            "contradiction_retained_under_legal_hold" => "contradiction_retained_under_legal_hold",
            "graph_cutoff_max_depth" => "graph_cutoff_max_depth",
            "graph_cutoff_budget" => "graph_cutoff_budget",
            "graph_cutoff_policy_namespace" => "graph_cutoff_policy_namespace",
            _ => "custom_reason_code",
        };
        let reason_family = match reason_code {
            "score_kept" | "no_match" | "tier2_exact_match" => "selection",
            "query_by_example_seed_materialized"
            | "query_by_example_seed_missing"
            | "query_by_example_candidate_expansion" => "query_by_example",
            "temporal_prefilter_metadata_only"
            | "temporal_payload_deferred"
            | "temporal_landmark_selected"
            | "temporal_landmark_not_selected" => "temporal",
            "contradiction_selected"
            | "contradiction_visible"
            | "contradiction_retained_under_legal_hold" => "conflict",
            "graph_cutoff_max_depth" | "graph_cutoff_budget" | "graph_cutoff_policy_namespace" => {
                "graph"
            }
            _ => "custom",
        };
        let route_stage = match reason_code {
            "score_kept"
            | "no_match"
            | "tier2_exact_match"
            | "query_by_example_seed_materialized"
            | "query_by_example_seed_missing"
            | "query_by_example_candidate_expansion"
            | "temporal_prefilter_metadata_only"
            | "temporal_payload_deferred"
            | "temporal_landmark_selected"
            | "temporal_landmark_not_selected"
            | "contradiction_selected"
            | "contradiction_visible"
            | "contradiction_retained_under_legal_hold" => TraceStage::Tier2Exact,
            "graph_cutoff_max_depth" | "graph_cutoff_budget" | "graph_cutoff_policy_namespace" => {
                TraceStage::GraphExpansion
            }
            _ => TraceStage::Packaging,
        };

        Self {
            memory_id: reason.memory_id,
            reason_code: reason_code.to_string(),
            reason_family: reason_family.to_string(),
            route_stage,
            policy_filter_applied: false,
            detail: reason.detail.clone(),
        }
    }
}

impl TracePolicySummary {
    /// Builds the shared policy summary from one retrieval result set.
    pub fn from_result_set(result_set: &RetrievalResultSet) -> Self {
        Self {
            effective_namespace: result_set
                .policy_summary
                .namespace_applied
                .as_str()
                .to_string(),
            policy_family: "namespace",
            outcome_class: result_set.policy_summary.outcome_class,
            blocked_stage: "not_blocked",
            redaction_fields: if result_set.policy_summary.redactions_applied {
                vec!["raw_text"]
            } else {
                Vec::new()
            },
            retention_state: FieldPresence::Absent,
            sharing_scope: FieldPresence::Absent,
            filters: result_set
                .policy_summary
                .filters
                .iter()
                .map(|filter| {
                    PolicyFilterSummary::new(
                        filter.effective_namespace.clone(),
                        filter.policy_family.clone(),
                        filter.outcome_class,
                        filter.blocked_stage.clone(),
                        filter.sharing_scope.clone(),
                        filter.retention_marker.clone(),
                        filter.redaction_fields.clone(),
                    )
                })
                .collect(),
        }
    }
}

impl TraceProvenanceSummary {
    /// Builds the shared provenance summary from one retrieval result set.
    pub fn from_result_set(result_set: &RetrievalResultSet) -> Self {
        Self {
            source_kind: result_set.provenance_summary.source_kind.clone(),
            source_reference: result_set.provenance_summary.source_reference.clone(),
            lineage_ancestors: result_set.provenance_summary.lineage_ancestors.clone(),
            relation_to_seed: result_set
                .provenance_summary
                .relation_to_seed
                .map(RelationKind::as_str)
                .map(FieldPresence::Present)
                .unwrap_or(FieldPresence::Absent),
            graph_seed: result_set
                .provenance_summary
                .graph_seed
                .map(|seed| seed.0)
                .map(FieldPresence::Present)
                .unwrap_or(FieldPresence::Absent),
        }
    }
}

/// Shared inspect summary for passive-observation provenance and retention semantics.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
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

/// Shared cache metrics envelope for explain and inspect surfaces.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
pub struct CacheMetricsSummary {
    pub cache_hit_count: usize,
    pub cache_miss_count: usize,
    pub cache_bypass_count: usize,
    pub cache_invalidation_count: usize,
    pub prefetch_used_count: usize,
    pub prefetch_dropped_count: usize,
    pub cold_fallback_count: usize,
    pub degraded_mode_served: bool,
    pub pre_cache_candidates: Option<usize>,
    pub post_tier1_candidates: Option<usize>,
    pub post_tier2_candidates: Option<usize>,
    pub post_ann_candidates: Option<usize>,
    pub prefetch_added_candidates: Option<usize>,
    pub tier1_item_hit_count: usize,
    pub tier2_query_hit_count: usize,
    pub negative_cache_hit_count: usize,
    pub result_cache_hit_count: usize,
    pub entity_neighborhood_hit_count: usize,
    pub summary_cache_hit_count: usize,
    pub ann_probe_hit_count: usize,
    pub prefetch_hit_count: usize,
    pub cache_traces: Vec<CacheEvalTrace>,
}

impl CacheMetricsSummary {
    pub fn from_cache_traces(
        cache_traces: Vec<CacheEvalTrace>,
        degraded_mode_served: bool,
    ) -> Self {
        let mut summary = Self {
            cache_hit_count: 0,
            cache_miss_count: 0,
            cache_bypass_count: 0,
            cache_invalidation_count: 0,
            prefetch_used_count: 0,
            prefetch_dropped_count: 0,
            cold_fallback_count: 0,
            degraded_mode_served,
            pre_cache_candidates: cache_traces.first().map(|trace| trace.candidates_before),
            post_tier1_candidates: None,
            post_tier2_candidates: None,
            post_ann_candidates: None,
            prefetch_added_candidates: None,
            tier1_item_hit_count: 0,
            tier2_query_hit_count: 0,
            negative_cache_hit_count: 0,
            result_cache_hit_count: 0,
            entity_neighborhood_hit_count: 0,
            summary_cache_hit_count: 0,
            ann_probe_hit_count: 0,
            prefetch_hit_count: 0,
            cache_traces,
        };

        for trace in &summary.cache_traces {
            use crate::observability::{CacheEventLabel, CacheFamilyLabel, CacheLookupOutcome};

            match trace.outcome {
                CacheLookupOutcome::Hit => summary.cache_hit_count += 1,
                CacheLookupOutcome::Miss => summary.cache_miss_count += 1,
                CacheLookupOutcome::Bypass | CacheLookupOutcome::StaleWarning => {
                    summary.cache_bypass_count += 1;
                }
                CacheLookupOutcome::Disabled => {
                    summary.degraded_mode_served = true;
                }
            }

            match trace.cache_event {
                CacheEventLabel::Invalidate => summary.cache_invalidation_count += 1,
                CacheEventLabel::Prefetch => summary.prefetch_used_count += 1,
                _ => {}
            }

            match trace.cache_family {
                CacheFamilyLabel::Tier1Item => {
                    summary.post_tier1_candidates = Some(trace.candidates_after);
                    if matches!(trace.outcome, CacheLookupOutcome::Hit) {
                        summary.tier1_item_hit_count += 1;
                    }
                }
                CacheFamilyLabel::Tier2Query => {
                    summary.post_tier2_candidates = Some(trace.candidates_after);
                    if matches!(trace.outcome, CacheLookupOutcome::Hit) {
                        summary.tier2_query_hit_count += 1;
                    }
                }
                CacheFamilyLabel::AnnProbe => {
                    summary.post_ann_candidates = Some(trace.candidates_after);
                    if matches!(trace.outcome, CacheLookupOutcome::Hit) {
                        summary.ann_probe_hit_count += 1;
                    }
                }
                CacheFamilyLabel::Result => {
                    if matches!(trace.outcome, CacheLookupOutcome::Hit) {
                        summary.result_cache_hit_count += 1;
                    }
                }
                CacheFamilyLabel::Summary => {
                    if matches!(trace.outcome, CacheLookupOutcome::Hit) {
                        summary.summary_cache_hit_count += 1;
                    }
                }
                CacheFamilyLabel::Negative => {
                    if matches!(trace.outcome, CacheLookupOutcome::Hit) {
                        summary.negative_cache_hit_count += 1;
                    }
                }
            }

            if trace.warm_reuse {
                summary.prefetch_hit_count +=
                    usize::from(matches!(trace.cache_event, CacheEventLabel::Prefetch));
            }
        }

        summary
    }
}

/// Shared freshness marker for explain surfaces.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
pub struct FreshnessMarker {
    pub code: &'static str,
    pub detail: &'static str,
}

/// Shared conflict marker for explain surfaces.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
pub struct ConflictMarker {
    pub code: &'static str,
    pub detail: &'static str,
}

/// Shared uncertainty marker for explain surfaces.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
pub struct UncertaintyMarker {
    pub code: &'static str,
    pub detail: &'static str,
}

/// Shared explain trace schema preserved across CLI, daemon, and MCP wrappers.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
pub struct ExplainTraceSchema {
    pub route_summary: RouteSummary,
    pub trace_stages: Vec<TraceStage>,
    pub result_reasons: Vec<ResultReason>,
    pub omitted_summary: TraceOmissionSummary,
    pub graph_expansion: GraphExpansionSummary,
    pub cache_metrics: CacheMetricsSummary,
    pub score_components: Vec<TraceScoreComponent>,
    pub policy_summary: TracePolicySummary,
    pub provenance_summary: TraceProvenanceSummary,
    pub freshness_markers: Vec<FreshnessMarker>,
    pub conflict_markers: Vec<ConflictMarker>,
    pub uncertainty_markers: Vec<UncertaintyMarker>,
}

/// Shared response envelope reused across CLI, daemon, and MCP wrappers.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
pub struct ResponseContext<T> {
    pub ok: bool,
    pub request_id: RequestId,
    pub namespace: NamespaceId,
    pub result: Option<T>,
    pub explain_trace: Option<ExplainTraceSchema>,
    pub route_summary: Option<RouteSummary>,
    pub trace_stages: Vec<TraceStage>,
    pub result_reasons: Vec<ResultReason>,
    pub graph_expansion: Option<GraphExpansionSummary>,
    pub cache_metrics: Option<CacheMetricsSummary>,
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
            explain_trace: None,
            route_summary: None,
            trace_stages: Vec::new(),
            result_reasons: Vec::new(),
            graph_expansion: None,
            cache_metrics: None,
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
    #[allow(clippy::too_many_arguments)]
    pub fn with_trace_schema(
        mut self,
        route_summary: RouteSummary,
        trace_stages: Vec<TraceStage>,
        result_reasons: Vec<ResultReason>,
        omitted_summary: TraceOmissionSummary,
        graph_expansion: GraphExpansionSummary,
        cache_metrics: CacheMetricsSummary,
        score_components: Vec<TraceScoreComponent>,
        policy_summary: TracePolicySummary,
        provenance_summary: TraceProvenanceSummary,
        freshness_markers: Vec<FreshnessMarker>,
        conflict_markers: Vec<ConflictMarker>,
        uncertainty_markers: Vec<UncertaintyMarker>,
    ) -> Self {
        self.explain_trace = Some(ExplainTraceSchema {
            route_summary: route_summary.clone(),
            trace_stages: trace_stages.clone(),
            result_reasons: result_reasons.clone(),
            omitted_summary,
            graph_expansion: graph_expansion.clone(),
            cache_metrics: cache_metrics.clone(),
            score_components,
            policy_summary: policy_summary.clone(),
            provenance_summary: provenance_summary.clone(),
            freshness_markers: freshness_markers.clone(),
            conflict_markers: conflict_markers.clone(),
            uncertainty_markers: uncertainty_markers.clone(),
        });
        self.route_summary = Some(route_summary);
        self.trace_stages = trace_stages;
        self.result_reasons = result_reasons;
        self.graph_expansion = Some(graph_expansion);
        self.cache_metrics = Some(cache_metrics);
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
            explain_trace: None,
            route_summary: None,
            trace_stages: Vec::new(),
            result_reasons: Vec::new(),
            graph_expansion: None,
            cache_metrics: None,
            outcome_class: OutcomeClass::Rejected,
            retryable: error_kind.retryable(),
            error_kind: Some(error_kind),
            partial_success: false,
            remediation: Some(RemediationHint::for_error(
                error_kind,
                error_kind.as_str().to_string(),
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
        ApiModule, AvailabilityPosture, AvailabilityReason, AvailabilitySummary,
        CacheMetricsSummary, ConflictMarker, ContextValidationError, ErrorKind, ExplainTraceSchema,
        FieldPresence, FreshnessMarker, GraphExpansionSummary, NamespaceId,
        PassiveObservationInspectSummary, PolicyContext, PolicyFilterSummary, RemediationHint,
        RemediationStep, RequestContext, RequestId, ResponseContext, ResponseWarning, ResultReason,
        RouteSummary, TraceOmissionSummary, TracePolicySummary, TraceProvenanceSummary,
        TraceScoreComponent, TraceStage, UncertaintyMarker,
    };
    use crate::engine::ranking::{RankingExplain, RerankMetadata};
    use crate::engine::recall::{RecallPlanKind, RecallTraceStage};
    use crate::engine::result::{
        AnsweredFrom, ConflictMarkers, DeferredPayload, DualOutputMode, EntryLane, EvidenceItem,
        EvidenceRole, FreshnessMarkers, OmissionSummary, PackagingMetadata, PayloadState,
        PolicySummary, ProvenanceSummary, RetrievalExplain, RetrievalResult, RetrievalResultSet,
        UncertaintyMarkers,
    };
    use crate::graph::RelationKind;
    use crate::observability::{
        CacheEvalTrace, CacheEventLabel, CacheFamilyLabel, CacheLookupOutcome, CacheReasonLabel,
        GenerationStatusLabel, OutcomeClass, WarmSourceLabel,
    };
    use crate::policy::{
        ConfidenceConstraint, ConfirmationState, OperationClass, PolicyDecision, PolicyGateway,
        PolicyModule, PreflightState, ReversibilityKind, SafeguardAudit,
        SafeguardOutcome as PolicySafeguardOutcome, SafeguardReasonCode, SafeguardRequest,
        SharingAccessDecision, SharingVisibility,
    };
    use crate::types::CanonicalMemoryType;
    use crate::types::SessionId;
    use serde_json::Value;

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
                vec!["raw_text".to_string()],
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
    fn passive_observation_summary_preserves_absent_vs_present_fields() {
        let absent = PassiveObservationInspectSummary::from_encode(
            &crate::engine::encode::PassiveObservationInspect {
                source_kind: "event",
                write_decision: "capture",
                captured_as_observation: false,
                observation_source: None,
                observation_chunk_id: None,
                retention_marker: "absent",
            },
        );
        assert_eq!(absent.observation_source.state_name(), "absent");
        assert_eq!(absent.observation_chunk_id.state_name(), "absent");
        assert_eq!(absent.retention_marker.state_name(), "absent");

        let present = PassiveObservationInspectSummary::from_encode(
            &crate::engine::encode::PassiveObservationInspect {
                source_kind: "observation",
                write_decision: "capture",
                captured_as_observation: true,
                observation_source: Some("passive_observation".into()),
                observation_chunk_id: Some("obs-0000000000000007".into()),
                retention_marker: "volatile_observation",
            },
        );
        assert_eq!(present.observation_source.state_name(), "present");
        assert_eq!(present.observation_chunk_id.state_name(), "present");
        assert_eq!(present.retention_marker.state_name(), "present");
    }

    #[test]
    fn route_summary_from_result_set_preserves_temporal_recall_and_candidate_counts() {
        let result_set = RetrievalResultSet {
            outcome_class: OutcomeClass::Accepted,
            evidence_pack: Vec::new(),
            action_pack: None,
            deferred_payloads: Vec::new(),
            explain: RetrievalExplain {
                recall_plan: RecallPlanKind::RecentTier1ThenTier2Exact,
                route_reason:
                    "small session lookup scans the Tier1 recent window before Tier2 exact"
                        .to_string(),
                tiers_consulted: vec!["tier1_recent".to_string(), "tier2_exact".to_string()],
                trace_stages: vec![
                    RecallTraceStage::Tier1RecentWindow,
                    RecallTraceStage::Tier2Exact,
                ],
                tier1_answered_directly: false,
                candidate_budget: 8,
                time_consumed_ms: Some(7),
                ranking_profile: "balanced".to_string(),
                contradictions_found: 0,
                query_by_example: None,
                result_reasons: Vec::new(),
            },
            policy_summary: PolicySummary {
                namespace_applied: NamespaceId::new("team.temporal").unwrap(),
                outcome_class: OutcomeClass::Accepted,
                redactions_applied: false,
                restrictions_active: Vec::new(),
                filters: Vec::new(),
            },
            provenance_summary: ProvenanceSummary {
                source_kind: "retrieval_pipeline".to_string(),
                source_reference: "temporal_recall".to_string(),
                source_agent: "core_engine".to_string(),
                original_namespace: NamespaceId::new("team.temporal").unwrap(),
                derived_from: None,
                lineage_ancestors: Vec::new(),
                relation_to_seed: None,
                graph_seed: None,
            },
            omitted_summary: OmissionSummary {
                policy_redacted: 0,
                threshold_dropped: 0,
                dedup_dropped: 0,
                budget_capped: 0,
                duplicate_collapsed: 0,
                low_confidence_suppressed: 0,
                stale_bypassed: 0,
                confidence_filtered: 0,
            },
            freshness_markers: FreshnessMarkers {
                oldest_item_days: 0,
                newest_item_days: 0,
                volatile_items_included: false,
                stale_warning: false,
                as_of_tick: Some(42),
            },
            packaging_metadata: PackagingMetadata {
                result_budget: 5,
                token_budget: None,
                graph_assistance: "none".to_string(),
                degraded_summary: None,
                packaging_mode: "evidence_only".to_string(),
                rerank_metadata: None,
            },
            output_mode: DualOutputMode::Balanced,
            truncated: false,
            total_candidates: 3,
        };

        let summary = RouteSummary::from_result_set(&result_set);

        assert_eq!(summary.route_family, "recent_tier1_then_tier2_exact");
        assert_eq!(summary.route_reason, "small_session_lookup");
        assert!(summary.tier1_consulted_first);
        assert!(!summary.tier1_answered_directly);
        assert!(summary.routes_to_deeper_tiers);
        assert_eq!(summary.candidate_budget, Some(8));
        assert_eq!(summary.pre_route_candidate_count, Some(3));
        assert_eq!(summary.post_route_candidate_count, Some(0));
        assert_eq!(summary.fallback_reason, Some("tier1_recent_insufficient"));
    }

    #[test]
    fn graph_expansion_summary_does_not_treat_non_graph_results_as_seeded() {
        let result_set = RetrievalResultSet {
            outcome_class: OutcomeClass::Accepted,
            evidence_pack: vec![EvidenceItem {
                result: RetrievalResult {
                    memory_id: crate::types::MemoryId(7),
                    namespace: NamespaceId::new("team.temporal").unwrap(),
                    session_id: crate::types::SessionId(3),
                    memory_type: CanonicalMemoryType::Event,
                    compact_text: "round trip".into(),
                    role: EvidenceRole::Primary,
                    entry_lane: EntryLane::Recent,
                    payload_state: PayloadState::Inline,
                    score: 900,
                    score_summary: crate::engine::result::ScoreSummary {
                        final_score: 900,
                        total_weighted_score: 900,
                        signal_breakdown: Vec::new(),
                        profile: "balanced".to_string(),
                    },
                    ranking_explain: RankingExplain {
                        final_score: 900,
                        total_weighted_score: 900,
                        signal_breakdown: Vec::new(),
                        profile: "balanced".to_string(),
                        has_conflict: false,
                        contradiction_details: Vec::new(),
                    },
                    contradiction_explains: Vec::new(),
                    conflict_markers: ConflictMarkers {
                        conflict_state: crate::engine::contradiction::ResolutionState::None,
                        conflict_record_ids: Vec::new(),
                        belief_chain_id: None,
                        superseded_by: None,
                        contradiction_lineage_pairs: Vec::new(),
                        resolution_reasons: Vec::new(),
                        audit_visible_count: 0,
                        omitted_conflict_siblings: 0,
                    },
                    uncertainty_markers: UncertaintyMarkers {
                        confidence: 900,
                        uncertainty_score: 0,
                        freshness_uncertainty: None,
                        contradiction_uncertainty: None,
                        missing_evidence_uncertainty: None,
                        corroboration_uncertainty: None,
                        confidence_interval: None,
                    },
                    answered_from: AnsweredFrom::Tier2Indexed,
                    rerank_metadata: RerankMetadata::float32_only(1, 1),
                },
                provenance_summary: ProvenanceSummary {
                    source_kind: "retrieval_pipeline".to_string(),
                    source_reference: "temporal_recall".to_string(),
                    source_agent: "core_engine".to_string(),
                    original_namespace: NamespaceId::new("team.temporal").unwrap(),
                    derived_from: None,
                    lineage_ancestors: Vec::new(),
                    relation_to_seed: None,
                    graph_seed: None,
                },
                freshness_markers: FreshnessMarkers {
                    oldest_item_days: 0,
                    newest_item_days: 0,
                    volatile_items_included: false,
                    stale_warning: false,
                    as_of_tick: Some(42),
                },
                omitted_fields: Vec::new(),
            }],
            action_pack: None,
            deferred_payloads: Vec::new(),
            explain: RetrievalExplain {
                recall_plan: RecallPlanKind::RecentTier1ThenTier2Exact,
                route_reason:
                    "small session lookup scans the Tier1 recent window before Tier2 exact"
                        .to_string(),
                tiers_consulted: vec!["tier1_recent".to_string(), "tier2_exact".to_string()],
                trace_stages: vec![
                    RecallTraceStage::Tier1RecentWindow,
                    RecallTraceStage::Tier2Exact,
                ],
                tier1_answered_directly: false,
                candidate_budget: 8,
                time_consumed_ms: Some(7),
                ranking_profile: "balanced".to_string(),
                contradictions_found: 0,
                query_by_example: None,
                result_reasons: Vec::new(),
            },
            policy_summary: PolicySummary {
                namespace_applied: NamespaceId::new("team.temporal").unwrap(),
                outcome_class: OutcomeClass::Accepted,
                redactions_applied: false,
                restrictions_active: Vec::new(),
                filters: Vec::new(),
            },
            provenance_summary: ProvenanceSummary {
                source_kind: "retrieval_pipeline".to_string(),
                source_reference: "temporal_recall".to_string(),
                source_agent: "core_engine".to_string(),
                original_namespace: NamespaceId::new("team.temporal").unwrap(),
                derived_from: None,
                lineage_ancestors: Vec::new(),
                relation_to_seed: None,
                graph_seed: None,
            },
            omitted_summary: OmissionSummary {
                policy_redacted: 0,
                threshold_dropped: 0,
                dedup_dropped: 0,
                budget_capped: 0,
                duplicate_collapsed: 0,
                low_confidence_suppressed: 0,
                stale_bypassed: 0,
                confidence_filtered: 0,
            },
            freshness_markers: FreshnessMarkers {
                oldest_item_days: 0,
                newest_item_days: 0,
                volatile_items_included: false,
                stale_warning: false,
                as_of_tick: Some(42),
            },
            packaging_metadata: PackagingMetadata {
                result_budget: 5,
                token_budget: None,
                graph_assistance: "none".to_string(),
                degraded_summary: None,
                packaging_mode: "evidence_only".to_string(),
                rerank_metadata: None,
            },
            output_mode: DualOutputMode::Balanced,
            truncated: false,
            total_candidates: 1,
        };

        let graph = GraphExpansionSummary::from_result_set(&result_set);

        assert_eq!(graph.graph_assistance, "none");
        assert!(graph.seed_memory_ids.is_empty());
        assert_eq!(graph.graph_seed, FieldPresence::Absent);
        assert!(graph.graph_entry_points.is_empty());
    }

    #[test]
    fn graph_expansion_summary_tracks_policy_blocked_omissions_and_cutoffs() {
        let result_set = RetrievalResultSet {
            outcome_class: OutcomeClass::Accepted,
            evidence_pack: vec![EvidenceItem {
                result: RetrievalResult {
                    memory_id: crate::types::MemoryId(41),
                    namespace: NamespaceId::new("team.graph").unwrap(),
                    session_id: crate::types::SessionId(9),
                    memory_type: CanonicalMemoryType::Event,
                    compact_text: "seeded graph neighbor".into(),
                    role: EvidenceRole::Supporting,
                    entry_lane: EntryLane::Graph,
                    payload_state: PayloadState::Inline,
                    score: 640,
                    score_summary: crate::engine::result::ScoreSummary {
                        final_score: 640,
                        total_weighted_score: 640,
                        signal_breakdown: Vec::new(),
                        profile: "balanced".to_string(),
                    },
                    ranking_explain: RankingExplain {
                        final_score: 640,
                        total_weighted_score: 640,
                        signal_breakdown: Vec::new(),
                        profile: "balanced".to_string(),
                        has_conflict: false,
                        contradiction_details: Vec::new(),
                    },
                    contradiction_explains: Vec::new(),
                    conflict_markers: ConflictMarkers {
                        conflict_state: crate::engine::contradiction::ResolutionState::None,
                        conflict_record_ids: Vec::new(),
                        belief_chain_id: None,
                        superseded_by: None,
                        contradiction_lineage_pairs: Vec::new(),
                        resolution_reasons: Vec::new(),
                        audit_visible_count: 0,
                        omitted_conflict_siblings: 0,
                    },
                    uncertainty_markers: UncertaintyMarkers {
                        confidence: 700,
                        uncertainty_score: 120,
                        freshness_uncertainty: None,
                        contradiction_uncertainty: None,
                        missing_evidence_uncertainty: None,
                        corroboration_uncertainty: None,
                        confidence_interval: None,
                    },
                    answered_from: AnsweredFrom::Tier2Indexed,
                    rerank_metadata: RerankMetadata::float32_only(1, 1),
                },
                provenance_summary: ProvenanceSummary {
                    source_kind: "retrieval_pipeline".to_string(),
                    source_reference: "graph_expansion".to_string(),
                    source_agent: "core_engine".to_string(),
                    original_namespace: NamespaceId::new("team.graph").unwrap(),
                    derived_from: None,
                    lineage_ancestors: Vec::new(),
                    relation_to_seed: Some(RelationKind::SharedTopic),
                    graph_seed: Some(crate::graph::EntityId(900)),
                },
                freshness_markers: FreshnessMarkers {
                    oldest_item_days: 1,
                    newest_item_days: 0,
                    volatile_items_included: false,
                    stale_warning: false,
                    as_of_tick: Some(77),
                },
                omitted_fields: Vec::new(),
            }],
            action_pack: None,
            deferred_payloads: vec![DeferredPayload {
                memory_id: crate::types::MemoryId(52),
                payload_state: PayloadState::Deferred,
                reason: "graph_omitted_neighbor: policy_namespace_blocked".to_string(),
                hydration_path: "graph://team.graph/omitted/52".to_string(),
            }],
            explain: RetrievalExplain {
                recall_plan: RecallPlanKind::Tier2ExactThenGraphExpansion,
                route_reason:
                    "request uses bounded graph expansion from the Tier2-authorized seed shortlist"
                        .to_string(),
                tiers_consulted: vec!["tier2_exact".to_string(), "graph_expansion".to_string()],
                trace_stages: vec![
                    RecallTraceStage::Tier2Exact,
                    RecallTraceStage::GraphExpansion,
                ],
                tier1_answered_directly: false,
                candidate_budget: 6,
                time_consumed_ms: Some(5),
                ranking_profile: "balanced".to_string(),
                contradictions_found: 0,
                query_by_example: None,
                result_reasons: vec![
                    crate::engine::result::ResultReason {
                        memory_id: Some(crate::types::MemoryId(52)),
                        reason_code: "graph_cutoff_policy_namespace".to_string(),
                        detail: "namespace gate blocked cross-namespace graph neighbor".to_string(),
                    },
                    crate::engine::result::ResultReason {
                        memory_id: None,
                        reason_code: "graph_cutoff_budget".to_string(),
                        detail: "bounded graph planner stopped after policy-filtered omission"
                            .to_string(),
                    },
                ],
            },
            policy_summary: PolicySummary {
                namespace_applied: NamespaceId::new("team.graph").unwrap(),
                outcome_class: OutcomeClass::Accepted,
                redactions_applied: false,
                restrictions_active: vec!["namespace_gate".to_string()],
                filters: Vec::new(),
            },
            provenance_summary: ProvenanceSummary {
                source_kind: "retrieval_pipeline".to_string(),
                source_reference: "graph_expansion".to_string(),
                source_agent: "core_engine".to_string(),
                original_namespace: NamespaceId::new("team.graph").unwrap(),
                derived_from: None,
                lineage_ancestors: Vec::new(),
                relation_to_seed: Some(RelationKind::SharedTopic),
                graph_seed: Some(crate::graph::EntityId(900)),
            },
            omitted_summary: OmissionSummary {
                policy_redacted: 0,
                threshold_dropped: 0,
                dedup_dropped: 0,
                budget_capped: 0,
                duplicate_collapsed: 0,
                low_confidence_suppressed: 0,
                stale_bypassed: 0,
                confidence_filtered: 0,
            },
            freshness_markers: FreshnessMarkers {
                oldest_item_days: 1,
                newest_item_days: 0,
                volatile_items_included: false,
                stale_warning: false,
                as_of_tick: Some(77),
            },
            packaging_metadata: PackagingMetadata {
                result_budget: 3,
                token_budget: None,
                graph_assistance: "graph_expanded_supporting_neighbors".to_string(),
                degraded_summary: None,
                packaging_mode: "evidence_only".to_string(),
                rerank_metadata: None,
            },
            output_mode: DualOutputMode::Balanced,
            truncated: false,
            total_candidates: 2,
        };

        let graph = GraphExpansionSummary::from_result_set(&result_set);
        let omission = TraceOmissionSummary::from_result_set(&result_set);
        let mapped_reasons = result_set
            .explain
            .result_reasons
            .iter()
            .map(ResultReason::from_result_reason)
            .collect::<Vec<_>>();

        assert_eq!(
            graph.graph_assistance,
            "graph_expanded_supporting_neighbors"
        );
        assert_eq!(graph.seed_memory_ids, vec![41]);
        assert_eq!(graph.graph_seed, FieldPresence::Present(900));
        assert_eq!(graph.graph_entry_points, vec![900]);
        assert_eq!(graph.followed_relations, vec!["shared_topic"]);
        assert_eq!(graph.supporting_memory_ids, vec![41]);
        assert_eq!(graph.omitted_neighbor_ids, vec![52]);
        assert_eq!(graph.omitted_neighbor_count, 1);
        assert_eq!(
            graph.cutoff_reasons,
            vec!["policy_namespace_blocked", "budget_exhausted"]
        );
        assert_eq!(omission.graph_omitted_neighbors, 1);
        assert_eq!(mapped_reasons.len(), 2);
        assert_eq!(
            mapped_reasons[0].reason_code,
            "graph_cutoff_policy_namespace"
        );
        assert_eq!(mapped_reasons[0].reason_family, "graph");
        assert_eq!(mapped_reasons[0].route_stage, TraceStage::GraphExpansion);
        assert_eq!(mapped_reasons[1].reason_code, "graph_cutoff_budget");
        assert_eq!(mapped_reasons[1].reason_family, "graph");
        assert_eq!(mapped_reasons[1].route_stage, TraceStage::GraphExpansion);
    }

    #[test]
    fn result_reason_from_temporal_reason_maps_temporal_family_and_stage() {
        let reason = crate::engine::result::ResultReason {
            memory_id: Some(crate::types::MemoryId(21)),
            reason_code: "temporal_landmark_selected".to_string(),
            detail: "landmark \"launch milestone\" opened era \"era-launch-milestone-0001\""
                .to_string(),
        };

        let mapped = ResultReason::from_result_reason(&reason);

        assert_eq!(mapped.memory_id, Some(crate::types::MemoryId(21)));
        assert_eq!(mapped.reason_code, "temporal_landmark_selected");
        assert_eq!(mapped.reason_family, "temporal");
        assert_eq!(mapped.route_stage, TraceStage::Tier2Exact);
        assert!(!mapped.policy_filter_applied);
        assert!(mapped.detail.contains("launch milestone"));
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
                tier1_answered_directly: false,
                routes_to_deeper_tiers: true,
                candidate_budget: Some(8),
                pre_route_candidate_count: Some(3),
                post_route_candidate_count: Some(1),
                fallback_reason: Some("tier1_recent_insufficient"),
            },
            vec![
                TraceStage::PolicyGate,
                TraceStage::Tier1RecentWindow,
                TraceStage::Tier2Exact,
                TraceStage::Packaging,
            ],
            vec![ResultReason {
                memory_id: None,
                reason_code: "tier2_exact_match".to_string(),
                reason_family: "selection".to_string(),
                route_stage: TraceStage::Tier2Exact,
                policy_filter_applied: false,
                detail: "candidate survived bounded ranking".to_string(),
            }],
            TraceOmissionSummary {
                policy_redacted: 0,
                threshold_dropped: 0,
                dedup_dropped: 0,
                budget_capped: 2,
                duplicate_collapsed: 1,
                low_confidence_suppressed: 0,
                stale_bypassed: 0,
                confidence_filtered: 0,
                graph_omitted_neighbors: 0,
            },
            GraphExpansionSummary {
                graph_assistance: "none",
                seed_memory_ids: Vec::new(),
                graph_seed: FieldPresence::Absent,
                graph_entry_points: Vec::new(),
                followed_relations: Vec::new(),
                supporting_memory_ids: Vec::new(),
                omitted_neighbor_ids: Vec::new(),
                omitted_neighbor_count: 0,
                cutoff_reasons: Vec::new(),
            },
            CacheMetricsSummary::from_cache_traces(Vec::new(), false),
            vec![
                TraceScoreComponent {
                    signal_family: "relevance",
                    raw_value: 820,
                    weight: 40,
                },
                TraceScoreComponent {
                    signal_family: "recency",
                    raw_value: 640,
                    weight: 20,
                },
            ],
            TracePolicySummary {
                effective_namespace: "team.gamma".into(),
                policy_family: "namespace",
                outcome_class: OutcomeClass::Accepted,
                blocked_stage: "not_blocked",
                redaction_fields: vec!["raw_text"],
                retention_state: FieldPresence::Absent,
                sharing_scope: FieldPresence::Present("same_namespace"),
                filters: vec![PolicyFilterSummary::new(
                    "team.gamma",
                    "namespace",
                    OutcomeClass::Accepted,
                    "not_blocked",
                    FieldPresence::Present("same_namespace".to_string()),
                    FieldPresence::Absent,
                    vec!["raw_text".to_string()],
                )],
            },
            TraceProvenanceSummary {
                source_kind: "memory".to_string(),
                source_reference: "memory_id".to_string(),
                lineage_ancestors: Vec::new(),
                relation_to_seed: FieldPresence::Absent,
                graph_seed: FieldPresence::Absent,
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
        assert_eq!(response.result_reasons[0].reason_family, "selection");
        assert_eq!(
            response.result_reasons[0].route_stage.as_str(),
            "tier2_exact"
        );
        assert_eq!(response.cache_metrics.as_ref().unwrap().cache_hit_count, 0);
        assert!(response
            .explain_trace
            .as_ref()
            .unwrap()
            .cache_metrics
            .cache_traces
            .is_empty());
        assert!(!response.result_reasons[0].policy_filter_applied);
        assert_eq!(
            response.route_summary.as_ref().unwrap().candidate_budget,
            Some(8)
        );
        assert_eq!(
            response
                .route_summary
                .as_ref()
                .unwrap()
                .pre_route_candidate_count,
            Some(3)
        );
        assert_eq!(
            response
                .route_summary
                .as_ref()
                .unwrap()
                .post_route_candidate_count,
            Some(1)
        );
        assert_eq!(
            response.route_summary.as_ref().unwrap().fallback_reason,
            Some("tier1_recent_insufficient")
        );
        assert!(
            !response
                .route_summary
                .as_ref()
                .unwrap()
                .tier1_answered_directly
        );
        assert_eq!(
            response
                .explain_trace
                .as_ref()
                .unwrap()
                .score_components
                .len(),
            2
        );
        assert_eq!(
            response
                .explain_trace
                .as_ref()
                .unwrap()
                .omitted_summary
                .budget_capped,
            2
        );
        assert_eq!(
            response
                .explain_trace
                .as_ref()
                .unwrap()
                .omitted_summary
                .duplicate_collapsed,
            1
        );
        assert_eq!(
            response.explain_trace.as_ref().unwrap().score_components[0].signal_family,
            "relevance"
        );
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
        assert_eq!(response.policy_summary.as_ref().unwrap().filters.len(), 1);
        assert_eq!(
            response
                .explain_trace
                .as_ref()
                .unwrap()
                .policy_summary
                .filters[0]
                .policy_family,
            "namespace"
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
        assert_eq!(response.uncertainty_markers[0].code, "low_confidence");
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
            response.explain_trace.as_ref().unwrap().trace_stages[0].as_str(),
            "policy_gate"
        );
        assert_eq!(
            response
                .explain_trace
                .as_ref()
                .unwrap()
                .route_summary
                .fallback_reason,
            Some("tier1_recent_insufficient")
        );
        assert_eq!(
            response.explain_trace,
            Some(ExplainTraceSchema {
                route_summary: response.route_summary.clone().unwrap(),
                trace_stages: response.trace_stages.clone(),
                result_reasons: response.result_reasons.clone(),
                omitted_summary: TraceOmissionSummary {
                    policy_redacted: 0,
                    threshold_dropped: 0,
                    dedup_dropped: 0,
                    budget_capped: 2,
                    duplicate_collapsed: 1,
                    low_confidence_suppressed: 0,
                    stale_bypassed: 0,
                    confidence_filtered: 0,
                    graph_omitted_neighbors: 0,
                },
                graph_expansion: GraphExpansionSummary {
                    graph_assistance: "none",
                    seed_memory_ids: Vec::new(),
                    graph_seed: FieldPresence::Absent,
                    graph_entry_points: Vec::new(),
                    followed_relations: Vec::new(),
                    supporting_memory_ids: Vec::new(),
                    omitted_neighbor_ids: Vec::new(),
                    omitted_neighbor_count: 0,
                    cutoff_reasons: Vec::new(),
                },
                cache_metrics: CacheMetricsSummary::from_cache_traces(Vec::new(), false),
                score_components: vec![
                    TraceScoreComponent {
                        signal_family: "relevance",
                        raw_value: 820,
                        weight: 40,
                    },
                    TraceScoreComponent {
                        signal_family: "recency",
                        raw_value: 640,
                        weight: 20,
                    },
                ],
                policy_summary: response.policy_summary.clone().unwrap(),
                provenance_summary: response.provenance_summary.clone().unwrap(),
                freshness_markers: response.freshness_markers.clone(),
                conflict_markers: response.conflict_markers.clone(),
                uncertainty_markers: response.uncertainty_markers.clone(),
            })
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
    fn cache_metrics_serialization_preserves_multi_family_trace_detail() {
        let cache_metrics = CacheMetricsSummary::from_cache_traces(
            vec![
                CacheEvalTrace {
                    cache_family: CacheFamilyLabel::Tier1Item,
                    cache_event: CacheEventLabel::Hit,
                    outcome: CacheLookupOutcome::Hit,
                    cache_reason: None,
                    warm_source: Some(WarmSourceLabel::Tier1ItemCache),
                    generation_status: GenerationStatusLabel::Valid,
                    candidates_before: 8,
                    candidates_after: 3,
                    warm_reuse: true,
                },
                CacheEvalTrace {
                    cache_family: CacheFamilyLabel::Summary,
                    cache_event: CacheEventLabel::Bypass,
                    outcome: CacheLookupOutcome::Bypass,
                    cache_reason: Some(CacheReasonLabel::PolicyBoundary),
                    warm_source: None,
                    generation_status: GenerationStatusLabel::Unknown,
                    candidates_before: 3,
                    candidates_after: 3,
                    warm_reuse: false,
                },
                CacheEvalTrace {
                    cache_family: CacheFamilyLabel::AnnProbe,
                    cache_event: CacheEventLabel::Miss,
                    outcome: CacheLookupOutcome::Miss,
                    cache_reason: Some(CacheReasonLabel::StaleGeneration),
                    warm_source: None,
                    generation_status: GenerationStatusLabel::Stale,
                    candidates_before: 3,
                    candidates_after: 1,
                    warm_reuse: false,
                },
            ],
            false,
        );
        let response = ResponseContext::success(
            NamespaceId::new("team.cache").unwrap(),
            RequestId::new("req-cache-multi-family").unwrap(),
            7u8,
        )
        .with_trace_schema(
            RouteSummary {
                route_family: "tiered_recall",
                route_reason: "bounded multi-family cache proof",
                tier1_consulted_first: true,
                tier1_answered_directly: false,
                routes_to_deeper_tiers: true,
                candidate_budget: Some(8),
                pre_route_candidate_count: Some(8),
                post_route_candidate_count: Some(1),
                fallback_reason: Some("cache_probe_continued"),
            },
            vec![
                TraceStage::Tier1RecentWindow,
                TraceStage::Tier2Exact,
                TraceStage::Packaging,
            ],
            vec![ResultReason {
                memory_id: None,
                reason_code: "tier2_exact_match".to_string(),
                reason_family: "selection".to_string(),
                route_stage: TraceStage::Tier2Exact,
                policy_filter_applied: false,
                detail: "candidate survived bounded ranking after cache evaluation".to_string(),
            }],
            TraceOmissionSummary {
                policy_redacted: 0,
                threshold_dropped: 0,
                dedup_dropped: 0,
                budget_capped: 0,
                duplicate_collapsed: 0,
                low_confidence_suppressed: 0,
                stale_bypassed: 0,
                confidence_filtered: 0,
                graph_omitted_neighbors: 0,
            },
            GraphExpansionSummary {
                graph_assistance: "none",
                seed_memory_ids: Vec::new(),
                graph_seed: FieldPresence::Absent,
                graph_entry_points: Vec::new(),
                followed_relations: Vec::new(),
                supporting_memory_ids: Vec::new(),
                omitted_neighbor_ids: Vec::new(),
                omitted_neighbor_count: 0,
                cutoff_reasons: Vec::new(),
            },
            cache_metrics.clone(),
            vec![TraceScoreComponent {
                signal_family: "relevance",
                raw_value: 820,
                weight: 40,
            }],
            TracePolicySummary {
                effective_namespace: "team.cache".into(),
                policy_family: "namespace",
                outcome_class: OutcomeClass::Accepted,
                blocked_stage: "not_blocked",
                redaction_fields: Vec::new(),
                retention_state: FieldPresence::Absent,
                sharing_scope: FieldPresence::Present("same_namespace"),
                filters: Vec::new(),
            },
            TraceProvenanceSummary {
                source_kind: "memory".to_string(),
                source_reference: "result_set".to_string(),
                lineage_ancestors: Vec::new(),
                relation_to_seed: FieldPresence::Absent,
                graph_seed: FieldPresence::Absent,
            },
            vec![FreshnessMarker {
                code: "fresh",
                detail: "cache-backed result remained fresh enough for packaging",
            }],
            Vec::new(),
            vec![UncertaintyMarker {
                code: "low_uncertainty",
                detail: "bounded evidence had low uncertainty",
            }],
        );

        assert_eq!(
            response.cache_metrics.as_ref().unwrap().cache_traces.len(),
            3
        );
        assert_eq!(response.cache_metrics.as_ref().unwrap().cache_hit_count, 1);
        assert_eq!(response.cache_metrics.as_ref().unwrap().cache_miss_count, 1);
        assert_eq!(
            response.cache_metrics.as_ref().unwrap().cache_bypass_count,
            1
        );
        assert_eq!(
            response
                .cache_metrics
                .as_ref()
                .unwrap()
                .tier1_item_hit_count,
            1
        );
        assert_eq!(
            response
                .cache_metrics
                .as_ref()
                .unwrap()
                .tier2_query_hit_count,
            0
        );
        assert_eq!(
            response
                .cache_metrics
                .as_ref()
                .unwrap()
                .summary_cache_hit_count,
            0
        );
        assert_eq!(
            response.cache_metrics.as_ref().unwrap().ann_probe_hit_count,
            0
        );
        assert_eq!(
            response
                .cache_metrics
                .as_ref()
                .unwrap()
                .pre_cache_candidates,
            Some(8)
        );
        assert_eq!(
            response
                .cache_metrics
                .as_ref()
                .unwrap()
                .post_tier1_candidates,
            Some(3)
        );
        assert_eq!(
            response.cache_metrics.as_ref().unwrap().post_ann_candidates,
            Some(1)
        );
        assert_eq!(
            response
                .explain_trace
                .as_ref()
                .unwrap()
                .cache_metrics
                .cache_traces
                .len(),
            3
        );

        let value = serde_json::to_value(&response).unwrap();
        let cache_metrics_json = &value["cache_metrics"];
        let cache_traces = cache_metrics_json["cache_traces"].as_array().unwrap();
        assert_eq!(cache_traces.len(), 3);
        assert_eq!(cache_traces[0]["cache_family"], "Tier1Item");
        assert_eq!(cache_traces[0]["cache_event"], "Hit");
        assert_eq!(cache_traces[0]["outcome"], "Hit");
        assert_eq!(cache_traces[0]["warm_source"], "Tier1ItemCache");
        assert_eq!(cache_traces[1]["cache_family"], "Summary");
        assert_eq!(cache_traces[1]["cache_event"], "Bypass");
        assert_eq!(cache_traces[1]["cache_reason"], "PolicyBoundary");
        assert_eq!(cache_traces[2]["cache_family"], "AnnProbe");
        assert_eq!(cache_traces[2]["cache_event"], "Miss");
        assert_eq!(cache_traces[2]["cache_reason"], "StaleGeneration");
        assert_eq!(cache_traces[2]["generation_status"], "Stale");
        assert_eq!(cache_metrics_json["cache_hit_count"], 1);
        assert_eq!(cache_metrics_json["cache_miss_count"], 1);
        assert_eq!(cache_metrics_json["cache_bypass_count"], 1);
        assert_eq!(cache_metrics_json["tier1_item_hit_count"], 1);
        assert_eq!(cache_metrics_json["tier2_query_hit_count"], 0);
        assert_eq!(cache_metrics_json["summary_cache_hit_count"], 0);
        assert_eq!(cache_metrics_json["ann_probe_hit_count"], 0);
    }

    #[test]
    fn response_context_and_explain_trace_serializes_with_expected_shape() {
        let response = ResponseContext::success(
            NamespaceId::new("team.gamma").unwrap(),
            RequestId::new("req-serde").unwrap(),
            11u8,
        )
        .with_trace_schema(
            RouteSummary {
                route_family: "tiered_recall",
                route_reason: "bounded tier1 then tier2",
                tier1_consulted_first: true,
                tier1_answered_directly: false,
                routes_to_deeper_tiers: true,
                candidate_budget: Some(8),
                pre_route_candidate_count: Some(3),
                post_route_candidate_count: Some(1),
                fallback_reason: Some("tier1_recent_insufficient"),
            },
            vec![TraceStage::PolicyGate, TraceStage::Tier2Exact],
            vec![ResultReason {
                memory_id: None,
                reason_code: "tier2_exact_match".to_string(),
                reason_family: "selection".to_string(),
                route_stage: TraceStage::Tier2Exact,
                policy_filter_applied: false,
                detail: "candidate survived bounded ranking".to_string(),
            }],
            TraceOmissionSummary {
                policy_redacted: 0,
                threshold_dropped: 0,
                dedup_dropped: 0,
                budget_capped: 1,
                duplicate_collapsed: 0,
                low_confidence_suppressed: 0,
                stale_bypassed: 0,
                confidence_filtered: 0,
                graph_omitted_neighbors: 0,
            },
            GraphExpansionSummary {
                graph_assistance: "none",
                seed_memory_ids: Vec::new(),
                graph_seed: FieldPresence::Absent,
                graph_entry_points: Vec::new(),
                followed_relations: Vec::new(),
                supporting_memory_ids: Vec::new(),
                omitted_neighbor_ids: Vec::new(),
                omitted_neighbor_count: 0,
                cutoff_reasons: Vec::new(),
            },
            CacheMetricsSummary::from_cache_traces(Vec::new(), false),
            vec![TraceScoreComponent {
                signal_family: "relevance",
                raw_value: 820,
                weight: 40,
            }],
            TracePolicySummary {
                effective_namespace: "team.gamma".into(),
                policy_family: "namespace",
                outcome_class: OutcomeClass::Accepted,
                blocked_stage: "not_blocked",
                redaction_fields: vec!["raw_text"],
                retention_state: FieldPresence::Absent,
                sharing_scope: FieldPresence::Present("same_namespace"),
                filters: vec![PolicyFilterSummary::new(
                    "team.gamma",
                    "namespace",
                    OutcomeClass::Accepted,
                    "not_blocked",
                    FieldPresence::Present("same_namespace".to_string()),
                    FieldPresence::Absent,
                    vec!["raw_text".to_string()],
                )],
            },
            TraceProvenanceSummary {
                source_kind: "memory".to_string(),
                source_reference: "memory_id".to_string(),
                lineage_ancestors: Vec::new(),
                relation_to_seed: FieldPresence::Absent,
                graph_seed: FieldPresence::Absent,
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
        .with_safeguard(PolicySafeguardOutcome {
            outcome_class: OutcomeClass::Blocked,
            preflight_state: PreflightState::Blocked,
            operation_class: OperationClass::AuthoritativeRewrite,
            affected_scope: "effective_namespace",
            impact_summary: "authoritative_rewrite_requires_window",
            blocked_reasons: vec![SafeguardReasonCode::MaintenanceWindowRequired],
            preflight_checks: vec![],
            check_results: vec![],
            warnings: vec!["stale_generation"],
            confidence_constraints: Some(ConfidenceConstraint {
                minimum_level: "medium",
                change_my_mind_conditions: vec!["fresh_snapshot"],
            }),
            reversibility: ReversibilityKind::RollbackViaSnapshot,
            confirmation: ConfirmationState {
                required: true,
                force_allowed: false,
                confirmed: false,
                generation_bound: Some(7),
            },
            audit: SafeguardAudit {
                event_kind: "safeguard_evaluation",
                actor_source: "core_policy",
                request_id: "policy-eval",
                preview_id: Some("preview-7"),
                related_run: None,
                scope_handle: "effective_namespace",
            },
            policy_summary: crate::policy::PolicySummary::deny(true),
        })
        .with_passive_observation(PassiveObservationInspectSummary {
            source_kind: "observation",
            write_decision: "capture",
            captured_as_observation: true,
            observation_source: FieldPresence::Present("passive_observation".into()),
            observation_chunk_id: FieldPresence::Present("obs-0000000000000042".into()),
            retention_marker: FieldPresence::Present("volatile_observation"),
        });

        let value = serde_json::to_value(&response).unwrap();
        assert_eq!(
            value
                .get("explain_trace")
                .and_then(Value::as_object)
                .unwrap()["route_summary"]["route_family"],
            "tiered_recall"
        );
        assert_eq!(
            value
                .get("explain_trace")
                .and_then(Value::as_object)
                .unwrap()["policy_summary"]["filters"][0]["policy_family"],
            "namespace"
        );
        assert_eq!(
            value
                .get("explain_trace")
                .and_then(Value::as_object)
                .unwrap()["omitted_summary"]["budget_capped"],
            1
        );
        assert_eq!(
            value
                .get("explain_trace")
                .and_then(Value::as_object)
                .unwrap()["cache_metrics"]["cache_hit_count"],
            0
        );
        assert_eq!(
            value
                .get("cache_metrics")
                .and_then(Value::as_object)
                .unwrap()["cache_miss_count"],
            0
        );
        assert_eq!(
            value.get("safeguard").and_then(Value::as_object).unwrap()["blocked_reasons"][0],
            "MaintenanceWindowRequired"
        );

        assert_eq!(value["ok"], true);
        assert_eq!(value["result"], 11);
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

        let repair_rollback = AvailabilitySummary::degraded(
            vec!["inspect", "durable_lookup"],
            vec!["preview_only"],
            vec![
                AvailabilityReason::RepairRollbackRequired,
                AvailabilityReason::RepairRollbackInProgress,
            ],
            vec![
                RemediationStep::RollbackRecentChange,
                RemediationStep::RunDoctor,
            ],
        );
        assert_eq!(
            repair_rollback.reason_names(),
            vec!["repair_rollback_required", "repair_rollback_in_progress",],
        );
        assert_eq!(
            repair_rollback.recovery_condition_names(),
            vec!["rollback_recent_change", "run_doctor"],
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
        assert!(!blocked_response.partial_success);
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
