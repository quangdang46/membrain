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

## 5. Graph repair

### Operation
Graph repair should run in background windows with bounded foreground interference.

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

## 8. Summary regeneration

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

## 9. Shard repair

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

## 10. Backfill re-encoding

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

