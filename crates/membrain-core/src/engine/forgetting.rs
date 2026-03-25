//! Forgetting, demotion, archive, and restore surfaces.
//!
//! Owns the bounded lifecycle pressure after consolidation: demoting noisy
//! memories, archiving low-utility ones without silently deleting truth,
//! and exposing explicit restore and policy-deletion semantics as distinct paths.

use crate::api::NamespaceId;
use crate::engine::maintenance::{
    DurableStateToken, InterruptedMaintenance, InterruptionReason, MaintenanceOperation,
    MaintenanceProgress, MaintenanceStep,
};
use crate::observability::AuditEventKind;
use crate::store::audit::AuditLogEntry;
use crate::store::cold::ArchiveState;
use crate::store::tier_router::{LifecycleState, TierOwnership};
use crate::types::MemoryId;

// ── Forgetting policy ────────────────────────────────────────────────────────

/// Policy controlling bounded forgetting and demotion behavior.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ForgettingPolicy {
    /// Score below which memories are archive-eligible on the utility-forgetting path (0..1000).
    pub forget_score_threshold: u16,
    /// Score below which memories are demoted one tier while staying active (0..1000).
    pub demotion_score_threshold: u16,
    /// Maximum memories to process in one bounded run.
    pub batch_size: usize,
    /// Days without access before a memory is eligible for demotion.
    pub idle_days_before_demotion: u32,
    /// Whether a separate explicit policy-deletion path may authorize irreversible hard delete.
    pub allow_hard_delete: bool,
}

impl Default for ForgettingPolicy {
    fn default() -> Self {
        Self {
            forget_score_threshold: 50,
            demotion_score_threshold: 200,
            batch_size: 100,
            idle_days_before_demotion: 90,
            allow_hard_delete: false,
        }
    }
}

// ── Forgetting actions ───────────────────────────────────────────────────────

/// Actions the forgetting engine can take on a memory.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ForgettingAction {
    /// Utility-driven forgetting archives the memory without destroying durable truth.
    Archive,
    /// Demote from hot → tier2 while keeping the memory active and recall-eligible.
    Demote {
        from_tier: &'static str,
        to_tier: &'static str,
    },
    /// Explicitly restore a previously archived memory to active serving eligibility.
    Restore {
        from_tier: &'static str,
        to_tier: &'static str,
        partial: bool,
    },
    /// Separate explicit policy-deletion path; never implied by utility forgetting.
    PolicyDelete,
    /// Skip — memory is still above the forgetting and demotion thresholds.
    Skip,
}

impl ForgettingAction {
    /// Returns the stable machine-readable label.
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Archive => "archive",
            Self::Demote { .. } => "demote",
            Self::Restore { .. } => "restore",
            Self::PolicyDelete => "policy_delete",
            Self::Skip => "skip",
        }
    }
}

/// Decision record for one memory evaluated by the forgetting engine.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ForgettingDecision {
    /// Memory being evaluated.
    pub memory_id: MemoryId,
    /// Action taken.
    pub action: ForgettingAction,
    /// Current score that triggered the decision.
    pub current_score: u16,
    /// Machine-readable reason.
    pub reason: &'static str,
}

// ── Forgetting summary ──────────────────────────────────────────────────────

/// Flat audit record preserved in operator-visible forgetting summaries.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
pub struct ForgettingAuditRecordFlat {
    pub memory_id: MemoryId,
    pub tick: u64,
    pub action: ForgettingAction,
    pub reason_code: &'static str,
    pub disposition: &'static str,
    pub current_score: u16,
    pub forget_threshold: u16,
    pub demotion_threshold: u16,
    pub policy_surface: &'static str,
    pub reversibility: Reversibility,
    pub source_tier: Option<&'static str>,
    pub target_tier: Option<&'static str>,
    pub prior_archive_state: Option<&'static str>,
    pub resulting_archive_state: Option<&'static str>,
    pub archive_reason: Option<&'static str>,
    pub retention_state: &'static str,
    pub lifecycle_state: &'static str,
    pub current_tier: &'static str,
    pub partial_restore: bool,
    pub audit_kind: &'static str,
}

/// Operator-visible summary after a forgetting run.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
pub struct ForgettingSummary {
    /// Total memories evaluated.
    pub evaluated: u32,
    /// Memories archived by utility forgetting.
    pub archived: u32,
    /// Memories demoted while remaining active.
    pub demoted: u32,
    /// Memories explicitly restored.
    pub restored: u32,
    /// Memories removed by explicit policy deletion.
    pub policy_deleted: u32,
    /// Memories skipped because they remained healthy.
    pub skipped: u32,
    /// Candidates surfaced for operator review instead of silent mutation.
    pub review_required: u32,
    /// Flat audit artifacts for later operator review.
    pub audit_records: Vec<ForgettingAuditRecordFlat>,
}

// ── Reversibility classification ─────────────────────────────────────────────

/// Whether a lifecycle action can be undone and under what conditions.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Reversibility {
    /// Reversible without crossing an archive boundary.
    Reversible,
    /// Archived by utility forgetting; explicit restore is required.
    RestoreRequired,
    /// Hard-delete: irreversible; durable truth was intentionally removed.
    Irreversible,
    /// No action was taken; nothing to reverse.
    NoAction,
}

impl Reversibility {
    /// Returns the stable machine-readable label.
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Reversible => "reversible",
            Self::RestoreRequired => "restore_required",
            Self::Irreversible => "irreversible",
            Self::NoAction => "no_action",
        }
    }
}

// ── Audit records ────────────────────────────────────────────────────────────

/// Structured audit record for a single lifecycle decision in the forgetting lane.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ForgettingAuditRecord {
    /// Memory that was evaluated.
    pub memory_id: MemoryId,
    /// Action taken on the memory.
    pub action: ForgettingAction,
    /// Effective score at evaluation time.
    pub current_score: u16,
    /// Forget threshold used for this run.
    pub forget_threshold: u16,
    /// Demotion threshold used for this run.
    pub demotion_threshold: u16,
    /// Machine-readable reason code.
    pub reason: &'static str,
    /// Whether the action is reversible and under what conditions.
    pub reversibility: Reversibility,
    /// Source tier before the action when tier ownership changed.
    pub source_tier: Option<&'static str>,
    /// Target tier after the action when tier ownership changed.
    pub target_tier: Option<&'static str>,
    /// Cold/archive lifecycle state before the action.
    pub prior_archive_state: Option<&'static str>,
    /// Cold/archive lifecycle state after the action.
    pub resulting_archive_state: Option<&'static str>,
    /// Why an archived item was archived, when applicable.
    pub archive_reason: Option<&'static str>,
    /// Whether restore returned degraded or partial fidelity.
    pub partial_restore: bool,
    /// Which policy surface produced the action.
    pub policy_surface: &'static str,
    /// Logical tick when the decision was recorded.
    pub tick: u64,
}

/// Operator-facing explain output for a single forgetting decision.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
pub struct ForgettingExplainEntry {
    /// Memory that was evaluated.
    pub memory_id: MemoryId,
    /// Human-readable explanation of the decision.
    pub explanation: String,
    /// Action taken.
    pub action: ForgettingAction,
    /// Effective score at evaluation time.
    pub current_score: u16,
    /// Threshold or explicit path that triggered the action.
    pub threshold_exceeded: &'static str,
    /// Machine-readable reason that produced or blocked the action.
    pub reason_code: &'static str,
    /// Eligibility disposition after policy/lifecycle guards.
    pub disposition: &'static str,
    /// Whether the action is reversible.
    pub reversibility: Reversibility,
    /// Which policy surface produced the action.
    pub policy_surface: &'static str,
    /// Tier before the action when routing changed.
    pub source_tier: Option<&'static str>,
    /// Tier after the action when routing changed.
    pub target_tier: Option<&'static str>,
    /// Retention/governance state seen during evaluation.
    pub retention_state: Option<&'static str>,
    /// Lifecycle state seen during evaluation.
    pub lifecycle_state: Option<&'static str>,
    /// Current canonical tier seen during evaluation.
    pub current_tier: Option<&'static str>,
    /// Resulting archive lifecycle state, when applicable.
    pub resulting_archive_state: Option<&'static str>,
    /// Whether restore returned degraded or partial fidelity.
    pub partial_restore: bool,
    /// Audit event kind emitted for the decision.
    pub audit_kind: &'static str,
    /// Logical tick associated with the decision.
    pub tick: u64,
}

/// Operator-facing explain output for an entire forgetting run.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
pub struct ForgettingExplainOutput {
    /// Namespace the run operated on.
    pub namespace: NamespaceId,
    /// Forget threshold used.
    pub forget_threshold: u16,
    /// Demotion threshold used.
    pub demotion_threshold: u16,
    /// Total candidates evaluated.
    pub evaluated: u32,
    /// Per-decision explain entries.
    pub entries: Vec<ForgettingExplainEntry>,
    /// Summary counts by reversibility class.
    pub reversible_count: u32,
    /// Count of archive actions that require explicit restore.
    pub restore_required_count: u32,
    /// Count of irreversible actions.
    pub irreversible_count: u32,
    /// Count of no-action (skip) entries.
    pub no_action_count: u32,
    /// Count of decisions preserved for operator review rather than mutation.
    pub review_count: u32,
}

/// Reversibility summary for operator review.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
pub struct ReversibilitySummary {
    /// Total decisions evaluated.
    pub total: u32,
    /// Fully reversible decisions.
    pub reversible: u32,
    /// Archive decisions that require explicit restore.
    pub restore_required: u32,
    /// Irreversible decisions (hard-delete).
    pub irreversible: u32,
    /// No-action decisions (skipped).
    pub no_action: u32,
    /// Decisions preserved for explicit operator review.
    pub review_required: u32,
}

impl ReversibilitySummary {
    /// Returns true if any irreversible actions were taken.
    pub const fn has_irreversible(&self) -> bool {
        self.irreversible > 0
    }

    /// Returns the fraction of decisions that are fully reversible.
    pub fn reversible_fraction(&self) -> f32 {
        if self.total == 0 {
            return 1.0;
        }
        self.reversible as f32 / self.total as f32
    }
}

// ── Forgetting operation ─────────────────────────────────────────────────────

/// Candidate memory evaluated by the forgetting run.
#[derive(Debug, Clone, PartialEq)]
pub struct ForgettingCandidate {
    /// Memory to evaluate.
    pub memory_id: MemoryId,
    /// Current effective score (0..1000).
    pub current_score: u16,
    /// Optional multi-factor eligibility score used to make the decision explicit.
    pub eligibility_score: Option<EligibilityScore>,
}

/// Bounded forgetting operation for the maintenance controller.
#[derive(Debug, Clone, PartialEq)]
pub struct ForgettingRun {
    namespace: NamespaceId,
    policy: ForgettingPolicy,
    candidates: Vec<ForgettingCandidate>,
    decisions: Vec<ForgettingDecision>,
    audit_records: Vec<ForgettingAuditRecord>,
    processed: u32,
    total: u32,
    archived: u32,
    demoted: u32,
    restored: u32,
    policy_deleted: u32,
    skipped: u32,
    completed: bool,
    durable_token: DurableStateToken,
    current_tick: u64,
}

impl ForgettingRun {
    /// Creates a new bounded forgetting run with explicit candidate scores.
    pub fn new(
        namespace: NamespaceId,
        policy: ForgettingPolicy,
        candidates: Vec<ForgettingCandidate>,
    ) -> Self {
        let total = candidates.len() as u32;
        Self {
            namespace,
            policy,
            candidates,
            decisions: Vec::new(),
            audit_records: Vec::new(),
            processed: 0,
            total,
            archived: 0,
            demoted: 0,
            restored: 0,
            policy_deleted: 0,
            skipped: 0,
            completed: false,
            durable_token: DurableStateToken(0),
            current_tick: 0,
        }
    }

    /// Sets the logical tick for audit records.
    pub fn with_tick(mut self, tick: u64) -> Self {
        self.current_tick = tick;
        self
    }

    /// Returns the recorded forgetting decisions.
    pub fn decisions(&self) -> &[ForgettingDecision] {
        &self.decisions
    }

    /// Returns the structured audit trail for this run.
    pub fn audit_records(&self) -> &[ForgettingAuditRecord] {
        &self.audit_records
    }

    /// Returns the namespace this run operated on.
    pub fn namespace(&self) -> &NamespaceId {
        &self.namespace
    }

    fn build_summary(&self) -> ForgettingSummary {
        let audit_records = self
            .audit_records
            .iter()
            .map(|record| ForgettingAuditRecordFlat {
                memory_id: record.memory_id,
                tick: record.tick,
                action: record.action,
                reason_code: record.reason,
                disposition: self.disposition_for_record(record),
                current_score: record.current_score,
                forget_threshold: record.forget_threshold,
                demotion_threshold: record.demotion_threshold,
                policy_surface: record.policy_surface,
                reversibility: record.reversibility,
                source_tier: record.source_tier,
                target_tier: record.target_tier,
                prior_archive_state: record.prior_archive_state,
                resulting_archive_state: record.resulting_archive_state,
                archive_reason: record.archive_reason,
                retention_state: self.retention_state_for_record(record),
                lifecycle_state: self.lifecycle_state_for_record(record),
                current_tier: self.current_tier_for_record(record),
                partial_restore: record.partial_restore,
                audit_kind: ForgettingEngine.audit_event_kind(&record.action).as_str(),
            })
            .collect();

        ForgettingSummary {
            evaluated: self.processed,
            archived: self.archived,
            demoted: self.demoted,
            restored: self.restored,
            policy_deleted: self.policy_deleted,
            skipped: self.skipped,
            review_required: self
                .audit_records
                .iter()
                .filter(|record| record.reason == "needs_review")
                .count() as u32,
            audit_records,
        }
    }

    fn retention_state_for_record(&self, record: &ForgettingAuditRecord) -> &'static str {
        match record.reason {
            "legal_hold_protected" => "legal_hold",
            "pinned_protected" => "pinned",
            "last_authoritative_evidence" => "authoritative_evidence",
            _ => "normal",
        }
    }

    fn lifecycle_state_for_record(&self, record: &ForgettingAuditRecord) -> &'static str {
        match record.reason {
            "archived_requires_restore_path" => LifecycleState::Archived.as_str(),
            "below_demotion_threshold"
            | "demotion_requires_dormant_lifecycle"
            | "demotion_requires_idle_days" => LifecycleState::Dormant.as_str(),
            _ => LifecycleState::Active.as_str(),
        }
    }

    fn current_tier_for_record(&self, record: &ForgettingAuditRecord) -> &'static str {
        match record.reason {
            "already_cold_canonical" | "archived_requires_restore_path" => {
                TierOwnership::Cold.as_str()
            }
            _ => TierOwnership::Hot.as_str(),
        }
    }

    fn disposition_for_record(&self, record: &ForgettingAuditRecord) -> &'static str {
        match record.action {
            ForgettingAction::Archive | ForgettingAction::Demote { .. } => "eligible",
            ForgettingAction::Restore { .. } | ForgettingAction::PolicyDelete => "explicit",
            ForgettingAction::Skip if record.reason == "needs_review" => "review",
            ForgettingAction::Skip => "ineligible",
        }
    }

    /// Builds an operator-facing explain output from the audit trail.
    pub fn explain(&self) -> ForgettingExplainOutput {
        let entries: Vec<ForgettingExplainEntry> = self
            .audit_records
            .iter()
            .map(|record| {
                let (explanation, threshold_exceeded) = match record.action {
                    ForgettingAction::Archive => (
                        format!(
                            "Memory {} scored {} which is below the forget threshold {} — archived by utility forgetting; explicit restore is required",
                            record.memory_id.0,
                            record.current_score,
                            record.forget_threshold,
                        ),
                        "forget_threshold",
                    ),
                    ForgettingAction::Demote {
                        from_tier,
                        to_tier,
                    } => (
                        format!(
                            "Memory {} scored {} which is between forget threshold {} and demotion threshold {} — demoted from {} to {} while remaining active",
                            record.memory_id.0,
                            record.current_score,
                            record.forget_threshold,
                            record.demotion_threshold,
                            from_tier,
                            to_tier,
                        ),
                        "demotion_threshold",
                    ),
                    ForgettingAction::Restore {
                        from_tier,
                        to_tier,
                        partial,
                    } => (
                        format!(
                            "Memory {} was explicitly restored from {} to {}{}",
                            record.memory_id.0,
                            from_tier,
                            to_tier,
                            if partial {
                                " with degraded/partial fidelity"
                            } else {
                                ""
                            }
                        ),
                        "explicit_restore",
                    ),
                    ForgettingAction::PolicyDelete => (
                        format!(
                            "Memory {} was permanently deleted through an explicit policy-deletion path",
                            record.memory_id.0,
                        ),
                        "policy_delete",
                    ),
                    ForgettingAction::Skip => (
                        format!(
                            "Memory {} scored {} and remained {} — no lifecycle action taken ({})",
                            record.memory_id.0,
                            record.current_score,
                            if record.current_score < record.demotion_threshold {
                                "protected"
                            } else {
                                "above threshold"
                            },
                            record.reason,
                        ),
                        match record.reason {
                            "needs_review" => "review_threshold",
                            "legal_hold_protected"
                            | "pinned_protected"
                            | "last_authoritative_evidence"
                            | "archived_requires_restore_path"
                            | "demotion_requires_idle_days" => "eligibility_guard",
                            _ => "none",
                        },
                    ),
                };
                ForgettingExplainEntry {
                    memory_id: record.memory_id,
                    explanation,
                    action: record.action,
                    current_score: record.current_score,
                    threshold_exceeded,
                    reason_code: record.reason,
                    disposition: self.disposition_for_record(record),
                    reversibility: record.reversibility,
                    policy_surface: record.policy_surface,
                    source_tier: record.source_tier,
                    target_tier: record.target_tier,
                    retention_state: Some(self.retention_state_for_record(record)),
                    lifecycle_state: Some(self.lifecycle_state_for_record(record)),
                    current_tier: Some(self.current_tier_for_record(record)),
                    resulting_archive_state: record.resulting_archive_state,
                    partial_restore: record.partial_restore,
                    audit_kind: ForgettingEngine.audit_event_kind(&record.action).as_str(),
                    tick: record.tick,
                }
            })
            .collect();

        let mut reversible_count = 0u32;
        let mut restore_required_count = 0u32;
        let mut irreversible_count = 0u32;
        let mut no_action_count = 0u32;
        let mut review_count = 0u32;
        for record in &self.audit_records {
            match record.reversibility {
                Reversibility::Reversible => reversible_count += 1,
                Reversibility::RestoreRequired => restore_required_count += 1,
                Reversibility::Irreversible => irreversible_count += 1,
                Reversibility::NoAction => no_action_count += 1,
            }
            if record.reason == "needs_review" {
                review_count += 1;
            }
        }

        ForgettingExplainOutput {
            namespace: self.namespace.clone(),
            forget_threshold: self.policy.forget_score_threshold,
            demotion_threshold: self.policy.demotion_score_threshold,
            evaluated: self.processed,
            entries,
            reversible_count,
            restore_required_count,
            irreversible_count,
            no_action_count,
            review_count,
        }
    }

    /// Builds a reversibility summary for operator review.
    pub fn reversibility_summary(&self) -> ReversibilitySummary {
        let mut reversible = 0u32;
        let mut restore_required = 0u32;
        let mut irreversible = 0u32;
        let mut no_action = 0u32;
        let mut review_required = 0u32;
        for record in &self.audit_records {
            match record.reversibility {
                Reversibility::Reversible => reversible += 1,
                Reversibility::RestoreRequired => restore_required += 1,
                Reversibility::Irreversible => irreversible += 1,
                Reversibility::NoAction => no_action += 1,
            }
            if record.reason == "needs_review" {
                review_required += 1;
            }
        }
        ReversibilitySummary {
            total: self.audit_records.len() as u32,
            reversible,
            restore_required,
            irreversible,
            no_action,
            review_required,
        }
    }
}

impl MaintenanceOperation for ForgettingRun {
    type Summary = ForgettingSummary;

    fn poll_step(&mut self) -> MaintenanceStep<Self::Summary> {
        if self.completed || self.processed >= self.total {
            self.completed = true;
            return MaintenanceStep::Completed(self.build_summary());
        }

        let batch_size = (self.policy.batch_size as u32).max(1);
        let batch_end = (self.processed + batch_size).min(self.total);
        let start = self.processed as usize;
        let end = batch_end as usize;

        let engine = ForgettingEngine;
        for candidate in &self.candidates[start..end] {
            let action = engine.evaluate_candidate(candidate, &self.policy);
            let reason = candidate
                .eligibility_score
                .as_ref()
                .map(|score| score.disposition_reason)
                .unwrap_or_else(|| engine.action_reason(&action));
            let reversibility = engine.evaluate_reversibility(&action, &self.policy);
            let policy_surface = engine.action_policy_surface(&action);
            let (prior_archive_state, resulting_archive_state) =
                engine.archive_state_transition(&action);
            let archive_reason =
                matches!(action, ForgettingAction::Archive).then_some("utility_forgetting_archive");
            let partial_restore = matches!(action, ForgettingAction::Restore { partial: true, .. });
            let (source_tier, target_tier) = match action {
                ForgettingAction::Demote { from_tier, to_tier }
                | ForgettingAction::Restore {
                    from_tier, to_tier, ..
                } => (Some(from_tier), Some(to_tier)),
                _ => (None, None),
            };

            match action {
                ForgettingAction::Archive => self.archived += 1,
                ForgettingAction::Demote { .. } => self.demoted += 1,
                ForgettingAction::Restore { .. } => self.restored += 1,
                ForgettingAction::PolicyDelete => self.policy_deleted += 1,
                ForgettingAction::Skip => self.skipped += 1,
            }

            self.decisions.push(ForgettingDecision {
                memory_id: candidate.memory_id,
                action,
                current_score: candidate.current_score,
                reason,
            });
            self.audit_records.push(ForgettingAuditRecord {
                memory_id: candidate.memory_id,
                action,
                current_score: candidate.current_score,
                forget_threshold: self.policy.forget_score_threshold,
                demotion_threshold: self.policy.demotion_score_threshold,
                reason,
                reversibility,
                source_tier,
                target_tier,
                prior_archive_state,
                resulting_archive_state,
                archive_reason,
                partial_restore,
                policy_surface,
                tick: self.current_tick,
            });
        }
        self.processed = batch_end;
        self.durable_token = DurableStateToken(self.processed as u64);

        if self.processed >= self.total {
            self.completed = true;
            MaintenanceStep::Completed(self.build_summary())
        } else {
            MaintenanceStep::Pending(MaintenanceProgress::new(self.processed, self.total))
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

// ── Eligibility scoring ──────────────────────────────────────────────────────

/// Governance and lifecycle constraints that may block demotion or archival.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct EligibilityGuards {
    /// Retention class or equivalent summary for explain and audit surfaces.
    pub retention_state: &'static str,
    /// Whether a legal, compliance, or deletion hold is active.
    pub legal_hold: bool,
    /// Whether the memory is pinned against utility-driven demotion or archival.
    pub pinned: bool,
    /// Whether this is the last authoritative evidence that must not disappear silently.
    pub last_authoritative_evidence: bool,
    /// Current lifecycle state that the forgetting lane is evaluating.
    pub lifecycle_state: LifecycleState,
    /// Current tier ownership before any reversible demotion happens.
    pub current_tier: TierOwnership,
}

impl Default for EligibilityGuards {
    fn default() -> Self {
        Self {
            retention_state: "normal",
            legal_hold: false,
            pinned: false,
            last_authoritative_evidence: false,
            lifecycle_state: LifecycleState::Active,
            current_tier: TierOwnership::Hot,
        }
    }
}

/// Multi-factor input for eligibility scoring.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EligibilityDisposition {
    /// Candidate may be demoted or archived according to the composite score.
    Eligible,
    /// Candidate is near the threshold and should be reviewed or reinforced, not changed silently.
    Review,
    /// Candidate is protected from utility-driven demotion or archival.
    Ineligible,
}

impl EligibilityDisposition {
    /// Returns the stable machine-readable label.
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Eligible => "eligible",
            Self::Review => "review",
            Self::Ineligible => "ineligible",
        }
    }
}

/// Multi-factor input for eligibility scoring.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct EligibilityFactors {
    /// Effective strength normalized to 0.0..=1.0 (1.0 = strongest).
    pub effective_strength: f32,
    /// Recency: 1.0 = just accessed, 0.0 = never accessed or very old.
    pub recency: f32,
    /// Access frequency normalized (1.0 = very frequent, 0.0 = never accessed).
    pub access_frequency: f32,
    /// Whether this memory is in a contradiction neighborhood.
    pub in_contradiction: bool,
    /// Emotional arousal (higher = more resistant to forgetting).
    pub emotional_arousal: f32,
    /// Whether the memory bypasses decay (emotionally tagged).
    pub bypass_decay: bool,
    /// Whether the item has been idle long enough for reversible demotion eligibility.
    pub idle_days: u32,
    /// Governance and lifecycle guards that can block lifecycle changes.
    pub guards: EligibilityGuards,
}

impl EligibilityFactors {
    /// Builds default factors for a memory with the given strength.
    pub fn with_strength(strength: f32) -> Self {
        Self {
            effective_strength: strength,
            recency: 0.5,
            access_frequency: 0.5,
            in_contradiction: false,
            emotional_arousal: 0.0,
            bypass_decay: false,
            idle_days: 0,
            guards: EligibilityGuards::default(),
        }
    }
}

/// Computed eligibility score with component breakdown.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct EligibilityScore {
    /// Composite score 0..1000 (lower = more eligible for forgetting).
    pub composite_score: u16,
    /// Strength component contribution.
    pub strength_component: f32,
    /// Recency component contribution.
    pub recency_component: f32,
    /// Access frequency component contribution.
    pub access_component: f32,
    /// Contradiction penalty applied.
    pub contradiction_penalty: f32,
    /// Emotional resistance bonus applied.
    pub emotional_bonus: f32,
    /// Idle pressure applied once the demotion idle threshold is exceeded.
    pub idle_pressure: f32,
    /// Review buffer applied for near-threshold memories.
    pub review_buffer: u16,
    /// Whether the memory is archive-eligible on score alone.
    pub archive_eligible: bool,
    /// Whether the memory is demotion-eligible on score and idle pressure.
    pub demotion_eligible: bool,
    /// Eligibility disposition after lifecycle and policy guards are applied.
    pub disposition: EligibilityDisposition,
    /// Machine-readable reason describing the disposition.
    pub disposition_reason: &'static str,
    /// Retention and lifecycle state summary used during evaluation.
    pub retention_state: &'static str,
    /// Lifecycle state used during evaluation.
    pub lifecycle_state: LifecycleState,
    /// Current tier ownership used during evaluation.
    pub current_tier: TierOwnership,
}

// ── Engine ───────────────────────────────────────────────────────────────────

/// Canonical forgetting engine owned by the core crate.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct ForgettingEngine;

impl ForgettingEngine {
    /// Returns the stable component identifier.
    pub const fn component_name(&self) -> &'static str {
        "engine.forgetting"
    }

    /// Creates a bounded forgetting run for the given namespace with placeholder candidates.
    pub fn create_run(
        &self,
        namespace: NamespaceId,
        policy: ForgettingPolicy,
        estimated_candidates: u32,
    ) -> ForgettingRun {
        let candidates: Vec<ForgettingCandidate> = (1..=estimated_candidates)
            .map(|i| ForgettingCandidate {
                memory_id: MemoryId(i as u64),
                current_score: 500,
                eligibility_score: None,
            })
            .collect();
        ForgettingRun::new(namespace, policy, candidates)
    }

    /// Evaluates a single memory for utility forgetting or demotion.
    pub fn evaluate_memory(
        &self,
        _memory_id: MemoryId,
        current_score: u16,
        policy: &ForgettingPolicy,
    ) -> ForgettingAction {
        if current_score < policy.forget_score_threshold {
            ForgettingAction::Archive
        } else if current_score < policy.demotion_score_threshold {
            ForgettingAction::Demote {
                from_tier: "tier1",
                to_tier: "tier2",
            }
        } else {
            ForgettingAction::Skip
        }
    }

    /// Returns the stable demotion route for reversible tier movement.
    pub const fn reversible_demote_action(&self, current_tier: TierOwnership) -> ForgettingAction {
        match current_tier {
            TierOwnership::Hot => ForgettingAction::Demote {
                from_tier: "tier1",
                to_tier: "tier2",
            },
            TierOwnership::Cold => ForgettingAction::Skip,
        }
    }

    /// Evaluates a bounded forgetting candidate.
    pub fn evaluate_candidate(
        &self,
        candidate: &ForgettingCandidate,
        policy: &ForgettingPolicy,
    ) -> ForgettingAction {
        if let Some(score) = candidate.eligibility_score {
            return match score.disposition {
                EligibilityDisposition::Ineligible | EligibilityDisposition::Review => {
                    ForgettingAction::Skip
                }
                EligibilityDisposition::Eligible if score.archive_eligible => {
                    ForgettingAction::Archive
                }
                EligibilityDisposition::Eligible if score.demotion_eligible => {
                    self.reversible_demote_action(score.current_tier)
                }
                EligibilityDisposition::Eligible => ForgettingAction::Skip,
            };
        }

        self.evaluate_memory(candidate.memory_id, candidate.current_score, policy)
    }

    /// Builds the explicit restore action. Ordinary recall must not infer this automatically.
    pub const fn plan_restore(&self, partial: bool) -> ForgettingAction {
        ForgettingAction::Restore {
            from_tier: "cold",
            to_tier: "tier1",
            partial,
        }
    }

    /// Builds an explicit policy-deletion action when policy allows it.
    pub fn plan_policy_delete(
        &self,
        policy: &ForgettingPolicy,
    ) -> Result<ForgettingAction, &'static str> {
        if policy.allow_hard_delete {
            Ok(ForgettingAction::PolicyDelete)
        } else {
            Err("policy_delete_not_enabled")
        }
    }

    /// Evaluates a single lifecycle action and returns its reversibility classification.
    pub fn evaluate_reversibility(
        &self,
        action: &ForgettingAction,
        _policy: &ForgettingPolicy,
    ) -> Reversibility {
        match action {
            ForgettingAction::Archive => Reversibility::RestoreRequired,
            ForgettingAction::Demote { .. } | ForgettingAction::Restore { .. } => {
                Reversibility::Reversible
            }
            ForgettingAction::PolicyDelete => Reversibility::Irreversible,
            ForgettingAction::Skip => Reversibility::NoAction,
        }
    }

    /// Returns the stable reason code for a forgetting action.
    pub const fn action_reason(&self, action: &ForgettingAction) -> &'static str {
        match action {
            ForgettingAction::Archive => "below_forget_threshold",
            ForgettingAction::Demote { .. } => "below_demotion_threshold",
            ForgettingAction::Restore { .. } => "explicit_restore",
            ForgettingAction::PolicyDelete => "explicit_policy_delete",
            ForgettingAction::Skip => "ineligible_or_retained",
        }
    }

    /// Distinguishes the utility forgetting, restore, and policy-deletion surfaces.
    pub const fn action_policy_surface(&self, action: &ForgettingAction) -> &'static str {
        match action {
            ForgettingAction::Archive | ForgettingAction::Demote { .. } => "utility_forgetting",
            ForgettingAction::Restore { .. } => "explicit_restore",
            ForgettingAction::PolicyDelete => "policy_delete",
            ForgettingAction::Skip => "none",
        }
    }

    /// Returns the cold/archive lifecycle transition implied by an action.
    pub const fn archive_state_transition(
        &self,
        action: &ForgettingAction,
    ) -> (Option<&'static str>, Option<&'static str>) {
        match action {
            ForgettingAction::Archive => (None, Some(ArchiveState::Archived.as_str())),
            ForgettingAction::Restore { .. } => (Some(ArchiveState::Archived.as_str()), None),
            ForgettingAction::PolicyDelete
            | ForgettingAction::Demote { .. }
            | ForgettingAction::Skip => (None, None),
        }
    }

    /// Returns the audit event kind emitted for one forgetting action.
    pub const fn audit_event_kind(&self, action: &ForgettingAction) -> AuditEventKind {
        match action {
            ForgettingAction::Archive
            | ForgettingAction::Demote { .. }
            | ForgettingAction::Restore { .. }
            | ForgettingAction::PolicyDelete
            | ForgettingAction::Skip => AuditEventKind::MaintenanceForgettingEvaluated,
        }
    }

    /// Builds append-only audit entries from a completed forgetting run.
    pub fn append_only_audit_entries(&self, run: &ForgettingRun) -> Vec<AuditLogEntry> {
        run.audit_records()
            .iter()
            .map(|record| {
                let detail = format!(
                    "forgetting decision at tick {} (action={}, reason_code={}, reversibility={}, policy_surface={}, score={}, forget_threshold={}, demotion_threshold={}, source_tier={}, target_tier={}, prior_archive_state={}, resulting_archive_state={}, archive_reason={}, partial_restore={})",
                    record.tick,
                    record.action.as_str(),
                    record.reason,
                    record.reversibility.as_str(),
                    record.policy_surface,
                    record.current_score,
                    record.forget_threshold,
                    record.demotion_threshold,
                    record.source_tier.unwrap_or("none"),
                    record.target_tier.unwrap_or("none"),
                    record.prior_archive_state.unwrap_or("none"),
                    record.resulting_archive_state.unwrap_or("none"),
                    record.archive_reason.unwrap_or("none"),
                    record.partial_restore,
                );

                AuditLogEntry::new(
                    self.audit_event_kind(&record.action).category(),
                    self.audit_event_kind(&record.action),
                    run.namespace().clone(),
                    "forgetting_run",
                    detail,
                )
                .with_memory_id(record.memory_id)
                .with_tick(record.tick)
                .with_related_run(format!("forgetting-tick-{}", record.tick))
            })
            .collect()
    }

    /// Builds an explain output from a completed forgetting run.
    pub fn explain_run(&self, run: &ForgettingRun) -> ForgettingExplainOutput {
        run.explain()
    }

    /// Builds a reversibility summary from a completed forgetting run.
    pub fn reversibility_summary(&self, run: &ForgettingRun) -> ReversibilitySummary {
        run.reversibility_summary()
    }

    fn action_for_eligibility_score(&self, score: &EligibilityScore) -> ForgettingAction {
        match score.disposition {
            EligibilityDisposition::Eligible if score.archive_eligible => ForgettingAction::Archive,
            EligibilityDisposition::Eligible if score.demotion_eligible => {
                self.reversible_demote_action(score.current_tier)
            }
            EligibilityDisposition::Eligible
            | EligibilityDisposition::Review
            | EligibilityDisposition::Ineligible => ForgettingAction::Skip,
        }
    }

    /// Computes a multi-factor eligibility score for a memory using the default policy.
    pub fn compute_eligibility_score(&self, factors: &EligibilityFactors) -> EligibilityScore {
        self.compute_eligibility_score_with_policy(factors, &ForgettingPolicy::default())
    }

    /// Computes a multi-factor eligibility score for a memory under an explicit forgetting policy.
    pub fn compute_eligibility_score_with_policy(
        &self,
        factors: &EligibilityFactors,
        policy: &ForgettingPolicy,
    ) -> EligibilityScore {
        const REVIEW_BUFFER: u16 = 10;

        let strength_component = factors.effective_strength.clamp(0.0, 1.0);
        let recency_component = factors.recency.clamp(0.0, 1.0);
        let access_component = factors.access_frequency.clamp(0.0, 1.0);

        let base = strength_component * 0.5 + recency_component * 0.25 + access_component * 0.25;
        let contradiction_penalty = if factors.in_contradiction { 0.1 } else { 0.0 };
        let emotional_bonus = if factors.bypass_decay {
            0.3
        } else {
            factors.emotional_arousal.clamp(0.0, 1.0) * 0.15
        };
        let idle_pressure = if factors.idle_days >= policy.idle_days_before_demotion {
            0.1
        } else {
            0.0
        };

        let adjusted =
            (base - contradiction_penalty - idle_pressure + emotional_bonus).clamp(0.0, 1.0);
        let composite_score = (adjusted * 1000.0) as u16;
        let archive_eligible = composite_score < policy.forget_score_threshold;
        let demotion_score_eligible = composite_score < policy.demotion_score_threshold
            && composite_score >= policy.forget_score_threshold;
        let demotion_eligible = demotion_score_eligible
            && factors.guards.current_tier == TierOwnership::Hot
            && factors.guards.lifecycle_state == LifecycleState::Dormant
            && factors.idle_days >= policy.idle_days_before_demotion;
        let review_floor = policy.forget_score_threshold.saturating_sub(REVIEW_BUFFER);
        let review_ceiling = policy.forget_score_threshold.saturating_add(REVIEW_BUFFER);
        let near_forget_boundary =
            composite_score >= review_floor && composite_score <= review_ceiling;

        let (disposition, disposition_reason) = if factors.guards.legal_hold {
            (EligibilityDisposition::Ineligible, "legal_hold_protected")
        } else if factors.guards.pinned {
            (EligibilityDisposition::Ineligible, "pinned_protected")
        } else if factors.guards.lifecycle_state == LifecycleState::Archived {
            (
                EligibilityDisposition::Ineligible,
                "archived_requires_restore_path",
            )
        } else if archive_eligible && factors.guards.last_authoritative_evidence {
            (
                EligibilityDisposition::Ineligible,
                "last_authoritative_evidence",
            )
        } else if near_forget_boundary {
            (EligibilityDisposition::Review, "needs_review")
        } else if archive_eligible {
            (EligibilityDisposition::Eligible, "below_forget_threshold")
        } else if demotion_score_eligible {
            if factors.guards.current_tier != TierOwnership::Hot {
                (EligibilityDisposition::Ineligible, "already_cold_canonical")
            } else if factors.guards.lifecycle_state != LifecycleState::Dormant {
                (
                    EligibilityDisposition::Ineligible,
                    "demotion_requires_dormant_lifecycle",
                )
            } else if factors.idle_days < policy.idle_days_before_demotion {
                (
                    EligibilityDisposition::Ineligible,
                    "demotion_requires_idle_days",
                )
            } else {
                (EligibilityDisposition::Eligible, "below_demotion_threshold")
            }
        } else {
            (EligibilityDisposition::Ineligible, "above_threshold")
        };

        EligibilityScore {
            composite_score,
            strength_component,
            recency_component,
            access_component,
            contradiction_penalty,
            emotional_bonus,
            idle_pressure,
            review_buffer: REVIEW_BUFFER,
            archive_eligible,
            demotion_eligible,
            disposition,
            disposition_reason,
            retention_state: factors.guards.retention_state,
            lifecycle_state: factors.guards.lifecycle_state,
            current_tier: factors.guards.current_tier,
        }
    }

    /// Evaluates a memory using multi-factor eligibility scoring.
    pub fn evaluate_memory_with_factors(
        &self,
        _memory_id: MemoryId,
        factors: &EligibilityFactors,
        policy: &ForgettingPolicy,
    ) -> (ForgettingAction, EligibilityScore) {
        let score = self.compute_eligibility_score_with_policy(factors, policy);
        let action = self.action_for_eligibility_score(&score);
        (action, score)
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
    fn evaluate_memory_respects_archive_demote_and_skip_thresholds() {
        let engine = ForgettingEngine;
        let policy = ForgettingPolicy::default();

        assert_eq!(
            engine.evaluate_memory(MemoryId(1), 30, &policy),
            ForgettingAction::Archive,
        );
        assert!(matches!(
            engine.evaluate_memory(MemoryId(2), 100, &policy),
            ForgettingAction::Demote { .. },
        ));
        assert_eq!(
            engine.evaluate_memory(MemoryId(3), 500, &policy),
            ForgettingAction::Skip,
        );
    }

    #[test]
    fn forgetting_run_completes_in_bounded_polls() {
        let engine = ForgettingEngine;
        let run = engine.create_run(
            ns("test"),
            ForgettingPolicy {
                batch_size: 1,
                ..Default::default()
            },
            2,
        );
        let mut handle = MaintenanceJobHandle::new(run, 3);

        let first = handle.poll();
        let MaintenanceJobState::Running { progress } = first.state else {
            panic!("expected running state for first bounded poll");
        };
        assert_eq!(progress, Some(MaintenanceProgress::new(1, 2)));

        let second = handle.poll();
        let MaintenanceJobState::Completed(summary) = second.state else {
            panic!("expected completion on second bounded poll");
        };
        assert_eq!(summary.evaluated, 2);
        assert_eq!(summary.skipped, 2);
        assert_eq!(summary.archived, 0);
    }

    #[test]
    fn forgetting_run_records_archive_demote_and_skip() {
        let candidates = vec![
            ForgettingCandidate {
                memory_id: MemoryId(1),
                current_score: 30,
                eligibility_score: None,
            },
            ForgettingCandidate {
                memory_id: MemoryId(2),
                current_score: 100,
                eligibility_score: None,
            },
            ForgettingCandidate {
                memory_id: MemoryId(3),
                current_score: 500,
                eligibility_score: None,
            },
        ];
        let run =
            ForgettingRun::new(ns("audit"), ForgettingPolicy::default(), candidates).with_tick(42);
        let mut handle = MaintenanceJobHandle::new(run, 5);

        let snapshot = handle.poll();
        let MaintenanceJobState::Completed(summary) = snapshot.state else {
            panic!("expected forgetting run to complete");
        };

        assert_eq!(summary.archived, 1);
        assert_eq!(summary.demoted, 1);
        assert_eq!(summary.restored, 0);
        assert_eq!(summary.policy_deleted, 0);
        assert_eq!(summary.skipped, 1);
        assert_eq!(summary.review_required, 0);
        assert_eq!(summary.audit_records.len(), 3);
        assert_eq!(summary.audit_records[0].action, ForgettingAction::Archive);
        assert_eq!(
            summary.audit_records[0].audit_kind,
            "maintenance_forgetting_evaluated"
        );

        let audit = handle.operation().audit_records();
        assert_eq!(audit.len(), 3);
        assert_eq!(audit[0].action, ForgettingAction::Archive);
        assert_eq!(audit[0].reversibility, Reversibility::RestoreRequired);
        assert_eq!(audit[0].policy_surface, "utility_forgetting");
        assert_eq!(audit[0].resulting_archive_state, Some("archived"));
        assert_eq!(audit[0].archive_reason, Some("utility_forgetting_archive"));
        assert_eq!(audit[0].tick, 42);

        assert_eq!(audit[1].source_tier, Some("tier1"));
        assert_eq!(audit[1].target_tier, Some("tier2"));
        assert_eq!(audit[1].resulting_archive_state, None);
        assert_eq!(audit[2].action, ForgettingAction::Skip);
    }

    #[test]
    fn explain_output_surfaces_archive_restore_requirement() {
        let candidates = vec![ForgettingCandidate {
            memory_id: MemoryId(1),
            current_score: 30,
            eligibility_score: None,
        }];
        let run = ForgettingRun::new(ns("explain"), ForgettingPolicy::default(), candidates);
        let mut handle = MaintenanceJobHandle::new(run, 5);
        handle.poll();

        let explain = handle.operation().explain();
        assert_eq!(explain.entries.len(), 1);
        assert_eq!(explain.restore_required_count, 1);
        assert_eq!(explain.entries[0].action, ForgettingAction::Archive);
        assert_eq!(explain.entries[0].reason_code, "below_forget_threshold");
        assert_eq!(explain.entries[0].disposition, "eligible");
        assert_eq!(explain.entries[0].policy_surface, "utility_forgetting");
        assert_eq!(
            explain.entries[0].audit_kind,
            "maintenance_forgetting_evaluated"
        );
        assert_eq!(explain.entries[0].resulting_archive_state, Some("archived"));
        assert!(explain.entries[0]
            .explanation
            .contains("explicit restore is required"));
    }

    #[test]
    fn explicit_restore_remains_separate_from_utility_forgetting() {
        let engine = ForgettingEngine;
        let policy = ForgettingPolicy::default();
        let action = engine.plan_restore(false);

        assert_eq!(action.as_str(), "restore");
        assert_eq!(engine.action_policy_surface(&action), "explicit_restore");
        assert_eq!(
            engine.evaluate_reversibility(&action, &policy),
            Reversibility::Reversible
        );
        assert_eq!(
            engine.archive_state_transition(&action),
            (Some("archived"), None)
        );
    }

    #[test]
    fn partial_restore_is_explicit_and_degraded() {
        let engine = ForgettingEngine;
        let action = engine.plan_restore(true);
        let ForgettingAction::Restore { partial, .. } = action else {
            panic!("expected restore action");
        };
        assert!(partial);
    }

    #[test]
    fn policy_delete_requires_explicit_policy_enable() {
        let engine = ForgettingEngine;
        let disabled = ForgettingPolicy::default();
        let enabled = ForgettingPolicy {
            allow_hard_delete: true,
            ..Default::default()
        };

        assert_eq!(
            engine.plan_policy_delete(&disabled),
            Err("policy_delete_not_enabled")
        );
        assert_eq!(
            engine.plan_policy_delete(&enabled),
            Ok(ForgettingAction::PolicyDelete)
        );
        assert_eq!(
            engine.evaluate_reversibility(&ForgettingAction::PolicyDelete, &enabled),
            Reversibility::Irreversible
        );
        assert_eq!(
            engine.action_policy_surface(&ForgettingAction::PolicyDelete),
            "policy_delete"
        );
    }

    #[test]
    fn reversibility_labels_are_stable() {
        assert_eq!(Reversibility::Reversible.as_str(), "reversible");
        assert_eq!(Reversibility::RestoreRequired.as_str(), "restore_required");
        assert_eq!(Reversibility::Irreversible.as_str(), "irreversible");
        assert_eq!(Reversibility::NoAction.as_str(), "no_action");
    }

    #[test]
    fn reversibility_summary_counts_actions_correctly() {
        let candidates = vec![
            ForgettingCandidate {
                memory_id: MemoryId(1),
                current_score: 30,
                eligibility_score: None,
            },
            ForgettingCandidate {
                memory_id: MemoryId(2),
                current_score: 100,
                eligibility_score: None,
            },
            ForgettingCandidate {
                memory_id: MemoryId(3),
                current_score: 500,
                eligibility_score: None,
            },
        ];
        let run = ForgettingRun::new(ns("sum"), ForgettingPolicy::default(), candidates);
        let mut handle = MaintenanceJobHandle::new(run, 5);
        handle.poll();

        let summary = handle.operation().reversibility_summary();
        assert_eq!(summary.total, 3);
        assert_eq!(summary.reversible, 1);
        assert_eq!(summary.restore_required, 1);
        assert_eq!(summary.irreversible, 0);
        assert_eq!(summary.no_action, 1);
        assert_eq!(summary.review_required, 0);
        assert!(!summary.has_irreversible());
    }

    #[test]
    fn eligibility_score_emotional_bypass_resists_forgetting() {
        let engine = ForgettingEngine;
        let neutral = EligibilityFactors::with_strength(0.2);
        let emotional = EligibilityFactors {
            effective_strength: 0.2,
            bypass_decay: true,
            emotional_arousal: 0.9,
            ..EligibilityFactors::with_strength(0.2)
        };

        let score_neutral = engine.compute_eligibility_score(&neutral);
        let score_emotional = engine.compute_eligibility_score(&emotional);

        assert!(score_emotional.composite_score > score_neutral.composite_score);
    }

    #[test]
    fn evaluate_memory_with_factors_archives_very_weak_memories() {
        let engine = ForgettingEngine;
        let policy = ForgettingPolicy::default();
        let weak = EligibilityFactors {
            effective_strength: 0.0,
            recency: 0.0,
            access_frequency: 0.0,
            ..EligibilityFactors::with_strength(0.0)
        };

        let (action, score) = engine.evaluate_memory_with_factors(MemoryId(1), &weak, &policy);

        assert_eq!(action, ForgettingAction::Archive);
        assert!(score.composite_score < policy.forget_score_threshold);
        assert_eq!(score.disposition, EligibilityDisposition::Eligible);
        assert_eq!(score.disposition_reason, "below_forget_threshold");
    }

    #[test]
    fn demotion_requires_idle_days_and_dormant_lifecycle() {
        let engine = ForgettingEngine;
        let policy = ForgettingPolicy::default();
        let factors = EligibilityFactors {
            effective_strength: 0.1,
            recency: 0.45,
            access_frequency: 0.1,
            idle_days: policy.idle_days_before_demotion.saturating_sub(1),
            guards: EligibilityGuards {
                lifecycle_state: LifecycleState::Dormant,
                current_tier: TierOwnership::Hot,
                ..EligibilityGuards::default()
            },
            ..EligibilityFactors::with_strength(0.1)
        };

        let (action, score) = engine.evaluate_memory_with_factors(MemoryId(7), &factors, &policy);

        assert_eq!(action, ForgettingAction::Skip);
        assert_eq!(score.disposition, EligibilityDisposition::Ineligible);
        assert_eq!(score.disposition_reason, "demotion_requires_idle_days");
        assert!(!score.archive_eligible);
    }

    #[test]
    fn dormant_hot_memory_becomes_reversibly_demotable_after_idle_threshold() {
        let engine = ForgettingEngine;
        let policy = ForgettingPolicy::default();
        let factors = EligibilityFactors {
            effective_strength: 0.1,
            recency: 0.45,
            access_frequency: 0.1,
            idle_days: policy.idle_days_before_demotion,
            guards: EligibilityGuards {
                lifecycle_state: LifecycleState::Dormant,
                current_tier: TierOwnership::Hot,
                ..EligibilityGuards::default()
            },
            ..EligibilityFactors::with_strength(0.1)
        };

        let (action, score) = engine.evaluate_memory_with_factors(MemoryId(8), &factors, &policy);

        assert!(matches!(
            action,
            ForgettingAction::Demote {
                from_tier: "tier1",
                to_tier: "tier2"
            }
        ));
        assert_eq!(score.disposition, EligibilityDisposition::Eligible);
        assert_eq!(score.disposition_reason, "below_demotion_threshold");
        assert!(score.demotion_eligible);
    }

    #[test]
    fn legal_hold_and_pinned_items_are_ineligible_even_with_low_scores() {
        let engine = ForgettingEngine;
        let policy = ForgettingPolicy::default();
        let held = EligibilityFactors {
            effective_strength: 0.0,
            recency: 0.0,
            access_frequency: 0.0,
            idle_days: policy.idle_days_before_demotion,
            guards: EligibilityGuards {
                legal_hold: true,
                ..EligibilityGuards::default()
            },
            ..EligibilityFactors::with_strength(0.0)
        };
        let pinned = EligibilityFactors {
            effective_strength: 0.0,
            recency: 0.0,
            access_frequency: 0.0,
            idle_days: policy.idle_days_before_demotion,
            guards: EligibilityGuards {
                pinned: true,
                ..EligibilityGuards::default()
            },
            ..EligibilityFactors::with_strength(0.0)
        };

        let (held_action, held_score) =
            engine.evaluate_memory_with_factors(MemoryId(9), &held, &policy);
        let (pinned_action, pinned_score) =
            engine.evaluate_memory_with_factors(MemoryId(10), &pinned, &policy);

        assert_eq!(held_action, ForgettingAction::Skip);
        assert_eq!(held_score.disposition, EligibilityDisposition::Ineligible);
        assert_eq!(held_score.disposition_reason, "legal_hold_protected");
        assert_eq!(pinned_action, ForgettingAction::Skip);
        assert_eq!(pinned_score.disposition, EligibilityDisposition::Ineligible);
        assert_eq!(pinned_score.disposition_reason, "pinned_protected");
    }

    #[test]
    fn last_authoritative_evidence_cannot_be_archived_by_utility_forgetting() {
        let engine = ForgettingEngine;
        let policy = ForgettingPolicy::default();
        let factors = EligibilityFactors {
            effective_strength: 0.0,
            recency: 0.0,
            access_frequency: 0.0,
            idle_days: policy.idle_days_before_demotion,
            guards: EligibilityGuards {
                last_authoritative_evidence: true,
                ..EligibilityGuards::default()
            },
            ..EligibilityFactors::with_strength(0.0)
        };

        let (action, score) = engine.evaluate_memory_with_factors(MemoryId(11), &factors, &policy);

        assert_eq!(action, ForgettingAction::Skip);
        assert_eq!(score.disposition, EligibilityDisposition::Ineligible);
        assert_eq!(score.disposition_reason, "last_authoritative_evidence");
        assert!(score.archive_eligible);
    }

    #[test]
    fn near_threshold_memories_are_marked_for_review_not_silently_demoted() {
        let engine = ForgettingEngine;
        let policy = ForgettingPolicy::default();
        let factors = EligibilityFactors {
            effective_strength: 0.1,
            recency: 0.3,
            access_frequency: 0.1,
            idle_days: policy.idle_days_before_demotion,
            guards: EligibilityGuards {
                lifecycle_state: LifecycleState::Dormant,
                current_tier: TierOwnership::Hot,
                ..EligibilityGuards::default()
            },
            ..EligibilityFactors::with_strength(0.11)
        };

        let (action, score) = engine.evaluate_memory_with_factors(MemoryId(12), &factors, &policy);

        assert_eq!(action, ForgettingAction::Skip);
        assert_eq!(score.disposition, EligibilityDisposition::Review);
        assert_eq!(score.disposition_reason, "needs_review");
        assert!(
            score.composite_score
                >= policy
                    .forget_score_threshold
                    .saturating_sub(score.review_buffer)
        );
        assert!(
            score.composite_score
                <= policy
                    .forget_score_threshold
                    .saturating_add(score.review_buffer)
        );
    }

    #[test]
    fn append_only_audit_entries_emit_forgetting_taxonomy_and_reason_codes() {
        let engine = ForgettingEngine;
        let candidates = vec![ForgettingCandidate {
            memory_id: MemoryId(1),
            current_score: 30,
            eligibility_score: None,
        }];
        let run = ForgettingRun::new(ns("forget-audit"), ForgettingPolicy::default(), candidates)
            .with_tick(99);
        let mut handle = MaintenanceJobHandle::new(run, 2);
        handle.poll();

        let rows = engine.append_only_audit_entries(handle.operation());
        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0].kind, AuditEventKind::MaintenanceForgettingEvaluated);
        assert_eq!(rows[0].memory_id, Some(MemoryId(1)));
        assert_eq!(rows[0].related_run.as_deref(), Some("forgetting-tick-99"));
        assert!(rows[0].detail.contains("action=archive"));
        assert!(rows[0]
            .detail
            .contains("reason_code=below_forget_threshold"));
    }

    #[test]
    fn review_required_counts_surface_in_summary_and_reversibility() {
        let engine = ForgettingEngine;
        let policy = ForgettingPolicy::default();
        let factors = EligibilityFactors {
            effective_strength: 0.1,
            recency: 0.3,
            access_frequency: 0.1,
            idle_days: policy.idle_days_before_demotion,
            guards: EligibilityGuards {
                lifecycle_state: LifecycleState::Dormant,
                current_tier: TierOwnership::Hot,
                ..EligibilityGuards::default()
            },
            ..EligibilityFactors::with_strength(0.11)
        };
        let (_, score) = engine.evaluate_memory_with_factors(MemoryId(12), &factors, &policy);
        let run = ForgettingRun::new(
            ns("forget-review"),
            policy,
            vec![ForgettingCandidate {
                memory_id: MemoryId(12),
                current_score: score.composite_score,
                eligibility_score: Some(score),
            }],
        );
        let mut handle = MaintenanceJobHandle::new(run, 2);
        let snapshot = handle.poll();
        let MaintenanceJobState::Completed(summary) = snapshot.state else {
            panic!("expected forgetting run to complete");
        };

        assert_eq!(summary.review_required, 1);
        assert_eq!(summary.audit_records.len(), 1);
        assert_eq!(summary.audit_records[0].reason_code, "needs_review");
        assert_eq!(summary.audit_records[0].disposition, "review");

        let explain = handle.operation().explain();
        assert_eq!(explain.review_count, 1);
        assert_eq!(explain.entries[0].disposition, "review");

        let reversibility = handle.operation().reversibility_summary();
        assert_eq!(reversibility.review_required, 1);
    }
}
