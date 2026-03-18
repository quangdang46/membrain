/// Shared runtime budgets for the initial core bootstrap.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RuntimeConfig {
    /// Maximum Tier1 candidates before later pruning stages.
    pub tier1_candidate_budget: usize,
    /// Maximum Tier2 candidates before later pruning stages.
    pub tier2_candidate_budget: usize,
}

impl Default for RuntimeConfig {
    fn default() -> Self {
        Self {
            tier1_candidate_budget: crate::constants::DEFAULT_TIER1_CANDIDATE_BUDGET,
            tier2_candidate_budget: crate::constants::DEFAULT_TIER2_CANDIDATE_BUDGET,
        }
    }
}
