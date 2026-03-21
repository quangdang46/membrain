use std::cell::RefCell;
use std::rc::Rc;

use membrain_core::api::NamespaceId;
use membrain_core::engine::maintenance::{
    DurableStateToken, InterruptedMaintenance, InterruptionReason, MaintenanceController,
    MaintenanceJobHandle, MaintenanceJobState, MaintenanceOperation, MaintenanceProgress,
    MaintenanceStep,
};
use membrain_core::engine::repair::{
    IndexRepairEntrypoint, RepairEngine, RepairStatus, RepairTarget,
};
use membrain_core::migrate::DurableSchemaObject;

#[derive(Debug, Clone)]
struct ScriptedMaintenance {
    steps: Vec<MaintenanceStep<&'static str>>,
    next_step: usize,
    preserved: DurableStateToken,
    interruptions: Rc<RefCell<Vec<InterruptionReason>>>,
}

impl ScriptedMaintenance {
    fn new(steps: Vec<MaintenanceStep<&'static str>>, preserved: DurableStateToken) -> Self {
        Self::with_interruptions(steps, preserved, Rc::new(RefCell::new(Vec::new())))
    }

    fn with_interruptions(
        steps: Vec<MaintenanceStep<&'static str>>,
        preserved: DurableStateToken,
        interruptions: Rc<RefCell<Vec<InterruptionReason>>>,
    ) -> Self {
        Self {
            steps,
            next_step: 0,
            preserved,
            interruptions,
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
        self.interruptions.borrow_mut().push(reason);
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
    let interruptions = Rc::new(RefCell::new(Vec::new()));
    let operation = ScriptedMaintenance::with_interruptions(
        vec![
            MaintenanceStep::Pending(MaintenanceProgress::new(1, 3)),
            MaintenanceStep::Completed("should not complete"),
        ],
        DurableStateToken(52),
        interruptions.clone(),
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
    assert_eq!(&*interruptions.borrow(), &[InterruptionReason::Cancelled]);

    assert_eq!(handle.start(), cancelled);
    assert_eq!(handle.poll(), cancelled);
    assert_eq!(handle.cancel(), cancelled);
    assert_eq!(&*interruptions.borrow(), &[InterruptionReason::Cancelled]);
}

#[test]
fn timeout_escalation_preserves_prior_durable_state() {
    let interruptions = Rc::new(RefCell::new(Vec::new()));
    let operation = ScriptedMaintenance::with_interruptions(
        vec![
            MaintenanceStep::Pending(MaintenanceProgress::new(1, 5)),
            MaintenanceStep::Pending(MaintenanceProgress::new(2, 5)),
            MaintenanceStep::Completed("too late"),
        ],
        DurableStateToken(63),
        interruptions.clone(),
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
    assert_eq!(&*interruptions.borrow(), &[InterruptionReason::TimedOut]);

    assert_eq!(handle.start(), timed_out);
    assert_eq!(handle.poll(), timed_out);
    assert_eq!(handle.cancel(), timed_out);
    assert_eq!(&*interruptions.borrow(), &[InterruptionReason::TimedOut]);
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
    assert_eq!(blocked_handle.start(), blocked_snapshot);
    assert_eq!(blocked_handle.poll(), blocked_snapshot);
    assert_eq!(blocked_handle.cancel(), blocked_snapshot);

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
    assert_eq!(degraded_handle.start(), degraded_snapshot);
    assert_eq!(degraded_handle.poll(), degraded_snapshot);
    assert_eq!(degraded_handle.cancel(), degraded_snapshot);
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
fn snapshot_reports_state_transitions_without_consuming_extra_work() {
    let operation = ScriptedMaintenance::new(
        vec![MaintenanceStep::Pending(MaintenanceProgress::new(1, 2))],
        DurableStateToken(73),
    );
    let mut handle = MaintenanceJobHandle::new(operation, 3);

    assert_eq!(handle.snapshot().state, MaintenanceJobState::Ready);
    assert_eq!(handle.snapshot().polls_used, 0);

    let started = handle.start();
    assert_eq!(
        started.state,
        MaintenanceJobState::Running { progress: None }
    );
    assert_eq!(started.polls_used, 0);
    assert_eq!(handle.snapshot(), started);

    let running = handle.poll();
    assert_eq!(
        running.state,
        MaintenanceJobState::Running {
            progress: Some(MaintenanceProgress::new(1, 2)),
        }
    );
    assert_eq!(running.polls_used, 1);
    assert_eq!(handle.snapshot(), running);

    let requested = handle.cancel();
    assert_eq!(
        requested.state,
        MaintenanceJobState::CancelRequested {
            progress: Some(MaintenanceProgress::new(1, 2)),
        }
    );
    assert_eq!(requested.polls_used, 1);
    assert_eq!(handle.snapshot(), requested);
}

#[test]
fn terminal_states_remain_stable_across_repeated_control_calls() {
    let operation = ScriptedMaintenance::new(
        vec![MaintenanceStep::Completed("done")],
        DurableStateToken(74),
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
fn interrupted_terminal_states_remain_stable_across_repeated_control_calls() {
    let cancelled_operation = ScriptedMaintenance::new(
        vec![MaintenanceStep::Completed("should never run")],
        DurableStateToken(75),
    );
    let mut cancelled_handle = MaintenanceJobHandle::new(cancelled_operation, 2);
    let cancelled = cancelled_handle.cancel();
    assert_eq!(
        cancelled.state,
        MaintenanceJobState::Cancelled(InterruptedMaintenance {
            reason: InterruptionReason::Cancelled,
            preserved_durable_state: DurableStateToken(75),
        })
    );
    assert_eq!(cancelled_handle.start(), cancelled);
    assert_eq!(cancelled_handle.poll(), cancelled);
    assert_eq!(cancelled_handle.cancel(), cancelled);

    let timed_out_operation = ScriptedMaintenance::new(
        vec![MaintenanceStep::Pending(MaintenanceProgress::new(1, 3))],
        DurableStateToken(76),
    );
    let mut timed_out_handle = MaintenanceJobHandle::new(timed_out_operation, 0);
    let timed_out = timed_out_handle.poll();
    assert_eq!(
        timed_out.state,
        MaintenanceJobState::TimedOut(InterruptedMaintenance {
            reason: InterruptionReason::TimedOut,
            preserved_durable_state: DurableStateToken(76),
        })
    );
    assert_eq!(timed_out_handle.start(), timed_out);
    assert_eq!(timed_out_handle.poll(), timed_out);
    assert_eq!(timed_out_handle.cancel(), timed_out);
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

#[test]
fn repair_runs_expose_verification_artifacts_through_maintenance_handle() {
    let namespace = NamespaceId::new("test").unwrap();
    let engine = RepairEngine;
    let run = engine.create_targeted(
        namespace,
        vec![
            RepairTarget::LexicalIndex,
            RepairTarget::SemanticHotIndex,
            RepairTarget::EngramIndex,
        ],
        IndexRepairEntrypoint::VerifyOnly,
    );
    let mut handle = MaintenanceJobHandle::new(run, 10);

    handle.start();
    loop {
        let snapshot = handle.poll();
        match snapshot.state {
            MaintenanceJobState::Completed(ref summary) => {
                assert_eq!(summary.targets_checked, 3);
                assert_eq!(summary.healthy, 3);
                assert_eq!(summary.verification_artifacts.len(), 3);
                assert_eq!(summary.operator_reports.len(), 3);
                assert!(summary.operator_reports.iter().all(|report| {
                    report.rebuild_entrypoint.is_none() && report.verification_passed
                }));

                let lexical = summary
                    .verification_artifacts
                    .get(&RepairTarget::LexicalIndex)
                    .unwrap();
                assert_eq!(lexical.artifact_name, "fts5_lexical_parity");
                assert_eq!(lexical.authoritative_rows, 128);
                assert_eq!(lexical.derived_rows, 128);
                assert_eq!(lexical.authoritative_generation, "durable.v1");
                assert_eq!(lexical.derived_generation, "durable.v1");
                assert_eq!(
                    lexical.parity_check,
                    "fts5_projection_matches_durable_truth"
                );

                let semantic_hot = summary
                    .verification_artifacts
                    .get(&RepairTarget::SemanticHotIndex)
                    .unwrap();
                assert_eq!(semantic_hot.authoritative_rows, 64);

                let engram = summary
                    .verification_artifacts
                    .get(&RepairTarget::EngramIndex)
                    .unwrap();
                assert_eq!(engram.authoritative_rows, 24);
                break;
            }
            MaintenanceJobState::Running { .. } => continue,
            _ => panic!("unexpected state"),
        }
    }
}

#[test]
fn repair_runs_preserve_index_rebuild_entrypoint_contracts_in_verification_artifacts() {
    let namespace = NamespaceId::new("test").unwrap();
    let engine = RepairEngine;
    let semantic_cold_plan = engine
        .plan_index_rebuild(
            RepairTarget::SemanticColdIndex,
            IndexRepairEntrypoint::ForceRebuild,
        )
        .expect("semantic cold index should expose rebuild plan");
    let metadata_plan = engine
        .plan_index_rebuild(
            RepairTarget::MetadataIndex,
            IndexRepairEntrypoint::RebuildIfNeeded,
        )
        .expect("metadata index should expose rebuild plan");

    let run = engine.create_targeted(
        namespace,
        vec![RepairTarget::SemanticColdIndex, RepairTarget::MetadataIndex],
        IndexRepairEntrypoint::ForceRebuild,
    );
    let mut handle = MaintenanceJobHandle::new(run, 10);

    let started = handle.start();
    assert_eq!(
        started.state,
        MaintenanceJobState::Running { progress: None }
    );

    let first_poll = handle.poll();
    assert_eq!(
        first_poll.state,
        MaintenanceJobState::Running {
            progress: Some(MaintenanceProgress::new(1, 2)),
        }
    );
    assert_eq!(first_poll.polls_used, 1);

    let completed = handle.poll();
    let MaintenanceJobState::Completed(summary) = completed.state else {
        panic!("expected completed repair summary after second poll");
    };
    assert_eq!(completed.polls_used, 2);
    assert_eq!(summary.targets_checked, 2);
    assert_eq!(summary.healthy, 0);
    assert_eq!(summary.rebuilt, 2);
    assert_eq!(summary.results.len(), 2);
    assert_eq!(summary.operator_reports.len(), 2);
    assert!(summary.results.iter().all(|result| {
        result.detail == "rebuilt_from_durable_truth_and_verified"
            && result.verification_passed
            && result.rebuild_entrypoint == Some(IndexRepairEntrypoint::ForceRebuild)
            && !result.rebuilt_outputs.is_empty()
    }));

    let semantic_cold_report = summary
        .operator_reports
        .iter()
        .find(|report| report.target == RepairTarget::SemanticColdIndex)
        .expect("semantic cold report should be present");
    assert_eq!(semantic_cold_report.status.as_str(), "rebuilt");
    assert_eq!(
        semantic_cold_report.rebuild_entrypoint,
        Some(IndexRepairEntrypoint::ForceRebuild)
    );
    assert!(semantic_cold_report
        .rebuilt_outputs
        .contains(&"usearch_cold_ann"));
    assert_eq!(
        semantic_cold_report.verification_artifact_name,
        "usearch_cold_parity"
    );

    let semantic_cold = summary
        .verification_artifacts
        .get(&RepairTarget::SemanticColdIndex)
        .expect("semantic cold artifact should be present");
    assert_eq!(
        semantic_cold.artifact_name,
        semantic_cold_plan.verification_artifact.artifact_name
    );
    assert_eq!(
        semantic_cold.parity_check,
        semantic_cold_plan.verification_artifact.parity_check
    );
    assert!(semantic_cold.verification_passed);
    assert_eq!(
        semantic_cold.authoritative_rows,
        semantic_cold_plan.verification_artifact.authoritative_rows
    );
    assert_eq!(
        semantic_cold.derived_generation,
        semantic_cold_plan.verification_artifact.derived_generation
    );
    assert_eq!(
        semantic_cold_plan.entrypoint,
        IndexRepairEntrypoint::ForceRebuild
    );
    assert!(semantic_cold_plan
        .rebuilt_outputs
        .contains(&"usearch_cold_ann"));
    assert!(semantic_cold_plan
        .durable_sources
        .contains(&"canonical_embeddings"));
    assert_eq!(
        semantic_cold_plan.authoritative_schema_objects,
        vec![DurableSchemaObject::DurableMemoryRecords]
    );

    let metadata_report = summary
        .operator_reports
        .iter()
        .find(|report| report.target == RepairTarget::MetadataIndex)
        .expect("metadata report should be present");
    assert_eq!(metadata_report.status.as_str(), "rebuilt");
    assert_eq!(
        metadata_report.rebuild_entrypoint,
        Some(IndexRepairEntrypoint::ForceRebuild)
    );
    assert!(metadata_report
        .rebuilt_outputs
        .contains(&"tier2_metadata_projection"));
    assert_eq!(
        metadata_report.verification_artifact_name,
        "tier2_metadata_parity"
    );

    let metadata = summary
        .verification_artifacts
        .get(&RepairTarget::MetadataIndex)
        .expect("metadata artifact should be present");
    assert_eq!(
        metadata.artifact_name,
        metadata_plan.verification_artifact.artifact_name
    );
    assert_eq!(
        metadata.parity_check,
        metadata_plan.verification_artifact.parity_check
    );
    assert!(metadata.verification_passed);
    assert_eq!(
        metadata.authoritative_rows,
        metadata_plan.verification_artifact.authoritative_rows
    );
    assert_eq!(
        metadata_plan.entrypoint,
        IndexRepairEntrypoint::RebuildIfNeeded
    );
    assert!(metadata_plan
        .rebuilt_outputs
        .contains(&"tier2_metadata_projection"));
    assert!(metadata_plan
        .durable_sources
        .contains(&"namespace_policy_metadata"));
    assert_eq!(
        metadata_plan.authoritative_schema_objects,
        vec![DurableSchemaObject::DurableMemoryRecords]
    );

    assert_eq!(handle.snapshot().polls_used, 2);
    assert_eq!(handle.start(), handle.snapshot());
    assert_eq!(handle.poll(), handle.snapshot());
    assert_eq!(handle.cancel(), handle.snapshot());
}

#[test]
fn repair_run_full_scan_reports_doctor_health_surfaces_without_rebuilds() {
    let namespace = NamespaceId::new("doctor.system").unwrap();
    let engine = RepairEngine;
    let run = engine.create_full_scan(namespace, IndexRepairEntrypoint::VerifyOnly);
    let mut handle = MaintenanceJobHandle::new(run, 16);

    let started = handle.start();
    assert_eq!(
        started.state,
        MaintenanceJobState::Running { progress: None }
    );

    let completed = loop {
        let snapshot = handle.poll();
        let state = snapshot.state.clone();
        match state {
            MaintenanceJobState::Completed(summary) => break (snapshot, summary),
            MaintenanceJobState::Running { .. } => continue,
            other => panic!("unexpected state: {other:?}"),
        }
    };
    let (snapshot, summary) = completed;

    assert_eq!(summary.targets_checked, 10);
    assert_eq!(summary.healthy, 10);
    assert_eq!(summary.degraded, 0);
    assert_eq!(summary.corrupt, 0);
    assert_eq!(summary.rebuilt, 0);
    assert_eq!(summary.results.len(), 10);
    assert_eq!(summary.operator_reports.len(), 10);
    assert_eq!(summary.verification_artifacts.len(), 10);
    assert_eq!(snapshot.polls_used, 10);

    for target in [
        RepairTarget::LexicalIndex,
        RepairTarget::MetadataIndex,
        RepairTarget::SemanticHotIndex,
        RepairTarget::SemanticColdIndex,
        RepairTarget::HotStoreConsistency,
        RepairTarget::PayloadIntegrity,
        RepairTarget::GraphConsistency,
        RepairTarget::CacheWarmState,
        RepairTarget::EngramIndex,
        RepairTarget::ContradictionConsistency,
    ] {
        let result = summary
            .results
            .iter()
            .find(|result| result.target == target)
            .unwrap_or_else(|| panic!("missing result for {}", target.as_str()));
        assert_eq!(result.status, RepairStatus::Healthy);
        assert!(result.verification_passed);
        assert_eq!(result.detail, "verified_against_durable_truth");
        assert!(result.rebuild_entrypoint.is_none());
        assert!(result.rebuilt_outputs.is_empty());

        let report = summary
            .operator_reports
            .iter()
            .find(|report| report.target == target)
            .unwrap_or_else(|| panic!("missing operator report for {}", target.as_str()));
        assert_eq!(report.status, RepairStatus::Healthy);
        assert!(report.verification_passed);
        assert!(report.rebuild_entrypoint.is_none());
        assert!(report.rebuilt_outputs.is_empty());
        assert_eq!(
            report.verification_artifact_name,
            target.verification_artifact_name()
        );

        let artifact = summary
            .verification_artifacts
            .get(&target)
            .unwrap_or_else(|| panic!("missing artifact for {}", target.as_str()));
        assert_eq!(artifact.artifact_name, target.verification_artifact_name());
        assert_eq!(artifact.parity_check, target.verification_parity_check());
        assert!(artifact.verification_passed);
        assert_eq!(artifact.authoritative_generation, "durable.v1");
        assert_eq!(artifact.derived_generation, "durable.v1");
        assert_eq!(artifact.authoritative_rows, artifact.derived_rows);
    }

    assert_eq!(handle.snapshot().polls_used, 10);
    assert_eq!(handle.start(), handle.snapshot());
    assert_eq!(handle.poll(), handle.snapshot());
    assert_eq!(handle.cancel(), handle.snapshot());
}

#[test]
fn repair_run_full_scan_rebuild_if_needed_keeps_doctor_surfaces_explainable() {
    let namespace = NamespaceId::new("doctor.system").unwrap();
    let engine = RepairEngine;
    let run = engine.create_full_scan(namespace, IndexRepairEntrypoint::RebuildIfNeeded);
    let mut handle = MaintenanceJobHandle::new(run, 16);

    let started = handle.start();
    assert_eq!(
        started.state,
        MaintenanceJobState::Running { progress: None }
    );

    let completed = loop {
        let snapshot = handle.poll();
        let state = snapshot.state.clone();
        match state {
            MaintenanceJobState::Completed(summary) => break (snapshot, summary),
            MaintenanceJobState::Running { .. } => continue,
            other => panic!("unexpected state: {other:?}"),
        }
    };
    let (snapshot, summary) = completed;

    assert_eq!(summary.targets_checked, 10);
    assert_eq!(summary.healthy, 0);
    assert_eq!(summary.degraded, 0);
    assert_eq!(summary.corrupt, 0);
    assert_eq!(summary.rebuilt, 6);
    assert_eq!(summary.results.len(), 10);
    assert_eq!(summary.operator_reports.len(), 10);
    assert_eq!(summary.verification_artifacts.len(), 10);
    assert_eq!(snapshot.polls_used, 10);

    for target in [
        RepairTarget::LexicalIndex,
        RepairTarget::MetadataIndex,
        RepairTarget::SemanticHotIndex,
        RepairTarget::SemanticColdIndex,
        RepairTarget::CacheWarmState,
        RepairTarget::EngramIndex,
    ] {
        let result = summary
            .results
            .iter()
            .find(|result| result.target == target)
            .unwrap_or_else(|| panic!("missing result for {}", target.as_str()));
        assert_eq!(result.status, RepairStatus::Rebuilt);
        assert!(result.verification_passed);
        assert_eq!(result.detail, "rebuilt_from_durable_truth_and_verified");
        assert_eq!(
            result.rebuild_entrypoint,
            Some(IndexRepairEntrypoint::RebuildIfNeeded)
        );
        assert!(!result.rebuilt_outputs.is_empty());

        let report = summary
            .operator_reports
            .iter()
            .find(|report| report.target == target)
            .unwrap_or_else(|| panic!("missing operator report for {}", target.as_str()));
        assert_eq!(report.status, RepairStatus::Rebuilt);
        assert!(report.verification_passed);
        assert_eq!(
            report.rebuild_entrypoint,
            Some(IndexRepairEntrypoint::RebuildIfNeeded)
        );
        assert_eq!(
            report.verification_artifact_name,
            target.verification_artifact_name()
        );
        assert!(!report.rebuilt_outputs.is_empty());
    }

    for target in [
        RepairTarget::HotStoreConsistency,
        RepairTarget::PayloadIntegrity,
        RepairTarget::GraphConsistency,
        RepairTarget::ContradictionConsistency,
    ] {
        let result = summary
            .results
            .iter()
            .find(|result| result.target == target)
            .unwrap_or_else(|| panic!("missing result for {}", target.as_str()));
        assert_eq!(result.status, RepairStatus::Skipped);
        assert!(result.verification_passed);
        assert_eq!(result.detail, "rebuild_not_supported_for_target");
        assert!(result.rebuild_entrypoint.is_none());
        assert!(result.rebuilt_outputs.is_empty());

        let report = summary
            .operator_reports
            .iter()
            .find(|report| report.target == target)
            .unwrap_or_else(|| panic!("missing operator report for {}", target.as_str()));
        assert_eq!(report.status, RepairStatus::Skipped);
        assert!(report.verification_passed);
        assert!(report.rebuild_entrypoint.is_none());
        assert!(report.rebuilt_outputs.is_empty());
        assert_eq!(
            report.verification_artifact_name,
            target.verification_artifact_name()
        );
    }

    for target in [
        RepairTarget::LexicalIndex,
        RepairTarget::MetadataIndex,
        RepairTarget::SemanticHotIndex,
        RepairTarget::SemanticColdIndex,
        RepairTarget::HotStoreConsistency,
        RepairTarget::PayloadIntegrity,
        RepairTarget::GraphConsistency,
        RepairTarget::CacheWarmState,
        RepairTarget::EngramIndex,
        RepairTarget::ContradictionConsistency,
    ] {
        let artifact = summary
            .verification_artifacts
            .get(&target)
            .unwrap_or_else(|| panic!("missing artifact for {}", target.as_str()));
        assert_eq!(artifact.artifact_name, target.verification_artifact_name());
        assert_eq!(artifact.parity_check, target.verification_parity_check());
        assert!(artifact.verification_passed);
        assert_eq!(artifact.authoritative_generation, "durable.v1");
        assert_eq!(artifact.derived_generation, "durable.v1");
        assert_eq!(artifact.authoritative_rows, artifact.derived_rows);
    }

    assert_eq!(handle.snapshot().polls_used, 10);
    assert_eq!(handle.start(), handle.snapshot());
    assert_eq!(handle.poll(), handle.snapshot());
    assert_eq!(handle.cancel(), handle.snapshot());
}
