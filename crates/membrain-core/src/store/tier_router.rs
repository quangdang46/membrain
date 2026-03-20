//! Tier routing, promotion, demotion, and lifecycle-aware placement decisions.
//!
//! This module implements explicit, traceable routing rules for moving memory items
//! between hot (Tier1/Tier2) and cold (Tier3) layers. All routing decisions are
//! logged with explicit reasons and remain inspectable after the fact.

use crate::api::NamespaceId;
use crate::types::{CanonicalMemoryType, MemoryId, SessionId};

/// Current tier ownership for a memory item.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TierOwnership {
    /// Memory lives in hot durable storage (hot.db) with Tier1 cache eligibility.
    Hot,
    /// Memory has been demoted to cold durable storage (cold.db).
    /// Hot tier may retain a serving mirror but canonical ownership is cold.
    Cold,
}

impl TierOwnership {
    /// Returns the stable machine-readable name for this tier.
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::Hot => "hot",
            Self::Cold => "cold",
        }
    }
}

/// Lifecycle state influencing tier routing decisions.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum LifecycleState {
    /// Recently created, high recency score, prime candidate for hot tier.
    Fresh,
    /// Actively recalled, maintains hot tier residency.
    Active,
    /// Not recently recalled, eligible for demotion consideration.
    Dormant,
    /// Explicitly archived, canonical ownership in cold tier.
    Archived,
}

impl LifecycleState {
    /// Returns the stable machine-readable name for this lifecycle state.
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::Fresh => "fresh",
            Self::Active => "active",
            Self::Dormant => "dormant",
            Self::Archived => "archived",
        }
    }
}

/// Routing decision outcome with explicit reason.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TierRoutingDecision {
    /// Promote from cold to hot tier.
    PromoteToHot {
        /// Reason for promotion.
        reason: TierRoutingReason,
    },
    /// Demote from hot to cold tier.
    DemoteToCold {
        /// Reason for demotion.
        reason: TierRoutingReason,
    },
    /// Keep in current tier.
    KeepInPlace {
        /// Current tier ownership.
        current_tier: TierOwnership,
        /// Reason for keeping in place.
        reason: TierRoutingReason,
    },
}

impl TierRoutingDecision {
    /// Returns the target tier after this decision is applied.
    pub const fn target_tier(&self) -> TierOwnership {
        match self {
            Self::PromoteToHot { .. } => TierOwnership::Hot,
            Self::DemoteToCold { .. } => TierOwnership::Cold,
            Self::KeepInPlace { current_tier, .. } => *current_tier,
        }
    }

    /// Returns true if this decision changes tier ownership.
    pub const fn is_transition(&self) -> bool {
        matches!(self, Self::PromoteToHot { .. } | Self::DemoteToCold { .. })
    }
}

/// Explicit reasons for tier routing decisions.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TierRoutingReason {
    // Promotion reasons
    /// Recent recall activity triggered promotion.
    RecallActivity,
    /// High salience score justifies hot tier residency.
    HighSalience,
    /// Explicit user or system request to promote.
    ExplicitPromotion,
    /// Repair or rebuild restored item to hot tier.
    RepairRestoration,

    // Demotion reasons
    /// Item has been dormant beyond retention threshold.
    DormancyExceeded,
    /// Low salience score makes cold tier appropriate.
    LowSalience,
    /// Payload size exceeds hot tier budget.
    PayloadTooLarge,
    /// Explicit archive request.
    ExplicitArchive,
    /// Consolidation moved canonical ownership to cold.
    ConsolidationDemotion,

    // Keep-in-place reasons
    /// Already in appropriate tier for current state.
    TierAppropriate,
    /// Recent activity maintains hot tier residency.
    RecentActivityMaintainsHot,
    /// Dormant but not yet exceeding demotion threshold.
    DormantWithinThreshold,
    /// Already archived, no further demotion possible.
    AlreadyArchived,
    /// Pinned item, protected from demotion.
    PinnedProtection,
}

impl TierRoutingReason {
    /// Returns a human-readable explanation for this reason.
    pub const fn explanation(&self) -> &'static str {
        match self {
            Self::RecallActivity => "Recent recall activity triggers hot tier promotion",
            Self::HighSalience => "Salience score exceeds hot tier threshold",
            Self::ExplicitPromotion => "Explicit request to promote to hot tier",
            Self::RepairRestoration => "Repair flow restored item to hot tier",
            Self::DormancyExceeded => "Dormancy period exceeded demotion threshold",
            Self::LowSalience => "Salience score below hot tier retention threshold",
            Self::PayloadTooLarge => "Payload size exceeds hot tier budget",
            Self::ExplicitArchive => "Explicit archive request to cold tier",
            Self::ConsolidationDemotion => "Consolidation moved canonical ownership to cold",
            Self::TierAppropriate => "Current tier matches item state",
            Self::RecentActivityMaintainsHot => "Recent activity maintains hot tier residency",
            Self::DormantWithinThreshold => "Dormant but within grace period",
            Self::AlreadyArchived => "Item already archived, no further demotion",
            Self::PinnedProtection => "Item is pinned, protected from demotion",
        }
    }
}

/// Input parameters for tier routing decisions.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TierRoutingInput {
    /// Namespace the memory belongs to.
    pub namespace: NamespaceId,
    /// Stable memory identifier.
    pub memory_id: MemoryId,
    /// Session the memory was created in.
    pub session_id: SessionId,
    /// Canonical memory type.
    pub memory_type: CanonicalMemoryType,
    /// Current tier ownership.
    pub current_tier: TierOwnership,
    /// Current lifecycle state.
    pub lifecycle_state: LifecycleState,
    /// Salience score (0-1000, higher = more salient).
    pub salience: u16,
    /// Number of ticks since last recall (0 = just recalled).
    pub ticks_since_recall: u64,
    /// Payload size in bytes.
    pub payload_size_bytes: usize,
    /// Whether the item is pinned (protected from demotion).
    pub pinned: bool,
}

/// Configuration thresholds for tier routing decisions.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TierRoutingConfig {
    /// Salience threshold for hot tier retention (0-1000).
    pub hot_salience_threshold: u16,
    /// Ticks of inactivity before demotion consideration.
    pub dormancy_demotion_threshold: u64,
    /// Maximum payload size for hot tier (bytes).
    pub hot_payload_size_limit: usize,
    /// Whether to allow automatic promotion on recall.
    pub promote_on_recall: bool,
}

impl Default for TierRoutingConfig {
    fn default() -> Self {
        Self {
            hot_salience_threshold: 300,
            dormancy_demotion_threshold: 10_000,
            hot_payload_size_limit: 64 * 1024, // 64 KB
            promote_on_recall: true,
        }
    }
}

/// Trace record for a tier routing decision.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TierRoutingTrace {
    /// Memory that was evaluated.
    pub memory_id: MemoryId,
    /// Decision that was made.
    pub decision: TierRoutingDecision,
    /// Salience at decision time.
    pub salience: u16,
    /// Ticks since last recall at decision time.
    pub ticks_since_recall: u64,
    /// Payload size at decision time.
    pub payload_size_bytes: usize,
    /// Lifecycle state at decision time.
    pub lifecycle_state: LifecycleState,
    /// Whether item was pinned.
    pub pinned: bool,
}

impl TierRoutingTrace {
    /// Returns a human-readable summary of this routing decision.
    pub fn summary(&self) -> String {
        let direction = match &self.decision {
            TierRoutingDecision::PromoteToHot { reason } => {
                format!("PROMOTE to hot: {}", reason.explanation())
            }
            TierRoutingDecision::DemoteToCold { reason } => {
                format!("DEMOTE to cold: {}", reason.explanation())
            }
            TierRoutingDecision::KeepInPlace {
                current_tier,
                reason,
            } => {
                format!(
                    "KEEP in {}: {}",
                    current_tier.as_str(),
                    reason.explanation()
                )
            }
        };
        format!(
            "[{}] {} (salience={}, ticks_since_recall={}, payload={}B, lifecycle={})",
            self.memory_id.0,
            direction,
            self.salience,
            self.ticks_since_recall,
            self.payload_size_bytes,
            self.lifecycle_state.as_str()
        )
    }
}

/// Tier router that evaluates routing decisions with explicit rules.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TierRouter {
    config: TierRoutingConfig,
}

impl TierRouter {
    /// Creates a new tier router with the given configuration.
    pub fn new(config: TierRoutingConfig) -> Self {
        Self { config }
    }

    /// Creates a tier router with default configuration.
    pub fn with_defaults() -> Self {
        Self::new(TierRoutingConfig::default())
    }

    /// Evaluates a routing decision for the given input.
    ///
    /// This method implements explicit routing rules in priority order:
    /// 1. Pinned items are protected from demotion
    /// 2. Archived items converge to cold canonical ownership
    /// 3. Explicit lifecycle state overrides
    /// 4. Salience-based decisions
    /// 5. Dormancy-based decisions
    /// 6. Payload size constraints
    pub fn evaluate(&self, input: &TierRoutingInput) -> TierRoutingDecision {
        // Rule 1: Pinned items are protected from demotion
        if input.pinned && input.current_tier == TierOwnership::Hot {
            return TierRoutingDecision::KeepInPlace {
                current_tier: TierOwnership::Hot,
                reason: TierRoutingReason::PinnedProtection,
            };
        }

        // Rule 2: Archived items converge to cold canonical ownership
        if input.lifecycle_state == LifecycleState::Archived {
            return if input.current_tier == TierOwnership::Cold {
                TierRoutingDecision::KeepInPlace {
                    current_tier: TierOwnership::Cold,
                    reason: TierRoutingReason::AlreadyArchived,
                }
            } else {
                TierRoutingDecision::DemoteToCold {
                    reason: TierRoutingReason::ExplicitArchive,
                }
            };
        }

        // Rule 3: Explicit archive request
        if input.lifecycle_state == LifecycleState::Dormant
            && input.current_tier == TierOwnership::Hot
            && input.ticks_since_recall >= self.config.dormancy_demotion_threshold
        {
            return TierRoutingDecision::DemoteToCold {
                reason: TierRoutingReason::DormancyExceeded,
            };
        }

        // Rule 4: Promote on recall activity (if enabled)
        if self.config.promote_on_recall
            && input.current_tier == TierOwnership::Cold
            && input.ticks_since_recall == 0
            && input.lifecycle_state != LifecycleState::Archived
        {
            return TierRoutingDecision::PromoteToHot {
                reason: TierRoutingReason::RecallActivity,
            };
        }

        // Rule 5: High salience promotes to hot
        if input.current_tier == TierOwnership::Cold
            && input.salience >= self.config.hot_salience_threshold
            && input.lifecycle_state != LifecycleState::Archived
        {
            return TierRoutingDecision::PromoteToHot {
                reason: TierRoutingReason::HighSalience,
            };
        }

        // Rule 6: Payload size exceeds hot tier limit even during freshness/dormancy grace
        if input.current_tier == TierOwnership::Hot
            && input.payload_size_bytes > self.config.hot_payload_size_limit
        {
            return TierRoutingDecision::DemoteToCold {
                reason: TierRoutingReason::PayloadTooLarge,
            };
        }

        // Rule 7: Fresh items in hot tier stay hot due to recent activity
        if input.current_tier == TierOwnership::Hot
            && input.lifecycle_state == LifecycleState::Fresh
        {
            return TierRoutingDecision::KeepInPlace {
                current_tier: TierOwnership::Hot,
                reason: TierRoutingReason::RecentActivityMaintainsHot,
            };
        }

        // Rule 8: Dormant items within the grace threshold stay hot
        if input.current_tier == TierOwnership::Hot
            && input.lifecycle_state == LifecycleState::Dormant
            && input.ticks_since_recall < self.config.dormancy_demotion_threshold
        {
            return TierRoutingDecision::KeepInPlace {
                current_tier: TierOwnership::Hot,
                reason: TierRoutingReason::DormantWithinThreshold,
            };
        }

        // Rule 9: Low salience demotes from hot once grace rules no longer apply
        if input.current_tier == TierOwnership::Hot
            && input.salience < self.config.hot_salience_threshold
            && input.lifecycle_state != LifecycleState::Fresh
        {
            return TierRoutingDecision::DemoteToCold {
                reason: TierRoutingReason::LowSalience,
            };
        }

        // Default: keep in current tier
        TierRoutingDecision::KeepInPlace {
            current_tier: input.current_tier,
            reason: TierRoutingReason::TierAppropriate,
        }
    }

    /// Returns whether a cold-owned item should still be surfaced directly to explicit cold-tier recall flows.
    pub const fn cold_recall_visibility(&self, lifecycle_state: LifecycleState) -> bool {
        matches!(
            lifecycle_state,
            LifecycleState::Archived | LifecycleState::Dormant | LifecycleState::Active
        )
    }

    /// Returns whether a cold-owned item can be restored to hot ownership.
    pub const fn cold_restore_allowed(&self, lifecycle_state: LifecycleState) -> bool {
        self.cold_recall_visibility(lifecycle_state)
    }

    /// Evaluates a routing decision and returns a trace record.
    pub fn evaluate_with_trace(&self, input: &TierRoutingInput) -> TierRoutingTrace {
        let decision = self.evaluate(input);
        TierRoutingTrace {
            memory_id: input.memory_id,
            decision,
            salience: input.salience,
            ticks_since_recall: input.ticks_since_recall,
            payload_size_bytes: input.payload_size_bytes,
            lifecycle_state: input.lifecycle_state,
            pinned: input.pinned,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::api::NamespaceId;
    use crate::types::{CanonicalMemoryType, MemoryId, SessionId};

    fn make_input(
        current_tier: TierOwnership,
        lifecycle_state: LifecycleState,
        salience: u16,
        ticks_since_recall: u64,
        payload_size_bytes: usize,
        pinned: bool,
    ) -> TierRoutingInput {
        TierRoutingInput {
            namespace: NamespaceId::new("test.namespace").unwrap(),
            memory_id: MemoryId(1),
            session_id: SessionId(1),
            memory_type: CanonicalMemoryType::Event,
            current_tier,
            lifecycle_state,
            salience,
            ticks_since_recall,
            payload_size_bytes,
            pinned,
        }
    }

    #[test]
    fn pinned_item_protected_from_demotion() {
        let router = TierRouter::with_defaults();
        let input = make_input(
            TierOwnership::Hot,
            LifecycleState::Dormant,
            100,    // low salience
            20_000, // long dormancy
            1024,
            true, // pinned
        );

        let decision = router.evaluate(&input);
        assert!(matches!(
            decision,
            TierRoutingDecision::KeepInPlace {
                current_tier: TierOwnership::Hot,
                reason: TierRoutingReason::PinnedProtection
            }
        ));
    }

    #[test]
    fn archived_cold_item_stays_archived() {
        let router = TierRouter::with_defaults();
        let input = make_input(
            TierOwnership::Cold,
            LifecycleState::Archived,
            900, // high salience
            0,   // recent recall
            1024,
            false,
        );

        let decision = router.evaluate(&input);
        assert!(matches!(
            decision,
            TierRoutingDecision::KeepInPlace {
                current_tier: TierOwnership::Cold,
                reason: TierRoutingReason::AlreadyArchived
            }
        ));
    }

    #[test]
    fn archived_hot_item_demotes_to_cold() {
        let router = TierRouter::with_defaults();
        let input = make_input(
            TierOwnership::Hot,
            LifecycleState::Archived,
            900,
            0,
            1024,
            false,
        );

        let decision = router.evaluate(&input);
        assert!(matches!(
            decision,
            TierRoutingDecision::DemoteToCold {
                reason: TierRoutingReason::ExplicitArchive
            }
        ));
    }

    #[test]
    fn dormancy_exceeded_triggers_demotion() {
        let router = TierRouter::with_defaults();
        let input = make_input(
            TierOwnership::Hot,
            LifecycleState::Dormant,
            500,
            15_000, // exceeds default threshold of 10,000
            1024,
            false,
        );

        let decision = router.evaluate(&input);
        assert!(matches!(
            decision,
            TierRoutingDecision::DemoteToCold {
                reason: TierRoutingReason::DormancyExceeded
            }
        ));
    }

    #[test]
    fn recall_activity_triggers_promotion() {
        let router = TierRouter::with_defaults();
        let input = make_input(
            TierOwnership::Cold,
            LifecycleState::Active,
            500,
            0, // just recalled
            1024,
            false,
        );

        let decision = router.evaluate(&input);
        assert!(matches!(
            decision,
            TierRoutingDecision::PromoteToHot {
                reason: TierRoutingReason::RecallActivity
            }
        ));
    }

    #[test]
    fn high_salience_triggers_promotion() {
        let router = TierRouter::with_defaults();
        let input = make_input(
            TierOwnership::Cold,
            LifecycleState::Active,
            800, // exceeds threshold of 300
            100,
            1024,
            false,
        );

        let decision = router.evaluate(&input);
        assert!(matches!(
            decision,
            TierRoutingDecision::PromoteToHot {
                reason: TierRoutingReason::HighSalience
            }
        ));
    }

    #[test]
    fn low_salience_triggers_demotion() {
        let router = TierRouter::with_defaults();
        let input = make_input(
            TierOwnership::Hot,
            LifecycleState::Active,
            100, // below threshold of 300
            100,
            1024,
            false,
        );

        let decision = router.evaluate(&input);
        assert!(matches!(
            decision,
            TierRoutingDecision::DemoteToCold {
                reason: TierRoutingReason::LowSalience
            }
        ));
    }

    #[test]
    fn large_payload_triggers_demotion() {
        let router = TierRouter::with_defaults();
        let input = make_input(
            TierOwnership::Hot,
            LifecycleState::Active,
            500,
            0,
            100 * 1024, // 100 KB, exceeds 64 KB limit
            false,
        );

        let decision = router.evaluate(&input);
        assert!(matches!(
            decision,
            TierRoutingDecision::DemoteToCold {
                reason: TierRoutingReason::PayloadTooLarge
            }
        ));
    }

    #[test]
    fn large_payload_overrides_dormancy_grace() {
        let router = TierRouter::with_defaults();
        let input = make_input(
            TierOwnership::Hot,
            LifecycleState::Dormant,
            500,
            5000,       // within dormancy grace threshold
            100 * 1024, // exceeds 64 KB limit
            false,
        );

        let decision = router.evaluate(&input);
        assert!(matches!(
            decision,
            TierRoutingDecision::DemoteToCold {
                reason: TierRoutingReason::PayloadTooLarge
            }
        ));
    }

    #[test]
    fn fresh_item_stays_hot() {
        let router = TierRouter::with_defaults();
        let input = make_input(
            TierOwnership::Hot,
            LifecycleState::Fresh,
            100, // low salience, but fresh
            0,
            1024,
            false,
        );

        let decision = router.evaluate(&input);
        assert!(matches!(
            decision,
            TierRoutingDecision::KeepInPlace {
                current_tier: TierOwnership::Hot,
                reason: TierRoutingReason::RecentActivityMaintainsHot
            }
        ));
    }

    #[test]
    fn dormant_within_threshold_stays_hot() {
        let router = TierRouter::with_defaults();
        let input = make_input(
            TierOwnership::Hot,
            LifecycleState::Dormant,
            500,
            5000, // below threshold of 10,000
            1024,
            false,
        );

        let decision = router.evaluate(&input);
        assert!(matches!(
            decision,
            TierRoutingDecision::KeepInPlace {
                current_tier: TierOwnership::Hot,
                reason: TierRoutingReason::DormantWithinThreshold
            }
        ));
    }

    #[test]
    fn dormant_low_salience_within_threshold_keeps_hot_until_grace_expires() {
        let router = TierRouter::with_defaults();
        let input = make_input(
            TierOwnership::Hot,
            LifecycleState::Dormant,
            100,  // low salience
            5000, // still within dormancy grace threshold
            1024,
            false,
        );

        let decision = router.evaluate(&input);
        assert!(matches!(
            decision,
            TierRoutingDecision::KeepInPlace {
                current_tier: TierOwnership::Hot,
                reason: TierRoutingReason::DormantWithinThreshold
            }
        ));
    }

    #[test]
    fn trace_summary_is_human_readable() {
        let router = TierRouter::with_defaults();
        let input = make_input(
            TierOwnership::Hot,
            LifecycleState::Dormant,
            500,
            5000,
            1024,
            false,
        );

        let trace = router.evaluate_with_trace(&input);
        let summary = trace.summary();

        assert!(summary.contains("KEEP"));
        assert!(summary.contains("salience=500"));
        assert!(summary.contains("ticks_since_recall=5000"));
        assert!(summary.contains("payload=1024B"));
        assert!(summary.contains("lifecycle=dormant"));
    }

    #[test]
    fn decision_target_tier() {
        let promote = TierRoutingDecision::PromoteToHot {
            reason: TierRoutingReason::RecallActivity,
        };
        assert_eq!(promote.target_tier(), TierOwnership::Hot);

        let demote = TierRoutingDecision::DemoteToCold {
            reason: TierRoutingReason::LowSalience,
        };
        assert_eq!(demote.target_tier(), TierOwnership::Cold);

        let keep = TierRoutingDecision::KeepInPlace {
            current_tier: TierOwnership::Hot,
            reason: TierRoutingReason::TierAppropriate,
        };
        assert_eq!(keep.target_tier(), TierOwnership::Hot);
    }

    #[test]
    fn decision_is_transition() {
        let promote = TierRoutingDecision::PromoteToHot {
            reason: TierRoutingReason::RecallActivity,
        };
        assert!(promote.is_transition());

        let demote = TierRoutingDecision::DemoteToCold {
            reason: TierRoutingReason::LowSalience,
        };
        assert!(demote.is_transition());

        let keep = TierRoutingDecision::KeepInPlace {
            current_tier: TierOwnership::Hot,
            reason: TierRoutingReason::TierAppropriate,
        };
        assert!(!keep.is_transition());
    }

    #[test]
    fn archived_dormant_and_active_cold_items_remain_recall_visible_and_restorable() {
        let router = TierRouter::with_defaults();
        assert!(router.cold_recall_visibility(LifecycleState::Archived));
        assert!(router.cold_restore_allowed(LifecycleState::Archived));
    }

    #[test]
    fn dormant_and_active_cold_items_remain_recall_visible_and_restorable() {
        let router = TierRouter::with_defaults();
        assert!(router.cold_recall_visibility(LifecycleState::Dormant));
        assert!(router.cold_restore_allowed(LifecycleState::Dormant));
        assert!(router.cold_recall_visibility(LifecycleState::Active));
        assert!(router.cold_restore_allowed(LifecycleState::Active));
    }
}
