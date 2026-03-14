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
| `tier` | enum | Tier1, Tier2, or Tier3 |
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
| `superseded_by` | Option | F2 Belief Versioning | Points to newer version |
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

`workspace_id`, `agent_id`, `session_id`, and `task_id` capture the execution scope around a memory without replacing `namespace`.

- `workspace_id` scopes memories to a workspace or repo-like boundary for isolation, replay, and retrieval filtering.
- `agent_id` identifies the actor that created the record or emitted a derived artifact; it is provenance, not an authorization shortcut.
- `session_id` groups memories from one live interaction window for episodic replay, handoff, and consolidation.
- `task_id` ties the memory to an explicit unit of work such as a goal, issue, or bead when one exists.
- Missing context fields mean `unknown` or `not applicable`, not `global`.
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

1. Newly encoded → `Labile` (within reconsolidation window)
2. Window expires without update → `SynapticDone`
3. Consolidation engine processes → `Consolidating` → `Consolidated`
4. Recalled → temporarily returns to `Labile`
5. Contradicted by newer memory → `Superseded`
6. Strength falls below threshold → `Archived`

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

When new information conflicts with existing:
1. Do **not** silently overwrite
2. Create or update a `ConflictRecord` / `belief_conflicts` entry
3. Attach evidence references for both sides
4. Let retrieval/ranking choose presentation order
5. Preserve enough metadata for audit and repair
6. Old memory gains state `Superseded`, linked via `superseded_by`

## Schema Rules

1. `version` increments on accepted mutation
2. Lineage preserved across summarize / merge / extract / repair
3. Payload and summary are separable
4. Fingerprints stable for duplicate-family handling
5. Policy flags travel with the memory, not only one index layer
6. Tier location is persisted state, not inference
7. `payload_ref` / `content_ref` must be stable, resolvable, or explicitly tombstoned

## Tiering

| Tier | Storage | Latency | Capacity |
|------|---------|---------|----------|
| **Tier1** | In-memory LRU cache | <0.1ms | 512 entries |
| **Tier2** | SQLite WAL + USearch HNSW (in-RAM) | <5ms | ~50k vectors |
| **Tier3** | SQLite + USearch mmap (disk-backed) | <50ms | Unlimited |

Promotion: successful slow-path results update higher tiers.
Demotion: consolidation moves hot → cold; decay + forgetting → archive.
