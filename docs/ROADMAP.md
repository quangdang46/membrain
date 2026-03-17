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

- **Scale-out and distribution** stay evidence-gated follow-ons. The measured-demand gate and `SHARDING_AND_DISTRIBUTION.md` contract exist to resist premature scale-out and keep simpler bounded remedies preferred until workload evidence proves otherwise.
- **Advanced operations and operator ergonomics** sharpen runbook clarity, automation, and incident/maintenance handling only after the base operational contract is already repairable and benchmarked; they do not bypass maintenance-class, rollback, or observability requirements from `OPERATIONS.md`.
- **Inspectable working-state and cognitive blackboard surfaces** stay later-stage and optional. They may expose a per-task or per-session working-state object for current goal, subgoals, pinned evidence, active beliefs, unknowns, next action, and blocked reason so long-running reasoning becomes easier to steer, debug, and resume, but they must remain a visible, bounded projection over selected evidence and active workflow state rather than a second hidden memory store.
- When this family is promoted, validation should prove that retrieval promotes bounded evidence into the blackboard instead of dumping raw candidate floods, that `blackboard.get` / `blackboard.pin` / `blackboard.dismiss` / `blackboard.snapshot` remain namespace- and policy-aware, and that checkpointed blackboard state is inspectable without becoming authoritative durable truth.
- **Quality-loop and skill-memory follow-ons** consume rerunnable benchmark evidence, maintenance outcomes, and bounded skill-extraction outputs to drive later recommendations or background tuning; they stay non-blocking, inspectable, and subordinate to auditable inputs rather than becoming a hidden prerequisite for core-path behavior.
- **Predictive pre-recall and prospective-trigger follow-ons** stay later-stage and optional. They may learn bounded query-sequence or future-cue patterns and asynchronously warm Tier1 or planner hints, but only after the baseline recall path, cache invalidation rules, and route observability are already trustworthy; they must never become hidden request-path work, policy-bypass shortcuts, or reasons to widen candidate budgets.
- When this family is promoted, validation should prove that speculative warmups remain namespace-bound, cancellable, generation-aware, and explicit in health/stats or explain surfaces, and that disabling the feature changes latency posture only rather than the durable meaning of recall results.

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
