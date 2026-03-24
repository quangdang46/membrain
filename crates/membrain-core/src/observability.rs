use crate::engine::recall::{RecallPlanKind, RecallTraceStage};
use crate::engine::result::{PolicySummary, ProvenanceSummary, ResultReason, RetrievalResultSet};
use crate::types::{CanonicalMemoryType, FastPathRouteFamily, LandmarkMetadata, LandmarkSignals};

/// High-level audit event families preserved in append-only storage.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub enum AuditEventCategory {
    Encode,
    Recall,
    Policy,
    Maintenance,
    Archive,
}

impl AuditEventCategory {
    /// Returns the stable machine-readable category name.
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Encode => "encode",
            Self::Recall => "recall",
            Self::Policy => "policy",
            Self::Maintenance => "maintenance",
            Self::Archive => "archive",
        }
    }
}

/// Stable audit event taxonomy for append-only log rows.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub enum AuditEventKind {
    EncodeAccepted,
    EncodeRejected,
    RecallServed,
    RecallDenied,
    PolicyDenied,
    PolicyRedacted,
    MaintenanceRepairStarted,
    MaintenanceRepairCompleted,
    MaintenanceRepairDegraded,
    MaintenanceRepairRollbackTriggered,
    MaintenanceRepairRollbackCompleted,
    MaintenanceMigrationApplied,
    MaintenanceCompactionApplied,
    MaintenanceConsolidationStarted,
    MaintenanceConsolidationCompleted,
    MaintenanceConsolidationPartial,
    MaintenanceReconsolidationApplied,
    MaintenanceReconsolidationDiscarded,
    MaintenanceReconsolidationDeferred,
    MaintenanceReconsolidationBlocked,
    IncidentRecorded,
    ArchiveRecorded,
}

impl AuditEventKind {
    /// Returns the stable machine-readable event name.
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::EncodeAccepted => "encode_accepted",
            Self::EncodeRejected => "encode_rejected",
            Self::RecallServed => "recall_served",
            Self::RecallDenied => "recall_denied",
            Self::PolicyDenied => "policy_denied",
            Self::PolicyRedacted => "policy_redacted",
            Self::MaintenanceRepairStarted => "maintenance_repair_started",
            Self::MaintenanceRepairCompleted => "maintenance_repair_completed",
            Self::MaintenanceRepairDegraded => "maintenance_repair_degraded",
            Self::MaintenanceRepairRollbackTriggered => "maintenance_repair_rollback_triggered",
            Self::MaintenanceRepairRollbackCompleted => "maintenance_repair_rollback_completed",
            Self::MaintenanceMigrationApplied => "maintenance_migration_applied",
            Self::MaintenanceCompactionApplied => "maintenance_compaction_applied",
            Self::MaintenanceConsolidationStarted => "maintenance_consolidation_started",
            Self::MaintenanceConsolidationCompleted => "maintenance_consolidation_completed",
            Self::MaintenanceConsolidationPartial => "maintenance_consolidation_partial",
            Self::MaintenanceReconsolidationApplied => "maintenance_reconsolidation_applied",
            Self::MaintenanceReconsolidationDiscarded => "maintenance_reconsolidation_discarded",
            Self::MaintenanceReconsolidationDeferred => "maintenance_reconsolidation_deferred",
            Self::MaintenanceReconsolidationBlocked => "maintenance_reconsolidation_blocked",
            Self::IncidentRecorded => "incident_recorded",
            Self::ArchiveRecorded => "archive_recorded",
        }
    }

    /// Returns the canonical category for this event kind.
    pub const fn category(self) -> AuditEventCategory {
        match self {
            Self::EncodeAccepted | Self::EncodeRejected => AuditEventCategory::Encode,
            Self::RecallServed | Self::RecallDenied => AuditEventCategory::Recall,
            Self::PolicyDenied | Self::PolicyRedacted => AuditEventCategory::Policy,
            Self::MaintenanceRepairStarted
            | Self::MaintenanceRepairCompleted
            | Self::MaintenanceRepairDegraded
            | Self::MaintenanceRepairRollbackTriggered
            | Self::MaintenanceRepairRollbackCompleted
            | Self::MaintenanceMigrationApplied
            | Self::MaintenanceCompactionApplied
            | Self::MaintenanceConsolidationStarted
            | Self::MaintenanceConsolidationCompleted
            | Self::MaintenanceConsolidationPartial
            | Self::MaintenanceReconsolidationApplied
            | Self::MaintenanceReconsolidationDiscarded
            | Self::MaintenanceReconsolidationDeferred
            | Self::MaintenanceReconsolidationBlocked
            | Self::IncidentRecorded => AuditEventCategory::Maintenance,
            Self::ArchiveRecorded => AuditEventCategory::Archive,
        }
    }
}

/// Canonical outcome classes shared across core APIs and wrappers.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum OutcomeClass {
    Accepted,
    Rejected,
    Partial,
    Preview,
    Blocked,
    Degraded,
}

impl OutcomeClass {
    /// Returns the stable machine-readable retrieval outcome label.
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Accepted => "accepted",
            Self::Rejected => "rejected",
            Self::Partial => "partial",
            Self::Preview => "preview",
            Self::Blocked => "blocked",
            Self::Degraded => "degraded",
        }
    }
}

/// Ordered synchronous stages on the encode fast path.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum EncodeFastPathStage {
    Normalize,
    Fingerprint,
    ShallowClassify,
    ProvisionalSalience,
    LandmarkTagging,
}

impl EncodeFastPathStage {
    /// Returns the stable machine-readable fast-path stage label.
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Normalize => "normalize",
            Self::Fingerprint => "fingerprint",
            Self::ShallowClassify => "shallow_classify",
            Self::ProvisionalSalience => "provisional_salience",
            Self::LandmarkTagging => "landmark_tagging",
        }
    }
}

/// Stable trace artifact for the synchronous encode fast path.
#[derive(Debug, Clone, PartialEq)]
pub struct EncodeFastPathTrace {
    /// Ordered fast-path stages executed before persistence.
    pub stages: [EncodeFastPathStage; 5],
    /// Stable normalization generation used for the normalized envelope.
    pub normalization_generation: &'static str,
    /// Canonical memory family frozen by normalization.
    pub memory_type: CanonicalMemoryType,
    /// Provisional route family selected by shallow classification.
    pub route_family: FastPathRouteFamily,
    /// First-pass salience scalar used for bounded routing inputs.
    pub provisional_salience: u16,
    /// Number of duplicate-hint candidates consulted on the fast path.
    pub duplicate_hint_candidate_count: usize,
    /// Landmark signals consulted for additive temporal enrichment.
    pub landmark_signals: Option<LandmarkSignals>,
    /// Landmark and era metadata derived during the fast path.
    pub landmark: LandmarkMetadata,
    /// Whether the fast path stayed inside its declared bounded latency contract.
    pub stayed_within_latency_budget: bool,
}

/// Tier1 lookup lanes that remain inspectable on the request path.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum Tier1LookupLane {
    ExactHandle,
    RecentWindow,
}

impl Tier1LookupLane {
    /// Returns the stable machine-readable Tier1 lane label.
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::ExactHandle => "exact_handle",
            Self::RecentWindow => "recent_window",
        }
    }
}

/// Machine-readable Tier1 outcomes for exact and recent hot-set reuse.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum Tier1LookupOutcome {
    Hit,
    Miss,
    Bypass,
    StaleBypass,
}

impl Tier1LookupOutcome {
    /// Returns the stable machine-readable Tier1 outcome label.
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Hit => "hit",
            Self::Miss => "miss",
            Self::Bypass => "bypass",
            Self::StaleBypass => "stale_bypass",
        }
    }
}

/// Stable trace artifact for Tier1 exact and recent lookups.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct Tier1LookupTrace {
    /// Which Tier1 lane fired on the request path.
    pub lane: Tier1LookupLane,
    /// Final lookup outcome for that Tier1 lane.
    pub outcome: Tier1LookupOutcome,
    /// Number of recent candidates inspected when scanning one session window.
    pub recent_candidates_inspected: usize,
    /// Whether the recent-window lane produced a hit.
    pub session_window_hit: bool,
    /// Number of heavyweight payload fetches triggered by the Tier1 lane.
    pub payload_fetch_count: usize,
}

/// Machine-readable Tier2 outcomes for metadata-first durable item planning.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum Tier2PrefilterOutcome {
    Ready,
    Bypass,
}

impl Tier2PrefilterOutcome {
    /// Returns the stable machine-readable Tier2 prefilter outcome label.
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Ready => "ready",
            Self::Bypass => "bypass",
        }
    }
}

/// Stable trace artifact for Tier2 metadata-first prefilter and index planning.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct Tier2PrefilterTrace {
    /// Whether the operation stayed on metadata-only durable rows.
    pub outcome: Tier2PrefilterOutcome,
    /// Number of durable metadata candidates exposed to the planner.
    pub metadata_candidate_count: usize,
    /// Number of heavyweight payload fetches triggered before the final cut.
    pub payload_fetch_count: usize,
}

/// Machine-readable admission outcomes for the working-memory controller.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum AdmissionOutcomeKind {
    /// The candidate was dropped before entering controller state.
    Discarded,
    /// The candidate was buffered in working memory without durable promotion.
    Buffered,
    /// The candidate or an overflow victim should be promoted to encode.
    Promoted,
}

impl AdmissionOutcomeKind {
    /// Returns the stable machine-readable working-memory outcome label.
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Discarded => "discarded",
            Self::Buffered => "buffered",
            Self::Promoted => "promoted",
        }
    }
}

/// Stable trace artifact for working-memory admission decisions.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct WorkingMemoryTrace {
    /// The final admission outcome.
    pub outcome: AdmissionOutcomeKind,
    /// Slot pressure observed when the decision was made.
    pub slot_pressure: usize,
    /// Threshold consulted for the decision.
    pub threshold: u16,
    /// Whether an overflow path was involved.
    pub overflowed: bool,
}

/// Machine-readable top-level route summary preserved across surfaces.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct RouteSummary {
    pub route_family: &'static str,
    pub route_reason: &'static str,
    pub tier1_consulted_first: bool,
    pub routes_to_deeper_tiers: bool,
}

impl RouteSummary {
    /// Builds a shared route summary from the canonical retrieval result envelope.
    pub fn from_result_set(result_set: &RetrievalResultSet) -> Self {
        let explain = &result_set.explain;
        Self {
            route_family: route_family(explain.recall_plan),
            route_reason: route_reason_label(&explain.route_reason),
            tier1_consulted_first: explain.trace_stages.iter().all(|stage| {
                !matches!(
                    stage,
                    RecallTraceStage::Tier2Exact
                        | RecallTraceStage::GraphExpansion
                        | RecallTraceStage::Tier3Fallback
                )
            }) || explain
                .trace_stages
                .iter()
                .position(|stage| {
                    matches!(
                        stage,
                        RecallTraceStage::Tier1ExactHandle | RecallTraceStage::Tier1RecentWindow
                    )
                })
                .is_some_and(|tier1_pos| {
                    explain
                        .trace_stages
                        .iter()
                        .position(|stage| {
                            matches!(
                                stage,
                                RecallTraceStage::Tier2Exact
                                    | RecallTraceStage::GraphExpansion
                                    | RecallTraceStage::Tier3Fallback
                            )
                        })
                        .is_none_or(|deeper_pos| tier1_pos < deeper_pos)
                }),
            routes_to_deeper_tiers: explain.trace_stages.iter().any(|stage| {
                matches!(
                    stage,
                    RecallTraceStage::Tier2Exact
                        | RecallTraceStage::GraphExpansion
                        | RecallTraceStage::Tier3Fallback
                )
            }),
        }
    }
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
    pub const fn from_recall(stage: RecallTraceStage) -> Self {
        match stage {
            RecallTraceStage::Tier1ExactHandle => Self::Tier1ExactHandle,
            RecallTraceStage::Tier1RecentWindow => Self::Tier1RecentWindow,
            RecallTraceStage::Tier2Exact => Self::Tier2Exact,
            RecallTraceStage::GraphExpansion => Self::GraphExpansion,
            RecallTraceStage::Tier3Fallback => Self::Tier3Fallback,
        }
    }
}

/// Shared freshness marker for explain surfaces.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct FreshnessMarker {
    pub code: &'static str,
    pub detail: &'static str,
}

/// Shared conflict marker for explain surfaces.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct ConflictMarker {
    pub code: &'static str,
    pub detail: &'static str,
}

/// Shared uncertainty marker for explain surfaces.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct UncertaintyMarker {
    pub code: &'static str,
    pub detail: &'static str,
}

/// Shared policy summary for explain surfaces.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct TracePolicySummary {
    pub effective_namespace: String,
    pub policy_family: &'static str,
    pub outcome_class: OutcomeClass,
    pub blocked_stage: &'static str,
    pub redaction_fields: Vec<&'static str>,
    pub retention_state: &'static str,
    pub sharing_scope: &'static str,
}

impl TracePolicySummary {
    /// Builds a shared policy summary from the canonical retrieval result envelope.
    pub fn from_result_set(result_set: &RetrievalResultSet) -> Self {
        let policy = &result_set.policy_summary;
        Self {
            effective_namespace: policy.namespace_applied.as_str().to_string(),
            policy_family: policy_family(policy),
            outcome_class: policy.outcome_class,
            blocked_stage: blocked_stage(policy),
            redaction_fields: redaction_fields(policy),
            retention_state: retention_state(policy),
            sharing_scope: sharing_scope(policy),
        }
    }
}

/// Shared provenance summary for explain surfaces.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct TraceProvenanceSummary {
    pub source_kind: &'static str,
    pub source_reference: &'static str,
    pub lineage_ancestors: Vec<u64>,
    pub relation_to_seed: crate::api::FieldPresence<&'static str>,
    pub graph_seed: crate::api::FieldPresence<u64>,
}

impl TraceProvenanceSummary {
    /// Builds a shared provenance summary from the canonical retrieval result envelope.
    pub fn from_result_set(result_set: &RetrievalResultSet) -> Self {
        Self::from_provenance(&result_set.provenance_summary)
    }

    /// Builds a shared provenance summary from canonical provenance state.
    pub fn from_provenance(provenance: &ProvenanceSummary) -> Self {
        Self {
            source_kind: source_kind_label(&provenance.source_kind),
            source_reference: source_reference_label(&provenance.source_reference),
            lineage_ancestors: provenance
                .lineage_ancestors
                .iter()
                .map(|memory_id| memory_id.0)
                .collect(),
            relation_to_seed: match provenance.relation_to_seed {
                Some(relation) => crate::api::FieldPresence::Present(relation.as_str()),
                None => crate::api::FieldPresence::Absent,
            },
            graph_seed: match provenance.graph_seed {
                Some(seed) => crate::api::FieldPresence::Present(seed.0),
                None => crate::api::FieldPresence::Absent,
            },
        }
    }
}

/// Machine-readable reason describing why an item appeared or was omitted.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct ExplainResultReason {
    pub memory_id: Option<u64>,
    pub reason_code: &'static str,
    pub detail: String,
}

impl ExplainResultReason {
    /// Builds a shared explain reason from canonical retrieval reasoning.
    pub fn from_result_reason(reason: &ResultReason) -> Self {
        Self {
            memory_id: reason.memory_id.map(|memory_id| memory_id.0),
            reason_code: reason_code_label(&reason.reason_code),
            detail: reason.detail.clone(),
        }
    }
}

/// Stable observability boundary for shared trace and audit vocabularies.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct ObservabilityModule;

impl ObservabilityModule {
    /// Returns the stable component identifier for this observability surface.
    pub const fn component_name(&self) -> &'static str {
        "observability"
    }

    /// Builds the shared route summary and trace stage family from a result set.
    pub fn explain_route(
        &self,
        result_set: &RetrievalResultSet,
    ) -> (RouteSummary, Vec<TraceStage>) {
        let route_summary = RouteSummary::from_result_set(result_set);
        let mut trace_stages = result_set
            .explain
            .trace_stages
            .iter()
            .copied()
            .map(TraceStage::from_recall)
            .collect::<Vec<_>>();
        trace_stages.push(TraceStage::PolicyGate);
        trace_stages.push(TraceStage::Packaging);
        (route_summary, trace_stages)
    }

    /// Builds the shared explain reason family from a result set.
    pub fn explain_result_reasons(
        &self,
        result_set: &RetrievalResultSet,
    ) -> Vec<ExplainResultReason> {
        result_set
            .explain
            .result_reasons
            .iter()
            .map(ExplainResultReason::from_result_reason)
            .collect()
    }

    /// Builds the shared policy and provenance summaries from a result set.
    pub fn explain_policy_and_provenance(
        &self,
        result_set: &RetrievalResultSet,
    ) -> (TracePolicySummary, TraceProvenanceSummary) {
        (
            TracePolicySummary::from_result_set(result_set),
            TraceProvenanceSummary::from_result_set(result_set),
        )
    }

    /// Builds the shared graph-expansion summary from a result set.
    pub fn explain_graph_expansion(
        &self,
        result_set: &RetrievalResultSet,
    ) -> crate::api::GraphExpansionSummary {
        crate::api::GraphExpansionSummary::from_result_set(result_set)
    }

    /// Builds the shared freshness, conflict, and uncertainty marker families.
    pub fn explain_markers(
        &self,
        result_set: &RetrievalResultSet,
    ) -> (
        Vec<FreshnessMarker>,
        Vec<ConflictMarker>,
        Vec<UncertaintyMarker>,
    ) {
        let freshness = vec![freshness_marker(&result_set.freshness_markers)];
        let conflict = result_set
            .evidence_pack
            .iter()
            .filter_map(|item| conflict_marker(&item.result.conflict_markers))
            .collect();
        let uncertainty = result_set
            .evidence_pack
            .iter()
            .map(|item| uncertainty_marker(&item.result.uncertainty_markers))
            .collect();
        (freshness, conflict, uncertainty)
    }
}

fn route_family(plan: RecallPlanKind) -> &'static str {
    match plan {
        RecallPlanKind::ExactIdTier1 => "exact_id_tier1",
        RecallPlanKind::RecentTier1ThenTier2Exact => "recent_tier1_then_tier2_exact",
        RecallPlanKind::Tier2ExactThenGraphExpansion => "tier2_exact_then_graph_expansion",
        RecallPlanKind::Tier2ExactThenTier3Fallback => "tier2_exact_then_tier3_fallback",
    }
}

fn route_reason_label(reason: &str) -> &'static str {
    match reason {
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
    }
}

fn policy_family(policy: &PolicySummary) -> &'static str {
    if policy.filters.is_empty() {
        "none"
    } else {
        "namespace"
    }
}

fn blocked_stage(policy: &PolicySummary) -> &'static str {
    match policy.outcome_class {
        OutcomeClass::Blocked | OutcomeClass::Rejected => "policy_gate",
        _ => "not_blocked",
    }
}

fn redaction_fields(policy: &PolicySummary) -> Vec<&'static str> {
    if policy.redactions_applied {
        vec!["payload"]
    } else {
        Vec::new()
    }
}

fn retention_state(policy: &PolicySummary) -> &'static str {
    if policy
        .restrictions_active
        .iter()
        .any(|restriction| restriction == "retention")
    {
        "retained"
    } else {
        "absent"
    }
}

fn sharing_scope(policy: &PolicySummary) -> &'static str {
    let Some(scope) = policy
        .filters
        .iter()
        .find_map(|filter| match &filter.sharing_scope {
            crate::api::FieldPresence::Present(scope) => Some(scope.as_str()),
            crate::api::FieldPresence::Absent | crate::api::FieldPresence::Redacted => None,
        })
    else {
        return "same_namespace";
    };

    match scope {
        "namespace_only" => "namespace_only",
        "approved_shared" => "approved_shared",
        "same_namespace" => "same_namespace",
        _ => "custom_sharing_scope",
    }
}

fn source_kind_label(source_kind: &str) -> &'static str {
    match source_kind {
        "memory" => "memory",
        "retrieval_pipeline" => "retrieval_pipeline",
        _ => "custom_source_kind",
    }
}

fn source_reference_label(source_reference: &str) -> &'static str {
    match source_reference {
        "memory_id" => "memory_id",
        "result_builder" => "result_builder",
        "result_set" => "result_set",
        "temporal_recall" => "temporal_recall",
        _ => "custom_source_reference",
    }
}

fn reason_code_label(reason_code: &str) -> &'static str {
    match reason_code {
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
        _ => "custom_reason_code",
    }
}

fn freshness_marker(markers: &crate::engine::result::FreshnessMarkers) -> FreshnessMarker {
    if markers.stale_warning {
        FreshnessMarker {
            code: "stale_warning",
            detail: "result set includes stale or aging evidence",
        }
    } else {
        FreshnessMarker {
            code: "fresh",
            detail: "result set freshness remained within the default packaging window",
        }
    }
}

fn conflict_marker(markers: &crate::engine::result::ConflictMarkers) -> Option<ConflictMarker> {
    if markers.conflict_record_ids.is_empty() {
        None
    } else {
        Some(ConflictMarker {
            code: "open_conflict",
            detail: "result retained contradiction-bearing evidence",
        })
    }
}

fn uncertainty_marker(markers: &crate::engine::result::UncertaintyMarkers) -> UncertaintyMarker {
    if markers.uncertainty_score >= 500 {
        UncertaintyMarker {
            code: "high_uncertainty",
            detail: "bounded evidence carried elevated uncertainty",
        }
    } else {
        UncertaintyMarker {
            code: "low_uncertainty",
            detail: "bounded evidence had low uncertainty",
        }
    }
}

/// Machine-readable cache lookup outcome for the observability trace stream.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum CacheLookupOutcome {
    Hit,
    Miss,
    Bypass,
    StaleWarning,
    Disabled,
}

impl CacheLookupOutcome {
    /// Returns the stable machine-readable cache lookup outcome label.
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Hit => "hit",
            Self::Miss => "miss",
            Self::Bypass => "bypass",
            Self::StaleWarning => "stale_warning",
            Self::Disabled => "disabled",
        }
    }
}

/// Machine-readable cache family label for cross-surface observability.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum CacheFamilyLabel {
    Tier1Item,
    Tier2Query,
    AnnProbe,
    Result,
    Summary,
    Negative,
}

impl CacheFamilyLabel {
    /// Returns the stable machine-readable cache family label.
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Tier1Item => "tier1_item",
            Self::Tier2Query => "tier2_query",
            Self::AnnProbe => "ann_probe",
            Self::Result => "result",
            Self::Summary => "summary",
            Self::Negative => "negative",
        }
    }
}

/// Machine-readable cache event label for observability traces.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum CacheEventLabel {
    Hit,
    Miss,
    Bypass,
    Invalidate,
    Prefetch,
}

impl CacheEventLabel {
    /// Returns the stable machine-readable cache event label.
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Hit => "hit",
            Self::Miss => "miss",
            Self::Bypass => "bypass",
            Self::Invalidate => "invalidate",
            Self::Prefetch => "prefetch",
        }
    }
}

/// Machine-readable cache reason label for misses, bypasses, and invalidations.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum CacheReasonLabel {
    PolicyBoundary,
    NamespaceMismatch,
    SnapshotRequired,
    LegalHold,
    StaleGeneration,
    ColdStart,
}

impl CacheReasonLabel {
    /// Returns the stable machine-readable cache reason label.
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::PolicyBoundary => "policy_boundary",
            Self::NamespaceMismatch => "namespace_mismatch",
            Self::SnapshotRequired => "snapshot_required",
            Self::LegalHold => "legal_hold",
            Self::StaleGeneration => "stale_generation",
            Self::ColdStart => "cold_start",
        }
    }
}

/// Machine-readable warm-source label for cache hits and prefetch reuse.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum WarmSourceLabel {
    Tier1ItemCache,
    Tier2QueryCache,
    AnnProbeCache,
    ResultCache,
    SummaryCache,
    PrefetchHints,
    ColdStartMitigation,
}

impl WarmSourceLabel {
    /// Returns the stable machine-readable warm-source label.
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Tier1ItemCache => "tier1_item_cache",
            Self::Tier2QueryCache => "tier2_query_cache",
            Self::AnnProbeCache => "ann_probe_cache",
            Self::ResultCache => "result_cache",
            Self::SummaryCache => "summary_cache",
            Self::PrefetchHints => "prefetch_hints",
            Self::ColdStartMitigation => "cold_start_mitigation",
        }
    }
}

/// Machine-readable generation-validation label for cache traces.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum GenerationStatusLabel {
    Valid,
    Stale,
    Unknown,
}

impl GenerationStatusLabel {
    /// Returns the stable machine-readable generation-status label.
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Valid => "valid",
            Self::Stale => "stale",
            Self::Unknown => "unknown",
        }
    }
}

/// Stable trace artifact for one cache-family evaluation on the request path.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct CacheEvalTrace {
    /// Which cache family was evaluated.
    pub cache_family: CacheFamilyLabel,
    /// Which cache event was emitted for this family.
    pub cache_event: CacheEventLabel,
    /// Stable request-level outcome for the cache evaluation.
    pub outcome: CacheLookupOutcome,
    /// Explicit reason for bypass, miss, or invalidation when present.
    pub cache_reason: Option<CacheReasonLabel>,
    /// Which warm source provided the cached entry when reused.
    pub warm_source: Option<WarmSourceLabel>,
    /// Generation validation status for the cache entry.
    pub generation_status: GenerationStatusLabel,
    /// Candidate count before this cache evaluation.
    pub candidates_before: usize,
    /// Candidate count after this cache evaluation.
    pub candidates_after: usize,
    /// Whether the result came from a warm source.
    pub warm_reuse: bool,
}

/// Machine-readable status for a bounded maintenance queue snapshot.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum MaintenanceQueueStatus {
    Idle,
    Running,
    Partial,
    Completed,
}

impl MaintenanceQueueStatus {
    /// Returns the stable machine-readable maintenance queue status label.
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Idle => "idle",
            Self::Running => "running",
            Self::Partial => "partial",
            Self::Completed => "completed",
        }
    }
}

/// Stable per-run metrics for bounded maintenance scheduling and partial-failure reporting.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct MaintenanceQueueReport {
    /// Stable queue family name.
    pub queue_family: &'static str,
    /// Current queue status after the bounded run.
    pub queue_status: MaintenanceQueueStatus,
    /// Number of jobs waiting before the run started.
    pub queue_depth_before: u32,
    /// Number of jobs still waiting after the run finished.
    pub queue_depth_after: u32,
    /// Number of jobs processed during this bounded run.
    pub jobs_processed: u32,
    /// Total bounded work units processed during the run.
    pub affected_item_count: u32,
    /// Total wall-clock duration captured for the run.
    pub duration_ms: u64,
    /// Number of retry attempts consumed by the run.
    pub retry_attempts: u32,
    /// Whether the run completed with partial-failure reporting.
    pub partial_run: bool,
}

#[cfg(test)]
mod tests {
    use super::{
        AdmissionOutcomeKind, AuditEventCategory, AuditEventKind, CacheEvalTrace, CacheEventLabel,
        CacheFamilyLabel, CacheLookupOutcome, CacheReasonLabel, EncodeFastPathStage,
        ExplainResultReason, GenerationStatusLabel, MaintenanceQueueReport, MaintenanceQueueStatus,
        ObservabilityModule, OutcomeClass, Tier1LookupLane, Tier1LookupOutcome,
        Tier2PrefilterOutcome, TraceStage, WarmSourceLabel,
    };
    use crate::api::{FieldPresence, NamespaceId};
    use crate::engine::recall::{RecallPlanKind, RecallTraceStage};
    use crate::engine::result::{
        FreshnessMarkers, OmissionSummary, PackagingMetadata, PolicySummary, ProvenanceSummary,
        ResultReason, RetrievalExplain, RetrievalResultSet,
    };

    #[test]
    fn retrieval_outcome_class_labels_match_contract() {
        assert_eq!(OutcomeClass::Accepted.as_str(), "accepted");
        assert_eq!(OutcomeClass::Rejected.as_str(), "rejected");
        assert_eq!(OutcomeClass::Partial.as_str(), "partial");
        assert_eq!(OutcomeClass::Preview.as_str(), "preview");
        assert_eq!(OutcomeClass::Blocked.as_str(), "blocked");
        assert_eq!(OutcomeClass::Degraded.as_str(), "degraded");
    }

    #[test]
    fn trace_lane_and_outcome_labels_remain_machine_readable() {
        assert_eq!(EncodeFastPathStage::Normalize.as_str(), "normalize");
        assert_eq!(
            EncodeFastPathStage::ProvisionalSalience.as_str(),
            "provisional_salience"
        );
        assert_eq!(Tier1LookupLane::ExactHandle.as_str(), "exact_handle");
        assert_eq!(Tier1LookupLane::RecentWindow.as_str(), "recent_window");
        assert_eq!(Tier1LookupOutcome::Hit.as_str(), "hit");
        assert_eq!(Tier1LookupOutcome::Miss.as_str(), "miss");
        assert_eq!(Tier1LookupOutcome::Bypass.as_str(), "bypass");
        assert_eq!(Tier1LookupOutcome::StaleBypass.as_str(), "stale_bypass");
        assert_eq!(Tier2PrefilterOutcome::Ready.as_str(), "ready");
        assert_eq!(Tier2PrefilterOutcome::Bypass.as_str(), "bypass");
    }

    #[test]
    fn maintenance_repair_audit_event_labels_cover_degraded_and_rollback() {
        assert_eq!(
            AuditEventKind::MaintenanceRepairDegraded.as_str(),
            "maintenance_repair_degraded"
        );
        assert_eq!(
            AuditEventKind::MaintenanceRepairRollbackTriggered.as_str(),
            "maintenance_repair_rollback_triggered"
        );
        assert_eq!(
            AuditEventKind::MaintenanceRepairRollbackCompleted.as_str(),
            "maintenance_repair_rollback_completed"
        );
        assert_eq!(
            AuditEventKind::MaintenanceRepairDegraded.category(),
            AuditEventCategory::Maintenance
        );
        assert_eq!(
            AuditEventKind::MaintenanceRepairRollbackTriggered.category(),
            AuditEventCategory::Maintenance
        );
        assert_eq!(
            AuditEventKind::MaintenanceRepairRollbackCompleted.category(),
            AuditEventCategory::Maintenance
        );
    }

    #[test]
    fn maintenance_consolidation_audit_events_and_queue_labels_remain_stable() {
        assert_eq!(
            AuditEventKind::MaintenanceConsolidationStarted.as_str(),
            "maintenance_consolidation_started"
        );
        assert_eq!(
            AuditEventKind::MaintenanceConsolidationCompleted.as_str(),
            "maintenance_consolidation_completed"
        );
        assert_eq!(
            AuditEventKind::MaintenanceConsolidationPartial.as_str(),
            "maintenance_consolidation_partial"
        );
        assert_eq!(
            AuditEventKind::MaintenanceConsolidationPartial.category(),
            AuditEventCategory::Maintenance
        );
        assert_eq!(
            AuditEventKind::MaintenanceReconsolidationApplied.as_str(),
            "maintenance_reconsolidation_applied"
        );
        assert_eq!(
            AuditEventKind::MaintenanceReconsolidationDiscarded.as_str(),
            "maintenance_reconsolidation_discarded"
        );
        assert_eq!(
            AuditEventKind::MaintenanceReconsolidationDeferred.as_str(),
            "maintenance_reconsolidation_deferred"
        );
        assert_eq!(
            AuditEventKind::MaintenanceReconsolidationBlocked.as_str(),
            "maintenance_reconsolidation_blocked"
        );
        assert_eq!(
            AuditEventKind::MaintenanceReconsolidationBlocked.category(),
            AuditEventCategory::Maintenance
        );
        assert_eq!(MaintenanceQueueStatus::Idle.as_str(), "idle");
        assert_eq!(MaintenanceQueueStatus::Running.as_str(), "running");
        assert_eq!(MaintenanceQueueStatus::Partial.as_str(), "partial");
        assert_eq!(MaintenanceQueueStatus::Completed.as_str(), "completed");

        let report = MaintenanceQueueReport {
            queue_family: "consolidation",
            queue_status: MaintenanceQueueStatus::Partial,
            queue_depth_before: 4,
            queue_depth_after: 2,
            jobs_processed: 2,
            affected_item_count: 7,
            duration_ms: 41,
            retry_attempts: 1,
            partial_run: true,
        };
        assert_eq!(report.queue_family, "consolidation");
        assert_eq!(report.queue_status, MaintenanceQueueStatus::Partial);
        assert_eq!(report.queue_depth_before, 4);
        assert_eq!(report.queue_depth_after, 2);
        assert_eq!(report.jobs_processed, 2);
        assert_eq!(report.affected_item_count, 7);
        assert_eq!(report.duration_ms, 41);
        assert_eq!(report.retry_attempts, 1);
        assert!(report.partial_run);
    }

    #[test]
    fn working_memory_and_cache_labels_remain_stable() {
        assert_eq!(AdmissionOutcomeKind::Discarded.as_str(), "discarded");
        assert_eq!(AdmissionOutcomeKind::Buffered.as_str(), "buffered");
        assert_eq!(AdmissionOutcomeKind::Promoted.as_str(), "promoted");
        assert_eq!(CacheLookupOutcome::Hit.as_str(), "hit");
        assert_eq!(CacheLookupOutcome::Miss.as_str(), "miss");
        assert_eq!(CacheLookupOutcome::Bypass.as_str(), "bypass");
        assert_eq!(CacheLookupOutcome::StaleWarning.as_str(), "stale_warning");
        assert_eq!(CacheLookupOutcome::Disabled.as_str(), "disabled");
        assert_eq!(CacheFamilyLabel::Tier1Item.as_str(), "tier1_item");
        assert_eq!(CacheFamilyLabel::Tier2Query.as_str(), "tier2_query");
        assert_eq!(CacheFamilyLabel::AnnProbe.as_str(), "ann_probe");
        assert_eq!(CacheFamilyLabel::Result.as_str(), "result");
        assert_eq!(CacheFamilyLabel::Summary.as_str(), "summary");
        assert_eq!(CacheFamilyLabel::Negative.as_str(), "negative");
        assert_eq!(CacheEventLabel::Hit.as_str(), "hit");
        assert_eq!(CacheEventLabel::Miss.as_str(), "miss");
        assert_eq!(CacheEventLabel::Bypass.as_str(), "bypass");
        assert_eq!(CacheEventLabel::Invalidate.as_str(), "invalidate");
        assert_eq!(CacheEventLabel::Prefetch.as_str(), "prefetch");
        assert_eq!(CacheReasonLabel::PolicyBoundary.as_str(), "policy_boundary");
        assert_eq!(
            CacheReasonLabel::NamespaceMismatch.as_str(),
            "namespace_mismatch"
        );
        assert_eq!(
            CacheReasonLabel::SnapshotRequired.as_str(),
            "snapshot_required"
        );
        assert_eq!(CacheReasonLabel::LegalHold.as_str(), "legal_hold");
        assert_eq!(
            CacheReasonLabel::StaleGeneration.as_str(),
            "stale_generation"
        );
        assert_eq!(CacheReasonLabel::ColdStart.as_str(), "cold_start");
        assert_eq!(WarmSourceLabel::Tier1ItemCache.as_str(), "tier1_item_cache");
        assert_eq!(
            WarmSourceLabel::Tier2QueryCache.as_str(),
            "tier2_query_cache"
        );
        assert_eq!(WarmSourceLabel::AnnProbeCache.as_str(), "ann_probe_cache");
        assert_eq!(WarmSourceLabel::ResultCache.as_str(), "result_cache");
        assert_eq!(WarmSourceLabel::SummaryCache.as_str(), "summary_cache");
        assert_eq!(WarmSourceLabel::PrefetchHints.as_str(), "prefetch_hints");
        assert_eq!(
            WarmSourceLabel::ColdStartMitigation.as_str(),
            "cold_start_mitigation"
        );
        assert_eq!(GenerationStatusLabel::Valid.as_str(), "valid");
        assert_eq!(GenerationStatusLabel::Stale.as_str(), "stale");
        assert_eq!(GenerationStatusLabel::Unknown.as_str(), "unknown");
    }

    #[test]
    fn cache_eval_trace_preserves_per_family_observability_shape() {
        let trace = CacheEvalTrace {
            cache_family: CacheFamilyLabel::Result,
            cache_event: CacheEventLabel::Hit,
            outcome: CacheLookupOutcome::Hit,
            cache_reason: None,
            warm_source: Some(WarmSourceLabel::ResultCache),
            generation_status: GenerationStatusLabel::Valid,
            candidates_before: 6,
            candidates_after: 3,
            warm_reuse: true,
        };

        let value = serde_json::to_value(trace).unwrap();
        assert_eq!(value["cache_family"], "Result");
        assert_eq!(value["cache_event"], "Hit");
        assert_eq!(value["outcome"], "Hit");
        assert_eq!(value["warm_source"], "ResultCache");
        assert_eq!(value["generation_status"], "Valid");
        assert_eq!(value["candidates_before"], 6);
        assert_eq!(value["candidates_after"], 3);
        assert_eq!(value["warm_reuse"], true);
    }

    #[test]
    fn explain_builders_project_shared_summary_families() {
        let result_set = RetrievalResultSet {
            outcome_class: OutcomeClass::Accepted,
            evidence_pack: Vec::new(),
            action_pack: None,
            deferred_payloads: Vec::new(),
            explain: RetrievalExplain {
                recall_plan: RecallPlanKind::RecentTier1ThenTier2Exact,
                route_reason:
                    "small lookup for active session can stay on hot recent window before durable fallback"
                        .to_string(),
                tiers_consulted: vec!["tier1_recent".to_string(), "tier2_exact".to_string()],
                trace_stages: vec![RecallTraceStage::Tier1RecentWindow, RecallTraceStage::Tier2Exact],
                tier1_answered_directly: false,
                candidate_budget: 8,
                time_consumed_ms: Some(12),
                ranking_profile: "balanced".to_string(),
                contradictions_found: 0,
                query_by_example: None,
                result_reasons: vec![ResultReason {
                    memory_id: None,
                    reason_code: "tier2_exact_match".to_string(),
                    detail: "candidate survived bounded ranking".to_string(),
                }],
            },
            policy_summary: PolicySummary {
                namespace_applied: NamespaceId::new("team.gamma").unwrap_or_else(|_| std::process::abort()),
                outcome_class: OutcomeClass::Accepted,
                redactions_applied: false,
                restrictions_active: Vec::new(),
                filters: Vec::new(),
            },
            provenance_summary: ProvenanceSummary {
                source_kind: "retrieval_pipeline".to_string(),
                source_reference: "result_builder".to_string(),
                source_agent: "core_engine".to_string(),
                original_namespace: NamespaceId::new("team.gamma")
                    .unwrap_or_else(|_| std::process::abort()),
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
                oldest_item_days: 1,
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
            output_mode: crate::engine::result::DualOutputMode::Balanced,
            truncated: false,
            total_candidates: 1,
        };

        let (route, stages) = ObservabilityModule.explain_route(&result_set);
        let reasons = ObservabilityModule.explain_result_reasons(&result_set);
        let graph = ObservabilityModule.explain_graph_expansion(&result_set);

        assert_eq!(route.route_family, "recent_tier1_then_tier2_exact");
        assert_eq!(route.route_reason, "small_session_lookup");
        assert_eq!(
            stages,
            vec![
                TraceStage::Tier1RecentWindow,
                TraceStage::Tier2Exact,
                TraceStage::PolicyGate,
                TraceStage::Packaging,
            ]
        );
        assert_eq!(
            reasons,
            vec![ExplainResultReason {
                memory_id: None,
                reason_code: "tier2_exact_match",
                detail: "candidate survived bounded ranking".to_string(),
            }]
        );
        assert_eq!(graph.graph_assistance, "none");
        assert_eq!(graph.graph_seed, FieldPresence::Absent);
        assert!(graph.followed_relations.is_empty());
        assert!(graph.omitted_neighbor_ids.is_empty());
    }

    #[test]
    fn explain_trace_provenance_preserves_temporal_recall_source_reference() {
        let provenance = ProvenanceSummary {
            source_kind: "retrieval_pipeline".to_string(),
            source_reference: "temporal_recall".to_string(),
            source_agent: "core_engine".to_string(),
            original_namespace: NamespaceId::new("team.temporal")
                .unwrap_or_else(|_| std::process::abort()),
            derived_from: None,
            lineage_ancestors: Vec::new(),
            relation_to_seed: Some(crate::graph::RelationKind::SharedTopic),
            graph_seed: Some(crate::graph::EntityId(44)),
        };

        let trace = super::TraceProvenanceSummary::from_provenance(&provenance);

        assert_eq!(trace.source_kind, "retrieval_pipeline");
        assert_eq!(trace.source_reference, "temporal_recall");
        assert!(trace.lineage_ancestors.is_empty());
        assert_eq!(
            trace.relation_to_seed,
            FieldPresence::Present("shared_topic")
        );
        assert_eq!(trace.graph_seed, FieldPresence::Present(44));
    }

    #[test]
    fn explain_route_maps_current_recall_reason_strings_to_stable_labels() {
        let fixtures = [
            (
                RecallPlanKind::ExactIdTier1,
                "exact memory id selects the direct Tier1 handle lane",
                vec![RecallTraceStage::Tier1ExactHandle],
                "exact_id_tier1",
                "exact_memory_id",
            ),
            (
                RecallPlanKind::RecentTier1ThenTier2Exact,
                "small session lookup scans the Tier1 recent window before Tier2 exact",
                vec![
                    RecallTraceStage::Tier1RecentWindow,
                    RecallTraceStage::Tier2Exact,
                ],
                "recent_tier1_then_tier2_exact",
                "small_session_lookup",
            ),
            (
                RecallPlanKind::Tier2ExactThenTier3Fallback,
                "request lacks a direct Tier1 answer and escalates to deeper indexed retrieval",
                vec![
                    RecallTraceStage::Tier2Exact,
                    RecallTraceStage::Tier3Fallback,
                ],
                "tier2_exact_then_tier3_fallback",
                "broader_durable_retrieval",
            ),
        ];

        for (plan_kind, route_reason, trace_stages, expected_family, expected_reason) in fixtures {
            let result_set = RetrievalResultSet {
                outcome_class: OutcomeClass::Accepted,
                evidence_pack: Vec::new(),
                action_pack: None,
                deferred_payloads: Vec::new(),
                explain: RetrievalExplain {
                    recall_plan: plan_kind,
                    route_reason: route_reason.to_string(),
                    tiers_consulted: Vec::new(),
                    trace_stages,
                    tier1_answered_directly: matches!(plan_kind, RecallPlanKind::ExactIdTier1),
                    candidate_budget: 4,
                    time_consumed_ms: Some(1),
                    ranking_profile: "balanced".to_string(),
                    contradictions_found: 0,
                    query_by_example: None,
                    result_reasons: Vec::new(),
                },
                policy_summary: PolicySummary {
                    namespace_applied: NamespaceId::new("team.gamma")
                        .unwrap_or_else(|_| std::process::abort()),
                    outcome_class: OutcomeClass::Accepted,
                    redactions_applied: false,
                    restrictions_active: Vec::new(),
                    filters: Vec::new(),
                },
                provenance_summary: ProvenanceSummary {
                    source_kind: "retrieval_pipeline".to_string(),
                    source_reference: "result_set".to_string(),
                    source_agent: "core_engine".to_string(),
                    original_namespace: NamespaceId::new("team.gamma")
                        .unwrap_or_else(|_| std::process::abort()),
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
                    as_of_tick: Some(7),
                },
                packaging_metadata: PackagingMetadata {
                    result_budget: 1,
                    token_budget: None,
                    graph_assistance: "none".to_string(),
                    degraded_summary: None,
                    packaging_mode: "evidence_only".to_string(),
                    rerank_metadata: None,
                },
                output_mode: crate::engine::result::DualOutputMode::Balanced,
                truncated: false,
                total_candidates: 0,
            };

            let (route, _) = ObservabilityModule.explain_route(&result_set);
            assert_eq!(route.route_family, expected_family);
            assert_eq!(route.route_reason, expected_reason);
        }
    }

    #[test]
    fn explain_result_reasons_preserve_temporal_payload_deferred_code() {
        let result_set = RetrievalResultSet {
            outcome_class: OutcomeClass::Accepted,
            evidence_pack: Vec::new(),
            action_pack: None,
            deferred_payloads: Vec::new(),
            explain: RetrievalExplain {
                recall_plan: RecallPlanKind::Tier2ExactThenTier3Fallback,
                route_reason: "request needs broader durable retrieval before cold fallback"
                    .to_string(),
                tiers_consulted: vec!["tier2_exact".to_string()],
                trace_stages: vec![RecallTraceStage::Tier2Exact],
                tier1_answered_directly: false,
                candidate_budget: 4,
                time_consumed_ms: Some(3),
                ranking_profile: "balanced".to_string(),
                contradictions_found: 0,
                query_by_example: None,
                result_reasons: vec![ResultReason {
                    memory_id: Some(crate::types::MemoryId(21)),
                    reason_code: "temporal_payload_deferred".to_string(),
                    detail: "heavyweight Tier2 payload remained deferred until hydration path tier2://team.gamma/payload/0015/21".to_string(),
                }],
            },
            policy_summary: PolicySummary {
                namespace_applied: NamespaceId::new("team.gamma").unwrap_or_else(|_| std::process::abort()),
                outcome_class: OutcomeClass::Accepted,
                redactions_applied: false,
                restrictions_active: Vec::new(),
                filters: Vec::new(),
            },
            provenance_summary: ProvenanceSummary {
                source_kind: "retrieval_pipeline".to_string(),
                source_reference: "result_set".to_string(),
                source_agent: "core_engine".to_string(),
                original_namespace: NamespaceId::new("team.gamma").unwrap_or_else(|_| std::process::abort()),
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
                as_of_tick: Some(7),
            },
            packaging_metadata: PackagingMetadata {
                result_budget: 1,
                token_budget: None,
                graph_assistance: "none".to_string(),
                degraded_summary: None,
                packaging_mode: "evidence_only".to_string(),
                rerank_metadata: None,
            },
            output_mode: crate::engine::result::DualOutputMode::Balanced,
            truncated: false,
            total_candidates: 1,
        };

        let reasons = ObservabilityModule.explain_result_reasons(&result_set);

        assert_eq!(
            reasons,
            vec![ExplainResultReason {
                memory_id: Some(21),
                reason_code: "temporal_payload_deferred",
                detail: "heavyweight Tier2 payload remained deferred until hydration path tier2://team.gamma/payload/0015/21".to_string(),
            }]
        );
    }

    #[test]
    fn explain_result_reasons_preserve_temporal_landmark_selection_codes() {
        let selected = RetrievalResultSet {
            outcome_class: OutcomeClass::Accepted,
            evidence_pack: Vec::new(),
            action_pack: None,
            deferred_payloads: Vec::new(),
            explain: RetrievalExplain {
                recall_plan: RecallPlanKind::Tier2ExactThenTier3Fallback,
                route_reason: "request needs broader durable retrieval before cold fallback"
                    .to_string(),
                tiers_consulted: vec!["tier2_exact".to_string()],
                trace_stages: vec![RecallTraceStage::Tier2Exact],
                tier1_answered_directly: false,
                candidate_budget: 4,
                time_consumed_ms: Some(3),
                ranking_profile: "balanced".to_string(),
                contradictions_found: 0,
                query_by_example: None,
                result_reasons: vec![ResultReason {
                    memory_id: Some(crate::types::MemoryId(21)),
                    reason_code: "temporal_landmark_selected".to_string(),
                    detail: "landmark \"launch milestone\" opened era \"era-launch-milestone-0001\" while staying on metadata-only Tier2 planning".to_string(),
                }],
            },
            policy_summary: PolicySummary {
                namespace_applied: NamespaceId::new("team.gamma").unwrap_or_else(|_| std::process::abort()),
                outcome_class: OutcomeClass::Accepted,
                redactions_applied: false,
                restrictions_active: Vec::new(),
                filters: Vec::new(),
            },
            provenance_summary: ProvenanceSummary {
                source_kind: "retrieval_pipeline".to_string(),
                source_reference: "result_set".to_string(),
                source_agent: "core_engine".to_string(),
                original_namespace: NamespaceId::new("team.gamma").unwrap_or_else(|_| std::process::abort()),
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
                as_of_tick: Some(7),
            },
            packaging_metadata: PackagingMetadata {
                result_budget: 1,
                token_budget: None,
                graph_assistance: "none".to_string(),
                degraded_summary: None,
                packaging_mode: "evidence_only".to_string(),
                rerank_metadata: None,
            },
            output_mode: crate::engine::result::DualOutputMode::Balanced,
            truncated: false,
            total_candidates: 1,
        };
        let not_selected = RetrievalResultSet {
            explain: RetrievalExplain {
                result_reasons: vec![ResultReason {
                    memory_id: Some(crate::types::MemoryId(34)),
                    reason_code: "temporal_landmark_not_selected".to_string(),
                    detail: "memory stayed recallable without landmark promotion or era creation"
                        .to_string(),
                }],
                ..selected.explain.clone()
            },
            ..selected.clone()
        };

        assert_eq!(
            ObservabilityModule.explain_result_reasons(&selected),
            vec![ExplainResultReason {
                memory_id: Some(21),
                reason_code: "temporal_landmark_selected",
                detail: "landmark \"launch milestone\" opened era \"era-launch-milestone-0001\" while staying on metadata-only Tier2 planning".to_string(),
            }]
        );
        assert_eq!(
            ObservabilityModule.explain_result_reasons(&not_selected),
            vec![ExplainResultReason {
                memory_id: Some(34),
                reason_code: "temporal_landmark_not_selected",
                detail: "memory stayed recallable without landmark promotion or era creation"
                    .to_string(),
            }]
        );
    }

    #[test]
    fn explain_result_reasons_preserve_contradiction_reason_codes() {
        let result_set = RetrievalResultSet {
            outcome_class: OutcomeClass::Accepted,
            evidence_pack: Vec::new(),
            action_pack: None,
            deferred_payloads: Vec::new(),
            explain: RetrievalExplain {
                recall_plan: RecallPlanKind::Tier2ExactThenTier3Fallback,
                route_reason: "request needs broader durable retrieval before cold fallback"
                    .to_string(),
                tiers_consulted: vec!["tier2_exact".to_string()],
                trace_stages: vec![RecallTraceStage::Tier2Exact],
                tier1_answered_directly: false,
                candidate_budget: 4,
                time_consumed_ms: Some(3),
                ranking_profile: "balanced".to_string(),
                contradictions_found: 2,
                query_by_example: None,
                result_reasons: vec![
                    ResultReason {
                        memory_id: Some(crate::types::MemoryId(44)),
                        reason_code: "contradiction_selected".to_string(),
                        detail: "manual review kept the preferred branch visible".to_string(),
                    },
                    ResultReason {
                        memory_id: Some(crate::types::MemoryId(12)),
                        reason_code: "contradiction_visible".to_string(),
                        detail: "manual review kept the losing branch inspectable".to_string(),
                    },
                    ResultReason {
                        memory_id: Some(crate::types::MemoryId(45)),
                        reason_code: "contradiction_retained_under_legal_hold".to_string(),
                        detail: "legal hold keeps archived authoritative evidence visible"
                            .to_string(),
                    },
                ],
            },
            policy_summary: PolicySummary {
                namespace_applied: NamespaceId::new("team.gamma")
                    .unwrap_or_else(|_| std::process::abort()),
                outcome_class: OutcomeClass::Accepted,
                redactions_applied: false,
                restrictions_active: Vec::new(),
                filters: Vec::new(),
            },
            provenance_summary: ProvenanceSummary {
                source_kind: "retrieval_pipeline".to_string(),
                source_reference: "result_set".to_string(),
                source_agent: "core_engine".to_string(),
                original_namespace: NamespaceId::new("team.gamma")
                    .unwrap_or_else(|_| std::process::abort()),
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
                as_of_tick: Some(7),
            },
            packaging_metadata: PackagingMetadata {
                result_budget: 1,
                token_budget: None,
                graph_assistance: "none".to_string(),
                degraded_summary: None,
                packaging_mode: "evidence_only".to_string(),
                rerank_metadata: None,
            },
            output_mode: crate::engine::result::DualOutputMode::Balanced,
            truncated: false,
            total_candidates: 3,
        };

        let reasons = ObservabilityModule.explain_result_reasons(&result_set);

        assert_eq!(
            reasons
                .iter()
                .map(|reason| reason.reason_code)
                .collect::<Vec<_>>(),
            vec![
                "contradiction_selected",
                "contradiction_visible",
                "contradiction_retained_under_legal_hold",
            ]
        );
    }

    #[test]
    fn explain_policy_summary_uses_policy_filter_sharing_scope_when_present() {
        let result_set = RetrievalResultSet {
            outcome_class: OutcomeClass::Accepted,
            evidence_pack: Vec::new(),
            action_pack: None,
            deferred_payloads: Vec::new(),
            explain: RetrievalExplain {
                recall_plan: RecallPlanKind::ExactIdTier1,
                route_reason: "exact memory id provided".to_string(),
                tiers_consulted: vec!["tier1_exact".to_string()],
                trace_stages: vec![RecallTraceStage::Tier1ExactHandle],
                tier1_answered_directly: true,
                candidate_budget: 1,
                time_consumed_ms: Some(1),
                ranking_profile: "balanced".to_string(),
                contradictions_found: 0,
                query_by_example: None,
                result_reasons: Vec::new(),
            },
            policy_summary: PolicySummary {
                namespace_applied: NamespaceId::new("team.beta")
                    .unwrap_or_else(|_| std::process::abort()),
                outcome_class: OutcomeClass::Accepted,
                redactions_applied: true,
                restrictions_active: Vec::new(),
                filters: vec![crate::api::PolicyFilterSummary::new(
                    "team.beta",
                    "namespace",
                    OutcomeClass::Accepted,
                    "not_blocked",
                    crate::api::FieldPresence::Present("approved_shared".to_string()),
                    crate::api::FieldPresence::Absent,
                    vec!["payload".to_string()],
                )],
            },
            provenance_summary: ProvenanceSummary {
                source_kind: "memory".to_string(),
                source_reference: "memory_id".to_string(),
                source_agent: "core_engine".to_string(),
                original_namespace: NamespaceId::new("team.beta")
                    .unwrap_or_else(|_| std::process::abort()),
                derived_from: None,
                lineage_ancestors: Vec::new(),
                relation_to_seed: None,
                graph_seed: None,
            },
            omitted_summary: OmissionSummary {
                policy_redacted: 1,
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
                as_of_tick: Some(7),
            },
            packaging_metadata: PackagingMetadata {
                result_budget: 1,
                token_budget: None,
                graph_assistance: "none".to_string(),
                degraded_summary: None,
                packaging_mode: "evidence_only".to_string(),
                rerank_metadata: None,
            },
            output_mode: crate::engine::result::DualOutputMode::Balanced,
            truncated: false,
            total_candidates: 1,
        };

        let (policy, provenance) = ObservabilityModule.explain_policy_and_provenance(&result_set);

        assert_eq!(policy.sharing_scope, "approved_shared");
        assert_eq!(policy.redaction_fields, vec!["payload"]);
        assert_eq!(provenance.source_kind, "memory");
        assert_eq!(provenance.source_reference, "memory_id");
    }
}
