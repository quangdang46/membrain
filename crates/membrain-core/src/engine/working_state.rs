use crate::api::{
    FieldPresence, GoalAbandonOutput, GoalPauseOutput, GoalResumeOutput, GoalStateOutput,
    NamespaceId, ResponseWarning, TaskId,
};
use crate::types::{
    BlackboardEvidenceHandle, BlackboardSnapshotArtifact, BlackboardState, GoalCheckpoint,
    GoalLifecycleStatus, GoalStackFrame, MemoryId, SessionId,
};
use std::collections::HashMap;

/// Stable degraded reason surfaced during resume instead of guessing through gaps.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ResumeWarning {
    StaleCheckpoint,
    MissingEvidence,
    PolicyIncompatible,
    PartialDependencies,
    NoCheckpoint,
    NotDormant,
    AlreadyAbandoned,
}

impl ResumeWarning {
    /// Returns the stable machine-readable warning label.
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::StaleCheckpoint => "stale_checkpoint",
            Self::MissingEvidence => "missing_evidence",
            Self::PolicyIncompatible => "policy_incompatible",
            Self::PartialDependencies => "partial_dependencies",
            Self::NoCheckpoint => "no_checkpoint",
            Self::NotDormant => "not_dormant",
            Self::AlreadyAbandoned => "already_abandoned",
        }
    }

    fn detail(self) -> &'static str {
        match self {
            Self::StaleCheckpoint => "checkpoint is stale relative to the active runtime tick",
            Self::MissingEvidence => "some referenced evidence handles are no longer available",
            Self::PolicyIncompatible => {
                "checkpoint scope is incompatible with current policy bindings"
            }
            Self::PartialDependencies => {
                "some pending dependencies changed since the checkpoint was created"
            }
            Self::NoCheckpoint => "no resumability checkpoint exists for this task",
            Self::NotDormant => "goal resume requires a dormant checkpointed task",
            Self::AlreadyAbandoned => "goal was intentionally abandoned and cannot be resumed",
        }
    }

    fn response_warning(self) -> ResponseWarning {
        ResponseWarning::new(self.as_str(), self.detail())
    }
}

/// Mutable task-scoped working-state record owned by the core facade.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GoalWorkingState {
    pub task_id: TaskId,
    pub namespace: NamespaceId,
    pub session_id: Option<SessionId>,
    pub status: GoalLifecycleStatus,
    pub goal_stack: Vec<GoalStackFrame>,
    pub blackboard: BlackboardState,
    pub selected_evidence_handles: Vec<MemoryId>,
    pub pending_dependencies: Vec<String>,
    pub latest_checkpoint: Option<GoalCheckpoint>,
    pub latest_snapshot: Option<BlackboardSnapshotArtifact>,
    pub abandonment_reason: Option<String>,
}

impl GoalWorkingState {
    /// Builds one active working-state record.
    pub fn new(
        task_id: TaskId,
        namespace: NamespaceId,
        session_id: Option<SessionId>,
        goal_stack: Vec<GoalStackFrame>,
        blackboard: BlackboardState,
    ) -> Self {
        Self {
            task_id,
            namespace,
            session_id,
            status: GoalLifecycleStatus::Active,
            goal_stack,
            blackboard,
            selected_evidence_handles: Vec::new(),
            pending_dependencies: Vec::new(),
            latest_checkpoint: None,
            latest_snapshot: None,
            abandonment_reason: None,
        }
    }

    /// Returns the current inspect view without mutating task state.
    pub fn state_output(&self) -> GoalStateOutput {
        GoalStateOutput {
            task_id: FieldPresence::Present(self.task_id.as_str().to_string()),
            status: self.status,
            goal_stack: self.goal_stack.clone(),
            latest_checkpoint: self
                .latest_checkpoint
                .clone()
                .map(FieldPresence::Present)
                .unwrap_or(FieldPresence::Absent),
            blackboard_state: FieldPresence::Present(self.blackboard.clone()),
            namespace: self.namespace.as_str().to_string(),
            authoritative_truth: "durable_memory",
        }
    }
}

/// Bounded working-state engine managing blackboard and resumability artifacts.
#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct WorkingStateEngine {
    states: HashMap<TaskId, GoalWorkingState>,
    next_checkpoint_id: u64,
    next_snapshot_id: u64,
}

impl WorkingStateEngine {
    /// Registers or replaces the working-state record for one task.
    pub fn upsert(&mut self, state: GoalWorkingState) {
        self.states.insert(state.task_id.clone(), state);
    }

    /// Returns the current working-state view for one task when present.
    pub fn goal_state(&self, task_id: &TaskId) -> Option<GoalStateOutput> {
        self.states.get(task_id).map(GoalWorkingState::state_output)
    }

    /// Returns a bounded read-only iterator over all task-scoped working-state records.
    pub fn states(&self) -> impl Iterator<Item = &GoalWorkingState> {
        self.states.values()
    }

    /// Pins one evidence handle in the task blackboard.
    pub fn pin_evidence(
        &mut self,
        task_id: &TaskId,
        memory_id: MemoryId,
    ) -> Option<GoalStateOutput> {
        let state = self.states.get_mut(task_id)?;
        if !state.selected_evidence_handles.contains(&memory_id) {
            state.selected_evidence_handles.push(memory_id);
        }
        if let Some(handle) = state
            .blackboard
            .active_evidence
            .iter_mut()
            .find(|handle| handle.memory_id == memory_id)
        {
            handle.pinned = true;
        } else {
            state
                .blackboard
                .active_evidence
                .push(BlackboardEvidenceHandle::new(memory_id, "selected").pinned());
        }
        Some(state.state_output())
    }

    /// Dismisses one evidence handle from the task blackboard.
    pub fn dismiss_evidence(
        &mut self,
        task_id: &TaskId,
        memory_id: MemoryId,
    ) -> Option<GoalStateOutput> {
        let state = self.states.get_mut(task_id)?;
        state
            .selected_evidence_handles
            .retain(|candidate| *candidate != memory_id);
        state
            .blackboard
            .active_evidence
            .retain(|handle| handle.memory_id != memory_id);
        Some(state.state_output())
    }

    /// Persists a bounded blackboard snapshot and marks the task dormant.
    pub fn pause_goal(
        &mut self,
        task_id: &TaskId,
        note: Option<String>,
        current_tick: u64,
    ) -> Option<GoalPauseOutput> {
        let state = self.states.get_mut(task_id)?;
        self.next_checkpoint_id = self.next_checkpoint_id.max(1);
        let checkpoint = GoalCheckpoint {
            checkpoint_id: format!("goal-checkpoint-{}", self.next_checkpoint_id),
            created_tick: current_tick,
            status: GoalLifecycleStatus::Dormant,
            evidence_handles: state.selected_evidence_handles.clone(),
            pending_dependencies: state.pending_dependencies.clone(),
            blocked_reason: state.blackboard.blocked_reason.clone(),
            blackboard_summary: Some(state.blackboard.current_goal.clone()),
            stale: false,
            namespace: state.namespace.clone(),
            task_id: Some(state.task_id.clone()),
            authoritative_truth: "durable_memory",
        };
        self.next_checkpoint_id = self.next_checkpoint_id.saturating_add(1);
        state.status = GoalLifecycleStatus::Dormant;
        state.latest_checkpoint = Some(checkpoint.clone());

        Some(GoalPauseOutput {
            task_id: FieldPresence::Present(state.task_id.as_str().to_string()),
            status: state.status,
            checkpoint,
            paused_at_tick: current_tick,
            note: note
                .map(FieldPresence::Present)
                .unwrap_or(FieldPresence::Absent),
            namespace: state.namespace.as_str().to_string(),
            authoritative_truth: "durable_memory",
        })
    }

    /// Rehydrates one dormant task from its newest checkpoint and surfaces warnings explicitly.
    pub fn resume_goal(
        &mut self,
        task_id: &TaskId,
        current_tick: u64,
    ) -> Result<GoalResumeOutput, ResumeWarning> {
        let state = self
            .states
            .get_mut(task_id)
            .ok_or(ResumeWarning::NoCheckpoint)?;
        if state.status == GoalLifecycleStatus::Abandoned {
            return Err(ResumeWarning::AlreadyAbandoned);
        }
        if state.status != GoalLifecycleStatus::Dormant {
            return Err(ResumeWarning::NotDormant);
        }
        let mut checkpoint = state
            .latest_checkpoint
            .clone()
            .ok_or(ResumeWarning::NoCheckpoint)?;

        let mut warnings = Vec::new();
        if checkpoint.stale {
            warnings.push(ResumeWarning::StaleCheckpoint);
        }
        if checkpoint.evidence_handles != state.selected_evidence_handles {
            warnings.push(ResumeWarning::MissingEvidence);
        }
        if checkpoint.pending_dependencies != state.pending_dependencies {
            warnings.push(ResumeWarning::PartialDependencies);
        }
        if current_tick > checkpoint.created_tick.saturating_add(100) {
            checkpoint.stale = true;
            warnings.push(ResumeWarning::StaleCheckpoint);
        }

        state.status = if checkpoint.stale {
            GoalLifecycleStatus::Stale
        } else {
            GoalLifecycleStatus::Active
        };
        state.latest_checkpoint = Some(checkpoint.clone());

        Ok(GoalResumeOutput {
            task_id: FieldPresence::Present(state.task_id.as_str().to_string()),
            status: state.status,
            checkpoint: checkpoint.clone(),
            resumed_at_tick: current_tick,
            restored_evidence_handles: checkpoint.evidence_handles.iter().map(|id| id.0).collect(),
            restored_dependencies: checkpoint.pending_dependencies.clone(),
            warnings: warnings
                .into_iter()
                .map(ResumeWarning::response_warning)
                .collect(),
            namespace: state.namespace.as_str().to_string(),
            authoritative_truth: "durable_memory",
        })
    }

    /// Ends one goal intentionally while preserving checkpoint metadata for inspection.
    pub fn abandon_goal(
        &mut self,
        task_id: &TaskId,
        reason: Option<String>,
        current_tick: u64,
    ) -> Option<GoalAbandonOutput> {
        let state = self.states.get_mut(task_id)?;
        state.status = GoalLifecycleStatus::Abandoned;
        state.abandonment_reason = reason.clone();
        Some(GoalAbandonOutput {
            task_id: FieldPresence::Present(state.task_id.as_str().to_string()),
            status: state.status,
            checkpoint: state
                .latest_checkpoint
                .clone()
                .map(FieldPresence::Present)
                .unwrap_or(FieldPresence::Absent),
            abandoned_at_tick: current_tick,
            reason: reason
                .map(FieldPresence::Present)
                .unwrap_or(FieldPresence::Absent),
            namespace: state.namespace.as_str().to_string(),
            authoritative_truth: "durable_memory",
        })
    }

    /// Emits a derived blackboard snapshot artifact for inspect, handoff, or resume support.
    pub fn snapshot_blackboard(
        &mut self,
        task_id: &TaskId,
        note: Option<String>,
        current_tick: u64,
    ) -> Option<BlackboardSnapshotArtifact> {
        let state = self.states.get_mut(task_id)?;
        self.next_snapshot_id = self.next_snapshot_id.max(1);
        let snapshot = BlackboardSnapshotArtifact {
            snapshot_id: format!("blackboard-snapshot-{}", self.next_snapshot_id),
            created_tick: current_tick,
            evidence_handles: state.selected_evidence_handles.clone(),
            note,
            artifact_kind: "blackboard_snapshot",
            authoritative_truth: "durable_memory",
        };
        self.next_snapshot_id = self.next_snapshot_id.saturating_add(1);
        state.latest_snapshot = Some(snapshot.clone());
        Some(snapshot)
    }
}

#[cfg(test)]
mod tests {
    use super::{GoalWorkingState, ResumeWarning, WorkingStateEngine};
    use crate::api::{FieldPresence, NamespaceId, TaskId};
    use crate::types::{
        BlackboardEvidenceHandle, BlackboardState, GoalLifecycleStatus, GoalStackFrame, MemoryId,
    };

    #[test]
    fn pause_resume_and_abandon_preserve_checkpointed_working_state() {
        let mut engine = WorkingStateEngine::default();
        let task_id = TaskId::new("deploy-incident");
        let namespace = NamespaceId::new("tests.goal").unwrap();
        let mut blackboard = BlackboardState::new(
            namespace.clone(),
            Some(task_id.clone()),
            None,
            "restore service",
        );
        blackboard.subgoals = vec!["check alarms".to_string(), "verify rollback".to_string()];
        blackboard.active_evidence = vec![
            BlackboardEvidenceHandle::new(MemoryId(7), "primary"),
            BlackboardEvidenceHandle::new(MemoryId(8), "secondary"),
        ];
        blackboard.active_beliefs = vec!["service can recover".to_string()];
        blackboard.unknowns = vec!["customer impact window".to_string()];
        blackboard.next_action = Some("page on-call".to_string());
        blackboard.blocked_reason = Some("waiting for approver".to_string());

        let mut state = GoalWorkingState::new(
            task_id.clone(),
            namespace.clone(),
            None,
            vec![GoalStackFrame::new("restore service")],
            blackboard,
        );
        state.selected_evidence_handles = vec![MemoryId(7), MemoryId(8)];
        state.pending_dependencies = vec!["approval-ticket".to_string()];
        engine.upsert(state);

        let paused = engine
            .pause_goal(&task_id, Some("waiting for approval".to_string()), 11)
            .expect("pause result");
        assert_eq!(paused.status, GoalLifecycleStatus::Dormant);
        assert_eq!(
            paused.note,
            FieldPresence::Present("waiting for approval".to_string())
        );

        let view = engine.goal_state(&task_id).expect("goal state");
        assert_eq!(view.status, GoalLifecycleStatus::Dormant);

        let resumed = engine.resume_goal(&task_id, 15).expect("resume result");
        assert_eq!(resumed.status, GoalLifecycleStatus::Active);
        assert!(resumed.warnings.is_empty());
        assert_eq!(resumed.restored_evidence_handles, vec![7, 8]);
        assert_eq!(resumed.restored_dependencies, vec!["approval-ticket"]);

        let abandoned = engine
            .abandon_goal(&task_id, Some("rollback superseded".to_string()), 16)
            .expect("abandon result");
        assert_eq!(abandoned.status, GoalLifecycleStatus::Abandoned);
        assert_eq!(
            abandoned.reason,
            FieldPresence::Present("rollback superseded".to_string())
        );
    }

    #[test]
    fn pin_and_dismiss_update_blackboard_without_copying_truth() {
        let mut engine = WorkingStateEngine::default();
        let task_id = TaskId::new("task-1");
        let namespace = NamespaceId::new("tests.blackboard").unwrap();
        let state = GoalWorkingState::new(
            task_id.clone(),
            namespace.clone(),
            None,
            vec![GoalStackFrame::new("triage issue")],
            BlackboardState::new(namespace, Some(task_id.clone()), None, "triage issue"),
        );
        engine.upsert(state);

        let pinned = engine
            .pin_evidence(&task_id, MemoryId(21))
            .expect("pinned view");
        let FieldPresence::Present(blackboard) = pinned.blackboard_state else {
            panic!("expected blackboard state");
        };
        assert_eq!(blackboard.active_evidence.len(), 1);
        assert!(blackboard.active_evidence[0].pinned);

        let dismissed = engine
            .dismiss_evidence(&task_id, MemoryId(21))
            .expect("dismissed view");
        let FieldPresence::Present(blackboard) = dismissed.blackboard_state else {
            panic!("expected blackboard state");
        };
        assert!(blackboard.active_evidence.is_empty());
    }

    #[test]
    fn resume_requires_dormant_checkpointed_state() {
        let mut engine = WorkingStateEngine::default();
        let task_id = TaskId::new("task-2");
        let namespace = NamespaceId::new("tests.resume").unwrap();
        engine.upsert(GoalWorkingState::new(
            task_id.clone(),
            namespace.clone(),
            None,
            vec![GoalStackFrame::new("investigate")],
            BlackboardState::new(namespace, Some(task_id.clone()), None, "investigate"),
        ));

        let error = engine
            .resume_goal(&task_id, 5)
            .expect_err("resume should fail");
        assert_eq!(error, ResumeWarning::NotDormant);
    }
}
