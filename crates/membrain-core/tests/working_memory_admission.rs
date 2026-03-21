use membrain_core::engine::encode::{WorkingMemoryController, WorkingMemoryError};
use membrain_core::observability::AdmissionOutcomeKind;
use membrain_core::types::{WorkingMemoryId, WorkingMemoryItem};
use membrain_core::RuntimeConfig;

fn test_config() -> RuntimeConfig {
    RuntimeConfig {
        tier1_candidate_budget: 128,
        tier2_candidate_budget: 5_000,
        working_memory_capacity: 2,
        working_memory_attention_threshold: 200,
        working_memory_promote_threshold: 700,
        cache_per_family_capacity: 1000,
        prefetch_queue_capacity: 50,
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
