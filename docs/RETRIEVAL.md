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

## Canonical recall query contract

All recall-facing transports should map onto one logical `RecallRequest` even when their syntax differs.

### Required core
- `query_text` is the primary cue text. It may be omitted only when `like_id` or `unlike_id` supplies the primary cue by reference.
- CLI `<QUERY>`, JSON-RPC `query` or `query_text`, and MCP task/goal text all populate this same canonical cue field rather than defining separate recall semantics.
- `mode` defaults to `auto` and may select `exact`, `recent`, `semantic`, `associative`, `constraint`, or `reconstruction`.
- `context_text` is optional caller-supplied task/session context that may sharpen ranking, but it must not silently replace the main cue.

### Scope and filters
- `namespace` names the requested effective namespace. Historical feature notes that say `namespace_id` for recall refer to this same input, not a second independent selector.
- `include_public` defaults to `false` and is the ordinary widening knob for approved shared/public surfaces.
- Optional scoped filters may include `workspace_id`, `agent_id`, `session_id`, `task_id`, `memory_kinds`, `era_id`, `as_of_tick`, `at_snapshot`, `min_strength`, `min_confidence`, `show_decaying`, and `mood_congruent`.
- `like_id` and `unlike_id` are query-by-example cues, not bypasses around policy, ranking, or boundedness rules.

### Budgets and explainability
- `result_budget`, `token_budget`, and `time_budget_ms` are caller hints; if more than one is present, the stricter bound wins.
- `effort` is `fast|normal|high` and tunes bounded candidate-generation and rerank budgets without exceeding hard system caps.
- `explain` is `none|summary|full` and controls requested explain verbosity, not whether routing/ranking traces exist internally.

### Graph and cold-path knobs
- `graph_mode` defaults to `auto` and may be `off` or `expand`, but every graph path remains subject to hard depth, node, and sibling caps.
- `cold_tier` defaults to `auto` and may be `avoid` or `allow`; it controls whether Tier3 candidate generation is considered, not whether cold payloads may be fetched before the final candidate cut.
- No request option may force pre-cut cold payload fetch, bypass namespace pruning, or override policy denial/redaction behavior.

### Cache and warm-path contract
- request-path caches, prefetch queues, and warm layers are derived accelerators, not authoritative evidence
- any cache or warm-path hit is valid only after request normalization, effective namespace binding, policy pruning, and owner-boundary checks for the current request
- warm-path optimizations may short-circuit expensive stages within bounded budgets, but they must not bypass namespace pruning, policy denial/redaction, sibling caps, or the no pre-cut cold payload fetch rule
- request-local reuse must track normalized request shape and relevant schema, index, policy, and ranking generations; item-, relation-, summary-, session-, task-, goal-, and process-local warm state must expire with its authoritative owner boundary
- if warm state is stale, version-mismatched, scoped too broadly, or missing a fresh generation anchor for the current request, the system must bypass it and continue on colder authoritative paths rather than serve an ambiguous hit
- absence, disablement, or repair of warm state may degrade latency but must not change the durable meaning of the request
- when cache or prefetch participation materially affects the route, explain and audit surfaces should preserve that fact in machine-readable metadata

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

### Request normalization rules
- Missing `query_text` is valid only when `like_id` or `unlike_id` is present.
- A request that combines incompatible time scopes such as `as_of_tick` and `at_snapshot`, or incompatible cue families that the interface cannot reconcile deterministically, must fail as validation error rather than guessing precedence.
- Unknown retrieval modes, invalid effort levels, malformed IDs, or malformed namespace values are validation failures before candidate generation.
- Omitted `namespace` is valid only when one deterministic default can be bound from authenticated context or stable session/job ownership.
- If request normalization widens scope to shared/public surfaces, the response must preserve that widening in explain/audit metadata.

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

## Pattern-completion contract
- pattern completion is a bounded recovery lane for fragmentary or partial-cue recall, not the default path when exact, recent, or indexed evidence already satisfies the request
- retrieval may enter this lane only after the normal direct and tiered shortlist has been scored, or when the caller explicitly asks for approximate or fragmentary recall
- expansion starts from a small scored seed set and may use local engram, graph, duplicate-family, entity, or temporal neighbors only within explicit per-seed caps
- metadata, snippets, and handles may be inspected during expansion, but cold or large payload fetch remains deferred until the final candidate cut
- partial-cue expansion must stop when node, depth, sibling, or payload budgets are exhausted, or when marginal gain falls below the lane's continuation threshold
- contradiction and supersession state remain first-class during pattern completion; fragmentary recall must not flatten open disagreement into one reconstructed answer

## Tip-of-the-tongue and reconstruction packaging
- if a single candidate or tightly bounded cluster survives the final cut, packaging may return a normal evidence set while recording that pattern completion assisted the route
- if no full candidate survives but bounded evidence fragments exist, the system must return an explicitly partial result instead of inventing the missing content
- partial results should expose anchored clues such as snippet spans, entity, time, or task matches, cluster or relation handles, matched cue dimensions, and why the system stopped short of a full answer
- reconstruction may combine multiple preserved fragments only when each fragment stays individually traceable to source memories and the package marks unresolved gaps or ambiguity explicitly
- low-signal or over-budget queries must end in a bounded miss or fragment shortlist, not a speculative completion

## Pattern-completion regression contract
- regression coverage must prove deterministic tier escalation, capped seed-set expansion, no pre-cut cold-payload fetch, explicit `full` versus `partial` versus `miss` result classification, and inspectable routing and ranking traces for the chosen lane
- adversarial cases must include near-duplicate cues, ambiguous entity or time hints, conflict or supersession siblings, and low-signal prompts that should terminate without speculative reconstruction
