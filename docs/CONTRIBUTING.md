# membrain — Contributing Guide

> Canonical sources: `PLAN.md` Sections 3, 12, and 42.

## Principles

1. **Keep hot path measurable** — every retrieval mode must have a hard candidate budget
2. **Preserve provenance** — every item retains source kind, reference, timestamps, lineage
3. **Write repairable code** — indexes and graph must be rebuildable from durable evidence
4. **Prefer explicit invariants over hidden behavior** — no silent overwrites, no hidden state
5. **Benchmark before and after performance-sensitive changes** — no regression without evidence

## Canonical Design Thesis

`PLAN.md` is authoritative. The repository-wide thesis is a **brain-inspired cognitive runtime** whose production contract is:

- bounded and measurable foreground work
- provenance-preserving storage and derivation
- explainable retrieval, ranking, routing, and filtering
- repairable derived state rebuilt from durable evidence
- explicit contradiction representation instead of silent overwrite
- policy and namespace enforcement before expensive work
- research mechanisms promoted to core behavior only when they remain bounded, explainable, and benchmarked

## Contract Scope and Ownership

This document freezes the contributor-facing contract that downstream docs, reviews, and backlog work can cite without reopening foundational ambiguity.

It is the home for:
- documentation-precedence rules for contract documents
- evidence and artifact requirements for major, performance-sensitive, and governance-sensitive changes
- PR rejection triggers tied to those contracts

It is not the repository's session playbook. Work selection, `br`/`bv` operating flow, Agent Mail coordination, linehash edit discipline, handoff payloads, and troubleshooting steps live in `../AGENTS.md` and the workflow backlog. Use this file to decide what a change must prove; use the workflow guidance to decide how contributors execute and hand off the work.

## Architecture Invariants (Non-Negotiable)

These rules cannot be violated by any PR:

- No unbounded graph walks on the request path
- No full scans on the request path
- No compaction, repair, or large migrations in foreground
- No silent overwrite of contradiction — conflicts must be represented explicitly
- No hard delete without explicit policy permission
- Tier routing decisions must be traceable/inspectable
- Retrieval results must be explainable (score components, sources, policy filters)
- Background jobs must not violate latency budgets for online recall
- Namespace isolation is checked before expensive retrieval work

## Restriction Contract

These restrictions translate `PLAN.md` into contributor-facing checks that must hold across CLI, daemon, MCP, IPC, tests, and documentation.

### Foreground and request-path restrictions

- No LLM or remote API calls in encode, recall, `on_recall`, reconsolidation-apply, or forgetting-eligibility paths
- No full-store `O(n)` scans on any request path; every retrieval mode must run within a declared candidate budget
- No cold-payload decompression or large payload fetch before the final candidate cut
- No graph traversal without hard depth and node caps, including inspect and explain paths
- No policy bypass in CLI, daemon, MCP, or IPC wrappers; wrappers preserve namespace and policy checks instead of skipping them for convenience

#### Frozen hot-path and forbidden-foreground checklist

Use this checklist for any bead or PR that touches encode, recall, `on_recall`, explain, inspect, prefetch, reconsolidation-apply, forgetting eligibility, or request-path wrappers.

| Forbidden behavior | Why it is forbidden | Regression signal that must move if violated |
|---|---|---|
| Full-store scan, archive scan, or any request-path expansion proportional to total corpus size | Breaks the bounded-work contract and makes latency grow with data size instead of with the declared query budget | candidate counts, planner/index stage timings, and p95/p99 request latency |
| Graph traversal without hard depth, node, and continuation caps | Lets explain or retrieval paths turn into unbounded walks that can starve the hot path and make results non-reproducible under load | graph hop counts, visited-node counters, traversal-budget exhaust signals, and degraded-route traces |
| Compaction, repair, rebuild, migration, lease sweeping, or other maintenance work on the foreground path | Turns user-visible recall or encode latency into maintenance latency and makes online behavior depend on background housekeeping | job-duration and affected-item metrics appearing on request traces, foreground latency delta, and degraded-mode or maintenance-on-request audit fields |
| Remote model, LLM, hosted embedder, or hidden network retry work on latency-sensitive paths | Adds network variance, cost, privacy, and availability coupling to the canonical fast path | route traces showing local-only execution, remote-call counters staying at zero on request paths, and explicit denial or degraded events if a caller tries to force remote work |
| Cold payload decompression, large payload fetch, or payload-heavy reconstruction before final candidate cut | Forces expensive work before bounded pruning, so low-value candidates can dominate request cost | pre-cut versus post-cut candidate counts, payload fetch counters, package-stage timings, and cache bypass or cold-fetch reasons |
| Namespace or policy checks after expensive retrieval, traversal, or packaging work | Risks cross-namespace leakage and wastes hot-path budget on candidates that should have been denied early | denial or redaction traces naming the early enforcement point, filtered-candidate counts, and parity across CLI, daemon, IPC, and MCP wrappers |
| Prefetch or warm-path activity that can delay live recall or encode | Allows speculative work to consume the same budget as foreground work and hide starvation regressions | queue depth, prefetch cancellation or drop counters, warm-source fields, and foreground latency delta during prefetch activity |

Every touched request-path change must name the candidate budget, policy enforcement point, and explain or trace fields that prove the path stayed bounded.

Required proof for this checklist:
- unit or policy tests that fail when a path widens beyond its declared budget, skips early policy enforcement, or allows forbidden foreground maintenance
- logging-heavy integration, explain, or inspect checks that expose the bounded-work and denial artifacts without re-reading code
- benchmark or targeted latency evidence whenever the touched surface is hot-path-sensitive under the evidence matrix and rejection rules below

### Storage and lifecycle restrictions

- Tier1 stores handles and hot metadata, not giant payloads
- Tier2 keeps metadata and filtering state separate from large content so prefilters stay bounded
- Tier3, indexes, and graph structures must remain rebuildable from durable records
- Archive and forgetting flows stay reversible by default unless explicit policy or compliance rules require irreversible deletion

### Benchmark and research restrictions

- Never publish or cite a benchmark without dataset cardinality, machine profile, build mode, and warm/cold declaration
- Benchmark evidence must also name the harness or command entrypoint, representativeness label, and artifact location so reviewers can rerun or audit the claim
- Do not present p95 or p99 claims from microbench-sized sample counts as production facts; label them exploratory until representative
- If a brain-inspired mechanism lacks measured benefit under benchmark and ablation, treat it as optional rather than canonical

## Required for Major PRs

Every major change must include:

| Artifact | When Required |
|----------|---------------|
| Design note | Always for architectural changes |
| Benchmark result | Any hot-path or performance-sensitive change |
| Migration note | Any schema change |
| Rollback note | Any behavior change |
| Governance analysis | Any forgetting/deletion semantic change |
| Observability hook | Any performance-sensitive complexity |

These are contract-level review inputs, not optional nice-to-haves. If a change spans multiple categories, it owes the union of the required artifacts.

- Performance-sensitive changes must include both benchmark evidence and the observability hook needed to detect regressions.
- Governance-sensitive changes must include governance analysis, and also a rollback note when the change alters externally visible behavior.
- Schema changes must include migration notes even when the schema surface is small.

### Change-type evidence matrix

| Change class | Required evidence | Review proof must name explicitly | Reject when... |
|---|---|---|---|
| Architecture or core behavior | design note; rollback note when externally visible behavior changes | affected invariants, contract surface, downstream docs that were updated, and why the new behavior still matches the thesis | behavior changed without a design note, rollback path, or resolved doc consistency |
| Hot-path or performance-sensitive work | benchmark result; observability hook | dataset cardinality, machine profile, build mode, warm/cold declaration, harness or command entrypoint, sample-count or representativeness label, artifact location, touched p50/p95/p99 path metrics, and bounded-work signals such as candidate counts or tier/cache hit rates | benchmark evidence is missing, metadata is incomplete, touched path latency is still unknown, or no observability hook is named |
| Schema or durable-storage change | migration note; rollback note when behavior changes | changed tables/columns/meanings, compatibility window, rebuild/backfill plan, rollback scope, and required doc propagation to `PLAN.md`, `MEMORY_MODEL.md`, `CLI.md`, or `MCP_API.md` when exposed | migration notes are missing, rollback scope is undefined, rebuildability is unproven, or dependent docs were left inconsistent |
| Forgetting, deletion, retention, or archiving semantics | governance analysis; dedicated policy coverage; rollback note when externally visible | retention class or legal-hold behavior, tombstone/loss handling, audit surface, and whether the change can ever remove last authoritative evidence | governance analysis is missing, policy coverage is absent, or deletion/retention behavior becomes silent or irreversible without explicit contract |
| Namespace, sharing, denial, or redaction behavior | dedicated isolation/denial tests; explain/audit evidence | enforcement point before expensive work, expected denial/redaction artifacts, and parity across CLI, daemon, IPC, and MCP surfaces that expose the flow | isolation/denial tests are missing, denial/redaction paths are not inspectable, or behavior differs silently across surfaces |
| Repair, rebuild, migration, or background operations | ops note; observability hook; resilience/rebuild coverage | job duration, queue depth, affected-item counts, foreground latency delta, degraded-mode behavior, rollback conditions, and durable-truth-first recovery proof | background work can regress foreground contracts without detection, repair cannot prove derived state rebuild from durable truth, or rollback/containment is unspecified |
| Interface contract changes (CLI, daemon, JSON-RPC, MCP) | design note or interface contract update; parity validation | changed envelopes/flags/errors, standalone-versus-daemon equivalence where relevant, `PLAN.md` plus the touched subsystem doc stay aligned, and cross-surface user-visible differences are explicit | interface docs are stale, parity is unverified, or one surface changes semantics or error behavior silently |

### Exact rejection trigger rules

- Missing a required artifact for any touched change class is a rejection condition, even if unrelated artifacts are present.
- Incomplete evidence counts as missing evidence. For example, a benchmark without dataset/machine/warm-cold metadata, rerunnable harness details, or representativeness labeling does not close the hot-path gate.
- Generic regression coverage does not substitute for dedicated scope-specific proof. Namespace or policy changes still need isolation/denial coverage; repair or migration changes still need rebuild/resilience coverage.
- Contradictory evidence across docs, tests, benchmarks, or rollout notes should be treated as unresolved rather than averaged together; the PR stays rejectable until the conflict is resolved explicitly.
- If a change spans multiple classes, reviewers should apply the union of the relevant rows above rather than choosing the easiest row that happens to fit.

### Executable PR review checklist

Use this checklist when reviewing any major PR or handoff bundle. The reviewer should be able to mark every touched change class as `present`, `not applicable`, or `reject` without re-reading the whole guide.

1. Identify every touched change class from the evidence matrix. If the PR spans multiple classes, apply the union of their proof obligations.
2. For each touched class, name the exact proof artifact in the PR description, handoff, or attached bundle rather than gesturing at "tests passed" or "benchmarks attached" in the abstract.
3. Reject the PR when any required artifact is missing, incomplete, contradictory, or points to a proof surface that does not actually cover the changed contract.
4. Reject the PR when one interface changes user-visible semantics, denial behavior, or machine-readable fields silently relative to another exposed surface.

| Review question | Reviewer must be able to point to | Reject when... |
|---|---|---|
| What changed at the contract level? | named change class, affected invariants or surface, and the updated canonical docs | the change class is unstated or the docs that freeze the contract were left ambiguous |
| Where is the benchmark proof for hot-path work? | benchmark report, harness or command entrypoint, dataset or fixture identity, machine profile, build mode, warm/cold semantics, sample count, representativeness label, artifact path, and touched p50/p95/p99 plus bounded-work signals | benchmark evidence is missing; metadata is incomplete; p95/p99 is absent for the touched path; or the artifact cannot be rerun or audited |
| Where is the schema proof? | migration note, rollback note when behavior changes, rebuild or backfill plan, and downstream doc propagation | migration notes are missing, rollback scope is undefined, rebuildability is unproven, or exposed docs drift silently |
| Where is the governance proof? | governance analysis, dedicated policy coverage, named audit or explain artifacts, and rollback note when the semantics are externally visible | governance analysis is missing, policy coverage is absent, audit or explain evidence is not inspectable, or retention/deletion changes become silent or irreversible |
| Where is the parity proof for namespace, denial, redaction, or interface work? | dedicated isolation or denial tests plus cross-surface parity artifacts for CLI, daemon, IPC, and MCP where the flow exists | parity is unverified, denial or redaction behavior differs silently, or one surface changed envelopes, flags, or errors without explicit contract notes |
| Where is the observability proof? | named metric, trace, audit, explain, inspect, benchmark, or dashboard field that should move on regression | the change adds performance-sensitive or operator-sensitive behavior without a checkable signal |
| Where is the logging-heavy end-to-end proof? | at least one command or script example plus captured machine-readable output showing the touched outcome class and regression signals | the PR claims boundedness, denial, redaction, migration safety, or parity behavior but offers only prose summaries or code references |

### Proof naming contract for PRs and handoffs

A valid PR description or handoff should name the proof artifacts explicitly enough that another contributor can fetch them without guesswork.

- Use concrete labels such as `design note`, `benchmark report`, `migration note`, `rollback note`, `governance analysis`, `parity fixture`, `observability hook`, `failure matrix`, `ops note`, or `logging-heavy end-to-end artifact`.
- Pair each named artifact with the command, fixture identity, dashboard, report path, or doc anchor where the proof lives.
- If a touched class is intentionally not in scope, say `not applicable` and name why the class does not apply.
- `tests passed`, `see CI`, or `benchmark attached` is not specific enough to satisfy review.

### Determinism review rules

Time-sensitive review proof must stay as reproducible as the functional contract it is testing.

- correctness claims about decay, recency, reconsolidation windows, retry budgets, timeouts, or other tick-sensitive behavior should point to interaction-tick, logical-tick, or injected-clock fixtures rather than ambient wall time
- wall-clock or sleep-based evidence is acceptable for benchmark or soak measurement only when the claim is about real elapsed performance; it does not replace deterministic semantic coverage
- a PR is rejectable if a touched time-sensitive rule cannot name the deterministic fixture, starting state, and expected artifact or if the only proof depends on nondeterministic sleeps or scheduler timing

### Observability Hook Contract

An observability hook is the operator-visible evidence that lets reviewers and maintainers detect whether a hot-path or semantics-changing change still honors the contract after it lands.

- The hook must name the concrete metric, trace, audit surface, or explain/inspect output that should move if the change regresses.
- Request-path changes should expose latency percentiles plus bounded-work signals such as candidate counts, tier-routing decisions, or cache/tier hit rates.
- Encode-path duplicate-routing or observability changes should also expose shortlist evidence, duplicate-family route outcome, nearest-neighbor or novelty summary, and whether interference work ran, was skipped, or was deferred.
- Policy or governance changes should expose denial, redaction, or audit signals and preserve enough explainability to show which policy path fired.
- Background-job or maintenance changes should expose job duration, queue depth, affected-item counts, and any foreground latency delta they impose.
- The accompanying change notes should point reviewers to the command, dashboard, benchmark artifact, or machine-readable field where the signal can be checked.
- If the only way to detect regression is to re-read code or attach a debugger, the change is not observable enough to satisfy this contract.

### Regression-signal matrix

Every performance-sensitive, semantics-changing, or operator-sensitive change must name the regression signals that should move if the contract is violated.

| Change class | Minimum operator-visible regression signals |
|---|---|
| Request-path latency or routing | touched path p50/p95/p99, route outcome, tier escalation trace, candidate counts before and after each pruning stage, and any degraded or capped-route marker |
| Encode-path routing or duplicate handling | encode latency, shortlist size, duplicate-route outcome, nearest-neighbor or novelty summary, cache hit or miss state, and interference applied/skipped/deferred markers |
| Policy, denial, redaction, or governance behavior | denial or redaction outcome class, enforcement stage, filtered-candidate counts, audit artifact handle, and parity across CLI, daemon, IPC, and MCP surfaces where the flow exists |
| Cache, prefetch, warmup, or invalidation behavior | cache family, cache event, cache reason, warm source, stale or bypass reason, candidate counts before and after cache-influenced stages, and distinguishable cold-versus-disabled-versus-stale outcomes |
| Background repair, rebuild, consolidation, or migration work | job duration, queue depth, affected-item counts, retry budget or escalation state when relevant, foreground latency delta, and degraded-mode or containment markers |
| Explain, inspect, or audit envelope changes | stable machine-readable fields for route traces, denial or omission summaries, conflict or stale markers, and any new or removed field families that operators are expected to inspect |

Review notes, benchmark bundles, and test artifacts should point to the exact command, fixture, report field, or dashboard surface where each listed signal is checked.

## PR Rejection Rules

A major PR should be rejected or sent back if:

- It changes hot-path behavior **without benchmark evidence**
- It alters schema **without migration notes**
- It changes forgetting/deletion semantics **without governance analysis**
- It changes namespace, sharing, denial, or redaction behavior **without dedicated isolation/denial tests**
- It adds performance-sensitive complexity **without observability**
- It weakens **repairability** or **lineage preservation**
- It introduces silent behavior that differs between CLI and MCP paths

## Performance Budget

| Path | Budget |
|------|--------|
| Tier1 lookup | <0.1ms |
| Tier2 retrieval | <5ms |
| Tier3 retrieval | <50ms |
| Encode | <10ms |

Any change that causes a measured regression must either:
- Fix the regression, or
- Provide justification and a plan to recover

## Code Quality Gates

1. All public APIs must have doc comments
2. Unsafe code requires explicit justification comment
3. No `unwrap()` on user-facing paths
4. Error types must distinguish: validation failure, policy denial, internal error
5. Background jobs must be cancellable and bounded
6. Every new table/column must include migration note in PR description

## Testing Requirements

- Unit tests for all scoring formulas (decay, strength, ranking)
- Integration tests for encode→recall round-trip
- Benchmark tests for hot-path operations
- No correctness, lifecycle, or policy test may depend on wall-clock time when interaction ticks or injected clocks can express the same behavior; benchmark harnesses may measure real elapsed time for latency claims, but not as the sole proof of semantic behavior

## Schema Change Protocol

1. Add column/table to initial migration SQL in PLAN.md Section 46.12 / 47.12
2. Update `MEMORY_MODEL.md` with new fields
3. Update `MCP_API.md` if the field is exposed
4. Update `CLI.md` if new commands are added
5. Include rollback SQL in PR description

## Documentation Hierarchy

```
PLAN.md          ← canonical design contract (source of truth)
MEMORY_MODEL.md  ← elaborates memory types, fields, lifecycle
CLI.md           ← elaborates CLI commands
MCP_API.md       ← elaborates MCP tools
OPERATIONS.md    ← elaborates production runbooks
NEURO_MAPPING.md ← elaborates brain → computation mappings
CONTRIBUTING.md  ← this file
INDEX.md         ← doc pointer
```

Subsystem docs should elaborate, **not contradict** PLAN.md.
If they diverge, resolve the conflict explicitly.

Contract precedence and scope ownership work as follows:

1. `PLAN.md` is the canonical design contract and tie-breaker.
2. Subsystem docs under `docs/` elaborate one surface at a time and must stay consistent with the plan.
3. `CONTRIBUTING.md` freezes contributor-facing evidence requirements, review gates, and PR rejection triggers derived from the canonical plan.
4. `INDEX.md` is a navigation aid; it helps readers find the right contract surface but does not redefine it.
5. `../AGENTS.md` translates these contracts into day-to-day execution, coordination, and handoff guidance; it clarifies workflow but does not change product behavior or evidence thresholds.
