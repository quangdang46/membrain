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

### Ownership boundaries
- each cache family must declare whether it is item-local, request-local, relation-local, summary-source-local, session-local, task-local, or process-local; reuse outside that declared owner boundary is a correctness bug
- request-local caches must not be reused across normalized request-shape changes; source- or relation-derived caches must not survive source mutation or lineage repair; session, task, goal, and process-local warm state expires with its owner even if bytes still look reusable

### Invalidation and stale handling
- invalidation anchors must come from authoritative mutations and generation changes, not from best-effort TTL alone
- writes, reconsolidation, forgetting, supersession or conflict resolution, repair, rebuild, migration, policy changes, redaction changes, schema changes, index changes, embedding changes, and ranking or model version changes must invalidate or bypass affected warm state when those inputs affect the family
- when the system cannot prove a cached artifact is still valid for the current namespace, owner boundary, policy, or version inputs, it should bypass or drop that artifact instead of serving an ambiguous hit
- failed or partial background work must roll back derived updates or mark them stale until repair completes

### Rebuild and fallback
- every cache family must be droppable and rebuildable from authoritative durable inputs
- online repair may repopulate caches while foreground reads continue, but slower durable-truth reads are preferable to semantically different warmed results
- rebuild must bind fresh generation anchors before a cache is eligible for reuse again after repair, rebuild, or migration
- if a cache cannot explain its ownership boundaries, stale state, or fallback behavior, it should be treated as not production-ready

### Prefetch and warmup boundaries
- prefetch is hint-driven, optional, budget-bounded, and cancelable when user intent changes
- prefetch, session warmup, goal-conditioned warming, and cold-start mitigation must not starve foreground work or cross namespace boundaries
- no warm-path optimization may force pre-cut cold payload fetch or bypass policy, namespace pruning, or redaction behavior

### Minimum observability
- operators and explain surfaces should be able to distinguish cache hit, miss, bypass, invalidation, stale warning, repair warmup, and disabled mode
- cache participation should remain inspectable enough to separate staleness, policy filtering, and ranking behavior during debugging and resilience work

## Cache family map

| Family | Owner boundary | Accelerates | Authoritative input | Key / generation anchors | Invalidate or bypass when... |
|---|---|---|---|---|---|
| Tier1 item cache | item-local within one effective namespace | exact and recent fetches for already-authorized items | durable memory records plus policy-bearing metadata | family + effective namespace + item identity + policy or redaction generation + record generation when present | memory mutation, forgetting, supersession, or namespace, policy, redaction, or record-version change |
| Negative cache | request-local within one effective namespace | repeated structurally valid misses | durable truth under the current scoped request | family + effective namespace + normalized request shape + policy generation + schema generation when lookup semantics depend on it | writes, repair, or policy, schema, or namespace changes that could turn a miss into a hit |
| Result cache | request-local within one effective namespace | packaged recall results for a normalized request | normalized request plus bounded candidate and ranking inputs | family + effective namespace + normalized request shape + candidate-policy scope + schema generation + index generation + ranking or reranker generation | writes, repair, rebuild, or schema, index, policy, ranking, or namespace change |
| Entity neighborhood cache | relation-local within one effective namespace | graph neighborhood lookups | canonical relation tables plus lineage | family + effective namespace + root entity or relation identity + relation generation + lineage generation | relation mutation, graph repair, lineage repair, or namespace, policy, or version change |
| Summary cache | summary-source-local within one effective namespace | derived summaries and projections | durable source memories plus redaction rules | family + effective namespace + source-set identity + summary schema generation + redaction or policy generation + source generation | source mutation, repair, or redaction, policy, schema, or source-version change |
| ANN probe cache | request-local within one effective namespace and one ANN generation | vector probe shortlists | canonical embeddings plus the current ANN generation | family + effective namespace + normalized query shape + embedding generation + index or ANN generation + model generation when embedding shape depends on it | embedding, index, ANN, repair, model, or version change |
| Prefetch hints | session-local or task-local within allowed scope | speculative next-lookups | current session or task intent plus allowed scope | family + effective namespace + session or task binding + intent signature + policy generation | intent change, budget exhaustion, or namespace, policy, or session or task change |
| Session warmup | session-local within one effective namespace | session-local hot set | current session binding plus effective namespace | family + effective namespace + session binding + policy generation + version anchors for warmed families | session end, intent shift, or namespace, policy, or version change |
| Goal-conditioned cache | task-local or goal-local within allowed scope | task or goal-local shortlist | current goal or task plus allowed scope | family + effective namespace + task or goal binding + policy generation + ranking generation when relevant | goal or task change, or policy, ranking, or version change |
| Cold-start mitigation | process-local for one process generation | process-local warm artifacts | current model, index, and process generation | family + process generation + effective namespace when reused in scoped requests + model + index generation | process restart, namespace change, model change, or index generation change |

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
- version-aware invalidation
- namespace-aware keys
- bounded memory use
- stale-result observability
- cache hit and miss metrics

## 8. Session warmup

### Purpose
Session warmup accelerates early session recall by preloading a bounded hot set for the current scoped session.

### Guardrails
- version-aware invalidation
- namespace-aware keys
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
- version-aware invalidation
- namespace-aware keys
- bounded memory use
- stale-result observability
- cache hit and miss metrics
