use crate::api::NamespaceId;
use crate::engine::maintenance::{
    DurableStateToken, InterruptedMaintenance, InterruptionReason, MaintenanceFailureArtifact,
    MaintenanceOperation, MaintenanceProgress, MaintenanceStep,
};

/// Bounded scheduling contract for offline dream-mode maintenance runs.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DreamPolicy {
    /// Whether background dream scheduling is enabled.
    pub enabled: bool,
    /// Minimum idle ticks required before a run may start.
    pub idle_threshold_ticks: u64,
    /// Maximum bounded poll steps the scheduler may consume for one run.
    pub max_poll_steps: u32,
    /// Maximum bounded candidate batches the dream cycle may inspect.
    pub batch_size: u32,
    /// Maximum bounded links a run may propose or create.
    pub max_links_per_run: u32,
}

impl Default for DreamPolicy {
    fn default() -> Self {
        Self {
            enabled: true,
            idle_threshold_ticks: 100,
            max_poll_steps: 4,
            batch_size: 64,
            max_links_per_run: 16,
        }
    }
}

/// Stable run trigger used by scheduler status and operator surfaces.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DreamTrigger {
    Manual,
    IdleWindow,
}

impl DreamTrigger {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Manual => "manual",
            Self::IdleWindow => "idle_window",
        }
    }
}

/// Stable skip reason when a bounded dream cycle is not allowed to start.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DreamSkipReason {
    Disabled,
    IdleWindowNotReached,
}

impl DreamSkipReason {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Disabled => "disabled",
            Self::IdleWindowNotReached => "idle_window_not_reached",
        }
    }
}

/// Inspectable scheduler snapshot for the latest or proposed dream run.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DreamStatus {
    pub enabled: bool,
    pub namespace: NamespaceId,
    pub trigger: DreamTrigger,
    pub idle_threshold_ticks: u64,
    pub idle_ticks_observed: u64,
    pub bounded_window_poll_budget: u32,
    pub batch_size: u32,
    pub max_links_per_run: u32,
    pub last_run_tick: Option<u64>,
    pub links_created_total: u64,
    pub paused_reason: Option<DreamSkipReason>,
}

impl DreamStatus {
    pub fn for_policy(
        namespace: NamespaceId,
        trigger: DreamTrigger,
        policy: DreamPolicy,
        idle_ticks_observed: u64,
        last_run_tick: Option<u64>,
        links_created_total: u64,
    ) -> Self {
        let paused_reason = if !policy.enabled {
            Some(DreamSkipReason::Disabled)
        } else if trigger == DreamTrigger::IdleWindow
            && idle_ticks_observed < policy.idle_threshold_ticks
        {
            Some(DreamSkipReason::IdleWindowNotReached)
        } else {
            None
        };

        Self {
            enabled: policy.enabled,
            namespace,
            trigger,
            idle_threshold_ticks: policy.idle_threshold_ticks,
            idle_ticks_observed,
            bounded_window_poll_budget: policy.max_poll_steps,
            batch_size: policy.batch_size,
            max_links_per_run: policy.max_links_per_run,
            last_run_tick,
            links_created_total,
            paused_reason,
        }
    }

    pub const fn should_skip(&self) -> bool {
        self.paused_reason.is_some()
    }
}

/// Operator-visible bounded dream-cycle summary.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DreamSummary {
    pub namespace: NamespaceId,
    pub trigger: DreamTrigger,
    pub execution_window: &'static str,
    pub polls_consumed: u32,
    pub links_created: u32,
    pub links_created_total: u64,
    pub candidate_batches_scanned: u32,
    pub idle_ticks_observed: u64,
    pub idle_threshold_ticks: u64,
    pub last_run_tick: u64,
    pub skipped_reason: Option<DreamSkipReason>,
    pub operator_log: Vec<String>,
}

/// Bounded offline dream cycle run driven by maintenance scheduling rules.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DreamRun {
    status: DreamStatus,
    polls_consumed: u32,
    links_created: u32,
    candidate_batches_scanned: u32,
    completed: bool,
    durable_token: DurableStateToken,
    operator_log: Vec<String>,
}

impl DreamRun {
    pub fn new(status: DreamStatus) -> Self {
        let mut operator_log = vec![format!(
            "dream trigger={} enabled={} idle_ticks={} threshold={} window_polls={} batch_size={} max_links={}",
            status.trigger.as_str(),
            status.enabled,
            status.idle_ticks_observed,
            status.idle_threshold_ticks,
            status.bounded_window_poll_budget,
            status.batch_size,
            status.max_links_per_run,
        )];

        if let Some(reason) = status.paused_reason {
            operator_log.push(format!("dream skipped: {}", reason.as_str()));
        }

        Self {
            status,
            polls_consumed: 0,
            links_created: 0,
            candidate_batches_scanned: 0,
            completed: false,
            durable_token: DurableStateToken(0),
            operator_log,
        }
    }

    fn build_summary(&self) -> DreamSummary {
        DreamSummary {
            namespace: self.status.namespace.clone(),
            trigger: self.status.trigger,
            execution_window: if self.status.trigger == DreamTrigger::Manual {
                "manual_bounded_run"
            } else {
                "idle_window_only"
            },
            polls_consumed: self.polls_consumed,
            links_created: self.links_created,
            links_created_total: self.status.links_created_total + self.links_created as u64,
            candidate_batches_scanned: self.candidate_batches_scanned,
            idle_ticks_observed: self.status.idle_ticks_observed,
            idle_threshold_ticks: self.status.idle_threshold_ticks,
            last_run_tick: self
                .status
                .last_run_tick
                .unwrap_or(self.status.idle_ticks_observed),
            skipped_reason: self.status.paused_reason,
            operator_log: self.operator_log.clone(),
        }
    }
}

impl MaintenanceOperation for DreamRun {
    type Summary = DreamSummary;

    fn poll_step(&mut self) -> MaintenanceStep<Self::Summary> {
        if self.completed {
            return MaintenanceStep::Completed(self.build_summary());
        }

        if let Some(reason) = self.status.paused_reason {
            self.completed = true;
            self.operator_log
                .push(format!("dream remained paused: {}", reason.as_str()));
            return MaintenanceStep::Blocked(reason.as_str());
        }

        if self.polls_consumed >= self.status.bounded_window_poll_budget {
            self.completed = true;
            self.operator_log
                .push("dream completed bounded execution window".to_string());
            return MaintenanceStep::Completed(self.build_summary());
        }

        self.polls_consumed += 1;
        self.candidate_batches_scanned += 1;
        let remaining_links = self
            .status
            .max_links_per_run
            .saturating_sub(self.links_created);
        let links_this_poll = remaining_links.min(4);
        self.links_created += links_this_poll;
        self.operator_log.push(format!(
            "dream poll={} scanned_batch={} links_created_this_poll={} cumulative_links={}",
            self.polls_consumed,
            self.candidate_batches_scanned,
            links_this_poll,
            self.links_created,
        ));

        if self.polls_consumed >= self.status.bounded_window_poll_budget
            || self.links_created >= self.status.max_links_per_run
        {
            self.completed = true;
            self.operator_log
                .push("dream reached bounded stop condition".to_string());
            MaintenanceStep::Completed(self.build_summary())
        } else {
            MaintenanceStep::Pending(MaintenanceProgress::new(
                self.polls_consumed,
                self.status.bounded_window_poll_budget,
            ))
        }
    }

    fn interrupt(&mut self, reason: InterruptionReason) -> InterruptedMaintenance {
        self.operator_log
            .push(format!("dream interrupted: {}", reason.as_str()));
        InterruptedMaintenance {
            reason,
            preserved_durable_state: self.durable_token,
            artifact: Some(MaintenanceFailureArtifact {
                artifact_name: "dream_cycle_interrupted",
                object_handle: format!(
                    "dream://{}/{}",
                    self.status.namespace.as_str(),
                    self.status.trigger.as_str()
                ),
                scope: self.status.namespace.as_str().to_string(),
                attempted_edge: "dream_scheduler",
                affected_item_count: self.links_created,
                pending_item_count: self
                    .status
                    .max_links_per_run
                    .saturating_sub(self.links_created),
                failure_family: "dream_cycle_boundary",
                retryable: true,
                escalation_boundary: "bounded_idle_window",
            }),
        }
    }
}

/// Canonical offline dream engine owned by the core crate.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct DreamEngine;

impl DreamEngine {
    pub const fn component_name(&self) -> &'static str {
        "engine.dream"
    }

    pub fn status(
        &self,
        namespace: NamespaceId,
        trigger: DreamTrigger,
        policy: DreamPolicy,
        idle_ticks_observed: u64,
        last_run_tick: Option<u64>,
        links_created_total: u64,
    ) -> DreamStatus {
        DreamStatus::for_policy(
            namespace,
            trigger,
            policy,
            idle_ticks_observed,
            last_run_tick,
            links_created_total,
        )
    }

    pub fn create_run(&self, status: DreamStatus) -> DreamRun {
        DreamRun::new(status)
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
    fn idle_trigger_blocks_before_threshold_is_reached() {
        let engine = DreamEngine;
        let status = engine.status(
            ns("team.alpha"),
            DreamTrigger::IdleWindow,
            DreamPolicy::default(),
            12,
            Some(4),
            7,
        );
        let mut handle = MaintenanceJobHandle::new(engine.create_run(status), 4);

        let snapshot = handle.poll();
        assert!(matches!(
            snapshot.state,
            MaintenanceJobState::Blocked("idle_window_not_reached")
        ));
    }

    #[test]
    fn disabled_dream_mode_blocks_even_for_manual_runs() {
        let engine = DreamEngine;
        let status = engine.status(
            ns("team.alpha"),
            DreamTrigger::Manual,
            DreamPolicy {
                enabled: false,
                ..DreamPolicy::default()
            },
            140,
            Some(12),
            3,
        );
        let mut handle = MaintenanceJobHandle::new(engine.create_run(status), 4);

        let snapshot = handle.poll();
        assert!(matches!(
            snapshot.state,
            MaintenanceJobState::Blocked("disabled")
        ));
    }

    #[test]
    fn manual_runs_complete_within_bounded_poll_budget() {
        let engine = DreamEngine;
        let status = engine.status(
            ns("team.alpha"),
            DreamTrigger::Manual,
            DreamPolicy {
                max_poll_steps: 3,
                max_links_per_run: 9,
                ..DreamPolicy::default()
            },
            0,
            Some(41),
            5,
        );
        let mut handle = MaintenanceJobHandle::new(engine.create_run(status), 6);
        handle.start();

        let first = handle.poll();
        assert!(matches!(first.state, MaintenanceJobState::Running { .. }));
        let second = handle.poll();
        assert!(matches!(second.state, MaintenanceJobState::Running { .. }));
        let third = handle.poll();

        match third.state {
            MaintenanceJobState::Completed(summary) => {
                assert_eq!(summary.namespace, ns("team.alpha"));
                assert_eq!(summary.trigger, DreamTrigger::Manual);
                assert_eq!(summary.execution_window, "manual_bounded_run");
                assert_eq!(summary.polls_consumed, 3);
                assert_eq!(summary.candidate_batches_scanned, 3);
                assert_eq!(summary.links_created, 9);
                assert_eq!(summary.links_created_total, 14);
                assert_eq!(summary.last_run_tick, 41);
                assert!(summary.skipped_reason.is_none());
                assert!(summary
                    .operator_log
                    .iter()
                    .any(|line| line.contains("dream reached bounded stop condition")));
            }
            other => panic!("expected completed dream summary, got {other:?}"),
        }
    }
}
