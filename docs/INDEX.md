# membrain — Documentation Index

> `PLAN.md` is the canonical design contract. All other docs elaborate on specific subsystems.
> If a subsystem doc and the plan diverge, the plan wins until the conflict is resolved explicitly.

## Core Documents

| Document | Purpose |
|----------|---------|
| [PLAN.md](PLAN.md) | Canonical mega-plan — architecture, schema, milestones, invariants, features |
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
3. `CONTRIBUTING.md` freezes contributor-facing evidence requirements, quality gates, and PR rejection triggers.
4. `../AGENTS.md` translates those contracts into day-to-day workflow, coordination, and troubleshooting guidance for active contributors.
5. Entry-point docs and indexes help readers navigate; they do not redefine the contract.

Use this split intentionally: decide **what the system promises** and **what evidence a change owes** from `PLAN.md` plus the relevant subsystem doc and `CONTRIBUTING.md`; decide **how to execute, coordinate, and hand off work** from `../AGENTS.md`.

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
- **Section 47 — Batch 2** (Features 11–20): Causal Chain Tracking, Snapshots + Time Travel, Attention Heatmap, Semantic Diff, Fork + Merge, Predictive Pre-recall, Schema Compression, Emotional Trajectory, Audit Log, Query Intent Routing



