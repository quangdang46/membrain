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
- per-conflict sibling caps
- result diversity constraints

## Ranking contract
- ranking runs only after namespace and policy pruning, candidate caps, dedup, and per-conflict sibling caps have been applied
- default final ordering is `baseline fusion -> optional bounded rerank -> packaging`, where reranking is allowed only on a small top-K shortlist
- baseline score families must stay separately inspectable: retrieval relevance, recency/strength/salience, confidence/utility, goal-task-entity-context alignment, memory-type priors, graph support, contradiction or supersession state, and duplicate/noise penalties
- reranking may sharpen session, task, entity, or packaging priorities, but it must not bypass hard policy masks, hide losing conflict evidence, or require unbounded payload fetches
- final ordering must preserve a machine-readable decomposition with baseline family scores, rerank adjustments, notable penalties or bonuses, and the final packaged order reason

## Conflict-aware retrieval contract
- contradiction state is a first-class retrieval and ranking input, not a post-processing guess from free-form text
- unresolved conflicts remain directly queryable and keep both sides eligible for bounded recall, inspect, and audit flows
- superseded memories stay preserved and inspectable; default recall may prefer the operative winner, but it must retain the losing evidence and chain links
- authoritative overrides may change the default packaged answer, but they must preserve the losing evidence plus the authority source and resolution reason
- retrieval may expand from a candidate to its linked conflict siblings or `ConflictRecord` artifacts only within explicit per-candidate caps

## Conflict-aware packaging rules
- returned candidates must carry machine-readable conflict metadata when present, including `conflict_state`, `conflict_record_ids`, `belief_chain_id`, and `superseded_by`
- packaged results may prioritize a preferred memory for normal task use, but they must still expose open disagreement, suppressed alternatives, or omitted conflict siblings when caps prevent returning the whole set
- duplicate-family collapse must not blend contradictory evidence into one synthetic statement
- inspect, explain, ranking, and repair flows must be able to reconstruct contradiction state from durable conflict artifacts plus preserved lineage and provenance
