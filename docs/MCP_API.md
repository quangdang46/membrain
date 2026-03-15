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

## Common Request Envelope

Every MCP request carries:

| Field | Required | Description |
|-------|----------|-------------|
| `namespace` | yes | Namespace scope |
| `workspace_id` | if applicable | Workspace identifier |
| `agent_id` | if applicable | Calling agent identity |
| `session_id` | if applicable | Active session |
| `task_id` | if applicable | Active task or governing goal/work-item handle |
| `request_id` | yes | Idempotency/tracing key |
| `policy_context` | yes | Policy hints |
| `time_budget_ms` | optional | Retrieval time budget |

Context-envelope rules:
- These fields are execution context, not authorization shortcuts; namespace and policy checks still gate every tool.
- `task_id` is the primary request-scope handle for explicit goals, beads, tickets, and other work items; additional many-to-many goal links should be represented in persisted memories through relations/lineage rather than ad hoc request fields.
- If a caller omits a field because it is unknown or not applicable, downstream responses and inspect/explain surfaces must preserve the distinction between absent and redacted.
- Servers may infer omitted context only from bounded transport metadata such as authenticated caller identity, session binding, scheduler ownership, or stable source mapping; they must not infer scope from free-form prompt text.

## Common Response Envelope

| Field | Description |
|-------|-------------|
| `ok` | Success boolean |
| `request_id` | Echo of request ID |
| `namespace` | Echo of namespace |
| `result` | Tool-specific payload |
| `warnings` | Non-fatal issues |
| `policy_filters_applied` | Which policies affected the result |
| `explain_handle` | Handle or embedded explanation |
| `metrics` | Latency, candidate counts |

---

## Core Tools

### `memory_put`

Ingest a new memory item.

**Inputs**: namespace, memory_type, content/payload, source metadata, optional salience/tags/entity_refs/relation_refs, retention hints

**Outputs**: memory_id, chosen tier, validation outcome, routing reason, deferred enrichment handle

**Rules**:
- Writes validate policy first
- Contradictory writes must not silently overwrite — must emit conflict metadata
- Supports `visibility` and `namespace_id` for cross-agent sharing (Feature 9)

### `memory_get`

Retrieve a memory item by ID.

**Outputs**: typed memory view, provenance fields, current tier, policy-redacted fields, machine-readable conflict metadata when present (`conflict_state`, `conflict_record_ids`, `belief_chain_id`, `superseded_by`)

**Rules**: exact lookup does not bypass redaction or namespace checks; exact lookup still preserves contradiction and supersession state instead of flattening to one silent winner

### `memory_search`

Bounded search over indexes, tags, entities, time ranges, or semantic hints.

**Inputs**: query string or structured filters, namespace/scope, memory types, session/task/goal filters, result budget

**Outputs**: candidate list, filter summary, index families used, omitted-result note if capped, conflict-state summaries for returned items when applicable

### `memory_recall`

Task-oriented bounded retrieval for context construction. The primary retrieval tool.

**Inputs**: task/goal description, retrieval mode hints, token budget or result budget, namespace/actor context

**Outputs**: ranked evidence set, score summaries, contradiction markers, decaying-soon markers, packaging metadata for prompt construction

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
- `namespace_id` / `include_public` — cross-agent (Feature 9)
- `mood_congruent` — emotional boost (Feature 18)

### `memory_link`

Create or update explicit relations between memories, entities, or goals.

**Rules**: links require namespace compatibility and policy approval; link provenance is stored; graph repair possible after creation

### `memory_inspect`

Retrieve diagnostic and structural details about a memory.

**Exposes**: current tier, lineage, policy flags, lifecycle state, index presence, graph neighborhood summary, decay/retention info, and linked contradiction state (`conflict_state`, related `ConflictRecord` handles, preferred memory if resolved)

### `memory_explain`

Explain why a memory was stored, routed, recalled, ranked, filtered, demoted, or forgotten.

**Explains**: routing signals, ranking components, policy filters, lineage ancestry, consolidation ancestry, forgetting/demotion reasons, and any contradiction resolution path (open conflict, supersession, or authoritative override)

### `memory_consolidate`

Trigger or schedule consolidation workloads.

**Supports**: session-scoped, task-scoped, duplicate collapse, fact extraction, summary generation, skill extraction

**Rules**: preserve evidence, emit artifact IDs for generated summaries/facts, safe for bounded background windows

### `memory_pin`

Raise retention protection or bypass normal forgetting/demotion.

**Rules**: pinning is policy-relevant and auditable, must not bypass redaction/governance, reason is recorded

### `memory_forget`

Controlled forgetting: suppress, decay, demote, compact, summarize, archive, redact, soft/hard delete.

**Rules**: distinguish utility-driven forgetting from compliance deletion; preserve lineage; never remove last authoritative evidence unless policy allows

### `memory_repair`

Run or schedule repair: indexes, graph, lineage, summaries, shards.

**Rules**: durable evidence wins over derived state; output what was fixed/rebuilt/unresolved; partial-fidelity repair records explicit loss

**Should return**:
- repaired surface kind (`index`, `graph`, `lineage`, `cache`, `summary`, `shard`)
- authoritative input set used for rebuild
- namespace or shard scope touched
- unresolved items still queued for repair
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

**Returns**: `{ chain_id, versions: [{id, content, tick, superseded_by}], conflicts }`

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

**Returns**: `{ name, tick, memory_count }`

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
