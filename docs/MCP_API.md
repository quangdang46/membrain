# membrain — MCP API Reference

> Canonical source: PLAN.md Section 34 (MCP Contract) and Section 9 (MCP Tools).
> Feature-specific tools: PLAN.md Sections 46–47.

## Global Design Rules

1. Every MCP tool preserves namespace and policy context
2. Never bypass governance checks
3. Return enough metadata for explainability
4. Distinguish user error, policy denial, and internal failure
5. Preserve idempotency where practical
6. Expose stable machine-readable outputs for automation
7. Keep request-path work bounded and measurable rather than hiding unbounded work behind tool wrappers
8. Treat provenance, lineage, and policy context as required execution inputs, not optional diagnostics
9. Shared operations exposed through CLI, daemon/JSON-RPC, and MCP should preserve the same underlying request and outcome semantics even when their human presentation differs

## Common Request Envelope

Every MCP request carries:

| Field | Required | Description |
|-------|----------|-------------|
| `namespace` | usually | Requested namespace scope; may be omitted only when the wrapper can bind one deterministic default before policy or retrieval work |
| `workspace_id` | if applicable | Workspace identifier |
| `agent_id` | if applicable | Calling agent identity |
| `session_id` | if applicable | Active session |
| `task_id` | if applicable | Active task or governing goal/work-item handle |
| `request_id` | yes | Idempotency/tracing key |
| `policy_context` | yes | Policy hints |
| `time_budget_ms` | optional | Retrieval time budget |

Context-envelope rules:
- These fields are execution context, not authorization shortcuts; namespace and policy checks still gate every tool.
- Every request resolves to exactly one effective namespace before storage, index, graph, or retrieval work begins.
- If `namespace` is omitted and no deterministic caller-scoped default exists, fail as validation error before candidate generation or writes.
- Malformed or unknown namespaces are validation failures; cross-namespace attempts without explicit approved sharing semantics are policy denials.
- `task_id` is the primary request-scope handle for explicit goals, beads, tickets, and other work items; additional many-to-many goal links should be represented in persisted memories through relations/lineage rather than ad hoc request fields.
- If a caller omits a field because it is unknown or not applicable, downstream responses and inspect/explain surfaces must preserve the distinction between absent and redacted.
- Servers may infer omitted context only from bounded transport metadata such as authenticated caller identity, session binding, scheduler ownership, or stable source mapping; they must not infer scope from free-form prompt text.
- Flags such as `include_public` may widen recall only to explicitly shareable surfaces allowed by policy; they do not bypass namespace ACLs or private visibility.

## Common Response Envelope

| Field | Description |
|-------|-------------|
| `ok` | Success boolean |
| `request_id` | Echo of request ID |
| `namespace` | Echo of the effective namespace after deterministic binding |
| `result` | Tool-specific payload |
| `error_kind` | Optional machine-readable failure family when `ok=false`: `validation_failure`, `policy_denied`, or `internal_failure` |
| `warnings` | Non-fatal issues |
| `policy_filters_applied` | Which policies affected the result |
| `explain_handle` | Handle or embedded explanation |
| `metrics` | Latency, candidate counts, and any cache/degraded-mode counters needed to explain bounded serving; when namespace enforcement depends on bypass, denial, or degraded fallback, the machine-readable evidence may live in `metrics`, `warnings`, or the explanation referenced by `explain_handle`, but it must remain inspectable |

CLI JSON output for equivalent operations may package these fields differently for command ergonomics, but it should preserve the same effective namespace, policy, explanation, warning, and degraded-serving meaning rather than inventing a separate semantic contract.

## Daemon JSON-RPC Contract

- The daemon transport is Unix socket plus JSON-RPC 2.0. Socket discovery, pid/socket lifecycle, and CLI fallback behavior stay owned by the daemon lifecycle contract, but once connected the daemon interface exposes the same underlying bounded operations as the CLI and MCP surfaces.
- Each daemon request uses the JSON-RPC 2.0 envelope: `jsonrpc`, `id`, `method`, and `params`. `params` carries the procedure-specific request plus any required common context-envelope fields from this document; transport wrappers must not infer missing namespace or policy scope from free-form prompt text.
- Canonical daemon procedure families mirror the stable CLI families: encode/intake (`remember`, `observe`, `import`), recall/query (`recall`, `ask`, `budget`), inspect/audit (`inspect`, `why`, `beliefs`, `audit`, `stats`, `health`, `doctor`), maintenance/admin (`repair`, `consolidate`, `compress`, `dream`, `export`), and history/namespace/change procedures (`timeline`, `snapshot`, `diff`, `share`, `unshare`, `forget`, `strengthen`, `update`, `fork`, `merge`, `namespace`). Method spelling may differ from MCP tool names, but it must not change the underlying request, policy, or outcome semantics.
- A successful protocol-level daemon response returns JSON-RPC `result` whose payload preserves the common response-envelope semantics defined above, including `ok`, `request_id`, effective `namespace`, `warnings`, `policy_filters_applied`, `explain_handle`, and `metrics` when those fields materially affect the outcome.
- Protocol- or dispatch-level failures use the JSON-RPC `error` object and the standard JSON-RPC 2.0 code families for parse failure, invalid request, method not found, invalid params, or internal transport/dispatch error. Once the daemon has accepted a recognized membrain procedure, domain-level failure families should stay machine-readable through the membrain response payload via `error_kind` (`validation_failure`, `policy_denied`, `internal_failure`) so parity with CLI and MCP is preserved.
- Batch requests may be supported only as independent bounded operations. Each element resolves namespace, policy, candidate budgets, and degraded warnings independently; one failing item must not widen scope or silently contaminate sibling outcomes.
- JSON-RPC notifications are optional and should be reserved for explicitly documented fire-and-forget procedures. Core retrieval, inspect, and mutating procedures should use request/response form so callers can observe validation failures, policy denials, warnings, degraded mode, and explanation handles.
- Concurrency is a daemon throughput property, not a semantic distinction. Concurrent requests must preserve per-call namespace isolation, bounded-work guarantees, and explainability exactly as if the same procedures were executed one at a time.
- Detailed remediation wording, user-facing recovery hints, and richer failure taxonomies remain owned by `mb-1hw.9`; this section only fixes the daemon transport contract and where machine-readable failure families live.

---

## Core Tools

### `memory_put`

Ingest a new memory item.

**Inputs**: namespace, memory_type, content/payload, source metadata, optional context bindings, optional emotional annotations, optional salience/tags/entity_refs/relation_refs, retention hints

**Outputs**: memory_id, chosen tier, validation outcome, routing reason, provenance summary, write-path summary (duplicate-family route, bounded similarity evidence, interference disposition), deferred enrichment handle

**Rules**:
- Writes validate policy first
- Contradictory writes must not silently overwrite — must emit conflict metadata
- Encode must preserve enough write-side metadata for later inspect/explain to distinguish caller-provided versus bounded-derived context, emotional annotations, advisory tags, provisional salience inputs, duplicate-family classification, and interference-lane participation
- `write-path summary` must keep duplicate-family route outcome, bounded shortlist evidence such as nearest-neighbor similarity or candidates inspected, and interference apply/skip/defer state separately inspectable rather than collapsing them into one opaque duplicate flag
- Supports `visibility` and `namespace_id` for cross-agent sharing (Feature 9)

### `memory_get`

Retrieve a memory item by ID.

**Outputs**: typed memory view, provenance fields, current tier, policy-redacted fields, machine-readable conflict metadata when present (`conflict_state`, `conflict_record_ids`, `belief_chain_id`, `superseded_by`)

**Rules**: exact lookup does not bypass redaction or namespace checks; exact lookup still preserves contradiction and supersession state instead of flattening to one silent winner

### `memory_search`

Bounded search over indexes, tags, entities, time ranges, or semantic hints.

**Inputs**: query string or structured filters, namespace/scope, memory types, session/task/goal filters, result budget

**Outputs**: candidate list, filter summary, index families used, omitted-result note if capped, conflict-state summaries for returned items when applicable

**Rules**:
- namespace, visibility, and policy pruning happen before index fanout or expensive retrieval work
- denied cross-namespace requests must fail without leaking protected candidate counts, record existence, or workspace/session detail

### `memory_recall`

Task-oriented bounded retrieval for context construction. The primary retrieval tool.

**Canonical request model**:
- `query_text` or task text as the primary cue
- optional `context_text`
- `mode`
- `result_budget`, `token_budget`, or `time_budget_ms`
- `effort`
- `explain`
- `namespace` plus optional `include_public`
- optional scoped filters (`workspace_id`, `agent_id`, `session_id`, `task_id`, `memory_kinds`, `era_id`, `as_of_tick`, `at_snapshot`, `min_strength`, `min_confidence`, `show_decaying`, `mood_congruent`)
- optional `like_id` / `unlike_id` query-by-example cues
- optional `graph_mode` and `cold_tier`

When `at_snapshot` is present, the request becomes bounded historical inspection rather than live recall: later-created memories are excluded, time-sensitive strength or freshness is recomputed against the snapshot tick, and the result must disclose partial/degraded historical visibility if current retention, policy, or repair state prevents a full reconstruction of what was once visible.

**Outputs**: ranked evidence set, score summaries, graph-assistance and associative-context summaries when applicable, contradiction markers, decaying-soon markers, packaging metadata for prompt construction, and explain metadata sufficient to summarize route choice, omitted-result reasons, provenance, freshness, cache or degraded-serving behavior, and full trace stages when requested

**Rules**:
- `query_text` may be omitted only when `like_id` or `unlike_id` provides the primary cue
- effective namespace and sharing scope must be resolved before candidate generation begins
- omitted `namespace` is valid only when one deterministic default can be bound from authenticated context or stable session/job ownership
- `include_public` widens only to explicitly shareable surfaces permitted by policy
- denied or redacted namespace filters must remain inspectable without disclosing protected record existence or payload details
- `graph_mode` and `cold_tier` may tune routing, but they must not bypass hard graph caps, trigger pre-cut cold payload fetch, or override policy denial/redaction behavior
- when graph assistance contributes, the response must preserve which returned memories entered directly, which were introduced by bounded graph expansion, whether graph support changed ranking versus only adding supporting context, and what associative context was omitted by caps or policy
- incompatible time scopes or malformed request knobs are validation failures, not precedence guesses

**Conflict contract**:
- unresolved conflicts remain queryable directly rather than requiring inference from free-form text
- supersession marks the older memory as preserved but non-default for normal packaging; it does not erase the losing evidence
- authoritative override may change the preferred operational answer, but the response must retain the override reason plus the losing evidence handle
- if result caps suppress some conflicting siblings, the response must indicate omission rather than implying consensus

**Extended options** (from features):
- `like_id` / `unlike_id` — query-by-example (Feature 3)
- `min_confidence` — confidence filter (Feature 7)
- `era_id` — temporal era filter (Feature 5)
- `at_snapshot` — time travel recall (Feature 12)
- `namespace` / `include_public` — cross-agent scope with one effective namespace plus optional approved widening (Feature 9)
- `mood_congruent` — emotional boost (Feature 18)

### `memory_link`

Create or update explicit relations between memories, entities, or goals.

**Rules**: links require namespace compatibility and policy approval; link provenance is stored; graph repair possible after creation

### `memory_inspect`

Retrieve diagnostic and structural details about a memory.

**Exposes**: current tier, lineage, policy flags, lifecycle state, index presence, graph neighborhood summary, decay/retention info, cache-related routing metadata when relevant, provenance summary, freshness markers, duplicate-family or interference-maintenance summaries when present, and linked contradiction state (`conflict_state`, related `ConflictRecord` handles, preferred memory if resolved)

### `memory_explain`

Explain why a memory was stored, routed, recalled, ranked, filtered, demoted, or forgotten.

**Explains**: routing signals, ranking components, cache family/event/reason metadata when cache behavior materially affected the route, policy filters, exclusion or omission reasons, lineage ancestry, consolidation ancestry, provenance summary, freshness markers, trace stages when full detail is requested, which context/emotional/tagging inputs were explicit versus bounded-derived, whether duplicate-family or interference lanes fired/bypassed/deferred, forgetting/demotion reasons, and any contradiction resolution path (open conflict, supersession, or authoritative override)

### `memory_consolidate`

Trigger or schedule consolidation workloads.

**Supports**: session-scoped, task-scoped, duplicate collapse, fact extraction, summary generation, skill extraction

**Rules**: preserve evidence, emit artifact IDs for generated summaries/facts, safe for bounded background windows

### `memory_pin`

Raise retention protection or bypass normal forgetting/demotion.

**Rules**: pinning is policy-relevant and auditable, must not bypass redaction/governance, reason is recorded

### `memory_forget`

Controlled forgetting: suppress, decay, demote, compact, summarize, archive, redact, soft/hard delete.

**Rules**: distinguish utility-driven forgetting from compliance deletion; preserve lineage; enforce retention and legal-hold denial paths explicitly; never remove last authoritative evidence unless policy allows

### `memory_repair`

Run or schedule repair: indexes, graph, lineage, summaries, shards.

**Rules**: durable evidence wins over derived state; prior durable state stays authoritative while repair is in flight; output what was fixed/rebuilt/unresolved; partial-fidelity repair records explicit loss; retry-budget exhaustion or escalation must remain visible when repair cannot complete automatically

**Should return**:
- repaired surface kind (`index`, `graph`, `lineage`, `cache`, `summary`, `shard`)
- authoritative input set used for rebuild
- namespace or shard scope touched
- unresolved items still queued for repair
- prior-state, stale-result, or degraded-serving markers when they materially affected the repair window
- explicit loss records when only degraded fidelity could be restored

---

## Feature-Specific Tools

### `ask(query, explain_intent?)` — Feature 20

Auto-classifies query intent and routes to optimal recall config. The recommended primary tool for agents.

**Returns**: `{ intent, intent_confidence, result: RetrievalResult, formatted_response }`

### `dream()` — Feature 1

Trigger offline synthesis cycle.

**Returns**: `{ links_created, engrams_merged, last_run_tick }`

### `belief_history(query)` — Feature 2

Returns the inspectable belief chain for a topic or query without rewriting the underlying contradiction/supersession model.

**Returns**: `{ chain_id, preferred_memory_id, resolution_state, versions: [{ id, content, tick, belief_version, superseded_by, conflict_state }], conflicts }`

**Rules**:
- this is a later-stage trust/introspection surface built on top of the already-canonical contradiction, lineage, and supersession records
- history views must preserve whether a chain is unresolved, coexisting, superseded, or under authoritative override rather than implying that all disagreement collapsed into one clean winner
- supersession-aware retrieval may prefer the current operational answer for default packaging, but belief-history results must keep older versions and losing evidence inspectable
- the tool should stay bounded to one effective namespace and one chain/topic at a time unless a later contract explicitly widens that scope

### `context_budget(token_budget, current_context?, working_memory_ids?, format?)` — Feature 4

Ranked, deduplicated, ready-to-inject memory list that fits within token budget.

**Returns**: `{ injections: [{memory_id, content, utility_score, token_count, reason}], tokens_used }`

### `timeline()` — Feature 5

**Returns**: `{ landmarks: [{id, label, era_start, era_end, memory_count}] }`

### `observe(content, context?, chunk_size?, source_label?)` — Feature 6

Segment content into memories via topic boundary detection.

**Returns**: `{ memories_created, topic_shifts }`

### `uncertain(top_k?)` — Feature 7

**Returns**: `{ memories: [{id, content, confidence, reconsolidation_count}] }`

### `skills()` / `extract_skills()` — Feature 8

**Returns**: `{ procedures: [{id, content, source_engram_id, confidence, member_count}] }`

### `share(id, namespace_id)` — Feature 9

Share a memory within a namespace for cross-agent access.

### `health()` — Feature 10

**Returns**: `BrainHealthReport` as JSON (tiers, quality, engrams, signals, activity)

### `why(id)` — Feature 11

Trace causal chain to root evidence.

**Returns**: `{ chain: [{memory_id, content, link_type, tick, confidence}], depth, all_roots_valid }`

### `invalidate(id, dry_run?)` — Feature 11

Cascade confidence penalty from invalidated root.

**Returns**: `{ memories_penalized, avg_confidence_delta }`

### `snapshot(name, note?)` / `list_snapshots()` — Feature 12

Creates and enumerates named historical inspection anchors.

**Returns**:
- `snapshot(name, note?)` → `{ name, tick, note, memory_count, namespace }`
- `list_snapshots()` → `{ snapshots: [{ name, tick, note, memory_count, namespace }] }`

**Rules**:
- snapshot creation records a namespace-scoped tick checkpoint plus compact metadata; it does not clone payloads or create a second authoritative store
- snapshot listing returns metadata only and must remain bounded enough for operator and automation use
- deleting a snapshot removes the handle for future historical inspection, but must respect maintenance and rollback policy that may require keeping the last restorable anchor for a scope
- a later `memory_recall` or equivalent tool using `at_snapshot` must exclude memories created after the snapshot tick and recompute time-sensitive strength/freshness against that historical tick
- snapshot-scoped inspection remains subject to current policy, redaction, and retained-authoritative-evidence limits; when later retention, repair loss, or policy changes prevent full reconstruction, the response should surface partial or degraded historical inspection rather than imply a perfect restore

### `hot_paths(top_n?)` / `dead_zones(min_age_ticks?)` — Feature 13

**Returns**: hot/dead zone entries with retrieve counts, scores, and age

### `diff(since, until?, top_n?)` — Feature 14

**Returns**: `BrainDiff` — new memories, strengthened, weakened, archived, conflicts resolved, new engrams

### `fork(name, parent_namespace?, inherit?, note?)` — Feature 15

**Returns**: `{ name, forked_at_tick, inherited_count }`

### `merge_fork(fork_name, target_namespace, conflict_strategy?, dry_run?)` — Feature 15

**Returns**: `MergeReport` with merge/conflict counts

### `compress(dry_run?)` / `schemas(top_n?)` — Feature 17

**Returns**: compression report or schema list

### `mood_history(since_tick?, namespace_id?)` — Feature 18

**Returns**: `{ timeline: [{tick_start, tick_end, avg_valence, avg_arousal, state, memory_count}] }`

### `audit(memory_id?, since_tick?, op?, limit?)` — Feature 19

**Returns**: `{ entries: [{op, memory_id, tick, before_strength, after_strength, triggered_by, note}] }`
