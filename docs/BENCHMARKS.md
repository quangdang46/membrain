# membrain — Benchmark Contracts

> Canonical sources: `PLAN.md` milestone acceptance criteria, performance-budget overlays, and phase-gate rules.
> If this document diverges from `PLAN.md`, the plan wins.

## Benchmark reporting contract

Every benchmark claim must record:

- scenario and stage name
- dataset cardinality and payload shape
- machine profile and build mode
- warm/cold declaration
- concurrency level
- p50, p95, and p99 when applicable
- pass/fail result against the declared contract

If the workload, corpus size, or sample count is not representative, label the result as exploratory instead of using it to close a stage gate.

## Global latency budgets

| Path | Contract |
|------|----------|
| Encode fast path | p95 < 10ms |
| Tier1 exact retrieval | p95 < 0.1ms |
| Tier2 indexed retrieval | p95 < 5ms at declared hot cardinality |
| Tier3 cold fallback | p95 < 50ms at declared cold cardinality |

Any request-path benchmark that reports only averages is incomplete. Tail behavior must stay visible enough that p99 does not invalidate the advertised fast-path story.

## Stage-by-stage benchmark and gate expectations

### Stage 1 — Foundation + Lazy Decay

**Required benchmark surfaces**
- encode → recall roundtrip
- WAL-enabled hot-store operation
- `effective_strength` stability/decay behavior
- embedding-cache hit versus miss benefit
- metadata-only hot prefilter behavior

**Gate expectations**
- benchmark evidence exists for the first measurable Tier1/encode baseline
- cache benefit is observable rather than assumed
- the prefilter path does not touch large payload tables
- no hidden full scan is introduced on the request path

### Stage 2 — Full Encode Pipeline

**Required benchmark surfaces**
- attention gating and discard thresholds
- novelty scoring and duplicate routing
- emotional bypass behavior without latency escape hatches
- working-memory eviction cost
- bounded proactive/retroactive interference updates

**Gate expectations**
- full encode remains within the declared encode envelope on representative payload sizes
- duplicate routing stays bounded by a small candidate search rather than unbounded scans
- stage evidence makes clear that encode-path restrictions still hold

### Stage 3 — `on_recall` / LTP-LTD

**Required benchmark surfaces**
- recall overhead before versus after `on_recall`
- monotonic stability-growth behavior
- labile-state persistence through restart
- Tier1 cache update cost on recall

**Gate expectations**
- `on_recall` remains request-bounded and measurable
- recall-side strengthening does not hide unbounded background work
- durable labile-state transitions survive restart without index/cache drift

### Stage 4 — Three-tier retrieval

**Required benchmark surfaces**
- Tier1 exact-hit latency
- Tier1 recent-window search latency at a declared active-window size
- Tier2 indexed retrieval latency at declared hot cardinality
- Tier3 fallback latency at declared cold cardinality
- context reranking benefit
- graph/engram expansion overhead inside explicit caps
- partial or tip-of-the-tongue path behavior across `full`, `partial`, and `miss` outcomes
- deferred payload-fetch counts for fragmentary recall lanes

**Gate expectations**
- Tier1, Tier2, and Tier3 latency contracts are demonstrated with representative corpora
- tier escalation is deterministic and inspectable
- graph expansion remains inside declared depth/node budgets
- partial-match paths do not fetch or leak full cold payloads before the final candidate cut
- fragmentary routes stay within the same declared candidate, node, sibling, and payload budgets as the full-recall contract
- benchmark output records whether the route ended as `full`, `partial`, or `miss` and how many payload fetches were deferred until after the final cut

### Stage 5 — Reconsolidation

**Required benchmark surfaces**
- labile-window enforcement
- update apply + re-embed + reindex coherence
- cache invalidation cost
- crash-safe update application behavior

**Gate expectations**
- reconsolidation evidence distinguishes foreground recall/update work from asynchronous apply work
- accepted updates leave the DB, ANN state, and cache in sync
- stale-window rejection remains explicit and cheap

### Stage 6 — Consolidation

**Required benchmark surfaces**
- migration throughput versus foreground latency delta
- retrievability after migration
- REM-like cross-linking auditability
- homeostasis behavior under pressure
- dry-run versus apply cost

**Gate expectations**
- background consolidation does not break online retrieval SLOs
- migration preserves retrievability after move
- pinned or authoritative evidence is never pruned as a side effect of consolidation
- benchmark output captures the foreground penalty, not just job duration

### Stage 7 — Graph maturity

**Required benchmark surfaces**
- centroid stability
- split and sibling-creation behavior
- BFS depth/node-cap enforcement
- serialization/deserialization integrity
- graph-assisted recall overhead

**Gate expectations**
- graph-assisted recall stays bounded under declared caps
- split logic remains stable enough for reproducible clustering behavior
- restart does not corrupt graph state or silently change traversal behavior

## Benchmark artifact templates

### Retrieval benchmark template

| Scenario | Corpus size | Warm/Cold | Concurrency | p50 | p95 | p99 | Pass? |
|---|---:|---|---:|---:|---:|---:|---|

### Encode benchmark template

| Scenario | Cache hit rate | Avg payload size | p50 | p95 | p99 | Pass? |
|---|---:|---:|---:|---:|---:|---|

### Consolidation benchmark template

| Job | Items moved | Foreground load | p95 foreground delta | Duration | Pass? |
|---|---:|---:|---:|---:|---|

## Global no-go conditions

Do not use benchmark evidence to close a stage if any of the following remain true:

- dataset cardinality, machine profile, or warm/cold status is missing
- p95 or p99 is unknown for a touched request path
- success depends on hidden policy bypass, full scans, or uncapped graph traversal
- background work meets throughput goals only by violating foreground latency contracts
