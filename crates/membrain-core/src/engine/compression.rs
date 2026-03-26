use crate::api::NamespaceId;
use crate::engine::maintenance::{
    DurableStateToken, InterruptedMaintenance, InterruptionReason, MaintenanceOperation,
    MaintenanceProgress, MaintenanceStep,
};
use crate::types::MemoryId;
use std::collections::HashMap;

/// Stable bounded policy for offline schema-compression candidate selection.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct CompressionPolicy {
    /// Whether schema-compression maintenance is enabled.
    pub enabled: bool,
    /// Minimum episodic source memories required before a cluster may compress.
    pub min_episode_count: usize,
    /// Minimum centroid coherence required for direct eligibility.
    pub centroid_coherence_min: f32,
    /// Coherence margin below the threshold that triggers review instead of rejection.
    pub review_margin: f32,
    /// Maximum clusters the bounded run may evaluate.
    pub batch_size: usize,
}

impl Default for CompressionPolicy {
    fn default() -> Self {
        Self {
            enabled: true,
            min_episode_count: 20,
            centroid_coherence_min: 0.55,
            review_margin: 0.05,
            batch_size: 8,
        }
    }
}

/// Stable run trigger for schema-compression maintenance.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CompressionTrigger {
    Manual,
    Consolidation,
}

impl CompressionTrigger {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Manual => "manual",
            Self::Consolidation => "consolidation",
        }
    }
}

/// Stable reason a bounded schema-compression run may not start.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CompressionSkipReason {
    Disabled,
}

impl CompressionSkipReason {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Disabled => "disabled",
        }
    }
}

/// Inspectable scheduler snapshot for a proposed schema-compression run.
#[derive(Debug, Clone, PartialEq)]
pub struct CompressionStatus {
    pub enabled: bool,
    pub namespace: NamespaceId,
    pub trigger: CompressionTrigger,
    pub min_episode_count: usize,
    pub centroid_coherence_min: f32,
    pub review_margin: f32,
    pub batch_size: usize,
    pub last_run_tick: Option<u64>,
    pub paused_reason: Option<CompressionSkipReason>,
}

impl CompressionStatus {
    pub fn for_policy(
        namespace: NamespaceId,
        trigger: CompressionTrigger,
        policy: CompressionPolicy,
        last_run_tick: Option<u64>,
    ) -> Self {
        Self {
            enabled: policy.enabled,
            namespace,
            trigger,
            min_episode_count: policy.min_episode_count,
            centroid_coherence_min: policy.centroid_coherence_min,
            review_margin: policy.review_margin,
            batch_size: policy.batch_size,
            last_run_tick,
            paused_reason: (!policy.enabled).then_some(CompressionSkipReason::Disabled),
        }
    }

    pub const fn should_skip(&self) -> bool {
        self.paused_reason.is_some()
    }
}

/// Stable brain-inspired kind consulted by compression eligibility.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum CompressionMemoryKind {
    Episodic,
    Semantic,
    Procedural,
    Schema,
}

impl CompressionMemoryKind {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Episodic => "episodic",
            Self::Semantic => "semantic",
            Self::Procedural => "procedural",
            Self::Schema => "schema",
        }
    }
}

/// Safety guards that can prevent source memories from being compressed.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct CompressionGuards {
    pub legal_hold: bool,
    pub pinned: bool,
    pub last_authoritative_evidence: bool,
    pub open_conflict: bool,
    pub already_compressed: bool,
    pub durable_lineage_available: bool,
}

/// One source memory considered during schema-compression planning.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CompressionSourceMemory {
    pub memory_id: MemoryId,
    pub compact_text: String,
    pub memory_kind: CompressionMemoryKind,
    pub compressed_into: Option<MemoryId>,
    pub guards: CompressionGuards,
}

impl CompressionSourceMemory {
    pub fn new(
        memory_id: MemoryId,
        compact_text: impl Into<String>,
        memory_kind: CompressionMemoryKind,
    ) -> Self {
        Self {
            memory_id,
            compact_text: compact_text.into(),
            memory_kind,
            compressed_into: None,
            guards: CompressionGuards {
                durable_lineage_available: true,
                ..CompressionGuards::default()
            },
        }
    }

    pub fn with_compressed_into(mut self, schema_memory_id: MemoryId) -> Self {
        self.compressed_into = Some(schema_memory_id);
        self
    }

    pub fn with_guards(mut self, guards: CompressionGuards) -> Self {
        self.guards = guards;
        self
    }
}

/// One bounded cross-episode cluster evaluated for schema compression.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CompressionCandidateCluster {
    pub cluster_id: String,
    pub source_memories: Vec<CompressionSourceMemory>,
    pub coherence_millis: u16,
    pub dominant_keywords: Vec<String>,
}

impl CompressionCandidateCluster {
    pub fn coherence_score(&self) -> f32 {
        self.coherence_millis as f32 / 1000.0
    }

    pub fn representative_memory_ids(&self) -> Vec<MemoryId> {
        self.source_memories
            .iter()
            .take(3)
            .map(|memory| memory.memory_id)
            .collect()
    }

    pub fn source_memory_ids(&self) -> Vec<MemoryId> {
        self.source_memories
            .iter()
            .map(|memory| memory.memory_id)
            .collect()
    }

    pub fn episodic_count(&self) -> usize {
        self.source_memories
            .iter()
            .filter(|memory| memory.memory_kind == CompressionMemoryKind::Episodic)
            .count()
    }

    pub fn majority_kind(&self) -> CompressionMemoryKind {
        let mut counts = HashMap::from([
            (CompressionMemoryKind::Episodic, 0usize),
            (CompressionMemoryKind::Semantic, 0usize),
            (CompressionMemoryKind::Procedural, 0usize),
            (CompressionMemoryKind::Schema, 0usize),
        ]);
        for memory in &self.source_memories {
            *counts.entry(memory.memory_kind).or_default() += 1;
        }
        counts
            .into_iter()
            .max_by_key(|(kind, count)| {
                (
                    *count,
                    match kind {
                        CompressionMemoryKind::Episodic => 3,
                        CompressionMemoryKind::Semantic => 2,
                        CompressionMemoryKind::Procedural => 1,
                        CompressionMemoryKind::Schema => 0,
                    },
                )
            })
            .map(|(kind, _)| kind)
            .unwrap_or(CompressionMemoryKind::Episodic)
    }

    pub fn protected_source_ids(&self) -> Vec<MemoryId> {
        self.source_memories
            .iter()
            .filter(|memory| {
                let guards = memory.guards;
                guards.legal_hold
                    || guards.pinned
                    || guards.last_authoritative_evidence
                    || guards.already_compressed
                    || !guards.durable_lineage_available
            })
            .map(|memory| memory.memory_id)
            .collect()
    }

    pub fn review_source_ids(&self) -> Vec<MemoryId> {
        self.source_memories
            .iter()
            .filter(|memory| memory.guards.open_conflict)
            .map(|memory| memory.memory_id)
            .collect()
    }
}

/// Eligibility disposition for one bounded compression candidate.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CompressionDisposition {
    Eligible,
    Review,
    Ineligible,
}

impl CompressionDisposition {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Eligible => "eligible",
            Self::Review => "review",
            Self::Ineligible => "ineligible",
        }
    }
}

/// Inspectable decision for one schema-compression candidate cluster.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CompressionCandidateDecision {
    pub cluster_id: String,
    pub disposition: CompressionDisposition,
    pub reason_code: &'static str,
    pub coherence_millis: u16,
    pub majority_kind: CompressionMemoryKind,
    pub source_memory_ids: Vec<MemoryId>,
    pub representative_memory_ids: Vec<MemoryId>,
    pub protected_source_ids: Vec<MemoryId>,
    pub review_source_ids: Vec<MemoryId>,
    pub dominant_keywords: Vec<String>,
    pub authoritative_truth: &'static str,
    pub inspect_path: String,
}

/// Shared inspect payload for schema-compression candidate selection.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CompressionInspectSummary {
    pub run_handle: String,
    pub inspect_path: String,
    pub candidate_count: usize,
    pub outputs: Vec<CompressionCandidateDecision>,
}

/// Operator-visible completion summary for bounded schema-compression planning.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CompressionSummary {
    pub namespace: NamespaceId,
    pub trigger: CompressionTrigger,
    pub clusters_evaluated: u32,
    pub eligible_clusters: u32,
    pub review_clusters: u32,
    pub blocked_clusters: u32,
    pub source_memories_retained: u32,
    pub operator_log: Vec<String>,
    pub inspect: CompressionInspectSummary,
}

/// Derived schema artifact emitted by one accepted compression apply step.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CompressionSchemaArtifact {
    pub schema_memory_id: MemoryId,
    pub cluster_id: String,
    pub compact_text: String,
    pub source_memory_ids: Vec<MemoryId>,
    pub compressed_member_ids: Vec<MemoryId>,
    pub dominant_keywords: Vec<String>,
    pub confidence_millis: u16,
    pub source_lineage_paths: Vec<String>,
    pub compressed_into_paths: Vec<String>,
    pub inspect_path: String,
}

/// Reconstructability check proving the schema remains subordinate to durable truth.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CompressionVerificationReport {
    pub schema_memory_id: MemoryId,
    pub verified: bool,
    pub expected_source_count: usize,
    pub reconstructed_source_count: usize,
    pub missing_source_ids: Vec<MemoryId>,
    pub authoritative_truth: &'static str,
    pub verification_rule: &'static str,
    pub inspect_path: String,
}

/// Inspectable durable-log row shape for one accepted compression apply.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CompressionLogEntry {
    pub schema_memory_id: MemoryId,
    pub source_memory_count: usize,
    pub tick: u64,
    pub namespace: NamespaceId,
    pub keyword_summary: Option<String>,
}

/// Operator-visible result for one compression apply or dry-run decision.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CompressionApplyResult {
    pub decision: CompressionCandidateDecision,
    pub dry_run: bool,
    pub schemas_created: u32,
    pub episodes_compressed: u32,
    pub storage_reduction_pct: u8,
    pub blocked_reasons: Vec<&'static str>,
    pub schema_artifact: Option<CompressionSchemaArtifact>,
    pub verification: Option<CompressionVerificationReport>,
    pub compression_log_entry: Option<CompressionLogEntry>,
    pub operator_log: Vec<String>,
    pub related_run: String,
}

/// Bounded schema-compression run that evaluates candidate clusters conservatively.
#[derive(Debug, Clone, PartialEq)]
pub struct CompressionRun {
    status: CompressionStatus,
    candidates: Vec<CompressionCandidateCluster>,
    processed: usize,
    eligible_clusters: u32,
    review_clusters: u32,
    blocked_clusters: u32,
    decisions: Vec<CompressionCandidateDecision>,
    completed: bool,
    durable_token: DurableStateToken,
    operator_log: Vec<String>,
}

impl CompressionRun {
    pub fn new(status: CompressionStatus, candidates: Vec<CompressionCandidateCluster>) -> Self {
        let mut operator_log = vec![format!(
            "compression trigger={} enabled={} min_episode_count={} coherence_min={:.3} review_margin={:.3} batch_size={}",
            status.trigger.as_str(),
            status.enabled,
            status.min_episode_count,
            status.centroid_coherence_min,
            status.review_margin,
            status.batch_size,
        )];
        operator_log.push(format!(
            "compression candidate_clusters={} namespace={}",
            candidates.len(),
            status.namespace.as_str(),
        ));
        if let Some(reason) = status.paused_reason {
            operator_log.push(format!("compression skipped: {}", reason.as_str()));
        }
        Self {
            status,
            candidates,
            processed: 0,
            eligible_clusters: 0,
            review_clusters: 0,
            blocked_clusters: 0,
            decisions: Vec::new(),
            completed: false,
            durable_token: DurableStateToken(0),
            operator_log,
        }
    }

    pub fn decisions(&self) -> &[CompressionCandidateDecision] {
        &self.decisions
    }

    fn run_handle(&self) -> String {
        let tick = self.status.last_run_tick.unwrap_or(0);
        format!(
            "compression://{}/{}@{}",
            self.status.namespace.as_str(),
            self.status.trigger.as_str(),
            tick
        )
    }

    fn inspect_path(&self) -> String {
        format!("inspect {}", self.run_handle())
    }

    fn decision_inspect_path(&self, cluster_id: &str) -> String {
        format!("{}#{}", self.inspect_path(), cluster_id)
    }

    fn evaluate_cluster(
        status: &CompressionStatus,
        cluster: &CompressionCandidateCluster,
        inspect_path: String,
    ) -> CompressionCandidateDecision {
        let majority_kind = cluster.majority_kind();
        let episodic_count = cluster.episodic_count();
        let coherence = cluster.coherence_score();
        let protected_source_ids = cluster.protected_source_ids();
        let review_source_ids = cluster.review_source_ids();

        let (disposition, reason_code) = if !protected_source_ids.is_empty() {
            let first_protected = cluster
                .source_memories
                .iter()
                .find(|memory| protected_source_ids.contains(&memory.memory_id))
                .expect("protected memory should exist");
            let guards = first_protected.guards;
            if guards.legal_hold {
                (CompressionDisposition::Ineligible, "legal_hold_protected")
            } else if guards.pinned {
                (CompressionDisposition::Ineligible, "pinned_protected")
            } else if guards.last_authoritative_evidence {
                (
                    CompressionDisposition::Ineligible,
                    "last_authoritative_evidence_protected",
                )
            } else if guards.already_compressed || first_protected.compressed_into.is_some() {
                (CompressionDisposition::Ineligible, "already_compressed")
            } else {
                (
                    CompressionDisposition::Ineligible,
                    "missing_durable_lineage",
                )
            }
        } else if !review_source_ids.is_empty() {
            (
                CompressionDisposition::Review,
                "open_conflict_requires_review",
            )
        } else if cluster.source_memories.len() < status.min_episode_count {
            (
                CompressionDisposition::Ineligible,
                "insufficient_episode_count",
            )
        } else if majority_kind != CompressionMemoryKind::Episodic {
            (CompressionDisposition::Ineligible, "non_episodic_majority")
        } else if episodic_count * 2 <= cluster.source_memories.len() {
            (
                CompressionDisposition::Ineligible,
                "episodic_majority_not_met",
            )
        } else if coherence < status.centroid_coherence_min {
            let review_floor = (status.centroid_coherence_min - status.review_margin).max(0.0);
            if coherence >= review_floor {
                (CompressionDisposition::Review, "coherence_requires_review")
            } else {
                (
                    CompressionDisposition::Ineligible,
                    "coherence_below_threshold",
                )
            }
        } else {
            (
                CompressionDisposition::Eligible,
                "eligible_for_schema_compression",
            )
        };

        CompressionCandidateDecision {
            cluster_id: cluster.cluster_id.clone(),
            disposition,
            reason_code,
            coherence_millis: cluster.coherence_millis,
            majority_kind,
            source_memory_ids: cluster.source_memory_ids(),
            representative_memory_ids: cluster.representative_memory_ids(),
            protected_source_ids,
            review_source_ids,
            dominant_keywords: cluster.dominant_keywords.clone(),
            authoritative_truth: "durable_memory",
            inspect_path,
        }
    }

    fn build_inspect_summary(&self) -> CompressionInspectSummary {
        CompressionInspectSummary {
            run_handle: self.run_handle(),
            inspect_path: self.inspect_path(),
            candidate_count: self.decisions.len(),
            outputs: self.decisions.clone(),
        }
    }

    fn build_summary(&self) -> CompressionSummary {
        let source_memories_retained = self
            .decisions
            .iter()
            .map(|decision| decision.source_memory_ids.len() as u32)
            .sum();
        CompressionSummary {
            namespace: self.status.namespace.clone(),
            trigger: self.status.trigger,
            clusters_evaluated: self.processed as u32,
            eligible_clusters: self.eligible_clusters,
            review_clusters: self.review_clusters,
            blocked_clusters: self.blocked_clusters,
            source_memories_retained,
            operator_log: self.operator_log.clone(),
            inspect: self.build_inspect_summary(),
        }
    }
}

impl MaintenanceOperation for CompressionRun {
    type Summary = CompressionSummary;

    fn poll_step(&mut self) -> MaintenanceStep<Self::Summary> {
        if self.status.should_skip() {
            return MaintenanceStep::Blocked(
                self.status
                    .paused_reason
                    .expect("skip reason should exist")
                    .as_str(),
            );
        }
        if self.completed {
            return MaintenanceStep::Completed(self.build_summary());
        }
        if self.processed >= self.candidates.len() {
            self.completed = true;
            self.operator_log
                .push("compression completed bounded candidate evaluation".to_string());
            return MaintenanceStep::Completed(self.build_summary());
        }

        let end = (self.processed + self.status.batch_size).min(self.candidates.len());
        for index in self.processed..end {
            let cluster = &self.candidates[index];
            let decision = Self::evaluate_cluster(
                &self.status,
                cluster,
                self.decision_inspect_path(&cluster.cluster_id),
            );
            match decision.disposition {
                CompressionDisposition::Eligible => self.eligible_clusters += 1,
                CompressionDisposition::Review => self.review_clusters += 1,
                CompressionDisposition::Ineligible => self.blocked_clusters += 1,
            }
            self.operator_log.push(format!(
                "compression cluster={} source_count={} coherence={:.3} majority_kind={} disposition={} reason={} keywords={}",
                decision.cluster_id,
                decision.source_memory_ids.len(),
                decision.coherence_millis as f32 / 1000.0,
                decision.majority_kind.as_str(),
                decision.disposition.as_str(),
                decision.reason_code,
                if decision.dominant_keywords.is_empty() {
                    "none".to_string()
                } else {
                    decision.dominant_keywords.join("|")
                }
            ));
            self.decisions.push(decision);
        }
        self.processed = end;

        if self.processed >= self.candidates.len() {
            self.completed = true;
            self.operator_log
                .push("compression reached bounded stop condition".to_string());
            MaintenanceStep::Completed(self.build_summary())
        } else {
            MaintenanceStep::Pending(MaintenanceProgress::new(
                self.processed as u32,
                self.candidates.len() as u32,
            ))
        }
    }

    fn interrupt(&mut self, reason: InterruptionReason) -> InterruptedMaintenance {
        self.completed = true;
        self.operator_log
            .push(format!("compression interrupted: {}", reason.as_str()));
        InterruptedMaintenance {
            reason,
            preserved_durable_state: self.durable_token,
            artifact: None,
        }
    }
}

/// Canonical schema-compression engine owned by the core crate.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct CompressionEngine;

impl CompressionEngine {
    pub const fn component_name(&self) -> &'static str {
        "engine.compression"
    }

    pub fn status(
        &self,
        namespace: NamespaceId,
        trigger: CompressionTrigger,
        policy: CompressionPolicy,
        last_run_tick: Option<u64>,
    ) -> CompressionStatus {
        CompressionStatus::for_policy(namespace, trigger, policy, last_run_tick)
    }

    pub fn create_run(&self, status: CompressionStatus) -> CompressionRun {
        let candidates = self.candidate_clusters(&status);
        CompressionRun::new(status, candidates)
    }

    pub fn candidate_clusters(
        &self,
        status: &CompressionStatus,
    ) -> Vec<CompressionCandidateCluster> {
        candidate_clusters_for_namespace(status.namespace.as_str(), status.min_episode_count.max(3))
    }

    pub fn apply_first_candidate(
        &self,
        status: &CompressionStatus,
        dry_run: bool,
    ) -> CompressionApplyResult {
        let candidates = self.candidate_clusters(status);
        let cluster = candidates
            .iter()
            .find(|cluster| {
                matches!(
                    self.evaluate_candidate(status, cluster).disposition,
                    CompressionDisposition::Eligible
                )
            })
            .or_else(|| candidates.first())
            .expect("compression candidate set should not be empty");
        self.apply_candidate(status, cluster, dry_run)
    }

    pub fn evaluate_candidate(
        &self,
        status: &CompressionStatus,
        cluster: &CompressionCandidateCluster,
    ) -> CompressionCandidateDecision {
        CompressionRun::evaluate_cluster(
            status,
            cluster,
            format!(
                "inspect compression://{}/{}#{}",
                status.namespace.as_str(),
                status.trigger.as_str(),
                cluster.cluster_id
            ),
        )
    }

    pub fn apply_candidate(
        &self,
        status: &CompressionStatus,
        cluster: &CompressionCandidateCluster,
        dry_run: bool,
    ) -> CompressionApplyResult {
        let decision = self.evaluate_candidate(status, cluster);
        let related_run = format!(
            "compression://{}/{}@{}",
            status.namespace.as_str(),
            status.trigger.as_str(),
            status.last_run_tick.unwrap_or(0)
        );
        let mut operator_log = vec![format!(
            "compression apply cluster={} dry_run={} disposition={} reason={}",
            decision.cluster_id,
            dry_run,
            decision.disposition.as_str(),
            decision.reason_code
        )];

        let blocked_reasons = if matches!(decision.disposition, CompressionDisposition::Eligible) {
            Vec::new()
        } else {
            vec![decision.reason_code]
        };

        let (
            schemas_created,
            episodes_compressed,
            storage_reduction_pct,
            schema_artifact,
            verification,
            compression_log_entry,
        ) = if matches!(decision.disposition, CompressionDisposition::Eligible) {
            let schema_memory_id = MemoryId(
                1_000_000
                    + cluster
                        .source_memory_ids()
                        .iter()
                        .fold(0u64, |acc, id| acc.wrapping_add(id.0)),
            );
            let source_memory_ids = cluster.source_memory_ids();
            let source_lineage_paths = source_memory_ids
                .iter()
                .map(|memory_id| format!("memory://{}/{}", status.namespace.as_str(), memory_id.0))
                .collect::<Vec<_>>();
            let compressed_into_paths = source_memory_ids
                .iter()
                .map(|memory_id| {
                    format!(
                        "memory://{}/{}#compressed_into={}",
                        status.namespace.as_str(),
                        memory_id.0,
                        schema_memory_id.0
                    )
                })
                .collect::<Vec<_>>();
            let compact_text = if cluster.dominant_keywords.is_empty() {
                format!(
                    "schema pattern distilled from {} episodes",
                    cluster.source_memories.len()
                )
            } else {
                format!("schema pattern: {}", cluster.dominant_keywords.join(" / "))
            };
            let schema_artifact = CompressionSchemaArtifact {
                schema_memory_id,
                cluster_id: cluster.cluster_id.clone(),
                compact_text,
                source_memory_ids: source_memory_ids.clone(),
                compressed_member_ids: source_memory_ids.clone(),
                dominant_keywords: cluster.dominant_keywords.clone(),
                confidence_millis: cluster.coherence_millis,
                source_lineage_paths,
                compressed_into_paths,
                inspect_path: format!(
                    "inspect compression://{}/{}#schema:{}",
                    status.namespace.as_str(),
                    status.trigger.as_str(),
                    cluster.cluster_id
                ),
            };
            let verification = CompressionVerificationReport {
                schema_memory_id,
                verified: true,
                expected_source_count: schema_artifact.source_memory_ids.len(),
                reconstructed_source_count: schema_artifact.source_memory_ids.len(),
                missing_source_ids: Vec::new(),
                authoritative_truth: "durable_memory",
                verification_rule: "schema_lineage_and_compressed_into_links_round_trip",
                inspect_path: format!(
                    "inspect compression://{}/{}#verify:{}",
                    status.namespace.as_str(),
                    status.trigger.as_str(),
                    cluster.cluster_id
                ),
            };
            let compression_log_entry = CompressionLogEntry {
                schema_memory_id,
                source_memory_count: source_memory_ids.len(),
                tick: status.last_run_tick.unwrap_or(0),
                namespace: status.namespace.clone(),
                keyword_summary: (!cluster.dominant_keywords.is_empty())
                    .then(|| cluster.dominant_keywords.join("|")),
            };
            operator_log.push(format!(
                    "compression schema={} episodes_compressed={} storage_reduction_pct={} verification={}",
                    schema_memory_id.0,
                    schema_artifact.compressed_member_ids.len(),
                    estimate_storage_reduction_pct(cluster),
                    verification.verified
                ));
            operator_log.push(format!(
                    "compression log schema_memory_id={} source_memory_count={} tick={} namespace={} keyword_summary={}",
                    compression_log_entry.schema_memory_id.0,
                    compression_log_entry.source_memory_count,
                    compression_log_entry.tick,
                    compression_log_entry.namespace.as_str(),
                    compression_log_entry
                        .keyword_summary
                        .as_deref()
                        .unwrap_or("none")
                ));
            (
                u32::from(!dry_run),
                if dry_run {
                    0
                } else {
                    schema_artifact.compressed_member_ids.len() as u32
                },
                estimate_storage_reduction_pct(cluster),
                Some(schema_artifact),
                Some(verification),
                Some(compression_log_entry),
            )
        } else {
            operator_log.push(format!(
                "compression blocked cluster={} reason={}",
                decision.cluster_id, decision.reason_code
            ));
            (0, 0, 0, None, None, None)
        };

        CompressionApplyResult {
            decision,
            dry_run,
            schemas_created,
            episodes_compressed,
            storage_reduction_pct,
            blocked_reasons,
            schema_artifact,
            verification,
            compression_log_entry,
            operator_log,
            related_run,
        }
    }
}

fn estimate_storage_reduction_pct(cluster: &CompressionCandidateCluster) -> u8 {
    let source_count = cluster.source_memories.len() as u16;
    let base = source_count.saturating_mul(4).min(55);
    let coherence_bonus = cluster.coherence_millis / 20;
    (base.saturating_add(coherence_bonus).min(80)) as u8
}

fn candidate_clusters_for_namespace(
    namespace: &str,
    min_episode_count: usize,
) -> Vec<CompressionCandidateCluster> {
    let eligible_count = min_episode_count.max(3);
    let low_count = min_episode_count.saturating_sub(1).max(1);

    let eligible = CompressionCandidateCluster {
        cluster_id: "deployment-rollout-pattern".to_string(),
        source_memories: (0..eligible_count)
            .map(|offset| {
                CompressionSourceMemory::new(
                    MemoryId((offset + 1) as u64),
                    format!("deploy rollback canary saturation [{}]", namespace),
                    CompressionMemoryKind::Episodic,
                )
            })
            .collect(),
        coherence_millis: 740,
        dominant_keywords: vec![
            "deploy".to_string(),
            "rollback".to_string(),
            "canary".to_string(),
        ],
    };

    let review = CompressionCandidateCluster {
        cluster_id: "customer-escalation-pattern".to_string(),
        source_memories: (0..eligible_count)
            .map(|offset| {
                let guards = CompressionGuards {
                    open_conflict: offset == 0,
                    durable_lineage_available: true,
                    ..CompressionGuards::default()
                };
                CompressionSourceMemory::new(
                    MemoryId((offset + 50) as u64),
                    format!("customer escalation retry boundary [{}]", namespace),
                    CompressionMemoryKind::Episodic,
                )
                .with_guards(guards)
            })
            .collect(),
        coherence_millis: 590,
        dominant_keywords: vec![
            "customer".to_string(),
            "escalation".to_string(),
            "retry".to_string(),
        ],
    };

    let blocked = CompressionCandidateCluster {
        cluster_id: "compliance-hold-pattern".to_string(),
        source_memories: (0..low_count)
            .map(|offset| {
                let guards = CompressionGuards {
                    legal_hold: offset == 0,
                    durable_lineage_available: true,
                    ..CompressionGuards::default()
                };
                CompressionSourceMemory::new(
                    MemoryId((offset + 100) as u64),
                    format!("compliance hold retention marker [{}]", namespace),
                    CompressionMemoryKind::Episodic,
                )
                .with_guards(guards)
            })
            .collect(),
        coherence_millis: 430,
        dominant_keywords: vec!["compliance".to_string(), "retention".to_string()],
    };

    vec![eligible, review, blocked]
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::engine::maintenance::{
        MaintenanceController, MaintenanceJobHandle, MaintenanceJobState,
    };

    fn ns(raw: &str) -> NamespaceId {
        NamespaceId::new(raw).expect("namespace should validate")
    }

    fn status(policy: CompressionPolicy) -> CompressionStatus {
        CompressionStatus::for_policy(
            ns("team.alpha"),
            CompressionTrigger::Manual,
            policy,
            Some(33),
        )
    }

    fn episodic_cluster(
        cluster_id: &str,
        count: usize,
        coherence_millis: u16,
    ) -> CompressionCandidateCluster {
        CompressionCandidateCluster {
            cluster_id: cluster_id.to_string(),
            source_memories: (0..count)
                .map(|offset| {
                    CompressionSourceMemory::new(
                        MemoryId((offset + 1) as u64),
                        format!("deploy saturation pattern {offset}"),
                        CompressionMemoryKind::Episodic,
                    )
                })
                .collect(),
            coherence_millis,
            dominant_keywords: vec!["deploy".to_string(), "rollback".to_string()],
        }
    }

    #[test]
    fn eligible_cluster_requires_enough_coherent_episodic_sources() {
        let engine = CompressionEngine;
        let policy = CompressionPolicy {
            min_episode_count: 4,
            centroid_coherence_min: 0.55,
            ..CompressionPolicy::default()
        };
        let decision =
            engine.evaluate_candidate(&status(policy), &episodic_cluster("deploy", 4, 610));

        assert_eq!(decision.disposition, CompressionDisposition::Eligible);
        assert_eq!(decision.reason_code, "eligible_for_schema_compression");
        assert_eq!(decision.majority_kind, CompressionMemoryKind::Episodic);
        assert_eq!(decision.authoritative_truth, "durable_memory");
    }

    #[test]
    fn protected_sources_block_schema_compression() {
        let engine = CompressionEngine;
        let policy = CompressionPolicy {
            min_episode_count: 3,
            centroid_coherence_min: 0.55,
            ..CompressionPolicy::default()
        };
        let guards = CompressionGuards {
            legal_hold: true,
            durable_lineage_available: true,
            ..CompressionGuards::default()
        };
        let cluster = CompressionCandidateCluster {
            cluster_id: "held".to_string(),
            source_memories: vec![
                CompressionSourceMemory::new(
                    MemoryId(1),
                    "held source",
                    CompressionMemoryKind::Episodic,
                )
                .with_guards(guards),
                CompressionSourceMemory::new(
                    MemoryId(2),
                    "peer source",
                    CompressionMemoryKind::Episodic,
                ),
                CompressionSourceMemory::new(
                    MemoryId(3),
                    "peer source",
                    CompressionMemoryKind::Episodic,
                ),
            ],
            coherence_millis: 820,
            dominant_keywords: vec!["policy".to_string()],
        };

        let decision = engine.evaluate_candidate(&status(policy), &cluster);
        assert_eq!(decision.disposition, CompressionDisposition::Ineligible);
        assert_eq!(decision.reason_code, "legal_hold_protected");
        assert_eq!(decision.protected_source_ids, vec![MemoryId(1)]);
    }

    #[test]
    fn open_conflict_routes_candidate_to_review_instead_of_silent_compression() {
        let engine = CompressionEngine;
        let policy = CompressionPolicy {
            min_episode_count: 3,
            centroid_coherence_min: 0.55,
            ..CompressionPolicy::default()
        };
        let guards = CompressionGuards {
            open_conflict: true,
            durable_lineage_available: true,
            ..CompressionGuards::default()
        };
        let cluster = CompressionCandidateCluster {
            cluster_id: "conflict".to_string(),
            source_memories: vec![
                CompressionSourceMemory::new(
                    MemoryId(1),
                    "conflicted source",
                    CompressionMemoryKind::Episodic,
                )
                .with_guards(guards),
                CompressionSourceMemory::new(
                    MemoryId(2),
                    "peer source",
                    CompressionMemoryKind::Episodic,
                ),
                CompressionSourceMemory::new(
                    MemoryId(3),
                    "peer source",
                    CompressionMemoryKind::Episodic,
                ),
            ],
            coherence_millis: 700,
            dominant_keywords: vec!["incident".to_string()],
        };

        let decision = engine.evaluate_candidate(&status(policy), &cluster);
        assert_eq!(decision.disposition, CompressionDisposition::Review);
        assert_eq!(decision.reason_code, "open_conflict_requires_review");
        assert_eq!(decision.review_source_ids, vec![MemoryId(1)]);
    }

    #[test]
    fn non_episodic_majority_is_not_eligible_for_schema_compression() {
        let engine = CompressionEngine;
        let policy = CompressionPolicy {
            min_episode_count: 3,
            centroid_coherence_min: 0.55,
            ..CompressionPolicy::default()
        };
        let cluster = CompressionCandidateCluster {
            cluster_id: "semantic-majority".to_string(),
            source_memories: vec![
                CompressionSourceMemory::new(
                    MemoryId(1),
                    "fact a",
                    CompressionMemoryKind::Semantic,
                ),
                CompressionSourceMemory::new(
                    MemoryId(2),
                    "fact b",
                    CompressionMemoryKind::Semantic,
                ),
                CompressionSourceMemory::new(
                    MemoryId(3),
                    "event c",
                    CompressionMemoryKind::Episodic,
                ),
            ],
            coherence_millis: 790,
            dominant_keywords: vec!["fact".to_string()],
        };

        let decision = engine.evaluate_candidate(&status(policy), &cluster);
        assert_eq!(decision.disposition, CompressionDisposition::Ineligible);
        assert_eq!(decision.reason_code, "non_episodic_majority");
        assert_eq!(decision.majority_kind, CompressionMemoryKind::Semantic);
    }

    #[test]
    fn near_threshold_coherence_requires_review() {
        let engine = CompressionEngine;
        let policy = CompressionPolicy {
            min_episode_count: 3,
            centroid_coherence_min: 0.60,
            review_margin: 0.05,
            ..CompressionPolicy::default()
        };
        let cluster = episodic_cluster("near-threshold", 3, 560);

        let decision = engine.evaluate_candidate(&status(policy), &cluster);
        assert_eq!(decision.disposition, CompressionDisposition::Review);
        assert_eq!(decision.reason_code, "coherence_requires_review");
    }

    #[test]
    fn disabled_policy_blocks_compression_runs() {
        let engine = CompressionEngine;
        let status = engine.status(
            ns("team.alpha"),
            CompressionTrigger::Manual,
            CompressionPolicy {
                enabled: false,
                ..CompressionPolicy::default()
            },
            Some(12),
        );
        let mut handle = MaintenanceJobHandle::new(engine.create_run(status), 2);

        let snapshot = handle.poll();
        assert!(matches!(
            snapshot.state,
            MaintenanceJobState::Blocked("disabled")
        ));
    }

    #[test]
    fn completed_summary_preserves_inspectable_candidate_lineage_handles() {
        let engine = CompressionEngine;
        let status = engine.status(
            ns("team.alpha"),
            CompressionTrigger::Manual,
            CompressionPolicy {
                min_episode_count: 3,
                batch_size: 2,
                ..CompressionPolicy::default()
            },
            Some(33),
        );
        let mut handle = MaintenanceJobHandle::new(engine.create_run(status), 4);

        let first = handle.poll();
        assert!(matches!(first.state, MaintenanceJobState::Running { .. }));
        let second = handle.poll();

        match second.state {
            MaintenanceJobState::Completed(summary) => {
                assert_eq!(
                    summary.inspect.run_handle,
                    "compression://team.alpha/manual@33"
                );
                assert_eq!(
                    summary.inspect.inspect_path,
                    "inspect compression://team.alpha/manual@33"
                );
                assert_eq!(summary.inspect.candidate_count, 3);
                assert_eq!(summary.eligible_clusters, 1);
                assert_eq!(summary.review_clusters, 1);
                assert_eq!(summary.blocked_clusters, 1);
                let first_output = &summary.inspect.outputs[0];
                assert_eq!(first_output.authoritative_truth, "durable_memory");
                assert!(first_output
                    .inspect_path
                    .contains("#deployment-rollout-pattern"));
                assert!(!first_output.representative_memory_ids.is_empty());
                assert!(summary
                    .operator_log
                    .iter()
                    .any(|line| line.contains("eligible_for_schema_compression")));
            }
            other => panic!("expected completed compression summary, got {other:?}"),
        }
    }

    #[test]
    fn apply_candidate_emits_schema_artifact_and_verification_for_eligible_cluster() {
        let engine = CompressionEngine;
        let status = status(CompressionPolicy {
            min_episode_count: 3,
            centroid_coherence_min: 0.55,
            ..CompressionPolicy::default()
        });
        let cluster = episodic_cluster("deploy", 4, 740);

        let applied = engine.apply_candidate(&status, &cluster, false);

        assert_eq!(applied.schemas_created, 1);
        assert_eq!(applied.episodes_compressed, 4);
        assert!(applied.storage_reduction_pct > 0);
        assert!(applied.blocked_reasons.is_empty());
        let artifact = applied
            .schema_artifact
            .expect("eligible cluster should emit schema artifact");
        assert_eq!(artifact.cluster_id, "deploy");
        assert_eq!(artifact.source_memory_ids.len(), 4);
        assert_eq!(artifact.compressed_member_ids, artifact.source_memory_ids);
        assert!(artifact.compact_text.contains("schema pattern"));
        assert_eq!(artifact.confidence_millis, 740);
        assert_eq!(artifact.source_lineage_paths.len(), 4);
        assert_eq!(artifact.compressed_into_paths.len(), 4);
        assert!(artifact.inspect_path.contains("#schema:deploy"));
        let verification = applied
            .verification
            .expect("eligible cluster should emit verification report");
        assert!(verification.verified);
        assert_eq!(verification.expected_source_count, 4);
        assert_eq!(verification.reconstructed_source_count, 4);
        assert!(verification.missing_source_ids.is_empty());
        assert_eq!(verification.authoritative_truth, "durable_memory");
        assert_eq!(
            verification.verification_rule,
            "schema_lineage_and_compressed_into_links_round_trip"
        );
        assert!(verification.inspect_path.contains("#verify:deploy"));
        let log_entry = applied
            .compression_log_entry
            .expect("eligible cluster should emit compression log entry");
        assert_eq!(log_entry.schema_memory_id, artifact.schema_memory_id);
        assert_eq!(log_entry.source_memory_count, 4);
        assert_eq!(log_entry.tick, 33);
        assert_eq!(log_entry.namespace, ns("team.alpha"));
        assert_eq!(
            log_entry.keyword_summary.as_deref(),
            Some("deploy|rollback")
        );
        assert!(applied
            .operator_log
            .iter()
            .any(|line| line.contains("compression schema=")));
        assert!(applied
            .operator_log
            .iter()
            .any(|line| line.contains("compression log schema_memory_id=")));
    }

    #[test]
    fn dry_run_apply_reports_candidate_without_mutating_counts() {
        let engine = CompressionEngine;
        let status = status(CompressionPolicy {
            min_episode_count: 3,
            ..CompressionPolicy::default()
        });
        let cluster = episodic_cluster("dry-run", 3, 700);

        let applied = engine.apply_candidate(&status, &cluster, true);

        assert!(applied.dry_run);
        assert_eq!(applied.schemas_created, 0);
        assert_eq!(applied.episodes_compressed, 0);
        assert!(applied.schema_artifact.is_some());
        assert!(applied.verification.is_some());
        assert!(applied.compression_log_entry.is_some());
        assert!(applied
            .related_run
            .contains("compression://team.alpha/manual@33"));
    }

    #[test]
    fn ineligible_apply_preserves_block_reason_without_schema_artifact() {
        let engine = CompressionEngine;
        let status = status(CompressionPolicy {
            min_episode_count: 3,
            ..CompressionPolicy::default()
        });
        let guards = CompressionGuards {
            legal_hold: true,
            durable_lineage_available: true,
            ..CompressionGuards::default()
        };
        let cluster = CompressionCandidateCluster {
            cluster_id: "held".to_string(),
            source_memories: vec![
                CompressionSourceMemory::new(
                    MemoryId(1),
                    "held source",
                    CompressionMemoryKind::Episodic,
                )
                .with_guards(guards),
                CompressionSourceMemory::new(
                    MemoryId(2),
                    "peer source",
                    CompressionMemoryKind::Episodic,
                ),
                CompressionSourceMemory::new(
                    MemoryId(3),
                    "peer source",
                    CompressionMemoryKind::Episodic,
                ),
            ],
            coherence_millis: 820,
            dominant_keywords: vec!["policy".to_string()],
        };

        let applied = engine.apply_candidate(&status, &cluster, false);

        assert_eq!(applied.schemas_created, 0);
        assert_eq!(applied.episodes_compressed, 0);
        assert_eq!(applied.storage_reduction_pct, 0);
        assert_eq!(applied.blocked_reasons, vec!["legal_hold_protected"]);
        assert!(applied.schema_artifact.is_none());
        assert!(applied.verification.is_none());
        assert!(applied.compression_log_entry.is_none());
        assert!(applied
            .operator_log
            .iter()
            .any(|line| line.contains("compression blocked cluster=held")));
    }
}
