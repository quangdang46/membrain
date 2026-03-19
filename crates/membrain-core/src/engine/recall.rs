use crate::config::RuntimeConfig;

/// Shared interface for Tier1 recall orchestration owned by `membrain-core`.
pub trait RecallRuntime {
    /// Returns the maximum Tier1 candidate budget this recall surface may consume.
    fn tier1_candidate_budget(&self, config: RuntimeConfig) -> usize;
}

/// Canonical recall engine placeholder owned by the core crate.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct RecallEngine;

impl RecallRuntime for RecallEngine {
    fn tier1_candidate_budget(&self, config: RuntimeConfig) -> usize {
        config.tier1_candidate_budget
    }
}
