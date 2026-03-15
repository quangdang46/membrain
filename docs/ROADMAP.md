# ROADMAP

> `PLAN.md` Sections 40 and 44 are canonical for milestone gates and execution order.
> This file is the condensed phase view. If it diverges from the plan, the plan wins.

## Phase 0 — Contracts and measurable foundation prerequisites
- freeze canonical types, schema semantics, and invariants
- freeze policy and namespace enforcement before expensive work
- freeze benchmark harness and reproducibility rules for later hot-path claims
- prove a measurable Tier1 MVP without expanding scope prematurely

## Phase 1 — Core encode, storage, and bounded retrieval baseline
- implement Tier1 fast encode and exact/recent retrieval
- implement Tier2 durable indexed storage and search
- ship session/entity queries and a measurable ranking baseline
- expose inspect/explain surfaces at least in debug or operator form

## Phase 2 — Contradiction handling, graph-assisted retrieval, and explainable packaging
- represent contradictions and conflict-aware storage explicitly
- add graph-assisted retrieval only under hard depth, node, and candidate budgets
- finish explainable recall packaging instead of opaque score-only output

## Phase 3 — Dynamic memory lifecycle, repair, and regression hardening
- implement consolidation pipelines
- implement forgetting and demotion pipelines
- make repair, rebuild, and compaction safe under failure injection
- expand benchmark regression artifacts around these flows

## Phase 4 — Operational tooling, justified scale-out, and later-stage extensions
- add doctor and operator tooling after core behavior is benchmarked and repairable
- introduce sharding or distribution only when empirical workload pressure justifies it
- keep advanced feature batches and later-stage extensions non-blocking with respect to the core execution spine

## Sequencing rules
1. Contracts, memory semantics, and namespace policy land before expensive retrieval work.
2. Retrieval becomes explainable before graph and advanced optimization layers become core behavior.
3. Repair and rebuild paths exist before large-scale operations or distribution.
4. Advanced features and sharding remain later-stage work; they never block the core architecture spine.

## Non-negotiable restriction overlays

These phase summaries inherit the same guardrails frozen in the canonical plan and condensed architecture docs:

- no full-store scans on request paths
- no uncapped graph expansion on request paths
- no policy or namespace bypass before expensive retrieval work
- no premature cold-payload fetch before the final candidate cut
- no benchmark claims without representative context (dataset cardinality, machine profile, build mode, warm/cold declaration)

Read the phases with those restrictions in mind: if a later implementation idea violates them, it belongs in research or redesign, not in the core execution spine.
