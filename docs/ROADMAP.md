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
- treat advanced operations and operational ergonomics as later-stage maturity work layered on top of the base repair, rollback, and governance surfaces rather than as blockers for the core execution spine
- treat quality-loop and skill-memory follow-ons as later-stage background/operator-guidance work driven by benchmark, maintenance, and extraction signals rather than as prerequisites for core correctness or phase promotion
- treat predictive pre-recall and prospective-trigger follow-ons as later-stage bounded-assistance work layered on top of proven retrieval, cache/prefetch, and observability contracts rather than as prerequisites for the baseline recall path
- treat snapshot and time-travel inspection as trust/introspection follow-on work: useful for safer operations and historical comparison, but not a prerequisite for proving the bounded core retrieval path
- treat belief versioning and inspectable history surfaces as trust/introspection follow-on work layered on top of the core contradiction/supersession contract rather than as a blocker for the bounded baseline retrieval spine

## Milestone gates and go-no-go evidence thresholds

Use these gates as explicit pass/fail review checkpoints. A phase is not complete because adjacent prose exists or a partial prototype runs; it is complete only when the named evidence exists, is inspectable, and matches the canonical `PLAN.md` and `CONTRIBUTING.md` contracts.

### Phase 0 go/no-go gate

Phase 0 may advance only when reviewers can point to all of the following:
- frozen contract artifacts for canonical types, schema semantics, invariants, and namespace or policy enforcement before expensive work
- a benchmark harness with rerunnable commands plus benchmark metadata covering dataset cardinality, machine profile, build mode, and warm/cold declaration
- measurable Tier1 MVP evidence with bounded-work signals and operator-visible observability hooks
- explicit rejection of any contract change that still lacks its required artifact set, such as missing migration notes, missing observability hooks, or unresolved doc inconsistency

Do not promote Phase 0 if contract freezes are still ambiguous, benchmark proof is anecdotal, or the only way to detect a regression is to re-read code.

### Phase 1 go/no-go gate

Phase 1 may advance only when reviewers can point to all of the following:
- Tier2 indexed retrieval evidence, session or entity query coverage, and a measurable ranking baseline
- inspect or explain surfaces that expose retrieval behavior in debug or operator form rather than opaque score-only output
- bounded-work validation proving request-path restrictions still hold, including no full scans, no uncapped graph traversal, no cold-payload fetch before the final candidate cut, and no policy bypass before expensive retrieval work
- benchmark or targeted latency artifacts for the touched hot path, plus the observability hook that would reveal candidate-budget, cache, or routing regressions

Do not promote Phase 1 if retrieval works but remains unexplainable, if ranking claims lack reproducible evidence, or if request-path restrictions are asserted without tests or inspectable signals.

### Phase 2 go/no-go gate

Phase 2 may advance only when reviewers can point to all of the following:
- contradiction records and conflict-aware storage represented explicitly rather than silently overwritten
- graph-assisted retrieval proof showing hard depth, node, and candidate budgets plus repairability of derived graph state
- explainable packaging for recall output that surfaces score components, sources, policy filters, and routing context
- dedicated isolation, denial, or audit evidence whenever namespace, sharing, visibility, or redaction behavior changes

Do not promote Phase 2 if graph retrieval is merely available but not budgeted, if contradiction handling is hidden behind overwrite semantics, or if governance-facing behavior changed without explain or audit proof.

### Phase 3 go/no-go gate

Phase 3 may advance only when reviewers can point to all of the following:
- benchmark-corpus evidence that consolidation improves utility and forgetting reduces noise without unacceptable fact loss
- deterministic fixtures for lifecycle-sensitive behavior instead of sleep-based correctness tests
- repair, rebuild, compaction, or maintenance coverage proving durable-truth-first recovery under failure injection
- observability for maintenance-class work, including job duration, queue depth, affected-item counts, and any foreground latency delta

Do not promote Phase 3 if maintenance behavior is safe only by convention, if rebuildability from durable evidence is unproven, or if lifecycle claims depend on nondeterministic timing.

### Phase 4 go/no-go gate

Phase 4 may advance only when reviewers can point to all of the following:
- measured-demand evidence that single-node bounded operation is under sustained pressure and simpler bounded remedies are no longer sufficient
- operations runbooks plus benchmarked shard movement, repair, and recovery artifacts when scale-out is proposed
- governance and isolation proof showing policy enforcement, auditability, and repairability remain intact across the operational surface
- explicit confirmation that later-stage features, operator ergonomics, and scale-out work remain non-blocking with respect to the core execution spine unless the canonical gate evidence says otherwise

Do not promote Phase 4 on ambition alone, on isolated operator anecdotes, or by treating later-stage features as hidden prerequisites for the bounded core system.

### Measured-demand readiness gate for sharding and distribution

Do not treat distribution as default roadmap direction. Promote it only when bounded single-node operation has credible evidence of strain that simpler optimizations cannot relieve.

Required readiness evidence should include:
- sustained benchmark pressure against current latency, throughput, rebuild, or maintenance budgets rather than isolated anecdote
- workload shape evidence showing that namespace, workspace, or temporal locality no longer fits the single-node operating model cleanly
- operator pain that remains unacceptable after bounded indexing, caching, repair, and maintenance improvements are applied
- recovery, repair, or compaction windows that are too large for the intended operating envelope
- governance and isolation proof showing that sharding would preserve policy enforcement, auditability, and repairability rather than weakening them

Distribution should stay blocked when the observed problem could still be solved by simpler bounded work such as better Tier1/Tier2 budgets, cache and prefetch tuning, maintenance scheduling, repair throughput improvements, or clearer operator tooling.

## Sequencing rules
1. Contracts, memory semantics, and namespace policy land before expensive retrieval work.
2. Retrieval becomes explainable before graph and advanced optimization layers become core behavior.
3. Repair and rebuild paths exist before large-scale operations or distribution.
4. Advanced features and sharding remain later-stage work; they never block the core architecture spine.

## Canonical execution order and dependency spine

Future beads should inherit this dependency spine directly rather than inventing local ordering:

1. define canonical types, schema semantics, and invariants
2. define policy model and namespace enforcement
3. implement Tier1 fast encode and exact/recent retrieval
4. implement Tier2 durable indexed storage and search
5. implement ranking explanation and inspect/explain surfaces
6. implement contradiction representation and conflict-aware storage
7. implement graph-assisted retrieval under hard budgets
8. implement consolidation pipelines
9. implement forgetting and demotion pipelines
10. implement repair and rebuild paths
11. implement benchmark harnesses and regression artifacts
12. implement operational tooling and doctor commands
13. introduce sharding only if empirical workload demands it

Treat this list as a prerequisite chain, not as a menu. Earlier steps unlock later work because the system must become measurable before complex, explainable before highly optimized, and repairable before operationally large.

### Dependency-spine validation checklist

Before a future bead adds or changes dependencies, confirm all of the following:
- the bead does not pull later-stage feature, scale-out, or operator-maturity work ahead of unfinished core-path prerequisites
- any retrieval, contradiction, graph, lifecycle, repair, or operations work still lands in the canonical order above
- the bead’s dependency story matches the relevant phase gate and does not bypass the required evidence for promotion
- if the bead changes ordering, the change is directly supported by `PLAN.md` rather than inferred from convenience or local prose
- if the proposed dependency order would make a later step block an earlier canonical prerequisite, stop and open or update the owning bead instead of guessing through the conflict

## Later-stage work gating and non-blocking rules

Use these rules to keep roadmap ambition from distorting the core execution order:

- Advanced feature batches, deep introspection surfaces, and Phase 4 scale-out ideas are promotion targets only after Phases 0–3 have satisfied their own gates; they are not prerequisites for proving the bounded core system.
- No later-stage feature may delay contract freezes, bounded encode/retrieval work, contradiction handling, repairability, observability, or governance enforcement that the earlier phases depend on.
- If a later-stage idea competes for the same design surface as core-path work, the core-path contract wins until the earlier gate is closed explicitly.
- Distribution and sharding remain evidence-gated follow-ons: they do not become default direction, required architecture, or release-blocking scope until actual workload pressure and Phase 4 gate evidence justify them.
- Advanced feature beads should be allowed to elaborate future value, but they must not reopen settled invariants such as bounded request-path work, explicit contradiction representation, deterministic namespace enforcement, explainability, and repairability from durable truth.
- Phase 4 operator ergonomics and quality-loop improvements may improve maturity after the core spine exists, but they must not be treated as blockers for the core architecture/retrieval/repair/governance sequence.
- When roadmap or backlog discussion introduces speculative later-stage work, keep it visible as deferred or gated follow-on scope rather than quietly folding it into near-term milestone promises.

## Phase 4 follow-on families

Use this framing to keep the parent Phase 4 bucket explicit without turning it into default near-term scope:

- **Batch 1 advanced feature family (Features 1–10)** is the navigation and decomposition layer for Dream Mode, Belief Versioning, Query-by-Example, Context Budget API, Temporal Landmarks, Passive Observation, Confidence Intervals, Skill Extraction, Cross-Agent Sharing, and Health Dashboard. Use the family to keep those later-stage feature threads discoverable and independently tracked without implying that they must ship together or can reorder the core execution spine.
- **Dream Mode / offline synthesis** stays later-stage, optional, and maintenance-class. It may add bounded cross-memory links or merge follow-on work during explicit idle windows after the graph and consolidation foundations are already trustworthy, but it must not become hidden request-path work, a reason to widen namespace scope, or a substitute for durable provenance and repairable graph truth.
- When Dream Mode is promoted, validation should prove bounded candidate selection, namespace/policy enforcement before synthesis, lineage-backed inspectability for emitted links or merge outcomes, restart-safe pause/resume behavior, and clear operator visibility into enabled/disabled or degraded posture.
- Batch 1 framing is not a second milestone system: individual child features may attach to earlier schema, retrieval, interface, or operator milestones when the canonical prerequisites already exist, but the family itself is not permission to reopen Phase 0–3 gates or turn unfinished siblings into hidden blockers.
- When one Batch 1 child is being refined, hand off future contributors to the matching child bead and subsystem contract instead of forcing them to re-derive the whole feature family from the mega-plan.
- **Scale-out and distribution** stay evidence-gated follow-ons. The measured-demand gate and `SHARDING_AND_DISTRIBUTION.md` contract exist to resist premature scale-out and keep simpler bounded remedies preferred until workload evidence proves otherwise.
- **Advanced operations and operator ergonomics** sharpen runbook clarity, automation, and incident/maintenance handling only after the base operational contract is already repairable and benchmarked; they do not bypass maintenance-class, rollback, or observability requirements from `OPERATIONS.md`.
- **Inspectable working-state and cognitive blackboard surfaces** stay later-stage and optional. They may expose a per-task or per-session working-state object for current goal, subgoals, pinned evidence, active beliefs, unknowns, next action, blocked reason, and resumable goal-stack context so long-running reasoning becomes easier to steer, debug, pause, abandon intentionally, and resume, but they must remain a visible, bounded projection over selected evidence and active workflow state rather than a second hidden memory store.
- This family depends on stable retrieval, snapshot, repair, and governance surfaces first: resumable-goal checkpoints are workflow-resume artifacts, not substitutes for Feature 12 named historical snapshots or for canonical task/session identity.
- When this family is promoted, validation should prove that retrieval promotes bounded evidence into the blackboard instead of dumping raw candidate floods, that `blackboard.get` / `blackboard.pin` / `blackboard.dismiss` / `blackboard.snapshot` remain namespace- and policy-aware, that resumable checkpoints restore explicit selected evidence and pending dependencies, fail closed rather than inventing a new active plan when task context is unreadable, and that checkpointed blackboard or goal-stack state is inspectable without becoming authoritative durable truth.
- **Quality-loop and skill-memory follow-ons** consume rerunnable benchmark evidence, maintenance outcomes, and bounded skill-extraction outputs to drive later recommendations or background tuning; they stay non-blocking, inspectable, and subordinate to auditable inputs rather than becoming a hidden prerequisite for core-path behavior.
- **Predictive pre-recall and prospective-trigger follow-ons** stay later-stage and optional. They may learn bounded query-sequence or future-cue patterns and asynchronously warm Tier1 or planner hints, but only after the baseline recall path, cache invalidation rules, and route observability are already trustworthy; they must never become hidden request-path work, policy-bypass shortcuts, or reasons to widen candidate budgets.
- When this family is promoted, validation should prove that speculative warmups remain namespace-bound, cancellable, generation-aware, and explicit in health/stats or explain surfaces, and that disabling the feature changes latency posture only rather than the durable meaning of recall results.
- **Emotional trajectory and mood history surfaces** stay later-stage and introspection-first. They may expose bounded valence/arousal history and optional mood-congruent ranking hints after the underlying encode metadata, uncertainty surfaces, and explainable ranking contracts are already stable, but they must not become a hidden default retrieval mode or a reason to widen recall scope.
- When this family is promoted, validation should prove that mood history remains read-only and namespace-aware, that any `mood_congruent` bonus is opt-in and explainable, and that disabling emotional trajectory changes ranking posture only rather than canonical memory meaning.

## Trust and historical introspection cluster

Use this capability lens when discussing later-stage user value across feature batches without erasing the canonical batch chronology.

This cluster currently spans:
- **Feature 2 — Belief Versioning** for inspectable belief chains, explicit resolution state, and supersession-aware history surfaces
- **Feature 11 — Causal Chain Tracking** for traceable derivation and invalidation flow
- **Feature 12 — Snapshots + Time Travel** for named historical inspection anchors
- **Feature 14 — Semantic Diff** for state comparison across ticks or snapshots
- **Feature 19 — Audit Log** for operation-history and policy-visible change review

Use the cluster to talk about trust, historical inspection, and operator confidence as one capability family. It complements the batch-1 / batch-2 rollout structure rather than replacing it, and it remains later-stage follow-on scope gated behind the bounded core retrieval, governance, and repair spine.

## Non-negotiable restriction overlays

These phase summaries inherit the same guardrails frozen in the canonical plan and condensed architecture docs:

- no full-store scans on request paths
- no uncapped graph expansion on request paths
- no policy or namespace bypass before expensive retrieval work
- no premature cold-payload fetch before the final candidate cut
- no benchmark claims without representative context (dataset cardinality, machine profile, build mode, warm/cold declaration)

Read the phases with those restrictions in mind: if a later implementation idea violates them, it belongs in research or redesign, not in the core execution spine.
