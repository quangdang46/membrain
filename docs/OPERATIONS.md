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

## Shared safeguard contract for preview, preflight, and destructive actions

This section owns the reusable safeguard contract referenced by `CLI.md`, `MCP_API.md`, and the plan's safe-preflight sandbox for risky or high-blast-radius operations.

It applies to forgetting, deletion, redaction, purge, invalidation, merge or namespace-scope rewrites, repair apply, compaction, compression, migration, rollback, restore, and any other operation whose blast radius can cross namespace boundaries, rewrite authoritative state, or emit irreversible-loss records. Pure read-only assessment surfaces such as `health`, `stats`, `audit`, `doctor`, snapshot listing, diff, or benchmark inspection stay outside confirmation flow unless they are the preflight step for a mutating follow-on.

### Operation classes and minimum safeguards

| Operation class | Typical examples | Minimum safeguard expectation |
|---|---|---|
| Class A — read-only assessment | `health`, `stats`, `audit`, `doctor`, snapshot list, diff, dry-run-only benchmark inspection | No confirmation required. May return `accepted` directly because no authoritative or derived state mutates. |
| Class B — derived-surface mutation | `repair index`, `repair graph`, `repair lineage`, `repair cache`, drop-and-rewarm, other rebuilds whose authoritative inputs remain readable | Preview is strongly expected. If the caller skips an explicit dry run, the system still computes readiness, affected scope, and degraded-mode requirements before apply. Block when authoritative inputs are unreadable, scope is ambiguous, or durable-truth fallback is unavailable. Reversibility must state that durable truth can rebuild the surface. |
| Class C — authoritative but logically reversible rewrite | `compress`, compaction, migration apply, namespace merge, large-scope invalidation, restore, rollback | Preview is required before mutation, either as an explicit dry-run request or as an embedded preflight in the same request. Confirmation is required. The preview must surface maintenance class, degraded or read-only posture, snapshot or rollback prerequisites, paused-job requirements, and bounded estimates for touched records or namespaces. |
| Class D — retention-affecting or irreversible mutation | hard delete, purge, destructive redaction, any forget path that can remove last authoritative evidence or emit irreversible-loss records | Preview is required. Explicit policy authorization and destructive confirmation are required. Block until the request can state the irreversibility boundary, authoritative-evidence-loss risk, audit plan, and any required backup or legal-hold outcome. |

### Shared safeguard object

Preview, blocked, rejected, and apply responses for Classes B-D should expose one consistent safeguard object even when CLI, daemon, and MCP package it differently.

| Field | Meaning |
|---|---|
| `outcome_class` | Shared outcome vocabulary: `preview`, `blocked`, `accepted`, `degraded`, or `rejected` as appropriate for the operation response. |
| `preflight_state` | `ready`, `blocked`, `missing_data`, or `stale_knowledge`. `ready` means safeguards are satisfied for this exact scope; it never means policy may be bypassed. |
| `operation_class` | The matched safeguard class above so operators know whether the path is read-only, derived-surface, authoritative rewrite, or retention/irreversible. |
| `affected_scope` | Effective namespace, shard, selector set, and bounded estimates for memories, payloads, indexes, caches, or namespaces touched; include whether public/shared widening participates. |
| `impact_summary` | What may move, rewrite, invalidate, redact, archive, delete, or temporarily degrade service, including any maintenance-window requirement, paused-job requirement, and explicit-loss risk. |
| `blocked_reasons` | Machine-readable reasons such as `confirmation_required`, `snapshot_required`, `maintenance_window_required`, `scope_ambiguous`, `authoritative_input_unreadable`, `policy_denied`, `legal_hold`, or `stale_preflight`. |
| `preflight_checks` | Structured results for policy, required-input, freshness, dependency, confidence, and generation/snapshot checks so the caller can see which gate passed, failed, or degraded. |
| `warnings` | Non-fatal freshness, namespace, policy-redaction, degraded-fallback, or confidence warnings that materially change how safely the result may be used. |
| `confidence_constraints` | Minimum evidence/confidence expectations for action-oriented guidance, including whether the current result is below a high-stakes threshold and what stronger evidence would be needed to upgrade it. |
| `reversibility` | One of `repairable_from_durable_truth`, `rollback_via_snapshot`, `partially_reversible`, or `irreversible`, plus the backup, snapshot, or repair prerequisites needed to recover safely. |
| `confirmation` | Whether local confirmation is required, what exact scope or generation it binds to, and whether `--force` or an allow token may satisfy only the local confirmation step. |
| `audit` | Event kind, actor/source, request id, preview/preflight id, related snapshot/repair/migration run, and the scope handles needed to correlate preview, apply, and rollback review. |

### Contract rules

- `--force` or an equivalent `preflight.allow` path may satisfy local confirmation only. It never bypasses policy denial, namespace isolation, retention rules, or legal hold.
- Confirmation must bind to the exact preflighted scope and generation. Widening namespace scope, changing selectors, or mutating after the underlying snapshot/generation moved invalidates the prior confirmation.
- `outcome_class=preview` means no mutation occurred and the safeguard object is the authoritative description of planned work.
- `outcome_class=blocked` means mutation did not proceed because a confirmation, freshness, scope, snapshot, maintenance-window, or similar readiness prerequisite is still missing. The response must include `blocked_reasons` plus the next missing prerequisite instead of relying on prose-only diagnostics.
- `outcome_class=rejected` is reserved for malformed, impossible, or policy-denied requests that cannot proceed even with local confirmation; it is not a synonym for confirmation-missing or stale-preflight states.
- `preview-only` and `force-confirmed` are preflight outcomes rather than separate top-level response classes: `preview-only` normally surfaces as `outcome_class=preview`, and `force-confirmed` surfaces through `confirmation` plus the remaining checks while policy, namespace, retention, and confidence constraints continue to apply.
- Successful apply responses should echo the final safeguard state or at least the `audit` correlation fields and any explicit loss, degraded-mode, or rollback markers that materially shaped the run.

### Preflight semantics for risky operations and high-stakes queries

The preflight surface complements preview and confirmation rather than replacing them. It may be exposed as explicit APIs such as `preflight.run`, `preflight.explain`, and `preflight.allow`, or embedded inside command/tool responses, but the underlying semantics should stay shared across CLI, daemon/JSON-RPC, and MCP.

#### Minimum preflight checks
- **policy check** — confirm namespace binding, sharing scope, retention/legal-hold state, and any deny/redaction rule before expensive work or mutation.
- **required-input check** — verify selectors, ids, snapshots, backups, and other mandatory inputs are present and mutually compatible.
- **freshness check** — detect stale snapshots, old preview generations, stale repair baselines, or degraded historical visibility.
- **dependency check** — confirm required maintenance windows, paused conflicting jobs, prerequisite snapshots, readable authoritative inputs, and dependent repair or migration state.
- **confidence check** — for action-oriented guidance or high-stakes queries, verify the evidence quality, conflict state, and freshness are strong enough for the requested action level.
- **generation check** — verify the scope and generation anchored by preview or prior confirmation still match current state.

#### Shared preflight outcomes
- **allowed / ready** — all required checks passed for the current scope; execution may proceed, subject to any confirmation requirement already declared.
- **preview-only** — the operation may be described safely, but mutation or high-confidence action guidance cannot proceed until missing confirmation, stronger evidence, or another prerequisite is satisfied.
- **blocked** — execution must not proceed; the response includes actionable `blocked_reasons` and the failing checks.
- **degraded** — execution or answer construction may continue only through a lower-fidelity path that remains explicit about colder reads, repair-aware serving, stale visibility, or uncertainty.
- **force-confirmed** — local confirmation has been supplied for the exact preflighted scope, but policy, namespace, retention, and confidence constraints still apply unchanged.

#### High-stakes query rules
- High-stakes actions should surface uncertainty by default. If evidence is stale, contradictory, too sparse, or policy-limited, the preflight should prefer `preview-only`, `blocked`, or `degraded` outcomes over silently emitting confident action guidance.
- `confidence_constraints` should state the minimum evidence quality required for action-oriented output and, when not met, what `change_my_mind_conditions` or stronger evidence would be needed to upgrade the answer.
- Intent-routed `ask` surfaces should preserve when low classifier confidence, explicit `override-intent`, or action-oriented intent classes changed the retrieval or packaging plan. Those route changes belong in machine-readable warnings or explanation fields rather than disappearing into a generic answer string.
- Missing and unauthorized data must remain distinguishable at the internal preflight boundary even when external presentation redacts details for security reasons.
- If the best route for a high-stakes or action-oriented query is blocked by stale knowledge, policy-limited visibility, or insufficient evidence, the system should return `preview-only`, `blocked`, or `degraded` semantics explicitly instead of silently falling back to a broader or lower-safety retrieval plan.
- Preflight warnings should preserve whether the risk comes from stale knowledge, missing inputs, policy-limited visibility, unresolved conflict, low intent confidence, explicit override, or degraded repair state so callers can decide whether to stop, inspect, or continue with bounded caution.

### Operation-family minimums

- **Forgetting, deletion, redaction, and invalidation** previews must report affected authoritative-evidence counts, retention or legal-hold blockers, whether losing evidence remains inspectable afterward, and whether the path is archive-by-default, redaction-only, or irreversible removal.
- **Merge or namespace-scope rewrites** must report source and target scopes, visibility or sharing changes, conflict-handling policy, and the rollback or unwind path before mutation.
- **Repair and rebuild apply** must report the authoritative input set used for rebuild, degraded-serving posture while repair is in flight, unresolved follow-up items, and whether explicit loss records may be emitted if full fidelity cannot be restored.
- **Compaction, compression, migration, rollback, and restore** must report maintenance class, pre-run snapshot or backup requirement, paused-job requirements, before/after telemetry expectations, rollback owner, and whether the run rewrites authoritative durable layout or only derived surfaces.

## Phase 4 operational ergonomics follow-on contract

- This surface is later-stage maturity work. It activates only after core runbooks, degraded-mode behavior, snapshot/rollback discipline, repair flows, and baseline operator diagnostics are already explicit and trustworthy.
- Scope includes operator-facing guidance or automation such as runbook selection aids, maintenance-window planning helpers, audit/incident correlation, repair-queue triage, recurring health-review packaging, and other ergonomics layers built on existing operational signals.
- These helpers remain subordinate to the canonical operations contract. They do not replace required preconditions, approval paths, snapshot requirements, rollback rules, namespace isolation, or policy checks, and they must not silently widen scope or execute destructive maintenance by default.
- Automation should consume auditable inputs that already exist elsewhere in the system — `health`, `stats`, `doctor`, `audit`, benchmark artifacts, snapshot metadata, maintenance reports, and repair or compaction outcomes — rather than opaque hidden state or untraceable heuristics.
- Any recommended or scheduled action must stay previewable, bounded, and explainable: operators should be able to see the triggering evidence, affected scope, required maintenance class, and why the system chose recommendation, degraded mode, or no-op.
- Later-stage ergonomics may queue background follow-up work or assemble operator context, but they must not become a prerequisite for normal request-path correctness, phase promotion, or ordinary maintenance success, and they must fail safely by emitting inspectable suggestions instead of hidden mutations.
- Future validation should prove low operational overhead, semantic parity across CLI, daemon, IPC, and MCP surfaces that expose the guidance, preserved auditability of assisted actions, and correct containment when source signals are stale, partial, or policy-restricted.

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
  total_engrams: .total_engrams,
  avg_cluster_size: .avg_cluster_size,
  top_engrams: .top_engrams[:5],
  last_dream: .last_dream_tick,
  repair_queue_depth: .repair_queue_depth,
  backpressure_state: .backpressure_state,
  availability_posture: .availability_posture,
  feature_availability: .feature_availability
}'
membrain doctor run
```

### Metrics to Watch
- Hot utilization trending up → schedule consolidation
- Average confidence dropping → investigate conflict rate
- Unresolved conflicts accumulating → trigger belief resolution
- Engram count or average cluster size drifting sharply → investigate graph fanout, split pressure, or stale rebuild state
- Repair queue depth growing or staying non-zero → inspect repair backlog growth before it becomes degraded serving
- Backpressure state leaving `normal` or availability posture leaving `full` → switch from routine review to the matching runbook or containment path
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

#### Shared availability object for degraded or partial service

Whenever degraded mode, repair posture, fallback, or fail-closed containment materially changes what callers can do, the response should expose one inspectable availability summary across CLI, daemon/JSON-RPC, and MCP.

| Field | Meaning |
|---|---|
| `posture` | `full`, `degraded`, `read_only`, or `offline` for the affected scope. |
| `affected_scope` | Namespace, shard, feature, or selector range whose availability changed. |
| `query_capabilities` | Which read paths remain available, such as durable-truth lookup, bounded recall, inspect, audit, explain, or health/doctor surfaces. |
| `mutation_capabilities` | Which writes remain available, preview-only, blocked, or refused. |
| `degraded_reasons` | Machine-readable reasons such as `graph_unavailable`, `index_bypassed`, `cache_invalidated`, `repair_in_flight`, `stale_generation`, or `authoritative_input_unreadable`. |
| `recovery_conditions` | The checks, repairs, or validations required before the scope may return to normal serving. |
| `availability_notes` | Human-facing summary of what changed and which safer fallback path is still available. |

Availability rules:
- `degraded` means some bounded queries still work, but the response must say which read paths survived and which fidelity or latency guarantees were reduced.
- `read_only` means query and inspect surfaces may remain available, but authoritative or derived-surface mutation must be blocked or preview-only until validation clears the scope.
- `offline` means the affected scope must refuse ordinary query and mutation traffic, leaving only explicitly allowed health, audit, doctor, or recovery-oriented surfaces.
- If correctness, policy isolation, or authoritative-input readability is ambiguous, prefer `read_only` or `offline` over best-effort mutation.
- A downgraded write path must surface `blocked` or `preview` semantics rather than pretending that degraded mode silently converted a mutating request into a normal success.
- Degraded availability must preserve policy and namespace boundaries exactly; fallback cannot widen scope, leak protected counts, or hydrate payloads that colder policy checks would deny.

### Verification contract for restart, rebuild, and recovery

Every restart, rebuild, or recovery runbook should leave operator-verifiable evidence that the system either returned to healthy service or remained explicitly degraded. At minimum capture:

- the scenario class (`clean_restart`, `derived_rebuild`, `degraded_recovery`, or `fail_closed_recovery`) and affected namespace or shard scope
- the authoritative inputs or generations validated at start, including whether canonical rows were readable and which snapshot or repair baseline anchored the run
- the serving posture during the run: full service, slower durable-truth fallback, graph or index bypass, read-only, or offline
- the exact parity checks used to clear the surface: durable-count or candidate-count parity, lineage or graph consistency, namespace and policy isolation, cache-generation freshness, and queued repair or irreversible-loss status
- the release criteria for clearing degraded mode, including which metrics, inspect or explain outputs, and audit records must pass first
- any remaining degraded scope or queued follow-up repair if the run stops short of full recovery

#### Scenario-specific proof rules
- **Clean restart** must prove canonical counters, generations, and durable handles were restored without trusting stale or mixed-generation warm state.
- **Derived-surface rebuild** must prove rebuilt indexes, graph materializations, lineage views, or caches match durable truth before the surface is advertised as healthy.
- **Degraded recovery** must keep the fallback path explicit and preserve the audit trail until final validation clears it.
- **Fail-closed recovery** must keep the affected scope unavailable or read-only until canonical validation passes; partial availability is not a substitute for proof.

### Lifecycle transition repair handoff
- Failed lifecycle transitions reuse the same durable-truth-first repair model: prior durable state stays authoritative while derived follow-up work is repaired or replayed.
- Only retryable internal failures should create automatic repair or replay work. Validation failures and policy denials are recorded but must not spin in automated retry loops.
- Repair queue entries for failed transitions should preserve object handle, namespace or shard scope, prior valid state, attempted edge, failure family, retryability, and escalation boundary.
- When retry budgets are exhausted, the system should escalate to operator-visible repair or incident handling and may place the affected namespace or shard into degraded mode if correctness is at risk.
- Pending repair must not silently widen authority: replay controllers keep the same namespace, visibility, and policy scope as the failed transition.

---

## 5.2 Graph and lineage repair

### Preconditions
- Canonical edge tables, engram membership rows, and lineage records are readable.
- Operators know whether the affected scope is missing only derived graph materializations or whether canonical graph rows themselves are suspect.
- Conflicting background compaction or migration jobs are paused.

### Persisted-versus-rebuildable graph surfaces
- Durable graph truth is limited to normalized canonical edge rows, stable engram records, parent-child engram lineage, and durable membership mappings needed to answer which memory belongs to which cluster.
- In-memory petgraph state, centroid ANN sidecars, neighborhood materializations, graph-health summaries, and similar warm surfaces are rebuildable accelerators.
- Traversal counters or hotness hints such as edge activation counts may persist for observability, but restart correctness must not depend on preserving them exactly; if they are stale or dropped, the system may reset them while keeping canonical edge truth intact.
- A serialized graph dump or checkpoint may accelerate restart, but it is never the sole source of truth and must always be replaceable from canonical durable rows.

### Command Sequence
```bash
membrain doctor run
membrain repair graph --dry-run
membrain repair graph --namespace default
membrain repair lineage --dry-run
membrain repair lineage --namespace default
membrain health --json | jq '{
  total_engrams: .total_engrams,
  avg_cluster_size: .avg_cluster_size,
  top_engrams: .top_engrams[:5]
}'
```

### Restart integrity contract
- Startup validates graph schema or format generation, durable edge-table readability, engram-membership readability, and deterministic handle mapping before graph-enabled recall is advertised as healthy.
- If only derived graph materializations are missing or stale, startup may continue in a colder graph-bypassed or graph-lazy-rewarm mode while authoritative durable rows remain available.
- If canonical edge tables or stable membership mappings are unreadable or mixed-generation, affected namespaces must fall back to graph-disabled degraded serving or read-only or offline posture until validation or repair passes.
- Restart is successful only when graph-enabled recall, inspect, and explain paths can resolve canonical endpoints again without trusting stale sidecars or mixed-generation dumps.

### Metrics to Watch
- Canonical edge count vs rebuilt materialized edge count
- Total engrams, average cluster size, and top-engram drift against expected baselines
- Broken-lineage and orphan-membership count before/after
- Graph-disabled, graph-bypassed, stale-graph, or degraded-mode rates during restart and repair
- Repair queue depth for follow-on unresolved items
- Any explicit loss records emitted during repair

### Rollback Conditions
- Repaired graph output diverges further from canonical edge tables
- Broken-lineage or orphan-membership count increases after mutation
- Graph-enabled recall or explain remains dependent on stale sidecars after validation
- Online retrieval latency breaches budget during the repair window

### Post-run Validation
- `memory_explain` can traverse lineage and graph ancestry again
- `graph(id)` or equivalent inspect surface resolves cluster membership and related memories from current canonical handles
- Health or stats surfaces expose sane `total_engrams`, `avg_cluster_size`, and bounded top-engram summaries for the affected scope
- No unresolved orphan nodes or broken ancestry chains remain without queued follow-up repair
- Any still-degraded graph-disabled or graph-bypassed scope remains explicitly declared to operators rather than silently serving partial graph results

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

Treat audit as the canonical operation-history surface for incident review: correlate entries with the affected namespace or shard, actor/source, repair or migration run, and any snapshot-protected maintenance event before deciding whether the observed state came from policy, degradation, or stale derived state.

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
