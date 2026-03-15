# membrain — Test Strategy

> Canonical sources: `PLAN.md` implementation milestones, benchmark overlays, and phase-gate rules.
> If this document diverges from `PLAN.md`, the plan wins.

The testing strategy must prove correctness, boundedness, durability, explainability, and stage readiness. Generic suite names are not enough by themselves; each stage must have explicit gate expectations and exit artifacts.

## Cross-cutting test rules

- Every suite emits structured artifacts useful for regression analysis.
- Failure-injection and recovery suites should also record the violated invariant, affected namespace or shard, authoritative durable input, whether prior durable state remained intact, and any repair or escalation artifact.
- No test depends on wall-clock time when logical ticks or controlled clocks can express the same behavior.
- Any request-path change must be tested for both correctness and boundedness.
- Any stateful mutation stage must include restart or crash-safety coverage where relevant.
- Policy and namespace checks must be validated before parity claims across CLI, daemon, IPC, or MCP layers.
- Benchmark results without representative metadata are exploratory evidence, not gate-closing evidence.

## Namespace and isolation minimum matrix

Any change to namespace, sharing, ACL, denial, or redaction semantics must include dedicated coverage for:

- explicit same-namespace allow paths
- deterministic default-namespace binding when one default exists
- missing-namespace validation failure when no deterministic default exists
- malformed or unknown namespace rejection before candidate generation or writes
- cross-namespace denial without leakage of protected counts, handles, or existence hints
- approved shared/public access paths via explicit visibility controls
- background-job, cache, and repair-path preservation of namespace scope
- parity across CLI, daemon, IPC, and MCP surfaces

## Lifecycle transition failure minimum matrix

Any change to lifecycle guards, state-machine edges, replay logic, or repair handoff must include dedicated coverage for:

- allowed-edge success for each touched object family
- forbidden-edge rejection as `validation_failure`
- namespace or policy guard rejection as `policy_denied`
- internal failure preserving prior durable state without silent half-transition
- repairable failure emitting a repair handle or queued follow-up artifact when applicable
- retry-budget exhaustion producing operator-visible escalation instead of infinite replay
- parity across request-path and background-controller behavior for the same edge family

## Formula and invariant minimum matrix

Any change to scoring, decay, ranking fusion, lifecycle math, or invariant-preserving transforms must include dedicated coverage for:

- deterministic unit vectors for canonical formulas such as `effective_strength`, `initial_strength`, `reconsolidation_window`, decay updates, and the shared ranking-family score decomposition where those formulas are implemented,
- property tests for monotonicity, bounded output ranges, duplicate-penalty or noise-penalty stability, and other declared invariants from the scoring or decay contracts,
- cross-implementation parity tests whenever the same formula or predicate exists in multiple execution lanes (for example Rust versus SQL prefilters or request-path versus maintenance-path evaluation),
- invariant-preserving transformation coverage for summarize, merge, extract, repair, consolidation, compaction, migration, or similar rewrites so lineage, namespace, policy-bearing metadata, contradiction state, and identity or supersession rules do not drift,
- failure-path coverage showing that invalid transforms reject cleanly, internal failure preserves prior durable state, and repair or escalation artifacts appear when the contract says they should, and
- controlled-clock or tick-driven fixtures for any age, recency, or reconsolidation math instead of wall-clock sleeps.

## Core suite families

### Unit tests
Use for formulas, scoring pieces, state transitions, and invariants that should fail fast and deterministically.

### Property tests
Use for monotonicity, bounds, idempotency, and invariant preservation across broad input ranges.

### Integration tests
Use for encode/recall/update/consolidate flows that cross stores, indexes, caches, and user-facing entry points.

### Latency and load tests
Use for request-path budgets, tail latency, concurrency safety, duplicate-storm containment, graph fanout containment, and background-foreground interference measurement.

### Chaos, rebuild, and migration tests
Use for failure injection, repairability, crash recovery, stale-result visibility under mutation or repair, failed-transition recovery, schema motion, restart and rebuild-from-durable-truth claims, and migration safety.

### Policy, namespace, and quality tests
Use for policy denial behavior, cross-namespace isolation, retrieval quality, retention and legal-hold regression behavior, explainability, and user-visible safety constraints.

## Cache observability regression minimum matrix

Any change to cache reuse, prefetch, warmup, invalidation, degraded serving, or explain integration must include dedicated coverage for:

- cache hit, miss, bypass, invalidation, and disabled-mode visibility by cache family
- explicit stale-warning or bypass-reason output when warm state is rejected for owner-boundary, namespace, policy, or generation-anchor mismatch
- routing-trace artifacts that preserve candidate counts before and after cache-influenced stages
- parity of cache metadata across CLI, daemon, IPC, and MCP explain or inspect surfaces where those surfaces exist
- degraded-mode or cache-disabled serving remaining distinguishable from an ordinary cold miss
- namespace or policy-filtered cache paths preserving denial or redaction semantics without leaking protected handles or counts

## Resilience and governance suite minimum matrix

Any change to repair, rebuild, migration, retention, namespace policy, or other operationally sensitive behavior must include dedicated coverage for:

- latency and load behavior under representative concurrency, including duplicate-storm and graph-fanout containment
- restart, crash, rebuild, and migration flows proving durable truth can restore derived state without silent widening of authority
- cache invalidation correctness and stale-result visibility during mutation, repair, and degraded serving
- failed-transition recovery preserving prior durable state and emitting repair or escalation artifacts when applicable
- cross-namespace leakage prevention across request paths, caches, repair controllers, and background jobs
- retention-policy and legal-hold regressions staying explicit, auditable, and policy-correct under repair and migration

## Stage-by-stage gate expectations

### Stage 1 — Foundation + Lazy Decay

**Required suites**
- unit tests for `effective_strength` and basic scoring
- integration tests for encode → recall roundtrip
- latency tests for initial Tier1/encode baseline

**Must prove before closing the stage**
- lazy decay is numerically stable
- WAL mode and metadata-only prefilter behavior are verified
- cache benefit is measured rather than assumed
- the first measurable request-path baseline is recorded

### Stage 2 — Full Encode Pipeline

**Required suites**
- integration tests for attention gating, novelty routing, duplicate handling, and working-memory eviction
- property or adversarial tests for bounded interference behavior
- latency tests for full encode cost under representative payloads

**Must prove before closing the stage**
- attention and novelty decisions are deterministic enough to debug and benchmark
- duplicate routing does not regress into unbounded scans
- working-memory eviction and interference updates stay bounded

### Stage 3 — `on_recall` / LTP-LTD

**Required suites**
- unit/property tests for stability growth and decay-reset behavior
- integration tests for labile transition, access-count updates, and cache refresh
- restart tests for durable labile-state persistence
- latency tests for recall overhead

**Must prove before closing the stage**
- `on_recall` remains request-bounded
- recall-induced updates survive restart without state drift
- strengthening behavior is monotonic and inspectable

### Stage 4 — Three-tier retrieval

**Required suites**
- integration tests for tier escalation, context reranking, tip-of-the-tongue behavior, and explicit `full` versus `partial` versus `miss` outcomes
- latency/load tests for Tier1, Tier2, and Tier3 budgets
- explainability tests for routing/ranking traces
- bounded graph-expansion tests when graph assistance is enabled
- adversarial tests for ambiguous partial cues, near-duplicate clusters, and contradiction-aware partial recall
- parity tests for summary versus full explain surfaces across CLI, daemon, IPC, and MCP where those surfaces exist

**Must prove before closing the stage**
- Tier1, Tier2, and Tier3 all meet their declared latency contracts on representative corpora
- graph/engram expansion respects hard node and depth caps
- partial-match paths do not leak full payloads before the final cut
- tip-of-the-tongue results stay explicit about fragmentary status, remaining ambiguity, and omitted evidence
- low-signal partial cues terminate with a bounded miss or fragment shortlist rather than speculative reconstruction
- explain outputs preserve returned-result reasons, omitted-result reasons, provenance summaries, freshness or conflict markers, and stable routing-trace fields without cross-surface semantic drift

### Stage 5 — Reconsolidation

**Required suites**
- integration tests for labile-window enforcement and accepted/rejected updates
- crash-recovery tests for pending update application
- coherence tests for DB, ANN, and cache state after update
- policy tests for invalid or forced update paths

**Must prove before closing the stage**
- reconsolidation does not leave cache/index divergence
- stale-window rejection is explicit and safe
- accepted updates are durable, inspectable, and bounded

### Stage 6 — Consolidation

**Required suites**
- integration tests for migration, retrievability after move, REM-like processing, and dry-run behavior
- load tests for foreground impact while consolidation runs
- chaos/rebuild tests for interrupted or partial consolidation cycles
- policy tests for pinned, retention-governed, legal-hold, and authoritative evidence handling

**Must prove before closing the stage**
- background consolidation preserves online SLOs
- migrated content remains retrievable and explainable
- consolidation never silently drops protected or authoritative evidence
- interrupted or restarted consolidation preserves prior durable truth, leaves repairable artifacts when needed, and does not leak stale or cross-namespace warmed state

### Stage 7 — Graph maturity

**Required suites**
- integration tests for formation, split, sibling creation, and recall expansion
- property tests for traversal caps and centroid stability
- restart tests for serialization integrity
- latency tests for graph-assisted retrieval overhead

**Must prove before closing the stage**
- graph-assisted retrieval stays bounded under declared caps
- graph persistence survives restart without corruption
- split logic remains reproducible enough for operational debugging

## Required exit artifacts for a completed stage

Every stage completion should leave behind:

- benchmark report
- failure matrix
- design note
- migration note if schema changed
- rollback note if behavior changed
- ops note if background jobs changed

## Global no-go conditions

Do not declare a stage ready if any of the following are still true:

- touched request paths have unknown p95/p99 behavior
- contradiction, policy, or namespace semantics changed without dedicated tests
- new derived state cannot be rebuilt or repaired from durable truth
- background execution succeeds only by degrading foreground contracts
- parity across standalone and service-facing surfaces is unverified where the stage depends on it

