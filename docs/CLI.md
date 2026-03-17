# membrain — CLI Reference

> Canonical source: PLAN.md Section 35.
> Historical CLI overview: PLAN.md Section 9.
> Feature-specific commands: PLAN.md Sections 46–47.

## Global Options

```
--json          Output as JSON (all commands)
--quiet, -q     Suppress informational output
--verbose, -v   Show extra details
--db-path       Override default database location
--tick          Show tick numbers in output
```

## Design Principles

- CLI commands map cleanly onto core memory actions
- Scriptable and machine-readable (`--json` everywhere)
- Human-readable output layered on top of structured results
- CLI must not create hidden behavior different from MCP behavior
- Every major command supports: human text, JSON, and meaningful exit codes

## CLI Contract Scope

This document fixes the stable CLI surface for `mb-1hw.1`: command families, naming rules, shared flag vocabulary, human-versus-JSON parity, and alias/deprecation expectations.

It does **not** finalize:
- the canonical retrieval result envelope from `mb-1hw.8`
- the destructive preflight / blocked-action schema from `mb-1hd.7`

The shared failure/remediation taxonomy in this document should stay aligned with `docs/MCP_API.md` and the degraded/read-only availability contract in `docs/OPERATIONS.md`. Those sibling contracts may add transport-specific envelope fields or stronger safeguards later, but they should plug into the command and outcome vocabulary defined here rather than redefining it.

For retrieval-facing commands, `mb-1hw.8` defines one canonical `RetrievalResult` envelope shared with daemon/JSON-RPC and MCP. CLI text may summarize that object for humans, but CLI `--json` must preserve the same outcome class, evidence-versus-action split (`evidence_pack` and optional `action_pack`), omission semantics, policy/provenance/freshness/conflict markers, deferred-payload meaning, and explanation families instead of inventing a CLI-only result shape.

## Stable Command Families and Naming Rules

Canonical CLI families:
- **encode / intake** — `remember`, `observe`, `import`
- **recall / query** — `recall`, `ask`, `budget`
- **inspect / explain / audit** — `inspect`, `why`, `beliefs`, `audit`
- **maintenance / admin** — `stats`, `health`, `doctor`, `repair`, `benchmark`, `consolidate`, `compress`, `dream`, `export`, `daemon`, `mcp`
- **history / namespace / change management** — `timeline`, `landmark`, `diff`, `snapshot`, `fork`, `merge`, `share`, `unshare`, `namespace`, `forget`, `strengthen`, `update`

Naming rules:
- prefer verb-first command names for user-triggered operations
- use noun subcommands only for grouped resource surfaces such as `namespace` and `snapshot`
- keep one canonical spelling per operation; new synonyms or shadow verbs are not part of the stable surface unless explicitly documented as aliases
- feature-specific work should extend an existing family unless it has a strong reason to introduce a new top-level command
- command spelling may vary from MCP tool names, but it must not change the underlying request, policy, or outcome semantics

## Shared Flag Vocabulary

These flags define shared CLI vocabulary even when only some commands accept them:

- `--json` — emit machine-readable output for the same semantic result shown in text mode; it does not select a different execution path
- `--quiet` — suppress non-essential human-oriented narration; it must not remove required machine-readable fields in `--json` mode
- `--verbose` — add detail only; it must not change the outcome class or underlying result semantics
- `--db-path` — override the storage location without changing logical namespace or policy scope
- `--tick` — include tick-oriented temporal markers when the command exposes them
- `--namespace` — bind one effective namespace; if no deterministic default exists, omission is a validation failure
- `--include-public` — widen only to explicitly shareable public/shared surfaces allowed by policy
- `--explain` — request `none`, `summary`, or `full` explanation verbosity without changing retrieval semantics
- `--dry-run` — request a previewable, non-mutating description of a command that would otherwise change state
- `--force` — bypass local confirmation or readiness prompts when the command allows it; it never bypasses policy checks
- `--at` / `--as-of` — select historical scope by named snapshot or tick; incompatible combinations are validation failures
- `--format` — command-specific rendering or export selector used only when a surface has multiple domain formats, such as `budget` markdown or `export` / `import` serialization formats; it complements rather than replaces the global `--json` contract

## Default-Safe Ergonomics

At the CLI layer, destructive or high-blast-radius commands should follow these ergonomics in addition to the shared safeguard contract owned by `docs/OPERATIONS.md`:
- default to the narrowest explicit scope rather than silently widening namespace, history, or maintenance coverage
- prefer `--dry-run` or another preview path before state-changing repair, merge, invalidation, compression, or deletion-adjacent work
- surface a **blocked** outcome when confirmation, scope, snapshot/generation freshness, or other readiness conditions are missing instead of guessing or proceeding implicitly
- treat `--force` as a local confirmation override only; it does not bypass policy, namespace, retention, legal-hold, or other safety invariants
- use **rejected** for malformed or policy-denied requests that still cannot proceed, not as a synonym for confirmation-missing or stale-preflight states
- keep preview, blocked, degraded, and accepted outcomes distinguishable in both text and JSON modes

## Alias and Deprecation Policy

- Every stable operation should have one documented canonical spelling.
- Aliases are compatibility shims, not parallel first-class commands.
- Documentation and examples should prefer canonical spellings once they exist.
- Any future alias or deprecation must name the replacement spelling, warning behavior, and planned removal boundary.
- JSON output should preserve enough machine-readable detail for callers to detect deprecation warnings without scraping human prose.

---

## Core Commands

### `membrain remember [CONTENT] [OPTIONS]`

Encode a new memory into the brain's hot store. Computes embedding, novelty, initial strength. Applies attention gating, emotional tagging, engram clustering.

```bash
membrain remember "Fixed the JWT expiry bug — was using utc() not now()"
membrain remember "Rate limit is 100 req/s for Stripe API" \
  --context "integrating payments" --kind semantic
membrain remember "Deploy caused 30-min downtime" --valence -0.8 --arousal 0.8
membrain remember "Background info" --attention 0.2
echo "content" | membrain remember --context "recent commits" --kind semantic
```

| Option | Default | Description |
|--------|---------|-------------|
| `--context, -c` | — | Current task/context (enhances retrieval later) |
| `--attention, -a` | 0.7 | Attention level 0.0–1.0 (below 0.2: discarded) |
| `--valence, -V` | 0.0 | Emotional valence -1.0 to +1.0 |
| `--arousal, -A` | 0.0 | Emotional arousal 0.0–1.0 |
| `--kind, -k` | episodic | Memory kind: episodic, semantic, procedural |
| `--source` | cli | Source tag: cli, mcp, api |
| `--share` | — | Mark as shared (cross-agent) |
| `--namespace` | default | Target namespace |

### `membrain recall <QUERY> [OPTIONS]`

3-tier retrieval: Tier1 cache → SQL pre-filter → HNSW search → rescore → engram BFS → unified scoring.

All CLI recall invocations normalize into the canonical `RecallRequest` described by `PLAN.md` and `RETRIEVAL.md`: `<QUERY>` populates `query_text`, `--context` maps to `context_text`, `--confidence` maps to bounded `effort`, and CLI-only spelling differences must not create different retrieval semantics.

The machine-readable result for `recall` is the canonical `RetrievalResult` envelope from `mb-1hw.8`. In practice that means CLI JSON should expose a bounded `evidence_pack`, optional `action_pack`, `outcome_class`, omission/deferred-payload state, and the same policy/provenance/freshness/conflict/explanation families used by daemon/JSON-RPC and MCP.

```bash
membrain recall "JWT authentication"
membrain recall "database connection" --context "fixing performance issue"
membrain recall "Rust async" --top 10 --confidence high
membrain recall "Python" --kind semantic
membrain recall "anything" --show-decaying
membrain recall "auth" --as-of 5000
membrain recall --like <uuid>              # query-by-example (Feature 3)
membrain recall --unlike <uuid>            # find most different (Feature 3)
membrain recall "debugging" --era <id>     # temporal era filter (Feature 5)
membrain recall "prefs" --min-confidence 0.8  # confidence filter (Feature 7)
membrain recall "prefs" --namespace project-x --include-public  # widened shared/public scope only
membrain recall "arch decision" --at before-refactor  # time travel (Feature 12)
membrain recall "session" --mood-congruent  # emotional boost (Feature 18)
membrain recall "auth" --explain summary --cold-tier avoid
```

| Option | Default | Description |
|--------|---------|-------------|
| `--context, -c` | — | Current context; maps to `context_text` |
| `--top, -n` | 5 | Result budget |
| `--kind, -k` | — | Filter: episodic, semantic, procedural |
| `--min-strength` | config | Minimum effective strength |
| `--confidence` | normal | Bounded effort level: fast, normal, high |
| `--show-decaying` | — | Include memories near decay threshold |
| `--no-engram` | — | Force `graph_mode=off` |
| `--as-of` | — | Time-travel using `as_of_tick` |
| `--like` | — | Query-by-example cue via `like_id` |
| `--unlike` | — | Query-by-example cue via `unlike_id` |
| `--era` | — | Filter to temporal era |
| `--min-confidence` | — | Minimum confidence score |
| `--at` | — | Historical recall at named snapshot (`at_snapshot`) |
| `--namespace` | caller default if deterministic, otherwise required | Namespace scope |
| `--include-public` | false | Widen only to policy-approved shared/public surfaces |
| `--explain` | summary | Explain verbosity: none, summary, full; summary explains why results appeared and what major route/policy/budget boundaries mattered, while full adds routing-trace stages and exclusion details |
| `--cold-tier` | auto | Cold-tier routing hint: avoid, auto, allow |

Normalization and safety rules:
- `<QUERY>` may be omitted only when `--like` or `--unlike` provides the primary cue.
- `--as-of` and `--at` must not be combined unless a later contract defines deterministic precedence.
- `--at` selects bounded historical inspection at a named snapshot: later-created memories are excluded and time-sensitive strength or freshness is recomputed using the snapshot tick as historical `now`.
- Snapshot-scoped recall is not a restore guarantee; it can answer only from durable evidence still retained and policy-visible at inspection time, and should return partial or degraded markers rather than fabricate a complete past state when retention, redaction, or repair loss removed authoritative evidence.
- `--include-public` does not bypass namespace ACLs or expose private cross-namespace data.
- `--cold-tier allow` may enable Tier3 candidate generation, but it does not permit pre-cut cold payload fetch.
- `--explain summary` should surface major route choices, omitted-result reasons, freshness/conflict markers, and any cache bypass, stale-warning, or degraded-mode outcome that materially affected returned results.
- `--explain full` should preserve machine-readable routing-trace parity with daemon, IPC/JSON-RPC, and MCP surfaces, including candidate counts, cache family or event data, and exclusion reasons where those surfaces exist.

### `membrain forget <ID> [OPTIONS]`

Archive (soft-delete) a memory. Never hard-deletes without explicit policy.

```bash
membrain forget <uuid>
membrain forget <uuid> --force    # bypass confirmation
```

Rules:
- `forget` archives by default; it does not imply hard deletion unless a separate policy-authorized destructive path is invoked.
- `membrain forget <uuid> --force` satisfies only the local confirmation step for the exact preflighted scope; policy denial, retention constraints, legal hold, stale preflight state, or widened scope must still block or reject the operation.
- Archived memories remain inspectable and auditable, and those surfaces should expose archive reason, restore eligibility, and any degraded-fidelity markers when only partial recovery is possible.
- Ordinary recall, cold-tier lookup, and snapshot inspection do not implicitly restore archived memories.
- Explicit restore or admin paths may reactivate archived memories when policy allows, but they restore only retained authoritative evidence and must surface partial or degraded state rather than pretending to recreate a perfect pre-archive copy.

### `membrain strengthen <ID>`

Manually apply LTP to a memory (same effect as on_recall).

### `membrain update <ID> <NEW_CONTENT>`

Submit a pending update during reconsolidation window.

### `membrain stats [OPTIONS]`

Brain health statistics.

Stats contract:
- `membrain stats` is the read-only operator summary for storage, quality, performance, and runtime counters; it is not a repair, approval, or mutation surface.
- `--json` should preserve the same machine-readable report consumed by daemon/JSON-RPC and MCP `stats()` equivalents, including tier counts and utilization, strength/confidence or uncertainty rollups, recall/cache metrics, runtime or daemon status when known, and any historical anchor selected via `--at`.
- When a field is unavailable because the namespace, snapshot, policy scope, or serving posture is degraded, the response should surface explicit warnings or availability state instead of silently dropping the field or replacing it with misleading zeros.

```bash
membrain stats
membrain stats --json
membrain stats --at before-refactor    # stats at snapshot point
```

### `membrain inspect <ID> [OPTIONS]`

Full memory details: tier, lineage, policy, lifecycle, archive reason and restore eligibility when relevant, graph neighborhood, decay, cache-related routing metadata when relevant, provenance summary, freshness markers, conflict state, and degraded-fidelity markers when partial archival recovery applies.

Inspect contract:
- `membrain inspect` is the item-anchored structural and diagnostic surface, not a second retrieval path with different semantics.
- In `--json` mode it should preserve the same machine-readable families the retrieval contract expects when they are relevant to the inspected item: `policy_summary`, `provenance_summary`, `freshness_markers`, `conflict_markers`, and `trace_stages` or an `explain_handle` when deeper routing context is deferred.
- When the inspected item was returned, filtered, archived, degraded, or partially reconstructed through a bounded route that still matters operationally, `inspect` should expose enough cache/tier/route metadata to explain that state without forcing operators to infer it from prose.

```bash
membrain inspect <uuid>
membrain inspect <uuid> --history       # belief version chain (Feature 2)
membrain inspect <uuid> --show-source   # source engram for procedurals (Feature 8)
```

### `membrain doctor`

Diagnose brain health: orphan edges, missing embeddings, stale indexes, broken lineage, checkpoint corruption.

Doctor contract:
- `membrain doctor` is a read-only diagnosis surface; it identifies corruption, stale derived state, and degraded serving posture without implicitly mutating anything. Repair remains an explicit `membrain repair ...` flow.
- `membrain doctor run --json` should preserve machine-readable per-check results with stable fields for check name, surface kind, status/severity, affected scope, degraded impact, and remediation or repair hints so CLI text, daemon/JSON-RPC, and MCP do not drift.
- When corruption or unreadable authoritative inputs prevent safe serving, `doctor` should surface the same `error_kind`, `availability`, and remediation semantics used by sibling daemon/MCP contracts rather than collapsing everything into prose-only warnings.

```bash
membrain doctor run
membrain doctor run --json
```

---

## Consolidation & Maintenance

### `membrain consolidate`

Manually trigger NREM+REM+Homeostasis cycle.

### `membrain compress [OPTIONS]` (Feature 17)

Schema Compression is a later-stage maintenance surface that synthesizes repeated episodic situation patterns into durable schema memories without deleting the underlying source evidence. It remains gated behind consolidation maturity, lineage preservation, and repairable auditability rather than acting as a default foreground path.

CLI expectations:
- `membrain compress` should report whether the pass is a dry run or apply run, how many candidate episodes or clusters were eligible, how many schema memories were created, and the bounded storage or maintenance impact.
- Compression should preserve inspectable lineage to the source episodes; `inspect`, audit, and related history surfaces must be able to show which source memories contributed to a schema and which episodes were marked `compressed_into` that schema.
- Compression should reduce source-episode strength rather than silently deleting source evidence, and any future restore/uncompress path must be explicit rather than implicit retrieval magic.
- Because this is later-stage follow-on work, CLI surfaces should make its gated/background nature visible instead of implying it is part of the core retrieval spine.

```bash
membrain compress              # trigger compression pass
membrain compress --dry-run    # show what would be compressed
```

### `membrain dream [OPTIONS]` (Feature 1)

```bash
membrain dream                 # trigger dream cycle manually
membrain dream --status        # last run, links created
membrain dream --disable       # pause background dreaming
```

---

## Observability & Diagnostics

### `membrain health [OPTIONS]` (Feature 10)

Full terminal dashboard: tier utilization, decay curves, engrams, signals, activity.

Health contract:
- `membrain health` is the operator dashboard rendering of the same bounded `BrainHealthReport` exposed through daemon/JSON-RPC and MCP `health()`.
- `--json`, `--brief`, and `--watch` are presentation variants over one machine-readable report; they must preserve the same tier/capacity, quality/conflict/uncertainty, activity/runtime, repair/backpressure, availability-posture, and feature-availability meaning.
- The report should make degraded posture, repair-queue growth, and backpressure state inspectable enough that operators can tell whether to continue normal work, inspect `doctor`, or switch to a runbook instead of inferring that state from prose.
- If historical scope, policy limits, or degraded serving change what can be shown, the surface should say so through warnings or availability markers instead of silently pretending the dashboard is complete.

```bash
membrain health
membrain health --watch               # live refresh
membrain health --watch --interval 5
membrain health --json
membrain health --brief               # one-line summary
membrain health --at before-refactor  # past state
```

### `membrain timeline [OPTIONS]` (Feature 5)

```bash
membrain timeline                     # list all landmarks
membrain timeline --detail            # landmarks + memory count per era
membrain landmark <uuid>              # promote memory to landmark
membrain landmark --label "v2 launch" <uuid>
```

### `membrain mood [OPTIONS]` (Feature 18)

```bash
membrain mood                         # current emotional state
membrain mood --history               # full timeline
membrain mood --history --since 5000
```

### `membrain diff [OPTIONS]` (Feature 14)

```bash
membrain diff --since before-refactor
membrain diff --since 4000
membrain diff --since before-refactor --until v1-launch
membrain diff --since 4000 --top 5 --json
```

### `membrain audit <ID> [OPTIONS]` (Feature 19)

Audit output should preserve retention, legal-hold, repair, and degraded-serving evidence strongly enough that operators can distinguish policy outcomes from stale or partially rebuilt derived state.

Audit expectations:
- entries remain read-only history records, not a restore or replay command surface
- each entry should preserve the operation, actor/source (`triggered_by`), effective namespace scope, before/after deltas when applicable, and an operator-meaningful reason or note
- policy-limited audit responses should surface redaction explicitly rather than silently omitting protected context
- when a visible state change came from repair, snapshot-protected maintenance, migration, compaction, or incident handling, the audit trail should retain enough correlation metadata or note text to connect the entry back to that run without treating snapshots as the audit log
- CLI text, CLI `--json`, daemon/JSON-RPC, and MCP `audit()` must preserve the same entry meaning, required correlation fields, and degraded or redacted history semantics even if one surface summarizes or paginates differently.

```bash
membrain audit <uuid>                       # full history
membrain audit <uuid> --since 5000
membrain audit <uuid> --op recall           # only recall events
membrain audit --since 5000 --op archive    # what was archived?
membrain audit --recent 100
```

### `membrain hot-paths` / `membrain dead-zones` (Feature 13)

```bash
membrain hot-paths --top 50 --json
membrain dead-zones --min-age 1000
membrain dead-zones --forget-all
```

### `membrain uncertain [OPTIONS]` (Feature 7)

The uncertainty surface is the user-facing inspection path for Feature 7. It should not stop at a scalar score: when uncertainty is surfaced, the CLI should make it inspectable as what is currently **known**, what is still **assumed**, what remains **uncertain**, what evidence is **missing**, and what `change_my_mind_conditions` or stronger evidence would most directly upgrade confidence.

CLI expectations:
- `membrain uncertain` should list memories or results whose uncertainty is high enough to warrant review, including `confidence`, `uncertainty_score`, and the dominant contributors such as corroboration, freshness, contradiction, or missing-evidence uncertainty.
- `--json` should preserve machine-readable uncertainty markers rather than flattening them into prose-only explanation. At minimum, each returned entry should be able to expose `known`, `assumed`, `uncertain`, `missing`, and `change_my_mind_conditions` when that structure is available.
- High-stakes or action-oriented paths should include confidence intervals and the same uncertainty markers by default rather than hiding them behind an optional diagnostics-only command.

```bash
membrain uncertain                    # memories with confidence < 0.5
membrain uncertain --top 20
```

---

## Belief & Knowledge Management

### `membrain beliefs [OPTIONS]` (Feature 2)

Belief Versioning is a later-stage trust/introspection feature built on the already-canonical contradiction, lineage, and supersession model. It turns those durable records into inspectable history surfaces rather than redefining core mutation semantics.

CLI expectations:
- `membrain beliefs <query>` should return the belief chain or topic history, including current preferred entry, older versions, and any unresolved disagreement or coexistence state.
- `membrain beliefs --conflicts` should surface open contradiction sets without flattening them into one winner.
- `membrain beliefs --resolve <id>` records an explicit resolution outcome such as coexistence, supersession, or authoritative override; it must not silently rewrite history.
- `membrain inspect <id> --history` remains the item-anchored path for full version-chain inspection.
- Supersession-aware retrieval should prefer the current operational answer for default packaging while still preserving handles to superseded or conflicting evidence when explanation, audit, or history views request them.

This feature stays gated behind the core contradiction/supersession contract and should not be treated as a prerequisite for bounded baseline retrieval.

```bash
membrain beliefs "user preferences"   # show belief chain
membrain beliefs --conflicts          # list contradictions
membrain beliefs --resolve <id>       # resolve pending conflict
```

### `membrain why <ID> [OPTIONS]` (Feature 11)

Trace causal and retrieval rationale for one item, including lineage and route ancestry when available.

Why-surface contract:
- `membrain why` is the focused route-and-causality view for one item or belief chain, complementing `inspect` rather than replacing it.
- `--json` should preserve machine-readable route and explanation families rather than a bespoke trace-only blob: `route_summary`, `result_reasons`, `omitted_summary` when siblings or alternatives were capped, `policy_summary`, `provenance_summary`, `freshness_markers`, `conflict_markers`, and `trace_stages` when full route ancestry is requested.
- If the command returns a deferred explanation handle instead of embedding the full trace, the handle should still point at the same semantic explanation contract used by recall, daemon/JSON-RPC, and MCP surfaces.

```bash
membrain why <uuid>                   # trace causal chain to root evidence
membrain why <uuid> --depth 5
membrain why <uuid> --json            # machine-readable route/provenance chain
```

### `membrain invalidate <ID> [OPTIONS]` (Feature 11)

```bash
membrain invalidate <uuid>            # cascade confidence penalty
membrain invalidate <uuid> --dry-run
```

### `membrain skills [OPTIONS]` (Feature 8)

```bash
membrain skills                       # list extracted procedural memories
membrain skills --extract             # trigger extraction pass
membrain engram <uuid> --extract      # extract from specific engram
```

### `membrain schemas [OPTIONS]` (Feature 17)

```bash
membrain schemas --top 10
membrain uncompress <schema-uuid>     # restore source episodes
```

---

## Query Intelligence

### `membrain ask <QUERY> [OPTIONS]` (Feature 20)

Auto-classifies intent and routes to optimal retrieval configuration. The primary entry point for agents.

`ask` reuses the same canonical `RetrievalResult` envelope as `recall`; intent classification and `formatted_response` are wrappers around that shared result object, not a separate answer schema. The machine-readable response should therefore preserve the underlying `evidence_pack`, optional `action_pack`, `outcome_class`, omission/deferred-payload state, and explanation families even when the default text rendering foregrounds the formatted answer.

```bash
membrain ask "what do I know about Rust lifetimes?"
membrain ask "did I ever encounter a borrow checker error?"
membrain ask "what's most important about deploy?"
membrain ask "why do I believe microservices are better?"
membrain ask "how to deploy the service?"

membrain ask "..." --explain-intent       # show classified intent
membrain ask "..." --override-intent semantic-broad
```

### `membrain budget [OPTIONS]` (Feature 4)

Context Budget is a later-stage packaging surface that turns bounded recall results into a ready-to-inject context pack. It reuses the same namespace, policy, omission, and degraded-serving semantics as `recall`; it does not introduce a second retrieval path or widen scope for convenience.

```bash
membrain budget --tokens 2000
membrain budget --tokens 2000 --context "debugging" --format markdown
membrain budget --tokens 1500 --namespace project-x --include-public
```

CLI expectations:
- `--tokens <n>` is the hard packing budget. Successful responses should report `tokens_used` and `tokens_remaining`, and budget-constrained omission should stay visible through the existing `partial` / `degraded` outcome vocabulary rather than disappearing into a generic success.
- The command starts from the same bounded, policy-pruned retrieval shortlist the recall contract allows for the effective namespace. Duplicate collapse, conflict markers, and the no-pre-cut-cold-payload rule still apply before greedy packing begins.
- Utility ordering may penalize memories already present in active working state, but that overlap signal affects ranking only; it must not bypass namespace or policy enforcement.
- `--format` changes only the final injection rendering (for example markdown versus plain text). In `--json` mode, the machine-readable shape should still preserve `injections[{memory_id, content, utility_score, token_count, reason}]`, `tokens_used`, `tokens_remaining`, and any warnings or availability state that materially shaped the result.
- Because this is later-stage follow-on work rather than a core-path prerequisite, unavailable or gated deployments should surface explicit feature-availability or policy-denial outcomes instead of silently folding the request into ordinary `recall` behavior.
- Follow-on validation for this surface should at minimum prove deterministic packing under hard token budgets, parity with `recall` for namespace and policy handling, and explicit signaling when budget or feature gating truncates the result.

---

## Snapshots & Branching

### `membrain snapshot [OPTIONS]` (Feature 12)

Snapshot commands manage named historical anchors for later inspection. A snapshot records the current tick and compact metadata for a namespace-scoped checkpoint; it is a time-travel reference, not an alternate authoritative store or full data clone.

Historical inspection against a snapshot should:
- exclude memories created after the snapshot tick,
- recompute time-sensitive strength or freshness using the snapshot tick as historical `now`,
- remain subject to current namespace, policy, redaction, and retained-evidence constraints,
- distinguish fully reproducible historical answers from partial or degraded inspection when later repair loss, retention, or policy changes removed authoritative evidence.

CLI surface:
- `membrain snapshot --name <name> [--note <text>]` creates a named snapshot in the effective namespace.
- `membrain snapshot list` returns snapshot metadata only, suitable for scripting and operator review.
- Operators should create clearly named pre-change snapshots before authoritative rewrites or rollback-sensitive operations when the operations contract requires a restorable safety anchor.
- `membrain snapshot delete <name>` removes the named snapshot handle, but must not silently remove the last restorable safety anchor still required by maintenance or rollback policy.

```bash
membrain snapshot --name before-refactor
membrain snapshot --name v1-launch --note "Day we shipped v1"
membrain snapshot list
membrain snapshot delete before-refactor
```

### `membrain fork / merge [OPTIONS]` (Feature 15)

```bash
membrain fork --name agent-specialist --inherit public
membrain fork list
membrain merge agent-specialist --into default --conflict recency-wins
membrain merge agent-specialist --into default --dry-run
membrain fork abandon experiment
```

---

## Namespaces & Sharing (Feature 9)

```bash
membrain remember "deploy steps" --share --namespace project-x
membrain recall "deploy steps" --namespace project-x
membrain share <uuid> --namespace project-x
membrain unshare <uuid>
membrain namespace list
membrain namespace stats project-x
```

---

## Passive Observation (Feature 6)

Passive Observation is a later-stage intake surface that segments piped or watched content into bounded observation fragments and feeds them through the normal encode pipeline. It improves capture ergonomics, but it does not bypass namespace binding, duplicate handling, provenance rules, or policy checks that would apply to explicit `remember`-style intake.

```bash
cat conversation.txt | membrain observe
echo "user prefers dark mode" | membrain observe --context "preferences"
membrain observe --watch ~/.claude/conversations/
membrain observe --watch ./logs/ --pattern "*.jsonl"
membrain observe --dry-run
```

CLI expectations:
- Observed fragments should preserve explicit provenance. When one observation pass yields multiple memories, each fragment should carry `observation_source` plus a shared `observation_chunk_id` so later inspect, audit, and repair flows can reconstruct where the bounded batch came from.
- Pipe mode and `--watch` mode are transport ergonomics only. Both should still normalize into `Observation` records under one effective namespace and the ordinary intake validation, duplicate-routing, and policy path.
- Segmentation stays bounded by declared chunk-size and topic-shift controls. Watch mode may tail or batch new input, but it must not hide unbounded in-memory buffering or full-history rescans behind a convenience flag.
- `--dry-run` should preview what would be encoded, including estimated fragment counts and any validation, policy, or source-shape blockers, without writing memories.
- Because this is later-stage follow-on work rather than a core-path prerequisite, unavailable or gated deployments should surface explicit feature-availability or policy-denial outcomes instead of silently pretending passive observation is always on.
- Follow-on validation for this surface should at minimum prove deterministic chunking under fixed inputs and thresholds, preserved source labeling and chunk grouping, and semantic parity between pipe/watch intake and the shared observe contract.

---

## Data I/O

Data I/O contract:
- `membrain export` is a policy-aware externalization surface. It should serialize only the effective namespace or shared scope the caller is allowed to read, and it should preserve machine-readable manifest facts such as chosen format, included stores, item counts, omission/redaction summaries, and any historical or degraded anchor that shaped the dump.
- `membrain import` is governed ingest, not a raw storage bypass. Imported records still pass ordinary validation, namespace binding, duplicate handling, provenance tagging, and bounded enrichment or repair rules.
- `--dry-run` on import should return a preview with created/merged/skipped/rejected counts plus any format, policy, or namespace blockers without writing.
- CLI text, CLI `--json`, daemon/JSON-RPC, and MCP transports may package payloads differently, but they must preserve the same import/export semantics, warnings, and outcome class.

### `membrain export`

```bash
membrain export --format json > backup.json
membrain export --format ndjson > backup.ndjson
```

### `membrain import`

```bash
membrain import < backup.json
membrain import --format ndjson < backup.ndjson
```

---

## Repair & Benchmarking

### `membrain repair <SURFACE> [OPTIONS]`

Repair commands preview or rebuild derived surfaces from durable truth without changing the logical meaning of a namespace.

```bash
membrain repair index --dry-run
membrain repair index --namespace default
membrain repair graph --dry-run
membrain repair lineage --namespace default
membrain repair cache --namespace default
```

### `membrain benchmark [TARGET]`

Benchmark and diagnostic coverage should include not only happy-path tier latency but also representative load, rebuild, and migration-sensitive evidence when those paths change.

```bash
membrain benchmark tier1
membrain benchmark tier2
membrain benchmark tier3
membrain benchmark encode
membrain benchmark retrieval
```

---

## Daemon & MCP Server

```bash
membrain daemon start
membrain daemon stop
membrain daemon status
membrain daemon restart

membrain mcp       # start MCP stdio server
```

### Lifecycle and semantic-parity contract

- `membrain daemon` manages the long-lived runtime described by the canonical plan: a warm process that owns the Unix socket / JSON-RPC service surface, keeps model and derived hot structures warm, and runs bounded background maintenance loops outside the foreground request path.
- `membrain daemon status` is the operator-facing readiness surface for that runtime. It should report whether the configured daemon is reachable and, when available, expose runtime facts such as socket path, pid, uptime, and other status details that matter for safe routing and diagnosis.
- `membrain daemon stop` and `membrain daemon restart` preserve graceful lifecycle semantics: stop accepting new work, finish or time-box in-flight requests, flush durable state as required, and remove daemon-owned pid/socket artifacts only after shutdown succeeds.
- Standalone CLI execution remains a first-class correctness mode for scripts, tests, one-off commands, and daemon-unavailable environments. Standalone may pay cold-start or warm-cache costs, but it must not change the logical request, namespace binding, policy checks, boundedness guarantees, or outcome semantics of an equivalent operation.
- When a compatible daemon is reachable, CLI commands may forward to it over the Unix socket; when it is unavailable, the CLI may fall back to standalone execution. That routing choice is a performance and lifecycle distinction, not permission to widen scope, skip safeguards, or alter encode/recall behavior.
- CLI direct mode, CLI-via-daemon, Unix-socket JSON-RPC, and `membrain mcp` are transport adapters over the same underlying operations. Their envelopes may differ, but they must preserve semantic parity for effective namespace, policy denials, explanation families, degraded warnings, and outcome class.
- Operator and data-mobility surfaces such as `stats`, `health`, `doctor`, `audit`, `export`, and `import` must keep the same underlying counters, warnings, remediation, availability, and outcome semantics across those transports even when one surface renders dashboards, streams files, or returns structured envelopes.
- If daemon unavailability or missing warm state materially affects service quality, the surface must say so through warnings or the existing `degraded` / `blocked` outcome vocabulary rather than silently pretending the long-lived service guarantees still held.
- `membrain mcp` is the stdio transport for MCP-capable clients. It should expose the same bounded operations and policy/explain behavior through MCP envelopes rather than inventing alternate command semantics.

## Output Modes and Outcome Classes

Every major command supports:
- Human-readable text (default)
- Structured JSON (`--json`)
- Exit codes: 0=success, 1=validation failure, 2=policy denial, 3=internal error

Output-mode rules:
- Text mode and JSON mode must describe the same command outcome even when they differ in presentation density.
- `--json` should expose warnings, route/explain handles, policy/degraded context, and the stable machine-readable explanation families from the canonical retrieval contract when those details materially affect the outcome. At minimum that means preserving `route_summary`, `result_reasons`, `omitted_summary`, `policy_summary`, `provenance_summary`, `freshness_markers`, `conflict_markers`, and `trace_stages` when the command or requested verbosity makes them relevant, rather than inventing a CLI-only explanation shape.
- `--quiet` may suppress extra narration in text mode, but it must not hide outcome class, actionable warnings, or policy-visible refusal.
- `--verbose` may add explanatory detail in either mode, but only as additive information.
- Command-specific `--format` options may change rendering or file serialization, but they must not redefine the success/failure semantics already represented by text mode and `--json`.

CLI-visible outcome classes:
- **accepted** — the command completed normally
- **rejected** — the command failed validation or was denied by policy; local confirmation would not make the request valid
- **partial** — the command returned a bounded but incomplete result and must say what was omitted or deferred
- **preview** — the command intentionally returned a non-mutating dry-run or inspection of planned work, including `preview-only` preflight states
- **blocked** — the command refused to proceed until another readiness, scope, freshness, or confirmation condition is met
- **degraded** — the command completed through a slower, reduced-fidelity, or repair-aware path and must surface that fact

These classes define what users must be able to distinguish at the CLI layer. The final machine-readable retrieval/result envelope remains owned by `mb-1hw.8`, while the destructive-action safeguard schema and degraded/read-only availability contract remain owned by `docs/OPERATIONS.md`.

### Failure taxonomy, retryability, and remediation contract

Text mode should use stable human-facing failure headlines, and `--json` should preserve the same underlying `error_kind`, `retryable`, optional `remediation`, and `availability` semantics shared with daemon/JSON-RPC and MCP.

| Machine-readable kind | Text headline | Retryable? | CLI exit code | Minimum remediation |
|---|---|---|---|---|
| `validation_failure` | Invalid request | No | 1 | Fix missing, malformed, incompatible, or stale request inputs before rerunning. |
| `policy_denied` | Denied by policy | No | 2 | Narrow scope, change namespace/visibility, or obtain the required approval instead of retrying blindly. |
| `unsupported_feature` | Feature unavailable | No | 1 | Check `health`, feature maturity, build/config support, or required opt-in before retrying. |
| `transient_failure` | Temporary failure | Yes | 3 | Retry with backoff after the temporary condition or contention clears. |
| `timeout_failure` | Timed out | Yes | 3 | Retry with a different effort/budget profile or after load drops; do not treat it as proof of permanent failure. |
| `corruption_failure` | Corruption detected | No | 3 | Run `doctor`, inspect the affected scope, and complete repair/restore before trusting retries or writes. |
| `internal_failure` | Internal failure | No | 3 | Inspect diagnostics and repair state; do not spin in blind retry loops. |

Rules:
- `--json` should preserve `error_kind`, `retryable`, and `remediation` even when text mode shortens the wording.
- `remediation` should include a short summary plus machine-readable next-step hints such as `fix_request`, `change_scope`, `check_health`, `retry_with_backoff`, `retry_with_higher_budget`, `run_doctor`, or `run_repair`.
- `degraded` is an outcome class rather than an `error_kind`: the command may still succeed, but text and JSON must say what remained queryable, what became read-only, and which actions were blocked instead of silently downgrading semantics.
- If only part of the requested result survived budgets, policy filters, fallback, or degraded serving, use `partial` or `degraded` with explicit omission/deferred context instead of flattening the response into a generic success message.
