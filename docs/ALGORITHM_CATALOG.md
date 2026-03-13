# ALGORITHM CATALOG

## 1. Fast Path Retrieval pattern 1

### Objective
Optimize fast path retrieval under bounded latency and bounded noise.

### Preconditions
- namespace is validated
- policy context is loaded
- latency budget is available
- tier capabilities are known

### Heuristic
1. start with the cheapest exact signals
2. cap expansion by query class
3. only pay semantic cost when exact and graph signals are insufficient
4. preserve explainability and provenance
5. update reinforcement or decay after outcome feedback

### Failure risks
- candidate explosion
- stale data bias
- low-confidence amplification
- cross-session interference

### Instrumentation
- p50/p95/p99 latency
- candidate counts before and after pruning
- success contribution estimate
- cache hit ratio
- repairability score

### Pseudocode
```text
fn pattern_1(ctx):
    budget = ctx.latency_budget
    exact = cheap_exact(ctx)
    graph = bounded_graph(ctx, exact)
    sem = maybe_semantic(ctx, budget, graph)
    merged = dedup_merge(exact, graph, sem)
    ranked = rank_with_policy(ctx, merged)
    return package(ctx, ranked[:ctx.limit])
```
## 2. Fast Path Retrieval pattern 2

### Objective
Optimize fast path retrieval under bounded latency and bounded noise.

### Preconditions
- namespace is validated
- policy context is loaded
- latency budget is available
- tier capabilities are known

### Heuristic
1. start with the cheapest exact signals
2. cap expansion by query class
3. only pay semantic cost when exact and graph signals are insufficient
4. preserve explainability and provenance
5. update reinforcement or decay after outcome feedback

### Failure risks
- candidate explosion
- stale data bias
- low-confidence amplification
- cross-session interference

### Instrumentation
- p50/p95/p99 latency
- candidate counts before and after pruning
- success contribution estimate
- cache hit ratio
- repairability score

### Pseudocode
```text
fn pattern_2(ctx):
    budget = ctx.latency_budget
    exact = cheap_exact(ctx)
    graph = bounded_graph(ctx, exact)
    sem = maybe_semantic(ctx, budget, graph)
    merged = dedup_merge(exact, graph, sem)
    ranked = rank_with_policy(ctx, merged)
    return package(ctx, ranked[:ctx.limit])
```
## 3. Fast Path Retrieval pattern 3

### Objective
Optimize fast path retrieval under bounded latency and bounded noise.

### Preconditions
- namespace is validated
- policy context is loaded
- latency budget is available
- tier capabilities are known

### Heuristic
1. start with the cheapest exact signals
2. cap expansion by query class
3. only pay semantic cost when exact and graph signals are insufficient
4. preserve explainability and provenance
5. update reinforcement or decay after outcome feedback

### Failure risks
- candidate explosion
- stale data bias
- low-confidence amplification
- cross-session interference

### Instrumentation
- p50/p95/p99 latency
- candidate counts before and after pruning
- success contribution estimate
- cache hit ratio
- repairability score

### Pseudocode
```text
fn pattern_3(ctx):
    budget = ctx.latency_budget
    exact = cheap_exact(ctx)
    graph = bounded_graph(ctx, exact)
    sem = maybe_semantic(ctx, budget, graph)
    merged = dedup_merge(exact, graph, sem)
    ranked = rank_with_policy(ctx, merged)
    return package(ctx, ranked[:ctx.limit])
```
## 4. Fast Path Retrieval pattern 4

### Objective
Optimize fast path retrieval under bounded latency and bounded noise.

### Preconditions
- namespace is validated
- policy context is loaded
- latency budget is available
- tier capabilities are known

### Heuristic
1. start with the cheapest exact signals
2. cap expansion by query class
3. only pay semantic cost when exact and graph signals are insufficient
4. preserve explainability and provenance
5. update reinforcement or decay after outcome feedback

### Failure risks
- candidate explosion
- stale data bias
- low-confidence amplification
- cross-session interference

### Instrumentation
- p50/p95/p99 latency
- candidate counts before and after pruning
- success contribution estimate
- cache hit ratio
- repairability score

### Pseudocode
```text
fn pattern_4(ctx):
    budget = ctx.latency_budget
    exact = cheap_exact(ctx)
    graph = bounded_graph(ctx, exact)
    sem = maybe_semantic(ctx, budget, graph)
    merged = dedup_merge(exact, graph, sem)
    ranked = rank_with_policy(ctx, merged)
    return package(ctx, ranked[:ctx.limit])
```
## 5. Fast Path Retrieval pattern 5

### Objective
Optimize fast path retrieval under bounded latency and bounded noise.

### Preconditions
- namespace is validated
- policy context is loaded
- latency budget is available
- tier capabilities are known

### Heuristic
1. start with the cheapest exact signals
2. cap expansion by query class
3. only pay semantic cost when exact and graph signals are insufficient
4. preserve explainability and provenance
5. update reinforcement or decay after outcome feedback

### Failure risks
- candidate explosion
- stale data bias
- low-confidence amplification
- cross-session interference

### Instrumentation
- p50/p95/p99 latency
- candidate counts before and after pruning
- success contribution estimate
- cache hit ratio
- repairability score

### Pseudocode
```text
fn pattern_5(ctx):
    budget = ctx.latency_budget
    exact = cheap_exact(ctx)
    graph = bounded_graph(ctx, exact)
    sem = maybe_semantic(ctx, budget, graph)
    merged = dedup_merge(exact, graph, sem)
    ranked = rank_with_policy(ctx, merged)
    return package(ctx, ranked[:ctx.limit])
```
## 6. Fast Path Retrieval pattern 6

### Objective
Optimize fast path retrieval under bounded latency and bounded noise.

### Preconditions
- namespace is validated
- policy context is loaded
- latency budget is available
- tier capabilities are known

### Heuristic
1. start with the cheapest exact signals
2. cap expansion by query class
3. only pay semantic cost when exact and graph signals are insufficient
4. preserve explainability and provenance
5. update reinforcement or decay after outcome feedback

### Failure risks
- candidate explosion
- stale data bias
- low-confidence amplification
- cross-session interference

### Instrumentation
- p50/p95/p99 latency
- candidate counts before and after pruning
- success contribution estimate
- cache hit ratio
- repairability score

### Pseudocode
```text
fn pattern_6(ctx):
    budget = ctx.latency_budget
    exact = cheap_exact(ctx)
    graph = bounded_graph(ctx, exact)
    sem = maybe_semantic(ctx, budget, graph)
    merged = dedup_merge(exact, graph, sem)
    ranked = rank_with_policy(ctx, merged)
    return package(ctx, ranked[:ctx.limit])
```
## 7. Fast Path Retrieval pattern 7

### Objective
Optimize fast path retrieval under bounded latency and bounded noise.

### Preconditions
- namespace is validated
- policy context is loaded
- latency budget is available
- tier capabilities are known

### Heuristic
1. start with the cheapest exact signals
2. cap expansion by query class
3. only pay semantic cost when exact and graph signals are insufficient
4. preserve explainability and provenance
5. update reinforcement or decay after outcome feedback

### Failure risks
- candidate explosion
- stale data bias
- low-confidence amplification
- cross-session interference

### Instrumentation
- p50/p95/p99 latency
- candidate counts before and after pruning
- success contribution estimate
- cache hit ratio
- repairability score

### Pseudocode
```text
fn pattern_7(ctx):
    budget = ctx.latency_budget
    exact = cheap_exact(ctx)
    graph = bounded_graph(ctx, exact)
    sem = maybe_semantic(ctx, budget, graph)
    merged = dedup_merge(exact, graph, sem)
    ranked = rank_with_policy(ctx, merged)
    return package(ctx, ranked[:ctx.limit])
```
## 8. Fast Path Retrieval pattern 8

### Objective
Optimize fast path retrieval under bounded latency and bounded noise.

### Preconditions
- namespace is validated
- policy context is loaded
- latency budget is available
- tier capabilities are known

### Heuristic
1. start with the cheapest exact signals
2. cap expansion by query class
3. only pay semantic cost when exact and graph signals are insufficient
4. preserve explainability and provenance
5. update reinforcement or decay after outcome feedback

### Failure risks
- candidate explosion
- stale data bias
- low-confidence amplification
- cross-session interference

### Instrumentation
- p50/p95/p99 latency
- candidate counts before and after pruning
- success contribution estimate
- cache hit ratio
- repairability score

### Pseudocode
```text
fn pattern_8(ctx):
    budget = ctx.latency_budget
    exact = cheap_exact(ctx)
    graph = bounded_graph(ctx, exact)
    sem = maybe_semantic(ctx, budget, graph)
    merged = dedup_merge(exact, graph, sem)
    ranked = rank_with_policy(ctx, merged)
    return package(ctx, ranked[:ctx.limit])
```
## 9. Fast Path Retrieval pattern 9

### Objective
Optimize fast path retrieval under bounded latency and bounded noise.

### Preconditions
- namespace is validated
- policy context is loaded
- latency budget is available
- tier capabilities are known

### Heuristic
1. start with the cheapest exact signals
2. cap expansion by query class
3. only pay semantic cost when exact and graph signals are insufficient
4. preserve explainability and provenance
5. update reinforcement or decay after outcome feedback

### Failure risks
- candidate explosion
- stale data bias
- low-confidence amplification
- cross-session interference

### Instrumentation
- p50/p95/p99 latency
- candidate counts before and after pruning
- success contribution estimate
- cache hit ratio
- repairability score

### Pseudocode
```text
fn pattern_9(ctx):
    budget = ctx.latency_budget
    exact = cheap_exact(ctx)
    graph = bounded_graph(ctx, exact)
    sem = maybe_semantic(ctx, budget, graph)
    merged = dedup_merge(exact, graph, sem)
    ranked = rank_with_policy(ctx, merged)
    return package(ctx, ranked[:ctx.limit])
```
## 10. Fast Path Retrieval pattern 10

### Objective
Optimize fast path retrieval under bounded latency and bounded noise.

### Preconditions
- namespace is validated
- policy context is loaded
- latency budget is available
- tier capabilities are known

### Heuristic
1. start with the cheapest exact signals
2. cap expansion by query class
3. only pay semantic cost when exact and graph signals are insufficient
4. preserve explainability and provenance
5. update reinforcement or decay after outcome feedback

### Failure risks
- candidate explosion
- stale data bias
- low-confidence amplification
- cross-session interference

### Instrumentation
- p50/p95/p99 latency
- candidate counts before and after pruning
- success contribution estimate
- cache hit ratio
- repairability score

### Pseudocode
```text
fn pattern_10(ctx):
    budget = ctx.latency_budget
    exact = cheap_exact(ctx)
    graph = bounded_graph(ctx, exact)
    sem = maybe_semantic(ctx, budget, graph)
    merged = dedup_merge(exact, graph, sem)
    ranked = rank_with_policy(ctx, merged)
    return package(ctx, ranked[:ctx.limit])
```
## 11. Fast Path Retrieval pattern 11

### Objective
Optimize fast path retrieval under bounded latency and bounded noise.

### Preconditions
- namespace is validated
- policy context is loaded
- latency budget is available
- tier capabilities are known

### Heuristic
1. start with the cheapest exact signals
2. cap expansion by query class
3. only pay semantic cost when exact and graph signals are insufficient
4. preserve explainability and provenance
5. update reinforcement or decay after outcome feedback

### Failure risks
- candidate explosion
- stale data bias
- low-confidence amplification
- cross-session interference

### Instrumentation
- p50/p95/p99 latency
- candidate counts before and after pruning
- success contribution estimate
- cache hit ratio
- repairability score

### Pseudocode
```text
fn pattern_11(ctx):
    budget = ctx.latency_budget
    exact = cheap_exact(ctx)
    graph = bounded_graph(ctx, exact)
    sem = maybe_semantic(ctx, budget, graph)
    merged = dedup_merge(exact, graph, sem)
    ranked = rank_with_policy(ctx, merged)
    return package(ctx, ranked[:ctx.limit])
```
## 12. Fast Path Retrieval pattern 12

### Objective
Optimize fast path retrieval under bounded latency and bounded noise.

### Preconditions
- namespace is validated
- policy context is loaded
- latency budget is available
- tier capabilities are known

### Heuristic
1. start with the cheapest exact signals
2. cap expansion by query class
3. only pay semantic cost when exact and graph signals are insufficient
4. preserve explainability and provenance
5. update reinforcement or decay after outcome feedback

### Failure risks
- candidate explosion
- stale data bias
- low-confidence amplification
- cross-session interference

### Instrumentation
- p50/p95/p99 latency
- candidate counts before and after pruning
- success contribution estimate
- cache hit ratio
- repairability score

### Pseudocode
```text
fn pattern_12(ctx):
    budget = ctx.latency_budget
    exact = cheap_exact(ctx)
    graph = bounded_graph(ctx, exact)
    sem = maybe_semantic(ctx, budget, graph)
    merged = dedup_merge(exact, graph, sem)
    ranked = rank_with_policy(ctx, merged)
    return package(ctx, ranked[:ctx.limit])
```
## 13. Fast Path Retrieval pattern 13

### Objective
Optimize fast path retrieval under bounded latency and bounded noise.

### Preconditions
- namespace is validated
- policy context is loaded
- latency budget is available
- tier capabilities are known

### Heuristic
1. start with the cheapest exact signals
2. cap expansion by query class
3. only pay semantic cost when exact and graph signals are insufficient
4. preserve explainability and provenance
5. update reinforcement or decay after outcome feedback

### Failure risks
- candidate explosion
- stale data bias
- low-confidence amplification
- cross-session interference

### Instrumentation
- p50/p95/p99 latency
- candidate counts before and after pruning
- success contribution estimate
- cache hit ratio
- repairability score

### Pseudocode
```text
fn pattern_13(ctx):
    budget = ctx.latency_budget
    exact = cheap_exact(ctx)
    graph = bounded_graph(ctx, exact)
    sem = maybe_semantic(ctx, budget, graph)
    merged = dedup_merge(exact, graph, sem)
    ranked = rank_with_policy(ctx, merged)
    return package(ctx, ranked[:ctx.limit])
```
## 14. Fast Path Retrieval pattern 14

### Objective
Optimize fast path retrieval under bounded latency and bounded noise.

### Preconditions
- namespace is validated
- policy context is loaded
- latency budget is available
- tier capabilities are known

### Heuristic
1. start with the cheapest exact signals
2. cap expansion by query class
3. only pay semantic cost when exact and graph signals are insufficient
4. preserve explainability and provenance
5. update reinforcement or decay after outcome feedback

### Failure risks
- candidate explosion
- stale data bias
- low-confidence amplification
- cross-session interference

### Instrumentation
- p50/p95/p99 latency
- candidate counts before and after pruning
- success contribution estimate
- cache hit ratio
- repairability score

### Pseudocode
```text
fn pattern_14(ctx):
    budget = ctx.latency_budget
    exact = cheap_exact(ctx)
    graph = bounded_graph(ctx, exact)
    sem = maybe_semantic(ctx, budget, graph)
    merged = dedup_merge(exact, graph, sem)
    ranked = rank_with_policy(ctx, merged)
    return package(ctx, ranked[:ctx.limit])
```
## 15. Fast Path Retrieval pattern 15

### Objective
Optimize fast path retrieval under bounded latency and bounded noise.

### Preconditions
- namespace is validated
- policy context is loaded
- latency budget is available
- tier capabilities are known

### Heuristic
1. start with the cheapest exact signals
2. cap expansion by query class
3. only pay semantic cost when exact and graph signals are insufficient
4. preserve explainability and provenance
5. update reinforcement or decay after outcome feedback

### Failure risks
- candidate explosion
- stale data bias
- low-confidence amplification
- cross-session interference

### Instrumentation
- p50/p95/p99 latency
- candidate counts before and after pruning
- success contribution estimate
- cache hit ratio
- repairability score

### Pseudocode
```text
fn pattern_15(ctx):
    budget = ctx.latency_budget
    exact = cheap_exact(ctx)
    graph = bounded_graph(ctx, exact)
    sem = maybe_semantic(ctx, budget, graph)
    merged = dedup_merge(exact, graph, sem)
    ranked = rank_with_policy(ctx, merged)
    return package(ctx, ranked[:ctx.limit])
```
## 16. Fast Path Retrieval pattern 16

### Objective
Optimize fast path retrieval under bounded latency and bounded noise.

### Preconditions
- namespace is validated
- policy context is loaded
- latency budget is available
- tier capabilities are known

### Heuristic
1. start with the cheapest exact signals
2. cap expansion by query class
3. only pay semantic cost when exact and graph signals are insufficient
4. preserve explainability and provenance
5. update reinforcement or decay after outcome feedback

### Failure risks
- candidate explosion
- stale data bias
- low-confidence amplification
- cross-session interference

### Instrumentation
- p50/p95/p99 latency
- candidate counts before and after pruning
- success contribution estimate
- cache hit ratio
- repairability score

### Pseudocode
```text
fn pattern_16(ctx):
    budget = ctx.latency_budget
    exact = cheap_exact(ctx)
    graph = bounded_graph(ctx, exact)
    sem = maybe_semantic(ctx, budget, graph)
    merged = dedup_merge(exact, graph, sem)
    ranked = rank_with_policy(ctx, merged)
    return package(ctx, ranked[:ctx.limit])
```
## 17. Fast Path Retrieval pattern 17

### Objective
Optimize fast path retrieval under bounded latency and bounded noise.

### Preconditions
- namespace is validated
- policy context is loaded
- latency budget is available
- tier capabilities are known

### Heuristic
1. start with the cheapest exact signals
2. cap expansion by query class
3. only pay semantic cost when exact and graph signals are insufficient
4. preserve explainability and provenance
5. update reinforcement or decay after outcome feedback

### Failure risks
- candidate explosion
- stale data bias
- low-confidence amplification
- cross-session interference

### Instrumentation
- p50/p95/p99 latency
- candidate counts before and after pruning
- success contribution estimate
- cache hit ratio
- repairability score

### Pseudocode
```text
fn pattern_17(ctx):
    budget = ctx.latency_budget
    exact = cheap_exact(ctx)
    graph = bounded_graph(ctx, exact)
    sem = maybe_semantic(ctx, budget, graph)
    merged = dedup_merge(exact, graph, sem)
    ranked = rank_with_policy(ctx, merged)
    return package(ctx, ranked[:ctx.limit])
```
## 18. Fast Path Retrieval pattern 18

### Objective
Optimize fast path retrieval under bounded latency and bounded noise.

### Preconditions
- namespace is validated
- policy context is loaded
- latency budget is available
- tier capabilities are known

### Heuristic
1. start with the cheapest exact signals
2. cap expansion by query class
3. only pay semantic cost when exact and graph signals are insufficient
4. preserve explainability and provenance
5. update reinforcement or decay after outcome feedback

### Failure risks
- candidate explosion
- stale data bias
- low-confidence amplification
- cross-session interference

### Instrumentation
- p50/p95/p99 latency
- candidate counts before and after pruning
- success contribution estimate
- cache hit ratio
- repairability score

### Pseudocode
```text
fn pattern_18(ctx):
    budget = ctx.latency_budget
    exact = cheap_exact(ctx)
    graph = bounded_graph(ctx, exact)
    sem = maybe_semantic(ctx, budget, graph)
    merged = dedup_merge(exact, graph, sem)
    ranked = rank_with_policy(ctx, merged)
    return package(ctx, ranked[:ctx.limit])
```
## 19. Fast Path Retrieval pattern 19

### Objective
Optimize fast path retrieval under bounded latency and bounded noise.

### Preconditions
- namespace is validated
- policy context is loaded
- latency budget is available
- tier capabilities are known

### Heuristic
1. start with the cheapest exact signals
2. cap expansion by query class
3. only pay semantic cost when exact and graph signals are insufficient
4. preserve explainability and provenance
5. update reinforcement or decay after outcome feedback

### Failure risks
- candidate explosion
- stale data bias
- low-confidence amplification
- cross-session interference

### Instrumentation
- p50/p95/p99 latency
- candidate counts before and after pruning
- success contribution estimate
- cache hit ratio
- repairability score

### Pseudocode
```text
fn pattern_19(ctx):
    budget = ctx.latency_budget
    exact = cheap_exact(ctx)
    graph = bounded_graph(ctx, exact)
    sem = maybe_semantic(ctx, budget, graph)
    merged = dedup_merge(exact, graph, sem)
    ranked = rank_with_policy(ctx, merged)
    return package(ctx, ranked[:ctx.limit])
```
## 20. Fast Path Retrieval pattern 20

### Objective
Optimize fast path retrieval under bounded latency and bounded noise.

### Preconditions
- namespace is validated
- policy context is loaded
- latency budget is available
- tier capabilities are known

### Heuristic
1. start with the cheapest exact signals
2. cap expansion by query class
3. only pay semantic cost when exact and graph signals are insufficient
4. preserve explainability and provenance
5. update reinforcement or decay after outcome feedback

### Failure risks
- candidate explosion
- stale data bias
- low-confidence amplification
- cross-session interference

### Instrumentation
- p50/p95/p99 latency
- candidate counts before and after pruning
- success contribution estimate
- cache hit ratio
- repairability score

### Pseudocode
```text
fn pattern_20(ctx):
    budget = ctx.latency_budget
    exact = cheap_exact(ctx)
    graph = bounded_graph(ctx, exact)
    sem = maybe_semantic(ctx, budget, graph)
    merged = dedup_merge(exact, graph, sem)
    ranked = rank_with_policy(ctx, merged)
    return package(ctx, ranked[:ctx.limit])
```
## 21. Fast Path Retrieval pattern 21

### Objective
Optimize fast path retrieval under bounded latency and bounded noise.

### Preconditions
- namespace is validated
- policy context is loaded
- latency budget is available
- tier capabilities are known

### Heuristic
1. start with the cheapest exact signals
2. cap expansion by query class
3. only pay semantic cost when exact and graph signals are insufficient
4. preserve explainability and provenance
5. update reinforcement or decay after outcome feedback

### Failure risks
- candidate explosion
- stale data bias
- low-confidence amplification
- cross-session interference

### Instrumentation
- p50/p95/p99 latency
- candidate counts before and after pruning
- success contribution estimate
- cache hit ratio
- repairability score

### Pseudocode
```text
fn pattern_21(ctx):
    budget = ctx.latency_budget
    exact = cheap_exact(ctx)
    graph = bounded_graph(ctx, exact)
    sem = maybe_semantic(ctx, budget, graph)
    merged = dedup_merge(exact, graph, sem)
    ranked = rank_with_policy(ctx, merged)
    return package(ctx, ranked[:ctx.limit])
```
## 22. Fast Path Retrieval pattern 22

### Objective
Optimize fast path retrieval under bounded latency and bounded noise.

### Preconditions
- namespace is validated
- policy context is loaded
- latency budget is available
- tier capabilities are known

### Heuristic
1. start with the cheapest exact signals
2. cap expansion by query class
3. only pay semantic cost when exact and graph signals are insufficient
4. preserve explainability and provenance
5. update reinforcement or decay after outcome feedback

### Failure risks
- candidate explosion
- stale data bias
- low-confidence amplification
- cross-session interference

### Instrumentation
- p50/p95/p99 latency
- candidate counts before and after pruning
- success contribution estimate
- cache hit ratio
- repairability score

### Pseudocode
```text
fn pattern_22(ctx):
    budget = ctx.latency_budget
    exact = cheap_exact(ctx)
    graph = bounded_graph(ctx, exact)
    sem = maybe_semantic(ctx, budget, graph)
    merged = dedup_merge(exact, graph, sem)
    ranked = rank_with_policy(ctx, merged)
    return package(ctx, ranked[:ctx.limit])
```
## 23. Fast Path Retrieval pattern 23

### Objective
Optimize fast path retrieval under bounded latency and bounded noise.

### Preconditions
- namespace is validated
- policy context is loaded
- latency budget is available
- tier capabilities are known

### Heuristic
1. start with the cheapest exact signals
2. cap expansion by query class
3. only pay semantic cost when exact and graph signals are insufficient
4. preserve explainability and provenance
5. update reinforcement or decay after outcome feedback

### Failure risks
- candidate explosion
- stale data bias
- low-confidence amplification
- cross-session interference

### Instrumentation
- p50/p95/p99 latency
- candidate counts before and after pruning
- success contribution estimate
- cache hit ratio
- repairability score

### Pseudocode
```text
fn pattern_23(ctx):
    budget = ctx.latency_budget
    exact = cheap_exact(ctx)
    graph = bounded_graph(ctx, exact)
    sem = maybe_semantic(ctx, budget, graph)
    merged = dedup_merge(exact, graph, sem)
    ranked = rank_with_policy(ctx, merged)
    return package(ctx, ranked[:ctx.limit])
```
## 24. Fast Path Retrieval pattern 24

### Objective
Optimize fast path retrieval under bounded latency and bounded noise.

### Preconditions
- namespace is validated
- policy context is loaded
- latency budget is available
- tier capabilities are known

### Heuristic
1. start with the cheapest exact signals
2. cap expansion by query class
3. only pay semantic cost when exact and graph signals are insufficient
4. preserve explainability and provenance
5. update reinforcement or decay after outcome feedback

### Failure risks
- candidate explosion
- stale data bias
- low-confidence amplification
- cross-session interference

### Instrumentation
- p50/p95/p99 latency
- candidate counts before and after pruning
- success contribution estimate
- cache hit ratio
- repairability score

### Pseudocode
```text
fn pattern_24(ctx):
    budget = ctx.latency_budget
    exact = cheap_exact(ctx)
    graph = bounded_graph(ctx, exact)
    sem = maybe_semantic(ctx, budget, graph)
    merged = dedup_merge(exact, graph, sem)
    ranked = rank_with_policy(ctx, merged)
    return package(ctx, ranked[:ctx.limit])
```
## 25. Fast Path Retrieval pattern 25

### Objective
Optimize fast path retrieval under bounded latency and bounded noise.

### Preconditions
- namespace is validated
- policy context is loaded
- latency budget is available
- tier capabilities are known

### Heuristic
1. start with the cheapest exact signals
2. cap expansion by query class
3. only pay semantic cost when exact and graph signals are insufficient
4. preserve explainability and provenance
5. update reinforcement or decay after outcome feedback

### Failure risks
- candidate explosion
- stale data bias
- low-confidence amplification
- cross-session interference

### Instrumentation
- p50/p95/p99 latency
- candidate counts before and after pruning
- success contribution estimate
- cache hit ratio
- repairability score

### Pseudocode
```text
fn pattern_25(ctx):
    budget = ctx.latency_budget
    exact = cheap_exact(ctx)
    graph = bounded_graph(ctx, exact)
    sem = maybe_semantic(ctx, budget, graph)
    merged = dedup_merge(exact, graph, sem)
    ranked = rank_with_policy(ctx, merged)
    return package(ctx, ranked[:ctx.limit])
```
## 26. Fast Path Retrieval pattern 26

### Objective
Optimize fast path retrieval under bounded latency and bounded noise.

### Preconditions
- namespace is validated
- policy context is loaded
- latency budget is available
- tier capabilities are known

### Heuristic
1. start with the cheapest exact signals
2. cap expansion by query class
3. only pay semantic cost when exact and graph signals are insufficient
4. preserve explainability and provenance
5. update reinforcement or decay after outcome feedback

### Failure risks
- candidate explosion
- stale data bias
- low-confidence amplification
- cross-session interference

### Instrumentation
- p50/p95/p99 latency
- candidate counts before and after pruning
- success contribution estimate
- cache hit ratio
- repairability score

### Pseudocode
```text
fn pattern_26(ctx):
    budget = ctx.latency_budget
    exact = cheap_exact(ctx)
    graph = bounded_graph(ctx, exact)
    sem = maybe_semantic(ctx, budget, graph)
    merged = dedup_merge(exact, graph, sem)
    ranked = rank_with_policy(ctx, merged)
    return package(ctx, ranked[:ctx.limit])
```
## 27. Fast Path Retrieval pattern 27

### Objective
Optimize fast path retrieval under bounded latency and bounded noise.

### Preconditions
- namespace is validated
- policy context is loaded
- latency budget is available
- tier capabilities are known

### Heuristic
1. start with the cheapest exact signals
2. cap expansion by query class
3. only pay semantic cost when exact and graph signals are insufficient
4. preserve explainability and provenance
5. update reinforcement or decay after outcome feedback

### Failure risks
- candidate explosion
- stale data bias
- low-confidence amplification
- cross-session interference

### Instrumentation
- p50/p95/p99 latency
- candidate counts before and after pruning
- success contribution estimate
- cache hit ratio
- repairability score

### Pseudocode
```text
fn pattern_27(ctx):
    budget = ctx.latency_budget
    exact = cheap_exact(ctx)
    graph = bounded_graph(ctx, exact)
    sem = maybe_semantic(ctx, budget, graph)
    merged = dedup_merge(exact, graph, sem)
    ranked = rank_with_policy(ctx, merged)
    return package(ctx, ranked[:ctx.limit])
```
## 28. Fast Path Retrieval pattern 28

### Objective
Optimize fast path retrieval under bounded latency and bounded noise.

### Preconditions
- namespace is validated
- policy context is loaded
- latency budget is available
- tier capabilities are known

### Heuristic
1. start with the cheapest exact signals
2. cap expansion by query class
3. only pay semantic cost when exact and graph signals are insufficient
4. preserve explainability and provenance
5. update reinforcement or decay after outcome feedback

### Failure risks
- candidate explosion
- stale data bias
- low-confidence amplification
- cross-session interference

### Instrumentation
- p50/p95/p99 latency
- candidate counts before and after pruning
- success contribution estimate
- cache hit ratio
- repairability score

### Pseudocode
```text
fn pattern_28(ctx):
    budget = ctx.latency_budget
    exact = cheap_exact(ctx)
    graph = bounded_graph(ctx, exact)
    sem = maybe_semantic(ctx, budget, graph)
    merged = dedup_merge(exact, graph, sem)
    ranked = rank_with_policy(ctx, merged)
    return package(ctx, ranked[:ctx.limit])
```
## 29. Fast Path Retrieval pattern 29

### Objective
Optimize fast path retrieval under bounded latency and bounded noise.

### Preconditions
- namespace is validated
- policy context is loaded
- latency budget is available
- tier capabilities are known

### Heuristic
1. start with the cheapest exact signals
2. cap expansion by query class
3. only pay semantic cost when exact and graph signals are insufficient
4. preserve explainability and provenance
5. update reinforcement or decay after outcome feedback

### Failure risks
- candidate explosion
- stale data bias
- low-confidence amplification
- cross-session interference

### Instrumentation
- p50/p95/p99 latency
- candidate counts before and after pruning
- success contribution estimate
- cache hit ratio
- repairability score

### Pseudocode
```text
fn pattern_29(ctx):
    budget = ctx.latency_budget
    exact = cheap_exact(ctx)
    graph = bounded_graph(ctx, exact)
    sem = maybe_semantic(ctx, budget, graph)
    merged = dedup_merge(exact, graph, sem)
    ranked = rank_with_policy(ctx, merged)
    return package(ctx, ranked[:ctx.limit])
```
## 30. Fast Path Retrieval pattern 30

### Objective
Optimize fast path retrieval under bounded latency and bounded noise.

### Preconditions
- namespace is validated
- policy context is loaded
- latency budget is available
- tier capabilities are known

### Heuristic
1. start with the cheapest exact signals
2. cap expansion by query class
3. only pay semantic cost when exact and graph signals are insufficient
4. preserve explainability and provenance
5. update reinforcement or decay after outcome feedback

### Failure risks
- candidate explosion
- stale data bias
- low-confidence amplification
- cross-session interference

### Instrumentation
- p50/p95/p99 latency
- candidate counts before and after pruning
- success contribution estimate
- cache hit ratio
- repairability score

### Pseudocode
```text
fn pattern_30(ctx):
    budget = ctx.latency_budget
    exact = cheap_exact(ctx)
    graph = bounded_graph(ctx, exact)
    sem = maybe_semantic(ctx, budget, graph)
    merged = dedup_merge(exact, graph, sem)
    ranked = rank_with_policy(ctx, merged)
    return package(ctx, ranked[:ctx.limit])
```
## 31. Tier Routing pattern 1

### Objective
Optimize tier routing under bounded latency and bounded noise.

### Preconditions
- namespace is validated
- policy context is loaded
- latency budget is available
- tier capabilities are known

### Heuristic
1. start with the cheapest exact signals
2. cap expansion by query class
3. only pay semantic cost when exact and graph signals are insufficient
4. preserve explainability and provenance
5. update reinforcement or decay after outcome feedback

### Failure risks
- candidate explosion
- stale data bias
- low-confidence amplification
- cross-session interference

### Instrumentation
- p50/p95/p99 latency
- candidate counts before and after pruning
- success contribution estimate
- cache hit ratio
- repairability score

### Pseudocode
```text
fn pattern_31(ctx):
    budget = ctx.latency_budget
    exact = cheap_exact(ctx)
    graph = bounded_graph(ctx, exact)
    sem = maybe_semantic(ctx, budget, graph)
    merged = dedup_merge(exact, graph, sem)
    ranked = rank_with_policy(ctx, merged)
    return package(ctx, ranked[:ctx.limit])
```
## 32. Tier Routing pattern 2

### Objective
Optimize tier routing under bounded latency and bounded noise.

### Preconditions
- namespace is validated
- policy context is loaded
- latency budget is available
- tier capabilities are known

### Heuristic
1. start with the cheapest exact signals
2. cap expansion by query class
3. only pay semantic cost when exact and graph signals are insufficient
4. preserve explainability and provenance
5. update reinforcement or decay after outcome feedback

### Failure risks
- candidate explosion
- stale data bias
- low-confidence amplification
- cross-session interference

### Instrumentation
- p50/p95/p99 latency
- candidate counts before and after pruning
- success contribution estimate
- cache hit ratio
- repairability score

### Pseudocode
```text
fn pattern_32(ctx):
    budget = ctx.latency_budget
    exact = cheap_exact(ctx)
    graph = bounded_graph(ctx, exact)
    sem = maybe_semantic(ctx, budget, graph)
    merged = dedup_merge(exact, graph, sem)
    ranked = rank_with_policy(ctx, merged)
    return package(ctx, ranked[:ctx.limit])
```
## 33. Tier Routing pattern 3

### Objective
Optimize tier routing under bounded latency and bounded noise.

### Preconditions
- namespace is validated
- policy context is loaded
- latency budget is available
- tier capabilities are known

### Heuristic
1. start with the cheapest exact signals
2. cap expansion by query class
3. only pay semantic cost when exact and graph signals are insufficient
4. preserve explainability and provenance
5. update reinforcement or decay after outcome feedback

### Failure risks
- candidate explosion
- stale data bias
- low-confidence amplification
- cross-session interference

### Instrumentation
- p50/p95/p99 latency
- candidate counts before and after pruning
- success contribution estimate
- cache hit ratio
- repairability score

### Pseudocode
```text
fn pattern_33(ctx):
    budget = ctx.latency_budget
    exact = cheap_exact(ctx)
    graph = bounded_graph(ctx, exact)
    sem = maybe_semantic(ctx, budget, graph)
    merged = dedup_merge(exact, graph, sem)
    ranked = rank_with_policy(ctx, merged)
    return package(ctx, ranked[:ctx.limit])
```
## 34. Tier Routing pattern 4

### Objective
Optimize tier routing under bounded latency and bounded noise.

### Preconditions
- namespace is validated
- policy context is loaded
- latency budget is available
- tier capabilities are known

### Heuristic
1. start with the cheapest exact signals
2. cap expansion by query class
3. only pay semantic cost when exact and graph signals are insufficient
4. preserve explainability and provenance
5. update reinforcement or decay after outcome feedback

### Failure risks
- candidate explosion
- stale data bias
- low-confidence amplification
- cross-session interference

### Instrumentation
- p50/p95/p99 latency
- candidate counts before and after pruning
- success contribution estimate
- cache hit ratio
- repairability score

### Pseudocode
```text
fn pattern_34(ctx):
    budget = ctx.latency_budget
    exact = cheap_exact(ctx)
    graph = bounded_graph(ctx, exact)
    sem = maybe_semantic(ctx, budget, graph)
    merged = dedup_merge(exact, graph, sem)
    ranked = rank_with_policy(ctx, merged)
    return package(ctx, ranked[:ctx.limit])
```
## 35. Tier Routing pattern 5

### Objective
Optimize tier routing under bounded latency and bounded noise.

### Preconditions
- namespace is validated
- policy context is loaded
- latency budget is available
- tier capabilities are known

### Heuristic
1. start with the cheapest exact signals
2. cap expansion by query class
3. only pay semantic cost when exact and graph signals are insufficient
4. preserve explainability and provenance
5. update reinforcement or decay after outcome feedback

### Failure risks
- candidate explosion
- stale data bias
- low-confidence amplification
- cross-session interference

### Instrumentation
- p50/p95/p99 latency
- candidate counts before and after pruning
- success contribution estimate
- cache hit ratio
- repairability score

### Pseudocode
```text
fn pattern_35(ctx):
    budget = ctx.latency_budget
    exact = cheap_exact(ctx)
    graph = bounded_graph(ctx, exact)
    sem = maybe_semantic(ctx, budget, graph)
    merged = dedup_merge(exact, graph, sem)
    ranked = rank_with_policy(ctx, merged)
    return package(ctx, ranked[:ctx.limit])
```
## 36. Tier Routing pattern 6

### Objective
Optimize tier routing under bounded latency and bounded noise.

### Preconditions
- namespace is validated
- policy context is loaded
- latency budget is available
- tier capabilities are known

### Heuristic
1. start with the cheapest exact signals
2. cap expansion by query class
3. only pay semantic cost when exact and graph signals are insufficient
4. preserve explainability and provenance
5. update reinforcement or decay after outcome feedback

### Failure risks
- candidate explosion
- stale data bias
- low-confidence amplification
- cross-session interference

### Instrumentation
- p50/p95/p99 latency
- candidate counts before and after pruning
- success contribution estimate
- cache hit ratio
- repairability score

### Pseudocode
```text
fn pattern_36(ctx):
    budget = ctx.latency_budget
    exact = cheap_exact(ctx)
    graph = bounded_graph(ctx, exact)
    sem = maybe_semantic(ctx, budget, graph)
    merged = dedup_merge(exact, graph, sem)
    ranked = rank_with_policy(ctx, merged)
    return package(ctx, ranked[:ctx.limit])
```
## 37. Tier Routing pattern 7

### Objective
Optimize tier routing under bounded latency and bounded noise.

### Preconditions
- namespace is validated
- policy context is loaded
- latency budget is available
- tier capabilities are known

### Heuristic
1. start with the cheapest exact signals
2. cap expansion by query class
3. only pay semantic cost when exact and graph signals are insufficient
4. preserve explainability and provenance
5. update reinforcement or decay after outcome feedback

### Failure risks
- candidate explosion
- stale data bias
- low-confidence amplification
- cross-session interference

### Instrumentation
- p50/p95/p99 latency
- candidate counts before and after pruning
- success contribution estimate
- cache hit ratio
- repairability score

### Pseudocode
```text
fn pattern_37(ctx):
    budget = ctx.latency_budget
    exact = cheap_exact(ctx)
    graph = bounded_graph(ctx, exact)
    sem = maybe_semantic(ctx, budget, graph)
    merged = dedup_merge(exact, graph, sem)
    ranked = rank_with_policy(ctx, merged)
    return package(ctx, ranked[:ctx.limit])
```
## 38. Tier Routing pattern 8

### Objective
Optimize tier routing under bounded latency and bounded noise.

### Preconditions
- namespace is validated
- policy context is loaded
- latency budget is available
- tier capabilities are known

### Heuristic
1. start with the cheapest exact signals
2. cap expansion by query class
3. only pay semantic cost when exact and graph signals are insufficient
4. preserve explainability and provenance
5. update reinforcement or decay after outcome feedback

### Failure risks
- candidate explosion
- stale data bias
- low-confidence amplification
- cross-session interference

### Instrumentation
- p50/p95/p99 latency
- candidate counts before and after pruning
- success contribution estimate
- cache hit ratio
- repairability score

### Pseudocode
```text
fn pattern_38(ctx):
    budget = ctx.latency_budget
    exact = cheap_exact(ctx)
    graph = bounded_graph(ctx, exact)
    sem = maybe_semantic(ctx, budget, graph)
    merged = dedup_merge(exact, graph, sem)
    ranked = rank_with_policy(ctx, merged)
    return package(ctx, ranked[:ctx.limit])
```
## 39. Tier Routing pattern 9

### Objective
Optimize tier routing under bounded latency and bounded noise.

### Preconditions
- namespace is validated
- policy context is loaded
- latency budget is available
- tier capabilities are known

### Heuristic
1. start with the cheapest exact signals
2. cap expansion by query class
3. only pay semantic cost when exact and graph signals are insufficient
4. preserve explainability and provenance
5. update reinforcement or decay after outcome feedback

### Failure risks
- candidate explosion
- stale data bias
- low-confidence amplification
- cross-session interference

### Instrumentation
- p50/p95/p99 latency
- candidate counts before and after pruning
- success contribution estimate
- cache hit ratio
- repairability score

### Pseudocode
```text
fn pattern_39(ctx):
    budget = ctx.latency_budget
    exact = cheap_exact(ctx)
    graph = bounded_graph(ctx, exact)
    sem = maybe_semantic(ctx, budget, graph)
    merged = dedup_merge(exact, graph, sem)
    ranked = rank_with_policy(ctx, merged)
    return package(ctx, ranked[:ctx.limit])
```
## 40. Tier Routing pattern 10

### Objective
Optimize tier routing under bounded latency and bounded noise.

### Preconditions
- namespace is validated
- policy context is loaded
- latency budget is available
- tier capabilities are known

### Heuristic
1. start with the cheapest exact signals
2. cap expansion by query class
3. only pay semantic cost when exact and graph signals are insufficient
4. preserve explainability and provenance
5. update reinforcement or decay after outcome feedback

### Failure risks
- candidate explosion
- stale data bias
- low-confidence amplification
- cross-session interference

### Instrumentation
- p50/p95/p99 latency
- candidate counts before and after pruning
- success contribution estimate
- cache hit ratio
- repairability score

### Pseudocode
```text
fn pattern_40(ctx):
    budget = ctx.latency_budget
    exact = cheap_exact(ctx)
    graph = bounded_graph(ctx, exact)
    sem = maybe_semantic(ctx, budget, graph)
    merged = dedup_merge(exact, graph, sem)
    ranked = rank_with_policy(ctx, merged)
    return package(ctx, ranked[:ctx.limit])
```
## 41. Tier Routing pattern 11

### Objective
Optimize tier routing under bounded latency and bounded noise.

### Preconditions
- namespace is validated
- policy context is loaded
- latency budget is available
- tier capabilities are known

### Heuristic
1. start with the cheapest exact signals
2. cap expansion by query class
3. only pay semantic cost when exact and graph signals are insufficient
4. preserve explainability and provenance
5. update reinforcement or decay after outcome feedback

### Failure risks
- candidate explosion
- stale data bias
- low-confidence amplification
- cross-session interference

### Instrumentation
- p50/p95/p99 latency
- candidate counts before and after pruning
- success contribution estimate
- cache hit ratio
- repairability score

### Pseudocode
```text
fn pattern_41(ctx):
    budget = ctx.latency_budget
    exact = cheap_exact(ctx)
    graph = bounded_graph(ctx, exact)
    sem = maybe_semantic(ctx, budget, graph)
    merged = dedup_merge(exact, graph, sem)
    ranked = rank_with_policy(ctx, merged)
    return package(ctx, ranked[:ctx.limit])
```
## 42. Tier Routing pattern 12

### Objective
Optimize tier routing under bounded latency and bounded noise.

### Preconditions
- namespace is validated
- policy context is loaded
- latency budget is available
- tier capabilities are known

### Heuristic
1. start with the cheapest exact signals
2. cap expansion by query class
3. only pay semantic cost when exact and graph signals are insufficient
4. preserve explainability and provenance
5. update reinforcement or decay after outcome feedback

### Failure risks
- candidate explosion
- stale data bias
- low-confidence amplification
- cross-session interference

### Instrumentation
- p50/p95/p99 latency
- candidate counts before and after pruning
- success contribution estimate
- cache hit ratio
- repairability score

### Pseudocode
```text
fn pattern_42(ctx):
    budget = ctx.latency_budget
    exact = cheap_exact(ctx)
    graph = bounded_graph(ctx, exact)
    sem = maybe_semantic(ctx, budget, graph)
    merged = dedup_merge(exact, graph, sem)
    ranked = rank_with_policy(ctx, merged)
    return package(ctx, ranked[:ctx.limit])
```
## 43. Tier Routing pattern 13

### Objective
Optimize tier routing under bounded latency and bounded noise.

### Preconditions
- namespace is validated
- policy context is loaded
- latency budget is available
- tier capabilities are known

### Heuristic
1. start with the cheapest exact signals
2. cap expansion by query class
3. only pay semantic cost when exact and graph signals are insufficient
4. preserve explainability and provenance
5. update reinforcement or decay after outcome feedback

### Failure risks
- candidate explosion
- stale data bias
- low-confidence amplification
- cross-session interference

### Instrumentation
- p50/p95/p99 latency
- candidate counts before and after pruning
- success contribution estimate
- cache hit ratio
- repairability score

### Pseudocode
```text
fn pattern_43(ctx):
    budget = ctx.latency_budget
    exact = cheap_exact(ctx)
    graph = bounded_graph(ctx, exact)
    sem = maybe_semantic(ctx, budget, graph)
    merged = dedup_merge(exact, graph, sem)
    ranked = rank_with_policy(ctx, merged)
    return package(ctx, ranked[:ctx.limit])
```
## 44. Tier Routing pattern 14

### Objective
Optimize tier routing under bounded latency and bounded noise.

### Preconditions
- namespace is validated
- policy context is loaded
- latency budget is available
- tier capabilities are known

### Heuristic
1. start with the cheapest exact signals
2. cap expansion by query class
3. only pay semantic cost when exact and graph signals are insufficient
4. preserve explainability and provenance
5. update reinforcement or decay after outcome feedback

### Failure risks
- candidate explosion
- stale data bias
- low-confidence amplification
- cross-session interference

### Instrumentation
- p50/p95/p99 latency
- candidate counts before and after pruning
- success contribution estimate
- cache hit ratio
- repairability score

### Pseudocode
```text
fn pattern_44(ctx):
    budget = ctx.latency_budget
    exact = cheap_exact(ctx)
    graph = bounded_graph(ctx, exact)
    sem = maybe_semantic(ctx, budget, graph)
    merged = dedup_merge(exact, graph, sem)
    ranked = rank_with_policy(ctx, merged)
    return package(ctx, ranked[:ctx.limit])
```
## 45. Tier Routing pattern 15

### Objective
Optimize tier routing under bounded latency and bounded noise.

### Preconditions
- namespace is validated
- policy context is loaded
- latency budget is available
- tier capabilities are known

### Heuristic
1. start with the cheapest exact signals
2. cap expansion by query class
3. only pay semantic cost when exact and graph signals are insufficient
4. preserve explainability and provenance
5. update reinforcement or decay after outcome feedback

### Failure risks
- candidate explosion
- stale data bias
- low-confidence amplification
- cross-session interference

### Instrumentation
- p50/p95/p99 latency
- candidate counts before and after pruning
- success contribution estimate
- cache hit ratio
- repairability score

### Pseudocode
```text
fn pattern_45(ctx):
    budget = ctx.latency_budget
    exact = cheap_exact(ctx)
    graph = bounded_graph(ctx, exact)
    sem = maybe_semantic(ctx, budget, graph)
    merged = dedup_merge(exact, graph, sem)
    ranked = rank_with_policy(ctx, merged)
    return package(ctx, ranked[:ctx.limit])
```
## 46. Tier Routing pattern 16

### Objective
Optimize tier routing under bounded latency and bounded noise.

### Preconditions
- namespace is validated
- policy context is loaded
- latency budget is available
- tier capabilities are known

### Heuristic
1. start with the cheapest exact signals
2. cap expansion by query class
3. only pay semantic cost when exact and graph signals are insufficient
4. preserve explainability and provenance
5. update reinforcement or decay after outcome feedback

### Failure risks
- candidate explosion
- stale data bias
- low-confidence amplification
- cross-session interference

### Instrumentation
- p50/p95/p99 latency
- candidate counts before and after pruning
- success contribution estimate
- cache hit ratio
- repairability score

### Pseudocode
```text
fn pattern_46(ctx):
    budget = ctx.latency_budget
    exact = cheap_exact(ctx)
    graph = bounded_graph(ctx, exact)
    sem = maybe_semantic(ctx, budget, graph)
    merged = dedup_merge(exact, graph, sem)
    ranked = rank_with_policy(ctx, merged)
    return package(ctx, ranked[:ctx.limit])
```
## 47. Tier Routing pattern 17

### Objective
Optimize tier routing under bounded latency and bounded noise.

### Preconditions
- namespace is validated
- policy context is loaded
- latency budget is available
- tier capabilities are known

### Heuristic
1. start with the cheapest exact signals
2. cap expansion by query class
3. only pay semantic cost when exact and graph signals are insufficient
4. preserve explainability and provenance
5. update reinforcement or decay after outcome feedback

### Failure risks
- candidate explosion
- stale data bias
- low-confidence amplification
- cross-session interference

### Instrumentation
- p50/p95/p99 latency
- candidate counts before and after pruning
- success contribution estimate
- cache hit ratio
- repairability score

### Pseudocode
```text
fn pattern_47(ctx):
    budget = ctx.latency_budget
    exact = cheap_exact(ctx)
    graph = bounded_graph(ctx, exact)
    sem = maybe_semantic(ctx, budget, graph)
    merged = dedup_merge(exact, graph, sem)
    ranked = rank_with_policy(ctx, merged)
    return package(ctx, ranked[:ctx.limit])
```
## 48. Tier Routing pattern 18

### Objective
Optimize tier routing under bounded latency and bounded noise.

### Preconditions
- namespace is validated
- policy context is loaded
- latency budget is available
- tier capabilities are known

### Heuristic
1. start with the cheapest exact signals
2. cap expansion by query class
3. only pay semantic cost when exact and graph signals are insufficient
4. preserve explainability and provenance
5. update reinforcement or decay after outcome feedback

### Failure risks
- candidate explosion
- stale data bias
- low-confidence amplification
- cross-session interference

### Instrumentation
- p50/p95/p99 latency
- candidate counts before and after pruning
- success contribution estimate
- cache hit ratio
- repairability score

### Pseudocode
```text
fn pattern_48(ctx):
    budget = ctx.latency_budget
    exact = cheap_exact(ctx)
    graph = bounded_graph(ctx, exact)
    sem = maybe_semantic(ctx, budget, graph)
    merged = dedup_merge(exact, graph, sem)
    ranked = rank_with_policy(ctx, merged)
    return package(ctx, ranked[:ctx.limit])
```
## 49. Tier Routing pattern 19

### Objective
Optimize tier routing under bounded latency and bounded noise.

### Preconditions
- namespace is validated
- policy context is loaded
- latency budget is available
- tier capabilities are known

### Heuristic
1. start with the cheapest exact signals
2. cap expansion by query class
3. only pay semantic cost when exact and graph signals are insufficient
4. preserve explainability and provenance
5. update reinforcement or decay after outcome feedback

### Failure risks
- candidate explosion
- stale data bias
- low-confidence amplification
- cross-session interference

### Instrumentation
- p50/p95/p99 latency
- candidate counts before and after pruning
- success contribution estimate
- cache hit ratio
- repairability score

### Pseudocode
```text
fn pattern_49(ctx):
    budget = ctx.latency_budget
    exact = cheap_exact(ctx)
    graph = bounded_graph(ctx, exact)
    sem = maybe_semantic(ctx, budget, graph)
    merged = dedup_merge(exact, graph, sem)
    ranked = rank_with_policy(ctx, merged)
    return package(ctx, ranked[:ctx.limit])
```
## 50. Tier Routing pattern 20

### Objective
Optimize tier routing under bounded latency and bounded noise.

### Preconditions
- namespace is validated
- policy context is loaded
- latency budget is available
- tier capabilities are known

### Heuristic
1. start with the cheapest exact signals
2. cap expansion by query class
3. only pay semantic cost when exact and graph signals are insufficient
4. preserve explainability and provenance
5. update reinforcement or decay after outcome feedback

### Failure risks
- candidate explosion
- stale data bias
- low-confidence amplification
- cross-session interference

### Instrumentation
- p50/p95/p99 latency
- candidate counts before and after pruning
- success contribution estimate
- cache hit ratio
- repairability score

### Pseudocode
```text
fn pattern_50(ctx):
    budget = ctx.latency_budget
    exact = cheap_exact(ctx)
    graph = bounded_graph(ctx, exact)
    sem = maybe_semantic(ctx, budget, graph)
    merged = dedup_merge(exact, graph, sem)
    ranked = rank_with_policy(ctx, merged)
    return package(ctx, ranked[:ctx.limit])
```
## 51. Tier Routing pattern 21

### Objective
Optimize tier routing under bounded latency and bounded noise.

### Preconditions
- namespace is validated
- policy context is loaded
- latency budget is available
- tier capabilities are known

### Heuristic
1. start with the cheapest exact signals
2. cap expansion by query class
3. only pay semantic cost when exact and graph signals are insufficient
4. preserve explainability and provenance
5. update reinforcement or decay after outcome feedback

### Failure risks
- candidate explosion
- stale data bias
- low-confidence amplification
- cross-session interference

### Instrumentation
- p50/p95/p99 latency
- candidate counts before and after pruning
- success contribution estimate
- cache hit ratio
- repairability score

### Pseudocode
```text
fn pattern_51(ctx):
    budget = ctx.latency_budget
    exact = cheap_exact(ctx)
    graph = bounded_graph(ctx, exact)
    sem = maybe_semantic(ctx, budget, graph)
    merged = dedup_merge(exact, graph, sem)
    ranked = rank_with_policy(ctx, merged)
    return package(ctx, ranked[:ctx.limit])
```
## 52. Tier Routing pattern 22

### Objective
Optimize tier routing under bounded latency and bounded noise.

### Preconditions
- namespace is validated
- policy context is loaded
- latency budget is available
- tier capabilities are known

### Heuristic
1. start with the cheapest exact signals
2. cap expansion by query class
3. only pay semantic cost when exact and graph signals are insufficient
4. preserve explainability and provenance
5. update reinforcement or decay after outcome feedback

### Failure risks
- candidate explosion
- stale data bias
- low-confidence amplification
- cross-session interference

### Instrumentation
- p50/p95/p99 latency
- candidate counts before and after pruning
- success contribution estimate
- cache hit ratio
- repairability score

### Pseudocode
```text
fn pattern_52(ctx):
    budget = ctx.latency_budget
    exact = cheap_exact(ctx)
    graph = bounded_graph(ctx, exact)
    sem = maybe_semantic(ctx, budget, graph)
    merged = dedup_merge(exact, graph, sem)
    ranked = rank_with_policy(ctx, merged)
    return package(ctx, ranked[:ctx.limit])
```
## 53. Tier Routing pattern 23

### Objective
Optimize tier routing under bounded latency and bounded noise.

### Preconditions
- namespace is validated
- policy context is loaded
- latency budget is available
- tier capabilities are known

### Heuristic
1. start with the cheapest exact signals
2. cap expansion by query class
3. only pay semantic cost when exact and graph signals are insufficient
4. preserve explainability and provenance
5. update reinforcement or decay after outcome feedback

### Failure risks
- candidate explosion
- stale data bias
- low-confidence amplification
- cross-session interference

### Instrumentation
- p50/p95/p99 latency
- candidate counts before and after pruning
- success contribution estimate
- cache hit ratio
- repairability score

### Pseudocode
```text
fn pattern_53(ctx):
    budget = ctx.latency_budget
    exact = cheap_exact(ctx)
    graph = bounded_graph(ctx, exact)
    sem = maybe_semantic(ctx, budget, graph)
    merged = dedup_merge(exact, graph, sem)
    ranked = rank_with_policy(ctx, merged)
    return package(ctx, ranked[:ctx.limit])
```
## 54. Tier Routing pattern 24

### Objective
Optimize tier routing under bounded latency and bounded noise.

### Preconditions
- namespace is validated
- policy context is loaded
- latency budget is available
- tier capabilities are known

### Heuristic
1. start with the cheapest exact signals
2. cap expansion by query class
3. only pay semantic cost when exact and graph signals are insufficient
4. preserve explainability and provenance
5. update reinforcement or decay after outcome feedback

### Failure risks
- candidate explosion
- stale data bias
- low-confidence amplification
- cross-session interference

### Instrumentation
- p50/p95/p99 latency
- candidate counts before and after pruning
- success contribution estimate
- cache hit ratio
- repairability score

### Pseudocode
```text
fn pattern_54(ctx):
    budget = ctx.latency_budget
    exact = cheap_exact(ctx)
    graph = bounded_graph(ctx, exact)
    sem = maybe_semantic(ctx, budget, graph)
    merged = dedup_merge(exact, graph, sem)
    ranked = rank_with_policy(ctx, merged)
    return package(ctx, ranked[:ctx.limit])
```
## 55. Tier Routing pattern 25

### Objective
Optimize tier routing under bounded latency and bounded noise.

### Preconditions
- namespace is validated
- policy context is loaded
- latency budget is available
- tier capabilities are known

### Heuristic
1. start with the cheapest exact signals
2. cap expansion by query class
3. only pay semantic cost when exact and graph signals are insufficient
4. preserve explainability and provenance
5. update reinforcement or decay after outcome feedback

### Failure risks
- candidate explosion
- stale data bias
- low-confidence amplification
- cross-session interference

### Instrumentation
- p50/p95/p99 latency
- candidate counts before and after pruning
- success contribution estimate
- cache hit ratio
- repairability score

### Pseudocode
```text
fn pattern_55(ctx):
    budget = ctx.latency_budget
    exact = cheap_exact(ctx)
    graph = bounded_graph(ctx, exact)
    sem = maybe_semantic(ctx, budget, graph)
    merged = dedup_merge(exact, graph, sem)
    ranked = rank_with_policy(ctx, merged)
    return package(ctx, ranked[:ctx.limit])
```
## 56. Tier Routing pattern 26

### Objective
Optimize tier routing under bounded latency and bounded noise.

### Preconditions
- namespace is validated
- policy context is loaded
- latency budget is available
- tier capabilities are known

### Heuristic
1. start with the cheapest exact signals
2. cap expansion by query class
3. only pay semantic cost when exact and graph signals are insufficient
4. preserve explainability and provenance
5. update reinforcement or decay after outcome feedback

### Failure risks
- candidate explosion
- stale data bias
- low-confidence amplification
- cross-session interference

### Instrumentation
- p50/p95/p99 latency
- candidate counts before and after pruning
- success contribution estimate
- cache hit ratio
- repairability score

### Pseudocode
```text
fn pattern_56(ctx):
    budget = ctx.latency_budget
    exact = cheap_exact(ctx)
    graph = bounded_graph(ctx, exact)
    sem = maybe_semantic(ctx, budget, graph)
    merged = dedup_merge(exact, graph, sem)
    ranked = rank_with_policy(ctx, merged)
    return package(ctx, ranked[:ctx.limit])
```
## 57. Tier Routing pattern 27

### Objective
Optimize tier routing under bounded latency and bounded noise.

### Preconditions
- namespace is validated
- policy context is loaded
- latency budget is available
- tier capabilities are known

### Heuristic
1. start with the cheapest exact signals
2. cap expansion by query class
3. only pay semantic cost when exact and graph signals are insufficient
4. preserve explainability and provenance
5. update reinforcement or decay after outcome feedback

### Failure risks
- candidate explosion
- stale data bias
- low-confidence amplification
- cross-session interference

### Instrumentation
- p50/p95/p99 latency
- candidate counts before and after pruning
- success contribution estimate
- cache hit ratio
- repairability score

### Pseudocode
```text
fn pattern_57(ctx):
    budget = ctx.latency_budget
    exact = cheap_exact(ctx)
    graph = bounded_graph(ctx, exact)
    sem = maybe_semantic(ctx, budget, graph)
    merged = dedup_merge(exact, graph, sem)
    ranked = rank_with_policy(ctx, merged)
    return package(ctx, ranked[:ctx.limit])
```
## 58. Tier Routing pattern 28

### Objective
Optimize tier routing under bounded latency and bounded noise.

### Preconditions
- namespace is validated
- policy context is loaded
- latency budget is available
- tier capabilities are known

### Heuristic
1. start with the cheapest exact signals
2. cap expansion by query class
3. only pay semantic cost when exact and graph signals are insufficient
4. preserve explainability and provenance
5. update reinforcement or decay after outcome feedback

### Failure risks
- candidate explosion
- stale data bias
- low-confidence amplification
- cross-session interference

### Instrumentation
- p50/p95/p99 latency
- candidate counts before and after pruning
- success contribution estimate
- cache hit ratio
- repairability score

### Pseudocode
```text
fn pattern_58(ctx):
    budget = ctx.latency_budget
    exact = cheap_exact(ctx)
    graph = bounded_graph(ctx, exact)
    sem = maybe_semantic(ctx, budget, graph)
    merged = dedup_merge(exact, graph, sem)
    ranked = rank_with_policy(ctx, merged)
    return package(ctx, ranked[:ctx.limit])
```
## 59. Tier Routing pattern 29

### Objective
Optimize tier routing under bounded latency and bounded noise.

### Preconditions
- namespace is validated
- policy context is loaded
- latency budget is available
- tier capabilities are known

### Heuristic
1. start with the cheapest exact signals
2. cap expansion by query class
3. only pay semantic cost when exact and graph signals are insufficient
4. preserve explainability and provenance
5. update reinforcement or decay after outcome feedback

### Failure risks
- candidate explosion
- stale data bias
- low-confidence amplification
- cross-session interference

### Instrumentation
- p50/p95/p99 latency
- candidate counts before and after pruning
- success contribution estimate
- cache hit ratio
- repairability score

### Pseudocode
```text
fn pattern_59(ctx):
    budget = ctx.latency_budget
    exact = cheap_exact(ctx)
    graph = bounded_graph(ctx, exact)
    sem = maybe_semantic(ctx, budget, graph)
    merged = dedup_merge(exact, graph, sem)
    ranked = rank_with_policy(ctx, merged)
    return package(ctx, ranked[:ctx.limit])
```
## 60. Tier Routing pattern 30

### Objective
Optimize tier routing under bounded latency and bounded noise.

### Preconditions
- namespace is validated
- policy context is loaded
- latency budget is available
- tier capabilities are known

### Heuristic
1. start with the cheapest exact signals
2. cap expansion by query class
3. only pay semantic cost when exact and graph signals are insufficient
4. preserve explainability and provenance
5. update reinforcement or decay after outcome feedback

### Failure risks
- candidate explosion
- stale data bias
- low-confidence amplification
- cross-session interference

### Instrumentation
- p50/p95/p99 latency
- candidate counts before and after pruning
- success contribution estimate
- cache hit ratio
- repairability score

### Pseudocode
```text
fn pattern_60(ctx):
    budget = ctx.latency_budget
    exact = cheap_exact(ctx)
    graph = bounded_graph(ctx, exact)
    sem = maybe_semantic(ctx, budget, graph)
    merged = dedup_merge(exact, graph, sem)
    ranked = rank_with_policy(ctx, merged)
    return package(ctx, ranked[:ctx.limit])
```
## 61. Memory Decay pattern 1

### Objective
Optimize memory decay under bounded latency and bounded noise.

### Preconditions
- namespace is validated
- policy context is loaded
- latency budget is available
- tier capabilities are known

### Heuristic
1. start with the cheapest exact signals
2. cap expansion by query class
3. only pay semantic cost when exact and graph signals are insufficient
4. preserve explainability and provenance
5. update reinforcement or decay after outcome feedback

### Failure risks
- candidate explosion
- stale data bias
- low-confidence amplification
- cross-session interference

### Instrumentation
- p50/p95/p99 latency
- candidate counts before and after pruning
- success contribution estimate
- cache hit ratio
- repairability score

### Pseudocode
```text
fn pattern_61(ctx):
    budget = ctx.latency_budget
    exact = cheap_exact(ctx)
    graph = bounded_graph(ctx, exact)
    sem = maybe_semantic(ctx, budget, graph)
    merged = dedup_merge(exact, graph, sem)
    ranked = rank_with_policy(ctx, merged)
    return package(ctx, ranked[:ctx.limit])
```
## 62. Memory Decay pattern 2

### Objective
Optimize memory decay under bounded latency and bounded noise.

### Preconditions
- namespace is validated
- policy context is loaded
- latency budget is available
- tier capabilities are known

### Heuristic
1. start with the cheapest exact signals
2. cap expansion by query class
3. only pay semantic cost when exact and graph signals are insufficient
4. preserve explainability and provenance
5. update reinforcement or decay after outcome feedback

### Failure risks
- candidate explosion
- stale data bias
- low-confidence amplification
- cross-session interference

### Instrumentation
- p50/p95/p99 latency
- candidate counts before and after pruning
- success contribution estimate
- cache hit ratio
- repairability score

### Pseudocode
```text
fn pattern_62(ctx):
    budget = ctx.latency_budget
    exact = cheap_exact(ctx)
    graph = bounded_graph(ctx, exact)
    sem = maybe_semantic(ctx, budget, graph)
    merged = dedup_merge(exact, graph, sem)
    ranked = rank_with_policy(ctx, merged)
    return package(ctx, ranked[:ctx.limit])
```
## 63. Memory Decay pattern 3

### Objective
Optimize memory decay under bounded latency and bounded noise.

### Preconditions
- namespace is validated
- policy context is loaded
- latency budget is available
- tier capabilities are known

### Heuristic
1. start with the cheapest exact signals
2. cap expansion by query class
3. only pay semantic cost when exact and graph signals are insufficient
4. preserve explainability and provenance
5. update reinforcement or decay after outcome feedback

### Failure risks
- candidate explosion
- stale data bias
- low-confidence amplification
- cross-session interference

### Instrumentation
- p50/p95/p99 latency
- candidate counts before and after pruning
- success contribution estimate
- cache hit ratio
- repairability score

### Pseudocode
```text
fn pattern_63(ctx):
    budget = ctx.latency_budget
    exact = cheap_exact(ctx)
    graph = bounded_graph(ctx, exact)
    sem = maybe_semantic(ctx, budget, graph)
    merged = dedup_merge(exact, graph, sem)
    ranked = rank_with_policy(ctx, merged)
    return package(ctx, ranked[:ctx.limit])
```
## 64. Memory Decay pattern 4

### Objective
Optimize memory decay under bounded latency and bounded noise.

### Preconditions
- namespace is validated
- policy context is loaded
- latency budget is available
- tier capabilities are known

### Heuristic
1. start with the cheapest exact signals
2. cap expansion by query class
3. only pay semantic cost when exact and graph signals are insufficient
4. preserve explainability and provenance
5. update reinforcement or decay after outcome feedback

### Failure risks
- candidate explosion
- stale data bias
- low-confidence amplification
- cross-session interference

### Instrumentation
- p50/p95/p99 latency
- candidate counts before and after pruning
- success contribution estimate
- cache hit ratio
- repairability score

### Pseudocode
```text
fn pattern_64(ctx):
    budget = ctx.latency_budget
    exact = cheap_exact(ctx)
    graph = bounded_graph(ctx, exact)
    sem = maybe_semantic(ctx, budget, graph)
    merged = dedup_merge(exact, graph, sem)
    ranked = rank_with_policy(ctx, merged)
    return package(ctx, ranked[:ctx.limit])
```
## 65. Memory Decay pattern 5

### Objective
Optimize memory decay under bounded latency and bounded noise.

### Preconditions
- namespace is validated
- policy context is loaded
- latency budget is available
- tier capabilities are known

### Heuristic
1. start with the cheapest exact signals
2. cap expansion by query class
3. only pay semantic cost when exact and graph signals are insufficient
4. preserve explainability and provenance
5. update reinforcement or decay after outcome feedback

### Failure risks
- candidate explosion
- stale data bias
- low-confidence amplification
- cross-session interference

### Instrumentation
- p50/p95/p99 latency
- candidate counts before and after pruning
- success contribution estimate
- cache hit ratio
- repairability score

### Pseudocode
```text
fn pattern_65(ctx):
    budget = ctx.latency_budget
    exact = cheap_exact(ctx)
    graph = bounded_graph(ctx, exact)
    sem = maybe_semantic(ctx, budget, graph)
    merged = dedup_merge(exact, graph, sem)
    ranked = rank_with_policy(ctx, merged)
    return package(ctx, ranked[:ctx.limit])
```
## 66. Memory Decay pattern 6

### Objective
Optimize memory decay under bounded latency and bounded noise.

### Preconditions
- namespace is validated
- policy context is loaded
- latency budget is available
- tier capabilities are known

### Heuristic
1. start with the cheapest exact signals
2. cap expansion by query class
3. only pay semantic cost when exact and graph signals are insufficient
4. preserve explainability and provenance
5. update reinforcement or decay after outcome feedback

### Failure risks
- candidate explosion
- stale data bias
- low-confidence amplification
- cross-session interference

### Instrumentation
- p50/p95/p99 latency
- candidate counts before and after pruning
- success contribution estimate
- cache hit ratio
- repairability score

### Pseudocode
```text
fn pattern_66(ctx):
    budget = ctx.latency_budget
    exact = cheap_exact(ctx)
    graph = bounded_graph(ctx, exact)
    sem = maybe_semantic(ctx, budget, graph)
    merged = dedup_merge(exact, graph, sem)
    ranked = rank_with_policy(ctx, merged)
    return package(ctx, ranked[:ctx.limit])
```
## 67. Memory Decay pattern 7

### Objective
Optimize memory decay under bounded latency and bounded noise.

### Preconditions
- namespace is validated
- policy context is loaded
- latency budget is available
- tier capabilities are known

### Heuristic
1. start with the cheapest exact signals
2. cap expansion by query class
3. only pay semantic cost when exact and graph signals are insufficient
4. preserve explainability and provenance
5. update reinforcement or decay after outcome feedback

### Failure risks
- candidate explosion
- stale data bias
- low-confidence amplification
- cross-session interference

### Instrumentation
- p50/p95/p99 latency
- candidate counts before and after pruning
- success contribution estimate
- cache hit ratio
- repairability score

### Pseudocode
```text
fn pattern_67(ctx):
    budget = ctx.latency_budget
    exact = cheap_exact(ctx)
    graph = bounded_graph(ctx, exact)
    sem = maybe_semantic(ctx, budget, graph)
    merged = dedup_merge(exact, graph, sem)
    ranked = rank_with_policy(ctx, merged)
    return package(ctx, ranked[:ctx.limit])
```
## 68. Memory Decay pattern 8

### Objective
Optimize memory decay under bounded latency and bounded noise.

### Preconditions
- namespace is validated
- policy context is loaded
- latency budget is available
- tier capabilities are known

### Heuristic
1. start with the cheapest exact signals
2. cap expansion by query class
3. only pay semantic cost when exact and graph signals are insufficient
4. preserve explainability and provenance
5. update reinforcement or decay after outcome feedback

### Failure risks
- candidate explosion
- stale data bias
- low-confidence amplification
- cross-session interference

### Instrumentation
- p50/p95/p99 latency
- candidate counts before and after pruning
- success contribution estimate
- cache hit ratio
- repairability score

### Pseudocode
```text
fn pattern_68(ctx):
    budget = ctx.latency_budget
    exact = cheap_exact(ctx)
    graph = bounded_graph(ctx, exact)
    sem = maybe_semantic(ctx, budget, graph)
    merged = dedup_merge(exact, graph, sem)
    ranked = rank_with_policy(ctx, merged)
    return package(ctx, ranked[:ctx.limit])
```
## 69. Memory Decay pattern 9

### Objective
Optimize memory decay under bounded latency and bounded noise.

### Preconditions
- namespace is validated
- policy context is loaded
- latency budget is available
- tier capabilities are known

### Heuristic
1. start with the cheapest exact signals
2. cap expansion by query class
3. only pay semantic cost when exact and graph signals are insufficient
4. preserve explainability and provenance
5. update reinforcement or decay after outcome feedback

### Failure risks
- candidate explosion
- stale data bias
- low-confidence amplification
- cross-session interference

### Instrumentation
- p50/p95/p99 latency
- candidate counts before and after pruning
- success contribution estimate
- cache hit ratio
- repairability score

### Pseudocode
```text
fn pattern_69(ctx):
    budget = ctx.latency_budget
    exact = cheap_exact(ctx)
    graph = bounded_graph(ctx, exact)
    sem = maybe_semantic(ctx, budget, graph)
    merged = dedup_merge(exact, graph, sem)
    ranked = rank_with_policy(ctx, merged)
    return package(ctx, ranked[:ctx.limit])
```
## 70. Memory Decay pattern 10

### Objective
Optimize memory decay under bounded latency and bounded noise.

### Preconditions
- namespace is validated
- policy context is loaded
- latency budget is available
- tier capabilities are known

### Heuristic
1. start with the cheapest exact signals
2. cap expansion by query class
3. only pay semantic cost when exact and graph signals are insufficient
4. preserve explainability and provenance
5. update reinforcement or decay after outcome feedback

### Failure risks
- candidate explosion
- stale data bias
- low-confidence amplification
- cross-session interference

### Instrumentation
- p50/p95/p99 latency
- candidate counts before and after pruning
- success contribution estimate
- cache hit ratio
- repairability score

### Pseudocode
```text
fn pattern_70(ctx):
    budget = ctx.latency_budget
    exact = cheap_exact(ctx)
    graph = bounded_graph(ctx, exact)
    sem = maybe_semantic(ctx, budget, graph)
    merged = dedup_merge(exact, graph, sem)
    ranked = rank_with_policy(ctx, merged)
    return package(ctx, ranked[:ctx.limit])
```
## 71. Memory Decay pattern 11

### Objective
Optimize memory decay under bounded latency and bounded noise.

### Preconditions
- namespace is validated
- policy context is loaded
- latency budget is available
- tier capabilities are known

### Heuristic
1. start with the cheapest exact signals
2. cap expansion by query class
3. only pay semantic cost when exact and graph signals are insufficient
4. preserve explainability and provenance
5. update reinforcement or decay after outcome feedback

### Failure risks
- candidate explosion
- stale data bias
- low-confidence amplification
- cross-session interference

### Instrumentation
- p50/p95/p99 latency
- candidate counts before and after pruning
- success contribution estimate
- cache hit ratio
- repairability score

### Pseudocode
```text
fn pattern_71(ctx):
    budget = ctx.latency_budget
    exact = cheap_exact(ctx)
    graph = bounded_graph(ctx, exact)
    sem = maybe_semantic(ctx, budget, graph)
    merged = dedup_merge(exact, graph, sem)
    ranked = rank_with_policy(ctx, merged)
    return package(ctx, ranked[:ctx.limit])
```
## 72. Memory Decay pattern 12

### Objective
Optimize memory decay under bounded latency and bounded noise.

### Preconditions
- namespace is validated
- policy context is loaded
- latency budget is available
- tier capabilities are known

### Heuristic
1. start with the cheapest exact signals
2. cap expansion by query class
3. only pay semantic cost when exact and graph signals are insufficient
4. preserve explainability and provenance
5. update reinforcement or decay after outcome feedback

### Failure risks
- candidate explosion
- stale data bias
- low-confidence amplification
- cross-session interference

### Instrumentation
- p50/p95/p99 latency
- candidate counts before and after pruning
- success contribution estimate
- cache hit ratio
- repairability score

### Pseudocode
```text
fn pattern_72(ctx):
    budget = ctx.latency_budget
    exact = cheap_exact(ctx)
    graph = bounded_graph(ctx, exact)
    sem = maybe_semantic(ctx, budget, graph)
    merged = dedup_merge(exact, graph, sem)
    ranked = rank_with_policy(ctx, merged)
    return package(ctx, ranked[:ctx.limit])
```
## 73. Memory Decay pattern 13

### Objective
Optimize memory decay under bounded latency and bounded noise.

### Preconditions
- namespace is validated
- policy context is loaded
- latency budget is available
- tier capabilities are known

### Heuristic
1. start with the cheapest exact signals
2. cap expansion by query class
3. only pay semantic cost when exact and graph signals are insufficient
4. preserve explainability and provenance
5. update reinforcement or decay after outcome feedback

### Failure risks
- candidate explosion
- stale data bias
- low-confidence amplification
- cross-session interference

### Instrumentation
- p50/p95/p99 latency
- candidate counts before and after pruning
- success contribution estimate
- cache hit ratio
- repairability score

### Pseudocode
```text
fn pattern_73(ctx):
    budget = ctx.latency_budget
    exact = cheap_exact(ctx)
    graph = bounded_graph(ctx, exact)
    sem = maybe_semantic(ctx, budget, graph)
    merged = dedup_merge(exact, graph, sem)
    ranked = rank_with_policy(ctx, merged)
    return package(ctx, ranked[:ctx.limit])
```
## 74. Memory Decay pattern 14

### Objective
Optimize memory decay under bounded latency and bounded noise.

### Preconditions
- namespace is validated
- policy context is loaded
- latency budget is available
- tier capabilities are known

### Heuristic
1. start with the cheapest exact signals
2. cap expansion by query class
3. only pay semantic cost when exact and graph signals are insufficient
4. preserve explainability and provenance
5. update reinforcement or decay after outcome feedback

### Failure risks
- candidate explosion
- stale data bias
- low-confidence amplification
- cross-session interference

### Instrumentation
- p50/p95/p99 latency
- candidate counts before and after pruning
- success contribution estimate
- cache hit ratio
- repairability score

### Pseudocode
```text
fn pattern_74(ctx):
    budget = ctx.latency_budget
    exact = cheap_exact(ctx)
    graph = bounded_graph(ctx, exact)
    sem = maybe_semantic(ctx, budget, graph)
    merged = dedup_merge(exact, graph, sem)
    ranked = rank_with_policy(ctx, merged)
    return package(ctx, ranked[:ctx.limit])
```
## 75. Memory Decay pattern 15

### Objective
Optimize memory decay under bounded latency and bounded noise.

### Preconditions
- namespace is validated
- policy context is loaded
- latency budget is available
- tier capabilities are known

### Heuristic
1. start with the cheapest exact signals
2. cap expansion by query class
3. only pay semantic cost when exact and graph signals are insufficient
4. preserve explainability and provenance
5. update reinforcement or decay after outcome feedback

### Failure risks
- candidate explosion
- stale data bias
- low-confidence amplification
- cross-session interference

### Instrumentation
- p50/p95/p99 latency
- candidate counts before and after pruning
- success contribution estimate
- cache hit ratio
- repairability score

### Pseudocode
```text
fn pattern_75(ctx):
    budget = ctx.latency_budget
    exact = cheap_exact(ctx)
    graph = bounded_graph(ctx, exact)
    sem = maybe_semantic(ctx, budget, graph)
    merged = dedup_merge(exact, graph, sem)
    ranked = rank_with_policy(ctx, merged)
    return package(ctx, ranked[:ctx.limit])
```
## 76. Memory Decay pattern 16

### Objective
Optimize memory decay under bounded latency and bounded noise.

### Preconditions
- namespace is validated
- policy context is loaded
- latency budget is available
- tier capabilities are known

### Heuristic
1. start with the cheapest exact signals
2. cap expansion by query class
3. only pay semantic cost when exact and graph signals are insufficient
4. preserve explainability and provenance
5. update reinforcement or decay after outcome feedback

### Failure risks
- candidate explosion
- stale data bias
- low-confidence amplification
- cross-session interference

### Instrumentation
- p50/p95/p99 latency
- candidate counts before and after pruning
- success contribution estimate
- cache hit ratio
- repairability score

### Pseudocode
```text
fn pattern_76(ctx):
    budget = ctx.latency_budget
    exact = cheap_exact(ctx)
    graph = bounded_graph(ctx, exact)
    sem = maybe_semantic(ctx, budget, graph)
    merged = dedup_merge(exact, graph, sem)
    ranked = rank_with_policy(ctx, merged)
    return package(ctx, ranked[:ctx.limit])
```
## 77. Memory Decay pattern 17

### Objective
Optimize memory decay under bounded latency and bounded noise.

### Preconditions
- namespace is validated
- policy context is loaded
- latency budget is available
- tier capabilities are known

### Heuristic
1. start with the cheapest exact signals
2. cap expansion by query class
3. only pay semantic cost when exact and graph signals are insufficient
4. preserve explainability and provenance
5. update reinforcement or decay after outcome feedback

### Failure risks
- candidate explosion
- stale data bias
- low-confidence amplification
- cross-session interference

### Instrumentation
- p50/p95/p99 latency
- candidate counts before and after pruning
- success contribution estimate
- cache hit ratio
- repairability score

### Pseudocode
```text
fn pattern_77(ctx):
    budget = ctx.latency_budget
    exact = cheap_exact(ctx)
    graph = bounded_graph(ctx, exact)
    sem = maybe_semantic(ctx, budget, graph)
    merged = dedup_merge(exact, graph, sem)
    ranked = rank_with_policy(ctx, merged)
    return package(ctx, ranked[:ctx.limit])
```
## 78. Memory Decay pattern 18

### Objective
Optimize memory decay under bounded latency and bounded noise.

### Preconditions
- namespace is validated
- policy context is loaded
- latency budget is available
- tier capabilities are known

### Heuristic
1. start with the cheapest exact signals
2. cap expansion by query class
3. only pay semantic cost when exact and graph signals are insufficient
4. preserve explainability and provenance
5. update reinforcement or decay after outcome feedback

### Failure risks
- candidate explosion
- stale data bias
- low-confidence amplification
- cross-session interference

### Instrumentation
- p50/p95/p99 latency
- candidate counts before and after pruning
- success contribution estimate
- cache hit ratio
- repairability score

### Pseudocode
```text
fn pattern_78(ctx):
    budget = ctx.latency_budget
    exact = cheap_exact(ctx)
    graph = bounded_graph(ctx, exact)
    sem = maybe_semantic(ctx, budget, graph)
    merged = dedup_merge(exact, graph, sem)
    ranked = rank_with_policy(ctx, merged)
    return package(ctx, ranked[:ctx.limit])
```
## 79. Memory Decay pattern 19

### Objective
Optimize memory decay under bounded latency and bounded noise.

### Preconditions
- namespace is validated
- policy context is loaded
- latency budget is available
- tier capabilities are known

### Heuristic
1. start with the cheapest exact signals
2. cap expansion by query class
3. only pay semantic cost when exact and graph signals are insufficient
4. preserve explainability and provenance
5. update reinforcement or decay after outcome feedback

### Failure risks
- candidate explosion
- stale data bias
- low-confidence amplification
- cross-session interference

### Instrumentation
- p50/p95/p99 latency
- candidate counts before and after pruning
- success contribution estimate
- cache hit ratio
- repairability score

### Pseudocode
```text
fn pattern_79(ctx):
    budget = ctx.latency_budget
    exact = cheap_exact(ctx)
    graph = bounded_graph(ctx, exact)
    sem = maybe_semantic(ctx, budget, graph)
    merged = dedup_merge(exact, graph, sem)
    ranked = rank_with_policy(ctx, merged)
    return package(ctx, ranked[:ctx.limit])
```
## 80. Memory Decay pattern 20

### Objective
Optimize memory decay under bounded latency and bounded noise.

### Preconditions
- namespace is validated
- policy context is loaded
- latency budget is available
- tier capabilities are known

### Heuristic
1. start with the cheapest exact signals
2. cap expansion by query class
3. only pay semantic cost when exact and graph signals are insufficient
4. preserve explainability and provenance
5. update reinforcement or decay after outcome feedback

### Failure risks
- candidate explosion
- stale data bias
- low-confidence amplification
- cross-session interference

### Instrumentation
- p50/p95/p99 latency
- candidate counts before and after pruning
- success contribution estimate
- cache hit ratio
- repairability score

### Pseudocode
```text
fn pattern_80(ctx):
    budget = ctx.latency_budget
    exact = cheap_exact(ctx)
    graph = bounded_graph(ctx, exact)
    sem = maybe_semantic(ctx, budget, graph)
    merged = dedup_merge(exact, graph, sem)
    ranked = rank_with_policy(ctx, merged)
    return package(ctx, ranked[:ctx.limit])
```
## 81. Memory Decay pattern 21

### Objective
Optimize memory decay under bounded latency and bounded noise.

### Preconditions
- namespace is validated
- policy context is loaded
- latency budget is available
- tier capabilities are known

### Heuristic
1. start with the cheapest exact signals
2. cap expansion by query class
3. only pay semantic cost when exact and graph signals are insufficient
4. preserve explainability and provenance
5. update reinforcement or decay after outcome feedback

### Failure risks
- candidate explosion
- stale data bias
- low-confidence amplification
- cross-session interference

### Instrumentation
- p50/p95/p99 latency
- candidate counts before and after pruning
- success contribution estimate
- cache hit ratio
- repairability score

### Pseudocode
```text
fn pattern_81(ctx):
    budget = ctx.latency_budget
    exact = cheap_exact(ctx)
    graph = bounded_graph(ctx, exact)
    sem = maybe_semantic(ctx, budget, graph)
    merged = dedup_merge(exact, graph, sem)
    ranked = rank_with_policy(ctx, merged)
    return package(ctx, ranked[:ctx.limit])
```
## 82. Memory Decay pattern 22

### Objective
Optimize memory decay under bounded latency and bounded noise.

### Preconditions
- namespace is validated
- policy context is loaded
- latency budget is available
- tier capabilities are known

### Heuristic
1. start with the cheapest exact signals
2. cap expansion by query class
3. only pay semantic cost when exact and graph signals are insufficient
4. preserve explainability and provenance
5. update reinforcement or decay after outcome feedback

### Failure risks
- candidate explosion
- stale data bias
- low-confidence amplification
- cross-session interference

### Instrumentation
- p50/p95/p99 latency
- candidate counts before and after pruning
- success contribution estimate
- cache hit ratio
- repairability score

### Pseudocode
```text
fn pattern_82(ctx):
    budget = ctx.latency_budget
    exact = cheap_exact(ctx)
    graph = bounded_graph(ctx, exact)
    sem = maybe_semantic(ctx, budget, graph)
    merged = dedup_merge(exact, graph, sem)
    ranked = rank_with_policy(ctx, merged)
    return package(ctx, ranked[:ctx.limit])
```
## 83. Memory Decay pattern 23

### Objective
Optimize memory decay under bounded latency and bounded noise.

### Preconditions
- namespace is validated
- policy context is loaded
- latency budget is available
- tier capabilities are known

### Heuristic
1. start with the cheapest exact signals
2. cap expansion by query class
3. only pay semantic cost when exact and graph signals are insufficient
4. preserve explainability and provenance
5. update reinforcement or decay after outcome feedback

### Failure risks
- candidate explosion
- stale data bias
- low-confidence amplification
- cross-session interference

### Instrumentation
- p50/p95/p99 latency
- candidate counts before and after pruning
- success contribution estimate
- cache hit ratio
- repairability score

### Pseudocode
```text
fn pattern_83(ctx):
    budget = ctx.latency_budget
    exact = cheap_exact(ctx)
    graph = bounded_graph(ctx, exact)
    sem = maybe_semantic(ctx, budget, graph)
    merged = dedup_merge(exact, graph, sem)
    ranked = rank_with_policy(ctx, merged)
    return package(ctx, ranked[:ctx.limit])
```
## 84. Memory Decay pattern 24

### Objective
Optimize memory decay under bounded latency and bounded noise.

### Preconditions
- namespace is validated
- policy context is loaded
- latency budget is available
- tier capabilities are known

### Heuristic
1. start with the cheapest exact signals
2. cap expansion by query class
3. only pay semantic cost when exact and graph signals are insufficient
4. preserve explainability and provenance
5. update reinforcement or decay after outcome feedback

### Failure risks
- candidate explosion
- stale data bias
- low-confidence amplification
- cross-session interference

### Instrumentation
- p50/p95/p99 latency
- candidate counts before and after pruning
- success contribution estimate
- cache hit ratio
- repairability score

### Pseudocode
```text
fn pattern_84(ctx):
    budget = ctx.latency_budget
    exact = cheap_exact(ctx)
    graph = bounded_graph(ctx, exact)
    sem = maybe_semantic(ctx, budget, graph)
    merged = dedup_merge(exact, graph, sem)
    ranked = rank_with_policy(ctx, merged)
    return package(ctx, ranked[:ctx.limit])
```
## 85. Memory Decay pattern 25

### Objective
Optimize memory decay under bounded latency and bounded noise.

### Preconditions
- namespace is validated
- policy context is loaded
- latency budget is available
- tier capabilities are known

### Heuristic
1. start with the cheapest exact signals
2. cap expansion by query class
3. only pay semantic cost when exact and graph signals are insufficient
4. preserve explainability and provenance
5. update reinforcement or decay after outcome feedback

### Failure risks
- candidate explosion
- stale data bias
- low-confidence amplification
- cross-session interference

### Instrumentation
- p50/p95/p99 latency
- candidate counts before and after pruning
- success contribution estimate
- cache hit ratio
- repairability score

### Pseudocode
```text
fn pattern_85(ctx):
    budget = ctx.latency_budget
    exact = cheap_exact(ctx)
    graph = bounded_graph(ctx, exact)
    sem = maybe_semantic(ctx, budget, graph)
    merged = dedup_merge(exact, graph, sem)
    ranked = rank_with_policy(ctx, merged)
    return package(ctx, ranked[:ctx.limit])
```
## 86. Memory Decay pattern 26

### Objective
Optimize memory decay under bounded latency and bounded noise.

### Preconditions
- namespace is validated
- policy context is loaded
- latency budget is available
- tier capabilities are known

### Heuristic
1. start with the cheapest exact signals
2. cap expansion by query class
3. only pay semantic cost when exact and graph signals are insufficient
4. preserve explainability and provenance
5. update reinforcement or decay after outcome feedback

### Failure risks
- candidate explosion
- stale data bias
- low-confidence amplification
- cross-session interference

### Instrumentation
- p50/p95/p99 latency
- candidate counts before and after pruning
- success contribution estimate
- cache hit ratio
- repairability score

### Pseudocode
```text
fn pattern_86(ctx):
    budget = ctx.latency_budget
    exact = cheap_exact(ctx)
    graph = bounded_graph(ctx, exact)
    sem = maybe_semantic(ctx, budget, graph)
    merged = dedup_merge(exact, graph, sem)
    ranked = rank_with_policy(ctx, merged)
    return package(ctx, ranked[:ctx.limit])
```
## 87. Memory Decay pattern 27

### Objective
Optimize memory decay under bounded latency and bounded noise.

### Preconditions
- namespace is validated
- policy context is loaded
- latency budget is available
- tier capabilities are known

### Heuristic
1. start with the cheapest exact signals
2. cap expansion by query class
3. only pay semantic cost when exact and graph signals are insufficient
4. preserve explainability and provenance
5. update reinforcement or decay after outcome feedback

### Failure risks
- candidate explosion
- stale data bias
- low-confidence amplification
- cross-session interference

### Instrumentation
- p50/p95/p99 latency
- candidate counts before and after pruning
- success contribution estimate
- cache hit ratio
- repairability score

### Pseudocode
```text
fn pattern_87(ctx):
    budget = ctx.latency_budget
    exact = cheap_exact(ctx)
    graph = bounded_graph(ctx, exact)
    sem = maybe_semantic(ctx, budget, graph)
    merged = dedup_merge(exact, graph, sem)
    ranked = rank_with_policy(ctx, merged)
    return package(ctx, ranked[:ctx.limit])
```
## 88. Memory Decay pattern 28

### Objective
Optimize memory decay under bounded latency and bounded noise.

### Preconditions
- namespace is validated
- policy context is loaded
- latency budget is available
- tier capabilities are known

### Heuristic
1. start with the cheapest exact signals
2. cap expansion by query class
3. only pay semantic cost when exact and graph signals are insufficient
4. preserve explainability and provenance
5. update reinforcement or decay after outcome feedback

### Failure risks
- candidate explosion
- stale data bias
- low-confidence amplification
- cross-session interference

### Instrumentation
- p50/p95/p99 latency
- candidate counts before and after pruning
- success contribution estimate
- cache hit ratio
- repairability score

### Pseudocode
```text
fn pattern_88(ctx):
    budget = ctx.latency_budget
    exact = cheap_exact(ctx)
    graph = bounded_graph(ctx, exact)
    sem = maybe_semantic(ctx, budget, graph)
    merged = dedup_merge(exact, graph, sem)
    ranked = rank_with_policy(ctx, merged)
    return package(ctx, ranked[:ctx.limit])
```
## 89. Memory Decay pattern 29

### Objective
Optimize memory decay under bounded latency and bounded noise.

### Preconditions
- namespace is validated
- policy context is loaded
- latency budget is available
- tier capabilities are known

### Heuristic
1. start with the cheapest exact signals
2. cap expansion by query class
3. only pay semantic cost when exact and graph signals are insufficient
4. preserve explainability and provenance
5. update reinforcement or decay after outcome feedback

### Failure risks
- candidate explosion
- stale data bias
- low-confidence amplification
- cross-session interference

### Instrumentation
- p50/p95/p99 latency
- candidate counts before and after pruning
- success contribution estimate
- cache hit ratio
- repairability score

### Pseudocode
```text
fn pattern_89(ctx):
    budget = ctx.latency_budget
    exact = cheap_exact(ctx)
    graph = bounded_graph(ctx, exact)
    sem = maybe_semantic(ctx, budget, graph)
    merged = dedup_merge(exact, graph, sem)
    ranked = rank_with_policy(ctx, merged)
    return package(ctx, ranked[:ctx.limit])
```
## 90. Memory Decay pattern 30

### Objective
Optimize memory decay under bounded latency and bounded noise.

### Preconditions
- namespace is validated
- policy context is loaded
- latency budget is available
- tier capabilities are known

### Heuristic
1. start with the cheapest exact signals
2. cap expansion by query class
3. only pay semantic cost when exact and graph signals are insufficient
4. preserve explainability and provenance
5. update reinforcement or decay after outcome feedback

### Failure risks
- candidate explosion
- stale data bias
- low-confidence amplification
- cross-session interference

### Instrumentation
- p50/p95/p99 latency
- candidate counts before and after pruning
- success contribution estimate
- cache hit ratio
- repairability score

### Pseudocode
```text
fn pattern_90(ctx):
    budget = ctx.latency_budget
    exact = cheap_exact(ctx)
    graph = bounded_graph(ctx, exact)
    sem = maybe_semantic(ctx, budget, graph)
    merged = dedup_merge(exact, graph, sem)
    ranked = rank_with_policy(ctx, merged)
    return package(ctx, ranked[:ctx.limit])
```
## 91. Promotion pattern 1

### Objective
Optimize promotion under bounded latency and bounded noise.

### Preconditions
- namespace is validated
- policy context is loaded
- latency budget is available
- tier capabilities are known

### Heuristic
1. start with the cheapest exact signals
2. cap expansion by query class
3. only pay semantic cost when exact and graph signals are insufficient
4. preserve explainability and provenance
5. update reinforcement or decay after outcome feedback

### Failure risks
- candidate explosion
- stale data bias
- low-confidence amplification
- cross-session interference

### Instrumentation
- p50/p95/p99 latency
- candidate counts before and after pruning
- success contribution estimate
- cache hit ratio
- repairability score

### Pseudocode
```text
fn pattern_91(ctx):
    budget = ctx.latency_budget
    exact = cheap_exact(ctx)
    graph = bounded_graph(ctx, exact)
    sem = maybe_semantic(ctx, budget, graph)
    merged = dedup_merge(exact, graph, sem)
    ranked = rank_with_policy(ctx, merged)
    return package(ctx, ranked[:ctx.limit])
```
## 92. Promotion pattern 2

### Objective
Optimize promotion under bounded latency and bounded noise.

### Preconditions
- namespace is validated
- policy context is loaded
- latency budget is available
- tier capabilities are known

### Heuristic
1. start with the cheapest exact signals
2. cap expansion by query class
3. only pay semantic cost when exact and graph signals are insufficient
4. preserve explainability and provenance
5. update reinforcement or decay after outcome feedback

### Failure risks
- candidate explosion
- stale data bias
- low-confidence amplification
- cross-session interference

### Instrumentation
- p50/p95/p99 latency
- candidate counts before and after pruning
- success contribution estimate
- cache hit ratio
- repairability score

### Pseudocode
```text
fn pattern_92(ctx):
    budget = ctx.latency_budget
    exact = cheap_exact(ctx)
    graph = bounded_graph(ctx, exact)
    sem = maybe_semantic(ctx, budget, graph)
    merged = dedup_merge(exact, graph, sem)
    ranked = rank_with_policy(ctx, merged)
    return package(ctx, ranked[:ctx.limit])
```
## 93. Promotion pattern 3

### Objective
Optimize promotion under bounded latency and bounded noise.

### Preconditions
- namespace is validated
- policy context is loaded
- latency budget is available
- tier capabilities are known

### Heuristic
1. start with the cheapest exact signals
2. cap expansion by query class
3. only pay semantic cost when exact and graph signals are insufficient
4. preserve explainability and provenance
5. update reinforcement or decay after outcome feedback

### Failure risks
- candidate explosion
- stale data bias
- low-confidence amplification
- cross-session interference

### Instrumentation
- p50/p95/p99 latency
- candidate counts before and after pruning
- success contribution estimate
- cache hit ratio
- repairability score

### Pseudocode
```text
fn pattern_93(ctx):
    budget = ctx.latency_budget
    exact = cheap_exact(ctx)
    graph = bounded_graph(ctx, exact)
    sem = maybe_semantic(ctx, budget, graph)
    merged = dedup_merge(exact, graph, sem)
    ranked = rank_with_policy(ctx, merged)
    return package(ctx, ranked[:ctx.limit])
```
## 94. Promotion pattern 4

### Objective
Optimize promotion under bounded latency and bounded noise.

### Preconditions
- namespace is validated
- policy context is loaded
- latency budget is available
- tier capabilities are known

### Heuristic
1. start with the cheapest exact signals
2. cap expansion by query class
3. only pay semantic cost when exact and graph signals are insufficient
4. preserve explainability and provenance
5. update reinforcement or decay after outcome feedback

### Failure risks
- candidate explosion
- stale data bias
- low-confidence amplification
- cross-session interference

### Instrumentation
- p50/p95/p99 latency
- candidate counts before and after pruning
- success contribution estimate
- cache hit ratio
- repairability score

### Pseudocode
```text
fn pattern_94(ctx):
    budget = ctx.latency_budget
    exact = cheap_exact(ctx)
    graph = bounded_graph(ctx, exact)
    sem = maybe_semantic(ctx, budget, graph)
    merged = dedup_merge(exact, graph, sem)
    ranked = rank_with_policy(ctx, merged)
    return package(ctx, ranked[:ctx.limit])
```
## 95. Promotion pattern 5

### Objective
Optimize promotion under bounded latency and bounded noise.

### Preconditions
- namespace is validated
- policy context is loaded
- latency budget is available
- tier capabilities are known

### Heuristic
1. start with the cheapest exact signals
2. cap expansion by query class
3. only pay semantic cost when exact and graph signals are insufficient
4. preserve explainability and provenance
5. update reinforcement or decay after outcome feedback

### Failure risks
- candidate explosion
- stale data bias
- low-confidence amplification
- cross-session interference

### Instrumentation
- p50/p95/p99 latency
- candidate counts before and after pruning
- success contribution estimate
- cache hit ratio
- repairability score

### Pseudocode
```text
fn pattern_95(ctx):
    budget = ctx.latency_budget
    exact = cheap_exact(ctx)
    graph = bounded_graph(ctx, exact)
    sem = maybe_semantic(ctx, budget, graph)
    merged = dedup_merge(exact, graph, sem)
    ranked = rank_with_policy(ctx, merged)
    return package(ctx, ranked[:ctx.limit])
```
## 96. Promotion pattern 6

### Objective
Optimize promotion under bounded latency and bounded noise.

### Preconditions
- namespace is validated
- policy context is loaded
- latency budget is available
- tier capabilities are known

### Heuristic
1. start with the cheapest exact signals
2. cap expansion by query class
3. only pay semantic cost when exact and graph signals are insufficient
4. preserve explainability and provenance
5. update reinforcement or decay after outcome feedback

### Failure risks
- candidate explosion
- stale data bias
- low-confidence amplification
- cross-session interference

### Instrumentation
- p50/p95/p99 latency
- candidate counts before and after pruning
- success contribution estimate
- cache hit ratio
- repairability score

### Pseudocode
```text
fn pattern_96(ctx):
    budget = ctx.latency_budget
    exact = cheap_exact(ctx)
    graph = bounded_graph(ctx, exact)
    sem = maybe_semantic(ctx, budget, graph)
    merged = dedup_merge(exact, graph, sem)
    ranked = rank_with_policy(ctx, merged)
    return package(ctx, ranked[:ctx.limit])
```
## 97. Promotion pattern 7

### Objective
Optimize promotion under bounded latency and bounded noise.

### Preconditions
- namespace is validated
- policy context is loaded
- latency budget is available
- tier capabilities are known

### Heuristic
1. start with the cheapest exact signals
2. cap expansion by query class
3. only pay semantic cost when exact and graph signals are insufficient
4. preserve explainability and provenance
5. update reinforcement or decay after outcome feedback

### Failure risks
- candidate explosion
- stale data bias
- low-confidence amplification
- cross-session interference

### Instrumentation
- p50/p95/p99 latency
- candidate counts before and after pruning
- success contribution estimate
- cache hit ratio
- repairability score

### Pseudocode
```text
fn pattern_97(ctx):
    budget = ctx.latency_budget
    exact = cheap_exact(ctx)
    graph = bounded_graph(ctx, exact)
    sem = maybe_semantic(ctx, budget, graph)
    merged = dedup_merge(exact, graph, sem)
    ranked = rank_with_policy(ctx, merged)
    return package(ctx, ranked[:ctx.limit])
```
## 98. Promotion pattern 8

### Objective
Optimize promotion under bounded latency and bounded noise.

### Preconditions
- namespace is validated
- policy context is loaded
- latency budget is available
- tier capabilities are known

### Heuristic
1. start with the cheapest exact signals
2. cap expansion by query class
3. only pay semantic cost when exact and graph signals are insufficient
4. preserve explainability and provenance
5. update reinforcement or decay after outcome feedback

### Failure risks
- candidate explosion
- stale data bias
- low-confidence amplification
- cross-session interference

### Instrumentation
- p50/p95/p99 latency
- candidate counts before and after pruning
- success contribution estimate
- cache hit ratio
- repairability score

### Pseudocode
```text
fn pattern_98(ctx):
    budget = ctx.latency_budget
    exact = cheap_exact(ctx)
    graph = bounded_graph(ctx, exact)
    sem = maybe_semantic(ctx, budget, graph)
    merged = dedup_merge(exact, graph, sem)
    ranked = rank_with_policy(ctx, merged)
    return package(ctx, ranked[:ctx.limit])
```
## 99. Promotion pattern 9

### Objective
Optimize promotion under bounded latency and bounded noise.

### Preconditions
- namespace is validated
- policy context is loaded
- latency budget is available
- tier capabilities are known

### Heuristic
1. start with the cheapest exact signals
2. cap expansion by query class
3. only pay semantic cost when exact and graph signals are insufficient
4. preserve explainability and provenance
5. update reinforcement or decay after outcome feedback

### Failure risks
- candidate explosion
- stale data bias
- low-confidence amplification
- cross-session interference

### Instrumentation
- p50/p95/p99 latency
- candidate counts before and after pruning
- success contribution estimate
- cache hit ratio
- repairability score

### Pseudocode
```text
fn pattern_99(ctx):
    budget = ctx.latency_budget
    exact = cheap_exact(ctx)
    graph = bounded_graph(ctx, exact)
    sem = maybe_semantic(ctx, budget, graph)
    merged = dedup_merge(exact, graph, sem)
    ranked = rank_with_policy(ctx, merged)
    return package(ctx, ranked[:ctx.limit])
```
## 100. Promotion pattern 10

### Objective
Optimize promotion under bounded latency and bounded noise.

### Preconditions
- namespace is validated
- policy context is loaded
- latency budget is available
- tier capabilities are known

### Heuristic
1. start with the cheapest exact signals
2. cap expansion by query class
3. only pay semantic cost when exact and graph signals are insufficient
4. preserve explainability and provenance
5. update reinforcement or decay after outcome feedback

### Failure risks
- candidate explosion
- stale data bias
- low-confidence amplification
- cross-session interference

### Instrumentation
- p50/p95/p99 latency
- candidate counts before and after pruning
- success contribution estimate
- cache hit ratio
- repairability score

### Pseudocode
```text
fn pattern_100(ctx):
    budget = ctx.latency_budget
    exact = cheap_exact(ctx)
    graph = bounded_graph(ctx, exact)
    sem = maybe_semantic(ctx, budget, graph)
    merged = dedup_merge(exact, graph, sem)
    ranked = rank_with_policy(ctx, merged)
    return package(ctx, ranked[:ctx.limit])
```
## 101. Promotion pattern 11

### Objective
Optimize promotion under bounded latency and bounded noise.

### Preconditions
- namespace is validated
- policy context is loaded
- latency budget is available
- tier capabilities are known

### Heuristic
1. start with the cheapest exact signals
2. cap expansion by query class
3. only pay semantic cost when exact and graph signals are insufficient
4. preserve explainability and provenance
5. update reinforcement or decay after outcome feedback

### Failure risks
- candidate explosion
- stale data bias
- low-confidence amplification
- cross-session interference

### Instrumentation
- p50/p95/p99 latency
- candidate counts before and after pruning
- success contribution estimate
- cache hit ratio
- repairability score

### Pseudocode
```text
fn pattern_101(ctx):
    budget = ctx.latency_budget
    exact = cheap_exact(ctx)
    graph = bounded_graph(ctx, exact)
    sem = maybe_semantic(ctx, budget, graph)
    merged = dedup_merge(exact, graph, sem)
    ranked = rank_with_policy(ctx, merged)
    return package(ctx, ranked[:ctx.limit])
```
## 102. Promotion pattern 12

### Objective
Optimize promotion under bounded latency and bounded noise.

### Preconditions
- namespace is validated
- policy context is loaded
- latency budget is available
- tier capabilities are known

### Heuristic
1. start with the cheapest exact signals
2. cap expansion by query class
3. only pay semantic cost when exact and graph signals are insufficient
4. preserve explainability and provenance
5. update reinforcement or decay after outcome feedback

### Failure risks
- candidate explosion
- stale data bias
- low-confidence amplification
- cross-session interference

### Instrumentation
- p50/p95/p99 latency
- candidate counts before and after pruning
- success contribution estimate
- cache hit ratio
- repairability score

### Pseudocode
```text
fn pattern_102(ctx):
    budget = ctx.latency_budget
    exact = cheap_exact(ctx)
    graph = bounded_graph(ctx, exact)
    sem = maybe_semantic(ctx, budget, graph)
    merged = dedup_merge(exact, graph, sem)
    ranked = rank_with_policy(ctx, merged)
    return package(ctx, ranked[:ctx.limit])
```
## 103. Promotion pattern 13

### Objective
Optimize promotion under bounded latency and bounded noise.

### Preconditions
- namespace is validated
- policy context is loaded
- latency budget is available
- tier capabilities are known

### Heuristic
1. start with the cheapest exact signals
2. cap expansion by query class
3. only pay semantic cost when exact and graph signals are insufficient
4. preserve explainability and provenance
5. update reinforcement or decay after outcome feedback

### Failure risks
- candidate explosion
- stale data bias
- low-confidence amplification
- cross-session interference

### Instrumentation
- p50/p95/p99 latency
- candidate counts before and after pruning
- success contribution estimate
- cache hit ratio
- repairability score

### Pseudocode
```text
fn pattern_103(ctx):
    budget = ctx.latency_budget
    exact = cheap_exact(ctx)
    graph = bounded_graph(ctx, exact)
    sem = maybe_semantic(ctx, budget, graph)
    merged = dedup_merge(exact, graph, sem)
    ranked = rank_with_policy(ctx, merged)
    return package(ctx, ranked[:ctx.limit])
```
## 104. Promotion pattern 14

### Objective
Optimize promotion under bounded latency and bounded noise.

### Preconditions
- namespace is validated
- policy context is loaded
- latency budget is available
- tier capabilities are known

### Heuristic
1. start with the cheapest exact signals
2. cap expansion by query class
3. only pay semantic cost when exact and graph signals are insufficient
4. preserve explainability and provenance
5. update reinforcement or decay after outcome feedback

### Failure risks
- candidate explosion
- stale data bias
- low-confidence amplification
- cross-session interference

### Instrumentation
- p50/p95/p99 latency
- candidate counts before and after pruning
- success contribution estimate
- cache hit ratio
- repairability score

### Pseudocode
```text
fn pattern_104(ctx):
    budget = ctx.latency_budget
    exact = cheap_exact(ctx)
    graph = bounded_graph(ctx, exact)
    sem = maybe_semantic(ctx, budget, graph)
    merged = dedup_merge(exact, graph, sem)
    ranked = rank_with_policy(ctx, merged)
    return package(ctx, ranked[:ctx.limit])
```
## 105. Promotion pattern 15

### Objective
Optimize promotion under bounded latency and bounded noise.

### Preconditions
- namespace is validated
- policy context is loaded
- latency budget is available
- tier capabilities are known

### Heuristic
1. start with the cheapest exact signals
2. cap expansion by query class
3. only pay semantic cost when exact and graph signals are insufficient
4. preserve explainability and provenance
5. update reinforcement or decay after outcome feedback

### Failure risks
- candidate explosion
- stale data bias
- low-confidence amplification
- cross-session interference

### Instrumentation
- p50/p95/p99 latency
- candidate counts before and after pruning
- success contribution estimate
- cache hit ratio
- repairability score

### Pseudocode
```text
fn pattern_105(ctx):
    budget = ctx.latency_budget
    exact = cheap_exact(ctx)
    graph = bounded_graph(ctx, exact)
    sem = maybe_semantic(ctx, budget, graph)
    merged = dedup_merge(exact, graph, sem)
    ranked = rank_with_policy(ctx, merged)
    return package(ctx, ranked[:ctx.limit])
```
## 106. Promotion pattern 16

### Objective
Optimize promotion under bounded latency and bounded noise.

### Preconditions
- namespace is validated
- policy context is loaded
- latency budget is available
- tier capabilities are known

### Heuristic
1. start with the cheapest exact signals
2. cap expansion by query class
3. only pay semantic cost when exact and graph signals are insufficient
4. preserve explainability and provenance
5. update reinforcement or decay after outcome feedback

### Failure risks
- candidate explosion
- stale data bias
- low-confidence amplification
- cross-session interference

### Instrumentation
- p50/p95/p99 latency
- candidate counts before and after pruning
- success contribution estimate
- cache hit ratio
- repairability score

### Pseudocode
```text
fn pattern_106(ctx):
    budget = ctx.latency_budget
    exact = cheap_exact(ctx)
    graph = bounded_graph(ctx, exact)
    sem = maybe_semantic(ctx, budget, graph)
    merged = dedup_merge(exact, graph, sem)
    ranked = rank_with_policy(ctx, merged)
    return package(ctx, ranked[:ctx.limit])
```
## 107. Promotion pattern 17

### Objective
Optimize promotion under bounded latency and bounded noise.

### Preconditions
- namespace is validated
- policy context is loaded
- latency budget is available
- tier capabilities are known

### Heuristic
1. start with the cheapest exact signals
2. cap expansion by query class
3. only pay semantic cost when exact and graph signals are insufficient
4. preserve explainability and provenance
5. update reinforcement or decay after outcome feedback

### Failure risks
- candidate explosion
- stale data bias
- low-confidence amplification
- cross-session interference

### Instrumentation
- p50/p95/p99 latency
- candidate counts before and after pruning
- success contribution estimate
- cache hit ratio
- repairability score

### Pseudocode
```text
fn pattern_107(ctx):
    budget = ctx.latency_budget
    exact = cheap_exact(ctx)
    graph = bounded_graph(ctx, exact)
    sem = maybe_semantic(ctx, budget, graph)
    merged = dedup_merge(exact, graph, sem)
    ranked = rank_with_policy(ctx, merged)
    return package(ctx, ranked[:ctx.limit])
```
## 108. Promotion pattern 18

### Objective
Optimize promotion under bounded latency and bounded noise.

### Preconditions
- namespace is validated
- policy context is loaded
- latency budget is available
- tier capabilities are known

### Heuristic
1. start with the cheapest exact signals
2. cap expansion by query class
3. only pay semantic cost when exact and graph signals are insufficient
4. preserve explainability and provenance
5. update reinforcement or decay after outcome feedback

### Failure risks
- candidate explosion
- stale data bias
- low-confidence amplification
- cross-session interference

### Instrumentation
- p50/p95/p99 latency
- candidate counts before and after pruning
- success contribution estimate
- cache hit ratio
- repairability score

### Pseudocode
```text
fn pattern_108(ctx):
    budget = ctx.latency_budget
    exact = cheap_exact(ctx)
    graph = bounded_graph(ctx, exact)
    sem = maybe_semantic(ctx, budget, graph)
    merged = dedup_merge(exact, graph, sem)
    ranked = rank_with_policy(ctx, merged)
    return package(ctx, ranked[:ctx.limit])
```
## 109. Promotion pattern 19

### Objective
Optimize promotion under bounded latency and bounded noise.

### Preconditions
- namespace is validated
- policy context is loaded
- latency budget is available
- tier capabilities are known

### Heuristic
1. start with the cheapest exact signals
2. cap expansion by query class
3. only pay semantic cost when exact and graph signals are insufficient
4. preserve explainability and provenance
5. update reinforcement or decay after outcome feedback

### Failure risks
- candidate explosion
- stale data bias
- low-confidence amplification
- cross-session interference

### Instrumentation
- p50/p95/p99 latency
- candidate counts before and after pruning
- success contribution estimate
- cache hit ratio
- repairability score

### Pseudocode
```text
fn pattern_109(ctx):
    budget = ctx.latency_budget
    exact = cheap_exact(ctx)
    graph = bounded_graph(ctx, exact)
    sem = maybe_semantic(ctx, budget, graph)
    merged = dedup_merge(exact, graph, sem)
    ranked = rank_with_policy(ctx, merged)
    return package(ctx, ranked[:ctx.limit])
```
## 110. Promotion pattern 20

### Objective
Optimize promotion under bounded latency and bounded noise.

### Preconditions
- namespace is validated
- policy context is loaded
- latency budget is available
- tier capabilities are known

### Heuristic
1. start with the cheapest exact signals
2. cap expansion by query class
3. only pay semantic cost when exact and graph signals are insufficient
4. preserve explainability and provenance
5. update reinforcement or decay after outcome feedback

### Failure risks
- candidate explosion
- stale data bias
- low-confidence amplification
- cross-session interference

### Instrumentation
- p50/p95/p99 latency
- candidate counts before and after pruning
- success contribution estimate
- cache hit ratio
- repairability score

### Pseudocode
```text
fn pattern_110(ctx):
    budget = ctx.latency_budget
    exact = cheap_exact(ctx)
    graph = bounded_graph(ctx, exact)
    sem = maybe_semantic(ctx, budget, graph)
    merged = dedup_merge(exact, graph, sem)
    ranked = rank_with_policy(ctx, merged)
    return package(ctx, ranked[:ctx.limit])
```
## 111. Promotion pattern 21

### Objective
Optimize promotion under bounded latency and bounded noise.

### Preconditions
- namespace is validated
- policy context is loaded
- latency budget is available
- tier capabilities are known

### Heuristic
1. start with the cheapest exact signals
2. cap expansion by query class
3. only pay semantic cost when exact and graph signals are insufficient
4. preserve explainability and provenance
5. update reinforcement or decay after outcome feedback

### Failure risks
- candidate explosion
- stale data bias
- low-confidence amplification
- cross-session interference

### Instrumentation
- p50/p95/p99 latency
- candidate counts before and after pruning
- success contribution estimate
- cache hit ratio
- repairability score

### Pseudocode
```text
fn pattern_111(ctx):
    budget = ctx.latency_budget
    exact = cheap_exact(ctx)
    graph = bounded_graph(ctx, exact)
    sem = maybe_semantic(ctx, budget, graph)
    merged = dedup_merge(exact, graph, sem)
    ranked = rank_with_policy(ctx, merged)
    return package(ctx, ranked[:ctx.limit])
```
## 112. Promotion pattern 22

### Objective
Optimize promotion under bounded latency and bounded noise.

### Preconditions
- namespace is validated
- policy context is loaded
- latency budget is available
- tier capabilities are known

### Heuristic
1. start with the cheapest exact signals
2. cap expansion by query class
3. only pay semantic cost when exact and graph signals are insufficient
4. preserve explainability and provenance
5. update reinforcement or decay after outcome feedback

### Failure risks
- candidate explosion
- stale data bias
- low-confidence amplification
- cross-session interference

### Instrumentation
- p50/p95/p99 latency
- candidate counts before and after pruning
- success contribution estimate
- cache hit ratio
- repairability score

### Pseudocode
```text
fn pattern_112(ctx):
    budget = ctx.latency_budget
    exact = cheap_exact(ctx)
    graph = bounded_graph(ctx, exact)
    sem = maybe_semantic(ctx, budget, graph)
    merged = dedup_merge(exact, graph, sem)
    ranked = rank_with_policy(ctx, merged)
    return package(ctx, ranked[:ctx.limit])
```
## 113. Promotion pattern 23

### Objective
Optimize promotion under bounded latency and bounded noise.

### Preconditions
- namespace is validated
- policy context is loaded
- latency budget is available
- tier capabilities are known

### Heuristic
1. start with the cheapest exact signals
2. cap expansion by query class
3. only pay semantic cost when exact and graph signals are insufficient
4. preserve explainability and provenance
5. update reinforcement or decay after outcome feedback

### Failure risks
- candidate explosion
- stale data bias
- low-confidence amplification
- cross-session interference

### Instrumentation
- p50/p95/p99 latency
- candidate counts before and after pruning
- success contribution estimate
- cache hit ratio
- repairability score

### Pseudocode
```text
fn pattern_113(ctx):
    budget = ctx.latency_budget
    exact = cheap_exact(ctx)
    graph = bounded_graph(ctx, exact)
    sem = maybe_semantic(ctx, budget, graph)
    merged = dedup_merge(exact, graph, sem)
    ranked = rank_with_policy(ctx, merged)
    return package(ctx, ranked[:ctx.limit])
```
## 114. Promotion pattern 24

### Objective
Optimize promotion under bounded latency and bounded noise.

### Preconditions
- namespace is validated
- policy context is loaded
- latency budget is available
- tier capabilities are known

### Heuristic
1. start with the cheapest exact signals
2. cap expansion by query class
3. only pay semantic cost when exact and graph signals are insufficient
4. preserve explainability and provenance
5. update reinforcement or decay after outcome feedback

### Failure risks
- candidate explosion
- stale data bias
- low-confidence amplification
- cross-session interference

### Instrumentation
- p50/p95/p99 latency
- candidate counts before and after pruning
- success contribution estimate
- cache hit ratio
- repairability score

### Pseudocode
```text
fn pattern_114(ctx):
    budget = ctx.latency_budget
    exact = cheap_exact(ctx)
    graph = bounded_graph(ctx, exact)
    sem = maybe_semantic(ctx, budget, graph)
    merged = dedup_merge(exact, graph, sem)
    ranked = rank_with_policy(ctx, merged)
    return package(ctx, ranked[:ctx.limit])
```
## 115. Promotion pattern 25

### Objective
Optimize promotion under bounded latency and bounded noise.

### Preconditions
- namespace is validated
- policy context is loaded
- latency budget is available
- tier capabilities are known

### Heuristic
1. start with the cheapest exact signals
2. cap expansion by query class
3. only pay semantic cost when exact and graph signals are insufficient
4. preserve explainability and provenance
5. update reinforcement or decay after outcome feedback

### Failure risks
- candidate explosion
- stale data bias
- low-confidence amplification
- cross-session interference

### Instrumentation
- p50/p95/p99 latency
- candidate counts before and after pruning
- success contribution estimate
- cache hit ratio
- repairability score

### Pseudocode
```text
fn pattern_115(ctx):
    budget = ctx.latency_budget
    exact = cheap_exact(ctx)
    graph = bounded_graph(ctx, exact)
    sem = maybe_semantic(ctx, budget, graph)
    merged = dedup_merge(exact, graph, sem)
    ranked = rank_with_policy(ctx, merged)
    return package(ctx, ranked[:ctx.limit])
```
## 116. Promotion pattern 26

### Objective
Optimize promotion under bounded latency and bounded noise.

### Preconditions
- namespace is validated
- policy context is loaded
- latency budget is available
- tier capabilities are known

### Heuristic
1. start with the cheapest exact signals
2. cap expansion by query class
3. only pay semantic cost when exact and graph signals are insufficient
4. preserve explainability and provenance
5. update reinforcement or decay after outcome feedback

### Failure risks
- candidate explosion
- stale data bias
- low-confidence amplification
- cross-session interference

### Instrumentation
- p50/p95/p99 latency
- candidate counts before and after pruning
- success contribution estimate
- cache hit ratio
- repairability score

### Pseudocode
```text
fn pattern_116(ctx):
    budget = ctx.latency_budget
    exact = cheap_exact(ctx)
    graph = bounded_graph(ctx, exact)
    sem = maybe_semantic(ctx, budget, graph)
    merged = dedup_merge(exact, graph, sem)
    ranked = rank_with_policy(ctx, merged)
    return package(ctx, ranked[:ctx.limit])
```
## 117. Promotion pattern 27

### Objective
Optimize promotion under bounded latency and bounded noise.

### Preconditions
- namespace is validated
- policy context is loaded
- latency budget is available
- tier capabilities are known

### Heuristic
1. start with the cheapest exact signals
2. cap expansion by query class
3. only pay semantic cost when exact and graph signals are insufficient
4. preserve explainability and provenance
5. update reinforcement or decay after outcome feedback

### Failure risks
- candidate explosion
- stale data bias
- low-confidence amplification
- cross-session interference

### Instrumentation
- p50/p95/p99 latency
- candidate counts before and after pruning
- success contribution estimate
- cache hit ratio
- repairability score

### Pseudocode
```text
fn pattern_117(ctx):
    budget = ctx.latency_budget
    exact = cheap_exact(ctx)
    graph = bounded_graph(ctx, exact)
    sem = maybe_semantic(ctx, budget, graph)
    merged = dedup_merge(exact, graph, sem)
    ranked = rank_with_policy(ctx, merged)
    return package(ctx, ranked[:ctx.limit])
```
## 118. Promotion pattern 28

### Objective
Optimize promotion under bounded latency and bounded noise.

### Preconditions
- namespace is validated
- policy context is loaded
- latency budget is available
- tier capabilities are known

### Heuristic
1. start with the cheapest exact signals
2. cap expansion by query class
3. only pay semantic cost when exact and graph signals are insufficient
4. preserve explainability and provenance
5. update reinforcement or decay after outcome feedback

### Failure risks
- candidate explosion
- stale data bias
- low-confidence amplification
- cross-session interference

### Instrumentation
- p50/p95/p99 latency
- candidate counts before and after pruning
- success contribution estimate
- cache hit ratio
- repairability score

### Pseudocode
```text
fn pattern_118(ctx):
    budget = ctx.latency_budget
    exact = cheap_exact(ctx)
    graph = bounded_graph(ctx, exact)
    sem = maybe_semantic(ctx, budget, graph)
    merged = dedup_merge(exact, graph, sem)
    ranked = rank_with_policy(ctx, merged)
    return package(ctx, ranked[:ctx.limit])
```
## 119. Promotion pattern 29

### Objective
Optimize promotion under bounded latency and bounded noise.

### Preconditions
- namespace is validated
- policy context is loaded
- latency budget is available
- tier capabilities are known

### Heuristic
1. start with the cheapest exact signals
2. cap expansion by query class
3. only pay semantic cost when exact and graph signals are insufficient
4. preserve explainability and provenance
5. update reinforcement or decay after outcome feedback

### Failure risks
- candidate explosion
- stale data bias
- low-confidence amplification
- cross-session interference

### Instrumentation
- p50/p95/p99 latency
- candidate counts before and after pruning
- success contribution estimate
- cache hit ratio
- repairability score

### Pseudocode
```text
fn pattern_119(ctx):
    budget = ctx.latency_budget
    exact = cheap_exact(ctx)
    graph = bounded_graph(ctx, exact)
    sem = maybe_semantic(ctx, budget, graph)
    merged = dedup_merge(exact, graph, sem)
    ranked = rank_with_policy(ctx, merged)
    return package(ctx, ranked[:ctx.limit])
```
## 120. Promotion pattern 30

### Objective
Optimize promotion under bounded latency and bounded noise.

### Preconditions
- namespace is validated
- policy context is loaded
- latency budget is available
- tier capabilities are known

### Heuristic
1. start with the cheapest exact signals
2. cap expansion by query class
3. only pay semantic cost when exact and graph signals are insufficient
4. preserve explainability and provenance
5. update reinforcement or decay after outcome feedback

### Failure risks
- candidate explosion
- stale data bias
- low-confidence amplification
- cross-session interference

### Instrumentation
- p50/p95/p99 latency
- candidate counts before and after pruning
- success contribution estimate
- cache hit ratio
- repairability score

### Pseudocode
```text
fn pattern_120(ctx):
    budget = ctx.latency_budget
    exact = cheap_exact(ctx)
    graph = bounded_graph(ctx, exact)
    sem = maybe_semantic(ctx, budget, graph)
    merged = dedup_merge(exact, graph, sem)
    ranked = rank_with_policy(ctx, merged)
    return package(ctx, ranked[:ctx.limit])
```
## 121. Demotion pattern 1

### Objective
Optimize demotion under bounded latency and bounded noise.

### Preconditions
- namespace is validated
- policy context is loaded
- latency budget is available
- tier capabilities are known

### Heuristic
1. start with the cheapest exact signals
2. cap expansion by query class
3. only pay semantic cost when exact and graph signals are insufficient
4. preserve explainability and provenance
5. update reinforcement or decay after outcome feedback

### Failure risks
- candidate explosion
- stale data bias
- low-confidence amplification
- cross-session interference

### Instrumentation
- p50/p95/p99 latency
- candidate counts before and after pruning
- success contribution estimate
- cache hit ratio
- repairability score

### Pseudocode
```text
fn pattern_121(ctx):
    budget = ctx.latency_budget
    exact = cheap_exact(ctx)
    graph = bounded_graph(ctx, exact)
    sem = maybe_semantic(ctx, budget, graph)
    merged = dedup_merge(exact, graph, sem)
    ranked = rank_with_policy(ctx, merged)
    return package(ctx, ranked[:ctx.limit])
```
## 122. Demotion pattern 2

### Objective
Optimize demotion under bounded latency and bounded noise.

### Preconditions
- namespace is validated
- policy context is loaded
- latency budget is available
- tier capabilities are known

### Heuristic
1. start with the cheapest exact signals
2. cap expansion by query class
3. only pay semantic cost when exact and graph signals are insufficient
4. preserve explainability and provenance
5. update reinforcement or decay after outcome feedback

### Failure risks
- candidate explosion
- stale data bias
- low-confidence amplification
- cross-session interference

### Instrumentation
- p50/p95/p99 latency
- candidate counts before and after pruning
- success contribution estimate
- cache hit ratio
- repairability score

### Pseudocode
```text
fn pattern_122(ctx):
    budget = ctx.latency_budget
    exact = cheap_exact(ctx)
    graph = bounded_graph(ctx, exact)
    sem = maybe_semantic(ctx, budget, graph)
    merged = dedup_merge(exact, graph, sem)
    ranked = rank_with_policy(ctx, merged)
    return package(ctx, ranked[:ctx.limit])
```
## 123. Demotion pattern 3

### Objective
Optimize demotion under bounded latency and bounded noise.

### Preconditions
- namespace is validated
- policy context is loaded
- latency budget is available
- tier capabilities are known

### Heuristic
1. start with the cheapest exact signals
2. cap expansion by query class
3. only pay semantic cost when exact and graph signals are insufficient
4. preserve explainability and provenance
5. update reinforcement or decay after outcome feedback

### Failure risks
- candidate explosion
- stale data bias
- low-confidence amplification
- cross-session interference

### Instrumentation
- p50/p95/p99 latency
- candidate counts before and after pruning
- success contribution estimate
- cache hit ratio
- repairability score

### Pseudocode
```text
fn pattern_123(ctx):
    budget = ctx.latency_budget
    exact = cheap_exact(ctx)
    graph = bounded_graph(ctx, exact)
    sem = maybe_semantic(ctx, budget, graph)
    merged = dedup_merge(exact, graph, sem)
    ranked = rank_with_policy(ctx, merged)
    return package(ctx, ranked[:ctx.limit])
```
## 124. Demotion pattern 4

### Objective
Optimize demotion under bounded latency and bounded noise.

### Preconditions
- namespace is validated
- policy context is loaded
- latency budget is available
- tier capabilities are known

### Heuristic
1. start with the cheapest exact signals
2. cap expansion by query class
3. only pay semantic cost when exact and graph signals are insufficient
4. preserve explainability and provenance
5. update reinforcement or decay after outcome feedback

### Failure risks
- candidate explosion
- stale data bias
- low-confidence amplification
- cross-session interference

### Instrumentation
- p50/p95/p99 latency
- candidate counts before and after pruning
- success contribution estimate
- cache hit ratio
- repairability score

### Pseudocode
```text
fn pattern_124(ctx):
    budget = ctx.latency_budget
    exact = cheap_exact(ctx)
    graph = bounded_graph(ctx, exact)
    sem = maybe_semantic(ctx, budget, graph)
    merged = dedup_merge(exact, graph, sem)
    ranked = rank_with_policy(ctx, merged)
    return package(ctx, ranked[:ctx.limit])
```
## 125. Demotion pattern 5

### Objective
Optimize demotion under bounded latency and bounded noise.

### Preconditions
- namespace is validated
- policy context is loaded
- latency budget is available
- tier capabilities are known

### Heuristic
1. start with the cheapest exact signals
2. cap expansion by query class
3. only pay semantic cost when exact and graph signals are insufficient
4. preserve explainability and provenance
5. update reinforcement or decay after outcome feedback

### Failure risks
- candidate explosion
- stale data bias
- low-confidence amplification
- cross-session interference

### Instrumentation
- p50/p95/p99 latency
- candidate counts before and after pruning
- success contribution estimate
- cache hit ratio
- repairability score

### Pseudocode
```text
fn pattern_125(ctx):
    budget = ctx.latency_budget
    exact = cheap_exact(ctx)
    graph = bounded_graph(ctx, exact)
    sem = maybe_semantic(ctx, budget, graph)
    merged = dedup_merge(exact, graph, sem)
    ranked = rank_with_policy(ctx, merged)
    return package(ctx, ranked[:ctx.limit])
```
## 126. Demotion pattern 6

### Objective
Optimize demotion under bounded latency and bounded noise.

### Preconditions
- namespace is validated
- policy context is loaded
- latency budget is available
- tier capabilities are known

### Heuristic
1. start with the cheapest exact signals
2. cap expansion by query class
3. only pay semantic cost when exact and graph signals are insufficient
4. preserve explainability and provenance
5. update reinforcement or decay after outcome feedback

### Failure risks
- candidate explosion
- stale data bias
- low-confidence amplification
- cross-session interference

### Instrumentation
- p50/p95/p99 latency
- candidate counts before and after pruning
- success contribution estimate
- cache hit ratio
- repairability score

### Pseudocode
```text
fn pattern_126(ctx):
    budget = ctx.latency_budget
    exact = cheap_exact(ctx)
    graph = bounded_graph(ctx, exact)
    sem = maybe_semantic(ctx, budget, graph)
    merged = dedup_merge(exact, graph, sem)
    ranked = rank_with_policy(ctx, merged)
    return package(ctx, ranked[:ctx.limit])
```
## 127. Demotion pattern 7

### Objective
Optimize demotion under bounded latency and bounded noise.

### Preconditions
- namespace is validated
- policy context is loaded
- latency budget is available
- tier capabilities are known

### Heuristic
1. start with the cheapest exact signals
2. cap expansion by query class
3. only pay semantic cost when exact and graph signals are insufficient
4. preserve explainability and provenance
5. update reinforcement or decay after outcome feedback

### Failure risks
- candidate explosion
- stale data bias
- low-confidence amplification
- cross-session interference

### Instrumentation
- p50/p95/p99 latency
- candidate counts before and after pruning
- success contribution estimate
- cache hit ratio
- repairability score

### Pseudocode
```text
fn pattern_127(ctx):
    budget = ctx.latency_budget
    exact = cheap_exact(ctx)
    graph = bounded_graph(ctx, exact)
    sem = maybe_semantic(ctx, budget, graph)
    merged = dedup_merge(exact, graph, sem)
    ranked = rank_with_policy(ctx, merged)
    return package(ctx, ranked[:ctx.limit])
```
## 128. Demotion pattern 8

### Objective
Optimize demotion under bounded latency and bounded noise.

### Preconditions
- namespace is validated
- policy context is loaded
- latency budget is available
- tier capabilities are known

### Heuristic
1. start with the cheapest exact signals
2. cap expansion by query class
3. only pay semantic cost when exact and graph signals are insufficient
4. preserve explainability and provenance
5. update reinforcement or decay after outcome feedback

### Failure risks
- candidate explosion
- stale data bias
- low-confidence amplification
- cross-session interference

### Instrumentation
- p50/p95/p99 latency
- candidate counts before and after pruning
- success contribution estimate
- cache hit ratio
- repairability score

### Pseudocode
```text
fn pattern_128(ctx):
    budget = ctx.latency_budget
    exact = cheap_exact(ctx)
    graph = bounded_graph(ctx, exact)
    sem = maybe_semantic(ctx, budget, graph)
    merged = dedup_merge(exact, graph, sem)
    ranked = rank_with_policy(ctx, merged)
    return package(ctx, ranked[:ctx.limit])
```
## 129. Demotion pattern 9

### Objective
Optimize demotion under bounded latency and bounded noise.

### Preconditions
- namespace is validated
- policy context is loaded
- latency budget is available
- tier capabilities are known

### Heuristic
1. start with the cheapest exact signals
2. cap expansion by query class
3. only pay semantic cost when exact and graph signals are insufficient
4. preserve explainability and provenance
5. update reinforcement or decay after outcome feedback

### Failure risks
- candidate explosion
- stale data bias
- low-confidence amplification
- cross-session interference

### Instrumentation
- p50/p95/p99 latency
- candidate counts before and after pruning
- success contribution estimate
- cache hit ratio
- repairability score

### Pseudocode
```text
fn pattern_129(ctx):
    budget = ctx.latency_budget
    exact = cheap_exact(ctx)
    graph = bounded_graph(ctx, exact)
    sem = maybe_semantic(ctx, budget, graph)
    merged = dedup_merge(exact, graph, sem)
    ranked = rank_with_policy(ctx, merged)
    return package(ctx, ranked[:ctx.limit])
```
## 130. Demotion pattern 10

### Objective
Optimize demotion under bounded latency and bounded noise.

### Preconditions
- namespace is validated
- policy context is loaded
- latency budget is available
- tier capabilities are known

### Heuristic
1. start with the cheapest exact signals
2. cap expansion by query class
3. only pay semantic cost when exact and graph signals are insufficient
4. preserve explainability and provenance
5. update reinforcement or decay after outcome feedback

### Failure risks
- candidate explosion
- stale data bias
- low-confidence amplification
- cross-session interference

### Instrumentation
- p50/p95/p99 latency
- candidate counts before and after pruning
- success contribution estimate
- cache hit ratio
- repairability score

### Pseudocode
```text
fn pattern_130(ctx):
    budget = ctx.latency_budget
    exact = cheap_exact(ctx)
    graph = bounded_graph(ctx, exact)
    sem = maybe_semantic(ctx, budget, graph)
    merged = dedup_merge(exact, graph, sem)
    ranked = rank_with_policy(ctx, merged)
    return package(ctx, ranked[:ctx.limit])
```
## 131. Demotion pattern 11

### Objective
Optimize demotion under bounded latency and bounded noise.

### Preconditions
- namespace is validated
- policy context is loaded
- latency budget is available
- tier capabilities are known

### Heuristic
1. start with the cheapest exact signals
2. cap expansion by query class
3. only pay semantic cost when exact and graph signals are insufficient
4. preserve explainability and provenance
5. update reinforcement or decay after outcome feedback

### Failure risks
- candidate explosion
- stale data bias
- low-confidence amplification
- cross-session interference

### Instrumentation
- p50/p95/p99 latency
- candidate counts before and after pruning
- success contribution estimate
- cache hit ratio
- repairability score

### Pseudocode
```text
fn pattern_131(ctx):
    budget = ctx.latency_budget
    exact = cheap_exact(ctx)
    graph = bounded_graph(ctx, exact)
    sem = maybe_semantic(ctx, budget, graph)
    merged = dedup_merge(exact, graph, sem)
    ranked = rank_with_policy(ctx, merged)
    return package(ctx, ranked[:ctx.limit])
```
## 132. Demotion pattern 12

### Objective
Optimize demotion under bounded latency and bounded noise.

### Preconditions
- namespace is validated
- policy context is loaded
- latency budget is available
- tier capabilities are known

### Heuristic
1. start with the cheapest exact signals
2. cap expansion by query class
3. only pay semantic cost when exact and graph signals are insufficient
4. preserve explainability and provenance
5. update reinforcement or decay after outcome feedback

### Failure risks
- candidate explosion
- stale data bias
- low-confidence amplification
- cross-session interference

### Instrumentation
- p50/p95/p99 latency
- candidate counts before and after pruning
- success contribution estimate
- cache hit ratio
- repairability score

### Pseudocode
```text
fn pattern_132(ctx):
    budget = ctx.latency_budget
    exact = cheap_exact(ctx)
    graph = bounded_graph(ctx, exact)
    sem = maybe_semantic(ctx, budget, graph)
    merged = dedup_merge(exact, graph, sem)
    ranked = rank_with_policy(ctx, merged)
    return package(ctx, ranked[:ctx.limit])
```
## 133. Demotion pattern 13

### Objective
Optimize demotion under bounded latency and bounded noise.

### Preconditions
- namespace is validated
- policy context is loaded
- latency budget is available
- tier capabilities are known

### Heuristic
1. start with the cheapest exact signals
2. cap expansion by query class
3. only pay semantic cost when exact and graph signals are insufficient
4. preserve explainability and provenance
5. update reinforcement or decay after outcome feedback

### Failure risks
- candidate explosion
- stale data bias
- low-confidence amplification
- cross-session interference

### Instrumentation
- p50/p95/p99 latency
- candidate counts before and after pruning
- success contribution estimate
- cache hit ratio
- repairability score

### Pseudocode
```text
fn pattern_133(ctx):
    budget = ctx.latency_budget
    exact = cheap_exact(ctx)
    graph = bounded_graph(ctx, exact)
    sem = maybe_semantic(ctx, budget, graph)
    merged = dedup_merge(exact, graph, sem)
    ranked = rank_with_policy(ctx, merged)
    return package(ctx, ranked[:ctx.limit])
```
## 134. Demotion pattern 14

### Objective
Optimize demotion under bounded latency and bounded noise.

### Preconditions
- namespace is validated
- policy context is loaded
- latency budget is available
- tier capabilities are known

### Heuristic
1. start with the cheapest exact signals
2. cap expansion by query class
3. only pay semantic cost when exact and graph signals are insufficient
4. preserve explainability and provenance
5. update reinforcement or decay after outcome feedback

### Failure risks
- candidate explosion
- stale data bias
- low-confidence amplification
- cross-session interference

### Instrumentation
- p50/p95/p99 latency
- candidate counts before and after pruning
- success contribution estimate
- cache hit ratio
- repairability score

### Pseudocode
```text
fn pattern_134(ctx):
    budget = ctx.latency_budget
    exact = cheap_exact(ctx)
    graph = bounded_graph(ctx, exact)
    sem = maybe_semantic(ctx, budget, graph)
    merged = dedup_merge(exact, graph, sem)
    ranked = rank_with_policy(ctx, merged)
    return package(ctx, ranked[:ctx.limit])
```
## 135. Demotion pattern 15

### Objective
Optimize demotion under bounded latency and bounded noise.

### Preconditions
- namespace is validated
- policy context is loaded
- latency budget is available
- tier capabilities are known

### Heuristic
1. start with the cheapest exact signals
2. cap expansion by query class
3. only pay semantic cost when exact and graph signals are insufficient
4. preserve explainability and provenance
5. update reinforcement or decay after outcome feedback

### Failure risks
- candidate explosion
- stale data bias
- low-confidence amplification
- cross-session interference

### Instrumentation
- p50/p95/p99 latency
- candidate counts before and after pruning
- success contribution estimate
- cache hit ratio
- repairability score

### Pseudocode
```text
fn pattern_135(ctx):
    budget = ctx.latency_budget
    exact = cheap_exact(ctx)
    graph = bounded_graph(ctx, exact)
    sem = maybe_semantic(ctx, budget, graph)
    merged = dedup_merge(exact, graph, sem)
    ranked = rank_with_policy(ctx, merged)
    return package(ctx, ranked[:ctx.limit])
```
## 136. Demotion pattern 16

### Objective
Optimize demotion under bounded latency and bounded noise.

### Preconditions
- namespace is validated
- policy context is loaded
- latency budget is available
- tier capabilities are known

### Heuristic
1. start with the cheapest exact signals
2. cap expansion by query class
3. only pay semantic cost when exact and graph signals are insufficient
4. preserve explainability and provenance
5. update reinforcement or decay after outcome feedback

### Failure risks
- candidate explosion
- stale data bias
- low-confidence amplification
- cross-session interference

### Instrumentation
- p50/p95/p99 latency
- candidate counts before and after pruning
- success contribution estimate
- cache hit ratio
- repairability score

### Pseudocode
```text
fn pattern_136(ctx):
    budget = ctx.latency_budget
    exact = cheap_exact(ctx)
    graph = bounded_graph(ctx, exact)
    sem = maybe_semantic(ctx, budget, graph)
    merged = dedup_merge(exact, graph, sem)
    ranked = rank_with_policy(ctx, merged)
    return package(ctx, ranked[:ctx.limit])
```
## 137. Demotion pattern 17

### Objective
Optimize demotion under bounded latency and bounded noise.

### Preconditions
- namespace is validated
- policy context is loaded
- latency budget is available
- tier capabilities are known

### Heuristic
1. start with the cheapest exact signals
2. cap expansion by query class
3. only pay semantic cost when exact and graph signals are insufficient
4. preserve explainability and provenance
5. update reinforcement or decay after outcome feedback

### Failure risks
- candidate explosion
- stale data bias
- low-confidence amplification
- cross-session interference

### Instrumentation
- p50/p95/p99 latency
- candidate counts before and after pruning
- success contribution estimate
- cache hit ratio
- repairability score

### Pseudocode
```text
fn pattern_137(ctx):
    budget = ctx.latency_budget
    exact = cheap_exact(ctx)
    graph = bounded_graph(ctx, exact)
    sem = maybe_semantic(ctx, budget, graph)
    merged = dedup_merge(exact, graph, sem)
    ranked = rank_with_policy(ctx, merged)
    return package(ctx, ranked[:ctx.limit])
```
## 138. Demotion pattern 18

### Objective
Optimize demotion under bounded latency and bounded noise.

### Preconditions
- namespace is validated
- policy context is loaded
- latency budget is available
- tier capabilities are known

### Heuristic
1. start with the cheapest exact signals
2. cap expansion by query class
3. only pay semantic cost when exact and graph signals are insufficient
4. preserve explainability and provenance
5. update reinforcement or decay after outcome feedback

### Failure risks
- candidate explosion
- stale data bias
- low-confidence amplification
- cross-session interference

### Instrumentation
- p50/p95/p99 latency
- candidate counts before and after pruning
- success contribution estimate
- cache hit ratio
- repairability score

### Pseudocode
```text
fn pattern_138(ctx):
    budget = ctx.latency_budget
    exact = cheap_exact(ctx)
    graph = bounded_graph(ctx, exact)
    sem = maybe_semantic(ctx, budget, graph)
    merged = dedup_merge(exact, graph, sem)
    ranked = rank_with_policy(ctx, merged)
    return package(ctx, ranked[:ctx.limit])
```
## 139. Demotion pattern 19

### Objective
Optimize demotion under bounded latency and bounded noise.

### Preconditions
- namespace is validated
- policy context is loaded
- latency budget is available
- tier capabilities are known

### Heuristic
1. start with the cheapest exact signals
2. cap expansion by query class
3. only pay semantic cost when exact and graph signals are insufficient
4. preserve explainability and provenance
5. update reinforcement or decay after outcome feedback

### Failure risks
- candidate explosion
- stale data bias
- low-confidence amplification
- cross-session interference

### Instrumentation
- p50/p95/p99 latency
- candidate counts before and after pruning
- success contribution estimate
- cache hit ratio
- repairability score

### Pseudocode
```text
fn pattern_139(ctx):
    budget = ctx.latency_budget
    exact = cheap_exact(ctx)
    graph = bounded_graph(ctx, exact)
    sem = maybe_semantic(ctx, budget, graph)
    merged = dedup_merge(exact, graph, sem)
    ranked = rank_with_policy(ctx, merged)
    return package(ctx, ranked[:ctx.limit])
```
## 140. Demotion pattern 20

### Objective
Optimize demotion under bounded latency and bounded noise.

### Preconditions
- namespace is validated
- policy context is loaded
- latency budget is available
- tier capabilities are known

### Heuristic
1. start with the cheapest exact signals
2. cap expansion by query class
3. only pay semantic cost when exact and graph signals are insufficient
4. preserve explainability and provenance
5. update reinforcement or decay after outcome feedback

### Failure risks
- candidate explosion
- stale data bias
- low-confidence amplification
- cross-session interference

### Instrumentation
- p50/p95/p99 latency
- candidate counts before and after pruning
- success contribution estimate
- cache hit ratio
- repairability score

### Pseudocode
```text
fn pattern_140(ctx):
    budget = ctx.latency_budget
    exact = cheap_exact(ctx)
    graph = bounded_graph(ctx, exact)
    sem = maybe_semantic(ctx, budget, graph)
    merged = dedup_merge(exact, graph, sem)
    ranked = rank_with_policy(ctx, merged)
    return package(ctx, ranked[:ctx.limit])
```
## 141. Demotion pattern 21

### Objective
Optimize demotion under bounded latency and bounded noise.

### Preconditions
- namespace is validated
- policy context is loaded
- latency budget is available
- tier capabilities are known

### Heuristic
1. start with the cheapest exact signals
2. cap expansion by query class
3. only pay semantic cost when exact and graph signals are insufficient
4. preserve explainability and provenance
5. update reinforcement or decay after outcome feedback

### Failure risks
- candidate explosion
- stale data bias
- low-confidence amplification
- cross-session interference

### Instrumentation
- p50/p95/p99 latency
- candidate counts before and after pruning
- success contribution estimate
- cache hit ratio
- repairability score

### Pseudocode
```text
fn pattern_141(ctx):
    budget = ctx.latency_budget
    exact = cheap_exact(ctx)
    graph = bounded_graph(ctx, exact)
    sem = maybe_semantic(ctx, budget, graph)
    merged = dedup_merge(exact, graph, sem)
    ranked = rank_with_policy(ctx, merged)
    return package(ctx, ranked[:ctx.limit])
```
## 142. Demotion pattern 22

### Objective
Optimize demotion under bounded latency and bounded noise.

### Preconditions
- namespace is validated
- policy context is loaded
- latency budget is available
- tier capabilities are known

### Heuristic
1. start with the cheapest exact signals
2. cap expansion by query class
3. only pay semantic cost when exact and graph signals are insufficient
4. preserve explainability and provenance
5. update reinforcement or decay after outcome feedback

### Failure risks
- candidate explosion
- stale data bias
- low-confidence amplification
- cross-session interference

### Instrumentation
- p50/p95/p99 latency
- candidate counts before and after pruning
- success contribution estimate
- cache hit ratio
- repairability score

### Pseudocode
```text
fn pattern_142(ctx):
    budget = ctx.latency_budget
    exact = cheap_exact(ctx)
    graph = bounded_graph(ctx, exact)
    sem = maybe_semantic(ctx, budget, graph)
    merged = dedup_merge(exact, graph, sem)
    ranked = rank_with_policy(ctx, merged)
    return package(ctx, ranked[:ctx.limit])
```
## 143. Demotion pattern 23

### Objective
Optimize demotion under bounded latency and bounded noise.

### Preconditions
- namespace is validated
- policy context is loaded
- latency budget is available
- tier capabilities are known

### Heuristic
1. start with the cheapest exact signals
2. cap expansion by query class
3. only pay semantic cost when exact and graph signals are insufficient
4. preserve explainability and provenance
5. update reinforcement or decay after outcome feedback

### Failure risks
- candidate explosion
- stale data bias
- low-confidence amplification
- cross-session interference

### Instrumentation
- p50/p95/p99 latency
- candidate counts before and after pruning
- success contribution estimate
- cache hit ratio
- repairability score

### Pseudocode
```text
fn pattern_143(ctx):
    budget = ctx.latency_budget
    exact = cheap_exact(ctx)
    graph = bounded_graph(ctx, exact)
    sem = maybe_semantic(ctx, budget, graph)
    merged = dedup_merge(exact, graph, sem)
    ranked = rank_with_policy(ctx, merged)
    return package(ctx, ranked[:ctx.limit])
```
## 144. Demotion pattern 24

### Objective
Optimize demotion under bounded latency and bounded noise.

### Preconditions
- namespace is validated
- policy context is loaded
- latency budget is available
- tier capabilities are known

### Heuristic
1. start with the cheapest exact signals
2. cap expansion by query class
3. only pay semantic cost when exact and graph signals are insufficient
4. preserve explainability and provenance
5. update reinforcement or decay after outcome feedback

### Failure risks
- candidate explosion
- stale data bias
- low-confidence amplification
- cross-session interference

### Instrumentation
- p50/p95/p99 latency
- candidate counts before and after pruning
- success contribution estimate
- cache hit ratio
- repairability score

### Pseudocode
```text
fn pattern_144(ctx):
    budget = ctx.latency_budget
    exact = cheap_exact(ctx)
    graph = bounded_graph(ctx, exact)
    sem = maybe_semantic(ctx, budget, graph)
    merged = dedup_merge(exact, graph, sem)
    ranked = rank_with_policy(ctx, merged)
    return package(ctx, ranked[:ctx.limit])
```
## 145. Demotion pattern 25

### Objective
Optimize demotion under bounded latency and bounded noise.

### Preconditions
- namespace is validated
- policy context is loaded
- latency budget is available
- tier capabilities are known

### Heuristic
1. start with the cheapest exact signals
2. cap expansion by query class
3. only pay semantic cost when exact and graph signals are insufficient
4. preserve explainability and provenance
5. update reinforcement or decay after outcome feedback

### Failure risks
- candidate explosion
- stale data bias
- low-confidence amplification
- cross-session interference

### Instrumentation
- p50/p95/p99 latency
- candidate counts before and after pruning
- success contribution estimate
- cache hit ratio
- repairability score

### Pseudocode
```text
fn pattern_145(ctx):
    budget = ctx.latency_budget
    exact = cheap_exact(ctx)
    graph = bounded_graph(ctx, exact)
    sem = maybe_semantic(ctx, budget, graph)
    merged = dedup_merge(exact, graph, sem)
    ranked = rank_with_policy(ctx, merged)
    return package(ctx, ranked[:ctx.limit])
```
## 146. Demotion pattern 26

### Objective
Optimize demotion under bounded latency and bounded noise.

### Preconditions
- namespace is validated
- policy context is loaded
- latency budget is available
- tier capabilities are known

### Heuristic
1. start with the cheapest exact signals
2. cap expansion by query class
3. only pay semantic cost when exact and graph signals are insufficient
4. preserve explainability and provenance
5. update reinforcement or decay after outcome feedback

### Failure risks
- candidate explosion
- stale data bias
- low-confidence amplification
- cross-session interference

### Instrumentation
- p50/p95/p99 latency
- candidate counts before and after pruning
- success contribution estimate
- cache hit ratio
- repairability score

### Pseudocode
```text
fn pattern_146(ctx):
    budget = ctx.latency_budget
    exact = cheap_exact(ctx)
    graph = bounded_graph(ctx, exact)
    sem = maybe_semantic(ctx, budget, graph)
    merged = dedup_merge(exact, graph, sem)
    ranked = rank_with_policy(ctx, merged)
    return package(ctx, ranked[:ctx.limit])
```
## 147. Demotion pattern 27

### Objective
Optimize demotion under bounded latency and bounded noise.

### Preconditions
- namespace is validated
- policy context is loaded
- latency budget is available
- tier capabilities are known

### Heuristic
1. start with the cheapest exact signals
2. cap expansion by query class
3. only pay semantic cost when exact and graph signals are insufficient
4. preserve explainability and provenance
5. update reinforcement or decay after outcome feedback

### Failure risks
- candidate explosion
- stale data bias
- low-confidence amplification
- cross-session interference

### Instrumentation
- p50/p95/p99 latency
- candidate counts before and after pruning
- success contribution estimate
- cache hit ratio
- repairability score

### Pseudocode
```text
fn pattern_147(ctx):
    budget = ctx.latency_budget
    exact = cheap_exact(ctx)
    graph = bounded_graph(ctx, exact)
    sem = maybe_semantic(ctx, budget, graph)
    merged = dedup_merge(exact, graph, sem)
    ranked = rank_with_policy(ctx, merged)
    return package(ctx, ranked[:ctx.limit])
```
## 148. Demotion pattern 28

### Objective
Optimize demotion under bounded latency and bounded noise.

### Preconditions
- namespace is validated
- policy context is loaded
- latency budget is available
- tier capabilities are known

### Heuristic
1. start with the cheapest exact signals
2. cap expansion by query class
3. only pay semantic cost when exact and graph signals are insufficient
4. preserve explainability and provenance
5. update reinforcement or decay after outcome feedback

### Failure risks
- candidate explosion
- stale data bias
- low-confidence amplification
- cross-session interference

### Instrumentation
- p50/p95/p99 latency
- candidate counts before and after pruning
- success contribution estimate
- cache hit ratio
- repairability score

### Pseudocode
```text
fn pattern_148(ctx):
    budget = ctx.latency_budget
    exact = cheap_exact(ctx)
    graph = bounded_graph(ctx, exact)
    sem = maybe_semantic(ctx, budget, graph)
    merged = dedup_merge(exact, graph, sem)
    ranked = rank_with_policy(ctx, merged)
    return package(ctx, ranked[:ctx.limit])
```
## 149. Demotion pattern 29

### Objective
Optimize demotion under bounded latency and bounded noise.

### Preconditions
- namespace is validated
- policy context is loaded
- latency budget is available
- tier capabilities are known

### Heuristic
1. start with the cheapest exact signals
2. cap expansion by query class
3. only pay semantic cost when exact and graph signals are insufficient
4. preserve explainability and provenance
5. update reinforcement or decay after outcome feedback

### Failure risks
- candidate explosion
- stale data bias
- low-confidence amplification
- cross-session interference

### Instrumentation
- p50/p95/p99 latency
- candidate counts before and after pruning
- success contribution estimate
- cache hit ratio
- repairability score

### Pseudocode
```text
fn pattern_149(ctx):
    budget = ctx.latency_budget
    exact = cheap_exact(ctx)
    graph = bounded_graph(ctx, exact)
    sem = maybe_semantic(ctx, budget, graph)
    merged = dedup_merge(exact, graph, sem)
    ranked = rank_with_policy(ctx, merged)
    return package(ctx, ranked[:ctx.limit])
```
## 150. Demotion pattern 30

### Objective
Optimize demotion under bounded latency and bounded noise.

### Preconditions
- namespace is validated
- policy context is loaded
- latency budget is available
- tier capabilities are known

### Heuristic
1. start with the cheapest exact signals
2. cap expansion by query class
3. only pay semantic cost when exact and graph signals are insufficient
4. preserve explainability and provenance
5. update reinforcement or decay after outcome feedback

### Failure risks
- candidate explosion
- stale data bias
- low-confidence amplification
- cross-session interference

### Instrumentation
- p50/p95/p99 latency
- candidate counts before and after pruning
- success contribution estimate
- cache hit ratio
- repairability score

### Pseudocode
```text
fn pattern_150(ctx):
    budget = ctx.latency_budget
    exact = cheap_exact(ctx)
    graph = bounded_graph(ctx, exact)
    sem = maybe_semantic(ctx, budget, graph)
    merged = dedup_merge(exact, graph, sem)
    ranked = rank_with_policy(ctx, merged)
    return package(ctx, ranked[:ctx.limit])
```
## 151. Duplicate Detection pattern 1

### Objective
Optimize duplicate detection under bounded latency and bounded noise.

### Preconditions
- namespace is validated
- policy context is loaded
- latency budget is available
- tier capabilities are known

### Heuristic
1. start with the cheapest exact signals
2. cap expansion by query class
3. only pay semantic cost when exact and graph signals are insufficient
4. preserve explainability and provenance
5. update reinforcement or decay after outcome feedback

### Failure risks
- candidate explosion
- stale data bias
- low-confidence amplification
- cross-session interference

### Instrumentation
- p50/p95/p99 latency
- candidate counts before and after pruning
- success contribution estimate
- cache hit ratio
- repairability score

### Pseudocode
```text
fn pattern_151(ctx):
    budget = ctx.latency_budget
    exact = cheap_exact(ctx)
    graph = bounded_graph(ctx, exact)
    sem = maybe_semantic(ctx, budget, graph)
    merged = dedup_merge(exact, graph, sem)
    ranked = rank_with_policy(ctx, merged)
    return package(ctx, ranked[:ctx.limit])
```
## 152. Duplicate Detection pattern 2

### Objective
Optimize duplicate detection under bounded latency and bounded noise.

### Preconditions
- namespace is validated
- policy context is loaded
- latency budget is available
- tier capabilities are known

### Heuristic
1. start with the cheapest exact signals
2. cap expansion by query class
3. only pay semantic cost when exact and graph signals are insufficient
4. preserve explainability and provenance
5. update reinforcement or decay after outcome feedback

### Failure risks
- candidate explosion
- stale data bias
- low-confidence amplification
- cross-session interference

### Instrumentation
- p50/p95/p99 latency
- candidate counts before and after pruning
- success contribution estimate
- cache hit ratio
- repairability score

### Pseudocode
```text
fn pattern_152(ctx):
    budget = ctx.latency_budget
    exact = cheap_exact(ctx)
    graph = bounded_graph(ctx, exact)
    sem = maybe_semantic(ctx, budget, graph)
    merged = dedup_merge(exact, graph, sem)
    ranked = rank_with_policy(ctx, merged)
    return package(ctx, ranked[:ctx.limit])
```
## 153. Duplicate Detection pattern 3

### Objective
Optimize duplicate detection under bounded latency and bounded noise.

### Preconditions
- namespace is validated
- policy context is loaded
- latency budget is available
- tier capabilities are known

### Heuristic
1. start with the cheapest exact signals
2. cap expansion by query class
3. only pay semantic cost when exact and graph signals are insufficient
4. preserve explainability and provenance
5. update reinforcement or decay after outcome feedback

### Failure risks
- candidate explosion
- stale data bias
- low-confidence amplification
- cross-session interference

### Instrumentation
- p50/p95/p99 latency
- candidate counts before and after pruning
- success contribution estimate
- cache hit ratio
- repairability score

### Pseudocode
```text
fn pattern_153(ctx):
    budget = ctx.latency_budget
    exact = cheap_exact(ctx)
    graph = bounded_graph(ctx, exact)
    sem = maybe_semantic(ctx, budget, graph)
    merged = dedup_merge(exact, graph, sem)
    ranked = rank_with_policy(ctx, merged)
    return package(ctx, ranked[:ctx.limit])
```
## 154. Duplicate Detection pattern 4

### Objective
Optimize duplicate detection under bounded latency and bounded noise.

### Preconditions
- namespace is validated
- policy context is loaded
- latency budget is available
- tier capabilities are known

### Heuristic
1. start with the cheapest exact signals
2. cap expansion by query class
3. only pay semantic cost when exact and graph signals are insufficient
4. preserve explainability and provenance
5. update reinforcement or decay after outcome feedback

### Failure risks
- candidate explosion
- stale data bias
- low-confidence amplification
- cross-session interference

### Instrumentation
- p50/p95/p99 latency
- candidate counts before and after pruning
- success contribution estimate
- cache hit ratio
- repairability score

### Pseudocode
```text
fn pattern_154(ctx):
    budget = ctx.latency_budget
    exact = cheap_exact(ctx)
    graph = bounded_graph(ctx, exact)
    sem = maybe_semantic(ctx, budget, graph)
    merged = dedup_merge(exact, graph, sem)
    ranked = rank_with_policy(ctx, merged)
    return package(ctx, ranked[:ctx.limit])
```
## 155. Duplicate Detection pattern 5

### Objective
Optimize duplicate detection under bounded latency and bounded noise.

### Preconditions
- namespace is validated
- policy context is loaded
- latency budget is available
- tier capabilities are known

### Heuristic
1. start with the cheapest exact signals
2. cap expansion by query class
3. only pay semantic cost when exact and graph signals are insufficient
4. preserve explainability and provenance
5. update reinforcement or decay after outcome feedback

### Failure risks
- candidate explosion
- stale data bias
- low-confidence amplification
- cross-session interference

### Instrumentation
- p50/p95/p99 latency
- candidate counts before and after pruning
- success contribution estimate
- cache hit ratio
- repairability score

### Pseudocode
```text
fn pattern_155(ctx):
    budget = ctx.latency_budget
    exact = cheap_exact(ctx)
    graph = bounded_graph(ctx, exact)
    sem = maybe_semantic(ctx, budget, graph)
    merged = dedup_merge(exact, graph, sem)
    ranked = rank_with_policy(ctx, merged)
    return package(ctx, ranked[:ctx.limit])
```
## 156. Duplicate Detection pattern 6

### Objective
Optimize duplicate detection under bounded latency and bounded noise.

### Preconditions
- namespace is validated
- policy context is loaded
- latency budget is available
- tier capabilities are known

### Heuristic
1. start with the cheapest exact signals
2. cap expansion by query class
3. only pay semantic cost when exact and graph signals are insufficient
4. preserve explainability and provenance
5. update reinforcement or decay after outcome feedback

### Failure risks
- candidate explosion
- stale data bias
- low-confidence amplification
- cross-session interference

### Instrumentation
- p50/p95/p99 latency
- candidate counts before and after pruning
- success contribution estimate
- cache hit ratio
- repairability score

### Pseudocode
```text
fn pattern_156(ctx):
    budget = ctx.latency_budget
    exact = cheap_exact(ctx)
    graph = bounded_graph(ctx, exact)
    sem = maybe_semantic(ctx, budget, graph)
    merged = dedup_merge(exact, graph, sem)
    ranked = rank_with_policy(ctx, merged)
    return package(ctx, ranked[:ctx.limit])
```
## 157. Duplicate Detection pattern 7

### Objective
Optimize duplicate detection under bounded latency and bounded noise.

### Preconditions
- namespace is validated
- policy context is loaded
- latency budget is available
- tier capabilities are known

### Heuristic
1. start with the cheapest exact signals
2. cap expansion by query class
3. only pay semantic cost when exact and graph signals are insufficient
4. preserve explainability and provenance
5. update reinforcement or decay after outcome feedback

### Failure risks
- candidate explosion
- stale data bias
- low-confidence amplification
- cross-session interference

### Instrumentation
- p50/p95/p99 latency
- candidate counts before and after pruning
- success contribution estimate
- cache hit ratio
- repairability score

### Pseudocode
```text
fn pattern_157(ctx):
    budget = ctx.latency_budget
    exact = cheap_exact(ctx)
    graph = bounded_graph(ctx, exact)
    sem = maybe_semantic(ctx, budget, graph)
    merged = dedup_merge(exact, graph, sem)
    ranked = rank_with_policy(ctx, merged)
    return package(ctx, ranked[:ctx.limit])
```
## 158. Duplicate Detection pattern 8

### Objective
Optimize duplicate detection under bounded latency and bounded noise.

### Preconditions
- namespace is validated
- policy context is loaded
- latency budget is available
- tier capabilities are known

### Heuristic
1. start with the cheapest exact signals
2. cap expansion by query class
3. only pay semantic cost when exact and graph signals are insufficient
4. preserve explainability and provenance
5. update reinforcement or decay after outcome feedback

### Failure risks
- candidate explosion
- stale data bias
- low-confidence amplification
- cross-session interference

### Instrumentation
- p50/p95/p99 latency
- candidate counts before and after pruning
- success contribution estimate
- cache hit ratio
- repairability score

### Pseudocode
```text
fn pattern_158(ctx):
    budget = ctx.latency_budget
    exact = cheap_exact(ctx)
    graph = bounded_graph(ctx, exact)
    sem = maybe_semantic(ctx, budget, graph)
    merged = dedup_merge(exact, graph, sem)
    ranked = rank_with_policy(ctx, merged)
    return package(ctx, ranked[:ctx.limit])
```
## 159. Duplicate Detection pattern 9

### Objective
Optimize duplicate detection under bounded latency and bounded noise.

### Preconditions
- namespace is validated
- policy context is loaded
- latency budget is available
- tier capabilities are known

### Heuristic
1. start with the cheapest exact signals
2. cap expansion by query class
3. only pay semantic cost when exact and graph signals are insufficient
4. preserve explainability and provenance
5. update reinforcement or decay after outcome feedback

### Failure risks
- candidate explosion
- stale data bias
- low-confidence amplification
- cross-session interference

### Instrumentation
- p50/p95/p99 latency
- candidate counts before and after pruning
- success contribution estimate
- cache hit ratio
- repairability score

### Pseudocode
```text
fn pattern_159(ctx):
    budget = ctx.latency_budget
    exact = cheap_exact(ctx)
    graph = bounded_graph(ctx, exact)
    sem = maybe_semantic(ctx, budget, graph)
    merged = dedup_merge(exact, graph, sem)
    ranked = rank_with_policy(ctx, merged)
    return package(ctx, ranked[:ctx.limit])
```
## 160. Duplicate Detection pattern 10

### Objective
Optimize duplicate detection under bounded latency and bounded noise.

### Preconditions
- namespace is validated
- policy context is loaded
- latency budget is available
- tier capabilities are known

### Heuristic
1. start with the cheapest exact signals
2. cap expansion by query class
3. only pay semantic cost when exact and graph signals are insufficient
4. preserve explainability and provenance
5. update reinforcement or decay after outcome feedback

### Failure risks
- candidate explosion
- stale data bias
- low-confidence amplification
- cross-session interference

### Instrumentation
- p50/p95/p99 latency
- candidate counts before and after pruning
- success contribution estimate
- cache hit ratio
- repairability score

### Pseudocode
```text
fn pattern_160(ctx):
    budget = ctx.latency_budget
    exact = cheap_exact(ctx)
    graph = bounded_graph(ctx, exact)
    sem = maybe_semantic(ctx, budget, graph)
    merged = dedup_merge(exact, graph, sem)
    ranked = rank_with_policy(ctx, merged)
    return package(ctx, ranked[:ctx.limit])
```
## 161. Duplicate Detection pattern 11

### Objective
Optimize duplicate detection under bounded latency and bounded noise.

### Preconditions
- namespace is validated
- policy context is loaded
- latency budget is available
- tier capabilities are known

### Heuristic
1. start with the cheapest exact signals
2. cap expansion by query class
3. only pay semantic cost when exact and graph signals are insufficient
4. preserve explainability and provenance
5. update reinforcement or decay after outcome feedback

### Failure risks
- candidate explosion
- stale data bias
- low-confidence amplification
- cross-session interference

### Instrumentation
- p50/p95/p99 latency
- candidate counts before and after pruning
- success contribution estimate
- cache hit ratio
- repairability score

### Pseudocode
```text
fn pattern_161(ctx):
    budget = ctx.latency_budget
    exact = cheap_exact(ctx)
    graph = bounded_graph(ctx, exact)
    sem = maybe_semantic(ctx, budget, graph)
    merged = dedup_merge(exact, graph, sem)
    ranked = rank_with_policy(ctx, merged)
    return package(ctx, ranked[:ctx.limit])
```
## 162. Duplicate Detection pattern 12

### Objective
Optimize duplicate detection under bounded latency and bounded noise.

### Preconditions
- namespace is validated
- policy context is loaded
- latency budget is available
- tier capabilities are known

### Heuristic
1. start with the cheapest exact signals
2. cap expansion by query class
3. only pay semantic cost when exact and graph signals are insufficient
4. preserve explainability and provenance
5. update reinforcement or decay after outcome feedback

### Failure risks
- candidate explosion
- stale data bias
- low-confidence amplification
- cross-session interference

### Instrumentation
- p50/p95/p99 latency
- candidate counts before and after pruning
- success contribution estimate
- cache hit ratio
- repairability score

### Pseudocode
```text
fn pattern_162(ctx):
    budget = ctx.latency_budget
    exact = cheap_exact(ctx)
    graph = bounded_graph(ctx, exact)
    sem = maybe_semantic(ctx, budget, graph)
    merged = dedup_merge(exact, graph, sem)
    ranked = rank_with_policy(ctx, merged)
    return package(ctx, ranked[:ctx.limit])
```
## 163. Duplicate Detection pattern 13

### Objective
Optimize duplicate detection under bounded latency and bounded noise.

### Preconditions
- namespace is validated
- policy context is loaded
- latency budget is available
- tier capabilities are known

### Heuristic
1. start with the cheapest exact signals
2. cap expansion by query class
3. only pay semantic cost when exact and graph signals are insufficient
4. preserve explainability and provenance
5. update reinforcement or decay after outcome feedback

### Failure risks
- candidate explosion
- stale data bias
- low-confidence amplification
- cross-session interference

### Instrumentation
- p50/p95/p99 latency
- candidate counts before and after pruning
- success contribution estimate
- cache hit ratio
- repairability score

### Pseudocode
```text
fn pattern_163(ctx):
    budget = ctx.latency_budget
    exact = cheap_exact(ctx)
    graph = bounded_graph(ctx, exact)
    sem = maybe_semantic(ctx, budget, graph)
    merged = dedup_merge(exact, graph, sem)
    ranked = rank_with_policy(ctx, merged)
    return package(ctx, ranked[:ctx.limit])
```
## 164. Duplicate Detection pattern 14

### Objective
Optimize duplicate detection under bounded latency and bounded noise.

### Preconditions
- namespace is validated
- policy context is loaded
- latency budget is available
- tier capabilities are known

### Heuristic
1. start with the cheapest exact signals
2. cap expansion by query class
3. only pay semantic cost when exact and graph signals are insufficient
4. preserve explainability and provenance
5. update reinforcement or decay after outcome feedback

### Failure risks
- candidate explosion
- stale data bias
- low-confidence amplification
- cross-session interference

### Instrumentation
- p50/p95/p99 latency
- candidate counts before and after pruning
- success contribution estimate
- cache hit ratio
- repairability score

### Pseudocode
```text
fn pattern_164(ctx):
    budget = ctx.latency_budget
    exact = cheap_exact(ctx)
    graph = bounded_graph(ctx, exact)
    sem = maybe_semantic(ctx, budget, graph)
    merged = dedup_merge(exact, graph, sem)
    ranked = rank_with_policy(ctx, merged)
    return package(ctx, ranked[:ctx.limit])
```
## 165. Duplicate Detection pattern 15

### Objective
Optimize duplicate detection under bounded latency and bounded noise.

### Preconditions
- namespace is validated
- policy context is loaded
- latency budget is available
- tier capabilities are known

### Heuristic
1. start with the cheapest exact signals
2. cap expansion by query class
3. only pay semantic cost when exact and graph signals are insufficient
4. preserve explainability and provenance
5. update reinforcement or decay after outcome feedback

### Failure risks
- candidate explosion
- stale data bias
- low-confidence amplification
- cross-session interference

### Instrumentation
- p50/p95/p99 latency
- candidate counts before and after pruning
- success contribution estimate
- cache hit ratio
- repairability score

### Pseudocode
```text
fn pattern_165(ctx):
    budget = ctx.latency_budget
    exact = cheap_exact(ctx)
    graph = bounded_graph(ctx, exact)
    sem = maybe_semantic(ctx, budget, graph)
    merged = dedup_merge(exact, graph, sem)
    ranked = rank_with_policy(ctx, merged)
    return package(ctx, ranked[:ctx.limit])
```
## 166. Duplicate Detection pattern 16

### Objective
Optimize duplicate detection under bounded latency and bounded noise.

### Preconditions
- namespace is validated
- policy context is loaded
- latency budget is available
- tier capabilities are known

### Heuristic
1. start with the cheapest exact signals
2. cap expansion by query class
3. only pay semantic cost when exact and graph signals are insufficient
4. preserve explainability and provenance
5. update reinforcement or decay after outcome feedback

### Failure risks
- candidate explosion
- stale data bias
- low-confidence amplification
- cross-session interference

### Instrumentation
- p50/p95/p99 latency
- candidate counts before and after pruning
- success contribution estimate
- cache hit ratio
- repairability score

### Pseudocode
```text
fn pattern_166(ctx):
    budget = ctx.latency_budget
    exact = cheap_exact(ctx)
    graph = bounded_graph(ctx, exact)
    sem = maybe_semantic(ctx, budget, graph)
    merged = dedup_merge(exact, graph, sem)
    ranked = rank_with_policy(ctx, merged)
    return package(ctx, ranked[:ctx.limit])
```
## 167. Duplicate Detection pattern 17

### Objective
Optimize duplicate detection under bounded latency and bounded noise.

### Preconditions
- namespace is validated
- policy context is loaded
- latency budget is available
- tier capabilities are known

### Heuristic
1. start with the cheapest exact signals
2. cap expansion by query class
3. only pay semantic cost when exact and graph signals are insufficient
4. preserve explainability and provenance
5. update reinforcement or decay after outcome feedback

### Failure risks
- candidate explosion
- stale data bias
- low-confidence amplification
- cross-session interference

### Instrumentation
- p50/p95/p99 latency
- candidate counts before and after pruning
- success contribution estimate
- cache hit ratio
- repairability score

### Pseudocode
```text
fn pattern_167(ctx):
    budget = ctx.latency_budget
    exact = cheap_exact(ctx)
    graph = bounded_graph(ctx, exact)
    sem = maybe_semantic(ctx, budget, graph)
    merged = dedup_merge(exact, graph, sem)
    ranked = rank_with_policy(ctx, merged)
    return package(ctx, ranked[:ctx.limit])
```
## 168. Duplicate Detection pattern 18

### Objective
Optimize duplicate detection under bounded latency and bounded noise.

### Preconditions
- namespace is validated
- policy context is loaded
- latency budget is available
- tier capabilities are known

### Heuristic
1. start with the cheapest exact signals
2. cap expansion by query class
3. only pay semantic cost when exact and graph signals are insufficient
4. preserve explainability and provenance
5. update reinforcement or decay after outcome feedback

### Failure risks
- candidate explosion
- stale data bias
- low-confidence amplification
- cross-session interference

### Instrumentation
- p50/p95/p99 latency
- candidate counts before and after pruning
- success contribution estimate
- cache hit ratio
- repairability score

### Pseudocode
```text
fn pattern_168(ctx):
    budget = ctx.latency_budget
    exact = cheap_exact(ctx)
    graph = bounded_graph(ctx, exact)
    sem = maybe_semantic(ctx, budget, graph)
    merged = dedup_merge(exact, graph, sem)
    ranked = rank_with_policy(ctx, merged)
    return package(ctx, ranked[:ctx.limit])
```
## 169. Duplicate Detection pattern 19

### Objective
Optimize duplicate detection under bounded latency and bounded noise.

### Preconditions
- namespace is validated
- policy context is loaded
- latency budget is available
- tier capabilities are known

### Heuristic
1. start with the cheapest exact signals
2. cap expansion by query class
3. only pay semantic cost when exact and graph signals are insufficient
4. preserve explainability and provenance
5. update reinforcement or decay after outcome feedback

### Failure risks
- candidate explosion
- stale data bias
- low-confidence amplification
- cross-session interference

### Instrumentation
- p50/p95/p99 latency
- candidate counts before and after pruning
- success contribution estimate
- cache hit ratio
- repairability score

### Pseudocode
```text
fn pattern_169(ctx):
    budget = ctx.latency_budget
    exact = cheap_exact(ctx)
    graph = bounded_graph(ctx, exact)
    sem = maybe_semantic(ctx, budget, graph)
    merged = dedup_merge(exact, graph, sem)
    ranked = rank_with_policy(ctx, merged)
    return package(ctx, ranked[:ctx.limit])
```
## 170. Duplicate Detection pattern 20

### Objective
Optimize duplicate detection under bounded latency and bounded noise.

### Preconditions
- namespace is validated
- policy context is loaded
- latency budget is available
- tier capabilities are known

### Heuristic
1. start with the cheapest exact signals
2. cap expansion by query class
3. only pay semantic cost when exact and graph signals are insufficient
4. preserve explainability and provenance
5. update reinforcement or decay after outcome feedback

### Failure risks
- candidate explosion
- stale data bias
- low-confidence amplification
- cross-session interference

### Instrumentation
- p50/p95/p99 latency
- candidate counts before and after pruning
- success contribution estimate
- cache hit ratio
- repairability score

### Pseudocode
```text
fn pattern_170(ctx):
    budget = ctx.latency_budget
    exact = cheap_exact(ctx)
    graph = bounded_graph(ctx, exact)
    sem = maybe_semantic(ctx, budget, graph)
    merged = dedup_merge(exact, graph, sem)
    ranked = rank_with_policy(ctx, merged)
    return package(ctx, ranked[:ctx.limit])
```
## 171. Duplicate Detection pattern 21

### Objective
Optimize duplicate detection under bounded latency and bounded noise.

### Preconditions
- namespace is validated
- policy context is loaded
- latency budget is available
- tier capabilities are known

### Heuristic
1. start with the cheapest exact signals
2. cap expansion by query class
3. only pay semantic cost when exact and graph signals are insufficient
4. preserve explainability and provenance
5. update reinforcement or decay after outcome feedback

### Failure risks
- candidate explosion
- stale data bias
- low-confidence amplification
- cross-session interference

### Instrumentation
- p50/p95/p99 latency
- candidate counts before and after pruning
- success contribution estimate
- cache hit ratio
- repairability score

### Pseudocode
```text
fn pattern_171(ctx):
    budget = ctx.latency_budget
    exact = cheap_exact(ctx)
    graph = bounded_graph(ctx, exact)
    sem = maybe_semantic(ctx, budget, graph)
    merged = dedup_merge(exact, graph, sem)
    ranked = rank_with_policy(ctx, merged)
    return package(ctx, ranked[:ctx.limit])
```
## 172. Duplicate Detection pattern 22

### Objective
Optimize duplicate detection under bounded latency and bounded noise.

### Preconditions
- namespace is validated
- policy context is loaded
- latency budget is available
- tier capabilities are known

### Heuristic
1. start with the cheapest exact signals
2. cap expansion by query class
3. only pay semantic cost when exact and graph signals are insufficient
4. preserve explainability and provenance
5. update reinforcement or decay after outcome feedback

### Failure risks
- candidate explosion
- stale data bias
- low-confidence amplification
- cross-session interference

### Instrumentation
- p50/p95/p99 latency
- candidate counts before and after pruning
- success contribution estimate
- cache hit ratio
- repairability score

### Pseudocode
```text
fn pattern_172(ctx):
    budget = ctx.latency_budget
    exact = cheap_exact(ctx)
    graph = bounded_graph(ctx, exact)
    sem = maybe_semantic(ctx, budget, graph)
    merged = dedup_merge(exact, graph, sem)
    ranked = rank_with_policy(ctx, merged)
    return package(ctx, ranked[:ctx.limit])
```
## 173. Duplicate Detection pattern 23

### Objective
Optimize duplicate detection under bounded latency and bounded noise.

### Preconditions
- namespace is validated
- policy context is loaded
- latency budget is available
- tier capabilities are known

### Heuristic
1. start with the cheapest exact signals
2. cap expansion by query class
3. only pay semantic cost when exact and graph signals are insufficient
4. preserve explainability and provenance
5. update reinforcement or decay after outcome feedback

### Failure risks
- candidate explosion
- stale data bias
- low-confidence amplification
- cross-session interference

### Instrumentation
- p50/p95/p99 latency
- candidate counts before and after pruning
- success contribution estimate
- cache hit ratio
- repairability score

### Pseudocode
```text
fn pattern_173(ctx):
    budget = ctx.latency_budget
    exact = cheap_exact(ctx)
    graph = bounded_graph(ctx, exact)
    sem = maybe_semantic(ctx, budget, graph)
    merged = dedup_merge(exact, graph, sem)
    ranked = rank_with_policy(ctx, merged)
    return package(ctx, ranked[:ctx.limit])
```
## 174. Duplicate Detection pattern 24

### Objective
Optimize duplicate detection under bounded latency and bounded noise.

### Preconditions
- namespace is validated
- policy context is loaded
- latency budget is available
- tier capabilities are known

### Heuristic
1. start with the cheapest exact signals
2. cap expansion by query class
3. only pay semantic cost when exact and graph signals are insufficient
4. preserve explainability and provenance
5. update reinforcement or decay after outcome feedback

### Failure risks
- candidate explosion
- stale data bias
- low-confidence amplification
- cross-session interference

### Instrumentation
- p50/p95/p99 latency
- candidate counts before and after pruning
- success contribution estimate
- cache hit ratio
- repairability score

### Pseudocode
```text
fn pattern_174(ctx):
    budget = ctx.latency_budget
    exact = cheap_exact(ctx)
    graph = bounded_graph(ctx, exact)
    sem = maybe_semantic(ctx, budget, graph)
    merged = dedup_merge(exact, graph, sem)
    ranked = rank_with_policy(ctx, merged)
    return package(ctx, ranked[:ctx.limit])
```
## 175. Duplicate Detection pattern 25

### Objective
Optimize duplicate detection under bounded latency and bounded noise.

### Preconditions
- namespace is validated
- policy context is loaded
- latency budget is available
- tier capabilities are known

### Heuristic
1. start with the cheapest exact signals
2. cap expansion by query class
3. only pay semantic cost when exact and graph signals are insufficient
4. preserve explainability and provenance
5. update reinforcement or decay after outcome feedback

### Failure risks
- candidate explosion
- stale data bias
- low-confidence amplification
- cross-session interference

### Instrumentation
- p50/p95/p99 latency
- candidate counts before and after pruning
- success contribution estimate
- cache hit ratio
- repairability score

### Pseudocode
```text
fn pattern_175(ctx):
    budget = ctx.latency_budget
    exact = cheap_exact(ctx)
    graph = bounded_graph(ctx, exact)
    sem = maybe_semantic(ctx, budget, graph)
    merged = dedup_merge(exact, graph, sem)
    ranked = rank_with_policy(ctx, merged)
    return package(ctx, ranked[:ctx.limit])
```
## 176. Duplicate Detection pattern 26

### Objective
Optimize duplicate detection under bounded latency and bounded noise.

### Preconditions
- namespace is validated
- policy context is loaded
- latency budget is available
- tier capabilities are known

### Heuristic
1. start with the cheapest exact signals
2. cap expansion by query class
3. only pay semantic cost when exact and graph signals are insufficient
4. preserve explainability and provenance
5. update reinforcement or decay after outcome feedback

### Failure risks
- candidate explosion
- stale data bias
- low-confidence amplification
- cross-session interference

### Instrumentation
- p50/p95/p99 latency
- candidate counts before and after pruning
- success contribution estimate
- cache hit ratio
- repairability score

### Pseudocode
```text
fn pattern_176(ctx):
    budget = ctx.latency_budget
    exact = cheap_exact(ctx)
    graph = bounded_graph(ctx, exact)
    sem = maybe_semantic(ctx, budget, graph)
    merged = dedup_merge(exact, graph, sem)
    ranked = rank_with_policy(ctx, merged)
    return package(ctx, ranked[:ctx.limit])
```
## 177. Duplicate Detection pattern 27

### Objective
Optimize duplicate detection under bounded latency and bounded noise.

### Preconditions
- namespace is validated
- policy context is loaded
- latency budget is available
- tier capabilities are known

### Heuristic
1. start with the cheapest exact signals
2. cap expansion by query class
3. only pay semantic cost when exact and graph signals are insufficient
4. preserve explainability and provenance
5. update reinforcement or decay after outcome feedback

### Failure risks
- candidate explosion
- stale data bias
- low-confidence amplification
- cross-session interference

### Instrumentation
- p50/p95/p99 latency
- candidate counts before and after pruning
- success contribution estimate
- cache hit ratio
- repairability score

### Pseudocode
```text
fn pattern_177(ctx):
    budget = ctx.latency_budget
    exact = cheap_exact(ctx)
    graph = bounded_graph(ctx, exact)
    sem = maybe_semantic(ctx, budget, graph)
    merged = dedup_merge(exact, graph, sem)
    ranked = rank_with_policy(ctx, merged)
    return package(ctx, ranked[:ctx.limit])
```
## 178. Duplicate Detection pattern 28

### Objective
Optimize duplicate detection under bounded latency and bounded noise.

### Preconditions
- namespace is validated
- policy context is loaded
- latency budget is available
- tier capabilities are known

### Heuristic
1. start with the cheapest exact signals
2. cap expansion by query class
3. only pay semantic cost when exact and graph signals are insufficient
4. preserve explainability and provenance
5. update reinforcement or decay after outcome feedback

### Failure risks
- candidate explosion
- stale data bias
- low-confidence amplification
- cross-session interference

### Instrumentation
- p50/p95/p99 latency
- candidate counts before and after pruning
- success contribution estimate
- cache hit ratio
- repairability score

### Pseudocode
```text
fn pattern_178(ctx):
    budget = ctx.latency_budget
    exact = cheap_exact(ctx)
    graph = bounded_graph(ctx, exact)
    sem = maybe_semantic(ctx, budget, graph)
    merged = dedup_merge(exact, graph, sem)
    ranked = rank_with_policy(ctx, merged)
    return package(ctx, ranked[:ctx.limit])
```
## 179. Duplicate Detection pattern 29

### Objective
Optimize duplicate detection under bounded latency and bounded noise.

### Preconditions
- namespace is validated
- policy context is loaded
- latency budget is available
- tier capabilities are known

### Heuristic
1. start with the cheapest exact signals
2. cap expansion by query class
3. only pay semantic cost when exact and graph signals are insufficient
4. preserve explainability and provenance
5. update reinforcement or decay after outcome feedback

### Failure risks
- candidate explosion
- stale data bias
- low-confidence amplification
- cross-session interference

### Instrumentation
- p50/p95/p99 latency
- candidate counts before and after pruning
- success contribution estimate
- cache hit ratio
- repairability score

### Pseudocode
```text
fn pattern_179(ctx):
    budget = ctx.latency_budget
    exact = cheap_exact(ctx)
    graph = bounded_graph(ctx, exact)
    sem = maybe_semantic(ctx, budget, graph)
    merged = dedup_merge(exact, graph, sem)
    ranked = rank_with_policy(ctx, merged)
    return package(ctx, ranked[:ctx.limit])
```
## 180. Duplicate Detection pattern 30

### Objective
Optimize duplicate detection under bounded latency and bounded noise.

### Preconditions
- namespace is validated
- policy context is loaded
- latency budget is available
- tier capabilities are known

### Heuristic
1. start with the cheapest exact signals
2. cap expansion by query class
3. only pay semantic cost when exact and graph signals are insufficient
4. preserve explainability and provenance
5. update reinforcement or decay after outcome feedback

### Failure risks
- candidate explosion
- stale data bias
- low-confidence amplification
- cross-session interference

### Instrumentation
- p50/p95/p99 latency
- candidate counts before and after pruning
- success contribution estimate
- cache hit ratio
- repairability score

### Pseudocode
```text
fn pattern_180(ctx):
    budget = ctx.latency_budget
    exact = cheap_exact(ctx)
    graph = bounded_graph(ctx, exact)
    sem = maybe_semantic(ctx, budget, graph)
    merged = dedup_merge(exact, graph, sem)
    ranked = rank_with_policy(ctx, merged)
    return package(ctx, ranked[:ctx.limit])
```
## 181. Graph Expansion pattern 1

### Objective
Optimize graph expansion under bounded latency and bounded noise.

### Preconditions
- namespace is validated
- policy context is loaded
- latency budget is available
- tier capabilities are known

### Heuristic
1. start with the cheapest exact signals
2. cap expansion by query class
3. only pay semantic cost when exact and graph signals are insufficient
4. preserve explainability and provenance
5. update reinforcement or decay after outcome feedback

### Failure risks
- candidate explosion
- stale data bias
- low-confidence amplification
- cross-session interference

### Instrumentation
- p50/p95/p99 latency
- candidate counts before and after pruning
- success contribution estimate
- cache hit ratio
- repairability score

### Pseudocode
```text
fn pattern_181(ctx):
    budget = ctx.latency_budget
    exact = cheap_exact(ctx)
    graph = bounded_graph(ctx, exact)
    sem = maybe_semantic(ctx, budget, graph)
    merged = dedup_merge(exact, graph, sem)
    ranked = rank_with_policy(ctx, merged)
    return package(ctx, ranked[:ctx.limit])
```
## 182. Graph Expansion pattern 2

### Objective
Optimize graph expansion under bounded latency and bounded noise.

### Preconditions
- namespace is validated
- policy context is loaded
- latency budget is available
- tier capabilities are known

### Heuristic
1. start with the cheapest exact signals
2. cap expansion by query class
3. only pay semantic cost when exact and graph signals are insufficient
4. preserve explainability and provenance
5. update reinforcement or decay after outcome feedback

### Failure risks
- candidate explosion
- stale data bias
- low-confidence amplification
- cross-session interference

### Instrumentation
- p50/p95/p99 latency
- candidate counts before and after pruning
- success contribution estimate
- cache hit ratio
- repairability score

### Pseudocode
```text
fn pattern_182(ctx):
    budget = ctx.latency_budget
    exact = cheap_exact(ctx)
    graph = bounded_graph(ctx, exact)
    sem = maybe_semantic(ctx, budget, graph)
    merged = dedup_merge(exact, graph, sem)
    ranked = rank_with_policy(ctx, merged)
    return package(ctx, ranked[:ctx.limit])
```
## 183. Graph Expansion pattern 3

### Objective
Optimize graph expansion under bounded latency and bounded noise.

### Preconditions
- namespace is validated
- policy context is loaded
- latency budget is available
- tier capabilities are known

### Heuristic
1. start with the cheapest exact signals
2. cap expansion by query class
3. only pay semantic cost when exact and graph signals are insufficient
4. preserve explainability and provenance
5. update reinforcement or decay after outcome feedback

### Failure risks
- candidate explosion
- stale data bias
- low-confidence amplification
- cross-session interference

### Instrumentation
- p50/p95/p99 latency
- candidate counts before and after pruning
- success contribution estimate
- cache hit ratio
- repairability score

### Pseudocode
```text
fn pattern_183(ctx):
    budget = ctx.latency_budget
    exact = cheap_exact(ctx)
    graph = bounded_graph(ctx, exact)
    sem = maybe_semantic(ctx, budget, graph)
    merged = dedup_merge(exact, graph, sem)
    ranked = rank_with_policy(ctx, merged)
    return package(ctx, ranked[:ctx.limit])
```
## 184. Graph Expansion pattern 4

### Objective
Optimize graph expansion under bounded latency and bounded noise.

### Preconditions
- namespace is validated
- policy context is loaded
- latency budget is available
- tier capabilities are known

### Heuristic
1. start with the cheapest exact signals
2. cap expansion by query class
3. only pay semantic cost when exact and graph signals are insufficient
4. preserve explainability and provenance
5. update reinforcement or decay after outcome feedback

### Failure risks
- candidate explosion
- stale data bias
- low-confidence amplification
- cross-session interference

### Instrumentation
- p50/p95/p99 latency
- candidate counts before and after pruning
- success contribution estimate
- cache hit ratio
- repairability score

### Pseudocode
```text
fn pattern_184(ctx):
    budget = ctx.latency_budget
    exact = cheap_exact(ctx)
    graph = bounded_graph(ctx, exact)
    sem = maybe_semantic(ctx, budget, graph)
    merged = dedup_merge(exact, graph, sem)
    ranked = rank_with_policy(ctx, merged)
    return package(ctx, ranked[:ctx.limit])
```
## 185. Graph Expansion pattern 5

### Objective
Optimize graph expansion under bounded latency and bounded noise.

### Preconditions
- namespace is validated
- policy context is loaded
- latency budget is available
- tier capabilities are known

### Heuristic
1. start with the cheapest exact signals
2. cap expansion by query class
3. only pay semantic cost when exact and graph signals are insufficient
4. preserve explainability and provenance
5. update reinforcement or decay after outcome feedback

### Failure risks
- candidate explosion
- stale data bias
- low-confidence amplification
- cross-session interference

### Instrumentation
- p50/p95/p99 latency
- candidate counts before and after pruning
- success contribution estimate
- cache hit ratio
- repairability score

### Pseudocode
```text
fn pattern_185(ctx):
    budget = ctx.latency_budget
    exact = cheap_exact(ctx)
    graph = bounded_graph(ctx, exact)
    sem = maybe_semantic(ctx, budget, graph)
    merged = dedup_merge(exact, graph, sem)
    ranked = rank_with_policy(ctx, merged)
    return package(ctx, ranked[:ctx.limit])
```
## 186. Graph Expansion pattern 6

### Objective
Optimize graph expansion under bounded latency and bounded noise.

### Preconditions
- namespace is validated
- policy context is loaded
- latency budget is available
- tier capabilities are known

### Heuristic
1. start with the cheapest exact signals
2. cap expansion by query class
3. only pay semantic cost when exact and graph signals are insufficient
4. preserve explainability and provenance
5. update reinforcement or decay after outcome feedback

### Failure risks
- candidate explosion
- stale data bias
- low-confidence amplification
- cross-session interference

### Instrumentation
- p50/p95/p99 latency
- candidate counts before and after pruning
- success contribution estimate
- cache hit ratio
- repairability score

### Pseudocode
```text
fn pattern_186(ctx):
    budget = ctx.latency_budget
    exact = cheap_exact(ctx)
    graph = bounded_graph(ctx, exact)
    sem = maybe_semantic(ctx, budget, graph)
    merged = dedup_merge(exact, graph, sem)
    ranked = rank_with_policy(ctx, merged)
    return package(ctx, ranked[:ctx.limit])
```
## 187. Graph Expansion pattern 7

### Objective
Optimize graph expansion under bounded latency and bounded noise.

### Preconditions
- namespace is validated
- policy context is loaded
- latency budget is available
- tier capabilities are known

### Heuristic
1. start with the cheapest exact signals
2. cap expansion by query class
3. only pay semantic cost when exact and graph signals are insufficient
4. preserve explainability and provenance
5. update reinforcement or decay after outcome feedback

### Failure risks
- candidate explosion
- stale data bias
- low-confidence amplification
- cross-session interference

### Instrumentation
- p50/p95/p99 latency
- candidate counts before and after pruning
- success contribution estimate
- cache hit ratio
- repairability score

### Pseudocode
```text
fn pattern_187(ctx):
    budget = ctx.latency_budget
    exact = cheap_exact(ctx)
    graph = bounded_graph(ctx, exact)
    sem = maybe_semantic(ctx, budget, graph)
    merged = dedup_merge(exact, graph, sem)
    ranked = rank_with_policy(ctx, merged)
    return package(ctx, ranked[:ctx.limit])
```
## 188. Graph Expansion pattern 8

### Objective
Optimize graph expansion under bounded latency and bounded noise.

### Preconditions
- namespace is validated
- policy context is loaded
- latency budget is available
- tier capabilities are known

### Heuristic
1. start with the cheapest exact signals
2. cap expansion by query class
3. only pay semantic cost when exact and graph signals are insufficient
4. preserve explainability and provenance
5. update reinforcement or decay after outcome feedback

### Failure risks
- candidate explosion
- stale data bias
- low-confidence amplification
- cross-session interference

### Instrumentation
- p50/p95/p99 latency
- candidate counts before and after pruning
- success contribution estimate
- cache hit ratio
- repairability score

### Pseudocode
```text
fn pattern_188(ctx):
    budget = ctx.latency_budget
    exact = cheap_exact(ctx)
    graph = bounded_graph(ctx, exact)
    sem = maybe_semantic(ctx, budget, graph)
    merged = dedup_merge(exact, graph, sem)
    ranked = rank_with_policy(ctx, merged)
    return package(ctx, ranked[:ctx.limit])
```
## 189. Graph Expansion pattern 9

### Objective
Optimize graph expansion under bounded latency and bounded noise.

### Preconditions
- namespace is validated
- policy context is loaded
- latency budget is available
- tier capabilities are known

### Heuristic
1. start with the cheapest exact signals
2. cap expansion by query class
3. only pay semantic cost when exact and graph signals are insufficient
4. preserve explainability and provenance
5. update reinforcement or decay after outcome feedback

### Failure risks
- candidate explosion
- stale data bias
- low-confidence amplification
- cross-session interference

### Instrumentation
- p50/p95/p99 latency
- candidate counts before and after pruning
- success contribution estimate
- cache hit ratio
- repairability score

### Pseudocode
```text
fn pattern_189(ctx):
    budget = ctx.latency_budget
    exact = cheap_exact(ctx)
    graph = bounded_graph(ctx, exact)
    sem = maybe_semantic(ctx, budget, graph)
    merged = dedup_merge(exact, graph, sem)
    ranked = rank_with_policy(ctx, merged)
    return package(ctx, ranked[:ctx.limit])
```
## 190. Graph Expansion pattern 10

### Objective
Optimize graph expansion under bounded latency and bounded noise.

### Preconditions
- namespace is validated
- policy context is loaded
- latency budget is available
- tier capabilities are known

### Heuristic
1. start with the cheapest exact signals
2. cap expansion by query class
3. only pay semantic cost when exact and graph signals are insufficient
4. preserve explainability and provenance
5. update reinforcement or decay after outcome feedback

### Failure risks
- candidate explosion
- stale data bias
- low-confidence amplification
- cross-session interference

### Instrumentation
- p50/p95/p99 latency
- candidate counts before and after pruning
- success contribution estimate
- cache hit ratio
- repairability score

### Pseudocode
```text
fn pattern_190(ctx):
    budget = ctx.latency_budget
    exact = cheap_exact(ctx)
    graph = bounded_graph(ctx, exact)
    sem = maybe_semantic(ctx, budget, graph)
    merged = dedup_merge(exact, graph, sem)
    ranked = rank_with_policy(ctx, merged)
    return package(ctx, ranked[:ctx.limit])
```
## 191. Graph Expansion pattern 11

### Objective
Optimize graph expansion under bounded latency and bounded noise.

### Preconditions
- namespace is validated
- policy context is loaded
- latency budget is available
- tier capabilities are known

### Heuristic
1. start with the cheapest exact signals
2. cap expansion by query class
3. only pay semantic cost when exact and graph signals are insufficient
4. preserve explainability and provenance
5. update reinforcement or decay after outcome feedback

### Failure risks
- candidate explosion
- stale data bias
- low-confidence amplification
- cross-session interference

### Instrumentation
- p50/p95/p99 latency
- candidate counts before and after pruning
- success contribution estimate
- cache hit ratio
- repairability score

### Pseudocode
```text
fn pattern_191(ctx):
    budget = ctx.latency_budget
    exact = cheap_exact(ctx)
    graph = bounded_graph(ctx, exact)
    sem = maybe_semantic(ctx, budget, graph)
    merged = dedup_merge(exact, graph, sem)
    ranked = rank_with_policy(ctx, merged)
    return package(ctx, ranked[:ctx.limit])
```
## 192. Graph Expansion pattern 12

### Objective
Optimize graph expansion under bounded latency and bounded noise.

### Preconditions
- namespace is validated
- policy context is loaded
- latency budget is available
- tier capabilities are known

### Heuristic
1. start with the cheapest exact signals
2. cap expansion by query class
3. only pay semantic cost when exact and graph signals are insufficient
4. preserve explainability and provenance
5. update reinforcement or decay after outcome feedback

### Failure risks
- candidate explosion
- stale data bias
- low-confidence amplification
- cross-session interference

### Instrumentation
- p50/p95/p99 latency
- candidate counts before and after pruning
- success contribution estimate
- cache hit ratio
- repairability score

### Pseudocode
```text
fn pattern_192(ctx):
    budget = ctx.latency_budget
    exact = cheap_exact(ctx)
    graph = bounded_graph(ctx, exact)
    sem = maybe_semantic(ctx, budget, graph)
    merged = dedup_merge(exact, graph, sem)
    ranked = rank_with_policy(ctx, merged)
    return package(ctx, ranked[:ctx.limit])
```
## 193. Graph Expansion pattern 13

### Objective
Optimize graph expansion under bounded latency and bounded noise.

### Preconditions
- namespace is validated
- policy context is loaded
- latency budget is available
- tier capabilities are known

### Heuristic
1. start with the cheapest exact signals
2. cap expansion by query class
3. only pay semantic cost when exact and graph signals are insufficient
4. preserve explainability and provenance
5. update reinforcement or decay after outcome feedback

### Failure risks
- candidate explosion
- stale data bias
- low-confidence amplification
- cross-session interference

### Instrumentation
- p50/p95/p99 latency
- candidate counts before and after pruning
- success contribution estimate
- cache hit ratio
- repairability score

### Pseudocode
```text
fn pattern_193(ctx):
    budget = ctx.latency_budget
    exact = cheap_exact(ctx)
    graph = bounded_graph(ctx, exact)
    sem = maybe_semantic(ctx, budget, graph)
    merged = dedup_merge(exact, graph, sem)
    ranked = rank_with_policy(ctx, merged)
    return package(ctx, ranked[:ctx.limit])
```
## 194. Graph Expansion pattern 14

### Objective
Optimize graph expansion under bounded latency and bounded noise.

### Preconditions
- namespace is validated
- policy context is loaded
- latency budget is available
- tier capabilities are known

### Heuristic
1. start with the cheapest exact signals
2. cap expansion by query class
3. only pay semantic cost when exact and graph signals are insufficient
4. preserve explainability and provenance
5. update reinforcement or decay after outcome feedback

### Failure risks
- candidate explosion
- stale data bias
- low-confidence amplification
- cross-session interference

### Instrumentation
- p50/p95/p99 latency
- candidate counts before and after pruning
- success contribution estimate
- cache hit ratio
- repairability score

### Pseudocode
```text
fn pattern_194(ctx):
    budget = ctx.latency_budget
    exact = cheap_exact(ctx)
    graph = bounded_graph(ctx, exact)
    sem = maybe_semantic(ctx, budget, graph)
    merged = dedup_merge(exact, graph, sem)
    ranked = rank_with_policy(ctx, merged)
    return package(ctx, ranked[:ctx.limit])
```
## 195. Graph Expansion pattern 15

### Objective
Optimize graph expansion under bounded latency and bounded noise.

### Preconditions
- namespace is validated
- policy context is loaded
- latency budget is available
- tier capabilities are known

### Heuristic
1. start with the cheapest exact signals
2. cap expansion by query class
3. only pay semantic cost when exact and graph signals are insufficient
4. preserve explainability and provenance
5. update reinforcement or decay after outcome feedback

### Failure risks
- candidate explosion
- stale data bias
- low-confidence amplification
- cross-session interference

### Instrumentation
- p50/p95/p99 latency
- candidate counts before and after pruning
- success contribution estimate
- cache hit ratio
- repairability score

### Pseudocode
```text
fn pattern_195(ctx):
    budget = ctx.latency_budget
    exact = cheap_exact(ctx)
    graph = bounded_graph(ctx, exact)
    sem = maybe_semantic(ctx, budget, graph)
    merged = dedup_merge(exact, graph, sem)
    ranked = rank_with_policy(ctx, merged)
    return package(ctx, ranked[:ctx.limit])
```
## 196. Graph Expansion pattern 16

### Objective
Optimize graph expansion under bounded latency and bounded noise.

### Preconditions
- namespace is validated
- policy context is loaded
- latency budget is available
- tier capabilities are known

### Heuristic
1. start with the cheapest exact signals
2. cap expansion by query class
3. only pay semantic cost when exact and graph signals are insufficient
4. preserve explainability and provenance
5. update reinforcement or decay after outcome feedback

### Failure risks
- candidate explosion
- stale data bias
- low-confidence amplification
- cross-session interference

### Instrumentation
- p50/p95/p99 latency
- candidate counts before and after pruning
- success contribution estimate
- cache hit ratio
- repairability score

### Pseudocode
```text
fn pattern_196(ctx):
    budget = ctx.latency_budget
    exact = cheap_exact(ctx)
    graph = bounded_graph(ctx, exact)
    sem = maybe_semantic(ctx, budget, graph)
    merged = dedup_merge(exact, graph, sem)
    ranked = rank_with_policy(ctx, merged)
    return package(ctx, ranked[:ctx.limit])
```
## 197. Graph Expansion pattern 17

### Objective
Optimize graph expansion under bounded latency and bounded noise.

### Preconditions
- namespace is validated
- policy context is loaded
- latency budget is available
- tier capabilities are known

### Heuristic
1. start with the cheapest exact signals
2. cap expansion by query class
3. only pay semantic cost when exact and graph signals are insufficient
4. preserve explainability and provenance
5. update reinforcement or decay after outcome feedback

### Failure risks
- candidate explosion
- stale data bias
- low-confidence amplification
- cross-session interference

### Instrumentation
- p50/p95/p99 latency
- candidate counts before and after pruning
- success contribution estimate
- cache hit ratio
- repairability score

### Pseudocode
```text
fn pattern_197(ctx):
    budget = ctx.latency_budget
    exact = cheap_exact(ctx)
    graph = bounded_graph(ctx, exact)
    sem = maybe_semantic(ctx, budget, graph)
    merged = dedup_merge(exact, graph, sem)
    ranked = rank_with_policy(ctx, merged)
    return package(ctx, ranked[:ctx.limit])
```
## 198. Graph Expansion pattern 18

### Objective
Optimize graph expansion under bounded latency and bounded noise.

### Preconditions
- namespace is validated
- policy context is loaded
- latency budget is available
- tier capabilities are known

### Heuristic
1. start with the cheapest exact signals
2. cap expansion by query class
3. only pay semantic cost when exact and graph signals are insufficient
4. preserve explainability and provenance
5. update reinforcement or decay after outcome feedback

### Failure risks
- candidate explosion
- stale data bias
- low-confidence amplification
- cross-session interference

### Instrumentation
- p50/p95/p99 latency
- candidate counts before and after pruning
- success contribution estimate
- cache hit ratio
- repairability score

### Pseudocode
```text
fn pattern_198(ctx):
    budget = ctx.latency_budget
    exact = cheap_exact(ctx)
    graph = bounded_graph(ctx, exact)
    sem = maybe_semantic(ctx, budget, graph)
    merged = dedup_merge(exact, graph, sem)
    ranked = rank_with_policy(ctx, merged)
    return package(ctx, ranked[:ctx.limit])
```
## 199. Graph Expansion pattern 19

### Objective
Optimize graph expansion under bounded latency and bounded noise.

### Preconditions
- namespace is validated
- policy context is loaded
- latency budget is available
- tier capabilities are known

### Heuristic
1. start with the cheapest exact signals
2. cap expansion by query class
3. only pay semantic cost when exact and graph signals are insufficient
4. preserve explainability and provenance
5. update reinforcement or decay after outcome feedback

### Failure risks
- candidate explosion
- stale data bias
- low-confidence amplification
- cross-session interference

### Instrumentation
- p50/p95/p99 latency
- candidate counts before and after pruning
- success contribution estimate
- cache hit ratio
- repairability score

### Pseudocode
```text
fn pattern_199(ctx):
    budget = ctx.latency_budget
    exact = cheap_exact(ctx)
    graph = bounded_graph(ctx, exact)
    sem = maybe_semantic(ctx, budget, graph)
    merged = dedup_merge(exact, graph, sem)
    ranked = rank_with_policy(ctx, merged)
    return package(ctx, ranked[:ctx.limit])
```
## 200. Graph Expansion pattern 20

### Objective
Optimize graph expansion under bounded latency and bounded noise.

### Preconditions
- namespace is validated
- policy context is loaded
- latency budget is available
- tier capabilities are known

### Heuristic
1. start with the cheapest exact signals
2. cap expansion by query class
3. only pay semantic cost when exact and graph signals are insufficient
4. preserve explainability and provenance
5. update reinforcement or decay after outcome feedback

### Failure risks
- candidate explosion
- stale data bias
- low-confidence amplification
- cross-session interference

### Instrumentation
- p50/p95/p99 latency
- candidate counts before and after pruning
- success contribution estimate
- cache hit ratio
- repairability score

### Pseudocode
```text
fn pattern_200(ctx):
    budget = ctx.latency_budget
    exact = cheap_exact(ctx)
    graph = bounded_graph(ctx, exact)
    sem = maybe_semantic(ctx, budget, graph)
    merged = dedup_merge(exact, graph, sem)
    ranked = rank_with_policy(ctx, merged)
    return package(ctx, ranked[:ctx.limit])
```
## 201. Graph Expansion pattern 21

### Objective
Optimize graph expansion under bounded latency and bounded noise.

### Preconditions
- namespace is validated
- policy context is loaded
- latency budget is available
- tier capabilities are known

### Heuristic
1. start with the cheapest exact signals
2. cap expansion by query class
3. only pay semantic cost when exact and graph signals are insufficient
4. preserve explainability and provenance
5. update reinforcement or decay after outcome feedback

### Failure risks
- candidate explosion
- stale data bias
- low-confidence amplification
- cross-session interference

### Instrumentation
- p50/p95/p99 latency
- candidate counts before and after pruning
- success contribution estimate
- cache hit ratio
- repairability score

### Pseudocode
```text
fn pattern_201(ctx):
    budget = ctx.latency_budget
    exact = cheap_exact(ctx)
    graph = bounded_graph(ctx, exact)
    sem = maybe_semantic(ctx, budget, graph)
    merged = dedup_merge(exact, graph, sem)
    ranked = rank_with_policy(ctx, merged)
    return package(ctx, ranked[:ctx.limit])
```
## 202. Graph Expansion pattern 22

### Objective
Optimize graph expansion under bounded latency and bounded noise.

### Preconditions
- namespace is validated
- policy context is loaded
- latency budget is available
- tier capabilities are known

### Heuristic
1. start with the cheapest exact signals
2. cap expansion by query class
3. only pay semantic cost when exact and graph signals are insufficient
4. preserve explainability and provenance
5. update reinforcement or decay after outcome feedback

### Failure risks
- candidate explosion
- stale data bias
- low-confidence amplification
- cross-session interference

### Instrumentation
- p50/p95/p99 latency
- candidate counts before and after pruning
- success contribution estimate
- cache hit ratio
- repairability score

### Pseudocode
```text
fn pattern_202(ctx):
    budget = ctx.latency_budget
    exact = cheap_exact(ctx)
    graph = bounded_graph(ctx, exact)
    sem = maybe_semantic(ctx, budget, graph)
    merged = dedup_merge(exact, graph, sem)
    ranked = rank_with_policy(ctx, merged)
    return package(ctx, ranked[:ctx.limit])
```
## 203. Graph Expansion pattern 23

### Objective
Optimize graph expansion under bounded latency and bounded noise.

### Preconditions
- namespace is validated
- policy context is loaded
- latency budget is available
- tier capabilities are known

### Heuristic
1. start with the cheapest exact signals
2. cap expansion by query class
3. only pay semantic cost when exact and graph signals are insufficient
4. preserve explainability and provenance
5. update reinforcement or decay after outcome feedback

### Failure risks
- candidate explosion
- stale data bias
- low-confidence amplification
- cross-session interference

### Instrumentation
- p50/p95/p99 latency
- candidate counts before and after pruning
- success contribution estimate
- cache hit ratio
- repairability score

### Pseudocode
```text
fn pattern_203(ctx):
    budget = ctx.latency_budget
    exact = cheap_exact(ctx)
    graph = bounded_graph(ctx, exact)
    sem = maybe_semantic(ctx, budget, graph)
    merged = dedup_merge(exact, graph, sem)
    ranked = rank_with_policy(ctx, merged)
    return package(ctx, ranked[:ctx.limit])
```
## 204. Graph Expansion pattern 24

### Objective
Optimize graph expansion under bounded latency and bounded noise.

### Preconditions
- namespace is validated
- policy context is loaded
- latency budget is available
- tier capabilities are known

### Heuristic
1. start with the cheapest exact signals
2. cap expansion by query class
3. only pay semantic cost when exact and graph signals are insufficient
4. preserve explainability and provenance
5. update reinforcement or decay after outcome feedback

### Failure risks
- candidate explosion
- stale data bias
- low-confidence amplification
- cross-session interference

### Instrumentation
- p50/p95/p99 latency
- candidate counts before and after pruning
- success contribution estimate
- cache hit ratio
- repairability score

### Pseudocode
```text
fn pattern_204(ctx):
    budget = ctx.latency_budget
    exact = cheap_exact(ctx)
    graph = bounded_graph(ctx, exact)
    sem = maybe_semantic(ctx, budget, graph)
    merged = dedup_merge(exact, graph, sem)
    ranked = rank_with_policy(ctx, merged)
    return package(ctx, ranked[:ctx.limit])
```
## 205. Graph Expansion pattern 25

### Objective
Optimize graph expansion under bounded latency and bounded noise.

### Preconditions
- namespace is validated
- policy context is loaded
- latency budget is available
- tier capabilities are known

### Heuristic
1. start with the cheapest exact signals
2. cap expansion by query class
3. only pay semantic cost when exact and graph signals are insufficient
4. preserve explainability and provenance
5. update reinforcement or decay after outcome feedback

### Failure risks
- candidate explosion
- stale data bias
- low-confidence amplification
- cross-session interference

### Instrumentation
- p50/p95/p99 latency
- candidate counts before and after pruning
- success contribution estimate
- cache hit ratio
- repairability score

### Pseudocode
```text
fn pattern_205(ctx):
    budget = ctx.latency_budget
    exact = cheap_exact(ctx)
    graph = bounded_graph(ctx, exact)
    sem = maybe_semantic(ctx, budget, graph)
    merged = dedup_merge(exact, graph, sem)
    ranked = rank_with_policy(ctx, merged)
    return package(ctx, ranked[:ctx.limit])
```
## 206. Graph Expansion pattern 26

### Objective
Optimize graph expansion under bounded latency and bounded noise.

### Preconditions
- namespace is validated
- policy context is loaded
- latency budget is available
- tier capabilities are known

### Heuristic
1. start with the cheapest exact signals
2. cap expansion by query class
3. only pay semantic cost when exact and graph signals are insufficient
4. preserve explainability and provenance
5. update reinforcement or decay after outcome feedback

### Failure risks
- candidate explosion
- stale data bias
- low-confidence amplification
- cross-session interference

### Instrumentation
- p50/p95/p99 latency
- candidate counts before and after pruning
- success contribution estimate
- cache hit ratio
- repairability score

### Pseudocode
```text
fn pattern_206(ctx):
    budget = ctx.latency_budget
    exact = cheap_exact(ctx)
    graph = bounded_graph(ctx, exact)
    sem = maybe_semantic(ctx, budget, graph)
    merged = dedup_merge(exact, graph, sem)
    ranked = rank_with_policy(ctx, merged)
    return package(ctx, ranked[:ctx.limit])
```
## 207. Graph Expansion pattern 27

### Objective
Optimize graph expansion under bounded latency and bounded noise.

### Preconditions
- namespace is validated
- policy context is loaded
- latency budget is available
- tier capabilities are known

### Heuristic
1. start with the cheapest exact signals
2. cap expansion by query class
3. only pay semantic cost when exact and graph signals are insufficient
4. preserve explainability and provenance
5. update reinforcement or decay after outcome feedback

### Failure risks
- candidate explosion
- stale data bias
- low-confidence amplification
- cross-session interference

### Instrumentation
- p50/p95/p99 latency
- candidate counts before and after pruning
- success contribution estimate
- cache hit ratio
- repairability score

### Pseudocode
```text
fn pattern_207(ctx):
    budget = ctx.latency_budget
    exact = cheap_exact(ctx)
    graph = bounded_graph(ctx, exact)
    sem = maybe_semantic(ctx, budget, graph)
    merged = dedup_merge(exact, graph, sem)
    ranked = rank_with_policy(ctx, merged)
    return package(ctx, ranked[:ctx.limit])
```
## 208. Graph Expansion pattern 28

### Objective
Optimize graph expansion under bounded latency and bounded noise.

### Preconditions
- namespace is validated
- policy context is loaded
- latency budget is available
- tier capabilities are known

### Heuristic
1. start with the cheapest exact signals
2. cap expansion by query class
3. only pay semantic cost when exact and graph signals are insufficient
4. preserve explainability and provenance
5. update reinforcement or decay after outcome feedback

### Failure risks
- candidate explosion
- stale data bias
- low-confidence amplification
- cross-session interference

### Instrumentation
- p50/p95/p99 latency
- candidate counts before and after pruning
- success contribution estimate
- cache hit ratio
- repairability score

### Pseudocode
```text
fn pattern_208(ctx):
    budget = ctx.latency_budget
    exact = cheap_exact(ctx)
    graph = bounded_graph(ctx, exact)
    sem = maybe_semantic(ctx, budget, graph)
    merged = dedup_merge(exact, graph, sem)
    ranked = rank_with_policy(ctx, merged)
    return package(ctx, ranked[:ctx.limit])
```
## 209. Graph Expansion pattern 29

### Objective
Optimize graph expansion under bounded latency and bounded noise.

### Preconditions
- namespace is validated
- policy context is loaded
- latency budget is available
- tier capabilities are known

### Heuristic
1. start with the cheapest exact signals
2. cap expansion by query class
3. only pay semantic cost when exact and graph signals are insufficient
4. preserve explainability and provenance
5. update reinforcement or decay after outcome feedback

### Failure risks
- candidate explosion
- stale data bias
- low-confidence amplification
- cross-session interference

### Instrumentation
- p50/p95/p99 latency
- candidate counts before and after pruning
- success contribution estimate
- cache hit ratio
- repairability score

### Pseudocode
```text
fn pattern_209(ctx):
    budget = ctx.latency_budget
    exact = cheap_exact(ctx)
    graph = bounded_graph(ctx, exact)
    sem = maybe_semantic(ctx, budget, graph)
    merged = dedup_merge(exact, graph, sem)
    ranked = rank_with_policy(ctx, merged)
    return package(ctx, ranked[:ctx.limit])
```
## 210. Graph Expansion pattern 30

### Objective
Optimize graph expansion under bounded latency and bounded noise.

### Preconditions
- namespace is validated
- policy context is loaded
- latency budget is available
- tier capabilities are known

### Heuristic
1. start with the cheapest exact signals
2. cap expansion by query class
3. only pay semantic cost when exact and graph signals are insufficient
4. preserve explainability and provenance
5. update reinforcement or decay after outcome feedback

### Failure risks
- candidate explosion
- stale data bias
- low-confidence amplification
- cross-session interference

### Instrumentation
- p50/p95/p99 latency
- candidate counts before and after pruning
- success contribution estimate
- cache hit ratio
- repairability score

### Pseudocode
```text
fn pattern_210(ctx):
    budget = ctx.latency_budget
    exact = cheap_exact(ctx)
    graph = bounded_graph(ctx, exact)
    sem = maybe_semantic(ctx, budget, graph)
    merged = dedup_merge(exact, graph, sem)
    ranked = rank_with_policy(ctx, merged)
    return package(ctx, ranked[:ctx.limit])
```
## 211. Candidate Diversification pattern 1

### Objective
Optimize candidate diversification under bounded latency and bounded noise.

### Preconditions
- namespace is validated
- policy context is loaded
- latency budget is available
- tier capabilities are known

### Heuristic
1. start with the cheapest exact signals
2. cap expansion by query class
3. only pay semantic cost when exact and graph signals are insufficient
4. preserve explainability and provenance
5. update reinforcement or decay after outcome feedback

### Failure risks
- candidate explosion
- stale data bias
- low-confidence amplification
- cross-session interference

### Instrumentation
- p50/p95/p99 latency
- candidate counts before and after pruning
- success contribution estimate
- cache hit ratio
- repairability score

### Pseudocode
```text
fn pattern_211(ctx):
    budget = ctx.latency_budget
    exact = cheap_exact(ctx)
    graph = bounded_graph(ctx, exact)
    sem = maybe_semantic(ctx, budget, graph)
    merged = dedup_merge(exact, graph, sem)
    ranked = rank_with_policy(ctx, merged)
    return package(ctx, ranked[:ctx.limit])
```
## 212. Candidate Diversification pattern 2

### Objective
Optimize candidate diversification under bounded latency and bounded noise.

### Preconditions
- namespace is validated
- policy context is loaded
- latency budget is available
- tier capabilities are known

### Heuristic
1. start with the cheapest exact signals
2. cap expansion by query class
3. only pay semantic cost when exact and graph signals are insufficient
4. preserve explainability and provenance
5. update reinforcement or decay after outcome feedback

### Failure risks
- candidate explosion
- stale data bias
- low-confidence amplification
- cross-session interference

### Instrumentation
- p50/p95/p99 latency
- candidate counts before and after pruning
- success contribution estimate
- cache hit ratio
- repairability score

### Pseudocode
```text
fn pattern_212(ctx):
    budget = ctx.latency_budget
    exact = cheap_exact(ctx)
    graph = bounded_graph(ctx, exact)
    sem = maybe_semantic(ctx, budget, graph)
    merged = dedup_merge(exact, graph, sem)
    ranked = rank_with_policy(ctx, merged)
    return package(ctx, ranked[:ctx.limit])
```
## 213. Candidate Diversification pattern 3

### Objective
Optimize candidate diversification under bounded latency and bounded noise.

### Preconditions
- namespace is validated
- policy context is loaded
- latency budget is available
- tier capabilities are known

### Heuristic
1. start with the cheapest exact signals
2. cap expansion by query class
3. only pay semantic cost when exact and graph signals are insufficient
4. preserve explainability and provenance
5. update reinforcement or decay after outcome feedback

### Failure risks
- candidate explosion
- stale data bias
- low-confidence amplification
- cross-session interference

### Instrumentation
- p50/p95/p99 latency
- candidate counts before and after pruning
- success contribution estimate
- cache hit ratio
- repairability score

### Pseudocode
```text
fn pattern_213(ctx):
    budget = ctx.latency_budget
    exact = cheap_exact(ctx)
    graph = bounded_graph(ctx, exact)
    sem = maybe_semantic(ctx, budget, graph)
    merged = dedup_merge(exact, graph, sem)
    ranked = rank_with_policy(ctx, merged)
    return package(ctx, ranked[:ctx.limit])
```
## 214. Candidate Diversification pattern 4

### Objective
Optimize candidate diversification under bounded latency and bounded noise.

### Preconditions
- namespace is validated
- policy context is loaded
- latency budget is available
- tier capabilities are known

### Heuristic
1. start with the cheapest exact signals
2. cap expansion by query class
3. only pay semantic cost when exact and graph signals are insufficient
4. preserve explainability and provenance
5. update reinforcement or decay after outcome feedback

### Failure risks
- candidate explosion
- stale data bias
- low-confidence amplification
- cross-session interference

### Instrumentation
- p50/p95/p99 latency
- candidate counts before and after pruning
- success contribution estimate
- cache hit ratio
- repairability score

### Pseudocode
```text
fn pattern_214(ctx):
    budget = ctx.latency_budget
    exact = cheap_exact(ctx)
    graph = bounded_graph(ctx, exact)
    sem = maybe_semantic(ctx, budget, graph)
    merged = dedup_merge(exact, graph, sem)
    ranked = rank_with_policy(ctx, merged)
    return package(ctx, ranked[:ctx.limit])
```
## 215. Candidate Diversification pattern 5

### Objective
Optimize candidate diversification under bounded latency and bounded noise.

### Preconditions
- namespace is validated
- policy context is loaded
- latency budget is available
- tier capabilities are known

### Heuristic
1. start with the cheapest exact signals
2. cap expansion by query class
3. only pay semantic cost when exact and graph signals are insufficient
4. preserve explainability and provenance
5. update reinforcement or decay after outcome feedback

### Failure risks
- candidate explosion
- stale data bias
- low-confidence amplification
- cross-session interference

### Instrumentation
- p50/p95/p99 latency
- candidate counts before and after pruning
- success contribution estimate
- cache hit ratio
- repairability score

### Pseudocode
```text
fn pattern_215(ctx):
    budget = ctx.latency_budget
    exact = cheap_exact(ctx)
    graph = bounded_graph(ctx, exact)
    sem = maybe_semantic(ctx, budget, graph)
    merged = dedup_merge(exact, graph, sem)
    ranked = rank_with_policy(ctx, merged)
    return package(ctx, ranked[:ctx.limit])
```
## 216. Candidate Diversification pattern 6

### Objective
Optimize candidate diversification under bounded latency and bounded noise.

### Preconditions
- namespace is validated
- policy context is loaded
- latency budget is available
- tier capabilities are known

### Heuristic
1. start with the cheapest exact signals
2. cap expansion by query class
3. only pay semantic cost when exact and graph signals are insufficient
4. preserve explainability and provenance
5. update reinforcement or decay after outcome feedback

### Failure risks
- candidate explosion
- stale data bias
- low-confidence amplification
- cross-session interference

### Instrumentation
- p50/p95/p99 latency
- candidate counts before and after pruning
- success contribution estimate
- cache hit ratio
- repairability score

### Pseudocode
```text
fn pattern_216(ctx):
    budget = ctx.latency_budget
    exact = cheap_exact(ctx)
    graph = bounded_graph(ctx, exact)
    sem = maybe_semantic(ctx, budget, graph)
    merged = dedup_merge(exact, graph, sem)
    ranked = rank_with_policy(ctx, merged)
    return package(ctx, ranked[:ctx.limit])
```
## 217. Candidate Diversification pattern 7

### Objective
Optimize candidate diversification under bounded latency and bounded noise.

### Preconditions
- namespace is validated
- policy context is loaded
- latency budget is available
- tier capabilities are known

### Heuristic
1. start with the cheapest exact signals
2. cap expansion by query class
3. only pay semantic cost when exact and graph signals are insufficient
4. preserve explainability and provenance
5. update reinforcement or decay after outcome feedback

### Failure risks
- candidate explosion
- stale data bias
- low-confidence amplification
- cross-session interference

### Instrumentation
- p50/p95/p99 latency
- candidate counts before and after pruning
- success contribution estimate
- cache hit ratio
- repairability score

### Pseudocode
```text
fn pattern_217(ctx):
    budget = ctx.latency_budget
    exact = cheap_exact(ctx)
    graph = bounded_graph(ctx, exact)
    sem = maybe_semantic(ctx, budget, graph)
    merged = dedup_merge(exact, graph, sem)
    ranked = rank_with_policy(ctx, merged)
    return package(ctx, ranked[:ctx.limit])
```
## 218. Candidate Diversification pattern 8

### Objective
Optimize candidate diversification under bounded latency and bounded noise.

### Preconditions
- namespace is validated
- policy context is loaded
- latency budget is available
- tier capabilities are known

### Heuristic
1. start with the cheapest exact signals
2. cap expansion by query class
3. only pay semantic cost when exact and graph signals are insufficient
4. preserve explainability and provenance
5. update reinforcement or decay after outcome feedback

### Failure risks
- candidate explosion
- stale data bias
- low-confidence amplification
- cross-session interference

### Instrumentation
- p50/p95/p99 latency
- candidate counts before and after pruning
- success contribution estimate
- cache hit ratio
- repairability score

### Pseudocode
```text
fn pattern_218(ctx):
    budget = ctx.latency_budget
    exact = cheap_exact(ctx)
    graph = bounded_graph(ctx, exact)
    sem = maybe_semantic(ctx, budget, graph)
    merged = dedup_merge(exact, graph, sem)
    ranked = rank_with_policy(ctx, merged)
    return package(ctx, ranked[:ctx.limit])
```
## 219. Candidate Diversification pattern 9

### Objective
Optimize candidate diversification under bounded latency and bounded noise.

### Preconditions
- namespace is validated
- policy context is loaded
- latency budget is available
- tier capabilities are known

### Heuristic
1. start with the cheapest exact signals
2. cap expansion by query class
3. only pay semantic cost when exact and graph signals are insufficient
4. preserve explainability and provenance
5. update reinforcement or decay after outcome feedback

### Failure risks
- candidate explosion
- stale data bias
- low-confidence amplification
- cross-session interference

### Instrumentation
- p50/p95/p99 latency
- candidate counts before and after pruning
- success contribution estimate
- cache hit ratio
- repairability score

### Pseudocode
```text
fn pattern_219(ctx):
    budget = ctx.latency_budget
    exact = cheap_exact(ctx)
    graph = bounded_graph(ctx, exact)
    sem = maybe_semantic(ctx, budget, graph)
    merged = dedup_merge(exact, graph, sem)
    ranked = rank_with_policy(ctx, merged)
    return package(ctx, ranked[:ctx.limit])
```
## 220. Candidate Diversification pattern 10

### Objective
Optimize candidate diversification under bounded latency and bounded noise.

### Preconditions
- namespace is validated
- policy context is loaded
- latency budget is available
- tier capabilities are known

### Heuristic
1. start with the cheapest exact signals
2. cap expansion by query class
3. only pay semantic cost when exact and graph signals are insufficient
4. preserve explainability and provenance
5. update reinforcement or decay after outcome feedback

### Failure risks
- candidate explosion
- stale data bias
- low-confidence amplification
- cross-session interference

### Instrumentation
- p50/p95/p99 latency
- candidate counts before and after pruning
- success contribution estimate
- cache hit ratio
- repairability score

### Pseudocode
```text
fn pattern_220(ctx):
    budget = ctx.latency_budget
    exact = cheap_exact(ctx)
    graph = bounded_graph(ctx, exact)
    sem = maybe_semantic(ctx, budget, graph)
    merged = dedup_merge(exact, graph, sem)
    ranked = rank_with_policy(ctx, merged)
    return package(ctx, ranked[:ctx.limit])
```
## 221. Candidate Diversification pattern 11

### Objective
Optimize candidate diversification under bounded latency and bounded noise.

### Preconditions
- namespace is validated
- policy context is loaded
- latency budget is available
- tier capabilities are known

### Heuristic
1. start with the cheapest exact signals
2. cap expansion by query class
3. only pay semantic cost when exact and graph signals are insufficient
4. preserve explainability and provenance
5. update reinforcement or decay after outcome feedback

### Failure risks
- candidate explosion
- stale data bias
- low-confidence amplification
- cross-session interference

### Instrumentation
- p50/p95/p99 latency
- candidate counts before and after pruning
- success contribution estimate
- cache hit ratio
- repairability score

### Pseudocode
```text
fn pattern_221(ctx):
    budget = ctx.latency_budget
    exact = cheap_exact(ctx)
    graph = bounded_graph(ctx, exact)
    sem = maybe_semantic(ctx, budget, graph)
    merged = dedup_merge(exact, graph, sem)
    ranked = rank_with_policy(ctx, merged)
    return package(ctx, ranked[:ctx.limit])
```
## 222. Candidate Diversification pattern 12

### Objective
Optimize candidate diversification under bounded latency and bounded noise.

### Preconditions
- namespace is validated
- policy context is loaded
- latency budget is available
- tier capabilities are known

### Heuristic
1. start with the cheapest exact signals
2. cap expansion by query class
3. only pay semantic cost when exact and graph signals are insufficient
4. preserve explainability and provenance
5. update reinforcement or decay after outcome feedback

### Failure risks
- candidate explosion
- stale data bias
- low-confidence amplification
- cross-session interference

### Instrumentation
- p50/p95/p99 latency
- candidate counts before and after pruning
- success contribution estimate
- cache hit ratio
- repairability score

### Pseudocode
```text
fn pattern_222(ctx):
    budget = ctx.latency_budget
    exact = cheap_exact(ctx)
    graph = bounded_graph(ctx, exact)
    sem = maybe_semantic(ctx, budget, graph)
    merged = dedup_merge(exact, graph, sem)
    ranked = rank_with_policy(ctx, merged)
    return package(ctx, ranked[:ctx.limit])
```
## 223. Candidate Diversification pattern 13

### Objective
Optimize candidate diversification under bounded latency and bounded noise.

### Preconditions
- namespace is validated
- policy context is loaded
- latency budget is available
- tier capabilities are known

### Heuristic
1. start with the cheapest exact signals
2. cap expansion by query class
3. only pay semantic cost when exact and graph signals are insufficient
4. preserve explainability and provenance
5. update reinforcement or decay after outcome feedback

### Failure risks
- candidate explosion
- stale data bias
- low-confidence amplification
- cross-session interference

### Instrumentation
- p50/p95/p99 latency
- candidate counts before and after pruning
- success contribution estimate
- cache hit ratio
- repairability score

### Pseudocode
```text
fn pattern_223(ctx):
    budget = ctx.latency_budget
    exact = cheap_exact(ctx)
    graph = bounded_graph(ctx, exact)
    sem = maybe_semantic(ctx, budget, graph)
    merged = dedup_merge(exact, graph, sem)
    ranked = rank_with_policy(ctx, merged)
    return package(ctx, ranked[:ctx.limit])
```
## 224. Candidate Diversification pattern 14

### Objective
Optimize candidate diversification under bounded latency and bounded noise.

### Preconditions
- namespace is validated
- policy context is loaded
- latency budget is available
- tier capabilities are known

### Heuristic
1. start with the cheapest exact signals
2. cap expansion by query class
3. only pay semantic cost when exact and graph signals are insufficient
4. preserve explainability and provenance
5. update reinforcement or decay after outcome feedback

### Failure risks
- candidate explosion
- stale data bias
- low-confidence amplification
- cross-session interference

### Instrumentation
- p50/p95/p99 latency
- candidate counts before and after pruning
- success contribution estimate
- cache hit ratio
- repairability score

### Pseudocode
```text
fn pattern_224(ctx):
    budget = ctx.latency_budget
    exact = cheap_exact(ctx)
    graph = bounded_graph(ctx, exact)
    sem = maybe_semantic(ctx, budget, graph)
    merged = dedup_merge(exact, graph, sem)
    ranked = rank_with_policy(ctx, merged)
    return package(ctx, ranked[:ctx.limit])
```
## 225. Candidate Diversification pattern 15

### Objective
Optimize candidate diversification under bounded latency and bounded noise.

### Preconditions
- namespace is validated
- policy context is loaded
- latency budget is available
- tier capabilities are known

### Heuristic
1. start with the cheapest exact signals
2. cap expansion by query class
3. only pay semantic cost when exact and graph signals are insufficient
4. preserve explainability and provenance
5. update reinforcement or decay after outcome feedback

### Failure risks
- candidate explosion
- stale data bias
- low-confidence amplification
- cross-session interference

### Instrumentation
- p50/p95/p99 latency
- candidate counts before and after pruning
- success contribution estimate
- cache hit ratio
- repairability score

### Pseudocode
```text
fn pattern_225(ctx):
    budget = ctx.latency_budget
    exact = cheap_exact(ctx)
    graph = bounded_graph(ctx, exact)
    sem = maybe_semantic(ctx, budget, graph)
    merged = dedup_merge(exact, graph, sem)
    ranked = rank_with_policy(ctx, merged)
    return package(ctx, ranked[:ctx.limit])
```
## 226. Candidate Diversification pattern 16

### Objective
Optimize candidate diversification under bounded latency and bounded noise.

### Preconditions
- namespace is validated
- policy context is loaded
- latency budget is available
- tier capabilities are known

### Heuristic
1. start with the cheapest exact signals
2. cap expansion by query class
3. only pay semantic cost when exact and graph signals are insufficient
4. preserve explainability and provenance
5. update reinforcement or decay after outcome feedback

### Failure risks
- candidate explosion
- stale data bias
- low-confidence amplification
- cross-session interference

### Instrumentation
- p50/p95/p99 latency
- candidate counts before and after pruning
- success contribution estimate
- cache hit ratio
- repairability score

### Pseudocode
```text
fn pattern_226(ctx):
    budget = ctx.latency_budget
    exact = cheap_exact(ctx)
    graph = bounded_graph(ctx, exact)
    sem = maybe_semantic(ctx, budget, graph)
    merged = dedup_merge(exact, graph, sem)
    ranked = rank_with_policy(ctx, merged)
    return package(ctx, ranked[:ctx.limit])
```
## 227. Candidate Diversification pattern 17

### Objective
Optimize candidate diversification under bounded latency and bounded noise.

### Preconditions
- namespace is validated
- policy context is loaded
- latency budget is available
- tier capabilities are known

### Heuristic
1. start with the cheapest exact signals
2. cap expansion by query class
3. only pay semantic cost when exact and graph signals are insufficient
4. preserve explainability and provenance
5. update reinforcement or decay after outcome feedback

### Failure risks
- candidate explosion
- stale data bias
- low-confidence amplification
- cross-session interference

### Instrumentation
- p50/p95/p99 latency
- candidate counts before and after pruning
- success contribution estimate
- cache hit ratio
- repairability score

### Pseudocode
```text
fn pattern_227(ctx):
    budget = ctx.latency_budget
    exact = cheap_exact(ctx)
    graph = bounded_graph(ctx, exact)
    sem = maybe_semantic(ctx, budget, graph)
    merged = dedup_merge(exact, graph, sem)
    ranked = rank_with_policy(ctx, merged)
    return package(ctx, ranked[:ctx.limit])
```
## 228. Candidate Diversification pattern 18

### Objective
Optimize candidate diversification under bounded latency and bounded noise.

### Preconditions
- namespace is validated
- policy context is loaded
- latency budget is available
- tier capabilities are known

### Heuristic
1. start with the cheapest exact signals
2. cap expansion by query class
3. only pay semantic cost when exact and graph signals are insufficient
4. preserve explainability and provenance
5. update reinforcement or decay after outcome feedback

### Failure risks
- candidate explosion
- stale data bias
- low-confidence amplification
- cross-session interference

### Instrumentation
- p50/p95/p99 latency
- candidate counts before and after pruning
- success contribution estimate
- cache hit ratio
- repairability score

### Pseudocode
```text
fn pattern_228(ctx):
    budget = ctx.latency_budget
    exact = cheap_exact(ctx)
    graph = bounded_graph(ctx, exact)
    sem = maybe_semantic(ctx, budget, graph)
    merged = dedup_merge(exact, graph, sem)
    ranked = rank_with_policy(ctx, merged)
    return package(ctx, ranked[:ctx.limit])
```
## 229. Candidate Diversification pattern 19

### Objective
Optimize candidate diversification under bounded latency and bounded noise.

### Preconditions
- namespace is validated
- policy context is loaded
- latency budget is available
- tier capabilities are known

### Heuristic
1. start with the cheapest exact signals
2. cap expansion by query class
3. only pay semantic cost when exact and graph signals are insufficient
4. preserve explainability and provenance
5. update reinforcement or decay after outcome feedback

### Failure risks
- candidate explosion
- stale data bias
- low-confidence amplification
- cross-session interference

### Instrumentation
- p50/p95/p99 latency
- candidate counts before and after pruning
- success contribution estimate
- cache hit ratio
- repairability score

### Pseudocode
```text
fn pattern_229(ctx):
    budget = ctx.latency_budget
    exact = cheap_exact(ctx)
    graph = bounded_graph(ctx, exact)
    sem = maybe_semantic(ctx, budget, graph)
    merged = dedup_merge(exact, graph, sem)
    ranked = rank_with_policy(ctx, merged)
    return package(ctx, ranked[:ctx.limit])
```
## 230. Candidate Diversification pattern 20

### Objective
Optimize candidate diversification under bounded latency and bounded noise.

### Preconditions
- namespace is validated
- policy context is loaded
- latency budget is available
- tier capabilities are known

### Heuristic
1. start with the cheapest exact signals
2. cap expansion by query class
3. only pay semantic cost when exact and graph signals are insufficient
4. preserve explainability and provenance
5. update reinforcement or decay after outcome feedback

### Failure risks
- candidate explosion
- stale data bias
- low-confidence amplification
- cross-session interference

### Instrumentation
- p50/p95/p99 latency
- candidate counts before and after pruning
- success contribution estimate
- cache hit ratio
- repairability score

### Pseudocode
```text
fn pattern_230(ctx):
    budget = ctx.latency_budget
    exact = cheap_exact(ctx)
    graph = bounded_graph(ctx, exact)
    sem = maybe_semantic(ctx, budget, graph)
    merged = dedup_merge(exact, graph, sem)
    ranked = rank_with_policy(ctx, merged)
    return package(ctx, ranked[:ctx.limit])
```
## 231. Candidate Diversification pattern 21

### Objective
Optimize candidate diversification under bounded latency and bounded noise.

### Preconditions
- namespace is validated
- policy context is loaded
- latency budget is available
- tier capabilities are known

### Heuristic
1. start with the cheapest exact signals
2. cap expansion by query class
3. only pay semantic cost when exact and graph signals are insufficient
4. preserve explainability and provenance
5. update reinforcement or decay after outcome feedback

### Failure risks
- candidate explosion
- stale data bias
- low-confidence amplification
- cross-session interference

### Instrumentation
- p50/p95/p99 latency
- candidate counts before and after pruning
- success contribution estimate
- cache hit ratio
- repairability score

### Pseudocode
```text
fn pattern_231(ctx):
    budget = ctx.latency_budget
    exact = cheap_exact(ctx)
    graph = bounded_graph(ctx, exact)
    sem = maybe_semantic(ctx, budget, graph)
    merged = dedup_merge(exact, graph, sem)
    ranked = rank_with_policy(ctx, merged)
    return package(ctx, ranked[:ctx.limit])
```
## 232. Candidate Diversification pattern 22

### Objective
Optimize candidate diversification under bounded latency and bounded noise.

### Preconditions
- namespace is validated
- policy context is loaded
- latency budget is available
- tier capabilities are known

### Heuristic
1. start with the cheapest exact signals
2. cap expansion by query class
3. only pay semantic cost when exact and graph signals are insufficient
4. preserve explainability and provenance
5. update reinforcement or decay after outcome feedback

### Failure risks
- candidate explosion
- stale data bias
- low-confidence amplification
- cross-session interference

### Instrumentation
- p50/p95/p99 latency
- candidate counts before and after pruning
- success contribution estimate
- cache hit ratio
- repairability score

### Pseudocode
```text
fn pattern_232(ctx):
    budget = ctx.latency_budget
    exact = cheap_exact(ctx)
    graph = bounded_graph(ctx, exact)
    sem = maybe_semantic(ctx, budget, graph)
    merged = dedup_merge(exact, graph, sem)
    ranked = rank_with_policy(ctx, merged)
    return package(ctx, ranked[:ctx.limit])
```
## 233. Candidate Diversification pattern 23

### Objective
Optimize candidate diversification under bounded latency and bounded noise.

### Preconditions
- namespace is validated
- policy context is loaded
- latency budget is available
- tier capabilities are known

### Heuristic
1. start with the cheapest exact signals
2. cap expansion by query class
3. only pay semantic cost when exact and graph signals are insufficient
4. preserve explainability and provenance
5. update reinforcement or decay after outcome feedback

### Failure risks
- candidate explosion
- stale data bias
- low-confidence amplification
- cross-session interference

### Instrumentation
- p50/p95/p99 latency
- candidate counts before and after pruning
- success contribution estimate
- cache hit ratio
- repairability score

### Pseudocode
```text
fn pattern_233(ctx):
    budget = ctx.latency_budget
    exact = cheap_exact(ctx)
    graph = bounded_graph(ctx, exact)
    sem = maybe_semantic(ctx, budget, graph)
    merged = dedup_merge(exact, graph, sem)
    ranked = rank_with_policy(ctx, merged)
    return package(ctx, ranked[:ctx.limit])
```
## 234. Candidate Diversification pattern 24

### Objective
Optimize candidate diversification under bounded latency and bounded noise.

### Preconditions
- namespace is validated
- policy context is loaded
- latency budget is available
- tier capabilities are known

### Heuristic
1. start with the cheapest exact signals
2. cap expansion by query class
3. only pay semantic cost when exact and graph signals are insufficient
4. preserve explainability and provenance
5. update reinforcement or decay after outcome feedback

### Failure risks
- candidate explosion
- stale data bias
- low-confidence amplification
- cross-session interference

### Instrumentation
- p50/p95/p99 latency
- candidate counts before and after pruning
- success contribution estimate
- cache hit ratio
- repairability score

### Pseudocode
```text
fn pattern_234(ctx):
    budget = ctx.latency_budget
    exact = cheap_exact(ctx)
    graph = bounded_graph(ctx, exact)
    sem = maybe_semantic(ctx, budget, graph)
    merged = dedup_merge(exact, graph, sem)
    ranked = rank_with_policy(ctx, merged)
    return package(ctx, ranked[:ctx.limit])
```
## 235. Candidate Diversification pattern 25

### Objective
Optimize candidate diversification under bounded latency and bounded noise.

### Preconditions
- namespace is validated
- policy context is loaded
- latency budget is available
- tier capabilities are known

### Heuristic
1. start with the cheapest exact signals
2. cap expansion by query class
3. only pay semantic cost when exact and graph signals are insufficient
4. preserve explainability and provenance
5. update reinforcement or decay after outcome feedback

### Failure risks
- candidate explosion
- stale data bias
- low-confidence amplification
- cross-session interference

### Instrumentation
- p50/p95/p99 latency
- candidate counts before and after pruning
- success contribution estimate
- cache hit ratio
- repairability score

### Pseudocode
```text
fn pattern_235(ctx):
    budget = ctx.latency_budget
    exact = cheap_exact(ctx)
    graph = bounded_graph(ctx, exact)
    sem = maybe_semantic(ctx, budget, graph)
    merged = dedup_merge(exact, graph, sem)
    ranked = rank_with_policy(ctx, merged)
    return package(ctx, ranked[:ctx.limit])
```
## 236. Candidate Diversification pattern 26

### Objective
Optimize candidate diversification under bounded latency and bounded noise.

### Preconditions
- namespace is validated
- policy context is loaded
- latency budget is available
- tier capabilities are known

### Heuristic
1. start with the cheapest exact signals
2. cap expansion by query class
3. only pay semantic cost when exact and graph signals are insufficient
4. preserve explainability and provenance
5. update reinforcement or decay after outcome feedback

### Failure risks
- candidate explosion
- stale data bias
- low-confidence amplification
- cross-session interference

### Instrumentation
- p50/p95/p99 latency
- candidate counts before and after pruning
- success contribution estimate
- cache hit ratio
- repairability score

### Pseudocode
```text
fn pattern_236(ctx):
    budget = ctx.latency_budget
    exact = cheap_exact(ctx)
    graph = bounded_graph(ctx, exact)
    sem = maybe_semantic(ctx, budget, graph)
    merged = dedup_merge(exact, graph, sem)
    ranked = rank_with_policy(ctx, merged)
    return package(ctx, ranked[:ctx.limit])
```
## 237. Candidate Diversification pattern 27

### Objective
Optimize candidate diversification under bounded latency and bounded noise.

### Preconditions
- namespace is validated
- policy context is loaded
- latency budget is available
- tier capabilities are known

### Heuristic
1. start with the cheapest exact signals
2. cap expansion by query class
3. only pay semantic cost when exact and graph signals are insufficient
4. preserve explainability and provenance
5. update reinforcement or decay after outcome feedback

### Failure risks
- candidate explosion
- stale data bias
- low-confidence amplification
- cross-session interference

### Instrumentation
- p50/p95/p99 latency
- candidate counts before and after pruning
- success contribution estimate
- cache hit ratio
- repairability score

### Pseudocode
```text
fn pattern_237(ctx):
    budget = ctx.latency_budget
    exact = cheap_exact(ctx)
    graph = bounded_graph(ctx, exact)
    sem = maybe_semantic(ctx, budget, graph)
    merged = dedup_merge(exact, graph, sem)
    ranked = rank_with_policy(ctx, merged)
    return package(ctx, ranked[:ctx.limit])
```
## 238. Candidate Diversification pattern 28

### Objective
Optimize candidate diversification under bounded latency and bounded noise.

### Preconditions
- namespace is validated
- policy context is loaded
- latency budget is available
- tier capabilities are known

### Heuristic
1. start with the cheapest exact signals
2. cap expansion by query class
3. only pay semantic cost when exact and graph signals are insufficient
4. preserve explainability and provenance
5. update reinforcement or decay after outcome feedback

### Failure risks
- candidate explosion
- stale data bias
- low-confidence amplification
- cross-session interference

### Instrumentation
- p50/p95/p99 latency
- candidate counts before and after pruning
- success contribution estimate
- cache hit ratio
- repairability score

### Pseudocode
```text
fn pattern_238(ctx):
    budget = ctx.latency_budget
    exact = cheap_exact(ctx)
    graph = bounded_graph(ctx, exact)
    sem = maybe_semantic(ctx, budget, graph)
    merged = dedup_merge(exact, graph, sem)
    ranked = rank_with_policy(ctx, merged)
    return package(ctx, ranked[:ctx.limit])
```
## 239. Candidate Diversification pattern 29

### Objective
Optimize candidate diversification under bounded latency and bounded noise.

### Preconditions
- namespace is validated
- policy context is loaded
- latency budget is available
- tier capabilities are known

### Heuristic
1. start with the cheapest exact signals
2. cap expansion by query class
3. only pay semantic cost when exact and graph signals are insufficient
4. preserve explainability and provenance
5. update reinforcement or decay after outcome feedback

### Failure risks
- candidate explosion
- stale data bias
- low-confidence amplification
- cross-session interference

### Instrumentation
- p50/p95/p99 latency
- candidate counts before and after pruning
- success contribution estimate
- cache hit ratio
- repairability score

### Pseudocode
```text
fn pattern_239(ctx):
    budget = ctx.latency_budget
    exact = cheap_exact(ctx)
    graph = bounded_graph(ctx, exact)
    sem = maybe_semantic(ctx, budget, graph)
    merged = dedup_merge(exact, graph, sem)
    ranked = rank_with_policy(ctx, merged)
    return package(ctx, ranked[:ctx.limit])
```
## 240. Candidate Diversification pattern 30

### Objective
Optimize candidate diversification under bounded latency and bounded noise.

### Preconditions
- namespace is validated
- policy context is loaded
- latency budget is available
- tier capabilities are known

### Heuristic
1. start with the cheapest exact signals
2. cap expansion by query class
3. only pay semantic cost when exact and graph signals are insufficient
4. preserve explainability and provenance
5. update reinforcement or decay after outcome feedback

### Failure risks
- candidate explosion
- stale data bias
- low-confidence amplification
- cross-session interference

### Instrumentation
- p50/p95/p99 latency
- candidate counts before and after pruning
- success contribution estimate
- cache hit ratio
- repairability score

### Pseudocode
```text
fn pattern_240(ctx):
    budget = ctx.latency_budget
    exact = cheap_exact(ctx)
    graph = bounded_graph(ctx, exact)
    sem = maybe_semantic(ctx, budget, graph)
    merged = dedup_merge(exact, graph, sem)
    ranked = rank_with_policy(ctx, merged)
    return package(ctx, ranked[:ctx.limit])
```
## 241. Episode Clustering pattern 1

### Objective
Optimize episode clustering under bounded latency and bounded noise.

### Preconditions
- namespace is validated
- policy context is loaded
- latency budget is available
- tier capabilities are known

### Heuristic
1. start with the cheapest exact signals
2. cap expansion by query class
3. only pay semantic cost when exact and graph signals are insufficient
4. preserve explainability and provenance
5. update reinforcement or decay after outcome feedback

### Failure risks
- candidate explosion
- stale data bias
- low-confidence amplification
- cross-session interference

### Instrumentation
- p50/p95/p99 latency
- candidate counts before and after pruning
- success contribution estimate
- cache hit ratio
- repairability score

### Pseudocode
```text
fn pattern_241(ctx):
    budget = ctx.latency_budget
    exact = cheap_exact(ctx)
    graph = bounded_graph(ctx, exact)
    sem = maybe_semantic(ctx, budget, graph)
    merged = dedup_merge(exact, graph, sem)
    ranked = rank_with_policy(ctx, merged)
    return package(ctx, ranked[:ctx.limit])
```
## 242. Episode Clustering pattern 2

### Objective
Optimize episode clustering under bounded latency and bounded noise.

### Preconditions
- namespace is validated
- policy context is loaded
- latency budget is available
- tier capabilities are known

### Heuristic
1. start with the cheapest exact signals
2. cap expansion by query class
3. only pay semantic cost when exact and graph signals are insufficient
4. preserve explainability and provenance
5. update reinforcement or decay after outcome feedback

### Failure risks
- candidate explosion
- stale data bias
- low-confidence amplification
- cross-session interference

### Instrumentation
- p50/p95/p99 latency
- candidate counts before and after pruning
- success contribution estimate
- cache hit ratio
- repairability score

### Pseudocode
```text
fn pattern_242(ctx):
    budget = ctx.latency_budget
    exact = cheap_exact(ctx)
    graph = bounded_graph(ctx, exact)
    sem = maybe_semantic(ctx, budget, graph)
    merged = dedup_merge(exact, graph, sem)
    ranked = rank_with_policy(ctx, merged)
    return package(ctx, ranked[:ctx.limit])
```
## 243. Episode Clustering pattern 3

### Objective
Optimize episode clustering under bounded latency and bounded noise.

### Preconditions
- namespace is validated
- policy context is loaded
- latency budget is available
- tier capabilities are known

### Heuristic
1. start with the cheapest exact signals
2. cap expansion by query class
3. only pay semantic cost when exact and graph signals are insufficient
4. preserve explainability and provenance
5. update reinforcement or decay after outcome feedback

### Failure risks
- candidate explosion
- stale data bias
- low-confidence amplification
- cross-session interference

### Instrumentation
- p50/p95/p99 latency
- candidate counts before and after pruning
- success contribution estimate
- cache hit ratio
- repairability score

### Pseudocode
```text
fn pattern_243(ctx):
    budget = ctx.latency_budget
    exact = cheap_exact(ctx)
    graph = bounded_graph(ctx, exact)
    sem = maybe_semantic(ctx, budget, graph)
    merged = dedup_merge(exact, graph, sem)
    ranked = rank_with_policy(ctx, merged)
    return package(ctx, ranked[:ctx.limit])
```
## 244. Episode Clustering pattern 4

### Objective
Optimize episode clustering under bounded latency and bounded noise.

### Preconditions
- namespace is validated
- policy context is loaded
- latency budget is available
- tier capabilities are known

### Heuristic
1. start with the cheapest exact signals
2. cap expansion by query class
3. only pay semantic cost when exact and graph signals are insufficient
4. preserve explainability and provenance
5. update reinforcement or decay after outcome feedback

### Failure risks
- candidate explosion
- stale data bias
- low-confidence amplification
- cross-session interference

### Instrumentation
- p50/p95/p99 latency
- candidate counts before and after pruning
- success contribution estimate
- cache hit ratio
- repairability score

### Pseudocode
```text
fn pattern_244(ctx):
    budget = ctx.latency_budget
    exact = cheap_exact(ctx)
    graph = bounded_graph(ctx, exact)
    sem = maybe_semantic(ctx, budget, graph)
    merged = dedup_merge(exact, graph, sem)
    ranked = rank_with_policy(ctx, merged)
    return package(ctx, ranked[:ctx.limit])
```
## 245. Episode Clustering pattern 5

### Objective
Optimize episode clustering under bounded latency and bounded noise.

### Preconditions
- namespace is validated
- policy context is loaded
- latency budget is available
- tier capabilities are known

### Heuristic
1. start with the cheapest exact signals
2. cap expansion by query class
3. only pay semantic cost when exact and graph signals are insufficient
4. preserve explainability and provenance
5. update reinforcement or decay after outcome feedback

### Failure risks
- candidate explosion
- stale data bias
- low-confidence amplification
- cross-session interference

### Instrumentation
- p50/p95/p99 latency
- candidate counts before and after pruning
- success contribution estimate
- cache hit ratio
- repairability score

### Pseudocode
```text
fn pattern_245(ctx):
    budget = ctx.latency_budget
    exact = cheap_exact(ctx)
    graph = bounded_graph(ctx, exact)
    sem = maybe_semantic(ctx, budget, graph)
    merged = dedup_merge(exact, graph, sem)
    ranked = rank_with_policy(ctx, merged)
    return package(ctx, ranked[:ctx.limit])
```
## 246. Episode Clustering pattern 6

### Objective
Optimize episode clustering under bounded latency and bounded noise.

### Preconditions
- namespace is validated
- policy context is loaded
- latency budget is available
- tier capabilities are known

### Heuristic
1. start with the cheapest exact signals
2. cap expansion by query class
3. only pay semantic cost when exact and graph signals are insufficient
4. preserve explainability and provenance
5. update reinforcement or decay after outcome feedback

### Failure risks
- candidate explosion
- stale data bias
- low-confidence amplification
- cross-session interference

### Instrumentation
- p50/p95/p99 latency
- candidate counts before and after pruning
- success contribution estimate
- cache hit ratio
- repairability score

### Pseudocode
```text
fn pattern_246(ctx):
    budget = ctx.latency_budget
    exact = cheap_exact(ctx)
    graph = bounded_graph(ctx, exact)
    sem = maybe_semantic(ctx, budget, graph)
    merged = dedup_merge(exact, graph, sem)
    ranked = rank_with_policy(ctx, merged)
    return package(ctx, ranked[:ctx.limit])
```
## 247. Episode Clustering pattern 7

### Objective
Optimize episode clustering under bounded latency and bounded noise.

### Preconditions
- namespace is validated
- policy context is loaded
- latency budget is available
- tier capabilities are known

### Heuristic
1. start with the cheapest exact signals
2. cap expansion by query class
3. only pay semantic cost when exact and graph signals are insufficient
4. preserve explainability and provenance
5. update reinforcement or decay after outcome feedback

### Failure risks
- candidate explosion
- stale data bias
- low-confidence amplification
- cross-session interference

### Instrumentation
- p50/p95/p99 latency
- candidate counts before and after pruning
- success contribution estimate
- cache hit ratio
- repairability score

### Pseudocode
```text
fn pattern_247(ctx):
    budget = ctx.latency_budget
    exact = cheap_exact(ctx)
    graph = bounded_graph(ctx, exact)
    sem = maybe_semantic(ctx, budget, graph)
    merged = dedup_merge(exact, graph, sem)
    ranked = rank_with_policy(ctx, merged)
    return package(ctx, ranked[:ctx.limit])
```
## 248. Episode Clustering pattern 8

### Objective
Optimize episode clustering under bounded latency and bounded noise.

### Preconditions
- namespace is validated
- policy context is loaded
- latency budget is available
- tier capabilities are known

### Heuristic
1. start with the cheapest exact signals
2. cap expansion by query class
3. only pay semantic cost when exact and graph signals are insufficient
4. preserve explainability and provenance
5. update reinforcement or decay after outcome feedback

### Failure risks
- candidate explosion
- stale data bias
- low-confidence amplification
- cross-session interference

### Instrumentation
- p50/p95/p99 latency
- candidate counts before and after pruning
- success contribution estimate
- cache hit ratio
- repairability score

### Pseudocode
```text
fn pattern_248(ctx):
    budget = ctx.latency_budget
    exact = cheap_exact(ctx)
    graph = bounded_graph(ctx, exact)
    sem = maybe_semantic(ctx, budget, graph)
    merged = dedup_merge(exact, graph, sem)
    ranked = rank_with_policy(ctx, merged)
    return package(ctx, ranked[:ctx.limit])
```
## 249. Episode Clustering pattern 9

### Objective
Optimize episode clustering under bounded latency and bounded noise.

### Preconditions
- namespace is validated
- policy context is loaded
- latency budget is available
- tier capabilities are known

### Heuristic
1. start with the cheapest exact signals
2. cap expansion by query class
3. only pay semantic cost when exact and graph signals are insufficient
4. preserve explainability and provenance
5. update reinforcement or decay after outcome feedback

### Failure risks
- candidate explosion
- stale data bias
- low-confidence amplification
- cross-session interference

### Instrumentation
- p50/p95/p99 latency
- candidate counts before and after pruning
- success contribution estimate
- cache hit ratio
- repairability score

### Pseudocode
```text
fn pattern_249(ctx):
    budget = ctx.latency_budget
    exact = cheap_exact(ctx)
    graph = bounded_graph(ctx, exact)
    sem = maybe_semantic(ctx, budget, graph)
    merged = dedup_merge(exact, graph, sem)
    ranked = rank_with_policy(ctx, merged)
    return package(ctx, ranked[:ctx.limit])
```
## 250. Episode Clustering pattern 10

### Objective
Optimize episode clustering under bounded latency and bounded noise.

### Preconditions
- namespace is validated
- policy context is loaded
- latency budget is available
- tier capabilities are known

### Heuristic
1. start with the cheapest exact signals
2. cap expansion by query class
3. only pay semantic cost when exact and graph signals are insufficient
4. preserve explainability and provenance
5. update reinforcement or decay after outcome feedback

### Failure risks
- candidate explosion
- stale data bias
- low-confidence amplification
- cross-session interference

### Instrumentation
- p50/p95/p99 latency
- candidate counts before and after pruning
- success contribution estimate
- cache hit ratio
- repairability score

### Pseudocode
```text
fn pattern_250(ctx):
    budget = ctx.latency_budget
    exact = cheap_exact(ctx)
    graph = bounded_graph(ctx, exact)
    sem = maybe_semantic(ctx, budget, graph)
    merged = dedup_merge(exact, graph, sem)
    ranked = rank_with_policy(ctx, merged)
    return package(ctx, ranked[:ctx.limit])
```
## 251. Episode Clustering pattern 11

### Objective
Optimize episode clustering under bounded latency and bounded noise.

### Preconditions
- namespace is validated
- policy context is loaded
- latency budget is available
- tier capabilities are known

### Heuristic
1. start with the cheapest exact signals
2. cap expansion by query class
3. only pay semantic cost when exact and graph signals are insufficient
4. preserve explainability and provenance
5. update reinforcement or decay after outcome feedback

### Failure risks
- candidate explosion
- stale data bias
- low-confidence amplification
- cross-session interference

### Instrumentation
- p50/p95/p99 latency
- candidate counts before and after pruning
- success contribution estimate
- cache hit ratio
- repairability score

### Pseudocode
```text
fn pattern_251(ctx):
    budget = ctx.latency_budget
    exact = cheap_exact(ctx)
    graph = bounded_graph(ctx, exact)
    sem = maybe_semantic(ctx, budget, graph)
    merged = dedup_merge(exact, graph, sem)
    ranked = rank_with_policy(ctx, merged)
    return package(ctx, ranked[:ctx.limit])
```
## 252. Episode Clustering pattern 12

### Objective
Optimize episode clustering under bounded latency and bounded noise.

### Preconditions
- namespace is validated
- policy context is loaded
- latency budget is available
- tier capabilities are known

### Heuristic
1. start with the cheapest exact signals
2. cap expansion by query class
3. only pay semantic cost when exact and graph signals are insufficient
4. preserve explainability and provenance
5. update reinforcement or decay after outcome feedback

### Failure risks
- candidate explosion
- stale data bias
- low-confidence amplification
- cross-session interference

### Instrumentation
- p50/p95/p99 latency
- candidate counts before and after pruning
- success contribution estimate
- cache hit ratio
- repairability score

### Pseudocode
```text
fn pattern_252(ctx):
    budget = ctx.latency_budget
    exact = cheap_exact(ctx)
    graph = bounded_graph(ctx, exact)
    sem = maybe_semantic(ctx, budget, graph)
    merged = dedup_merge(exact, graph, sem)
    ranked = rank_with_policy(ctx, merged)
    return package(ctx, ranked[:ctx.limit])
```
## 253. Episode Clustering pattern 13

### Objective
Optimize episode clustering under bounded latency and bounded noise.

### Preconditions
- namespace is validated
- policy context is loaded
- latency budget is available
- tier capabilities are known

### Heuristic
1. start with the cheapest exact signals
2. cap expansion by query class
3. only pay semantic cost when exact and graph signals are insufficient
4. preserve explainability and provenance
5. update reinforcement or decay after outcome feedback

### Failure risks
- candidate explosion
- stale data bias
- low-confidence amplification
- cross-session interference

### Instrumentation
- p50/p95/p99 latency
- candidate counts before and after pruning
- success contribution estimate
- cache hit ratio
- repairability score

### Pseudocode
```text
fn pattern_253(ctx):
    budget = ctx.latency_budget
    exact = cheap_exact(ctx)
    graph = bounded_graph(ctx, exact)
    sem = maybe_semantic(ctx, budget, graph)
    merged = dedup_merge(exact, graph, sem)
    ranked = rank_with_policy(ctx, merged)
    return package(ctx, ranked[:ctx.limit])
```
## 254. Episode Clustering pattern 14

### Objective
Optimize episode clustering under bounded latency and bounded noise.

### Preconditions
- namespace is validated
- policy context is loaded
- latency budget is available
- tier capabilities are known

### Heuristic
1. start with the cheapest exact signals
2. cap expansion by query class
3. only pay semantic cost when exact and graph signals are insufficient
4. preserve explainability and provenance
5. update reinforcement or decay after outcome feedback

### Failure risks
- candidate explosion
- stale data bias
- low-confidence amplification
- cross-session interference

### Instrumentation
- p50/p95/p99 latency
- candidate counts before and after pruning
- success contribution estimate
- cache hit ratio
- repairability score

### Pseudocode
```text
fn pattern_254(ctx):
    budget = ctx.latency_budget
    exact = cheap_exact(ctx)
    graph = bounded_graph(ctx, exact)
    sem = maybe_semantic(ctx, budget, graph)
    merged = dedup_merge(exact, graph, sem)
    ranked = rank_with_policy(ctx, merged)
    return package(ctx, ranked[:ctx.limit])
```
## 255. Episode Clustering pattern 15

### Objective
Optimize episode clustering under bounded latency and bounded noise.

### Preconditions
- namespace is validated
- policy context is loaded
- latency budget is available
- tier capabilities are known

### Heuristic
1. start with the cheapest exact signals
2. cap expansion by query class
3. only pay semantic cost when exact and graph signals are insufficient
4. preserve explainability and provenance
5. update reinforcement or decay after outcome feedback

### Failure risks
- candidate explosion
- stale data bias
- low-confidence amplification
- cross-session interference

### Instrumentation
- p50/p95/p99 latency
- candidate counts before and after pruning
- success contribution estimate
- cache hit ratio
- repairability score

### Pseudocode
```text
fn pattern_255(ctx):
    budget = ctx.latency_budget
    exact = cheap_exact(ctx)
    graph = bounded_graph(ctx, exact)
    sem = maybe_semantic(ctx, budget, graph)
    merged = dedup_merge(exact, graph, sem)
    ranked = rank_with_policy(ctx, merged)
    return package(ctx, ranked[:ctx.limit])
```
## 256. Episode Clustering pattern 16

### Objective
Optimize episode clustering under bounded latency and bounded noise.

### Preconditions
- namespace is validated
- policy context is loaded
- latency budget is available
- tier capabilities are known

### Heuristic
1. start with the cheapest exact signals
2. cap expansion by query class
3. only pay semantic cost when exact and graph signals are insufficient
4. preserve explainability and provenance
5. update reinforcement or decay after outcome feedback

### Failure risks
- candidate explosion
- stale data bias
- low-confidence amplification
- cross-session interference

### Instrumentation
- p50/p95/p99 latency
- candidate counts before and after pruning
- success contribution estimate
- cache hit ratio
- repairability score

### Pseudocode
```text
fn pattern_256(ctx):
    budget = ctx.latency_budget
    exact = cheap_exact(ctx)
    graph = bounded_graph(ctx, exact)
    sem = maybe_semantic(ctx, budget, graph)
    merged = dedup_merge(exact, graph, sem)
    ranked = rank_with_policy(ctx, merged)
    return package(ctx, ranked[:ctx.limit])
```
## 257. Episode Clustering pattern 17

### Objective
Optimize episode clustering under bounded latency and bounded noise.

### Preconditions
- namespace is validated
- policy context is loaded
- latency budget is available
- tier capabilities are known

### Heuristic
1. start with the cheapest exact signals
2. cap expansion by query class
3. only pay semantic cost when exact and graph signals are insufficient
4. preserve explainability and provenance
5. update reinforcement or decay after outcome feedback

### Failure risks
- candidate explosion
- stale data bias
- low-confidence amplification
- cross-session interference

### Instrumentation
- p50/p95/p99 latency
- candidate counts before and after pruning
- success contribution estimate
- cache hit ratio
- repairability score

### Pseudocode
```text
fn pattern_257(ctx):
    budget = ctx.latency_budget
    exact = cheap_exact(ctx)
    graph = bounded_graph(ctx, exact)
    sem = maybe_semantic(ctx, budget, graph)
    merged = dedup_merge(exact, graph, sem)
    ranked = rank_with_policy(ctx, merged)
    return package(ctx, ranked[:ctx.limit])
```
## 258. Episode Clustering pattern 18

### Objective
Optimize episode clustering under bounded latency and bounded noise.

### Preconditions
- namespace is validated
- policy context is loaded
- latency budget is available
- tier capabilities are known

### Heuristic
1. start with the cheapest exact signals
2. cap expansion by query class
3. only pay semantic cost when exact and graph signals are insufficient
4. preserve explainability and provenance
5. update reinforcement or decay after outcome feedback

### Failure risks
- candidate explosion
- stale data bias
- low-confidence amplification
- cross-session interference

### Instrumentation
- p50/p95/p99 latency
- candidate counts before and after pruning
- success contribution estimate
- cache hit ratio
- repairability score

### Pseudocode
```text
fn pattern_258(ctx):
    budget = ctx.latency_budget
    exact = cheap_exact(ctx)
    graph = bounded_graph(ctx, exact)
    sem = maybe_semantic(ctx, budget, graph)
    merged = dedup_merge(exact, graph, sem)
    ranked = rank_with_policy(ctx, merged)
    return package(ctx, ranked[:ctx.limit])
```
## 259. Episode Clustering pattern 19

### Objective
Optimize episode clustering under bounded latency and bounded noise.

### Preconditions
- namespace is validated
- policy context is loaded
- latency budget is available
- tier capabilities are known

### Heuristic
1. start with the cheapest exact signals
2. cap expansion by query class
3. only pay semantic cost when exact and graph signals are insufficient
4. preserve explainability and provenance
5. update reinforcement or decay after outcome feedback

### Failure risks
- candidate explosion
- stale data bias
- low-confidence amplification
- cross-session interference

### Instrumentation
- p50/p95/p99 latency
- candidate counts before and after pruning
- success contribution estimate
- cache hit ratio
- repairability score

### Pseudocode
```text
fn pattern_259(ctx):
    budget = ctx.latency_budget
    exact = cheap_exact(ctx)
    graph = bounded_graph(ctx, exact)
    sem = maybe_semantic(ctx, budget, graph)
    merged = dedup_merge(exact, graph, sem)
    ranked = rank_with_policy(ctx, merged)
    return package(ctx, ranked[:ctx.limit])
```
## 260. Episode Clustering pattern 20

### Objective
Optimize episode clustering under bounded latency and bounded noise.

### Preconditions
- namespace is validated
- policy context is loaded
- latency budget is available
- tier capabilities are known

### Heuristic
1. start with the cheapest exact signals
2. cap expansion by query class
3. only pay semantic cost when exact and graph signals are insufficient
4. preserve explainability and provenance
5. update reinforcement or decay after outcome feedback

### Failure risks
- candidate explosion
- stale data bias
- low-confidence amplification
- cross-session interference

### Instrumentation
- p50/p95/p99 latency
- candidate counts before and after pruning
- success contribution estimate
- cache hit ratio
- repairability score

### Pseudocode
```text
fn pattern_260(ctx):
    budget = ctx.latency_budget
    exact = cheap_exact(ctx)
    graph = bounded_graph(ctx, exact)
    sem = maybe_semantic(ctx, budget, graph)
    merged = dedup_merge(exact, graph, sem)
    ranked = rank_with_policy(ctx, merged)
    return package(ctx, ranked[:ctx.limit])
```
## 261. Episode Clustering pattern 21

### Objective
Optimize episode clustering under bounded latency and bounded noise.

### Preconditions
- namespace is validated
- policy context is loaded
- latency budget is available
- tier capabilities are known

### Heuristic
1. start with the cheapest exact signals
2. cap expansion by query class
3. only pay semantic cost when exact and graph signals are insufficient
4. preserve explainability and provenance
5. update reinforcement or decay after outcome feedback

### Failure risks
- candidate explosion
- stale data bias
- low-confidence amplification
- cross-session interference

### Instrumentation
- p50/p95/p99 latency
- candidate counts before and after pruning
- success contribution estimate
- cache hit ratio
- repairability score

### Pseudocode
```text
fn pattern_261(ctx):
    budget = ctx.latency_budget
    exact = cheap_exact(ctx)
    graph = bounded_graph(ctx, exact)
    sem = maybe_semantic(ctx, budget, graph)
    merged = dedup_merge(exact, graph, sem)
    ranked = rank_with_policy(ctx, merged)
    return package(ctx, ranked[:ctx.limit])
```
## 262. Episode Clustering pattern 22

### Objective
Optimize episode clustering under bounded latency and bounded noise.

### Preconditions
- namespace is validated
- policy context is loaded
- latency budget is available
- tier capabilities are known

### Heuristic
1. start with the cheapest exact signals
2. cap expansion by query class
3. only pay semantic cost when exact and graph signals are insufficient
4. preserve explainability and provenance
5. update reinforcement or decay after outcome feedback

### Failure risks
- candidate explosion
- stale data bias
- low-confidence amplification
- cross-session interference

### Instrumentation
- p50/p95/p99 latency
- candidate counts before and after pruning
- success contribution estimate
- cache hit ratio
- repairability score

### Pseudocode
```text
fn pattern_262(ctx):
    budget = ctx.latency_budget
    exact = cheap_exact(ctx)
    graph = bounded_graph(ctx, exact)
    sem = maybe_semantic(ctx, budget, graph)
    merged = dedup_merge(exact, graph, sem)
    ranked = rank_with_policy(ctx, merged)
    return package(ctx, ranked[:ctx.limit])
```
## 263. Episode Clustering pattern 23

### Objective
Optimize episode clustering under bounded latency and bounded noise.

### Preconditions
- namespace is validated
- policy context is loaded
- latency budget is available
- tier capabilities are known

### Heuristic
1. start with the cheapest exact signals
2. cap expansion by query class
3. only pay semantic cost when exact and graph signals are insufficient
4. preserve explainability and provenance
5. update reinforcement or decay after outcome feedback

### Failure risks
- candidate explosion
- stale data bias
- low-confidence amplification
- cross-session interference

### Instrumentation
- p50/p95/p99 latency
- candidate counts before and after pruning
- success contribution estimate
- cache hit ratio
- repairability score

### Pseudocode
```text
fn pattern_263(ctx):
    budget = ctx.latency_budget
    exact = cheap_exact(ctx)
    graph = bounded_graph(ctx, exact)
    sem = maybe_semantic(ctx, budget, graph)
    merged = dedup_merge(exact, graph, sem)
    ranked = rank_with_policy(ctx, merged)
    return package(ctx, ranked[:ctx.limit])
```
## 264. Episode Clustering pattern 24

### Objective
Optimize episode clustering under bounded latency and bounded noise.

### Preconditions
- namespace is validated
- policy context is loaded
- latency budget is available
- tier capabilities are known

### Heuristic
1. start with the cheapest exact signals
2. cap expansion by query class
3. only pay semantic cost when exact and graph signals are insufficient
4. preserve explainability and provenance
5. update reinforcement or decay after outcome feedback

### Failure risks
- candidate explosion
- stale data bias
- low-confidence amplification
- cross-session interference

### Instrumentation
- p50/p95/p99 latency
- candidate counts before and after pruning
- success contribution estimate
- cache hit ratio
- repairability score

### Pseudocode
```text
fn pattern_264(ctx):
    budget = ctx.latency_budget
    exact = cheap_exact(ctx)
    graph = bounded_graph(ctx, exact)
    sem = maybe_semantic(ctx, budget, graph)
    merged = dedup_merge(exact, graph, sem)
    ranked = rank_with_policy(ctx, merged)
    return package(ctx, ranked[:ctx.limit])
```
## 265. Episode Clustering pattern 25

### Objective
Optimize episode clustering under bounded latency and bounded noise.

### Preconditions
- namespace is validated
- policy context is loaded
- latency budget is available
- tier capabilities are known

### Heuristic
1. start with the cheapest exact signals
2. cap expansion by query class
3. only pay semantic cost when exact and graph signals are insufficient
4. preserve explainability and provenance
5. update reinforcement or decay after outcome feedback

### Failure risks
- candidate explosion
- stale data bias
- low-confidence amplification
- cross-session interference

### Instrumentation
- p50/p95/p99 latency
- candidate counts before and after pruning
- success contribution estimate
- cache hit ratio
- repairability score

### Pseudocode
```text
fn pattern_265(ctx):
    budget = ctx.latency_budget
    exact = cheap_exact(ctx)
    graph = bounded_graph(ctx, exact)
    sem = maybe_semantic(ctx, budget, graph)
    merged = dedup_merge(exact, graph, sem)
    ranked = rank_with_policy(ctx, merged)
    return package(ctx, ranked[:ctx.limit])
```
## 266. Episode Clustering pattern 26

### Objective
Optimize episode clustering under bounded latency and bounded noise.

### Preconditions
- namespace is validated
- policy context is loaded
- latency budget is available
- tier capabilities are known

### Heuristic
1. start with the cheapest exact signals
2. cap expansion by query class
3. only pay semantic cost when exact and graph signals are insufficient
4. preserve explainability and provenance
5. update reinforcement or decay after outcome feedback

### Failure risks
- candidate explosion
- stale data bias
- low-confidence amplification
- cross-session interference

### Instrumentation
- p50/p95/p99 latency
- candidate counts before and after pruning
- success contribution estimate
- cache hit ratio
- repairability score

### Pseudocode
```text
fn pattern_266(ctx):
    budget = ctx.latency_budget
    exact = cheap_exact(ctx)
    graph = bounded_graph(ctx, exact)
    sem = maybe_semantic(ctx, budget, graph)
    merged = dedup_merge(exact, graph, sem)
    ranked = rank_with_policy(ctx, merged)
    return package(ctx, ranked[:ctx.limit])
```
## 267. Episode Clustering pattern 27

### Objective
Optimize episode clustering under bounded latency and bounded noise.

### Preconditions
- namespace is validated
- policy context is loaded
- latency budget is available
- tier capabilities are known

### Heuristic
1. start with the cheapest exact signals
2. cap expansion by query class
3. only pay semantic cost when exact and graph signals are insufficient
4. preserve explainability and provenance
5. update reinforcement or decay after outcome feedback

### Failure risks
- candidate explosion
- stale data bias
- low-confidence amplification
- cross-session interference

### Instrumentation
- p50/p95/p99 latency
- candidate counts before and after pruning
- success contribution estimate
- cache hit ratio
- repairability score

### Pseudocode
```text
fn pattern_267(ctx):
    budget = ctx.latency_budget
    exact = cheap_exact(ctx)
    graph = bounded_graph(ctx, exact)
    sem = maybe_semantic(ctx, budget, graph)
    merged = dedup_merge(exact, graph, sem)
    ranked = rank_with_policy(ctx, merged)
    return package(ctx, ranked[:ctx.limit])
```
## 268. Episode Clustering pattern 28

### Objective
Optimize episode clustering under bounded latency and bounded noise.

### Preconditions
- namespace is validated
- policy context is loaded
- latency budget is available
- tier capabilities are known

### Heuristic
1. start with the cheapest exact signals
2. cap expansion by query class
3. only pay semantic cost when exact and graph signals are insufficient
4. preserve explainability and provenance
5. update reinforcement or decay after outcome feedback

### Failure risks
- candidate explosion
- stale data bias
- low-confidence amplification
- cross-session interference

### Instrumentation
- p50/p95/p99 latency
- candidate counts before and after pruning
- success contribution estimate
- cache hit ratio
- repairability score

### Pseudocode
```text
fn pattern_268(ctx):
    budget = ctx.latency_budget
    exact = cheap_exact(ctx)
    graph = bounded_graph(ctx, exact)
    sem = maybe_semantic(ctx, budget, graph)
    merged = dedup_merge(exact, graph, sem)
    ranked = rank_with_policy(ctx, merged)
    return package(ctx, ranked[:ctx.limit])
```
## 269. Episode Clustering pattern 29

### Objective
Optimize episode clustering under bounded latency and bounded noise.

### Preconditions
- namespace is validated
- policy context is loaded
- latency budget is available
- tier capabilities are known

### Heuristic
1. start with the cheapest exact signals
2. cap expansion by query class
3. only pay semantic cost when exact and graph signals are insufficient
4. preserve explainability and provenance
5. update reinforcement or decay after outcome feedback

### Failure risks
- candidate explosion
- stale data bias
- low-confidence amplification
- cross-session interference

### Instrumentation
- p50/p95/p99 latency
- candidate counts before and after pruning
- success contribution estimate
- cache hit ratio
- repairability score

### Pseudocode
```text
fn pattern_269(ctx):
    budget = ctx.latency_budget
    exact = cheap_exact(ctx)
    graph = bounded_graph(ctx, exact)
    sem = maybe_semantic(ctx, budget, graph)
    merged = dedup_merge(exact, graph, sem)
    ranked = rank_with_policy(ctx, merged)
    return package(ctx, ranked[:ctx.limit])
```
## 270. Episode Clustering pattern 30

### Objective
Optimize episode clustering under bounded latency and bounded noise.

### Preconditions
- namespace is validated
- policy context is loaded
- latency budget is available
- tier capabilities are known

### Heuristic
1. start with the cheapest exact signals
2. cap expansion by query class
3. only pay semantic cost when exact and graph signals are insufficient
4. preserve explainability and provenance
5. update reinforcement or decay after outcome feedback

### Failure risks
- candidate explosion
- stale data bias
- low-confidence amplification
- cross-session interference

### Instrumentation
- p50/p95/p99 latency
- candidate counts before and after pruning
- success contribution estimate
- cache hit ratio
- repairability score

### Pseudocode
```text
fn pattern_270(ctx):
    budget = ctx.latency_budget
    exact = cheap_exact(ctx)
    graph = bounded_graph(ctx, exact)
    sem = maybe_semantic(ctx, budget, graph)
    merged = dedup_merge(exact, graph, sem)
    ranked = rank_with_policy(ctx, merged)
    return package(ctx, ranked[:ctx.limit])
```
## 271. Fact Extraction pattern 1

### Objective
Optimize fact extraction under bounded latency and bounded noise.

### Preconditions
- namespace is validated
- policy context is loaded
- latency budget is available
- tier capabilities are known

### Heuristic
1. start with the cheapest exact signals
2. cap expansion by query class
3. only pay semantic cost when exact and graph signals are insufficient
4. preserve explainability and provenance
5. update reinforcement or decay after outcome feedback

### Failure risks
- candidate explosion
- stale data bias
- low-confidence amplification
- cross-session interference

### Instrumentation
- p50/p95/p99 latency
- candidate counts before and after pruning
- success contribution estimate
- cache hit ratio
- repairability score

### Pseudocode
```text
fn pattern_271(ctx):
    budget = ctx.latency_budget
    exact = cheap_exact(ctx)
    graph = bounded_graph(ctx, exact)
    sem = maybe_semantic(ctx, budget, graph)
    merged = dedup_merge(exact, graph, sem)
    ranked = rank_with_policy(ctx, merged)
    return package(ctx, ranked[:ctx.limit])
```
## 272. Fact Extraction pattern 2

### Objective
Optimize fact extraction under bounded latency and bounded noise.

### Preconditions
- namespace is validated
- policy context is loaded
- latency budget is available
- tier capabilities are known

### Heuristic
1. start with the cheapest exact signals
2. cap expansion by query class
3. only pay semantic cost when exact and graph signals are insufficient
4. preserve explainability and provenance
5. update reinforcement or decay after outcome feedback

### Failure risks
- candidate explosion
- stale data bias
- low-confidence amplification
- cross-session interference

### Instrumentation
- p50/p95/p99 latency
- candidate counts before and after pruning
- success contribution estimate
- cache hit ratio
- repairability score

### Pseudocode
```text
fn pattern_272(ctx):
    budget = ctx.latency_budget
    exact = cheap_exact(ctx)
    graph = bounded_graph(ctx, exact)
    sem = maybe_semantic(ctx, budget, graph)
    merged = dedup_merge(exact, graph, sem)
    ranked = rank_with_policy(ctx, merged)
    return package(ctx, ranked[:ctx.limit])
```
## 273. Fact Extraction pattern 3

### Objective
Optimize fact extraction under bounded latency and bounded noise.

### Preconditions
- namespace is validated
- policy context is loaded
- latency budget is available
- tier capabilities are known

### Heuristic
1. start with the cheapest exact signals
2. cap expansion by query class
3. only pay semantic cost when exact and graph signals are insufficient
4. preserve explainability and provenance
5. update reinforcement or decay after outcome feedback

### Failure risks
- candidate explosion
- stale data bias
- low-confidence amplification
- cross-session interference

### Instrumentation
- p50/p95/p99 latency
- candidate counts before and after pruning
- success contribution estimate
- cache hit ratio
- repairability score

### Pseudocode
```text
fn pattern_273(ctx):
    budget = ctx.latency_budget
    exact = cheap_exact(ctx)
    graph = bounded_graph(ctx, exact)
    sem = maybe_semantic(ctx, budget, graph)
    merged = dedup_merge(exact, graph, sem)
    ranked = rank_with_policy(ctx, merged)
    return package(ctx, ranked[:ctx.limit])
```
## 274. Fact Extraction pattern 4

### Objective
Optimize fact extraction under bounded latency and bounded noise.

### Preconditions
- namespace is validated
- policy context is loaded
- latency budget is available
- tier capabilities are known

### Heuristic
1. start with the cheapest exact signals
2. cap expansion by query class
3. only pay semantic cost when exact and graph signals are insufficient
4. preserve explainability and provenance
5. update reinforcement or decay after outcome feedback

### Failure risks
- candidate explosion
- stale data bias
- low-confidence amplification
- cross-session interference

### Instrumentation
- p50/p95/p99 latency
- candidate counts before and after pruning
- success contribution estimate
- cache hit ratio
- repairability score

### Pseudocode
```text
fn pattern_274(ctx):
    budget = ctx.latency_budget
    exact = cheap_exact(ctx)
    graph = bounded_graph(ctx, exact)
    sem = maybe_semantic(ctx, budget, graph)
    merged = dedup_merge(exact, graph, sem)
    ranked = rank_with_policy(ctx, merged)
    return package(ctx, ranked[:ctx.limit])
```
## 275. Fact Extraction pattern 5

### Objective
Optimize fact extraction under bounded latency and bounded noise.

### Preconditions
- namespace is validated
- policy context is loaded
- latency budget is available
- tier capabilities are known

### Heuristic
1. start with the cheapest exact signals
2. cap expansion by query class
3. only pay semantic cost when exact and graph signals are insufficient
4. preserve explainability and provenance
5. update reinforcement or decay after outcome feedback

### Failure risks
- candidate explosion
- stale data bias
- low-confidence amplification
- cross-session interference

### Instrumentation
- p50/p95/p99 latency
- candidate counts before and after pruning
- success contribution estimate
- cache hit ratio
- repairability score

### Pseudocode
```text
fn pattern_275(ctx):
    budget = ctx.latency_budget
    exact = cheap_exact(ctx)
    graph = bounded_graph(ctx, exact)
    sem = maybe_semantic(ctx, budget, graph)
    merged = dedup_merge(exact, graph, sem)
    ranked = rank_with_policy(ctx, merged)
    return package(ctx, ranked[:ctx.limit])
```
## 276. Fact Extraction pattern 6

### Objective
Optimize fact extraction under bounded latency and bounded noise.

### Preconditions
- namespace is validated
- policy context is loaded
- latency budget is available
- tier capabilities are known

### Heuristic
1. start with the cheapest exact signals
2. cap expansion by query class
3. only pay semantic cost when exact and graph signals are insufficient
4. preserve explainability and provenance
5. update reinforcement or decay after outcome feedback

### Failure risks
- candidate explosion
- stale data bias
- low-confidence amplification
- cross-session interference

### Instrumentation
- p50/p95/p99 latency
- candidate counts before and after pruning
- success contribution estimate
- cache hit ratio
- repairability score

### Pseudocode
```text
fn pattern_276(ctx):
    budget = ctx.latency_budget
    exact = cheap_exact(ctx)
    graph = bounded_graph(ctx, exact)
    sem = maybe_semantic(ctx, budget, graph)
    merged = dedup_merge(exact, graph, sem)
    ranked = rank_with_policy(ctx, merged)
    return package(ctx, ranked[:ctx.limit])
```
## 277. Fact Extraction pattern 7

### Objective
Optimize fact extraction under bounded latency and bounded noise.

### Preconditions
- namespace is validated
- policy context is loaded
- latency budget is available
- tier capabilities are known

### Heuristic
1. start with the cheapest exact signals
2. cap expansion by query class
3. only pay semantic cost when exact and graph signals are insufficient
4. preserve explainability and provenance
5. update reinforcement or decay after outcome feedback

### Failure risks
- candidate explosion
- stale data bias
- low-confidence amplification
- cross-session interference

### Instrumentation
- p50/p95/p99 latency
- candidate counts before and after pruning
- success contribution estimate
- cache hit ratio
- repairability score

### Pseudocode
```text
fn pattern_277(ctx):
    budget = ctx.latency_budget
    exact = cheap_exact(ctx)
    graph = bounded_graph(ctx, exact)
    sem = maybe_semantic(ctx, budget, graph)
    merged = dedup_merge(exact, graph, sem)
    ranked = rank_with_policy(ctx, merged)
    return package(ctx, ranked[:ctx.limit])
```
## 278. Fact Extraction pattern 8

### Objective
Optimize fact extraction under bounded latency and bounded noise.

### Preconditions
- namespace is validated
- policy context is loaded
- latency budget is available
- tier capabilities are known

### Heuristic
1. start with the cheapest exact signals
2. cap expansion by query class
3. only pay semantic cost when exact and graph signals are insufficient
4. preserve explainability and provenance
5. update reinforcement or decay after outcome feedback

### Failure risks
- candidate explosion
- stale data bias
- low-confidence amplification
- cross-session interference

### Instrumentation
- p50/p95/p99 latency
- candidate counts before and after pruning
- success contribution estimate
- cache hit ratio
- repairability score

### Pseudocode
```text
fn pattern_278(ctx):
    budget = ctx.latency_budget
    exact = cheap_exact(ctx)
    graph = bounded_graph(ctx, exact)
    sem = maybe_semantic(ctx, budget, graph)
    merged = dedup_merge(exact, graph, sem)
    ranked = rank_with_policy(ctx, merged)
    return package(ctx, ranked[:ctx.limit])
```
## 279. Fact Extraction pattern 9

### Objective
Optimize fact extraction under bounded latency and bounded noise.

### Preconditions
- namespace is validated
- policy context is loaded
- latency budget is available
- tier capabilities are known

### Heuristic
1. start with the cheapest exact signals
2. cap expansion by query class
3. only pay semantic cost when exact and graph signals are insufficient
4. preserve explainability and provenance
5. update reinforcement or decay after outcome feedback

### Failure risks
- candidate explosion
- stale data bias
- low-confidence amplification
- cross-session interference

### Instrumentation
- p50/p95/p99 latency
- candidate counts before and after pruning
- success contribution estimate
- cache hit ratio
- repairability score

### Pseudocode
```text
fn pattern_279(ctx):
    budget = ctx.latency_budget
    exact = cheap_exact(ctx)
    graph = bounded_graph(ctx, exact)
    sem = maybe_semantic(ctx, budget, graph)
    merged = dedup_merge(exact, graph, sem)
    ranked = rank_with_policy(ctx, merged)
    return package(ctx, ranked[:ctx.limit])
```
## 280. Fact Extraction pattern 10

### Objective
Optimize fact extraction under bounded latency and bounded noise.

### Preconditions
- namespace is validated
- policy context is loaded
- latency budget is available
- tier capabilities are known

### Heuristic
1. start with the cheapest exact signals
2. cap expansion by query class
3. only pay semantic cost when exact and graph signals are insufficient
4. preserve explainability and provenance
5. update reinforcement or decay after outcome feedback

### Failure risks
- candidate explosion
- stale data bias
- low-confidence amplification
- cross-session interference

### Instrumentation
- p50/p95/p99 latency
- candidate counts before and after pruning
- success contribution estimate
- cache hit ratio
- repairability score

### Pseudocode
```text
fn pattern_280(ctx):
    budget = ctx.latency_budget
    exact = cheap_exact(ctx)
    graph = bounded_graph(ctx, exact)
    sem = maybe_semantic(ctx, budget, graph)
    merged = dedup_merge(exact, graph, sem)
    ranked = rank_with_policy(ctx, merged)
    return package(ctx, ranked[:ctx.limit])
```
## 281. Fact Extraction pattern 11

### Objective
Optimize fact extraction under bounded latency and bounded noise.

### Preconditions
- namespace is validated
- policy context is loaded
- latency budget is available
- tier capabilities are known

### Heuristic
1. start with the cheapest exact signals
2. cap expansion by query class
3. only pay semantic cost when exact and graph signals are insufficient
4. preserve explainability and provenance
5. update reinforcement or decay after outcome feedback

### Failure risks
- candidate explosion
- stale data bias
- low-confidence amplification
- cross-session interference

### Instrumentation
- p50/p95/p99 latency
- candidate counts before and after pruning
- success contribution estimate
- cache hit ratio
- repairability score

### Pseudocode
```text
fn pattern_281(ctx):
    budget = ctx.latency_budget
    exact = cheap_exact(ctx)
    graph = bounded_graph(ctx, exact)
    sem = maybe_semantic(ctx, budget, graph)
    merged = dedup_merge(exact, graph, sem)
    ranked = rank_with_policy(ctx, merged)
    return package(ctx, ranked[:ctx.limit])
```
## 282. Fact Extraction pattern 12

### Objective
Optimize fact extraction under bounded latency and bounded noise.

### Preconditions
- namespace is validated
- policy context is loaded
- latency budget is available
- tier capabilities are known

### Heuristic
1. start with the cheapest exact signals
2. cap expansion by query class
3. only pay semantic cost when exact and graph signals are insufficient
4. preserve explainability and provenance
5. update reinforcement or decay after outcome feedback

### Failure risks
- candidate explosion
- stale data bias
- low-confidence amplification
- cross-session interference

### Instrumentation
- p50/p95/p99 latency
- candidate counts before and after pruning
- success contribution estimate
- cache hit ratio
- repairability score

### Pseudocode
```text
fn pattern_282(ctx):
    budget = ctx.latency_budget
    exact = cheap_exact(ctx)
    graph = bounded_graph(ctx, exact)
    sem = maybe_semantic(ctx, budget, graph)
    merged = dedup_merge(exact, graph, sem)
    ranked = rank_with_policy(ctx, merged)
    return package(ctx, ranked[:ctx.limit])
```
## 283. Fact Extraction pattern 13

### Objective
Optimize fact extraction under bounded latency and bounded noise.

### Preconditions
- namespace is validated
- policy context is loaded
- latency budget is available
- tier capabilities are known

### Heuristic
1. start with the cheapest exact signals
2. cap expansion by query class
3. only pay semantic cost when exact and graph signals are insufficient
4. preserve explainability and provenance
5. update reinforcement or decay after outcome feedback

### Failure risks
- candidate explosion
- stale data bias
- low-confidence amplification
- cross-session interference

### Instrumentation
- p50/p95/p99 latency
- candidate counts before and after pruning
- success contribution estimate
- cache hit ratio
- repairability score

### Pseudocode
```text
fn pattern_283(ctx):
    budget = ctx.latency_budget
    exact = cheap_exact(ctx)
    graph = bounded_graph(ctx, exact)
    sem = maybe_semantic(ctx, budget, graph)
    merged = dedup_merge(exact, graph, sem)
    ranked = rank_with_policy(ctx, merged)
    return package(ctx, ranked[:ctx.limit])
```
## 284. Fact Extraction pattern 14

### Objective
Optimize fact extraction under bounded latency and bounded noise.

### Preconditions
- namespace is validated
- policy context is loaded
- latency budget is available
- tier capabilities are known

### Heuristic
1. start with the cheapest exact signals
2. cap expansion by query class
3. only pay semantic cost when exact and graph signals are insufficient
4. preserve explainability and provenance
5. update reinforcement or decay after outcome feedback

### Failure risks
- candidate explosion
- stale data bias
- low-confidence amplification
- cross-session interference

### Instrumentation
- p50/p95/p99 latency
- candidate counts before and after pruning
- success contribution estimate
- cache hit ratio
- repairability score

### Pseudocode
```text
fn pattern_284(ctx):
    budget = ctx.latency_budget
    exact = cheap_exact(ctx)
    graph = bounded_graph(ctx, exact)
    sem = maybe_semantic(ctx, budget, graph)
    merged = dedup_merge(exact, graph, sem)
    ranked = rank_with_policy(ctx, merged)
    return package(ctx, ranked[:ctx.limit])
```
## 285. Fact Extraction pattern 15

### Objective
Optimize fact extraction under bounded latency and bounded noise.

### Preconditions
- namespace is validated
- policy context is loaded
- latency budget is available
- tier capabilities are known

### Heuristic
1. start with the cheapest exact signals
2. cap expansion by query class
3. only pay semantic cost when exact and graph signals are insufficient
4. preserve explainability and provenance
5. update reinforcement or decay after outcome feedback

### Failure risks
- candidate explosion
- stale data bias
- low-confidence amplification
- cross-session interference

### Instrumentation
- p50/p95/p99 latency
- candidate counts before and after pruning
- success contribution estimate
- cache hit ratio
- repairability score

### Pseudocode
```text
fn pattern_285(ctx):
    budget = ctx.latency_budget
    exact = cheap_exact(ctx)
    graph = bounded_graph(ctx, exact)
    sem = maybe_semantic(ctx, budget, graph)
    merged = dedup_merge(exact, graph, sem)
    ranked = rank_with_policy(ctx, merged)
    return package(ctx, ranked[:ctx.limit])
```
## 286. Fact Extraction pattern 16

### Objective
Optimize fact extraction under bounded latency and bounded noise.

### Preconditions
- namespace is validated
- policy context is loaded
- latency budget is available
- tier capabilities are known

### Heuristic
1. start with the cheapest exact signals
2. cap expansion by query class
3. only pay semantic cost when exact and graph signals are insufficient
4. preserve explainability and provenance
5. update reinforcement or decay after outcome feedback

### Failure risks
- candidate explosion
- stale data bias
- low-confidence amplification
- cross-session interference

### Instrumentation
- p50/p95/p99 latency
- candidate counts before and after pruning
- success contribution estimate
- cache hit ratio
- repairability score

### Pseudocode
```text
fn pattern_286(ctx):
    budget = ctx.latency_budget
    exact = cheap_exact(ctx)
    graph = bounded_graph(ctx, exact)
    sem = maybe_semantic(ctx, budget, graph)
    merged = dedup_merge(exact, graph, sem)
    ranked = rank_with_policy(ctx, merged)
    return package(ctx, ranked[:ctx.limit])
```
## 287. Fact Extraction pattern 17

### Objective
Optimize fact extraction under bounded latency and bounded noise.

### Preconditions
- namespace is validated
- policy context is loaded
- latency budget is available
- tier capabilities are known

### Heuristic
1. start with the cheapest exact signals
2. cap expansion by query class
3. only pay semantic cost when exact and graph signals are insufficient
4. preserve explainability and provenance
5. update reinforcement or decay after outcome feedback

### Failure risks
- candidate explosion
- stale data bias
- low-confidence amplification
- cross-session interference

### Instrumentation
- p50/p95/p99 latency
- candidate counts before and after pruning
- success contribution estimate
- cache hit ratio
- repairability score

### Pseudocode
```text
fn pattern_287(ctx):
    budget = ctx.latency_budget
    exact = cheap_exact(ctx)
    graph = bounded_graph(ctx, exact)
    sem = maybe_semantic(ctx, budget, graph)
    merged = dedup_merge(exact, graph, sem)
    ranked = rank_with_policy(ctx, merged)
    return package(ctx, ranked[:ctx.limit])
```
## 288. Fact Extraction pattern 18

### Objective
Optimize fact extraction under bounded latency and bounded noise.

### Preconditions
- namespace is validated
- policy context is loaded
- latency budget is available
- tier capabilities are known

### Heuristic
1. start with the cheapest exact signals
2. cap expansion by query class
3. only pay semantic cost when exact and graph signals are insufficient
4. preserve explainability and provenance
5. update reinforcement or decay after outcome feedback

### Failure risks
- candidate explosion
- stale data bias
- low-confidence amplification
- cross-session interference

### Instrumentation
- p50/p95/p99 latency
- candidate counts before and after pruning
- success contribution estimate
- cache hit ratio
- repairability score

### Pseudocode
```text
fn pattern_288(ctx):
    budget = ctx.latency_budget
    exact = cheap_exact(ctx)
    graph = bounded_graph(ctx, exact)
    sem = maybe_semantic(ctx, budget, graph)
    merged = dedup_merge(exact, graph, sem)
    ranked = rank_with_policy(ctx, merged)
    return package(ctx, ranked[:ctx.limit])
```
## 289. Fact Extraction pattern 19

### Objective
Optimize fact extraction under bounded latency and bounded noise.

### Preconditions
- namespace is validated
- policy context is loaded
- latency budget is available
- tier capabilities are known

### Heuristic
1. start with the cheapest exact signals
2. cap expansion by query class
3. only pay semantic cost when exact and graph signals are insufficient
4. preserve explainability and provenance
5. update reinforcement or decay after outcome feedback

### Failure risks
- candidate explosion
- stale data bias
- low-confidence amplification
- cross-session interference

### Instrumentation
- p50/p95/p99 latency
- candidate counts before and after pruning
- success contribution estimate
- cache hit ratio
- repairability score

### Pseudocode
```text
fn pattern_289(ctx):
    budget = ctx.latency_budget
    exact = cheap_exact(ctx)
    graph = bounded_graph(ctx, exact)
    sem = maybe_semantic(ctx, budget, graph)
    merged = dedup_merge(exact, graph, sem)
    ranked = rank_with_policy(ctx, merged)
    return package(ctx, ranked[:ctx.limit])
```
## 290. Fact Extraction pattern 20

### Objective
Optimize fact extraction under bounded latency and bounded noise.

### Preconditions
- namespace is validated
- policy context is loaded
- latency budget is available
- tier capabilities are known

### Heuristic
1. start with the cheapest exact signals
2. cap expansion by query class
3. only pay semantic cost when exact and graph signals are insufficient
4. preserve explainability and provenance
5. update reinforcement or decay after outcome feedback

### Failure risks
- candidate explosion
- stale data bias
- low-confidence amplification
- cross-session interference

### Instrumentation
- p50/p95/p99 latency
- candidate counts before and after pruning
- success contribution estimate
- cache hit ratio
- repairability score

### Pseudocode
```text
fn pattern_290(ctx):
    budget = ctx.latency_budget
    exact = cheap_exact(ctx)
    graph = bounded_graph(ctx, exact)
    sem = maybe_semantic(ctx, budget, graph)
    merged = dedup_merge(exact, graph, sem)
    ranked = rank_with_policy(ctx, merged)
    return package(ctx, ranked[:ctx.limit])
```
## 291. Fact Extraction pattern 21

### Objective
Optimize fact extraction under bounded latency and bounded noise.

### Preconditions
- namespace is validated
- policy context is loaded
- latency budget is available
- tier capabilities are known

### Heuristic
1. start with the cheapest exact signals
2. cap expansion by query class
3. only pay semantic cost when exact and graph signals are insufficient
4. preserve explainability and provenance
5. update reinforcement or decay after outcome feedback

### Failure risks
- candidate explosion
- stale data bias
- low-confidence amplification
- cross-session interference

### Instrumentation
- p50/p95/p99 latency
- candidate counts before and after pruning
- success contribution estimate
- cache hit ratio
- repairability score

### Pseudocode
```text
fn pattern_291(ctx):
    budget = ctx.latency_budget
    exact = cheap_exact(ctx)
    graph = bounded_graph(ctx, exact)
    sem = maybe_semantic(ctx, budget, graph)
    merged = dedup_merge(exact, graph, sem)
    ranked = rank_with_policy(ctx, merged)
    return package(ctx, ranked[:ctx.limit])
```
## 292. Fact Extraction pattern 22

### Objective
Optimize fact extraction under bounded latency and bounded noise.

### Preconditions
- namespace is validated
- policy context is loaded
- latency budget is available
- tier capabilities are known

### Heuristic
1. start with the cheapest exact signals
2. cap expansion by query class
3. only pay semantic cost when exact and graph signals are insufficient
4. preserve explainability and provenance
5. update reinforcement or decay after outcome feedback

### Failure risks
- candidate explosion
- stale data bias
- low-confidence amplification
- cross-session interference

### Instrumentation
- p50/p95/p99 latency
- candidate counts before and after pruning
- success contribution estimate
- cache hit ratio
- repairability score

### Pseudocode
```text
fn pattern_292(ctx):
    budget = ctx.latency_budget
    exact = cheap_exact(ctx)
    graph = bounded_graph(ctx, exact)
    sem = maybe_semantic(ctx, budget, graph)
    merged = dedup_merge(exact, graph, sem)
    ranked = rank_with_policy(ctx, merged)
    return package(ctx, ranked[:ctx.limit])
```
## 293. Fact Extraction pattern 23

### Objective
Optimize fact extraction under bounded latency and bounded noise.

### Preconditions
- namespace is validated
- policy context is loaded
- latency budget is available
- tier capabilities are known

### Heuristic
1. start with the cheapest exact signals
2. cap expansion by query class
3. only pay semantic cost when exact and graph signals are insufficient
4. preserve explainability and provenance
5. update reinforcement or decay after outcome feedback

### Failure risks
- candidate explosion
- stale data bias
- low-confidence amplification
- cross-session interference

### Instrumentation
- p50/p95/p99 latency
- candidate counts before and after pruning
- success contribution estimate
- cache hit ratio
- repairability score

### Pseudocode
```text
fn pattern_293(ctx):
    budget = ctx.latency_budget
    exact = cheap_exact(ctx)
    graph = bounded_graph(ctx, exact)
    sem = maybe_semantic(ctx, budget, graph)
    merged = dedup_merge(exact, graph, sem)
    ranked = rank_with_policy(ctx, merged)
    return package(ctx, ranked[:ctx.limit])
```
## 294. Fact Extraction pattern 24

### Objective
Optimize fact extraction under bounded latency and bounded noise.

### Preconditions
- namespace is validated
- policy context is loaded
- latency budget is available
- tier capabilities are known

### Heuristic
1. start with the cheapest exact signals
2. cap expansion by query class
3. only pay semantic cost when exact and graph signals are insufficient
4. preserve explainability and provenance
5. update reinforcement or decay after outcome feedback

### Failure risks
- candidate explosion
- stale data bias
- low-confidence amplification
- cross-session interference

### Instrumentation
- p50/p95/p99 latency
- candidate counts before and after pruning
- success contribution estimate
- cache hit ratio
- repairability score

### Pseudocode
```text
fn pattern_294(ctx):
    budget = ctx.latency_budget
    exact = cheap_exact(ctx)
    graph = bounded_graph(ctx, exact)
    sem = maybe_semantic(ctx, budget, graph)
    merged = dedup_merge(exact, graph, sem)
    ranked = rank_with_policy(ctx, merged)
    return package(ctx, ranked[:ctx.limit])
```
## 295. Fact Extraction pattern 25

### Objective
Optimize fact extraction under bounded latency and bounded noise.

### Preconditions
- namespace is validated
- policy context is loaded
- latency budget is available
- tier capabilities are known

### Heuristic
1. start with the cheapest exact signals
2. cap expansion by query class
3. only pay semantic cost when exact and graph signals are insufficient
4. preserve explainability and provenance
5. update reinforcement or decay after outcome feedback

### Failure risks
- candidate explosion
- stale data bias
- low-confidence amplification
- cross-session interference

### Instrumentation
- p50/p95/p99 latency
- candidate counts before and after pruning
- success contribution estimate
- cache hit ratio
- repairability score

### Pseudocode
```text
fn pattern_295(ctx):
    budget = ctx.latency_budget
    exact = cheap_exact(ctx)
    graph = bounded_graph(ctx, exact)
    sem = maybe_semantic(ctx, budget, graph)
    merged = dedup_merge(exact, graph, sem)
    ranked = rank_with_policy(ctx, merged)
    return package(ctx, ranked[:ctx.limit])
```
## 296. Fact Extraction pattern 26

### Objective
Optimize fact extraction under bounded latency and bounded noise.

### Preconditions
- namespace is validated
- policy context is loaded
- latency budget is available
- tier capabilities are known

### Heuristic
1. start with the cheapest exact signals
2. cap expansion by query class
3. only pay semantic cost when exact and graph signals are insufficient
4. preserve explainability and provenance
5. update reinforcement or decay after outcome feedback

### Failure risks
- candidate explosion
- stale data bias
- low-confidence amplification
- cross-session interference

### Instrumentation
- p50/p95/p99 latency
- candidate counts before and after pruning
- success contribution estimate
- cache hit ratio
- repairability score

### Pseudocode
```text
fn pattern_296(ctx):
    budget = ctx.latency_budget
    exact = cheap_exact(ctx)
    graph = bounded_graph(ctx, exact)
    sem = maybe_semantic(ctx, budget, graph)
    merged = dedup_merge(exact, graph, sem)
    ranked = rank_with_policy(ctx, merged)
    return package(ctx, ranked[:ctx.limit])
```
## 297. Fact Extraction pattern 27

### Objective
Optimize fact extraction under bounded latency and bounded noise.

### Preconditions
- namespace is validated
- policy context is loaded
- latency budget is available
- tier capabilities are known

### Heuristic
1. start with the cheapest exact signals
2. cap expansion by query class
3. only pay semantic cost when exact and graph signals are insufficient
4. preserve explainability and provenance
5. update reinforcement or decay after outcome feedback

### Failure risks
- candidate explosion
- stale data bias
- low-confidence amplification
- cross-session interference

### Instrumentation
- p50/p95/p99 latency
- candidate counts before and after pruning
- success contribution estimate
- cache hit ratio
- repairability score

### Pseudocode
```text
fn pattern_297(ctx):
    budget = ctx.latency_budget
    exact = cheap_exact(ctx)
    graph = bounded_graph(ctx, exact)
    sem = maybe_semantic(ctx, budget, graph)
    merged = dedup_merge(exact, graph, sem)
    ranked = rank_with_policy(ctx, merged)
    return package(ctx, ranked[:ctx.limit])
```
## 298. Fact Extraction pattern 28

### Objective
Optimize fact extraction under bounded latency and bounded noise.

### Preconditions
- namespace is validated
- policy context is loaded
- latency budget is available
- tier capabilities are known

### Heuristic
1. start with the cheapest exact signals
2. cap expansion by query class
3. only pay semantic cost when exact and graph signals are insufficient
4. preserve explainability and provenance
5. update reinforcement or decay after outcome feedback

### Failure risks
- candidate explosion
- stale data bias
- low-confidence amplification
- cross-session interference

### Instrumentation
- p50/p95/p99 latency
- candidate counts before and after pruning
- success contribution estimate
- cache hit ratio
- repairability score

### Pseudocode
```text
fn pattern_298(ctx):
    budget = ctx.latency_budget
    exact = cheap_exact(ctx)
    graph = bounded_graph(ctx, exact)
    sem = maybe_semantic(ctx, budget, graph)
    merged = dedup_merge(exact, graph, sem)
    ranked = rank_with_policy(ctx, merged)
    return package(ctx, ranked[:ctx.limit])
```
## 299. Fact Extraction pattern 29

### Objective
Optimize fact extraction under bounded latency and bounded noise.

### Preconditions
- namespace is validated
- policy context is loaded
- latency budget is available
- tier capabilities are known

### Heuristic
1. start with the cheapest exact signals
2. cap expansion by query class
3. only pay semantic cost when exact and graph signals are insufficient
4. preserve explainability and provenance
5. update reinforcement or decay after outcome feedback

### Failure risks
- candidate explosion
- stale data bias
- low-confidence amplification
- cross-session interference

### Instrumentation
- p50/p95/p99 latency
- candidate counts before and after pruning
- success contribution estimate
- cache hit ratio
- repairability score

### Pseudocode
```text
fn pattern_299(ctx):
    budget = ctx.latency_budget
    exact = cheap_exact(ctx)
    graph = bounded_graph(ctx, exact)
    sem = maybe_semantic(ctx, budget, graph)
    merged = dedup_merge(exact, graph, sem)
    ranked = rank_with_policy(ctx, merged)
    return package(ctx, ranked[:ctx.limit])
```
## 300. Fact Extraction pattern 30

### Objective
Optimize fact extraction under bounded latency and bounded noise.

### Preconditions
- namespace is validated
- policy context is loaded
- latency budget is available
- tier capabilities are known

### Heuristic
1. start with the cheapest exact signals
2. cap expansion by query class
3. only pay semantic cost when exact and graph signals are insufficient
4. preserve explainability and provenance
5. update reinforcement or decay after outcome feedback

### Failure risks
- candidate explosion
- stale data bias
- low-confidence amplification
- cross-session interference

### Instrumentation
- p50/p95/p99 latency
- candidate counts before and after pruning
- success contribution estimate
- cache hit ratio
- repairability score

### Pseudocode
```text
fn pattern_300(ctx):
    budget = ctx.latency_budget
    exact = cheap_exact(ctx)
    graph = bounded_graph(ctx, exact)
    sem = maybe_semantic(ctx, budget, graph)
    merged = dedup_merge(exact, graph, sem)
    ranked = rank_with_policy(ctx, merged)
    return package(ctx, ranked[:ctx.limit])
```
## 301. Skill Extraction pattern 1

### Objective
Optimize skill extraction under bounded latency and bounded noise.

### Preconditions
- namespace is validated
- policy context is loaded
- latency budget is available
- tier capabilities are known

### Heuristic
1. start with the cheapest exact signals
2. cap expansion by query class
3. only pay semantic cost when exact and graph signals are insufficient
4. preserve explainability and provenance
5. update reinforcement or decay after outcome feedback

### Failure risks
- candidate explosion
- stale data bias
- low-confidence amplification
- cross-session interference

### Instrumentation
- p50/p95/p99 latency
- candidate counts before and after pruning
- success contribution estimate
- cache hit ratio
- repairability score

### Pseudocode
```text
fn pattern_301(ctx):
    budget = ctx.latency_budget
    exact = cheap_exact(ctx)
    graph = bounded_graph(ctx, exact)
    sem = maybe_semantic(ctx, budget, graph)
    merged = dedup_merge(exact, graph, sem)
    ranked = rank_with_policy(ctx, merged)
    return package(ctx, ranked[:ctx.limit])
```
## 302. Skill Extraction pattern 2

### Objective
Optimize skill extraction under bounded latency and bounded noise.

### Preconditions
- namespace is validated
- policy context is loaded
- latency budget is available
- tier capabilities are known

### Heuristic
1. start with the cheapest exact signals
2. cap expansion by query class
3. only pay semantic cost when exact and graph signals are insufficient
4. preserve explainability and provenance
5. update reinforcement or decay after outcome feedback

### Failure risks
- candidate explosion
- stale data bias
- low-confidence amplification
- cross-session interference

### Instrumentation
- p50/p95/p99 latency
- candidate counts before and after pruning
- success contribution estimate
- cache hit ratio
- repairability score

### Pseudocode
```text
fn pattern_302(ctx):
    budget = ctx.latency_budget
    exact = cheap_exact(ctx)
    graph = bounded_graph(ctx, exact)
    sem = maybe_semantic(ctx, budget, graph)
    merged = dedup_merge(exact, graph, sem)
    ranked = rank_with_policy(ctx, merged)
    return package(ctx, ranked[:ctx.limit])
```
## 303. Skill Extraction pattern 3

### Objective
Optimize skill extraction under bounded latency and bounded noise.

### Preconditions
- namespace is validated
- policy context is loaded
- latency budget is available
- tier capabilities are known

### Heuristic
1. start with the cheapest exact signals
2. cap expansion by query class
3. only pay semantic cost when exact and graph signals are insufficient
4. preserve explainability and provenance
5. update reinforcement or decay after outcome feedback

### Failure risks
- candidate explosion
- stale data bias
- low-confidence amplification
- cross-session interference

### Instrumentation
- p50/p95/p99 latency
- candidate counts before and after pruning
- success contribution estimate
- cache hit ratio
- repairability score

### Pseudocode
```text
fn pattern_303(ctx):
    budget = ctx.latency_budget
    exact = cheap_exact(ctx)
    graph = bounded_graph(ctx, exact)
    sem = maybe_semantic(ctx, budget, graph)
    merged = dedup_merge(exact, graph, sem)
    ranked = rank_with_policy(ctx, merged)
    return package(ctx, ranked[:ctx.limit])
```
## 304. Skill Extraction pattern 4

### Objective
Optimize skill extraction under bounded latency and bounded noise.

### Preconditions
- namespace is validated
- policy context is loaded
- latency budget is available
- tier capabilities are known

### Heuristic
1. start with the cheapest exact signals
2. cap expansion by query class
3. only pay semantic cost when exact and graph signals are insufficient
4. preserve explainability and provenance
5. update reinforcement or decay after outcome feedback

### Failure risks
- candidate explosion
- stale data bias
- low-confidence amplification
- cross-session interference

### Instrumentation
- p50/p95/p99 latency
- candidate counts before and after pruning
- success contribution estimate
- cache hit ratio
- repairability score

### Pseudocode
```text
fn pattern_304(ctx):
    budget = ctx.latency_budget
    exact = cheap_exact(ctx)
    graph = bounded_graph(ctx, exact)
    sem = maybe_semantic(ctx, budget, graph)
    merged = dedup_merge(exact, graph, sem)
    ranked = rank_with_policy(ctx, merged)
    return package(ctx, ranked[:ctx.limit])
```
## 305. Skill Extraction pattern 5

### Objective
Optimize skill extraction under bounded latency and bounded noise.

### Preconditions
- namespace is validated
- policy context is loaded
- latency budget is available
- tier capabilities are known

### Heuristic
1. start with the cheapest exact signals
2. cap expansion by query class
3. only pay semantic cost when exact and graph signals are insufficient
4. preserve explainability and provenance
5. update reinforcement or decay after outcome feedback

### Failure risks
- candidate explosion
- stale data bias
- low-confidence amplification
- cross-session interference

### Instrumentation
- p50/p95/p99 latency
- candidate counts before and after pruning
- success contribution estimate
- cache hit ratio
- repairability score

### Pseudocode
```text
fn pattern_305(ctx):
    budget = ctx.latency_budget
    exact = cheap_exact(ctx)
    graph = bounded_graph(ctx, exact)
    sem = maybe_semantic(ctx, budget, graph)
    merged = dedup_merge(exact, graph, sem)
    ranked = rank_with_policy(ctx, merged)
    return package(ctx, ranked[:ctx.limit])
```
## 306. Skill Extraction pattern 6

### Objective
Optimize skill extraction under bounded latency and bounded noise.

### Preconditions
- namespace is validated
- policy context is loaded
- latency budget is available
- tier capabilities are known

### Heuristic
1. start with the cheapest exact signals
2. cap expansion by query class
3. only pay semantic cost when exact and graph signals are insufficient
4. preserve explainability and provenance
5. update reinforcement or decay after outcome feedback

### Failure risks
- candidate explosion
- stale data bias
- low-confidence amplification
- cross-session interference

### Instrumentation
- p50/p95/p99 latency
- candidate counts before and after pruning
- success contribution estimate
- cache hit ratio
- repairability score

### Pseudocode
```text
fn pattern_306(ctx):
    budget = ctx.latency_budget
    exact = cheap_exact(ctx)
    graph = bounded_graph(ctx, exact)
    sem = maybe_semantic(ctx, budget, graph)
    merged = dedup_merge(exact, graph, sem)
    ranked = rank_with_policy(ctx, merged)
    return package(ctx, ranked[:ctx.limit])
```
## 307. Skill Extraction pattern 7

### Objective
Optimize skill extraction under bounded latency and bounded noise.

### Preconditions
- namespace is validated
- policy context is loaded
- latency budget is available
- tier capabilities are known

### Heuristic
1. start with the cheapest exact signals
2. cap expansion by query class
3. only pay semantic cost when exact and graph signals are insufficient
4. preserve explainability and provenance
5. update reinforcement or decay after outcome feedback

### Failure risks
- candidate explosion
- stale data bias
- low-confidence amplification
- cross-session interference

### Instrumentation
- p50/p95/p99 latency
- candidate counts before and after pruning
- success contribution estimate
- cache hit ratio
- repairability score

### Pseudocode
```text
fn pattern_307(ctx):
    budget = ctx.latency_budget
    exact = cheap_exact(ctx)
    graph = bounded_graph(ctx, exact)
    sem = maybe_semantic(ctx, budget, graph)
    merged = dedup_merge(exact, graph, sem)
    ranked = rank_with_policy(ctx, merged)
    return package(ctx, ranked[:ctx.limit])
```
## 308. Skill Extraction pattern 8

### Objective
Optimize skill extraction under bounded latency and bounded noise.

### Preconditions
- namespace is validated
- policy context is loaded
- latency budget is available
- tier capabilities are known

### Heuristic
1. start with the cheapest exact signals
2. cap expansion by query class
3. only pay semantic cost when exact and graph signals are insufficient
4. preserve explainability and provenance
5. update reinforcement or decay after outcome feedback

### Failure risks
- candidate explosion
- stale data bias
- low-confidence amplification
- cross-session interference

### Instrumentation
- p50/p95/p99 latency
- candidate counts before and after pruning
- success contribution estimate
- cache hit ratio
- repairability score

### Pseudocode
```text
fn pattern_308(ctx):
    budget = ctx.latency_budget
    exact = cheap_exact(ctx)
    graph = bounded_graph(ctx, exact)
    sem = maybe_semantic(ctx, budget, graph)
    merged = dedup_merge(exact, graph, sem)
    ranked = rank_with_policy(ctx, merged)
    return package(ctx, ranked[:ctx.limit])
```
## 309. Skill Extraction pattern 9

### Objective
Optimize skill extraction under bounded latency and bounded noise.

### Preconditions
- namespace is validated
- policy context is loaded
- latency budget is available
- tier capabilities are known

### Heuristic
1. start with the cheapest exact signals
2. cap expansion by query class
3. only pay semantic cost when exact and graph signals are insufficient
4. preserve explainability and provenance
5. update reinforcement or decay after outcome feedback

### Failure risks
- candidate explosion
- stale data bias
- low-confidence amplification
- cross-session interference

### Instrumentation
- p50/p95/p99 latency
- candidate counts before and after pruning
- success contribution estimate
- cache hit ratio
- repairability score

### Pseudocode
```text
fn pattern_309(ctx):
    budget = ctx.latency_budget
    exact = cheap_exact(ctx)
    graph = bounded_graph(ctx, exact)
    sem = maybe_semantic(ctx, budget, graph)
    merged = dedup_merge(exact, graph, sem)
    ranked = rank_with_policy(ctx, merged)
    return package(ctx, ranked[:ctx.limit])
```
## 310. Skill Extraction pattern 10

### Objective
Optimize skill extraction under bounded latency and bounded noise.

### Preconditions
- namespace is validated
- policy context is loaded
- latency budget is available
- tier capabilities are known

### Heuristic
1. start with the cheapest exact signals
2. cap expansion by query class
3. only pay semantic cost when exact and graph signals are insufficient
4. preserve explainability and provenance
5. update reinforcement or decay after outcome feedback

### Failure risks
- candidate explosion
- stale data bias
- low-confidence amplification
- cross-session interference

### Instrumentation
- p50/p95/p99 latency
- candidate counts before and after pruning
- success contribution estimate
- cache hit ratio
- repairability score

### Pseudocode
```text
fn pattern_310(ctx):
    budget = ctx.latency_budget
    exact = cheap_exact(ctx)
    graph = bounded_graph(ctx, exact)
    sem = maybe_semantic(ctx, budget, graph)
    merged = dedup_merge(exact, graph, sem)
    ranked = rank_with_policy(ctx, merged)
    return package(ctx, ranked[:ctx.limit])
```
## 311. Skill Extraction pattern 11

### Objective
Optimize skill extraction under bounded latency and bounded noise.

### Preconditions
- namespace is validated
- policy context is loaded
- latency budget is available
- tier capabilities are known

### Heuristic
1. start with the cheapest exact signals
2. cap expansion by query class
3. only pay semantic cost when exact and graph signals are insufficient
4. preserve explainability and provenance
5. update reinforcement or decay after outcome feedback

### Failure risks
- candidate explosion
- stale data bias
- low-confidence amplification
- cross-session interference

### Instrumentation
- p50/p95/p99 latency
- candidate counts before and after pruning
- success contribution estimate
- cache hit ratio
- repairability score

### Pseudocode
```text
fn pattern_311(ctx):
    budget = ctx.latency_budget
    exact = cheap_exact(ctx)
    graph = bounded_graph(ctx, exact)
    sem = maybe_semantic(ctx, budget, graph)
    merged = dedup_merge(exact, graph, sem)
    ranked = rank_with_policy(ctx, merged)
    return package(ctx, ranked[:ctx.limit])
```
## 312. Skill Extraction pattern 12

### Objective
Optimize skill extraction under bounded latency and bounded noise.

### Preconditions
- namespace is validated
- policy context is loaded
- latency budget is available
- tier capabilities are known

### Heuristic
1. start with the cheapest exact signals
2. cap expansion by query class
3. only pay semantic cost when exact and graph signals are insufficient
4. preserve explainability and provenance
5. update reinforcement or decay after outcome feedback

### Failure risks
- candidate explosion
- stale data bias
- low-confidence amplification
- cross-session interference

### Instrumentation
- p50/p95/p99 latency
- candidate counts before and after pruning
- success contribution estimate
- cache hit ratio
- repairability score

### Pseudocode
```text
fn pattern_312(ctx):
    budget = ctx.latency_budget
    exact = cheap_exact(ctx)
    graph = bounded_graph(ctx, exact)
    sem = maybe_semantic(ctx, budget, graph)
    merged = dedup_merge(exact, graph, sem)
    ranked = rank_with_policy(ctx, merged)
    return package(ctx, ranked[:ctx.limit])
```
## 313. Skill Extraction pattern 13

### Objective
Optimize skill extraction under bounded latency and bounded noise.

### Preconditions
- namespace is validated
- policy context is loaded
- latency budget is available
- tier capabilities are known

### Heuristic
1. start with the cheapest exact signals
2. cap expansion by query class
3. only pay semantic cost when exact and graph signals are insufficient
4. preserve explainability and provenance
5. update reinforcement or decay after outcome feedback

### Failure risks
- candidate explosion
- stale data bias
- low-confidence amplification
- cross-session interference

### Instrumentation
- p50/p95/p99 latency
- candidate counts before and after pruning
- success contribution estimate
- cache hit ratio
- repairability score

### Pseudocode
```text
fn pattern_313(ctx):
    budget = ctx.latency_budget
    exact = cheap_exact(ctx)
    graph = bounded_graph(ctx, exact)
    sem = maybe_semantic(ctx, budget, graph)
    merged = dedup_merge(exact, graph, sem)
    ranked = rank_with_policy(ctx, merged)
    return package(ctx, ranked[:ctx.limit])
```
## 314. Skill Extraction pattern 14

### Objective
Optimize skill extraction under bounded latency and bounded noise.

### Preconditions
- namespace is validated
- policy context is loaded
- latency budget is available
- tier capabilities are known

### Heuristic
1. start with the cheapest exact signals
2. cap expansion by query class
3. only pay semantic cost when exact and graph signals are insufficient
4. preserve explainability and provenance
5. update reinforcement or decay after outcome feedback

### Failure risks
- candidate explosion
- stale data bias
- low-confidence amplification
- cross-session interference

### Instrumentation
- p50/p95/p99 latency
- candidate counts before and after pruning
- success contribution estimate
- cache hit ratio
- repairability score

### Pseudocode
```text
fn pattern_314(ctx):
    budget = ctx.latency_budget
    exact = cheap_exact(ctx)
    graph = bounded_graph(ctx, exact)
    sem = maybe_semantic(ctx, budget, graph)
    merged = dedup_merge(exact, graph, sem)
    ranked = rank_with_policy(ctx, merged)
    return package(ctx, ranked[:ctx.limit])
```
## 315. Skill Extraction pattern 15

### Objective
Optimize skill extraction under bounded latency and bounded noise.

### Preconditions
- namespace is validated
- policy context is loaded
- latency budget is available
- tier capabilities are known

### Heuristic
1. start with the cheapest exact signals
2. cap expansion by query class
3. only pay semantic cost when exact and graph signals are insufficient
4. preserve explainability and provenance
5. update reinforcement or decay after outcome feedback

### Failure risks
- candidate explosion
- stale data bias
- low-confidence amplification
- cross-session interference

### Instrumentation
- p50/p95/p99 latency
- candidate counts before and after pruning
- success contribution estimate
- cache hit ratio
- repairability score

### Pseudocode
```text
fn pattern_315(ctx):
    budget = ctx.latency_budget
    exact = cheap_exact(ctx)
    graph = bounded_graph(ctx, exact)
    sem = maybe_semantic(ctx, budget, graph)
    merged = dedup_merge(exact, graph, sem)
    ranked = rank_with_policy(ctx, merged)
    return package(ctx, ranked[:ctx.limit])
```
## 316. Skill Extraction pattern 16

### Objective
Optimize skill extraction under bounded latency and bounded noise.

### Preconditions
- namespace is validated
- policy context is loaded
- latency budget is available
- tier capabilities are known

### Heuristic
1. start with the cheapest exact signals
2. cap expansion by query class
3. only pay semantic cost when exact and graph signals are insufficient
4. preserve explainability and provenance
5. update reinforcement or decay after outcome feedback

### Failure risks
- candidate explosion
- stale data bias
- low-confidence amplification
- cross-session interference

### Instrumentation
- p50/p95/p99 latency
- candidate counts before and after pruning
- success contribution estimate
- cache hit ratio
- repairability score

### Pseudocode
```text
fn pattern_316(ctx):
    budget = ctx.latency_budget
    exact = cheap_exact(ctx)
    graph = bounded_graph(ctx, exact)
    sem = maybe_semantic(ctx, budget, graph)
    merged = dedup_merge(exact, graph, sem)
    ranked = rank_with_policy(ctx, merged)
    return package(ctx, ranked[:ctx.limit])
```
## 317. Skill Extraction pattern 17

### Objective
Optimize skill extraction under bounded latency and bounded noise.

### Preconditions
- namespace is validated
- policy context is loaded
- latency budget is available
- tier capabilities are known

### Heuristic
1. start with the cheapest exact signals
2. cap expansion by query class
3. only pay semantic cost when exact and graph signals are insufficient
4. preserve explainability and provenance
5. update reinforcement or decay after outcome feedback

### Failure risks
- candidate explosion
- stale data bias
- low-confidence amplification
- cross-session interference

### Instrumentation
- p50/p95/p99 latency
- candidate counts before and after pruning
- success contribution estimate
- cache hit ratio
- repairability score

### Pseudocode
```text
fn pattern_317(ctx):
    budget = ctx.latency_budget
    exact = cheap_exact(ctx)
    graph = bounded_graph(ctx, exact)
    sem = maybe_semantic(ctx, budget, graph)
    merged = dedup_merge(exact, graph, sem)
    ranked = rank_with_policy(ctx, merged)
    return package(ctx, ranked[:ctx.limit])
```
## 318. Skill Extraction pattern 18

### Objective
Optimize skill extraction under bounded latency and bounded noise.

### Preconditions
- namespace is validated
- policy context is loaded
- latency budget is available
- tier capabilities are known

### Heuristic
1. start with the cheapest exact signals
2. cap expansion by query class
3. only pay semantic cost when exact and graph signals are insufficient
4. preserve explainability and provenance
5. update reinforcement or decay after outcome feedback

### Failure risks
- candidate explosion
- stale data bias
- low-confidence amplification
- cross-session interference

### Instrumentation
- p50/p95/p99 latency
- candidate counts before and after pruning
- success contribution estimate
- cache hit ratio
- repairability score

### Pseudocode
```text
fn pattern_318(ctx):
    budget = ctx.latency_budget
    exact = cheap_exact(ctx)
    graph = bounded_graph(ctx, exact)
    sem = maybe_semantic(ctx, budget, graph)
    merged = dedup_merge(exact, graph, sem)
    ranked = rank_with_policy(ctx, merged)
    return package(ctx, ranked[:ctx.limit])
```
## 319. Skill Extraction pattern 19

### Objective
Optimize skill extraction under bounded latency and bounded noise.

### Preconditions
- namespace is validated
- policy context is loaded
- latency budget is available
- tier capabilities are known

### Heuristic
1. start with the cheapest exact signals
2. cap expansion by query class
3. only pay semantic cost when exact and graph signals are insufficient
4. preserve explainability and provenance
5. update reinforcement or decay after outcome feedback

### Failure risks
- candidate explosion
- stale data bias
- low-confidence amplification
- cross-session interference

### Instrumentation
- p50/p95/p99 latency
- candidate counts before and after pruning
- success contribution estimate
- cache hit ratio
- repairability score

### Pseudocode
```text
fn pattern_319(ctx):
    budget = ctx.latency_budget
    exact = cheap_exact(ctx)
    graph = bounded_graph(ctx, exact)
    sem = maybe_semantic(ctx, budget, graph)
    merged = dedup_merge(exact, graph, sem)
    ranked = rank_with_policy(ctx, merged)
    return package(ctx, ranked[:ctx.limit])
```
## 320. Skill Extraction pattern 20

### Objective
Optimize skill extraction under bounded latency and bounded noise.

### Preconditions
- namespace is validated
- policy context is loaded
- latency budget is available
- tier capabilities are known

### Heuristic
1. start with the cheapest exact signals
2. cap expansion by query class
3. only pay semantic cost when exact and graph signals are insufficient
4. preserve explainability and provenance
5. update reinforcement or decay after outcome feedback

### Failure risks
- candidate explosion
- stale data bias
- low-confidence amplification
- cross-session interference

### Instrumentation
- p50/p95/p99 latency
- candidate counts before and after pruning
- success contribution estimate
- cache hit ratio
- repairability score

### Pseudocode
```text
fn pattern_320(ctx):
    budget = ctx.latency_budget
    exact = cheap_exact(ctx)
    graph = bounded_graph(ctx, exact)
    sem = maybe_semantic(ctx, budget, graph)
    merged = dedup_merge(exact, graph, sem)
    ranked = rank_with_policy(ctx, merged)
    return package(ctx, ranked[:ctx.limit])
```
## 321. Skill Extraction pattern 21

### Objective
Optimize skill extraction under bounded latency and bounded noise.

### Preconditions
- namespace is validated
- policy context is loaded
- latency budget is available
- tier capabilities are known

### Heuristic
1. start with the cheapest exact signals
2. cap expansion by query class
3. only pay semantic cost when exact and graph signals are insufficient
4. preserve explainability and provenance
5. update reinforcement or decay after outcome feedback

### Failure risks
- candidate explosion
- stale data bias
- low-confidence amplification
- cross-session interference

### Instrumentation
- p50/p95/p99 latency
- candidate counts before and after pruning
- success contribution estimate
- cache hit ratio
- repairability score

### Pseudocode
```text
fn pattern_321(ctx):
    budget = ctx.latency_budget
    exact = cheap_exact(ctx)
    graph = bounded_graph(ctx, exact)
    sem = maybe_semantic(ctx, budget, graph)
    merged = dedup_merge(exact, graph, sem)
    ranked = rank_with_policy(ctx, merged)
    return package(ctx, ranked[:ctx.limit])
```
## 322. Skill Extraction pattern 22

### Objective
Optimize skill extraction under bounded latency and bounded noise.

### Preconditions
- namespace is validated
- policy context is loaded
- latency budget is available
- tier capabilities are known

### Heuristic
1. start with the cheapest exact signals
2. cap expansion by query class
3. only pay semantic cost when exact and graph signals are insufficient
4. preserve explainability and provenance
5. update reinforcement or decay after outcome feedback

### Failure risks
- candidate explosion
- stale data bias
- low-confidence amplification
- cross-session interference

### Instrumentation
- p50/p95/p99 latency
- candidate counts before and after pruning
- success contribution estimate
- cache hit ratio
- repairability score

### Pseudocode
```text
fn pattern_322(ctx):
    budget = ctx.latency_budget
    exact = cheap_exact(ctx)
    graph = bounded_graph(ctx, exact)
    sem = maybe_semantic(ctx, budget, graph)
    merged = dedup_merge(exact, graph, sem)
    ranked = rank_with_policy(ctx, merged)
    return package(ctx, ranked[:ctx.limit])
```
## 323. Skill Extraction pattern 23

### Objective
Optimize skill extraction under bounded latency and bounded noise.

### Preconditions
- namespace is validated
- policy context is loaded
- latency budget is available
- tier capabilities are known

### Heuristic
1. start with the cheapest exact signals
2. cap expansion by query class
3. only pay semantic cost when exact and graph signals are insufficient
4. preserve explainability and provenance
5. update reinforcement or decay after outcome feedback

### Failure risks
- candidate explosion
- stale data bias
- low-confidence amplification
- cross-session interference

### Instrumentation
- p50/p95/p99 latency
- candidate counts before and after pruning
- success contribution estimate
- cache hit ratio
- repairability score

### Pseudocode
```text
fn pattern_323(ctx):
    budget = ctx.latency_budget
    exact = cheap_exact(ctx)
    graph = bounded_graph(ctx, exact)
    sem = maybe_semantic(ctx, budget, graph)
    merged = dedup_merge(exact, graph, sem)
    ranked = rank_with_policy(ctx, merged)
    return package(ctx, ranked[:ctx.limit])
```
## 324. Skill Extraction pattern 24

### Objective
Optimize skill extraction under bounded latency and bounded noise.

### Preconditions
- namespace is validated
- policy context is loaded
- latency budget is available
- tier capabilities are known

### Heuristic
1. start with the cheapest exact signals
2. cap expansion by query class
3. only pay semantic cost when exact and graph signals are insufficient
4. preserve explainability and provenance
5. update reinforcement or decay after outcome feedback

### Failure risks
- candidate explosion
- stale data bias
- low-confidence amplification
- cross-session interference

### Instrumentation
- p50/p95/p99 latency
- candidate counts before and after pruning
- success contribution estimate
- cache hit ratio
- repairability score

### Pseudocode
```text
fn pattern_324(ctx):
    budget = ctx.latency_budget
    exact = cheap_exact(ctx)
    graph = bounded_graph(ctx, exact)
    sem = maybe_semantic(ctx, budget, graph)
    merged = dedup_merge(exact, graph, sem)
    ranked = rank_with_policy(ctx, merged)
    return package(ctx, ranked[:ctx.limit])
```
## 325. Skill Extraction pattern 25

### Objective
Optimize skill extraction under bounded latency and bounded noise.

### Preconditions
- namespace is validated
- policy context is loaded
- latency budget is available
- tier capabilities are known

### Heuristic
1. start with the cheapest exact signals
2. cap expansion by query class
3. only pay semantic cost when exact and graph signals are insufficient
4. preserve explainability and provenance
5. update reinforcement or decay after outcome feedback

### Failure risks
- candidate explosion
- stale data bias
- low-confidence amplification
- cross-session interference

### Instrumentation
- p50/p95/p99 latency
- candidate counts before and after pruning
- success contribution estimate
- cache hit ratio
- repairability score

### Pseudocode
```text
fn pattern_325(ctx):
    budget = ctx.latency_budget
    exact = cheap_exact(ctx)
    graph = bounded_graph(ctx, exact)
    sem = maybe_semantic(ctx, budget, graph)
    merged = dedup_merge(exact, graph, sem)
    ranked = rank_with_policy(ctx, merged)
    return package(ctx, ranked[:ctx.limit])
```
## 326. Skill Extraction pattern 26

### Objective
Optimize skill extraction under bounded latency and bounded noise.

### Preconditions
- namespace is validated
- policy context is loaded
- latency budget is available
- tier capabilities are known

### Heuristic
1. start with the cheapest exact signals
2. cap expansion by query class
3. only pay semantic cost when exact and graph signals are insufficient
4. preserve explainability and provenance
5. update reinforcement or decay after outcome feedback

### Failure risks
- candidate explosion
- stale data bias
- low-confidence amplification
- cross-session interference

### Instrumentation
- p50/p95/p99 latency
- candidate counts before and after pruning
- success contribution estimate
- cache hit ratio
- repairability score

### Pseudocode
```text
fn pattern_326(ctx):
    budget = ctx.latency_budget
    exact = cheap_exact(ctx)
    graph = bounded_graph(ctx, exact)
    sem = maybe_semantic(ctx, budget, graph)
    merged = dedup_merge(exact, graph, sem)
    ranked = rank_with_policy(ctx, merged)
    return package(ctx, ranked[:ctx.limit])
```
## 327. Skill Extraction pattern 27

### Objective
Optimize skill extraction under bounded latency and bounded noise.

### Preconditions
- namespace is validated
- policy context is loaded
- latency budget is available
- tier capabilities are known

### Heuristic
1. start with the cheapest exact signals
2. cap expansion by query class
3. only pay semantic cost when exact and graph signals are insufficient
4. preserve explainability and provenance
5. update reinforcement or decay after outcome feedback

### Failure risks
- candidate explosion
- stale data bias
- low-confidence amplification
- cross-session interference

### Instrumentation
- p50/p95/p99 latency
- candidate counts before and after pruning
- success contribution estimate
- cache hit ratio
- repairability score

### Pseudocode
```text
fn pattern_327(ctx):
    budget = ctx.latency_budget
    exact = cheap_exact(ctx)
    graph = bounded_graph(ctx, exact)
    sem = maybe_semantic(ctx, budget, graph)
    merged = dedup_merge(exact, graph, sem)
    ranked = rank_with_policy(ctx, merged)
    return package(ctx, ranked[:ctx.limit])
```
## 328. Skill Extraction pattern 28

### Objective
Optimize skill extraction under bounded latency and bounded noise.

### Preconditions
- namespace is validated
- policy context is loaded
- latency budget is available
- tier capabilities are known

### Heuristic
1. start with the cheapest exact signals
2. cap expansion by query class
3. only pay semantic cost when exact and graph signals are insufficient
4. preserve explainability and provenance
5. update reinforcement or decay after outcome feedback

### Failure risks
- candidate explosion
- stale data bias
- low-confidence amplification
- cross-session interference

### Instrumentation
- p50/p95/p99 latency
- candidate counts before and after pruning
- success contribution estimate
- cache hit ratio
- repairability score

### Pseudocode
```text
fn pattern_328(ctx):
    budget = ctx.latency_budget
    exact = cheap_exact(ctx)
    graph = bounded_graph(ctx, exact)
    sem = maybe_semantic(ctx, budget, graph)
    merged = dedup_merge(exact, graph, sem)
    ranked = rank_with_policy(ctx, merged)
    return package(ctx, ranked[:ctx.limit])
```
## 329. Skill Extraction pattern 29

### Objective
Optimize skill extraction under bounded latency and bounded noise.

### Preconditions
- namespace is validated
- policy context is loaded
- latency budget is available
- tier capabilities are known

### Heuristic
1. start with the cheapest exact signals
2. cap expansion by query class
3. only pay semantic cost when exact and graph signals are insufficient
4. preserve explainability and provenance
5. update reinforcement or decay after outcome feedback

### Failure risks
- candidate explosion
- stale data bias
- low-confidence amplification
- cross-session interference

### Instrumentation
- p50/p95/p99 latency
- candidate counts before and after pruning
- success contribution estimate
- cache hit ratio
- repairability score

### Pseudocode
```text
fn pattern_329(ctx):
    budget = ctx.latency_budget
    exact = cheap_exact(ctx)
    graph = bounded_graph(ctx, exact)
    sem = maybe_semantic(ctx, budget, graph)
    merged = dedup_merge(exact, graph, sem)
    ranked = rank_with_policy(ctx, merged)
    return package(ctx, ranked[:ctx.limit])
```
## 330. Skill Extraction pattern 30

### Objective
Optimize skill extraction under bounded latency and bounded noise.

### Preconditions
- namespace is validated
- policy context is loaded
- latency budget is available
- tier capabilities are known

### Heuristic
1. start with the cheapest exact signals
2. cap expansion by query class
3. only pay semantic cost when exact and graph signals are insufficient
4. preserve explainability and provenance
5. update reinforcement or decay after outcome feedback

### Failure risks
- candidate explosion
- stale data bias
- low-confidence amplification
- cross-session interference

### Instrumentation
- p50/p95/p99 latency
- candidate counts before and after pruning
- success contribution estimate
- cache hit ratio
- repairability score

### Pseudocode
```text
fn pattern_330(ctx):
    budget = ctx.latency_budget
    exact = cheap_exact(ctx)
    graph = bounded_graph(ctx, exact)
    sem = maybe_semantic(ctx, budget, graph)
    merged = dedup_merge(exact, graph, sem)
    ranked = rank_with_policy(ctx, merged)
    return package(ctx, ranked[:ctx.limit])
```
## 331. Tier3 Reconstruction pattern 1

### Objective
Optimize tier3 reconstruction under bounded latency and bounded noise.

### Preconditions
- namespace is validated
- policy context is loaded
- latency budget is available
- tier capabilities are known

### Heuristic
1. start with the cheapest exact signals
2. cap expansion by query class
3. only pay semantic cost when exact and graph signals are insufficient
4. preserve explainability and provenance
5. update reinforcement or decay after outcome feedback

### Failure risks
- candidate explosion
- stale data bias
- low-confidence amplification
- cross-session interference

### Instrumentation
- p50/p95/p99 latency
- candidate counts before and after pruning
- success contribution estimate
- cache hit ratio
- repairability score

### Pseudocode
```text
fn pattern_331(ctx):
    budget = ctx.latency_budget
    exact = cheap_exact(ctx)
    graph = bounded_graph(ctx, exact)
    sem = maybe_semantic(ctx, budget, graph)
    merged = dedup_merge(exact, graph, sem)
    ranked = rank_with_policy(ctx, merged)
    return package(ctx, ranked[:ctx.limit])
```
## 332. Tier3 Reconstruction pattern 2

### Objective
Optimize tier3 reconstruction under bounded latency and bounded noise.

### Preconditions
- namespace is validated
- policy context is loaded
- latency budget is available
- tier capabilities are known

### Heuristic
1. start with the cheapest exact signals
2. cap expansion by query class
3. only pay semantic cost when exact and graph signals are insufficient
4. preserve explainability and provenance
5. update reinforcement or decay after outcome feedback

### Failure risks
- candidate explosion
- stale data bias
- low-confidence amplification
- cross-session interference

### Instrumentation
- p50/p95/p99 latency
- candidate counts before and after pruning
- success contribution estimate
- cache hit ratio
- repairability score

### Pseudocode
```text
fn pattern_332(ctx):
    budget = ctx.latency_budget
    exact = cheap_exact(ctx)
    graph = bounded_graph(ctx, exact)
    sem = maybe_semantic(ctx, budget, graph)
    merged = dedup_merge(exact, graph, sem)
    ranked = rank_with_policy(ctx, merged)
    return package(ctx, ranked[:ctx.limit])
```
## 333. Tier3 Reconstruction pattern 3

### Objective
Optimize tier3 reconstruction under bounded latency and bounded noise.

### Preconditions
- namespace is validated
- policy context is loaded
- latency budget is available
- tier capabilities are known

### Heuristic
1. start with the cheapest exact signals
2. cap expansion by query class
3. only pay semantic cost when exact and graph signals are insufficient
4. preserve explainability and provenance
5. update reinforcement or decay after outcome feedback

### Failure risks
- candidate explosion
- stale data bias
- low-confidence amplification
- cross-session interference

### Instrumentation
- p50/p95/p99 latency
- candidate counts before and after pruning
- success contribution estimate
- cache hit ratio
- repairability score

### Pseudocode
```text
fn pattern_333(ctx):
    budget = ctx.latency_budget
    exact = cheap_exact(ctx)
    graph = bounded_graph(ctx, exact)
    sem = maybe_semantic(ctx, budget, graph)
    merged = dedup_merge(exact, graph, sem)
    ranked = rank_with_policy(ctx, merged)
    return package(ctx, ranked[:ctx.limit])
```
## 334. Tier3 Reconstruction pattern 4

### Objective
Optimize tier3 reconstruction under bounded latency and bounded noise.

### Preconditions
- namespace is validated
- policy context is loaded
- latency budget is available
- tier capabilities are known

### Heuristic
1. start with the cheapest exact signals
2. cap expansion by query class
3. only pay semantic cost when exact and graph signals are insufficient
4. preserve explainability and provenance
5. update reinforcement or decay after outcome feedback

### Failure risks
- candidate explosion
- stale data bias
- low-confidence amplification
- cross-session interference

### Instrumentation
- p50/p95/p99 latency
- candidate counts before and after pruning
- success contribution estimate
- cache hit ratio
- repairability score

### Pseudocode
```text
fn pattern_334(ctx):
    budget = ctx.latency_budget
    exact = cheap_exact(ctx)
    graph = bounded_graph(ctx, exact)
    sem = maybe_semantic(ctx, budget, graph)
    merged = dedup_merge(exact, graph, sem)
    ranked = rank_with_policy(ctx, merged)
    return package(ctx, ranked[:ctx.limit])
```
## 335. Tier3 Reconstruction pattern 5

### Objective
Optimize tier3 reconstruction under bounded latency and bounded noise.

### Preconditions
- namespace is validated
- policy context is loaded
- latency budget is available
- tier capabilities are known

### Heuristic
1. start with the cheapest exact signals
2. cap expansion by query class
3. only pay semantic cost when exact and graph signals are insufficient
4. preserve explainability and provenance
5. update reinforcement or decay after outcome feedback

### Failure risks
- candidate explosion
- stale data bias
- low-confidence amplification
- cross-session interference

### Instrumentation
- p50/p95/p99 latency
- candidate counts before and after pruning
- success contribution estimate
- cache hit ratio
- repairability score

### Pseudocode
```text
fn pattern_335(ctx):
    budget = ctx.latency_budget
    exact = cheap_exact(ctx)
    graph = bounded_graph(ctx, exact)
    sem = maybe_semantic(ctx, budget, graph)
    merged = dedup_merge(exact, graph, sem)
    ranked = rank_with_policy(ctx, merged)
    return package(ctx, ranked[:ctx.limit])
```
## 336. Tier3 Reconstruction pattern 6

### Objective
Optimize tier3 reconstruction under bounded latency and bounded noise.

### Preconditions
- namespace is validated
- policy context is loaded
- latency budget is available
- tier capabilities are known

### Heuristic
1. start with the cheapest exact signals
2. cap expansion by query class
3. only pay semantic cost when exact and graph signals are insufficient
4. preserve explainability and provenance
5. update reinforcement or decay after outcome feedback

### Failure risks
- candidate explosion
- stale data bias
- low-confidence amplification
- cross-session interference

### Instrumentation
- p50/p95/p99 latency
- candidate counts before and after pruning
- success contribution estimate
- cache hit ratio
- repairability score

### Pseudocode
```text
fn pattern_336(ctx):
    budget = ctx.latency_budget
    exact = cheap_exact(ctx)
    graph = bounded_graph(ctx, exact)
    sem = maybe_semantic(ctx, budget, graph)
    merged = dedup_merge(exact, graph, sem)
    ranked = rank_with_policy(ctx, merged)
    return package(ctx, ranked[:ctx.limit])
```
## 337. Tier3 Reconstruction pattern 7

### Objective
Optimize tier3 reconstruction under bounded latency and bounded noise.

### Preconditions
- namespace is validated
- policy context is loaded
- latency budget is available
- tier capabilities are known

### Heuristic
1. start with the cheapest exact signals
2. cap expansion by query class
3. only pay semantic cost when exact and graph signals are insufficient
4. preserve explainability and provenance
5. update reinforcement or decay after outcome feedback

### Failure risks
- candidate explosion
- stale data bias
- low-confidence amplification
- cross-session interference

### Instrumentation
- p50/p95/p99 latency
- candidate counts before and after pruning
- success contribution estimate
- cache hit ratio
- repairability score

### Pseudocode
```text
fn pattern_337(ctx):
    budget = ctx.latency_budget
    exact = cheap_exact(ctx)
    graph = bounded_graph(ctx, exact)
    sem = maybe_semantic(ctx, budget, graph)
    merged = dedup_merge(exact, graph, sem)
    ranked = rank_with_policy(ctx, merged)
    return package(ctx, ranked[:ctx.limit])
```
## 338. Tier3 Reconstruction pattern 8

### Objective
Optimize tier3 reconstruction under bounded latency and bounded noise.

### Preconditions
- namespace is validated
- policy context is loaded
- latency budget is available
- tier capabilities are known

### Heuristic
1. start with the cheapest exact signals
2. cap expansion by query class
3. only pay semantic cost when exact and graph signals are insufficient
4. preserve explainability and provenance
5. update reinforcement or decay after outcome feedback

### Failure risks
- candidate explosion
- stale data bias
- low-confidence amplification
- cross-session interference

### Instrumentation
- p50/p95/p99 latency
- candidate counts before and after pruning
- success contribution estimate
- cache hit ratio
- repairability score

### Pseudocode
```text
fn pattern_338(ctx):
    budget = ctx.latency_budget
    exact = cheap_exact(ctx)
    graph = bounded_graph(ctx, exact)
    sem = maybe_semantic(ctx, budget, graph)
    merged = dedup_merge(exact, graph, sem)
    ranked = rank_with_policy(ctx, merged)
    return package(ctx, ranked[:ctx.limit])
```
## 339. Tier3 Reconstruction pattern 9

### Objective
Optimize tier3 reconstruction under bounded latency and bounded noise.

### Preconditions
- namespace is validated
- policy context is loaded
- latency budget is available
- tier capabilities are known

### Heuristic
1. start with the cheapest exact signals
2. cap expansion by query class
3. only pay semantic cost when exact and graph signals are insufficient
4. preserve explainability and provenance
5. update reinforcement or decay after outcome feedback

### Failure risks
- candidate explosion
- stale data bias
- low-confidence amplification
- cross-session interference

### Instrumentation
- p50/p95/p99 latency
- candidate counts before and after pruning
- success contribution estimate
- cache hit ratio
- repairability score

### Pseudocode
```text
fn pattern_339(ctx):
    budget = ctx.latency_budget
    exact = cheap_exact(ctx)
    graph = bounded_graph(ctx, exact)
    sem = maybe_semantic(ctx, budget, graph)
    merged = dedup_merge(exact, graph, sem)
    ranked = rank_with_policy(ctx, merged)
    return package(ctx, ranked[:ctx.limit])
```
## 340. Tier3 Reconstruction pattern 10

### Objective
Optimize tier3 reconstruction under bounded latency and bounded noise.

### Preconditions
- namespace is validated
- policy context is loaded
- latency budget is available
- tier capabilities are known

### Heuristic
1. start with the cheapest exact signals
2. cap expansion by query class
3. only pay semantic cost when exact and graph signals are insufficient
4. preserve explainability and provenance
5. update reinforcement or decay after outcome feedback

### Failure risks
- candidate explosion
- stale data bias
- low-confidence amplification
- cross-session interference

### Instrumentation
- p50/p95/p99 latency
- candidate counts before and after pruning
- success contribution estimate
- cache hit ratio
- repairability score

### Pseudocode
```text
fn pattern_340(ctx):
    budget = ctx.latency_budget
    exact = cheap_exact(ctx)
    graph = bounded_graph(ctx, exact)
    sem = maybe_semantic(ctx, budget, graph)
    merged = dedup_merge(exact, graph, sem)
    ranked = rank_with_policy(ctx, merged)
    return package(ctx, ranked[:ctx.limit])
```
## 341. Tier3 Reconstruction pattern 11

### Objective
Optimize tier3 reconstruction under bounded latency and bounded noise.

### Preconditions
- namespace is validated
- policy context is loaded
- latency budget is available
- tier capabilities are known

### Heuristic
1. start with the cheapest exact signals
2. cap expansion by query class
3. only pay semantic cost when exact and graph signals are insufficient
4. preserve explainability and provenance
5. update reinforcement or decay after outcome feedback

### Failure risks
- candidate explosion
- stale data bias
- low-confidence amplification
- cross-session interference

### Instrumentation
- p50/p95/p99 latency
- candidate counts before and after pruning
- success contribution estimate
- cache hit ratio
- repairability score

### Pseudocode
```text
fn pattern_341(ctx):
    budget = ctx.latency_budget
    exact = cheap_exact(ctx)
    graph = bounded_graph(ctx, exact)
    sem = maybe_semantic(ctx, budget, graph)
    merged = dedup_merge(exact, graph, sem)
    ranked = rank_with_policy(ctx, merged)
    return package(ctx, ranked[:ctx.limit])
```
## 342. Tier3 Reconstruction pattern 12

### Objective
Optimize tier3 reconstruction under bounded latency and bounded noise.

### Preconditions
- namespace is validated
- policy context is loaded
- latency budget is available
- tier capabilities are known

### Heuristic
1. start with the cheapest exact signals
2. cap expansion by query class
3. only pay semantic cost when exact and graph signals are insufficient
4. preserve explainability and provenance
5. update reinforcement or decay after outcome feedback

### Failure risks
- candidate explosion
- stale data bias
- low-confidence amplification
- cross-session interference

### Instrumentation
- p50/p95/p99 latency
- candidate counts before and after pruning
- success contribution estimate
- cache hit ratio
- repairability score

### Pseudocode
```text
fn pattern_342(ctx):
    budget = ctx.latency_budget
    exact = cheap_exact(ctx)
    graph = bounded_graph(ctx, exact)
    sem = maybe_semantic(ctx, budget, graph)
    merged = dedup_merge(exact, graph, sem)
    ranked = rank_with_policy(ctx, merged)
    return package(ctx, ranked[:ctx.limit])
```
## 343. Tier3 Reconstruction pattern 13

### Objective
Optimize tier3 reconstruction under bounded latency and bounded noise.

### Preconditions
- namespace is validated
- policy context is loaded
- latency budget is available
- tier capabilities are known

### Heuristic
1. start with the cheapest exact signals
2. cap expansion by query class
3. only pay semantic cost when exact and graph signals are insufficient
4. preserve explainability and provenance
5. update reinforcement or decay after outcome feedback

### Failure risks
- candidate explosion
- stale data bias
- low-confidence amplification
- cross-session interference

### Instrumentation
- p50/p95/p99 latency
- candidate counts before and after pruning
- success contribution estimate
- cache hit ratio
- repairability score

### Pseudocode
```text
fn pattern_343(ctx):
    budget = ctx.latency_budget
    exact = cheap_exact(ctx)
    graph = bounded_graph(ctx, exact)
    sem = maybe_semantic(ctx, budget, graph)
    merged = dedup_merge(exact, graph, sem)
    ranked = rank_with_policy(ctx, merged)
    return package(ctx, ranked[:ctx.limit])
```
## 344. Tier3 Reconstruction pattern 14

### Objective
Optimize tier3 reconstruction under bounded latency and bounded noise.

### Preconditions
- namespace is validated
- policy context is loaded
- latency budget is available
- tier capabilities are known

### Heuristic
1. start with the cheapest exact signals
2. cap expansion by query class
3. only pay semantic cost when exact and graph signals are insufficient
4. preserve explainability and provenance
5. update reinforcement or decay after outcome feedback

### Failure risks
- candidate explosion
- stale data bias
- low-confidence amplification
- cross-session interference

### Instrumentation
- p50/p95/p99 latency
- candidate counts before and after pruning
- success contribution estimate
- cache hit ratio
- repairability score

### Pseudocode
```text
fn pattern_344(ctx):
    budget = ctx.latency_budget
    exact = cheap_exact(ctx)
    graph = bounded_graph(ctx, exact)
    sem = maybe_semantic(ctx, budget, graph)
    merged = dedup_merge(exact, graph, sem)
    ranked = rank_with_policy(ctx, merged)
    return package(ctx, ranked[:ctx.limit])
```
## 345. Tier3 Reconstruction pattern 15

### Objective
Optimize tier3 reconstruction under bounded latency and bounded noise.

### Preconditions
- namespace is validated
- policy context is loaded
- latency budget is available
- tier capabilities are known

### Heuristic
1. start with the cheapest exact signals
2. cap expansion by query class
3. only pay semantic cost when exact and graph signals are insufficient
4. preserve explainability and provenance
5. update reinforcement or decay after outcome feedback

### Failure risks
- candidate explosion
- stale data bias
- low-confidence amplification
- cross-session interference

### Instrumentation
- p50/p95/p99 latency
- candidate counts before and after pruning
- success contribution estimate
- cache hit ratio
- repairability score

### Pseudocode
```text
fn pattern_345(ctx):
    budget = ctx.latency_budget
    exact = cheap_exact(ctx)
    graph = bounded_graph(ctx, exact)
    sem = maybe_semantic(ctx, budget, graph)
    merged = dedup_merge(exact, graph, sem)
    ranked = rank_with_policy(ctx, merged)
    return package(ctx, ranked[:ctx.limit])
```
## 346. Tier3 Reconstruction pattern 16

### Objective
Optimize tier3 reconstruction under bounded latency and bounded noise.

### Preconditions
- namespace is validated
- policy context is loaded
- latency budget is available
- tier capabilities are known

### Heuristic
1. start with the cheapest exact signals
2. cap expansion by query class
3. only pay semantic cost when exact and graph signals are insufficient
4. preserve explainability and provenance
5. update reinforcement or decay after outcome feedback

### Failure risks
- candidate explosion
- stale data bias
- low-confidence amplification
- cross-session interference

### Instrumentation
- p50/p95/p99 latency
- candidate counts before and after pruning
- success contribution estimate
- cache hit ratio
- repairability score

### Pseudocode
```text
fn pattern_346(ctx):
    budget = ctx.latency_budget
    exact = cheap_exact(ctx)
    graph = bounded_graph(ctx, exact)
    sem = maybe_semantic(ctx, budget, graph)
    merged = dedup_merge(exact, graph, sem)
    ranked = rank_with_policy(ctx, merged)
    return package(ctx, ranked[:ctx.limit])
```
## 347. Tier3 Reconstruction pattern 17

### Objective
Optimize tier3 reconstruction under bounded latency and bounded noise.

### Preconditions
- namespace is validated
- policy context is loaded
- latency budget is available
- tier capabilities are known

### Heuristic
1. start with the cheapest exact signals
2. cap expansion by query class
3. only pay semantic cost when exact and graph signals are insufficient
4. preserve explainability and provenance
5. update reinforcement or decay after outcome feedback

### Failure risks
- candidate explosion
- stale data bias
- low-confidence amplification
- cross-session interference

### Instrumentation
- p50/p95/p99 latency
- candidate counts before and after pruning
- success contribution estimate
- cache hit ratio
- repairability score

### Pseudocode
```text
fn pattern_347(ctx):
    budget = ctx.latency_budget
    exact = cheap_exact(ctx)
    graph = bounded_graph(ctx, exact)
    sem = maybe_semantic(ctx, budget, graph)
    merged = dedup_merge(exact, graph, sem)
    ranked = rank_with_policy(ctx, merged)
    return package(ctx, ranked[:ctx.limit])
```
## 348. Tier3 Reconstruction pattern 18

### Objective
Optimize tier3 reconstruction under bounded latency and bounded noise.

### Preconditions
- namespace is validated
- policy context is loaded
- latency budget is available
- tier capabilities are known

### Heuristic
1. start with the cheapest exact signals
2. cap expansion by query class
3. only pay semantic cost when exact and graph signals are insufficient
4. preserve explainability and provenance
5. update reinforcement or decay after outcome feedback

### Failure risks
- candidate explosion
- stale data bias
- low-confidence amplification
- cross-session interference

### Instrumentation
- p50/p95/p99 latency
- candidate counts before and after pruning
- success contribution estimate
- cache hit ratio
- repairability score

### Pseudocode
```text
fn pattern_348(ctx):
    budget = ctx.latency_budget
    exact = cheap_exact(ctx)
    graph = bounded_graph(ctx, exact)
    sem = maybe_semantic(ctx, budget, graph)
    merged = dedup_merge(exact, graph, sem)
    ranked = rank_with_policy(ctx, merged)
    return package(ctx, ranked[:ctx.limit])
```
## 349. Tier3 Reconstruction pattern 19

### Objective
Optimize tier3 reconstruction under bounded latency and bounded noise.

### Preconditions
- namespace is validated
- policy context is loaded
- latency budget is available
- tier capabilities are known

### Heuristic
1. start with the cheapest exact signals
2. cap expansion by query class
3. only pay semantic cost when exact and graph signals are insufficient
4. preserve explainability and provenance
5. update reinforcement or decay after outcome feedback

### Failure risks
- candidate explosion
- stale data bias
- low-confidence amplification
- cross-session interference

### Instrumentation
- p50/p95/p99 latency
- candidate counts before and after pruning
- success contribution estimate
- cache hit ratio
- repairability score

### Pseudocode
```text
fn pattern_349(ctx):
    budget = ctx.latency_budget
    exact = cheap_exact(ctx)
    graph = bounded_graph(ctx, exact)
    sem = maybe_semantic(ctx, budget, graph)
    merged = dedup_merge(exact, graph, sem)
    ranked = rank_with_policy(ctx, merged)
    return package(ctx, ranked[:ctx.limit])
```
## 350. Tier3 Reconstruction pattern 20

### Objective
Optimize tier3 reconstruction under bounded latency and bounded noise.

### Preconditions
- namespace is validated
- policy context is loaded
- latency budget is available
- tier capabilities are known

### Heuristic
1. start with the cheapest exact signals
2. cap expansion by query class
3. only pay semantic cost when exact and graph signals are insufficient
4. preserve explainability and provenance
5. update reinforcement or decay after outcome feedback

### Failure risks
- candidate explosion
- stale data bias
- low-confidence amplification
- cross-session interference

### Instrumentation
- p50/p95/p99 latency
- candidate counts before and after pruning
- success contribution estimate
- cache hit ratio
- repairability score

### Pseudocode
```text
fn pattern_350(ctx):
    budget = ctx.latency_budget
    exact = cheap_exact(ctx)
    graph = bounded_graph(ctx, exact)
    sem = maybe_semantic(ctx, budget, graph)
    merged = dedup_merge(exact, graph, sem)
    ranked = rank_with_policy(ctx, merged)
    return package(ctx, ranked[:ctx.limit])
```
## 351. Tier3 Reconstruction pattern 21

### Objective
Optimize tier3 reconstruction under bounded latency and bounded noise.

### Preconditions
- namespace is validated
- policy context is loaded
- latency budget is available
- tier capabilities are known

### Heuristic
1. start with the cheapest exact signals
2. cap expansion by query class
3. only pay semantic cost when exact and graph signals are insufficient
4. preserve explainability and provenance
5. update reinforcement or decay after outcome feedback

### Failure risks
- candidate explosion
- stale data bias
- low-confidence amplification
- cross-session interference

### Instrumentation
- p50/p95/p99 latency
- candidate counts before and after pruning
- success contribution estimate
- cache hit ratio
- repairability score

### Pseudocode
```text
fn pattern_351(ctx):
    budget = ctx.latency_budget
    exact = cheap_exact(ctx)
    graph = bounded_graph(ctx, exact)
    sem = maybe_semantic(ctx, budget, graph)
    merged = dedup_merge(exact, graph, sem)
    ranked = rank_with_policy(ctx, merged)
    return package(ctx, ranked[:ctx.limit])
```
## 352. Tier3 Reconstruction pattern 22

### Objective
Optimize tier3 reconstruction under bounded latency and bounded noise.

### Preconditions
- namespace is validated
- policy context is loaded
- latency budget is available
- tier capabilities are known

### Heuristic
1. start with the cheapest exact signals
2. cap expansion by query class
3. only pay semantic cost when exact and graph signals are insufficient
4. preserve explainability and provenance
5. update reinforcement or decay after outcome feedback

### Failure risks
- candidate explosion
- stale data bias
- low-confidence amplification
- cross-session interference

### Instrumentation
- p50/p95/p99 latency
- candidate counts before and after pruning
- success contribution estimate
- cache hit ratio
- repairability score

### Pseudocode
```text
fn pattern_352(ctx):
    budget = ctx.latency_budget
    exact = cheap_exact(ctx)
    graph = bounded_graph(ctx, exact)
    sem = maybe_semantic(ctx, budget, graph)
    merged = dedup_merge(exact, graph, sem)
    ranked = rank_with_policy(ctx, merged)
    return package(ctx, ranked[:ctx.limit])
```
## 353. Tier3 Reconstruction pattern 23

### Objective
Optimize tier3 reconstruction under bounded latency and bounded noise.

### Preconditions
- namespace is validated
- policy context is loaded
- latency budget is available
- tier capabilities are known

### Heuristic
1. start with the cheapest exact signals
2. cap expansion by query class
3. only pay semantic cost when exact and graph signals are insufficient
4. preserve explainability and provenance
5. update reinforcement or decay after outcome feedback

### Failure risks
- candidate explosion
- stale data bias
- low-confidence amplification
- cross-session interference

### Instrumentation
- p50/p95/p99 latency
- candidate counts before and after pruning
- success contribution estimate
- cache hit ratio
- repairability score

### Pseudocode
```text
fn pattern_353(ctx):
    budget = ctx.latency_budget
    exact = cheap_exact(ctx)
    graph = bounded_graph(ctx, exact)
    sem = maybe_semantic(ctx, budget, graph)
    merged = dedup_merge(exact, graph, sem)
    ranked = rank_with_policy(ctx, merged)
    return package(ctx, ranked[:ctx.limit])
```
## 354. Tier3 Reconstruction pattern 24

### Objective
Optimize tier3 reconstruction under bounded latency and bounded noise.

### Preconditions
- namespace is validated
- policy context is loaded
- latency budget is available
- tier capabilities are known

### Heuristic
1. start with the cheapest exact signals
2. cap expansion by query class
3. only pay semantic cost when exact and graph signals are insufficient
4. preserve explainability and provenance
5. update reinforcement or decay after outcome feedback

### Failure risks
- candidate explosion
- stale data bias
- low-confidence amplification
- cross-session interference

### Instrumentation
- p50/p95/p99 latency
- candidate counts before and after pruning
- success contribution estimate
- cache hit ratio
- repairability score

### Pseudocode
```text
fn pattern_354(ctx):
    budget = ctx.latency_budget
    exact = cheap_exact(ctx)
    graph = bounded_graph(ctx, exact)
    sem = maybe_semantic(ctx, budget, graph)
    merged = dedup_merge(exact, graph, sem)
    ranked = rank_with_policy(ctx, merged)
    return package(ctx, ranked[:ctx.limit])
```
## 355. Tier3 Reconstruction pattern 25

### Objective
Optimize tier3 reconstruction under bounded latency and bounded noise.

### Preconditions
- namespace is validated
- policy context is loaded
- latency budget is available
- tier capabilities are known

### Heuristic
1. start with the cheapest exact signals
2. cap expansion by query class
3. only pay semantic cost when exact and graph signals are insufficient
4. preserve explainability and provenance
5. update reinforcement or decay after outcome feedback

### Failure risks
- candidate explosion
- stale data bias
- low-confidence amplification
- cross-session interference

### Instrumentation
- p50/p95/p99 latency
- candidate counts before and after pruning
- success contribution estimate
- cache hit ratio
- repairability score

### Pseudocode
```text
fn pattern_355(ctx):
    budget = ctx.latency_budget
    exact = cheap_exact(ctx)
    graph = bounded_graph(ctx, exact)
    sem = maybe_semantic(ctx, budget, graph)
    merged = dedup_merge(exact, graph, sem)
    ranked = rank_with_policy(ctx, merged)
    return package(ctx, ranked[:ctx.limit])
```
## 356. Tier3 Reconstruction pattern 26

### Objective
Optimize tier3 reconstruction under bounded latency and bounded noise.

### Preconditions
- namespace is validated
- policy context is loaded
- latency budget is available
- tier capabilities are known

### Heuristic
1. start with the cheapest exact signals
2. cap expansion by query class
3. only pay semantic cost when exact and graph signals are insufficient
4. preserve explainability and provenance
5. update reinforcement or decay after outcome feedback

### Failure risks
- candidate explosion
- stale data bias
- low-confidence amplification
- cross-session interference

### Instrumentation
- p50/p95/p99 latency
- candidate counts before and after pruning
- success contribution estimate
- cache hit ratio
- repairability score

### Pseudocode
```text
fn pattern_356(ctx):
    budget = ctx.latency_budget
    exact = cheap_exact(ctx)
    graph = bounded_graph(ctx, exact)
    sem = maybe_semantic(ctx, budget, graph)
    merged = dedup_merge(exact, graph, sem)
    ranked = rank_with_policy(ctx, merged)
    return package(ctx, ranked[:ctx.limit])
```
## 357. Tier3 Reconstruction pattern 27

### Objective
Optimize tier3 reconstruction under bounded latency and bounded noise.

### Preconditions
- namespace is validated
- policy context is loaded
- latency budget is available
- tier capabilities are known

### Heuristic
1. start with the cheapest exact signals
2. cap expansion by query class
3. only pay semantic cost when exact and graph signals are insufficient
4. preserve explainability and provenance
5. update reinforcement or decay after outcome feedback

### Failure risks
- candidate explosion
- stale data bias
- low-confidence amplification
- cross-session interference

### Instrumentation
- p50/p95/p99 latency
- candidate counts before and after pruning
- success contribution estimate
- cache hit ratio
- repairability score

### Pseudocode
```text
fn pattern_357(ctx):
    budget = ctx.latency_budget
    exact = cheap_exact(ctx)
    graph = bounded_graph(ctx, exact)
    sem = maybe_semantic(ctx, budget, graph)
    merged = dedup_merge(exact, graph, sem)
    ranked = rank_with_policy(ctx, merged)
    return package(ctx, ranked[:ctx.limit])
```
## 358. Tier3 Reconstruction pattern 28

### Objective
Optimize tier3 reconstruction under bounded latency and bounded noise.

### Preconditions
- namespace is validated
- policy context is loaded
- latency budget is available
- tier capabilities are known

### Heuristic
1. start with the cheapest exact signals
2. cap expansion by query class
3. only pay semantic cost when exact and graph signals are insufficient
4. preserve explainability and provenance
5. update reinforcement or decay after outcome feedback

### Failure risks
- candidate explosion
- stale data bias
- low-confidence amplification
- cross-session interference

### Instrumentation
- p50/p95/p99 latency
- candidate counts before and after pruning
- success contribution estimate
- cache hit ratio
- repairability score

### Pseudocode
```text
fn pattern_358(ctx):
    budget = ctx.latency_budget
    exact = cheap_exact(ctx)
    graph = bounded_graph(ctx, exact)
    sem = maybe_semantic(ctx, budget, graph)
    merged = dedup_merge(exact, graph, sem)
    ranked = rank_with_policy(ctx, merged)
    return package(ctx, ranked[:ctx.limit])
```
## 359. Tier3 Reconstruction pattern 29

### Objective
Optimize tier3 reconstruction under bounded latency and bounded noise.

### Preconditions
- namespace is validated
- policy context is loaded
- latency budget is available
- tier capabilities are known

### Heuristic
1. start with the cheapest exact signals
2. cap expansion by query class
3. only pay semantic cost when exact and graph signals are insufficient
4. preserve explainability and provenance
5. update reinforcement or decay after outcome feedback

### Failure risks
- candidate explosion
- stale data bias
- low-confidence amplification
- cross-session interference

### Instrumentation
- p50/p95/p99 latency
- candidate counts before and after pruning
- success contribution estimate
- cache hit ratio
- repairability score

### Pseudocode
```text
fn pattern_359(ctx):
    budget = ctx.latency_budget
    exact = cheap_exact(ctx)
    graph = bounded_graph(ctx, exact)
    sem = maybe_semantic(ctx, budget, graph)
    merged = dedup_merge(exact, graph, sem)
    ranked = rank_with_policy(ctx, merged)
    return package(ctx, ranked[:ctx.limit])
```
## 360. Tier3 Reconstruction pattern 30

### Objective
Optimize tier3 reconstruction under bounded latency and bounded noise.

### Preconditions
- namespace is validated
- policy context is loaded
- latency budget is available
- tier capabilities are known

### Heuristic
1. start with the cheapest exact signals
2. cap expansion by query class
3. only pay semantic cost when exact and graph signals are insufficient
4. preserve explainability and provenance
5. update reinforcement or decay after outcome feedback

### Failure risks
- candidate explosion
- stale data bias
- low-confidence amplification
- cross-session interference

### Instrumentation
- p50/p95/p99 latency
- candidate counts before and after pruning
- success contribution estimate
- cache hit ratio
- repairability score

### Pseudocode
```text
fn pattern_360(ctx):
    budget = ctx.latency_budget
    exact = cheap_exact(ctx)
    graph = bounded_graph(ctx, exact)
    sem = maybe_semantic(ctx, budget, graph)
    merged = dedup_merge(exact, graph, sem)
    ranked = rank_with_policy(ctx, merged)
    return package(ctx, ranked[:ctx.limit])
```
## 361. Compaction Planning pattern 1

### Objective
Optimize compaction planning under bounded latency and bounded noise.

### Preconditions
- namespace is validated
- policy context is loaded
- latency budget is available
- tier capabilities are known

### Heuristic
1. start with the cheapest exact signals
2. cap expansion by query class
3. only pay semantic cost when exact and graph signals are insufficient
4. preserve explainability and provenance
5. update reinforcement or decay after outcome feedback

### Failure risks
- candidate explosion
- stale data bias
- low-confidence amplification
- cross-session interference

### Instrumentation
- p50/p95/p99 latency
- candidate counts before and after pruning
- success contribution estimate
- cache hit ratio
- repairability score

### Pseudocode
```text
fn pattern_361(ctx):
    budget = ctx.latency_budget
    exact = cheap_exact(ctx)
    graph = bounded_graph(ctx, exact)
    sem = maybe_semantic(ctx, budget, graph)
    merged = dedup_merge(exact, graph, sem)
    ranked = rank_with_policy(ctx, merged)
    return package(ctx, ranked[:ctx.limit])
```
## 362. Compaction Planning pattern 2

### Objective
Optimize compaction planning under bounded latency and bounded noise.

### Preconditions
- namespace is validated
- policy context is loaded
- latency budget is available
- tier capabilities are known

### Heuristic
1. start with the cheapest exact signals
2. cap expansion by query class
3. only pay semantic cost when exact and graph signals are insufficient
4. preserve explainability and provenance
5. update reinforcement or decay after outcome feedback

### Failure risks
- candidate explosion
- stale data bias
- low-confidence amplification
- cross-session interference

### Instrumentation
- p50/p95/p99 latency
- candidate counts before and after pruning
- success contribution estimate
- cache hit ratio
- repairability score

### Pseudocode
```text
fn pattern_362(ctx):
    budget = ctx.latency_budget
    exact = cheap_exact(ctx)
    graph = bounded_graph(ctx, exact)
    sem = maybe_semantic(ctx, budget, graph)
    merged = dedup_merge(exact, graph, sem)
    ranked = rank_with_policy(ctx, merged)
    return package(ctx, ranked[:ctx.limit])
```
## 363. Compaction Planning pattern 3

### Objective
Optimize compaction planning under bounded latency and bounded noise.

### Preconditions
- namespace is validated
- policy context is loaded
- latency budget is available
- tier capabilities are known

### Heuristic
1. start with the cheapest exact signals
2. cap expansion by query class
3. only pay semantic cost when exact and graph signals are insufficient
4. preserve explainability and provenance
5. update reinforcement or decay after outcome feedback

### Failure risks
- candidate explosion
- stale data bias
- low-confidence amplification
- cross-session interference

### Instrumentation
- p50/p95/p99 latency
- candidate counts before and after pruning
- success contribution estimate
- cache hit ratio
- repairability score

### Pseudocode
```text
fn pattern_363(ctx):
    budget = ctx.latency_budget
    exact = cheap_exact(ctx)
    graph = bounded_graph(ctx, exact)
    sem = maybe_semantic(ctx, budget, graph)
    merged = dedup_merge(exact, graph, sem)
    ranked = rank_with_policy(ctx, merged)
    return package(ctx, ranked[:ctx.limit])
```
## 364. Compaction Planning pattern 4

### Objective
Optimize compaction planning under bounded latency and bounded noise.

### Preconditions
- namespace is validated
- policy context is loaded
- latency budget is available
- tier capabilities are known

### Heuristic
1. start with the cheapest exact signals
2. cap expansion by query class
3. only pay semantic cost when exact and graph signals are insufficient
4. preserve explainability and provenance
5. update reinforcement or decay after outcome feedback

### Failure risks
- candidate explosion
- stale data bias
- low-confidence amplification
- cross-session interference

### Instrumentation
- p50/p95/p99 latency
- candidate counts before and after pruning
- success contribution estimate
- cache hit ratio
- repairability score

### Pseudocode
```text
fn pattern_364(ctx):
    budget = ctx.latency_budget
    exact = cheap_exact(ctx)
    graph = bounded_graph(ctx, exact)
    sem = maybe_semantic(ctx, budget, graph)
    merged = dedup_merge(exact, graph, sem)
    ranked = rank_with_policy(ctx, merged)
    return package(ctx, ranked[:ctx.limit])
```
## 365. Compaction Planning pattern 5

### Objective
Optimize compaction planning under bounded latency and bounded noise.

### Preconditions
- namespace is validated
- policy context is loaded
- latency budget is available
- tier capabilities are known

### Heuristic
1. start with the cheapest exact signals
2. cap expansion by query class
3. only pay semantic cost when exact and graph signals are insufficient
4. preserve explainability and provenance
5. update reinforcement or decay after outcome feedback

### Failure risks
- candidate explosion
- stale data bias
- low-confidence amplification
- cross-session interference

### Instrumentation
- p50/p95/p99 latency
- candidate counts before and after pruning
- success contribution estimate
- cache hit ratio
- repairability score

### Pseudocode
```text
fn pattern_365(ctx):
    budget = ctx.latency_budget
    exact = cheap_exact(ctx)
    graph = bounded_graph(ctx, exact)
    sem = maybe_semantic(ctx, budget, graph)
    merged = dedup_merge(exact, graph, sem)
    ranked = rank_with_policy(ctx, merged)
    return package(ctx, ranked[:ctx.limit])
```
## 366. Compaction Planning pattern 6

### Objective
Optimize compaction planning under bounded latency and bounded noise.

### Preconditions
- namespace is validated
- policy context is loaded
- latency budget is available
- tier capabilities are known

### Heuristic
1. start with the cheapest exact signals
2. cap expansion by query class
3. only pay semantic cost when exact and graph signals are insufficient
4. preserve explainability and provenance
5. update reinforcement or decay after outcome feedback

### Failure risks
- candidate explosion
- stale data bias
- low-confidence amplification
- cross-session interference

### Instrumentation
- p50/p95/p99 latency
- candidate counts before and after pruning
- success contribution estimate
- cache hit ratio
- repairability score

### Pseudocode
```text
fn pattern_366(ctx):
    budget = ctx.latency_budget
    exact = cheap_exact(ctx)
    graph = bounded_graph(ctx, exact)
    sem = maybe_semantic(ctx, budget, graph)
    merged = dedup_merge(exact, graph, sem)
    ranked = rank_with_policy(ctx, merged)
    return package(ctx, ranked[:ctx.limit])
```
## 367. Compaction Planning pattern 7

### Objective
Optimize compaction planning under bounded latency and bounded noise.

### Preconditions
- namespace is validated
- policy context is loaded
- latency budget is available
- tier capabilities are known

### Heuristic
1. start with the cheapest exact signals
2. cap expansion by query class
3. only pay semantic cost when exact and graph signals are insufficient
4. preserve explainability and provenance
5. update reinforcement or decay after outcome feedback

### Failure risks
- candidate explosion
- stale data bias
- low-confidence amplification
- cross-session interference

### Instrumentation
- p50/p95/p99 latency
- candidate counts before and after pruning
- success contribution estimate
- cache hit ratio
- repairability score

### Pseudocode
```text
fn pattern_367(ctx):
    budget = ctx.latency_budget
    exact = cheap_exact(ctx)
    graph = bounded_graph(ctx, exact)
    sem = maybe_semantic(ctx, budget, graph)
    merged = dedup_merge(exact, graph, sem)
    ranked = rank_with_policy(ctx, merged)
    return package(ctx, ranked[:ctx.limit])
```
## 368. Compaction Planning pattern 8

### Objective
Optimize compaction planning under bounded latency and bounded noise.

### Preconditions
- namespace is validated
- policy context is loaded
- latency budget is available
- tier capabilities are known

### Heuristic
1. start with the cheapest exact signals
2. cap expansion by query class
3. only pay semantic cost when exact and graph signals are insufficient
4. preserve explainability and provenance
5. update reinforcement or decay after outcome feedback

### Failure risks
- candidate explosion
- stale data bias
- low-confidence amplification
- cross-session interference

### Instrumentation
- p50/p95/p99 latency
- candidate counts before and after pruning
- success contribution estimate
- cache hit ratio
- repairability score

### Pseudocode
```text
fn pattern_368(ctx):
    budget = ctx.latency_budget
    exact = cheap_exact(ctx)
    graph = bounded_graph(ctx, exact)
    sem = maybe_semantic(ctx, budget, graph)
    merged = dedup_merge(exact, graph, sem)
    ranked = rank_with_policy(ctx, merged)
    return package(ctx, ranked[:ctx.limit])
```
## 369. Compaction Planning pattern 9

### Objective
Optimize compaction planning under bounded latency and bounded noise.

### Preconditions
- namespace is validated
- policy context is loaded
- latency budget is available
- tier capabilities are known

### Heuristic
1. start with the cheapest exact signals
2. cap expansion by query class
3. only pay semantic cost when exact and graph signals are insufficient
4. preserve explainability and provenance
5. update reinforcement or decay after outcome feedback

### Failure risks
- candidate explosion
- stale data bias
- low-confidence amplification
- cross-session interference

### Instrumentation
- p50/p95/p99 latency
- candidate counts before and after pruning
- success contribution estimate
- cache hit ratio
- repairability score

### Pseudocode
```text
fn pattern_369(ctx):
    budget = ctx.latency_budget
    exact = cheap_exact(ctx)
    graph = bounded_graph(ctx, exact)
    sem = maybe_semantic(ctx, budget, graph)
    merged = dedup_merge(exact, graph, sem)
    ranked = rank_with_policy(ctx, merged)
    return package(ctx, ranked[:ctx.limit])
```
## 370. Compaction Planning pattern 10

### Objective
Optimize compaction planning under bounded latency and bounded noise.

### Preconditions
- namespace is validated
- policy context is loaded
- latency budget is available
- tier capabilities are known

### Heuristic
1. start with the cheapest exact signals
2. cap expansion by query class
3. only pay semantic cost when exact and graph signals are insufficient
4. preserve explainability and provenance
5. update reinforcement or decay after outcome feedback

### Failure risks
- candidate explosion
- stale data bias
- low-confidence amplification
- cross-session interference

### Instrumentation
- p50/p95/p99 latency
- candidate counts before and after pruning
- success contribution estimate
- cache hit ratio
- repairability score

### Pseudocode
```text
fn pattern_370(ctx):
    budget = ctx.latency_budget
    exact = cheap_exact(ctx)
    graph = bounded_graph(ctx, exact)
    sem = maybe_semantic(ctx, budget, graph)
    merged = dedup_merge(exact, graph, sem)
    ranked = rank_with_policy(ctx, merged)
    return package(ctx, ranked[:ctx.limit])
```
## 371. Compaction Planning pattern 11

### Objective
Optimize compaction planning under bounded latency and bounded noise.

### Preconditions
- namespace is validated
- policy context is loaded
- latency budget is available
- tier capabilities are known

### Heuristic
1. start with the cheapest exact signals
2. cap expansion by query class
3. only pay semantic cost when exact and graph signals are insufficient
4. preserve explainability and provenance
5. update reinforcement or decay after outcome feedback

### Failure risks
- candidate explosion
- stale data bias
- low-confidence amplification
- cross-session interference

### Instrumentation
- p50/p95/p99 latency
- candidate counts before and after pruning
- success contribution estimate
- cache hit ratio
- repairability score

### Pseudocode
```text
fn pattern_371(ctx):
    budget = ctx.latency_budget
    exact = cheap_exact(ctx)
    graph = bounded_graph(ctx, exact)
    sem = maybe_semantic(ctx, budget, graph)
    merged = dedup_merge(exact, graph, sem)
    ranked = rank_with_policy(ctx, merged)
    return package(ctx, ranked[:ctx.limit])
```
## 372. Compaction Planning pattern 12

### Objective
Optimize compaction planning under bounded latency and bounded noise.

### Preconditions
- namespace is validated
- policy context is loaded
- latency budget is available
- tier capabilities are known

### Heuristic
1. start with the cheapest exact signals
2. cap expansion by query class
3. only pay semantic cost when exact and graph signals are insufficient
4. preserve explainability and provenance
5. update reinforcement or decay after outcome feedback

### Failure risks
- candidate explosion
- stale data bias
- low-confidence amplification
- cross-session interference

### Instrumentation
- p50/p95/p99 latency
- candidate counts before and after pruning
- success contribution estimate
- cache hit ratio
- repairability score

### Pseudocode
```text
fn pattern_372(ctx):
    budget = ctx.latency_budget
    exact = cheap_exact(ctx)
    graph = bounded_graph(ctx, exact)
    sem = maybe_semantic(ctx, budget, graph)
    merged = dedup_merge(exact, graph, sem)
    ranked = rank_with_policy(ctx, merged)
    return package(ctx, ranked[:ctx.limit])
```
## 373. Compaction Planning pattern 13

### Objective
Optimize compaction planning under bounded latency and bounded noise.

### Preconditions
- namespace is validated
- policy context is loaded
- latency budget is available
- tier capabilities are known

### Heuristic
1. start with the cheapest exact signals
2. cap expansion by query class
3. only pay semantic cost when exact and graph signals are insufficient
4. preserve explainability and provenance
5. update reinforcement or decay after outcome feedback

### Failure risks
- candidate explosion
- stale data bias
- low-confidence amplification
- cross-session interference

### Instrumentation
- p50/p95/p99 latency
- candidate counts before and after pruning
- success contribution estimate
- cache hit ratio
- repairability score

### Pseudocode
```text
fn pattern_373(ctx):
    budget = ctx.latency_budget
    exact = cheap_exact(ctx)
    graph = bounded_graph(ctx, exact)
    sem = maybe_semantic(ctx, budget, graph)
    merged = dedup_merge(exact, graph, sem)
    ranked = rank_with_policy(ctx, merged)
    return package(ctx, ranked[:ctx.limit])
```
## 374. Compaction Planning pattern 14

### Objective
Optimize compaction planning under bounded latency and bounded noise.

### Preconditions
- namespace is validated
- policy context is loaded
- latency budget is available
- tier capabilities are known

### Heuristic
1. start with the cheapest exact signals
2. cap expansion by query class
3. only pay semantic cost when exact and graph signals are insufficient
4. preserve explainability and provenance
5. update reinforcement or decay after outcome feedback

### Failure risks
- candidate explosion
- stale data bias
- low-confidence amplification
- cross-session interference

### Instrumentation
- p50/p95/p99 latency
- candidate counts before and after pruning
- success contribution estimate
- cache hit ratio
- repairability score

### Pseudocode
```text
fn pattern_374(ctx):
    budget = ctx.latency_budget
    exact = cheap_exact(ctx)
    graph = bounded_graph(ctx, exact)
    sem = maybe_semantic(ctx, budget, graph)
    merged = dedup_merge(exact, graph, sem)
    ranked = rank_with_policy(ctx, merged)
    return package(ctx, ranked[:ctx.limit])
```
## 375. Compaction Planning pattern 15

### Objective
Optimize compaction planning under bounded latency and bounded noise.

### Preconditions
- namespace is validated
- policy context is loaded
- latency budget is available
- tier capabilities are known

### Heuristic
1. start with the cheapest exact signals
2. cap expansion by query class
3. only pay semantic cost when exact and graph signals are insufficient
4. preserve explainability and provenance
5. update reinforcement or decay after outcome feedback

### Failure risks
- candidate explosion
- stale data bias
- low-confidence amplification
- cross-session interference

### Instrumentation
- p50/p95/p99 latency
- candidate counts before and after pruning
- success contribution estimate
- cache hit ratio
- repairability score

### Pseudocode
```text
fn pattern_375(ctx):
    budget = ctx.latency_budget
    exact = cheap_exact(ctx)
    graph = bounded_graph(ctx, exact)
    sem = maybe_semantic(ctx, budget, graph)
    merged = dedup_merge(exact, graph, sem)
    ranked = rank_with_policy(ctx, merged)
    return package(ctx, ranked[:ctx.limit])
```
## 376. Compaction Planning pattern 16

### Objective
Optimize compaction planning under bounded latency and bounded noise.

### Preconditions
- namespace is validated
- policy context is loaded
- latency budget is available
- tier capabilities are known

### Heuristic
1. start with the cheapest exact signals
2. cap expansion by query class
3. only pay semantic cost when exact and graph signals are insufficient
4. preserve explainability and provenance
5. update reinforcement or decay after outcome feedback

### Failure risks
- candidate explosion
- stale data bias
- low-confidence amplification
- cross-session interference

### Instrumentation
- p50/p95/p99 latency
- candidate counts before and after pruning
- success contribution estimate
- cache hit ratio
- repairability score

### Pseudocode
```text
fn pattern_376(ctx):
    budget = ctx.latency_budget
    exact = cheap_exact(ctx)
    graph = bounded_graph(ctx, exact)
    sem = maybe_semantic(ctx, budget, graph)
    merged = dedup_merge(exact, graph, sem)
    ranked = rank_with_policy(ctx, merged)
    return package(ctx, ranked[:ctx.limit])
```
## 377. Compaction Planning pattern 17

### Objective
Optimize compaction planning under bounded latency and bounded noise.

### Preconditions
- namespace is validated
- policy context is loaded
- latency budget is available
- tier capabilities are known

### Heuristic
1. start with the cheapest exact signals
2. cap expansion by query class
3. only pay semantic cost when exact and graph signals are insufficient
4. preserve explainability and provenance
5. update reinforcement or decay after outcome feedback

### Failure risks
- candidate explosion
- stale data bias
- low-confidence amplification
- cross-session interference

### Instrumentation
- p50/p95/p99 latency
- candidate counts before and after pruning
- success contribution estimate
- cache hit ratio
- repairability score

### Pseudocode
```text
fn pattern_377(ctx):
    budget = ctx.latency_budget
    exact = cheap_exact(ctx)
    graph = bounded_graph(ctx, exact)
    sem = maybe_semantic(ctx, budget, graph)
    merged = dedup_merge(exact, graph, sem)
    ranked = rank_with_policy(ctx, merged)
    return package(ctx, ranked[:ctx.limit])
```
## 378. Compaction Planning pattern 18

### Objective
Optimize compaction planning under bounded latency and bounded noise.

### Preconditions
- namespace is validated
- policy context is loaded
- latency budget is available
- tier capabilities are known

### Heuristic
1. start with the cheapest exact signals
2. cap expansion by query class
3. only pay semantic cost when exact and graph signals are insufficient
4. preserve explainability and provenance
5. update reinforcement or decay after outcome feedback

### Failure risks
- candidate explosion
- stale data bias
- low-confidence amplification
- cross-session interference

### Instrumentation
- p50/p95/p99 latency
- candidate counts before and after pruning
- success contribution estimate
- cache hit ratio
- repairability score

### Pseudocode
```text
fn pattern_378(ctx):
    budget = ctx.latency_budget
    exact = cheap_exact(ctx)
    graph = bounded_graph(ctx, exact)
    sem = maybe_semantic(ctx, budget, graph)
    merged = dedup_merge(exact, graph, sem)
    ranked = rank_with_policy(ctx, merged)
    return package(ctx, ranked[:ctx.limit])
```
## 379. Compaction Planning pattern 19

### Objective
Optimize compaction planning under bounded latency and bounded noise.

### Preconditions
- namespace is validated
- policy context is loaded
- latency budget is available
- tier capabilities are known

### Heuristic
1. start with the cheapest exact signals
2. cap expansion by query class
3. only pay semantic cost when exact and graph signals are insufficient
4. preserve explainability and provenance
5. update reinforcement or decay after outcome feedback

### Failure risks
- candidate explosion
- stale data bias
- low-confidence amplification
- cross-session interference

### Instrumentation
- p50/p95/p99 latency
- candidate counts before and after pruning
- success contribution estimate
- cache hit ratio
- repairability score

### Pseudocode
```text
fn pattern_379(ctx):
    budget = ctx.latency_budget
    exact = cheap_exact(ctx)
    graph = bounded_graph(ctx, exact)
    sem = maybe_semantic(ctx, budget, graph)
    merged = dedup_merge(exact, graph, sem)
    ranked = rank_with_policy(ctx, merged)
    return package(ctx, ranked[:ctx.limit])
```
## 380. Compaction Planning pattern 20

### Objective
Optimize compaction planning under bounded latency and bounded noise.

### Preconditions
- namespace is validated
- policy context is loaded
- latency budget is available
- tier capabilities are known

### Heuristic
1. start with the cheapest exact signals
2. cap expansion by query class
3. only pay semantic cost when exact and graph signals are insufficient
4. preserve explainability and provenance
5. update reinforcement or decay after outcome feedback

### Failure risks
- candidate explosion
- stale data bias
- low-confidence amplification
- cross-session interference

### Instrumentation
- p50/p95/p99 latency
- candidate counts before and after pruning
- success contribution estimate
- cache hit ratio
- repairability score

### Pseudocode
```text
fn pattern_380(ctx):
    budget = ctx.latency_budget
    exact = cheap_exact(ctx)
    graph = bounded_graph(ctx, exact)
    sem = maybe_semantic(ctx, budget, graph)
    merged = dedup_merge(exact, graph, sem)
    ranked = rank_with_policy(ctx, merged)
    return package(ctx, ranked[:ctx.limit])
```
## 381. Compaction Planning pattern 21

### Objective
Optimize compaction planning under bounded latency and bounded noise.

### Preconditions
- namespace is validated
- policy context is loaded
- latency budget is available
- tier capabilities are known

### Heuristic
1. start with the cheapest exact signals
2. cap expansion by query class
3. only pay semantic cost when exact and graph signals are insufficient
4. preserve explainability and provenance
5. update reinforcement or decay after outcome feedback

### Failure risks
- candidate explosion
- stale data bias
- low-confidence amplification
- cross-session interference

### Instrumentation
- p50/p95/p99 latency
- candidate counts before and after pruning
- success contribution estimate
- cache hit ratio
- repairability score

### Pseudocode
```text
fn pattern_381(ctx):
    budget = ctx.latency_budget
    exact = cheap_exact(ctx)
    graph = bounded_graph(ctx, exact)
    sem = maybe_semantic(ctx, budget, graph)
    merged = dedup_merge(exact, graph, sem)
    ranked = rank_with_policy(ctx, merged)
    return package(ctx, ranked[:ctx.limit])
```
## 382. Compaction Planning pattern 22

### Objective
Optimize compaction planning under bounded latency and bounded noise.

### Preconditions
- namespace is validated
- policy context is loaded
- latency budget is available
- tier capabilities are known

### Heuristic
1. start with the cheapest exact signals
2. cap expansion by query class
3. only pay semantic cost when exact and graph signals are insufficient
4. preserve explainability and provenance
5. update reinforcement or decay after outcome feedback

### Failure risks
- candidate explosion
- stale data bias
- low-confidence amplification
- cross-session interference

### Instrumentation
- p50/p95/p99 latency
- candidate counts before and after pruning
- success contribution estimate
- cache hit ratio
- repairability score

### Pseudocode
```text
fn pattern_382(ctx):
    budget = ctx.latency_budget
    exact = cheap_exact(ctx)
    graph = bounded_graph(ctx, exact)
    sem = maybe_semantic(ctx, budget, graph)
    merged = dedup_merge(exact, graph, sem)
    ranked = rank_with_policy(ctx, merged)
    return package(ctx, ranked[:ctx.limit])
```
## 383. Compaction Planning pattern 23

### Objective
Optimize compaction planning under bounded latency and bounded noise.

### Preconditions
- namespace is validated
- policy context is loaded
- latency budget is available
- tier capabilities are known

### Heuristic
1. start with the cheapest exact signals
2. cap expansion by query class
3. only pay semantic cost when exact and graph signals are insufficient
4. preserve explainability and provenance
5. update reinforcement or decay after outcome feedback

### Failure risks
- candidate explosion
- stale data bias
- low-confidence amplification
- cross-session interference

### Instrumentation
- p50/p95/p99 latency
- candidate counts before and after pruning
- success contribution estimate
- cache hit ratio
- repairability score

### Pseudocode
```text
fn pattern_383(ctx):
    budget = ctx.latency_budget
    exact = cheap_exact(ctx)
    graph = bounded_graph(ctx, exact)
    sem = maybe_semantic(ctx, budget, graph)
    merged = dedup_merge(exact, graph, sem)
    ranked = rank_with_policy(ctx, merged)
    return package(ctx, ranked[:ctx.limit])
```
## 384. Compaction Planning pattern 24

### Objective
Optimize compaction planning under bounded latency and bounded noise.

### Preconditions
- namespace is validated
- policy context is loaded
- latency budget is available
- tier capabilities are known

### Heuristic
1. start with the cheapest exact signals
2. cap expansion by query class
3. only pay semantic cost when exact and graph signals are insufficient
4. preserve explainability and provenance
5. update reinforcement or decay after outcome feedback

### Failure risks
- candidate explosion
- stale data bias
- low-confidence amplification
- cross-session interference

### Instrumentation
- p50/p95/p99 latency
- candidate counts before and after pruning
- success contribution estimate
- cache hit ratio
- repairability score

### Pseudocode
```text
fn pattern_384(ctx):
    budget = ctx.latency_budget
    exact = cheap_exact(ctx)
    graph = bounded_graph(ctx, exact)
    sem = maybe_semantic(ctx, budget, graph)
    merged = dedup_merge(exact, graph, sem)
    ranked = rank_with_policy(ctx, merged)
    return package(ctx, ranked[:ctx.limit])
```
## 385. Compaction Planning pattern 25

### Objective
Optimize compaction planning under bounded latency and bounded noise.

### Preconditions
- namespace is validated
- policy context is loaded
- latency budget is available
- tier capabilities are known

### Heuristic
1. start with the cheapest exact signals
2. cap expansion by query class
3. only pay semantic cost when exact and graph signals are insufficient
4. preserve explainability and provenance
5. update reinforcement or decay after outcome feedback

### Failure risks
- candidate explosion
- stale data bias
- low-confidence amplification
- cross-session interference

### Instrumentation
- p50/p95/p99 latency
- candidate counts before and after pruning
- success contribution estimate
- cache hit ratio
- repairability score

### Pseudocode
```text
fn pattern_385(ctx):
    budget = ctx.latency_budget
    exact = cheap_exact(ctx)
    graph = bounded_graph(ctx, exact)
    sem = maybe_semantic(ctx, budget, graph)
    merged = dedup_merge(exact, graph, sem)
    ranked = rank_with_policy(ctx, merged)
    return package(ctx, ranked[:ctx.limit])
```
## 386. Compaction Planning pattern 26

### Objective
Optimize compaction planning under bounded latency and bounded noise.

### Preconditions
- namespace is validated
- policy context is loaded
- latency budget is available
- tier capabilities are known

### Heuristic
1. start with the cheapest exact signals
2. cap expansion by query class
3. only pay semantic cost when exact and graph signals are insufficient
4. preserve explainability and provenance
5. update reinforcement or decay after outcome feedback

### Failure risks
- candidate explosion
- stale data bias
- low-confidence amplification
- cross-session interference

### Instrumentation
- p50/p95/p99 latency
- candidate counts before and after pruning
- success contribution estimate
- cache hit ratio
- repairability score

### Pseudocode
```text
fn pattern_386(ctx):
    budget = ctx.latency_budget
    exact = cheap_exact(ctx)
    graph = bounded_graph(ctx, exact)
    sem = maybe_semantic(ctx, budget, graph)
    merged = dedup_merge(exact, graph, sem)
    ranked = rank_with_policy(ctx, merged)
    return package(ctx, ranked[:ctx.limit])
```
## 387. Compaction Planning pattern 27

### Objective
Optimize compaction planning under bounded latency and bounded noise.

### Preconditions
- namespace is validated
- policy context is loaded
- latency budget is available
- tier capabilities are known

### Heuristic
1. start with the cheapest exact signals
2. cap expansion by query class
3. only pay semantic cost when exact and graph signals are insufficient
4. preserve explainability and provenance
5. update reinforcement or decay after outcome feedback

### Failure risks
- candidate explosion
- stale data bias
- low-confidence amplification
- cross-session interference

### Instrumentation
- p50/p95/p99 latency
- candidate counts before and after pruning
- success contribution estimate
- cache hit ratio
- repairability score

### Pseudocode
```text
fn pattern_387(ctx):
    budget = ctx.latency_budget
    exact = cheap_exact(ctx)
    graph = bounded_graph(ctx, exact)
    sem = maybe_semantic(ctx, budget, graph)
    merged = dedup_merge(exact, graph, sem)
    ranked = rank_with_policy(ctx, merged)
    return package(ctx, ranked[:ctx.limit])
```
## 388. Compaction Planning pattern 28

### Objective
Optimize compaction planning under bounded latency and bounded noise.

### Preconditions
- namespace is validated
- policy context is loaded
- latency budget is available
- tier capabilities are known

### Heuristic
1. start with the cheapest exact signals
2. cap expansion by query class
3. only pay semantic cost when exact and graph signals are insufficient
4. preserve explainability and provenance
5. update reinforcement or decay after outcome feedback

### Failure risks
- candidate explosion
- stale data bias
- low-confidence amplification
- cross-session interference

### Instrumentation
- p50/p95/p99 latency
- candidate counts before and after pruning
- success contribution estimate
- cache hit ratio
- repairability score

### Pseudocode
```text
fn pattern_388(ctx):
    budget = ctx.latency_budget
    exact = cheap_exact(ctx)
    graph = bounded_graph(ctx, exact)
    sem = maybe_semantic(ctx, budget, graph)
    merged = dedup_merge(exact, graph, sem)
    ranked = rank_with_policy(ctx, merged)
    return package(ctx, ranked[:ctx.limit])
```
## 389. Compaction Planning pattern 29

### Objective
Optimize compaction planning under bounded latency and bounded noise.

### Preconditions
- namespace is validated
- policy context is loaded
- latency budget is available
- tier capabilities are known

### Heuristic
1. start with the cheapest exact signals
2. cap expansion by query class
3. only pay semantic cost when exact and graph signals are insufficient
4. preserve explainability and provenance
5. update reinforcement or decay after outcome feedback

### Failure risks
- candidate explosion
- stale data bias
- low-confidence amplification
- cross-session interference

### Instrumentation
- p50/p95/p99 latency
- candidate counts before and after pruning
- success contribution estimate
- cache hit ratio
- repairability score

### Pseudocode
```text
fn pattern_389(ctx):
    budget = ctx.latency_budget
    exact = cheap_exact(ctx)
    graph = bounded_graph(ctx, exact)
    sem = maybe_semantic(ctx, budget, graph)
    merged = dedup_merge(exact, graph, sem)
    ranked = rank_with_policy(ctx, merged)
    return package(ctx, ranked[:ctx.limit])
```
## 390. Compaction Planning pattern 30

### Objective
Optimize compaction planning under bounded latency and bounded noise.

### Preconditions
- namespace is validated
- policy context is loaded
- latency budget is available
- tier capabilities are known

### Heuristic
1. start with the cheapest exact signals
2. cap expansion by query class
3. only pay semantic cost when exact and graph signals are insufficient
4. preserve explainability and provenance
5. update reinforcement or decay after outcome feedback

### Failure risks
- candidate explosion
- stale data bias
- low-confidence amplification
- cross-session interference

### Instrumentation
- p50/p95/p99 latency
- candidate counts before and after pruning
- success contribution estimate
- cache hit ratio
- repairability score

### Pseudocode
```text
fn pattern_390(ctx):
    budget = ctx.latency_budget
    exact = cheap_exact(ctx)
    graph = bounded_graph(ctx, exact)
    sem = maybe_semantic(ctx, budget, graph)
    merged = dedup_merge(exact, graph, sem)
    ranked = rank_with_policy(ctx, merged)
    return package(ctx, ranked[:ctx.limit])
```
## 391. Shard Rebalancing pattern 1

### Objective
Optimize shard rebalancing under bounded latency and bounded noise.

### Preconditions
- namespace is validated
- policy context is loaded
- latency budget is available
- tier capabilities are known

### Heuristic
1. start with the cheapest exact signals
2. cap expansion by query class
3. only pay semantic cost when exact and graph signals are insufficient
4. preserve explainability and provenance
5. update reinforcement or decay after outcome feedback

### Failure risks
- candidate explosion
- stale data bias
- low-confidence amplification
- cross-session interference

### Instrumentation
- p50/p95/p99 latency
- candidate counts before and after pruning
- success contribution estimate
- cache hit ratio
- repairability score

### Pseudocode
```text
fn pattern_391(ctx):
    budget = ctx.latency_budget
    exact = cheap_exact(ctx)
    graph = bounded_graph(ctx, exact)
    sem = maybe_semantic(ctx, budget, graph)
    merged = dedup_merge(exact, graph, sem)
    ranked = rank_with_policy(ctx, merged)
    return package(ctx, ranked[:ctx.limit])
```
## 392. Shard Rebalancing pattern 2

### Objective
Optimize shard rebalancing under bounded latency and bounded noise.

### Preconditions
- namespace is validated
- policy context is loaded
- latency budget is available
- tier capabilities are known

### Heuristic
1. start with the cheapest exact signals
2. cap expansion by query class
3. only pay semantic cost when exact and graph signals are insufficient
4. preserve explainability and provenance
5. update reinforcement or decay after outcome feedback

### Failure risks
- candidate explosion
- stale data bias
- low-confidence amplification
- cross-session interference

### Instrumentation
- p50/p95/p99 latency
- candidate counts before and after pruning
- success contribution estimate
- cache hit ratio
- repairability score

### Pseudocode
```text
fn pattern_392(ctx):
    budget = ctx.latency_budget
    exact = cheap_exact(ctx)
    graph = bounded_graph(ctx, exact)
    sem = maybe_semantic(ctx, budget, graph)
    merged = dedup_merge(exact, graph, sem)
    ranked = rank_with_policy(ctx, merged)
    return package(ctx, ranked[:ctx.limit])
```
## 393. Shard Rebalancing pattern 3

### Objective
Optimize shard rebalancing under bounded latency and bounded noise.

### Preconditions
- namespace is validated
- policy context is loaded
- latency budget is available
- tier capabilities are known

### Heuristic
1. start with the cheapest exact signals
2. cap expansion by query class
3. only pay semantic cost when exact and graph signals are insufficient
4. preserve explainability and provenance
5. update reinforcement or decay after outcome feedback

### Failure risks
- candidate explosion
- stale data bias
- low-confidence amplification
- cross-session interference

### Instrumentation
- p50/p95/p99 latency
- candidate counts before and after pruning
- success contribution estimate
- cache hit ratio
- repairability score

### Pseudocode
```text
fn pattern_393(ctx):
    budget = ctx.latency_budget
    exact = cheap_exact(ctx)
    graph = bounded_graph(ctx, exact)
    sem = maybe_semantic(ctx, budget, graph)
    merged = dedup_merge(exact, graph, sem)
    ranked = rank_with_policy(ctx, merged)
    return package(ctx, ranked[:ctx.limit])
```
## 394. Shard Rebalancing pattern 4

### Objective
Optimize shard rebalancing under bounded latency and bounded noise.

### Preconditions
- namespace is validated
- policy context is loaded
- latency budget is available
- tier capabilities are known

### Heuristic
1. start with the cheapest exact signals
2. cap expansion by query class
3. only pay semantic cost when exact and graph signals are insufficient
4. preserve explainability and provenance
5. update reinforcement or decay after outcome feedback

### Failure risks
- candidate explosion
- stale data bias
- low-confidence amplification
- cross-session interference

### Instrumentation
- p50/p95/p99 latency
- candidate counts before and after pruning
- success contribution estimate
- cache hit ratio
- repairability score

### Pseudocode
```text
fn pattern_394(ctx):
    budget = ctx.latency_budget
    exact = cheap_exact(ctx)
    graph = bounded_graph(ctx, exact)
    sem = maybe_semantic(ctx, budget, graph)
    merged = dedup_merge(exact, graph, sem)
    ranked = rank_with_policy(ctx, merged)
    return package(ctx, ranked[:ctx.limit])
```
## 395. Shard Rebalancing pattern 5

### Objective
Optimize shard rebalancing under bounded latency and bounded noise.

### Preconditions
- namespace is validated
- policy context is loaded
- latency budget is available
- tier capabilities are known

### Heuristic
1. start with the cheapest exact signals
2. cap expansion by query class
3. only pay semantic cost when exact and graph signals are insufficient
4. preserve explainability and provenance
5. update reinforcement or decay after outcome feedback

### Failure risks
- candidate explosion
- stale data bias
- low-confidence amplification
- cross-session interference

### Instrumentation
- p50/p95/p99 latency
- candidate counts before and after pruning
- success contribution estimate
- cache hit ratio
- repairability score

### Pseudocode
```text
fn pattern_395(ctx):
    budget = ctx.latency_budget
    exact = cheap_exact(ctx)
    graph = bounded_graph(ctx, exact)
    sem = maybe_semantic(ctx, budget, graph)
    merged = dedup_merge(exact, graph, sem)
    ranked = rank_with_policy(ctx, merged)
    return package(ctx, ranked[:ctx.limit])
```
## 396. Shard Rebalancing pattern 6

### Objective
Optimize shard rebalancing under bounded latency and bounded noise.

### Preconditions
- namespace is validated
- policy context is loaded
- latency budget is available
- tier capabilities are known

### Heuristic
1. start with the cheapest exact signals
2. cap expansion by query class
3. only pay semantic cost when exact and graph signals are insufficient
4. preserve explainability and provenance
5. update reinforcement or decay after outcome feedback

### Failure risks
- candidate explosion
- stale data bias
- low-confidence amplification
- cross-session interference

### Instrumentation
- p50/p95/p99 latency
- candidate counts before and after pruning
- success contribution estimate
- cache hit ratio
- repairability score

### Pseudocode
```text
fn pattern_396(ctx):
    budget = ctx.latency_budget
    exact = cheap_exact(ctx)
    graph = bounded_graph(ctx, exact)
    sem = maybe_semantic(ctx, budget, graph)
    merged = dedup_merge(exact, graph, sem)
    ranked = rank_with_policy(ctx, merged)
    return package(ctx, ranked[:ctx.limit])
```
## 397. Shard Rebalancing pattern 7

### Objective
Optimize shard rebalancing under bounded latency and bounded noise.

### Preconditions
- namespace is validated
- policy context is loaded
- latency budget is available
- tier capabilities are known

### Heuristic
1. start with the cheapest exact signals
2. cap expansion by query class
3. only pay semantic cost when exact and graph signals are insufficient
4. preserve explainability and provenance
5. update reinforcement or decay after outcome feedback

### Failure risks
- candidate explosion
- stale data bias
- low-confidence amplification
- cross-session interference

### Instrumentation
- p50/p95/p99 latency
- candidate counts before and after pruning
- success contribution estimate
- cache hit ratio
- repairability score

### Pseudocode
```text
fn pattern_397(ctx):
    budget = ctx.latency_budget
    exact = cheap_exact(ctx)
    graph = bounded_graph(ctx, exact)
    sem = maybe_semantic(ctx, budget, graph)
    merged = dedup_merge(exact, graph, sem)
    ranked = rank_with_policy(ctx, merged)
    return package(ctx, ranked[:ctx.limit])
```
## 398. Shard Rebalancing pattern 8

### Objective
Optimize shard rebalancing under bounded latency and bounded noise.

### Preconditions
- namespace is validated
- policy context is loaded
- latency budget is available
- tier capabilities are known

### Heuristic
1. start with the cheapest exact signals
2. cap expansion by query class
3. only pay semantic cost when exact and graph signals are insufficient
4. preserve explainability and provenance
5. update reinforcement or decay after outcome feedback

### Failure risks
- candidate explosion
- stale data bias
- low-confidence amplification
- cross-session interference

### Instrumentation
- p50/p95/p99 latency
- candidate counts before and after pruning
- success contribution estimate
- cache hit ratio
- repairability score

### Pseudocode
```text
fn pattern_398(ctx):
    budget = ctx.latency_budget
    exact = cheap_exact(ctx)
    graph = bounded_graph(ctx, exact)
    sem = maybe_semantic(ctx, budget, graph)
    merged = dedup_merge(exact, graph, sem)
    ranked = rank_with_policy(ctx, merged)
    return package(ctx, ranked[:ctx.limit])
```
## 399. Shard Rebalancing pattern 9

### Objective
Optimize shard rebalancing under bounded latency and bounded noise.

### Preconditions
- namespace is validated
- policy context is loaded
- latency budget is available
- tier capabilities are known

### Heuristic
1. start with the cheapest exact signals
2. cap expansion by query class
3. only pay semantic cost when exact and graph signals are insufficient
4. preserve explainability and provenance
5. update reinforcement or decay after outcome feedback

### Failure risks
- candidate explosion
- stale data bias
- low-confidence amplification
- cross-session interference

### Instrumentation
- p50/p95/p99 latency
- candidate counts before and after pruning
- success contribution estimate
- cache hit ratio
- repairability score

### Pseudocode
```text
fn pattern_399(ctx):
    budget = ctx.latency_budget
    exact = cheap_exact(ctx)
    graph = bounded_graph(ctx, exact)
    sem = maybe_semantic(ctx, budget, graph)
    merged = dedup_merge(exact, graph, sem)
    ranked = rank_with_policy(ctx, merged)
    return package(ctx, ranked[:ctx.limit])
```
## 400. Shard Rebalancing pattern 10

### Objective
Optimize shard rebalancing under bounded latency and bounded noise.

### Preconditions
- namespace is validated
- policy context is loaded
- latency budget is available
- tier capabilities are known

### Heuristic
1. start with the cheapest exact signals
2. cap expansion by query class
3. only pay semantic cost when exact and graph signals are insufficient
4. preserve explainability and provenance
5. update reinforcement or decay after outcome feedback

### Failure risks
- candidate explosion
- stale data bias
- low-confidence amplification
- cross-session interference

### Instrumentation
- p50/p95/p99 latency
- candidate counts before and after pruning
- success contribution estimate
- cache hit ratio
- repairability score

### Pseudocode
```text
fn pattern_400(ctx):
    budget = ctx.latency_budget
    exact = cheap_exact(ctx)
    graph = bounded_graph(ctx, exact)
    sem = maybe_semantic(ctx, budget, graph)
    merged = dedup_merge(exact, graph, sem)
    ranked = rank_with_policy(ctx, merged)
    return package(ctx, ranked[:ctx.limit])
```
## 401. Shard Rebalancing pattern 11

### Objective
Optimize shard rebalancing under bounded latency and bounded noise.

### Preconditions
- namespace is validated
- policy context is loaded
- latency budget is available
- tier capabilities are known

### Heuristic
1. start with the cheapest exact signals
2. cap expansion by query class
3. only pay semantic cost when exact and graph signals are insufficient
4. preserve explainability and provenance
5. update reinforcement or decay after outcome feedback

### Failure risks
- candidate explosion
- stale data bias
- low-confidence amplification
- cross-session interference

### Instrumentation
- p50/p95/p99 latency
- candidate counts before and after pruning
- success contribution estimate
- cache hit ratio
- repairability score

### Pseudocode
```text
fn pattern_401(ctx):
    budget = ctx.latency_budget
    exact = cheap_exact(ctx)
    graph = bounded_graph(ctx, exact)
    sem = maybe_semantic(ctx, budget, graph)
    merged = dedup_merge(exact, graph, sem)
    ranked = rank_with_policy(ctx, merged)
    return package(ctx, ranked[:ctx.limit])
```
## 402. Shard Rebalancing pattern 12

### Objective
Optimize shard rebalancing under bounded latency and bounded noise.

### Preconditions
- namespace is validated
- policy context is loaded
- latency budget is available
- tier capabilities are known

### Heuristic
1. start with the cheapest exact signals
2. cap expansion by query class
3. only pay semantic cost when exact and graph signals are insufficient
4. preserve explainability and provenance
5. update reinforcement or decay after outcome feedback

### Failure risks
- candidate explosion
- stale data bias
- low-confidence amplification
- cross-session interference

### Instrumentation
- p50/p95/p99 latency
- candidate counts before and after pruning
- success contribution estimate
- cache hit ratio
- repairability score

### Pseudocode
```text
fn pattern_402(ctx):
    budget = ctx.latency_budget
    exact = cheap_exact(ctx)
    graph = bounded_graph(ctx, exact)
    sem = maybe_semantic(ctx, budget, graph)
    merged = dedup_merge(exact, graph, sem)
    ranked = rank_with_policy(ctx, merged)
    return package(ctx, ranked[:ctx.limit])
```
## 403. Shard Rebalancing pattern 13

### Objective
Optimize shard rebalancing under bounded latency and bounded noise.

### Preconditions
- namespace is validated
- policy context is loaded
- latency budget is available
- tier capabilities are known

### Heuristic
1. start with the cheapest exact signals
2. cap expansion by query class
3. only pay semantic cost when exact and graph signals are insufficient
4. preserve explainability and provenance
5. update reinforcement or decay after outcome feedback

### Failure risks
- candidate explosion
- stale data bias
- low-confidence amplification
- cross-session interference

### Instrumentation
- p50/p95/p99 latency
- candidate counts before and after pruning
- success contribution estimate
- cache hit ratio
- repairability score

### Pseudocode
```text
fn pattern_403(ctx):
    budget = ctx.latency_budget
    exact = cheap_exact(ctx)
    graph = bounded_graph(ctx, exact)
    sem = maybe_semantic(ctx, budget, graph)
    merged = dedup_merge(exact, graph, sem)
    ranked = rank_with_policy(ctx, merged)
    return package(ctx, ranked[:ctx.limit])
```
## 404. Shard Rebalancing pattern 14

### Objective
Optimize shard rebalancing under bounded latency and bounded noise.

### Preconditions
- namespace is validated
- policy context is loaded
- latency budget is available
- tier capabilities are known

### Heuristic
1. start with the cheapest exact signals
2. cap expansion by query class
3. only pay semantic cost when exact and graph signals are insufficient
4. preserve explainability and provenance
5. update reinforcement or decay after outcome feedback

### Failure risks
- candidate explosion
- stale data bias
- low-confidence amplification
- cross-session interference

### Instrumentation
- p50/p95/p99 latency
- candidate counts before and after pruning
- success contribution estimate
- cache hit ratio
- repairability score

### Pseudocode
```text
fn pattern_404(ctx):
    budget = ctx.latency_budget
    exact = cheap_exact(ctx)
    graph = bounded_graph(ctx, exact)
    sem = maybe_semantic(ctx, budget, graph)
    merged = dedup_merge(exact, graph, sem)
    ranked = rank_with_policy(ctx, merged)
    return package(ctx, ranked[:ctx.limit])
```
## 405. Shard Rebalancing pattern 15

### Objective
Optimize shard rebalancing under bounded latency and bounded noise.

### Preconditions
- namespace is validated
- policy context is loaded
- latency budget is available
- tier capabilities are known

### Heuristic
1. start with the cheapest exact signals
2. cap expansion by query class
3. only pay semantic cost when exact and graph signals are insufficient
4. preserve explainability and provenance
5. update reinforcement or decay after outcome feedback

### Failure risks
- candidate explosion
- stale data bias
- low-confidence amplification
- cross-session interference

### Instrumentation
- p50/p95/p99 latency
- candidate counts before and after pruning
- success contribution estimate
- cache hit ratio
- repairability score

### Pseudocode
```text
fn pattern_405(ctx):
    budget = ctx.latency_budget
    exact = cheap_exact(ctx)
    graph = bounded_graph(ctx, exact)
    sem = maybe_semantic(ctx, budget, graph)
    merged = dedup_merge(exact, graph, sem)
    ranked = rank_with_policy(ctx, merged)
    return package(ctx, ranked[:ctx.limit])
```
## 406. Shard Rebalancing pattern 16

### Objective
Optimize shard rebalancing under bounded latency and bounded noise.

### Preconditions
- namespace is validated
- policy context is loaded
- latency budget is available
- tier capabilities are known

### Heuristic
1. start with the cheapest exact signals
2. cap expansion by query class
3. only pay semantic cost when exact and graph signals are insufficient
4. preserve explainability and provenance
5. update reinforcement or decay after outcome feedback

### Failure risks
- candidate explosion
- stale data bias
- low-confidence amplification
- cross-session interference

### Instrumentation
- p50/p95/p99 latency
- candidate counts before and after pruning
- success contribution estimate
- cache hit ratio
- repairability score

### Pseudocode
```text
fn pattern_406(ctx):
    budget = ctx.latency_budget
    exact = cheap_exact(ctx)
    graph = bounded_graph(ctx, exact)
    sem = maybe_semantic(ctx, budget, graph)
    merged = dedup_merge(exact, graph, sem)
    ranked = rank_with_policy(ctx, merged)
    return package(ctx, ranked[:ctx.limit])
```
## 407. Shard Rebalancing pattern 17

### Objective
Optimize shard rebalancing under bounded latency and bounded noise.

### Preconditions
- namespace is validated
- policy context is loaded
- latency budget is available
- tier capabilities are known

### Heuristic
1. start with the cheapest exact signals
2. cap expansion by query class
3. only pay semantic cost when exact and graph signals are insufficient
4. preserve explainability and provenance
5. update reinforcement or decay after outcome feedback

### Failure risks
- candidate explosion
- stale data bias
- low-confidence amplification
- cross-session interference

### Instrumentation
- p50/p95/p99 latency
- candidate counts before and after pruning
- success contribution estimate
- cache hit ratio
- repairability score

### Pseudocode
```text
fn pattern_407(ctx):
    budget = ctx.latency_budget
    exact = cheap_exact(ctx)
    graph = bounded_graph(ctx, exact)
    sem = maybe_semantic(ctx, budget, graph)
    merged = dedup_merge(exact, graph, sem)
    ranked = rank_with_policy(ctx, merged)
    return package(ctx, ranked[:ctx.limit])
```
## 408. Shard Rebalancing pattern 18

### Objective
Optimize shard rebalancing under bounded latency and bounded noise.

### Preconditions
- namespace is validated
- policy context is loaded
- latency budget is available
- tier capabilities are known

### Heuristic
1. start with the cheapest exact signals
2. cap expansion by query class
3. only pay semantic cost when exact and graph signals are insufficient
4. preserve explainability and provenance
5. update reinforcement or decay after outcome feedback

### Failure risks
- candidate explosion
- stale data bias
- low-confidence amplification
- cross-session interference

### Instrumentation
- p50/p95/p99 latency
- candidate counts before and after pruning
- success contribution estimate
- cache hit ratio
- repairability score

### Pseudocode
```text
fn pattern_408(ctx):
    budget = ctx.latency_budget
    exact = cheap_exact(ctx)
    graph = bounded_graph(ctx, exact)
    sem = maybe_semantic(ctx, budget, graph)
    merged = dedup_merge(exact, graph, sem)
    ranked = rank_with_policy(ctx, merged)
    return package(ctx, ranked[:ctx.limit])
```
## 409. Shard Rebalancing pattern 19

### Objective
Optimize shard rebalancing under bounded latency and bounded noise.

### Preconditions
- namespace is validated
- policy context is loaded
- latency budget is available
- tier capabilities are known

### Heuristic
1. start with the cheapest exact signals
2. cap expansion by query class
3. only pay semantic cost when exact and graph signals are insufficient
4. preserve explainability and provenance
5. update reinforcement or decay after outcome feedback

### Failure risks
- candidate explosion
- stale data bias
- low-confidence amplification
- cross-session interference

### Instrumentation
- p50/p95/p99 latency
- candidate counts before and after pruning
- success contribution estimate
- cache hit ratio
- repairability score

### Pseudocode
```text
fn pattern_409(ctx):
    budget = ctx.latency_budget
    exact = cheap_exact(ctx)
    graph = bounded_graph(ctx, exact)
    sem = maybe_semantic(ctx, budget, graph)
    merged = dedup_merge(exact, graph, sem)
    ranked = rank_with_policy(ctx, merged)
    return package(ctx, ranked[:ctx.limit])
```
## 410. Shard Rebalancing pattern 20

### Objective
Optimize shard rebalancing under bounded latency and bounded noise.

### Preconditions
- namespace is validated
- policy context is loaded
- latency budget is available
- tier capabilities are known

### Heuristic
1. start with the cheapest exact signals
2. cap expansion by query class
3. only pay semantic cost when exact and graph signals are insufficient
4. preserve explainability and provenance
5. update reinforcement or decay after outcome feedback

### Failure risks
- candidate explosion
- stale data bias
- low-confidence amplification
- cross-session interference

### Instrumentation
- p50/p95/p99 latency
- candidate counts before and after pruning
- success contribution estimate
- cache hit ratio
- repairability score

### Pseudocode
```text
fn pattern_410(ctx):
    budget = ctx.latency_budget
    exact = cheap_exact(ctx)
    graph = bounded_graph(ctx, exact)
    sem = maybe_semantic(ctx, budget, graph)
    merged = dedup_merge(exact, graph, sem)
    ranked = rank_with_policy(ctx, merged)
    return package(ctx, ranked[:ctx.limit])
```
## 411. Shard Rebalancing pattern 21

### Objective
Optimize shard rebalancing under bounded latency and bounded noise.

### Preconditions
- namespace is validated
- policy context is loaded
- latency budget is available
- tier capabilities are known

### Heuristic
1. start with the cheapest exact signals
2. cap expansion by query class
3. only pay semantic cost when exact and graph signals are insufficient
4. preserve explainability and provenance
5. update reinforcement or decay after outcome feedback

### Failure risks
- candidate explosion
- stale data bias
- low-confidence amplification
- cross-session interference

### Instrumentation
- p50/p95/p99 latency
- candidate counts before and after pruning
- success contribution estimate
- cache hit ratio
- repairability score

### Pseudocode
```text
fn pattern_411(ctx):
    budget = ctx.latency_budget
    exact = cheap_exact(ctx)
    graph = bounded_graph(ctx, exact)
    sem = maybe_semantic(ctx, budget, graph)
    merged = dedup_merge(exact, graph, sem)
    ranked = rank_with_policy(ctx, merged)
    return package(ctx, ranked[:ctx.limit])
```
## 412. Shard Rebalancing pattern 22

### Objective
Optimize shard rebalancing under bounded latency and bounded noise.

### Preconditions
- namespace is validated
- policy context is loaded
- latency budget is available
- tier capabilities are known

### Heuristic
1. start with the cheapest exact signals
2. cap expansion by query class
3. only pay semantic cost when exact and graph signals are insufficient
4. preserve explainability and provenance
5. update reinforcement or decay after outcome feedback

### Failure risks
- candidate explosion
- stale data bias
- low-confidence amplification
- cross-session interference

### Instrumentation
- p50/p95/p99 latency
- candidate counts before and after pruning
- success contribution estimate
- cache hit ratio
- repairability score

### Pseudocode
```text
fn pattern_412(ctx):
    budget = ctx.latency_budget
    exact = cheap_exact(ctx)
    graph = bounded_graph(ctx, exact)
    sem = maybe_semantic(ctx, budget, graph)
    merged = dedup_merge(exact, graph, sem)
    ranked = rank_with_policy(ctx, merged)
    return package(ctx, ranked[:ctx.limit])
```
## 413. Shard Rebalancing pattern 23

### Objective
Optimize shard rebalancing under bounded latency and bounded noise.

### Preconditions
- namespace is validated
- policy context is loaded
- latency budget is available
- tier capabilities are known

### Heuristic
1. start with the cheapest exact signals
2. cap expansion by query class
3. only pay semantic cost when exact and graph signals are insufficient
4. preserve explainability and provenance
5. update reinforcement or decay after outcome feedback

### Failure risks
- candidate explosion
- stale data bias
- low-confidence amplification
- cross-session interference

### Instrumentation
- p50/p95/p99 latency
- candidate counts before and after pruning
- success contribution estimate
- cache hit ratio
- repairability score

### Pseudocode
```text
fn pattern_413(ctx):
    budget = ctx.latency_budget
    exact = cheap_exact(ctx)
    graph = bounded_graph(ctx, exact)
    sem = maybe_semantic(ctx, budget, graph)
    merged = dedup_merge(exact, graph, sem)
    ranked = rank_with_policy(ctx, merged)
    return package(ctx, ranked[:ctx.limit])
```
## 414. Shard Rebalancing pattern 24

### Objective
Optimize shard rebalancing under bounded latency and bounded noise.

### Preconditions
- namespace is validated
- policy context is loaded
- latency budget is available
- tier capabilities are known

### Heuristic
1. start with the cheapest exact signals
2. cap expansion by query class
3. only pay semantic cost when exact and graph signals are insufficient
4. preserve explainability and provenance
5. update reinforcement or decay after outcome feedback

### Failure risks
- candidate explosion
- stale data bias
- low-confidence amplification
- cross-session interference

### Instrumentation
- p50/p95/p99 latency
- candidate counts before and after pruning
- success contribution estimate
- cache hit ratio
- repairability score

### Pseudocode
```text
fn pattern_414(ctx):
    budget = ctx.latency_budget
    exact = cheap_exact(ctx)
    graph = bounded_graph(ctx, exact)
    sem = maybe_semantic(ctx, budget, graph)
    merged = dedup_merge(exact, graph, sem)
    ranked = rank_with_policy(ctx, merged)
    return package(ctx, ranked[:ctx.limit])
```
## 415. Shard Rebalancing pattern 25

### Objective
Optimize shard rebalancing under bounded latency and bounded noise.

### Preconditions
- namespace is validated
- policy context is loaded
- latency budget is available
- tier capabilities are known

### Heuristic
1. start with the cheapest exact signals
2. cap expansion by query class
3. only pay semantic cost when exact and graph signals are insufficient
4. preserve explainability and provenance
5. update reinforcement or decay after outcome feedback

### Failure risks
- candidate explosion
- stale data bias
- low-confidence amplification
- cross-session interference

### Instrumentation
- p50/p95/p99 latency
- candidate counts before and after pruning
- success contribution estimate
- cache hit ratio
- repairability score

### Pseudocode
```text
fn pattern_415(ctx):
    budget = ctx.latency_budget
    exact = cheap_exact(ctx)
    graph = bounded_graph(ctx, exact)
    sem = maybe_semantic(ctx, budget, graph)
    merged = dedup_merge(exact, graph, sem)
    ranked = rank_with_policy(ctx, merged)
    return package(ctx, ranked[:ctx.limit])
```
## 416. Shard Rebalancing pattern 26

### Objective
Optimize shard rebalancing under bounded latency and bounded noise.

### Preconditions
- namespace is validated
- policy context is loaded
- latency budget is available
- tier capabilities are known

### Heuristic
1. start with the cheapest exact signals
2. cap expansion by query class
3. only pay semantic cost when exact and graph signals are insufficient
4. preserve explainability and provenance
5. update reinforcement or decay after outcome feedback

### Failure risks
- candidate explosion
- stale data bias
- low-confidence amplification
- cross-session interference

### Instrumentation
- p50/p95/p99 latency
- candidate counts before and after pruning
- success contribution estimate
- cache hit ratio
- repairability score

### Pseudocode
```text
fn pattern_416(ctx):
    budget = ctx.latency_budget
    exact = cheap_exact(ctx)
    graph = bounded_graph(ctx, exact)
    sem = maybe_semantic(ctx, budget, graph)
    merged = dedup_merge(exact, graph, sem)
    ranked = rank_with_policy(ctx, merged)
    return package(ctx, ranked[:ctx.limit])
```
## 417. Shard Rebalancing pattern 27

### Objective
Optimize shard rebalancing under bounded latency and bounded noise.

### Preconditions
- namespace is validated
- policy context is loaded
- latency budget is available
- tier capabilities are known

### Heuristic
1. start with the cheapest exact signals
2. cap expansion by query class
3. only pay semantic cost when exact and graph signals are insufficient
4. preserve explainability and provenance
5. update reinforcement or decay after outcome feedback

### Failure risks
- candidate explosion
- stale data bias
- low-confidence amplification
- cross-session interference

### Instrumentation
- p50/p95/p99 latency
- candidate counts before and after pruning
- success contribution estimate
- cache hit ratio
- repairability score

### Pseudocode
```text
fn pattern_417(ctx):
    budget = ctx.latency_budget
    exact = cheap_exact(ctx)
    graph = bounded_graph(ctx, exact)
    sem = maybe_semantic(ctx, budget, graph)
    merged = dedup_merge(exact, graph, sem)
    ranked = rank_with_policy(ctx, merged)
    return package(ctx, ranked[:ctx.limit])
```
## 418. Shard Rebalancing pattern 28

### Objective
Optimize shard rebalancing under bounded latency and bounded noise.

### Preconditions
- namespace is validated
- policy context is loaded
- latency budget is available
- tier capabilities are known

### Heuristic
1. start with the cheapest exact signals
2. cap expansion by query class
3. only pay semantic cost when exact and graph signals are insufficient
4. preserve explainability and provenance
5. update reinforcement or decay after outcome feedback

### Failure risks
- candidate explosion
- stale data bias
- low-confidence amplification
- cross-session interference

### Instrumentation
- p50/p95/p99 latency
- candidate counts before and after pruning
- success contribution estimate
- cache hit ratio
- repairability score

### Pseudocode
```text
fn pattern_418(ctx):
    budget = ctx.latency_budget
    exact = cheap_exact(ctx)
    graph = bounded_graph(ctx, exact)
    sem = maybe_semantic(ctx, budget, graph)
    merged = dedup_merge(exact, graph, sem)
    ranked = rank_with_policy(ctx, merged)
    return package(ctx, ranked[:ctx.limit])
```
## 419. Shard Rebalancing pattern 29

### Objective
Optimize shard rebalancing under bounded latency and bounded noise.

### Preconditions
- namespace is validated
- policy context is loaded
- latency budget is available
- tier capabilities are known

### Heuristic
1. start with the cheapest exact signals
2. cap expansion by query class
3. only pay semantic cost when exact and graph signals are insufficient
4. preserve explainability and provenance
5. update reinforcement or decay after outcome feedback

### Failure risks
- candidate explosion
- stale data bias
- low-confidence amplification
- cross-session interference

### Instrumentation
- p50/p95/p99 latency
- candidate counts before and after pruning
- success contribution estimate
- cache hit ratio
- repairability score

### Pseudocode
```text
fn pattern_419(ctx):
    budget = ctx.latency_budget
    exact = cheap_exact(ctx)
    graph = bounded_graph(ctx, exact)
    sem = maybe_semantic(ctx, budget, graph)
    merged = dedup_merge(exact, graph, sem)
    ranked = rank_with_policy(ctx, merged)
    return package(ctx, ranked[:ctx.limit])
```
## 420. Shard Rebalancing pattern 30

### Objective
Optimize shard rebalancing under bounded latency and bounded noise.

### Preconditions
- namespace is validated
- policy context is loaded
- latency budget is available
- tier capabilities are known

### Heuristic
1. start with the cheapest exact signals
2. cap expansion by query class
3. only pay semantic cost when exact and graph signals are insufficient
4. preserve explainability and provenance
5. update reinforcement or decay after outcome feedback

### Failure risks
- candidate explosion
- stale data bias
- low-confidence amplification
- cross-session interference

### Instrumentation
- p50/p95/p99 latency
- candidate counts before and after pruning
- success contribution estimate
- cache hit ratio
- repairability score

### Pseudocode
```text
fn pattern_420(ctx):
    budget = ctx.latency_budget
    exact = cheap_exact(ctx)
    graph = bounded_graph(ctx, exact)
    sem = maybe_semantic(ctx, budget, graph)
    merged = dedup_merge(exact, graph, sem)
    ranked = rank_with_policy(ctx, merged)
    return package(ctx, ranked[:ctx.limit])
```
## 421. Cache Invalidation pattern 1

### Objective
Optimize cache invalidation under bounded latency and bounded noise.

### Preconditions
- namespace is validated
- policy context is loaded
- latency budget is available
- tier capabilities are known

### Heuristic
1. start with the cheapest exact signals
2. cap expansion by query class
3. only pay semantic cost when exact and graph signals are insufficient
4. preserve explainability and provenance
5. update reinforcement or decay after outcome feedback

### Failure risks
- candidate explosion
- stale data bias
- low-confidence amplification
- cross-session interference

### Instrumentation
- p50/p95/p99 latency
- candidate counts before and after pruning
- success contribution estimate
- cache hit ratio
- repairability score

### Pseudocode
```text
fn pattern_421(ctx):
    budget = ctx.latency_budget
    exact = cheap_exact(ctx)
    graph = bounded_graph(ctx, exact)
    sem = maybe_semantic(ctx, budget, graph)
    merged = dedup_merge(exact, graph, sem)
    ranked = rank_with_policy(ctx, merged)
    return package(ctx, ranked[:ctx.limit])
```
## 422. Cache Invalidation pattern 2

### Objective
Optimize cache invalidation under bounded latency and bounded noise.

### Preconditions
- namespace is validated
- policy context is loaded
- latency budget is available
- tier capabilities are known

### Heuristic
1. start with the cheapest exact signals
2. cap expansion by query class
3. only pay semantic cost when exact and graph signals are insufficient
4. preserve explainability and provenance
5. update reinforcement or decay after outcome feedback

### Failure risks
- candidate explosion
- stale data bias
- low-confidence amplification
- cross-session interference

### Instrumentation
- p50/p95/p99 latency
- candidate counts before and after pruning
- success contribution estimate
- cache hit ratio
- repairability score

### Pseudocode
```text
fn pattern_422(ctx):
    budget = ctx.latency_budget
    exact = cheap_exact(ctx)
    graph = bounded_graph(ctx, exact)
    sem = maybe_semantic(ctx, budget, graph)
    merged = dedup_merge(exact, graph, sem)
    ranked = rank_with_policy(ctx, merged)
    return package(ctx, ranked[:ctx.limit])
```
## 423. Cache Invalidation pattern 3

### Objective
Optimize cache invalidation under bounded latency and bounded noise.

### Preconditions
- namespace is validated
- policy context is loaded
- latency budget is available
- tier capabilities are known

### Heuristic
1. start with the cheapest exact signals
2. cap expansion by query class
3. only pay semantic cost when exact and graph signals are insufficient
4. preserve explainability and provenance
5. update reinforcement or decay after outcome feedback

### Failure risks
- candidate explosion
- stale data bias
- low-confidence amplification
- cross-session interference

### Instrumentation
- p50/p95/p99 latency
- candidate counts before and after pruning
- success contribution estimate
- cache hit ratio
- repairability score

### Pseudocode
```text
fn pattern_423(ctx):
    budget = ctx.latency_budget
    exact = cheap_exact(ctx)
    graph = bounded_graph(ctx, exact)
    sem = maybe_semantic(ctx, budget, graph)
    merged = dedup_merge(exact, graph, sem)
    ranked = rank_with_policy(ctx, merged)
    return package(ctx, ranked[:ctx.limit])
```
## 424. Cache Invalidation pattern 4

### Objective
Optimize cache invalidation under bounded latency and bounded noise.

### Preconditions
- namespace is validated
- policy context is loaded
- latency budget is available
- tier capabilities are known

### Heuristic
1. start with the cheapest exact signals
2. cap expansion by query class
3. only pay semantic cost when exact and graph signals are insufficient
4. preserve explainability and provenance
5. update reinforcement or decay after outcome feedback

### Failure risks
- candidate explosion
- stale data bias
- low-confidence amplification
- cross-session interference

### Instrumentation
- p50/p95/p99 latency
- candidate counts before and after pruning
- success contribution estimate
- cache hit ratio
- repairability score

### Pseudocode
```text
fn pattern_424(ctx):
    budget = ctx.latency_budget
    exact = cheap_exact(ctx)
    graph = bounded_graph(ctx, exact)
    sem = maybe_semantic(ctx, budget, graph)
    merged = dedup_merge(exact, graph, sem)
    ranked = rank_with_policy(ctx, merged)
    return package(ctx, ranked[:ctx.limit])
```
## 425. Cache Invalidation pattern 5

### Objective
Optimize cache invalidation under bounded latency and bounded noise.

### Preconditions
- namespace is validated
- policy context is loaded
- latency budget is available
- tier capabilities are known

### Heuristic
1. start with the cheapest exact signals
2. cap expansion by query class
3. only pay semantic cost when exact and graph signals are insufficient
4. preserve explainability and provenance
5. update reinforcement or decay after outcome feedback

### Failure risks
- candidate explosion
- stale data bias
- low-confidence amplification
- cross-session interference

### Instrumentation
- p50/p95/p99 latency
- candidate counts before and after pruning
- success contribution estimate
- cache hit ratio
- repairability score

### Pseudocode
```text
fn pattern_425(ctx):
    budget = ctx.latency_budget
    exact = cheap_exact(ctx)
    graph = bounded_graph(ctx, exact)
    sem = maybe_semantic(ctx, budget, graph)
    merged = dedup_merge(exact, graph, sem)
    ranked = rank_with_policy(ctx, merged)
    return package(ctx, ranked[:ctx.limit])
```
## 426. Cache Invalidation pattern 6

### Objective
Optimize cache invalidation under bounded latency and bounded noise.

### Preconditions
- namespace is validated
- policy context is loaded
- latency budget is available
- tier capabilities are known

### Heuristic
1. start with the cheapest exact signals
2. cap expansion by query class
3. only pay semantic cost when exact and graph signals are insufficient
4. preserve explainability and provenance
5. update reinforcement or decay after outcome feedback

### Failure risks
- candidate explosion
- stale data bias
- low-confidence amplification
- cross-session interference

### Instrumentation
- p50/p95/p99 latency
- candidate counts before and after pruning
- success contribution estimate
- cache hit ratio
- repairability score

### Pseudocode
```text
fn pattern_426(ctx):
    budget = ctx.latency_budget
    exact = cheap_exact(ctx)
    graph = bounded_graph(ctx, exact)
    sem = maybe_semantic(ctx, budget, graph)
    merged = dedup_merge(exact, graph, sem)
    ranked = rank_with_policy(ctx, merged)
    return package(ctx, ranked[:ctx.limit])
```
## 427. Cache Invalidation pattern 7

### Objective
Optimize cache invalidation under bounded latency and bounded noise.

### Preconditions
- namespace is validated
- policy context is loaded
- latency budget is available
- tier capabilities are known

### Heuristic
1. start with the cheapest exact signals
2. cap expansion by query class
3. only pay semantic cost when exact and graph signals are insufficient
4. preserve explainability and provenance
5. update reinforcement or decay after outcome feedback

### Failure risks
- candidate explosion
- stale data bias
- low-confidence amplification
- cross-session interference

### Instrumentation
- p50/p95/p99 latency
- candidate counts before and after pruning
- success contribution estimate
- cache hit ratio
- repairability score

### Pseudocode
```text
fn pattern_427(ctx):
    budget = ctx.latency_budget
    exact = cheap_exact(ctx)
    graph = bounded_graph(ctx, exact)
    sem = maybe_semantic(ctx, budget, graph)
    merged = dedup_merge(exact, graph, sem)
    ranked = rank_with_policy(ctx, merged)
    return package(ctx, ranked[:ctx.limit])
```
## 428. Cache Invalidation pattern 8

### Objective
Optimize cache invalidation under bounded latency and bounded noise.

### Preconditions
- namespace is validated
- policy context is loaded
- latency budget is available
- tier capabilities are known

### Heuristic
1. start with the cheapest exact signals
2. cap expansion by query class
3. only pay semantic cost when exact and graph signals are insufficient
4. preserve explainability and provenance
5. update reinforcement or decay after outcome feedback

### Failure risks
- candidate explosion
- stale data bias
- low-confidence amplification
- cross-session interference

### Instrumentation
- p50/p95/p99 latency
- candidate counts before and after pruning
- success contribution estimate
- cache hit ratio
- repairability score

### Pseudocode
```text
fn pattern_428(ctx):
    budget = ctx.latency_budget
    exact = cheap_exact(ctx)
    graph = bounded_graph(ctx, exact)
    sem = maybe_semantic(ctx, budget, graph)
    merged = dedup_merge(exact, graph, sem)
    ranked = rank_with_policy(ctx, merged)
    return package(ctx, ranked[:ctx.limit])
```
## 429. Cache Invalidation pattern 9

### Objective
Optimize cache invalidation under bounded latency and bounded noise.

### Preconditions
- namespace is validated
- policy context is loaded
- latency budget is available
- tier capabilities are known

### Heuristic
1. start with the cheapest exact signals
2. cap expansion by query class
3. only pay semantic cost when exact and graph signals are insufficient
4. preserve explainability and provenance
5. update reinforcement or decay after outcome feedback

### Failure risks
- candidate explosion
- stale data bias
- low-confidence amplification
- cross-session interference

### Instrumentation
- p50/p95/p99 latency
- candidate counts before and after pruning
- success contribution estimate
- cache hit ratio
- repairability score

### Pseudocode
```text
fn pattern_429(ctx):
    budget = ctx.latency_budget
    exact = cheap_exact(ctx)
    graph = bounded_graph(ctx, exact)
    sem = maybe_semantic(ctx, budget, graph)
    merged = dedup_merge(exact, graph, sem)
    ranked = rank_with_policy(ctx, merged)
    return package(ctx, ranked[:ctx.limit])
```
## 430. Cache Invalidation pattern 10

### Objective
Optimize cache invalidation under bounded latency and bounded noise.

### Preconditions
- namespace is validated
- policy context is loaded
- latency budget is available
- tier capabilities are known

### Heuristic
1. start with the cheapest exact signals
2. cap expansion by query class
3. only pay semantic cost when exact and graph signals are insufficient
4. preserve explainability and provenance
5. update reinforcement or decay after outcome feedback

### Failure risks
- candidate explosion
- stale data bias
- low-confidence amplification
- cross-session interference

### Instrumentation
- p50/p95/p99 latency
- candidate counts before and after pruning
- success contribution estimate
- cache hit ratio
- repairability score

### Pseudocode
```text
fn pattern_430(ctx):
    budget = ctx.latency_budget
    exact = cheap_exact(ctx)
    graph = bounded_graph(ctx, exact)
    sem = maybe_semantic(ctx, budget, graph)
    merged = dedup_merge(exact, graph, sem)
    ranked = rank_with_policy(ctx, merged)
    return package(ctx, ranked[:ctx.limit])
```
## 431. Cache Invalidation pattern 11

### Objective
Optimize cache invalidation under bounded latency and bounded noise.

### Preconditions
- namespace is validated
- policy context is loaded
- latency budget is available
- tier capabilities are known

### Heuristic
1. start with the cheapest exact signals
2. cap expansion by query class
3. only pay semantic cost when exact and graph signals are insufficient
4. preserve explainability and provenance
5. update reinforcement or decay after outcome feedback

### Failure risks
- candidate explosion
- stale data bias
- low-confidence amplification
- cross-session interference

### Instrumentation
- p50/p95/p99 latency
- candidate counts before and after pruning
- success contribution estimate
- cache hit ratio
- repairability score

### Pseudocode
```text
fn pattern_431(ctx):
    budget = ctx.latency_budget
    exact = cheap_exact(ctx)
    graph = bounded_graph(ctx, exact)
    sem = maybe_semantic(ctx, budget, graph)
    merged = dedup_merge(exact, graph, sem)
    ranked = rank_with_policy(ctx, merged)
    return package(ctx, ranked[:ctx.limit])
```
## 432. Cache Invalidation pattern 12

### Objective
Optimize cache invalidation under bounded latency and bounded noise.

### Preconditions
- namespace is validated
- policy context is loaded
- latency budget is available
- tier capabilities are known

### Heuristic
1. start with the cheapest exact signals
2. cap expansion by query class
3. only pay semantic cost when exact and graph signals are insufficient
4. preserve explainability and provenance
5. update reinforcement or decay after outcome feedback

### Failure risks
- candidate explosion
- stale data bias
- low-confidence amplification
- cross-session interference

### Instrumentation
- p50/p95/p99 latency
- candidate counts before and after pruning
- success contribution estimate
- cache hit ratio
- repairability score

### Pseudocode
```text
fn pattern_432(ctx):
    budget = ctx.latency_budget
    exact = cheap_exact(ctx)
    graph = bounded_graph(ctx, exact)
    sem = maybe_semantic(ctx, budget, graph)
    merged = dedup_merge(exact, graph, sem)
    ranked = rank_with_policy(ctx, merged)
    return package(ctx, ranked[:ctx.limit])
```
## 433. Cache Invalidation pattern 13

### Objective
Optimize cache invalidation under bounded latency and bounded noise.

### Preconditions
- namespace is validated
- policy context is loaded
- latency budget is available
- tier capabilities are known

### Heuristic
1. start with the cheapest exact signals
2. cap expansion by query class
3. only pay semantic cost when exact and graph signals are insufficient
4. preserve explainability and provenance
5. update reinforcement or decay after outcome feedback

### Failure risks
- candidate explosion
- stale data bias
- low-confidence amplification
- cross-session interference

### Instrumentation
- p50/p95/p99 latency
- candidate counts before and after pruning
- success contribution estimate
- cache hit ratio
- repairability score

### Pseudocode
```text
fn pattern_433(ctx):
    budget = ctx.latency_budget
    exact = cheap_exact(ctx)
    graph = bounded_graph(ctx, exact)
    sem = maybe_semantic(ctx, budget, graph)
    merged = dedup_merge(exact, graph, sem)
    ranked = rank_with_policy(ctx, merged)
    return package(ctx, ranked[:ctx.limit])
```
## 434. Cache Invalidation pattern 14

### Objective
Optimize cache invalidation under bounded latency and bounded noise.

### Preconditions
- namespace is validated
- policy context is loaded
- latency budget is available
- tier capabilities are known

### Heuristic
1. start with the cheapest exact signals
2. cap expansion by query class
3. only pay semantic cost when exact and graph signals are insufficient
4. preserve explainability and provenance
5. update reinforcement or decay after outcome feedback

### Failure risks
- candidate explosion
- stale data bias
- low-confidence amplification
- cross-session interference

### Instrumentation
- p50/p95/p99 latency
- candidate counts before and after pruning
- success contribution estimate
- cache hit ratio
- repairability score

### Pseudocode
```text
fn pattern_434(ctx):
    budget = ctx.latency_budget
    exact = cheap_exact(ctx)
    graph = bounded_graph(ctx, exact)
    sem = maybe_semantic(ctx, budget, graph)
    merged = dedup_merge(exact, graph, sem)
    ranked = rank_with_policy(ctx, merged)
    return package(ctx, ranked[:ctx.limit])
```
## 435. Cache Invalidation pattern 15

### Objective
Optimize cache invalidation under bounded latency and bounded noise.

### Preconditions
- namespace is validated
- policy context is loaded
- latency budget is available
- tier capabilities are known

### Heuristic
1. start with the cheapest exact signals
2. cap expansion by query class
3. only pay semantic cost when exact and graph signals are insufficient
4. preserve explainability and provenance
5. update reinforcement or decay after outcome feedback

### Failure risks
- candidate explosion
- stale data bias
- low-confidence amplification
- cross-session interference

### Instrumentation
- p50/p95/p99 latency
- candidate counts before and after pruning
- success contribution estimate
- cache hit ratio
- repairability score

### Pseudocode
```text
fn pattern_435(ctx):
    budget = ctx.latency_budget
    exact = cheap_exact(ctx)
    graph = bounded_graph(ctx, exact)
    sem = maybe_semantic(ctx, budget, graph)
    merged = dedup_merge(exact, graph, sem)
    ranked = rank_with_policy(ctx, merged)
    return package(ctx, ranked[:ctx.limit])
```
## 436. Cache Invalidation pattern 16

### Objective
Optimize cache invalidation under bounded latency and bounded noise.

### Preconditions
- namespace is validated
- policy context is loaded
- latency budget is available
- tier capabilities are known

### Heuristic
1. start with the cheapest exact signals
2. cap expansion by query class
3. only pay semantic cost when exact and graph signals are insufficient
4. preserve explainability and provenance
5. update reinforcement or decay after outcome feedback

### Failure risks
- candidate explosion
- stale data bias
- low-confidence amplification
- cross-session interference

### Instrumentation
- p50/p95/p99 latency
- candidate counts before and after pruning
- success contribution estimate
- cache hit ratio
- repairability score

### Pseudocode
```text
fn pattern_436(ctx):
    budget = ctx.latency_budget
    exact = cheap_exact(ctx)
    graph = bounded_graph(ctx, exact)
    sem = maybe_semantic(ctx, budget, graph)
    merged = dedup_merge(exact, graph, sem)
    ranked = rank_with_policy(ctx, merged)
    return package(ctx, ranked[:ctx.limit])
```
## 437. Cache Invalidation pattern 17

### Objective
Optimize cache invalidation under bounded latency and bounded noise.

### Preconditions
- namespace is validated
- policy context is loaded
- latency budget is available
- tier capabilities are known

### Heuristic
1. start with the cheapest exact signals
2. cap expansion by query class
3. only pay semantic cost when exact and graph signals are insufficient
4. preserve explainability and provenance
5. update reinforcement or decay after outcome feedback

### Failure risks
- candidate explosion
- stale data bias
- low-confidence amplification
- cross-session interference

### Instrumentation
- p50/p95/p99 latency
- candidate counts before and after pruning
- success contribution estimate
- cache hit ratio
- repairability score

### Pseudocode
```text
fn pattern_437(ctx):
    budget = ctx.latency_budget
    exact = cheap_exact(ctx)
    graph = bounded_graph(ctx, exact)
    sem = maybe_semantic(ctx, budget, graph)
    merged = dedup_merge(exact, graph, sem)
    ranked = rank_with_policy(ctx, merged)
    return package(ctx, ranked[:ctx.limit])
```
## 438. Cache Invalidation pattern 18

### Objective
Optimize cache invalidation under bounded latency and bounded noise.

### Preconditions
- namespace is validated
- policy context is loaded
- latency budget is available
- tier capabilities are known

### Heuristic
1. start with the cheapest exact signals
2. cap expansion by query class
3. only pay semantic cost when exact and graph signals are insufficient
4. preserve explainability and provenance
5. update reinforcement or decay after outcome feedback

### Failure risks
- candidate explosion
- stale data bias
- low-confidence amplification
- cross-session interference

### Instrumentation
- p50/p95/p99 latency
- candidate counts before and after pruning
- success contribution estimate
- cache hit ratio
- repairability score

### Pseudocode
```text
fn pattern_438(ctx):
    budget = ctx.latency_budget
    exact = cheap_exact(ctx)
    graph = bounded_graph(ctx, exact)
    sem = maybe_semantic(ctx, budget, graph)
    merged = dedup_merge(exact, graph, sem)
    ranked = rank_with_policy(ctx, merged)
    return package(ctx, ranked[:ctx.limit])
```
## 439. Cache Invalidation pattern 19

### Objective
Optimize cache invalidation under bounded latency and bounded noise.

### Preconditions
- namespace is validated
- policy context is loaded
- latency budget is available
- tier capabilities are known

### Heuristic
1. start with the cheapest exact signals
2. cap expansion by query class
3. only pay semantic cost when exact and graph signals are insufficient
4. preserve explainability and provenance
5. update reinforcement or decay after outcome feedback

### Failure risks
- candidate explosion
- stale data bias
- low-confidence amplification
- cross-session interference

### Instrumentation
- p50/p95/p99 latency
- candidate counts before and after pruning
- success contribution estimate
- cache hit ratio
- repairability score

### Pseudocode
```text
fn pattern_439(ctx):
    budget = ctx.latency_budget
    exact = cheap_exact(ctx)
    graph = bounded_graph(ctx, exact)
    sem = maybe_semantic(ctx, budget, graph)
    merged = dedup_merge(exact, graph, sem)
    ranked = rank_with_policy(ctx, merged)
    return package(ctx, ranked[:ctx.limit])
```
## 440. Cache Invalidation pattern 20

### Objective
Optimize cache invalidation under bounded latency and bounded noise.

### Preconditions
- namespace is validated
- policy context is loaded
- latency budget is available
- tier capabilities are known

### Heuristic
1. start with the cheapest exact signals
2. cap expansion by query class
3. only pay semantic cost when exact and graph signals are insufficient
4. preserve explainability and provenance
5. update reinforcement or decay after outcome feedback

### Failure risks
- candidate explosion
- stale data bias
- low-confidence amplification
- cross-session interference

### Instrumentation
- p50/p95/p99 latency
- candidate counts before and after pruning
- success contribution estimate
- cache hit ratio
- repairability score

### Pseudocode
```text
fn pattern_440(ctx):
    budget = ctx.latency_budget
    exact = cheap_exact(ctx)
    graph = bounded_graph(ctx, exact)
    sem = maybe_semantic(ctx, budget, graph)
    merged = dedup_merge(exact, graph, sem)
    ranked = rank_with_policy(ctx, merged)
    return package(ctx, ranked[:ctx.limit])
```
## 441. Cache Invalidation pattern 21

### Objective
Optimize cache invalidation under bounded latency and bounded noise.

### Preconditions
- namespace is validated
- policy context is loaded
- latency budget is available
- tier capabilities are known

### Heuristic
1. start with the cheapest exact signals
2. cap expansion by query class
3. only pay semantic cost when exact and graph signals are insufficient
4. preserve explainability and provenance
5. update reinforcement or decay after outcome feedback

### Failure risks
- candidate explosion
- stale data bias
- low-confidence amplification
- cross-session interference

### Instrumentation
- p50/p95/p99 latency
- candidate counts before and after pruning
- success contribution estimate
- cache hit ratio
- repairability score

### Pseudocode
```text
fn pattern_441(ctx):
    budget = ctx.latency_budget
    exact = cheap_exact(ctx)
    graph = bounded_graph(ctx, exact)
    sem = maybe_semantic(ctx, budget, graph)
    merged = dedup_merge(exact, graph, sem)
    ranked = rank_with_policy(ctx, merged)
    return package(ctx, ranked[:ctx.limit])
```
## 442. Cache Invalidation pattern 22

### Objective
Optimize cache invalidation under bounded latency and bounded noise.

### Preconditions
- namespace is validated
- policy context is loaded
- latency budget is available
- tier capabilities are known

### Heuristic
1. start with the cheapest exact signals
2. cap expansion by query class
3. only pay semantic cost when exact and graph signals are insufficient
4. preserve explainability and provenance
5. update reinforcement or decay after outcome feedback

### Failure risks
- candidate explosion
- stale data bias
- low-confidence amplification
- cross-session interference

### Instrumentation
- p50/p95/p99 latency
- candidate counts before and after pruning
- success contribution estimate
- cache hit ratio
- repairability score

### Pseudocode
```text
fn pattern_442(ctx):
    budget = ctx.latency_budget
    exact = cheap_exact(ctx)
    graph = bounded_graph(ctx, exact)
    sem = maybe_semantic(ctx, budget, graph)
    merged = dedup_merge(exact, graph, sem)
    ranked = rank_with_policy(ctx, merged)
    return package(ctx, ranked[:ctx.limit])
```
## 443. Cache Invalidation pattern 23

### Objective
Optimize cache invalidation under bounded latency and bounded noise.

### Preconditions
- namespace is validated
- policy context is loaded
- latency budget is available
- tier capabilities are known

### Heuristic
1. start with the cheapest exact signals
2. cap expansion by query class
3. only pay semantic cost when exact and graph signals are insufficient
4. preserve explainability and provenance
5. update reinforcement or decay after outcome feedback

### Failure risks
- candidate explosion
- stale data bias
- low-confidence amplification
- cross-session interference

### Instrumentation
- p50/p95/p99 latency
- candidate counts before and after pruning
- success contribution estimate
- cache hit ratio
- repairability score

### Pseudocode
```text
fn pattern_443(ctx):
    budget = ctx.latency_budget
    exact = cheap_exact(ctx)
    graph = bounded_graph(ctx, exact)
    sem = maybe_semantic(ctx, budget, graph)
    merged = dedup_merge(exact, graph, sem)
    ranked = rank_with_policy(ctx, merged)
    return package(ctx, ranked[:ctx.limit])
```
## 444. Cache Invalidation pattern 24

### Objective
Optimize cache invalidation under bounded latency and bounded noise.

### Preconditions
- namespace is validated
- policy context is loaded
- latency budget is available
- tier capabilities are known

### Heuristic
1. start with the cheapest exact signals
2. cap expansion by query class
3. only pay semantic cost when exact and graph signals are insufficient
4. preserve explainability and provenance
5. update reinforcement or decay after outcome feedback

### Failure risks
- candidate explosion
- stale data bias
- low-confidence amplification
- cross-session interference

### Instrumentation
- p50/p95/p99 latency
- candidate counts before and after pruning
- success contribution estimate
- cache hit ratio
- repairability score

### Pseudocode
```text
fn pattern_444(ctx):
    budget = ctx.latency_budget
    exact = cheap_exact(ctx)
    graph = bounded_graph(ctx, exact)
    sem = maybe_semantic(ctx, budget, graph)
    merged = dedup_merge(exact, graph, sem)
    ranked = rank_with_policy(ctx, merged)
    return package(ctx, ranked[:ctx.limit])
```
## 445. Cache Invalidation pattern 25

### Objective
Optimize cache invalidation under bounded latency and bounded noise.

### Preconditions
- namespace is validated
- policy context is loaded
- latency budget is available
- tier capabilities are known

### Heuristic
1. start with the cheapest exact signals
2. cap expansion by query class
3. only pay semantic cost when exact and graph signals are insufficient
4. preserve explainability and provenance
5. update reinforcement or decay after outcome feedback

### Failure risks
- candidate explosion
- stale data bias
- low-confidence amplification
- cross-session interference

### Instrumentation
- p50/p95/p99 latency
- candidate counts before and after pruning
- success contribution estimate
- cache hit ratio
- repairability score

### Pseudocode
```text
fn pattern_445(ctx):
    budget = ctx.latency_budget
    exact = cheap_exact(ctx)
    graph = bounded_graph(ctx, exact)
    sem = maybe_semantic(ctx, budget, graph)
    merged = dedup_merge(exact, graph, sem)
    ranked = rank_with_policy(ctx, merged)
    return package(ctx, ranked[:ctx.limit])
```
## 446. Cache Invalidation pattern 26

### Objective
Optimize cache invalidation under bounded latency and bounded noise.

### Preconditions
- namespace is validated
- policy context is loaded
- latency budget is available
- tier capabilities are known

### Heuristic
1. start with the cheapest exact signals
2. cap expansion by query class
3. only pay semantic cost when exact and graph signals are insufficient
4. preserve explainability and provenance
5. update reinforcement or decay after outcome feedback

### Failure risks
- candidate explosion
- stale data bias
- low-confidence amplification
- cross-session interference

### Instrumentation
- p50/p95/p99 latency
- candidate counts before and after pruning
- success contribution estimate
- cache hit ratio
- repairability score

### Pseudocode
```text
fn pattern_446(ctx):
    budget = ctx.latency_budget
    exact = cheap_exact(ctx)
    graph = bounded_graph(ctx, exact)
    sem = maybe_semantic(ctx, budget, graph)
    merged = dedup_merge(exact, graph, sem)
    ranked = rank_with_policy(ctx, merged)
    return package(ctx, ranked[:ctx.limit])
```
## 447. Cache Invalidation pattern 27

### Objective
Optimize cache invalidation under bounded latency and bounded noise.

### Preconditions
- namespace is validated
- policy context is loaded
- latency budget is available
- tier capabilities are known

### Heuristic
1. start with the cheapest exact signals
2. cap expansion by query class
3. only pay semantic cost when exact and graph signals are insufficient
4. preserve explainability and provenance
5. update reinforcement or decay after outcome feedback

### Failure risks
- candidate explosion
- stale data bias
- low-confidence amplification
- cross-session interference

### Instrumentation
- p50/p95/p99 latency
- candidate counts before and after pruning
- success contribution estimate
- cache hit ratio
- repairability score

### Pseudocode
```text
fn pattern_447(ctx):
    budget = ctx.latency_budget
    exact = cheap_exact(ctx)
    graph = bounded_graph(ctx, exact)
    sem = maybe_semantic(ctx, budget, graph)
    merged = dedup_merge(exact, graph, sem)
    ranked = rank_with_policy(ctx, merged)
    return package(ctx, ranked[:ctx.limit])
```
## 448. Cache Invalidation pattern 28

### Objective
Optimize cache invalidation under bounded latency and bounded noise.

### Preconditions
- namespace is validated
- policy context is loaded
- latency budget is available
- tier capabilities are known

### Heuristic
1. start with the cheapest exact signals
2. cap expansion by query class
3. only pay semantic cost when exact and graph signals are insufficient
4. preserve explainability and provenance
5. update reinforcement or decay after outcome feedback

### Failure risks
- candidate explosion
- stale data bias
- low-confidence amplification
- cross-session interference

### Instrumentation
- p50/p95/p99 latency
- candidate counts before and after pruning
- success contribution estimate
- cache hit ratio
- repairability score

### Pseudocode
```text
fn pattern_448(ctx):
    budget = ctx.latency_budget
    exact = cheap_exact(ctx)
    graph = bounded_graph(ctx, exact)
    sem = maybe_semantic(ctx, budget, graph)
    merged = dedup_merge(exact, graph, sem)
    ranked = rank_with_policy(ctx, merged)
    return package(ctx, ranked[:ctx.limit])
```
## 449. Cache Invalidation pattern 29

### Objective
Optimize cache invalidation under bounded latency and bounded noise.

### Preconditions
- namespace is validated
- policy context is loaded
- latency budget is available
- tier capabilities are known

### Heuristic
1. start with the cheapest exact signals
2. cap expansion by query class
3. only pay semantic cost when exact and graph signals are insufficient
4. preserve explainability and provenance
5. update reinforcement or decay after outcome feedback

### Failure risks
- candidate explosion
- stale data bias
- low-confidence amplification
- cross-session interference

### Instrumentation
- p50/p95/p99 latency
- candidate counts before and after pruning
- success contribution estimate
- cache hit ratio
- repairability score

### Pseudocode
```text
fn pattern_449(ctx):
    budget = ctx.latency_budget
    exact = cheap_exact(ctx)
    graph = bounded_graph(ctx, exact)
    sem = maybe_semantic(ctx, budget, graph)
    merged = dedup_merge(exact, graph, sem)
    ranked = rank_with_policy(ctx, merged)
    return package(ctx, ranked[:ctx.limit])
```
## 450. Cache Invalidation pattern 30

### Objective
Optimize cache invalidation under bounded latency and bounded noise.

### Preconditions
- namespace is validated
- policy context is loaded
- latency budget is available
- tier capabilities are known

### Heuristic
1. start with the cheapest exact signals
2. cap expansion by query class
3. only pay semantic cost when exact and graph signals are insufficient
4. preserve explainability and provenance
5. update reinforcement or decay after outcome feedback

### Failure risks
- candidate explosion
- stale data bias
- low-confidence amplification
- cross-session interference

### Instrumentation
- p50/p95/p99 latency
- candidate counts before and after pruning
- success contribution estimate
- cache hit ratio
- repairability score

### Pseudocode
```text
fn pattern_450(ctx):
    budget = ctx.latency_budget
    exact = cheap_exact(ctx)
    graph = bounded_graph(ctx, exact)
    sem = maybe_semantic(ctx, budget, graph)
    merged = dedup_merge(exact, graph, sem)
    ranked = rank_with_policy(ctx, merged)
    return package(ctx, ranked[:ctx.limit])
```
## 451. Negative Caching pattern 1

### Objective
Optimize negative caching under bounded latency and bounded noise.

### Preconditions
- namespace is validated
- policy context is loaded
- latency budget is available
- tier capabilities are known

### Heuristic
1. start with the cheapest exact signals
2. cap expansion by query class
3. only pay semantic cost when exact and graph signals are insufficient
4. preserve explainability and provenance
5. update reinforcement or decay after outcome feedback

### Failure risks
- candidate explosion
- stale data bias
- low-confidence amplification
- cross-session interference

### Instrumentation
- p50/p95/p99 latency
- candidate counts before and after pruning
- success contribution estimate
- cache hit ratio
- repairability score

### Pseudocode
```text
fn pattern_451(ctx):
    budget = ctx.latency_budget
    exact = cheap_exact(ctx)
    graph = bounded_graph(ctx, exact)
    sem = maybe_semantic(ctx, budget, graph)
    merged = dedup_merge(exact, graph, sem)
    ranked = rank_with_policy(ctx, merged)
    return package(ctx, ranked[:ctx.limit])
```
## 452. Negative Caching pattern 2

### Objective
Optimize negative caching under bounded latency and bounded noise.

### Preconditions
- namespace is validated
- policy context is loaded
- latency budget is available
- tier capabilities are known

### Heuristic
1. start with the cheapest exact signals
2. cap expansion by query class
3. only pay semantic cost when exact and graph signals are insufficient
4. preserve explainability and provenance
5. update reinforcement or decay after outcome feedback

### Failure risks
- candidate explosion
- stale data bias
- low-confidence amplification
- cross-session interference

### Instrumentation
- p50/p95/p99 latency
- candidate counts before and after pruning
- success contribution estimate
- cache hit ratio
- repairability score

### Pseudocode
```text
fn pattern_452(ctx):
    budget = ctx.latency_budget
    exact = cheap_exact(ctx)
    graph = bounded_graph(ctx, exact)
    sem = maybe_semantic(ctx, budget, graph)
    merged = dedup_merge(exact, graph, sem)
    ranked = rank_with_policy(ctx, merged)
    return package(ctx, ranked[:ctx.limit])
```
## 453. Negative Caching pattern 3

### Objective
Optimize negative caching under bounded latency and bounded noise.

### Preconditions
- namespace is validated
- policy context is loaded
- latency budget is available
- tier capabilities are known

### Heuristic
1. start with the cheapest exact signals
2. cap expansion by query class
3. only pay semantic cost when exact and graph signals are insufficient
4. preserve explainability and provenance
5. update reinforcement or decay after outcome feedback

### Failure risks
- candidate explosion
- stale data bias
- low-confidence amplification
- cross-session interference

### Instrumentation
- p50/p95/p99 latency
- candidate counts before and after pruning
- success contribution estimate
- cache hit ratio
- repairability score

### Pseudocode
```text
fn pattern_453(ctx):
    budget = ctx.latency_budget
    exact = cheap_exact(ctx)
    graph = bounded_graph(ctx, exact)
    sem = maybe_semantic(ctx, budget, graph)
    merged = dedup_merge(exact, graph, sem)
    ranked = rank_with_policy(ctx, merged)
    return package(ctx, ranked[:ctx.limit])
```
## 454. Negative Caching pattern 4

### Objective
Optimize negative caching under bounded latency and bounded noise.

### Preconditions
- namespace is validated
- policy context is loaded
- latency budget is available
- tier capabilities are known

### Heuristic
1. start with the cheapest exact signals
2. cap expansion by query class
3. only pay semantic cost when exact and graph signals are insufficient
4. preserve explainability and provenance
5. update reinforcement or decay after outcome feedback

### Failure risks
- candidate explosion
- stale data bias
- low-confidence amplification
- cross-session interference

### Instrumentation
- p50/p95/p99 latency
- candidate counts before and after pruning
- success contribution estimate
- cache hit ratio
- repairability score

### Pseudocode
```text
fn pattern_454(ctx):
    budget = ctx.latency_budget
    exact = cheap_exact(ctx)
    graph = bounded_graph(ctx, exact)
    sem = maybe_semantic(ctx, budget, graph)
    merged = dedup_merge(exact, graph, sem)
    ranked = rank_with_policy(ctx, merged)
    return package(ctx, ranked[:ctx.limit])
```
## 455. Negative Caching pattern 5

### Objective
Optimize negative caching under bounded latency and bounded noise.

### Preconditions
- namespace is validated
- policy context is loaded
- latency budget is available
- tier capabilities are known

### Heuristic
1. start with the cheapest exact signals
2. cap expansion by query class
3. only pay semantic cost when exact and graph signals are insufficient
4. preserve explainability and provenance
5. update reinforcement or decay after outcome feedback

### Failure risks
- candidate explosion
- stale data bias
- low-confidence amplification
- cross-session interference

### Instrumentation
- p50/p95/p99 latency
- candidate counts before and after pruning
- success contribution estimate
- cache hit ratio
- repairability score

### Pseudocode
```text
fn pattern_455(ctx):
    budget = ctx.latency_budget
    exact = cheap_exact(ctx)
    graph = bounded_graph(ctx, exact)
    sem = maybe_semantic(ctx, budget, graph)
    merged = dedup_merge(exact, graph, sem)
    ranked = rank_with_policy(ctx, merged)
    return package(ctx, ranked[:ctx.limit])
```
## 456. Negative Caching pattern 6

### Objective
Optimize negative caching under bounded latency and bounded noise.

### Preconditions
- namespace is validated
- policy context is loaded
- latency budget is available
- tier capabilities are known

### Heuristic
1. start with the cheapest exact signals
2. cap expansion by query class
3. only pay semantic cost when exact and graph signals are insufficient
4. preserve explainability and provenance
5. update reinforcement or decay after outcome feedback

### Failure risks
- candidate explosion
- stale data bias
- low-confidence amplification
- cross-session interference

### Instrumentation
- p50/p95/p99 latency
- candidate counts before and after pruning
- success contribution estimate
- cache hit ratio
- repairability score

### Pseudocode
```text
fn pattern_456(ctx):
    budget = ctx.latency_budget
    exact = cheap_exact(ctx)
    graph = bounded_graph(ctx, exact)
    sem = maybe_semantic(ctx, budget, graph)
    merged = dedup_merge(exact, graph, sem)
    ranked = rank_with_policy(ctx, merged)
    return package(ctx, ranked[:ctx.limit])
```
## 457. Negative Caching pattern 7

### Objective
Optimize negative caching under bounded latency and bounded noise.

### Preconditions
- namespace is validated
- policy context is loaded
- latency budget is available
- tier capabilities are known

### Heuristic
1. start with the cheapest exact signals
2. cap expansion by query class
3. only pay semantic cost when exact and graph signals are insufficient
4. preserve explainability and provenance
5. update reinforcement or decay after outcome feedback

### Failure risks
- candidate explosion
- stale data bias
- low-confidence amplification
- cross-session interference

### Instrumentation
- p50/p95/p99 latency
- candidate counts before and after pruning
- success contribution estimate
- cache hit ratio
- repairability score

### Pseudocode
```text
fn pattern_457(ctx):
    budget = ctx.latency_budget
    exact = cheap_exact(ctx)
    graph = bounded_graph(ctx, exact)
    sem = maybe_semantic(ctx, budget, graph)
    merged = dedup_merge(exact, graph, sem)
    ranked = rank_with_policy(ctx, merged)
    return package(ctx, ranked[:ctx.limit])
```
## 458. Negative Caching pattern 8

### Objective
Optimize negative caching under bounded latency and bounded noise.

### Preconditions
- namespace is validated
- policy context is loaded
- latency budget is available
- tier capabilities are known

### Heuristic
1. start with the cheapest exact signals
2. cap expansion by query class
3. only pay semantic cost when exact and graph signals are insufficient
4. preserve explainability and provenance
5. update reinforcement or decay after outcome feedback

### Failure risks
- candidate explosion
- stale data bias
- low-confidence amplification
- cross-session interference

### Instrumentation
- p50/p95/p99 latency
- candidate counts before and after pruning
- success contribution estimate
- cache hit ratio
- repairability score

### Pseudocode
```text
fn pattern_458(ctx):
    budget = ctx.latency_budget
    exact = cheap_exact(ctx)
    graph = bounded_graph(ctx, exact)
    sem = maybe_semantic(ctx, budget, graph)
    merged = dedup_merge(exact, graph, sem)
    ranked = rank_with_policy(ctx, merged)
    return package(ctx, ranked[:ctx.limit])
```
## 459. Negative Caching pattern 9

### Objective
Optimize negative caching under bounded latency and bounded noise.

### Preconditions
- namespace is validated
- policy context is loaded
- latency budget is available
- tier capabilities are known

### Heuristic
1. start with the cheapest exact signals
2. cap expansion by query class
3. only pay semantic cost when exact and graph signals are insufficient
4. preserve explainability and provenance
5. update reinforcement or decay after outcome feedback

### Failure risks
- candidate explosion
- stale data bias
- low-confidence amplification
- cross-session interference

### Instrumentation
- p50/p95/p99 latency
- candidate counts before and after pruning
- success contribution estimate
- cache hit ratio
- repairability score

### Pseudocode
```text
fn pattern_459(ctx):
    budget = ctx.latency_budget
    exact = cheap_exact(ctx)
    graph = bounded_graph(ctx, exact)
    sem = maybe_semantic(ctx, budget, graph)
    merged = dedup_merge(exact, graph, sem)
    ranked = rank_with_policy(ctx, merged)
    return package(ctx, ranked[:ctx.limit])
```
## 460. Negative Caching pattern 10

### Objective
Optimize negative caching under bounded latency and bounded noise.

### Preconditions
- namespace is validated
- policy context is loaded
- latency budget is available
- tier capabilities are known

### Heuristic
1. start with the cheapest exact signals
2. cap expansion by query class
3. only pay semantic cost when exact and graph signals are insufficient
4. preserve explainability and provenance
5. update reinforcement or decay after outcome feedback

### Failure risks
- candidate explosion
- stale data bias
- low-confidence amplification
- cross-session interference

### Instrumentation
- p50/p95/p99 latency
- candidate counts before and after pruning
- success contribution estimate
- cache hit ratio
- repairability score

### Pseudocode
```text
fn pattern_460(ctx):
    budget = ctx.latency_budget
    exact = cheap_exact(ctx)
    graph = bounded_graph(ctx, exact)
    sem = maybe_semantic(ctx, budget, graph)
    merged = dedup_merge(exact, graph, sem)
    ranked = rank_with_policy(ctx, merged)
    return package(ctx, ranked[:ctx.limit])
```
## 461. Negative Caching pattern 11

### Objective
Optimize negative caching under bounded latency and bounded noise.

### Preconditions
- namespace is validated
- policy context is loaded
- latency budget is available
- tier capabilities are known

### Heuristic
1. start with the cheapest exact signals
2. cap expansion by query class
3. only pay semantic cost when exact and graph signals are insufficient
4. preserve explainability and provenance
5. update reinforcement or decay after outcome feedback

### Failure risks
- candidate explosion
- stale data bias
- low-confidence amplification
- cross-session interference

### Instrumentation
- p50/p95/p99 latency
- candidate counts before and after pruning
- success contribution estimate
- cache hit ratio
- repairability score

### Pseudocode
```text
fn pattern_461(ctx):
    budget = ctx.latency_budget
    exact = cheap_exact(ctx)
    graph = bounded_graph(ctx, exact)
    sem = maybe_semantic(ctx, budget, graph)
    merged = dedup_merge(exact, graph, sem)
    ranked = rank_with_policy(ctx, merged)
    return package(ctx, ranked[:ctx.limit])
```
## 462. Negative Caching pattern 12

### Objective
Optimize negative caching under bounded latency and bounded noise.

### Preconditions
- namespace is validated
- policy context is loaded
- latency budget is available
- tier capabilities are known

### Heuristic
1. start with the cheapest exact signals
2. cap expansion by query class
3. only pay semantic cost when exact and graph signals are insufficient
4. preserve explainability and provenance
5. update reinforcement or decay after outcome feedback

### Failure risks
- candidate explosion
- stale data bias
- low-confidence amplification
- cross-session interference

### Instrumentation
- p50/p95/p99 latency
- candidate counts before and after pruning
- success contribution estimate
- cache hit ratio
- repairability score

### Pseudocode
```text
fn pattern_462(ctx):
    budget = ctx.latency_budget
    exact = cheap_exact(ctx)
    graph = bounded_graph(ctx, exact)
    sem = maybe_semantic(ctx, budget, graph)
    merged = dedup_merge(exact, graph, sem)
    ranked = rank_with_policy(ctx, merged)
    return package(ctx, ranked[:ctx.limit])
```
## 463. Negative Caching pattern 13

### Objective
Optimize negative caching under bounded latency and bounded noise.

### Preconditions
- namespace is validated
- policy context is loaded
- latency budget is available
- tier capabilities are known

### Heuristic
1. start with the cheapest exact signals
2. cap expansion by query class
3. only pay semantic cost when exact and graph signals are insufficient
4. preserve explainability and provenance
5. update reinforcement or decay after outcome feedback

### Failure risks
- candidate explosion
- stale data bias
- low-confidence amplification
- cross-session interference

### Instrumentation
- p50/p95/p99 latency
- candidate counts before and after pruning
- success contribution estimate
- cache hit ratio
- repairability score

### Pseudocode
```text
fn pattern_463(ctx):
    budget = ctx.latency_budget
    exact = cheap_exact(ctx)
    graph = bounded_graph(ctx, exact)
    sem = maybe_semantic(ctx, budget, graph)
    merged = dedup_merge(exact, graph, sem)
    ranked = rank_with_policy(ctx, merged)
    return package(ctx, ranked[:ctx.limit])
```
## 464. Negative Caching pattern 14

### Objective
Optimize negative caching under bounded latency and bounded noise.

### Preconditions
- namespace is validated
- policy context is loaded
- latency budget is available
- tier capabilities are known

### Heuristic
1. start with the cheapest exact signals
2. cap expansion by query class
3. only pay semantic cost when exact and graph signals are insufficient
4. preserve explainability and provenance
5. update reinforcement or decay after outcome feedback

### Failure risks
- candidate explosion
- stale data bias
- low-confidence amplification
- cross-session interference

### Instrumentation
- p50/p95/p99 latency
- candidate counts before and after pruning
- success contribution estimate
- cache hit ratio
- repairability score

### Pseudocode
```text
fn pattern_464(ctx):
    budget = ctx.latency_budget
    exact = cheap_exact(ctx)
    graph = bounded_graph(ctx, exact)
    sem = maybe_semantic(ctx, budget, graph)
    merged = dedup_merge(exact, graph, sem)
    ranked = rank_with_policy(ctx, merged)
    return package(ctx, ranked[:ctx.limit])
```
## 465. Negative Caching pattern 15

### Objective
Optimize negative caching under bounded latency and bounded noise.

### Preconditions
- namespace is validated
- policy context is loaded
- latency budget is available
- tier capabilities are known

### Heuristic
1. start with the cheapest exact signals
2. cap expansion by query class
3. only pay semantic cost when exact and graph signals are insufficient
4. preserve explainability and provenance
5. update reinforcement or decay after outcome feedback

### Failure risks
- candidate explosion
- stale data bias
- low-confidence amplification
- cross-session interference

### Instrumentation
- p50/p95/p99 latency
- candidate counts before and after pruning
- success contribution estimate
- cache hit ratio
- repairability score

### Pseudocode
```text
fn pattern_465(ctx):
    budget = ctx.latency_budget
    exact = cheap_exact(ctx)
    graph = bounded_graph(ctx, exact)
    sem = maybe_semantic(ctx, budget, graph)
    merged = dedup_merge(exact, graph, sem)
    ranked = rank_with_policy(ctx, merged)
    return package(ctx, ranked[:ctx.limit])
```
## 466. Negative Caching pattern 16

### Objective
Optimize negative caching under bounded latency and bounded noise.

### Preconditions
- namespace is validated
- policy context is loaded
- latency budget is available
- tier capabilities are known

### Heuristic
1. start with the cheapest exact signals
2. cap expansion by query class
3. only pay semantic cost when exact and graph signals are insufficient
4. preserve explainability and provenance
5. update reinforcement or decay after outcome feedback

### Failure risks
- candidate explosion
- stale data bias
- low-confidence amplification
- cross-session interference

### Instrumentation
- p50/p95/p99 latency
- candidate counts before and after pruning
- success contribution estimate
- cache hit ratio
- repairability score

### Pseudocode
```text
fn pattern_466(ctx):
    budget = ctx.latency_budget
    exact = cheap_exact(ctx)
    graph = bounded_graph(ctx, exact)
    sem = maybe_semantic(ctx, budget, graph)
    merged = dedup_merge(exact, graph, sem)
    ranked = rank_with_policy(ctx, merged)
    return package(ctx, ranked[:ctx.limit])
```
## 467. Negative Caching pattern 17

### Objective
Optimize negative caching under bounded latency and bounded noise.

### Preconditions
- namespace is validated
- policy context is loaded
- latency budget is available
- tier capabilities are known

### Heuristic
1. start with the cheapest exact signals
2. cap expansion by query class
3. only pay semantic cost when exact and graph signals are insufficient
4. preserve explainability and provenance
5. update reinforcement or decay after outcome feedback

### Failure risks
- candidate explosion
- stale data bias
- low-confidence amplification
- cross-session interference

### Instrumentation
- p50/p95/p99 latency
- candidate counts before and after pruning
- success contribution estimate
- cache hit ratio
- repairability score

### Pseudocode
```text
fn pattern_467(ctx):
    budget = ctx.latency_budget
    exact = cheap_exact(ctx)
    graph = bounded_graph(ctx, exact)
    sem = maybe_semantic(ctx, budget, graph)
    merged = dedup_merge(exact, graph, sem)
    ranked = rank_with_policy(ctx, merged)
    return package(ctx, ranked[:ctx.limit])
```
## 468. Negative Caching pattern 18

### Objective
Optimize negative caching under bounded latency and bounded noise.

### Preconditions
- namespace is validated
- policy context is loaded
- latency budget is available
- tier capabilities are known

### Heuristic
1. start with the cheapest exact signals
2. cap expansion by query class
3. only pay semantic cost when exact and graph signals are insufficient
4. preserve explainability and provenance
5. update reinforcement or decay after outcome feedback

### Failure risks
- candidate explosion
- stale data bias
- low-confidence amplification
- cross-session interference

### Instrumentation
- p50/p95/p99 latency
- candidate counts before and after pruning
- success contribution estimate
- cache hit ratio
- repairability score

### Pseudocode
```text
fn pattern_468(ctx):
    budget = ctx.latency_budget
    exact = cheap_exact(ctx)
    graph = bounded_graph(ctx, exact)
    sem = maybe_semantic(ctx, budget, graph)
    merged = dedup_merge(exact, graph, sem)
    ranked = rank_with_policy(ctx, merged)
    return package(ctx, ranked[:ctx.limit])
```
## 469. Negative Caching pattern 19

### Objective
Optimize negative caching under bounded latency and bounded noise.

### Preconditions
- namespace is validated
- policy context is loaded
- latency budget is available
- tier capabilities are known

### Heuristic
1. start with the cheapest exact signals
2. cap expansion by query class
3. only pay semantic cost when exact and graph signals are insufficient
4. preserve explainability and provenance
5. update reinforcement or decay after outcome feedback

### Failure risks
- candidate explosion
- stale data bias
- low-confidence amplification
- cross-session interference

### Instrumentation
- p50/p95/p99 latency
- candidate counts before and after pruning
- success contribution estimate
- cache hit ratio
- repairability score

### Pseudocode
```text
fn pattern_469(ctx):
    budget = ctx.latency_budget
    exact = cheap_exact(ctx)
    graph = bounded_graph(ctx, exact)
    sem = maybe_semantic(ctx, budget, graph)
    merged = dedup_merge(exact, graph, sem)
    ranked = rank_with_policy(ctx, merged)
    return package(ctx, ranked[:ctx.limit])
```
## 470. Negative Caching pattern 20

### Objective
Optimize negative caching under bounded latency and bounded noise.

### Preconditions
- namespace is validated
- policy context is loaded
- latency budget is available
- tier capabilities are known

### Heuristic
1. start with the cheapest exact signals
2. cap expansion by query class
3. only pay semantic cost when exact and graph signals are insufficient
4. preserve explainability and provenance
5. update reinforcement or decay after outcome feedback

### Failure risks
- candidate explosion
- stale data bias
- low-confidence amplification
- cross-session interference

### Instrumentation
- p50/p95/p99 latency
- candidate counts before and after pruning
- success contribution estimate
- cache hit ratio
- repairability score

### Pseudocode
```text
fn pattern_470(ctx):
    budget = ctx.latency_budget
    exact = cheap_exact(ctx)
    graph = bounded_graph(ctx, exact)
    sem = maybe_semantic(ctx, budget, graph)
    merged = dedup_merge(exact, graph, sem)
    ranked = rank_with_policy(ctx, merged)
    return package(ctx, ranked[:ctx.limit])
```
## 471. Negative Caching pattern 21

### Objective
Optimize negative caching under bounded latency and bounded noise.

### Preconditions
- namespace is validated
- policy context is loaded
- latency budget is available
- tier capabilities are known

### Heuristic
1. start with the cheapest exact signals
2. cap expansion by query class
3. only pay semantic cost when exact and graph signals are insufficient
4. preserve explainability and provenance
5. update reinforcement or decay after outcome feedback

### Failure risks
- candidate explosion
- stale data bias
- low-confidence amplification
- cross-session interference

### Instrumentation
- p50/p95/p99 latency
- candidate counts before and after pruning
- success contribution estimate
- cache hit ratio
- repairability score

### Pseudocode
```text
fn pattern_471(ctx):
    budget = ctx.latency_budget
    exact = cheap_exact(ctx)
    graph = bounded_graph(ctx, exact)
    sem = maybe_semantic(ctx, budget, graph)
    merged = dedup_merge(exact, graph, sem)
    ranked = rank_with_policy(ctx, merged)
    return package(ctx, ranked[:ctx.limit])
```
## 472. Negative Caching pattern 22

### Objective
Optimize negative caching under bounded latency and bounded noise.

### Preconditions
- namespace is validated
- policy context is loaded
- latency budget is available
- tier capabilities are known

### Heuristic
1. start with the cheapest exact signals
2. cap expansion by query class
3. only pay semantic cost when exact and graph signals are insufficient
4. preserve explainability and provenance
5. update reinforcement or decay after outcome feedback

### Failure risks
- candidate explosion
- stale data bias
- low-confidence amplification
- cross-session interference

### Instrumentation
- p50/p95/p99 latency
- candidate counts before and after pruning
- success contribution estimate
- cache hit ratio
- repairability score

### Pseudocode
```text
fn pattern_472(ctx):
    budget = ctx.latency_budget
    exact = cheap_exact(ctx)
    graph = bounded_graph(ctx, exact)
    sem = maybe_semantic(ctx, budget, graph)
    merged = dedup_merge(exact, graph, sem)
    ranked = rank_with_policy(ctx, merged)
    return package(ctx, ranked[:ctx.limit])
```
## 473. Negative Caching pattern 23

### Objective
Optimize negative caching under bounded latency and bounded noise.

### Preconditions
- namespace is validated
- policy context is loaded
- latency budget is available
- tier capabilities are known

### Heuristic
1. start with the cheapest exact signals
2. cap expansion by query class
3. only pay semantic cost when exact and graph signals are insufficient
4. preserve explainability and provenance
5. update reinforcement or decay after outcome feedback

### Failure risks
- candidate explosion
- stale data bias
- low-confidence amplification
- cross-session interference

### Instrumentation
- p50/p95/p99 latency
- candidate counts before and after pruning
- success contribution estimate
- cache hit ratio
- repairability score

### Pseudocode
```text
fn pattern_473(ctx):
    budget = ctx.latency_budget
    exact = cheap_exact(ctx)
    graph = bounded_graph(ctx, exact)
    sem = maybe_semantic(ctx, budget, graph)
    merged = dedup_merge(exact, graph, sem)
    ranked = rank_with_policy(ctx, merged)
    return package(ctx, ranked[:ctx.limit])
```
## 474. Negative Caching pattern 24

### Objective
Optimize negative caching under bounded latency and bounded noise.

### Preconditions
- namespace is validated
- policy context is loaded
- latency budget is available
- tier capabilities are known

### Heuristic
1. start with the cheapest exact signals
2. cap expansion by query class
3. only pay semantic cost when exact and graph signals are insufficient
4. preserve explainability and provenance
5. update reinforcement or decay after outcome feedback

### Failure risks
- candidate explosion
- stale data bias
- low-confidence amplification
- cross-session interference

### Instrumentation
- p50/p95/p99 latency
- candidate counts before and after pruning
- success contribution estimate
- cache hit ratio
- repairability score

### Pseudocode
```text
fn pattern_474(ctx):
    budget = ctx.latency_budget
    exact = cheap_exact(ctx)
    graph = bounded_graph(ctx, exact)
    sem = maybe_semantic(ctx, budget, graph)
    merged = dedup_merge(exact, graph, sem)
    ranked = rank_with_policy(ctx, merged)
    return package(ctx, ranked[:ctx.limit])
```
## 475. Negative Caching pattern 25

### Objective
Optimize negative caching under bounded latency and bounded noise.

### Preconditions
- namespace is validated
- policy context is loaded
- latency budget is available
- tier capabilities are known

### Heuristic
1. start with the cheapest exact signals
2. cap expansion by query class
3. only pay semantic cost when exact and graph signals are insufficient
4. preserve explainability and provenance
5. update reinforcement or decay after outcome feedback

### Failure risks
- candidate explosion
- stale data bias
- low-confidence amplification
- cross-session interference

### Instrumentation
- p50/p95/p99 latency
- candidate counts before and after pruning
- success contribution estimate
- cache hit ratio
- repairability score

### Pseudocode
```text
fn pattern_475(ctx):
    budget = ctx.latency_budget
    exact = cheap_exact(ctx)
    graph = bounded_graph(ctx, exact)
    sem = maybe_semantic(ctx, budget, graph)
    merged = dedup_merge(exact, graph, sem)
    ranked = rank_with_policy(ctx, merged)
    return package(ctx, ranked[:ctx.limit])
```
## 476. Negative Caching pattern 26

### Objective
Optimize negative caching under bounded latency and bounded noise.

### Preconditions
- namespace is validated
- policy context is loaded
- latency budget is available
- tier capabilities are known

### Heuristic
1. start with the cheapest exact signals
2. cap expansion by query class
3. only pay semantic cost when exact and graph signals are insufficient
4. preserve explainability and provenance
5. update reinforcement or decay after outcome feedback

### Failure risks
- candidate explosion
- stale data bias
- low-confidence amplification
- cross-session interference

### Instrumentation
- p50/p95/p99 latency
- candidate counts before and after pruning
- success contribution estimate
- cache hit ratio
- repairability score

### Pseudocode
```text
fn pattern_476(ctx):
    budget = ctx.latency_budget
    exact = cheap_exact(ctx)
    graph = bounded_graph(ctx, exact)
    sem = maybe_semantic(ctx, budget, graph)
    merged = dedup_merge(exact, graph, sem)
    ranked = rank_with_policy(ctx, merged)
    return package(ctx, ranked[:ctx.limit])
```
## 477. Negative Caching pattern 27

### Objective
Optimize negative caching under bounded latency and bounded noise.

### Preconditions
- namespace is validated
- policy context is loaded
- latency budget is available
- tier capabilities are known

### Heuristic
1. start with the cheapest exact signals
2. cap expansion by query class
3. only pay semantic cost when exact and graph signals are insufficient
4. preserve explainability and provenance
5. update reinforcement or decay after outcome feedback

### Failure risks
- candidate explosion
- stale data bias
- low-confidence amplification
- cross-session interference

### Instrumentation
- p50/p95/p99 latency
- candidate counts before and after pruning
- success contribution estimate
- cache hit ratio
- repairability score

### Pseudocode
```text
fn pattern_477(ctx):
    budget = ctx.latency_budget
    exact = cheap_exact(ctx)
    graph = bounded_graph(ctx, exact)
    sem = maybe_semantic(ctx, budget, graph)
    merged = dedup_merge(exact, graph, sem)
    ranked = rank_with_policy(ctx, merged)
    return package(ctx, ranked[:ctx.limit])
```
## 478. Negative Caching pattern 28

### Objective
Optimize negative caching under bounded latency and bounded noise.

### Preconditions
- namespace is validated
- policy context is loaded
- latency budget is available
- tier capabilities are known

### Heuristic
1. start with the cheapest exact signals
2. cap expansion by query class
3. only pay semantic cost when exact and graph signals are insufficient
4. preserve explainability and provenance
5. update reinforcement or decay after outcome feedback

### Failure risks
- candidate explosion
- stale data bias
- low-confidence amplification
- cross-session interference

### Instrumentation
- p50/p95/p99 latency
- candidate counts before and after pruning
- success contribution estimate
- cache hit ratio
- repairability score

### Pseudocode
```text
fn pattern_478(ctx):
    budget = ctx.latency_budget
    exact = cheap_exact(ctx)
    graph = bounded_graph(ctx, exact)
    sem = maybe_semantic(ctx, budget, graph)
    merged = dedup_merge(exact, graph, sem)
    ranked = rank_with_policy(ctx, merged)
    return package(ctx, ranked[:ctx.limit])
```
## 479. Negative Caching pattern 29

### Objective
Optimize negative caching under bounded latency and bounded noise.

### Preconditions
- namespace is validated
- policy context is loaded
- latency budget is available
- tier capabilities are known

### Heuristic
1. start with the cheapest exact signals
2. cap expansion by query class
3. only pay semantic cost when exact and graph signals are insufficient
4. preserve explainability and provenance
5. update reinforcement or decay after outcome feedback

### Failure risks
- candidate explosion
- stale data bias
- low-confidence amplification
- cross-session interference

### Instrumentation
- p50/p95/p99 latency
- candidate counts before and after pruning
- success contribution estimate
- cache hit ratio
- repairability score

### Pseudocode
```text
fn pattern_479(ctx):
    budget = ctx.latency_budget
    exact = cheap_exact(ctx)
    graph = bounded_graph(ctx, exact)
    sem = maybe_semantic(ctx, budget, graph)
    merged = dedup_merge(exact, graph, sem)
    ranked = rank_with_policy(ctx, merged)
    return package(ctx, ranked[:ctx.limit])
```
## 480. Negative Caching pattern 30

### Objective
Optimize negative caching under bounded latency and bounded noise.

### Preconditions
- namespace is validated
- policy context is loaded
- latency budget is available
- tier capabilities are known

### Heuristic
1. start with the cheapest exact signals
2. cap expansion by query class
3. only pay semantic cost when exact and graph signals are insufficient
4. preserve explainability and provenance
5. update reinforcement or decay after outcome feedback

### Failure risks
- candidate explosion
- stale data bias
- low-confidence amplification
- cross-session interference

### Instrumentation
- p50/p95/p99 latency
- candidate counts before and after pruning
- success contribution estimate
- cache hit ratio
- repairability score

### Pseudocode
```text
fn pattern_480(ctx):
    budget = ctx.latency_budget
    exact = cheap_exact(ctx)
    graph = bounded_graph(ctx, exact)
    sem = maybe_semantic(ctx, budget, graph)
    merged = dedup_merge(exact, graph, sem)
    ranked = rank_with_policy(ctx, merged)
    return package(ctx, ranked[:ctx.limit])
```
## 481. Retention Enforcement pattern 1

### Objective
Optimize retention enforcement under bounded latency and bounded noise.

### Preconditions
- namespace is validated
- policy context is loaded
- latency budget is available
- tier capabilities are known

### Heuristic
1. start with the cheapest exact signals
2. cap expansion by query class
3. only pay semantic cost when exact and graph signals are insufficient
4. preserve explainability and provenance
5. update reinforcement or decay after outcome feedback

### Failure risks
- candidate explosion
- stale data bias
- low-confidence amplification
- cross-session interference

### Instrumentation
- p50/p95/p99 latency
- candidate counts before and after pruning
- success contribution estimate
- cache hit ratio
- repairability score

### Pseudocode
```text
fn pattern_481(ctx):
    budget = ctx.latency_budget
    exact = cheap_exact(ctx)
    graph = bounded_graph(ctx, exact)
    sem = maybe_semantic(ctx, budget, graph)
    merged = dedup_merge(exact, graph, sem)
    ranked = rank_with_policy(ctx, merged)
    return package(ctx, ranked[:ctx.limit])
```
## 482. Retention Enforcement pattern 2

### Objective
Optimize retention enforcement under bounded latency and bounded noise.

### Preconditions
- namespace is validated
- policy context is loaded
- latency budget is available
- tier capabilities are known

### Heuristic
1. start with the cheapest exact signals
2. cap expansion by query class
3. only pay semantic cost when exact and graph signals are insufficient
4. preserve explainability and provenance
5. update reinforcement or decay after outcome feedback

### Failure risks
- candidate explosion
- stale data bias
- low-confidence amplification
- cross-session interference

### Instrumentation
- p50/p95/p99 latency
- candidate counts before and after pruning
- success contribution estimate
- cache hit ratio
- repairability score

### Pseudocode
```text
fn pattern_482(ctx):
    budget = ctx.latency_budget
    exact = cheap_exact(ctx)
    graph = bounded_graph(ctx, exact)
    sem = maybe_semantic(ctx, budget, graph)
    merged = dedup_merge(exact, graph, sem)
    ranked = rank_with_policy(ctx, merged)
    return package(ctx, ranked[:ctx.limit])
```
## 483. Retention Enforcement pattern 3

### Objective
Optimize retention enforcement under bounded latency and bounded noise.

### Preconditions
- namespace is validated
- policy context is loaded
- latency budget is available
- tier capabilities are known

### Heuristic
1. start with the cheapest exact signals
2. cap expansion by query class
3. only pay semantic cost when exact and graph signals are insufficient
4. preserve explainability and provenance
5. update reinforcement or decay after outcome feedback

### Failure risks
- candidate explosion
- stale data bias
- low-confidence amplification
- cross-session interference

### Instrumentation
- p50/p95/p99 latency
- candidate counts before and after pruning
- success contribution estimate
- cache hit ratio
- repairability score

### Pseudocode
```text
fn pattern_483(ctx):
    budget = ctx.latency_budget
    exact = cheap_exact(ctx)
    graph = bounded_graph(ctx, exact)
    sem = maybe_semantic(ctx, budget, graph)
    merged = dedup_merge(exact, graph, sem)
    ranked = rank_with_policy(ctx, merged)
    return package(ctx, ranked[:ctx.limit])
```
## 484. Retention Enforcement pattern 4

### Objective
Optimize retention enforcement under bounded latency and bounded noise.

### Preconditions
- namespace is validated
- policy context is loaded
- latency budget is available
- tier capabilities are known

### Heuristic
1. start with the cheapest exact signals
2. cap expansion by query class
3. only pay semantic cost when exact and graph signals are insufficient
4. preserve explainability and provenance
5. update reinforcement or decay after outcome feedback

### Failure risks
- candidate explosion
- stale data bias
- low-confidence amplification
- cross-session interference

### Instrumentation
- p50/p95/p99 latency
- candidate counts before and after pruning
- success contribution estimate
- cache hit ratio
- repairability score

### Pseudocode
```text
fn pattern_484(ctx):
    budget = ctx.latency_budget
    exact = cheap_exact(ctx)
    graph = bounded_graph(ctx, exact)
    sem = maybe_semantic(ctx, budget, graph)
    merged = dedup_merge(exact, graph, sem)
    ranked = rank_with_policy(ctx, merged)
    return package(ctx, ranked[:ctx.limit])
```
## 485. Retention Enforcement pattern 5

### Objective
Optimize retention enforcement under bounded latency and bounded noise.

### Preconditions
- namespace is validated
- policy context is loaded
- latency budget is available
- tier capabilities are known

### Heuristic
1. start with the cheapest exact signals
2. cap expansion by query class
3. only pay semantic cost when exact and graph signals are insufficient
4. preserve explainability and provenance
5. update reinforcement or decay after outcome feedback

### Failure risks
- candidate explosion
- stale data bias
- low-confidence amplification
- cross-session interference

### Instrumentation
- p50/p95/p99 latency
- candidate counts before and after pruning
- success contribution estimate
- cache hit ratio
- repairability score

### Pseudocode
```text
fn pattern_485(ctx):
    budget = ctx.latency_budget
    exact = cheap_exact(ctx)
    graph = bounded_graph(ctx, exact)
    sem = maybe_semantic(ctx, budget, graph)
    merged = dedup_merge(exact, graph, sem)
    ranked = rank_with_policy(ctx, merged)
    return package(ctx, ranked[:ctx.limit])
```
## 486. Retention Enforcement pattern 6

### Objective
Optimize retention enforcement under bounded latency and bounded noise.

### Preconditions
- namespace is validated
- policy context is loaded
- latency budget is available
- tier capabilities are known

### Heuristic
1. start with the cheapest exact signals
2. cap expansion by query class
3. only pay semantic cost when exact and graph signals are insufficient
4. preserve explainability and provenance
5. update reinforcement or decay after outcome feedback

### Failure risks
- candidate explosion
- stale data bias
- low-confidence amplification
- cross-session interference

### Instrumentation
- p50/p95/p99 latency
- candidate counts before and after pruning
- success contribution estimate
- cache hit ratio
- repairability score

### Pseudocode
```text
fn pattern_486(ctx):
    budget = ctx.latency_budget
    exact = cheap_exact(ctx)
    graph = bounded_graph(ctx, exact)
    sem = maybe_semantic(ctx, budget, graph)
    merged = dedup_merge(exact, graph, sem)
    ranked = rank_with_policy(ctx, merged)
    return package(ctx, ranked[:ctx.limit])
```
## 487. Retention Enforcement pattern 7

### Objective
Optimize retention enforcement under bounded latency and bounded noise.

### Preconditions
- namespace is validated
- policy context is loaded
- latency budget is available
- tier capabilities are known

### Heuristic
1. start with the cheapest exact signals
2. cap expansion by query class
3. only pay semantic cost when exact and graph signals are insufficient
4. preserve explainability and provenance
5. update reinforcement or decay after outcome feedback

### Failure risks
- candidate explosion
- stale data bias
- low-confidence amplification
- cross-session interference

### Instrumentation
- p50/p95/p99 latency
- candidate counts before and after pruning
- success contribution estimate
- cache hit ratio
- repairability score

### Pseudocode
```text
fn pattern_487(ctx):
    budget = ctx.latency_budget
    exact = cheap_exact(ctx)
    graph = bounded_graph(ctx, exact)
    sem = maybe_semantic(ctx, budget, graph)
    merged = dedup_merge(exact, graph, sem)
    ranked = rank_with_policy(ctx, merged)
    return package(ctx, ranked[:ctx.limit])
```
## 488. Retention Enforcement pattern 8

### Objective
Optimize retention enforcement under bounded latency and bounded noise.

### Preconditions
- namespace is validated
- policy context is loaded
- latency budget is available
- tier capabilities are known

### Heuristic
1. start with the cheapest exact signals
2. cap expansion by query class
3. only pay semantic cost when exact and graph signals are insufficient
4. preserve explainability and provenance
5. update reinforcement or decay after outcome feedback

### Failure risks
- candidate explosion
- stale data bias
- low-confidence amplification
- cross-session interference

### Instrumentation
- p50/p95/p99 latency
- candidate counts before and after pruning
- success contribution estimate
- cache hit ratio
- repairability score

### Pseudocode
```text
fn pattern_488(ctx):
    budget = ctx.latency_budget
    exact = cheap_exact(ctx)
    graph = bounded_graph(ctx, exact)
    sem = maybe_semantic(ctx, budget, graph)
    merged = dedup_merge(exact, graph, sem)
    ranked = rank_with_policy(ctx, merged)
    return package(ctx, ranked[:ctx.limit])
```
## 489. Retention Enforcement pattern 9

### Objective
Optimize retention enforcement under bounded latency and bounded noise.

### Preconditions
- namespace is validated
- policy context is loaded
- latency budget is available
- tier capabilities are known

### Heuristic
1. start with the cheapest exact signals
2. cap expansion by query class
3. only pay semantic cost when exact and graph signals are insufficient
4. preserve explainability and provenance
5. update reinforcement or decay after outcome feedback

### Failure risks
- candidate explosion
- stale data bias
- low-confidence amplification
- cross-session interference

### Instrumentation
- p50/p95/p99 latency
- candidate counts before and after pruning
- success contribution estimate
- cache hit ratio
- repairability score

### Pseudocode
```text
fn pattern_489(ctx):
    budget = ctx.latency_budget
    exact = cheap_exact(ctx)
    graph = bounded_graph(ctx, exact)
    sem = maybe_semantic(ctx, budget, graph)
    merged = dedup_merge(exact, graph, sem)
    ranked = rank_with_policy(ctx, merged)
    return package(ctx, ranked[:ctx.limit])
```
## 490. Retention Enforcement pattern 10

### Objective
Optimize retention enforcement under bounded latency and bounded noise.

### Preconditions
- namespace is validated
- policy context is loaded
- latency budget is available
- tier capabilities are known

### Heuristic
1. start with the cheapest exact signals
2. cap expansion by query class
3. only pay semantic cost when exact and graph signals are insufficient
4. preserve explainability and provenance
5. update reinforcement or decay after outcome feedback

### Failure risks
- candidate explosion
- stale data bias
- low-confidence amplification
- cross-session interference

### Instrumentation
- p50/p95/p99 latency
- candidate counts before and after pruning
- success contribution estimate
- cache hit ratio
- repairability score

### Pseudocode
```text
fn pattern_490(ctx):
    budget = ctx.latency_budget
    exact = cheap_exact(ctx)
    graph = bounded_graph(ctx, exact)
    sem = maybe_semantic(ctx, budget, graph)
    merged = dedup_merge(exact, graph, sem)
    ranked = rank_with_policy(ctx, merged)
    return package(ctx, ranked[:ctx.limit])
```
## 491. Retention Enforcement pattern 11

### Objective
Optimize retention enforcement under bounded latency and bounded noise.

### Preconditions
- namespace is validated
- policy context is loaded
- latency budget is available
- tier capabilities are known

### Heuristic
1. start with the cheapest exact signals
2. cap expansion by query class
3. only pay semantic cost when exact and graph signals are insufficient
4. preserve explainability and provenance
5. update reinforcement or decay after outcome feedback

### Failure risks
- candidate explosion
- stale data bias
- low-confidence amplification
- cross-session interference

### Instrumentation
- p50/p95/p99 latency
- candidate counts before and after pruning
- success contribution estimate
- cache hit ratio
- repairability score

### Pseudocode
```text
fn pattern_491(ctx):
    budget = ctx.latency_budget
    exact = cheap_exact(ctx)
    graph = bounded_graph(ctx, exact)
    sem = maybe_semantic(ctx, budget, graph)
    merged = dedup_merge(exact, graph, sem)
    ranked = rank_with_policy(ctx, merged)
    return package(ctx, ranked[:ctx.limit])
```
## 492. Retention Enforcement pattern 12

### Objective
Optimize retention enforcement under bounded latency and bounded noise.

### Preconditions
- namespace is validated
- policy context is loaded
- latency budget is available
- tier capabilities are known

### Heuristic
1. start with the cheapest exact signals
2. cap expansion by query class
3. only pay semantic cost when exact and graph signals are insufficient
4. preserve explainability and provenance
5. update reinforcement or decay after outcome feedback

### Failure risks
- candidate explosion
- stale data bias
- low-confidence amplification
- cross-session interference

### Instrumentation
- p50/p95/p99 latency
- candidate counts before and after pruning
- success contribution estimate
- cache hit ratio
- repairability score

### Pseudocode
```text
fn pattern_492(ctx):
    budget = ctx.latency_budget
    exact = cheap_exact(ctx)
    graph = bounded_graph(ctx, exact)
    sem = maybe_semantic(ctx, budget, graph)
    merged = dedup_merge(exact, graph, sem)
    ranked = rank_with_policy(ctx, merged)
    return package(ctx, ranked[:ctx.limit])
```
## 493. Retention Enforcement pattern 13

### Objective
Optimize retention enforcement under bounded latency and bounded noise.

### Preconditions
- namespace is validated
- policy context is loaded
- latency budget is available
- tier capabilities are known

### Heuristic
1. start with the cheapest exact signals
2. cap expansion by query class
3. only pay semantic cost when exact and graph signals are insufficient
4. preserve explainability and provenance
5. update reinforcement or decay after outcome feedback

### Failure risks
- candidate explosion
- stale data bias
- low-confidence amplification
- cross-session interference

### Instrumentation
- p50/p95/p99 latency
- candidate counts before and after pruning
- success contribution estimate
- cache hit ratio
- repairability score

### Pseudocode
```text
fn pattern_493(ctx):
    budget = ctx.latency_budget
    exact = cheap_exact(ctx)
    graph = bounded_graph(ctx, exact)
    sem = maybe_semantic(ctx, budget, graph)
    merged = dedup_merge(exact, graph, sem)
    ranked = rank_with_policy(ctx, merged)
    return package(ctx, ranked[:ctx.limit])
```
## 494. Retention Enforcement pattern 14

### Objective
Optimize retention enforcement under bounded latency and bounded noise.

### Preconditions
- namespace is validated
- policy context is loaded
- latency budget is available
- tier capabilities are known

### Heuristic
1. start with the cheapest exact signals
2. cap expansion by query class
3. only pay semantic cost when exact and graph signals are insufficient
4. preserve explainability and provenance
5. update reinforcement or decay after outcome feedback

### Failure risks
- candidate explosion
- stale data bias
- low-confidence amplification
- cross-session interference

### Instrumentation
- p50/p95/p99 latency
- candidate counts before and after pruning
- success contribution estimate
- cache hit ratio
- repairability score

### Pseudocode
```text
fn pattern_494(ctx):
    budget = ctx.latency_budget
    exact = cheap_exact(ctx)
    graph = bounded_graph(ctx, exact)
    sem = maybe_semantic(ctx, budget, graph)
    merged = dedup_merge(exact, graph, sem)
    ranked = rank_with_policy(ctx, merged)
    return package(ctx, ranked[:ctx.limit])
```
## 495. Retention Enforcement pattern 15

### Objective
Optimize retention enforcement under bounded latency and bounded noise.

### Preconditions
- namespace is validated
- policy context is loaded
- latency budget is available
- tier capabilities are known

### Heuristic
1. start with the cheapest exact signals
2. cap expansion by query class
3. only pay semantic cost when exact and graph signals are insufficient
4. preserve explainability and provenance
5. update reinforcement or decay after outcome feedback

### Failure risks
- candidate explosion
- stale data bias
- low-confidence amplification
- cross-session interference

### Instrumentation
- p50/p95/p99 latency
- candidate counts before and after pruning
- success contribution estimate
- cache hit ratio
- repairability score

### Pseudocode
```text
fn pattern_495(ctx):
    budget = ctx.latency_budget
    exact = cheap_exact(ctx)
    graph = bounded_graph(ctx, exact)
    sem = maybe_semantic(ctx, budget, graph)
    merged = dedup_merge(exact, graph, sem)
    ranked = rank_with_policy(ctx, merged)
    return package(ctx, ranked[:ctx.limit])
```
## 496. Retention Enforcement pattern 16

### Objective
Optimize retention enforcement under bounded latency and bounded noise.

### Preconditions
- namespace is validated
- policy context is loaded
- latency budget is available
- tier capabilities are known

### Heuristic
1. start with the cheapest exact signals
2. cap expansion by query class
3. only pay semantic cost when exact and graph signals are insufficient
4. preserve explainability and provenance
5. update reinforcement or decay after outcome feedback

### Failure risks
- candidate explosion
- stale data bias
- low-confidence amplification
- cross-session interference

### Instrumentation
- p50/p95/p99 latency
- candidate counts before and after pruning
- success contribution estimate
- cache hit ratio
- repairability score

### Pseudocode
```text
fn pattern_496(ctx):
    budget = ctx.latency_budget
    exact = cheap_exact(ctx)
    graph = bounded_graph(ctx, exact)
    sem = maybe_semantic(ctx, budget, graph)
    merged = dedup_merge(exact, graph, sem)
    ranked = rank_with_policy(ctx, merged)
    return package(ctx, ranked[:ctx.limit])
```
## 497. Retention Enforcement pattern 17

### Objective
Optimize retention enforcement under bounded latency and bounded noise.

### Preconditions
- namespace is validated
- policy context is loaded
- latency budget is available
- tier capabilities are known

### Heuristic
1. start with the cheapest exact signals
2. cap expansion by query class
3. only pay semantic cost when exact and graph signals are insufficient
4. preserve explainability and provenance
5. update reinforcement or decay after outcome feedback

### Failure risks
- candidate explosion
- stale data bias
- low-confidence amplification
- cross-session interference

### Instrumentation
- p50/p95/p99 latency
- candidate counts before and after pruning
- success contribution estimate
- cache hit ratio
- repairability score

### Pseudocode
```text
fn pattern_497(ctx):
    budget = ctx.latency_budget
    exact = cheap_exact(ctx)
    graph = bounded_graph(ctx, exact)
    sem = maybe_semantic(ctx, budget, graph)
    merged = dedup_merge(exact, graph, sem)
    ranked = rank_with_policy(ctx, merged)
    return package(ctx, ranked[:ctx.limit])
```
## 498. Retention Enforcement pattern 18

### Objective
Optimize retention enforcement under bounded latency and bounded noise.

### Preconditions
- namespace is validated
- policy context is loaded
- latency budget is available
- tier capabilities are known

### Heuristic
1. start with the cheapest exact signals
2. cap expansion by query class
3. only pay semantic cost when exact and graph signals are insufficient
4. preserve explainability and provenance
5. update reinforcement or decay after outcome feedback

### Failure risks
- candidate explosion
- stale data bias
- low-confidence amplification
- cross-session interference

### Instrumentation
- p50/p95/p99 latency
- candidate counts before and after pruning
- success contribution estimate
- cache hit ratio
- repairability score

### Pseudocode
```text
fn pattern_498(ctx):
    budget = ctx.latency_budget
    exact = cheap_exact(ctx)
    graph = bounded_graph(ctx, exact)
    sem = maybe_semantic(ctx, budget, graph)
    merged = dedup_merge(exact, graph, sem)
    ranked = rank_with_policy(ctx, merged)
    return package(ctx, ranked[:ctx.limit])
```
## 499. Retention Enforcement pattern 19

### Objective
Optimize retention enforcement under bounded latency and bounded noise.

### Preconditions
- namespace is validated
- policy context is loaded
- latency budget is available
- tier capabilities are known

### Heuristic
1. start with the cheapest exact signals
2. cap expansion by query class
3. only pay semantic cost when exact and graph signals are insufficient
4. preserve explainability and provenance
5. update reinforcement or decay after outcome feedback

### Failure risks
- candidate explosion
- stale data bias
- low-confidence amplification
- cross-session interference

### Instrumentation
- p50/p95/p99 latency
- candidate counts before and after pruning
- success contribution estimate
- cache hit ratio
- repairability score

### Pseudocode
```text
fn pattern_499(ctx):
    budget = ctx.latency_budget
    exact = cheap_exact(ctx)
    graph = bounded_graph(ctx, exact)
    sem = maybe_semantic(ctx, budget, graph)
    merged = dedup_merge(exact, graph, sem)
    ranked = rank_with_policy(ctx, merged)
    return package(ctx, ranked[:ctx.limit])
```
## 500. Retention Enforcement pattern 20

### Objective
Optimize retention enforcement under bounded latency and bounded noise.

### Preconditions
- namespace is validated
- policy context is loaded
- latency budget is available
- tier capabilities are known

### Heuristic
1. start with the cheapest exact signals
2. cap expansion by query class
3. only pay semantic cost when exact and graph signals are insufficient
4. preserve explainability and provenance
5. update reinforcement or decay after outcome feedback

### Failure risks
- candidate explosion
- stale data bias
- low-confidence amplification
- cross-session interference

### Instrumentation
- p50/p95/p99 latency
- candidate counts before and after pruning
- success contribution estimate
- cache hit ratio
- repairability score

### Pseudocode
```text
fn pattern_500(ctx):
    budget = ctx.latency_budget
    exact = cheap_exact(ctx)
    graph = bounded_graph(ctx, exact)
    sem = maybe_semantic(ctx, budget, graph)
    merged = dedup_merge(exact, graph, sem)
    ranked = rank_with_policy(ctx, merged)
    return package(ctx, ranked[:ctx.limit])
```
## 501. Retention Enforcement pattern 21

### Objective
Optimize retention enforcement under bounded latency and bounded noise.

### Preconditions
- namespace is validated
- policy context is loaded
- latency budget is available
- tier capabilities are known

### Heuristic
1. start with the cheapest exact signals
2. cap expansion by query class
3. only pay semantic cost when exact and graph signals are insufficient
4. preserve explainability and provenance
5. update reinforcement or decay after outcome feedback

### Failure risks
- candidate explosion
- stale data bias
- low-confidence amplification
- cross-session interference

### Instrumentation
- p50/p95/p99 latency
- candidate counts before and after pruning
- success contribution estimate
- cache hit ratio
- repairability score

### Pseudocode
```text
fn pattern_501(ctx):
    budget = ctx.latency_budget
    exact = cheap_exact(ctx)
    graph = bounded_graph(ctx, exact)
    sem = maybe_semantic(ctx, budget, graph)
    merged = dedup_merge(exact, graph, sem)
    ranked = rank_with_policy(ctx, merged)
    return package(ctx, ranked[:ctx.limit])
```
## 502. Retention Enforcement pattern 22

### Objective
Optimize retention enforcement under bounded latency and bounded noise.

### Preconditions
- namespace is validated
- policy context is loaded
- latency budget is available
- tier capabilities are known

### Heuristic
1. start with the cheapest exact signals
2. cap expansion by query class
3. only pay semantic cost when exact and graph signals are insufficient
4. preserve explainability and provenance
5. update reinforcement or decay after outcome feedback

### Failure risks
- candidate explosion
- stale data bias
- low-confidence amplification
- cross-session interference

### Instrumentation
- p50/p95/p99 latency
- candidate counts before and after pruning
- success contribution estimate
- cache hit ratio
- repairability score

### Pseudocode
```text
fn pattern_502(ctx):
    budget = ctx.latency_budget
    exact = cheap_exact(ctx)
    graph = bounded_graph(ctx, exact)
    sem = maybe_semantic(ctx, budget, graph)
    merged = dedup_merge(exact, graph, sem)
    ranked = rank_with_policy(ctx, merged)
    return package(ctx, ranked[:ctx.limit])
```
## 503. Retention Enforcement pattern 23

### Objective
Optimize retention enforcement under bounded latency and bounded noise.

### Preconditions
- namespace is validated
- policy context is loaded
- latency budget is available
- tier capabilities are known

### Heuristic
1. start with the cheapest exact signals
2. cap expansion by query class
3. only pay semantic cost when exact and graph signals are insufficient
4. preserve explainability and provenance
5. update reinforcement or decay after outcome feedback

### Failure risks
- candidate explosion
- stale data bias
- low-confidence amplification
- cross-session interference

### Instrumentation
- p50/p95/p99 latency
- candidate counts before and after pruning
- success contribution estimate
- cache hit ratio
- repairability score

### Pseudocode
```text
fn pattern_503(ctx):
    budget = ctx.latency_budget
    exact = cheap_exact(ctx)
    graph = bounded_graph(ctx, exact)
    sem = maybe_semantic(ctx, budget, graph)
    merged = dedup_merge(exact, graph, sem)
    ranked = rank_with_policy(ctx, merged)
    return package(ctx, ranked[:ctx.limit])
```
## 504. Retention Enforcement pattern 24

### Objective
Optimize retention enforcement under bounded latency and bounded noise.

### Preconditions
- namespace is validated
- policy context is loaded
- latency budget is available
- tier capabilities are known

### Heuristic
1. start with the cheapest exact signals
2. cap expansion by query class
3. only pay semantic cost when exact and graph signals are insufficient
4. preserve explainability and provenance
5. update reinforcement or decay after outcome feedback

### Failure risks
- candidate explosion
- stale data bias
- low-confidence amplification
- cross-session interference

### Instrumentation
- p50/p95/p99 latency
- candidate counts before and after pruning
- success contribution estimate
- cache hit ratio
- repairability score

### Pseudocode
```text
fn pattern_504(ctx):
    budget = ctx.latency_budget
    exact = cheap_exact(ctx)
    graph = bounded_graph(ctx, exact)
    sem = maybe_semantic(ctx, budget, graph)
    merged = dedup_merge(exact, graph, sem)
    ranked = rank_with_policy(ctx, merged)
    return package(ctx, ranked[:ctx.limit])
```
## 505. Retention Enforcement pattern 25

### Objective
Optimize retention enforcement under bounded latency and bounded noise.

### Preconditions
- namespace is validated
- policy context is loaded
- latency budget is available
- tier capabilities are known

### Heuristic
1. start with the cheapest exact signals
2. cap expansion by query class
3. only pay semantic cost when exact and graph signals are insufficient
4. preserve explainability and provenance
5. update reinforcement or decay after outcome feedback

### Failure risks
- candidate explosion
- stale data bias
- low-confidence amplification
- cross-session interference

### Instrumentation
- p50/p95/p99 latency
- candidate counts before and after pruning
- success contribution estimate
- cache hit ratio
- repairability score

### Pseudocode
```text
fn pattern_505(ctx):
    budget = ctx.latency_budget
    exact = cheap_exact(ctx)
    graph = bounded_graph(ctx, exact)
    sem = maybe_semantic(ctx, budget, graph)
    merged = dedup_merge(exact, graph, sem)
    ranked = rank_with_policy(ctx, merged)
    return package(ctx, ranked[:ctx.limit])
```
## 506. Retention Enforcement pattern 26

### Objective
Optimize retention enforcement under bounded latency and bounded noise.

### Preconditions
- namespace is validated
- policy context is loaded
- latency budget is available
- tier capabilities are known

### Heuristic
1. start with the cheapest exact signals
2. cap expansion by query class
3. only pay semantic cost when exact and graph signals are insufficient
4. preserve explainability and provenance
5. update reinforcement or decay after outcome feedback

### Failure risks
- candidate explosion
- stale data bias
- low-confidence amplification
- cross-session interference

### Instrumentation
- p50/p95/p99 latency
- candidate counts before and after pruning
- success contribution estimate
- cache hit ratio
- repairability score

### Pseudocode
```text
fn pattern_506(ctx):
    budget = ctx.latency_budget
    exact = cheap_exact(ctx)
    graph = bounded_graph(ctx, exact)
    sem = maybe_semantic(ctx, budget, graph)
    merged = dedup_merge(exact, graph, sem)
    ranked = rank_with_policy(ctx, merged)
    return package(ctx, ranked[:ctx.limit])
```
## 507. Retention Enforcement pattern 27

### Objective
Optimize retention enforcement under bounded latency and bounded noise.

### Preconditions
- namespace is validated
- policy context is loaded
- latency budget is available
- tier capabilities are known

### Heuristic
1. start with the cheapest exact signals
2. cap expansion by query class
3. only pay semantic cost when exact and graph signals are insufficient
4. preserve explainability and provenance
5. update reinforcement or decay after outcome feedback

### Failure risks
- candidate explosion
- stale data bias
- low-confidence amplification
- cross-session interference

### Instrumentation
- p50/p95/p99 latency
- candidate counts before and after pruning
- success contribution estimate
- cache hit ratio
- repairability score

### Pseudocode
```text
fn pattern_507(ctx):
    budget = ctx.latency_budget
    exact = cheap_exact(ctx)
    graph = bounded_graph(ctx, exact)
    sem = maybe_semantic(ctx, budget, graph)
    merged = dedup_merge(exact, graph, sem)
    ranked = rank_with_policy(ctx, merged)
    return package(ctx, ranked[:ctx.limit])
```
## 508. Retention Enforcement pattern 28

### Objective
Optimize retention enforcement under bounded latency and bounded noise.

### Preconditions
- namespace is validated
- policy context is loaded
- latency budget is available
- tier capabilities are known

### Heuristic
1. start with the cheapest exact signals
2. cap expansion by query class
3. only pay semantic cost when exact and graph signals are insufficient
4. preserve explainability and provenance
5. update reinforcement or decay after outcome feedback

### Failure risks
- candidate explosion
- stale data bias
- low-confidence amplification
- cross-session interference

### Instrumentation
- p50/p95/p99 latency
- candidate counts before and after pruning
- success contribution estimate
- cache hit ratio
- repairability score

### Pseudocode
```text
fn pattern_508(ctx):
    budget = ctx.latency_budget
    exact = cheap_exact(ctx)
    graph = bounded_graph(ctx, exact)
    sem = maybe_semantic(ctx, budget, graph)
    merged = dedup_merge(exact, graph, sem)
    ranked = rank_with_policy(ctx, merged)
    return package(ctx, ranked[:ctx.limit])
```
## 509. Retention Enforcement pattern 29

### Objective
Optimize retention enforcement under bounded latency and bounded noise.

### Preconditions
- namespace is validated
- policy context is loaded
- latency budget is available
- tier capabilities are known

### Heuristic
1. start with the cheapest exact signals
2. cap expansion by query class
3. only pay semantic cost when exact and graph signals are insufficient
4. preserve explainability and provenance
5. update reinforcement or decay after outcome feedback

### Failure risks
- candidate explosion
- stale data bias
- low-confidence amplification
- cross-session interference

### Instrumentation
- p50/p95/p99 latency
- candidate counts before and after pruning
- success contribution estimate
- cache hit ratio
- repairability score

### Pseudocode
```text
fn pattern_509(ctx):
    budget = ctx.latency_budget
    exact = cheap_exact(ctx)
    graph = bounded_graph(ctx, exact)
    sem = maybe_semantic(ctx, budget, graph)
    merged = dedup_merge(exact, graph, sem)
    ranked = rank_with_policy(ctx, merged)
    return package(ctx, ranked[:ctx.limit])
```
## 510. Retention Enforcement pattern 30

### Objective
Optimize retention enforcement under bounded latency and bounded noise.

### Preconditions
- namespace is validated
- policy context is loaded
- latency budget is available
- tier capabilities are known

### Heuristic
1. start with the cheapest exact signals
2. cap expansion by query class
3. only pay semantic cost when exact and graph signals are insufficient
4. preserve explainability and provenance
5. update reinforcement or decay after outcome feedback

### Failure risks
- candidate explosion
- stale data bias
- low-confidence amplification
- cross-session interference

### Instrumentation
- p50/p95/p99 latency
- candidate counts before and after pruning
- success contribution estimate
- cache hit ratio
- repairability score

### Pseudocode
```text
fn pattern_510(ctx):
    budget = ctx.latency_budget
    exact = cheap_exact(ctx)
    graph = bounded_graph(ctx, exact)
    sem = maybe_semantic(ctx, budget, graph)
    merged = dedup_merge(exact, graph, sem)
    ranked = rank_with_policy(ctx, merged)
    return package(ctx, ranked[:ctx.limit])
```
## 511. Conflict Resolution pattern 1

### Objective
Optimize conflict resolution under bounded latency and bounded noise.

### Preconditions
- namespace is validated
- policy context is loaded
- latency budget is available
- tier capabilities are known

### Heuristic
1. start with the cheapest exact signals
2. cap expansion by query class
3. only pay semantic cost when exact and graph signals are insufficient
4. preserve explainability and provenance
5. update reinforcement or decay after outcome feedback

### Failure risks
- candidate explosion
- stale data bias
- low-confidence amplification
- cross-session interference

### Instrumentation
- p50/p95/p99 latency
- candidate counts before and after pruning
- success contribution estimate
- cache hit ratio
- repairability score

### Pseudocode
```text
fn pattern_511(ctx):
    budget = ctx.latency_budget
    exact = cheap_exact(ctx)
    graph = bounded_graph(ctx, exact)
    sem = maybe_semantic(ctx, budget, graph)
    merged = dedup_merge(exact, graph, sem)
    ranked = rank_with_policy(ctx, merged)
    return package(ctx, ranked[:ctx.limit])
```
## 512. Conflict Resolution pattern 2

### Objective
Optimize conflict resolution under bounded latency and bounded noise.

### Preconditions
- namespace is validated
- policy context is loaded
- latency budget is available
- tier capabilities are known

### Heuristic
1. start with the cheapest exact signals
2. cap expansion by query class
3. only pay semantic cost when exact and graph signals are insufficient
4. preserve explainability and provenance
5. update reinforcement or decay after outcome feedback

### Failure risks
- candidate explosion
- stale data bias
- low-confidence amplification
- cross-session interference

### Instrumentation
- p50/p95/p99 latency
- candidate counts before and after pruning
- success contribution estimate
- cache hit ratio
- repairability score

### Pseudocode
```text
fn pattern_512(ctx):
    budget = ctx.latency_budget
    exact = cheap_exact(ctx)
    graph = bounded_graph(ctx, exact)
    sem = maybe_semantic(ctx, budget, graph)
    merged = dedup_merge(exact, graph, sem)
    ranked = rank_with_policy(ctx, merged)
    return package(ctx, ranked[:ctx.limit])
```
## 513. Conflict Resolution pattern 3

### Objective
Optimize conflict resolution under bounded latency and bounded noise.

### Preconditions
- namespace is validated
- policy context is loaded
- latency budget is available
- tier capabilities are known

### Heuristic
1. start with the cheapest exact signals
2. cap expansion by query class
3. only pay semantic cost when exact and graph signals are insufficient
4. preserve explainability and provenance
5. update reinforcement or decay after outcome feedback

### Failure risks
- candidate explosion
- stale data bias
- low-confidence amplification
- cross-session interference

### Instrumentation
- p50/p95/p99 latency
- candidate counts before and after pruning
- success contribution estimate
- cache hit ratio
- repairability score

### Pseudocode
```text
fn pattern_513(ctx):
    budget = ctx.latency_budget
    exact = cheap_exact(ctx)
    graph = bounded_graph(ctx, exact)
    sem = maybe_semantic(ctx, budget, graph)
    merged = dedup_merge(exact, graph, sem)
    ranked = rank_with_policy(ctx, merged)
    return package(ctx, ranked[:ctx.limit])
```
## 514. Conflict Resolution pattern 4

### Objective
Optimize conflict resolution under bounded latency and bounded noise.

### Preconditions
- namespace is validated
- policy context is loaded
- latency budget is available
- tier capabilities are known

### Heuristic
1. start with the cheapest exact signals
2. cap expansion by query class
3. only pay semantic cost when exact and graph signals are insufficient
4. preserve explainability and provenance
5. update reinforcement or decay after outcome feedback

### Failure risks
- candidate explosion
- stale data bias
- low-confidence amplification
- cross-session interference

### Instrumentation
- p50/p95/p99 latency
- candidate counts before and after pruning
- success contribution estimate
- cache hit ratio
- repairability score

### Pseudocode
```text
fn pattern_514(ctx):
    budget = ctx.latency_budget
    exact = cheap_exact(ctx)
    graph = bounded_graph(ctx, exact)
    sem = maybe_semantic(ctx, budget, graph)
    merged = dedup_merge(exact, graph, sem)
    ranked = rank_with_policy(ctx, merged)
    return package(ctx, ranked[:ctx.limit])
```
## 515. Conflict Resolution pattern 5

### Objective
Optimize conflict resolution under bounded latency and bounded noise.

### Preconditions
- namespace is validated
- policy context is loaded
- latency budget is available
- tier capabilities are known

### Heuristic
1. start with the cheapest exact signals
2. cap expansion by query class
3. only pay semantic cost when exact and graph signals are insufficient
4. preserve explainability and provenance
5. update reinforcement or decay after outcome feedback

### Failure risks
- candidate explosion
- stale data bias
- low-confidence amplification
- cross-session interference

### Instrumentation
- p50/p95/p99 latency
- candidate counts before and after pruning
- success contribution estimate
- cache hit ratio
- repairability score

### Pseudocode
```text
fn pattern_515(ctx):
    budget = ctx.latency_budget
    exact = cheap_exact(ctx)
    graph = bounded_graph(ctx, exact)
    sem = maybe_semantic(ctx, budget, graph)
    merged = dedup_merge(exact, graph, sem)
    ranked = rank_with_policy(ctx, merged)
    return package(ctx, ranked[:ctx.limit])
```
## 516. Conflict Resolution pattern 6

### Objective
Optimize conflict resolution under bounded latency and bounded noise.

### Preconditions
- namespace is validated
- policy context is loaded
- latency budget is available
- tier capabilities are known

### Heuristic
1. start with the cheapest exact signals
2. cap expansion by query class
3. only pay semantic cost when exact and graph signals are insufficient
4. preserve explainability and provenance
5. update reinforcement or decay after outcome feedback

### Failure risks
- candidate explosion
- stale data bias
- low-confidence amplification
- cross-session interference

### Instrumentation
- p50/p95/p99 latency
- candidate counts before and after pruning
- success contribution estimate
- cache hit ratio
- repairability score

### Pseudocode
```text
fn pattern_516(ctx):
    budget = ctx.latency_budget
    exact = cheap_exact(ctx)
    graph = bounded_graph(ctx, exact)
    sem = maybe_semantic(ctx, budget, graph)
    merged = dedup_merge(exact, graph, sem)
    ranked = rank_with_policy(ctx, merged)
    return package(ctx, ranked[:ctx.limit])
```
## 517. Conflict Resolution pattern 7

### Objective
Optimize conflict resolution under bounded latency and bounded noise.

### Preconditions
- namespace is validated
- policy context is loaded
- latency budget is available
- tier capabilities are known

### Heuristic
1. start with the cheapest exact signals
2. cap expansion by query class
3. only pay semantic cost when exact and graph signals are insufficient
4. preserve explainability and provenance
5. update reinforcement or decay after outcome feedback

### Failure risks
- candidate explosion
- stale data bias
- low-confidence amplification
- cross-session interference

### Instrumentation
- p50/p95/p99 latency
- candidate counts before and after pruning
- success contribution estimate
- cache hit ratio
- repairability score

### Pseudocode
```text
fn pattern_517(ctx):
    budget = ctx.latency_budget
    exact = cheap_exact(ctx)
    graph = bounded_graph(ctx, exact)
    sem = maybe_semantic(ctx, budget, graph)
    merged = dedup_merge(exact, graph, sem)
    ranked = rank_with_policy(ctx, merged)
    return package(ctx, ranked[:ctx.limit])
```
## 518. Conflict Resolution pattern 8

### Objective
Optimize conflict resolution under bounded latency and bounded noise.

### Preconditions
- namespace is validated
- policy context is loaded
- latency budget is available
- tier capabilities are known

### Heuristic
1. start with the cheapest exact signals
2. cap expansion by query class
3. only pay semantic cost when exact and graph signals are insufficient
4. preserve explainability and provenance
5. update reinforcement or decay after outcome feedback

### Failure risks
- candidate explosion
- stale data bias
- low-confidence amplification
- cross-session interference

### Instrumentation
- p50/p95/p99 latency
- candidate counts before and after pruning
- success contribution estimate
- cache hit ratio
- repairability score

### Pseudocode
```text
fn pattern_518(ctx):
    budget = ctx.latency_budget
    exact = cheap_exact(ctx)
    graph = bounded_graph(ctx, exact)
    sem = maybe_semantic(ctx, budget, graph)
    merged = dedup_merge(exact, graph, sem)
    ranked = rank_with_policy(ctx, merged)
    return package(ctx, ranked[:ctx.limit])
```
## 519. Conflict Resolution pattern 9

### Objective
Optimize conflict resolution under bounded latency and bounded noise.

### Preconditions
- namespace is validated
- policy context is loaded
- latency budget is available
- tier capabilities are known

### Heuristic
1. start with the cheapest exact signals
2. cap expansion by query class
3. only pay semantic cost when exact and graph signals are insufficient
4. preserve explainability and provenance
5. update reinforcement or decay after outcome feedback

### Failure risks
- candidate explosion
- stale data bias
- low-confidence amplification
- cross-session interference

### Instrumentation
- p50/p95/p99 latency
- candidate counts before and after pruning
- success contribution estimate
- cache hit ratio
- repairability score

### Pseudocode
```text
fn pattern_519(ctx):
    budget = ctx.latency_budget
    exact = cheap_exact(ctx)
    graph = bounded_graph(ctx, exact)
    sem = maybe_semantic(ctx, budget, graph)
    merged = dedup_merge(exact, graph, sem)
    ranked = rank_with_policy(ctx, merged)
    return package(ctx, ranked[:ctx.limit])
```
## 520. Conflict Resolution pattern 10

### Objective
Optimize conflict resolution under bounded latency and bounded noise.

### Preconditions
- namespace is validated
- policy context is loaded
- latency budget is available
- tier capabilities are known

### Heuristic
1. start with the cheapest exact signals
2. cap expansion by query class
3. only pay semantic cost when exact and graph signals are insufficient
4. preserve explainability and provenance
5. update reinforcement or decay after outcome feedback

### Failure risks
- candidate explosion
- stale data bias
- low-confidence amplification
- cross-session interference

### Instrumentation
- p50/p95/p99 latency
- candidate counts before and after pruning
- success contribution estimate
- cache hit ratio
- repairability score

### Pseudocode
```text
fn pattern_520(ctx):
    budget = ctx.latency_budget
    exact = cheap_exact(ctx)
    graph = bounded_graph(ctx, exact)
    sem = maybe_semantic(ctx, budget, graph)
    merged = dedup_merge(exact, graph, sem)
    ranked = rank_with_policy(ctx, merged)
    return package(ctx, ranked[:ctx.limit])
```
## 521. Conflict Resolution pattern 11

### Objective
Optimize conflict resolution under bounded latency and bounded noise.

### Preconditions
- namespace is validated
- policy context is loaded
- latency budget is available
- tier capabilities are known

### Heuristic
1. start with the cheapest exact signals
2. cap expansion by query class
3. only pay semantic cost when exact and graph signals are insufficient
4. preserve explainability and provenance
5. update reinforcement or decay after outcome feedback

### Failure risks
- candidate explosion
- stale data bias
- low-confidence amplification
- cross-session interference

### Instrumentation
- p50/p95/p99 latency
- candidate counts before and after pruning
- success contribution estimate
- cache hit ratio
- repairability score

### Pseudocode
```text
fn pattern_521(ctx):
    budget = ctx.latency_budget
    exact = cheap_exact(ctx)
    graph = bounded_graph(ctx, exact)
    sem = maybe_semantic(ctx, budget, graph)
    merged = dedup_merge(exact, graph, sem)
    ranked = rank_with_policy(ctx, merged)
    return package(ctx, ranked[:ctx.limit])
```
## 522. Conflict Resolution pattern 12

### Objective
Optimize conflict resolution under bounded latency and bounded noise.

### Preconditions
- namespace is validated
- policy context is loaded
- latency budget is available
- tier capabilities are known

### Heuristic
1. start with the cheapest exact signals
2. cap expansion by query class
3. only pay semantic cost when exact and graph signals are insufficient
4. preserve explainability and provenance
5. update reinforcement or decay after outcome feedback

### Failure risks
- candidate explosion
- stale data bias
- low-confidence amplification
- cross-session interference

### Instrumentation
- p50/p95/p99 latency
- candidate counts before and after pruning
- success contribution estimate
- cache hit ratio
- repairability score

### Pseudocode
```text
fn pattern_522(ctx):
    budget = ctx.latency_budget
    exact = cheap_exact(ctx)
    graph = bounded_graph(ctx, exact)
    sem = maybe_semantic(ctx, budget, graph)
    merged = dedup_merge(exact, graph, sem)
    ranked = rank_with_policy(ctx, merged)
    return package(ctx, ranked[:ctx.limit])
```
## 523. Conflict Resolution pattern 13

### Objective
Optimize conflict resolution under bounded latency and bounded noise.

### Preconditions
- namespace is validated
- policy context is loaded
- latency budget is available
- tier capabilities are known

### Heuristic
1. start with the cheapest exact signals
2. cap expansion by query class
3. only pay semantic cost when exact and graph signals are insufficient
4. preserve explainability and provenance
5. update reinforcement or decay after outcome feedback

### Failure risks
- candidate explosion
- stale data bias
- low-confidence amplification
- cross-session interference

### Instrumentation
- p50/p95/p99 latency
- candidate counts before and after pruning
- success contribution estimate
- cache hit ratio
- repairability score

### Pseudocode
```text
fn pattern_523(ctx):
    budget = ctx.latency_budget
    exact = cheap_exact(ctx)
    graph = bounded_graph(ctx, exact)
    sem = maybe_semantic(ctx, budget, graph)
    merged = dedup_merge(exact, graph, sem)
    ranked = rank_with_policy(ctx, merged)
    return package(ctx, ranked[:ctx.limit])
```
## 524. Conflict Resolution pattern 14

### Objective
Optimize conflict resolution under bounded latency and bounded noise.

### Preconditions
- namespace is validated
- policy context is loaded
- latency budget is available
- tier capabilities are known

### Heuristic
1. start with the cheapest exact signals
2. cap expansion by query class
3. only pay semantic cost when exact and graph signals are insufficient
4. preserve explainability and provenance
5. update reinforcement or decay after outcome feedback

### Failure risks
- candidate explosion
- stale data bias
- low-confidence amplification
- cross-session interference

### Instrumentation
- p50/p95/p99 latency
- candidate counts before and after pruning
- success contribution estimate
- cache hit ratio
- repairability score

### Pseudocode
```text
fn pattern_524(ctx):
    budget = ctx.latency_budget
    exact = cheap_exact(ctx)
    graph = bounded_graph(ctx, exact)
    sem = maybe_semantic(ctx, budget, graph)
    merged = dedup_merge(exact, graph, sem)
    ranked = rank_with_policy(ctx, merged)
    return package(ctx, ranked[:ctx.limit])
```
## 525. Conflict Resolution pattern 15

### Objective
Optimize conflict resolution under bounded latency and bounded noise.

### Preconditions
- namespace is validated
- policy context is loaded
- latency budget is available
- tier capabilities are known

### Heuristic
1. start with the cheapest exact signals
2. cap expansion by query class
3. only pay semantic cost when exact and graph signals are insufficient
4. preserve explainability and provenance
5. update reinforcement or decay after outcome feedback

### Failure risks
- candidate explosion
- stale data bias
- low-confidence amplification
- cross-session interference

### Instrumentation
- p50/p95/p99 latency
- candidate counts before and after pruning
- success contribution estimate
- cache hit ratio
- repairability score

### Pseudocode
```text
fn pattern_525(ctx):
    budget = ctx.latency_budget
    exact = cheap_exact(ctx)
    graph = bounded_graph(ctx, exact)
    sem = maybe_semantic(ctx, budget, graph)
    merged = dedup_merge(exact, graph, sem)
    ranked = rank_with_policy(ctx, merged)
    return package(ctx, ranked[:ctx.limit])
```
## 526. Conflict Resolution pattern 16

### Objective
Optimize conflict resolution under bounded latency and bounded noise.

### Preconditions
- namespace is validated
- policy context is loaded
- latency budget is available
- tier capabilities are known

### Heuristic
1. start with the cheapest exact signals
2. cap expansion by query class
3. only pay semantic cost when exact and graph signals are insufficient
4. preserve explainability and provenance
5. update reinforcement or decay after outcome feedback

### Failure risks
- candidate explosion
- stale data bias
- low-confidence amplification
- cross-session interference

### Instrumentation
- p50/p95/p99 latency
- candidate counts before and after pruning
- success contribution estimate
- cache hit ratio
- repairability score

### Pseudocode
```text
fn pattern_526(ctx):
    budget = ctx.latency_budget
    exact = cheap_exact(ctx)
    graph = bounded_graph(ctx, exact)
    sem = maybe_semantic(ctx, budget, graph)
    merged = dedup_merge(exact, graph, sem)
    ranked = rank_with_policy(ctx, merged)
    return package(ctx, ranked[:ctx.limit])
```
## 527. Conflict Resolution pattern 17

### Objective
Optimize conflict resolution under bounded latency and bounded noise.

### Preconditions
- namespace is validated
- policy context is loaded
- latency budget is available
- tier capabilities are known

### Heuristic
1. start with the cheapest exact signals
2. cap expansion by query class
3. only pay semantic cost when exact and graph signals are insufficient
4. preserve explainability and provenance
5. update reinforcement or decay after outcome feedback

### Failure risks
- candidate explosion
- stale data bias
- low-confidence amplification
- cross-session interference

### Instrumentation
- p50/p95/p99 latency
- candidate counts before and after pruning
- success contribution estimate
- cache hit ratio
- repairability score

### Pseudocode
```text
fn pattern_527(ctx):
    budget = ctx.latency_budget
    exact = cheap_exact(ctx)
    graph = bounded_graph(ctx, exact)
    sem = maybe_semantic(ctx, budget, graph)
    merged = dedup_merge(exact, graph, sem)
    ranked = rank_with_policy(ctx, merged)
    return package(ctx, ranked[:ctx.limit])
```
## 528. Conflict Resolution pattern 18

### Objective
Optimize conflict resolution under bounded latency and bounded noise.

### Preconditions
- namespace is validated
- policy context is loaded
- latency budget is available
- tier capabilities are known

### Heuristic
1. start with the cheapest exact signals
2. cap expansion by query class
3. only pay semantic cost when exact and graph signals are insufficient
4. preserve explainability and provenance
5. update reinforcement or decay after outcome feedback

### Failure risks
- candidate explosion
- stale data bias
- low-confidence amplification
- cross-session interference

### Instrumentation
- p50/p95/p99 latency
- candidate counts before and after pruning
- success contribution estimate
- cache hit ratio
- repairability score

### Pseudocode
```text
fn pattern_528(ctx):
    budget = ctx.latency_budget
    exact = cheap_exact(ctx)
    graph = bounded_graph(ctx, exact)
    sem = maybe_semantic(ctx, budget, graph)
    merged = dedup_merge(exact, graph, sem)
    ranked = rank_with_policy(ctx, merged)
    return package(ctx, ranked[:ctx.limit])
```
## 529. Conflict Resolution pattern 19

### Objective
Optimize conflict resolution under bounded latency and bounded noise.

### Preconditions
- namespace is validated
- policy context is loaded
- latency budget is available
- tier capabilities are known

### Heuristic
1. start with the cheapest exact signals
2. cap expansion by query class
3. only pay semantic cost when exact and graph signals are insufficient
4. preserve explainability and provenance
5. update reinforcement or decay after outcome feedback

### Failure risks
- candidate explosion
- stale data bias
- low-confidence amplification
- cross-session interference

### Instrumentation
- p50/p95/p99 latency
- candidate counts before and after pruning
- success contribution estimate
- cache hit ratio
- repairability score

### Pseudocode
```text
fn pattern_529(ctx):
    budget = ctx.latency_budget
    exact = cheap_exact(ctx)
    graph = bounded_graph(ctx, exact)
    sem = maybe_semantic(ctx, budget, graph)
    merged = dedup_merge(exact, graph, sem)
    ranked = rank_with_policy(ctx, merged)
    return package(ctx, ranked[:ctx.limit])
```
## 530. Conflict Resolution pattern 20

### Objective
Optimize conflict resolution under bounded latency and bounded noise.

### Preconditions
- namespace is validated
- policy context is loaded
- latency budget is available
- tier capabilities are known

### Heuristic
1. start with the cheapest exact signals
2. cap expansion by query class
3. only pay semantic cost when exact and graph signals are insufficient
4. preserve explainability and provenance
5. update reinforcement or decay after outcome feedback

### Failure risks
- candidate explosion
- stale data bias
- low-confidence amplification
- cross-session interference

### Instrumentation
- p50/p95/p99 latency
- candidate counts before and after pruning
- success contribution estimate
- cache hit ratio
- repairability score

### Pseudocode
```text
fn pattern_530(ctx):
    budget = ctx.latency_budget
    exact = cheap_exact(ctx)
    graph = bounded_graph(ctx, exact)
    sem = maybe_semantic(ctx, budget, graph)
    merged = dedup_merge(exact, graph, sem)
    ranked = rank_with_policy(ctx, merged)
    return package(ctx, ranked[:ctx.limit])
```
## 531. Conflict Resolution pattern 21

### Objective
Optimize conflict resolution under bounded latency and bounded noise.

### Preconditions
- namespace is validated
- policy context is loaded
- latency budget is available
- tier capabilities are known

### Heuristic
1. start with the cheapest exact signals
2. cap expansion by query class
3. only pay semantic cost when exact and graph signals are insufficient
4. preserve explainability and provenance
5. update reinforcement or decay after outcome feedback

### Failure risks
- candidate explosion
- stale data bias
- low-confidence amplification
- cross-session interference

### Instrumentation
- p50/p95/p99 latency
- candidate counts before and after pruning
- success contribution estimate
- cache hit ratio
- repairability score

### Pseudocode
```text
fn pattern_531(ctx):
    budget = ctx.latency_budget
    exact = cheap_exact(ctx)
    graph = bounded_graph(ctx, exact)
    sem = maybe_semantic(ctx, budget, graph)
    merged = dedup_merge(exact, graph, sem)
    ranked = rank_with_policy(ctx, merged)
    return package(ctx, ranked[:ctx.limit])
```
## 532. Conflict Resolution pattern 22

### Objective
Optimize conflict resolution under bounded latency and bounded noise.

### Preconditions
- namespace is validated
- policy context is loaded
- latency budget is available
- tier capabilities are known

### Heuristic
1. start with the cheapest exact signals
2. cap expansion by query class
3. only pay semantic cost when exact and graph signals are insufficient
4. preserve explainability and provenance
5. update reinforcement or decay after outcome feedback

### Failure risks
- candidate explosion
- stale data bias
- low-confidence amplification
- cross-session interference

### Instrumentation
- p50/p95/p99 latency
- candidate counts before and after pruning
- success contribution estimate
- cache hit ratio
- repairability score

### Pseudocode
```text
fn pattern_532(ctx):
    budget = ctx.latency_budget
    exact = cheap_exact(ctx)
    graph = bounded_graph(ctx, exact)
    sem = maybe_semantic(ctx, budget, graph)
    merged = dedup_merge(exact, graph, sem)
    ranked = rank_with_policy(ctx, merged)
    return package(ctx, ranked[:ctx.limit])
```
## 533. Conflict Resolution pattern 23

### Objective
Optimize conflict resolution under bounded latency and bounded noise.

### Preconditions
- namespace is validated
- policy context is loaded
- latency budget is available
- tier capabilities are known

### Heuristic
1. start with the cheapest exact signals
2. cap expansion by query class
3. only pay semantic cost when exact and graph signals are insufficient
4. preserve explainability and provenance
5. update reinforcement or decay after outcome feedback

### Failure risks
- candidate explosion
- stale data bias
- low-confidence amplification
- cross-session interference

### Instrumentation
- p50/p95/p99 latency
- candidate counts before and after pruning
- success contribution estimate
- cache hit ratio
- repairability score

### Pseudocode
```text
fn pattern_533(ctx):
    budget = ctx.latency_budget
    exact = cheap_exact(ctx)
    graph = bounded_graph(ctx, exact)
    sem = maybe_semantic(ctx, budget, graph)
    merged = dedup_merge(exact, graph, sem)
    ranked = rank_with_policy(ctx, merged)
    return package(ctx, ranked[:ctx.limit])
```
## 534. Conflict Resolution pattern 24

### Objective
Optimize conflict resolution under bounded latency and bounded noise.

### Preconditions
- namespace is validated
- policy context is loaded
- latency budget is available
- tier capabilities are known

### Heuristic
1. start with the cheapest exact signals
2. cap expansion by query class
3. only pay semantic cost when exact and graph signals are insufficient
4. preserve explainability and provenance
5. update reinforcement or decay after outcome feedback

### Failure risks
- candidate explosion
- stale data bias
- low-confidence amplification
- cross-session interference

### Instrumentation
- p50/p95/p99 latency
- candidate counts before and after pruning
- success contribution estimate
- cache hit ratio
- repairability score

### Pseudocode
```text
fn pattern_534(ctx):
    budget = ctx.latency_budget
    exact = cheap_exact(ctx)
    graph = bounded_graph(ctx, exact)
    sem = maybe_semantic(ctx, budget, graph)
    merged = dedup_merge(exact, graph, sem)
    ranked = rank_with_policy(ctx, merged)
    return package(ctx, ranked[:ctx.limit])
```
## 535. Conflict Resolution pattern 25

### Objective
Optimize conflict resolution under bounded latency and bounded noise.

### Preconditions
- namespace is validated
- policy context is loaded
- latency budget is available
- tier capabilities are known

### Heuristic
1. start with the cheapest exact signals
2. cap expansion by query class
3. only pay semantic cost when exact and graph signals are insufficient
4. preserve explainability and provenance
5. update reinforcement or decay after outcome feedback

### Failure risks
- candidate explosion
- stale data bias
- low-confidence amplification
- cross-session interference

### Instrumentation
- p50/p95/p99 latency
- candidate counts before and after pruning
- success contribution estimate
- cache hit ratio
- repairability score

### Pseudocode
```text
fn pattern_535(ctx):
    budget = ctx.latency_budget
    exact = cheap_exact(ctx)
    graph = bounded_graph(ctx, exact)
    sem = maybe_semantic(ctx, budget, graph)
    merged = dedup_merge(exact, graph, sem)
    ranked = rank_with_policy(ctx, merged)
    return package(ctx, ranked[:ctx.limit])
```
## 536. Conflict Resolution pattern 26

### Objective
Optimize conflict resolution under bounded latency and bounded noise.

### Preconditions
- namespace is validated
- policy context is loaded
- latency budget is available
- tier capabilities are known

### Heuristic
1. start with the cheapest exact signals
2. cap expansion by query class
3. only pay semantic cost when exact and graph signals are insufficient
4. preserve explainability and provenance
5. update reinforcement or decay after outcome feedback

### Failure risks
- candidate explosion
- stale data bias
- low-confidence amplification
- cross-session interference

### Instrumentation
- p50/p95/p99 latency
- candidate counts before and after pruning
- success contribution estimate
- cache hit ratio
- repairability score

### Pseudocode
```text
fn pattern_536(ctx):
    budget = ctx.latency_budget
    exact = cheap_exact(ctx)
    graph = bounded_graph(ctx, exact)
    sem = maybe_semantic(ctx, budget, graph)
    merged = dedup_merge(exact, graph, sem)
    ranked = rank_with_policy(ctx, merged)
    return package(ctx, ranked[:ctx.limit])
```
## 537. Conflict Resolution pattern 27

### Objective
Optimize conflict resolution under bounded latency and bounded noise.

### Preconditions
- namespace is validated
- policy context is loaded
- latency budget is available
- tier capabilities are known

### Heuristic
1. start with the cheapest exact signals
2. cap expansion by query class
3. only pay semantic cost when exact and graph signals are insufficient
4. preserve explainability and provenance
5. update reinforcement or decay after outcome feedback

### Failure risks
- candidate explosion
- stale data bias
- low-confidence amplification
- cross-session interference

### Instrumentation
- p50/p95/p99 latency
- candidate counts before and after pruning
- success contribution estimate
- cache hit ratio
- repairability score

### Pseudocode
```text
fn pattern_537(ctx):
    budget = ctx.latency_budget
    exact = cheap_exact(ctx)
    graph = bounded_graph(ctx, exact)
    sem = maybe_semantic(ctx, budget, graph)
    merged = dedup_merge(exact, graph, sem)
    ranked = rank_with_policy(ctx, merged)
    return package(ctx, ranked[:ctx.limit])
```
## 538. Conflict Resolution pattern 28

### Objective
Optimize conflict resolution under bounded latency and bounded noise.

### Preconditions
- namespace is validated
- policy context is loaded
- latency budget is available
- tier capabilities are known

### Heuristic
1. start with the cheapest exact signals
2. cap expansion by query class
3. only pay semantic cost when exact and graph signals are insufficient
4. preserve explainability and provenance
5. update reinforcement or decay after outcome feedback

### Failure risks
- candidate explosion
- stale data bias
- low-confidence amplification
- cross-session interference

### Instrumentation
- p50/p95/p99 latency
- candidate counts before and after pruning
- success contribution estimate
- cache hit ratio
- repairability score

### Pseudocode
```text
fn pattern_538(ctx):
    budget = ctx.latency_budget
    exact = cheap_exact(ctx)
    graph = bounded_graph(ctx, exact)
    sem = maybe_semantic(ctx, budget, graph)
    merged = dedup_merge(exact, graph, sem)
    ranked = rank_with_policy(ctx, merged)
    return package(ctx, ranked[:ctx.limit])
```
## 539. Conflict Resolution pattern 29

### Objective
Optimize conflict resolution under bounded latency and bounded noise.

### Preconditions
- namespace is validated
- policy context is loaded
- latency budget is available
- tier capabilities are known

### Heuristic
1. start with the cheapest exact signals
2. cap expansion by query class
3. only pay semantic cost when exact and graph signals are insufficient
4. preserve explainability and provenance
5. update reinforcement or decay after outcome feedback

### Failure risks
- candidate explosion
- stale data bias
- low-confidence amplification
- cross-session interference

### Instrumentation
- p50/p95/p99 latency
- candidate counts before and after pruning
- success contribution estimate
- cache hit ratio
- repairability score

### Pseudocode
```text
fn pattern_539(ctx):
    budget = ctx.latency_budget
    exact = cheap_exact(ctx)
    graph = bounded_graph(ctx, exact)
    sem = maybe_semantic(ctx, budget, graph)
    merged = dedup_merge(exact, graph, sem)
    ranked = rank_with_policy(ctx, merged)
    return package(ctx, ranked[:ctx.limit])
```
## 540. Conflict Resolution pattern 30

### Objective
Optimize conflict resolution under bounded latency and bounded noise.

### Preconditions
- namespace is validated
- policy context is loaded
- latency budget is available
- tier capabilities are known

### Heuristic
1. start with the cheapest exact signals
2. cap expansion by query class
3. only pay semantic cost when exact and graph signals are insufficient
4. preserve explainability and provenance
5. update reinforcement or decay after outcome feedback

### Failure risks
- candidate explosion
- stale data bias
- low-confidence amplification
- cross-session interference

### Instrumentation
- p50/p95/p99 latency
- candidate counts before and after pruning
- success contribution estimate
- cache hit ratio
- repairability score

### Pseudocode
```text
fn pattern_540(ctx):
    budget = ctx.latency_budget
    exact = cheap_exact(ctx)
    graph = bounded_graph(ctx, exact)
    sem = maybe_semantic(ctx, budget, graph)
    merged = dedup_merge(exact, graph, sem)
    ranked = rank_with_policy(ctx, merged)
    return package(ctx, ranked[:ctx.limit])
```
## 541. Lineage Reconstruction pattern 1

### Objective
Optimize lineage reconstruction under bounded latency and bounded noise.

### Preconditions
- namespace is validated
- policy context is loaded
- latency budget is available
- tier capabilities are known

### Heuristic
1. start with the cheapest exact signals
2. cap expansion by query class
3. only pay semantic cost when exact and graph signals are insufficient
4. preserve explainability and provenance
5. update reinforcement or decay after outcome feedback

### Failure risks
- candidate explosion
- stale data bias
- low-confidence amplification
- cross-session interference

### Instrumentation
- p50/p95/p99 latency
- candidate counts before and after pruning
- success contribution estimate
- cache hit ratio
- repairability score

### Pseudocode
```text
fn pattern_541(ctx):
    budget = ctx.latency_budget
    exact = cheap_exact(ctx)
    graph = bounded_graph(ctx, exact)
    sem = maybe_semantic(ctx, budget, graph)
    merged = dedup_merge(exact, graph, sem)
    ranked = rank_with_policy(ctx, merged)
    return package(ctx, ranked[:ctx.limit])
```
## 542. Lineage Reconstruction pattern 2

### Objective
Optimize lineage reconstruction under bounded latency and bounded noise.

### Preconditions
- namespace is validated
- policy context is loaded
- latency budget is available
- tier capabilities are known

### Heuristic
1. start with the cheapest exact signals
2. cap expansion by query class
3. only pay semantic cost when exact and graph signals are insufficient
4. preserve explainability and provenance
5. update reinforcement or decay after outcome feedback

### Failure risks
- candidate explosion
- stale data bias
- low-confidence amplification
- cross-session interference

### Instrumentation
- p50/p95/p99 latency
- candidate counts before and after pruning
- success contribution estimate
- cache hit ratio
- repairability score

### Pseudocode
```text
fn pattern_542(ctx):
    budget = ctx.latency_budget
    exact = cheap_exact(ctx)
    graph = bounded_graph(ctx, exact)
    sem = maybe_semantic(ctx, budget, graph)
    merged = dedup_merge(exact, graph, sem)
    ranked = rank_with_policy(ctx, merged)
    return package(ctx, ranked[:ctx.limit])
```
## 543. Lineage Reconstruction pattern 3

### Objective
Optimize lineage reconstruction under bounded latency and bounded noise.

### Preconditions
- namespace is validated
- policy context is loaded
- latency budget is available
- tier capabilities are known

### Heuristic
1. start with the cheapest exact signals
2. cap expansion by query class
3. only pay semantic cost when exact and graph signals are insufficient
4. preserve explainability and provenance
5. update reinforcement or decay after outcome feedback

### Failure risks
- candidate explosion
- stale data bias
- low-confidence amplification
- cross-session interference

### Instrumentation
- p50/p95/p99 latency
- candidate counts before and after pruning
- success contribution estimate
- cache hit ratio
- repairability score

### Pseudocode
```text
fn pattern_543(ctx):
    budget = ctx.latency_budget
    exact = cheap_exact(ctx)
    graph = bounded_graph(ctx, exact)
    sem = maybe_semantic(ctx, budget, graph)
    merged = dedup_merge(exact, graph, sem)
    ranked = rank_with_policy(ctx, merged)
    return package(ctx, ranked[:ctx.limit])
```
## 544. Lineage Reconstruction pattern 4

### Objective
Optimize lineage reconstruction under bounded latency and bounded noise.

### Preconditions
- namespace is validated
- policy context is loaded
- latency budget is available
- tier capabilities are known

### Heuristic
1. start with the cheapest exact signals
2. cap expansion by query class
3. only pay semantic cost when exact and graph signals are insufficient
4. preserve explainability and provenance
5. update reinforcement or decay after outcome feedback

### Failure risks
- candidate explosion
- stale data bias
- low-confidence amplification
- cross-session interference

### Instrumentation
- p50/p95/p99 latency
- candidate counts before and after pruning
- success contribution estimate
- cache hit ratio
- repairability score

### Pseudocode
```text
fn pattern_544(ctx):
    budget = ctx.latency_budget
    exact = cheap_exact(ctx)
    graph = bounded_graph(ctx, exact)
    sem = maybe_semantic(ctx, budget, graph)
    merged = dedup_merge(exact, graph, sem)
    ranked = rank_with_policy(ctx, merged)
    return package(ctx, ranked[:ctx.limit])
```
## 545. Lineage Reconstruction pattern 5

### Objective
Optimize lineage reconstruction under bounded latency and bounded noise.

### Preconditions
- namespace is validated
- policy context is loaded
- latency budget is available
- tier capabilities are known

### Heuristic
1. start with the cheapest exact signals
2. cap expansion by query class
3. only pay semantic cost when exact and graph signals are insufficient
4. preserve explainability and provenance
5. update reinforcement or decay after outcome feedback

### Failure risks
- candidate explosion
- stale data bias
- low-confidence amplification
- cross-session interference

### Instrumentation
- p50/p95/p99 latency
- candidate counts before and after pruning
- success contribution estimate
- cache hit ratio
- repairability score

### Pseudocode
```text
fn pattern_545(ctx):
    budget = ctx.latency_budget
    exact = cheap_exact(ctx)
    graph = bounded_graph(ctx, exact)
    sem = maybe_semantic(ctx, budget, graph)
    merged = dedup_merge(exact, graph, sem)
    ranked = rank_with_policy(ctx, merged)
    return package(ctx, ranked[:ctx.limit])
```
## 546. Lineage Reconstruction pattern 6

### Objective
Optimize lineage reconstruction under bounded latency and bounded noise.

### Preconditions
- namespace is validated
- policy context is loaded
- latency budget is available
- tier capabilities are known

### Heuristic
1. start with the cheapest exact signals
2. cap expansion by query class
3. only pay semantic cost when exact and graph signals are insufficient
4. preserve explainability and provenance
5. update reinforcement or decay after outcome feedback

### Failure risks
- candidate explosion
- stale data bias
- low-confidence amplification
- cross-session interference

### Instrumentation
- p50/p95/p99 latency
- candidate counts before and after pruning
- success contribution estimate
- cache hit ratio
- repairability score

### Pseudocode
```text
fn pattern_546(ctx):
    budget = ctx.latency_budget
    exact = cheap_exact(ctx)
    graph = bounded_graph(ctx, exact)
    sem = maybe_semantic(ctx, budget, graph)
    merged = dedup_merge(exact, graph, sem)
    ranked = rank_with_policy(ctx, merged)
    return package(ctx, ranked[:ctx.limit])
```
## 547. Lineage Reconstruction pattern 7

### Objective
Optimize lineage reconstruction under bounded latency and bounded noise.

### Preconditions
- namespace is validated
- policy context is loaded
- latency budget is available
- tier capabilities are known

### Heuristic
1. start with the cheapest exact signals
2. cap expansion by query class
3. only pay semantic cost when exact and graph signals are insufficient
4. preserve explainability and provenance
5. update reinforcement or decay after outcome feedback

### Failure risks
- candidate explosion
- stale data bias
- low-confidence amplification
- cross-session interference

### Instrumentation
- p50/p95/p99 latency
- candidate counts before and after pruning
- success contribution estimate
- cache hit ratio
- repairability score

### Pseudocode
```text
fn pattern_547(ctx):
    budget = ctx.latency_budget
    exact = cheap_exact(ctx)
    graph = bounded_graph(ctx, exact)
    sem = maybe_semantic(ctx, budget, graph)
    merged = dedup_merge(exact, graph, sem)
    ranked = rank_with_policy(ctx, merged)
    return package(ctx, ranked[:ctx.limit])
```
## 548. Lineage Reconstruction pattern 8

### Objective
Optimize lineage reconstruction under bounded latency and bounded noise.

### Preconditions
- namespace is validated
- policy context is loaded
- latency budget is available
- tier capabilities are known

### Heuristic
1. start with the cheapest exact signals
2. cap expansion by query class
3. only pay semantic cost when exact and graph signals are insufficient
4. preserve explainability and provenance
5. update reinforcement or decay after outcome feedback

### Failure risks
- candidate explosion
- stale data bias
- low-confidence amplification
- cross-session interference

### Instrumentation
- p50/p95/p99 latency
- candidate counts before and after pruning
- success contribution estimate
- cache hit ratio
- repairability score

### Pseudocode
```text
fn pattern_548(ctx):
    budget = ctx.latency_budget
    exact = cheap_exact(ctx)
    graph = bounded_graph(ctx, exact)
    sem = maybe_semantic(ctx, budget, graph)
    merged = dedup_merge(exact, graph, sem)
    ranked = rank_with_policy(ctx, merged)
    return package(ctx, ranked[:ctx.limit])
```
## 549. Lineage Reconstruction pattern 9

### Objective
Optimize lineage reconstruction under bounded latency and bounded noise.

### Preconditions
- namespace is validated
- policy context is loaded
- latency budget is available
- tier capabilities are known

### Heuristic
1. start with the cheapest exact signals
2. cap expansion by query class
3. only pay semantic cost when exact and graph signals are insufficient
4. preserve explainability and provenance
5. update reinforcement or decay after outcome feedback

### Failure risks
- candidate explosion
- stale data bias
- low-confidence amplification
- cross-session interference

### Instrumentation
- p50/p95/p99 latency
- candidate counts before and after pruning
- success contribution estimate
- cache hit ratio
- repairability score

### Pseudocode
```text
fn pattern_549(ctx):
    budget = ctx.latency_budget
    exact = cheap_exact(ctx)
    graph = bounded_graph(ctx, exact)
    sem = maybe_semantic(ctx, budget, graph)
    merged = dedup_merge(exact, graph, sem)
    ranked = rank_with_policy(ctx, merged)
    return package(ctx, ranked[:ctx.limit])
```
## 550. Lineage Reconstruction pattern 10

### Objective
Optimize lineage reconstruction under bounded latency and bounded noise.

### Preconditions
- namespace is validated
- policy context is loaded
- latency budget is available
- tier capabilities are known

### Heuristic
1. start with the cheapest exact signals
2. cap expansion by query class
3. only pay semantic cost when exact and graph signals are insufficient
4. preserve explainability and provenance
5. update reinforcement or decay after outcome feedback

### Failure risks
- candidate explosion
- stale data bias
- low-confidence amplification
- cross-session interference

### Instrumentation
- p50/p95/p99 latency
- candidate counts before and after pruning
- success contribution estimate
- cache hit ratio
- repairability score

### Pseudocode
```text
fn pattern_550(ctx):
    budget = ctx.latency_budget
    exact = cheap_exact(ctx)
    graph = bounded_graph(ctx, exact)
    sem = maybe_semantic(ctx, budget, graph)
    merged = dedup_merge(exact, graph, sem)
    ranked = rank_with_policy(ctx, merged)
    return package(ctx, ranked[:ctx.limit])
```
## 551. Lineage Reconstruction pattern 11

### Objective
Optimize lineage reconstruction under bounded latency and bounded noise.

### Preconditions
- namespace is validated
- policy context is loaded
- latency budget is available
- tier capabilities are known

### Heuristic
1. start with the cheapest exact signals
2. cap expansion by query class
3. only pay semantic cost when exact and graph signals are insufficient
4. preserve explainability and provenance
5. update reinforcement or decay after outcome feedback

### Failure risks
- candidate explosion
- stale data bias
- low-confidence amplification
- cross-session interference

### Instrumentation
- p50/p95/p99 latency
- candidate counts before and after pruning
- success contribution estimate
- cache hit ratio
- repairability score

### Pseudocode
```text
fn pattern_551(ctx):
    budget = ctx.latency_budget
    exact = cheap_exact(ctx)
    graph = bounded_graph(ctx, exact)
    sem = maybe_semantic(ctx, budget, graph)
    merged = dedup_merge(exact, graph, sem)
    ranked = rank_with_policy(ctx, merged)
    return package(ctx, ranked[:ctx.limit])
```
## 552. Lineage Reconstruction pattern 12

### Objective
Optimize lineage reconstruction under bounded latency and bounded noise.

### Preconditions
- namespace is validated
- policy context is loaded
- latency budget is available
- tier capabilities are known

### Heuristic
1. start with the cheapest exact signals
2. cap expansion by query class
3. only pay semantic cost when exact and graph signals are insufficient
4. preserve explainability and provenance
5. update reinforcement or decay after outcome feedback

### Failure risks
- candidate explosion
- stale data bias
- low-confidence amplification
- cross-session interference

### Instrumentation
- p50/p95/p99 latency
- candidate counts before and after pruning
- success contribution estimate
- cache hit ratio
- repairability score

### Pseudocode
```text
fn pattern_552(ctx):
    budget = ctx.latency_budget
    exact = cheap_exact(ctx)
    graph = bounded_graph(ctx, exact)
    sem = maybe_semantic(ctx, budget, graph)
    merged = dedup_merge(exact, graph, sem)
    ranked = rank_with_policy(ctx, merged)
    return package(ctx, ranked[:ctx.limit])
```
## 553. Lineage Reconstruction pattern 13

### Objective
Optimize lineage reconstruction under bounded latency and bounded noise.

### Preconditions
- namespace is validated
- policy context is loaded
- latency budget is available
- tier capabilities are known

### Heuristic
1. start with the cheapest exact signals
2. cap expansion by query class
3. only pay semantic cost when exact and graph signals are insufficient
4. preserve explainability and provenance
5. update reinforcement or decay after outcome feedback

### Failure risks
- candidate explosion
- stale data bias
- low-confidence amplification
- cross-session interference

### Instrumentation
- p50/p95/p99 latency
- candidate counts before and after pruning
- success contribution estimate
- cache hit ratio
- repairability score

### Pseudocode
```text
fn pattern_553(ctx):
    budget = ctx.latency_budget
    exact = cheap_exact(ctx)
    graph = bounded_graph(ctx, exact)
    sem = maybe_semantic(ctx, budget, graph)
    merged = dedup_merge(exact, graph, sem)
    ranked = rank_with_policy(ctx, merged)
    return package(ctx, ranked[:ctx.limit])
```
## 554. Lineage Reconstruction pattern 14

### Objective
Optimize lineage reconstruction under bounded latency and bounded noise.

### Preconditions
- namespace is validated
- policy context is loaded
- latency budget is available
- tier capabilities are known

### Heuristic
1. start with the cheapest exact signals
2. cap expansion by query class
3. only pay semantic cost when exact and graph signals are insufficient
4. preserve explainability and provenance
5. update reinforcement or decay after outcome feedback

### Failure risks
- candidate explosion
- stale data bias
- low-confidence amplification
- cross-session interference

### Instrumentation
- p50/p95/p99 latency
- candidate counts before and after pruning
- success contribution estimate
- cache hit ratio
- repairability score

### Pseudocode
```text
fn pattern_554(ctx):
    budget = ctx.latency_budget
    exact = cheap_exact(ctx)
    graph = bounded_graph(ctx, exact)
    sem = maybe_semantic(ctx, budget, graph)
    merged = dedup_merge(exact, graph, sem)
    ranked = rank_with_policy(ctx, merged)
    return package(ctx, ranked[:ctx.limit])
```
## 555. Lineage Reconstruction pattern 15

### Objective
Optimize lineage reconstruction under bounded latency and bounded noise.

### Preconditions
- namespace is validated
- policy context is loaded
- latency budget is available
- tier capabilities are known

### Heuristic
1. start with the cheapest exact signals
2. cap expansion by query class
3. only pay semantic cost when exact and graph signals are insufficient
4. preserve explainability and provenance
5. update reinforcement or decay after outcome feedback

### Failure risks
- candidate explosion
- stale data bias
- low-confidence amplification
- cross-session interference

### Instrumentation
- p50/p95/p99 latency
- candidate counts before and after pruning
- success contribution estimate
- cache hit ratio
- repairability score

### Pseudocode
```text
fn pattern_555(ctx):
    budget = ctx.latency_budget
    exact = cheap_exact(ctx)
    graph = bounded_graph(ctx, exact)
    sem = maybe_semantic(ctx, budget, graph)
    merged = dedup_merge(exact, graph, sem)
    ranked = rank_with_policy(ctx, merged)
    return package(ctx, ranked[:ctx.limit])
```
## 556. Lineage Reconstruction pattern 16

### Objective
Optimize lineage reconstruction under bounded latency and bounded noise.

### Preconditions
- namespace is validated
- policy context is loaded
- latency budget is available
- tier capabilities are known

### Heuristic
1. start with the cheapest exact signals
2. cap expansion by query class
3. only pay semantic cost when exact and graph signals are insufficient
4. preserve explainability and provenance
5. update reinforcement or decay after outcome feedback

### Failure risks
- candidate explosion
- stale data bias
- low-confidence amplification
- cross-session interference

### Instrumentation
- p50/p95/p99 latency
- candidate counts before and after pruning
- success contribution estimate
- cache hit ratio
- repairability score

### Pseudocode
```text
fn pattern_556(ctx):
    budget = ctx.latency_budget
    exact = cheap_exact(ctx)
    graph = bounded_graph(ctx, exact)
    sem = maybe_semantic(ctx, budget, graph)
    merged = dedup_merge(exact, graph, sem)
    ranked = rank_with_policy(ctx, merged)
    return package(ctx, ranked[:ctx.limit])
```
## 557. Lineage Reconstruction pattern 17

### Objective
Optimize lineage reconstruction under bounded latency and bounded noise.

### Preconditions
- namespace is validated
- policy context is loaded
- latency budget is available
- tier capabilities are known

### Heuristic
1. start with the cheapest exact signals
2. cap expansion by query class
3. only pay semantic cost when exact and graph signals are insufficient
4. preserve explainability and provenance
5. update reinforcement or decay after outcome feedback

### Failure risks
- candidate explosion
- stale data bias
- low-confidence amplification
- cross-session interference

### Instrumentation
- p50/p95/p99 latency
- candidate counts before and after pruning
- success contribution estimate
- cache hit ratio
- repairability score

### Pseudocode
```text
fn pattern_557(ctx):
    budget = ctx.latency_budget
    exact = cheap_exact(ctx)
    graph = bounded_graph(ctx, exact)
    sem = maybe_semantic(ctx, budget, graph)
    merged = dedup_merge(exact, graph, sem)
    ranked = rank_with_policy(ctx, merged)
    return package(ctx, ranked[:ctx.limit])
```
## 558. Lineage Reconstruction pattern 18

### Objective
Optimize lineage reconstruction under bounded latency and bounded noise.

### Preconditions
- namespace is validated
- policy context is loaded
- latency budget is available
- tier capabilities are known

### Heuristic
1. start with the cheapest exact signals
2. cap expansion by query class
3. only pay semantic cost when exact and graph signals are insufficient
4. preserve explainability and provenance
5. update reinforcement or decay after outcome feedback

### Failure risks
- candidate explosion
- stale data bias
- low-confidence amplification
- cross-session interference

### Instrumentation
- p50/p95/p99 latency
- candidate counts before and after pruning
- success contribution estimate
- cache hit ratio
- repairability score

### Pseudocode
```text
fn pattern_558(ctx):
    budget = ctx.latency_budget
    exact = cheap_exact(ctx)
    graph = bounded_graph(ctx, exact)
    sem = maybe_semantic(ctx, budget, graph)
    merged = dedup_merge(exact, graph, sem)
    ranked = rank_with_policy(ctx, merged)
    return package(ctx, ranked[:ctx.limit])
```
## 559. Lineage Reconstruction pattern 19

### Objective
Optimize lineage reconstruction under bounded latency and bounded noise.

### Preconditions
- namespace is validated
- policy context is loaded
- latency budget is available
- tier capabilities are known

### Heuristic
1. start with the cheapest exact signals
2. cap expansion by query class
3. only pay semantic cost when exact and graph signals are insufficient
4. preserve explainability and provenance
5. update reinforcement or decay after outcome feedback

### Failure risks
- candidate explosion
- stale data bias
- low-confidence amplification
- cross-session interference

### Instrumentation
- p50/p95/p99 latency
- candidate counts before and after pruning
- success contribution estimate
- cache hit ratio
- repairability score

### Pseudocode
```text
fn pattern_559(ctx):
    budget = ctx.latency_budget
    exact = cheap_exact(ctx)
    graph = bounded_graph(ctx, exact)
    sem = maybe_semantic(ctx, budget, graph)
    merged = dedup_merge(exact, graph, sem)
    ranked = rank_with_policy(ctx, merged)
    return package(ctx, ranked[:ctx.limit])
```
## 560. Lineage Reconstruction pattern 20

### Objective
Optimize lineage reconstruction under bounded latency and bounded noise.

### Preconditions
- namespace is validated
- policy context is loaded
- latency budget is available
- tier capabilities are known

### Heuristic
1. start with the cheapest exact signals
2. cap expansion by query class
3. only pay semantic cost when exact and graph signals are insufficient
4. preserve explainability and provenance
5. update reinforcement or decay after outcome feedback

### Failure risks
- candidate explosion
- stale data bias
- low-confidence amplification
- cross-session interference

### Instrumentation
- p50/p95/p99 latency
- candidate counts before and after pruning
- success contribution estimate
- cache hit ratio
- repairability score

### Pseudocode
```text
fn pattern_560(ctx):
    budget = ctx.latency_budget
    exact = cheap_exact(ctx)
    graph = bounded_graph(ctx, exact)
    sem = maybe_semantic(ctx, budget, graph)
    merged = dedup_merge(exact, graph, sem)
    ranked = rank_with_policy(ctx, merged)
    return package(ctx, ranked[:ctx.limit])
```
## 561. Lineage Reconstruction pattern 21

### Objective
Optimize lineage reconstruction under bounded latency and bounded noise.

### Preconditions
- namespace is validated
- policy context is loaded
- latency budget is available
- tier capabilities are known

### Heuristic
1. start with the cheapest exact signals
2. cap expansion by query class
3. only pay semantic cost when exact and graph signals are insufficient
4. preserve explainability and provenance
5. update reinforcement or decay after outcome feedback

### Failure risks
- candidate explosion
- stale data bias
- low-confidence amplification
- cross-session interference

### Instrumentation
- p50/p95/p99 latency
- candidate counts before and after pruning
- success contribution estimate
- cache hit ratio
- repairability score

### Pseudocode
```text
fn pattern_561(ctx):
    budget = ctx.latency_budget
    exact = cheap_exact(ctx)
    graph = bounded_graph(ctx, exact)
    sem = maybe_semantic(ctx, budget, graph)
    merged = dedup_merge(exact, graph, sem)
    ranked = rank_with_policy(ctx, merged)
    return package(ctx, ranked[:ctx.limit])
```
## 562. Lineage Reconstruction pattern 22

### Objective
Optimize lineage reconstruction under bounded latency and bounded noise.

### Preconditions
- namespace is validated
- policy context is loaded
- latency budget is available
- tier capabilities are known

### Heuristic
1. start with the cheapest exact signals
2. cap expansion by query class
3. only pay semantic cost when exact and graph signals are insufficient
4. preserve explainability and provenance
5. update reinforcement or decay after outcome feedback

### Failure risks
- candidate explosion
- stale data bias
- low-confidence amplification
- cross-session interference

### Instrumentation
- p50/p95/p99 latency
- candidate counts before and after pruning
- success contribution estimate
- cache hit ratio
- repairability score

### Pseudocode
```text
fn pattern_562(ctx):
    budget = ctx.latency_budget
    exact = cheap_exact(ctx)
    graph = bounded_graph(ctx, exact)
    sem = maybe_semantic(ctx, budget, graph)
    merged = dedup_merge(exact, graph, sem)
    ranked = rank_with_policy(ctx, merged)
    return package(ctx, ranked[:ctx.limit])
```
## 563. Lineage Reconstruction pattern 23

### Objective
Optimize lineage reconstruction under bounded latency and bounded noise.

### Preconditions
- namespace is validated
- policy context is loaded
- latency budget is available
- tier capabilities are known

### Heuristic
1. start with the cheapest exact signals
2. cap expansion by query class
3. only pay semantic cost when exact and graph signals are insufficient
4. preserve explainability and provenance
5. update reinforcement or decay after outcome feedback

### Failure risks
- candidate explosion
- stale data bias
- low-confidence amplification
- cross-session interference

### Instrumentation
- p50/p95/p99 latency
- candidate counts before and after pruning
- success contribution estimate
- cache hit ratio
- repairability score

### Pseudocode
```text
fn pattern_563(ctx):
    budget = ctx.latency_budget
    exact = cheap_exact(ctx)
    graph = bounded_graph(ctx, exact)
    sem = maybe_semantic(ctx, budget, graph)
    merged = dedup_merge(exact, graph, sem)
    ranked = rank_with_policy(ctx, merged)
    return package(ctx, ranked[:ctx.limit])
```
## 564. Lineage Reconstruction pattern 24

### Objective
Optimize lineage reconstruction under bounded latency and bounded noise.

### Preconditions
- namespace is validated
- policy context is loaded
- latency budget is available
- tier capabilities are known

### Heuristic
1. start with the cheapest exact signals
2. cap expansion by query class
3. only pay semantic cost when exact and graph signals are insufficient
4. preserve explainability and provenance
5. update reinforcement or decay after outcome feedback

### Failure risks
- candidate explosion
- stale data bias
- low-confidence amplification
- cross-session interference

### Instrumentation
- p50/p95/p99 latency
- candidate counts before and after pruning
- success contribution estimate
- cache hit ratio
- repairability score

### Pseudocode
```text
fn pattern_564(ctx):
    budget = ctx.latency_budget
    exact = cheap_exact(ctx)
    graph = bounded_graph(ctx, exact)
    sem = maybe_semantic(ctx, budget, graph)
    merged = dedup_merge(exact, graph, sem)
    ranked = rank_with_policy(ctx, merged)
    return package(ctx, ranked[:ctx.limit])
```
## 565. Lineage Reconstruction pattern 25

### Objective
Optimize lineage reconstruction under bounded latency and bounded noise.

### Preconditions
- namespace is validated
- policy context is loaded
- latency budget is available
- tier capabilities are known

### Heuristic
1. start with the cheapest exact signals
2. cap expansion by query class
3. only pay semantic cost when exact and graph signals are insufficient
4. preserve explainability and provenance
5. update reinforcement or decay after outcome feedback

### Failure risks
- candidate explosion
- stale data bias
- low-confidence amplification
- cross-session interference

### Instrumentation
- p50/p95/p99 latency
- candidate counts before and after pruning
- success contribution estimate
- cache hit ratio
- repairability score

### Pseudocode
```text
fn pattern_565(ctx):
    budget = ctx.latency_budget
    exact = cheap_exact(ctx)
    graph = bounded_graph(ctx, exact)
    sem = maybe_semantic(ctx, budget, graph)
    merged = dedup_merge(exact, graph, sem)
    ranked = rank_with_policy(ctx, merged)
    return package(ctx, ranked[:ctx.limit])
```
## 566. Lineage Reconstruction pattern 26

### Objective
Optimize lineage reconstruction under bounded latency and bounded noise.

### Preconditions
- namespace is validated
- policy context is loaded
- latency budget is available
- tier capabilities are known

### Heuristic
1. start with the cheapest exact signals
2. cap expansion by query class
3. only pay semantic cost when exact and graph signals are insufficient
4. preserve explainability and provenance
5. update reinforcement or decay after outcome feedback

### Failure risks
- candidate explosion
- stale data bias
- low-confidence amplification
- cross-session interference

### Instrumentation
- p50/p95/p99 latency
- candidate counts before and after pruning
- success contribution estimate
- cache hit ratio
- repairability score

### Pseudocode
```text
fn pattern_566(ctx):
    budget = ctx.latency_budget
    exact = cheap_exact(ctx)
    graph = bounded_graph(ctx, exact)
    sem = maybe_semantic(ctx, budget, graph)
    merged = dedup_merge(exact, graph, sem)
    ranked = rank_with_policy(ctx, merged)
    return package(ctx, ranked[:ctx.limit])
```
## 567. Lineage Reconstruction pattern 27

### Objective
Optimize lineage reconstruction under bounded latency and bounded noise.

### Preconditions
- namespace is validated
- policy context is loaded
- latency budget is available
- tier capabilities are known

### Heuristic
1. start with the cheapest exact signals
2. cap expansion by query class
3. only pay semantic cost when exact and graph signals are insufficient
4. preserve explainability and provenance
5. update reinforcement or decay after outcome feedback

### Failure risks
- candidate explosion
- stale data bias
- low-confidence amplification
- cross-session interference

### Instrumentation
- p50/p95/p99 latency
- candidate counts before and after pruning
- success contribution estimate
- cache hit ratio
- repairability score

### Pseudocode
```text
fn pattern_567(ctx):
    budget = ctx.latency_budget
    exact = cheap_exact(ctx)
    graph = bounded_graph(ctx, exact)
    sem = maybe_semantic(ctx, budget, graph)
    merged = dedup_merge(exact, graph, sem)
    ranked = rank_with_policy(ctx, merged)
    return package(ctx, ranked[:ctx.limit])
```
## 568. Lineage Reconstruction pattern 28

### Objective
Optimize lineage reconstruction under bounded latency and bounded noise.

### Preconditions
- namespace is validated
- policy context is loaded
- latency budget is available
- tier capabilities are known

### Heuristic
1. start with the cheapest exact signals
2. cap expansion by query class
3. only pay semantic cost when exact and graph signals are insufficient
4. preserve explainability and provenance
5. update reinforcement or decay after outcome feedback

### Failure risks
- candidate explosion
- stale data bias
- low-confidence amplification
- cross-session interference

### Instrumentation
- p50/p95/p99 latency
- candidate counts before and after pruning
- success contribution estimate
- cache hit ratio
- repairability score

### Pseudocode
```text
fn pattern_568(ctx):
    budget = ctx.latency_budget
    exact = cheap_exact(ctx)
    graph = bounded_graph(ctx, exact)
    sem = maybe_semantic(ctx, budget, graph)
    merged = dedup_merge(exact, graph, sem)
    ranked = rank_with_policy(ctx, merged)
    return package(ctx, ranked[:ctx.limit])
```
## 569. Lineage Reconstruction pattern 29

### Objective
Optimize lineage reconstruction under bounded latency and bounded noise.

### Preconditions
- namespace is validated
- policy context is loaded
- latency budget is available
- tier capabilities are known

### Heuristic
1. start with the cheapest exact signals
2. cap expansion by query class
3. only pay semantic cost when exact and graph signals are insufficient
4. preserve explainability and provenance
5. update reinforcement or decay after outcome feedback

### Failure risks
- candidate explosion
- stale data bias
- low-confidence amplification
- cross-session interference

### Instrumentation
- p50/p95/p99 latency
- candidate counts before and after pruning
- success contribution estimate
- cache hit ratio
- repairability score

### Pseudocode
```text
fn pattern_569(ctx):
    budget = ctx.latency_budget
    exact = cheap_exact(ctx)
    graph = bounded_graph(ctx, exact)
    sem = maybe_semantic(ctx, budget, graph)
    merged = dedup_merge(exact, graph, sem)
    ranked = rank_with_policy(ctx, merged)
    return package(ctx, ranked[:ctx.limit])
```
## 570. Lineage Reconstruction pattern 30

### Objective
Optimize lineage reconstruction under bounded latency and bounded noise.

### Preconditions
- namespace is validated
- policy context is loaded
- latency budget is available
- tier capabilities are known

### Heuristic
1. start with the cheapest exact signals
2. cap expansion by query class
3. only pay semantic cost when exact and graph signals are insufficient
4. preserve explainability and provenance
5. update reinforcement or decay after outcome feedback

### Failure risks
- candidate explosion
- stale data bias
- low-confidence amplification
- cross-session interference

### Instrumentation
- p50/p95/p99 latency
- candidate counts before and after pruning
- success contribution estimate
- cache hit ratio
- repairability score

### Pseudocode
```text
fn pattern_570(ctx):
    budget = ctx.latency_budget
    exact = cheap_exact(ctx)
    graph = bounded_graph(ctx, exact)
    sem = maybe_semantic(ctx, budget, graph)
    merged = dedup_merge(exact, graph, sem)
    ranked = rank_with_policy(ctx, merged)
    return package(ctx, ranked[:ctx.limit])
```
## 571. Repair Orchestration pattern 1

### Objective
Optimize repair orchestration under bounded latency and bounded noise.

### Preconditions
- namespace is validated
- policy context is loaded
- latency budget is available
- tier capabilities are known

### Heuristic
1. start with the cheapest exact signals
2. cap expansion by query class
3. only pay semantic cost when exact and graph signals are insufficient
4. preserve explainability and provenance
5. update reinforcement or decay after outcome feedback

### Failure risks
- candidate explosion
- stale data bias
- low-confidence amplification
- cross-session interference

### Instrumentation
- p50/p95/p99 latency
- candidate counts before and after pruning
- success contribution estimate
- cache hit ratio
- repairability score

### Pseudocode
```text
fn pattern_571(ctx):
    budget = ctx.latency_budget
    exact = cheap_exact(ctx)
    graph = bounded_graph(ctx, exact)
    sem = maybe_semantic(ctx, budget, graph)
    merged = dedup_merge(exact, graph, sem)
    ranked = rank_with_policy(ctx, merged)
    return package(ctx, ranked[:ctx.limit])
```
## 572. Repair Orchestration pattern 2

### Objective
Optimize repair orchestration under bounded latency and bounded noise.

### Preconditions
- namespace is validated
- policy context is loaded
- latency budget is available
- tier capabilities are known

### Heuristic
1. start with the cheapest exact signals
2. cap expansion by query class
3. only pay semantic cost when exact and graph signals are insufficient
4. preserve explainability and provenance
5. update reinforcement or decay after outcome feedback

### Failure risks
- candidate explosion
- stale data bias
- low-confidence amplification
- cross-session interference

### Instrumentation
- p50/p95/p99 latency
- candidate counts before and after pruning
- success contribution estimate
- cache hit ratio
- repairability score

### Pseudocode
```text
fn pattern_572(ctx):
    budget = ctx.latency_budget
    exact = cheap_exact(ctx)
    graph = bounded_graph(ctx, exact)
    sem = maybe_semantic(ctx, budget, graph)
    merged = dedup_merge(exact, graph, sem)
    ranked = rank_with_policy(ctx, merged)
    return package(ctx, ranked[:ctx.limit])
```
## 573. Repair Orchestration pattern 3

### Objective
Optimize repair orchestration under bounded latency and bounded noise.

### Preconditions
- namespace is validated
- policy context is loaded
- latency budget is available
- tier capabilities are known

### Heuristic
1. start with the cheapest exact signals
2. cap expansion by query class
3. only pay semantic cost when exact and graph signals are insufficient
4. preserve explainability and provenance
5. update reinforcement or decay after outcome feedback

### Failure risks
- candidate explosion
- stale data bias
- low-confidence amplification
- cross-session interference

### Instrumentation
- p50/p95/p99 latency
- candidate counts before and after pruning
- success contribution estimate
- cache hit ratio
- repairability score

### Pseudocode
```text
fn pattern_573(ctx):
    budget = ctx.latency_budget
    exact = cheap_exact(ctx)
    graph = bounded_graph(ctx, exact)
    sem = maybe_semantic(ctx, budget, graph)
    merged = dedup_merge(exact, graph, sem)
    ranked = rank_with_policy(ctx, merged)
    return package(ctx, ranked[:ctx.limit])
```
## 574. Repair Orchestration pattern 4

### Objective
Optimize repair orchestration under bounded latency and bounded noise.

### Preconditions
- namespace is validated
- policy context is loaded
- latency budget is available
- tier capabilities are known

### Heuristic
1. start with the cheapest exact signals
2. cap expansion by query class
3. only pay semantic cost when exact and graph signals are insufficient
4. preserve explainability and provenance
5. update reinforcement or decay after outcome feedback

### Failure risks
- candidate explosion
- stale data bias
- low-confidence amplification
- cross-session interference

### Instrumentation
- p50/p95/p99 latency
- candidate counts before and after pruning
- success contribution estimate
- cache hit ratio
- repairability score

### Pseudocode
```text
fn pattern_574(ctx):
    budget = ctx.latency_budget
    exact = cheap_exact(ctx)
    graph = bounded_graph(ctx, exact)
    sem = maybe_semantic(ctx, budget, graph)
    merged = dedup_merge(exact, graph, sem)
    ranked = rank_with_policy(ctx, merged)
    return package(ctx, ranked[:ctx.limit])
```
## 575. Repair Orchestration pattern 5

### Objective
Optimize repair orchestration under bounded latency and bounded noise.

### Preconditions
- namespace is validated
- policy context is loaded
- latency budget is available
- tier capabilities are known

### Heuristic
1. start with the cheapest exact signals
2. cap expansion by query class
3. only pay semantic cost when exact and graph signals are insufficient
4. preserve explainability and provenance
5. update reinforcement or decay after outcome feedback

### Failure risks
- candidate explosion
- stale data bias
- low-confidence amplification
- cross-session interference

### Instrumentation
- p50/p95/p99 latency
- candidate counts before and after pruning
- success contribution estimate
- cache hit ratio
- repairability score

### Pseudocode
```text
fn pattern_575(ctx):
    budget = ctx.latency_budget
    exact = cheap_exact(ctx)
    graph = bounded_graph(ctx, exact)
    sem = maybe_semantic(ctx, budget, graph)
    merged = dedup_merge(exact, graph, sem)
    ranked = rank_with_policy(ctx, merged)
    return package(ctx, ranked[:ctx.limit])
```
## 576. Repair Orchestration pattern 6

### Objective
Optimize repair orchestration under bounded latency and bounded noise.

### Preconditions
- namespace is validated
- policy context is loaded
- latency budget is available
- tier capabilities are known

### Heuristic
1. start with the cheapest exact signals
2. cap expansion by query class
3. only pay semantic cost when exact and graph signals are insufficient
4. preserve explainability and provenance
5. update reinforcement or decay after outcome feedback

### Failure risks
- candidate explosion
- stale data bias
- low-confidence amplification
- cross-session interference

### Instrumentation
- p50/p95/p99 latency
- candidate counts before and after pruning
- success contribution estimate
- cache hit ratio
- repairability score

### Pseudocode
```text
fn pattern_576(ctx):
    budget = ctx.latency_budget
    exact = cheap_exact(ctx)
    graph = bounded_graph(ctx, exact)
    sem = maybe_semantic(ctx, budget, graph)
    merged = dedup_merge(exact, graph, sem)
    ranked = rank_with_policy(ctx, merged)
    return package(ctx, ranked[:ctx.limit])
```
## 577. Repair Orchestration pattern 7

### Objective
Optimize repair orchestration under bounded latency and bounded noise.

### Preconditions
- namespace is validated
- policy context is loaded
- latency budget is available
- tier capabilities are known

### Heuristic
1. start with the cheapest exact signals
2. cap expansion by query class
3. only pay semantic cost when exact and graph signals are insufficient
4. preserve explainability and provenance
5. update reinforcement or decay after outcome feedback

### Failure risks
- candidate explosion
- stale data bias
- low-confidence amplification
- cross-session interference

### Instrumentation
- p50/p95/p99 latency
- candidate counts before and after pruning
- success contribution estimate
- cache hit ratio
- repairability score

### Pseudocode
```text
fn pattern_577(ctx):
    budget = ctx.latency_budget
    exact = cheap_exact(ctx)
    graph = bounded_graph(ctx, exact)
    sem = maybe_semantic(ctx, budget, graph)
    merged = dedup_merge(exact, graph, sem)
    ranked = rank_with_policy(ctx, merged)
    return package(ctx, ranked[:ctx.limit])
```
## 578. Repair Orchestration pattern 8

### Objective
Optimize repair orchestration under bounded latency and bounded noise.

### Preconditions
- namespace is validated
- policy context is loaded
- latency budget is available
- tier capabilities are known

### Heuristic
1. start with the cheapest exact signals
2. cap expansion by query class
3. only pay semantic cost when exact and graph signals are insufficient
4. preserve explainability and provenance
5. update reinforcement or decay after outcome feedback

### Failure risks
- candidate explosion
- stale data bias
- low-confidence amplification
- cross-session interference

### Instrumentation
- p50/p95/p99 latency
- candidate counts before and after pruning
- success contribution estimate
- cache hit ratio
- repairability score

### Pseudocode
```text
fn pattern_578(ctx):
    budget = ctx.latency_budget
    exact = cheap_exact(ctx)
    graph = bounded_graph(ctx, exact)
    sem = maybe_semantic(ctx, budget, graph)
    merged = dedup_merge(exact, graph, sem)
    ranked = rank_with_policy(ctx, merged)
    return package(ctx, ranked[:ctx.limit])
```
## 579. Repair Orchestration pattern 9

### Objective
Optimize repair orchestration under bounded latency and bounded noise.

### Preconditions
- namespace is validated
- policy context is loaded
- latency budget is available
- tier capabilities are known

### Heuristic
1. start with the cheapest exact signals
2. cap expansion by query class
3. only pay semantic cost when exact and graph signals are insufficient
4. preserve explainability and provenance
5. update reinforcement or decay after outcome feedback

### Failure risks
- candidate explosion
- stale data bias
- low-confidence amplification
- cross-session interference

### Instrumentation
- p50/p95/p99 latency
- candidate counts before and after pruning
- success contribution estimate
- cache hit ratio
- repairability score

### Pseudocode
```text
fn pattern_579(ctx):
    budget = ctx.latency_budget
    exact = cheap_exact(ctx)
    graph = bounded_graph(ctx, exact)
    sem = maybe_semantic(ctx, budget, graph)
    merged = dedup_merge(exact, graph, sem)
    ranked = rank_with_policy(ctx, merged)
    return package(ctx, ranked[:ctx.limit])
```
## 580. Repair Orchestration pattern 10

### Objective
Optimize repair orchestration under bounded latency and bounded noise.

### Preconditions
- namespace is validated
- policy context is loaded
- latency budget is available
- tier capabilities are known

### Heuristic
1. start with the cheapest exact signals
2. cap expansion by query class
3. only pay semantic cost when exact and graph signals are insufficient
4. preserve explainability and provenance
5. update reinforcement or decay after outcome feedback

### Failure risks
- candidate explosion
- stale data bias
- low-confidence amplification
- cross-session interference

### Instrumentation
- p50/p95/p99 latency
- candidate counts before and after pruning
- success contribution estimate
- cache hit ratio
- repairability score

### Pseudocode
```text
fn pattern_580(ctx):
    budget = ctx.latency_budget
    exact = cheap_exact(ctx)
    graph = bounded_graph(ctx, exact)
    sem = maybe_semantic(ctx, budget, graph)
    merged = dedup_merge(exact, graph, sem)
    ranked = rank_with_policy(ctx, merged)
    return package(ctx, ranked[:ctx.limit])
```
## 581. Repair Orchestration pattern 11

### Objective
Optimize repair orchestration under bounded latency and bounded noise.

### Preconditions
- namespace is validated
- policy context is loaded
- latency budget is available
- tier capabilities are known

### Heuristic
1. start with the cheapest exact signals
2. cap expansion by query class
3. only pay semantic cost when exact and graph signals are insufficient
4. preserve explainability and provenance
5. update reinforcement or decay after outcome feedback

### Failure risks
- candidate explosion
- stale data bias
- low-confidence amplification
- cross-session interference

### Instrumentation
- p50/p95/p99 latency
- candidate counts before and after pruning
- success contribution estimate
- cache hit ratio
- repairability score

### Pseudocode
```text
fn pattern_581(ctx):
    budget = ctx.latency_budget
    exact = cheap_exact(ctx)
    graph = bounded_graph(ctx, exact)
    sem = maybe_semantic(ctx, budget, graph)
    merged = dedup_merge(exact, graph, sem)
    ranked = rank_with_policy(ctx, merged)
    return package(ctx, ranked[:ctx.limit])
```
## 582. Repair Orchestration pattern 12

### Objective
Optimize repair orchestration under bounded latency and bounded noise.

### Preconditions
- namespace is validated
- policy context is loaded
- latency budget is available
- tier capabilities are known

### Heuristic
1. start with the cheapest exact signals
2. cap expansion by query class
3. only pay semantic cost when exact and graph signals are insufficient
4. preserve explainability and provenance
5. update reinforcement or decay after outcome feedback

### Failure risks
- candidate explosion
- stale data bias
- low-confidence amplification
- cross-session interference

### Instrumentation
- p50/p95/p99 latency
- candidate counts before and after pruning
- success contribution estimate
- cache hit ratio
- repairability score

### Pseudocode
```text
fn pattern_582(ctx):
    budget = ctx.latency_budget
    exact = cheap_exact(ctx)
    graph = bounded_graph(ctx, exact)
    sem = maybe_semantic(ctx, budget, graph)
    merged = dedup_merge(exact, graph, sem)
    ranked = rank_with_policy(ctx, merged)
    return package(ctx, ranked[:ctx.limit])
```
## 583. Repair Orchestration pattern 13

### Objective
Optimize repair orchestration under bounded latency and bounded noise.

### Preconditions
- namespace is validated
- policy context is loaded
- latency budget is available
- tier capabilities are known

### Heuristic
1. start with the cheapest exact signals
2. cap expansion by query class
3. only pay semantic cost when exact and graph signals are insufficient
4. preserve explainability and provenance
5. update reinforcement or decay after outcome feedback

### Failure risks
- candidate explosion
- stale data bias
- low-confidence amplification
- cross-session interference

### Instrumentation
- p50/p95/p99 latency
- candidate counts before and after pruning
- success contribution estimate
- cache hit ratio
- repairability score

### Pseudocode
```text
fn pattern_583(ctx):
    budget = ctx.latency_budget
    exact = cheap_exact(ctx)
    graph = bounded_graph(ctx, exact)
    sem = maybe_semantic(ctx, budget, graph)
    merged = dedup_merge(exact, graph, sem)
    ranked = rank_with_policy(ctx, merged)
    return package(ctx, ranked[:ctx.limit])
```
## 584. Repair Orchestration pattern 14

### Objective
Optimize repair orchestration under bounded latency and bounded noise.

### Preconditions
- namespace is validated
- policy context is loaded
- latency budget is available
- tier capabilities are known

### Heuristic
1. start with the cheapest exact signals
2. cap expansion by query class
3. only pay semantic cost when exact and graph signals are insufficient
4. preserve explainability and provenance
5. update reinforcement or decay after outcome feedback

### Failure risks
- candidate explosion
- stale data bias
- low-confidence amplification
- cross-session interference

### Instrumentation
- p50/p95/p99 latency
- candidate counts before and after pruning
- success contribution estimate
- cache hit ratio
- repairability score

### Pseudocode
```text
fn pattern_584(ctx):
    budget = ctx.latency_budget
    exact = cheap_exact(ctx)
    graph = bounded_graph(ctx, exact)
    sem = maybe_semantic(ctx, budget, graph)
    merged = dedup_merge(exact, graph, sem)
    ranked = rank_with_policy(ctx, merged)
    return package(ctx, ranked[:ctx.limit])
```
## 585. Repair Orchestration pattern 15

### Objective
Optimize repair orchestration under bounded latency and bounded noise.

### Preconditions
- namespace is validated
- policy context is loaded
- latency budget is available
- tier capabilities are known

### Heuristic
1. start with the cheapest exact signals
2. cap expansion by query class
3. only pay semantic cost when exact and graph signals are insufficient
4. preserve explainability and provenance
5. update reinforcement or decay after outcome feedback

### Failure risks
- candidate explosion
- stale data bias
- low-confidence amplification
- cross-session interference

### Instrumentation
- p50/p95/p99 latency
- candidate counts before and after pruning
- success contribution estimate
- cache hit ratio
- repairability score

### Pseudocode
```text
fn pattern_585(ctx):
    budget = ctx.latency_budget
    exact = cheap_exact(ctx)
    graph = bounded_graph(ctx, exact)
    sem = maybe_semantic(ctx, budget, graph)
    merged = dedup_merge(exact, graph, sem)
    ranked = rank_with_policy(ctx, merged)
    return package(ctx, ranked[:ctx.limit])
```
## 586. Repair Orchestration pattern 16

### Objective
Optimize repair orchestration under bounded latency and bounded noise.

### Preconditions
- namespace is validated
- policy context is loaded
- latency budget is available
- tier capabilities are known

### Heuristic
1. start with the cheapest exact signals
2. cap expansion by query class
3. only pay semantic cost when exact and graph signals are insufficient
4. preserve explainability and provenance
5. update reinforcement or decay after outcome feedback

### Failure risks
- candidate explosion
- stale data bias
- low-confidence amplification
- cross-session interference

### Instrumentation
- p50/p95/p99 latency
- candidate counts before and after pruning
- success contribution estimate
- cache hit ratio
- repairability score

### Pseudocode
```text
fn pattern_586(ctx):
    budget = ctx.latency_budget
    exact = cheap_exact(ctx)
    graph = bounded_graph(ctx, exact)
    sem = maybe_semantic(ctx, budget, graph)
    merged = dedup_merge(exact, graph, sem)
    ranked = rank_with_policy(ctx, merged)
    return package(ctx, ranked[:ctx.limit])
```
## 587. Repair Orchestration pattern 17

### Objective
Optimize repair orchestration under bounded latency and bounded noise.

### Preconditions
- namespace is validated
- policy context is loaded
- latency budget is available
- tier capabilities are known

### Heuristic
1. start with the cheapest exact signals
2. cap expansion by query class
3. only pay semantic cost when exact and graph signals are insufficient
4. preserve explainability and provenance
5. update reinforcement or decay after outcome feedback

### Failure risks
- candidate explosion
- stale data bias
- low-confidence amplification
- cross-session interference

### Instrumentation
- p50/p95/p99 latency
- candidate counts before and after pruning
- success contribution estimate
- cache hit ratio
- repairability score

### Pseudocode
```text
fn pattern_587(ctx):
    budget = ctx.latency_budget
    exact = cheap_exact(ctx)
    graph = bounded_graph(ctx, exact)
    sem = maybe_semantic(ctx, budget, graph)
    merged = dedup_merge(exact, graph, sem)
    ranked = rank_with_policy(ctx, merged)
    return package(ctx, ranked[:ctx.limit])
```
## 588. Repair Orchestration pattern 18

### Objective
Optimize repair orchestration under bounded latency and bounded noise.

### Preconditions
- namespace is validated
- policy context is loaded
- latency budget is available
- tier capabilities are known

### Heuristic
1. start with the cheapest exact signals
2. cap expansion by query class
3. only pay semantic cost when exact and graph signals are insufficient
4. preserve explainability and provenance
5. update reinforcement or decay after outcome feedback

### Failure risks
- candidate explosion
- stale data bias
- low-confidence amplification
- cross-session interference

### Instrumentation
- p50/p95/p99 latency
- candidate counts before and after pruning
- success contribution estimate
- cache hit ratio
- repairability score

### Pseudocode
```text
fn pattern_588(ctx):
    budget = ctx.latency_budget
    exact = cheap_exact(ctx)
    graph = bounded_graph(ctx, exact)
    sem = maybe_semantic(ctx, budget, graph)
    merged = dedup_merge(exact, graph, sem)
    ranked = rank_with_policy(ctx, merged)
    return package(ctx, ranked[:ctx.limit])
```
## 589. Repair Orchestration pattern 19

### Objective
Optimize repair orchestration under bounded latency and bounded noise.

### Preconditions
- namespace is validated
- policy context is loaded
- latency budget is available
- tier capabilities are known

### Heuristic
1. start with the cheapest exact signals
2. cap expansion by query class
3. only pay semantic cost when exact and graph signals are insufficient
4. preserve explainability and provenance
5. update reinforcement or decay after outcome feedback

### Failure risks
- candidate explosion
- stale data bias
- low-confidence amplification
- cross-session interference

### Instrumentation
- p50/p95/p99 latency
- candidate counts before and after pruning
- success contribution estimate
- cache hit ratio
- repairability score

### Pseudocode
```text
fn pattern_589(ctx):
    budget = ctx.latency_budget
    exact = cheap_exact(ctx)
    graph = bounded_graph(ctx, exact)
    sem = maybe_semantic(ctx, budget, graph)
    merged = dedup_merge(exact, graph, sem)
    ranked = rank_with_policy(ctx, merged)
    return package(ctx, ranked[:ctx.limit])
```
## 590. Repair Orchestration pattern 20

### Objective
Optimize repair orchestration under bounded latency and bounded noise.

### Preconditions
- namespace is validated
- policy context is loaded
- latency budget is available
- tier capabilities are known

### Heuristic
1. start with the cheapest exact signals
2. cap expansion by query class
3. only pay semantic cost when exact and graph signals are insufficient
4. preserve explainability and provenance
5. update reinforcement or decay after outcome feedback

### Failure risks
- candidate explosion
- stale data bias
- low-confidence amplification
- cross-session interference

### Instrumentation
- p50/p95/p99 latency
- candidate counts before and after pruning
- success contribution estimate
- cache hit ratio
- repairability score

### Pseudocode
```text
fn pattern_590(ctx):
    budget = ctx.latency_budget
    exact = cheap_exact(ctx)
    graph = bounded_graph(ctx, exact)
    sem = maybe_semantic(ctx, budget, graph)
    merged = dedup_merge(exact, graph, sem)
    ranked = rank_with_policy(ctx, merged)
    return package(ctx, ranked[:ctx.limit])
```
## 591. Repair Orchestration pattern 21

### Objective
Optimize repair orchestration under bounded latency and bounded noise.

### Preconditions
- namespace is validated
- policy context is loaded
- latency budget is available
- tier capabilities are known

### Heuristic
1. start with the cheapest exact signals
2. cap expansion by query class
3. only pay semantic cost when exact and graph signals are insufficient
4. preserve explainability and provenance
5. update reinforcement or decay after outcome feedback

### Failure risks
- candidate explosion
- stale data bias
- low-confidence amplification
- cross-session interference

### Instrumentation
- p50/p95/p99 latency
- candidate counts before and after pruning
- success contribution estimate
- cache hit ratio
- repairability score

### Pseudocode
```text
fn pattern_591(ctx):
    budget = ctx.latency_budget
    exact = cheap_exact(ctx)
    graph = bounded_graph(ctx, exact)
    sem = maybe_semantic(ctx, budget, graph)
    merged = dedup_merge(exact, graph, sem)
    ranked = rank_with_policy(ctx, merged)
    return package(ctx, ranked[:ctx.limit])
```
## 592. Repair Orchestration pattern 22

### Objective
Optimize repair orchestration under bounded latency and bounded noise.

### Preconditions
- namespace is validated
- policy context is loaded
- latency budget is available
- tier capabilities are known

### Heuristic
1. start with the cheapest exact signals
2. cap expansion by query class
3. only pay semantic cost when exact and graph signals are insufficient
4. preserve explainability and provenance
5. update reinforcement or decay after outcome feedback

### Failure risks
- candidate explosion
- stale data bias
- low-confidence amplification
- cross-session interference

### Instrumentation
- p50/p95/p99 latency
- candidate counts before and after pruning
- success contribution estimate
- cache hit ratio
- repairability score

### Pseudocode
```text
fn pattern_592(ctx):
    budget = ctx.latency_budget
    exact = cheap_exact(ctx)
    graph = bounded_graph(ctx, exact)
    sem = maybe_semantic(ctx, budget, graph)
    merged = dedup_merge(exact, graph, sem)
    ranked = rank_with_policy(ctx, merged)
    return package(ctx, ranked[:ctx.limit])
```
## 593. Repair Orchestration pattern 23

### Objective
Optimize repair orchestration under bounded latency and bounded noise.

### Preconditions
- namespace is validated
- policy context is loaded
- latency budget is available
- tier capabilities are known

### Heuristic
1. start with the cheapest exact signals
2. cap expansion by query class
3. only pay semantic cost when exact and graph signals are insufficient
4. preserve explainability and provenance
5. update reinforcement or decay after outcome feedback

### Failure risks
- candidate explosion
- stale data bias
- low-confidence amplification
- cross-session interference

### Instrumentation
- p50/p95/p99 latency
- candidate counts before and after pruning
- success contribution estimate
- cache hit ratio
- repairability score

### Pseudocode
```text
fn pattern_593(ctx):
    budget = ctx.latency_budget
    exact = cheap_exact(ctx)
    graph = bounded_graph(ctx, exact)
    sem = maybe_semantic(ctx, budget, graph)
    merged = dedup_merge(exact, graph, sem)
    ranked = rank_with_policy(ctx, merged)
    return package(ctx, ranked[:ctx.limit])
```
## 594. Repair Orchestration pattern 24

### Objective
Optimize repair orchestration under bounded latency and bounded noise.

### Preconditions
- namespace is validated
- policy context is loaded
- latency budget is available
- tier capabilities are known

### Heuristic
1. start with the cheapest exact signals
2. cap expansion by query class
3. only pay semantic cost when exact and graph signals are insufficient
4. preserve explainability and provenance
5. update reinforcement or decay after outcome feedback

### Failure risks
- candidate explosion
- stale data bias
- low-confidence amplification
- cross-session interference

### Instrumentation
- p50/p95/p99 latency
- candidate counts before and after pruning
- success contribution estimate
- cache hit ratio
- repairability score

### Pseudocode
```text
fn pattern_594(ctx):
    budget = ctx.latency_budget
    exact = cheap_exact(ctx)
    graph = bounded_graph(ctx, exact)
    sem = maybe_semantic(ctx, budget, graph)
    merged = dedup_merge(exact, graph, sem)
    ranked = rank_with_policy(ctx, merged)
    return package(ctx, ranked[:ctx.limit])
```
## 595. Repair Orchestration pattern 25

### Objective
Optimize repair orchestration under bounded latency and bounded noise.

### Preconditions
- namespace is validated
- policy context is loaded
- latency budget is available
- tier capabilities are known

### Heuristic
1. start with the cheapest exact signals
2. cap expansion by query class
3. only pay semantic cost when exact and graph signals are insufficient
4. preserve explainability and provenance
5. update reinforcement or decay after outcome feedback

### Failure risks
- candidate explosion
- stale data bias
- low-confidence amplification
- cross-session interference

### Instrumentation
- p50/p95/p99 latency
- candidate counts before and after pruning
- success contribution estimate
- cache hit ratio
- repairability score

### Pseudocode
```text
fn pattern_595(ctx):
    budget = ctx.latency_budget
    exact = cheap_exact(ctx)
    graph = bounded_graph(ctx, exact)
    sem = maybe_semantic(ctx, budget, graph)
    merged = dedup_merge(exact, graph, sem)
    ranked = rank_with_policy(ctx, merged)
    return package(ctx, ranked[:ctx.limit])
```
## 596. Repair Orchestration pattern 26

### Objective
Optimize repair orchestration under bounded latency and bounded noise.

### Preconditions
- namespace is validated
- policy context is loaded
- latency budget is available
- tier capabilities are known

### Heuristic
1. start with the cheapest exact signals
2. cap expansion by query class
3. only pay semantic cost when exact and graph signals are insufficient
4. preserve explainability and provenance
5. update reinforcement or decay after outcome feedback

### Failure risks
- candidate explosion
- stale data bias
- low-confidence amplification
- cross-session interference

### Instrumentation
- p50/p95/p99 latency
- candidate counts before and after pruning
- success contribution estimate
- cache hit ratio
- repairability score

### Pseudocode
```text
fn pattern_596(ctx):
    budget = ctx.latency_budget
    exact = cheap_exact(ctx)
    graph = bounded_graph(ctx, exact)
    sem = maybe_semantic(ctx, budget, graph)
    merged = dedup_merge(exact, graph, sem)
    ranked = rank_with_policy(ctx, merged)
    return package(ctx, ranked[:ctx.limit])
```
## 597. Repair Orchestration pattern 27

### Objective
Optimize repair orchestration under bounded latency and bounded noise.

### Preconditions
- namespace is validated
- policy context is loaded
- latency budget is available
- tier capabilities are known

### Heuristic
1. start with the cheapest exact signals
2. cap expansion by query class
3. only pay semantic cost when exact and graph signals are insufficient
4. preserve explainability and provenance
5. update reinforcement or decay after outcome feedback

### Failure risks
- candidate explosion
- stale data bias
- low-confidence amplification
- cross-session interference

### Instrumentation
- p50/p95/p99 latency
- candidate counts before and after pruning
- success contribution estimate
- cache hit ratio
- repairability score

### Pseudocode
```text
fn pattern_597(ctx):
    budget = ctx.latency_budget
    exact = cheap_exact(ctx)
    graph = bounded_graph(ctx, exact)
    sem = maybe_semantic(ctx, budget, graph)
    merged = dedup_merge(exact, graph, sem)
    ranked = rank_with_policy(ctx, merged)
    return package(ctx, ranked[:ctx.limit])
```
## 598. Repair Orchestration pattern 28

### Objective
Optimize repair orchestration under bounded latency and bounded noise.

### Preconditions
- namespace is validated
- policy context is loaded
- latency budget is available
- tier capabilities are known

### Heuristic
1. start with the cheapest exact signals
2. cap expansion by query class
3. only pay semantic cost when exact and graph signals are insufficient
4. preserve explainability and provenance
5. update reinforcement or decay after outcome feedback

### Failure risks
- candidate explosion
- stale data bias
- low-confidence amplification
- cross-session interference

### Instrumentation
- p50/p95/p99 latency
- candidate counts before and after pruning
- success contribution estimate
- cache hit ratio
- repairability score

### Pseudocode
```text
fn pattern_598(ctx):
    budget = ctx.latency_budget
    exact = cheap_exact(ctx)
    graph = bounded_graph(ctx, exact)
    sem = maybe_semantic(ctx, budget, graph)
    merged = dedup_merge(exact, graph, sem)
    ranked = rank_with_policy(ctx, merged)
    return package(ctx, ranked[:ctx.limit])
```
## 599. Repair Orchestration pattern 29

### Objective
Optimize repair orchestration under bounded latency and bounded noise.

### Preconditions
- namespace is validated
- policy context is loaded
- latency budget is available
- tier capabilities are known

### Heuristic
1. start with the cheapest exact signals
2. cap expansion by query class
3. only pay semantic cost when exact and graph signals are insufficient
4. preserve explainability and provenance
5. update reinforcement or decay after outcome feedback

### Failure risks
- candidate explosion
- stale data bias
- low-confidence amplification
- cross-session interference

### Instrumentation
- p50/p95/p99 latency
- candidate counts before and after pruning
- success contribution estimate
- cache hit ratio
- repairability score

### Pseudocode
```text
fn pattern_599(ctx):
    budget = ctx.latency_budget
    exact = cheap_exact(ctx)
    graph = bounded_graph(ctx, exact)
    sem = maybe_semantic(ctx, budget, graph)
    merged = dedup_merge(exact, graph, sem)
    ranked = rank_with_policy(ctx, merged)
    return package(ctx, ranked[:ctx.limit])
```
## 600. Repair Orchestration pattern 30

### Objective
Optimize repair orchestration under bounded latency and bounded noise.

### Preconditions
- namespace is validated
- policy context is loaded
- latency budget is available
- tier capabilities are known

### Heuristic
1. start with the cheapest exact signals
2. cap expansion by query class
3. only pay semantic cost when exact and graph signals are insufficient
4. preserve explainability and provenance
5. update reinforcement or decay after outcome feedback

### Failure risks
- candidate explosion
- stale data bias
- low-confidence amplification
- cross-session interference

### Instrumentation
- p50/p95/p99 latency
- candidate counts before and after pruning
- success contribution estimate
- cache hit ratio
- repairability score

### Pseudocode
```text
fn pattern_600(ctx):
    budget = ctx.latency_budget
    exact = cheap_exact(ctx)
    graph = bounded_graph(ctx, exact)
    sem = maybe_semantic(ctx, budget, graph)
    merged = dedup_merge(exact, graph, sem)
    ranked = rank_with_policy(ctx, merged)
    return package(ctx, ranked[:ctx.limit])
```
