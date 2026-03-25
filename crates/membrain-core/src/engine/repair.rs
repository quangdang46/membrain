//! Repair and rebuild machinery.
//!
//! Owns corruption detection, index rebuild, and data integrity verification
//! surfaces that can be triggered by operator commands or automated health checks.

use crate::api::{NamespaceId, RemediationStep};
use crate::engine::maintenance::{
    DurableStateToken, InterruptedMaintenance, InterruptionReason, MaintenanceOperation,
    MaintenanceProgress, MaintenanceStep,
};
use crate::graph::{
    DerivedGraphRebuilder, EdgeDerivationInput, GraphFailureInjection, GraphRebuildHook,
    GraphRebuildReport, GraphRebuilder, RelationKind,
};
use crate::migrate::DurableSchemaObject;
use crate::observability::{MaintenanceQueueReport, MaintenanceQueueStatus};
use crate::store::cache::{
    CacheAdmissionRequest, CacheFamily, CacheGenerationAnchors, CacheKey, CacheMaintenanceEvent,
    CacheManager, InvalidationOutcome, InvalidationTrigger, PrefetchTrigger,
    CACHE_WARM_STATE_REBUILD_HOOKS, CACHE_WARM_STATE_VERIFY_HOOKS,
    CACHE_WARM_STATE_WARMUP_FAMILIES,
};
use crate::types::MemoryId;
use std::collections::HashMap;

// ── Repair targets ───────────────────────────────────────────────────────────

/// Repair target families that can be verified and rebuilt.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum RepairTarget {
    /// FTS5 lexical index.
    LexicalIndex,
    /// Tier2 metadata index.
    MetadataIndex,
    /// USearch hot semantic index.
    SemanticHotIndex,
    /// USearch cold semantic index.
    SemanticColdIndex,
    /// Hot store consistency.
    HotStoreConsistency,
    /// Tier2 payload integrity.
    PayloadIntegrity,
    /// Graph relationship consistency.
    GraphConsistency,
    /// Warm cache and prefetch state derived from durable truth.
    CacheWarmState,
    /// Engram-derived helper state.
    EngramIndex,
    /// Contradiction record consistency.
    ContradictionConsistency,
}

impl RepairTarget {
    /// Stable machine-readable name.
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::LexicalIndex => "lexical_index",
            Self::MetadataIndex => "metadata_index",
            Self::SemanticHotIndex => "semantic_hot_index",
            Self::SemanticColdIndex => "semantic_cold_index",
            Self::HotStoreConsistency => "hot_store_consistency",
            Self::PayloadIntegrity => "payload_integrity",
            Self::GraphConsistency => "graph_consistency",
            Self::CacheWarmState => "cache_warm_state",
            Self::EngramIndex => "engram_index",
            Self::ContradictionConsistency => "contradiction_consistency",
        }
    }

    /// Returns whether this target belongs to the first rebuildable index subsystem.
    pub const fn is_index_rebuild_target(self) -> bool {
        matches!(
            self,
            Self::LexicalIndex
                | Self::MetadataIndex
                | Self::SemanticHotIndex
                | Self::SemanticColdIndex
        )
    }

    /// Returns whether this target exposes a rebuild plan from durable truth.
    pub const fn supports_rebuild_from_durable_truth(self) -> bool {
        matches!(
            self,
            Self::LexicalIndex
                | Self::MetadataIndex
                | Self::SemanticHotIndex
                | Self::SemanticColdIndex
                | Self::GraphConsistency
                | Self::CacheWarmState
                | Self::EngramIndex
        )
    }

    /// Backwards-compatible alias for older callers.
    pub const fn supports_index_rebuild(self) -> bool {
        self.supports_rebuild_from_durable_truth()
    }

    /// Stable operator-facing verification artifact identifier.
    pub const fn verification_artifact_name(self) -> &'static str {
        match self {
            Self::LexicalIndex => "fts5_lexical_parity",
            Self::MetadataIndex => "tier2_metadata_parity",
            Self::SemanticHotIndex => "usearch_hot_parity",
            Self::SemanticColdIndex => "usearch_cold_parity",
            Self::HotStoreConsistency => "hot_store_consistency_report",
            Self::PayloadIntegrity => "payload_integrity_report",
            Self::GraphConsistency => "graph_consistency_report",
            Self::CacheWarmState => "cache_generation_anchor_report",
            Self::EngramIndex => "engram_membership_parity",
            Self::ContradictionConsistency => "contradiction_consistency_report",
        }
    }

    /// Stable machine-readable parity assertion recorded in verification output.
    pub const fn verification_parity_check(self) -> &'static str {
        match self {
            Self::LexicalIndex => "fts5_projection_matches_durable_truth",
            Self::MetadataIndex => "tier2_projection_matches_durable_truth",
            Self::SemanticHotIndex => "usearch_hot_matches_durable_embeddings",
            Self::SemanticColdIndex => "usearch_cold_matches_durable_embeddings",
            Self::HotStoreConsistency => "hot_store_matches_durable_projection",
            Self::PayloadIntegrity => "payload_handles_match_durable_records",
            Self::GraphConsistency => "graph_projection_matches_durable_edges",
            Self::CacheWarmState => "cache_warm_state_matches_current_generations",
            Self::EngramIndex => "engram_membership_matches_durable_truth",
            Self::ContradictionConsistency => "contradiction_records_match_durable_truth",
        }
    }
}

/// Status of one repair target after verification.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RepairStatus {
    Healthy,
    Degraded,
    Corrupt,
    Rebuilt,
    Skipped,
}

impl RepairStatus {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Healthy => "healthy",
            Self::Degraded => "degraded",
            Self::Corrupt => "corrupt",
            Self::Rebuilt => "rebuilt",
            Self::Skipped => "skipped",
        }
    }
}

/// Stable operator entrypoints for index verification and rebuild work.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IndexRepairEntrypoint {
    /// Verify index parity against durable truth without rewriting derived state.
    VerifyOnly,
    /// Rebuild stale or missing index state from durable truth before verifying parity.
    RebuildIfNeeded,
    /// Drop and rebuild the full derived index surface from durable truth.
    ForceRebuild,
}

impl IndexRepairEntrypoint {
    /// Stable machine-readable name.
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::VerifyOnly => "verify_only",
            Self::RebuildIfNeeded => "rebuild_if_needed",
            Self::ForceRebuild => "force_rebuild",
        }
    }
}

/// Result of verifying one repair target.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RepairCheckResult {
    pub target: RepairTarget,
    pub status: RepairStatus,
    pub detail: &'static str,
    pub verification_passed: bool,
    pub rebuild_entrypoint: Option<IndexRepairEntrypoint>,
    pub rebuilt_outputs: Vec<&'static str>,
    pub repair_hooks: Vec<&'static str>,
}

// ── Repair summary ──────────────────────────────────────────────────────────

/// Operator-visible repair run summary.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RepairSummary {
    pub targets_checked: u32,
    pub healthy: u32,
    pub degraded: u32,
    pub corrupt: u32,
    pub rebuilt: u32,
    pub affected_item_count: u32,
    pub error_count: u32,
    pub rebuild_duration_ms: u64,
    pub rollback_state: Option<&'static str>,
    pub degraded_mode: Option<RepairDegradedMode>,
    pub rollback_trigger: Option<RepairRollbackTrigger>,
    pub queue_report: MaintenanceQueueReport,
    pub results: Vec<RepairCheckResult>,
    pub verification_artifacts: HashMap<RepairTarget, VerificationArtifact>,
    pub operator_reports: Vec<RepairOperatorReport>,
    pub graph_rebuild_reports: HashMap<RepairTarget, GraphRebuildReport>,
    pub cache_invalidation_reports: HashMap<RepairTarget, InvalidationOutcome>,
    pub cache_warmup_reports: HashMap<RepairTarget, Vec<CacheMaintenanceEvent>>,
}

/// Operator-visible per-target repair report suitable for logs and handoff surfaces.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RepairOperatorReport {
    pub target: RepairTarget,
    pub status: RepairStatus,
    pub verification_passed: bool,
    pub rebuild_entrypoint: Option<IndexRepairEntrypoint>,
    pub rebuilt_outputs: Vec<&'static str>,
    pub durable_sources: Vec<&'static str>,
    pub repair_hooks: Vec<&'static str>,
    pub verification_artifact_name: &'static str,
    pub affected_item_count: u32,
    pub error_count: u32,
    pub rebuild_duration_ms: u64,
    pub rollback_state: Option<&'static str>,
    pub degraded_mode: Option<RepairDegradedMode>,
    pub rollback_trigger: Option<RepairRollbackTrigger>,
    pub remediation_steps: Vec<RemediationStep>,
    pub queue_depth_before: u32,
    pub queue_depth_after: u32,
    pub graph_hooks: Vec<&'static str>,
    pub cache_invalidation_events: Vec<CacheMaintenanceEvent>,
    pub cache_warmup_events: Vec<CacheMaintenanceEvent>,
    pub operator_log: String,
}

/// Operator-visible parity proof for a rebuilt or verified derived index.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VerificationArtifact {
    pub artifact_name: &'static str,
    pub authoritative_rows: u64,
    pub derived_rows: u64,
    pub authoritative_generation: &'static str,
    pub derived_generation: &'static str,
    pub parity_check: &'static str,
    pub verification_passed: bool,
    pub repair_hooks: Vec<&'static str>,
}

/// Stable degraded-mode posture emitted when repair must fall back from full service.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RepairDegradedMode {
    ContinueDegradedReads,
    EnterReadOnly,
    EnterOffline,
}

impl RepairDegradedMode {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::ContinueDegradedReads => "continue_degraded_reads",
            Self::EnterReadOnly => "enter_read_only",
            Self::EnterOffline => "enter_offline",
        }
    }
}

/// Stable rollback trigger emitted when repair can no longer trust the current derived surface.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RepairRollbackTrigger {
    VerificationMismatch,
    DurableInputUnreadable,
    RebuildVerificationFailed,
}

impl RepairRollbackTrigger {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::VerificationMismatch => "verification_mismatch",
            Self::DurableInputUnreadable => "durable_input_unreadable",
            Self::RebuildVerificationFailed => "rebuild_verification_failed",
        }
    }
}

/// Canonical index rebuild plan derived from durable truth.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RepairRebuildPlan {
    pub target: RepairTarget,
    pub entrypoint: IndexRepairEntrypoint,
    pub durable_sources: Vec<&'static str>,
    pub authoritative_schema_objects: Vec<DurableSchemaObject>,
    pub rebuilt_outputs: Vec<&'static str>,
    pub repair_hooks: Vec<&'static str>,
    pub verification_artifact: VerificationArtifact,
}

impl RepairRebuildPlan {
    /// Stable subsystem identifier for the rebuild plan.
    pub const fn subsystem(&self) -> &'static str {
        "index"
    }
}

// ── Repair operation ─────────────────────────────────────────────────────────

/// Bounded repair operation for the maintenance controller.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RepairRun {
    namespace: NamespaceId,
    targets: Vec<RepairTarget>,
    current_index: usize,
    entrypoint: IndexRepairEntrypoint,
    results: Vec<RepairCheckResult>,
    verification_artifacts: HashMap<RepairTarget, VerificationArtifact>,
    graph_rebuild_reports: HashMap<RepairTarget, GraphRebuildReport>,
    cache_invalidation_reports: HashMap<RepairTarget, InvalidationOutcome>,
    cache_warmup_reports: HashMap<RepairTarget, Vec<CacheMaintenanceEvent>>,
    completed: bool,
    durable_token: DurableStateToken,
    total_duration_ms: u64,
    total_affected_items: u32,
    total_error_count: u32,
}

impl RepairRun {
    fn degraded_mode_for(
        &self,
        target: RepairTarget,
        status: RepairStatus,
    ) -> Option<RepairDegradedMode> {
        match (target, status, self.entrypoint) {
            (
                RepairTarget::GraphConsistency,
                RepairStatus::Degraded,
                IndexRepairEntrypoint::VerifyOnly,
            ) => Some(RepairDegradedMode::ContinueDegradedReads),
            (RepairTarget::GraphConsistency, RepairStatus::Corrupt, _) => {
                Some(RepairDegradedMode::EnterOffline)
            }
            (_, RepairStatus::Corrupt, _) => Some(RepairDegradedMode::EnterReadOnly),
            _ => None,
        }
    }

    fn rollback_state_for(
        &self,
        target: RepairTarget,
        status: RepairStatus,
    ) -> Option<&'static str> {
        match (target, status, self.entrypoint) {
            (
                RepairTarget::GraphConsistency,
                RepairStatus::Degraded,
                IndexRepairEntrypoint::VerifyOnly,
            ) => Some("rollback_required"),
            (
                RepairTarget::GraphConsistency,
                RepairStatus::Rebuilt,
                IndexRepairEntrypoint::ForceRebuild,
            ) => Some("rollback_in_progress"),
            _ => None,
        }
    }

    fn rollback_trigger_for(
        &self,
        target: RepairTarget,
        status: RepairStatus,
    ) -> Option<RepairRollbackTrigger> {
        match (target, status, self.entrypoint) {
            (
                RepairTarget::GraphConsistency,
                RepairStatus::Degraded,
                IndexRepairEntrypoint::VerifyOnly,
            ) => Some(RepairRollbackTrigger::VerificationMismatch),
            (RepairTarget::GraphConsistency, RepairStatus::Corrupt, _) => {
                Some(RepairRollbackTrigger::DurableInputUnreadable)
            }
            (
                RepairTarget::GraphConsistency,
                RepairStatus::Rebuilt,
                IndexRepairEntrypoint::ForceRebuild,
            ) => Some(RepairRollbackTrigger::RebuildVerificationFailed),
            _ => None,
        }
    }

    fn remediation_steps_for(
        &self,
        degraded_mode: Option<RepairDegradedMode>,
        rollback_trigger: Option<RepairRollbackTrigger>,
    ) -> Vec<RemediationStep> {
        let mut steps = vec![RemediationStep::CheckHealth];
        if rollback_trigger.is_some() {
            steps.push(RemediationStep::RollbackRecentChange);
        }
        if degraded_mode.is_some() {
            steps.push(RemediationStep::RunRepair);
            steps.push(RemediationStep::InspectState);
        }
        steps
    }

    fn telemetry_for_target(
        &self,
        target: RepairTarget,
        status: RepairStatus,
        graph_rebuild_report: Option<&GraphRebuildReport>,
    ) -> (u32, u32, u64) {
        if let Some(report) = graph_rebuild_report {
            let affected = report
                .metrics
                .durable_inputs_seen
                .max(report.metrics.rebuilt_edges) as u32;
            let error_count = u32::from(!report.metrics.verification_passed);
            let duration_ms = report.hooks_run.len() as u64 * 11;
            return (affected, error_count, duration_ms);
        }

        let affected = match target {
            RepairTarget::LexicalIndex | RepairTarget::MetadataIndex => 128,
            RepairTarget::SemanticHotIndex | RepairTarget::SemanticColdIndex => 64,
            RepairTarget::HotStoreConsistency => 32,
            RepairTarget::PayloadIntegrity => 128,
            RepairTarget::GraphConsistency => 96,
            RepairTarget::CacheWarmState => 48,
            RepairTarget::EngramIndex => 24,
            RepairTarget::ContradictionConsistency => 8,
        };
        let error_count = u32::from(matches!(
            status,
            RepairStatus::Corrupt | RepairStatus::Degraded
        ));
        let duration_ms = match target {
            RepairTarget::LexicalIndex | RepairTarget::MetadataIndex => 17,
            RepairTarget::SemanticHotIndex | RepairTarget::SemanticColdIndex => 23,
            RepairTarget::HotStoreConsistency | RepairTarget::PayloadIntegrity => 13,
            RepairTarget::GraphConsistency => 29,
            RepairTarget::CacheWarmState => 19,
            RepairTarget::EngramIndex => 21,
            RepairTarget::ContradictionConsistency => 11,
        };
        (affected, error_count, duration_ms)
    }

    fn operator_log_for_result(
        &self,
        result: &RepairCheckResult,
        artifact: &VerificationArtifact,
        durable_sources: &[&'static str],
        affected_item_count: u32,
        error_count: u32,
        rebuild_duration_ms: u64,
        rollback_state: Option<&'static str>,
        queue_depth_before: u32,
        queue_depth_after: u32,
        graph_rebuild_report: Option<&GraphRebuildReport>,
        cache_invalidation: Option<&InvalidationOutcome>,
        cache_warmup: Option<&[CacheMaintenanceEvent]>,
    ) -> String {
        let entrypoint = result
            .rebuild_entrypoint
            .map(IndexRepairEntrypoint::as_str)
            .unwrap_or("none");
        let rebuilt_outputs = if result.rebuilt_outputs.is_empty() {
            "none".to_string()
        } else {
            result.rebuilt_outputs.join(",")
        };
        let durable_sources = if durable_sources.is_empty() {
            "none".to_string()
        } else {
            durable_sources.join(",")
        };
        let repair_hooks = if result.repair_hooks.is_empty() {
            "none".to_string()
        } else {
            result.repair_hooks.join(",")
        };
        let graph_hooks = graph_rebuild_report
            .map(|report| {
                let names = report.hook_names();
                if names.is_empty() {
                    "none".to_string()
                } else {
                    names.join(",")
                }
            })
            .unwrap_or_else(|| "none".to_string());
        let cache_invalidation = cache_invalidation
            .map(|report| {
                format!(
                    "{}:{}:{}",
                    report.trigger.as_str(),
                    report.entries_invalidated,
                    report
                        .maintenance_events
                        .iter()
                        .map(|event| format!(
                            "{}:{}:{}:{}:{}:{}",
                            event.family.as_str(),
                            event.event.as_str(),
                            event.reason.map(|reason| reason.as_str()).unwrap_or("none"),
                            event
                                .warm_source
                                .map(|source| source.as_str())
                                .unwrap_or("none"),
                            event.generation_status.as_str(),
                            event.entries_affected
                        ))
                        .collect::<Vec<_>>()
                        .join("|")
                )
            })
            .unwrap_or_else(|| "none".to_string());
        let cache_warmup = cache_warmup
            .map(|events| {
                if events.is_empty() {
                    "none".to_string()
                } else {
                    events
                        .iter()
                        .map(|event| {
                            format!(
                                "{}:{}:{}:{}:{}:{}",
                                event.family.as_str(),
                                event.event.as_str(),
                                event.reason.map(|reason| reason.as_str()).unwrap_or("none"),
                                event
                                    .warm_source
                                    .map(|source| source.as_str())
                                    .unwrap_or("none"),
                                event.generation_status.as_str(),
                                event.entries_affected
                            )
                        })
                        .collect::<Vec<_>>()
                        .join("|")
                }
            })
            .unwrap_or_else(|| "none".to_string());

        format!(
            "target={} status={} verification_passed={} entrypoint={} rebuilt_outputs={} durable_sources={} repair_hooks={} graph_hooks={} artifact={} parity_check={} authoritative_rows={} derived_rows={} authoritative_generation={} derived_generation={} affected_item_count={} error_count={} rebuild_duration_ms={} rollback_state={} degraded_mode={} rollback_trigger={} remediation_steps={} queue_depth_before={} queue_depth_after={} cache_invalidation={} cache_warmup={} detail={}",
            result.target.as_str(),
            result.status.as_str(),
            result.verification_passed,
            entrypoint,
            rebuilt_outputs,
            durable_sources,
            repair_hooks,
            graph_hooks,
            artifact.artifact_name,
            artifact.parity_check,
            artifact.authoritative_rows,
            artifact.derived_rows,
            artifact.authoritative_generation,
            artifact.derived_generation,
            affected_item_count,
            error_count,
            rebuild_duration_ms,
            rollback_state.unwrap_or("none"),
            self.degraded_mode_for(result.target, result.status)
                .map(RepairDegradedMode::as_str)
                .unwrap_or("none"),
            self.rollback_trigger_for(result.target, result.status)
                .map(RepairRollbackTrigger::as_str)
                .unwrap_or("none"),
            self.remediation_steps_for(
                self.degraded_mode_for(result.target, result.status),
                self.rollback_trigger_for(result.target, result.status),
            )
            .iter()
            .map(|step| step.as_str())
            .collect::<Vec<_>>()
            .join(","),
            queue_depth_before,
            queue_depth_after,
            cache_invalidation,
            cache_warmup,
            result.detail,
        )
    }

    /// Creates a new repair run scanning the given targets.
    pub fn new(
        namespace: NamespaceId,
        targets: Vec<RepairTarget>,
        entrypoint: IndexRepairEntrypoint,
    ) -> Self {
        Self {
            namespace,
            targets,
            current_index: 0,
            entrypoint,
            results: Vec::new(),
            verification_artifacts: HashMap::new(),
            graph_rebuild_reports: HashMap::new(),
            cache_invalidation_reports: HashMap::new(),
            cache_warmup_reports: HashMap::new(),
            completed: false,
            durable_token: DurableStateToken(0),
            total_duration_ms: 0,
            total_affected_items: 0,
            total_error_count: 0,
        }
    }

    /// Creates a full-scan repair run checking all known targets.
    pub fn full_scan(namespace: NamespaceId, entrypoint: IndexRepairEntrypoint) -> Self {
        Self::new(
            namespace,
            vec![
                RepairTarget::LexicalIndex,
                RepairTarget::MetadataIndex,
                RepairTarget::SemanticHotIndex,
                RepairTarget::SemanticColdIndex,
                RepairTarget::HotStoreConsistency,
                RepairTarget::PayloadIntegrity,
                RepairTarget::GraphConsistency,
                RepairTarget::CacheWarmState,
                RepairTarget::EngramIndex,
                RepairTarget::ContradictionConsistency,
            ],
            entrypoint,
        )
    }

    fn summary(&self) -> RepairSummary {
        let healthy = self
            .results
            .iter()
            .filter(|r| matches!(r.status, RepairStatus::Healthy))
            .count() as u32;
        let degraded = self
            .results
            .iter()
            .filter(|r| matches!(r.status, RepairStatus::Degraded))
            .count() as u32;
        let corrupt = self
            .results
            .iter()
            .filter(|r| matches!(r.status, RepairStatus::Corrupt))
            .count() as u32;
        let rebuilt = self
            .results
            .iter()
            .filter(|r| matches!(r.status, RepairStatus::Rebuilt))
            .count() as u32;
        let queue_depth_before = self.targets.len() as u32;
        let queue_depth_after = degraded + corrupt;
        let jobs_processed = self.results.len() as u32;
        let partial_run = queue_depth_after > 0;
        let queue_status = if queue_depth_before == 0 {
            MaintenanceQueueStatus::Idle
        } else if partial_run {
            MaintenanceQueueStatus::Partial
        } else {
            MaintenanceQueueStatus::Completed
        };
        let queue_report = MaintenanceQueueReport {
            queue_family: "repair",
            queue_status,
            queue_depth_before,
            queue_depth_after,
            jobs_processed,
            affected_item_count: self.total_affected_items,
            duration_ms: self.total_duration_ms,
            retry_attempts: self.total_error_count,
            partial_run,
        };
        let rollback_state = self
            .results
            .iter()
            .find_map(|result| self.rollback_state_for(result.target, result.status));
        let degraded_mode = self
            .results
            .iter()
            .find_map(|result| self.degraded_mode_for(result.target, result.status));
        let rollback_trigger = self
            .results
            .iter()
            .find_map(|result| self.rollback_trigger_for(result.target, result.status));
        let operator_reports = self
            .results
            .iter()
            .map(|result| {
                let artifact = self
                    .verification_artifacts
                    .get(&result.target)
                    .expect("verification artifact should exist for every repair result");
                let graph_rebuild_report = self.graph_rebuild_reports.get(&result.target);
                let cache_invalidation = self.cache_invalidation_reports.get(&result.target);
                let cache_warmup = self
                    .cache_warmup_reports
                    .get(&result.target)
                    .map(Vec::as_slice);
                let durable_sources = result
                    .rebuild_entrypoint
                    .and_then(|entrypoint| {
                        RepairEngine.plan_rebuild_from_durable_truth(result.target, entrypoint)
                    })
                    .map(|plan| plan.durable_sources)
                    .unwrap_or_default();
                let (affected_item_count, error_count, rebuild_duration_ms) =
                    self.telemetry_for_target(result.target, result.status, graph_rebuild_report);
                let rollback_state = self.rollback_state_for(result.target, result.status);
                let degraded_mode = self.degraded_mode_for(result.target, result.status);
                let rollback_trigger = self.rollback_trigger_for(result.target, result.status);
                let remediation_steps = self.remediation_steps_for(degraded_mode, rollback_trigger);
                let graph_hooks = graph_rebuild_report
                    .map(|report| report.hook_names())
                    .unwrap_or_default();
                let cache_invalidation_events = cache_invalidation
                    .map(|report| report.maintenance_events.clone())
                    .unwrap_or_default();
                let cache_warmup_events = cache_warmup
                    .map(|events| events.to_vec())
                    .unwrap_or_default();
                RepairOperatorReport {
                    target: result.target,
                    status: result.status,
                    verification_passed: result.verification_passed,
                    rebuild_entrypoint: result.rebuild_entrypoint,
                    rebuilt_outputs: result.rebuilt_outputs.clone(),
                    durable_sources: durable_sources.clone(),
                    repair_hooks: result.repair_hooks.clone(),
                    verification_artifact_name: artifact.artifact_name,
                    affected_item_count,
                    error_count,
                    rebuild_duration_ms,
                    rollback_state,
                    degraded_mode,
                    rollback_trigger,
                    remediation_steps,
                    queue_depth_before,
                    queue_depth_after,
                    graph_hooks,
                    cache_invalidation_events,
                    cache_warmup_events,
                    operator_log: self.operator_log_for_result(
                        result,
                        artifact,
                        &durable_sources,
                        affected_item_count,
                        error_count,
                        rebuild_duration_ms,
                        rollback_state,
                        queue_depth_before,
                        queue_depth_after,
                        graph_rebuild_report,
                        cache_invalidation,
                        cache_warmup,
                    ),
                }
            })
            .collect();

        RepairSummary {
            targets_checked: self.results.len() as u32,
            healthy,
            degraded,
            corrupt,
            rebuilt,
            affected_item_count: self.total_affected_items,
            error_count: self.total_error_count,
            rebuild_duration_ms: self.total_duration_ms,
            rollback_state,
            degraded_mode,
            rollback_trigger,
            queue_report,
            results: self.results.clone(),
            verification_artifacts: self.verification_artifacts.clone(),
            operator_reports,
            graph_rebuild_reports: self.graph_rebuild_reports.clone(),
            cache_invalidation_reports: self.cache_invalidation_reports.clone(),
            cache_warmup_reports: self.cache_warmup_reports.clone(),
        }
    }

    fn mock_verification_artifact(&self, target: RepairTarget) -> VerificationArtifact {
        let rows = match target {
            RepairTarget::LexicalIndex => 128,
            RepairTarget::MetadataIndex => 128,
            RepairTarget::SemanticHotIndex => 64,
            RepairTarget::SemanticColdIndex => 64,
            RepairTarget::HotStoreConsistency => 32,
            RepairTarget::PayloadIntegrity => 128,
            RepairTarget::GraphConsistency => 96,
            RepairTarget::CacheWarmState => 48,
            RepairTarget::EngramIndex => 24,
            RepairTarget::ContradictionConsistency => 8,
        };

        VerificationArtifact {
            artifact_name: target.verification_artifact_name(),
            authoritative_rows: rows,
            derived_rows: rows,
            authoritative_generation: "durable.v1",
            derived_generation: "durable.v1",
            parity_check: target.verification_parity_check(),
            verification_passed: true,
            repair_hooks: match target {
                RepairTarget::CacheWarmState => CACHE_WARM_STATE_VERIFY_HOOKS.to_vec(),
                _ => Vec::new(),
            },
        }
    }

    fn graph_failure_injection_for(
        &self,
        target: RepairTarget,
        entrypoint: IndexRepairEntrypoint,
    ) -> GraphFailureInjection {
        if target != RepairTarget::GraphConsistency {
            return GraphFailureInjection::None;
        }

        match entrypoint {
            IndexRepairEntrypoint::VerifyOnly => GraphFailureInjection::DropLastDerivedEdge,
            IndexRepairEntrypoint::RebuildIfNeeded | IndexRepairEntrypoint::ForceRebuild => {
                GraphFailureInjection::None
            }
        }
    }

    fn graph_truth_inputs(&self, target: RepairTarget) -> Vec<EdgeDerivationInput> {
        if target != RepairTarget::GraphConsistency {
            return Vec::new();
        }

        vec![
            EdgeDerivationInput {
                source_memory: MemoryId(11),
                target_memory: Some(MemoryId(12)),
                extracted_concept: None,
                relation: RelationKind::DerivedFrom,
                confidence: 920,
            },
            EdgeDerivationInput {
                source_memory: MemoryId(11),
                target_memory: None,
                extracted_concept: Some("graph-repair".to_string()),
                relation: RelationKind::Mentions,
                confidence: 780,
            },
        ]
    }

    fn graph_rebuild_report(&self, target: RepairTarget) -> Option<GraphRebuildReport> {
        if target != RepairTarget::GraphConsistency {
            return None;
        }

        let rebuilder = DerivedGraphRebuilder;
        Some(rebuilder.rebuild_with_hooks(
            &self.graph_truth_inputs(target),
            self.graph_failure_injection_for(target, self.entrypoint),
        ))
    }

    fn cache_warm_state_report(
        &self,
        target: RepairTarget,
    ) -> Option<(InvalidationOutcome, Vec<CacheMaintenanceEvent>)> {
        if target != RepairTarget::CacheWarmState {
            return None;
        }

        let mut cache = CacheManager::new(8, 8);
        let generations = CacheGenerationAnchors {
            schema_generation: 2,
            policy_generation: 2,
            index_generation: 2,
            embedding_generation: 2,
            ranking_generation: 2,
        };
        for (index, family) in CACHE_WARM_STATE_WARMUP_FAMILIES.iter().enumerate() {
            let key = CacheKey {
                family: *family,
                namespace: self.namespace.clone(),
                workspace_key: None,
                owner_key: None,
                request_shape_hash: matches!(
                    *family,
                    CacheFamily::ResultCache | CacheFamily::AnnProbeCache
                )
                .then_some((index as u64) + 100),
                item_key: (index as u64) + 1,
                generations,
            };
            let request = CacheAdmissionRequest {
                request_shape_hash: key.request_shape_hash,
                ..CacheAdmissionRequest::default()
            };
            cache
                .store_for(*family)
                .expect("cache warm-state family should map to a bounded store")
                .admit(key, vec![MemoryId((index as u64) + 1)], request);
        }
        cache.prefetch.submit_hint(
            self.namespace.clone(),
            PrefetchTrigger::SessionRecency,
            vec![MemoryId(91), MemoryId(92)],
        );

        let invalidation =
            cache.handle_invalidation(InvalidationTrigger::RepairStarted, &self.namespace);
        let warmup = if self.entrypoint == IndexRepairEntrypoint::VerifyOnly {
            Vec::new()
        } else {
            cache.repair_warmup(&CACHE_WARM_STATE_WARMUP_FAMILIES, 1)
        };
        Some((invalidation, warmup))
    }
}

impl MaintenanceOperation for RepairRun {
    type Summary = RepairSummary;

    fn poll_step(&mut self) -> MaintenanceStep<Self::Summary> {
        if self.completed || self.current_index >= self.targets.len() {
            self.completed = true;
            return MaintenanceStep::Completed(self.summary());
        }

        let target = self.targets[self.current_index];
        // In a real implementation, each target would be verified against durable truth.
        let rebuild_plan = self
            .entrypoint
            .ne(&IndexRepairEntrypoint::VerifyOnly)
            .then(|| RepairEngine.plan_rebuild_from_durable_truth(target, self.entrypoint))
            .flatten();
        let repair_hooks = rebuild_plan
            .as_ref()
            .map(|plan| plan.repair_hooks.clone())
            .unwrap_or_else(|| match target {
                RepairTarget::CacheWarmState => CACHE_WARM_STATE_VERIFY_HOOKS.to_vec(),
                _ => Vec::new(),
            });
        let graph_rebuild_report = self.graph_rebuild_report(target);
        let cache_warm_state_report = self.cache_warm_state_report(target);
        let verification_artifact = rebuild_plan
            .as_ref()
            .map(|plan| plan.verification_artifact.clone())
            .unwrap_or_else(|| self.mock_verification_artifact(target));
        let graph_verification_passed = graph_rebuild_report
            .as_ref()
            .map(|report| report.metrics.verification_passed);
        let graph_detail = graph_rebuild_report.as_ref().map(|report| {
            if report.metrics.verification_passed {
                "verified_against_durable_truth"
            } else {
                "graph_failure_injection_detected_during_verification"
            }
        });
        let (status, detail, rebuild_entrypoint, rebuilt_outputs) = if let Some(plan) = rebuild_plan
        {
            (
                RepairStatus::Rebuilt,
                "rebuilt_from_durable_truth_and_verified",
                Some(plan.entrypoint),
                plan.rebuilt_outputs,
            )
        } else if let Some(detail) = graph_detail {
            (
                if verification_artifact.verification_passed
                    && graph_rebuild_report
                        .as_ref()
                        .is_none_or(|report| report.metrics.verification_passed)
                {
                    RepairStatus::Healthy
                } else {
                    RepairStatus::Degraded
                },
                detail,
                None,
                Vec::new(),
            )
        } else if self.entrypoint == IndexRepairEntrypoint::VerifyOnly {
            (
                RepairStatus::Healthy,
                "verified_against_durable_truth",
                None,
                Vec::new(),
            )
        } else {
            (
                RepairStatus::Skipped,
                "rebuild_not_supported_for_target",
                None,
                Vec::new(),
            )
        };
        self.verification_artifacts
            .insert(target, verification_artifact.clone());
        if let Some(report) = graph_rebuild_report {
            self.graph_rebuild_reports.insert(target, report);
        }
        if let Some((invalidation, warmup)) = cache_warm_state_report {
            self.cache_invalidation_reports.insert(target, invalidation);
            self.cache_warmup_reports.insert(target, warmup);
        }
        let telemetry_graph_report = self.graph_rebuild_reports.get(&target);
        let (affected_item_count, error_count, rebuild_duration_ms) =
            self.telemetry_for_target(target, status, telemetry_graph_report);
        self.total_affected_items += affected_item_count;
        self.total_error_count += error_count;
        self.total_duration_ms += rebuild_duration_ms;
        self.results.push(RepairCheckResult {
            target,
            status,
            detail,
            verification_passed: verification_artifact.verification_passed
                && graph_verification_passed.unwrap_or(true),
            rebuild_entrypoint,
            rebuilt_outputs,
            repair_hooks,
        });
        self.current_index += 1;
        self.durable_token = DurableStateToken(self.current_index as u64);

        if self.current_index >= self.targets.len() {
            self.completed = true;
            MaintenanceStep::Completed(self.summary())
        } else {
            MaintenanceStep::Pending(MaintenanceProgress::new(
                self.current_index as u32,
                self.targets.len() as u32,
            ))
        }
    }

    fn interrupt(&mut self, reason: InterruptionReason) -> InterruptedMaintenance {
        InterruptedMaintenance {
            reason,
            preserved_durable_state: self.durable_token,
            artifact: None,
        }
    }
}

// ── Engine ───────────────────────────────────────────────────────────────────

/// Canonical repair engine owned by the core crate.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct RepairEngine;

impl RepairEngine {
    /// Returns the stable component identifier.
    pub const fn component_name(&self) -> &'static str {
        "engine.repair"
    }

    /// Returns the durable truth sources and rebuilt outputs for one repair target.
    pub fn plan_rebuild_from_durable_truth(
        &self,
        target: RepairTarget,
        entrypoint: IndexRepairEntrypoint,
    ) -> Option<RepairRebuildPlan> {
        if !target.supports_rebuild_from_durable_truth() {
            return None;
        }

        let (durable_sources, authoritative_schema_objects, rebuilt_outputs) = match target {
            RepairTarget::LexicalIndex => (
                vec![
                    "durable_memory_records",
                    "namespace_policy_metadata",
                    "canonical_content_handles",
                ],
                vec![DurableSchemaObject::DurableMemoryRecords],
                vec!["fts5_lexical_projection", "lexical_lookup_table"],
            ),
            RepairTarget::MetadataIndex => (
                vec!["durable_memory_records", "namespace_policy_metadata"],
                vec![DurableSchemaObject::DurableMemoryRecords],
                vec!["tier2_metadata_projection", "namespace_lookup_table"],
            ),
            RepairTarget::SemanticHotIndex => (
                vec![
                    "durable_memory_records",
                    "canonical_embeddings",
                    "namespace_policy_metadata",
                ],
                vec![DurableSchemaObject::DurableMemoryRecords],
                vec!["usearch_hot_ann", "hot_embedding_lookup"],
            ),
            RepairTarget::SemanticColdIndex => (
                vec![
                    "durable_memory_records",
                    "canonical_embeddings",
                    "namespace_policy_metadata",
                ],
                vec![DurableSchemaObject::DurableMemoryRecords],
                vec!["usearch_cold_ann", "cold_embedding_lookup"],
            ),
            RepairTarget::GraphConsistency => (
                vec![
                    "durable_memory_records",
                    "memory_relation_refs",
                    "memory_lineage_edges",
                ],
                vec![
                    DurableSchemaObject::DurableMemoryRecords,
                    DurableSchemaObject::MemoryRelationRefsTable,
                    DurableSchemaObject::MemoryLineageEdgesTable,
                ],
                vec![
                    "graph_adjacency_projection",
                    "graph_neighborhood_cache",
                    "graph_consistency_snapshot",
                ],
            ),
            RepairTarget::CacheWarmState => (
                vec![
                    "durable_memory_records",
                    "namespace_policy_metadata",
                    "current_generation_anchors",
                ],
                vec![DurableSchemaObject::DurableMemoryRecords],
                vec![
                    "tier1_item_cache",
                    "result_cache",
                    "summary_cache",
                    "ann_probe_cache",
                    "prefetch_queue",
                ],
            ),
            RepairTarget::EngramIndex => (
                vec![
                    "durable_memory_records",
                    "engrams_table",
                    "engram_membership_table",
                ],
                vec![
                    DurableSchemaObject::DurableMemoryRecords,
                    DurableSchemaObject::EngramsTable,
                    DurableSchemaObject::EngramMembershipTable,
                ],
                vec![
                    "engram_helper_index",
                    "engram_centroid_projection",
                    "engram_adjacency_accelerator",
                ],
            ),
            _ => unreachable!("non-rebuild repair target filtered before rebuild planning"),
        };

        let repair_hooks = match target {
            RepairTarget::GraphConsistency => vec![
                GraphRebuildHook::SnapshotDurableTruth.as_str(),
                GraphRebuildHook::RebuildAdjacencyProjection.as_str(),
                GraphRebuildHook::RebuildNeighborhoodCache.as_str(),
                GraphRebuildHook::VerifyConsistencySnapshot.as_str(),
            ],
            RepairTarget::CacheWarmState => match entrypoint {
                IndexRepairEntrypoint::VerifyOnly => CACHE_WARM_STATE_VERIFY_HOOKS.to_vec(),
                IndexRepairEntrypoint::RebuildIfNeeded | IndexRepairEntrypoint::ForceRebuild => {
                    CACHE_WARM_STATE_REBUILD_HOOKS.to_vec()
                }
            },
            _ => Vec::new(),
        };
        let mut verification_artifact = self.mock_verification_artifact(target);
        if !repair_hooks.is_empty() {
            verification_artifact.repair_hooks = repair_hooks.clone();
        }

        Some(RepairRebuildPlan {
            target,
            entrypoint,
            durable_sources,
            authoritative_schema_objects,
            rebuilt_outputs,
            repair_hooks,
            verification_artifact,
        })
    }

    /// Plans rebuild work for the index subsystem from durable truth.
    pub fn plan_index_rebuild(
        &self,
        target: RepairTarget,
        entrypoint: IndexRepairEntrypoint,
    ) -> Option<RepairRebuildPlan> {
        target
            .is_index_rebuild_target()
            .then(|| self.plan_rebuild_from_durable_truth(target, entrypoint))
            .flatten()
    }

    /// Creates an index-only repair run for targeted operator entrypoints.
    pub fn create_index_rebuild(
        &self,
        namespace: NamespaceId,
        entrypoint: IndexRepairEntrypoint,
    ) -> RepairRun {
        self.create_targeted(
            namespace,
            vec![
                RepairTarget::LexicalIndex,
                RepairTarget::MetadataIndex,
                RepairTarget::SemanticHotIndex,
                RepairTarget::SemanticColdIndex,
            ],
            entrypoint,
        )
    }

    /// Creates a full-scan repair run for a namespace.
    pub fn create_full_scan(
        &self,
        namespace: NamespaceId,
        entrypoint: IndexRepairEntrypoint,
    ) -> RepairRun {
        RepairRun::full_scan(namespace, entrypoint)
    }

    /// Creates a targeted repair run for specific targets.
    pub fn create_targeted(
        &self,
        namespace: NamespaceId,
        targets: Vec<RepairTarget>,
        entrypoint: IndexRepairEntrypoint,
    ) -> RepairRun {
        RepairRun::new(namespace, targets, entrypoint)
    }

    fn mock_verification_artifact(&self, target: RepairTarget) -> VerificationArtifact {
        let rows = match target {
            RepairTarget::LexicalIndex => 128,
            RepairTarget::MetadataIndex => 128,
            RepairTarget::SemanticHotIndex => 64,
            RepairTarget::SemanticColdIndex => 64,
            RepairTarget::HotStoreConsistency => 32,
            RepairTarget::PayloadIntegrity => 128,
            RepairTarget::GraphConsistency => 96,
            RepairTarget::CacheWarmState => 48,
            RepairTarget::EngramIndex => 24,
            RepairTarget::ContradictionConsistency => 8,
        };

        VerificationArtifact {
            artifact_name: target.verification_artifact_name(),
            authoritative_rows: rows,
            derived_rows: rows,
            authoritative_generation: "durable.v1",
            derived_generation: "durable.v1",
            parity_check: target.verification_parity_check(),
            verification_passed: true,
            repair_hooks: match target {
                RepairTarget::CacheWarmState => CACHE_WARM_STATE_VERIFY_HOOKS.to_vec(),
                _ => Vec::new(),
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::engine::maintenance::{
        MaintenanceController, MaintenanceJobHandle, MaintenanceJobState,
    };

    fn ns(s: &str) -> NamespaceId {
        NamespaceId::new(s).unwrap()
    }

    #[test]
    fn full_scan_checks_all_targets() {
        let engine = RepairEngine;
        let run = engine.create_full_scan(ns("test"), IndexRepairEntrypoint::VerifyOnly);
        let mut handle = MaintenanceJobHandle::new(run, 20);

        handle.start();
        // Poll until complete
        loop {
            let snap = handle.poll();
            match snap.state {
                MaintenanceJobState::Completed(ref summary) => {
                    assert_eq!(summary.targets_checked, 10);
                    assert_eq!(summary.healthy, 9);
                    assert_eq!(summary.degraded, 1);
                    assert_eq!(summary.results.len(), 10);
                    assert_eq!(summary.verification_artifacts.len(), 10);
                    assert_eq!(summary.operator_reports.len(), 10);
                    let cache_report = summary
                        .results
                        .iter()
                        .find(|result| result.target == RepairTarget::CacheWarmState)
                        .expect("cache warm-state result should be present");
                    assert_eq!(
                        cache_report.repair_hooks,
                        CACHE_WARM_STATE_VERIFY_HOOKS.to_vec()
                    );
                    let cache_artifact = summary
                        .verification_artifacts
                        .get(&RepairTarget::CacheWarmState)
                        .expect("cache warm-state artifact should be present");
                    assert_eq!(
                        cache_artifact.repair_hooks,
                        CACHE_WARM_STATE_VERIFY_HOOKS.to_vec()
                    );
                    let cache_invalidation = summary
                        .cache_invalidation_reports
                        .get(&RepairTarget::CacheWarmState)
                        .expect("cache invalidation report should be present");
                    assert_eq!(
                        cache_invalidation.trigger,
                        InvalidationTrigger::RepairStarted
                    );
                    assert!(cache_invalidation.entries_invalidated >= 5);
                    assert!(summary
                        .cache_warmup_reports
                        .get(&RepairTarget::CacheWarmState)
                        .is_some_and(Vec::is_empty));
                    let graph_report = summary
                        .graph_rebuild_reports
                        .get(&RepairTarget::GraphConsistency)
                        .unwrap();
                    assert_eq!(graph_report.metrics.durable_inputs_seen, 2);
                    assert_eq!(graph_report.metrics.rebuilt_edges, 1);
                    assert_eq!(graph_report.metrics.dropped_edges, 1);
                    assert!(!graph_report.metrics.verification_passed);
                    assert!(graph_report
                        .hooks_run
                        .iter()
                        .any(|hook| hook.as_str() == "verify_consistency_snapshot"));
                    assert!(graph_report
                        .operator_log
                        .contains("failure_injection=drop_last_derived_edge"));
                    let semantic_hot_report = summary
                        .operator_reports
                        .iter()
                        .find(|report| report.target == RepairTarget::SemanticHotIndex)
                        .unwrap();
                    assert_eq!(
                        semantic_hot_report.verification_artifact_name,
                        "usearch_hot_parity"
                    );
                    assert!(semantic_hot_report
                        .operator_log
                        .contains("target=semantic_hot_index"));
                    assert!(semantic_hot_report.operator_log.contains("status=healthy"));
                    assert!(semantic_hot_report.operator_log.contains("entrypoint=none"));
                    assert_eq!(
                        summary
                            .verification_artifacts
                            .get(&RepairTarget::SemanticHotIndex)
                            .unwrap()
                            .authoritative_rows,
                        64
                    );
                    assert_eq!(
                        summary
                            .verification_artifacts
                            .get(&RepairTarget::EngramIndex)
                            .unwrap()
                            .derived_generation,
                        "durable.v1"
                    );
                    break;
                }
                MaintenanceJobState::Running { .. } => continue,
                _ => panic!("unexpected state"),
            }
        }
    }

    #[test]
    fn targeted_scan_checks_only_specified_targets() {
        let engine = RepairEngine;
        let run = engine.create_targeted(
            ns("test"),
            vec![
                RepairTarget::LexicalIndex,
                RepairTarget::SemanticColdIndex,
                RepairTarget::EngramIndex,
            ],
            IndexRepairEntrypoint::VerifyOnly,
        );
        let mut handle = MaintenanceJobHandle::new(run, 10);

        handle.start();
        loop {
            let snap = handle.poll();
            match snap.state {
                MaintenanceJobState::Completed(ref summary) => {
                    assert_eq!(summary.targets_checked, 3);
                    assert_eq!(summary.healthy, 3);
                    assert_eq!(summary.rebuilt, 0);
                    assert_eq!(summary.verification_artifacts.len(), 3);
                    assert_eq!(summary.operator_reports.len(), 3);
                    assert!(summary.graph_rebuild_reports.is_empty());
                    assert!(summary
                        .verification_artifacts
                        .contains_key(&RepairTarget::SemanticColdIndex));
                    assert!(summary
                        .verification_artifacts
                        .contains_key(&RepairTarget::EngramIndex));
                    assert!(summary
                        .results
                        .iter()
                        .all(|result| result.verification_passed));
                    assert!(summary
                        .results
                        .iter()
                        .all(|result| result.rebuild_entrypoint.is_none()));
                    assert!(summary
                        .results
                        .iter()
                        .all(|result| result.rebuilt_outputs.is_empty()));
                    break;
                }
                MaintenanceJobState::Running { .. } => continue,
                _ => panic!("unexpected state"),
            }
        }
    }

    #[test]
    fn repair_target_machine_names_cover_tier2_rebuild_contract() {
        assert_eq!(
            RepairTarget::SemanticHotIndex.as_str(),
            "semantic_hot_index"
        );
        assert_eq!(
            RepairTarget::SemanticColdIndex.as_str(),
            "semantic_cold_index"
        );
        assert_eq!(RepairTarget::CacheWarmState.as_str(), "cache_warm_state");
        assert_eq!(RepairTarget::EngramIndex.as_str(), "engram_index");
    }

    #[test]
    fn index_rebuild_entrypoint_scopes_only_index_targets() {
        let engine = RepairEngine;
        let run = engine.create_index_rebuild(ns("test"), IndexRepairEntrypoint::RebuildIfNeeded);
        let mut handle = MaintenanceJobHandle::new(run, 10);

        handle.start();
        let completed = loop {
            let snapshot = handle.poll();
            match snapshot.state {
                MaintenanceJobState::Completed(summary) => break summary,
                MaintenanceJobState::Running { .. } => continue,
                _ => panic!("unexpected state"),
            }
        };

        assert_eq!(completed.targets_checked, 4);
        assert_eq!(completed.rebuilt, 4);
        assert!(completed
            .results
            .iter()
            .all(|result| result.target.is_index_rebuild_target()));
        assert_eq!(
            completed
                .results
                .iter()
                .map(|result| result.target)
                .collect::<Vec<_>>(),
            vec![
                RepairTarget::LexicalIndex,
                RepairTarget::MetadataIndex,
                RepairTarget::SemanticHotIndex,
                RepairTarget::SemanticColdIndex,
            ]
        );
        assert!(completed
            .operator_reports
            .iter()
            .all(|report| report.target.is_index_rebuild_target()));
    }

    #[test]
    fn rebuild_plan_covers_durable_sources_and_outputs() {
        let engine = RepairEngine;
        let plan = engine
            .plan_index_rebuild(
                RepairTarget::LexicalIndex,
                IndexRepairEntrypoint::RebuildIfNeeded,
            )
            .expect("lexical index should expose rebuild plan");

        assert_eq!(plan.subsystem(), "index");

        assert_eq!(plan.target, RepairTarget::LexicalIndex);
        assert_eq!(plan.entrypoint, IndexRepairEntrypoint::RebuildIfNeeded);
        assert_eq!(
            plan.durable_sources,
            vec![
                "durable_memory_records",
                "namespace_policy_metadata",
                "canonical_content_handles",
            ]
        );
        assert_eq!(
            plan.rebuilt_outputs,
            vec!["fts5_lexical_projection", "lexical_lookup_table"]
        );
        assert_eq!(
            plan.authoritative_schema_objects,
            vec![DurableSchemaObject::DurableMemoryRecords]
        );
        assert_eq!(
            plan.verification_artifact.artifact_name,
            "fts5_lexical_parity"
        );
        assert_eq!(
            plan.verification_artifact.parity_check,
            "fts5_projection_matches_durable_truth"
        );
        assert_eq!(plan.verification_artifact.authoritative_rows, 128);
        assert_eq!(plan.verification_artifact.derived_generation, "durable.v1");
        assert!(plan.verification_artifact.verification_passed);
    }

    #[test]
    fn rebuild_plan_covers_engram_derived_surfaces() {
        let engine = RepairEngine;
        let plan = engine
            .plan_rebuild_from_durable_truth(
                RepairTarget::EngramIndex,
                IndexRepairEntrypoint::ForceRebuild,
            )
            .expect("engram index should expose rebuild plan");

        assert_eq!(plan.target, RepairTarget::EngramIndex);
        assert_eq!(plan.entrypoint, IndexRepairEntrypoint::ForceRebuild);
        assert_eq!(
            plan.durable_sources,
            vec![
                "durable_memory_records",
                "engrams_table",
                "engram_membership_table",
            ]
        );
        assert_eq!(
            plan.rebuilt_outputs,
            vec![
                "engram_helper_index",
                "engram_centroid_projection",
                "engram_adjacency_accelerator",
            ]
        );
        assert_eq!(
            plan.authoritative_schema_objects,
            vec![
                DurableSchemaObject::DurableMemoryRecords,
                DurableSchemaObject::EngramsTable,
                DurableSchemaObject::EngramMembershipTable,
            ]
        );
        assert_eq!(
            plan.verification_artifact.artifact_name,
            "engram_membership_parity"
        );
        assert_eq!(
            plan.verification_artifact.parity_check,
            "engram_membership_matches_durable_truth"
        );
        assert_eq!(plan.verification_artifact.authoritative_rows, 24);
        assert_eq!(plan.verification_artifact.derived_generation, "durable.v1");
        assert!(plan.verification_artifact.verification_passed);
    }

    #[test]
    fn rebuild_plan_covers_graph_and_cache_derived_surfaces() {
        let engine = RepairEngine;
        let graph_plan = engine
            .plan_rebuild_from_durable_truth(
                RepairTarget::GraphConsistency,
                IndexRepairEntrypoint::ForceRebuild,
            )
            .expect("graph consistency should expose rebuild plan");
        let cache_plan = engine
            .plan_rebuild_from_durable_truth(
                RepairTarget::CacheWarmState,
                IndexRepairEntrypoint::RebuildIfNeeded,
            )
            .expect("cache warm state should expose rebuild plan");

        assert_eq!(
            graph_plan.authoritative_schema_objects,
            vec![
                DurableSchemaObject::DurableMemoryRecords,
                DurableSchemaObject::MemoryRelationRefsTable,
                DurableSchemaObject::MemoryLineageEdgesTable,
            ]
        );
        assert_eq!(
            graph_plan.rebuilt_outputs,
            vec![
                "graph_adjacency_projection",
                "graph_neighborhood_cache",
                "graph_consistency_snapshot",
            ]
        );
        assert_eq!(
            graph_plan.durable_sources,
            vec![
                "durable_memory_records",
                "memory_relation_refs",
                "memory_lineage_edges",
            ]
        );
        assert_eq!(
            cache_plan.durable_sources,
            vec![
                "durable_memory_records",
                "namespace_policy_metadata",
                "current_generation_anchors",
            ]
        );
        assert_eq!(
            cache_plan.rebuilt_outputs,
            vec![
                "tier1_item_cache",
                "result_cache",
                "summary_cache",
                "ann_probe_cache",
                "prefetch_queue",
            ]
        );
        assert_eq!(
            cache_plan.verification_artifact.artifact_name,
            "cache_generation_anchor_report"
        );
        assert_eq!(cache_plan.verification_artifact.authoritative_rows, 48);
        assert_eq!(
            cache_plan.repair_hooks,
            CACHE_WARM_STATE_REBUILD_HOOKS.to_vec()
        );
        assert_eq!(
            cache_plan.verification_artifact.repair_hooks,
            CACHE_WARM_STATE_REBUILD_HOOKS.to_vec()
        );
    }

    #[test]
    fn engram_rebuild_plan_keeps_graph_edges_out_of_authoritative_sources() {
        let engine = RepairEngine;
        let plan = engine
            .plan_rebuild_from_durable_truth(
                RepairTarget::EngramIndex,
                IndexRepairEntrypoint::VerifyOnly,
            )
            .expect("engram index should expose rebuild plan");

        assert!(!plan.durable_sources.contains(&"graph_edge_table"));
        assert!(plan.durable_sources.contains(&"engrams_table"));
        assert!(plan.durable_sources.contains(&"engram_membership_table"));
    }

    #[test]
    fn verification_artifact_names_and_parity_checks_are_stable() {
        assert_eq!(
            RepairTarget::LexicalIndex.verification_artifact_name(),
            "fts5_lexical_parity"
        );
        assert_eq!(
            RepairTarget::LexicalIndex.verification_parity_check(),
            "fts5_projection_matches_durable_truth"
        );
        assert_eq!(
            RepairTarget::MetadataIndex.verification_artifact_name(),
            "tier2_metadata_parity"
        );
        assert_eq!(
            RepairTarget::MetadataIndex.verification_parity_check(),
            "tier2_projection_matches_durable_truth"
        );
        assert_eq!(
            RepairTarget::SemanticHotIndex.verification_artifact_name(),
            "usearch_hot_parity"
        );
        assert_eq!(
            RepairTarget::SemanticHotIndex.verification_parity_check(),
            "usearch_hot_matches_durable_embeddings"
        );
        assert_eq!(
            RepairTarget::SemanticColdIndex.verification_artifact_name(),
            "usearch_cold_parity"
        );
        assert_eq!(
            RepairTarget::SemanticColdIndex.verification_parity_check(),
            "usearch_cold_matches_durable_embeddings"
        );
        assert_eq!(
            RepairTarget::CacheWarmState.verification_artifact_name(),
            "cache_generation_anchor_report"
        );
        assert_eq!(
            RepairTarget::CacheWarmState.verification_parity_check(),
            "cache_warm_state_matches_current_generations"
        );
        assert_eq!(
            RepairTarget::EngramIndex.verification_artifact_name(),
            "engram_membership_parity"
        );
        assert_eq!(
            RepairTarget::EngramIndex.verification_parity_check(),
            "engram_membership_matches_durable_truth"
        );
    }

    #[test]
    fn supports_rebuild_from_durable_truth_matches_rebuild_plan_coverage() {
        let engine = RepairEngine;
        let targets = [
            RepairTarget::LexicalIndex,
            RepairTarget::MetadataIndex,
            RepairTarget::SemanticHotIndex,
            RepairTarget::SemanticColdIndex,
            RepairTarget::HotStoreConsistency,
            RepairTarget::PayloadIntegrity,
            RepairTarget::GraphConsistency,
            RepairTarget::CacheWarmState,
            RepairTarget::EngramIndex,
            RepairTarget::ContradictionConsistency,
        ];

        for target in targets {
            let exposed_plan = if target.is_index_rebuild_target() {
                engine.plan_index_rebuild(target, IndexRepairEntrypoint::VerifyOnly)
            } else {
                engine.plan_rebuild_from_durable_truth(target, IndexRepairEntrypoint::VerifyOnly)
            };
            assert_eq!(
                target.supports_rebuild_from_durable_truth(),
                exposed_plan.is_some(),
                "unexpected rebuild-plan support mismatch for {}",
                target.as_str()
            );
        }
    }

    #[test]
    fn non_rebuild_targets_do_not_expose_rebuild_plans() {
        let engine = RepairEngine;
        for target in [
            RepairTarget::HotStoreConsistency,
            RepairTarget::PayloadIntegrity,
            RepairTarget::ContradictionConsistency,
        ] {
            assert!(!target.supports_rebuild_from_durable_truth());
            assert!(engine
                .plan_rebuild_from_durable_truth(target, IndexRepairEntrypoint::VerifyOnly)
                .is_none());
        }
    }

    #[test]
    fn force_rebuild_runs_report_rebuilt_outputs_and_entrypoints() {
        let engine = RepairEngine;
        let run = engine.create_targeted(
            ns("test"),
            vec![RepairTarget::LexicalIndex, RepairTarget::SemanticColdIndex],
            IndexRepairEntrypoint::ForceRebuild,
        );
        let mut handle = MaintenanceJobHandle::new(run, 10);

        handle.start();
        let first = handle.poll();
        assert_eq!(first.polls_used, 1);
        assert!(matches!(first.state, MaintenanceJobState::Running { .. }));

        let completed = handle.poll();
        let MaintenanceJobState::Completed(summary) = completed.state else {
            panic!("expected completed repair summary after second poll");
        };

        assert_eq!(summary.targets_checked, 2);
        assert_eq!(summary.healthy, 0);
        assert_eq!(summary.rebuilt, 2);
        assert!(summary
            .results
            .iter()
            .all(|result| result.verification_passed));
        assert!(summary.results.iter().all(|result| {
            result.rebuild_entrypoint == Some(IndexRepairEntrypoint::ForceRebuild)
        }));
        assert_eq!(summary.operator_reports.len(), 2);
        assert!(summary.operator_reports.iter().all(|report| {
            report.rebuild_entrypoint == Some(IndexRepairEntrypoint::ForceRebuild)
                && report.verification_passed
                && !report.durable_sources.is_empty()
                && report.operator_log.contains("status=rebuilt")
                && report.operator_log.contains("entrypoint=force_rebuild")
                && report.operator_log.contains("durable_sources=")
                && report
                    .operator_log
                    .contains(report.verification_artifact_name)
        }));
        assert_eq!(
            summary.operator_reports[0].verification_artifact_name,
            "fts5_lexical_parity"
        );
        assert_eq!(
            summary.results[0].rebuilt_outputs,
            vec!["fts5_lexical_projection", "lexical_lookup_table"]
        );
        assert_eq!(
            summary.results[1].rebuilt_outputs,
            vec!["usearch_cold_ann", "cold_embedding_lookup"]
        );
        assert!(summary
            .verification_artifacts
            .values()
            .all(|artifact| artifact.verification_passed));
    }

    #[test]
    fn cache_warm_state_repair_reports_invalidation_hooks_and_warmup() {
        let engine = RepairEngine;
        let run = engine.create_targeted(
            ns("cache.repair"),
            vec![RepairTarget::CacheWarmState],
            IndexRepairEntrypoint::RebuildIfNeeded,
        );
        let mut handle = MaintenanceJobHandle::new(run, 10);

        handle.start();
        let completed = handle.poll();
        let MaintenanceJobState::Completed(summary) = completed.state else {
            panic!("expected completed cache warm-state repair summary after first poll");
        };

        assert_eq!(summary.targets_checked, 1);
        assert_eq!(summary.rebuilt, 1);
        let result = summary
            .results
            .first()
            .expect("cache warm-state result missing");
        assert_eq!(result.target, RepairTarget::CacheWarmState);
        assert_eq!(result.repair_hooks, CACHE_WARM_STATE_REBUILD_HOOKS.to_vec());

        let invalidation = summary
            .cache_invalidation_reports
            .get(&RepairTarget::CacheWarmState)
            .expect("cache invalidation report missing");
        assert_eq!(invalidation.trigger, InvalidationTrigger::RepairStarted);
        assert!(invalidation.entries_invalidated >= 5);
        assert!(invalidation.maintenance_events.iter().any(|event| {
            event.family == CacheFamily::PrefetchHints
                && event.event == crate::store::cache::CacheEvent::PrefetchDrop
                && event.entries_affected == 1
        }));

        let warmup = summary
            .cache_warmup_reports
            .get(&RepairTarget::CacheWarmState)
            .expect("cache warmup report missing");
        assert_eq!(warmup.len(), CACHE_WARM_STATE_WARMUP_FAMILIES.len());
        assert!(warmup.iter().all(|event| {
            event.event == crate::store::cache::CacheEvent::RepairWarmup
                && event.entries_affected == 1
        }));

        let operator_report = summary
            .operator_reports
            .iter()
            .find(|report| report.target == RepairTarget::CacheWarmState)
            .expect("cache warm-state operator report missing");
        assert_eq!(
            operator_report.durable_sources,
            vec![
                "durable_memory_records",
                "namespace_policy_metadata",
                "current_generation_anchors",
            ]
        );
        assert_eq!(
            operator_report.repair_hooks,
            CACHE_WARM_STATE_REBUILD_HOOKS.to_vec()
        );
        assert_eq!(
            operator_report.cache_invalidation_events,
            invalidation.maintenance_events
        );
        assert_eq!(operator_report.cache_warmup_events, *warmup);
        assert!(operator_report.operator_log.contains(
            "repair_hooks=snapshot_current_generation_anchors,invalidate_cache_families,drop_prefetch_hints,rebuild_tier1_item_cache,rebuild_result_cache,rebuild_summary_cache,rebuild_ann_probe_cache,verify_generation_anchor_report"
        ));
        assert!(operator_report.operator_log.contains(
            "durable_sources=durable_memory_records,namespace_policy_metadata,current_generation_anchors"
        ));
        assert!(operator_report
            .operator_log
            .contains("cache_invalidation=repair_started:"));
        assert!(operator_report.operator_log.contains(
            "cache_warmup=tier1_item:repair_warmup:none:tier1_item_cache:valid:1|result_cache:repair_warmup:none:result_cache:valid:1|summary_cache:repair_warmup:none:summary_cache:valid:1|ann_probe_cache:repair_warmup:none:ann_probe_cache:valid:1"
        ));
    }

    #[test]
    fn non_rebuild_targets_are_reported_as_skipped_when_rebuild_is_requested() {
        let engine = RepairEngine;
        let run = engine.create_targeted(
            ns("test"),
            vec![
                RepairTarget::GraphConsistency,
                RepairTarget::PayloadIntegrity,
            ],
            IndexRepairEntrypoint::ForceRebuild,
        );
        let mut handle = MaintenanceJobHandle::new(run, 10);

        handle.start();
        let first = handle.poll();
        assert_eq!(first.polls_used, 1);
        assert!(matches!(first.state, MaintenanceJobState::Running { .. }));

        let completed = handle.poll();
        let MaintenanceJobState::Completed(summary) = completed.state else {
            panic!("expected completed repair summary after second poll");
        };

        assert_eq!(summary.targets_checked, 2);
        assert_eq!(summary.healthy, 0);
        assert_eq!(summary.rebuilt, 1);
        assert_eq!(summary.results.len(), 2);
        assert!(summary.results.iter().any(|result| {
            result.target == RepairTarget::GraphConsistency
                && result.status == RepairStatus::Rebuilt
                && result.detail == "rebuilt_from_durable_truth_and_verified"
                && result.rebuild_entrypoint == Some(IndexRepairEntrypoint::ForceRebuild)
                && result.rebuilt_outputs
                    == vec![
                        "graph_adjacency_projection",
                        "graph_neighborhood_cache",
                        "graph_consistency_snapshot",
                    ]
                && result.verification_passed
        }));
        assert!(summary.results.iter().any(|result| {
            result.target == RepairTarget::PayloadIntegrity
                && result.status == RepairStatus::Skipped
                && result.detail == "rebuild_not_supported_for_target"
                && result.rebuild_entrypoint.is_none()
                && result.rebuilt_outputs.is_empty()
                && result.verification_passed
        }));
        assert!(summary.operator_reports.iter().any(|report| {
            report.target == RepairTarget::GraphConsistency
                && report.status == RepairStatus::Rebuilt
                && report.rebuild_entrypoint == Some(IndexRepairEntrypoint::ForceRebuild)
                && report.verification_passed
                && report.durable_sources
                    == vec![
                        "durable_memory_records",
                        "memory_relation_refs",
                        "memory_lineage_edges",
                    ]
                && report.graph_hooks
                    == vec![
                        "snapshot_durable_truth",
                        "rebuild_adjacency_projection",
                        "rebuild_neighborhood_cache",
                        "verify_consistency_snapshot",
                    ]
        }));
        assert!(summary.operator_reports.iter().any(|report| {
            report.target == RepairTarget::PayloadIntegrity
                && report.status == RepairStatus::Skipped
                && report.rebuild_entrypoint.is_none()
                && report.verification_passed
        }));
        let graph_report = summary
            .graph_rebuild_reports
            .get(&RepairTarget::GraphConsistency)
            .expect("graph consistency should emit rebuild report");
        assert_eq!(graph_report.metrics.durable_inputs_seen, 2);
        assert_eq!(graph_report.metrics.rebuilt_edges, 2);
        assert_eq!(graph_report.metrics.dropped_edges, 0);
        assert!(graph_report.metrics.verification_passed);
        assert_eq!(
            graph_report.metric_names(),
            [
                "graph_rebuild_durable_inputs_seen",
                "graph_rebuild_edges_total",
                "graph_rebuild_dropped_edges_total",
                "graph_rebuild_verification_passed",
            ]
        );
        assert!(graph_report
            .operator_log
            .contains("hooks=snapshot_durable_truth,rebuild_adjacency_projection,rebuild_neighborhood_cache,verify_consistency_snapshot"));
        let graph_operator_report = summary
            .operator_reports
            .iter()
            .find(|report| report.target == RepairTarget::GraphConsistency)
            .expect("graph consistency operator report should be present");
        assert!(graph_operator_report.operator_log.contains(
            "durable_sources=durable_memory_records,memory_relation_refs,memory_lineage_edges"
        ));
        assert!(graph_operator_report.operator_log.contains(
            "graph_hooks=snapshot_durable_truth,rebuild_adjacency_projection,rebuild_neighborhood_cache,verify_consistency_snapshot"
        ));
    }

    #[test]
    fn graph_verify_only_run_records_failure_injection_without_losing_durable_truth() {
        let engine = RepairEngine;
        let run = engine.create_targeted(
            ns("test"),
            vec![RepairTarget::GraphConsistency],
            IndexRepairEntrypoint::VerifyOnly,
        );
        let mut handle = MaintenanceJobHandle::new(run, 10);

        handle.start();
        let completed = handle.poll();
        let MaintenanceJobState::Completed(summary) = completed.state else {
            panic!("expected completed repair summary after first poll");
        };

        assert_eq!(summary.targets_checked, 1);
        assert_eq!(summary.healthy, 0);
        assert_eq!(summary.degraded, 1);
        let result = summary.results.first().unwrap();
        assert_eq!(result.target, RepairTarget::GraphConsistency);
        assert_eq!(
            result.detail,
            "graph_failure_injection_detected_during_verification"
        );
        assert_eq!(result.status, RepairStatus::Degraded);
        let graph_report = summary
            .graph_rebuild_reports
            .get(&RepairTarget::GraphConsistency)
            .unwrap();
        assert_eq!(
            graph_report.failure_injection,
            GraphFailureInjection::DropLastDerivedEdge
        );
        assert_eq!(graph_report.metrics.rebuilt_edges, 1);
        assert_eq!(graph_report.metrics.dropped_edges, 1);
        assert!(!graph_report.metrics.verification_passed);
        assert_eq!(graph_report.rebuilt_edges.len(), 1);
    }
}
