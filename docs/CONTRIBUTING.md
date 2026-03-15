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

### Storage and lifecycle restrictions

- Tier1 stores handles and hot metadata, not giant payloads
- Tier2 keeps metadata and filtering state separate from large content so prefilters stay bounded
- Tier3, indexes, and graph structures must remain rebuildable from durable records
- Archive and forgetting flows stay reversible by default unless explicit policy or compliance rules require irreversible deletion

### Benchmark and research restrictions

- Never publish or cite a benchmark without dataset cardinality, machine profile, build mode, and warm/cold declaration
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

### Observability Hook Contract

An observability hook is the operator-visible evidence that lets reviewers and maintainers detect whether a hot-path or semantics-changing change still honors the contract after it lands.

- The hook must name the concrete metric, trace, audit surface, or explain/inspect output that should move if the change regresses.
- Request-path changes should expose latency percentiles plus bounded-work signals such as candidate counts, tier-routing decisions, or cache/tier hit rates.
- Policy or governance changes should expose denial, redaction, or audit signals and preserve enough explainability to show which policy path fired.
- Background-job or maintenance changes should expose job duration, queue depth, affected-item counts, and any foreground latency delta they impose.
- The accompanying change notes should point reviewers to the command, dashboard, benchmark artifact, or machine-readable field where the signal can be checked.
- If the only way to detect regression is to re-read code or attach a debugger, the change is not observable enough to satisfy this contract.

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
- No test may depend on wall-clock time (use interaction ticks)

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
