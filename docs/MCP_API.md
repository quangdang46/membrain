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
| `error_kind` | Optional machine-readable failure family when `ok=false`: `validation_failure`, `policy_denied`, `unsupported_feature`, `transient_failure`, `timeout_failure`, `corruption_failure`, or `internal_failure` |
| `retryable` | Boolean indicating whether the caller may safely retry the operation (`transient_failure` and `timeout_failure` are retryable; validation, policy, unsupported, corruption, and internal failures are not) |
| `partial_success` | Boolean indicating whether the operation produced a partial result (bounded by budget, time, or candidate limits) rather than a complete response |
| `remediation` | Optional machine-readable recovery guidance with a short summary plus stable next-step hints such as `fix_request`, `change_scope`, `check_health`, `retry_with_backoff`, `retry_with_higher_budget`, `run_doctor`, or `run_repair` |
| `availability` | Optional machine-readable availability summary when the affected scope is degraded, read-only, or partially unavailable |
| `warnings` | Non-fatal issues |
| `safeguard` | Optional shared safeguard object for risky or mutating operations that need preview / blocked / degraded / rejected / destructive-action semantics |

#### Error Kind Mappings

| Error Kind | Retryable? | Description | Minimum remediation |
|-------------|------------|-------------|---------------------|
| `validation_failure` | No | Request was malformed or violated a required constraint. Caller must fix the request before retrying. | Return `remediation.next_steps` including `fix_request`. |
| `policy_denied` | No | Governance or namespace policy blocked the operation. Changing policy or namespace may allow retry. | Return `remediation.next_steps` including `change_scope` or equivalent policy-approval guidance. |
| `unsupported_feature` | No | The requested tool or capability is not available in this build, deployment, or maturity set. | Return `remediation.next_steps` including `check_health`. |
| `transient_failure` | Yes | Temporary condition (network, resource contention, temporary unavailability). Caller may retry after backoff. | Return `remediation.next_steps` including `retry_with_backoff`. |
| `timeout_failure` | Yes | Operation exceeded its time budget. May be retried with increased budget or different effort level. | Return `remediation.next_steps` including `retry_with_higher_budget` when that distinction is known. |
| `corruption_failure` | No | Corruption or unreadable canonical/derived state was detected and safe serving or mutation must stop or degrade. | Return `remediation.next_steps` including `run_doctor` and, when applicable, `run_repair`. |
| `internal_failure` | No | Internal invariant violation or other non-retryable system failure occurred. | Return `remediation.next_steps` including inspection/repair guidance rather than blind retry. |

#### Partial-Result Semantics

When `partial_success=true` and `ok=true`, the operation succeeded but produced an incomplete response due to:

- **Budget exhaustion** - result_budget, token_budget, or time_budget_ms was reached
- **Candidate caps** - retrieval returned fewer candidates than requested (e.g., bounded graph expansion)
- **Scope narrowing** - partial namespace coverage due to policy constraints or degraded mode
- **Timeout with fallback** - operation timed out but returned a degraded or cached result

The response must still include `request_id`, effective `namespace`, and descriptive `metrics` so automation can distinguish between complete success and gracefully degraded partial outcomes.

When degraded or repair-aware serving changes what can still be queried or mutated, the response should also expose an `availability` object aligned with `docs/OPERATIONS.md`, including at minimum:
- `posture` — `full`, `degraded`, `read_only`, or `offline`
- `query_capabilities` — which read paths remain available
- `mutation_capabilities` — which writes remain available, blocked, or forced into preview-only handling
- `degraded_reasons` — machine-readable reasons such as `graph_unavailable`, `index_bypassed`, `cache_invalidated`, `repair_in_flight`, or `authoritative_input_unreadable`
- `recovery_conditions` — the checks or repairs required to clear degraded mode safely
- `policy_filters_applied` should still record which policies materially shaped the result

#### Idempotency Expectations

| Tool | Idempotent? | Notes |
|------|--------------|-------|
| `memory_put` | **Yes, if same content** | Duplicate content returns same memory_id. Subsequent puts with different content create new memories. |
| `memory_get` | **Yes** | Read operation is always idempotent. |
| `memory_search` | **Yes** | Read operation is always idempotent. |
| `memory_recall` | **Yes** | Bounded retrieval is idempotent for same inputs. |
| `memory_link` | **No** | Creating a new link adds state; must check for duplicates if idempotency needed. |
| `memory_inspect` | **Yes** | Read operation is always idempotent. |
| `memory_explain` | **Yes** | Read operation is always idempotent. |
| `memory_consolidate` | **No** | May generate new summaries/facts. |
| `memory_pin` | **No** | Adding a pin changes state; removing and re-adding is a cycle. |
| `memory_forget` | **No** | Removing or demoting memories changes state. |
| `memory_repair` | **Context-dependent** | Idempotent if called with same inputs while repair window is stable. |
| `stats()` | **Yes** | Read-only operator summary is idempotent for the same visible scope. |
| `doctor()` | **Yes** | Read-only diagnostics are idempotent for the same visible scope and generation. |
| `ask()` | **Yes** | Same query produces consistent retrieval. |
| `dream()` | **No** | May generate new engrams/summaries. |
| `belief_history()` | **Yes** | Read operation is always idempotent. |
| `context_budget()` | **Yes** | Same budget produces consistent results. |
| `timeline()` | **Yes** | Read operation is always idempotent. |
| `observe()` | **Yes, if same content** | Duplicate content returns same memory_id. |
| `uncertain()` | **Yes** | Same query produces consistent results. |
| `skills()` | **No** | May extract/update skill procedures. |
| `share()` | **Yes, if not already shared** | Re-sharing same memory is idempotent. |
| `health()` | **Yes** | Read operation is always idempotent. |
| `audit()` | **Yes** | Read-only history inspection is idempotent for the same visible scope. |
| `why()` | **Yes** | Read operation is always idempotent. |
| `invalidate()` | **No** | Cascades confidence changes. |
| `snapshot()` | **Yes, if not exists** | Re-creating snapshot with same name returns tick. |
| `list_snapshots()` | **Yes** | Read operation is always idempotent. |
| `hot_paths()` | **Yes** | Read operation is always idempotent. |
| `dead_zones()` | **Yes** | Read operation is always idempotent. |
| `diff()` | **Yes** | Read operation is always idempotent. |
| `fork()` | **Yes, if not exists** | Re-forking returns same fork. |
| `merge_fork()` | **No** | Merge changes authoritative state. |
| `compress()` | **No** | Compression changes memory representations. |
| `export()` | **Yes** | Same export produces consistent results. |
| `import()` | **Context-dependent** | Idempotent if same content (returns same memory_ids). |

**Note:** "Context-dependent" means idempotency depends on whether underlying durable state changes between calls. Automation clients should store returned IDs and use them for deduplication if strict idempotency is required.

---
| `explain_handle` | Handle or embedded explanation |
| `metrics` | Latency, candidate counts, and any cache/degraded-mode counters needed to explain bounded serving; when namespace enforcement depends on bypass, denial, or degraded fallback, the machine-readable evidence may live in `metrics`, `warnings`, or the explanation referenced by `explain_handle`, but it must remain inspectable |

When an operation returns embedded explanation rather than only an `explain_handle`, the machine-readable explanation contract should reuse the canonical field families from `docs/RETRIEVAL.md` where relevant rather than inventing per-tool envelopes. The stable families are `route_summary`, `result_reasons`, `omitted_summary`, `policy_summary`, `provenance_summary`, `freshness_markers`, `conflict_markers`, and `trace_stages` when full routing detail is requested.

For recall-facing operations, the tool-specific `result` payload should reuse one canonical `RetrievalResult` envelope rather than inventing separate MCP-only answer shapes. That shared object carries `outcome_class`, bounded `evidence_pack`, optional `action_pack`, omission/deferred-payload state, policy/provenance/freshness/conflict summaries, packaging metadata, and either embedded explanation families or an `explain_handle`.

CLI JSON output for equivalent operations may package these fields differently for command ergonomics, but it should preserve the same effective namespace, policy, explanation, warning, and degraded-serving meaning rather than inventing a separate semantic contract.

For risky or mutating operations whose blast radius can rewrite authoritative state, widen namespace scope, emit irreversible-loss records, or require high-stakes action gating, MCP responses should also reuse the shared safeguard contract from `docs/OPERATIONS.md`. That means preview, blocked, degraded, rejected, and accepted responses for those tools should expose the same machine-readable safeguard fields for `operation_class`, `preflight_state`, `affected_scope`, `impact_summary`, `blocked_reasons`, `preflight_checks`, `warnings`, `confidence_constraints`, `reversibility`, `confirmation`, and `audit`, even when the tool-specific `result` payload carries additional domain data.

Read-only operator and data-mobility surfaces such as `stats`, `health`, `doctor`, `audit`, `export`, and `import` should likewise preserve semantic parity with CLI and daemon/JSON-RPC around counters, warnings, remediation hints, availability posture, and data-manifest meaning instead of introducing MCP-only interpretations.

When the caller sees `safeguard.outcome_class=blocked`, the request is still structurally valid but is missing readiness prerequisites such as confirmation, snapshot/generation freshness, or a required maintenance condition. When the caller sees `error_kind=validation_failure` or `error_kind=policy_denied`, the request is rejected at the domain level and local confirmation would not make it acceptable.

## Daemon JSON-RPC Contract

- The daemon transport is Unix socket plus JSON-RPC 2.0. Socket discovery, pid/socket lifecycle, and CLI fallback behavior stay owned by the daemon lifecycle contract, but once connected the daemon interface exposes the same underlying bounded operations as the CLI and MCP surfaces.
- Each daemon request uses the JSON-RPC 2.0 envelope: `jsonrpc`, `id`, `method`, and `params`. `params` carries the procedure-specific request plus any required common context-envelope fields from this document; transport wrappers must not infer missing namespace or policy scope from free-form prompt text.
- Canonical daemon procedure families mirror the stable CLI families: encode/intake (`remember`, `observe`, `import`), recall/query (`recall`, `ask`, `budget`), inspect/audit (`inspect`, `why`, `beliefs`, `audit`, `stats`, `health`, `doctor`), maintenance/admin (`repair`, `consolidate`, `compress`, `dream`, `export`), and history/namespace/change procedures (`timeline`, `snapshot`, `diff`, `share`, `unshare`, `forget`, `strengthen`, `update`, `fork`, `merge`, `namespace`). Method spelling may differ from MCP tool names, but it must not change the underlying request, policy, or outcome semantics.
- A successful protocol-level daemon response returns JSON-RPC `result` whose payload preserves the common response-envelope semantics defined above, including `ok`, `request_id`, effective `namespace`, `warnings`, `policy_filters_applied`, `explain_handle`, `metrics`, and any `availability` or `remediation` fields when those materially affect the outcome.
- Protocol- or dispatch-level failures use the JSON-RPC `error` object and the standard JSON-RPC 2.0 code families for parse failure, invalid request, method not found, invalid params, or internal transport/dispatch error. Once the daemon has accepted a recognized membrain procedure, domain-level failure families should stay machine-readable through the membrain response payload via `error_kind` (`validation_failure`, `policy_denied`, `unsupported_feature`, `transient_failure`, `timeout_failure`, `corruption_failure`, or `internal_failure`) so parity with CLI and MCP is preserved, while the shared safeguard object distinguishes blocked readiness/confirmation failures from true rejected requests.
- Batch requests may be supported only as independent bounded operations. Each element resolves namespace, policy, candidate budgets, and degraded warnings independently; one failing item must not widen scope or silently contaminate sibling outcomes.
- JSON-RPC notifications are optional and should be reserved for explicitly documented fire-and-forget procedures. Core retrieval, inspect, and mutating procedures should use request/response form so callers can observe validation failures, policy denials, warnings, degraded mode, and explanation handles.
- Concurrency is a daemon throughput property, not a semantic distinction. Concurrent requests must preserve per-call namespace isolation, bounded-work guarantees, and explainability exactly as if the same procedures were executed one at a time.
- Detailed remediation wording may vary by transport, but the machine-readable failure families, `retryable` semantics, `remediation.next_steps`, and `availability` posture should stay semantically aligned across CLI, daemon/JSON-RPC, and MCP.

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

**Outputs**: candidate list, filter summary, index families used, omitted-result note if capped, conflict-state summaries for returned items when applicable. Returned candidates include per-item uncertainty fields defined in `docs/RETRIEVAL.md` section "Uncertainty surface contract".

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

**Outputs**: the canonical `RetrievalResult` envelope, including bounded `evidence_pack`, optional `action_pack`, `outcome_class`, score summaries, graph-assistance and associative-context summaries when applicable, contradiction markers, decaying-soon markers, deferred-payload state, packaging metadata for prompt construction, and explain metadata sufficient to summarize route choice, omitted-result reasons, provenance, freshness, cache or degraded-serving behavior, and full trace stages when requested

When explanation is embedded, `memory_recall` should preserve the same stable machine-readable families named in the canonical retrieval contract: `route_summary`, `result_reasons`, `omitted_summary`, `policy_summary`, `provenance_summary`, `freshness_markers`, `conflict_markers`, `trace_stages`, `uncertainty_markers` when full routing detail is requested.

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

**Exposes**: current tier, lineage, policy flags, lifecycle state, archive reason and restore eligibility when relevant, index presence, graph neighborhood summary, decay/retention info, cache-related routing metadata when relevant, provenance summary, freshness markers, duplicate-family or interference-maintenance summaries when present, degraded or partial-fidelity markers when archival recovery is incomplete, and linked contradiction state (`conflict_state`, related `ConflictRecord` handles, preferred memory if resolved)

When `memory_inspect` includes embedded explanation or route context, it should reuse the canonical families relevant to the inspected item rather than inventing a separate inspect-only schema, especially `policy_summary`, `provenance_summary`, `freshness_markers`, `conflict_markers`, and `trace_stages` or an `explain_handle` for deferred detail.

### `memory_explain`

Explain why a memory was stored, routed, recalled, ranked, filtered, demoted, or forgotten.

**Explains**: routing signals, ranking components, cache family/event/reason metadata when cache behavior materially affected the route, policy filters, exclusion or omission reasons, lineage ancestry, consolidation ancestry, provenance summary, freshness markers, trace stages when full detail is requested, which context/emotional/tagging inputs were explicit versus bounded-derived, whether duplicate-family or interference lanes fired/bypassed/deferred, forgetting/demotion reasons, archive-versus-delete reasoning, restore eligibility or denial basis when relevant, and any contradiction resolution path (open conflict, supersession, or authoritative override)

`memory_explain` is the canonical explicit explanation surface. Its machine-readable output should preserve the stable families from the retrieval contract rather than a bespoke per-tool trace shape: `route_summary`, `result_reasons`, `omitted_summary`, `policy_summary`, `provenance_summary`, `freshness_markers`, `conflict_markers`, and `trace_stages` when full routing detail is requested.

### `memory_consolidate`

Trigger or schedule consolidation workloads.

**Supports**: session-scoped, task-scoped, duplicate collapse, fact extraction, summary generation, skill extraction

**Rules**: preserve evidence, emit artifact IDs for generated summaries/facts, safe for bounded background windows

### `memory_pin`

Raise retention protection or bypass normal forgetting/demotion.

**Rules**: pinning is policy-relevant and auditable, must not bypass redaction/governance, reason is recorded

### `memory_forget`

Controlled forgetting: suppress, decay, demote, compact, summarize, archive, redact, soft/hard delete.

**Rules**: distinguish utility-driven forgetting from compliance deletion; preserve lineage; enforce retention and legal-hold denial paths explicitly; never remove last authoritative evidence unless policy allows; archive-by-default forgetting remains inspectable and recoverable only through explicit restore paths rather than implicit recall

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

### `stats()`

Return the bounded operator summary shared with CLI `membrain stats`.

**Returns**: aggregated storage, quality, performance, graph, and runtime counters such as tier counts/utilization, strength or confidence rollups, cache/recall hit rates, graph totals, current tick, and last consolidation when known.

**Rules**:
- `stats()` is read-only and must not trigger repair, warming, or other hidden mutation.
- MCP, daemon/JSON-RPC, and CLI `--json` should preserve the same counter meanings even if one surface renders them as a table or dashboard.
- When policy scope, historical anchors, or degraded serving make a counter unavailable, the response should expose warnings or `availability` state instead of silently fabricating zeros or dropping the field.

### `doctor()`

Diagnose corruption, stale derived state, and degraded-serving posture.

**Returns**: `{ checks: [{name, surface_kind, status, severity, affected_scope, note?, remediation?}], summary, availability? }`

**Rules**:
- `doctor()` is a read-only diagnostic surface; repair remains an explicit `memory_repair` or CLI `membrain repair ...` flow.
- Per-check machine-readable results should stay stable enough that CLI text, daemon/JSON-RPC, and MCP can agree on what failed, which scope is affected, and what remediation comes next.
- When authoritative inputs are unreadable or corruption blocks safe serving, the response should use the shared `error_kind`, `remediation`, and `availability` semantics rather than burying the state in prose only.

### `export(format?, include_cold?, include_archive?, kind?, min_strength?, at_snapshot?)`

Externalize memories within the caller's allowed scope.

**Returns**: `{ manifest: {format, namespace, included_stores, memory_count, redaction_summary?, warnings?}, export_ref? }`

**Rules**:
- `export()` is policy-aware externalization, not a namespace or redaction bypass.
- Transport wrappers may stream bytes, attach a file, or return an export handle, but they must preserve the same manifest meaning as CLI export.
- The manifest should record omission/redaction, selected format, included stores, and any historical or degraded anchor that materially shaped the exported view.

### `import(payload_ref, format?, dry_run?, merge?, kind?, source?)`

Import externalized memories through normal governed ingest.

**Returns**: `{ created, merged, skipped, rejected, warnings?, blocked_reasons?, manifest? }`

**Rules**:
- `import()` is governed ingest, not a raw storage bypass; imported records still pass namespace binding, validation, duplicate handling, provenance tagging, and bounded enrichment or repair rules.
- `dry_run` must return preview counts and blockers without writing.
- CLI, daemon/JSON-RPC, and MCP transports may package payload transfer differently, but they must preserve the same created/merged/skipped/rejected semantics and warning vocabulary.

---

## Feature-Specific Tools

### `ask(query, explain_intent?)` — Feature 20

Auto-classifies query intent and routes to optimal recall config. The recommended primary tool for agents.

**Returns**: `{ intent, intent_confidence, result: RetrievalResult, formatted_response }`

`result` is the same canonical retrieval/result object used by `memory_recall`, not an ask-specific schema. `formatted_response` is a rendering convenience layered on top of that shared `RetrievalResult`, whose machine-readable fields still carry the evidence-versus-action split and all omission, policy, freshness, conflict, deferred-payload, and explanation semantics.

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

**Returns**: `{ memories: [{id, content, confidence, uncertainty_score, uncertainty_interval?, corroboration_count, freshness_uncertainty?, contradiction_uncertainty?, missing_evidence_uncertainty?, known?, assumed?, uncertain?, missing?, change_my_mind_conditions?, reconsolidation_count}] }`

**Rules**:
- The uncertainty surface should expose more than a scalar rank. When the underlying evidence supports it, each entry should preserve machine-readable `known`, `assumed`, `uncertain`, `missing`, and `change_my_mind_conditions` fields so callers can tell what is established, what remains inferred, what evidence is absent, and what new evidence would most directly reduce uncertainty.
- High-stakes or action-oriented callers should prefer confidence intervals plus these uncertainty markers by default instead of treating them as optional diagnostics-only add-ons.

### `skills()` / `extract_skills()` — Feature 8

**Returns**: `{ procedures: [{id, content, source_engram_id, confidence, member_count}] }`

### `share(id, namespace_id)` — Feature 9

Share a memory within a namespace for cross-agent access.

### `health()` — Feature 10

**Returns**: `BrainHealthReport` as JSON (tiers, quality, engrams, signals, activity)

**Rules**:
- `health()` is the bounded machine-readable operator dashboard shared with CLI `membrain health`; transport-specific rendering may differ, but the underlying report semantics must not.
- The report should preserve tier and capacity counters, quality/conflict/uncertainty signals, runtime activity, and feature-availability facts strongly enough that automation can make the same decisions a human operator would make from the CLI dashboard.
- When policy scope, historical anchors, or degraded serving limit visibility, `health()` should return explicit warnings or `availability` state instead of silently fabricating a fully healthy view.

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

**Returns**:
- `compress(dry_run?)` → `{ schemas_created, episodes_compressed, candidate_clusters?, storage_reduction_pct, blocked_reasons?, related_run? }`
- `schemas(top_n?)` → `{ schemas: [{id, content, source_count, confidence, keywords, compressed_member_ids?}] }`

**Rules**:
- Schema compression is a later-stage, consolidation-adjacent follow-on rather than a prerequisite for the bounded core retrieval path.
- `compress()` should preserve repairable lineage by keeping source-memory links, `compressed_into` relationships, and audit-visible before/after effects inspectable rather than flattening many episodes into an opaque summary.
- Compression reduces or reroutes source-episode prominence; it does not silently hard-delete the underlying episodic evidence.
- `dry_run` should surface candidate cluster counts, affected source-memory estimates, expected schema creations, and any maintenance or policy blockers before mutation.
- `schemas()` should make the resulting abstract pattern memories inspectable as schema artifacts with source counts, confidence, and dominant keywords or equivalent summaries.

### `mood_history(since_tick?, namespace_id?)` — Feature 18

**Returns**: `{ timeline: [{tick_start, tick_end, avg_valence, avg_arousal, state, memory_count}] }`

### `audit(memory_id?, since_tick?, op?, limit?)` — Feature 19

**Returns**: `{ entries: [{op, memory_id, tick, before_strength, after_strength, triggered_by, note, namespace, redaction, related_snapshot, related_run}] }`

**Rules**:
- audit is a read-only forensic surface for memory and operation history; it does not authorize replay, restore, or mutation
- entries should preserve enough machine-readable context to connect visible state changes to the originating actor/source, effective namespace scope, and the relevant maintenance, repair, migration, compaction, or incident run when applicable
- snapshot references in audit output identify safety anchors or historical checkpoints associated with a change; they do not make snapshots the authoritative audit record
- policy-limited output should expose redaction explicitly instead of silently collapsing protected actor, namespace, or reason fields
- CLI `membrain audit`, daemon/JSON-RPC, and MCP `audit()` should stay semantically aligned about what an entry means, which correlation fields are mandatory, and how degraded or redacted history is surfaced.

### `memory_uncertain` — List low-uncertainty memories

List memories with uncertainty score above a threshold, for inspecting which memories have sparse evidence, poor corroboration, or are conflicted.

**Inputs**:
- `min_uncertainty` (optional): Include only memories with `uncertainty_score >= threshold`

---

## Feature-Specific Tools and Maturity Gating

Core MCP tools (`memory_put`, `memory_get`, `memory_search`, `memory_recall`, `memory_link`, `memory_inspect`, `memory_explain`, `memory_repair`, `stats()`, `doctor()`, `export()`, and `import()`) are always available and use the stable response envelope. Feature-specific tools extend core capabilities and may be gated by feature flags or maturity levels.

### Maturity States

| Maturity | Description | Client Behavior |
|----------|-------------|----------------|
| `experimental` | Early-stage feature for testing. May change without notice. Do not depend on in production workflows. |
| `guarded` | Feature is available but requires explicit opt-in or policy approval. Behavior is relatively stable but may evolve. |
| `stable` | Production-ready feature. Clients can rely on stable contract. |

### Feature Discovery

The `health()` tool returns a `BrainHealthReport` that includes available features. Clients should check feature availability before calling feature-specific tools.

### Degraded and Unsupported Behavior

When a feature-specific tool is called for an unavailable or gated feature:

| Situation | Response | Error Handling |
|-----------|---------|--------------|
| Feature not in `health()` report | Return `{ok: false, error_kind: "unsupported_feature"}` | Client should check `health()` first |
| Feature requires opt-in or policy | Return `{ok: false, error_kind: "policy_denied"}` | Client should not bypass governance |
| Feature available but user lacks permission | Return `{ok: false, error_kind: "policy_denied"}` | Use `policy_denied` consistently |
| Feature disabled via config | Return `{ok: false, error_kind: "validation_failure"}` | Distinguish from unsupported feature |

### Stability Guarantees

| Maturity | Stability Expectation |
|----------|-------------------|
| `stable` | Core tools and response envelope are stable. Feature-specific tools at `stable` maturity must not break core contracts. |
| `guarded` | May evolve but changes are bounded and documented. |
| `experimental` | No stability guarantee. |

### `query_by_example(example_id, explain_intent?, result_budget?, token_budget?, time_budget_ms?, effort?, explain?)` — Feature 3

Find memories similar to a provided example memory using vector similarity, enabling "find things like X" queries that are difficult to express with pure text.

**Returns**: `{ intent, intent_confidence, result: RetrievalResult, formatted_response }`

**Intent Recognition**:
- Classifies query as: find-similar, find-opposite, find-related, find-cause, find-context, find-application, find-definition
- Returns `intent` string for explicit handling by automation
- `intent_confidence` (0-0-1) indicates confidence in intent classification

**Search Behavior**:
- Uses example memory's embedding as query vector for ANN search
- Returns candidates ranked by similarity to example
- Respects namespace, policy, and standard retrieval constraints
- Result uses canonical `RetrievalResult` envelope

**Difference from `memory_search`**:
- `memory_search` uses text-based search across indexes
- `query_by_example` uses example-based vector search for finding conceptually similar memories
- Use `query_by_example` when user has a reference memory but lacks the right terminology or conceptual understanding

**Maturity**: Stable

---

## Core Contract Preservation Rules

1. **Core tools must remain stable** - Feature-specific tools must not change `error_kind` values, add required fields to the common envelope, or break established semantics.
2. **Core response envelope unchanged** - The `ok`, `request_id`, `namespace`, `result`, `error_kind`, `warnings`, `policy_filters_applied`, `explain_handle`, and `metrics` fields remain the common baseline for all tools, with optional `retryable`, `partial_success`, `remediation`, `availability`, and `safeguard` fields when the outcome needs them.
3. **Feature additions are additive** - Feature-specific tools add new fields via their specific response but must preserve the canonical envelope structure.
4. **Maturity is tool-level, not request-level** - A tool's maturity applies to all requests, not toggled per-call.
5. **Backward compatibility** - New stable tools must not break existing clients. Guarded tools should evolve through explicit versioning or maturity transitions.

### Feature Rollout and Deprecation

- Experimental tools may be introduced via `--experimental` flags or feature toggles.
- Promoting to `guarded` requires testing and documentation.
- Promoting to `stable` requires going through a maturity gate with feature freeze and deprecation notice.
- Deprecating a feature requires at least one major version cycle and documented migration path.
- `top_k`: Maximum number of memories to return

**Outputs**:
- `{ memories: [{id, content, confidence, uncertainty_score, uncertainty_interval?, corroboration_count, last_access_at, known?, assumed?, uncertain?, missing?, change_my_mind_conditions?}] }`

See uncertainty dimensions and scoring in `docs/RETRIEVAL.md` section "Uncertainty surface contract".
The same marker families should be available to `memory_search`/recall-facing responses when uncertainty is surfaced, especially on high-stakes or action-oriented paths.

---

### `memory_search` extended options

Add to existing optional scoped filters for `memory_search`:

- `min_uncertainty` (optional): Filter to only memories with uncertainty below threshold (useful for "high-confidence only" paths)
- `include_uncertainty_interval` (optional): If `true` and request is classified as high-stakes, include confidence interval bounds for each returned memory. Default depends on request classification; action-oriented, decision-oriented, and safety-critical paths should always include intervals.

**Note**: This extension preserves existing `min_confidence` filter. Uncertainty filtering complements confidence filtering rather than replacing it.

EOF'
