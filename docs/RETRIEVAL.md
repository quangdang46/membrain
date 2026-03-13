# RETRIEVAL

## Retrieval objective
Return the smallest evidence set that maximizes downstream task success.

## Retrieval modes
- exact retrieval
- recent retrieval
- semantic retrieval
- associative retrieval
- constraint retrieval
- reconstruction retrieval

## Candidate generation phases
1. direct key or id hints
2. tier1 active-window scan
3. tier2 exact index search
4. tier2 graph neighborhood expansion
5. tier2 semantic candidate generation
6. tier3 fallback
7. dedup and diversify
8. ranking
9. packaging

## Candidate explosion control
- hard caps by query type
- per-edge traversal budgets
- early-stop thresholds
- stale candidate penalties
- namespace pruning
- low-confidence suppression
- duplicate family collapse
- result diversity constraints
