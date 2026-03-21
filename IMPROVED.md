# IMPROVED.md — Human-Like Memory Gap Closure Plan

> Status: advisory improvement overlay
> Canonical authority remains `docs/PLAN.md` plus subsystem docs.
> This document is intentionally ambitious and detailed. It exists to help future planning, not to silently override the current canon.

---

## 1. Why this document exists

`membrain` is already pointed in the right direction for **durable**, **large-scale**, and **brain-inspired** memory. It has strong canonical commitments around:

- bounded foreground work
- provenance and lineage
- explicit contradiction handling
- rebuildable graph/index state
- tiered retrieval and storage
- maintenance-class consolidation and forgetting

Those are exactly the right foundations for a serious AI memory system.

What is still missing is not “basic memory infrastructure.” The bigger gap is the set of capabilities that make memory feel more **human-like** rather than only **durable and inspectable**.

In practical terms, the repo is already moving toward:

- **remember long**
- **remember a lot**
- **retrieve with association and conflict awareness**

But it is less mature in:

- **remembering with a live active workspace**
- **remembering in a self-relevant, autobiographical way**
- **remembering procedures and habits**
- **letting recall reshape future recall in richer ways**
- **using emotion, source trust, and social context as first-class controllers**
- **building schemas, gist, and constructive recall without violating the repo’s safety constraints**

This document describes a comprehensive improvement plan for those missing layers.

---

## 2. Reading this document correctly

This plan is subordinate to the current repository canon.

That means every improvement here must preserve the current architectural thesis:

1. foreground work stays bounded and measurable
2. provenance and lineage stay first-class
3. explainability remains mandatory
4. repairability outranks convenience
5. contradictions remain represented, not erased
6. governance and namespace checks happen before expensive work
7. brain-inspired mechanisms become canonical only when bounded, explainable, and benchmarked

If any proposed improvement conflicts with those rules, the canonical rules win.

This document therefore aims for **human-like behavior under production constraints**, not literal brain emulation.

---

## 3. Current strengths to preserve

Before changing anything, preserve what the current design already gets right.

### 3.1 Durable truth and rebuild authority
The repo correctly treats canonical durable rows as authority and caches / ANN / graph accelerators as rebuildable derived state.

This is essential. Human-like additions must not weaken it.

### 3.2 Tiered memory architecture
The hot / warm / cold split is the right backbone for “remember a lot” without turning the whole system into one undifferentiated store.

### 3.3 Bounded graph-assisted recall
The current graph and engram approach is promising because it supports association without allowing unbounded traversal.

### 3.4 Explicit contradiction preservation
Human-like memory is messy, but production systems must not collapse disagreement into one silent answer. The current contradiction contract is a major strength and must remain intact.

### 3.5 Maintenance-class consolidation and forgetting
Treating consolidation, forgetting, repair, and dream-like synthesis as bounded background work is the right production posture.

---

## 4. Main missing capability families

The biggest remaining gaps are these:

1. **Working memory / conscious workspace**
2. **Autobiographical spine and self-model**
3. **Procedural memory and habit formation**
4. **Richer reconsolidation and interference dynamics**
5. **Emotion as controller, not just metadata**
6. **Social memory and dynamic source trust**
7. **Constructive recall, gist, and schema memory**
8. **Offline dream / synthesis maturity**
9. **Human-like evaluation and promotion gates**

The rest of this document turns those gaps into an implementation roadmap.

---

## 5. North-star outcome

The goal is **not** “a system that merely stores more memory rows.”

The goal is a system that can increasingly behave like this while staying within the repo’s canonical restrictions:

- keep a small active workspace of what matters right now
- remember important things longer because they matter to the current self, goals, and history
- form reusable procedures from repeated episodes
- let recall reopen and reshape future recall in a controlled way
- use source trust, conflict state, and social context during recall
- build gist and schemas without deleting the underlying evidence
- reorganize memory offline during bounded maintenance windows
- prove these behaviors with benchmarks and deterministic tests

---

## 6. Non-goals and anti-goals

The project should **not** do the following in pursuit of “more human-like” behavior:

- no hidden full-corpus scans on request paths
- no magical persona variable that silently overrides evidence
- no emotional bias that bypasses policy or namespace boundaries
- no dream or synthesis path that becomes authoritative truth by accident
- no procedural extraction that deletes source episodes
- no constructive recall that turns into untraceable hallucination
- no self-model that becomes a second identity system detached from canonical memory handles
- no source-trust score that collapses contradiction or provenance into one opaque scalar

In short: the project must become **more cognitively rich without becoming less inspectable**.

---

## 7. Delivery strategy

Use a staged roadmap.

Do **not** try to build every human-like feature at once.

Recommended order:

1. Foundation instrumentation and contracts
2. Working memory and cognitive blackboard
3. Procedural memory and habit extraction
4. Autobiographical memory and self-model
5. Reconsolidation and interference v2
6. Emotion and state-dependent recall v2
7. Social memory and source trust
8. Schema / gist / constructive recall
9. Dream / offline synthesis maturity
10. Human-like evaluation and promotion gates

This order is deliberate:

- it strengthens the active-work and learning loops first
- it delays the riskiest “soft cognition” features until the scaffolding is measurable
- it aligns with the repo’s existing preference for bounded, evidence-backed promotion

---

# 8. Stage 0 — Foundation instrumentation and contract hardening

## Objective
Before adding richer cognition-like behavior, make sure the system can measure and explain it.

## Why this comes first
Most human-like features fail not because the idea is wrong, but because there is no clean way to prove whether they improved anything.

## Deliverables

### 8.0.1 Add a “human-like memory gap” design note
Create a canonical design note describing:

- which cognitive behaviors are desired
- which ones are currently only aspirational
- which ones remain later-stage or optional
- which ones must never bypass current invariants

### 8.0.2 Add explicit evaluation dimensions
Introduce a stable vocabulary for these dimensions:

- persistence quality
- active-work continuity
- autobiographical continuity
- procedural reuse
- interference resilience
- source-trust calibration
- schema quality
- emotional-state influence
- dream-synthesis utility

### 8.0.3 Extend observability
Add metrics / traces for:

- working-memory occupancy and eviction reasons
- rehearsal count and focus changes
- autobiographical thread hits
- self-relevance contribution to ranking
- procedural-memory retrieval and success reuse
- reconsolidation reopen / accepted update / rejected update counts
- interference penalties applied
- trust adjustments per source family
- mood-congruent ranking participation
- schema-hit versus verbatim-hit breakdown
- dream candidate counts, accepted links, rejected links

### 8.0.4 Add benchmark harness skeletons
Create benchmark families for:

- long-horizon retention
- repeated-task procedural improvement
- autobiographical continuity under time gaps
- conflict-heavy recall quality
- source-trust calibration
- emotional salience persistence
- schema compression utility

## Required docs to update later
- `docs/BENCHMARKS.md`
- `docs/TEST_STRATEGY.md`
- `docs/OPERATIONS.md`
- `docs/ARCHITECTURE.md`

## Acceptance gate
Do not promote later stages until there is a stable benchmark and observability vocabulary for them.

---

# 9. Stage 1 — Working memory and cognitive blackboard

## Objective
Turn the current working-memory idea into a truly useful, bounded active workspace.

## Problem
The repo already acknowledges working memory, attention, and bounded active state. But today the system is still much stronger at **durable memory** than at **live thought-state management**.

A more human-like system needs a strong answer to:

- what is the system actively holding in mind right now?
- which evidence is pinned versus merely available?
- what gets rehearsed, dropped, or re-focused?
- how does a long-running goal survive pause/resume without re-deriving everything?

## Target behavior
Working memory should become an explicit controller layer that:

- keeps a small active set of evidence, goals, and constraints
- supports attention competition and explicit focus changes
- distinguishes pinned evidence from ambient activation
- supports task-local rehearsal and repeated access
- feeds better encode and recall decisions
- remains non-authoritative compared with durable truth

## Detailed implementation plan

### 9.1 Data model
Introduce a controller-level working-state model with:

- slot id
- occupant type (`memory_handle`, `goal_handle`, `constraint_handle`, `context_fragment`, `scratch_binding`)
- source of activation (`recall`, `goal_resume`, `explicit pin`, `operator inject`, `recent encode`, `task context`)
- activation strength
- recency
- focus boost
- rehearsal count
- pin flag
- expiration or decay deadline in logical ticks
- task / session / goal binding
- namespace binding

Important:
- this should remain controller state, not durable memory authority
- checkpoints may persist resumability metadata, but that checkpoint must remain visibly different from canonical memory truth

### 9.2 Core behaviors
Implement:

- `focus(handle)`
- `pin(handle)` / `unpin(handle)`
- `rehearse(handle)`
- `drop(handle)`
- `replace(slot)` with deterministic eviction reasons
- task-resume restoration from bounded checkpoints
- explicit occupancy caps and per-type caps

### 9.3 Retrieval integration
Retrieval should be able to use working-state signals as bounded ranking inputs:

- active-goal match
- pinned-evidence reinforcement
- current-task relevance
- current-session continuity

But these must remain additive ranking families, not policy bypasses or new full scans.

### 9.4 Encode integration
Working memory should affect encode by:

- improving task relevance signals
- enriching context binding
- supporting “keep hot” decisions for likely-immediate reuse
- enabling deliberate discard of low-value attended material before persistence

### 9.5 CLI / MCP surface
Potential user-visible surfaces:

- `membrain wm show`
- `membrain wm focus <id>`
- `membrain wm pin <id>`
- `membrain wm drop <id>`
- `membrain wm checkpoint show`

MCP / daemon should expose a machine-readable working-state object with explicit redaction / omission markers.

### 9.6 Observability
Expose:

- occupancy
- pin count
- mean dwell time
- eviction reasons (`capacity`, `stale`, `goal switch`, `explicit drop`, `policy`, `checkpoint restore conflict`)
- replay success rate on goal resume

### 9.7 Tests
Add deterministic tests for:

- 7±2 bounded occupancy behavior
- pinned evidence surviving ordinary churn
- unpinned items being evicted deterministically
- checkpoint save/resume fidelity
- no namespace leakage through checkpoint restore
- no silent promotion of controller state into durable truth

## Promotion gate
Working memory is mature enough when:

- long-running tasks resume with measurably less reconstruction
- active evidence handling is inspectable
- eviction and focus changes are deterministic
- no working-memory feature widens request-path cost beyond declared budgets

---

# 10. Stage 2 — Procedural memory and habit formation

## Objective
Move beyond episodic and semantic storage into “how to do this again” memory.

## Problem
The repo already names procedural memory and skill extraction as later-stage work, but that layer is critical for any system that wants to feel like it learns from experience rather than only stores facts.

Without procedural memory, the system may remember incidents but not improve at repeated tasks.

## Target behavior
The system should be able to:

- detect repeated successful episode patterns
- extract candidate procedures from them
- keep those procedures lineage-backed and tentative at first
- promote them into reliable procedural memory only through explicit criteria
- retrieve procedures efficiently for relevant tasks
- degrade or supersede stale procedures without deleting source episodes

## Detailed implementation plan

### 10.1 Candidate procedure extraction
Use bounded background extraction from:

- repeated successful tool chains
- repeated successful goal-completion clusters
- mature engrams with stable member patterns
- low-conflict, high-confidence episode sequences

### 10.2 Proposed procedural artifact model
Add a derived artifact family with fields such as:

- procedure id
- source episode or engram handles
- trigger pattern
- ordered step template
- preconditions
- expected outcome
- success rate
- confidence / uncertainty
- freshness
- superseded_by / replaced_by
- acceptance state (`derived`, `reviewed`, `accepted`, `deprecated`)

### 10.3 Promotion path
Do **not** let repeated use silently promote tentative procedures.

Instead, require explicit promotion criteria such as:

- repeated successful reuse across contexts
- no unresolved contradiction with stronger evidence
- acceptable failure rate
- bounded drift from source pattern
- intact lineage to supporting episodes

### 10.4 Retrieval integration
Procedures should become a distinct retrieval or ranking family when the query intent is procedural:

- “how do I do X?”
- “what worked last time?”
- “what is the safest known playbook?”

This must stay explainable:

- which episodes or clusters justified the procedure
- whether it is accepted or tentative
- what assumptions or prerequisites it carries

### 10.5 Forgetting and supersession
Add lifecycle rules for:

- procedure aging when it repeatedly fails
- deprecating a procedure when the environment changes
- linking stale procedures to new replacements
- preserving old procedures for audit instead of deleting them silently

### 10.6 CLI / MCP surface
Potential surfaces:

- `membrain skills`
- `membrain skills --extract`
- `membrain procedure inspect <id>`
- `membrain procedure deprecate <id>`
- `procedures()` / `extract_skills()` MCP outputs

### 10.7 Tests
Need coverage for:

- deterministic candidate extraction
- lineage preservation
- non-promotion on ambiguous evidence
- retrieval preference for accepted procedures over tentative ones
- proper supersession of outdated procedures
- no hidden loss of source episodic evidence

## Promotion gate
Procedural memory is mature enough when:

- repeated tasks complete faster or with fewer misses after prior experience
- extracted procedures remain inspectable and lineage-backed
- stale procedures can be deprecated without semantic corruption

---

# 11. Stage 3 — Autobiographical memory and self-model

## Objective
Give the system a stable autobiographical spine and self-relevance layer.

## Problem
The current design has strong provenance, goals, session bindings, and task relevance. But it does not yet appear to have a mature **self-model** or autobiographical narrative surface.

Without that layer, the system can remember many things without strongly understanding:

- which memories matter to “me” across time
- which commitments and roles persist
- which recurring themes define long-horizon identity
- which memories are self-relevant versus merely adjacent facts

## Key principle
Do **not** build a mystical persona blob.

A proper self-model in this repo should be:

- derived
- inspectable
- lineage-backed
- revisable
- conflict-aware
- explicitly subordinate to raw evidence

## Target behavior
The system should be able to:

- maintain autobiographical threads over long spans
- track recurring roles, commitments, constraints, and preferences
- weight self-relevance during ranking without hiding stronger contrary evidence
- preserve identity continuity across sessions and goals
- expose what currently forms the self-model and why

## Detailed implementation plan

### 11.1 Autobiographical thread layer
Introduce a derived thread abstraction that groups memories by:

- same long-horizon goal family
- same role or commitment
- repeated self-referential patterns
- enduring preference beliefs
- stable project / workspace / agent identity themes

Each thread should preserve:

- member handles
- start / end or active interval
- stability score
- central theme label
- role / preference / goal relevance
- conflict / supersession markers
- lineage back to supporting memories

### 11.2 Identity-facet model
Instead of one monolithic persona, represent multiple identity facets such as:

- role beliefs
- long-term commitments
- recurring preferences
- standing constraints
- stable working styles
- important recurring relationships

All facets should be:

- inspectable
- conflict-aware
- revisable under contradiction
- explicitly linked to source evidence

### 11.3 Retrieval integration
Add a small bounded ranking family for:

- self relevance
- autobiographical continuity
- ongoing role / goal fit
- stable preference relevance

This must never:

- outrank materially stronger direct evidence by itself
- erase contradictions
- become a hidden policy or trust shortcut

### 11.4 Relation to belief versioning
Autobiographical memory should integrate cleanly with belief versioning and contradiction surfaces.

When self-beliefs change:

- preserve older self-beliefs as inspectable history
- do not mutate the past in place
- allow current operative self-model to differ from older versions without erasing them

### 11.5 CLI / MCP surface
Potential surfaces:

- `membrain self show`
- `membrain self threads`
- `membrain self preferences`
- `membrain self history`

These should always show lineage, confidence, conflict state, and change conditions.

### 11.6 Tests
Need coverage for:

- stable thread formation
- no second-identity-system drift
- explicit conflict handling between self facets
- preservation of older self-beliefs after revision
- retrieval ranking that uses self relevance only as bounded additive support

## Promotion gate
Autobiographical memory is mature enough when:

- long-span retrieval can explain why certain memories matter to the current self or role
- preference / commitment continuity becomes measurably more stable across sessions
- self-model drift remains inspectable rather than hidden

---

# 12. Stage 4 — Reconsolidation and interference v2

## Objective
Make recall change future recall in a richer, more human-like way without sacrificing truth and auditability.

## Problem
The repo already has reconsolidation and interference concepts. The gap is that they still appear closer to a safe lifecycle/update contract than to a rich memory-dynamics engine.

Human-like memory is not just:

- recall
- reopen
- update
- restabilize

It is also:

- reinterpretation under new context
- selective strengthening and weakening
- competition between similar memories
- retrieval difficulty shaped by recent activity
- memory change because it was revisited from a new perspective

## Target behavior
The system should support richer, explicit reconsolidation outcomes such as:

- reinforcement
- correction
- contextual reinterpretation
- confidence reweighting
- contradiction minting when a reopened belief conflicts with new evidence
- link reinforcement or weakening among related memories

## Detailed implementation plan

### 12.1 Add reconsolidation outcome classes
Extend reconsolidation beyond “pending update yes/no” toward explicit outcome types:

- `reinforce`
- `correct`
- `reinterpret`
- `attach_context`
- `raise_uncertainty`
- `lower_uncertainty`
- `mint_contradiction`
- `supersede_with_authority`

### 12.2 Add interference state surfaces
Track more than a one-off penalty. Add inspectable interference fields such as:

- competing memory handles
- interference family (`retroactive`, `proactive`, `retrieval competition`, `confusable duplicate-neighbor`)
- strength of competition
- last competition event
- retrieval difficulty adjustment

### 12.3 Retrieval integration
Use interference and reconsolidation as bounded ranking / recall modifiers:

- lower confidence for easily confusable memories
- surface competition explicitly in explain / inspect
- preserve conflict siblings and alternatives instead of collapsing them

### 12.4 Maintenance integration
Background passes should be able to:

- revisit unstable conflict neighborhoods
n- refresh embeddings after accepted reinterpretation
- harden or split interference-prone engrams
- lower or raise uncertainty as corroboration changes

### 12.5 Tests
Need deterministic coverage for:

- reopen-window behavior
- no silent overwrite during reinterpretation
- contradiction minting on genuine conflict
- preserved lineage across updated or reweighted beliefs
- interference penalties that do not corrupt duplicate detection or policy behavior

## Promotion gate
This stage is mature when the system can explain not only **what it remembers**, but **how recent recall changed what becomes easy or hard to remember next**.

---

# 13. Stage 5 — Emotion and state-dependent memory v2

## Objective
Promote emotion from simple tags into a bounded controller input.

## Problem
The repo already has valence / arousal, bypass-decay ideas, and optional mood-congruent retrieval. That is a solid base, but still shallow compared with how emotion shapes human memory.

Human-like memory uses emotion to affect:

- what is encoded at all
- what stays salient
- what returns under pressure
- what needs desensitization over time
- what gets associated with similar contexts later

## Key constraint
Emotion must never become:

- a policy bypass
- a hidden default scope widener
- an excuse to override stronger direct evidence
- an opaque ranking black box

## Target behavior
The system should support:

- stronger encode-side emotional control
- recall-time state snapshots
- emotional desensitization over time
- state-dependent retrieval as optional, inspectable behavior
- emotional carryover in autobiographical and procedural layers

## Detailed implementation plan

### 13.1 Expand emotional state model
Track:

- encode-time emotion
- current bounded mood snapshot
- recent mood trajectory
- desensitization state
- emotional trigger categories
- arousal half-life or dampening history

### 13.2 Encode behavior improvements
Use emotion to affect:

- attention gate threshold
- initial salience
- provisional retention priority
- likelihood of staying hot after encode

### 13.3 Retrieval behavior improvements
Allow emotion to act only as a bounded additive family:

- mood-congruent boost for already-eligible candidates
- recall of emotionally salient autobiographical threads
- preference for emotionally central events during explicit reflection / introspection modes

### 13.4 Maintenance behavior improvements
Add a REM-style desensitization and cross-link pass that:

- reduces over-dominance of high-arousal items
- creates explainable cross-links between emotionally related experiences
- keeps traumatic or critical memories inspectable without forcing them to dominate ordinary recall forever

### 13.5 Tests
Need coverage for:

- encode salience changes under emotion
- bounded mood-congruent ranking effects
- desensitization over logical time
- no policy or namespace widening through emotional state
- no permanent over-dominance of a single high-arousal memory family

## Promotion gate
This stage is mature when emotional metadata measurably improves encode / recall usefulness without becoming a hidden global bias engine.

---

# 14. Stage 6 — Social memory and dynamic source trust

## Objective
Give the system richer source, relationship, and trust dynamics.

## Problem
The repo already has provenance, authoritativeness, contradiction state, and actor handles. But that is not yet the same as social memory.

Human-like memory often depends on:

- who said it
- whether that source has been right before
- whether that source is reliable for this topic but not another
- what relationship exists between the current self, goals, and that source

## Key principle
Do not collapse trust into one opaque source score.

Trust must remain:

- topic-aware when needed
- bounded
- explainable
- revisable
- subordinate to provenance and contradiction evidence

## Target behavior
The system should support:

- source-specific trust history
- domain- or topic-specific trust calibration
- memory of relationship context
- social/interaction threads
- trust-aware retrieval and packaging

## Detailed implementation plan

### 14.1 Trust model
Add a trust artifact or source-calibration layer with fields such as:

- source handle
- scope (`global`, `topic`, `goal family`, `relation family`)
- corroboration history
- contradiction history
- recency of evidence
- confidence / uncertainty
- last adjustment reason

### 14.2 Relationship memory
Add durable or derived relationship summaries for:

- repeated collaborator patterns
- source-role relationships
- user preference stability by source
- disagreement patterns

### 14.3 Retrieval integration
Use trust only as a bounded ranking family:

- help order equally plausible evidence
- help explain why one source was preferred
- never erase lower-trust contradictory evidence when it remains relevant

### 14.4 CLI / MCP surface
Potential surfaces:

- `membrain source inspect <id>`
- `membrain trust show <source>`
- `membrain social thread <id>`
- `source_trust()` MCP surfaces

### 14.5 Tests
Need coverage for:

- trust updates from corroboration and contradiction
- topic-specific trust not leaking into unrelated topics
- packaging that preserves both preferred and losing evidence when contradictions remain open
- no hidden trust-based policy shortcuts

## Promotion gate
This stage is mature when the system can explain not just “what evidence exists,” but “why this source was weighted as more or less reliable here.”

---

# 15. Stage 7 — Schema memory, gist, and constructive recall

## Objective
Move from pure item recall toward structured abstraction without sacrificing auditability.

## Problem
Human memory is not a verbatim database. It remembers:

- gist
- patterns
- typical situations
- scripts
- abstractions

The repo already has later-stage schema compression and summary-oriented work, but not yet a fully mature “constructive recall” layer.

## Key principle
Constructive recall must never delete or hide the underlying evidence.

## Target behavior
The system should support:

- schema memories for repeated situations
- gist versus verbatim distinction
- explainable constructive recall
- explicit uncertainty and “change my mind” conditions for abstractions
- bounded, inspectable packaging that distinguishes direct evidence from schema-level inference

## Detailed implementation plan

### 15.1 Schema artifact model
Add or strengthen durable/derived schema memory artifacts with:

- schema id
- source episode handles
- source engram handles
- common pattern description
- confidence
- uncertainty components
- known / assumed / uncertain / missing structure
- triggering contexts
- applicability limits
- supersession lineage

### 15.2 Retrieval and packaging
Allow retrieval to return mixed packages such as:

- direct evidence items
- supporting graph-expanded context
- schema / gist overlay
- explicit omissions
- uncertainty bounds

Each must remain distinguishable.

### 15.3 Conflict handling
Schema memories must not flatten contradiction.

If the source episodes disagree, the schema should either:

- preserve disagreement explicitly
- split into multiple schemas
- stay tentative rather than pretending there is one stable abstraction

### 15.4 Tests
Need coverage for:

- schema formation from repeated episodes
- lineage to source episodes
- uncertainty surfaces for schemas
- conflict-aware abstraction behavior
- no destructive replacement of source evidence

## Promotion gate
This stage is mature when the system can say:

- “here is the usual pattern”
- “here is the direct evidence behind it”
- “here is where the pattern may not apply”

without becoming an opaque summarizer.

---

# 16. Stage 8 — Dream mode and offline synthesis maturity

## Objective
Make offline associative restructuring useful, bounded, and measurable.

## Problem
The repo already frames dream mode correctly as optional, bounded, background-only synthesis. The missing step is turning it into a mature, clearly useful subsystem.

## Target behavior
Dream-like maintenance should be able to:

- propose distant but meaningful links
- identify split/merge candidates for engrams
- suggest schema formation opportunities
- reinforce multi-hop autobiographical associations
- surface tentative hypotheses without silently promoting them to truth

## Detailed implementation plan

### 16.1 Candidate generation
Dream mode may operate over bounded candidates selected from:

- emotionally strong queues
- unresolved but non-contradictory related clusters
- repeated co-activation patterns
- under-linked autobiographical or procedural neighborhoods

### 16.2 Output families
Dream mode should be allowed to emit only bounded, inspectable outputs such as:

- tentative link proposals
- candidate schema suggestions
- cluster split / merge suggestions
- autobiographical bridge suggestions
- relation reinforcement suggestions

### 16.3 Acceptance policy
Not every dream result should auto-commit.

Use acceptance classes such as:

- `auto-safe`
- `requires-review`
- `tentative-only`
- `rejected`

with clear reasons.

### 16.4 Observability
Expose:

- candidate counts
- accepted link counts
- rejected link counts
- false-positive review rate
- maintenance cost
- foreground latency delta during or after runs

### 16.5 Tests
Need coverage for:

- bounded candidate generation
- namespace and policy preservation
- no unbounded graph fanout
- no silent authority promotion
- stable replay under restart / interruption

## Promotion gate
Dream mode is mature when it adds measurable recall or abstraction value without creating opaque, noisy, or trust-damaging synthetic structure.

---

# 17. Stage 9 — Human-like evaluation framework

## Objective
Define how the project will know whether it is actually becoming more human-like in useful ways.

## Problem
Without explicit evaluation, “more human-like” will degrade into vibes and anecdote.

## Evaluation dimensions

### 17.1 Persistence quality
Can the system retain important items over long logical horizons while demoting unimportant noise?

### 17.2 Active-work continuity
Can the system pause and resume complex work with less reconstruction?

### 17.3 Procedural reuse
Does prior successful experience improve future task execution?

### 17.4 Autobiographical continuity
Does the system preserve meaningful identity / preference / commitment continuity over time?

### 17.5 Conflict integrity
Can the system remain coherent without deleting disagreement?

### 17.6 Interference realism
Do similar memories compete in plausible, inspectable ways without corrupting truth?

### 17.7 Emotional usefulness
Does emotional state help prioritize important memories without creating hidden bias or noise?

### 17.8 Social trust calibration
Does the system learn which sources are reliable, and is that calibration topic-aware and explainable?

### 17.9 Schema usefulness
Do abstractions improve downstream usefulness while preserving direct evidence and uncertainty?

## Benchmark families to add

1. **Longitudinal retention benchmark**
2. **Repeated-task skill extraction benchmark**
3. **Pause/resume goal checkpoint benchmark**
4. **Contradiction-heavy recall benchmark**
5. **Source trust calibration benchmark**
6. **Emotion-assisted prioritization benchmark**
7. **Schema compression utility benchmark**
8. **Dream-link usefulness benchmark**

## Promotion discipline
No human-like feature should become core simply because it looks cognitively interesting.

Promotion should require:

- measurable utility
- bounded cost
- explainability
- replayable behavior
- no conflict with canonical invariants

---

# 18. Cross-cutting schema and API changes likely needed

This section groups the most likely structural changes needed across multiple stages.

## 18.1 Schema families likely to grow

Potential additions or extensions:

- working-state / blackboard checkpoints
- autobiographical thread records
- identity facet / self-belief records
- procedural memory artifacts
- trust calibration records
- richer reconsolidation artifacts
- emotional trajectory / desensitization records
- schema-memory artifacts
- dream proposal / acceptance records

Important rule:
all of these must remain clearly classified as either:

- authoritative durable truth
- or derived, lineage-backed artifacts

Never blur the line.

## 18.2 RetrievalResult growth
Future retrieval payloads likely need richer explicit structure for:

- direct evidence
- graph-supported context
- schema/gist overlays
- self-relevance signals
- trust contribution
- emotional contribution
- uncertainty and change-my-mind conditions

## 18.3 CLI / MCP contract growth
New surfaces should probably be grouped into these families:

- working state
- self / autobiographical inspect
- procedures / skills
- trust / source inspect
- schema inspect
- dream status / inspect

---

# 19. Suggested documentation additions

This plan does **not** require creating all of these immediately, but these docs would likely help if the work is pursued:

- `docs/WORKING_MEMORY.md`
- `docs/SELF_MODEL.md`
- `docs/PROCEDURAL_MEMORY.md`
- `docs/SOCIAL_MEMORY.md`
- `docs/SCHEMA_MEMORY.md`
- `docs/DREAM_MODE.md`
- `docs/HUMAN_LIKE_EVALUATION.md`

Each should remain subordinate to `docs/PLAN.md` and align with subsystem docs.

---

# 20. Suggested bead / epic breakdown

If this plan is later converted into beads, a sensible decomposition would be:

1. **Foundation metrics and observability for advanced memory behaviors**
2. **Working-memory controller and blackboard checkpoints**
3. **Working-memory ranking integration and tests**
4. **Procedural extraction candidate pipeline**
5. **Procedure promotion / supersession / inspect surfaces**
6. **Autobiographical thread formation**
7. **Self-model and preference-belief inspectability**
8. **Reconsolidation outcome taxonomy and replay safety**
9. **Interference-state modeling and retrieval integration**
10. **Emotion controller v2 and desensitization surfaces**
11. **Source trust calibration model**
12. **Relationship/social thread memory**
13. **Schema-memory formation and uncertainty surfaces**
14. **Dream-mode proposal engine and acceptance policy**
15. **Human-like benchmark suite and promotion gates**

---

# 21. Recommended execution order

If only a subset of this plan will be built, use this priority order:

## Highest-leverage near-term work
1. working memory / blackboard
2. procedural memory
3. autobiographical threading
4. reconsolidation / interference v2

## Next wave
5. source trust and social memory
6. emotion controller v2
7. schema/gist memory

## Last wave
8. dream-mode maturity
9. strong human-like evaluation suite

Reasoning:
- working memory and procedures improve usefulness fastest
- autobiographical continuity gives identity-level persistence
- reconsolidation/interference deepen memory dynamics
- social, emotional, and dream features are powerful but riskier

---

# 22. Hard questions to resolve before adopting this plan

These are the most important open questions.

1. How much of self-model should be durable truth versus derived summary?
2. What is the safest promotion path from extracted procedure to accepted procedure?
3. How much emotional influence is useful before it becomes noise?
4. How should trust be topic-scoped without exploding complexity?
5. How should schema formation remain useful without hiding contradiction?
6. Which dream outputs, if any, are safe for auto-accept?
7. What benchmark threshold is strong enough to justify promoting these features from optional to canonical?

These questions should be answered explicitly before the repo adopts major parts of this overlay.

---

# 23. Bottom-line recommendation

The project is already on the right path for **durable, scalable, explainable AI memory**.

To move closer to **human-like memory**, the next major push should be:

1. strengthen active working-state handling
2. turn repeated successful experience into procedural memory
3. build autobiographical and self-relevant continuity
4. deepen reconsolidation / interference / emotional dynamics
5. add social trust, schema, and dream maturity only after the earlier layers are benchmarked

That path preserves the repo’s current strengths instead of abandoning them.

The correct target is not “be messy like a human brain.”
The correct target is:

> **be more human-like where it improves usefulness, while staying bounded, inspectable, repairable, and benchmarkable.**

---

## Appendix A — One-sentence summary of each missing capability

- **Working memory:** the repo needs a stronger answer to “what is active in mind right now?”
- **Procedural memory:** the repo needs to remember how to do things, not just what happened.
- **Autobiographical continuity:** the repo needs long-horizon self-relevance and identity-thread memory.
- **Reconsolidation v2:** the repo needs richer explicit ways for recall to reshape future recall.
- **Interference:** the repo needs more realistic competition between similar memories.
- **Emotion v2:** the repo needs emotion to act as a bounded controller, not only a tag.
- **Source trust:** the repo needs to learn who is reliable and in what context.
- **Schema/gist:** the repo needs abstraction without deleting evidence.
- **Dream maturity:** the repo needs bounded offline synthesis that is useful and safe.
- **Evaluation:** the repo needs a first-class framework to prove these features are genuinely improving memory quality.
