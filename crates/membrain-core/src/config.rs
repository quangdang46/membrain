/// Shared runtime budgets for the initial core bootstrap.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RuntimeConfig {
    /// Maximum Tier1 candidates before later pruning stages.
    pub tier1_candidate_budget: usize,
    /// Maximum Tier2 candidates before later pruning stages.
    pub tier2_candidate_budget: usize,
    /// Maximum number of working-memory slots tracked before eviction.
    pub working_memory_capacity: usize,
    /// Minimum attention score required to keep an item in working memory.
    pub working_memory_attention_threshold: u16,
    /// Minimum attention score required to promote an evicted item into encode.
    pub working_memory_promote_threshold: u16,
}

impl Default for RuntimeConfig {
    fn default() -> Self {
        Self {
            tier1_candidate_budget: crate::constants::DEFAULT_TIER1_CANDIDATE_BUDGET,
            tier2_candidate_budget: crate::constants::DEFAULT_TIER2_CANDIDATE_BUDGET,
            working_memory_capacity: 7,
            working_memory_attention_threshold: 200,
            working_memory_promote_threshold: 700,
        }
    }
}
