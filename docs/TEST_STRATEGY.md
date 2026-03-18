# membrain — Test Strategy

> Canonical sources: `PLAN.md` implementation milestones, benchmark overlays, and phase-gate rules.
> If this document diverges from `PLAN.md`, the plan wins.

The testing strategy must prove correctness, boundedness, durability, explainability, and stage readiness. Generic suite names are not enough by themselves; each stage must have explicit gate expectations and exit artifacts.

## Cross-cutting test rules

- Every suite emits structured artifacts useful for regression analysis.
- Failure-injection and recovery suites should also record the violated invariant, affected namespace or shard, authoritative durable input, whether prior durable state remained intact, and any repair or escalation artifact.
- No correctness, lifecycle, or policy test depends on wall-clock time when interaction ticks, logical ticks, or controlled clocks can express the same behavior; benchmark and soak harnesses may measure real elapsed time for performance claims but not as the sole proof of semantic behavior.
- Any request-path change must be tested for both correctness and boundedness.
- Any stateful mutation stage must include restart or crash-safety coverage where relevant.
- Policy and namespace checks must be validated before parity claims across CLI, daemon, IPC, or MCP layers.
- Benchmark results without representative metadata are exploratory evidence, not gate-closing evidence.
- Any benchmark-closing suite must record the benchmark harness or command entrypoint, dataset or fixture identity, build mode, warm/cold semantics, sample count, representativeness label, and artifact location needed to rerun or audit the measurement.

### Benchmark reproducibility proof obligations

Use these rules whenever tests or release evidence cite benchmark results as proof instead of as illustrative numbers:

- benchmark-closing suites must emit enough metadata to let another contributor rerun the same harness on the same declared workload class without reverse-engineering hidden setup
- the local loop may use smaller fixtures, but any claimed gate-closing benchmark still owes the same harness identity, dataset identity, build mode, warm/cold semantics, sample count, representativeness label, and artifact location as the larger run
- CI should fail closed when benchmark metadata is missing, contradictory, or detached from the artifact bundle even if the raw numbers themselves look acceptable
- release-tier or representative benchmark runs should reuse the same contract family and fixture naming as the smaller local or CI checks so reviewers can trace one proof chain across tiers instead of comparing unrelated benchmark prose
- benchmark metadata validation is part of the shared contract: a run that cannot be rerun, audited, or classified as representative versus exploratory does not satisfy the touched evidence gate

## Deterministic time-control contract

Use time as data, not ambient process state.

- correctness, lifecycle, retry-budget, timeout, decay, recency, reconsolidation, and policy tests should drive time through interaction ticks, logical ticks, or injected clock fixtures
- time-sensitive test artifacts should name the clock or tick source, the starting state, and the tick sequence needed to reproduce the outcome
- wall-clock sleeps, scheduler timing, or real elapsed time are acceptable only for benchmark or soak harnesses measuring performance characteristics; they do not close semantic correctness gates by themselves
- when a benchmark also touches time-sensitive semantics, pair the wall-clock measurement with deterministic fixture coverage for the semantic rule
- failure and repair artifacts for time-sensitive regressions should record the relevant tick or clock position so the scenario is replayable

### Rejectable sleep-based semantic test patterns

Treat the following patterns as rejectable for correctness, lifecycle, policy, or other semantic proof when deterministic fixtures could express the same rule:

- sleeping to wait for decay, timeout, retry-budget, backoff, reconsolidation-window, or recency semantics instead of advancing an injected clock or logical tick source
- polling wall-clock time until a state transition "probably" happened rather than asserting the exact guard, trigger, and expected artifact
- using benchmark latency runs as the only evidence for semantic claims such as denial timing, retry exhaustion, or lifecycle-edge legality
- depending on scheduler jitter, async race timing, or host load to make a test pass instead of naming the ordering contract explicitly
- mixing one nondeterministic sleep-heavy integration test with otherwise deterministic fixtures and treating the combined result as equivalent to replayable semantic coverage

A semantic test is incomplete when another contributor cannot rerun the same starting state, tick or clock sequence, and expected outcome without depending on ambient elapsed time.

## Namespace and isolation minimum matrix

Any change to namespace, sharing, ACL, denial, or redaction semantics must include dedicated coverage for:

- explicit same-namespace allow paths
- deterministic default-namespace binding when one default exists
- missing-namespace validation failure when no deterministic default exists
- malformed or unknown namespace rejection before candidate generation or writes
- cross-namespace denial without leakage of protected counts, handles, or existence hints
- approved shared/public access paths via explicit visibility controls
- explicit `share` / `unshare` mutation paths showing re-share idempotency, revocation of widened access, and preservation of namespace binding and durable identity
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

### Formula and transition fixture contract

Use named, replayable fixtures for any touched formula or lifecycle edge so later contributors can rerun the exact semantic proof instead of reconstructing it from prose.

- Each touched canonical formula fixture should record the authoritative formula family, full input fields, tick or clock anchor, expected output or tolerance, and whether parity against another execution lane is required.
- Lifecycle-edge fixtures should record the pre-state, attempted action, guard context, expected outcome class (`success`, `validation_failure`, `policy_denied`, or `internal_failure`), post-state expectation, and any required repair or escalation artifact.
- When formula math and lifecycle mutation interact in one flow, include deterministic fixtures for the ordering contract rather than testing the pieces in isolation. This especially applies to paths where decay is first persisted, then recall-side reinforcement or labile-window reopening is applied, and to rejected or retried flows that must not double-apply the accepted mutation.
- Property suites should persist failing seeds or counterexample inputs in their structured artifacts so invariant regressions can be replayed without rediscovering the input space.

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

## Contributor validation pyramid

Contributors should run the smallest tier that can still prove the touched contract. If a change spans multiple classes, run the union of the required tiers; representative benchmarks or release rehearsals do not replace deterministic local or CI semantic proof.

### Local loop
Use before handoff and for every change.

- run targeted deterministic suites for the touched contract surface: unit/property/transition fixtures, doc-example parity fixtures, focused integration paths, and boundedness smoke checks as applicable
- prefer the smallest replayable fixture or workload that can prove the claim instead of whole-system suites
- local artifacts should name the fixture or workload identity, changed surface, and any intentionally deferred higher-tier checks

### Required CI loop
Use as the shared merge gate whenever a change affects repository-visible behavior.

- rerun the touched deterministic suites in a clean environment
- include cross-surface parity, namespace/policy, restart/rebuild, or boundedness suites when those contracts are in scope
- validate benchmark metadata, schema/package manifests, and machine-readable interface artifacts that contributors should not trust from one workstation alone
- fail closed when CI cannot prove the changed contract; do not treat a skipped gate as equivalent to a pass

### Release or heavyweight loop
Use when evidence depends on representative workloads, cross-platform/package behavior, or operational failure modes that exceed ordinary merge-gate cost.

- run representative benchmark/load/soak suites, repair or migration drills, install/package checks, and other operator-facing rehearsals on declared hardware or corpora
- produce the benchmark report, failure matrix, ops note, and related release artifacts required by the touched phase gate or change class
- reserve this tier for stage-closing proof, hot-path claims, repair/governance-sensitive changes, distribution work, or other cases where CI-scale fixtures are intentionally smaller than production evidence

### Review-proof collection checklist

Use this checklist when assembling the validation payload for a PR, bead handoff, or release gate:

- name the touched change classes before choosing suites
- collect the smallest deterministic fixture set that proves semantic correctness for each touched class
- add parity, restart, isolation, or boundedness suites whenever the changed contract crosses those surfaces
- add benchmark or representative workload evidence only for the paths that need elapsed-performance proof
- attach at least one logging-heavy end-to-end artifact when reviewers need to inspect route traces, denial or redaction evidence, degraded markers, migration safeguards, or other machine-readable operator signals
- reject the proof bundle if any named gate is replaced by prose, screenshots without structured output, or CI status without a fixture or artifact identity

### Change-class to minimum tier mapping

| Change class | Minimum local proof | Required shared gate | Heavyweight/release proof when... |
| --- | --- | --- | --- |
| Prose-only doc clarification with no contract or example drift | targeted doc review plus nearby deterministic references stay accurate | optional unless a shared docs gate already exists | not required |
| Interface/example/flag/envelope changes | doc-example parity fixtures and canonical spelling checks for touched examples | cross-surface parity for the touched CLI, daemon/JSON-RPC, and MCP surfaces | install/package/release docs or shipped surface manifests change |
| Formula, lifecycle, namespace, or policy semantics | deterministic unit/property/transition fixtures on the changed rule | rerun touched deterministic suites plus restart/policy/isolation coverage where relevant | representative load or stage-gate evidence is also being claimed |
| Retrieval, ranking, cache, or other hot-path behavior | correctness plus boundedness smoke fixtures and targeted microbenchmarks | regression suites and benchmark-metadata checks for the touched path | latency/SLO claims, representative corpora, or promotion gates depend on the result |
| Repair, migration, retention, or degraded-mode operations | dry-run/blocked/error-path fixtures plus durable-truth expectations | restart/rebuild/governance matrix for the affected surface | operator runbooks, migration safety, or failure-injection claims are being promoted |
| Packaging, install, release, or distribution changes | local manifest/schema sanity checks | package/install smoke on supported build surfaces | binaries, installers, cross-platform guarantees, or distribution readiness are being asserted |

### Fixture and workload promotion strategy

- keep stable fixture identities across tiers: a local micro fixture, CI scenario fixture, and release representative workload should describe the same contract family even when scale differs
- use micro fixtures for deterministic semantics, scenario fixtures for multi-step restart/repair/parity flows, and representative workloads for benchmark/load/release evidence
- doc examples should map each source doc anchor to a normalized canonical operation plus expected machine-readable fields so drift can be classified precisely
- benchmark workloads must declare cardinality, hardware, build mode, warm/cold status, sample count, representativeness label, and artifact location; CI may validate those declarations even when only release-tier infrastructure can generate the final numbers
- when one change touches both semantic correctness and performance, pair a deterministic fixture with the larger workload rather than asking the larger workload to prove both alone

## Interface contract and doc-example parity minimum matrix

Any change to CLI, daemon/JSON-RPC, or MCP command examples, shared flag vocabulary, request envelopes, response envelopes, or remediation examples must include dedicated coverage for:

- golden example fixtures that normalize documented CLI invocations, daemon/JSON-RPC requests, and MCP tool calls into the same canonical operation so example drift is detected as contract drift rather than left to prose review
- canonical spelling checks for command names, tool names, method names, shared flags, and command-specific parameters so examples cannot silently reintroduce undeclared aliases, stale flags, or stale request fields after the stable interface contract changes
- response-envelope parity checks ensuring example success, partial, degraded, blocked, rejected, and policy-denied flows preserve the correct machine-readable fields and meanings across CLI JSON, daemon/JSON-RPC, and MCP surfaces
- explicit validation of explanation families, safeguard objects, remediation hints, and availability markers whenever an example claims to show them, including the stable field-family names rather than transport-specific paraphrase
- health/ops example parity checks whenever docs change `health`, `stats`, or `doctor` semantics, including repair-queue, backpressure, availability-posture, and feature-availability fields when those signals are claimed as operator-visible
- timeline/landmark example parity checks whenever docs change Feature 5 semantics, including stable `era_id` usage, read-only `timeline` behavior, explicit `landmark` mutation examples, and active-versus-closed era visibility when those are claimed as operator-visible
- structured artifacts that record the source doc section or anchor, normalized operation, expected canonical fields, and drift classification so reviewers can tell whether a failure reflects semantic breakage or example hygiene noise

### Doc-example drift blocking rules

Treat the change as blocked until examples are updated when drift changes any of:

- canonical command, tool, or method spelling
- shared or command-specific flag / parameter names, requiredness, incompatibility rules, or normalization semantics
- machine-readable response fields or outcome-class meaning
- remediation, availability, or safeguard semantics that affect user recovery or safety expectations
- example claims about namespace, policy, degraded-mode, or cross-surface parity behavior

Do not block changes for formatting-only drift such as:

- whitespace, line wrapping, or key ordering where the machine-readable meaning stays identical
- placeholder IDs, timestamps, or example values that keep the same contract meaning
- prose-only rewording that does not alter command spelling, field names, or envelope semantics

## Recall quality, policy, and isolation minimum matrix

Any change to recall ranking, packaging, sharing, redaction, or namespace enforcement must include dedicated coverage for:

- representative judged corpora or canonical fixtures that make expected `full`, `partial`, `miss`, contradiction-aware, and tip-of-the-tongue outcomes explicit enough to detect ranking or packaging drift
- stable top-K or shortlist expectations for direct, recent, lexical, semantic, and bounded graph-assisted paths where those lanes are in scope, including returned-result reasons and omitted-result reasons rather than score-only assertions
- same-namespace allow behavior and explicit shared/public allow paths producing the same evidence, policy summaries, and deferred-payload behavior across CLI, daemon, IPC, and MCP surfaces
- era-scoped recall fixtures proving that `era_id` narrows to one explicit era without widening to adjacent history, and that malformed, unknown, or unauthorized era selectors fail explicitly before widened candidate generation
- cross-namespace, owner-boundary, and policy-denied requests rejecting before widened candidate generation and without leaking protected handles, counts, existence hints, or suppressed conflict siblings
- policy-filtered or redacted winners degrading to explicit omission, preview, partial, or miss semantics rather than silently substituting unauthorized payloads or widening the candidate cut
- adversarial ambiguous cues, near-duplicate clusters, contradiction-bearing candidate sets, and fragmentary cues preserving bounded behavior, inspectable uncertainty, and conflict markers instead of speculative reconstruction
- cache, prefetch, and repair-path recall preserving the same namespace, policy, omission, and explanation semantics as the colder canonical path
- intent-routed `ask` fixtures proving visible intent class, low-confidence fallback, explicit `--override-intent` / `override_intent` behavior, and action-oriented safety downgrades preserve bounded candidate generation and machine-readable route metadata across CLI, daemon, IPC, and MCP surfaces
- structured artifacts that record fixture identity, namespace or visibility setup, expected winners and omissions, policy decisions, and machine-readable explanation fields such as `result_reasons`, `omitted_summary`, `policy_summary`, `conflict_markers`, and `trace_stages` when available

## Observability and regression-signal minimum matrix

Any change that promises observability, boundedness, or regression detection must include dedicated coverage for:

- operator-visible latency, route-trace, audit, explain, or inspect fields for the touched contract surface rather than prose-only claims
- structured artifacts that preserve the exact signal names, outcome classes, and field families reviewers are expected to compare across runs
- negative-path checks proving the signal changes when the contract is violated, capped, denied, degraded, retried, or timed out rather than staying silent
- parity of the named regression signals across CLI, daemon, IPC, and MCP surfaces where the changed flow exists
- benchmark or load evidence paired with deterministic semantic fixtures when the same change touches both elapsed-performance behavior and semantic correctness

Minimum change-class coverage:

- request-path changes: p50/p95/p99 plus candidate counts, tier-routing or escalation traces, and capped-route or degraded markers
- encode-path routing changes: shortlist or nearest-neighbor evidence, duplicate-route outcome, cache state, and interference applied/skipped/deferred markers
- policy or governance changes: denial or redaction outcome class, enforcement stage, filtered counts, audit-handle visibility, and no-leakage behavior
- cache or warm-path changes: cache family, hit/miss/bypass/invalidation events, stale or bypass reasons, warm source, and distinguishable cold-versus-disabled-versus-stale outcomes
- background-job changes: job duration, queue depth, affected-item counts, foreground latency delta, retry budget or escalation state when relevant, and degraded-mode markers
- explain or inspect envelope changes: stable machine-readable route-trace, omission, denial, stale, and conflict field families that downstream checks can compare directly

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
- preview / blocked / degraded / rejected safeguard parity across CLI, daemon, IPC, and MCP surfaces whenever the changed operation can mutate authoritative state, widen scope, or emit irreversible-loss records
- differentiation between `policy_denied` or malformed-request rejection versus confirmation-missing, snapshot-missing, stale-preflight, or other blocked-readiness outcomes
- force-confirmed flows proving that local confirmation changes only the confirmation state while policy, namespace, retention, confidence, and legal-hold checks continue to apply unchanged

## Restart, rebuild, and recovery verification minimum matrix

Any change to startup bootstrap, durable-state replay, derived-surface rebuild, repair resume, or degraded recovery posture must include dedicated coverage for:

- clean restart restoring canonical counters, generation anchors, durable handles, and other persisted state needed for request semantics without trusting stale or mixed-generation warm state
- startup with missing or stale derived indexes, graph materializations, caches, or sidecars proving the system falls back to colder reads or explicit degraded/read-only posture until validation passes rather than advertising full health prematurely
- startup with unreadable, ambiguous, or mixed-generation authoritative inputs failing closed for the affected namespace or shard instead of replaying speculative recovery from derived artifacts
- seeded divergence tests for `repair index`, `repair graph`, `repair lineage`, cache drop-and-rewarm, and similar rebuilds proving rebuilt outputs regain candidate-count, lineage, namespace, and policy parity with durable truth before the surface is marked healthy
- interrupted restart, rebuild, or recovery runs resuming idempotently from durable checkpoints or explicit repair queue state without duplicating mutations, widening scope, or losing the prior durable state
- when resumable-goal or blackboard checkpoint features are enabled, restart fixtures that prove pause/resume/abandon state restores from explicit task/session-bound checkpoints, preserves selected-evidence handles and pending dependencies, and fails closed rather than guessing a new active plan when the checkpoint or governing task context is unreadable
- operator-visible recovery artifacts recording the failing surface, serving posture, authoritative input generation or snapshot, queued follow-up repairs, any irreversible-loss record, and the exact checks required to clear degraded mode
- cross-surface parity for stats, health, doctor, explain, inspect, audit, and import/export manifest signals that expose degraded, graph-disabled, index-bypassed, stale-warning, replay-pending, stale-preflight-invalidated, redacted, or partial-transfer state across CLI, daemon, IPC, and MCP where those surfaces exist

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
- if emotional trajectory metadata is included, `encoding_valence` / `encoding_arousal` capture remains explainable, bounded, and reproducible enough for later inspect, uncertainty, and consolidation flows

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
- recall-quality and policy/isolation tests covering same-namespace allow, cross-namespace denial without leakage, redacted-winner degradation, and contradiction-aware partial recall
- adversarial tests for ambiguous partial cues, near-duplicate clusters, and contradiction-aware partial recall
- parity tests for summary versus full explain surfaces across CLI, daemon, IPC, and MCP where those surfaces exist

**Must prove before closing the stage**
- Tier1, Tier2, and Tier3 all meet their declared latency contracts on representative corpora
- graph/engram expansion respects hard node and depth caps
- partial-match paths do not leak full payloads before the final cut
- tip-of-the-tongue results stay explicit about fragmentary status, remaining ambiguity, and omitted evidence
- low-signal partial cues terminate with a bounded miss or fragment shortlist rather than speculative reconstruction
- same-namespace allow paths, explicit shared/public allow paths, and cross-namespace or policy-denied paths all preserve the same bounded candidate-generation and omission semantics without existence leakage
- redacted or policy-filtered winning candidates degrade explicitly to omission, preview, partial, or miss semantics rather than silently substituting unauthorized payloads
- explain outputs preserve returned-result reasons, omitted-result reasons, provenance summaries, freshness or conflict markers, and stable routing-trace fields without cross-surface semantic drift
- if `mood_congruent` is enabled, the emotional bonus remains opt-in, bounded to already-eligible candidates, and inspectable enough to show whether it changed ordering or was non-decisive

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
- if emotional trajectory is included, timeline emission remains namespace-aware, read-only from the operator perspective, and rebuildable from bounded emotional metadata rather than becoming a hidden second truth source

### Stage 7 — Graph maturity

**Required suites**
- integration tests for formation, split, sibling creation, recall expansion, and any later-stage Dream Mode follow-on that emits graph-affecting synthesis artifacts
- property tests for traversal caps, centroid stability, and bounded Dream Mode candidate or link-creation caps when that feature is enabled
- restart tests for serialization integrity, including interrupted or paused Dream Mode state if Stage 7 promotion includes the offline synthesis path
- latency tests for graph-assisted retrieval overhead

**Must prove before closing the stage**
- graph-assisted retrieval stays bounded under declared caps
- graph persistence survives restart without corruption
- split logic remains reproducible enough for operational debugging
- if Dream Mode is included, the synthesis pass remains background-only, namespace/policy-aware, lineage-backed, and visibly non-blocking to foreground latency budgets

## Required exit artifacts for a completed stage

Every stage completion should leave behind:

- benchmark report
- failure matrix
- design note
- migration note if schema changed
- rollback note if behavior changed
- ops note if background jobs changed

## Phase gate proof map

Each roadmap phase closes only when the stage artifacts above are strong enough to satisfy the corresponding `PLAN.md` phase gate rather than merely proving isolated tests passed.

### Phase 0 — Contracts and measurable foundation prerequisites
- the benchmark report must prove a rerunnable harness exists and that Tier1 MVP latency is measurable on declared hardware
- the design note must freeze the object-model and invariant assumptions needed for benchmarkable work
- the failure matrix must cover the baseline boundedness and correctness assumptions that would invalidate early measurement if they drift

### Phase 1 — Core encode, storage, and bounded retrieval baseline
- benchmark reports must cover Tier2 indexed retrieval, session or entity query behavior, and a measurable ranking baseline
- the design note or interface note must identify the debug or operator retrieval explanation surface used to satisfy the phase gate
- rollback or migration notes must be present whenever new retrieval or storage behavior changes external semantics

### Phase 2 — Contradiction handling, graph-assisted retrieval, and explainable packaging
- benchmark and failure artifacts must prove graph support stays budgeted and repairable, contradiction records exist, and explainable recall packaging is inspectable
- the design note must identify which routing, contradiction, and packaging fields are canonical for cross-surface validation
- migration or rollback notes must accompany any envelope, packaging, or contradiction-schema change that this phase introduces

### Phase 3 — Dynamic lifecycle, repair, and regression hardening
- benchmark reports must show consolidation utility, forgetting quality, and repair or compaction safety under failure injection
- the failure matrix must make unacceptable fact loss, stale-result exposure, or repair drift visible enough to block the phase if they remain unresolved
- ops notes are required whenever background maintenance behavior, repair controllers, or degraded-mode assumptions change

### Phase 4 — Operational tooling, justified scale-out, and later-stage extensions
- benchmark reports must justify sharding or distribution with empirical workload pressure rather than architectural preference
- the artifact bundle must include the operations runbook or design note for shard movement, repair, recovery, and governance enforcement
- failure and benchmark evidence must cover shard movement, repair, recovery, and cross-shard governance before the phase can close

## Global no-go conditions

Do not declare a stage ready if any of the following are still true:

- touched request paths have unknown p95/p99 behavior
- contradiction, policy, or namespace semantics changed without dedicated tests
- new derived state cannot be rebuilt or repaired from durable truth
- background execution succeeds only by degrading foreground contracts
- parity across standalone and service-facing surfaces is unverified where the stage depends on it

