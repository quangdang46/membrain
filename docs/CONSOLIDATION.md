# CONSOLIDATION

> Canonical sources: `PLAN.md` Sections 21, 40 Stage 6, and `MEMORY_MODEL.md` lifecycle/tiering rules.

## Objective
Turn hot, repeated, or emotionally significant experience into colder, more reusable memory without losing durable evidence, breaking lineage, or violating foreground latency budgets.

## Workloads
- episode summarization
- fact extraction
- skill extraction
- duplicate family collapse
- contradiction detection
- relation reinforcement
- archive compaction

## Canonical consolidation cycle
One bounded background consolidation cycle runs in this order:

1. **NREM-style migration**
2. **REM-style linking and emotional processing**
3. **Homeostasis**

The cycle may be triggered by:
- hot-route pressure
- total-strength pressure
- explicit operator invocation such as `membrain consolidate`
- periodic maintenance cadence

Cycle-level rules:
- run asynchronously and remain pausable when foreground latency or correctness would be threatened
- emit inspectable reports for queue depth, moved items, linked items, archived items, and foreground latency delta
- bind namespace, policy, and lifecycle eligibility before mutating durable or derived state
- preserve prior committed state on failure and hand stale derived work to repair instead of leaving half-applied truth

## NREM-style migration contract
- NREM-style migration is the replay-and-transfer pass for eligible hot-route memories; it is not merely Tier1 cache eviction.
- Candidate selection stays bounded and may use signals such as effective strength, recency, recall or replay relevance, salience, and emotional priority.
- Migration may refresh embeddings, compress content, or create colder semantic artifacts, but it must preserve provenance, lineage, and canonical identity/version rules.
- Consolidation must never destroy the last authoritative evidence for a memory family merely because a summary, extract, or colder representation was produced.
- After migration, recall must remain able to reach the colder durable representation; any hot pointers, ANN sidecars, or cache entries remain derived and rebuildable.
- If migration updates engram membership, centroids, or relation structure, those graph changes must stay auditable and repairable from durable evidence rather than becoming hidden truth.

## REM-style linking and emotional processing contract
- REM-style work runs after the migration pass on a bounded emotional or novelty queue rather than over the full corpus.
- The pass may gradually reduce emotional arousal or bypass-decay pressure so emotionally heavy memories become less dominant without erasing their content.
- The pass may add auditable cross-links between distant but meaningfully related memories or engrams, supporting later associative recall and offline synthesis work.
- Any summary, relation, or synthesis artifact emitted by this pass must remain derived, inspectable, and lineage-backed; REM-style linking must not invent sole authoritative truth.
- Cross-linking and synthesis remain budgeted by explicit node, depth, sibling, and candidate caps so the background path does not reopen the full corpus or monopolize the online system.

## Homeostasis contract
- Homeostasis runs last as the saturation-control pass for the hot route and may no-op when post-NREM and post-REM load is already acceptable.
- It may bulk downscale hot-route strengths to restore signal-to-noise ratio and prevent saturation.
- After downscaling, only policy-eligible weak memories may be archived, and that archival must be explicit, recoverable, and reason-coded rather than silent deletion.
- Homeostasis must never prune pinned memories, legal-hold data, or the last authoritative evidence required to explain a memory, summary chain, or conflict state.
- Homeostasis is not a privacy/compliance delete path and must not bypass retention, governance, or namespace rules.

## Episode formation criteria
Events may be grouped into one episode when they share:
- task id
- session id
- goal context
- time proximity
- entity overlap
- tool chain continuity
- failure-retry continuity

## Safety rules
- preserve lineage long enough to explain every summary, extract, link, and archive decision
- prefer colder migration and compression before archival deletion semantics
- keep background work bounded so encode, recall, and repairable truth reads remain available
- treat graph links, caches, sidecars, and synthesis outputs as derived artifacts unless policy explicitly promotes them
- surface explicit loss or tombstone records instead of inventing missing truth when fidelity cannot be preserved
