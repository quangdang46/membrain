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

## Lease-sensitive freshness
- Memory-like objects carry explicit lease policies so freshness does not remain implicit forever. The canonical policy classes are `volatile`, `normal`, `durable`, and `pinned`.
- Lease evaluation produces explicit freshness states such as `fresh`, `lease_sensitive`, `stale`, and `recheck_required`; `pinned` items remain exempt from lease-expiry downgrade unless another policy lane acts on them.
- `lease_sensitive` means the lease expired and confidence must be downgraded explicitly. `recheck_required` means stale action-critical evidence must be re-checked or withheld instead of silently supporting a high-confidence action surface.
- Bounded lease scanning is a transition-only maintenance pass. It may update freshness states, recheck flags, and queue telemetry, but it must not perform full-store rescoring or payload-wide recomputation on the request path.

## Interference versus emotional-bypass retention
- Interference is a similarity-driven pressure between distinct memories. It applies bounded maintenance or retrieval effects such as retroactive weakening of similar older memories or proactive retrieval difficulty, and it remains separate from duplicate-family, contradiction, and policy lanes.
- Emotional-bypass retention is a per-memory decay override derived from emotional tagging. While `bypass_decay` remains active, elapsed logical ticks do not lower `effective_strength`, and decay-only pruning must not treat that memory as ordinary weak-forgetting input.
- Emotional bypass is not blanket immunity from every forgetting or governance path. It does not authorize identity reuse, policy bypass, silent restore, or exemption from explicit demotion, archival review, or other non-decay lifecycle decisions.
- The mechanisms may co-exist: a highly emotional memory can still participate in interference checks, and interference penalties do not themselves set, clear, or explain `bypass_decay`.
- Any later reevaluation of emotional bypass belongs to explicit emotional-processing or desensitization controller behavior, not to the interference lane.

## Demotion, archive, deletion, and restore distinctions
- Demotion reduces default serving priority, payload attachment, or hot-route residency, but the memory remains in an active non-`Archived` lifecycle state and may still surface in normal recall when policy and ranking allow.
- Demotion may leave some previously served payload surfaces detached or lower fidelity; later promotion or reattachment can recover retained payloads where available, but that is rehydration of an active memory rather than restore from `Archived`.
- Archive is an explicit durable lifecycle state entered by forgetting or retention action. Archived memories stop surfacing in ordinary default recall, but remain inspectable through metadata, lineage, policy, and audit surfaces.
- Hard deletion is a separate policy- or compliance-governed action. It may remove recoverability, but it must not masquerade as utility-driven forgetting.

## Restore semantics
- Restore is an explicit controller or operator action, not an automatic consequence of ordinary recall, cold-tier lookup, snapshot inspection, or cache warming.
- Restore requires namespace and policy authorization, must respect pins, legal holds, retention or deletion rules, and must preserve identity, provenance, lineage, contradiction state, and archive-reason history.
- The restore workflow is ordered and inspectable: `validate_restore_request` -> `load_durable_metadata` -> `rehydrate_available_payload` -> `commit_durable_state` -> `refresh_derived_state`.
- A successful restore returns the memory to active serving eligibility and rebuilds or refreshes derived indexes and caches only after the durable state change commits.
- If only part of the previously served form can be recovered because payloads were detached, redacted, or lost, the system should surface `partially_restored` rather than fabricating a perfect pre-archive copy.
- Ranking, inspect, and explain surfaces must disclose when an item is archived, restored, or only partially restorable so operators can tell whether omission from normal recall is lifecycle- or policy-driven rather than silent truth loss.
- Forgetting explain and audit outputs must be operator-reviewable after the run: each surfaced decision should preserve `action`, `reason_code`, `disposition`, `policy_surface`, `reversibility`, archive-state transitions, and an `audit_kind`, while near-threshold retained items increment explicit review-required counters instead of disappearing into a generic `skip` bucket.

## Concrete archive and loss contract
- Archived cold records carry a stable `archive_state`, `archive_reason`, `content_ref`, `payload_ref`, `payload_state`, and bounded `compact_text` so metadata inspect remains available without fetching detached payload bytes.
- `payload_state` must distinguish `inline`, `detached`, `tombstoned`, and `unavailable` payload ownership rather than collapsing all non-hot data into one opaque archive bucket.
- Loss indicators are machine-readable and surface both the affected surface (`payload`, `content`, `lineage`, `derived_state`) and the degradation kind (`tombstoned`, `unavailable`, `stale`).
- Tombstones and loss indicators do not erase the archived row. They preserve the fact that the memory existed, why payload fidelity changed, and why restore may reopen only a degraded form.
