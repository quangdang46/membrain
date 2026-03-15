# INDEXING STRATEGIES

This document defines recommended indexes and when to use them.

## Index authority rule

Indexes are derived projections for retrieval speed, not canonical truth.

- An index may be persisted across restarts and still remain derived state.
- Durable records, lineage, and policy-bearing metadata remain authoritative when index output diverges.
- Every index family must be rebuildable from authoritative durable evidence with explicit telemetry and repair commands.

## Cross-cutting resilience requirements

- Namespace-bearing metadata, policy markers, and canonical ids must stay outside sidecars so rebuild, repair, or migration cannot silently widen scope or leak across namespaces.
- Rebuild and migration telemetry should record stale-index ratio, rebuild duration and source, foreground latency delta, and whether degraded or colder fallback serving was required.
- Explain, inspect, or audit surfaces should be able to distinguish healthy index participation from bypassed, stale, rebuilding, or degraded participation when those states materially affected the route.
- Duplicate amplification, oversized candidate pools, and graph-adjacent fanout must remain bounded by explicit caps and operator-visible counters rather than being discovered only after latency collapse.
- Retention, legal-hold, and other policy-bearing durable rows remain authoritative during repair, rebuild, and migration; sidecars may not erase, outrank, or reinterpret those controls.

## 1. Primary id index

### Why this index exists
Primary id index supports a distinct retrieval need and should not be merged blindly with unrelated access patterns.

### Storage notes
- choose write-friendly structures in Tier2
- choose sparse structures in Tier3
- preserve rebuildability from durable logs

### Operational notes
- track hit rate
- track stale index ratio
- track repair backlog
- expose rebuild commands

## 2. Entity inverted index

### Why this index exists
Entity inverted index supports a distinct retrieval need and should not be merged blindly with unrelated access patterns.

### Storage notes
- choose write-friendly structures in Tier2
- choose sparse structures in Tier3
- preserve rebuildability from durable logs

### Operational notes
- track hit rate
- track stale index ratio
- track repair backlog
- expose rebuild commands

## 3. Tag index

### Why this index exists
Tag index supports a distinct retrieval need and should not be merged blindly with unrelated access patterns.

### Storage notes
- choose write-friendly structures in Tier2
- choose sparse structures in Tier3
- preserve rebuildability from durable logs

### Operational notes
- track hit rate
- track stale index ratio
- track repair backlog
- expose rebuild commands

## 4. Session index

### Why this index exists
Session index supports a distinct retrieval need and should not be merged blindly with unrelated access patterns.

### Storage notes
- choose write-friendly structures in Tier2
- choose sparse structures in Tier3
- preserve rebuildability from durable logs

### Operational notes
- track hit rate
- track stale index ratio
- track repair backlog
- expose rebuild commands

## 5. Goal index

### Why this index exists
Goal index supports a distinct retrieval need and should not be merged blindly with unrelated access patterns.

### Storage notes
- choose write-friendly structures in Tier2
- choose sparse structures in Tier3
- preserve rebuildability from durable logs

### Operational notes
- track hit rate
- track stale index ratio
- track repair backlog
- expose rebuild commands

## 6. Time-bucket index

### Why this index exists
Time-bucket index supports a distinct retrieval need and should not be merged blindly with unrelated access patterns.

### Storage notes
- choose write-friendly structures in Tier2
- choose sparse structures in Tier3
- preserve rebuildability from durable logs

### Operational notes
- track hit rate
- track stale index ratio
- track repair backlog
- expose rebuild commands

## 7. Graph adjacency index

### Why this index exists
Graph adjacency index supports a distinct retrieval need and should not be merged blindly with unrelated access patterns.

### Storage notes
- choose write-friendly structures in Tier2
- choose sparse structures in Tier3
- preserve rebuildability from durable logs

### Operational notes
- track hit rate
- track stale index ratio
- track repair backlog
- expose rebuild commands

## 8. ANN index sidecar

### Why this index exists
ANN index sidecar serves semantic similarity retrieval while keeping foreground work bounded and metadata-first. It is an acceleration layer, not a replacement for durable truth.

### Storage notes
- split ANN into a hot in-memory lane for Tier2 and a cold mmap-backed lane for Tier3 rather than one monolithic semantic index
- keep canonical ids, namespace-bearing metadata, lineage, contradiction state, and authoritative embeddings outside the ANN sidecars
- use metadata-first SQL or equivalent structured prefilters before ANN search so the vector lane never becomes an implicit full-store scan
- keep hot ANN bounded by configured residency limits and eligible-memory rules instead of treating it as an ever-growing store
- keep cold ANN persisted as a derived sidecar that can be remapped quickly but rebuilt from durable records when stale or corrupt
- store search-friendly quantized representations in ANN sidecars; preserve full-fidelity durable embedding evidence for rescore and repair

### Operational notes
- track hit rate separately for hot and cold ANN lanes
- track prefilter candidate counts, ANN shortlist size, rescore slice size, and cold-escalation rate so bounded-work claims stay observable
- track stale index ratio, rebuild duration, rebuild source, and repair backlog
- expose rebuild commands for hot-only rebuilds and full cold-sidecar rebuilds
- alert when cold payload fetch or decompression happens before final candidate trimming, because that violates the request-path contract
- treat ANN divergence as a repair event: durable records win, sidecars are discarded and rebuilt if needed

## 9. Bloom filters

### Why this index exists
Bloom filters support a distinct retrieval need and should not be merged blindly with unrelated access patterns.

### Storage notes
- choose write-friendly structures in Tier2
- choose sparse structures in Tier3
- preserve rebuildability from durable logs

### Operational notes
- track hit rate
- track stale index ratio
- track repair backlog
- expose rebuild commands

## 10. Prefix indexes

### Why this index exists
Prefix indexes support a distinct retrieval need and should not be merged blindly with unrelated access patterns.

### Storage notes
- choose write-friendly structures in Tier2
- choose sparse structures in Tier3
- preserve rebuildability from durable logs

### Operational notes
- track hit rate
- track stale index ratio
- track repair backlog
- expose rebuild commands

