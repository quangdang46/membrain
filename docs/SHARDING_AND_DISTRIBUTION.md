# SHARDING AND DISTRIBUTION

> Phase 4 follow-on only. `docs/PLAN.md` Section 29 and Section 40.5 plus `docs/ROADMAP.md` Phase 4 are canonical. This document narrows the future scale-out space; it does not promote distribution into the core execution spine.

## 1. Purpose and scope

This document records the later-stage distribution options that may be activated only after the bounded single-node system is proven.

- default deployment remains single-node with bounded Tier1/Tier2/Tier3 behavior
- sharding is a response to measured workload pressure, not architectural aesthetics
- simpler bounded remedies win first: budget tuning, cache/prefetch tuning, maintenance scheduling, repair throughput, and better operator tooling
- no strategy here may reopen early-phase contracts around bounded request-path work, namespace/policy enforcement, contradiction visibility, explainability, or repairability from durable truth

## 2. Promotion gate

Treat sharding or distribution as eligible only when all of the following have been assembled in one evidence bundle:

- sustained benchmark pressure against latency, throughput, rebuild, or maintenance budgets, captured as rerunnable artifacts rather than anecdote
- workload-shape evidence showing that namespace, workspace, temporal, or hot/cold locality no longer fits the single-node model
- operator pain that remains unacceptable after simpler bounded optimizations
- before-and-after comparison showing that budget tuning, cache or prefetch tuning, maintenance scheduling, repair throughput, and other bounded single-node remedies were attempted first and still left the system outside its intended operating envelope
- shard movement, repair, recovery, and rollback notes in `docs/OPERATIONS.md`
- failure and benchmark evidence for shard movement, repair, recovery, and cross-shard governance as required by `docs/TEST_STRATEGY.md`
- governance proof that policy enforcement, auditability, retention, and namespace isolation survive the proposed split

The benchmark portion of that evidence bundle should be machine-readable and rerunnable. At minimum it should preserve the benchmark target, iteration count, total duration, and latency fields already used elsewhere in the contract (`target`, `iterations`, `total_duration_ms`, `avg_duration_us`, `p50_duration_us`, `p95_duration_us`) so reviewers can compare the proposed promotion against the single-node baseline and the post-tuning baseline.

Distribution remains blocked when the observed problem can still be solved by bounded single-node improvements.

## 3. Shared invariants for any distributed design

Every future strategy must preserve the existing core contract:

- no full-store scans or uncapped broadcast fanout on request paths
- no policy or namespace bypass before shard selection or retrieval work
- no cross-shard recall that fetches cold payloads before the final candidate cut
- shard descriptors, placement metadata, caches, summaries, and sidecars may aid routing, but they must not become the sole source of truth for memory existence, lineage, policy, or contradiction state
- derived shard-local indexes, graphs, and caches must remain rebuildable from authoritative durable records
- degraded or partial cross-shard serving must be explicit to users and operators rather than silently presented as complete
- repair, rollback, and containment must be able to isolate the affected namespace or shard without widening blast radius unnecessarily

## 4. Namespace, consistency, and repair implications

Any future distributed design must make the harder semantics explicit before implementation starts.

### 4.1 Namespace and governance implications

Namespace remains the first execution boundary even when placement becomes more complex.

- shard routing may use namespace, workspace, time-range, or locality metadata, but effective namespace binding and policy checks must still resolve before widened candidate generation, cross-shard fanout, repair, or movement work begins
- placement metadata may describe where a namespace or memory currently lives, but it must not replace canonical namespace identity, ownership, visibility, retention, lineage, or contradiction records in durable truth
- namespace movement, splitting, or merge-like operations must preserve the same deterministic scope rules already required on a single node: same-namespace allow, explicit shared or public widening, malformed or unknown namespace rejection, and policy-denied behavior without protected existence leakage
- cross-shard public or shared recall must remain a governed widening step rather than a routing shortcut; routing convenience is not permission to reopen namespaces that the request was never allowed to see
- background maintenance, cache warming, replay, restore, and repair controllers must stay namespace-aware so degraded or recovering shards do not leak redacted payloads, protected counts, or contradiction siblings across boundaries
- operator-facing health, doctor, explain, audit, and repair surfaces must be able to name the affected namespace and shard scope separately so placement issues do not blur governance scope

### 4.2 Consistency posture and authoritative truth

Distribution must preserve the durable-truth-first hierarchy rather than introducing hidden semantic authorities.

- authoritative memory, lineage, policy, contradiction, and retention state remain the canonical source of truth even if replicas, shard descriptors, placement indexes, or caches lag behind
- cross-shard reads may merge bounded candidate sets, but they must not silently claim completeness when a shard is stale, unavailable, replaying, or serving under degraded posture
- eventual consistency may be acceptable for derived accelerators such as caches, ANN indexes, graph materializations, shard-local summaries, and placement sidecars, but not for whether canonical authoritative state was accepted, denied, superseded, redacted, or repaired
- any lag, replay, or failover posture that can change result completeness, freshness, or policy confidence must surface explicit degraded, partial, stale-warning, or blocked semantics instead of collapsing into an ordinary miss
- promotion or failover rules must prove that a replacement shard or replica has the authoritative generation, snapshot, or replay state needed for safe serving before it can advertise healthy authoritative reads
- cross-shard ranking and packaging must preserve omission markers, policy summaries, conflict markers, and trace detail so merged answers remain inspectable rather than becoming an opaque federation result

### 4.3 Repair, rollback, and containment implications

Scale-out is acceptable only if repair remains narrower than the failure.

- repair controllers must be able to isolate a single namespace, shard, replica set, or movement run without forcing fleet-wide rebuilds when authoritative durable inputs for the rest of the system remain healthy
- rebuilds of shard-local indexes, graphs, caches, and placement metadata must verify parity against durable truth before the repaired surface is marked healthy again
- shard movement, restore, and replay operations must preserve checkpoint, audit, and preflight evidence showing source scope, destination scope, affected namespaces, generation or snapshot anchors, and whether degraded or read-only posture was required during the run
- rollback must be defined for both placement mistakes and semantic drift: moving data back is not enough if policy markers, contradiction state, lineage, or retention semantics were widened or lost during the failed operation
- interrupted repair, rebalance, or failover runs must resume idempotently from durable checkpoints or explicit repair queue state rather than duplicating mutations or widening visibility during recovery
- containment posture must remain explicit: if governance, authoritative replay, or parity validation is uncertain, the affected scope should stay blocked, read-only, or degraded rather than silently rejoining normal service

## 5. Candidate strategies

### 5.1 Namespace sharding

Best fit when namespaces already form the dominant isolation and locality boundary.

- strengths: strong policy alignment, bounded blast radius, easier per-namespace maintenance windows
- main costs: hotspot namespaces, cross-namespace recall or duplication workflows, rebalance pressure when tenant sizes diverge
- follow-on concerns: namespace routing metadata, public/shared surface rules, shard-aware repair, and incident isolation

### 5.2 Workspace sharding

Best fit when workspaces, projects, or installations behave like mostly independent operating units.

- strengths: operational locality, clearer ownership boundaries, simpler export or migration units
- main costs: cross-workspace recall complexity, duplicated warm state, coordination overhead for shared/public artifacts
- follow-on concerns: workspace-scoped health, backup, migration, and recovery semantics

### 5.3 Time-range sharding

Best fit when recall and maintenance patterns are strongly time-local and retention windows dominate cost.

- strengths: bounded compaction windows, predictable archive movement, easier age-based maintenance
- main costs: cross-range recall, lifecycle transitions across shard boundaries, skew between hot recent data and historical truth
- follow-on concerns: retention enforcement, snapshot semantics, audit replay, and time-sliced repair

### 5.4 Hot/cold split

Best fit when operational pressure comes mainly from separating hot mutable working sets from colder durable history.

- strengths: preserves locality for the active set, simplifies cold storage policy, can improve maintenance scheduling
- main costs: more routing states, cold-path latency cliffs, stricter rules for what may prefetch or promote across the boundary
- follow-on concerns: Tier semantics, promotion/demotion policy, and cold-read degraded-mode signaling

### 5.5 Rebalancing

Not a primary shard axis by itself; this is the operational capability required once any shard axis exists.

- strengths: restores balance after growth or skew
- main costs: movement cost, transient degraded mode, rollback complexity
- follow-on concerns: move planning, audit trails, validation checkpoints, and safe interruption/restart semantics

### 5.6 Tenant isolation

Use only when legal, security, or blast-radius requirements demand stronger separation than a shared deployment can provide.

- strengths: clearer compliance boundaries, easier containment
- main costs: coordination overhead, duplicated infrastructure, harder shared recall or sharing features
- follow-on concerns: policy enforcement proofs, cross-tenant denial behavior, and incident handling as a security surface

### 5.7 Cross-shard recall

This is a bounded follow-on retrieval mode, not permission to broadcast requests across the fleet.

- it may open only from an already-authorized shard shortlist chosen by metadata, namespace, or locality routing
- it must preserve the existing request-path caps, candidate budgeting, and explainability requirements
- it should degrade to narrower or fail-closed behavior when budgets, policy, or shard availability make safe fanout impossible
- follow-on concerns: per-shard candidate caps, merged ranking traces, omission markers, and degraded-mode packaging

### 5.8 Shard-local caching

Cache layers remain derived accelerators even after sharding.

- strengths: locality-aware warm state, lower repeated remote lookup cost
- main costs: invalidation complexity, stale-state divergence, cold-start amplification after failover or repair
- follow-on concerns: generation anchors, owner-boundary binding, and cache bypass behavior during shard repair or migration

### 5.9 Replication

Use replication for availability and recovery, not as a substitute for authoritative semantics.

- strengths: better failover posture, disaster readiness
- main costs: lag, consistency validation, operational overhead
- follow-on concerns: authoritative promotion rules, failover auditability, and avoiding stale replicas as semantic truth

### 5.10 Disaster recovery

Treat DR as an operator and governance requirement around distribution, not a separate product feature.

- strengths: bounded recovery targets, clearer restore planning
- main costs: snapshot/export discipline, restore validation, recovery testing overhead
- follow-on concerns: shard-compatible backup formats, repair after restore, and proof that governance markers survive recovery

## 6. Follow-on implementation surfaces

If `mb-z0i.3.1` is ever activated beyond design notes, the next work should revisit:

- `docs/OPERATIONS.md` for shard movement, repair, recovery, rollback, degraded mode, and containment runbooks
- `docs/TEST_STRATEGY.md` for benchmark pressure, cross-shard governance, repair drills, and failure-injection evidence
- `docs/MCP_API.md` and `docs/CLI.md` only where operator-facing shard scope, degraded-mode reporting, or shard-aware repair/health surfaces become user-visible
- `docs/DATA_SCHEMAS.md` only for durable placement metadata that remains secondary to authoritative memory, lineage, policy, and contradiction records

## 7. Minimum evidence before implementation

Do not treat this document as permission to start distributed implementation. Before code or schema work begins, future contributors should be able to point to:

- the specific workload pressure that single-node bounded operation failed to absorb
- the chosen shard axis and the rejected alternatives
- the cross-shard recall, repair, and degraded-mode story
- the governance and tenant-isolation proof for the chosen split
- the benchmark, failure-injection, and recovery artifacts that satisfy the Phase 4 gate
- explicit proof that the single-node system was benchmarked, remains repairable from durable truth, and is operationally understood well enough that scale-out is solving measured demand rather than compensating for unknown single-node failure modes
