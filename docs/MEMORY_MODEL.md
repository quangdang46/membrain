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
| `last_access_at` | Option | Last recall timestamp |
| `retention_class` | enum | volatile, normal, durable, pinned |
| `decay_state` | struct | Current decay parameters |
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
| `distilled_from_engram` | Option | F8 Skill Extraction | Source engram for procedurals |
| `namespace_id` | String | F9 Cross-Agent | Namespace for sharing |
| `visibility` | enum | F9 | private, shared, public |
| `has_causal_parents` | bool | F11 Causal | Has causal provenance |
| `has_causal_children` | bool | F11 | Is source of derived beliefs |
| `compressed_into` | Option | F17 Compression | Schema memory this was compressed into |
| `encoding_valence` | Option | F18 Emotional | Mood at encoding time |
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
- `payload_ref` / `content_ref` must be stable, resolvable, or explicitly tombstoned so identity survives storage relocation.

## Metadata Families and Contracts

### Context Metadata

`workspace_id`, `agent_id`, `session_id`, and `task_id` capture the execution scope around a memory without replacing `namespace`. Goal context uses the same activity-context contract: when one explicit goal or work item governs the activity, `task_id` is the primary handle; when goal association is many-to-many or preserved historically, it should travel through `relation_refs` and lineage to `Goal` memories rather than through a second conflicting scope key.

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
- Derived memories may add fresh context of their own, but they must keep lineage back to source memories instead of replacing older context.

### Provenance Metadata

`source_kind`, `source_ref`, `authoritativeness`, `content_ref`, timestamps, and `lineage` form the provenance envelope.

- `source_kind` identifies the producing path (`cli`, `mcp`, `api`, `observe`, `import`, `repair`, `consolidation`, or an equivalent bounded extension).
- `source_ref` is a stable pointer to the originating request, file/span, tool call, message, or imported artifact.
- `authoritativeness` scores source reliability, not belief truth or recall priority.
- `content_ref` points at the canonical stored content handle used for redaction, repair, and lazy fetch.
- `lineage` records parent memory IDs and is mandatory for summarize, merge, extract, repair, contradiction, and consolidation outputs.
- A stored memory is only canonical if it can be traced either directly to a source reference or indirectly through lineage to durable evidence.

### Utility, Retention, and Recall Metadata

`salience`, `confidence`, `utility_estimate`, `recall_count`, `last_access_at`, `retention_class`, `decay_state`, and `policy_flags` describe how the system should treat the memory over time.

- `salience` is the immediate routing and ranking signal for current importance.
- `confidence` captures reliability and corroboration state; it is separate from strength and source authoritativeness.
- `utility_estimate` predicts future usefulness for ranking, promotion, demotion, and context-budget packing.
- `recall_count` and `last_access_at` track successful surfaced reuse, not merely candidate generation.
- `retention_class` expresses intended durability (`volatile`, `normal`, `durable`, `pinned`) independently from current tier.
- `decay_state` stores the parameters needed for lazy decay and reinforcement updates without eager whole-store rewrites.
- `policy_flags` carry governance-critical markers such as legal hold, compliance lock, redaction state, and shareability constraints.
- Utility or salience may tune ranking and maintenance, but they must never override policy flags, pins, or namespace checks.

### Linkage and Classification Metadata

`tags`, `entity_refs`, and `relation_refs` enrich retrieval without becoming hidden truth.

- `tags` are advisory labels supplied by users, agents, or background jobs.
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
- Distilled `Fact`, `Summary`, `Relation`, `Skill`, `Constraint`, or `Hypothesis` records should normally arise from typed ingestion or later extraction and consolidation, not speculative promotion of raw text during first-write normalization.

### Preservation and non-inference rules

- Normalization may derive structured metadata only from explicit input, request envelopes, authenticated context, stable source mappings, or bounded derivation rules that preserve lineage.
- Free-form text alone must not invent `workspace_id`, `agent_id`, `session_id`, `task_id`, entity links, relation edges, or user-preference status.
- Large or structured raw payloads must keep a bounded `compact_text` while preserving canonical detail through `content_ref` or `payload_ref`; lossy truncation without a payload handle is invalid.
- Ambiguous intake should prefer the more conservative raw-evidence type (`Event` or `Observation`) plus preserved provenance over speculative semantic elevation.
- If normalization cannot bind namespace, traceable provenance, or a valid canonical type, the write must fail validation rather than persisting opaque text with guessed metadata.

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

| Parameter | Description |
|-----------|-------------|
| `base_strength` | Current persisted strength |
| `stability` | How resistant to decay (increases with each recall) |
| `Δtick` | Ticks since last access |
| `bypass_decay` | If true (emotional): always returns base_strength |

### Strength Modification

- **LTP (on recall)**: `strength += LTP_DELTA` (capped at MAX_STRENGTH)
- **Stability increase**: `stability += STABILITY_INCREMENT × stability`
- **Interference**: `similar.strength -= interference_penalty(similarity)`
- **Reconsolidation**: `strength += RECONSOLIDATION_BONUS` (if updated during window)

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
