use membrain_core::engine::maintenance::{
    DurableStateToken, InterruptedMaintenance, InterruptionReason, MaintenanceController,
    MaintenanceJobHandle, MaintenanceJobState, MaintenanceOperation, MaintenanceProgress,
    MaintenanceStep,
};

#[derive(Debug, Clone, PartialEq, Eq)]
struct ScriptedMaintenance {
    steps: Vec<MaintenanceStep<&'static str>>,
    next_step: usize,
    preserved: DurableStateToken,
    interruptions: Vec<InterruptionReason>,
}

impl ScriptedMaintenance {
    fn new(steps: Vec<MaintenanceStep<&'static str>>, preserved: DurableStateToken) -> Self {
        Self {
            steps,
            next_step: 0,
            preserved,
            interruptions: Vec::new(),
        }
    }
}

impl MaintenanceOperation for ScriptedMaintenance {
    type Summary = &'static str;

    fn poll_step(&mut self) -> MaintenanceStep<Self::Summary> {
        let step = self.steps[self.next_step].clone();
        self.next_step += 1;
        step
    }

    fn interrupt(&mut self, reason: InterruptionReason) -> InterruptedMaintenance {
        self.interruptions.push(reason);
        InterruptedMaintenance {
            reason,
            preserved_durable_state: self.preserved,
        }
    }
}

#[test]
fn cancel_before_start_preserves_prior_durable_state() {
    let operation = ScriptedMaintenance::new(
        vec![MaintenanceStep::Completed("should never run")],
        DurableStateToken(41),
    );
    let mut handle = MaintenanceJobHandle::new(operation, 3);

    let cancelled = handle.cancel();

    assert_eq!(
        cancelled.state,
        MaintenanceJobState::Cancelled(InterruptedMaintenance {
            reason: InterruptionReason::Cancelled,
            preserved_durable_state: DurableStateToken(41),
        })
    );
    assert_eq!(cancelled.polls_used, 0);
}

#[test]
fn cancel_during_run_finishes_as_cancelled_on_next_poll() {
    let operation = ScriptedMaintenance::new(
        vec![
            MaintenanceStep::Pending(MaintenanceProgress::new(1, 3)),
            MaintenanceStep::Completed("should not complete"),
        ],
        DurableStateToken(52),
    );
    let mut handle = MaintenanceJobHandle::new(operation, 4);

    let first_poll = handle.poll();
    assert_eq!(
        first_poll.state,
        MaintenanceJobState::Running {
            progress: Some(MaintenanceProgress::new(1, 3)),
        }
    );

    let requested = handle.cancel();
    assert_eq!(
        requested.state,
        MaintenanceJobState::CancelRequested {
            progress: Some(MaintenanceProgress::new(1, 3)),
        }
    );

    let cancelled = handle.poll();
    assert_eq!(
        cancelled.state,
        MaintenanceJobState::Cancelled(InterruptedMaintenance {
            reason: InterruptionReason::Cancelled,
            preserved_durable_state: DurableStateToken(52),
        })
    );
    assert_eq!(cancelled.polls_used, 1);
}

#[test]
fn timeout_escalation_preserves_prior_durable_state() {
    let operation = ScriptedMaintenance::new(
        vec![
            MaintenanceStep::Pending(MaintenanceProgress::new(1, 5)),
            MaintenanceStep::Pending(MaintenanceProgress::new(2, 5)),
            MaintenanceStep::Completed("too late"),
        ],
        DurableStateToken(63),
    );
    let mut handle = MaintenanceJobHandle::new(operation, 2);

    let first = handle.poll();
    assert_eq!(first.polls_used, 1);
    let second = handle.poll();
    assert_eq!(second.polls_used, 2);

    let timed_out = handle.poll();
    assert_eq!(
        timed_out.state,
        MaintenanceJobState::TimedOut(InterruptedMaintenance {
            reason: InterruptionReason::TimedOut,
            preserved_durable_state: DurableStateToken(63),
        })
    );
    assert_eq!(timed_out.polls_used, 2);
}

#[test]
fn blocked_and_degraded_results_stay_operator_visible() {
    let blocked = ScriptedMaintenance::new(
        vec![MaintenanceStep::Blocked("snapshot_required")],
        DurableStateToken(70),
    );
    let mut blocked_handle = MaintenanceJobHandle::new(blocked, 2);
    let blocked_snapshot = blocked_handle.poll();
    assert_eq!(
        blocked_snapshot.state,
        MaintenanceJobState::Blocked("snapshot_required")
    );

    let degraded = ScriptedMaintenance::new(
        vec![MaintenanceStep::Degraded("foreground_latency_guard")],
        DurableStateToken(71),
    );
    let mut degraded_handle = MaintenanceJobHandle::new(degraded, 2);
    let degraded_snapshot = degraded_handle.poll();
    assert_eq!(
        degraded_snapshot.state,
        MaintenanceJobState::Degraded("foreground_latency_guard")
    );
}

#[test]
fn cancel_after_explicit_start_preserves_prior_durable_state_without_polling_work() {
    let operation = ScriptedMaintenance::new(
        vec![MaintenanceStep::Completed("should never run")],
        DurableStateToken(72),
    );
    let mut handle = MaintenanceJobHandle::new(operation, 3);

    let started = handle.start();
    assert_eq!(
        started.state,
        MaintenanceJobState::Running { progress: None }
    );
    assert_eq!(started.polls_used, 0);

    let requested = handle.cancel();
    assert_eq!(
        requested.state,
        MaintenanceJobState::CancelRequested { progress: None }
    );

    let cancelled = handle.poll();
    assert_eq!(
        cancelled.state,
        MaintenanceJobState::Cancelled(InterruptedMaintenance {
            reason: InterruptionReason::Cancelled,
            preserved_durable_state: DurableStateToken(72),
        })
    );
    assert_eq!(cancelled.polls_used, 0);
}

#[test]
fn terminal_states_remain_stable_across_repeated_control_calls() {
    let operation = ScriptedMaintenance::new(
        vec![MaintenanceStep::Completed("done")],
        DurableStateToken(73),
    );
    let mut handle = MaintenanceJobHandle::new(operation, 2);

    let completed = handle.poll();
    assert_eq!(completed.state, MaintenanceJobState::Completed("done"));
    assert_eq!(completed.polls_used, 1);

    assert_eq!(handle.start(), completed);
    assert_eq!(handle.poll(), completed);
    assert_eq!(handle.cancel(), completed);
}

#[test]
fn repeated_cancel_requests_stay_stable_until_poll_finalizes_cancellation() {
    let operation = ScriptedMaintenance::new(
        vec![MaintenanceStep::Pending(MaintenanceProgress::new(1, 2))],
        DurableStateToken(74),
    );
    let mut handle = MaintenanceJobHandle::new(operation, 3);

    let running = handle.poll();
    assert_eq!(
        running.state,
        MaintenanceJobState::Running {
            progress: Some(MaintenanceProgress::new(1, 2)),
        }
    );
    assert_eq!(running.polls_used, 1);

    let first_cancel = handle.cancel();
    assert_eq!(
        first_cancel.state,
        MaintenanceJobState::CancelRequested {
            progress: Some(MaintenanceProgress::new(1, 2)),
        }
    );
    assert_eq!(first_cancel.polls_used, 1);

    let second_cancel = handle.cancel();
    assert_eq!(second_cancel, first_cancel);

    let cancelled = handle.poll();
    assert_eq!(
        cancelled.state,
        MaintenanceJobState::Cancelled(InterruptedMaintenance {
            reason: InterruptionReason::Cancelled,
            preserved_durable_state: DurableStateToken(74),
        })
    );
    assert_eq!(cancelled.polls_used, 1);
}
