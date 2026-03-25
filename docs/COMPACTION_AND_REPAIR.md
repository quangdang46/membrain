# COMPACTION AND REPAIR

Compaction reduces cost while repair preserves correctness after crashes, drift, or partial failures.

## Durable-versus-derived repair contract

- Repair rebuilds derived state from authoritative durable evidence.
- Compaction may replace convenience artifacts, but it must not remove the last authoritative evidence unless policy explicitly allows it.
- Persisted summaries, checkpoints, and sidecars remain derived artifacts unless explicitly promoted by policy.
- If a repair or compaction pass cannot recover full fidelity, it must emit an explicit irreversible-loss record instead of inventing missing truth.

## 1. Segment compaction

### Operation
Segment compaction should run in background windows with bounded foreground interference.

### Operator report contract
- emit a stable `compaction_run_report` object for every bounded run
- include unit kind, authoritative truth source, selected/completed/pending units, queue budget, and queue depth before/after
- when the queue budget is exhausted or bounded work must pause, set `degraded_mode=continue_degraded_reads`, `rollback_trigger=verification_mismatch`, and remediation steps `check_health`, `rollback_recent_change`, `run_repair`, `inspect_state`
- when the run completes without fallback, leave degraded-mode and rollback-trigger fields empty rather than inferring warnings from prose only

### Safety invariants
- do not lose authoritative evidence
- preserve lineage or record irreversible loss
- keep metrics for before/after state
- rate-limit large repair jobs

### Recommended telemetry
- bytes before and after
- affected item count
- error count
- rebuild duration
- queue depth before and after
- selected, completed, and pending compaction units

## 2. Duplicate family collapse

### Operation
Duplicate family collapse should run in background windows with bounded foreground interference.

### Safety invariants
- do not lose authoritative evidence
- preserve lineage or record irreversible loss
- keep metrics for before/after state
- rate-limit large repair jobs

### Recommended telemetry
- bytes before and after
- affected item count
- error count
- rebuild duration

## 3. Lineage pruning

### Operation
Lineage pruning should run in background windows with bounded foreground interference.

### Safety invariants
- do not lose authoritative evidence
- preserve lineage or record irreversible loss
- keep metrics for before/after state
- rate-limit large repair jobs

### Recommended telemetry
- bytes before and after
- affected item count
- error count
- rebuild duration

## 4. Index rebuild

### Operation
Index rebuild should run in background windows with bounded foreground interference.

### Repair inputs and outputs
- authoritative inputs: durable records, canonical embeddings, namespace/policy-bearing metadata
- rebuilt outputs: lexical projections, ANN structures, auxiliary lookup tables

### Safety invariants
- do not lose authoritative evidence
- preserve lineage or record irreversible loss
- keep metrics for before/after state
- rate-limit large repair jobs

### Recommended telemetry
- bytes before and after
- affected item count
- error count
- rebuild duration
- durable count vs rebuilt count
- named cache-maintenance hooks for verify-only parity checks and rebuild flows, plus invalidation and repair-warmup event counts when warm state is dropped and repopulated

## 5. Graph repair

### Operation
Graph repair should run in background windows with bounded foreground interference.

### Repair inputs and outputs
- authoritative inputs: durable memory records, canonical relation refs, and durable lineage edges
- rebuilt outputs: graph adjacency projection, graph neighborhood cache, and graph consistency snapshots
- verify-only hooks: `snapshot_durable_truth`, `verify_consistency_snapshot`
- rebuild hooks: `snapshot_durable_truth`, `rebuild_adjacency_projection`, `rebuild_neighborhood_cache`, `verify_consistency_snapshot`

### Repair behavior after drift or corruption
- durable graph truth comes from canonical relation and lineage rows, not from cached neighborhoods or prior materialized adjacency
- verify-only runs compare graph projections against durable edges and keep the prior durable state authoritative
- rebuild runs regenerate adjacency and neighborhood projections from durable rows, then verify the rebuilt snapshot before the surface is marked healthy
- if canonical edge tables are unreadable, graph-serving paths must stay degraded, read-only, or offline rather than trusting stale projections
- if reconstruction cannot preserve full lineage or endpoint fidelity, repair must emit an explicit irreversible-loss or degraded-state record instead of inventing replacement links
- operator-facing repair reports must expose `degraded_mode`, `rollback_trigger`, and ordered `remediation_steps`; graph verification mismatch maps to `continue_degraded_reads` plus `verification_mismatch`, while unreadable canonical truth escalates to read-only or offline containment

### Safety invariants
- do not lose authoritative evidence
- preserve lineage or record irreversible loss
- keep metrics for before/after state
- rate-limit large repair jobs

### Recommended telemetry
- bytes before and after
- affected item count
- error count
- rebuild duration
- durable edge count vs rebuilt adjacency count
- verification artifact proving `graph_projection_matches_durable_edges`
- repair hook names and rollback state when graph rebuild remains degraded
- degraded mode, rollback trigger, and remediation steps for operator handoff and audit correlation

## 6. Tombstone sweep

### Operation
Tombstone sweep should run in background windows with bounded foreground interference.

### Safety invariants
- do not lose authoritative evidence
- preserve lineage or record irreversible loss
- keep metrics for before/after state
- rate-limit large repair jobs

### Recommended telemetry
- bytes before and after
- affected item count
- error count
- rebuild duration

## 7. Payload detachment

### Operation
Payload detachment should run in background windows with bounded foreground interference.

### Safety invariants
- do not lose authoritative evidence
- preserve lineage or record irreversible loss
- keep metrics for before/after state
- rate-limit large repair jobs

### Recommended telemetry
- bytes before and after
- affected item count
- error count
- rebuild duration

## 8. Cache warm-state rebuild

### Operation
Cache warm-state rebuild should run in background windows with bounded foreground interference.

### Repair inputs and outputs
- authoritative inputs: durable memory records, namespace/policy metadata, and current generation anchors
- rebuilt outputs: tier1 item cache, result cache, summary cache, ANN probe cache, and the bounded prefetch queue
- verify-only hooks: `snapshot_current_generation_anchors`, `verify_generation_anchor_report`
- rebuild hooks: `snapshot_current_generation_anchors`, `invalidate_cache_families`, `drop_prefetch_hints`, `rebuild_tier1_item_cache`, `rebuild_result_cache`, `rebuild_summary_cache`, `rebuild_ann_probe_cache`, `verify_generation_anchor_report`

### Repair behavior after drift or corruption
- caches, sidecars, and prefetch state are disposable derived surfaces; durable truth and fresh generation anchors remain authoritative
- verify-only runs compare warm-state generation anchors against current durable generations without reusing stale entries
- rebuild runs first invalidate affected families and discard speculative prefetch hints, then repopulate bounded warm families from current durable truth and rebind fresh generation anchors before reuse
- if invalidation scope is uncertain after repair, migration, policy change, or namespace rebinding, request paths should bypass warm state and fall back to colder durable-truth reads until validation passes
- prefetch hints must be rebuilt only from live session or task intent; stale repaired hints should be dropped rather than replayed across owner-boundary or intent drift

### Safety invariants
- do not lose authoritative evidence
- preserve lineage or record irreversible loss
- keep metrics for before/after state
- rate-limit large repair jobs

### Recommended telemetry
- bytes before and after
- affected item count
- error count
- rebuild duration
- durable generation anchors vs rebuilt warm-state anchors
- invalidation and repair-warmup event counts by cache family
- candidate parity and namespace/policy correctness during rewarm

## 9. Summary regeneration

### Operation
Summary regeneration should run in background windows with bounded foreground interference.

### Safety invariants
- do not lose authoritative evidence
- preserve lineage or record irreversible loss
- keep metrics for before/after state
- rate-limit large repair jobs

### Recommended telemetry
- bytes before and after
- affected item count
- error count
- rebuild duration

## 10. Shard repair

### Operation
Shard repair should run in background windows with bounded foreground interference.

### Safety invariants
- do not lose authoritative evidence
- preserve lineage or record irreversible loss
- keep metrics for before/after state
- rate-limit large repair jobs

### Recommended telemetry
- bytes before and after
- affected item count
- error count
- rebuild duration

## 11. Backfill re-encoding

### Operation
Backfill re-encoding should run in background windows with bounded foreground interference.

### Safety invariants
- do not lose authoritative evidence
- preserve lineage or record irreversible loss
- keep metrics for before/after state
- rate-limit large repair jobs

### Recommended telemetry
- bytes before and after
- affected item count
- error count
- rebuild duration

