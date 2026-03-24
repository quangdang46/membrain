use std::fmt::Debug;

/// Stable durable-state generation preserved across interrupted maintenance work.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct DurableStateToken(pub u64);

/// Structured progress snapshot for bounded maintenance work.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MaintenanceProgress {
    /// Units of work completed so far.
    pub completed_units: u32,
    /// Total planned units for the bounded run.
    pub total_units: u32,
}

impl MaintenanceProgress {
    /// Builds a bounded progress snapshot.
    pub const fn new(completed_units: u32, total_units: u32) -> Self {
        Self {
            completed_units,
            total_units,
        }
    }
}

/// Deterministic logical clock used by lifecycle and timeout fixtures.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct LogicalClock {
    current_tick: u64,
}

impl LogicalClock {
    /// Builds a logical clock anchored at the given starting tick.
    pub const fn new(current_tick: u64) -> Self {
        Self { current_tick }
    }

    /// Returns the current logical tick.
    pub const fn current_tick(&self) -> u64 {
        self.current_tick
    }

    /// Advances the clock by a bounded number of ticks.
    pub fn advance_by(&mut self, delta: u64) -> u64 {
        self.current_tick = self.current_tick.saturating_add(delta);
        self.current_tick
    }

    /// Advances the clock to an explicit tick without allowing backwards motion.
    pub fn advance_to(&mut self, tick: u64) -> u64 {
        self.current_tick = self.current_tick.max(tick);
        self.current_tick
    }
}

/// Replayable artifact that names the deterministic tick scenario and its history.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TickScenarioArtifact {
    /// Stable scenario name emitted by the fixture owner.
    pub scenario_name: &'static str,
    /// Starting logical tick for the scenario.
    pub start_tick: u64,
    /// Exact replayable tick sequence captured by the fixture.
    pub tick_history: Vec<u64>,
}

/// Replayable fixture that records the exact logical tick sequence for a scenario.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TickSequenceFixture {
    clock: LogicalClock,
    history: Vec<u64>,
}

impl TickSequenceFixture {
    /// Builds a fixture anchored at the given starting tick.
    pub fn new(start_tick: u64) -> Self {
        Self {
            clock: LogicalClock::new(start_tick),
            history: vec![start_tick],
        }
    }

    /// Returns the current logical tick.
    pub const fn current_tick(&self) -> u64 {
        self.clock.current_tick()
    }

    /// Returns the replayable tick history captured so far.
    pub fn history(&self) -> &[u64] {
        &self.history
    }

    /// Returns a named replay artifact for later deterministic assertions.
    pub fn artifact(&self, scenario_name: &'static str) -> TickScenarioArtifact {
        TickScenarioArtifact {
            scenario_name,
            start_tick: self.history.first().copied().unwrap_or(0),
            tick_history: self.history.clone(),
        }
    }

    /// Advances by a bounded number of ticks and records the new position.
    pub fn advance_by(&mut self, delta: u64) -> u64 {
        let tick = self.clock.advance_by(delta);
        self.history.push(tick);
        tick
    }

    /// Advances to an explicit tick and records the new position.
    pub fn advance_to(&mut self, tick: u64) -> u64 {
        let tick = self.clock.advance_to(tick);
        self.history.push(tick);
        tick
    }
}

/// Stable interruption reasons shared by cancellable maintenance work.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InterruptionReason {
    Cancelled,
    TimedOut,
}

/// Durable-state preservation summary returned after an interrupted run.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct InterruptedMaintenance {
    /// Why the maintenance run stopped before completion.
    pub reason: InterruptionReason,
    /// Prior durable state that remained authoritative after interruption.
    pub preserved_durable_state: DurableStateToken,
}

/// One bounded execution step produced by a maintenance operation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MaintenanceStep<S> {
    /// Work is still in flight and may be polled again.
    Pending(MaintenanceProgress),
    /// Work finished successfully with an operator-visible summary.
    Completed(S),
    /// Work could not proceed without operator-visible blocking state.
    Blocked(&'static str),
    /// Work proceeded in a lower-fidelity mode that must remain inspectable.
    Degraded(&'static str),
}

/// Shared operation contract for bounded maintenance jobs.
pub trait MaintenanceOperation {
    /// Operator-visible completion summary for this maintenance family.
    type Summary: Clone + Debug + PartialEq + Eq;

    /// Executes one bounded unit of maintenance work.
    fn poll_step(&mut self) -> MaintenanceStep<Self::Summary>;

    /// Finalizes interruption while preserving the last authoritative durable state.
    fn interrupt(&mut self, reason: InterruptionReason) -> InterruptedMaintenance;
}

/// Stable job states exposed to schedulers and operator surfaces.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MaintenanceJobState<S> {
    Ready,
    Running {
        progress: Option<MaintenanceProgress>,
    },
    CancelRequested {
        progress: Option<MaintenanceProgress>,
    },
    Completed(S),
    Cancelled(InterruptedMaintenance),
    TimedOut(InterruptedMaintenance),
    Blocked(&'static str),
    Degraded(&'static str),
}

/// Snapshot returned after every start, poll, cancel, or inspection action.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MaintenanceJobSnapshot<S> {
    /// Stable state machine view after the control action.
    pub state: MaintenanceJobState<S>,
    /// Number of poll attempts already consumed.
    pub polls_used: u32,
    /// Maximum poll attempts allowed before timeout escalation.
    pub max_polls: u32,
}

/// Shared controller trait for later schedulers and maintenance wrappers.
pub trait MaintenanceController<S>
where
    S: Clone + Debug + PartialEq + Eq,
{
    fn start(&mut self) -> MaintenanceJobSnapshot<S>;
    fn poll(&mut self) -> MaintenanceJobSnapshot<S>;
    fn cancel(&mut self) -> MaintenanceJobSnapshot<S>;
    fn snapshot(&self) -> MaintenanceJobSnapshot<S>;
}

/// Cancellable bounded job handle shared by maintenance operations.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MaintenanceJobHandle<O>
where
    O: MaintenanceOperation,
{
    operation: O,
    state: MaintenanceJobState<O::Summary>,
    polls_used: u32,
    max_polls: u32,
}

impl<O> MaintenanceJobHandle<O>
where
    O: MaintenanceOperation,
{
    /// Builds a new bounded maintenance job handle.
    pub fn new(operation: O, max_polls: u32) -> Self {
        Self {
            operation,
            state: MaintenanceJobState::Ready,
            polls_used: 0,
            max_polls,
        }
    }

    /// Returns a shared reference to the underlying operation.
    pub fn operation(&self) -> &O {
        &self.operation
    }

    fn snapshot_with_state(
        &self,
        state: MaintenanceJobState<O::Summary>,
    ) -> MaintenanceJobSnapshot<O::Summary> {
        MaintenanceJobSnapshot {
            state,
            polls_used: self.polls_used,
            max_polls: self.max_polls,
        }
    }

    fn finalize_interruption(
        &mut self,
        reason: InterruptionReason,
    ) -> MaintenanceJobSnapshot<O::Summary> {
        let interrupted = self.operation.interrupt(reason);
        self.state = match reason {
            InterruptionReason::Cancelled => MaintenanceJobState::Cancelled(interrupted),
            InterruptionReason::TimedOut => MaintenanceJobState::TimedOut(interrupted),
        };
        self.snapshot()
    }

    fn terminal_snapshot(&self) -> Option<MaintenanceJobSnapshot<O::Summary>> {
        match &self.state {
            MaintenanceJobState::Completed(_)
            | MaintenanceJobState::Cancelled(_)
            | MaintenanceJobState::TimedOut(_)
            | MaintenanceJobState::Blocked(_)
            | MaintenanceJobState::Degraded(_) => Some(self.snapshot()),
            _ => None,
        }
    }
}

impl<O> MaintenanceController<O::Summary> for MaintenanceJobHandle<O>
where
    O: MaintenanceOperation,
{
    fn start(&mut self) -> MaintenanceJobSnapshot<O::Summary> {
        if let Some(snapshot) = self.terminal_snapshot() {
            return snapshot;
        }

        self.state = match self.state {
            MaintenanceJobState::Ready => MaintenanceJobState::Running { progress: None },
            _ => self.state.clone(),
        };
        self.snapshot()
    }

    fn poll(&mut self) -> MaintenanceJobSnapshot<O::Summary> {
        if let Some(snapshot) = self.terminal_snapshot() {
            return snapshot;
        }

        if matches!(self.state, MaintenanceJobState::Ready) {
            self.state = MaintenanceJobState::Running { progress: None };
        }

        if matches!(self.state, MaintenanceJobState::CancelRequested { .. }) {
            return self.finalize_interruption(InterruptionReason::Cancelled);
        }

        if self.polls_used >= self.max_polls {
            return self.finalize_interruption(InterruptionReason::TimedOut);
        }

        self.polls_used += 1;
        self.state = match self.operation.poll_step() {
            MaintenanceStep::Pending(progress) => MaintenanceJobState::Running {
                progress: Some(progress),
            },
            MaintenanceStep::Completed(summary) => MaintenanceJobState::Completed(summary),
            MaintenanceStep::Blocked(reason) => MaintenanceJobState::Blocked(reason),
            MaintenanceStep::Degraded(reason) => MaintenanceJobState::Degraded(reason),
        };
        self.snapshot()
    }

    fn cancel(&mut self) -> MaintenanceJobSnapshot<O::Summary> {
        if let Some(snapshot) = self.terminal_snapshot() {
            return snapshot;
        }

        match &self.state {
            MaintenanceJobState::Ready => self.finalize_interruption(InterruptionReason::Cancelled),
            MaintenanceJobState::Running { progress } => {
                self.state = MaintenanceJobState::CancelRequested {
                    progress: *progress,
                };
                self.snapshot()
            }
            MaintenanceJobState::CancelRequested { .. } => self.snapshot(),
            _ => self.snapshot(),
        }
    }

    fn snapshot(&self) -> MaintenanceJobSnapshot<O::Summary> {
        self.snapshot_with_state(self.state.clone())
    }
}
