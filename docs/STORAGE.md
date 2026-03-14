# STORAGE

## Tier definitions
### Tier1
In-process hot memory with bounded size.

### Tier2
Warm indexed store supporting exact and bounded hybrid retrieval.

### Tier3
Cold durable archive supporting cheap storage and reconstructable recall.

## Principles
- every tier has a different cost profile
- tier transitions must be explicit
- payloads and summaries should be separated
- rebuild must be possible from durable evidence
- indexes must be repairable
- tier1 should avoid large payload ownership

## Durable-versus-derived state matrix

| State or artifact | Class | Canonical store | Rebuild source | Can be sole source of truth? |
|---|---|---|---|---|
| memory identity, provenance, lineage, lifecycle, policy, contradiction state | authoritative durable | SQLite durable tables | n/a | yes |
| canonical relation edges | authoritative durable | normalized SQLite graph tables | lineage + durable records | yes |
| canonical content handles (`content_ref`, `payload_ref`) | authoritative durable | SQLite durable tables | lineage + durable records | yes |
| authoritative float embeddings | authoritative durable | durable embedding storage | source content + embedding pipeline | yes |
| summaries, extracted facts, skills | derived durable artifact | SQLite/object storage with lineage | authoritative durable evidence | no |
| checkpoints, shard descriptors, compaction artifacts | derived durable artifact | SQLite/object storage | authoritative durable evidence | no |
| ANN indexes, FTS projections, graph materializations, bloom filters, prefix indexes, caches | derived acceleration state | sidecars / derived tables / memory | authoritative durable evidence | no |

## Boundary rules
- Persisted does not automatically mean authoritative.
- When authoritative durable state and a derived artifact disagree, authoritative durable state wins.
- No summary, extracted fact, checkpoint, shard descriptor, or index may become the only surviving record of memory existence, lineage, policy, or contradiction semantics.
- Derived artifacts may be discarded and rebuilt; authoritative durable state may be migrated or compacted only if identity, lineage, policy, and contradiction semantics remain intact.
- If rebuild can recover only partial fidelity, the system must emit an explicit loss record instead of inventing missing truth.
