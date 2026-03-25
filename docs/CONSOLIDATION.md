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
- scheduler drains must stay bounded by explicit queue-job and candidate budgets; queue-budget boundaries should emit partial-run artifacts instead of silently continuing in the background
- partial derivation failures may consume only controller-owned retry budget, and once that budget is exhausted the run must leave durable truth authoritative and surface an operator-visible escalation artifact
- interrupted runs should emit a resumable interruption artifact naming the namespace scope, preserved durable checkpoint, pending work count, and whether automatic resume is still allowed

## Skill extraction follow-on contract
- Skill extraction is a later-stage, non-blocking consolidation follow-on that distills repeated successful episodes or mature engrams into tentative procedural artifacts; core consolidation, compaction, and repair correctness must not depend on it.
- Run it only from a bounded eligible set whose namespace, policy, lineage, and lifecycle eligibility are already known; it must not scan the full corpus or widen scope across namespaces just to find candidates.
- Extraction may be triggered by `membrain skills --extract`, `membrain engram <uuid> --extract`, or a separately budgeted idle-window maintenance pass. It is not a mandatory step in every consolidation cycle or compaction window.
- The extraction path stays background-only and explainable. It may use bounded cluster, keyword, or centroid heuristics, but it must not put unbounded synthesis or an LLM dependency on a foreground recall path.
- Extracted skills remain derived durable artifacts until an explicit acceptance path promotes them into authoritative procedural state. Durability or repeated reuse alone must not silently promote them.
- Every extracted procedure must preserve lineage to the source engram or member set, record confidence or tentative status, and keep the underlying episodic or semantic evidence available for inspect, audit, and repair.
- Reflection-compiler follow-ons may reinterpret the same bounded episode set as advisory `procedure`, `anti_pattern`, or checklist-style guidance, but those artifacts remain derived and non-authoritative until a separate acceptance path promotes them.
- The stored artifact should be reviewable and recallable without reparsing opaque content alone: operator surfaces need explicit storage/review/recall metadata such as derivation rule, review status, supporting-evidence counts, bounded recall cues/seed handles, and any reflection metadata needed to show advisory release rules.
- Interface hooks should remain semantically aligned across `membrain skills`, `membrain skills --extract`, `membrain engram <uuid> --extract`, `skills()`, and `extract_skills()` rather than inventing separate meanings per surface.
- Future validation should prove bounded background cost, deterministic candidate selection or rejection, preserved source evidence, explicit non-promotion on ambiguity, and namespace or policy enforcement for every extracted artifact.
- Later follow-ons such as schema compression or quality-loop automation may consume extracted skills, but they must treat this surface as gated background output rather than a prerequisite for core-path behavior.

## Quality-loop follow-on contract
- Quality-loop follow-ons are Phase 4 maturity work: they use benchmark feedback, maintenance outcomes, and extracted-skill signals to improve operator guidance or later background tuning after the core retrieval, governance, and repair spine is already proven.
- This loop remains explicitly non-blocking. It must not become a hidden prerequisite for phase promotion, request-path correctness, or ordinary consolidation success, and it must not reopen settled bounded-work or policy invariants.
- Inputs to the loop should come from existing auditable surfaces such as benchmark artifacts, maintenance reports, repair or compaction outcomes, and bounded skill-extraction outputs rather than from opaque heuristics or untraceable background state.
- Any automation derived from the loop stays background-only, namespace-aware, policy-aware, and bounded by explicit candidate or queue budgets. It may recommend or schedule follow-on work, but it must not silently mutate authoritative truth on the request path.
- Interface hooks should remain inspectable and semantically aligned across operator docs and machine-readable surfaces so contributors can tell which benchmark artifact, maintenance signal, or extracted-skill set triggered a later recommendation.
- Future validation should prove that the loop consumes rerunnable benchmark evidence, preserves lineage and policy semantics for any skill-backed recommendation, stays observably non-blocking under maintenance load, and degrades safely by emitting recommendations or queued follow-up work instead of forcing hidden mutations.

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

## Structured derivation contract
- Allowed structured derivations from one bounded source set are `Summary`, `Fact`, `Gist`, and `Relation`; later follow-ons such as skills or procedures stay on their own gated paths and must not be smuggled into core consolidation.
- Every derived fact or relation must keep the source-set fixture, namespace, source memory ids, per-source timestamps, and stable source citations so inspect and repair can trace the artifact back to durable evidence.
- Stable provenance handles should use deterministic `consolidation://{namespace}/{fixture}#{kind}` source refs for the derived artifact itself and stable per-source citations such as `memory://{namespace}/{memory_id}` for the underlying evidence.
- Facts remain derived semantic claims, not canonical replacements for the source evidence; they must preserve a path back to evidence and must not silently replace contradictory facts.
- Relations remain derived, inspectable links; they must carry enough confidence or status metadata for explain, contradiction handling, and repair to treat them as structured evidence rather than opaque annotations.
- Relation reinforcement is contradiction-aware: consolidation may only emit a reinforced relation when supporting evidence agrees on the bounded relation signal. In practice that means a bounded source set needs corroborating continuity beyond a lone primary grouping reason before a reinforced relation is emitted.
- If the source set is singleton or contradiction handling is unresolved, the run must preserve evidence and emit a partial derivation or no reinforced relation instead of fabricating consensus.
- Explain surfaces for structured derivations should expose at least the derivation rule (`episode_summary`, `fact_extraction`, `gist_compaction`, or `relation_reinforcement`), a confidence score, the supporting continuity fields, and whether the result is `complete`, `partial`, or `reinforced`.

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
