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

- sustained benchmark pressure against latency, throughput, rebuild, or maintenance budgets
- workload-shape evidence showing that namespace, workspace, temporal, or hot/cold locality no longer fits the single-node model
- operator pain that remains unacceptable after simpler bounded optimizations
- shard movement, repair, recovery, and rollback notes in `docs/OPERATIONS.md`
- failure and benchmark evidence for shard movement, repair, recovery, and cross-shard governance as required by `docs/TEST_STRATEGY.md`
- governance proof that policy enforcement, auditability, retention, and namespace isolation survive the proposed split

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

## 4. Candidate strategies

### 4.1 Namespace sharding

Best fit when namespaces already form the dominant isolation and locality boundary.

- strengths: strong policy alignment, bounded blast radius, easier per-namespace maintenance windows
- main costs: hotspot namespaces, cross-namespace recall or duplication workflows, rebalance pressure when tenant sizes diverge
- follow-on concerns: namespace routing metadata, public/shared surface rules, shard-aware repair, and incident isolation

### 4.2 Workspace sharding

Best fit when workspaces, projects, or installations behave like mostly independent operating units.

- strengths: operational locality, clearer ownership boundaries, simpler export or migration units
- main costs: cross-workspace recall complexity, duplicated warm state, coordination overhead for shared/public artifacts
- follow-on concerns: workspace-scoped health, backup, migration, and recovery semantics

### 4.3 Time-range sharding

Best fit when recall and maintenance patterns are strongly time-local and retention windows dominate cost.

- strengths: bounded compaction windows, predictable archive movement, easier age-based maintenance
- main costs: cross-range recall, lifecycle transitions across shard boundaries, skew between hot recent data and historical truth
- follow-on concerns: retention enforcement, snapshot semantics, audit replay, and time-sliced repair

### 4.4 Hot/cold split

Best fit when operational pressure comes mainly from separating hot mutable working sets from colder durable history.

- strengths: preserves locality for the active set, simplifies cold storage policy, can improve maintenance scheduling
- main costs: more routing states, cold-path latency cliffs, stricter rules for what may prefetch or promote across the boundary
- follow-on concerns: Tier semantics, promotion/demotion policy, and cold-read degraded-mode signaling

### 4.5 Rebalancing

Not a primary shard axis by itself; this is the operational capability required once any shard axis exists.

- strengths: restores balance after growth or skew
- main costs: movement cost, transient degraded mode, rollback complexity
- follow-on concerns: move planning, audit trails, validation checkpoints, and safe interruption/restart semantics

### 4.6 Tenant isolation

Use only when legal, security, or blast-radius requirements demand stronger separation than a shared deployment can provide.

- strengths: clearer compliance boundaries, easier containment
- main costs: coordination overhead, duplicated infrastructure, harder shared recall or sharing features
- follow-on concerns: policy enforcement proofs, cross-tenant denial behavior, and incident handling as a security surface

### 4.7 Cross-shard recall

This is a bounded follow-on retrieval mode, not permission to broadcast requests across the fleet.

- it may open only from an already-authorized shard shortlist chosen by metadata, namespace, or locality routing
- it must preserve the existing request-path caps, candidate budgeting, and explainability requirements
- it should degrade to narrower or fail-closed behavior when budgets, policy, or shard availability make safe fanout impossible
- follow-on concerns: per-shard candidate caps, merged ranking traces, omission markers, and degraded-mode packaging

### 4.8 Shard-local caching

Cache layers remain derived accelerators even after sharding.

- strengths: locality-aware warm state, lower repeated remote lookup cost
- main costs: invalidation complexity, stale-state divergence, cold-start amplification after failover or repair
- follow-on concerns: generation anchors, owner-boundary binding, and cache bypass behavior during shard repair or migration

### 4.9 Replication

Use replication for availability and recovery, not as a substitute for authoritative semantics.

- strengths: better failover posture, disaster readiness
- main costs: lag, consistency validation, operational overhead
- follow-on concerns: authoritative promotion rules, failover auditability, and avoiding stale replicas as semantic truth

### 4.10 Disaster recovery

Treat DR as an operator and governance requirement around distribution, not a separate product feature.

- strengths: bounded recovery targets, clearer restore planning
- main costs: snapshot/export discipline, restore validation, recovery testing overhead
- follow-on concerns: shard-compatible backup formats, repair after restore, and proof that governance markers survive recovery

## 5. Follow-on implementation surfaces

If `mb-z0i.3.1` is ever activated beyond design notes, the next work should revisit:

- `docs/OPERATIONS.md` for shard movement, repair, recovery, rollback, degraded mode, and containment runbooks
- `docs/TEST_STRATEGY.md` for benchmark pressure, cross-shard governance, repair drills, and failure-injection evidence
- `docs/MCP_API.md` and `docs/CLI.md` only where operator-facing shard scope, degraded-mode reporting, or shard-aware repair/health surfaces become user-visible
- `docs/DATA_SCHEMAS.md` only for durable placement metadata that remains secondary to authoritative memory, lineage, policy, and contradiction records

## 6. Minimum evidence before implementation

Do not treat this document as permission to start distributed implementation. Before code or schema work begins, future contributors should be able to point to:

- the specific workload pressure that single-node bounded operation failed to absorb
- the chosen shard axis and the rejected alternatives
- the cross-shard recall, repair, and degraded-mode story
- the governance and tenant-isolation proof for the chosen split
- the benchmark, failure-injection, and recovery artifacts that satisfy the Phase 4 gate
