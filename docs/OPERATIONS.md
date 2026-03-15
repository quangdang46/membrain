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

## Change-Introduction Observability Bundle

When a major change alters hot-path behavior, schema, forgetting/deletion semantics, or background execution, the rollout or handoff note should leave operators with both the required artifact bundle and the concrete signals to monitor after the change lands.

### Required note bundle by change type
- design note for architectural changes
- benchmark result for hot-path or performance-sensitive changes
- migration note for schema changes
- rollback note for behavior changes
- governance analysis for forgetting or deletion semantic changes
- ops note when background jobs, repair flows, or maintenance cadence change

### Metrics and traces to name explicitly
- request-path latency p50/p95/p99, candidate counts, tier hit rates, cache hit rates, or routing traces for retrieval/encode changes
- denial or redaction counts, audit events, and explain/inspect evidence for policy, namespace, or governance changes
- job duration, queue depth, moved or repaired item counts, and foreground latency delta for consolidation, repair, compaction, or similar background work

The note should also point to the exact command, dashboard, benchmark artifact, or machine-readable field operators will use, and should follow the same `metrics to watch` plus `rollback conditions` discipline as the standard runbook shape above.

## Maintenance Execution Classes

Classify maintenance by how it interacts with authoritative state before choosing a runbook, serving posture, or approval path.

### Class A — Read-only assessment
Examples: `health`, `stats`, `audit`, `doctor run`, dry runs, benchmarks, snapshot listing, and diffs.
- May run without a maintenance window.
- Must not mutate authoritative or derived state.
- Does not require a fresh snapshot unless it is the preflight step for a write operation that does.

### Class B — Online derived-surface maintenance
Examples: cache drop-and-rewarm, index rebuilds, sidecar regeneration, and other repairable projections whose authoritative durable inputs remain readable.
- May run in the background without a dedicated maintenance window only when request paths stay within latency budgets, fallback to slower durable-truth reads exists, and the work is pausable.
- Scope should start at the narrowest namespace or shard possible.
- Operators should be able to observe candidate-count parity, queue depth, and degraded-mode or bypass signals while the job runs.

### Class C — Window-required authoritative rewrite
Examples: storage compaction, compression, lineage-preserving layout rewrites, or other maintenance that mutates authoritative durable structures while the system remains partially online.
- Requires a declared maintenance window for the affected namespace, shard, or deployment scope even if some read traffic can continue in degraded or read-only mode.
- Requires a fresh pre-run snapshot or equivalent durable backup, an explicit rollback path, and before/after telemetry.
- Should pause or defer other conflicting background jobs first.

### Class D — Offline or fail-closed maintenance
Examples: schema rollback, repair when canonical inputs are not trustworthy enough for live serving, or any task that mutates the same canonical structures needed for safe reads.
- Requires a maintenance window plus explicit degraded, read-only, or offline posture for the affected scope.
- Full availability returns only after canonical validation passes.
- If correctness or policy isolation is ambiguous, prefer fail-closed containment over partial availability.

### A maintenance window is required when
- the operation rewrites authoritative durable layout, lineage-bearing structures, or policy-bearing markers,
- the rollback plan depends on snapshot restore rather than simple pause-and-resume,
- the affected scope must enter degraded mode, read-only mode, or offline service,
- hot-path latency or I/O contention cannot be bounded safely under ordinary load, or
- the work may emit irreversible-loss records, retention-affecting mutations, or other operator-significant semantic changes.

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

Backpressure response is for bounded pressure relief that can run without entering a maintenance window. If relief would require authoritative compaction, retention enforcement, or any rollback path that depends on snapshot restore, stop after the assessment steps here and schedule the appropriate runbook instead.

### Preconditions
- Hot tier utilization > 70%

### Command Sequence
```bash
membrain consolidate
membrain dead-zones --min-age 5000 --json   # inspect archive candidates; do not mutate from this runbook
membrain compress --dry-run                 # estimate compaction benefit; schedule §4 if needed
```

### Metrics to Watch
- Hot count before/after
- Consolidation duration
- Candidate archive count and estimated compaction benefit

### Rollback Conditions
- Consolidation takes >30s → interrupt, investigate
- Pressure relief would require retention-affecting archive, compression, or other authoritative rewrite → switch to §4 or §6 instead of continuing inline

### Post-run Validation
- Pressure is reduced, or the need for a window-required follow-up is explicitly recorded.
- No retention-, lineage-, or policy-bearing mutation occurred unless another runbook was invoked.

---

## 4. Compaction Windows

Compaction is a maintenance-window task whenever it rewrites authoritative durable layout, lineage-bearing structures, or policy-bearing markers. If the work only rebuilds derived indexes, caches, or sidecars, use the repair flows in §5 instead of treating it as truth-rewriting compaction.

### Preconditions
- An announced maintenance window exists for the affected namespace, shard, or deployment scope, with a named rollback owner
- No active high-priority recall workloads remain on the affected scope
- Daemon is running
- A fresh snapshot or equivalent durable backup exists for any namespace whose authoritative storage layout will be rewritten
- Conflicting repair, migration, cache-rewarm, or other write-heavy background jobs are paused
- Operators have confirmed the planned work is compaction or rebuild of existing truth, not an implicit retention or deletion action

### Command Sequence
```bash
membrain snapshot --name pre-compaction --note "Before storage compaction"
membrain doctor run
membrain consolidate
membrain compress
# Optional only for explicitly idle windows with separate success criteria:
membrain dream
membrain skills --extract
```

`dream` and `skills --extract` are follow-on maintenance operations, not mandatory steps in every compaction pass. Run them only when the window is explicitly idle enough to tolerate their additional background load and when their validation criteria are tracked separately from the compaction rewrite itself.

### Metrics to Watch
- Background job duration
- Hot-path latency during compaction (must not exceed budget)
- Schemas created, episodes compressed
- Any moved or rewritten payload count vs preserved durable-record count
- Any explicit loss, tombstone, or staleness records emitted during the window

### Rollback Conditions
- Online recall latency exceeds budget → pause background jobs
- Count parity, lineage, policy, or retention markers diverge from durable truth during or after compaction
- The operation would require treating a derived artifact as the only surviving source of truth

### Post-run Validation
- Durable counts and policy-bearing markers remain unchanged except for the explicitly planned compaction rewrite
- `content_ref` stability, payload-handle resolvability, and lineage continuity still hold for the compacted scope
- Any regenerated artifacts or sidecars are marked derived and rebuildable rather than authoritative

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

| Surface | Authoritative input | Repair command shape | Availability during rebuild | Post-run proof |
|---|---|---|---|---|
| indexes / ANN / lexical projections | durable records + canonical embeddings + policy-bearing metadata | `membrain repair index ...` | online by default; exact lookup and durable-truth reads remain available, ranked recall may run colder/slower | candidate counts match durable truth; latency returns to budget |
| graph edges / neighborhoods / materialized clusters | normalized SQLite relation tables + lineage | `membrain repair graph ...` | online only if canonical edge tables remain readable; otherwise degrade affected namespaces to read-only or offline | graph counts match canonical edge tables; bounded traversal resumes |
| lineage chains / ancestry views | durable lineage records + supersession/conflict records | `membrain repair lineage ...` | online if explain/inspect can fall back to durable lineage reads; otherwise degrade explain surfaces until validated | no broken chains; explain surfaces resolve ancestry |
| caches / sidecars / planner accelerators | durable records + policy/namespace filters | `membrain repair cache ...` or drop-and-rewarm | fully online; slower reads are acceptable while caches repopulate | caches repopulate without becoming sole truth |

### Flow requirements
- Run doctor and a dry run before mutating repair when available.
- Use namespace-scoped repair first; widen scope only when divergence is proven cross-namespace.
- Durable truth wins whenever rebuilt output disagrees with a derived surface.
- If full fidelity cannot be restored, emit an irreversible-loss record and leave the degraded surface visible to operators.

### Repair mode and availability contract
- **Online repair** is the default for derived surfaces whose authoritative inputs remain readable. During online repair, exact lookup, policy checks, and reads from durable truth stay available; derived surfaces may be stale, colder, or temporarily bypassed, but they must not return invented results.
- **Offline repair** is required when the authoritative durable inputs are not trustworthy enough for bounded live serving, when migration or schema rollback is in progress, or when repair mutates the same canonical structures needed for safe reads. During offline repair, affected namespaces or shards must be placed in degraded mode or read-only service until canonical validation passes.
- If an index, graph projection, lineage view, or cache is unavailable during rebuild, the system must either fall back to slower durable-truth reads or refuse the affected operation explicitly; it must not pretend the rebuilt surface is complete before validation.
- Every repair run should declare which namespaces or shards remain fully available, which are degraded to slower reads, and which are temporarily read-only or offline.

### Lifecycle transition repair handoff
- Failed lifecycle transitions reuse the same durable-truth-first repair model: prior durable state stays authoritative while derived follow-up work is repaired or replayed.
- Only retryable internal failures should create automatic repair or replay work. Validation failures and policy denials are recorded but must not spin in automated retry loops.
- Repair queue entries for failed transitions should preserve object handle, namespace or shard scope, prior valid state, attempted edge, failure family, retryability, and escalation boundary.
- When retry budgets are exhausted, the system should escalate to operator-visible repair or incident handling and may place the affected namespace or shard into degraded mode if correctness is at risk.
- Pending repair must not silently widen authority: replay controllers keep the same namespace, visibility, and policy scope as the failed transition.

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
- Stale-warning, bypass, invalidation, and degraded-mode rates by cache family

### Rollback Conditions
- Warmed caches surface items blocked by namespace or policy filters
- Rewarmed candidate counts continue to diverge from durable truth

### Post-run Validation
- Cache repopulation completes without new policy or lineage drift
- Retrieval behavior is slower temporarily, but semantically unchanged

### Derived-cache repair rules
- Caches, sidecars, and prefetch queues are drop-and-rewarm derived surfaces keyed by effective namespace, declared owner boundary, and the policy and generation inputs that affect reuse.
- Prefetch queues must be rebuilt from live session or task intent only; repaired hints should be discarded rather than replayed if the original owner binding or intent signature is no longer authoritative.
- Session warmup recovery may rebuild a bounded session-local hot set, but rewarmed entries remain unusable until the session binding, visibility scope, and warmed-family generation anchors are rebound.
- Process-local cold-start mitigation may be reinitialized during repair or restart, but request-serving paths must treat prewarm artifacts as bootstrap-only until current namespace and generation checks succeed.
- Repair and migration must rebind fresh generation anchors before rewarmed state becomes eligible for reuse.
- If invalidation anchors are uncertain after repair, migration, policy change, namespace rebinding, owner-boundary change, or partial failure, bypass warm state and fall back to slower durable-truth reads until validation passes.
- Warmup recovery succeeds only when latency improves without changing namespace isolation, candidate parity, owner-boundary correctness, or policy outcomes.

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
- Legal-hold or purge-eligibility count drift after policy or migration changes
- Tombstone and retention-audit anomalies that suggest a hold, purge, or archive marker was lost in repair

---

## 7. Incident Response

### Immediate Response Pattern

For all major incidents:
1. **Isolate** affected namespace or shard if needed
2. **Stop** destructive background jobs (`membrain dream --disable`)
3. **Preserve** forensic logs (`membrain audit --recent 500 --json > incident.json`)
4. **Enable** degraded mode if available

Namespace-isolation or cross-tenant leakage must be treated as a security incident, not ordinary validation noise.
- Freeze or narrow affected shared/public surfaces until namespace filters are revalidated.
- Keep denied or redacted explain paths from echoing protected handles while the incident is active.
- Post-incident validation must prove candidate counts, cache warmup, repair outputs, and audit traces all respect namespace boundaries again.

Duplicate storms, graph fanout explosions, and stale-cache incidents should also be handled as boundedness failures rather than mere latency noise.
- Confirm degraded mode or colder fallback is active before allowing request load to continue.
- Verify routing traces still expose candidate counts, cache bypass reasons, and any stale-warning or degraded-mode markers needed to explain the containment path.
- If transition repair is in flight, verify the prior durable state remains authoritative and that retry-budget exhaustion or escalation artifacts are visible to operators.

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

### Rollback eligibility decision

Use rollback only when reverting a recent change is safer than continued degraded serving or live mutation.

- **Prefer containment plus repair** when the incident is confined to derived state and slower durable-truth serving preserves correctness.
- **Prefer fail-closed containment** for namespace leakage, policy drift, retention ambiguity, or any incident where continued serving could expose protected data or perform destructive actions on ambiguous truth.
- **Prefer rollback** when a recent binary, config, schema, ranking, routing, or policy rollout is the likely cause and reverting reduces risk faster than rebuilding or replaying forward.
- **Do not roll back** by trusting stale caches, indexes, summaries, graph projections, or other derived artifacts over authoritative durable records.

### Degraded-service communication contract

When incident handling changes what users can safely get from the system, the interface should communicate that explicitly:

- reduced-fidelity serving should say which bounded fallback is active (colder reads, narrower planner, graph-disabled, index-bypassed, etc.),
- fail-closed serving should say the affected surface is temporarily unavailable or restricted rather than pretending the result set is complete,
- warnings, metrics, or explain handles should preserve enough machine-readable evidence for operators to distinguish degraded serving from ordinary misses, and
- cross-interface behavior should stay semantically aligned across CLI, daemon, IPC/JSON-RPC, and MCP surfaces.

### Post-incident verification

Before closing the incident or disabling containment, prove all of the following:

1. the intended serving mode is restored for the affected namespace, shard, or surface,
2. durable counts, lineage, contradiction state, namespace isolation, and policy-bearing markers match authoritative expectations again,
3. degraded-mode, bypass, stale-warning, or fail-closed indicators have cleared or remain explicitly declared for any still-limited surface,
4. recent repair, rollback, or migration actions did not leave new unresolved queue growth or hidden semantic drift, and
5. operators have a concrete follow-up record for any intentionally deferred cleanup.

---

## 8. Migration

Migration is a window-required operation whenever it rewrites authoritative schema, policy-bearing markers, or other canonical structures whose rollback path may depend on restore.

### Preconditions
- An announced maintenance window exists for the affected scope, with rollback ownership and serving posture declared.
- A fresh pre-migration snapshot or equivalent durable backup exists.
- Operators have captured the namespace, retention, legal-hold, and policy baseline that post-run validation must preserve unless the migration explicitly changes it.

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
- Cache invalidation, stale-warning, or degraded-mode signals indicate migrated warm state is being reused ambiguously
- Namespace, retention, or legal-hold validation diverges from pre-migration expectations

### Post-run Validation
- Repaired or migrated caches rebuild from fresh generation anchors instead of reusing ambiguous warm state.
- Candidate counts, explain traces, and cache metadata remain semantically consistent across CLI, daemon, IPC, and MCP surfaces.
- Namespace isolation, retention markers, legal-hold markers, and tombstones match pre-migration durable truth unless an explicit migration step changed them.

---

## 9. Version Rollout

A version rollout may proceed without a full maintenance window only when restart, warmup, and fallback behavior remain bounded for the affected scope. If rollout requires offline service, schema rewrite, restore-based rollback, or prolonged degraded serving, treat it as window-required maintenance.

### Preconditions
- The intended rollout posture is declared: rolling, degraded-but-online, read-only, or offline.
- A fresh pre-upgrade snapshot exists when rollback may depend on restore or when the rollout is coupled to migration or authoritative rewrite.
- Operators know whether cache warmup, tier benchmarks, or namespace validation will be temporarily colder after restart.

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

Operational snapshots are safety anchors for maintenance, rollback, and diff-based verification. They support safe operations around authoritative durable state, but they do not replace lineage, audit records, or durable-truth validation, and they do not make derived artifacts authoritative.

### Snapshot requirement rules
A fresh pre-run snapshot or equivalent durable backup is required before:
- compaction, compression, or any other authoritative layout rewrite,
- schema migration or version rollout whose rollback path may depend on restore,
- maintenance that may emit irreversible-loss records or mutate policy-bearing markers, or
- any operation whose safe fallback is restore rather than pause-and-resume.

A snapshot is optional for:
- read-only assessment, dry runs, benchmarks, audits, and diffs,
- purely derived cache or index rebuilds that can be dropped and rebuilt from durable truth, and
- bounded online repair that can be safely paused without restore.

### Handling rules
- Name snapshots for the event and scope (`pre-compaction`, `pre-migration`, `pre-upgrade`).
- Record the reason, affected namespace or shard, and related incident or change identifier in the snapshot note or adjacent runbook artifact.
- Keep the last known-good pre-run snapshot until post-run validation passes and the rollback decision window closes.
- Do not delete the last restorable snapshot for a scope while migration, compaction, or incident recovery remains unresolved.

### Command Sequence
```bash
membrain snapshot list                              # review existing
membrain snapshot --name weekly-$(date +%Y%m%d)    # create periodic
membrain snapshot --name pre-maintenance --note "Before authoritative rewrite"
membrain snapshot delete <old-snapshot>            # clean up only after validation clears the restore point
```

### Post-run Validation
- The intended pre-run snapshot is present, named clearly, and restorable for the affected scope.
- The change record links the snapshot to the maintenance window, migration, or rollout it protects.
- Snapshot retention still leaves at least one recent restorable anchor for each actively managed scope.

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
