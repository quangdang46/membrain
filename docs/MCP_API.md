# membrain — MCP API Reference

> Canonical source: PLAN.md Section 34 (MCP Contract) and Section 9 (MCP Tools).
> Feature-specific tools: PLAN.md Sections 46–47.

## Global Design Rules

Status note:
- This document mixes the long-term MCP contract with the currently implemented runtime surfaces.
- The live daemon/MCP tool catalog is presently the bounded six-tool surface (`encode`, `recall`, `inspect`, `why`, `health`, `doctor`) plus `resources.list`, `resource.read`, `streams.list`, and `shutdown` on the transport side.
- The broader `memory_*`, feature-specific, and later-stage operator tools described below remain the canonical target contract unless a section explicitly says they are live today.
- Normal daemon/MCP recall now returns hydrated evidence on success; explicit degraded/fallback language below should be read as applying to no-hydrated-evidence, capped, or repair/degraded cases rather than the default success path.
- `membrain mcp` is a stdio transport adapter with process-local reuse only; daemon-owned repeated-request warm-runtime guarantees belong to the long-lived Unix-socket daemon.

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
| `policy_filters_applied` | Optional machine-readable summary of the policy families, sharing scope, retention or hold markers, and redaction decisions that materially shaped the visible outcome |
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

The table below is the canonical contract vocabulary for the full planned MCP surface. It is broader than the live MCP tool catalog advertised by the current runtime. For the currently exposed runtime tools, the relevant entries are `health()`, `doctor()`, `why()`, and the recall/inspect semantics that map onto `memory_recall` / `memory_inspect`.

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
| `share()` | **Yes, if same effective visibility** | Re-sharing the same memory into the same approved scope is idempotent. |
| `unshare()` | **Yes, if already private/non-shared** | Re-tightening the same visibility is idempotent. |
| `health()` | **Yes** | Read operation is always idempotent. |
| `audit()` | **Yes** | Read-only history inspection is idempotent for the same visible scope. |
| `why()` | **Yes** | Read operation is always idempotent. |
| `invalidate()` | **No** | Cascades confidence changes. |
| `snapshot()` | **Yes, if not exists** | Re-creating snapshot with same name returns tick. |
| `list_snapshots()` | **Yes** | Read operation is always idempotent. |
| `goal_state()` | **Yes** | Read operation is always idempotent. |
| `goal_pause()` | **Yes, if already paused at the same checkpoint** | Re-pausing the same dormant goal may return the existing checkpoint handle. |
| `goal_resume()` | **Context-dependent** | Re-resuming may return the same active checkpoint only when no newer checkpoint or state transition intervened. |
| `goal_abandon()` | **Yes, if already abandoned** | Re-abandoning should preserve the prior inactive state and audit/checkpoint handles. |
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

For predictive pre-recall work, MCP explain output must keep bypass and fallback behavior inspectable rather than treating prediction as invisible background behavior. At minimum, callers must be able to distinguish predictive trigger versus bypass, the stable bypass reason when prediction was skipped, and the stable fallback behavior that governed the bounded route after prediction was considered.

For recall-facing operations, the tool-specific `result` payload should reuse one canonical `RetrievalResult` envelope rather than inventing separate MCP-only answer shapes. That shared object carries `outcome_class`, bounded `evidence_pack`, optional `action_pack`, explicit `output_mode`, omission/deferred-payload state, policy/provenance/freshness/conflict summaries, packaging metadata, and either embedded explanation families or an `explain_handle`.

When MCP callers provide `mode` or `effort`, implementations may map those labels onto the dual-output packaging modes from Section 10.1: `strict`/`high` suppresses unsafe derived actions, `balanced`/`normal` preserves ordinary evidence-plus-action packaging, and `fast` keeps action suggestions available unless other policy gates remove them.

MCP regression coverage for future implementation beads must prove that accepted, partial, preview, blocked, degraded, and rejected retrieval outcomes preserve the same top-level field families and semantic meaning as CLI and daemon/JSON-RPC, including the sample accepted and partial/deferred/conflict-bearing shapes defined in `docs/RETRIEVAL.md`.

CLI JSON output for equivalent operations may package these fields differently for command ergonomics, but it should preserve the same effective namespace, policy, explanation, warning, and degraded-serving meaning rather than inventing a separate semantic contract.

For risky or mutating operations whose blast radius can rewrite authoritative state, widen namespace scope, emit irreversible-loss records, or require high-stakes action gating, MCP responses should also reuse the shared safeguard contract from `docs/OPERATIONS.md`. That means preview, blocked, degraded, rejected, and accepted responses for those tools should expose the same machine-readable safeguard fields for `operation_class`, `preflight_state`, `affected_scope`, `impact_summary`, `blocked_reasons`, `preflight_checks`, `warnings`, `confidence_constraints`, `reversibility`, `confirmation`, and `audit`, even when the tool-specific `result` payload carries additional domain data.

The minimum explicit preflight wrapper surface is `preflight.run`, `preflight.explain`, and `preflight.allow`. These wrapper names may contain dots even on MCP because they are transport labels for the shared safeguard contract, not separate wrapper-local semantics. Their request and response bodies must round-trip with the same machine-readable fields used by JSON-RPC and any equivalent CLI `preflight` command.

Read-only operator and data-mobility surfaces such as `stats`, `health`, `doctor`, `audit`, `export`, and `import` should likewise preserve semantic parity with CLI and daemon/JSON-RPC around counters, warnings, remediation hints, availability posture, and data-manifest meaning instead of introducing MCP-only interpretations.

When the caller sees `safeguard.outcome_class=blocked`, the request is still structurally valid but is missing readiness prerequisites such as confirmation, snapshot/generation freshness, or a required maintenance condition. When the caller sees `error_kind=validation_failure` or `error_kind=policy_denied`, the request is rejected at the domain level and local confirmation would not make it acceptable.

## Governance explain, audit, and parity contract

Governance-sensitive MCP responses must preserve inspectable evidence for denial, redaction, retention, legal-hold, approved sharing, and namespace-isolation outcomes instead of relying on transport-specific prose.

### Required explain fields

When a tool is denied, redacted, narrowed by retention or hold policy, or widened through approved sharing, the response must expose these machine-readable explain fields either inline or via `explain_handle`:

- `policy_summary.effective_namespace`
- `policy_summary.policy_family`
- `policy_summary.outcome_class`
- `policy_summary.blocked_stage`
- `policy_summary.redaction_fields`
- `policy_summary.retention_state`
- `policy_summary.sharing_scope`

These fields must preserve the same semantics as CLI JSON, daemon/JSON-RPC, IPC, inspect, explain, repair, and audit surfaces. MCP may format them differently, but it must not silently collapse them into generic warnings, empty results, or prose-only explanations.

### Required audit correlation

Every governance-sensitive MCP outcome must remain traceable into the authoritative audit trail. At minimum, responses and audit artifacts together must preserve:

- `request_id` or equivalent correlation handle
- `effective_namespace` and any source or target namespace involved in approved widening
- `policy_family`, `outcome_class`, and `blocked_stage`
- `retention_state` and hold markers when lifecycle policy shaped the result
- `redaction_summary` when fields or payload slices were intentionally hidden
- `related_run` or `related_snapshot` when repair, rebuild, restore, compaction, or incident handling materially shaped the outcome

### Cross-interface parity obligation

MCP is not allowed to invent wrapper-local governance semantics. For any scenario also exposed through CLI, daemon/JSON-RPC, IPC, inspect, explain, repair, or audit surfaces, the machine-readable governance outcome must stay parity-consistent across interfaces.

Minimum parity coverage for governance-sensitive work includes:

- validation failure versus policy denial
- policy denial versus redacted success
- approved widening versus same-namespace allow
- archival versus hard deletion
- ordinary retention pressure versus legal-hold or compliance-lock override
- namespace-isolation enforcement during degraded, cache-bypassed, repair, or background execution

Silent cross-surface divergence is a contract violation, not an acceptable transport difference.

## Easy Connection

The easiest current integration path is:

```bash
membrain mcp
```

This launches Membrain as a stdio MCP-style server so clients can spawn it directly instead of manually connecting to the Unix daemon socket.

This stdio path is a transport adapter, not the authoritative warm-runtime service. It may reuse state within the current process, but runtime-authority reporting must keep that scoped to local-process guarantees like `local_process_state` and `best_effort_same_process_reuse`; daemon-owned guarantees like `daemon_owned_runtime_state` and `repeated_request_warmth` belong to the long-lived Unix-socket daemon. The same rule applies to `embedder_runtime`: stdio may report process-local `not_loaded` → `loaded` or `warm` transitions, but only the daemon can claim long-lived repeated-request warm reuse as the authoritative runtime.

### Claude Code integration

For Claude Code, configure Membrain in the `mcpServers` section like this:

```json
{
  "mcpServers": {
    "membrain": {
      "command": "membrain",
      "args": ["mcp"]
    }
  }
}
```

If you want Claude Code to use a specific storage root instead of the default local state, pass `--db-path` too:

```json
{
  "mcpServers": {
    "membrain": {
      "command": "membrain",
      "args": ["mcp", "--db-path", "/path/to/state-root"]
    }
  }
}
```

A practical project-level `.claude/settings.json` can also combine Membrain MCP with Claude Code hooks:

```json
{
  "mcpServers": {
    "membrain": {
      "command": "membrain",
      "args": ["mcp"]
    }
  },
  "hooks": {
    "SessionStart": [
      {
        "matcher": "startup|resume",
        "hooks": [
          {
            "type": "command",
            "command": "bash -lc 'pgrep -f \"membrain-daemon\" >/dev/null || nohup membrain-daemon >/tmp/membrain-daemon.log 2>&1 &'"
          },
          {
            "type": "command",
            "command": "echo 'Membrain is available in this project. Prefer using Membrain MCP or CLI recall/inspect/why before guessing prior context. Local state lives under ~/.membrain by default.'"
          }
        ]
      }
    ],
    "UserPromptSubmit": [
      {
        "hooks": [
          {
            "type": "command",
            "command": "echo 'Membrain reminder: use memory tools for prior context, incidents, and reusable facts. Prefer `membrain recall`, `membrain inspect`, `membrain why`, or the Membrain MCP server when context may already exist.'"
          }
        ]
      }
    ]
  }
}
```

Manual smoke test before wiring a client:

```bash
python3 - <<'PY'
import json, subprocess
p = subprocess.Popen(['membrain', 'mcp'], stdin=subprocess.PIPE, stdout=subprocess.PIPE, text=True)
p.stdin.write(json.dumps({"jsonrpc":"2.0","id":"1","method":"resources.list","params":{}}) + "\n")
p.stdin.flush()
print(p.stdout.readline().strip())
p.stdin.write(json.dumps({"jsonrpc":"2.0","id":"2","method":"shutdown","params":{}}) + "\n")
p.stdin.flush()
print(p.stdout.readline().strip())
PY
```

When you want a long-lived background service instead, use:

```bash
membrain daemon
# or
membrain-daemon
```

That path serves the same underlying operation families over a Unix domain socket, defaulting to:

```bash
~/.membrain/membrain.sock
```

Recommended usage split:
- `membrain mcp` — easiest client/subprocess integration, including Claude Code
- `membrain daemon` / `membrain-daemon` — background local service mode

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

Task-oriented bounded retrieval for context construction. The canonical retrieval contract; the current live MCP runtime exposes this behavior through the bounded `recall` tool rather than a `memory_recall`-named tool.

**Canonical request model**:
- `query_text` or task text as the primary cue
- optional `context_text`
- `mode`
- `result_budget`, `token_budget`, or `time_budget_ms`
- `effort`
- `explain`
- `namespace` plus optional `include_public`
- optional scoped filters (`workspace_id`, `agent_id`, `session_id`, `task_id`, `memory_kinds`, `era_id`, `as_of_tick`, `at_snapshot`, `min_strength`, `min_confidence`, `show_decaying`, `mood_congruent`)
- when confidence-aware filtering is active, explain surfaces must preserve that confidence still influenced ordering before post-ranking suppression, and omission/explain payloads must expose `confidence_filtered`, `low_confidence_suppressed`, and any surviving uncertainty markers such as reconsolidation churn
- optional `like_id` / `unlike_id` query-by-example cues
- optional `graph_mode` and `cold_tier`

When `at_snapshot` is present, the request becomes bounded historical inspection rather than live recall: later-created memories are excluded, time-sensitive strength or freshness is recomputed against the snapshot tick, and the result must disclose partial/degraded historical visibility if current retention, policy, or repair state prevents a full reconstruction of what was once visible.

**Outputs**: the canonical `RetrievalResult` envelope, including bounded `evidence_pack`, optional `action_pack`, `outcome_class`, score summaries, graph-assistance and associative-context summaries when applicable, contradiction markers, decaying-soon markers, deferred-payload state, packaging metadata for prompt construction, and explain metadata sufficient to summarize route choice, omitted-result reasons, provenance, freshness, cache or degraded-serving behavior, and full trace stages when requested.

On the live normal-success path, `evidence_pack` should contain hydrated canonical evidence rather than planner-only route scaffolding. Explicit degraded summaries belong to no-hydrated-evidence, capped, repair-limited, or otherwise degraded/fallback cases and should remain machine-visible as degraded state rather than masquerading as ordinary success retrieval.

When explanation is embedded, `memory_recall` should preserve the same stable machine-readable families named in the canonical retrieval contract: `route_summary`, `result_reasons`, `omitted_summary`, `policy_summary`, `provenance_summary`, `freshness_markers`, `conflict_markers`, `trace_stages`, `uncertainty_markers` when full routing detail is requested.

**Rules**:
- `query_text` may be omitted only when `like_id` or `unlike_id` provides the primary cue
- effective namespace and sharing scope must be resolved before candidate generation begins
- omitted `namespace` is valid only when one deterministic default can be bound from authenticated context or stable session/job ownership
- `include_public` widens only to explicitly shareable surfaces permitted by policy
- denied or redacted namespace filters must remain inspectable without disclosing protected record existence or payload details
- `era_id` narrows recall to one explicit era inside the effective namespace; malformed, unknown, or unauthorized era selectors fail as validation or policy outcomes rather than widening to neighboring eras or silently degrading into ordinary unscoped recall
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

Retrieve diagnostic and structural details about a memory. The current live MCP runtime exposes this behavior through the bounded `inspect` tool rather than a `memory_inspect`-named tool.

**Exposes**: current tier, lineage, policy flags, lifecycle state, archive reason and restore eligibility when relevant, index presence, graph neighborhood summary, decay/retention info, cache-related routing metadata when relevant, provenance summary, freshness markers, duplicate-family or interference-maintenance summaries when present, degraded or partial-fidelity markers when archival recovery is incomplete, passive-observation inspect metadata (`source_kind`, `write_decision`, `captured_as_observation`, `observation_source`, `observation_chunk_id`, `retention_marker`) when relevant, and linked contradiction state (`conflict_state`, related `ConflictRecord` handles, preferred memory if resolved)

When `memory_inspect` includes embedded explanation or route context, it should reuse the canonical families relevant to the inspected item rather than inventing a separate inspect-only schema, especially `policy_summary`, `provenance_summary`, `freshness_markers`, `conflict_markers`, `passive_observation`, and `trace_stages` or an `explain_handle` for deferred detail.

### `memory_explain`

Explain why a memory was stored, routed, recalled, ranked, filtered, demoted, or forgotten. The current live MCP runtime exposes the currently landed explanation behavior through the bounded `why` tool rather than a `memory_explain`-named tool.

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

**Should return**:
- `action`, `reason_code`, `disposition`, `policy_surface`, and `reversibility`
- `prior_archive_state`, `resulting_archive_state`, and `partial_restore` when relevant
- `audit_kind` plus either embedded audit rows or an audit handle for later review
- operator-review markers such as `review_required`, `operator_review_required`, or equivalent summary counts when near-threshold items were retained for human inspection

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
- for index repair specifically, per-target rebuilt outputs plus verification artifacts (`verification_artifact_name`, `parity_check`, authoritative/derived row counts, authoritative/derived generations) so MCP, CLI, and daemon surfaces can prove the rebuilt view matches durable truth
- for graph repair, authoritative inputs should name durable memory rows plus canonical relation and lineage tables, and verification should prove the rebuilt projection still satisfies `graph_projection_matches_durable_edges` before the graph surface is reported healthy
- for cache repair, authoritative inputs should name durable rows, namespace/policy metadata, and current generation anchors; responses should show invalidation plus repair-warmup events by family so callers can tell that stale warm state was dropped before bounded rewarm

### `stats()`

Return the bounded operator summary shared with CLI `membrain stats`.

**Returns**: aggregated storage, quality, performance, graph, and runtime counters such as tier counts/utilization, strength or confidence rollups, cache/recall hit rates, graph totals, current tick, and last consolidation when known.

**Rules**:
- `stats()` is read-only and must not trigger repair, warming, or other hidden mutation.
- MCP, daemon/JSON-RPC, and CLI `--json` should preserve the same counter meanings even if one surface renders them as a table or dashboard.
- When policy scope, historical anchors, or degraded serving make a counter unavailable, the response should expose warnings or `availability` state instead of silently fabricating zeros or dropping the field.

### `doctor()`

Diagnose current runtime posture, stale derived state, and degraded-serving posture.

**Returns**: `{ checks: [{name, surface_kind, status, severity, affected_scope, degraded_impact?, remediation?}], summary, repair_engine_component, runbook_hints, availability?, remediation?, error_kind? }`

**Rules**:
- `doctor()` is a read-only diagnostic surface; repair remains an explicit `memory_repair` or CLI `membrain repair ...` flow.
- In the currently landed runtime, `doctor()` should be read as a bounded operator report over the active mode, health report, feature availability, and surfaced checks; it is not evidence that every later-stage subsystem in the design contract is already implemented.
- Per-check machine-readable results should stay stable enough that CLI text, daemon/JSON-RPC, and MCP can agree on what failed, which scope is affected, and what remediation comes next.
- `summary` should count ok/warn/fail check totals, and `runbook_hints[*]` should point to the canonical docs section operators should follow for the surfaced degraded-mode or incident class.
- When authoritative inputs are unreadable or corruption blocks safe serving, the response should use the shared `error_kind`, `remediation`, and `availability` semantics rather than burying the state in prose only.
- If stale action-critical evidence reaches `recheck_required` or `withhold` handling, `doctor()` should expose that through freshness-oriented checks and warnings instead of silently presenting a fully healthy surface.
- The canonical logging-heavy daemon/MCP proof artifact for these runtime workflows is `crates/membrain-daemon/tests/e2e_mcp.sh`. It should emit human-readable logs plus deterministic parity checks for retrieval envelopes, preflight/policy denial, share/unshare redaction, forgetting archive/restore/delete flows, repair/doctor diagnostics, and observe/inspect provenance.

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

### `ask(query, explain_intent?, override_intent?)` — Feature 20

Auto-classifies query intent and routes to optimal recall config. The recommended primary tool for agents. This is a later-stage packaging surface over canonical recall rather than a second retrieval path.

**Returns**: `{ intent, intent_confidence, result: RetrievalResult, formatted_response }`

`result` is the same canonical retrieval/result object used by `memory_recall`, not an ask-specific schema. `formatted_response` is a rendering convenience layered on top of that shared `RetrievalResult`, whose machine-readable fields still carry the evidence-versus-action split and all omission, policy, freshness, conflict, deferred-payload, and explanation semantics.

Visible intent classes should cover the canonical Feature 20 set: `semantic_broad`, `existence_check`, `recent_first`, `strength_weighted`, `uncertainty_focused`, `causal_trace`, `temporal_anchor`, `diverse_sample`, `procedural_lookup`, and the later-stage `emotional_filter`.

**Rules**:
- `override_intent` may pin one of the visible intent classes when the caller wants a specific bounded packaging posture or when classifier confidence is low. It changes routing and packaging only; it never widens namespace scope, bypasses policy or retention checks, or authorizes a broader retrieval lane than `memory_recall` would allow for the same effective scope.
- When `intent_confidence` is low, the tool should keep the safest bounded route or fall back to ordinary recall-equivalent packaging with explicit machine-readable route metadata rather than silently broadening retrieval breadth.
- Low-confidence decisions, explicit overrides, and action-oriented route changes must remain inspectable through the returned `result` explanation families such as `route_summary`, `result_reasons`, `policy_summary`, and `trace_stages` when full traces are requested.
- If the safer action-oriented route is blocked by stale knowledge, policy-limited visibility, or insufficient evidence, the tool should return explicit `preview`, `blocked`, or `degraded` semantics through `result.outcome_class` rather than hiding the fallback in `formatted_response`.

### `dream()` — Feature 1

Trigger a later-stage offline synthesis cycle.

**Returns**: `{ links_created, links_created_total, last_run_tick, inspect }`

**Rules**:
- `dream()` is a background maintenance mutation, not a request-path retrieval shortcut. It stays optional, explicitly triggered or idle-window scheduled, and non-blocking with respect to the core encode, recall, repair, and governance spine.
- The operation may add bounded synthetic links or merge follow-on work only from an already-authorized, bounded candidate set; it must not scan the full corpus, widen namespace scope, or bypass policy checks just because the caller asked for synthesis.
- Any emitted dream links, merge decisions, or related synthesis artifacts must remain inspectable, lineage-backed, and repairable from durable evidence rather than becoming hidden authoritative truth.
- The `inspect` payload should expose a stable run handle plus per-artifact inspect paths, summaries, and lineage (`source_memory_ids`, citation kinds, `derived_from`, and `authoritative_truth`) so wrappers preserve the same operator evidence surface as the core dream summary.
- Transport-specific wrappers may differ in how they present status or operator warnings, but they must preserve the same mutation semantics, enable/disable posture, and bounded-work expectations as CLI `membrain dream` and daemon equivalents.

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

**Returns**: `{ injections: [{memory_id, content, utility_score, token_count, reason}], tokens_used, tokens_remaining }`

**Rules**:
- this is a later-stage bounded packaging surface layered on top of the canonical recall pipeline; it does not authorize a separate retrieval path, wider namespace scope, or relaxed policy handling
- `token_budget` is a hard upper bound for the packed result. If useful candidates remain after the greedy cut, the response should use `partial_success`, warnings, or omission metadata to show that the budget—not lack of evidence—truncated the output
- one effective namespace plus any approved shared/public widening must bind before candidate generation, duplicate collapse, utility scoring, or formatting begins
- the shortlist remains subject to the ordinary bounded retrieval restrictions: no full-store scans, no uncapped graph fanout, and no pre-cut cold payload fetch just because the caller asked for ready-to-inject output
- utility ordering may penalize `working_memory_ids` overlap, but overlap only affects ranking within the authorized shortlist; it must not suppress conflict visibility or bypass policy filters
- `format` changes rendering only. The underlying injection members, utility ordering, warnings, and availability posture should stay semantically aligned with CLI `membrain budget` and daemon equivalents
- follow-on validation should prove deterministic packing under hard token budgets, parity with recall-side namespace/policy handling, and inspectable truncation when the budget clips otherwise eligible injections

### `timeline()` — Feature 5

**Returns**: `{ landmarks: [{id, label, era_id, era_start, era_end, memory_count, current_era}] }`

**Rules**:
- `timeline()` is a read-only temporal-navigation surface. It summarizes landmark-defined eras for one effective namespace and does not reopen, merge, or relabel eras implicitly.
- Returned landmarks must remain ordered, namespace-scoped, and policy-filtered before any detail is packaged.
- `era_id` is the stable selector used by recall-side era filtering; `label` is human-facing text and must not become the authoritative join key.
- `current_era` indicates whether the listed landmark anchors the namespace's still-open era so callers can distinguish active versus closed eras without inferring from a missing `era_end` alone.

### `observe(content, context?, chunk_size?, source_label?)` — Feature 6

Segment content into memories via topic boundary detection.

**Returns**: `{ memories_created, topic_shifts, bytes_processed }`

**Rules**:
- this is a later-stage bounded intake surface, not a raw-ingest bypass. Observed content still resolves to one effective namespace and goes through the ordinary validation, provenance, duplicate-routing, and policy path before persistence
- when one observe call yields multiple fragments, returned and persisted state should preserve `observation_source` plus a shared `observation_chunk_id` so later inspect, audit, and repair flows can trace the bounded observation batch without inventing a synthetic session boundary
- `source_label` is provenance metadata only; it must not widen namespace scope, suppress policy checks, or override caller identity
- segmentation remains bounded by explicit chunk-size and topic-shift controls. Convenience wrappers around file or stream observation must not hide unbounded rescans, unbounded buffering, or whole-store work behind the observe surface
- the underlying outcome semantics should stay aligned with CLI and daemon observe flows: explicit warnings when the batch is truncated, blocked, or gated, and no silent fallback that changes the meaning of ingestion
- follow-on validation should prove deterministic chunking under fixed inputs, preserved source-label and chunk-group metadata, and parity with the shared intake contract across transports

### `uncertain(top_k?)` — Feature 7

**Returns**: `{ memories: [{id, content, confidence, uncertainty_score, uncertainty_interval?, corroboration_count, freshness_uncertainty?, contradiction_uncertainty?, missing_evidence_uncertainty?, known?, assumed?, uncertain?, missing?, change_my_mind_conditions?, reconsolidation_count}] }`

**Rules**:
- The uncertainty surface should expose more than a scalar rank. When the underlying evidence supports it, each entry should preserve machine-readable `known`, `assumed`, `uncertain`, `missing`, and `change_my_mind_conditions` fields so callers can tell what is established, what remains inferred, what evidence is absent, and what new evidence would most directly reduce uncertainty.
- High-stakes or action-oriented callers should prefer confidence intervals plus these uncertainty markers by default instead of treating them as optional diagnostics-only add-ons.

### `skills()` / `extract_skills()` — Feature 8

**Returns**: `{ namespace, extraction_trigger, extracted_count, skipped_count, reflection_compiler_active, procedures: [{ namespace, fixture_name, content, confidence, storage: { storage_class, authority_class, acceptance_state, review_status, durable, rebuildable, canonical_rebuild_source, freshness_status, repair_status }, review: { derivation_rule, tentative, accepted, supporting_memory_count, source_citation_count, supporting_fields, operator_review_required, review_reason, reflection: { artifact_class, source_outcome, checklist_items, advisory, trusted_by_default, release_rule, promotion_basis } | null }, recall: { recall_surface, retrievable_as_procedural_hint, retrieval_kind, query_cues, source_engram_id, member_count } }] }`

**Rules**:
- `skills()` is the review/list surface for already stored derived skill artifacts; `extract_skills()` runs a bounded extraction pass first, then returns the same artifact family.
- Returned procedures remain explicitly derived durable artifacts until a separate acceptance path promotes them. The payload must keep tentative/non-authoritative state visible rather than implying implicit promotion.
- Each procedure should expose storage, review, and recall semantics directly so operators and wrappers can inspect whether the artifact is rebuildable, still requires review, and which cues make it retrievable as a procedural hint.
- When the reflection-compiler contract is active, `review.reflection` should make advisory status explicit by carrying the artifact class (`procedure` vs `anti_pattern`), source outcome (`successful_episode` vs `failed_episode`), bounded checklist items, and the release rule showing that promotion still requires explicit acceptance or repeated usefulness with lineage.
- `source_engram_id` may be absent when the bounded source set does not resolve to one stable seed, but the recall surface must still preserve `member_count` and query cues derived from the supporting evidence.

### `share(id, namespace_id)` / `unshare(id)` — Feature 9

Adjust visibility for cross-agent access without changing the memory's canonical identity or durable ownership.

**Returns**:
- `share(id, namespace_id)` → `{ id, namespace, visibility, policy_summary }`
- `unshare(id)` → `{ id, namespace, visibility, policy_summary }`

**Rules**:
- `share` and `unshare` mutate visibility metadata; they do not mint a second authoritative copy, move the memory into a different canonical namespace, or bypass lineage/provenance requirements.
- `share` must bind an explicit approved namespace scope and still obey workspace ACL, agent ACL, session visibility, and any later read-time redaction rules.
- `unshare` tightens future widened access without deleting the underlying memory or changing durable identity.
- Malformed, unknown, or unauthorized namespace targets fail as `validation_failure` or `policy_denied` outcomes before any hidden copy, fanout, or candidate-generation work occurs.
- Repeating `share` or `unshare` with the same effective visibility is idempotent.
- Recall/search/get/explain surfaces must preserve when approved shared/public widening materially shaped the visible result set through `policy_summary` or the equivalent explain-family fields.

### `health()` — Feature 10

**Returns**: `BrainHealthReport` as JSON for the current visible runtime posture (tiers, quality, engrams, signals, activity, and feature availability within the active mode)

**Rules**:
- `health()` is the bounded machine-readable operator dashboard shared with CLI `membrain health`; transport-specific rendering may differ, but the underlying report semantics must not.
- In stdio MCP mode, `health()` reports process-local runtime posture and best-effort reuse within that process. Only the Unix-socket daemon may claim daemon-owned repeated-request warm-runtime guarantees.
- `health()` and `doctor()` should surface `embedder_runtime` explicitly with stable machine-readable state (`not_loaded`, `loaded`, `warm`, `degraded`, `unavailable`) plus backend/generation/cache counters so clients do not guess whether fastembed is actually operational in the current runtime.
- The report should preserve tier and capacity counters, quality/conflict/uncertainty signals, runtime activity, repair/backpressure indicators, availability posture, explicit degraded-status guidance, and feature-availability facts strongly enough that automation can make the same decisions a human operator would make from the CLI dashboard.
- The machine-readable view should expose enough detail to distinguish healthy service from degraded-but-servable posture, including repair-queue growth or backlog signals, backpressure state, which read or write paths still survive, and which feature surfaces are unavailable or maturity-gated.
- The canonical `BrainHealthReport` should expose `dashboard_views`, `alerts`, and `drill_down_paths` in addition to raw counters so MCP clients can render the same overview, alerting, and subsystem-investigation flow as CLI and daemon surfaces without inventing wrapper-local logic.
- `dashboard_views` should enumerate the main operator views with stable identifiers, summaries, alert counts, and drill-down targets; the `attention` view should surface hotspot counts, max score, bounded prewarm state, and the canonical `/health/attention` drill-down; `alerts` should carry stable severity, reason codes, runbook hints, and drill-down paths.
- `attention.hotspots[*]` should expose stable heatmap fields (`heat_bucket`, `heat_band`) plus explicit bounded prewarm guidance (`prewarm_trigger`, `prewarm_action`, `prewarm_target_family`) so MCP operators can inspect hotspot-driven warming decisions without guessing hidden heuristics.
- When policy scope, historical anchors, or degraded serving limit visibility, `health()` should return explicit warnings or `availability` state instead of silently fabricating a fully healthy view.

### `why(id)` — Feature 11

Trace causal chain to root evidence.

**Returns**: canonical retrieval/explain envelope families for the targeted memory, including causal-chain ancestry in `result_reasons`, `provenance_summary`, `graph_expansion`, and bounded cutoff metadata rather than a transport-only bespoke trace blob.

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

### `goal_state(task_id?)` / `goal_pause(task_id?, note?)` / `goal_resume(task_id?)` / `goal_abandon(task_id?, reason?)` — Plan §10.6

Manage later-stage resumable goal-stack state for long-running work without turning checkpoints into authoritative memory truth.

**Returns**:
- `goal_state(task_id?)` → `{ task_id, status, goal_stack: [{goal, parent_goal?, priority?, blocked_reason?}], latest_checkpoint: {checkpoint_id, created_tick, evidence_handles, pending_dependencies, stale}, blackboard_summary?, namespace }`
- `goal_pause(task_id?, note?)` → `{ task_id, status, checkpoint_id, paused_at_tick, note?, namespace }`
- `goal_resume(task_id?)` → `{ task_id, status, checkpoint_id, resumed_at_tick, restored_evidence_handles, restored_dependencies, warnings?, namespace }`
- `goal_abandon(task_id?, reason?)` → `{ task_id, status, checkpoint_id?, abandoned_at_tick, reason?, namespace }`

**Rules**:
- these tools are later-stage working-state surfaces layered on top of stable task/session identity, namespace binding, and policy enforcement; they do not create hidden workflow scope or a second memory authority
- goal-stack checkpoints are bounded resumability anchors for active work and must stay distinct from Feature 12 named historical inspection anchors
- `goal_state()` should surface a visible blackboard projection with `projection_kind = "working_state_projection"` and `authoritative_truth = "durable_memory"` so clients can distinguish working-state views from authoritative memory truth
- checkpoint payloads should preserve explicit selected-evidence handles, pending dependencies, blocked reason, and compact blackboard summary or equivalent working-state metadata rather than copying authoritative memories into a second store
- `goal_pause()` persists the latest valid resumability checkpoint and marks the task dormant without widening scope or mutating durable truth beyond checkpoint metadata
- `goal_resume()` rehydrates only from the newest valid checkpoint, restores referenced evidence and dependency handles when still readable, and must surface stale, missing, or policy-incompatible checkpoint state explicitly instead of guessing from scratch
- `goal_abandon()` ends the active goal intentionally, preserves the checkpoint or audit trail needed for later inspection and handoff, and never silently deletes authoritative evidence or leaves the goal implicitly active
- related inspectable parity artifacts may expose structured per-action logs for get, pin, dismiss, snapshot, pause, resume, and abandon flows, but those logs remain derived operator evidence rather than authoritative memory state
- when resumability state is unavailable, disabled, or blocked by policy, the response should fail explicitly or return a degraded/blocked posture rather than silently fabricating a reconstructed plan

### `hot_paths(top_n?)` / `dead_zones(min_age_ticks?)` — Feature 13

**Returns**: hot/dead zone entries with retrieve counts, scores, and age

### `diff(since, until?, top_n?)` — Feature 14

**Returns**: `BrainDiff` — new memories, strengthened, weakened, archived, conflicts resolved, new engrams

### `fork(name, parent_namespace?, inherit?, note?)` — Feature 15

**Returns**: `{ name, forked_at_tick, inherited_count }`

### `merge_fork(fork_name, target_namespace, conflict_strategy?, dry_run?)` — Feature 15

**Returns**: `MergeReport` with merge/conflict counts

### `compress(dry_run?)` / `schemas(top_n?)` — Feature 17

**Inputs**:
- `compress(dry_run?)` → `namespace`, optional `dry_run`, plus the common request envelope fields
- `schemas(top_n?)` → `namespace`, optional `top_n`, plus any bounded policy/context envelope fields exposed by the wrapper

**Returns**:
- `compress(dry_run?)` → `{ namespace, dry_run, decision, schemas_created, episodes_compressed, storage_reduction_pct, blocked_reasons?, related_run?, schema_artifact?, verification?, compression_log_entries? }`
- `schemas(top_n?)` → `{ schemas: [{id, content, source_count, confidence, keywords, compressed_member_ids?}] }`

**Rules**:
- Schema compression is a later-stage, consolidation-adjacent follow-on rather than a prerequisite for the bounded core retrieval path.
- `compress()` should preserve repairable lineage by keeping source-memory links, `compressed_into` relationships, verification state, and audit-visible before/after effects inspectable rather than flattening many episodes into an opaque summary.
- Compression reduces or reroutes source-episode prominence; it does not silently hard-delete the underlying episodic evidence.
- `dry_run` should surface candidate cluster counts, affected source-memory estimates, expected schema creations, and any maintenance or policy blockers before mutation.
- Accepted non-dry-run `compress()` results should keep the chosen candidate decision, emitted schema artifact, reconstructability verification, and bounded compression-log history machine-readable so MCP, daemon/JSON-RPC, and CLI stay parity-aligned.
- `schemas()` should make the resulting abstract pattern memories inspectable as schema artifacts with source counts, confidence, and dominant keywords or equivalent summaries.

### `mood_history(since_tick?, namespace_id?)` — Feature 18

**Returns**: `{ timeline: [{tick_start, tick_end, avg_valence, avg_arousal, state, memory_count}] }`

**Rules**:
- `mood_history()` is a later-stage, read-only introspection surface over emotional trajectory rows; it does not mutate memories, synthesize a new canonical mood object, or bypass the ordinary namespace and policy contract.
- Returned timeline rows should stay bounded to one effective namespace and the requested time window, with explicit degraded or omission signaling when history is partial, redacted, or unavailable.
- Retrieval-side `mood_congruent` behavior may consume the same underlying emotional metadata, but the read-only history view must remain semantically separate from the optional ranking hint so callers can inspect history without silently changing recall behavior.
- `health()` should expose the corresponding bounded summary through the `affect_trajectory` dashboard view plus a `/mood_history` drill-down target, so MCP clients can render the same recall-facing trajectory context without inventing transport-local history cards.

### `audit(memory_id?, since_tick?, op?, limit?)` — Feature 19

**Returns**: `{ entries: [{op, memory_id, tick, before_strength, after_strength, before_confidence, after_confidence, triggered_by, note, namespace, redaction, related_snapshot, related_run}] }`

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

The `health()` tool returns a `BrainHealthReport` that includes available features. Clients should check feature availability before calling feature-specific tools. Operators that render heatmaps or prewarm state should also consume `attention.hotspots`, `dashboard_views`, `drill_down_paths`, and `cache.adaptive_prewarm_*` from the same report instead of inventing transport-local summaries.

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
