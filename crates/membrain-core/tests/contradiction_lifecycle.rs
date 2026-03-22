//! Golden lifecycle test for contradiction explain, audit, and lineage surfaces.
//!
//! This test proves the full contradiction inspect flow:
//!   1. Write-path branching produces explicit traces
//!   2. Explain payloads show correct state at each stage
//!   3. Resolution sets preferred answer with confidence
//!   4. Retention/archival preserves audit visibility
//!   5. Legal hold prevents evidence destruction
//!   6. Lineage pairs remain intact through the entire lifecycle

use membrain_core::api::NamespaceId;
use membrain_core::brain_store::BrainStore;
use membrain_core::engine::contradiction::{
    ContradictionError, ContradictionKind, ContradictionStore, PreferredAnswerState,
    ResolutionState,
};
use membrain_core::engine::encode::EncodeWriteBranch;
use membrain_core::types::MemoryId;

fn ns(s: &str) -> NamespaceId {
    NamespaceId::new(s).unwrap()
}

// ── 1. Write-path branching produces explicit traces ─────────────────────────

#[test]
fn write_branch_trace_proves_which_branch_was_taken() {
    let mut store = BrainStore::default();
    let namespace = ns("golden/branch");

    let outcome = store
        .record_encode_contradiction(
            namespace,
            MemoryId(10),
            MemoryId(20),
            ContradictionKind::Revision,
            850,
        )
        .unwrap();

    assert_eq!(outcome.branch, EncodeWriteBranch::ContradictionRecorded);
    assert_eq!(outcome.existing_memory, MemoryId(10));
    assert_eq!(outcome.incoming_memory, MemoryId(20));
    assert_eq!(outcome.kind, ContradictionKind::Revision);
    assert!(outcome.contradiction_id.0 > 0);
}

// ── 2. Explain payloads show correct state at each stage ─────────────────────

#[test]
fn explain_shows_unresolved_state_before_resolution() {
    let mut store = BrainStore::default();
    let namespace = ns("golden/explain");

    store
        .record_encode_contradiction(
            namespace,
            MemoryId(100),
            MemoryId(200),
            ContradictionKind::Coexistence,
            500,
        )
        .unwrap();

    // Explain for memory_a
    let explains_a = store
        .contradiction_engine()
        .explain_for_memory(MemoryId(100));
    assert_eq!(explains_a.len(), 1);

    let exp = &explains_a[0];
    assert_eq!(exp.kind, ContradictionKind::Coexistence);
    assert_eq!(exp.resolution, ResolutionState::Unresolved);
    assert_eq!(exp.preferred_answer_state, PreferredAnswerState::Unset);
    assert_eq!(exp.preferred_memory, None);
    assert_eq!(exp.confidence_signal, 0);
    assert_eq!(exp.conflicting_memory, MemoryId(200));
    assert_eq!(exp.lineage_pair, [MemoryId(100), MemoryId(200)]);
    assert!(!exp.result_is_preferred);
    assert_eq!(exp.superseded_memory, None);
    assert!(exp.resolution_reason.is_none());
    assert!(!exp.active_contradiction);
    assert!(!exp.archived);
    assert!(!exp.legal_hold);
    assert!(exp.authoritative_evidence);
    assert!(exp.audit_visible);
}

#[test]
fn explain_shows_preferred_state_after_resolution() {
    let mut store = BrainStore::default();
    let namespace = ns("golden/resolved");

    let outcome = store
        .record_encode_contradiction(
            namespace,
            MemoryId(100),
            MemoryId(200),
            ContradictionKind::Supersession,
            900,
        )
        .unwrap();

    // Resolve: memory_b supersedes memory_a
    store
        .contradiction_engine_mut()
        .resolve(
            outcome.contradiction_id,
            ResolutionState::AutoResolved,
            MemoryId(200),
            "newer supersedes older",
        )
        .unwrap();

    // Explain for the preferred memory (200)
    let explains_preferred = store
        .contradiction_engine()
        .explain_for_memory(MemoryId(200));
    assert_eq!(explains_preferred.len(), 1);

    let exp = &explains_preferred[0];
    assert_eq!(exp.resolution, ResolutionState::AutoResolved);
    assert_eq!(exp.preferred_answer_state, PreferredAnswerState::Preferred);
    assert_eq!(exp.preferred_memory, Some(MemoryId(200)));
    assert!(exp.confidence_signal > 800); // Supersession + AutoResolved → high confidence
    assert!(exp.result_is_preferred);
    assert_eq!(exp.superseded_memory, Some(MemoryId(100)));
    assert_eq!(
        exp.resolution_reason,
        Some("newer supersedes older".to_string())
    );
    assert!(!exp.active_contradiction);
    assert!(exp.audit_visible);

    // Explain for the superseded memory (100)
    let explains_superseded = store
        .contradiction_engine()
        .explain_for_memory(MemoryId(100));
    assert_eq!(explains_superseded.len(), 1);

    let sup = &explains_superseded[0];
    assert!(!sup.result_is_preferred);
    assert_eq!(sup.conflicting_memory, MemoryId(200));
    assert_eq!(sup.superseded_memory, Some(MemoryId(100)));
    assert!(sup.audit_visible);
}

// ── 3. Active contradiction markers preserve disagreement ───────────────────

#[test]
fn active_contradiction_marker_surfaces_disagreement_without_overwrite() {
    let mut store = BrainStore::default();
    let namespace = ns("golden/active");

    store
        .record_encode_contradiction(
            namespace,
            MemoryId(300),
            MemoryId(400),
            ContradictionKind::Coexistence,
            400,
        )
        .unwrap();

    // Mark as active contradiction (no resolution)
    let records = store.contradiction_engine().find_by_memory(MemoryId(300));
    let mut record = records[0].clone();
    record.mark_active_contradiction();

    assert_eq!(record.resolution, ResolutionState::Unresolved);
    assert_eq!(record.preferred_memory, None);
    assert_eq!(
        record.preferred_answer_state,
        PreferredAnswerState::ActiveContradiction
    );
    assert!(record.has_active_contradiction());
    assert!(record.confidence_signal > 0); // Active contradiction still has a confidence signal

    // Both memories should still be queryable — no silent overwrite
    assert_eq!(record.memory_a, MemoryId(300));
    assert_eq!(record.memory_b, MemoryId(400));
}

// ── 4. Retention/archival preserves audit visibility ─────────────────────────

#[test]
fn archive_with_legal_hold_keeps_audit_visible() {
    let mut store = BrainStore::default();
    let namespace = ns("golden/archive");

    let outcome = store
        .record_encode_contradiction(
            namespace,
            MemoryId(500),
            MemoryId(600),
            ContradictionKind::AuthoritativeOverride,
            950,
        )
        .unwrap();

    // Archive under legal hold
    store
        .contradiction_engine_mut()
        .apply_retention_policy(
            outcome.contradiction_id,
            true, // archived
            true, // legal_hold
            true, // authoritative_evidence
            "legal hold preserves archive for audit",
        )
        .unwrap();

    let explains = store
        .contradiction_engine()
        .explain_for_memory(MemoryId(500));
    assert_eq!(explains.len(), 1);

    let exp = &explains[0];
    assert!(exp.archived);
    assert!(exp.legal_hold);
    assert!(exp.authoritative_evidence);
    assert_eq!(
        exp.retention_reason,
        Some("legal hold preserves archive for audit".to_string())
    );
    assert!(exp.audit_visible); // legal hold + authoritative evidence → still visible
}

// ── 5. Legal hold prevents evidence destruction ──────────────────────────────

#[test]
fn cannot_archive_last_authoritative_evidence_without_legal_hold() {
    let mut store = BrainStore::default();
    let namespace = ns("golden/hold");

    let outcome = store
        .record_encode_contradiction(
            namespace,
            MemoryId(700),
            MemoryId(800),
            ContradictionKind::Supersession,
            800,
        )
        .unwrap();

    // Attempt to archive the last authoritative evidence without legal hold
    let err = store
        .contradiction_engine_mut()
        .apply_retention_policy(
            outcome.contradiction_id,
            true,  // archived
            false, // no legal hold
            true,  // authoritative_evidence
            "attempt to archive last evidence",
        )
        .unwrap_err();

    assert_eq!(err, ContradictionError::AuthoritativeEvidenceRequired);
}

// ── 6. Lineage pairs remain intact through the entire lifecycle ──────────────

#[test]
fn lineage_pairs_survive_resolution_and_archival() {
    let mut store = BrainStore::default();
    let namespace = ns("golden/lineage");

    let outcome = store
        .record_encode_contradiction(
            namespace,
            MemoryId(10),
            MemoryId(20),
            ContradictionKind::Revision,
            700,
        )
        .unwrap();

    let original_pair = [MemoryId(10), MemoryId(20)];

    // Before resolution
    let explains = store
        .contradiction_engine()
        .explain_for_memory(MemoryId(10));
    assert_eq!(explains[0].lineage_pair, original_pair);

    // After resolution
    store
        .contradiction_engine_mut()
        .resolve(
            outcome.contradiction_id,
            ResolutionState::ManuallyResolved,
            MemoryId(20),
            "human chose newer version",
        )
        .unwrap();

    let explains = store
        .contradiction_engine()
        .explain_for_memory(MemoryId(10));
    assert_eq!(explains[0].lineage_pair, original_pair);

    let explains = store
        .contradiction_engine()
        .explain_for_memory(MemoryId(20));
    assert_eq!(explains[0].lineage_pair, original_pair);

    // After archival under legal hold
    store
        .contradiction_engine_mut()
        .apply_retention_policy(
            outcome.contradiction_id,
            true,
            true,
            true,
            "lineage preserved under legal hold",
        )
        .unwrap();

    let explains = store
        .contradiction_engine()
        .explain_for_memory(MemoryId(10));
    assert_eq!(explains[0].lineage_pair, original_pair);
    assert!(explains[0].audit_visible);

    let explains = store
        .contradiction_engine()
        .explain_for_memory(MemoryId(20));
    assert_eq!(explains[0].lineage_pair, original_pair);
    assert!(explains[0].audit_visible);
}

// ── 7. Full golden lifecycle: create → explain → resolve → archive → audit ──

#[test]
fn full_golden_contradiction_lifecycle() {
    let mut store = BrainStore::default();
    let namespace = ns("golden/full");

    // Step 1: Create contradiction via write-path branching
    let outcome = store
        .record_encode_contradiction(
            namespace.clone(),
            MemoryId(1),
            MemoryId(2),
            ContradictionKind::Revision,
            820,
        )
        .unwrap();

    assert_eq!(outcome.branch, EncodeWriteBranch::ContradictionRecorded);
    let cid = outcome.contradiction_id;

    // Step 2: Explain shows unresolved state
    let explains = store.contradiction_engine().explain_for_memory(MemoryId(1));
    assert_eq!(explains.len(), 1);
    assert_eq!(explains[0].resolution, ResolutionState::Unresolved);
    assert_eq!(
        explains[0].preferred_answer_state,
        PreferredAnswerState::Unset
    );
    assert!(explains[0].audit_visible);

    // Step 3: Resolve — memory_b preferred (revision of memory_a)
    store
        .contradiction_engine_mut()
        .resolve(
            cid,
            ResolutionState::AutoResolved,
            MemoryId(2),
            "revision detected: newer version preferred",
        )
        .unwrap();

    let explains = store.contradiction_engine().explain_for_memory(MemoryId(2));
    let exp = &explains[0];
    assert_eq!(exp.resolution, ResolutionState::AutoResolved);
    assert_eq!(exp.preferred_answer_state, PreferredAnswerState::Preferred);
    assert_eq!(exp.preferred_memory, Some(MemoryId(2)));
    assert!(exp.confidence_signal > 700);
    assert!(exp.result_is_preferred);
    assert_eq!(exp.superseded_memory, Some(MemoryId(1)));
    assert!(!exp.active_contradiction);
    assert!(exp.audit_visible);
    assert_eq!(exp.lineage_pair, [MemoryId(1), MemoryId(2)]);

    // Step 4: Archive under legal hold
    store
        .contradiction_engine_mut()
        .apply_retention_policy(
            cid,
            true,
            true,
            true,
            "contradiction archived under legal hold after resolution",
        )
        .unwrap();

    let explains = store.contradiction_engine().explain_for_memory(MemoryId(1));
    let exp = &explains[0];
    assert!(exp.archived);
    assert!(exp.legal_hold);
    assert!(exp.authoritative_evidence);
    assert!(exp.audit_visible); // Legal hold keeps it visible
    assert_eq!(exp.lineage_pair, [MemoryId(1), MemoryId(2)]);

    // Step 5: Counting — namespace has exactly 1 contradiction, 0 unresolved
    assert_eq!(
        store.contradiction_engine().count_in_namespace(&namespace),
        1
    );
    assert_eq!(store.contradiction_engine().count_unresolved(&namespace), 0);
}

// ── 8. Namespace isolation in explain/audit ──────────────────────────────────

#[test]
fn explain_respects_namespace_boundaries() {
    let mut store = BrainStore::default();
    let ns_a = ns("golden/ns-a");
    let ns_b = ns("golden/ns-b");

    store
        .record_encode_contradiction(
            ns_a.clone(),
            MemoryId(100),
            MemoryId(200),
            ContradictionKind::Duplicate,
            1000,
        )
        .unwrap();

    store
        .record_encode_contradiction(
            ns_b.clone(),
            MemoryId(100),
            MemoryId(300),
            ContradictionKind::Revision,
            800,
        )
        .unwrap();

    // Contradictions are isolated per namespace
    assert_eq!(store.contradiction_engine().count_in_namespace(&ns_a), 1);
    assert_eq!(store.contradiction_engine().count_in_namespace(&ns_b), 1);

    // Unresolved queries are namespace-scoped
    assert_eq!(store.contradiction_engine().count_unresolved(&ns_a), 1);
    assert_eq!(store.contradiction_engine().count_unresolved(&ns_b), 1);
}

// ── 9. Confidence signal varies by resolution kind ──────────────────────────

#[test]
fn confidence_reflects_resolution_kind() {
    let mut store = BrainStore::default();
    let namespace = ns("golden/confidence");

    // Authoritative override → highest confidence
    let o1 = store
        .record_encode_contradiction(
            namespace.clone(),
            MemoryId(1),
            MemoryId(2),
            ContradictionKind::AuthoritativeOverride,
            900,
        )
        .unwrap();
    store
        .contradiction_engine_mut()
        .resolve(
            o1.contradiction_id,
            ResolutionState::AuthoritativelyResolved,
            MemoryId(2),
            "auth",
        )
        .unwrap();
    let c1 = store.contradiction_engine().explain_for_memory(MemoryId(2))[0].confidence_signal;

    // Supersession → high confidence
    let o2 = store
        .record_encode_contradiction(
            namespace.clone(),
            MemoryId(3),
            MemoryId(4),
            ContradictionKind::Supersession,
            900,
        )
        .unwrap();
    store
        .contradiction_engine_mut()
        .resolve(
            o2.contradiction_id,
            ResolutionState::AutoResolved,
            MemoryId(4),
            "super",
        )
        .unwrap();
    let c2 = store.contradiction_engine().explain_for_memory(MemoryId(4))[0].confidence_signal;

    // Revision → moderate confidence
    let o3 = store
        .record_encode_contradiction(
            namespace.clone(),
            MemoryId(5),
            MemoryId(6),
            ContradictionKind::Revision,
            900,
        )
        .unwrap();
    store
        .contradiction_engine_mut()
        .resolve(
            o3.contradiction_id,
            ResolutionState::AutoResolved,
            MemoryId(6),
            "rev",
        )
        .unwrap();
    let c3 = store.contradiction_engine().explain_for_memory(MemoryId(6))[0].confidence_signal;

    // Authoritative > Supersession > Revision
    assert!(
        c1 > c2,
        "authoritative ({}) should exceed supersession ({})",
        c1,
        c2
    );
    assert!(
        c2 > c3,
        "supersession ({}) should exceed revision ({})",
        c2,
        c3
    );
    assert!(c3 > 700, "revision confidence ({}) should be above 700", c3);
}
