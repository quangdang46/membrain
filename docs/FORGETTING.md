# FORGETTING

> Canonical sources: `PLAN.md` Section 22, `CONSOLIDATION.md`, and `MEMORY_MODEL.md` retention/lifecycle rules.

## Design philosophy
Forgetting is a controlled reduction of detail, priority, or accessibility in order to preserve utility and reduce noise.

## Operations
- suppress
- decay
- demote
- compact
- summarize
- archive
- redact
- soft delete
- hard delete

## Rules
1. Prefer compression over deletion.
2. Prefer demotion over deletion.
3. Never delete the only authoritative evidence unless policy explicitly allows it.
4. Keep lineage long enough to explain how summaries were formed.
5. Separate privacy-driven deletion from utility-driven forgetting.

## Homeostasis and archival safety
- Homeostasis is a bounded forgetting pressure, not blanket deletion.
- Strength downscaling may make weak memories archive-eligible, but archival still requires explicit threshold checks, policy gating, and an auditable archive reason.
- Pinned memories, legal-hold data, and last authoritative evidence remain ineligible for homeostatic archival.
- Archival is recoverable soft deletion; privacy, compliance, or operator-driven destructive deletion follows separate policy-governed paths.
- Near-threshold memories may be surfaced for reinforcement or operator review instead of silently disappearing when their future utility is still uncertain.
