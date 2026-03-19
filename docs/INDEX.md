# membrain — Documentation Index

> `PLAN.md` is the canonical design contract. All other docs elaborate on specific subsystems.
> If a subsystem doc and the plan diverge, the plan wins until the conflict is resolved explicitly.

## Core Documents

| Document | Purpose |
|----------|---------|
| [PLAN.md](PLAN.md) | Canonical mega-plan — architecture, schema, milestones, invariants, features |
| [ARCHITECTURE.md](ARCHITECTURE.md) | System read/write paths, component boundaries, workspace skeleton, and module ownership seams |
| [MEMORY_MODEL.md](MEMORY_MODEL.md) | Memory taxonomy, lifecycle states, core fields, contradiction handling |
| [NEURO_MAPPING.md](NEURO_MAPPING.md) | Brain mechanism → computational primitive mapping |
| [CLI.md](CLI.md) | Full CLI command reference with examples |
| [MCP_API.md](MCP_API.md) | MCP tool contract — inputs, outputs, rules |
| [OPERATIONS.md](OPERATIONS.md) | Production runbooks, failure modes, incident response |
| [CONTRIBUTING.md](CONTRIBUTING.md) | Contributor-facing contract: evidence requirements, PR rules, quality gates |
| [../AGENTS.md](../AGENTS.md) | Active repository workflow: coordination, reservations, handoff, and execution discipline |

## Documentation Precedence and Scope

1. `PLAN.md` is the canonical design contract and wins when another document drifts.
2. Subsystem docs under `docs/` elaborate the relevant surface; they do not override the plan.
3. `ARCHITECTURE.md` freezes workspace shape, module ownership seams, and read/write-path boundaries derived from the plan.
4. `CONTRIBUTING.md` freezes contributor-facing evidence requirements, quality gates, and PR rejection triggers.
5. `../AGENTS.md` translates those contracts into day-to-day workflow, coordination, and troubleshooting guidance for active contributors.
6. Entry-point docs and indexes help readers navigate; they do not redefine the contract.

Use this split intentionally: decide **what the system promises** from `PLAN.md` plus the relevant subsystem doc; use `ARCHITECTURE.md` when the question is workspace shape, module ownership, or boundary placement; use `CONTRIBUTING.md` to decide **what evidence a change owes**; use `../AGENTS.md` to decide **how to execute, coordinate, and hand off work**.

### Safe local clarification vs. stop-and-escalate

Use this rule set when docs appear incomplete, overlapping, or slightly out of sync:

- **Safe local clarification:** you may patch `INDEX.md`, a subsystem doc, or workflow guidance directly when `PLAN.md` already makes the canonical meaning clear, the change only restates or sharpens that existing contract, and the edit does not introduce new product behavior, workflow rules, or evidence thresholds.
- **Still safe:** navigation fixes, wording cleanups, cross-reference repairs, and explicit reminders that `PLAN.md` wins are local clarifications when they do not change scope.
- **Stop and escalate:** do not guess when multiple plausible readings would change product behavior, contributor workflow, acceptance evidence, or dependency ordering.
- **Also escalate:** stop when resolving the conflict would require inventing new contract language, creating a new exception, or deciding which of two non-canonical docs should win without direct support from `PLAN.md`.
- **Escalation target:** capture the ambiguity or conflict in the active bead; if no bead cleanly owns it, create a focused follow-up bead that names the conflicting files or sections and the decision that remains blocked.
- **Coordination rule:** when you stop for ambiguity, notify collaborators in the matching Agent Mail thread so parallel work does not silently diverge.

## PLAN.md Structure Reference

The mega-plan contains these major regions:

### Original Plan (compact)
- Sections 1–11: Problem → Brain analysis → Gap → Port mechanisms → Architecture → Performance → Techstack → Schema → CLI/MCP → Milestones → Acceptance

### Detailed Plan (expanded)
- Sections 1–15: Same topics with full depth — ~10,000 lines of SQL, Rust, pseudocode, and rationale
- Section 10: Top 10 high-level feature extensions (Dual Memory Output, Belief Ledger, Memory Leases, Reflection Compiler, Cognitive Blackboard, Resumable Goals, Preflight Sandbox, Namespace Lenses, Uncertainty Surface, Journal+Doctor)
- Section 12: Implementation milestones with detailed deliverables, tests, and acceptance criteria

### Upgrade Overlays
- Research framing, design invariants, non-negotiable restrictions, performance budgets, benchmark contracts, quality gates

### Mega-Plan Additions (Sections 12–45)
- 12: Canonical architecture invariants
- 13: Memory model extension (taxonomy, fields, schema rules)
- 14: Lifecycle and state transition rules
- 15: Retrieval architecture contract
- 16–20: Ranking, storage, indexing, association graph, cache/prefetch
- 21–22: Consolidation plan, forgetting plan
- 23–31: Compaction, governance, operations, failure modes, benchmarks, tests, sharding, research, priorities
- 33–35: Detailed data schema, MCP API contract, CLI contract
- 36–39: Algorithm catalog, ranking calibration, performance budget, speed checklist
- 40–45: Milestone gates, Rust workspace skeleton, contributor workflow, execution order, final thesis

### Feature Implementation Specs
- **Section 46 — Batch 1** (Features 1–10): Dream Mode, Belief Versioning, Query-by-Example, Context Budget API, Temporal Landmarks, Passive Observation, Confidence Intervals, Skill Extraction, Cross-Agent Sharing, Health Dashboard
- **Batch 1 nuance**: some features carry early schema hooks but later operational surfaces; Feature 9 lands `namespace_id`, `agent_id`, and `visibility` early while keeping the full sharing API gated behind the namespace/governance contract.
- **Section 47 — Batch 2** (Features 11–20): Causal Chain Tracking, Snapshots + Time Travel, Attention Heatmap, Semantic Diff, Fork + Merge, Predictive Pre-recall, Schema Compression, Emotional Trajectory, Audit Log, Query Intent Routing
- **Batch 2 framing**: later-stage, non-blocking follow-on scope for advanced trust/introspection, branching, predictive, and compression-adjacent features; use it to navigate deferred value without reopening the core execution spine in Sections 40–45.

### Cross-batch capability lenses
- **Trust and historical introspection**: use this lens when discussing later-stage user value across batches. It currently groups Belief Versioning (F2), Causal Chain Tracking (F11), Snapshots + Time Travel (F12), Semantic Diff (F14), and Audit Log (F19).
- These lenses complement the canonical batch chronology; they do not replace milestone order or reopen the core execution spine.

### Phase 4 follow-on framing
- Use this framing to keep deferred Phase 4 work visible without turning it into default architecture direction or reopening the core execution spine.
- **Scale-out and distribution**: later-stage, evidence-gated follow-ons that stay blocked until measured workload pressure justifies leaving the bounded single-node model.
- **Advanced operations and operator ergonomics**: later-stage maturity work that sharpens runbooks, automation, and operator guidance after the base repair/governance spine exists.
- **Inspectable working-state and cognitive blackboard surfaces**: later-stage, optional work that exposes active task/session state, pinned evidence, resumable goal stacks, checkpoint visibility, and explicit pause/resume/abandon status as a visible working-state object. Use this family when clarifying resume semantics or blackboard snapshots, but keep it gated behind the stable retrieval, repair, snapshot, and governance spine, bounded and namespace-aware, subordinate to the canonical retrieval/explain surface, and explicitly non-authoritative compared with durable memory truth and Feature 12 named historical snapshots.
- **Quality-loop and skill-memory follow-ons**: later-stage work that turns benchmark, maintenance, and skill-extraction signals into operator guidance or bounded background tuning without becoming core-path prerequisites.



