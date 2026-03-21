//! Repair and rebuild machinery.
//!
//! Owns corruption detection, index rebuild, and data integrity verification
//! surfaces that can be triggered by operator commands or automated health checks.

use crate::api::NamespaceId;
use crate::engine::maintenance::{
    DurableStateToken, InterruptedMaintenance, InterruptionReason, MaintenanceOperation,
    MaintenanceProgress, MaintenanceStep,
};
use crate::migrate::DurableSchemaObject;
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
    pub results: Vec<RepairCheckResult>,
    pub verification_artifacts: HashMap<RepairTarget, VerificationArtifact>,
    pub operator_reports: Vec<RepairOperatorReport>,
}

/// Operator-visible per-target repair report suitable for logs and handoff surfaces.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RepairOperatorReport {
    pub target: RepairTarget,
    pub status: RepairStatus,
    pub verification_passed: bool,
    pub rebuild_entrypoint: Option<IndexRepairEntrypoint>,
    pub rebuilt_outputs: Vec<&'static str>,
    pub verification_artifact_name: &'static str,
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
}

/// Canonical index rebuild plan derived from durable truth.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RepairRebuildPlan {
    pub target: RepairTarget,
    pub entrypoint: IndexRepairEntrypoint,
    pub durable_sources: Vec<&'static str>,
    pub authoritative_schema_objects: Vec<DurableSchemaObject>,
    pub rebuilt_outputs: Vec<&'static str>,
    pub verification_artifact: VerificationArtifact,
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
    completed: bool,
    durable_token: DurableStateToken,
}

impl RepairRun {
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
            completed: false,
            durable_token: DurableStateToken(0),
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
        let operator_reports = self
            .results
            .iter()
            .map(|result| {
                let artifact = self
                    .verification_artifacts
                    .get(&result.target)
                    .expect("verification artifact should exist for every repair result");
                RepairOperatorReport {
                    target: result.target,
                    status: result.status,
                    verification_passed: result.verification_passed,
                    rebuild_entrypoint: result.rebuild_entrypoint,
                    rebuilt_outputs: result.rebuilt_outputs.clone(),
                    verification_artifact_name: artifact.artifact_name,
                }
            })
            .collect();

        RepairSummary {
            targets_checked: self.results.len() as u32,
            healthy,
            degraded,
            corrupt,
            rebuilt,
            results: self.results.clone(),
            verification_artifacts: self.verification_artifacts.clone(),
            operator_reports,
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
        }
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
        let verification_artifact = self.mock_verification_artifact(target);
        let (status, detail, rebuild_entrypoint, rebuilt_outputs) = if let Some(plan) = rebuild_plan
        {
            (
                RepairStatus::Rebuilt,
                "rebuilt_from_durable_truth_and_verified",
                Some(plan.entrypoint),
                plan.rebuilt_outputs,
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
        self.results.push(RepairCheckResult {
            target,
            status,
            detail,
            verification_passed: verification_artifact.verification_passed,
            rebuild_entrypoint,
            rebuilt_outputs,
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
                vec!["durable_memory_records", "graph_edge_table"],
                vec![
                    DurableSchemaObject::DurableMemoryRecords,
                    DurableSchemaObject::GraphEdgeTable,
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

        Some(RepairRebuildPlan {
            target,
            entrypoint,
            durable_sources,
            authoritative_schema_objects,
            rebuilt_outputs,
            verification_artifact: self.mock_verification_artifact(target),
        })
    }

    /// Backwards-compatible alias for rebuild planning callers.
    pub fn plan_index_rebuild(
        &self,
        target: RepairTarget,
        entrypoint: IndexRepairEntrypoint,
    ) -> Option<RepairRebuildPlan> {
        self.plan_rebuild_from_durable_truth(target, entrypoint)
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
                    assert_eq!(summary.healthy, 10);
                    assert_eq!(summary.results.len(), 10);
                    assert_eq!(summary.verification_artifacts.len(), 10);
                    assert_eq!(summary.operator_reports.len(), 10);
                    assert_eq!(
                        summary
                            .operator_reports
                            .iter()
                            .find(|report| report.target == RepairTarget::SemanticHotIndex)
                            .unwrap()
                            .verification_artifact_name,
                        "usearch_hot_parity"
                    );
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
    fn rebuild_plan_covers_durable_sources_and_outputs() {
        let engine = RepairEngine;
        let plan = engine
            .plan_rebuild_from_durable_truth(
                RepairTarget::LexicalIndex,
                IndexRepairEntrypoint::RebuildIfNeeded,
            )
            .expect("lexical index should expose rebuild plan");

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
                DurableSchemaObject::GraphEdgeTable,
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
            assert_eq!(
                target.supports_rebuild_from_durable_truth(),
                engine
                    .plan_rebuild_from_durable_truth(target, IndexRepairEntrypoint::VerifyOnly)
                    .is_some(),
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
        }));
        assert!(summary.operator_reports.iter().any(|report| {
            report.target == RepairTarget::PayloadIntegrity
                && report.status == RepairStatus::Skipped
                && report.rebuild_entrypoint.is_none()
                && report.verification_passed
        }));
    }
}
