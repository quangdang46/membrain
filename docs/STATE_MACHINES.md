# membrain — Lifecycle State Machines

> Canonical sources: `MEMORY_MODEL.md` lifecycle contract, `OPERATIONS.md` repair flow, and `SECURITY_GOVERNANCE.md` namespace and ACL rules.

This document defines the lower-level transition vocabulary, guard matrix, failure semantics, and repair handoff rules that support the canonical persisted lifecycle for memory objects. If this document diverges from `MEMORY_MODEL.md`, the persisted lifecycle in `MEMORY_MODEL.md` wins.

## Persisted lifecycle versus controller transitions

The canonical persisted lifecycle for memory items is:

`Labile -> SynapticDone -> Consolidating -> Consolidated -> Archived`

with `Superseded` as an explicit contradiction-resolution outcome.

Lower-level controller outcomes such as `created`, `indexed`, `recalled`, `reinforced`, `decayed`, `demoted`, and `deleted` are operational transition classes or maintenance outcomes, not a second conflicting persisted lifecycle enum. They describe **how** an object moves or is served, while the persisted lifecycle describes **what durable state the memory is currently in**.

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

## Forbidden edge classes

The following are globally forbidden unless a stricter subsystem contract explicitly allows them and preserves auditability:

- skipping directly from active mutable states to hard deletion
- turning `Superseded` into a routine decay, archival, or compaction synonym
- reopening `Archived` or `Superseded` objects through ordinary recall without an explicit restore, replay, or contradiction-resolution path
- mutating lifecycle while a repair, migration, or replay controller holds the repair-job lock
- using cross-namespace movement as a lifecycle transition instead of an explicit share, fork, merge, or re-ingest flow
- treating controller metadata changes (retention, decay, shard routing) as authorization to bypass namespace or policy checks

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

## Relationship to memory and ops docs

- `MEMORY_MODEL.md` defines the durable memory lifecycle and the rule that failed transitions preserve prior state.
- This document defines the reusable guard vocabulary, allowed/forbidden edges, and failure taxonomy for lower-level controllers.
- `OPERATIONS.md` and `FAILURE_PLAYBOOK.md` define how repair queues, degraded mode, and incidents are handled once transition failures accumulate or threaten correctness.
