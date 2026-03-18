# membrain — Memory Model

> Canonical source: PLAN.md Sections 13 (Memory Model Extension), 14 (Lifecycle), 33 (Data Schema).
> Feature extensions: PLAN.md Sections 46–47.

## Memory Taxonomy

The system supports these memory categories:

| Type | Purpose |
|------|---------|
| **Event** | Raw observed occurrence, tool call, message, action, or state change |
| **Episode** | Grouped sequence of related events with temporal continuity |
| **Fact** | Distilled proposition intended for repeated recall |
| **Relation** | Link between entities, memories, goals, or concepts |
| **Summary** | Compressed representation of lower-level evidence |
| **Goal** | Active or historical objective shaping retrieval priority |
| **Skill** | Reusable procedural knowledge extracted from repeated success |
| **Constraint** | Rules, limits, or obligations that must remain visible |
| **Hypothesis** | Tentative belief awaiting confirmation |
| **ConflictRecord** | Explicit contradiction artifact |
| **PolicyArtifact** | Retention/governance/compliance-relevant item |
| **Observation** | State observation or environmental signal |
| **ToolOutcome** | Result of tool execution with operational value |
| **UserPreference** | Stable user-specific preference or convention |
| **SessionMarker** | Boundary and context marker for session-level grouping |

## Memory Kinds (Brain-Inspired Layer)

Orthogonal to the taxonomy above, the brain-inspired encoding pipeline uses:

| Kind | Description |
|------|-------------|
| `Episodic` | Experience-based, time-stamped, decays normally |
| `Semantic` | Distilled knowledge, slower decay, higher consolidation priority |
| `Procedural` | How-to knowledge, extracted from clusters, bypasses decay |
| `Schema` | Abstract pattern distilled from repeated episodes (Feature 17) |

### Canonical vocabulary rules

- Every persisted memory has exactly one canonical `memory_type` from the taxonomy above; downstream schema overlays, APIs, and explain surfaces should use that vocabulary consistently.
- Brain-inspired kinds are a separate classification axis from `memory_type`. A `Fact`, `Summary`, or `Goal` may still be encoded or treated as `Semantic`; an `Event` or `ToolOutcome` is commonly `Episodic`; a `Skill` is commonly `Procedural`.
- The canonical kind inventory is currently `Episodic`, `Semantic`, `Procedural`, and `Schema`.
- Historical snapshot prose in `PLAN.md` sometimes names `Emotional` as a kind. The canonical contract does **not** treat `Emotional` as a separate `MemoryKind`; emotional significance is represented through emotional-tag or trajectory metadata such as `encoding_valence`, `encoding_arousal`, salience, confidence effects, and decay modifiers.
- Kinds are intended to guide encoding, consolidation, retrieval, and forgetting behavior; they do not replace provenance, lineage, contradiction state, or policy metadata.
- Interfaces should reject or explicitly migrate unknown or legacy kind labels rather than silently accepting a divergent vocabulary.

## Core Fields

Every memory item carries these attributes (directly or derivably):

### Required Base

| Field | Type | Description |
|-------|------|-------------|
| `id` | UUID | Globally unique within namespace |
| `memory_type` | enum | One of the 15 types above |
| `namespace` | String | Always present before persistence |
| `created_at` | i64 | Creation timestamp |
| `updated_at` | i64 | Last mutation (`created_at <= updated_at`) |
| `version` | u64 | Increments on accepted mutation |

### Strongly Expected

| Field | Type | Description |
|-------|------|-------------|
| `workspace_id` | Option | Workspace identifier |
| `agent_id` | Option | Agent that created this memory |
| `session_id` | Option | Session context |
| `task_id` | Option | Task context |
| `source_kind` | Option | How it was created (cli, mcp, api, observe) |
| `source_ref` | Option | Reference to source |
| `authoritativeness` | f32 | Source reliability score |
| `content_ref` | Option | Reference to canonical content storage |
| `payload_ref` | Option | Reference to detachable full payload when content is stored out-of-line |
| `compact_text` | String | Human-readable summary that preserves the item's working identity |
| `fingerprint` | u64 | Duplicate-family hint, never a substitute for canonical identity |
| `tier` | enum | Current tier route (`Tier1`, `Tier2`, or `Tier3`); working-memory residency is separate controller state |
| `salience` | f32 | Current importance score |
| `confidence` | f32 | Reliability/certainty score (Feature 7) |
| `utility_estimate` | f32 | Predicted future usefulness |
| `recall_count` | u32 | Times successfully recalled |
| `last_access_at` | Option | Last surfaced-reuse audit timestamp; not the decay clock anchor |
| `retention_class` | enum | volatile, normal, durable, pinned |
| `decay_state` | struct | Logical-tick decay parameters such as `base_strength`, `stability`, `last_accessed_tick`, and `bypass_decay` |
| `policy_flags` | Vec | Active policy markers |
| `lineage` | Vec | Parent memory IDs |
| `tags` | Vec | User/agent-assigned tags |
| `entity_refs` | Vec | Referenced entities |
| `relation_refs` | Vec | Referenced relations |

### Feature-Specific Fields

| Field | Type | Feature | Description |
|-------|------|---------|-------------|
| `conflict_state` | enum | Core contradiction handling | `none`, `open`, `resolved`, `superseded`, or `authoritative_override` |
| `conflict_record_ids` | Vec<UUID> | Core contradiction handling | Linked contradiction artifacts that inspect/recall can surface without losing provenance |
| `superseded_by` | Option | F2 Belief Versioning | Points to newer version when a contradiction resolves as supersession |
| `belief_version` | u32 | F2 | Version number in belief chain |
| `belief_chain_id` | Option | F2 | Groups versions together |
| `is_landmark` | bool | F5 Landmarks | Temporal anchor |
| `landmark_label` | Option | F5 | Human-readable era label |
| `era_id` | Option | F5 | Which era this memory belongs to |
| `corroboration_count` | u32 | F7 Confidence | How many memories corroborate |
| `reconsolidation_count` | u32 | F7 | How many times reconsolidated |
| `engram_id` | Option<UUID> | Core associative graph | Primary stable cluster handle when the memory participates in bounded engram expansion |
| `distilled_from_engram` | Option | F8 Skill Extraction | Source engram for procedurals |
| `namespace_id` | String | F9 Cross-Agent | Namespace for sharing |
| `visibility` | enum | F9 | private, shared, public |
| `has_causal_parents` | bool | F11 Causal | Has causal provenance |
| `has_causal_children` | bool | F11 | Is source of derived beliefs |
| `compressed_into` | Option | F17 Compression | Schema memory this was compressed into |
| `encoding_valence` | Option | F18 Emotional | Mood at encoding time |
|.*| F18 Emotional | Mood at encoding time |/| `encoding_arousal` | Option | F18 Emotional | Mood at encoding time |
| `uncertainty_score` | f32 | F7 Uncertainty | Combined 0-1 measure from corroboration, freshness, conflict, and evidence sparsity |
| `corroboration_uncertainty` | f32 | F7 Uncertainty | Uncertainty from lack of supporting evidence |
| `freshness_uncertainty` | f32 | F7 Uncertainty | Uncertainty from temporal staleness |
| `contradiction_uncertainty` | f32 | F7 Uncertainty | Uncertainty from conflict state |
| `missing_evidence_uncertainty` | f32 | F7 Uncertainty | Uncertainty from sparse causal links or weak authoritativeness |
| `confidence_interval` | Option<UncertaintyBounds> | F7 Uncertainty | Confidence interval bounds for high-stakes paths |
| `encoding_arousal` | Option | F18 | Arousal at encoding time |
| `observation_source` | Option | F6 Observation | Source: stdin, file, watch |
| `observation_chunk_id` | Option | F6 Observation | Groups bounded observation fragments produced by one watch or batch ingestion pass |

## Identity and Version Rules

### Canonical identity contract

A memory's canonical identity is the tuple of `namespace` + `id`.

- `id` must be globally unique within the applicable namespace policy boundary.
- `memory_type` classifies the item but does not participate in identity; type migrations must preserve identity unless policy requires a replacement record.
- `compact_text` is the human-readable working identity for recall, review, and debugging, but it is not a durable key.
- `fingerprint` is a duplicate-family and collision-detection hint, never an authorization token or canonical identifier.
- `content_ref` and `payload_ref` are storage handles; changing them does not mint a new identity.
- If a payload is detached, redacted, or compacted, the memory keeps the same `id` so long as lineage and policy semantics remain continuous.

### Core-table and rebuild-authority contract

The first schema baseline treats these durable records as the authoritative rebuild source for memory semantics and derived-state repair:

- canonical memory rows in `memories` carry stable identity, lifecycle, provenance-bearing metadata, and the durable state that retrieval and maintenance flows must trust first
- authoritative float embedding records in `memory_embeddings` back semantic rebuild; quantized vectors, ANN row ids, and hot-search projections are derived accelerators only
- durable cluster and graph rows in `engrams`, `engram_members`, and `graph_edges` must stay inspectable and rebuildable without becoming a second identity system
- durable process state such as logical counters or generation anchors belongs in `brain_state` and must survive restart or repair before derived services claim healthy state
- caches, FTS projections, warmed routing mirrors, and other serving accelerators may be dropped, invalidated, or rebuilt at any time because they do not define canonical truth

### Migration, rollback, and schema-test expectations

Any schema work touching the canonical baseline must carry explicit migration and rollback expectations rather than leaving them implicit in prose.

- migration notes must name the changed tables, columns, meanings, compatibility window, and the rebuild plan for any affected derived state
- rollback notes must say how the prior authoritative shape is restored without losing stable identifiers or silently changing policy-bearing semantics
- deterministic schema coverage should prove namespace-scoped identifier stability, foreign-key integrity, and rebuild-from-durable-truth behavior after restart, repair, or migration
- repair or migration flows must preserve prior durable rows or emit explicit blocked/degraded artifacts; they must not silently widen authority to caches, sidecars, or other derived stores

### Canonical version contract

`version` tracks accepted mutation of one canonical memory record.

- `version` starts at `1` on first persistence and increments only when a mutation is accepted.
- Rejected writes, policy-denied updates, and no-op repair passes do not increment `version`.
- `created_at` is immutable after first persistence; accepted mutation updates `updated_at` so `created_at <= updated_at` always holds.
- Mutations that preserve identity may update content, metadata, decay state, tier, or policy-carrying fields, but they must preserve lineage and auditability.
- Changes that replace meaning rather than revise it must create a new record linked by lineage or supersession rather than reusing the old version counter.

### Replacement, supersession, and duplicate-family rules

Identity/version rules must keep revision, replacement, and collision handling separate.

- Contradictory or superseding facts create a new memory identity and connect it with `superseded_by`, `belief_version`, `belief_chain_id`, or a `ConflictRecord`; they do not silently overwrite an existing record in place.
- Summaries, extracts, repairs, and consolidations usually create new records with their own identities while preserving parent lineage.
- Duplicate-family collapse may merge retrieval treatment or maintenance state, but canonical IDs remain stable until an explicit merge artifact records the transformation.
- Duplicate-family handling and interference handling are separate lanes: duplicate-family logic groups materially same or near-same evidence, while interference logic applies bounded maintenance effects to similar but still-distinct memories.
- `fingerprint` may seed duplicate-family lookup or collision detection, but it does not authorize identity reuse, silent overwrite, cross-namespace merging, or policy bypass.
- `payload_ref` / `content_ref` must be stable, resolvable, or explicitly tombstoned so identity survives storage relocation.

### Mutation routing and belief-chain versioning rules

Contradiction-aware writes must classify whether accepted new evidence is revising one canonical memory or introducing a competing belief.

- If the accepted change preserves the same operative claim, the system may mutate that record in place, preserve lineage, and increment `version`.
- If the accepted change would materially alter the claim, time-slice interpretation, or other policy-relevant meaning, the system must mint a new memory identity, preserve evidence for both sides, and attach or create the relevant `ConflictRecord` and `belief_chain_id` instead of overwriting the older row.
- `belief_version` is chain-local ordering across distinct belief entries. It advances when a new competing or successor record joins the chain; it does not replace the per-record `version` counter for accepted in-place mutation.
- Open conflict or coexistence may influence default packaging, but they must not rewrite the losing memory's content in place or mark it `Superseded` unless supersession is explicitly accepted.
- Authoritative override records authority metadata and the preferred operational interpretation in durable conflict state without erasing the losing evidence.
- Only true supersession sets `superseded_by`, moves the older memory to `Superseded`, and makes the newer entry the default operative version.

## Metadata Families and Contracts

### Context Metadata

`workspace_id`, `agent_id`, `session_id`, and `task_id` capture the execution scope around a memory without replacing `namespace`. Goal context uses the same activity-context contract: when one explicit goal or work item governs the activity, `task_id` is the primary handle; when goal association is many-to-many or preserved historically, it should travel through `relation_refs` and lineage to `Goal` memories rather than through a second conflicting scope key. Later-stage resumable-goal and blackboard checkpoints are derived workflow artifacts, not a replacement authoritative scope model: they may summarize the active goal stack, selected evidence, pending dependencies, and pause/resume state for one task/session, but they must continue to point back to canonical task, session, and memory handles rather than becoming a second hidden identity layer.

| Field | Persisted shape | Required when | Nullable when | Allowed inference | Output / redaction |
|-------|-----------------|---------------|---------------|-------------------|--------------------|
| `workspace_id` | `Option<String>` | The source belongs to a concrete workspace/repo boundary or retrieval isolation depends on workspace scoping | Imported, global, or system artifacts have no single workspace | From the request envelope or a stable `source_ref` mapping only | Normal recall/search/get surfaces may return an opaque handle or omit the raw value with a redaction marker when the caller lacks workspace visibility |
| `agent_id` | `Option<String>` | An authenticated agent, worker, or scheduled job produced the record | Human-authored, imported, or system-originated artifacts lack a stable actor identity | From authenticated caller or job ownership only | Normal surfaces may coarsen to an opaque actor handle; inspect/audit may reveal the raw value only after policy checks |
| `session_id` | `Option<String>` | Live interaction events, `ToolOutcome`, `SessionMarker`, or session-scoped episodes/summaries come from one session window | Imported history, cross-session facts, and maintenance outputs have no single session | From active session binding or bounded batch scope only | Search/recall may omit or redact raw session identifiers unless the caller is entitled to session detail; inspect/explain must still distinguish redacted from absent |
| `task_id` | `Option<String>` | The memory was produced under an explicit issue, bead, ticket, task, or goal-shaped work item | Ambient exploration or background state has no explicit task anchor | From bound task context only; never from free-form text guesses | Return the raw handle only when the caller can resolve that task context; otherwise use an opaque handle or redaction marker |
| Goal context | `task_id` or goal-linked `relation_refs` | An explicit goal shaped the activity or later consolidation must preserve goal association | No explicit goal exists | Copied from request/task binding or parent lineage, never guessed from content text | Expose the governing task handle or goal-linked metadata subject to the same namespace, policy, and redaction rules as other linked memories |

- Missing context fields mean `unknown` or `not applicable`, not `global`.
- Inference is allowed only from bounded execution metadata such as the request envelope, auth/session binding, scheduler ownership, stable source mapping, or parent lineage; it must never come from model speculation over free-form text.
- Once persisted, explicit and inferred context values participate equally in filtering, replay, and audit, but inspect/explain surfaces should still be able to say when a value was redacted or unavailable.
- Encode must preserve enough provenance for each bound context field to distinguish caller-supplied values, trusted execution-envelope bindings, lineage carry-forwards, and bounded local derivations; later surfaces must not present all context as if it were user-authored.
- Derived memories may add fresh context of their own, but they must keep lineage back to source memories instead of replacing older context.

### Provenance Metadata

`source_kind`, `source_ref`, `authoritativeness`, `content_ref`, timestamps, and `lineage` form the provenance envelope.

- `source_kind` identifies the producing path (`cli`, `mcp`, `api`, `observe`, `import`, `repair`, `consolidation`, or an equivalent bounded extension).
- `source_ref` is a stable pointer to the originating request, file/span, tool call, message, or imported artifact.
- `authoritativeness` scores source reliability, not belief truth or recall priority.
- `content_ref` points at the canonical stored content handle used for redaction, repair, and lazy fetch.
- `lineage` records parent memory IDs and is mandatory for summarize, merge, extract, repair, contradiction, and consolidation outputs.
- The provenance envelope must also be sufficient to explain whether attached context, emotional annotations, advisory tags, and provisional write-time scoring came from explicit input, trusted source mapping, parent lineage, or bounded local derivation.
- `authoritativeness` carries source-trust semantics separately from confidence or salience so later ranking, governance, and explain surfaces do not flatten trust into one opaque importance number.
- A stored memory is only canonical if it can be traced either directly to a source reference or indirectly through lineage to durable evidence.

#### Durable family inheritance rules

- Every durable memory-item family in the M1 schema baseline inherits `created_at_ms`, `updated_at_ms`, `source_kind`, and `source_ref` as the minimum first-order provenance floor even when a narrower type table focuses on type-specific fields.
- Derived, repaired, consolidated, summarized, extracted, merged, and contradiction-related records must also carry `lineage`; first-order intake may leave `lineage` empty when `source_kind` plus `source_ref` already identify the durable evidence directly.
- If policy requires source redaction, the system must preserve a stable opaque `source_ref` handle or an explicit tombstone record rather than dropping source traceability.
- Schema, migration, rollback, compaction, and repair work must preserve provenance envelopes and lineage across every durable item family; accelerators and sidecars may mirror this data, but they cannot become the sole surviving ancestry record.

#### Deterministic schema-proof expectations

- schema fixtures should prove each durable item family either stores `source_kind` and `source_ref` directly or resolves them through explicit lineage to durable parent evidence
- derivation fixtures should prove summarize, extract, merge, repair, contradiction, and consolidation flows preserve parent lineage rather than replacing it with opaque prose
- migration and rollback fixtures should prove provenance handles, timestamps, and lineage survive schema motion unchanged unless policy explicitly emits a redaction or loss artifact
- inspect and repair fixtures should prove caches, ANN projections, graph materializations, and other derived accelerators cannot satisfy provenance reconstruction on their own when the canonical durable record is absent

#### Governance and isolation inheritance rules

- Every durable memory-item family in the M1 schema baseline also inherits `namespace` as a mandatory field and must keep it valid before persistence, replay, repair, or migration.
- `agent_id` is the producing actor handle when one exists; it may be absent for human-authored, imported, or system-originated artifacts, but if present it must come from authenticated execution metadata rather than content inference.
- Shared-capable or policy-scoped families must preserve explicit `visibility` state using the bounded vocabulary `private`, `shared`, or `public`; wrappers and repair flows must not widen visibility implicitly.
- Every durable family must be able to represent `retention_class` and `policy_flags` so retention, legal-hold, compliance, redaction, shareability, and deletion constraints are enforceable from stored metadata alone.
- Namespace, visibility, retention, and policy markers must survive migration, rollback, archival, restore, and policy-driven deletion flows exactly or emit an explicit tombstone/loss artifact instead of silently disappearing.

#### Deterministic governance-proof expectations

- schema fixtures should prove every durable item family persists a valid `namespace` and can expose `retention_class` plus `policy_flags` without detached-payload hydration
- scope-binding fixtures should prove `agent_id` is sourced from authenticated caller or job ownership metadata only
- visibility fixtures should prove shared-capable families reject unknown labels, preserve `private/shared/public` state across migration, and fail closed instead of widening access during wrapper translation or repair
- lifecycle and policy fixtures should prove archival, restore, redaction, legal-hold, and deletion flows preserve or explicitly tombstone prior governance metadata

#### Schema migration, backfill, rollback, and doc propagation rules

- Every schema-changing bead or PR must carry a migration note naming the changed tables, columns, meanings, compatibility window, rollout order, and any rebuild or reindex surfaces affected by the change.
- A backfill plan is required whenever existing rows, derived stores, or governance metadata need reinterpretation; it must say whether the job is online or background, restart-safe, bounded, and how partial completion is surfaced.
- Every schema-changing work packet must carry a rollback note that names the exact restore path, irreversible edges if any, containment behavior on failure, and how stable identifiers, lineage, namespace, retention, and policy semantics survive reversal.
- Doc propagation is part of schema correctness: when a schema meaning changes, the owning work must update the canonical `docs/PLAN.md` contract when needed plus every affected elaboration surface such as `docs/DATA_SCHEMAS.md`, `docs/MEMORY_MODEL.md`, `docs/MCP_API.md`, and `docs/CLI.md`.
- Backfill and rollback flows must rebuild from authoritative durable evidence rather than from caches, ANN projections, warmed mirrors, or graph-only copies.
- If migration, backfill, or rollback cannot complete cleanly, the system must emit explicit degraded or blocked artifacts instead of silently presenting the new schema as healthy.

#### Deterministic migration-proof expectations

- migration fixtures should prove old and new shapes preserve stable identifiers, timestamps, lineage, namespace binding, and policy-bearing metadata exactly unless the contract explicitly emits a tombstone or loss artifact
- backfill fixtures should prove existing rows and derived stores converge from authoritative durable evidence under the documented mapping rule
- rollback fixtures should prove the documented restore path either re-establishes the prior authoritative semantics or fails closed with explicit degraded evidence
- doc-propagation checks should prove every touched exposed schema surface updated its matching contract docs instead of leaving canonical or interface docs stale

### Utility, Retention, and Recall Metadata

`salience`, `confidence`, `utility_estimate`, `recall_count`, `last_access_at`, `retention_class`, `decay_state`, and `policy_flags` describe how the system should treat the memory over time.

- `salience` is the immediate routing and ranking signal for current importance.
- `confidence` captures reliability and corroboration state; it is separate from strength and source authoritativeness.
- `utility_estimate` predicts future usefulness for ranking, promotion, demotion, and context-budget packing.
- `recall_count` and `last_access_at` track successful surfaced reuse, not merely candidate generation; they are audit-facing reuse signals rather than the canonical logical decay clock.
- `retention_class` expresses intended durability (`volatile`, `normal`, `durable`, `pinned`) independently from current tier.
- `decay_state` stores the parameters needed for lazy decay and reinforcement updates without eager whole-store rewrites, including the authoritative logical-tick anchor (`last_accessed_tick`) used by the decay formula.
- `policy_flags` carry governance-critical markers such as legal hold, compliance lock, redaction state, and shareability constraints.
- Initial `salience` is a provisional write-side signal. It may reflect bounded encode cues such as attention outcome, novelty lane, explicit task relevance, and emotional arousal, but it must not erase provenance or collapse source trust into the same scalar.
- `encoding_valence` and `encoding_arousal` capture emotional significance when explicit input or bounded local derivation supplies it; their origin must remain explainable alongside salience.
- Utility or salience may tune ranking and maintenance, but they must never override policy flags, pins, or namespace checks.

### Linkage and Classification Metadata

`tags`, `entity_refs`, and `relation_refs` enrich retrieval without becoming hidden truth.

- `tags` are advisory labels supplied by users, agents, or background jobs.
- Encode-time contextual tags may describe bounded task, topic, environment, or operator cues that help later recall and ranking, but they remain advisory labels rather than namespace, identity, or relation truth.
- Tags that materially affect ranking, governance, or explain surfaces must remain attributable to their source path rather than appearing as anonymous annotations.
- `entity_refs` point to canonical entities for entity-centric recall and repairable graph links.
- `relation_refs` point to explicit relation records or resolvable tombstones.
- Summaries, facts, skills, and repaired artifacts must preserve enough tags/entity/relation linkage to remain explainable after compaction or consolidation.

### Entity and relation canonicalization contract

`entity_refs` and `relation_refs` must resolve to stable canonical handles rather than ad hoc extracted strings. The contract freezes what downstream systems may rely on without forcing one extraction or normalization algorithm.

#### Entity canonicalization rules

- Canonical entity identity is namespace-scoped and stable: preferred display-name changes, formatting cleanup, or alias additions do not mint a new entity identity.
- Observed surface forms such as aliases, spellings, handles, or mention text are evidence about an entity, not the entity identity itself.
- A memory may carry multiple entity references when the evidence truly mentions multiple entities; it must not collapse them into one handle for convenience.
- If canonicalization is uncertain, the system must preserve the unresolved mention or low-confidence candidate state rather than forcing a speculative entity link.
- Later merge or split decisions for entities must be explicit, auditable transformations so retrieval, graph repair, and contradiction workflows can explain why an older reference now resolves differently.

#### Relation canonicalization rules

- Canonical relation identity is separate from free-form prose and separate from memory identity; `relation_refs` point to normalized relation records, not just edge labels inferred on the fly.
- Every canonical relation record must identify explicit endpoints using canonical memory, entity, or goal handles, unless one endpoint has been tombstoned explicitly.
- Relation kind or edge label should normalize to a shared durable vocabulary at the storage boundary, but the docs do not require one particular ontology or extraction model.
- Relation records must preserve provenance or derivation handles and enough confidence or status metadata for explain, contradiction handling, and repair to treat them as structured evidence rather than opaque annotations.
- Competing or contradictory relations between the same endpoints must coexist as inspectable state until resolved; the system must not silently overwrite one edge with another.
- Cross-namespace relation edges require explicit policy support; relation canonicalization must never become a backdoor around namespace or visibility controls.

#### Downstream expectations

- Retrieval may use canonical entity and relation handles for filtering, expansion, and reranking, but alias text alone must never become the sole surviving truth.
- Inspect and explain surfaces should be able to show the canonical handle, the observed alias or normalized relation kind that produced it, and whether the reference is resolved, ambiguous, or tombstoned.
- Graph materializations, caches, and extracted summaries remain derived state; authoritative durable entity and relation records win during repair or rebuild.
- When compaction, repair, or consolidation cannot preserve an entity or relation reference exactly, the system must emit an explicit loss or tombstone record instead of inventing a replacement.

## Graph identity and edge taxonomy contract

Graph-linked retrieval adds bounded neighborhood structure without minting a second identity system.

### Graph entity classes

- Memory nodes use the canonical memory identity tuple (`namespace` + `id`). Graph traversal, caches, and inspect surfaces must point back to that same handle rather than a graph-local surrogate.
- Engram or cluster records use their own stable `engram_id` for membership, centroid maintenance, and bounded expansion seeds. An `engram_id` is a cluster handle, not a replacement memory identity.
- Cluster lineage after split or hierarchy maintenance must remain explicit through stable parent/child linkage such as `parent_engram_id`, so repair and inspect can explain how a later cluster descends from an earlier one.
- Entity, relation, goal, contradiction, and lineage handles may seed adjacency or filtering, but their authoritative meaning remains in canonical entity, relation, conflict, and workflow records rather than in graph-only copies.

### Edge taxonomy

- `Associative` edges represent similarity or co-activation neighborhoods used for bounded engram expansion and partial-cue recall. They are rebuildable from durable evidence plus embeddings and must not become the sole surviving record of a semantic claim.
- `Temporal` edges represent ordered adjacency grounded in timestamps, session or episode continuity, or other durable chronology evidence.
- `Causal` edges represent claimed or derived “led to” structure and must remain backed by inspectable evidence, lineage, or canonical relation records rather than existing only as an opaque traversal hint.
- `Contradictory` edges represent active contrast or disagreement neighborhoods that support explain, belief-history, and conflict-aware recall; they do not replace `ConflictRecord`, supersession, or belief-chain state.
- Additional graph-local labels are allowed only when documented as derived materializations, and they must not silently widen the canonical user-visible taxonomy above.

### Persistence identifier and authority rules

- Persistent graph rows use stable canonical handles at their endpoints: memory ids for memory-to-memory edges, stable engram ids for cluster metadata, and canonical entity or relation handles when those records are the real authority.
- Sidecar-only ids such as ANN row ids or `usearch_id` may exist for index maintenance, but they are deterministic external mappings rather than durable public identifiers and must never be the only way to interpret a graph row after repair.
- Purely similarity-driven graph materializations are derived state and may be dropped or rebuilt; edges that encode canonical relation, contradiction, lineage, or policy-relevant truth must continue to resolve through authoritative durable records.
- Cross-namespace graph or cluster links require explicit policy support and must never bypass namespace or visibility enforcement merely because two endpoints are near each other in vector or graph space.
- Tombstoned, redacted, or superseded endpoints keep inspectable loss or policy markers; graph repair may remove stale accelerators, but it must not invent replacement endpoints or silently collapse conflict semantics.

## Encode-Time Normalization Contract

All write paths must normalize raw intake into one canonical memory object envelope before persistence. Normalization chooses the first persisted `memory_type`, binds scope and provenance, and preserves raw evidence without prematurely collapsing it into distilled knowledge.

### Normalization outputs

Before persistence, a normalized candidate must have:

- a namespace-bound identity allocation path plus timestamps suitable for canonical storage,
- a chosen `memory_type` and any explicit brain-inspired kind classification,
- bounded `compact_text` plus either inline content or a stable `content_ref` / `payload_ref`,
- a source envelope (`source_kind`, `source_ref`, `authoritativeness`) that preserves how the item entered the system,
- any request, auth, session, workspace, or task context that is explicitly bound by execution metadata, and
- lineage only when the item is derived from prior memories rather than first-order intake.

### Source-to-type mapping rules

- Raw messages, actions, tool calls, and state changes default to `Event` on first persistence unless a more specific canonical family below applies.
- Passive observation, watch, file, or environment signals normalize to `Observation`; `observation_source` is required when known, and `observation_chunk_id` groups bounded observation fragments from one observation pass without inventing new session boundaries.
- Completed tool executions normalize to `ToolOutcome`; normalization must preserve tool identity, execution reference, execution context, outcome category, and detachable output payload handles when results are too large for hot inline storage.
- Explicit durable user conventions or standing instructions normalize to `UserPreference` only when caller intent or typed ingestion marks them as stable preference state; otherwise they remain raw evidence first.
- Session starts, stops, checkpoints, handoffs, and equivalent temporal boundaries normalize to `SessionMarker`.
- Resume/checkpoint markers for later-stage working-state features should record explicit boundary events, not guessed semantic milestones. If a resumable-goal stack or blackboard checkpoint is persisted, it remains a derived artifact tied to the governing `task_id`, `session_id`, and source memory handles rather than a new canonical memory family.
- Distilled `Fact`, `Summary`, `Relation`, `Skill`, `Constraint`, or `Hypothesis` records should normally arise from typed ingestion or later extraction and consolidation, not speculative promotion of raw text during first-write normalization.

### Preservation and non-inference rules

- Normalization may derive structured metadata only from explicit input, request envelopes, authenticated context, stable source mappings, or bounded derivation rules that preserve lineage.
- Free-form text alone must not invent `workspace_id`, `agent_id`, `session_id`, `task_id`, entity links, relation edges, or user-preference status.
- Large or structured raw payloads must keep a bounded `compact_text` while preserving canonical detail through `content_ref` or `payload_ref`; lossy truncation without a payload handle is invalid.
- Ambiguous intake should prefer the more conservative raw-evidence type (`Event` or `Observation`) plus preserved provenance over speculative semantic elevation.
- If normalization cannot bind namespace, traceable provenance, or a valid canonical type, the write must fail validation rather than persisting opaque text with guessed metadata.

### Encode-time tagging and write-side explainability

Encoding attaches context, emotion, source-trust, and provisional ranking signals before persistence so later retrieval, governance, and explain surfaces do not need to reconstruct them from raw text alone.

- Context binding happens before routing and durable insert; if workspace, session, task, or goal context cannot be bound from explicit input, trusted envelopes, or lineage, the write proceeds without that field rather than guessing.
- Emotional tagging uses `encoding_valence` and `encoding_arousal`; caller-provided values may be stored directly, and bounded local derivation may add them only when the derivation path remains inspectable.
- Source trust travels through `authoritativeness` plus provenance handles, not through hidden salience-only heuristics.
- Initial salience is provisional write-side scoring used for admission, routing, and later ranking decomposition. It must remain explainable as a combination of bounded encode cues rather than an opaque black-box value.
- Successful writes must preserve enough route metadata that `memory_inspect` and `memory_explain` can answer what content or payload was stored, which context and tags were explicit versus inferred, which provenance handles justified the write, and which write-side signals affected admission or tier choice.

### Duplicate-family, interference, and fast-path observability

Duplicate handling, interference checks, and write-path observability are related encode concerns, but they must stay separately inspectable.

- Duplicate-family handling answers whether intake belongs to an existing materially same evidence family; interference answers whether similar but distinct memories should carry bounded maintenance penalties or retrieval difficulty. Neither lane replaces contradiction or supersession handling.
- Duplicate-family routing is an encode-time identity decision. Its observable outcome must say whether the write minted a new canonical identity, attached to an existing duplicate family, or updated/reconsolidated an existing canonical record under explicit duplicate rules rather than leaving operators to infer that from later maintenance state.
- Duplicate-family checks may use `fingerprint`, bounded similarity search, and source/provenance hints, but a duplicate outcome must not silently overwrite canonical content, provenance, or policy state.
- Interference checks run only on a bounded similar-candidate slice after near-identical duplicate-family cases are excluded; any resulting penalties change maintenance state, not canonical identity or belief resolution.
- Encode fast-path duplicate and interference work must stay within explicit shortlist and latency budgets. If secondary maintenance cannot complete inside those budgets, the system should skip or defer it rather than scanning the full corpus or blocking the write indefinitely.
- Write-path observability must distinguish admission outcome, duplicate-family route outcome, shortlist evidence such as candidates inspected and nearest-neighbor similarity or novelty, interference adjustments, and skipped or deferred maintenance so later debugging and benchmark work can attribute latency and behavior correctly.
- `memory_inspect` and `memory_explain` should be able to report which duplicate/interference lanes fired, which were bypassed or deferred, whether pattern separation preserved a new record despite high similarity, and whether those lanes later affected retrieval or maintenance behavior.

## Memory States (Lifecycle)

```
Labile → SynapticDone → Consolidating → Consolidated → Archived
                                                    ↗
                                        Superseded ─┘
```

| State | Description |
|-------|-------------|
| `Labile` | Freshly created or just recalled — mutable, within reconsolidation window |
| `SynapticDone` | Initial encoding complete, pending consolidation |
| `Consolidating` | Being processed by consolidation engine |
| `Consolidated` | Stable long-term memory |
| `Superseded` | Replaced by a newer belief version (Feature 2) |
| `Archived` | Soft-deleted by decay or active forgetting |

### State Transition Rules

1. Newly encoded memories enter `Labile` and stay there only for the reconsolidation window.
2. When the reconsolidation window expires without an accepted mutation, the memory becomes `SynapticDone`.
3. Consolidation moves `SynapticDone -> Consolidating -> Consolidated`; this pipeline must preserve identity, lineage, and policy-bearing metadata.
4. Successful recall may temporarily reopen a `SynapticDone` or `Consolidated` memory into `Labile` so reconsolidation can occur without minting a new identity.
5. True contradiction resolution may move a memory to `Superseded`; supersession is not decay, archival, or deletion.
6. Forgetting or retention action may move a non-pinned memory to `Archived`; archived memories remain durable and inspectable even when normal recall stops surfacing them.

### Lifecycle guard summary

Before any lifecycle transition is committed, the system must validate namespace access control, policy pinning or legal hold, retention constraints, lineage preservation, unresolved contradiction semantics, and repair-job lock safety.

### Demotion, archive, restore, and deletion distinctions

- Demotion changes tier ownership, payload attachment, or default serving priority without moving the memory into `Archived`; demoted memories remain eligible for normal recall when policy and ranking allow, so later promotion is not itself a restore operation.
- Demotion may still leave payloads detached, snippet-only, or otherwise lower fidelity on the default serving path. Rehydrating those retained surfaces is a bounded active-memory operation, not an archive restore, and it must report degraded state when full payload reattachment is unavailable.
- `Archived` means the memory has left ordinary default recall and now requires an explicit restore or replay path before it can return to active serving. Ordinary recall, cache warming, and snapshot inspection must not silently reopen archived memories.
- Policy-driven hard deletion is separate from archival. It may remove content or payload recoverability, but it must leave durable tombstone or audit evidence when policy requires and must never be implied by utility-driven forgetting alone.
- Restore preserves canonical identity, provenance, lineage, policy markers, contradiction state, and archive history. Derived indexes, caches, and warm routing mirrors are rebuilt after the durable lifecycle change commits; they do not define whether restore succeeded.
- If only retained authoritative metadata survives while payload or other previously served surfaces were redacted, detached, or lost, the restored item must carry explicit degraded or partial-fidelity markers rather than masquerading as a full rewind of the pre-archive state.
- Archived items remain inspectable and auditable even when default recall omits them. After explicit restore, they re-enter normal ranking and routing subject to the same policy, freshness, contradiction, and lifecycle rules as other active memories.

### Failure behavior summary

If a lifecycle transition fails mid-flight:
- preserve the last known valid state
- emit a transition error artifact or event
- enqueue repairable follow-up work when possible
- never leave the memory in a silent half-transitioned state

### Relationship to lower-level state machines

This section freezes the canonical persisted lifecycle for memory items. The `created`, `indexed`, `recalled`, `reinforced`, `decayed`, `demoted`, and `deleted` stages described in `docs/STATE_MACHINES.md` are lower-level logical/controller transitions and operational outcomes, not a second conflicting persisted lifecycle enum for memories.

## Strength & Decay Model

```
effective_strength = base_strength × e^(-Δtick / stability)
```

The decay clock is logical and usage-driven rather than wall-clock driven.

- `Δtick` means `current_tick - decay_state.last_accessed_tick`, where `current_tick` comes from the process-wide interaction counter.
- The interaction counter advances on request-path encode and recall operations that materially exercise the memory system, so decay reflects intervening brain activity rather than elapsed human time.
- The canonical durable anchor for decay is `decay_state.last_accessed_tick`; `last_access_at` is an audit timestamp for surfaced reuse and must not replace the logical tick anchor in formulas, SQL prefilters, or repair/rebuild flows.
- The global interaction counter itself is durable brain state and must resume monotonically across restart or replay rather than resetting opportunistically.
- Implementations may derive or cache `effective_strength`, but the canonical durable truth remains `base_strength`, `stability`, `last_accessed_tick`, and `bypass_decay` stored in `decay_state`.
- Normal request paths compute decay lazily on demand; they must not require eager per-tick rewrites of the whole store.

| Parameter | Description |
|-----------|-------------|
| `base_strength` | Current persisted strength baseline before applying elapsed logical-tick decay |
| `stability` | How resistant to decay (increases with each successful recall) |
| `Δtick` | `current_tick - last_accessed_tick`, measured in interaction ticks rather than elapsed wall time |
| `bypass_decay` | If true, effective strength remains `base_strength` until the bypass condition is explicitly cleared |

### Lazy decay semantics

- `effective_strength` is computed from persisted decay state at read, ranking, prefilter, consolidation, and maintenance time; idle memories incur zero background decay cost.
- Persisting decay means replacing `base_strength` with the currently computed `effective_strength` and resetting `last_accessed_tick` to `current_tick`; this happens only when a workflow explicitly commits that decay-aware state transition.
- Successful recall persists decay first, then applies recall-side reinforcement and stability growth, then resets the decay clock to the current tick as part of the same accepted mutation.
- Maintenance jobs such as consolidation, demotion, or forgetting may also persist decay when they commit a durable lifecycle or routing decision, but they must preserve the same formula and clock semantics as request-path recall.
- SQL or index prefilters may evaluate the same formula against stored decay parameters, but those accelerators remain derived behavior and must not redefine the canonical formula.

### Successful recall contract

- A memory counts as successfully recalled only when it survives namespace, policy, and lifecycle gating and lands in the final bounded returned set for the request, or when an explicit recall-side mutation surface intentionally invokes the same strengthening path for that memory.
- Candidate inspection, ANN shortlist membership, graph expansion, reranker consideration, cache warming, inspect-only mention, and redacted or denied rows do not count as successful recall and must not increment `recall_count` or apply LTP.
- The canonical recall-side mutation is one durable accepted update that persists pending decay, increments `recall_count`, records `last_access_at`, applies bounded `base_strength` and `stability` growth, updates `decay_state.last_accessed_tick`, and reopens the memory into `Labile` when the reconsolidation contract says recall should do so.
- If an in-window content update would materially contradict the reopened memory's current claim, recall must fork into the contradiction path and mint a new chain member instead of applying that content change in place.
- Tier1 refresh, cache updates, and other hot serving mirrors happen only after that durable mutation commits; they are derived side effects and must not become the source of truth for whether recall-side strengthening happened.
- Retries, restart recovery, repair, or cache replay must rebuild from the last committed durable row and must not double-apply recall-side strengthening, counter increments, or labile-window reopening.

### Strength Modification

- **LTP (on recall)**: after persisting any pending lazy decay, `base_strength += LTP_DELTA` (capped at `MAX_STRENGTH`)
- **Stability increase**: on each successful recall, `stability += STABILITY_INCREMENT × stability`, bounded by the configured ceiling
- **Interference**: related memories may receive bounded interference penalties under the separate interference contract; this is not itself the decay formula
- **Reconsolidation**: accepted updates during the reconsolidation window may add bounded bonuses without changing the logical-tick decay model

### Determinism and audit rules

- Correctness claims about decay, recency, forgetting thresholds, or reconsolidation windows must be expressed with interaction ticks, logical ticks, or injected clocks rather than ambient wall time.
- Explain, inspect, and audit surfaces should be able to expose the decay inputs that mattered for a result or lifecycle action: `current_tick`, `last_accessed_tick`, `stability`, whether `bypass_decay` applied, and whether decay was merely computed or also persisted.
- Restart, rebuild, and repair flows must preserve enough durable brain state to reproduce the same decay math after recovery; resetting the logical counter or silently substituting wall-clock timestamps would violate this contract.

## Confidence Model (Feature 7)

Separate from strength. Strength = consolidation level. Confidence = reliability.

| Event | Effect |
|-------|--------|
| Reconsolidation | `confidence -= 0.05 × √(reconsolidation_count)` |
| Corroboration (sim > 0.9) | `confidence += 0.1 / √(corroboration_count)` |
| Contradiction detected | `confidence -= 0.2` |
| Causal root invalidated (F11) | depth 1: −0.20, depth 2: −0.10, depth 3+: −0.05 |

Floor: 0.1 (never fully invalidated). Ceiling: 1.0.

## Contradiction Handling

### Canonical semantic distinctions

- **Contradiction** means two or more memories make materially incompatible claims about the same subject, time slice, or policy-relevant fact. Contradictions create explicit conflict artifacts; they do not silently rewrite either side.
- **Supersession** is one possible contradiction resolution where a newer memory becomes the default operative version for normal retrieval. The older memory remains preserved, inspectable, and linked by `superseded_by`, `belief_chain_id`, and lineage.
- **Unresolved conflict** means the system has detected contradiction but no winner has been accepted yet. Both sides remain first-class evidence and recall/inspect surfaces must expose the disagreement explicitly.
- **Authoritative override** is a policy-aware resolution where an explicitly higher-authority source or human decision marks one side as the preferred operational interpretation without erasing the losing evidence.

### Machine-readable representation

When new information conflicts with existing evidence:
1. Do **not** silently overwrite either record.
2. Create or update a `ConflictRecord` / `belief_conflicts` entry.
3. Preserve evidence references, provenance, and lineage for both sides.
4. Record `conflict_state` on affected memories as one of `open`, `resolved`, `superseded`, or `authoritative_override`.
5. Keep linked `conflict_record_ids` so inspect/recall can surface conflict state without reconstructing it from raw text.
6. Only set `superseded_by` and move the older memory to lifecycle state `Superseded` when the resolution is actually supersession.
7. For authoritative overrides, record the preferred memory plus the authority source/reason in the conflict artifact rather than mutating old evidence in place.

### Mutation and resolution flow

- Conflict handling must classify whether the accepted write is a same-identity revision, an open conflict, a coexistence outcome, a supersession, or an authoritative override before durable state commits.
- `version` advances only for accepted in-place mutation of one canonical record. `belief_version` advances only when a distinct belief entry joins an existing `belief_chain_id` or starts a successor position within that chain.
- Open conflict and coexistence keep the participating memories queryable as first-class evidence. They may expose a preferred operational answer for default packaging, but they must not rewrite the losing record's content or lifecycle in place.
- Authoritative override updates durable conflict metadata with authority source, authority reference, resolution reason, and the preferred operational interpretation while preserving both sides as inspectable evidence.
- Supersession is the only resolution that marks the older memory `Superseded`; it must preserve lineage and leave enough durable conflict metadata to reconstruct why the operative default changed.

### Retrieval, inspect, and repair implications

- Retrieval and inspect surfaces may prefer one side for ranking or packaging, but they must still be able to expose open conflict state, suppressed alternatives, and the evidence lineage behind the decision.
- Ranking treats contradiction state as a first-class signal; it is not a post-processing hack layered on after retrieval.
- Repair and audit flows must be able to rebuild contradiction state from durable conflict artifacts plus preserved source lineage.

## Schema Rules

1. `version` increments on accepted mutation
2. Lineage preserved across summarize / merge / extract / repair
3. Payload and summary are separable
4. Fingerprints stable for duplicate-family handling
5. Policy flags travel with the memory, not only one index layer
6. Tier location is persisted state, not inference
7. `payload_ref` / `content_ref` must be stable, resolvable, or explicitly tombstoned
8. Hot-path filter fields, lifecycle/tier state, policy markers, contradiction handles, and provenance handles must remain queryable without fetching detached payload bytes or decoding opaque metadata blobs

## Tiering

| Tier | Storage | Latency | Capacity |
|------|---------|---------|----------|
| **Tier1** | In-memory LRU cache | <0.1ms | 512 entries |
| **Tier2** | SQLite WAL + USearch HNSW (in-RAM) | <5ms | ~50k vectors |
| **Tier3** | SQLite + USearch mmap (disk-backed) | <50ms | Unlimited |

- Canonical tier assignment begins only after a write is accepted for persistence; working-memory residency is controller state rather than a fourth persisted tier.
- First-write admission lands in the hot durable route before any cache seeding. Ordinary online encode therefore starts as hot-owned state rather than minting new canonical memories directly into Tier3.
- Successful slow-path results may refresh or seed higher warm tiers for bounded reuse, but that cache warming does not itself change the canonical durable owner.
- Consolidation or explicit demotion moves hot durable ownership toward cold durable surfaces; forgetting or retention action may then archive that durable record according to policy.
- Working-memory eviction and Tier1 cache eviction are not demotion events by themselves.

### Background consolidation pipeline contract

Background consolidation is one bounded controller workflow ordered as `NREM-like migration -> REM-like linking/desensitization -> homeostasis`.

- The NREM-like pass evaluates eligible hot-route memories and may move canonical durable ownership colder while preserving retrieval continuity, provenance, lineage, and identity/version rules.
- The REM-like pass may reduce emotional charge and add auditable cross-links or other lineage-backed derived artifacts, but it must not create untraceable sole truth.
- Homeostasis is the saturation-control pass after those earlier stages; it may archive only policy-eligible weak memories after downscaling and must never prune pinned or last-authoritative evidence.
- These passes are controller work rather than a separate persisted lifecycle enum; if any stage fails, the last committed durable state remains authoritative and stale derived work is repaired or rebuilt later.
