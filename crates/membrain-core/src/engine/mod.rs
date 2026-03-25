/// Belief-history storage and version timeline surfaces.
pub mod belief_history;
/// Confidence-interval storage and scoring input surfaces.
pub mod confidence;
/// Consolidation maintenance surfaces.
pub mod consolidation;
/// Context-budget request contract and greedy packing semantics.
pub mod context_budget;
/// Contradiction records and conflict-aware storage surfaces.
pub mod contradiction;
/// Semantic comparison inputs and snapshot diff surfaces.
pub mod diff;
/// Offline dream-mode scheduling and bounded synthesis surfaces.
pub mod dream;
/// Encode-path orchestration surfaces.
pub mod encode;
/// Episode formation and grouping rules.
pub mod episode;
/// Forgetting and demotion maintenance surfaces.
pub mod forgetting;
/// Query-intent classification and routing-input taxonomy surfaces.
pub mod intent;
/// Proactive and retroactive interference detection and penalty surfaces.
pub mod interference;
/// Lease-policy, freshness-state, and bounded lease scanning surfaces.
pub mod lease;
/// Shared cancellable maintenance control surfaces.
pub mod maintenance;
/// Passive-observation segmentation and bounded intake surfaces.
pub mod observe;
/// Ranking and score-fusion surfaces.
pub mod ranking;
/// Recall-path orchestration surfaces.
pub mod recall;
/// Reconsolidation, labile-window, and pending-update surfaces.
pub mod reconsolidation;
/// Repair and rebuild maintenance surfaces.
pub mod repair;
/// Retrieval result envelope and packaging surfaces.
pub mod result;
/// Bounded Tier2/Tier3 retrieval planners with escalation logic.
pub mod retrieval_planner;
/// LTP/LTD strength update and lazy Ebbinghaus decay surfaces.
pub mod strength;
