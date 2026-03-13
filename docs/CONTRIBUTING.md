# membrain — Contributing Guide

> Canonical source: PLAN.md Section 42 (Contributor Workflow and PR Acceptance).

## Principles

1. **Keep hot path measurable** — every retrieval mode must have a hard candidate budget
2. **Preserve provenance** — every item retains source kind, reference, timestamps, lineage
3. **Write repairable code** — indexes and graph must be rebuildable from durable evidence
4. **Prefer explicit invariants over hidden behavior** — no silent overwrites, no hidden state
5. **Benchmark before and after performance-sensitive changes** — no regression without evidence

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

## PR Rejection Rules

A major PR should be rejected or sent back if:

- It changes hot-path behavior **without benchmark evidence**
- It alters schema **without migration notes**
- It changes forgetting/deletion semantics **without governance analysis**
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
