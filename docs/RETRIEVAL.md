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
- `era_id` scopes recall to one explicit temporal era within the already-bound effective namespace. Unknown, malformed, or unauthorized era selectors are validation or policy failures rather than cues to widen, guess, or silently fall back to cross-era retrieval.
- `mood_congruent` is a later-stage, opt-in ranking hint layered on top of the ordinary bounded recall pipeline. It may slightly boost candidates whose stored `encoding_valence` / `encoding_arousal` metadata fit the current bounded mood snapshot or requested mood context, but it must not widen namespace scope, bypass policy filters, invent emotional metadata when none is available, or become a hidden default retrieval mode.
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
- prefetch hints remain bound to the current session or task intent and may warm only handles or bounded shortlists; they must be canceled when intent, namespace scope, or policy scope changes
- session warmup may preload a bounded session-local hot set, but every warmed family still needs fresh generation anchors before reuse on the live request path
- process-local cold-start mitigation may reduce bootstrap latency, but request-visible reuse still must bind the current effective namespace, owner boundary, and relevant model or index generations
- if warm state is stale, version-mismatched, scoped too broadly, or missing a fresh generation anchor for the current request, the system must bypass it and continue on colder authoritative paths rather than serve an ambiguous hit
- absence, disablement, or repair of warm state may degrade latency but must not change the durable meaning of the request
- when cache or prefetch participation materially affects the route, explain and audit surfaces should preserve that fact in machine-readable metadata, including cache family, cache event, cache reason, warm source, and generation status
- stale or invalidated warm state must surface as an explicit warning or bypass reason rather than being silently recoded as an ordinary miss
- route metadata should preserve candidate counts before and after cache-influenced stages and whether degraded mode or cache-disabled serving forced colder fallback

## Candidate generation phases
1. direct key or id hints, including Tier1 exact-handle get when resident
2. tier1 active-window scan for recent/hot reuse
3. tier2 exact index search
4. tier2 semantic candidate generation
5. bounded tier2 graph neighborhood expansion from the capped shortlist
6. tier3 fallback
7. dedup and diversify
8. ranking
9. packaging

## Canonical retrieval/result packaging object
- the packaging stage emits the canonical `RetrievalResult` envelope defined in `PLAN.md` Section 34.3 rather than a transport-specific answer blob
- `outcome_class` communicates whether the bounded retrieval path ended as `accepted`, `partial`, `degraded`, `blocked`, `preview`, or `rejected`
- `evidence_pack` is the primary retrieval payload: bounded returned evidence with per-item role (`primary` or `supporting`), lane/route provenance, score summaries, provenance summaries, freshness markers, conflict markers, and payload state such as inline, preview-only, deferred, or redacted
- `action_pack` is optional derived guidance layered on top of the evidence set; it may summarize, recommend, or format next steps, but it must keep supporting evidence ids/handles and uncertainty markers so synthesis never replaces provenance
- `deferred_payloads` is the canonical place to record payloads intentionally not hydrated before or after the final cut because of budget, policy, redaction, or degraded-serving constraints
- `omitted_summary`, `policy_summary`, `provenance_summary`, `freshness_markers`, and `conflict_markers` remain first-class result fields rather than human-only narration so callers can inspect why the package looks incomplete, filtered, stale, or conflict-aware
- CLI, daemon/JSON-RPC, and MCP may render this object differently for humans, but they must preserve the same evidence-versus-action split and the same omission, policy, freshness, conflict, and deferred-payload semantics

## Canonical RetrievalResult field contract

The canonical `RetrievalResult` envelope is a stable machine-readable object shared across CLI JSON, daemon/JSON-RPC, IPC, and MCP recall-facing surfaces.

### Required top-level fields

| Field | Required | Contract |
|-------|----------|----------|
| `outcome_class` | yes | One of `accepted`, `partial`, `preview`, `blocked`, `degraded`, or `rejected`. This is the shared retrieval/result status vocabulary across transports. |
| `evidence_pack` | yes | Bounded returned evidence set used for answer construction. Present even when empty. |
| `action_pack` | no | Optional derived answer/recommendation layer. Never replaces `evidence_pack`. |
| `omitted_summary` | yes | Machine-readable omission counts and reasons so truncation, redaction, suppression, or budget cuts never masquerade as consensus. |
| `policy_summary` | yes | Effective namespace, widening/redaction/denial state, and any policy-driven payload or sibling suppression that shaped the visible result. |
| `provenance_summary` | yes | Source-kind mix, lineage anchors, and derived-versus-raw evidence mix for the packaged result. |
| `freshness_markers` | yes | Time-sensitivity and lifecycle markers that materially affect safe use of the result. |
| `conflict_markers` | yes | Conflict/supersession/override state for the packaged result set. |
| `deferred_payloads` | no | Handles for payloads intentionally not materialized yet, including reason and hydration conditions. |
| `packaging_metadata` | yes | Prompt-construction and downstream-consumption facts such as result budget, token budget, graph-assistance summary, cache/degraded summary, and packaging mode. |
| `explain_handle` or embedded explanation families | yes | Either a stable handle for deferred explanation or embedded `route_summary`, `result_reasons`, `omitted_summary`, `policy_summary`, `provenance_summary`, `freshness_markers`, `conflict_markers`, and `trace_stages` when full trace detail is requested. |

### `evidence_pack` item contract

Each returned evidence item must preserve enough structure that downstream CLI/MCP/daemon surfaces do not need transport-specific guesswork.

| Field | Required | Contract |
|-------|----------|----------|
| `memory_id` or canonical handle | yes | Stable durable identity or canonical opaque handle for the returned evidence item. |
| `role` | yes | `primary` or `supporting`. Supporting items must not be silently merged into primary evidence. |
| `entry_lane` | yes | The lane or route by which the item entered the bounded shortlist, such as `exact`, `recent`, `lexical`, `semantic`, `graph`, or `cold_fallback`. |
| `payload_state` | yes | One of `inline`, `preview_only`, `deferred`, or `redacted`. |
| `score_summary` | yes | Bounded score decomposition sufficient to explain ranking and packaging without exposing unstable internals as mandatory schema. |
| `provenance_summary` | yes | Per-item source kind, source reference or opaque handle, and lineage anchors. |
| `freshness_markers` | yes | Per-item freshness/lease/snapshot/stale-derived markers when applicable. |
| `conflict_markers` | yes | Per-item contradiction, supersession, override, or omitted-sibling state when applicable. |
| `uncertainty` markers | when available | Includes `uncertainty_score` and its component fields when available; high-stakes paths must include this surface by default. |
| `payload` / snippet / preview | conditional | Included only when allowed by `payload_state`, budget, and policy. |

### `action_pack` contract

When present, `action_pack` is the only stable home for synthesized answer text, recommended actions, or next-step guidance. It must preserve:
- the synthesized content or structured action payload,
- supporting evidence ids or handles,
- uncertainty markers that qualify the synthesis,
- any policy or freshness caveats that materially constrain safe use.

`output_mode` controls whether the action pack survives packaging:
- `strict` keeps the evidence pack authoritative and suppresses derived actions when they carry blocking uncertainty markers (`low_confidence`, `high_uncertainty`, `missing_evidence`, `reconsolidation_churn`) or any policy/freshness caveat.
- `balanced` permits derived actions only when their confidence remains at least medium and leaves lower-confidence guidance in the evidence pack/explain surfaces instead of foregrounding it as action output.
- `fast` preserves any derived action artifact that survived earlier policy filtering so interfaces can optimize for speed over conservatism.

### `omitted_summary` contract

`omitted_summary` must preserve counts and reasons for at least these families when relevant:
- `budget_capped`
- `policy_filtered`
- `redacted`
- `duplicate_collapsed`
- `low_confidence_suppressed`
- `stale_bypassed`
- `conflict_siblings_omitted`
- `deferred_payload_only`

### `policy_summary` contract

`policy_summary` must preserve:
- `effective_namespace`
- approved widening such as `include_public`
- redaction or denial state that shaped the packaged answer
- whether policy prevented payload hydration or sibling disclosure

### `provenance_summary` contract

`provenance_summary` must preserve:
- source-kind mix,
- lineage anchors or handles,
- whether returned artifacts are raw evidence, summaries, extracted facts, or other derived artifacts.

### `freshness_markers` contract

`freshness_markers` must preserve relevant markers such as:
- `decaying_soon`
- `snapshot_scoped`
- `as_of_scoped`
- `lease_sensitive`
- `recheck_required`
- `stale_derived`
- `archival_recovery_partial`

When lease-sensitive evidence is action-critical, the packaged result must disclose that the item requires re-check or withholding instead of silently keeping a high-confidence answer surface.

### `conflict_markers` contract

`conflict_markers` must preserve relevant markers such as:
- `open_conflict`
- `superseded`
- `authoritative_override`
- `preferred_operational_answer`
- `omitted_conflict_sibling`

### Sample outcome shapes

These examples are normative for field presence and semantic meaning, not exact formatting.

#### Accepted evidence-only result

```json
{
  "outcome_class": "accepted",
  "evidence_pack": [
    {
      "memory_id": "mem_123",
      "role": "primary",
      "entry_lane": "semantic",
      "payload_state": "inline",
      "score_summary": { "final_score": 0.91 },
      "provenance_summary": { "source_kind": "raw_memory", "lineage": ["enc_77"] },
      "freshness_markers": [],
      "conflict_markers": [],
      "uncertainty_score": 0.08
    }
  ],
  "omitted_summary": { "budget_capped": 2 },
  "policy_summary": { "effective_namespace": "project-x", "include_public": false },
  "provenance_summary": { "source_kinds": ["raw_memory"], "derived_artifacts_present": false },
  "freshness_markers": [],
  "conflict_markers": [],
  "packaging_metadata": { "result_budget": 5, "packaging_mode": "evidence_only" },
  "explain_handle": "exp_456"
}
```

#### Partial result with deferred/redacted payloads and conflict visibility

```json
{
  "outcome_class": "partial",
  "evidence_pack": [
    {
      "memory_id": "mem_200",
      "role": "primary",
      "entry_lane": "exact",
      "payload_state": "preview_only",
      "score_summary": { "final_score": 0.84 },
      "provenance_summary": { "source_kind": "summary_artifact", "lineage": ["mem_120", "mem_121"] },
      "freshness_markers": ["snapshot_scoped"],
      "conflict_markers": ["open_conflict", "omitted_conflict_sibling"]
    }
  ],
  "omitted_summary": {
    "policy_filtered": 1,
    "conflict_siblings_omitted": 1,
    "deferred_payload_only": 1
  },
  "policy_summary": {
    "effective_namespace": "project-x",
    "include_public": true,
    "payload_hydration_blocked": true
  },
  "provenance_summary": {
    "source_kinds": ["summary_artifact", "raw_memory"],
    "derived_artifacts_present": true
  },
  "freshness_markers": ["snapshot_scoped"],
  "conflict_markers": ["open_conflict", "preferred_operational_answer"],
  "deferred_payloads": [
    {
      "handle": "payload_9",
      "reason": "budget_capped",
      "hydrate_via": "inspect",
      "condition": "remaining_token_budget"
    }
  ],
  "packaging_metadata": {
    "result_budget": 3,
    "graph_assistance": { "used": true, "added_supporting": 1 },
    "packaging_mode": "evidence_plus_action"
  },
  "route_summary": { "planner": "tier2_then_graph", "tier3_escalated": false }
}
```

### Regression obligations

Schema and serialization tests for future implementation beads must prove that:
- all transports preserve the same top-level field families and semantic meaning,
- `action_pack` never replaces or silently redefines `evidence_pack`,
- omission, freshness, provenance, and conflict markers survive transport-specific formatting,
- partial, blocked, degraded, and preview outcomes remain distinguishable,
- sample accepted and partial/deferred/conflict-bearing outputs remain representable without inventing transport-local fields.

## Tier1 exact and recent retrieval contract
- Tier1 is the first bounded retrieval surface for exact and recent recall. It is an in-process derived accelerator, not authoritative evidence.
- Tier1 may return or shortlist only already-authorized items after request normalization, deterministic effective-namespace binding, and policy or owner-boundary checks for the live request.
- Tier1 exact retrieval is a direct handle path: given a stable memory id, external id mapping, or deterministic exact cue that resolves to one hot item, the system should attempt an O(1)-style lookup against Tier1-resident hot metadata before consulting Tier2.
- Tier1 recent retrieval is a bounded active-window scan over a recency-ordered ring buffer or equivalent bounded hot structure for one effective namespace. The scan budget is capped by the window and query-class limits, never by total durable corpus size.
- Tier1 entries may carry stable ids, compact text or snippets, recency markers, strength or salience scalars, freshness or generation anchors, and policy-bearing hot metadata needed to validate reuse; Tier1 must not own giant payloads or become the sole durable source.
- Exact or recent Tier1 outcomes must remain inspectable as hit, miss, bypass, or stale-bypass style events, including whether the exact-handle lane or recent-window lane fired and how many recent candidates were inspected when that materially affects the route.
- If the current request cannot prove a Tier1 entry is valid for its namespace, policy, version, or freshness anchors, the system must bypass that entry and continue on the colder canonical path rather than serving an ambiguous hit.
- Tier1 participation may short-circuit later candidate generation for a satisfied request, but it must not trigger ANN search, graph expansion, namespace widening, or pre-cut cold payload fetch inside the Tier1 lane itself.
- Successful encode, successful recall, and successful slower-tier retrieval may seed or refresh Tier1 for bounded reuse, but seeding, eviction, or refresh does not by itself change canonical durable ownership, archive state, or supersession state.
- When Tier1 is seeded from the encode fast path, the seed must reflect only the bounded synchronous outputs already frozen before persistence, such as the normalized envelope, stable fingerprint hint, shallow class, provisional salience, and hot metadata needed for later exact or recent reuse. Encode-side Tier1 seeding must remain inspectable through structured trace or log fields that name the fast-path stages, seeding decision, and any duplicate-hint candidate count consulted on that path.

## Tier2 indexed retrieval contract
- Tier2 is the warm indexed retrieval surface and the normal request-path workhorse after Tier1. It is the primary home for exact search, structured filtering, bounded lexical retrieval, and metadata-first semantic candidate generation over hot durable state.
- Tier2 evaluates only already-normalized requests with a deterministic effective namespace and live policy scope. Namespace widening, policy pruning, lifecycle eligibility, and owner-boundary checks must bind before structured indexes, FTS, ANN, or graph-adjacent expansion are allowed to contribute candidates.
- Tier2 authoritative inputs live in hot durable metadata, hot bounded text surfaces, authoritative embeddings, and canonical link tables. FTS projections, ANN sidecars, and other Tier2 indexes are derived accelerators; if they diverge, the authoritative hot durable rows win and the derived surfaces are rebuilt or bypassed.
- Tier2 begins with a metadata-first prefilter over narrow queryable fields such as namespace, visibility, lifecycle or tier eligibility, canonical type or kind, relation or cluster handles, and rank-driving scalars like strength, confidence, or salience. The prefilter must bound the eligible id set before ANN or graph work and must not touch detached cold payloads or giant text blobs.
- The exact/indexed lane may satisfy or seed retrieval through stable id lookup, structured secondary indexes, and bounded lexical projections such as `compact_text`/FTS over hot text surfaces. Exact or lexical participation must remain inspectable as a distinct Tier2 lane rather than being hidden inside the semantic path.
- The semantic lane may run only on the prefiltered eligible set. At hot-cardinality scale it remains bounded by an explicit candidate budget rather than total hot corpus size; canonical plan examples use a prefilter cap around 5,000 ids, an ANN shortlist around top-100 hits, and a smaller full-precision rescore slice around top-20 candidates.
- Tier2 may merge exact/lexical and semantic shortlists before dedup, ranking, or bounded graph expansion, but it must preserve candidate provenance by lane so inspect surfaces can explain whether an item entered via exact, lexical, semantic, or later expansion logic.
- If Tier2 hands off to graph expansion, that handoff remains subject to separate depth, node, and sibling caps; Tier2 does not grant graph paths license to reopen the full corpus.
- Tier2 may short-circuit the colder path when the bounded shortlist produces an inspectable confident result, and it may hand a capped shortlist to later ranking or packaging stages. If Tier2 exhausts its declared budget, produces only low-confidence candidates, or detects stale or invalid derived surfaces, the planner must escalate to Tier3 or colder authoritative paths instead of widening scope or performing an implicit full scan.
- Route metadata for Tier2 should preserve the prefilter candidate count, which indexed lanes fired, ANN shortlist size, rescore slice size, early-stop or bypass reasons, and whether Tier3 escalation happened.
- Encode, reconsolidation, and repair flows mutate authoritative hot durable rows first and refresh Tier2 indexes second. A stale or missing Tier2 index is a latency or retrieval-quality defect, not a truth-loss event; repair may discard and rebuild derived Tier2 indexes from authoritative hot durable evidence.

## Tier3 cold fallback contract
- Tier3 is the cold fallback retrieval lane over authoritative cold durable rows for consolidated or archived memories whose canonical durable ownership lives off the hot path. It is a bounded request-path fallback, not the default first resort when Tier1 or Tier2 can already satisfy the request.
- `cold_tier=avoid` suppresses Tier3 candidate generation for ordinary recall and should return the best hotter bounded outcome, degraded partial, or miss rather than silently overriding the hint. `cold_tier=auto` allows planner-driven escalation when hotter lanes exhaust their declared budget, return no eligible candidates, or surface only low-confidence or archive-missing results. `cold_tier=allow` permits Tier3 consideration without skipping earlier normalization, policy, and candidate-trimming stages.
- Escalation into Tier3 occurs only after request normalization, deterministic effective-namespace binding, policy or owner-boundary checks, and the bounded Tier1/Tier2 lanes have either failed to satisfy the request or produced an inspectable reason to consult cold evidence. Tier3 entry must remain inspectable as a deliberate planner decision, not an implicit widening of scope.
- Tier3 begins with metadata-first SQL prefiltering over cold durable rows using namespace, visibility, lifecycle or archive eligibility, retention state, contradiction or supersession handles, canonical type/kind, and other rank-driving scalars needed for the first bounded cut. This prefilter must bound the cold eligible set before any cold ANN probe, lexical projection, or payload materialization.
- Tier3 may use stable-id or exact lookup, bounded lexical preview or snippet search, and cold ANN over the prefiltered eligible set. ANN sidecars, preview surfaces, and other cold accelerators remain derived lanes; when they diverge from `cold.db`, authoritative cold durable rows and cold embedding records win and the derived surfaces are rebuilt or bypassed.
- Tier3 candidate generation may inspect only cold metadata, bounded preview text, snippet surfaces, authoritative embeddings or refs, and archive-control metadata needed to filter, rank, inspect, and explain. It must not decompress detached payloads, fetch object bodies, or materialize large cold content before the final candidate cut.
- Detached cold payload materialization is permitted only after dedup, ranking, and the final bounded candidate cut, and only for the small winning set needed for final packaging, inspect, or explicit full-result assembly. If a winning cold record is redacted, tombstoned, unavailable, or over the remaining budget, the response must degrade explicitly to bounded preview, partial, or miss semantics rather than widen the cut or fetch more payloads speculatively.
- If cold derived lanes are stale, missing, or under repair, Tier3 may bypass those lanes and continue with colder authoritative metadata or preview paths inside the declared budget, or return an explicit degraded, partial, or miss outcome. It must not respond by falling back to an unbounded full-store scan or hiding the loss of cold evidence.
- Route metadata for Tier3 should preserve whether the cold lane was suppressed, considered, or entered; why escalation happened; the cold prefilter candidate count; which cold lanes fired; ANN shortlist or rescore slice sizes when used; how many cold payload fetches were deferred until after the final cut; and whether final packaging ended as `full`, `partial`, `miss`, or degraded.

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

### Bounded graph-expansion contract
- graph expansion may open only from a bounded, already-authorized seed shortlist produced by earlier direct, exact, lexical, or semantic stages; graph traversal must not bootstrap itself from the whole corpus or reopen namespaces that earlier stages excluded.
- the default request-path traversal is priority-weighted BFS from the current seed set, with higher-confidence or higher-weight edges explored before weaker edges, but always under the same hard continuation rules.
- every traversal must enforce at least a hard max depth, hard max collected-node budget, and minimum traversable edge weight; the current canonical defaults remain `max_depth=3`, `max_nodes=50`, and `min_edge_weight=0.5` unless a later benchmarked contract changes them explicitly.
- traversal may inspect only bounded metadata, snippets, handles, and graph-local scalars while expanding; large or cold payload fetch remains deferred until the final candidate cut.
- traversal continues only while the next hop stays within depth, node, sibling, and time budgets and still improves the bounded candidate set; when the next eligible hop falls below the lane's continuation threshold, expansion stops rather than widening heuristics.
- graph expansion must be skippable without semantic corruption: if `graph_mode=off`, the graph is degraded or disabled, the bounded seed set is empty, or request-path budgets are already exhausted, retrieval should continue with a graph-bypassed route instead of forcing traversal.
- graph-disabled or graph-capped serving must remain inspectable. Explain and inspect surfaces should preserve whether graph expansion ran, why it was skipped, which seeds opened the traversal, how many hops and nodes were consumed, and whether a cap or degraded-mode fallback terminated expansion early.
- contradiction, supersession, duplicate-family, and namespace-denial boundaries remain in force during traversal; BFS may surface related evidence, but it must not flatten conflicts, bypass policy masks, or manufacture one synthetic answer from mutually incompatible nodes.

### Request normalization rules
- Missing `query_text` is valid only when `like_id` or `unlike_id` is present.
- A request that combines incompatible time scopes such as `as_of_tick` and `at_snapshot`, or incompatible cue families that the interface cannot reconcile deterministically, must fail as validation error rather than guessing precedence.
- Unknown retrieval modes, invalid effort levels, malformed IDs, or malformed namespace values are validation failures before candidate generation.
- Omitted `namespace` is valid only when one deterministic default can be bound from authenticated context or stable session/job ownership.
- If request normalization widens scope to shared/public surfaces, the response must preserve that widening in explain/audit metadata.

## Uncertainty surface contract

- **uncertainty** is a richer, multi-dimensional measure of reliability than the scalar `confidence` field. While `confidence` tracks belief strength after corroboration and reconsolidation, **uncertainty** aggregates evidence gaps, temporal staleness, contradiction state, and source sparsity into a composite reliability indicator.

- Each memory may carry multiple uncertainty dimensions that combine into an overall score and optional confidence interval bounds.

### Uncertainty dimensions

| Dimension | Description | Source |
|-----------|-------------|--------|
| **corroboration_uncertainty** | Uncertainty from lack of supporting evidence. More corroboration → lower uncertainty. |
| **freshness_uncertainty** | Uncertainty from temporal staleness. Older memories that haven't been accessed recently have higher uncertainty. |
| **contradiction_uncertainty** | Uncertainty from conflict state. Conflicted or superseded memories have elevated uncertainty. |
| **missing_evidence_uncertainty** | Uncertainty from sparse source support. Memories with few causal links or weak authoritativeness have higher uncertainty. |

### Combined uncertainty scoring

```rust
// Combined uncertainty score (0.0 = most certain, 1.0 = most uncertain)
fn compute_uncertainty(memory: &Memory, now_tick: u64) -> f32 {
    let corroboration_factor = 1.0 / (1.0 + memory.corroboration_count as f32).max(0.8);
    let freshness_factor = if let Some(last_access) = last_access {
        let days_since_access = (now_tick - last_access) / 86400.0; // Assuming 10ms ticks
        (days_since_access / FRESHNESS_THRESHOLD_DAYS).min(1.0) // 365 days
    } else {
        1.0 // No access history = high uncertainty
    };
    let conflict_factor = match memory.conflict_state {
        ConflictState::None | ConflictState::Resolved => 0.0,
        ConflictState::Open | ConflictState::Superseded => 0.3,
    };
    let evidence_factor = if memory.has_causal_parents == 0 && memory.authoritativeness < 0.5 {
        1.0 // Little to no evidence
    } else {
        (memory.authoritativeness / 1.0).min(1.0) // Normalize to [0,1]
    };

    // Weighted combination (adjustable per deployment/retrieval scenario)
    let base_uncertainty = (
        corroboration_factor * CORROBORATION_WEIGHT +
        freshness_factor * FRESHNESS_WEIGHT +
        conflict_factor * CONFLICT_WEIGHT +
        evidence_factor * EVIDENCE_WEIGHT
    ) / (CORROBORATION_WEIGHT + FRESHNESS_WEIGHT + CONFLICT_WEIGHT + EVIDENCE_WEIGHT);

    // Clamp to [0,1]
    base_uncertainty.max(1.0)
}
```

### Confidence intervals

For high-stakes or action-oriented paths, uncertainty should be exposed as **confidence intervals** rather than a single point estimate:

```rust
pub struct UncertaintyBounds {
    pub lower_bound: f32,  // confidence - uncertainty_margin
    pub upper_bound: f32,  // confidence + uncertainty_margin
    pub margin: f32,         // Uncertainty quantile (e.g., 95% CI uses 2×stderr)
    pub confidence_level: f32,  // Underlying scalar confidence
}

fn compute_confidence_interval(
    confidence: f32,
    uncertainty: f32,
    quantile_multiplier: f32, // e.g., 1.96 for 95% CI
) -> UncertaintyBounds {
    let margin = uncertainty * quantile_multiplier;
    UncertaintyBounds {
        lower_bound: (confidence - margin).max(0.0),
        upper_bound: (confidence + margin).min(1.0),
        margin,
        confidence_level: confidence,
    }
}
```

### Uncertainty in RetrievalResult

- `evidence_pack` items must include per-item uncertainty fields when available:
  - `uncertainty_score`: combined 0-1 measure
  - `corroboration_uncertainty`: contribution from lack of corroboration
  - `freshness_uncertainty`: contribution from staleness
  - `contradiction_uncertainty`: contribution from conflict state
  - `missing_evidence_uncertainty`: contribution from sparse evidence
  - Optional `confidence_interval`: bounds for high-stakes paths

- `action_pack` or synthesis must preserve uncertainty markers so users understand limitations of recommendations.
- Rankings should incorporate uncertainty as a penalty factor for high-uncertainty candidates when multiple high-confidence options exist.

### High-stakes uncertainty requirement

- High-stakes paths (actions, decisions, policy operations, safety-critical recalls) **MUST** include uncertainty surfaces by default.
- Normal retrieval may include uncertainty, but it may be suppressed for low-uncertainty results.
- The system never hides uncertainty behind optional diagnostics-only tools for high-stakes operations.

## Ranking contract
- ranking runs only after namespace and policy pruning, candidate caps, dedup, and per-conflict sibling caps have been applied
- default final ordering is `baseline fusion -> optional bounded rerank -> packaging`, where reranking is allowed only on a small top-K shortlist
- baseline score families must stay separately inspectable: retrieval relevance, recency/strength/salience, confidence/utility, goal-task-entity-context alignment, memory-type priors, graph support, contradiction or supersession state, and duplicate/noise penalties
- emotional-trajectory participation is later-stage and optional. When `mood_congruent` is enabled, any valence/arousal bonus must remain a small additive ranking family over already-eligible candidates rather than a new candidate-generation lane, and explain surfaces should preserve whether the bonus materially changed ordering or was present but non-decisive.
- reranking may sharpen session, task, entity, or packaging priorities, but it must not bypass hard policy masks, hide losing conflict evidence, or require unbounded payload fetches
- final ordering must preserve a machine-readable decomposition with baseline family scores, rerank adjustments, notable penalties or bonuses, and the final packaged order reason

## Graph participation in ranking and packaging
- graph support is an additive bounded score family derived from inspectable graph facts such as seed provenance, hop depth, path weight, and whether the candidate also entered through a direct lane; it must not replace primary query relevance or hide that a candidate was graph-assisted.
- graph-expanded neighbors are supporting evidence, not automatic winners. A candidate should not outrank a materially stronger direct, exact, lexical, or semantic match solely because it shares an engram or neighborhood path.
- if a candidate entered through both a direct lane and graph expansion, ranking and packaging may combine those reasons, but explain surfaces must preserve both sources of support rather than collapsing the item into one opaque graph hit.
- packaging should distinguish primary evidence from supporting associative context. When graph expansion contributes returned neighbors, the package should preserve which items were direct anchors or seed-aligned hits and which items were admitted as graph-supported context.
- if graph assistance materially affects inclusion or ordering, the packaged result or its explanation should preserve the seed or engram handle that opened expansion, the bounded hop or path-strength summary, and whether graph support changed ranking, only added supporting context, or was present but non-decisive.
- graph-only neighbors may appear in the final packaged set only if they still satisfy bounded rescoring against the user cue; when caps force trade-offs, the response should keep the strongest direct evidence visible instead of replacing the whole answer with opaque associative context.
- if graph-expanded context is omitted by caps or policy, the existing omitted-summary and trace surfaces should say so explicitly rather than implying the returned set exhausted the associative neighborhood.

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

## Explain and inspect surface contract
- `explain=summary` is the default result-consumption surface: it should say why returned items appeared, which major route choices fired, which policy or budget boundaries mattered, which historical boundary was selected for time-travel retrieval, and which freshness or conflict markers affect use of the result
- `explain=full` or explicit inspect mode should add stage-by-stage routing traces, including candidate entry reasons, exclusion reasons, candidate counts, graph hops, cache and tier decisions, baseline score families, rerank deltas, and final packaging reasons
- `explain=none` may suppress embedded explanation in the main response, but it must not change retrieval semantics or prevent later inspection through an explanation handle or equivalent trace reference
- explanation surfaces must distinguish why an item appeared from why alternatives did not, including policy-filtered, budget-capped, duplicate-collapsed, low-confidence, superseded, stale-bypassed, or conflict-suppressed outcomes while respecting redaction boundaries
- provenance summaries should identify source kind, source reference or opaque handle, lineage ancestry, and any summary or consolidation ancestry needed to inspect the returned item without treating derived artifacts as sole truth
- freshness markers should surface decaying-soon, snapshot or as-of scoping, stale-derived warnings, and other time-sensitivity signals; historical retrieval should preserve a machine-readable `historical_context` block naming the selected `window_kind`, `selection_reason`, optional `selected_tick_window`, applied `as_of_tick`, and resolved snapshot identity when one anchored the request; conflict markers should surface open disagreement, supersession lineage, override reason, and omitted-sibling notes when applicable
- CLI, daemon or JSON-RPC, and MCP surfaces may format explanations differently for humans, but the machine-readable field families should stay equivalent across interfaces, including `route_summary`, `historical_context`, `result_reasons`, `omitted_summary`, `policy_summary`, `provenance_summary`, `freshness_markers`, `conflict_markers`, and `trace_stages` when full traces are requested
- intent-routed `ask` surfaces must preserve the chosen or overridden intent class, classifier confidence, and any low-confidence fallback or safer-route downgrade in machine-readable explanation metadata; these route-plan changes are explanation facts, not human-only formatting details

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
