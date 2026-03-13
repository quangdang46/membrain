# COMPACTION AND REPAIR

Compaction reduces cost while repair preserves correctness after crashes, drift, or partial failures.

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

