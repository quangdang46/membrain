# STORAGE

## Tier definitions
### Tier1
In-process hot cache with bounded size.

### Tier2
Warm indexed store supporting exact and bounded hybrid retrieval.

### Tier3
Cold durable archive supporting cheap storage, metadata-first prefiltering, and reconstructable recall.

## Principles
- every tier has a different cost profile
- tier transitions must be explicit
- payloads and summaries should be separated
- rebuild must be possible from durable evidence
- indexes must be repairable
- tier1 should avoid large payload ownership

## Working-memory admission and first-write routing
- Working memory is a bounded pre-persistence controller surface for attended input. It is not authoritative durable state and it is not a synonym for Tier1 cache residency.
- Working-memory admission is gated by bounded encode-side attention or relevance checks. Intake that never crosses that gate may leave working memory without minting a canonical memory id, durable lineage, or tier assignment.
- Encode admission into canonical durable storage happens only when the candidate still qualifies for persistence at the moment the controller flushes it (for example direct typed intake, explicit durable write, or a bounded working-memory eviction or flush path). Inputs that fall below the encode gate may disappear from controller state without becoming canonical memories.
- Once an item is accepted for persistence, the first authoritative durable write lands in hot durable surfaces (`hot.db` plus bounded hot text and embedding surfaces) and only then may Tier1 or other warm accelerators be seeded.
- Ordinary online encode does not mint new canonical memories directly into `cold.db`; cold durable ownership is reached later through consolidation, archive, import, migration, or repair flows.
- Successful recall or successful slow-path retrieval may refresh Tier1 serving state, but cache residency changes alone do not change canonical durable ownership.
- Eviction from working memory or Tier1 changes only controller/cache residency. It does not by itself archive, delete, supersede, or change the canonical durable owner.
- Hot-to-cold durable movement is triggered by explicit consolidation, archive, retention, migration, or repair controllers. Promotion or demotion must therefore be inspectable as a durable controller action rather than inferred from one cache event.

## hot.db contract

### Role and ownership
- `hot.db` is the authoritative durable metadata store backing Tier2 exact and hybrid retrieval plus Tier1 promotion for memories whose canonical durable metadata still lives in the hot tier.
- It keeps the filterable hot-path record for memories that still participate in foreground retrieval, including cases where bulky payload bytes have already moved to cold storage while canonical metadata ownership remains hot.
- Once consolidation or demotion moves a memory's canonical durable metadata ownership to `cold.db`, any remaining hot-tier row for that memory is a derived query-serving mirror rather than a second authoritative source.
- It does not become the long-term owner of giant payload blobs, ANN search vectors, or other acceleration artifacts whose loss must remain repairable.

### Logical schema split

| Table family | Required contents | Authority class | Request-path role |
|---|---|---|---|
| hot memory index | stable identity, namespace, canonical type/kind, lifecycle state, current tier, version, strength/salience/decay fields, recall counters, retention and policy markers, provenance handles, contradiction or supersession handles, `content_ref`, `payload_ref`, and any bounded routing flags | authoritative durable | metadata-first prefilter and bounded candidate selection |
| hot text surface | `compact_text` plus other bounded lexical fields needed for exact or keyword retrieval and explainability | authoritative durable text surface | exact lookup and FTS source without payload fetch |
| hot embedding surface | authoritative float embeddings or durable refs to them for hot memories | authoritative durable | exact rescore source and rebuild input for the hot ANN lane |
| normalized link tables | entity, relation, lineage, contradiction, engram membership, and graph-edge rows keyed by stable ids | authoritative for canonical links and stable cluster membership handles; derived for purely similarity-driven materializations | bounded expansion, inspect, and repair |
| control metadata | schema version, migration state, repair watermarks, config generation, and any stable id mapping needed by sidecars | authoritative durable control state | safe startup, migration, and rebuild orchestration |

### Metadata layout rules
- Keep the prefilter row narrow and stable: fields needed for namespace, policy, lifecycle, ranking, and routing decisions must be queryable without touching large text or detached payload bytes.
- `compact_text` and other bounded hot lexical fields may live in `hot.db`, but giant payload ownership stays behind `payload_ref` or the cold durable store.
- `content_ref` and `payload_ref` survive compaction, demotion, redaction, and payload relocation; changing a storage handle must not mint a new identity.
- Filter-participating entity, relation, contradiction, provenance, and graph-membership metadata must remain available through normalized or otherwise directly queryable structures instead of opaque per-row blobs or ANN-only annotations.
- Opaque "all metadata in one blob" layouts are out of contract for hot-path state because they force post-filter decoding and weaken inspectability.

### Graph persistence contract
- Memory-to-memory graph endpoints persist canonical memory ids; cluster metadata persists stable `engram_id` handles plus any explicit parent or child lineage needed to explain later splits.
- Engram membership rows are authoritative for the stable mapping between a memory and its current cluster handle, while centroid vectors, traversal weights, activation counters, and similar search accelerators remain rebuildable graph state.
- Canonical relation, contradiction, lineage, and policy-bearing links may be queryable through the same normalized storage family, but graph-local traversal materializations must not become their only surviving representation.
- Index-local ids such as ANN or centroid sidecar row ids may exist for maintenance, but storage must preserve a deterministic mapping back to canonical memory or engram handles so restart and repair do not depend on opaque transient identifiers.
- If a graph surface is dropped or rebuilt, inspectable cluster membership, canonical link truth, and tombstone or supersession markers must still resolve from durable rows without replaying the whole corpus from raw payload bytes.
- Cross-namespace graph adjacency is never implied by vector proximity alone; any persisted cross-namespace linkage requires explicit policy support and remains subject to the same namespace and visibility checks as other canonical links.

### Hot-path index families
- Identity indexes must support stable `(namespace, id)` lookup plus any deterministic external-id mapping required by ANN or graph sidecars.
- Composite prefilter indexes must cover effective namespace or visibility scope, lifecycle or tier eligibility, and the rank-driving scalars used for the first bounded cut so SQLite can trim candidates before ANN, graph expansion, or payload fetch.
- Secondary indexes should support the structured filters the request path actually uses, such as canonical type or kind, engram or cluster handle, relation or entity adjacency, and recent or labile state when those are part of a bounded route.
- FTS5 or equivalent lexical projections over `compact_text` and selected bounded text fields are allowed and expected, but they are derived from authoritative hot rows rather than a second source of truth.
- Maintenance indexes for consolidation, reconsolidation windows, forgetting, and repair may exist alongside request-path indexes, but they do not justify full-store scans on foreground paths.

### Mutation and consistency rules
- Encode writes authoritative hot rows first: identity, metadata, handles, and any hot text or embedding surfaces. Only after the durable write succeeds may FTS projections, hot ANN membership, or caches be refreshed.
- `on_recall` updates recency, recall counters, strength or stability fields, and any labile-window metadata in `hot.db`, then refreshes Tier1. It must not rewrite bulky payloads or rebuild ANN state unless embedding-bearing state actually changed.
- Reconsolidation, contradiction resolution, policy changes, and schema-preserving edits must follow the memory-model versioning rules in durable rows first, then regenerate derived hot projections from those accepted rows.
- Consolidation or demotion may move bulky payload ownership and later the memory's canonical durable metadata ownership to cold surfaces. Once that ownership transfer is complete, any retained hot row is only a bounded serving mirror for identity, policy, provenance, and routing, and removal from the hot ANN lane happens only after the cold durable metadata reflects the new canonical state.
- Startup and repair rebuild hot ANN, FTS, cluster materializations, cached counts, and similar accelerators from authoritative hot rows or from cold durable truth when a retained hot row is only a serving mirror. When a derived accelerator disagrees with canonical durable ownership, the canonical durable store wins.

## cold.db contract

### Role and ownership
- `cold.db` is the authoritative durable Tier3 record for consolidated and archived memories whose canonical durable ownership has moved off the hot path.
- It owns the cold durable metadata row plus any cold-owned payload bytes, detached content objects, retention markers, and archive state needed to preserve inspectable memory truth over long horizons.
- When a memory's canonical durable ownership lives in `cold.db`, any hot-tier copy kept for bounded foreground routing is derived serving state and must be refreshed or dropped from cold durable truth.
- It does not share authority with the cold ANN sidecar, FTS projections, bloom filters, shard descriptors, caches, or other accelerators; those remain rebuildable conveniences.

### Logical schema split

| Table family | Required contents | Authority class | Request-path role |
|---|---|---|---|
| cold memory index | stable identity, namespace, canonical type/kind, lifecycle state, current tier, version, retention class, archive status, policy markers, strength/salience/decay snapshot, provenance handles, contradiction or supersession handles, `content_ref`, `payload_ref`, integrity markers, and any cold routing flags | authoritative durable | metadata-first prefilter and durable inspect anchor |
| cold payload ownership | detached payload bytes, compression or encoding metadata, chunk manifests, tombstones, and integrity data when the durable payload is owned in Tier3 | authoritative durable payload surface | final payload materialization after bounded candidate cut |
| cold text or snippet surface | bounded lexical preview, excerpt, or other inspectable text needed for explainability and exact fallback without pulling the full detached payload | authoritative durable only when stored as the canonical bounded text surface; otherwise derived | metadata-first exact fallback and inspect |
| cold embedding records | authoritative float embeddings or durable refs needed to rescore cold candidates and rebuild quantized search sidecars | authoritative durable | cold semantic rescore and rebuild input |
| archive control metadata | retention policy, hold or purge markers, migration watermarks, repair state, and audit records for archive transitions | authoritative durable control state | retention enforcement, repair, and inspect |

### Payload layout and detached handle rules
- `content_ref` names the canonical logical content handle for the memory and stays stable across compaction, archive promotion, payload rechunking, and storage-class moves unless it is explicitly tombstoned.
- `payload_ref` names the current durable payload location or manifest used to materialize the full body; it may change when bytes are recompressed, rechunked, or relocated, but the memory identity and `content_ref` must not.
- A cold durable row must preserve enough directly queryable metadata to route, filter, rank, inspect, and explain the memory without decoding the detached payload body first.
- Detached payload layouts may use inline blobs, chunk manifests, or object-style references, but each layout must remain resolvable from durable records and must carry integrity metadata plus an explicit tombstone path when content is intentionally removed.
- If policy, redaction, or retention removes payload bytes, the durable row must retain the identity, lineage, provenance, policy outcome, and tombstone reason rather than pretending the memory never existed.

### Archive and retention rules
- Archive is a durable lifecycle state, not a hidden trash bin: archived memories remain inspectable through metadata, lineage, policy, and retention surfaces even when their payload is detached, compressed, or partially withheld.
- Retention decisions must be represented in cold durable metadata with explicit class, horizon, hold, and purge eligibility markers so operators can explain why a memory remains, is frozen, or is pending deletion.
- Archiving may move payload bytes to cheaper layouts or storage classes, but it must not orphan `content_ref`, `payload_ref`, or the ability to inspect why the record exists.
- Purge or payload-drop flows must leave a durable tombstone or loss record whenever policy requires preserving the fact of prior existence, prior authority, or prior retention action.
- Inspect surfaces should be able to answer what exists, where the durable payload lives, whether it is archived or tombstoned, and which retention rule controls it without performing a full payload fetch.
- Migration and repair flows must preserve legal-hold, retention-horizon, and tombstone markers even when payload layouts, manifests, or sidecars are rewritten.
- If detached payloads or derived cold surfaces are stale, partially restored, or unavailable during repair, inspect and audit surfaces must expose explicit staleness or loss markers rather than implying the record disappeared or was fully rebuilt.

### Cold-path query and rebuild rules
- Cold retrieval starts with metadata-first SQL prefiltering over cold durable rows, using namespace, policy, lifecycle, archive, retention, and rank-driving scalars before any ANN probe or payload fetch.
- Bounded lexical previews, snippets, and other small text surfaces may participate in cold filtering and explainability, but full detached payload materialization stays deferred until after the final candidate cut.
- No cold path may require decompression or object fetch just to decide basic eligibility, contradiction state, retention visibility, or archive status.
- Startup and repair may rebuild cold ANN, FTS, manifests, and other sidecars from authoritative cold rows plus authoritative embeddings and payload ownership records.
- When `cold.db`, payload manifests, and a cold sidecar diverge, the authoritative durable record set wins; repair rebuilds the sidecar or emits explicit loss telemetry if durable payload evidence is incomplete.

## Procedural and derived-state persistence surfaces

### Surface classes

| Surface class | Typical examples | Authority class | Canonical source / rebuild source | Why it persists |
|---|---|---|---|---|
| procedural durable surface | accepted pattern→action mappings, policies, preferences, durable procedural metadata | authoritative durable when explicitly accepted as the operative rule or behavior | n/a for accepted truth; promotion or edit still keeps lineage to acceptance/source context | exact procedural lookup and policy-bearing actionability |
| derived durable artifacts | summaries, extracted facts, tentative skills, graph or neighborhood summaries, task/session checkpoints, blackboard snapshots, resumable goal-stack state, shard descriptors, compaction manifests | derived durable artifact | authoritative memories, canonical relation tables, lineage, policy/redaction state, and workflow inputs | reuse, inspect, audit, or handoff without recomputing every time |
| derived acceleration surfaces | Tier1 caches, result caches, summary caches, entity neighborhood caches, ANN probe caches, graph materializations, FTS, bloom filters, prefix indexes | derived acceleration state | authoritative durable rows plus current schema, policy, lineage, index, embedding, and model-dependent generations | latency reduction only |

### Procedural durable surface rules
- A dedicated procedural store such as `procedural.db` is allowed for stable pattern→action mappings, policies, preferences, and other procedural metadata whose value is the action rule itself rather than a summary of some other memory.
- Procedural entries that are explicitly authored, approved, or otherwise accepted as the operative rule are authoritative durable state and must survive cache drops, index rebuilds, and tier repair.
- Extracted skills synthesized from repeated episodes or outcomes are not automatically authoritative just because they are stored durably; they remain lineage-bearing derived artifacts until an explicit acceptance path promotes them.
- Skill extraction or procedural generalization must not delete the underlying episodic or semantic evidence or the source engram links that justify the rule.

### Derived durable artifact rules
- Persisted summaries, extracted facts, graph summaries, tentative skills, task or session checkpoints, blackboard snapshots, resumable goal-stack state, and compaction outputs must record artifact type, source-set or lineage handles, producer/workflow generation, freshness or repair status, and the rebuild inputs needed to regenerate them.
- Durable storage of an artifact is justified when reuse, inspectability, auditability, or background coordination is valuable; durability alone does not grant authority over memory existence, policy, contradiction, or canonical relation truth.
- Graph or neighborhood summaries may be stored for bounded inspect and recall support, but canonical entity and relation tables remain the source of truth and summary projections must be replaceable.
- If source evidence, lineage, policy, redaction, schema, or workflow generation changes, dependent derived artifacts must be invalidated, regenerated, or marked stale instead of silently surviving across the mismatch.

### Acceleration and cache rules
- Summary caches, entity neighborhood caches, ANN probe caches, Tier1 item caches, result caches, negative caches, graph materializations, and other warm surfaces remain derived acceleration state whether they live in RAM, SQLite, mmap sidecars, or separate files.
- Each family must declare its owner boundary, generation anchors, invalidation triggers, and degraded-mode fallback so repair can decide whether to bypass, rebuild, or discard it.
- Foreground correctness must not depend on any warm surface being populated; when cache state is missing, stale, or ambiguous, the system serves slower authoritative paths rather than semantically different warmed results.
- Process-local, session-local, task-local, or goal-local warm state expires with its owner boundary even if bytes are still present on disk or in memory.

### Rebuild and promotion rules
- Rebuild flows always proceed downward from authoritative durable evidence to derived durable artifacts and then to acceleration layers; they must never silently promote a stale artifact into truth because it is convenient.
- Promotion of a derived procedural artifact into authoritative procedural state requires an explicit acceptance event, retained lineage, and a durable version transition rather than an implicit side effect of repeated reuse.
- When a derived surface can be restored only partially, the system must emit explicit loss or staleness telemetry and keep the authoritative record of what was or was not reconstructed.
- A persisted derived surface that cannot explain its source lineage, freshness, owner boundary, or rebuild path is out of contract for production use.

## Compaction, repair, and rebuild expectations for persistent state

### Compaction scope and invariants
- Compaction may rewrite payload layout, chunk or manifest layout, bounded lexical previews, retained hot serving mirrors, derived durable artifacts, and drop-and-rewarm accelerators when the rewrite preserves stable memory identity and the durable-truth hierarchy.
- Compaction may coalesce or regenerate summaries, checkpoints, graph summaries, and compaction manifests only as lineage-bearing derived artifacts; it must never silently promote them into canonical memory truth.
- Compaction must preserve namespace scope, policy markers, retention or hold markers, contradiction state, lineage, canonical relation truth, `content_ref`, and any still-authoritative `payload_ref`. Repacking bytes or relocating payloads is acceptable; changing identity or authority boundaries without an explicit migration is not.
- Compaction is a storage-efficiency and rebuildability tool, not a hidden forgetting path. Payload dropping, hard deletion, or contradiction collapse still require the explicit policy or governance flow and any required tombstone or loss records.

### Rebuild classes and allowed sources
- Authoritative durable rows in hot, cold, and accepted procedural stores are not rebuilt from summaries, caches, ANN sidecars, or compaction artifacts. If those rows are damaged, recovery must come from snapshot, backup, restore, or another explicitly authoritative durable copy rather than from derived state.
- Derived durable artifacts such as summaries, task or session checkpoints, blackboard snapshots, resumable goal-stack state, graph summaries, tentative skills, and compaction manifests are rebuildable from authoritative memories, canonical relations, lineage, and policy-bearing metadata. Repair may drop and regenerate them when those rebuild inputs remain intact.
- Acceleration surfaces such as ANN, FTS, graph materializations, caches, and serving mirrors are always disposable. Repair may bypass, clear, and rewarm them without changing durable semantics.

### Repair and validation expectations
- Repair should proceed namespace-first when possible, record the authoritative source generations it used, and keep the affected surface visibly stale or degraded until post-run validation passes.
- Post-repair proof must cover count parity with durable truth, preserved lineage or policy or tombstone markers, and explicit loss or staleness records for anything that could not be reconstructed fully.
- When compaction or repair must mutate authoritative structures that live on the read path, the affected namespace or shard must move to degraded, read-only, or offline service rather than serving mixed-generation truth.

## Migration, versioning, and rollback protocol

### Schema version contract
- Every authoritative durable store, derived durable store, and persisted sidecar that participates in storage or retrieval must carry an explicit schema or format generation that can be inspected before read, repair, or rebuild work begins.
- Schema generation must be tracked separately for canonical durable rows, payload manifests, procedural durable surfaces, and persisted derived accelerators so operators can tell which layer changed and which layers only need rebuild.
- A version bump is required whenever stored field meaning, invariants, retention semantics, policy-bearing metadata, identity handles, rebuild inputs, or on-disk artifact interpretation changes.
- Additive fields that do not alter interpretation may remain backward-readable during rollout, but the generation boundary and compatibility window still must be documented explicitly.

### Migration unit and ordering rules
- Migrations must preserve stable memory identity, lineage, timestamps, namespace scope, policy markers, contradiction state, retention state, and canonical content handles across hot, cold, and procedural durable stores.
- Canonical durable schema changes land before dependent derived durable artifacts or sidecars are migrated or rebuilt; derived layers must never be upgraded first and then treated as truth.
- A migration plan must declare which surfaces are in-place transforms, copy-forward rewrites, rebuild-only sidecars, or explicit drop-and-regenerate artifacts.
- Cross-store migrations must keep old and new records interpretable long enough for bounded validation, rollback, or replay; do not create a window where neither representation is authoritative.

### Rollback contract
- Every schema or storage-behavior change must declare rollback conditions, rollback scope, and the precise durable truth the system falls back to if validation fails.
- Rollback may discard newly built sidecars, caches, and other derived surfaces freely, but it must not discard accepted authoritative durable mutations without an explicit restore or reverse-migration path.
- If a migration changes externally visible behavior, operators need both storage rollback notes and behavior rollback notes so CLI, daemon, IPC, and MCP surfaces can be restored consistently.
- When rollback cannot fully restore a partially migrated derived artifact, the system should prefer dropping or rebuilding that artifact from authoritative durable truth instead of preserving ambiguous mixed-generation state.

### Validation and repair gates
- Before a mutating migration, operators should be able to snapshot durable truth, export or back up the affected scope, and run doctor or dry-run checks appropriate to the surface.
- After migration, validation must prove schema readability, durable-record counts or invariants, policy and retention marker preservation, lineage continuity, and rebuildability of dependent indexes, caches, and graph surfaces.
- If validation fails, affected namespaces or shards may run in degraded or read-only mode while rollback or repair proceeds, but they must not serve ambiguous mixed-generation truth.
- Repair after migration follows the same durable-truth-first rule as ordinary rebuild: canonical rows win, derived artifacts are regenerated downward, and partial fidelity requires explicit loss or staleness telemetry.

### Cross-doc propagation rules
- Any schema change must update the canonical schema location in `PLAN.md`, the relevant field or lifecycle contract in `MEMORY_MODEL.md`, and any exposed CLI or MCP surfaces in `CLI.md` or `MCP_API.md` before the change is considered documented.
- Storage migrations that affect operations must also update `OPERATIONS.md` with command sequence, rollback conditions, and post-run validation expectations.
- Contributor-facing migration and rollback notes belong in `CONTRIBUTING.md`; storage-specific versioning and ordering rules belong here in `STORAGE.md`.
- If a change touches only derived accelerators or cache generations, the docs still must say whether the change is rebuild-only, requires warm-state invalidation, or requires a durable migration.

### Compatibility and coexistence rules
- Mixed-version coexistence is allowed only for an explicitly declared compatibility window with clear read/write rules; silent perpetual backward-compatibility shims are out of contract.
- Readers may tolerate older additive layouts temporarily, but writers must not emit ambiguous records whose interpretation depends on guessing the active generation.
- Namespace isolation, policy checks, retention markers, legal-hold markers, tombstones, and lineage semantics must survive migrations exactly unless the migration itself explicitly changes those semantics and records the reason.
- A migration that cannot preserve the ability to rebuild dependent derived surfaces from authoritative durable truth is unacceptable until the rebuild path is specified.

## Vector indexing contract

### Official split
- Semantic vector retrieval uses USearch-based ANN sidecars split into a bounded hot lane and a disk-backed cold lane.
- Both lanes accelerate recall only; authoritative truth stays in durable SQLite records plus authoritative embeddings.

### Hot vector lane (Tier2)
- Purpose: low-latency semantic recall over recent or partially consolidated memories.
- Residency: in-memory only and bounded by `hot_capacity` (the canonical plan examples use 50,000 vectors).
- Representation: reduced-precision search vectors chosen by benchmarked configuration; the plan's default examples use float16-style hot storage/query quantization while durable float32 embeddings remain available for exact rescore.
- Candidate budget: metadata-first SQL prefilter before ANN, capped by the configured prefilter limit (canonical plan example: 5,000 ids), then a bounded ANN shortlist (canonical plan examples: top-100 raw hits) and a smaller full-precision rescore slice (canonical plan examples: top-20) before graph expansion and final packaging.
- Mutation path: add or update on encode and any embedding-changing reconsolidation; remove when consolidation migrates a memory out of the hot semantic set.
- Persistence: derived and rebuildable, not authoritative. The hot ANN sidecar may be rebuilt from hot durable tables on daemon start or repair instead of being treated as durable truth.
- Failure contract: a missing or stale hot index may degrade latency or routing quality, but it must not lose memory existence, lineage, policy, or contradiction state.

### Cold vector lane (Tier3)
- Purpose: disk-scale semantic recall over consolidated or cold memories after a hot miss or low-confidence hot result.
- Residency: persisted mmap-backed sidecar, remapped on startup, disk-bounded rather than RAM-bounded.
- Representation: quantized search vectors optimized for mmap retrieval (the canonical plan examples use int8), while authoritative float32 embeddings or equivalent durable evidence remain available for rescore.
- Candidate budget: cold ANN probe remains bounded (canonical plan examples: top-100), and payload decompression or fetch stays deferred until after final candidate trimming and policy-safe packaging.
- Mutation path: append or update during consolidation, archive promotion, or repair rebuilds rather than on every hot write.
- Persistence: persisted derived artifact. When the sidecar diverges from durable records, `cold.db` and the authoritative embedding records win and the sidecar is rebuilt.
- Failure contract: cold sidecar corruption is a repairable operational defect, not data loss; rebuild uses durable records and emits explicit loss telemetry if exact reconstruction is impossible.

### Cross-tier rules
- Namespace and policy pruning happen before either ANN lane runs.
- No request path may force a full-store scan, pre-cut payload fetch, or bypass declared candidate budgets.
- Hot and cold indexes use the same stable ids as durable records; index-local handles must not create separate memory identities.
- Tier transitions must update durable metadata first or atomically with index mutation so repair can replay safely.
- Benchmark claims for vector retrieval must declare dimensions, quantization format, prefilter limit, ANN shortlist size, rescore slice, hardware, and warm versus cold conditions.

## Durable-versus-derived state matrix

| State or artifact | Class | Canonical store | Rebuild source | Can be sole source of truth? |
|---|---|---|---|---|
| memory identity, provenance, lineage, lifecycle, policy, contradiction state | authoritative durable | SQLite durable tables | n/a | yes |
| canonical relation edges | authoritative durable | normalized SQLite graph tables | lineage + durable records | yes |
| canonical content handles (`content_ref`, `payload_ref`) | authoritative durable | SQLite durable tables | lineage + durable records | yes |
| authoritative float embeddings | authoritative durable | durable embedding storage | source content + embedding pipeline | yes |
| summaries, extracted facts, skills | derived durable artifact | SQLite/object storage with lineage | authoritative durable evidence | no |
| task/session checkpoints, blackboard snapshots, resumable goal-stack state, shard descriptors, compaction artifacts | derived durable artifact | SQLite/object storage | authoritative durable evidence | no |
| ANN indexes, FTS projections, graph materializations, bloom filters, prefix indexes, caches | derived acceleration state | sidecars / derived tables / memory | authoritative durable evidence | no |

## Boundary rules
- Persisted does not automatically mean authoritative.
- When authoritative durable state and a derived artifact disagree, authoritative durable state wins.
- No summary, extracted fact, checkpoint, shard descriptor, or index may become the only surviving record of memory existence, lineage, policy, or contradiction semantics.
- Derived artifacts may be discarded and rebuilt; authoritative durable state may be migrated or compacted only if identity, lineage, policy, and contradiction semantics remain intact.
- If rebuild can recover only partial fidelity, the system must emit an explicit loss record instead of inventing missing truth.
