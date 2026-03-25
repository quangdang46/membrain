# membrain — Lifecycle State Machines

> Canonical sources: `MEMORY_MODEL.md` lifecycle contract, `OPERATIONS.md` repair flow, and `SECURITY_GOVERNANCE.md` namespace and ACL rules.

This document defines the lower-level transition vocabulary, guard matrix, failure semantics, and repair handoff rules that support the canonical persisted lifecycle for memory objects. If this document diverges from `MEMORY_MODEL.md`, the persisted lifecycle in `MEMORY_MODEL.md` wins.

## Persisted lifecycle versus controller transitions

The canonical persisted lifecycle for memory items is a closed durable enum:

- `Labile`
- `SynapticDone`
- `Consolidating`
- `Consolidated`
- `Superseded`
- `Archived`

Legal durable edges are:

- `Labile -> SynapticDone`
- `SynapticDone -> Consolidating`
- `Consolidating -> Consolidated`
- `SynapticDone -> Labile` on successful recall reopen
- `Consolidated -> Labile` on successful recall reopen
- `Labile -> Superseded` only through explicit contradiction-resolution artifacts
- `SynapticDone -> Superseded` only through explicit contradiction-resolution artifacts
- `Consolidated -> Superseded` only through explicit contradiction-resolution artifacts
- `SynapticDone -> Archived` when forgetting, retention, or explicit archive policy allows
- `Consolidated -> Archived` when forgetting, retention, or explicit archive policy allows
- `Superseded -> Archived` when retention policy allows the superseded record to leave active serving while remaining auditable

Every other durable edge is forbidden unless a stricter canonical contract later adds it explicitly.

Lower-level controller outcomes such as `created`, `indexed`, `recalled`, `reinforced`, `decayed`, `demoted`, `restored`, and `deleted` are operational transition classes or maintenance outcomes, not a second conflicting persisted lifecycle enum. They describe **how** an object moves or is served, while the persisted lifecycle describes **what durable state the memory is currently in**. In particular, the older `created -> indexed -> recalled -> reinforced -> decayed -> demoted -> archived -> deleted` prose should be read as controller activity vocabulary, not as the durable lifecycle state enum.

## Canonical guard vocabulary

Every lifecycle transition validates guards from a shared vocabulary so controllers, interfaces, and operators do not invent incompatible rules.

| Guard | Meaning | Typical failure family when violated |
|---|---|---|
| namespace access control | The caller, job, or maintenance controller is allowed to touch the object's effective namespace and any visibility-widened surfaces. | `policy_denied` |
| policy pinning / legal hold / retention rule | The requested edge does not bypass pins, legal holds, retention class, deletion guarantees, or other policy locks. | `policy_denied` |
| contradiction / supersession compatibility | The edge is compatible with current conflict state and does not erase unresolved disagreement or misuse `Superseded`. | `validation_failure` |
| lineage preservation | The transition preserves required parent links, source evidence, relation endpoints, and durable-truth traceability. | `validation_failure` or `internal_failure` |
| repair-job lock | No conflicting repair, rebuild, migration, or replay controller currently owns the object or required derived surfaces. | `internal_failure` |
| endpoint / durable-reference integrity | Required referenced memories, entities, relations, content handles, or shard bindings still resolve or are explicitly tombstoned. | `validation_failure` or `internal_failure` |
| controller scope binding | Background jobs and maintenance controllers carry a deterministic namespace, shard, or repair-plan binding before mutating state. | `validation_failure` |

## Object-level transition matrix

| Object family | Allowed lifecycle edges | Additional required guards | Forbidden edges / notes |
|---|---|---|---|
| Event, Episode, Observation, ToolOutcome | `Labile -> SynapticDone`; `SynapticDone -> Consolidating -> Consolidated`; `SynapticDone|Consolidated -> Labile` on successful recall; any non-pinned stable state `-> Archived` when forgetting policy allows | provenance/source integrity, lineage preservation | no direct jump from `Labile` or `Consolidating` to deletion; no archival or deletion that bypasses retention policy; `Superseded` is not the normal outcome for routine decay or compaction |
| Fact, Summary, Goal, Skill, Constraint, Hypothesis, UserPreference | same edges as above, plus `{Labile|SynapticDone|Consolidated} -> Superseded` only through explicit contradiction-resolution artifacts | contradiction/supersession compatibility, provenance and lineage preservation | no ordinary recall path may reopen `Superseded` into `Labile`; no silent overwrite in place; no direct delete bypassing archival or policy |
| Relation | `Labile -> SynapticDone`; `SynapticDone -> Consolidated` once endpoints and policy checks pass; `Consolidated -> Archived` when retention or endpoint-tombstone policy allows | endpoint integrity, namespace compatibility, relation-kind normalization, contradiction coexistence | no automatic overwrite of competing relation edges; no cross-namespace link without explicit policy support; no delete while authoritative endpoints still require the relation for explainability |
| ConflictRecord | `Labile -> SynapticDone -> Consolidated`; `Consolidated -> Archived` only after resolution and retention rules allow it | linked-evidence integrity, authority metadata when resolved, namespace access control | unresolved conflicts must not be archived or deleted while they remain needed to explain operative state; conflict closure must not mutate losing evidence in place |
| Retention / decay / shard-related controller state | controller-owned updates may accompany lifecycle edges or maintenance actions, but they do not independently authorize a memory lifecycle jump | controller scope binding, repair-job lock, policy precedence | no direct user-facing `recalled`, `reinforced`, or `superseded` edge on controller state; no namespace changes or identity reuse via controller metadata alone |

## Workflow-to-lifecycle guard map

This section binds the major workflows called out in the canonical plan to the legal durable edges above so later background jobs, policy evaluators, and audit surfaces do not guess which lifecycle transition they are allowed to perform.

| Workflow | Legal durable edge(s) | Required guards before commit | Inspectable guard-failure expectations |
|---|---|---|---|
| encode | create new durable object in `Labile` only; never a direct jump into `SynapticDone`, `Consolidated`, `Superseded`, or `Archived` | namespace access control; policy pinning / legal hold / retention rule; lineage preservation; endpoint / durable-reference integrity for referenced entities; controller scope binding when a background ingest job performs the write | failures emit an auditable artifact that names the rejected create or edge attempt, failed guard family, and why the object was not admitted into the durable lifecycle |
| consolidation | `Labile -> SynapticDone`; `SynapticDone -> Consolidating`; `Consolidating -> Consolidated` | namespace access control; policy pinning / legal hold / retention rule; lineage preservation; repair-job lock; endpoint / durable-reference integrity; controller scope binding | failures must show which stage of the consolidation pipeline was attempted, whether prior durable state remained authoritative, and whether derived refresh was queued for repair |
| forgetting | `SynapticDone -> Archived`; `Consolidated -> Archived`; `Superseded -> Archived` only when retention policy allows | policy pinning / legal hold / retention rule; contradiction / supersession compatibility; lineage preservation; repair-job lock; controller scope binding for batch forgetting jobs | failures must name whether forgetting was blocked by legal hold, retention class, unresolved contradiction, or controller-scope mismatch rather than silently leaving the object half-archived |
| lease scan | no durable lifecycle edge by itself; bounded freshness transitions only (`fresh -> lease_sensitive -> stale -> recheck_required`) | deterministic logical clock, explicit lease policy, controller scope binding, repair-job lock if derived freshness indexes are rewritten | failures must preserve the prior freshness state, avoid request-path full scans, and surface whether stale action-critical evidence was downgraded, recheck-flagged, or withheld |
| explicit archive | `SynapticDone -> Archived`; `Consolidated -> Archived`; `Superseded -> Archived` when an operator or policy action requests archival | namespace access control; policy pinning / legal hold / retention rule; contradiction / supersession compatibility; lineage preservation; repair-job lock | failures must preserve the prior stable state and expose the denied archive request, requesting actor or controller, and blocking policy or validation cause |
| restore | `Archived -> Labile` only through an explicit restore, replay, or policy-approved maintenance path; ordinary recall is not a restore path | namespace access control; policy pinning / legal hold / retention rule; lineage preservation; endpoint / durable-reference integrity; repair-job lock; controller scope binding | failures must show that restore was explicitly requested, why ordinary recall was insufficient, and which guard prevented reopening the archived durable row |

Restore execution order is fixed and inspectable:
1. `validate_restore_request`
2. `load_durable_metadata`
3. `rehydrate_available_payload`
4. `commit_durable_state`
5. `refresh_derived_state`

If `rehydrate_available_payload` encounters tombstoned or unavailable cold payload state, the workflow still may complete as a degraded `partially_restored` reopen so long as durable identity, lineage, policy state, and bounded inspect surfaces remain intact.

### Workflow-specific guard notes

- **Encode admission is deterministic.** The encode fast path may create a new `Labile` object only after namespace, provenance, and policy admission checks pass. It does not authorize lifecycle shortcuts such as pre-marking the object `SynapticDone`, directly archiving low-quality input, or overwriting an existing row in place.
- **Consolidation is staged, not magical.** Consolidation jobs may only advance along the explicit stable-state edges above. Derived rebuilds, cache warming, compression, payload detachment, or engram maintenance may accompany the workflow, but they do not independently authorize a durable edge.
- **Forgetting and explicit archive share the same archive gate.** Both are archival workflows, so both must satisfy the same retention, legal-hold, contradiction, and auditability checks before the durable state may become `Archived`.
- **Restore is exceptional and policy-visible.** Reopening an archived memory requires an explicit restore or replay intent that is inspectable in explain and audit surfaces; ordinary recall, cache hydration, or background warming must never smuggle `Archived -> Labile`.
- **Guard failures are first-class evidence.** Any failed workflow attempt must leave an inspectable artifact naming the attempted workflow, attempted durable edge, prior durable state, failure family, and whether repair follow-up was created.

## Recall reopen and reconsolidation-apply contract

When successful recall reopens a stable memory into `Labile`:

- recall opens a bounded mutation window only; it does not itself submit or apply a semantic content change
- any pending update must be accepted before the window expires and while namespace, policy, contradiction, lineage, and repair-job guards still pass
- if an in-window content update would materially contradict the reopened memory's current claim, the system must route through explicit contradiction handling and mint a new belief-chain member instead of mutating the reopened row in place
- accepted reconsolidation mutates the authoritative durable row first, then refreshes derived embeddings, ANN or FTS indexes, caches, and other warm surfaces; if a derived refresh fails, the durable update remains authoritative and the derived surface is marked stale or queued for repair
- if the window expires first, stale pending updates are rejected or discarded explicitly and the memory restabilizes to its pre-reopen durable stable state without applying them
- reconsolidation reopen and restabilization do not silently demote a previously `Consolidated` memory, promote an unconsolidated one, or change canonical durable ownership; those edges remain explicit controller actions
- ordinary recall must not use reconsolidation to smuggle in silent overwrite, forced supersession, policy bypass, or an implicit restore of `Archived` material

## Forbidden edge classes

The following are globally forbidden unless a stricter subsystem contract explicitly allows them and preserves auditability:

- any durable edge not listed in the persisted enum contract above
- skipping directly from active mutable states to hard deletion
- turning `Superseded` into a routine decay, archival, or compaction synonym
- reopening `Archived` or `Superseded` objects through ordinary recall without an explicit restore, replay, or contradiction-resolution path
- mutating lifecycle while a repair, migration, or replay controller holds the repair-job lock
- using cross-namespace movement as a lifecycle transition instead of an explicit share, fork, merge, or re-ingest flow
- treating controller metadata changes (retention, decay, shard routing) as authorization to bypass namespace or policy checks
- treating demotion, payload detachment, cache warming, index rebuild, or shard migration as an implicit durable lifecycle change

## Deterministic test and fixture obligations

Any implementation of this state machine must ship deterministic evidence that the durable enum and legal edges cannot drift silently.

Minimum required coverage:

- enum fixture coverage that names every durable state exactly as `Labile`, `SynapticDone`, `Consolidating`, `Consolidated`, `Superseded`, and `Archived`
- positive transition fixtures for every legal durable edge listed in this document
- negative transition fixtures for representative forbidden edges, including at least `Labile -> Archived`, `Labile -> Consolidated`, `Consolidating -> Archived`, `Archived -> Labile`, `Superseded -> Labile`, and any direct hard-delete jump from an active state
- guard-failure fixtures that prove `validation_failure`, `policy_denied`, and `internal_failure` preserve the prior durable state and emit an auditable failure artifact
- deterministic recall-window fixtures that prove reopen only allows `SynapticDone|Consolidated -> Labile`, rejects stale in-window updates after expiry, and routes material contradiction through explicit supersession flow rather than silent overwrite
- archive-path fixtures that prove archiving stays inspectable and auditable and that ordinary recall does not silently restore `Archived` or `Superseded` items
- workflow-guard fixtures that explicitly cover legal and illegal attempts for encode admission, staged consolidation, forgetting-driven archival, explicit archive requests, and explicit restore requests, with deterministic artifact names such as `lifecycle_encode_guard_fixture`, `lifecycle_consolidation_stage_fixture`, `lifecycle_forgetting_archive_guard_fixture`, `lifecycle_explicit_archive_guard_fixture`, and `lifecycle_restore_guard_fixture`

The fixture or test artifact names must make the covered edge explicit so drift is visible in review. Representative naming patterns include `lifecycle_enum_fixture`, `lifecycle_edge_matrix_fixture`, `lifecycle_forbidden_edges_fixture`, `lifecycle_failure_preserves_prior_state`, and `lifecycle_recall_window_fixture`.

## Transition failure contract

### Failure families

Every failed lifecycle transition must classify itself as one of:

- `validation_failure` — the requested edge is malformed, forbidden, stale, or missing prerequisites such as endpoints, lineage, or deterministic controller scope
- `policy_denied` — namespace ACL, workspace ACL, agent ACL, session visibility, retention, legal hold, or pinning forbids the transition
- `internal_failure` — durable writes, locks, derived-state dependencies, rebuild prerequisites, or controller execution fail after the request passed validation and policy checks

### Prior-state preservation rules

If a transition does not commit successfully:

- the last known valid persisted lifecycle state remains authoritative
- `version` and other commit-coupled durable state do not advance
- partially prepared derived-state side effects must be rolled back or marked stale for later repair
- inspect, explain, and audit surfaces must show that the transition was attempted and failed, without pretending the new state committed
- the object must never be left in a silent half-transitioned state

### Transition-failure artifact minimum

A failed transition must emit an auditable artifact or event that preserves at least:

- object handle and effective namespace
- prior durable state
- attempted edge or controller action
- failure family plus the guard or subsystem that failed
- triggering request, job, or controller handle when available
- whether the failure is retryable
- repair handle or queue reference when follow-up work was created

## Repair handoff and retry boundaries

### When repair handoff is required

A failed transition should enqueue repairable follow-up work only when the failure is repairable without changing the semantic intent of the request, such as:

- transient durable-write or lock conflicts
- stale or missing derived surfaces that can be rebuilt from durable truth
- replayable controller work that stopped after preserving prior state but before completing derived updates

Validation failures and policy denials are terminal for that attempted edge. They are recorded, but they are not auto-retried.

### Retry boundaries

- automatic retries must be bounded by controller-owned retry budgets and cooldown rules
- the same object and edge must not spin indefinitely on the request path or in background replay
- once retry budget is exhausted, the system escalates to an operator-visible repair or incident surface
- if repeated failures threaten correctness for one namespace or shard, the system should degrade or narrow that surface rather than masking the issue with repeated best-effort retries

### Behavior while repair is pending

- reads continue from the last valid durable state
- inspect, explain, and audit should expose that repair is pending or that a retry boundary has been reached
- conflicting writes may be blocked, rejected, or forced through an explicit reconciliation path rather than racing the pending repair
- repair queues and replay controllers must preserve namespace, shard, and policy scope so follow-up work does not widen authority accidentally

## Deterministic tick, clock, retry-budget, and timeout semantics

Timing-sensitive lifecycle behavior must be driven by deterministic logical time, not ambient scheduler timing or wall-clock sleeps. This keeps request-path behavior, repair replay, and test evidence stable across machines and runs.

### Canonical time source

- the canonical lifecycle clock is the monotonic interaction tick restored from durable state and advanced by the controller actions the canon already names, not by wall-clock elapsed time
- any workflow that evaluates decay, reconsolidation windows, retry cooldowns, timeout expiry, or repair replay deadlines must consume an explicit `current_tick` or equivalent injected logical clock value
- replay, recovery, and fixture execution must preserve monotonic tick progression; they must not substitute local wall time, scheduler jitter, or process uptime as a hidden time source
- wall-clock timestamps may still appear in audit and observability artifacts, but they are descriptive metadata, not authorization to advance lifecycle state

### Timeout and retry semantics

- timeout-sensitive lifecycle decisions must be expressed in logical ticks or deterministic attempt budgets that are evaluated against the injected logical clock
- bounded mutation windows, retry cooldowns, repair backoff, and replay cutoffs must remain deterministic under fixture control; sleeping for an approximate duration is not acceptable evidence that a timeout path works
- retryable `internal_failure` paths may use controller-owned retry budgets and cooldown rules, but both the remaining budget and the next eligible tick must be inspectable in repair or audit surfaces
- once the retry budget is exhausted, the object remains on its last valid durable state and escalates to repair or incident handling without hidden extra attempts
- `validation_failure` and `policy_denied` remain terminal for the attempted edge even if wall clock passes; time passing alone must not convert them into retryable work

### Deterministic fixture and harness obligations

- every timing-sensitive fixture must name the injected tick source, starting tick, tick advances, and the exact edge or failure boundary being proven
- fixture and harness APIs should support explicit tick advancement, deterministic cooldown expiry, and deterministic timeout expiry without `sleep`, polling loops, or dependence on scheduler fairness
- retry-budget fixtures must prove at least first-attempt failure, deterministic cooldown or next-eligible-tick behavior, bounded retry exhaustion, and escalation after the configured budget is spent
- timeout fixtures must prove both sides of the boundary: no expiry before the declared logical deadline and explicit expiry once the injected clock crosses it
- explain and audit artifacts produced by these fixtures must expose enough timing evidence to reconstruct why a transition retried, timed out, or stopped

Representative artifact names include `lifecycle_logical_clock_fixture`, `lifecycle_retry_budget_fixture`, `lifecycle_timeout_boundary_fixture`, and `lifecycle_repair_cooldown_fixture`.

### Wall-clock-only benchmark exceptions

Wall clock is allowed only for benchmark and operations evidence that measures real elapsed performance, such as encode latency, consolidation throughput, repair throughput, or benchmark warm versus cold behavior. Those benchmark surfaces must declare that they are measuring elapsed time rather than authorizing lifecycle state changes. Benchmark evidence may use wall-clock timers, but correctness fixtures for lifecycle transitions, retries, cooldowns, and timeout guards must continue to use deterministic logical time.

## Relationship to memory and ops docs

- `MEMORY_MODEL.md` defines the durable memory lifecycle and the rule that failed transitions preserve prior state.
- This document defines the reusable guard vocabulary, allowed/forbidden edges, and failure taxonomy for lower-level controllers.
- `OPERATIONS.md` and `FAILURE_PLAYBOOK.md` define how repair queues, degraded mode, and incidents are handled once transition failures accumulate or threaten correctness.
