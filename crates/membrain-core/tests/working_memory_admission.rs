use membrain_core::api::{FieldPresence, NamespaceId, TaskId};
use membrain_core::engine::encode::{WorkingMemoryController, WorkingMemoryError};
use membrain_core::observability::{AdmissionOutcomeKind, AuditEventKind};
use membrain_core::types::{
    BlackboardEvidenceHandle, BlackboardState, GoalCheckpoint, GoalLifecycleStatus, GoalStackFrame,
    MemoryId, SessionId, WorkingMemoryId, WorkingMemoryItem,
};
use membrain_core::{BrainStore, GoalWorkingState, RuntimeConfig};

fn test_config() -> RuntimeConfig {
    RuntimeConfig {
        working_memory_capacity: 2,
        ..RuntimeConfig::default()
    }
}

#[test]
fn below_threshold_item_is_discarded_before_buffering() {
    let mut controller = WorkingMemoryController::new(test_config());
    let item = WorkingMemoryItem::new(WorkingMemoryId(1), 150);

    let admission = controller
        .admit(item.clone())
        .expect("admission should succeed");

    assert_eq!(admission.outcome, AdmissionOutcomeKind::Discarded);
    assert_eq!(admission.item, item);
    assert!(admission.promoted_item.is_none());
    assert!(admission.evicted_item.is_none());
    assert_eq!(admission.trace.slot_pressure, 0);
    assert_eq!(admission.trace.threshold, 200);
    assert!(!admission.trace.overflowed);
    assert!(controller.slots().is_empty());
}

#[test]
fn admitted_item_is_buffered_without_durable_write_before_overflow() {
    let mut controller = WorkingMemoryController::new(test_config());
    let item = WorkingMemoryItem::new(WorkingMemoryId(2), 400);

    let admission = controller
        .admit(item.clone())
        .expect("admission should succeed");

    assert_eq!(admission.outcome, AdmissionOutcomeKind::Buffered);
    assert!(admission.promoted_item.is_none());
    assert!(admission.evicted_item.is_none());
    assert_eq!(admission.trace.slot_pressure, 0);
    assert_eq!(admission.trace.threshold, 200);
    assert!(!admission.trace.overflowed);
    assert_eq!(controller.slots(), &[item]);
}

#[test]
fn buffered_admission_reports_pre_decision_slot_pressure() {
    let mut controller = WorkingMemoryController::new(test_config());
    controller
        .admit(WorkingMemoryItem::new(WorkingMemoryId(60), 400))
        .expect("seed item should admit");

    let admission = controller
        .admit(WorkingMemoryItem::new(WorkingMemoryId(61), 450))
        .expect("second buffered item should admit");

    assert_eq!(admission.outcome, AdmissionOutcomeKind::Buffered);
    assert_eq!(admission.trace.slot_pressure, 1);
    assert_eq!(admission.trace.threshold, 200);
    assert!(!admission.trace.overflowed);
}

#[test]
fn overflow_promotes_evicted_candidate_when_threshold_is_met() {
    let mut controller = WorkingMemoryController::new(test_config());
    controller
        .admit(WorkingMemoryItem::new(WorkingMemoryId(10), 800))
        .expect("seed item should admit");
    controller
        .admit(WorkingMemoryItem::new(WorkingMemoryId(11), 750))
        .expect("seed item should admit");

    let incoming = WorkingMemoryItem::new(WorkingMemoryId(12), 450);
    let admission = controller
        .admit(incoming.clone())
        .expect("overflow should resolve");

    assert_eq!(admission.outcome, AdmissionOutcomeKind::Promoted);
    assert_eq!(
        admission.promoted_item,
        Some(WorkingMemoryItem::new(WorkingMemoryId(11), 750))
    );
    assert_eq!(
        admission.evicted_item,
        Some(WorkingMemoryItem::new(WorkingMemoryId(11), 750))
    );
    assert!(admission.trace.overflowed);
    assert_eq!(admission.trace.slot_pressure, 2);
    assert_eq!(admission.trace.threshold, 700);
    assert_eq!(controller.slots().len(), 2);
    assert!(controller
        .slots()
        .contains(&WorkingMemoryItem::new(WorkingMemoryId(10), 800)));
    assert!(controller.slots().contains(&incoming));
}

#[test]
fn overflow_discards_evicted_candidate_when_promotion_threshold_is_not_met() {
    let mut controller = WorkingMemoryController::new(test_config());
    controller
        .admit(WorkingMemoryItem::new(WorkingMemoryId(20), 300))
        .expect("seed item should admit");
    controller
        .admit(WorkingMemoryItem::new(WorkingMemoryId(21), 250))
        .expect("seed item should admit");

    let incoming = WorkingMemoryItem::new(WorkingMemoryId(22), 600);
    let admission = controller
        .admit(incoming.clone())
        .expect("overflow should resolve");

    assert_eq!(admission.outcome, AdmissionOutcomeKind::Buffered);
    assert!(admission.promoted_item.is_none());
    assert_eq!(
        admission.evicted_item,
        Some(WorkingMemoryItem::new(WorkingMemoryId(21), 250))
    );
    assert!(controller
        .slots()
        .contains(&WorkingMemoryItem::new(WorkingMemoryId(20), 300)));
    assert!(controller.slots().contains(&incoming));
}

#[test]
fn pinned_items_are_not_evicted_during_overflow() {
    let mut controller = WorkingMemoryController::new(test_config());
    let pinned = WorkingMemoryItem::new(WorkingMemoryId(30), 250).pinned();
    controller
        .admit(pinned.clone())
        .expect("pinned item should admit");
    controller
        .admit(WorkingMemoryItem::new(WorkingMemoryId(31), 300))
        .expect("second item should admit");

    let incoming = WorkingMemoryItem::new(WorkingMemoryId(32), 500);
    let admission = controller
        .admit(incoming.clone())
        .expect("overflow should resolve");

    assert_eq!(
        admission.evicted_item,
        Some(WorkingMemoryItem::new(WorkingMemoryId(31), 300))
    );
    assert!(controller.slots().contains(&pinned));
    assert!(controller.slots().contains(&incoming));
}

#[test]
fn overflow_evicts_lowest_id_when_attention_scores_tie() {
    let mut controller = WorkingMemoryController::new(test_config());
    controller
        .admit(WorkingMemoryItem::new(WorkingMemoryId(71), 450))
        .expect("first tied item should admit");
    controller
        .admit(WorkingMemoryItem::new(WorkingMemoryId(70), 450))
        .expect("second tied item should admit");

    let incoming = WorkingMemoryItem::new(WorkingMemoryId(72), 500);
    let admission = controller
        .admit(incoming.clone())
        .expect("overflow should resolve deterministically");

    assert_eq!(admission.outcome, AdmissionOutcomeKind::Buffered);
    assert_eq!(
        admission.evicted_item,
        Some(WorkingMemoryItem::new(WorkingMemoryId(70), 450))
    );
    assert!(admission.promoted_item.is_none());
    assert!(controller
        .slots()
        .contains(&WorkingMemoryItem::new(WorkingMemoryId(71), 450)));
    assert!(controller.slots().contains(&incoming));
    assert!(!controller
        .slots()
        .contains(&WorkingMemoryItem::new(WorkingMemoryId(70), 450)));
}

#[test]
fn all_pinned_slots_block_overflow_admission() {
    let mut controller = WorkingMemoryController::new(test_config());
    controller
        .admit(WorkingMemoryItem::new(WorkingMemoryId(40), 400).pinned())
        .expect("pinned item should admit");
    controller
        .admit(WorkingMemoryItem::new(WorkingMemoryId(41), 450).pinned())
        .expect("pinned item should admit");

    let err = controller
        .admit(WorkingMemoryItem::new(WorkingMemoryId(42), 500))
        .expect_err("all pinned slots should block overflow");

    assert_eq!(err, WorkingMemoryError::AllSlotsPinned);
    assert_eq!(controller.slots().len(), 2);
}

#[test]
fn focus_raises_attention_for_buffered_candidates() {
    let mut controller = WorkingMemoryController::new(test_config());
    controller
        .admit(WorkingMemoryItem::new(WorkingMemoryId(50), 400))
        .expect("seed item should admit");

    assert!(controller.focus(WorkingMemoryId(50), 50));
    assert_eq!(controller.slots()[0].attention_score, 450);
    assert!(!controller.focus(WorkingMemoryId(999), 50));
}

#[test]
fn blackboard_state_stays_projection_not_authoritative_truth() {
    let namespace = NamespaceId::new("team.alpha").expect("namespace should validate");
    let mut blackboard = BlackboardState::new(
        namespace.clone(),
        Some(TaskId::new("task-42")),
        Some(SessionId(7)),
        "stabilize deploy incident",
    );
    blackboard.active_evidence = vec![BlackboardEvidenceHandle::new(
        MemoryId(9),
        "selected_evidence",
    )];
    blackboard.unknowns = vec!["waiting on customer confirmation".to_string()];

    assert_eq!(blackboard.namespace, namespace);
    assert_eq!(blackboard.projection_kind, "working_state_projection");
    assert_eq!(blackboard.authoritative_truth, "durable_memory");
    assert_eq!(blackboard.active_evidence[0].memory_id, MemoryId(9));
}

#[test]
fn goal_checkpoint_preserves_handles_without_copying_authority() {
    let checkpoint = GoalCheckpoint {
        checkpoint_id: "goal-checkpoint-team.alpha-task-42".to_string(),
        created_tick: 42,
        status: GoalLifecycleStatus::Dormant,
        evidence_handles: vec![MemoryId(1), MemoryId(2)],
        pending_dependencies: vec!["dependency:review".to_string()],
        blocked_reason: Some("waiting_for_review".to_string()),
        blackboard_summary: Some("goal=deploy fix".to_string()),
        stale: false,
        namespace: NamespaceId::new("team.alpha").expect("namespace should validate"),
        task_id: Some(TaskId::new("task-42")),
        authoritative_truth: "durable_memory",
    };

    assert_eq!(checkpoint.status, GoalLifecycleStatus::Dormant);
    assert_eq!(checkpoint.evidence_handles, vec![MemoryId(1), MemoryId(2)]);
    assert_eq!(checkpoint.authoritative_truth, "durable_memory");
    assert!(!checkpoint.stale);
}

#[test]
fn public_brain_store_goal_lifecycle_preserves_projection_and_audit() {
    let mut store = BrainStore::default();
    let namespace = NamespaceId::new("team.alpha").expect("namespace should validate");
    let task_id = TaskId::new("task-42");
    let mut blackboard = BlackboardState::new(
        namespace.clone(),
        Some(task_id.clone()),
        Some(SessionId(7)),
        "stabilize deploy incident",
    );
    blackboard.active_evidence = vec![BlackboardEvidenceHandle::new(
        MemoryId(9),
        "selected_evidence",
    )];
    blackboard.unknowns = vec!["waiting on customer confirmation".to_string()];
    blackboard.blocked_reason = Some("waiting_for_review".to_string());

    let mut state = GoalWorkingState::new(
        task_id.clone(),
        namespace.clone(),
        Some(SessionId(7)),
        vec![GoalStackFrame::new("stabilize deploy incident")],
        blackboard,
    );
    state.selected_evidence_handles = vec![MemoryId(9)];
    state.pending_dependencies = vec!["dependency:review".to_string()];
    store.upsert_goal_working_state(state);

    let initial = store.goal_state(&task_id).expect("goal state should exist");
    assert_eq!(initial.status, GoalLifecycleStatus::Active);
    let FieldPresence::Present(blackboard) = initial.blackboard_state else {
        panic!("expected blackboard state")
    };
    assert_eq!(blackboard.projection_kind, "working_state_projection");
    assert_eq!(blackboard.authoritative_truth, "durable_memory");

    let pinned = store
        .blackboard_pin(&task_id, MemoryId(12))
        .expect("pin should succeed");
    let FieldPresence::Present(blackboard) = pinned.blackboard_state else {
        panic!("expected blackboard state after pin")
    };
    assert!(blackboard
        .active_evidence
        .iter()
        .any(|handle| handle.memory_id == MemoryId(12) && handle.pinned));

    let paused = store
        .goal_pause(&task_id, Some("waiting for review".to_string()))
        .expect("pause should succeed");
    assert_eq!(paused.status, GoalLifecycleStatus::Dormant);
    assert_eq!(paused.checkpoint.status, GoalLifecycleStatus::Dormant);
    assert_eq!(paused.checkpoint.authoritative_truth, "durable_memory");

    let resumed = store.goal_resume(&task_id).expect("resume should succeed");
    assert_eq!(resumed.status, GoalLifecycleStatus::Active);
    assert!(resumed.restored_evidence_handles.contains(&9));
    assert!(resumed.restored_evidence_handles.contains(&12));
    assert!(resumed.warnings.is_empty());

    let abandoned = store
        .goal_abandon(&task_id, Some("rollback superseded".to_string()))
        .expect("abandon should succeed");
    assert_eq!(abandoned.status, GoalLifecycleStatus::Abandoned);
    let FieldPresence::Present(checkpoint) = abandoned.checkpoint else {
        panic!("expected checkpoint on abandon")
    };
    assert_eq!(checkpoint.authoritative_truth, "durable_memory");

    let entries = store.audit_entries();
    assert!(entries.iter().any(|entry| {
        entry.kind == AuditEventKind::IncidentRecorded
            && entry.detail.contains("paused task=task-42")
    }));
    assert!(entries.iter().any(|entry| {
        entry.kind == AuditEventKind::IncidentRecorded
            && entry.detail.contains("resumed task=task-42")
    }));
    assert!(entries.iter().any(|entry| {
        entry.kind == AuditEventKind::IncidentRecorded
            && entry.detail.contains("abandoned task=task-42")
    }));
}

#[test]
fn public_brain_store_goal_resume_surfaces_explicit_degraded_checkpoint_warnings() {
    let mut store = BrainStore::default();
    let namespace = NamespaceId::new("team.alpha").expect("namespace should validate");
    let task_id = TaskId::new("task-99");
    let mut blackboard = BlackboardState::new(
        namespace.clone(),
        Some(task_id.clone()),
        Some(SessionId(9)),
        "stabilize deploy incident",
    );
    blackboard.active_evidence = vec![BlackboardEvidenceHandle::new(
        MemoryId(1),
        "selected_evidence",
    )];
    blackboard.blocked_reason = Some("waiting_for_review".to_string());

    let mut state = GoalWorkingState::new(
        task_id.clone(),
        namespace.clone(),
        Some(SessionId(9)),
        vec![GoalStackFrame::new("stabilize deploy incident")],
        blackboard,
    );
    state.selected_evidence_handles = vec![MemoryId(1)];
    state.pending_dependencies = vec!["dependency:review".to_string()];
    store.upsert_goal_working_state(state);

    let paused = store
        .goal_pause(&task_id, Some("waiting for review".to_string()))
        .expect("pause should succeed");

    let mut degraded_blackboard = BlackboardState::new(
        namespace.clone(),
        Some(task_id.clone()),
        Some(SessionId(9)),
        "stabilize deploy incident",
    );
    degraded_blackboard.active_evidence = vec![BlackboardEvidenceHandle::new(
        MemoryId(5),
        "selected_evidence",
    )];
    degraded_blackboard.blocked_reason = Some("waiting_for_review".to_string());

    let mut degraded_state = GoalWorkingState::new(
        task_id.clone(),
        namespace,
        Some(SessionId(9)),
        vec![GoalStackFrame::new("stabilize deploy incident")],
        degraded_blackboard,
    );
    degraded_state.status = GoalLifecycleStatus::Dormant;
    degraded_state.selected_evidence_handles = vec![MemoryId(5)];
    degraded_state.pending_dependencies = vec!["dependency:security-review".to_string()];
    degraded_state.latest_checkpoint = Some(GoalCheckpoint {
        stale: true,
        ..paused.checkpoint.clone()
    });
    store.upsert_goal_working_state(degraded_state);

    let resumed = store
        .goal_resume(&task_id)
        .expect("resume should return degraded output");
    assert_eq!(resumed.status, GoalLifecycleStatus::Stale);
    assert_eq!(resumed.checkpoint.authoritative_truth, "durable_memory");
    assert_eq!(resumed.restored_evidence_handles, vec![1]);
    assert_eq!(resumed.restored_dependencies, vec!["dependency:review"]);
    assert!(resumed
        .warnings
        .iter()
        .any(|warning| warning.code == "stale_checkpoint"));
    assert!(resumed
        .warnings
        .iter()
        .any(|warning| warning.code == "missing_evidence"));
    assert!(resumed
        .warnings
        .iter()
        .any(|warning| warning.code == "partial_dependencies"));

    let entries = store.audit_entries();
    assert!(entries.iter().any(|entry| {
        entry.kind == AuditEventKind::IncidentRecorded
            && entry.detail.contains("resumed task=task-99")
            && entry.detail.contains("warnings=3")
    }));
}

#[test]
fn fork_metadata_reports_isolation_and_divergence_from_fork_local_state() {
    let mut store = BrainStore::default();
    let parent_namespace = NamespaceId::new("team.alpha").expect("namespace should validate");
    let fork = store.fork(membrain_core::brain_store::ForkConfig {
        name: "agent-specialist".to_string(),
        parent_namespace: parent_namespace.clone(),
        inherit_visibility: membrain_core::api::ForkInheritance::SharedToo,
        note: Some("testing".to_string()),
    });

    assert_eq!(fork.parent_namespace, "team.alpha");
    assert_eq!(fork.namespace, "agent-specialist");
    assert_eq!(
        fork.isolation_semantics,
        "inherit_by_reference_until_explicit_merge"
    );
    assert_eq!(fork.divergence_basis, "fork_namespace_local_state");
    assert!(!fork.diverged);
    assert_eq!(fork.fork_local_procedure_count, 0);
    assert_eq!(fork.fork_working_state_count, 0);

    let task_id = TaskId::new("fork-task");
    let fork_namespace =
        NamespaceId::new("agent-specialist").expect("fork namespace should validate");
    let state = GoalWorkingState::new(
        task_id,
        fork_namespace.clone(),
        Some(SessionId(9)),
        vec![GoalStackFrame::new("compare branch outputs")],
        BlackboardState::new(
            fork_namespace,
            None,
            Some(SessionId(9)),
            "compare branch outputs",
        ),
    );
    store.upsert_goal_working_state(state);

    let listed = store.list_forks();
    assert_eq!(listed.len(), 1);
    assert_eq!(listed[0].fork_working_state_count, 1);
    assert!(listed[0].diverged);

    let merge = store
        .merge_fork(membrain_core::brain_store::MergeConfig {
            fork_name: "agent-specialist".to_string(),
            target_namespace: parent_namespace,
            conflict_strategy: membrain_core::api::MergeConflictStrategy::Manual,
            dry_run: true,
        })
        .expect("merge report should exist");
    assert!(merge.divergence_detected);
    assert_eq!(merge.fork_working_state_count, 1);
    assert_eq!(merge.fork_local_procedure_count, 0);
    assert_eq!(
        merge.isolation_semantics,
        "inherit_by_reference_until_explicit_merge"
    );
}
