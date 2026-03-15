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
- the detailed error/remediation taxonomy from `mb-1hw.9`
- the destructive preflight / blocked-action schema from `mb-1hd.7`

Those sibling contracts may add payload fields or stronger safeguards later, but they should plug into the command and outcome vocabulary defined here rather than redefining it.

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

At the CLI layer, destructive or high-blast-radius commands should follow these ergonomics even before the fuller safeguard contract is specified elsewhere:
- default to the narrowest explicit scope rather than silently widening namespace, history, or maintenance coverage
- prefer `--dry-run` or another preview path before state-changing repair, merge, invalidation, compression, or deletion-adjacent work
- surface a **blocked** outcome when confirmation, scope, or readiness conditions are missing instead of guessing or proceeding implicitly
- treat `--force` as a local confirmation override only; it does not bypass policy, namespace, or safety invariants
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
| `--at` | — | Recall at named snapshot (`at_snapshot`) |
| `--namespace` | caller default if deterministic, otherwise required | Namespace scope |
| `--include-public` | false | Widen only to policy-approved shared/public surfaces |
| `--explain` | summary | Explain verbosity: none, summary, full; summary explains why results appeared and what major route/policy/budget boundaries mattered, while full adds routing-trace stages and exclusion details |
| `--cold-tier` | auto | Cold-tier routing hint: avoid, auto, allow |

Normalization and safety rules:
- `<QUERY>` may be omitted only when `--like` or `--unlike` provides the primary cue.
- `--as-of` and `--at` must not be combined unless a later contract defines deterministic precedence.
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

### `membrain strengthen <ID>`

Manually apply LTP to a memory (same effect as on_recall).

### `membrain update <ID> <NEW_CONTENT>`

Submit a pending update during reconsolidation window.

### `membrain stats [OPTIONS]`

Brain health statistics.

```bash
membrain stats
membrain stats --json
membrain stats --at before-refactor    # stats at snapshot point
```

### `membrain inspect <ID> [OPTIONS]`

Full memory details: tier, lineage, policy, lifecycle, graph neighborhood, decay, cache-related routing metadata when relevant, provenance summary, freshness markers, and conflict state.

```bash
membrain inspect <uuid>
membrain inspect <uuid> --history       # belief version chain (Feature 2)
membrain inspect <uuid> --show-source   # source engram for procedurals (Feature 8)
```

### `membrain doctor`

Diagnose brain health: orphan edges, missing embeddings, stale indexes, broken lineage, checkpoint corruption.

```bash
membrain doctor run
membrain doctor run --json
```

---

## Consolidation & Maintenance

### `membrain consolidate`

Manually trigger NREM+REM+Homeostasis cycle.

### `membrain compress [OPTIONS]` (Feature 17)

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

```bash
membrain uncertain                    # memories with confidence < 0.5
membrain uncertain --top 20
```

---

## Belief & Knowledge Management

### `membrain beliefs [OPTIONS]` (Feature 2)

```bash
membrain beliefs "user preferences"   # show belief chain
membrain beliefs --conflicts          # list contradictions
membrain beliefs --resolve <id>       # resolve pending conflict
```

### `membrain why <ID> [OPTIONS]` (Feature 11)

Trace causal and retrieval rationale for one item, including lineage and route ancestry when available.

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

```bash
membrain budget --tokens 2000
membrain budget --tokens 2000 --context "debugging" --format markdown
```

---

## Snapshots & Branching

### `membrain snapshot [OPTIONS]` (Feature 12)

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

```bash
cat conversation.txt | membrain observe
echo "user prefers dark mode" | membrain observe --context "preferences"
membrain observe --watch ~/.claude/conversations/
membrain observe --watch ./logs/ --pattern "*.jsonl"
membrain observe --dry-run
```

---

## Data I/O

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

## Output Modes and Outcome Classes

Every major command supports:
- Human-readable text (default)
- Structured JSON (`--json`)
- Exit codes: 0=success, 1=validation failure, 2=policy denial, 3=internal error

Output-mode rules:
- Text mode and JSON mode must describe the same command outcome even when they differ in presentation density.
- `--json` should expose warnings, route/explain handles, and policy/degraded context in machine-readable form when those details materially affect the outcome.
- `--quiet` may suppress extra narration in text mode, but it must not hide outcome class, actionable warnings, or policy-visible refusal.
- `--verbose` may add explanatory detail in either mode, but only as additive information.
- Command-specific `--format` options may change rendering or file serialization, but they must not redefine the success/failure semantics already represented by text mode and `--json`.

CLI-visible outcome classes:
- **accepted** — the command completed normally
- **rejected** — the command failed validation or was denied by policy
- **partial** — the command returned a bounded but incomplete result and must say what was omitted or deferred
- **preview** — the command intentionally returned a non-mutating dry-run or inspection of planned work
- **blocked** — the command refused to proceed until another readiness, scope, or confirmation condition is met
- **degraded** — the command completed through a slower, reduced-fidelity, or repair-aware path and must surface that fact

These classes define what users must be able to distinguish at the CLI layer. The final machine-readable result envelope, detailed remediation taxonomy, and destructive-action safeguard schemas remain owned by `mb-1hw.8`, `mb-1hw.9`, and `mb-1hd.7`.
