use crate::observability::{MaintenanceQueueReport, MaintenanceQueueStatus};
use crate::types::{CanonicalMemoryType, MemoryId};

/// Canonical lease policy classes applied to memory-like durable objects.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum LeasePolicy {
    Volatile,
    Normal,
    Durable,
    Pinned,
}

impl LeasePolicy {
    /// Returns the stable machine-readable policy label.
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Volatile => "volatile",
            Self::Normal => "normal",
            Self::Durable => "durable",
            Self::Pinned => "pinned",
        }
    }

    /// Returns the bounded logical-tick TTL for the lease, if one exists.
    pub const fn ttl_ticks(self) -> Option<u64> {
        match self {
            Self::Volatile => Some(32),
            Self::Normal => Some(256),
            Self::Durable => Some(2_048),
            Self::Pinned => None,
        }
    }

    /// Returns the bounded grace window after expiry before the lease becomes fully stale.
    pub const fn grace_ticks(self) -> u64 {
        match self {
            Self::Volatile => 8,
            Self::Normal => 64,
            Self::Durable => 256,
            Self::Pinned => 0,
        }
    }

    /// Returns the confidence cap used once the lease becomes sensitive but not fully stale.
    pub const fn lease_sensitive_confidence_cap(self) -> Option<u16> {
        match self {
            Self::Volatile => Some(650),
            Self::Normal => Some(800),
            Self::Durable => Some(900),
            Self::Pinned => None,
        }
    }

    /// Returns the confidence cap used once the lease is stale.
    pub const fn stale_confidence_cap(self) -> Option<u16> {
        match self {
            Self::Volatile => Some(300),
            Self::Normal => Some(550),
            Self::Durable => Some(700),
            Self::Pinned => None,
        }
    }

    /// Returns the confidence cap used when action-critical use requires re-check or withholding.
    pub const fn action_critical_confidence_cap(self) -> Option<u16> {
        match self {
            Self::Volatile => Some(250),
            Self::Normal => Some(450),
            Self::Durable => Some(600),
            Self::Pinned => None,
        }
    }
}

/// Explicit freshness state produced by lease evaluation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FreshnessState {
    Fresh,
    LeaseSensitive,
    Stale,
    RecheckRequired,
    Pinned,
}

impl FreshnessState {
    /// Returns the stable machine-readable freshness label.
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Fresh => "fresh",
            Self::LeaseSensitive => "lease_sensitive",
            Self::Stale => "stale",
            Self::RecheckRequired => "recheck_required",
            Self::Pinned => "pinned",
        }
    }
}

/// Explicit lease metadata stored alongside one memory-like durable object.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct LeaseMetadata {
    pub lease_policy: LeasePolicy,
    pub freshness_state: FreshnessState,
    pub lease_expires_at_tick: Option<u64>,
    pub last_refreshed_at_tick: u64,
}

impl LeaseMetadata {
    /// Builds lease metadata anchored at one deterministic logical tick.
    pub const fn new(lease_policy: LeasePolicy, refreshed_at_tick: u64) -> Self {
        Self {
            lease_policy,
            freshness_state: match lease_policy {
                LeasePolicy::Pinned => FreshnessState::Pinned,
                _ => FreshnessState::Fresh,
            },
            lease_expires_at_tick: match lease_policy.ttl_ticks() {
                Some(ttl) => Some(refreshed_at_tick.saturating_add(ttl)),
                None => None,
            },
            last_refreshed_at_tick: refreshed_at_tick,
        }
    }

    /// Builds the default lease metadata for one canonical memory family.
    pub const fn recommended(memory_type: CanonicalMemoryType, observation_source: bool) -> Self {
        Self::new(recommended_policy(memory_type, observation_source), 0)
    }

    /// Builds pinned lease metadata that never ages out.
    pub const fn pinned(refreshed_at_tick: u64) -> Self {
        Self::new(LeasePolicy::Pinned, refreshed_at_tick)
    }

    /// Refreshes the lease from one deterministic logical tick.
    pub const fn refresh_at(mut self, refreshed_at_tick: u64) -> Self {
        self.last_refreshed_at_tick = refreshed_at_tick;
        self.lease_expires_at_tick = match self.lease_policy.ttl_ticks() {
            Some(ttl) => Some(refreshed_at_tick.saturating_add(ttl)),
            None => None,
        };
        self.freshness_state = match self.lease_policy {
            LeasePolicy::Pinned => FreshnessState::Pinned,
            _ => FreshnessState::Fresh,
        };
        self
    }

    /// Evaluates the lease at one deterministic logical tick.
    pub const fn evaluate(self, current_tick: u64, action_critical: bool) -> LeaseDecision {
        let previous_state = self.freshness_state;

        if matches!(self.lease_policy, LeasePolicy::Pinned) {
            return LeaseDecision {
                previous_state,
                freshness_state: FreshnessState::Pinned,
                action: LeaseAction::None,
                confidence_cap: None,
                reason: "pinned lease bypasses freshness decay and stale withholding",
            };
        }

        let Some(expires_at_tick) = self.lease_expires_at_tick else {
            return LeaseDecision {
                previous_state,
                freshness_state: FreshnessState::Fresh,
                action: LeaseAction::None,
                confidence_cap: None,
                reason: "lease has no explicit expiry and remains fresh",
            };
        };

        if current_tick <= expires_at_tick {
            return LeaseDecision {
                previous_state,
                freshness_state: FreshnessState::Fresh,
                action: LeaseAction::None,
                confidence_cap: None,
                reason: "lease remains inside its freshness window",
            };
        }

        let grace_end_tick = expires_at_tick.saturating_add(self.lease_policy.grace_ticks());
        if current_tick <= grace_end_tick {
            return LeaseDecision {
                previous_state,
                freshness_state: FreshnessState::LeaseSensitive,
                action: LeaseAction::LowerConfidence,
                confidence_cap: self.lease_policy.lease_sensitive_confidence_cap(),
                reason: "lease expired and entered the bounded confidence-downgrade grace window",
            };
        }

        if action_critical {
            return LeaseDecision {
                previous_state,
                freshness_state: FreshnessState::RecheckRequired,
                action: match self.lease_policy {
                    LeasePolicy::Volatile => LeaseAction::Withhold,
                    LeasePolicy::Normal | LeasePolicy::Durable | LeasePolicy::Pinned => {
                        LeaseAction::Recheck
                    }
                },
                confidence_cap: self.lease_policy.action_critical_confidence_cap(),
                reason: "stale action-critical evidence requires re-check or withholding",
            };
        }

        LeaseDecision {
            previous_state,
            freshness_state: FreshnessState::Stale,
            action: LeaseAction::LowerConfidence,
            confidence_cap: self.lease_policy.stale_confidence_cap(),
            reason: "lease exceeded grace and now downgrades stale evidence explicitly",
        }
    }

    /// Returns a copy with the evaluated freshness state applied.
    pub const fn transitioned(self, current_tick: u64, action_critical: bool) -> Self {
        let decision = self.evaluate(current_tick, action_critical);
        Self {
            freshness_state: decision.freshness_state,
            ..self
        }
    }
}

/// Explicit action the system must take when a lease degrades.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LeaseAction {
    None,
    LowerConfidence,
    Recheck,
    Withhold,
}

impl LeaseAction {
    /// Returns the stable machine-readable action label.
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::None => "none",
            Self::LowerConfidence => "lower_confidence",
            Self::Recheck => "recheck",
            Self::Withhold => "withhold",
        }
    }
}

/// Decision emitted by deterministic lease evaluation.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct LeaseDecision {
    pub previous_state: FreshnessState,
    pub freshness_state: FreshnessState,
    pub action: LeaseAction,
    pub confidence_cap: Option<u16>,
    pub reason: &'static str,
}

impl LeaseDecision {
    /// Applies the lease-imposed confidence cap to one candidate confidence score.
    pub const fn apply_confidence_cap(self, confidence: u16) -> u16 {
        match self.confidence_cap {
            Some(cap) => {
                if confidence < cap {
                    confidence
                } else {
                    cap
                }
            }
            None => confidence,
        }
    }

    /// Returns whether this decision blocks high-confidence serving at the given threshold.
    pub const fn blocks_high_confidence_output(self, threshold: u16) -> bool {
        matches!(self.action, LeaseAction::Recheck | LeaseAction::Withhold)
            || self.apply_confidence_cap(1000) < threshold
    }
}

/// One candidate examined by the bounded lease scanner.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct LeaseScanItem {
    pub memory_id: MemoryId,
    pub lease: LeaseMetadata,
    pub action_critical: bool,
}

/// Transition log emitted for one scanned item.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct LeaseTransitionLog {
    pub memory_id: MemoryId,
    pub lease_policy: LeasePolicy,
    pub previous_state: FreshnessState,
    pub next_state: FreshnessState,
    pub action: LeaseAction,
    pub confidence_cap: Option<u16>,
    pub reason: &'static str,
}

/// Bounded scanner result proving how many items were scanned and transitioned.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LeaseScanReport {
    pub scanned_items: usize,
    pub transitioned_items: usize,
    pub recheck_required_items: usize,
    pub withheld_items: usize,
    pub transitions: Vec<LeaseTransitionLog>,
    pub queue_report: MaintenanceQueueReport,
}

/// Deterministic bounded lease scanner that only performs freshness transitions.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct LeaseScanner;

impl LeaseScanner {
    /// Scans at most `max_items` candidates and returns transition-only output.
    pub fn scan(
        &self,
        items: &[LeaseScanItem],
        current_tick: u64,
        max_items: usize,
    ) -> LeaseScanReport {
        if items.is_empty() || max_items == 0 {
            return LeaseScanReport {
                scanned_items: 0,
                transitioned_items: 0,
                recheck_required_items: 0,
                withheld_items: 0,
                transitions: Vec::new(),
                queue_report: MaintenanceQueueReport {
                    queue_family: "lease_scanner",
                    queue_status: MaintenanceQueueStatus::Idle,
                    queue_depth_before: items.len() as u32,
                    queue_depth_after: items.len() as u32,
                    jobs_processed: 0,
                    affected_item_count: 0,
                    duration_ms: 0,
                    retry_attempts: 0,
                    partial_run: false,
                },
            };
        }

        let scan_limit = items.len().min(max_items);
        let mut transitions = Vec::with_capacity(scan_limit);
        let mut transitioned_items = 0usize;
        let mut recheck_required_items = 0usize;
        let mut withheld_items = 0usize;

        for item in items.iter().take(scan_limit) {
            let decision = item.lease.evaluate(current_tick, item.action_critical);
            if decision.previous_state != decision.freshness_state {
                transitioned_items += 1;
            }
            if matches!(decision.freshness_state, FreshnessState::RecheckRequired) {
                recheck_required_items += 1;
            }
            if matches!(decision.action, LeaseAction::Withhold) {
                withheld_items += 1;
            }
            transitions.push(LeaseTransitionLog {
                memory_id: item.memory_id,
                lease_policy: item.lease.lease_policy,
                previous_state: decision.previous_state,
                next_state: decision.freshness_state,
                action: decision.action,
                confidence_cap: decision.confidence_cap,
                reason: decision.reason,
            });
        }

        let partial_run = items.len() > scan_limit;
        LeaseScanReport {
            scanned_items: scan_limit,
            transitioned_items,
            recheck_required_items,
            withheld_items,
            transitions,
            queue_report: MaintenanceQueueReport {
                queue_family: "lease_scanner",
                queue_status: if partial_run {
                    MaintenanceQueueStatus::Partial
                } else {
                    MaintenanceQueueStatus::Completed
                },
                queue_depth_before: items.len() as u32,
                queue_depth_after: items.len().saturating_sub(scan_limit) as u32,
                jobs_processed: scan_limit as u32,
                affected_item_count: transitioned_items as u32,
                duration_ms: 0,
                retry_attempts: 0,
                partial_run,
            },
        }
    }
}

/// Returns the recommended lease policy for one canonical memory family.
pub const fn recommended_policy(
    memory_type: CanonicalMemoryType,
    observation_source: bool,
) -> LeasePolicy {
    if observation_source {
        return LeasePolicy::Volatile;
    }

    match memory_type {
        CanonicalMemoryType::Observation => LeasePolicy::Volatile,
        CanonicalMemoryType::UserPreference => LeasePolicy::Durable,
        CanonicalMemoryType::Event
        | CanonicalMemoryType::ToolOutcome
        | CanonicalMemoryType::SessionMarker => LeasePolicy::Normal,
    }
}

#[cfg(test)]
mod tests {
    use super::{
        recommended_policy, FreshnessState, LeaseAction, LeaseMetadata, LeasePolicy, LeaseScanItem,
        LeaseScanner,
    };
    use crate::observability::MaintenanceQueueStatus;
    use crate::types::{CanonicalMemoryType, MemoryId};

    #[test]
    fn recommended_policy_assigns_explicit_classes() {
        assert_eq!(
            recommended_policy(CanonicalMemoryType::Observation, false),
            LeasePolicy::Volatile
        );
        assert_eq!(
            recommended_policy(CanonicalMemoryType::UserPreference, false),
            LeasePolicy::Durable
        );
        assert_eq!(
            recommended_policy(CanonicalMemoryType::Event, false),
            LeasePolicy::Normal
        );
        assert_eq!(
            recommended_policy(CanonicalMemoryType::Event, true),
            LeasePolicy::Volatile
        );
    }

    #[test]
    fn lease_transitions_from_fresh_to_sensitive_to_stale() {
        let lease = LeaseMetadata::new(LeasePolicy::Normal, 10);

        let fresh = lease.evaluate(32, false);
        assert_eq!(fresh.freshness_state, FreshnessState::Fresh);
        assert_eq!(fresh.action, LeaseAction::None);

        let sensitive = lease.evaluate(300, false);
        assert_eq!(sensitive.freshness_state, FreshnessState::LeaseSensitive);
        assert_eq!(sensitive.action, LeaseAction::LowerConfidence);
        assert_eq!(sensitive.confidence_cap, Some(800));

        let stale = lease.evaluate(400, false);
        assert_eq!(stale.freshness_state, FreshnessState::Stale);
        assert_eq!(stale.action, LeaseAction::LowerConfidence);
        assert_eq!(stale.confidence_cap, Some(550));
    }

    #[test]
    fn stale_action_critical_lease_requires_recheck_or_withhold() {
        let normal = LeaseMetadata::new(LeasePolicy::Normal, 0).evaluate(600, true);
        assert_eq!(normal.freshness_state, FreshnessState::RecheckRequired);
        assert_eq!(normal.action, LeaseAction::Recheck);
        assert_eq!(normal.confidence_cap, Some(450));

        let volatile = LeaseMetadata::new(LeasePolicy::Volatile, 0).evaluate(100, true);
        assert_eq!(volatile.freshness_state, FreshnessState::RecheckRequired);
        assert_eq!(volatile.action, LeaseAction::Withhold);
        assert_eq!(volatile.confidence_cap, Some(250));
    }

    #[test]
    fn pinned_lease_stays_pinned_forever() {
        let pinned = LeaseMetadata::pinned(12).evaluate(50_000, true);
        assert_eq!(pinned.freshness_state, FreshnessState::Pinned);
        assert_eq!(pinned.action, LeaseAction::None);
        assert_eq!(pinned.confidence_cap, None);
    }

    #[test]
    fn bounded_scanner_limits_work_and_reports_partial_queue_state() {
        let scanner = LeaseScanner;
        let items = vec![
            LeaseScanItem {
                memory_id: MemoryId(1),
                lease: LeaseMetadata::new(LeasePolicy::Normal, 0),
                action_critical: false,
            },
            LeaseScanItem {
                memory_id: MemoryId(2),
                lease: LeaseMetadata::new(LeasePolicy::Volatile, 0),
                action_critical: true,
            },
            LeaseScanItem {
                memory_id: MemoryId(3),
                lease: LeaseMetadata::pinned(0),
                action_critical: true,
            },
        ];

        let report = scanner.scan(&items, 100, 2);
        assert_eq!(report.scanned_items, 2);
        assert_eq!(report.transitioned_items, 1);
        assert_eq!(report.recheck_required_items, 1);
        assert_eq!(report.withheld_items, 1);
        assert_eq!(report.transitions.len(), 2);
        assert_eq!(
            report.queue_report.queue_status,
            MaintenanceQueueStatus::Partial
        );
        assert_eq!(report.queue_report.queue_depth_before, 3);
        assert_eq!(report.queue_report.queue_depth_after, 1);
        assert!(report.queue_report.partial_run);
        assert_eq!(report.transitions[1].memory_id, MemoryId(2));
        assert_eq!(report.transitions[1].action, LeaseAction::Withhold);
    }
}
