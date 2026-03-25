# DATA SCHEMAS

## M1 core tables, stable identifiers, and rebuild authority

This baseline freezes the durable core tables that later retrieval, contradiction, lifecycle, and repair work must treat as authoritative.

### Canonical durable tables

| Table | Canonical identifier | Why it is authoritative | Rebuild / repair rule |
|---|---|---|---|
| `memories` | `namespace + id` | Canonical memory metadata, lifecycle, provenance-bearing fields, and stable memory identity live here | Rebuild derived indexes, caches, graph accelerators, and warm projections from these rows plus referenced durable evidence; never from cache or ANN sidecars alone |
| `memory_embeddings` | `memory_id` | Authoritative float embeddings and embedding-generation metadata belong here even when hot search uses quantized projections | Quantized vectors, ANN row ids, and search-specific encodings are disposable derived state and must be regenerated from the authoritative embedding record |
| `engrams` | `id` (`engram_id`) | Stable cluster handles and durable cluster metadata live here | Centroids and membership-derived counters may be recomputed without minting a new `engram_id` |
| `engram_members` | `(engram_id, memory_id)` | Durable, inspectable membership rows back bounded engram expansion | Membership repair must reconcile back to canonical memory and engram handles rather than trusting ANN neighborhood output alone |
| `graph_edges` | `(src_memory_id, dst_memory_id, edge_type)` | Normalized durable graph rows are the inspectable graph surface | Similarity-only accelerators may be dropped and rebuilt, but canonical contradiction, lineage, or relation meaning must still resolve through durable records |
| `causal_links` | `(src_memory_id, dst_memory_id)` | Explicit durable causal provenance rows keep "why do I believe this?" claims source-backed instead of implicit in graph traversal | Rebuild explain or traversal helpers from these rows plus their cited durable evidence; never infer canonical causality from similarity or cache state alone |
| `brain_state` | `key` | Stores durable process-level logical state such as counters or generation anchors required for reproducible repair | Restart, rebuild, and migration flows must restore these values before derived services claim healthy state |

### Stable identifier rules

- Canonical memory identity is the tuple `namespace + id`; `memory_type`, `kind`, `fingerprint`, `content_ref`, `payload_ref`, or storage location do not replace it.
- `engram_id` is a stable cluster handle for bounded graph expansion and membership repair; centroid rewrites, split bookkeeping, or ANN reindexing must not mint a new cluster identity unless lineage explicitly records a replacement.
- Sidecar, cache, FTS, or ANN-local identifiers are implementation details only. They must remain deterministic mappings back to canonical durable handles and must never become the only surviving identifier in repair or inspect flows.
- Type migration, payload relocation, redaction, compaction, or storage rewrites preserve stable identifiers unless policy explicitly requires a replacement artifact linked by lineage or supersession.

### Authoritative rebuild source contract

Derived state must rebuild from durable truth in this order:
1. canonical `memories` rows and referenced durable evidence (`content_ref`, `payload_ref`, provenance, lineage)
2. authoritative float embedding records in `memory_embeddings`
3. durable cluster, causal, and graph rows in `engrams`, `engram_members`, `causal_links`, and `graph_edges`
4. durable process state in `brain_state`

Caches, quantized vectors, FTS projections, warm routing mirrors, and other accelerators are rebuild targets, not rebuild authority.

### Migration, rollback, and schema-test expectations

- Any schema change touching these tables owes migration notes and rollback notes in the owning bead or PR packet.
- Migration coverage must prove canonical IDs survive schema motion unchanged and that derived state can be rebuilt from durable rows after restart.
- Deterministic schema tests should cover at least: primary-key stability, namespace-scoped uniqueness, foreign-key integrity, rebuild-from-durable-truth behavior, and refusal to treat caches or sidecars as authoritative.
- Repair or migration failures must preserve prior durable rows or emit explicit blocked/degraded artifacts; they must not silently widen authority to derived stores.

### Provenance, source, and lineage baseline for durable item families

Every durable item family in this schema baseline inherits the same provenance floor even when a narrower per-type field list only highlights type-specific additions.

#### Required provenance envelope

- `created_at_ms` and `updated_at_ms` are mandatory on every durable item family covered below.
- `source_kind` and `source_ref` are mandatory for first-order intake records and mandatory on derived records unless lineage points to the durable parent evidence they came from.
- `lineage` is mandatory for any derived, repaired, consolidated, summarized, extracted, merged, or contradiction-related record; first-order intake may leave it empty.
- `causal_links` and any user-visible `Causal` graph edges must carry explicit evidence attribution. At minimum one cited durable memory handle is required, and any additional supporting evidence must come from inspectable durable families such as reconsolidation audit rows, consolidation artifacts, or belief-version diffs rather than opaque traversal hints.

#### Shared source and lineage rules

- `source_kind` names the producing path or actor class such as `cli`, `mcp`, `api`, `observe`, `import`, `repair`, or `consolidation`; later bounded extensions may add values, but no durable family may omit the source path entirely.
- `source_ref` is a stable pointer to the originating request, tool call, file/span, message, job run, or imported artifact; if policy redacts the raw pointer, the durable row still needs a stable opaque handle or tombstone marker.
- `lineage` stores stable canonical parent handles rather than free-form prose and must remain inspectable across repair, migration, compaction, contradiction handling, and rollback.
- Source and lineage metadata must survive schema motion, payload relocation, and rebuild. Sidecars, caches, ANN ids, and graph-local accelerators may mirror them, but they must not become the only surviving provenance record.

#### Schema-level inheritance tests future implementation beads should carry

- fixture coverage proving each durable item family either stores `source_kind` plus `source_ref` directly or resolves them through explicit lineage to durable parent evidence
- derivation coverage proving summarize, extract, merge, repair, contradiction, and consolidation outputs keep lineage to parent memories instead of replacing ancestry with opaque text
- migration and rollback coverage proving source handles, timestamps, and lineage survive schema motion unchanged or emit explicit tombstone/loss artifacts when policy requires redaction
- inspect or repair coverage proving caches, ANN rows, graph materializations, and other accelerators cannot satisfy provenance reconstruction when the canonical durable record is missing

### Namespace, actor, visibility, retention, and policy baseline for durable item families

Every durable item family in this schema baseline also inherits the same governance floor so later interfaces do not backfill policy-critical fields after the fact.

#### Required governance envelope

- `namespace` is mandatory on every durable item family covered below and must be valid before persistence.
- `retention_class` and `policy_flags` must be representable for every durable item family even when a narrower per-type field list focuses on other payload details.
- `visibility` is mandatory whenever a record can be shared, searched outside its producer, surfaced across agent boundaries, or preserved as a policy-scoped artifact; families that stay local-only may omit it until promoted into a shared surface.

#### Shared governance and scope rules

- `agent_id` records the producing agent or job identity when one exists; it is provenance and audit scope, not an authorization shortcut.
- `namespace` is the canonical isolation boundary. No durable family may rely on `workspace_id`, `agent_id`, `session_id`, tags, or relation links as a substitute for namespace binding.
- `visibility` values must stay explicit and bounded to the canonical vocabulary `private`, `shared`, or `public`; ad hoc visibility labels or implicit widening through wrappers are out of contract.
- `retention_class` expresses intended durability such as `volatile`, `normal`, `durable`, or `pinned`; utility changes, compaction, or retrieval demand must not silently rewrite retention intent.
- `policy_flags` carry governance-critical markers such as legal hold, compliance lock, redaction state, deletion hold, and shareability constraints; policy-bearing state must remain queryable without decoding opaque payload blobs.
- Cross-namespace links, visibility widening, archival, restore, and policy-driven deletion must preserve the authoritative namespace and policy markers rather than inferring them from derived stores.

#### Schema-level governance tests future implementation beads should carry

- fixture coverage proving every durable item family persists a valid `namespace` and can expose `retention_class` plus `policy_flags` without payload hydration
- scope-binding coverage proving `agent_id` comes only from authenticated caller or job ownership metadata and never from free-form text inference
- visibility coverage proving shared-capable families reject unknown visibility labels, preserve `private/shared/public` state across migration, and do not widen visibility implicitly during repair or wrapper translation
- policy and retention coverage proving archival, restore, redaction, legal-hold, and deletion flows preserve or explicitly tombstone the prior governance markers instead of dropping them silently

### Schema migration, backfill, rollback, and doc propagation contract

Future schema-changing work must inherit one reusable obligations set instead of restating process ad hoc in each bead or PR.

#### Required work packet for any schema change

- a migration note naming changed tables, columns, meanings, compatibility window, rollout order, and rebuild or reindex surfaces affected by the change
- a backfill plan when existing rows, derived projections, or policy metadata need to be populated, rewritten, or reclassified; the plan must state whether the backfill is online, background, restartable, and bounded
- a rollback note naming the exact downgrade or restore path, what state is reversible, what must be tombstoned or explicitly loss-marked, and how stable identifiers, lineage, namespace, and policy markers survive reversal
- a doc propagation list naming every contract surface that must be updated when the schema meaning changes, including at minimum `docs/PLAN.md` when canonical contract text moves, plus `docs/DATA_SCHEMAS.md`, `docs/MEMORY_MODEL.md`, `docs/MCP_API.md`, and `docs/CLI.md` whenever the change affects those exposed surfaces

#### Backfill and rebuild rules

- Backfills must treat authoritative durable rows as the source of truth; caches, ANN sidecars, graph materializations, and warmed mirrors are rebuild targets, not input authority.
- Backfill and reindex jobs must be restart-safe, namespace-aware, and auditable; partial completion must emit degraded or pending state instead of silently presenting the schema as fully migrated.
- If a schema change needs data reclassification or policy remapping, the work packet must name the deterministic mapping rule and the artifact that proves legacy rows were handled.

#### Rollback and failure-containment rules

- Rollback must fail closed: if the prior shape cannot be restored safely, the system must preserve durable evidence and emit explicit blocked or degraded artifacts rather than pretending rollback succeeded.
- No rollback path may silently drop lineage, namespace binding, retention intent, policy markers, or contradiction state just to regain compatibility.
- Schema migration failures must name the containment boundary, including whether reads stay on the prior shape, whether writes pause, and which derived services must rebuild before healthy state resumes.

#### Deterministic migration-proof expectations

- migration fixtures should prove old and new shapes preserve stable identifiers, timestamps, lineage, namespace binding, and policy-bearing metadata exactly unless the contract explicitly emits a tombstone or loss artifact
- backfill fixtures should prove existing rows and derived stores converge to the new shape from authoritative durable evidence without requiring manual interpretation
- rollback fixtures should prove the documented reversal path restores the prior authoritative semantics or fails with explicit degraded evidence rather than silent partial rollback
- doc-propagation checks should prove every touched exposed schema surface updated its matching contract docs instead of leaving PLAN/API/CLI/model docs stale

## Schema 1: MemoryItem

### Required fields
- id
- created_at_ms
- updated_at_ms
- namespace
- version
- source_kind
- source_ref

### Optional fields
- workspace_id
- agent_id
- session_id
- task_id
- payload_ref
- tags
- lineage
- visibility
- retention_class
- policy_flags

### Validation rules
- ids must be globally unique within namespace policy
- created_at_ms must not exceed updated_at_ms
- version must increment on mutation
- payload_ref must be stable or tombstoned
- archived or cold-owned payload rows must preserve a machine-readable `payload_state` (`inline`, `detached`, `tombstoned`, or `unavailable`) plus explicit loss indicators when restore fidelity is degraded
- first-order intake rows must carry both `source_kind` and `source_ref`; derived rows may satisfy source traceability through explicit lineage only when the parent evidence remains durable and inspectable
- any non-empty `lineage` entry must resolve to stable canonical handles in the same namespace unless an explicit policy contract allows a linked cross-namespace reference
- mutation, repair, rollback, and compaction flows must preserve source traceability and lineage or emit an explicit tombstone/loss artifact instead of dropping ancestry silently
- `namespace` must be valid before persistence and must remain the authoritative isolation boundary for the row
- if `agent_id` is present it must come from authenticated caller or job ownership metadata, never from free-form content inference
- `visibility`, when the family participates in shared or policy-scoped surfaces, must stay within `private`, `shared`, or `public` and must not widen implicitly during repair, migration, or wrapper translation
- `retention_class` and `policy_flags` must survive archival, restore, redaction, and deletion-related schema motion without silent loss of governance semantics

## Schema 2: Episode

### Required fields
- id
- created_at_ms
- updated_at_ms
- namespace
- version
- source_kind
- source_ref

### Optional fields
- workspace_id
- agent_id
- session_id
- task_id
- payload_ref
- tags
- lineage
- visibility
- retention_class
- policy_flags

### Validation rules
- ids must be globally unique within namespace policy
- created_at_ms must not exceed updated_at_ms
- version must increment on mutation
- payload_ref must be stable or tombstoned
- first-order intake rows must carry both `source_kind` and `source_ref`; derived rows may satisfy source traceability through explicit lineage only when the parent evidence remains durable and inspectable
- any non-empty `lineage` entry must resolve to stable canonical handles in the same namespace unless an explicit policy contract allows a linked cross-namespace reference
- mutation, repair, rollback, and compaction flows must preserve source traceability and lineage or emit an explicit tombstone/loss artifact instead of dropping ancestry silently
- `namespace` must be valid before persistence and must remain the authoritative isolation boundary for the row
- if `agent_id` is present it must come from authenticated caller or job ownership metadata, never from free-form content inference
- `visibility`, when the family participates in shared or policy-scoped surfaces, must stay within `private`, `shared`, or `public` and must not widen implicitly during repair, migration, or wrapper translation
- `retention_class` and `policy_flags` must survive archival, restore, redaction, and deletion-related schema motion without silent loss of governance semantics

## Schema 3: Fact

### Required fields
- id
- created_at_ms
- updated_at_ms
- namespace
- version
- source_kind
- source_ref

### Optional fields
- workspace_id
- agent_id
- session_id
- task_id
- payload_ref
- tags
- lineage
- visibility
- retention_class
- policy_flags

### Validation rules
- ids must be globally unique within namespace policy
- created_at_ms must not exceed updated_at_ms
- version must increment on mutation
- payload_ref must be stable or tombstoned
- first-order intake rows must carry both `source_kind` and `source_ref`; derived rows may satisfy source traceability through explicit lineage only when the parent evidence remains durable and inspectable
- any non-empty `lineage` entry must resolve to stable canonical handles in the same namespace unless an explicit policy contract allows a linked cross-namespace reference
- mutation, repair, rollback, and compaction flows must preserve source traceability and lineage or emit an explicit tombstone/loss artifact instead of dropping ancestry silently
- `namespace` must be valid before persistence and must remain the authoritative isolation boundary for the row
- if `agent_id` is present it must come from authenticated caller or job ownership metadata, never from free-form content inference
- `visibility`, when the family participates in shared or policy-scoped surfaces, must stay within `private`, `shared`, or `public` and must not widen implicitly during repair, migration, or wrapper translation
- `retention_class` and `policy_flags` must survive archival, restore, redaction, and deletion-related schema motion without silent loss of governance semantics

## Schema 4: Summary

### Required fields
- id
- created_at_ms
- updated_at_ms
- namespace
- version
- source_kind
- source_ref

### Optional fields
- workspace_id
- agent_id
- session_id
- task_id
- payload_ref
- tags
- lineage
- visibility
- retention_class
- policy_flags

### Validation rules
- ids must be globally unique within namespace policy
- created_at_ms must not exceed updated_at_ms
- version must increment on mutation
- payload_ref must be stable or tombstoned
- first-order intake rows must carry both `source_kind` and `source_ref`; derived rows may satisfy source traceability through explicit lineage only when the parent evidence remains durable and inspectable
- any non-empty `lineage` entry must resolve to stable canonical handles in the same namespace unless an explicit policy contract allows a linked cross-namespace reference
- mutation, repair, rollback, and compaction flows must preserve source traceability and lineage or emit an explicit tombstone/loss artifact instead of dropping ancestry silently
- `namespace` must be valid before persistence and must remain the authoritative isolation boundary for the row
- if `agent_id` is present it must come from authenticated caller or job ownership metadata, never from free-form content inference
- `visibility`, when the family participates in shared or policy-scoped surfaces, must stay within `private`, `shared`, or `public` and must not widen implicitly during repair, migration, or wrapper translation
- `retention_class` and `policy_flags` must survive archival, restore, redaction, and deletion-related schema motion without silent loss of governance semantics

## Schema 5: Relation

### Required fields
- id
- created_at_ms
- updated_at_ms
- namespace
- version
- source_kind
- source_ref

### Optional fields
- workspace_id
- agent_id
- session_id
- task_id
- payload_ref
- tags
- lineage
- visibility
- retention_class
- policy_flags

### Validation rules
- ids must be globally unique within namespace policy
- created_at_ms must not exceed updated_at_ms
- version must increment on mutation
- payload_ref must be stable or tombstoned
- first-order intake rows must carry both `source_kind` and `source_ref`; derived rows may satisfy source traceability through explicit lineage only when the parent evidence remains durable and inspectable
- any non-empty `lineage` entry must resolve to stable canonical handles in the same namespace unless an explicit policy contract allows a linked cross-namespace reference
- mutation, repair, rollback, and compaction flows must preserve source traceability and lineage or emit an explicit tombstone/loss artifact instead of dropping ancestry silently
- `namespace` must be valid before persistence and must remain the authoritative isolation boundary for the row
- if `agent_id` is present it must come from authenticated caller or job ownership metadata, never from free-form content inference
- `visibility`, when the family participates in shared or policy-scoped surfaces, must stay within `private`, `shared`, or `public` and must not widen implicitly during repair, migration, or wrapper translation
- `retention_class` and `policy_flags` must survive archival, restore, redaction, and deletion-related schema motion without silent loss of governance semantics

## Schema 6: ConflictRecord

### Required fields
- id
- created_at_ms
- updated_at_ms
- namespace
- version
- source_kind
- source_ref

### Optional fields
- workspace_id
- agent_id
- session_id
- task_id
- payload_ref
- tags
- lineage
- visibility
- retention_class
- policy_flags

### Validation rules
- ids must be globally unique within namespace policy
- created_at_ms must not exceed updated_at_ms
- version must increment on mutation
- payload_ref must be stable or tombstoned
- first-order intake rows must carry both `source_kind` and `source_ref`; derived rows may satisfy source traceability through explicit lineage only when the parent evidence remains durable and inspectable
- any non-empty `lineage` entry must resolve to stable canonical handles in the same namespace unless an explicit policy contract allows a linked cross-namespace reference
- mutation, repair, rollback, and compaction flows must preserve source traceability and lineage or emit an explicit tombstone/loss artifact instead of dropping ancestry silently
- `namespace` must be valid before persistence and must remain the authoritative isolation boundary for the row
- if `agent_id` is present it must come from authenticated caller or job ownership metadata, never from free-form content inference
- `visibility`, when the family participates in shared or policy-scoped surfaces, must stay within `private`, `shared`, or `public` and must not widen implicitly during repair, migration, or wrapper translation
- `retention_class` and `policy_flags` must survive archival, restore, redaction, and deletion-related schema motion without silent loss of governance semantics

## Schema 7: Goal

### Required fields
- id
- created_at_ms
- updated_at_ms
- namespace
- version
- source_kind
- source_ref

### Optional fields
- workspace_id
- agent_id
- session_id
- task_id
- payload_ref
- tags
- lineage
- visibility
- retention_class
- policy_flags

### Validation rules
- ids must be globally unique within namespace policy
- created_at_ms must not exceed updated_at_ms
- version must increment on mutation
- payload_ref must be stable or tombstoned
- first-order intake rows must carry both `source_kind` and `source_ref`; derived rows may satisfy source traceability through explicit lineage only when the parent evidence remains durable and inspectable
- any non-empty `lineage` entry must resolve to stable canonical handles in the same namespace unless an explicit policy contract allows a linked cross-namespace reference
- mutation, repair, rollback, and compaction flows must preserve source traceability and lineage or emit an explicit tombstone/loss artifact instead of dropping ancestry silently
- `namespace` must be valid before persistence and must remain the authoritative isolation boundary for the row
- if `agent_id` is present it must come from authenticated caller or job ownership metadata, never from free-form content inference
- `visibility`, when the family participates in shared or policy-scoped surfaces, must stay within `private`, `shared`, or `public` and must not widen implicitly during repair, migration, or wrapper translation
- `retention_class` and `policy_flags` must survive archival, restore, redaction, and deletion-related schema motion without silent loss of governance semantics

## Schema 8: Skill

### Required fields
- id
- created_at_ms
- updated_at_ms
- namespace
- version
- source_kind
- source_ref

### Optional fields
- workspace_id
- agent_id
- session_id
- task_id
- payload_ref
- tags
- lineage
- visibility
- retention_class
- policy_flags

### Validation rules
- ids must be globally unique within namespace policy
- created_at_ms must not exceed updated_at_ms
- version must increment on mutation
- payload_ref must be stable or tombstoned
- first-order intake rows must carry both `source_kind` and `source_ref`; derived rows may satisfy source traceability through explicit lineage only when the parent evidence remains durable and inspectable
- any non-empty `lineage` entry must resolve to stable canonical handles in the same namespace unless an explicit policy contract allows a linked cross-namespace reference
- mutation, repair, rollback, and compaction flows must preserve source traceability and lineage or emit an explicit tombstone/loss artifact instead of dropping ancestry silently
- `namespace` must be valid before persistence and must remain the authoritative isolation boundary for the row
- if `agent_id` is present it must come from authenticated caller or job ownership metadata, never from free-form content inference
- `visibility`, when the family participates in shared or policy-scoped surfaces, must stay within `private`, `shared`, or `public` and must not widen implicitly during repair, migration, or wrapper translation
- `retention_class` and `policy_flags` must survive archival, restore, redaction, and deletion-related schema motion without silent loss of governance semantics

## Schema 9: Constraint

### Required fields
- id
- created_at_ms
- updated_at_ms
- namespace
- version
- source_kind
- source_ref

### Optional fields
- workspace_id
- agent_id
- session_id
- task_id
- payload_ref
- tags
- lineage
- visibility
- retention_class
- policy_flags

### Validation rules
- ids must be globally unique within namespace policy
- created_at_ms must not exceed updated_at_ms
- version must increment on mutation
- payload_ref must be stable or tombstoned
- first-order intake rows must carry both `source_kind` and `source_ref`; derived rows may satisfy source traceability through explicit lineage only when the parent evidence remains durable and inspectable
- any non-empty `lineage` entry must resolve to stable canonical handles in the same namespace unless an explicit policy contract allows a linked cross-namespace reference
- mutation, repair, rollback, and compaction flows must preserve source traceability and lineage or emit an explicit tombstone/loss artifact instead of dropping ancestry silently
- `namespace` must be valid before persistence and must remain the authoritative isolation boundary for the row
- if `agent_id` is present it must come from authenticated caller or job ownership metadata, never from free-form content inference
- `visibility`, when the family participates in shared or policy-scoped surfaces, must stay within `private`, `shared`, or `public` and must not widen implicitly during repair, migration, or wrapper translation
- `retention_class` and `policy_flags` must survive archival, restore, redaction, and deletion-related schema motion without silent loss of governance semantics

## Schema 10: DecayState

### Required fields
- id
- created_at_ms
- updated_at_ms
- namespace
- version
- source_kind
- source_ref

### Optional fields
- workspace_id
- agent_id
- session_id
- task_id
- payload_ref
- tags
- lineage
- visibility
- retention_class
- policy_flags

### Validation rules
- ids must be globally unique within namespace policy
- created_at_ms must not exceed updated_at_ms
- version must increment on mutation
- payload_ref must be stable or tombstoned
- first-order intake rows must carry both `source_kind` and `source_ref`; derived rows may satisfy source traceability through explicit lineage only when the parent evidence remains durable and inspectable
- any non-empty `lineage` entry must resolve to stable canonical handles in the same namespace unless an explicit policy contract allows a linked cross-namespace reference
- mutation, repair, rollback, and compaction flows must preserve source traceability and lineage or emit an explicit tombstone/loss artifact instead of dropping ancestry silently
- `namespace` must be valid before persistence and must remain the authoritative isolation boundary for the row
- if `agent_id` is present it must come from authenticated caller or job ownership metadata, never from free-form content inference
- `visibility`, when the family participates in shared or policy-scoped surfaces, must stay within `private`, `shared`, or `public` and must not widen implicitly during repair, migration, or wrapper translation
- `retention_class` and `policy_flags` must survive archival, restore, redaction, and deletion-related schema motion without silent loss of governance semantics

## Schema 11: RetentionRule

### Required fields
- id
- created_at_ms
- updated_at_ms
- namespace
- version
- source_kind
- source_ref

### Optional fields
- workspace_id
- agent_id
- session_id
- task_id
- payload_ref
- tags
- lineage
- visibility
- retention_class
- policy_flags

### Validation rules
- ids must be globally unique within namespace policy
- created_at_ms must not exceed updated_at_ms
- version must increment on mutation
- payload_ref must be stable or tombstoned
- first-order intake rows must carry both `source_kind` and `source_ref`; derived rows may satisfy source traceability through explicit lineage only when the parent evidence remains durable and inspectable
- any non-empty `lineage` entry must resolve to stable canonical handles in the same namespace unless an explicit policy contract allows a linked cross-namespace reference
- mutation, repair, rollback, and compaction flows must preserve source traceability and lineage or emit an explicit tombstone/loss artifact instead of dropping ancestry silently
- `namespace` must be valid before persistence and must remain the authoritative isolation boundary for the row
- if `agent_id` is present it must come from authenticated caller or job ownership metadata, never from free-form content inference
- `visibility`, when the family participates in shared or policy-scoped surfaces, must stay within `private`, `shared`, or `public` and must not widen implicitly during repair, migration, or wrapper translation
- `retention_class` and `policy_flags` must survive archival, restore, redaction, and deletion-related schema motion without silent loss of governance semantics

## Schema 12: ShardDescriptor

### Required fields
- id
- created_at_ms
- updated_at_ms
- namespace
- version
- source_kind
- source_ref

### Optional fields
- workspace_id
- agent_id
- session_id
- task_id
- payload_ref
- tags
- lineage
- visibility
- retention_class
- policy_flags

### Validation rules
- ids must be globally unique within namespace policy
- created_at_ms must not exceed updated_at_ms
- version must increment on mutation
- payload_ref must be stable or tombstoned
- first-order intake rows must carry both `source_kind` and `source_ref`; derived rows may satisfy source traceability through explicit lineage only when the parent evidence remains durable and inspectable
- any non-empty `lineage` entry must resolve to stable canonical handles in the same namespace unless an explicit policy contract allows a linked cross-namespace reference
- mutation, repair, rollback, and compaction flows must preserve source traceability and lineage or emit an explicit tombstone/loss artifact instead of dropping ancestry silently
- `namespace` must be valid before persistence and must remain the authoritative isolation boundary for the row
- if `agent_id` is present it must come from authenticated caller or job ownership metadata, never from free-form content inference
- `visibility`, when the family participates in shared or policy-scoped surfaces, must stay within `private`, `shared`, or `public` and must not widen implicitly during repair, migration, or wrapper translation
- `retention_class` and `policy_flags` must survive archival, restore, redaction, and deletion-related schema motion without silent loss of governance semantics

## Schema 13: Engram

### Required fields
- id
- namespace
- created_at_ms
- updated_at_ms
- version
- centroid_ref or centroid_embedding
- member_count

### Optional fields
- parent_engram_id
- total_strength
- last_activated_at_ms
- split_at_ms
- lineage
- policy_flags

### Validation rules
- `id` is the stable cluster handle; centroid or ANN-sidecar ids must not replace it
- `parent_engram_id` must resolve to another engram in the same namespace unless an explicit cross-namespace policy says otherwise
- `member_count` must match durable membership rows after repair or emit a staleness record until rebuilt
- centroid storage may be rewritten during rebuild without minting a new engram identity
- split or merge history must remain explainable through lineage or explicit parent-child linkage

## Schema 14: EngramMember

### Required fields
- engram_id
- memory_id
- namespace
- created_at_ms

### Optional fields
- similarity
- membership_source
- membership_confidence
- version

### Validation rules
- `engram_id` and `memory_id` must resolve to stable canonical handles
- membership rows must not be the sole surviving record of memory identity or lineage
- a memory may have no engram, but if it has one active primary membership the mapping must be inspectable without ANN sidecars
- membership updates must preserve auditability across split, merge, repair, or reassignment flows

## Schema 15: GraphEdge

### Required fields
- src_id
- dst_id
- namespace
- edge_type
- created_at_ms

### Optional fields
- weight
- activation_count
- source_relation_id
- source_conflict_id
- source_lineage_id
- version
- tombstone_reason

### Validation rules
- endpoints must resolve to canonical memory ids unless the edge explicitly targets another canonical graph entity class documented by contract
- `edge_type` must be one of `Associative`, `Temporal`, `Causal`, or `Contradictory` for canonical user-visible semantics
- associative weights may be rebuilt from durable evidence; contradiction, lineage, or policy-bearing meaning must not exist only in this edge row
- duplicate or contradictory edges may coexist only when their semantics remain inspectable rather than silently collapsed
- redacted, tombstoned, or superseded endpoints must remain representable with explicit markers instead of disappearing from repair or inspect surfaces
