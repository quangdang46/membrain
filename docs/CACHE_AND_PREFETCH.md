# CACHE AND PREFETCH

Caching and prefetch are useful only if they reduce tail latency without poisoning correctness.

## Canonical role

Every cache, sidecar, warm layer, and prefetch queue in retrieval is a bounded derived accelerator, not a source of truth.

- durable records, canonical embeddings, lineage, and policy-bearing metadata remain authoritative
- warm state may be dropped, bypassed, or rebuilt without changing retrieval semantics
- if a derived surface disagrees with durable truth, durable truth wins and the cache should be invalidated, repaired, or bypassed
- recall correctness must not depend on cache warmth, session warmup, goal-conditioned warming, or cold-start mitigation

## Shared contract

### Keying and scope
- cache hits are valid only after the request binds one effective namespace and passes policy checks
- every cache key should start with a family tag plus the effective namespace and any explicit widening or narrowing scope that affects visibility, such as workspace binding, session binding, task or goal binding, and normalized request shape when relevant
- keys must also include the generations that change reuse semantics, such as schema generation, policy or redaction generation, index generation, embedding generation, and model or reranker generation when those stages participate in the cached output
- shared or public widening must remain explicit in keys and explain or audit metadata; warm state must never widen a request on its own
- when a family's semantics depend on workspace binding, snapshot or as-of scope, session/task/goal ownership, or normalized request shape, those inputs belong in the cache key and generation tuple rather than in side metadata or best-effort caller memory

### Ownership boundaries
- each cache family must declare whether it is item-local, request-local, relation-local, summary-source-local, session-local, task-local, or process-local; reuse outside that declared owner boundary is a correctness bug
- request-local caches must not be reused across normalized request-shape changes; source- or relation-derived caches must not survive source mutation or lineage repair; session, task, goal, and process-local warm state expires with its owner even if bytes still look reusable

### Invalidation and stale handling
- invalidation anchors must come from authoritative mutations and generation changes, not from best-effort TTL alone
- writes, reconsolidation, forgetting, supersession or conflict resolution, repair, rebuild, migration, policy changes, redaction changes, schema changes, index changes, embedding changes, and ranking or model version changes must invalidate or bypass affected warm state when those inputs affect the family
- when the system cannot prove a cached artifact is still valid for the current namespace, owner boundary, policy, or version inputs, it should bypass or drop that artifact instead of serving an ambiguous hit
- failed or partial background work must roll back derived updates or mark them stale until repair completes
- invalidation must be selective by family and trigger set, but when the system cannot localize the affected owner boundary or generation span safely, it should invalidate or bypass the broader family rather than risk stale cross-namespace or cross-version reuse

### Rebuild and fallback
- every cache family must be droppable and rebuildable from authoritative durable inputs
- online repair may repopulate caches while foreground reads continue, but slower durable-truth reads are preferable to semantically different warmed results
- rebuild must bind fresh generation anchors before a cache is eligible for reuse again after repair, rebuild, or migration
- if a cache cannot explain its ownership boundaries, stale state, or fallback behavior, it should be treated as not production-ready
- cache warm-state repair should keep named operator hooks for both verify-only parity checks (`snapshot_current_generation_anchors`, `verify_generation_anchor_report`) and rebuild flows (`invalidate_cache_families`, `drop_prefetch_hints`, `rebuild_tier1_item_cache`, `rebuild_result_cache`, `rebuild_summary_cache`, `rebuild_ann_probe_cache`, `verify_generation_anchor_report`) so regression suites can prove which maintenance path ran

### Prefetch and warmup boundaries
- prefetch hints are speculative, owner-bound handles or shortlists keyed to the current session or task intent; they are optional, budget-bounded, and cancelable when user intent changes
- session warmup may preload only a bounded hot set for one bound session and one effective namespace; it must expire when that session or its visibility scope changes
- cold-start mitigation may prewarm process-local bootstrap artifacts, but any request-visible reuse still must bind the current effective namespace, owner boundary, and fresh generation anchors before it affects results
- prefetch, session warmup, goal-conditioned warming, and cold-start mitigation must not starve foreground work or cross namespace boundaries
- no warm-path optimization may force pre-cut cold payload fetch or bypass policy, namespace pruning, or redaction behavior

### Minimum observability
- operators and explain surfaces should be able to distinguish cache hit, miss, bypass, invalidation, stale warning, repair warmup, disabled mode, and cache-induced degraded mode
- stale or invalidated warm state must surface as an explicit warning or bypass reason rather than collapsing into an ordinary miss
- cache participation should remain inspectable enough to separate staleness, policy filtering, ranking behavior, and cold fallback during debugging and resilience work

### Explain and routing-trace integration
- machine-readable explain, inspect, and audit metadata should expose at minimum `cache_family`, `cache_event`, `cache_reason`, `warm_source`, and `generation_status` whenever cache or warm-path behavior materially affects the route
- routing traces should also preserve candidate counts before and after each cache-influenced stage so operators can tell whether latency changes came from reuse, pruning, or colder fallback
- policy-denied or redacted routes may hide protected handles or counts, but they must still preserve that a cache path was skipped, bypassed, or filtered for policy reasons
- cache-disabled or degraded-mode serving must stay distinct from an ordinary cold miss so operators and tests can attribute regressions correctly

## Cache family map

| Family | Owner boundary | Accelerates | Authoritative input | Key / generation anchors | Invalidate or bypass when... |
|---|---|---|---|---|---|
| Tier1 item cache | item-local within one effective namespace | exact and recent fetches for already-authorized items | durable memory records plus policy-bearing metadata | family + effective namespace + optional workspace binding when visibility depends on it + item identity + policy or redaction generation + record generation when present | memory mutation, forgetting, supersession, or namespace, workspace, policy, redaction, or record-version change |
| Negative cache | request-local within one effective namespace | repeated structurally valid misses | durable truth under the current scoped request | family + effective namespace + optional workspace binding + normalized request shape + policy generation + schema generation when lookup semantics depend on it | writes, repair, or workspace, policy, schema, or namespace changes that could turn a miss into a hit |
| Result cache | request-local within one effective namespace | packaged recall results for a normalized request | normalized request plus bounded candidate and ranking inputs | family + effective namespace + optional workspace binding + normalized request shape + candidate-policy scope + snapshot or as-of scope when present + schema generation + index generation + ranking or reranker generation | writes, repair, rebuild, or snapshot, schema, index, workspace, policy, ranking, or namespace change |
| Entity neighborhood cache | relation-local within one effective namespace | graph neighborhood lookups | canonical relation tables plus lineage | family + effective namespace + optional workspace binding + root entity or relation identity + relation generation + lineage generation | relation mutation, graph repair, lineage repair, or namespace, workspace, policy, or version change |
| Summary cache | summary-source-local within one effective namespace | derived summaries and projections | durable source memories plus redaction rules | family + effective namespace + optional workspace binding + source-set identity + summary schema generation + redaction or policy generation + source generation | source mutation, repair, or redaction, workspace, policy, schema, or source-version change |
| ANN probe cache | request-local within one effective namespace and one ANN generation | vector probe shortlists | canonical embeddings plus the current ANN generation | family + effective namespace + optional workspace binding + normalized query shape + snapshot or as-of scope when present + embedding generation + index or ANN generation + model generation when embedding shape depends on it | embedding, index, ANN, repair, model, snapshot, workspace, or version change |
| Prefetch hints | session-local or task-local within allowed scope | speculative next-lookups | current session or task intent plus allowed scope | family + effective namespace + optional workspace binding + session or task binding + intent signature + policy generation | intent change, budget exhaustion, or namespace, workspace, policy, or session or task change |
| Session warmup | session-local within one effective namespace | session-local hot set | current session binding plus effective namespace | family + effective namespace + optional workspace binding + session binding + policy generation + version anchors for warmed families | session end, intent shift, or namespace, workspace, policy, or version change |
| Goal-conditioned cache | task-local or goal-local within allowed scope | task or goal-local shortlist | current goal or task plus allowed scope | family + effective namespace + optional workspace binding + task or goal binding + policy generation + ranking generation when relevant | goal or task change, or workspace, policy, ranking, or version change |
| Cold-start mitigation | process-local for one process generation | process-local warm artifacts | current model, index, and process generation | family + process generation + effective namespace when reused in scoped requests + optional workspace binding when request-visible reuse depends on it + model + index generation | process restart, namespace change, workspace change, model change, or index generation change |

## 1. Tier1 item cache

### Purpose
Tier1 item cache accelerates exact and recent retrieval for already-authorized hot memories.

### Guardrails
- version-aware invalidation
- namespace-aware keys
- bounded memory use
- stale-result observability
- cache hit and miss metrics

## 2. Negative cache

### Purpose
Negative cache accelerates repeated scoped misses without turning policy denials into synthetic absence.

### Guardrails
- version-aware invalidation
- namespace-aware keys
- bounded memory use
- stale-result observability
- cache hit and miss metrics

## 3. Result cache

### Purpose
Result cache accelerates reuse of packaged recall results for the same normalized request shape and scope.

### Guardrails
- version-aware invalidation
- namespace-aware keys
- bounded memory use
- stale-result observability
- cache hit and miss metrics

## 4. Entity neighborhood cache

### Purpose
Entity neighborhood cache accelerates bounded graph-neighborhood lookup derived from canonical relation state.

### Guardrails
- version-aware invalidation
- namespace-aware keys
- bounded memory use
- stale-result observability
- cache hit and miss metrics

## 5. Summary cache

### Purpose
Summary cache accelerates reuse of derived summaries and projections without replacing durable evidence.

### Guardrails
- version-aware invalidation
- namespace-aware keys
- bounded memory use
- stale-result observability
- cache hit and miss metrics

## 6. ANN probe cache

### Purpose
ANN probe cache accelerates repeated vector-search shortlist generation for the current index generation.

### Guardrails
- version-aware invalidation
- namespace-aware keys
- bounded memory use
- stale-result observability
- cache hit and miss metrics

## 7. Prefetch hints

### Purpose
Prefetch hints accelerate likely next lookups as speculative handles, not authoritative payloads.

### Guardrails
- session-local or task-local ownership only; hints must not be reused outside the bound intent owner
- keys should include family + effective namespace + bound session or task + intent signature + relevant policy generation
- authoritative intent change, session or task rebinding, policy change, or budget exhaustion should cancel, bypass, or drop queued hints
- prefetch may speculate on handles or shortlist candidates only; payload materialization still waits for the final bounded request path
- bounded memory use
- stale-result observability
- cache hit and miss metrics

## 8. Session warmup

### Purpose
Session warmup accelerates early session recall by preloading a bounded hot set for the current scoped session.

### Guardrails
- session-local ownership within one effective namespace; warmed state expires with the bound session even if bytes remain resident
- keys should include family + effective namespace + session binding + policy generation + fresh generation anchors for every warmed family the session reuses
- authoritative session rebinding, namespace widening or narrowing, policy change, or warmed-family generation shift must invalidate or bypass the warmed set
- session warmup may improve early latency, but foreground retrieval must fall back to colder authoritative paths whenever warmed state is missing, stale, or scoped too broadly
- bounded memory use
- stale-result observability
- cache hit and miss metrics

## 9. Goal-conditioned cache

### Purpose
Goal-conditioned cache accelerates repeated retrieval for the current task or goal without widening scope or becoming durable truth.

### Guardrails
- version-aware invalidation
- namespace-aware keys
- bounded memory use
- stale-result observability
- cache hit and miss metrics

## 10. Cold-start mitigation

### Purpose
Cold-start mitigation accelerates startup and first-query behavior through discardable warm artifacts only.

### Guardrails
- process-local ownership only; bootstrap artifacts may survive within one process generation but must not masquerade as durable shared state
- keys should include family + process generation + effective namespace when request-visible reuse occurs + model generation + index generation
- process restart, model or index generation shift, namespace rebinding, or uncertain repair state must invalidate, bypass, or rebuild mitigations before reuse
- cold-start mitigation may preload bootstrap handles, indexes, or models, but request-serving paths still need fresh owner-bound checks and generation anchors before warmed artifacts affect results
- bounded memory use
- stale-result observability
- cache hit and miss metrics
# Cache event taxonomy
## Cache event taxonomy

### cache_family

| Family | Description |
|--------|-------------|
| `tier1_item` | Exact and recent fetches for already-authorized items |
| `negative_cache` | Repeated structurally valid misses under current policy scope |
| `result_cache` | Packaged recall results for normalized request shape |
| `entity_neighborhood` | Graph neighborhood lookups derived from canonical relation tables |
| `summary_cache` | Derived summaries and projections |
| `ann_probe_cache` | Vector-search shortlist generation for current index generation |
| `prefetch_hints` | Speculative handles or shortlists keyed to session or task intent |
| `session_warmup` | Session-local hot set for current scoped session |
| `goal_conditioned` | Task or goal-local shortlist for repeated retrieval |
| `cold_start_mitigation` | Process-local bootstrap warm artifacts only |

### cache_event

| Event | Description |
|--------|-------------|
| `hit` | Cache returned a valid entry for current request namespace, owner boundary, policy, and generation anchors |
| `miss` | No valid entry found in cache for current key and scope |
| `bypass` | Cache exists but was skipped due to staleness, version mismatch, scope mismatch, or owner-boundary violation |
| `invalidation` | Cache entry invalidated due to authoritative mutation or generation change |
| `repair_warmup` | Cache populated or refreshed during repair operation |
| `stale_warning` | Cache entry exists but was rejected as stale and result came from colder path |
| `disabled` | Cache is explicitly disabled or in degraded mode for current request |
| `prefetch_drop` | Prefetch hint canceled due to intent change, budget exhaustion, or policy scope change |
| `session_expired` | Session warmup invalidated due to session end or scope change |

### cache_reason

| Reason | Description |
|---------|-------------|
| `owner_boundary_mismatch` | Cache entry owner boundary does not match current request owner |
| `namespace_mismatch` | Cache entry namespace does not match current effective namespace |
| `policy_denied` | Cache entry not authorized under current policy scope |
| `generation_anchor_mismatch` | Cache entry lacks fresh generation anchor for current request |
| `version_mismatch` | Cache entry belongs to different schema, index, embedding, or model generation |
| `scope_too_broad` | Cache entry scoped more broadly than current request allows |
| `record_not_present` | Referenced memory record no longer exists in authoritative store |
| `repair_incomplete` | Cache entry from failed or partial repair operation |
| `policy_changed` | Cache entry invalidated due to policy rule change |
| `redaction_changed` | Cache entry invalidated due to redaction rules update |
| `schema_changed` | Cache entry invalidated due to schema generation change |
| `index_changed` | Cache entry invalidated due to index generation change |
| `embedding_changed` | Cache entry invalidated due to embedding generation change |
| `ranking_changed` | Cache entry invalidated due to reranker model version change |
| `intent_changed` | Prefetch or warmup invalidated due to session or task intent change |
| `budget_exhausted` | Prefetch canceled due to budget exhaustion |
| `namespace_narrowed` | Session warmup invalidated due to namespace scope narrowing |
| `namespace_widened` | Session warmup invalidated due to namespace scope widening |

### warm_source

| Source | Description |
|---------|-------------|
| `tier1_item_cache` | Hot metadata store for exact-handle and recent-item lookups |
| `negative_cache` | Request-local cache for structurally valid misses |
| `result_cache` | Request-local cache for packaged recall results |
| `entity_neighborhood` | Relation-local cache derived from canonical relation tables |
| `summary_cache` | Summary-source-local cache for derived summaries and projections |
| `ann_probe_cache` | Request-local cache for ANN shortlist probes |
| `prefetch_queue` | Session-local or task-local speculative lookup queue |
| `session_warmup` | Session-local hot set |
| `goal_cache` | Task-local or goal-local shortlist cache |
| `cold_start_cache` | Process-local bootstrap artifact cache |

### generation_status

| Status | Description |
|---------|-------------|
| `valid` | Cache entry has all required generation anchors for current request |
| `stale` | Cache entry exists but lacks fresh generation anchor |
| `version_mismatched` | Cache entry generation version differs from current required version |
| `unknown` | Cache entry generation status cannot be determined |

## Metrics structure for testing

### Per-request metrics

All cache-influenced requests must populate the `metrics` object in the common response envelope with at minimum the following counters:

| Field | Type | Description | Required when... |
|-------|------|-------------|-----------------|
| `cache_hit_count` | integer | Number of cache hits across all families for this request | Always |
| `cache_miss_count` | integer | Number of cache misses across all families for this request | Always |
| `cache_bypass_count` | integer | Number of cache entries bypassed due to staleness, version mismatch, or scope violation | When any bypass occurs |
| `cache_invalidation_count` | integer | Number of cache entries invalidated for this request (may be 0) | When invalidation occurs |
| `prefetch_used_count` | integer | Number of prefetch hints consumed for this request | When prefetch is used |
| `prefetch_dropped_count` | integer | Number of prefetch hints canceled for this request | When prefetch drops occur |
| `cold_fallback_count` | integer | Number of times request escalated from cache to colder authoritative path | When cold fallback occurs |
| `degraded_mode_served` | boolean | `true` if response was served while system was in degraded mode | When degraded mode is active |

### Candidate count preservation

Routing traces must preserve candidate counts at key checkpoints to distinguish cache acceleration from cold-path work:

| Checkpoint | Description | Required for... |
|-----------|-------------|-----------------|
| `pre_cache_candidates` | integer | Candidates available before any cache lookup | Always |
| `post_tier1_candidates` | integer | Candidates after Tier1 cache evaluation | When Tier1 is active |
| `post_tier2_candidates` | integer | Candidates after Tier2 cache evaluation | When Tier2 is active |
| `post_ann_candidates` | integer | Candidates after ANN probe cache evaluation | When ANN cache is active |
| `prefetch_added_candidates` | integer | Candidates added by prefetch queue for this request | When prefetch contributes |

### Per-family breakdown

When multiple cache families participate in a request, metrics must distinguish contributions by family:

| Field | Type | Description | Required when... |
|-------|------|-------------|-----------------|
| `tier1_item_hit_count` | integer | Hits from Tier1 item cache | When Tier1 is queried |
| `negative_cache_hit_count` | integer | Hits from negative cache | When negative cache is queried |
| `result_cache_hit_count` | integer | Hits from result cache | When result cache is active |
| `entity_neighborhood_hit_count` | integer | Hits from entity neighborhood cache | When graph expansion is active |
| `summary_cache_hit_count` | integer | Hits from summary cache | When summary cache is queried |
| `ann_probe_hit_count` | integer | Hits from ANN probe cache | When ANN search is active |
| `prefetch_hit_count` | integer | Hits from prefetch queue | When prefetch is used |

## Regression artifacts for testing

### Distinguishable outcomes

Tests must be able to distinguish following cache-related outcomes without inspecting internal state:

| Outcome | How to distinguish | Test evidence required |
|---------|-----------------|---------------------|
| Cache hit vs cold miss | Cache `metrics.cache_hit_count > 0` vs `metrics.cache_miss_count > 0` and `cache_event=hit` in trace | `metrics` object populated with hit/miss counts |
| Cache bypass | `metrics.cache_bypass_count > 0` and explicit `cache_reason` in `trace_stages` | `cache_event=bypass` with reason in trace |
| Stale-result warning | `cache_event=stale_warning` in `trace_stages` or explicit `stale_warning` in `warnings` | Trace contains stale warning or warnings array |
| Disabled or degraded mode | `metrics.degraded_mode_served=true` or `cache_event=disabled` | `degraded_mode_served` flag or disabled event in trace |
| Prefetch vs authoritative | `metrics.prefetch_used_count > 0` vs same candidates returned from authoritative path | Both counters present in `metrics` |
| Cache invalidation | `metrics.cache_invalidation_count > 0` or `cache_event=invalidation` in trace | Invalidation count or event in trace |

### Required coverage

Every cache observability change must include regression coverage for:

1. **Hit/miss visibility by family**: Each cache family must emit hit/miss counts that can be distinguished in `metrics` output
2. **Stale-result bypass**: Stale cache entries must surface as `bypass` events with explicit reason, not silent `miss`
3. **Degraded mode serving**: When cache is disabled or degraded, `degraded_mode_served` must be `true` and distinguishable from ordinary cold miss
4. **Candidate count preservation**: `pre_cache_candidates`, `post_tier2_candidates`, etc. must be preserved in traces when those families participate
5. **Prefetch transparency**: `prefetch_used_count` and `prefetch_dropped_count` must both be populated when prefetch is configured
6. **Per-family breakdown**: Individual family hit counts must be available when multiple families participate
7. **Parity across interfaces**: Cache metadata (`cache_family`, `cache_event`, `cache_reason`, `warm_source`, `generation_status`) must be identical across CLI, daemon, and MCP explain/inspect surfaces

### Test scenario examples

| Scenario | Expected metrics behavior |
|-----------|------------------------|
| Fresh cache hit | `cache_hit_count=1, cache_miss_count=0, cache_bypass_count=0`, `cache_event=hit`, relevant family hit count incremented |
| Cache miss with no cache | `cache_hit_count=0, cache_miss_count=1, cache_bypass_count=0`, `cache_event=miss` in trace |
| Stale cache bypass | `cache_hit_count=0, cache_miss_count=0, cache_bypass_count=1`, `cache_event=bypass` with `cache_reason=stale_warning` |
| Prefetch hit + cold fallback | `cache_hit_count=1, cold_fallback_count=1`, `cache_event=hit` for prefetch, later `cold_fallback_count` incremented |
| Degraded mode serving | `degraded_mode_served=true`, `cache_event=disabled` or explicit `degraded` warning in `warnings` |
| Cache invalidation mid-request | `cache_hit_count=1, cache_invalidation_count=1`, both `hit` and `invalidation` events in `trace_stages` |

### Integration with trace_stages

When `explain=full` is requested or explicit inspect mode is active, cache-related events must appear in `trace_stages` with the following structure:

| Field | Type | Description |
|-------|------|-------------|
| `stage` | string | Stage name (e.g., `tier1_cache_eval`, `ann_probe_cache_eval`) |
| `cache_family` | string | Which cache family participated (from taxonomy) |
| `cache_event` | string | Event type (`hit`, `miss`, `bypass`, `invalidation`, `stale_warning`, `disabled`, `prefetch_drop`, `session_expired`) |
| `cache_reason` | string | Reason for bypass or invalidation when applicable (from taxonomy) |
| `warm_source` | string | Which warm source provided to cached entry (from taxonomy) |
| `generation_status` | string | Generation status validation result (from taxonomy) |
| `candidates_before` | integer | Candidate count before cache evaluation at this stage |
| `candidates_after` | integer | Candidate count after cache evaluation at this stage |
