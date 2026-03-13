# PSEUDOCODE

## fast_encode

```text
fn fast_encode(event):
    norm = normalize(event)
    fp = fingerprint(norm)
    class = shallow_classify(norm)
    sal = provisional_salience(norm, class)
    tier = route_fast(class, sal, norm.payload_size)
    item = make_memory_item(norm, fp, class, sal, tier)
    persist(item)
    schedule_deferred_enrichment(item.id)
    return item.id
```

## retrieval_plan

```text
fn retrieval_plan(query_ctx):
    if query_ctx.id_hint:
        return ExactById
    if query_ctx.active_session and query_ctx.is_small_lookup:
        return Tier1RecentThenTier2Exact
    if query_ctx.entity_heavy:
        return Tier2EntityThenGraph
    if query_ctx.semantic_need_high:
        return Tier2HybridWithBudget
    return Tier2ExactThenTier3Fallback
```

## tier1_get

```text
fn tier1_get(key):
    slot = hot_index.lookup(key)
    if slot is None:
        return None
    item = arena.read(slot)
    if item.expired():
        return None
    return item
```

## hybrid_recall

```text
fn hybrid_recall(query):
    cands = []
    cands += exact_indexes(query)
    cands += entity_indexes(query)
    if budget_left():
        cands += ann_candidates(query)
    cands = dedup(cands)
    cands = bounded_graph_expand(cands, query)
    return rank(cands, query)
```

## decay_update

```text
fn decay_update(item, now):
    age = now - item.last_access_or_create()
    disuse = sigmoid(age / tau_age)
    penalty = disuse * (1 - item.utility_estimate)
    if item.retention_class.is_pinned():
        penalty *= 0.1
    item.decay_score = clamp(penalty, 0, 1)
    return item
```

## consolidate_episode

```text
fn consolidate_episode(events):
    cluster = sort_by_time(events)
    summary = summarize(cluster)
    facts = extract_facts(cluster)
    relations = derive_relations(cluster)
    write(summary)
    for fact in facts:
        write(fact)
    for rel in relations:
        write(rel)
    mark_cluster_consolidated(cluster.ids)
```

