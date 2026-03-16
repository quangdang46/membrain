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

## Demotion, archive, deletion, and restore distinctions
- Demotion reduces default serving priority, payload attachment, or hot-route residency, but the memory remains in an active non-`Archived` lifecycle state and may still surface in normal recall when policy and ranking allow.
- Demotion may leave some previously served payload surfaces detached or lower fidelity; later promotion or reattachment can recover retained payloads where available, but that is rehydration of an active memory rather than restore from `Archived`.
- Archive is an explicit durable lifecycle state entered by forgetting or retention action. Archived memories stop surfacing in ordinary default recall, but remain inspectable through metadata, lineage, policy, and audit surfaces.
- Hard deletion is a separate policy- or compliance-governed action. It may remove recoverability, but it must not masquerade as utility-driven forgetting.

## Restore semantics
- Restore is an explicit controller or operator action, not an automatic consequence of ordinary recall, cold-tier lookup, snapshot inspection, or cache warming.
- Restore requires namespace and policy authorization, must respect pins, legal holds, retention or deletion rules, and must preserve identity, provenance, lineage, contradiction state, and archive-reason history.
- A successful restore returns the memory to active serving eligibility and rebuilds or refreshes derived indexes and caches only after the durable state change commits.
- If only part of the previously served form can be recovered because payloads were detached, redacted, or lost, the system should surface a degraded or partially restorable result rather than fabricating a perfect pre-archive copy.
- Ranking, inspect, and explain surfaces must disclose when an item is archived, restored, or only partially restorable so operators can tell whether omission from normal recall is lifecycle- or policy-driven rather than silent truth loss.
