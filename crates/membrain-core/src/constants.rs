/// Maximum Tier1 candidates before later pruning stages.
pub const DEFAULT_TIER1_CANDIDATE_BUDGET: usize = 128;
/// Maximum Tier2 candidates before later pruning stages.
pub const DEFAULT_TIER2_CANDIDATE_BUDGET: usize = 5_000;
/// Default per-family cache capacity (entries).
pub const DEFAULT_CACHE_PER_FAMILY_CAPACITY: usize = 256;
/// Default prefetch queue depth (hints).
pub const DEFAULT_PREFETCH_QUEUE_CAPACITY: usize = 32;
/// Default bounded graph expansion depth.
pub const DEFAULT_GRAPH_MAX_DEPTH: u8 = 3;
/// Default bounded graph expansion node budget.
pub const DEFAULT_GRAPH_MAX_NODES: usize = 50;
/// Default minimum graph edge strength to follow.
pub const DEFAULT_GRAPH_MIN_EDGE_STRENGTH: u16 = 500;
