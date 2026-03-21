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

## Benchmark harness reproducibility contract

Gate-closing benchmark evidence must come from a rerunnable harness or command entrypoint rather than a one-off anecdotal run.

Every benchmark-closing run must also record:

- the exact harness or command entrypoint used to produce the run
- the declared fixture or dataset identity, including the generation recipe when the workload is synthetic
- the release or comparable production build mode on the declared hardware profile
- the warm/cold semantics for process, model, index, cache, or other state that materially affects the path
- the sample count or iteration count used to derive the reported percentiles
- whether the run is representative or exploratory
- the artifact location for the machine-readable report and any bounded-work counters needed to audit the result

### Canonical benchmark metadata checklist

Treat this checklist as the minimum metadata contract for any benchmark another contributor might cite, rerun, or reject:

- **workload identity:** scenario name, touched path or stage, dataset or fixture ID, dataset cardinality, payload-shape summary, and synthetic-data generation recipe when applicable
- **execution environment:** machine profile, OS/kernel where relevant, CPU and memory class, storage posture when it materially affects the result, and the exact build mode
- **run semantics:** warm/cold declaration for process, cache, model, index, and other stateful surfaces; concurrency level; iteration/sample count; and whether percentile output is derived from steady-state, cold-start, or mixed runs
- **result bundle:** p50/p95/p99 for touched request paths, pass/fail against the stated contract, bounded-work counters or routing traces relevant to the claim, and the artifact location for machine-readable output
- **representativeness label:** explicit `representative` versus `exploratory` labeling, plus the reason when the run does not reflect the intended production envelope
- **rerun handle:** the exact command or harness entrypoint and any fixture selector, config file, or seed needed for another contributor to reproduce the same run

### Reproducibility rejection rules

Treat benchmark evidence as incomplete and non-gate-closing when any of the following is missing or ambiguous:

- the harness or command entrypoint cannot be rerun from the recorded metadata
- dataset identity, cardinality, or payload shape is omitted for a workload-sensitive claim
- machine profile, build mode, or warm/cold semantics are missing for a latency claim
- percentiles are quoted without the sample count or representativeness label that explains what they mean
- artifact paths or machine-readable outputs are absent, so reviewers cannot audit the reported numbers or bounded-work signals

## Global latency budgets

| Path | Contract |
|------|----------|
| Encode fast path | p95 < 10ms |
| Tier1 exact retrieval | p95 < 0.1ms |
| Tier2 indexed retrieval | p95 < 5ms at declared hot cardinality |
| Tier3 cold fallback | p95 < 50ms at declared cold cardinality |

Any request-path benchmark that reports only averages is incomplete. Tail behavior must stay visible enough that p99 does not invalidate the advertised fast-path story.

### Benchmark-versus-semantic-proof boundary

Benchmarks may measure elapsed performance, but they do not replace deterministic semantic coverage.

- Use benchmark or soak runs to measure latency, throughput, contention, queue growth, and other real-time performance characteristics.
- Do not cite benchmark timing alone as proof of correctness for timeout semantics, retry exhaustion, lifecycle transitions, policy denial timing, recency windows, or other rules that deterministic clocks or logical ticks can express.
- When a benchmark touches a time-sensitive semantic rule, pair it with a deterministic fixture that records the same starting state, clock or tick source, and expected semantic outcome.
- Treat sleep-based benchmark harnesses as performance evidence only; they do not close semantic test gates by themselves.

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
- novelty scoring, duplicate routing, and write-path observability
- emotional bypass behavior without latency escape hatches
- working-memory eviction cost
- bounded proactive/retroactive interference updates

**Gate expectations**
- full encode remains within the declared encode envelope on representative payload sizes
- duplicate routing stays bounded by a small candidate search rather than unbounded scans
- benchmark artifacts expose the duplicate-route observability needed to attribute latency, including shortlist evidence such as candidates inspected or nearest-neighbor similarity plus whether interference work was applied, skipped, or deferred
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

## Stage-close evidence map

### Stage 1 — Foundation + Lazy Decay
A gate-closing bundle should at minimum include:
- a benchmark report covering the first measurable Tier1 or encode baseline, cache hit-versus-miss behavior, and metadata-only prefilter behavior
- a failure matrix covering WAL verification, decay instability, cache regressions, and hidden full-scan regressions
- a design note naming the object-model and invariant assumptions frozen tightly enough to support benchmarkable work
- migration or rollback notes when Stage 1 work also changes schema or externally visible behavior

### Stage 2 — Full Encode Pipeline
A gate-closing bundle should at minimum include:
- a benchmark report covering attention gating, novelty scoring, duplicate routing, working-memory eviction, and bounded interference costs
- a failure matrix covering duplicate storms, hidden-scan regressions, observability gaps, and boundedness failures in interference updates
- a design note describing the deterministic encode decisions and write-path observability surfaces the benchmark evidence relies on
- rollback or migration notes when the encode path changes externally visible routing or storage semantics

### Stage 3 — `on_recall` / LTP-LTD
A gate-closing bundle should at minimum include:
- a benchmark report covering recall overhead, stability-growth behavior, labile-state persistence, and Tier1 refresh cost
- a failure matrix covering restart drift, cache or index divergence after recall-side updates, and request-boundedness regressions
- a design note describing recall-side mutation rules and their observable checkpoints
- rollback notes when recall-side strengthening or labile transition behavior changes user-visible semantics

### Stage 4 — Three-tier retrieval
A gate-closing bundle should at minimum include:
- a benchmark report covering Tier1, Tier2, and Tier3 latency, partial versus full versus miss outcomes, graph overhead, and deferred payload-fetch counts
- a failure matrix covering escalation mistakes, graph-cap violations, contradiction-aware partial-recall regressions, and premature payload-fetch leakage
- a design note describing the routing, ranking, and explainability fields reviewers should inspect when validating the stage
- migration or rollback notes when retrieval packaging, ranking fields, or exposed result envelopes change

### Stage 5 — Reconsolidation
A gate-closing bundle should at minimum include:
- a benchmark report covering labile-window enforcement, update apply and reindex coherence, cache invalidation cost, and crash-safe update behavior
- a failure matrix covering stale-window rejection, interrupted apply flows, DB or ANN or cache divergence, and policy-sensitive update rejection
- a design note describing reconsolidation checkpoints and durable-state assumptions
- rollback notes when reconsolidation changes externally visible update semantics

### Stage 6 — Consolidation
A gate-closing bundle should at minimum include:
- a benchmark report covering migration throughput, foreground latency delta, retrievability after move, REM-like auditability, and dry-run versus apply cost
- a failure matrix covering interrupted cycles, protected-evidence handling, stale warmed-state exposure, and degraded-mode behavior under foreground load
- a design note describing consolidation-controller scope and durable-truth-first assumptions
- an ops note for the background-job behavior and observability surfaced by the stage, plus rollback notes when externally visible retention or movement behavior changes

### Stage 7 — Graph maturity
A gate-closing bundle should at minimum include:
- a benchmark report covering centroid stability, split or sibling behavior, traversal-cap enforcement, restart integrity, and graph-assisted recall overhead
- a failure matrix covering serialization corruption, traversal budget escape, clustering instability, and repair or rebuild drift
- a design note describing graph-formation and graph-recall assumptions that operators and reviewers should treat as canonical for the stage
- an ops note when graph rebuild, repair, or maintenance behavior changes how the mature graph is validated in production

## Minimum benchmark artifact bundle

Each benchmark artifact bundle should include:

- a summary report naming the scenario, stage, contract, and pass/fail outcome
- machine-readable latency output sufficient to inspect p50, p95, p99, concurrency, and warm/cold status for the measured run
- workload metadata linking the run to the declared dataset or fixture identity, payload shape, and representativeness label
- bounded-work or routing evidence relevant to the measured path, such as candidate counts, tier hits or escalations, cache hit or miss state, deferred payload-fetch counts, or foreground latency delta for background jobs
- an explicit note when the run is exploratory and therefore cannot close a stage gate

### Human-like-memory evaluation lenses

For later-stage follow-ons that claim to improve how memory behaves rather than only raw speed, benchmark and evaluation bundles should make the claimed gain explicit. Useful lenses include persistence quality, active-work continuity, procedural reuse, autobiographical continuity, interference resilience, source-trust calibration, schema usefulness, and dream-synthesis utility.

These lenses do not replace the existing stage-gate benchmarks above. They complement them, and any feature using them still owes bounded-work, route, cache, denial, and foreground-latency evidence where relevant.

### Review checklist for benchmark proof

Use this checklist when a PR or handoff cites benchmark evidence as part of its acceptance story:

- name the exact benchmark report or artifact bundle rather than citing an unlabeled screenshot or prose summary
- record the harness or command entrypoint, dataset or fixture identity, machine profile, build mode, warm/cold semantics, sample count, and representativeness label
- show touched p50/p95/p99 path metrics when the benchmark covers a request path
- preserve the bounded-work, route, cache, denial, omission, or background-job fields that explain why the number moved
- reject the proof if another reviewer cannot rerun the command or trace the artifact path without reverse-engineering hidden setup

### Logging-heavy end-to-end proof example

Use a logging-heavy proof artifact when a PR needs to show more than a passing latency number. The artifact should preserve enough machine-readable output that a reviewer can inspect the route, bounded-work counters, and outcome class directly.

```bash
membrain recall --query "capital of france" --namespace demo --json \
  | tee artifacts/mb-dve.5.4/recall_cli.json
```

A usable artifact from a command like this should let the reviewer inspect at minimum:

- the exact command or harness that produced the artifact
- the fixture or namespace identity used for the run
- request-path p50/p95/p99 when the proof is benchmark-closing rather than a single smoke example
- route outcome, tier selection or escalation, candidate counts, and capped/degraded markers when the touched contract is retrieval boundedness
- denial, redaction, filtered-count, or omission fields when the touched contract is policy-sensitive
- the artifact path checked into the PR bundle, release evidence bundle, or referenced ops location

This kind of proof does not replace deterministic semantic fixtures. It complements them by showing the operator-visible trace and machine-readable fields that later regressions should preserve.

### Regression-signal expectations by benchmark class

Benchmarks are not only pass or fail latency snapshots. They must also preserve the operator-visible signals that would make regressions diagnosable after landing.

- request-path benchmarks should emit the touched path latency percentiles together with route outcome, tier selection or escalation, candidate counts, and any capped or degraded-path markers
- encode-path benchmarks should emit duplicate-route outcome, shortlist or nearest-neighbor evidence, cache hit or miss state, and whether interference work was applied, skipped, or deferred
- cache or warm-path benchmarks should emit cache family, cache event, cache reason, warm source, and distinct cold-versus-disabled-versus-stale outcomes when those lanes are in scope
- background-job benchmarks should emit job duration, queue depth or backlog growth, affected-item counts, retry or escalation state when applicable, and foreground latency delta rather than reporting only isolated throughput
- denial, policy, or redaction-sensitive benchmarks should preserve the same denial or omission outcome class and filtered-count evidence that the explain or audit path would expose on the live surface
- explainability-sensitive benchmarks should keep route-trace, omission, or audit fields machine-readable so later changes can compare contract fields instead of relying on prose summaries

A benchmark result is incomplete when it reports acceptable latency but drops the trace, count, cache, denial, or background-job fields that would explain why the number moved.

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
