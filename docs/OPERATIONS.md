# membrain — Operations Guide

> Canonical source: PLAN.md Sections 25 (Operations Acceptance), 26 (Failure Mode Matrix), 23 (Compaction & Repair).

## Operational Success Criteria

A workflow is accepted only if it completes without violating:
- Latency budgets (Tier1 <0.1ms, Tier2 <5ms, Tier3 <50ms, Encode <10ms)
- Data integrity guarantees
- Lineage guarantees
- Policy guarantees

---

## Standard Runbook Shape

Every operational runbook follows this structure:

1. **Preconditions** — what must be true before starting
2. **Command sequence** — steps to execute
3. **Metrics to watch** — what to monitor during execution
4. **Rollback conditions** — when to abort
5. **Post-run validation** — how to verify success

---

## 1. Capacity Planning

### Preconditions
- Access to `membrain stats --json` and `membrain health --json`
- Knowledge of current workload growth rate

### Command Sequence
```bash
membrain stats --json | jq '{hot: .hot_count, cold: .cold_count, utilization: .hot_utilization_pct}'
membrain health --json | jq '.decay_rate'
membrain hot-paths --top 20 --json
membrain dead-zones --min-age 5000 --json
```

### Metrics to Watch
- Hot tier utilization % (alert at >80%)
- Decay rate per 1k ticks
- Dead zone accumulation rate

### Rollback Conditions
- Not applicable (read-only assessment)

### Post-run Validation
- Capacity forecast matches expected growth
- No tier is unexpectedly near saturation

---

## 2. Daily Health Review

### Command Sequence
```bash
membrain health --brief
membrain health --json | jq '{
  hot_pct: .hot_utilization_pct,
  avg_strength: .avg_strength,
  avg_confidence: .avg_confidence,
  unresolved_conflicts: .unresolved_conflicts,
  uncertain: .uncertain_count,
  last_dream: .last_dream_tick
}'
membrain doctor run
```

### Metrics to Watch
- Hot utilization trending up → schedule consolidation
- Average confidence dropping → investigate conflict rate
- Unresolved conflicts accumulating → trigger belief resolution
- Doctor warnings → schedule repair

---

## 3. Backpressure Management

### Preconditions
- Hot tier utilization > 70%

### Command Sequence
```bash
membrain consolidate
membrain dead-zones --forget-all    # archive never-retrieved old memories
membrain compress --dry-run         # check if compression would help
membrain compress                   # apply schema compression
```

### Metrics to Watch
- Hot count before/after
- Consolidation duration
- Memories archived

### Rollback Conditions
- Consolidation takes >30s → interrupt, investigate
- Critical memories being archived → check min_strength threshold

---

## 4. Compaction Windows

### Preconditions
- No active high-priority recall workloads
- Daemon is running

### Command Sequence
```bash
membrain consolidate
membrain compress
membrain dream               # run dream cycle if idle
membrain skills --extract    # extract procedural memories from mature engrams
```

### Metrics to Watch
- Background job duration
- Hot-path latency during compaction (must not exceed budget)
- Schemas created, episodes compressed

### Rollback Conditions
- Online recall latency exceeds budget → pause background jobs

---

## 5. Index Rebuild Operations

### Command Sequence
```bash
membrain doctor run                    # diagnose issues first
membrain repair index --dry-run       # preview
membrain repair index --namespace default
membrain benchmark tier2              # verify performance restored
```

### Metrics to Watch
- Index count vs durable record count (must match)
- Search latency before/after rebuild
- Any orphan embeddings or missing vectors

### Post-run Validation
```bash
membrain benchmark tier2 --json | jq '.p99_us'
```

---

## 5.1 Canonical rebuild flow matrix

| Surface | Authoritative input | Repair command shape | Post-run proof |
|---|---|---|---|
| indexes / ANN / lexical projections | durable records + canonical embeddings + policy-bearing metadata | `membrain repair index ...` | candidate counts match durable truth; latency returns to budget |
| graph edges / neighborhoods / materialized clusters | normalized SQLite relation tables + lineage | `membrain repair graph ...` | graph counts match canonical edge tables; bounded traversal resumes |
| lineage chains / ancestry views | durable lineage records + supersession/conflict records | `membrain repair lineage ...` | no broken chains; explain surfaces resolve ancestry |
| caches / sidecars / planner accelerators | durable records + policy/namespace filters | `membrain repair cache ...` or drop-and-rewarm | caches repopulate without becoming sole truth |

### Flow requirements
- Run doctor and a dry run before mutating repair when available.
- Use namespace-scoped repair first; widen scope only when divergence is proven cross-namespace.
- Durable truth wins whenever rebuilt output disagrees with a derived surface.
- If full fidelity cannot be restored, emit an irreversible-loss record and leave the degraded surface visible to operators.

---

## 5.2 Graph and lineage repair

### Preconditions
- Canonical edge tables and lineage records are readable.
- Conflicting background compaction or migration jobs are paused.

### Command Sequence
```bash
membrain doctor run
membrain repair graph --dry-run
membrain repair graph --namespace default
membrain repair lineage --dry-run
membrain repair lineage --namespace default
```

### Metrics to Watch
- Canonical edge count vs rebuilt materialized edge count
- Broken-lineage count before/after
- Repair queue depth for follow-on unresolved items
- Any explicit loss records emitted during repair

### Rollback Conditions
- Repaired graph output diverges further from canonical edge tables
- Broken-lineage count increases after mutation
- Online retrieval latency breaches budget during the repair window

### Post-run Validation
- `memory_explain` can traverse lineage and graph ancestry again
- No unresolved orphan nodes or broken ancestry chains remain without queued follow-up repair

---

## 5.3 Cache and sidecar rebuild

### Preconditions
- Operators have confirmed caches and sidecars are not being treated as sole truth.
- The namespace or shard can tolerate temporary cold-start latency.

### Command Sequence
```bash
membrain doctor run
membrain repair cache --dry-run
membrain repair cache --namespace default
membrain health --brief
```

### Metrics to Watch
- Cache hit rate dip and recovery curve
- Candidate count parity against durable truth
- Namespace isolation and policy-filter integrity during warmup

### Rollback Conditions
- Warmed caches surface items blocked by namespace or policy filters
- Rewarmed candidate counts continue to diverge from durable truth

### Post-run Validation
- Cache repopulation completes without new policy or lineage drift
- Retrieval behavior is slower temporarily, but semantically unchanged

---

## 6. Retention Enforcement

### Command Sequence
```bash
membrain audit --op archive --since <last_check_tick>   # what was archived
membrain uncertain --top 50                              # low-confidence review
membrain dead-zones --min-age 10000                     # very old unaccessed
```

### Metrics to Watch
- Archive rate vs encode rate (healthy: archive ~10-20% of encode)
- Pinned memory count growth (unbounded pinning is an anti-pattern)

---

## 7. Incident Response

### Immediate Response Pattern

For all major incidents:
1. **Isolate** affected namespace or shard if needed
2. **Stop** destructive background jobs (`membrain dream --disable`)
3. **Preserve** forensic logs (`membrain audit --recent 500 --json > incident.json`)
4. **Enable** degraded mode if available

### Root-Cause Investigation

```bash
membrain audit <affected-uuid> --json              # full history
membrain audit --since <incident_tick> --op interference
membrain doctor run
membrain diff --since <pre_incident_tick>           # what changed
```

### Checklist
- Lineage validation (no broken chains)
- Index count vs durable records
- Recent deploy inspection
- Repair queue growth
- Compaction history

---

## 8. Migration

### Command Sequence
```bash
membrain snapshot --name pre-migration --note "Before schema migration"
membrain export --format ndjson > backup.ndjson
# Apply migration
membrain doctor run                    # verify integrity
membrain benchmark tier2              # verify performance
membrain diff --since pre-migration   # review changes
```

### Rollback Conditions
- Doctor reports errors post-migration → restore from backup
- Benchmark shows regression > 20% → investigate

---

## 9. Version Rollout

### Command Sequence
```bash
membrain snapshot --name pre-upgrade
membrain daemon stop
# Deploy new binary
membrain daemon start
membrain doctor run
membrain health --brief
membrain benchmark tier1
membrain benchmark tier2
```

### Post-run Validation
- All benchmarks within expected bounds
- Doctor clean
- Health dashboard shows no anomalies
- Recall latency spot-check passes

---

## 10. Snapshot Management

```bash
membrain snapshot list                              # review existing
membrain snapshot --name weekly-$(date +%Y%m%d)    # create periodic
membrain snapshot delete <old-snapshot>              # clean up
```

---

## Canonical Failure Modes

| Failure | Immediate Action | Investigation |
|---------|-----------------|---------------|
| Tier1 overflow | Consolidate, increase cache eviction | Check encode rate vs capacity |
| Tier2 index drift | Rebuild index | Compare index count vs DB records |
| Contradiction masking | Review `beliefs --conflicts` | Check conflict detection thresholds |
| Duplicate storms | Check fingerprint dedup | Review novelty threshold |
| Graph fanout explosion | Cap BFS depth | Audit engram sizes |
| Latency regression | Benchmark all tiers | Check background job interference |
| Cross-namespace leakage | Audit namespace filters | Review visibility rules |
| Retention-policy bug | Audit archive events | Review decay + forgetting params |

## Design Implication

A production architecture is incomplete if it cannot enter a safe degraded mode under these failures. Every failure must be detectable, isolatable, and recoverable without data loss.
