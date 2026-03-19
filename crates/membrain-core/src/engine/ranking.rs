/// Shared interface for ranking and packaging owned by `membrain-core`.
pub trait RankingRuntime {
    /// Returns whether this ranking surface packages explainable results.
    fn packages_explainable_results(&self) -> bool;
}

/// Canonical ranking engine placeholder owned by the core crate.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct RankingEngine;

impl RankingRuntime for RankingEngine {
    fn packages_explainable_results(&self) -> bool {
        true
    }
}
