use crate::api::{ForkInheritance, ForkStatus, MergeConflictOutput, MergeConflictStrategy};

/// Stable inspectable summary of one governed fork namespace.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ForkInfo {
    pub name: String,
    pub parent_namespace: String,
    pub inherit_visibility: ForkInheritance,
    pub status: ForkStatus,
    pub forked_at_tick: u64,
    pub inherited_count: usize,
    pub fork_local_procedure_count: usize,
    pub fork_working_state_count: usize,
    pub diverged: bool,
    pub divergence_basis: &'static str,
    pub isolation_semantics: &'static str,
    pub note: Option<String>,
}

/// Stable inspectable summary of one governed merge execution.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MergeReport {
    pub fork_name: String,
    pub target_namespace: String,
    pub conflict_strategy: MergeConflictStrategy,
    pub dry_run: bool,
    pub memories_merged: usize,
    pub merged_items: Vec<String>,
    pub conflicts_found: usize,
    pub conflicts_auto_resolved: usize,
    pub conflicts_pending: usize,
    pub conflict_items: Vec<MergeConflictOutput>,
    pub engrams_merged: usize,
    pub fork_status: ForkStatus,
    pub fork_local_procedure_count: usize,
    pub fork_working_state_count: usize,
    pub audit_sequences: Vec<u64>,
    pub divergence_detected: bool,
    pub divergence_basis: &'static str,
    pub isolation_semantics: &'static str,
}
