# MISSING.md — Spec vs implementation gap audit

This document captures the current delta between the intended Membrain architecture described in `docs/PLAN.md` / `AGENTS.md` and the implementation currently reachable through the CLI, daemon, and MCP surfaces.

The goal is not to restate the full plan. The goal is to identify the missing runtime properties that must exist before we can honestly say Membrain behaves like the designed local-memory system rather than primarily a persistence-backed store with partial planning/explain surfaces.

---

## Executive summary

Membrain already has important building blocks:

- local persistence under `~/.membrain`
- CLI commands for remember/recall/inspect/why/health/doctor
- daemon and MCP entry points
- a real `fastembed` wrapper in core code
- retrieval planning / explainability scaffolding
- health, cache, and observability types

However, the current implementation still falls short of the target architecture in a smaller set of material ways than before:

1. **Normal daemon/MCP recall and explain now describe success paths as hydrated runtime evidence, while planner/degraded wording is reserved for explicit no-hydrated-evidence or other degraded cases.**
2. **The embedding runtime is now operationally proven on the main daemon path: health/doctor/runtime surfaces expose warm embedder reuse with load/request/cache counters after real recall traffic.**
3. **CLI / daemon / MCP semantics are now materially closer, and the main semantic-recall contract is backed by explicit parity artifacts rather than only architectural intent.**
4. **The runtime now shows bounded cognitive behavior on real surfaces instead of only persistence/query: maintenance can project retained evidence onto cold and reconsolidating wrapper paths, and recall/why expose those lifecycle consequences explicitly.**
5. **The earlier restore-oriented parity gap is now closed: transport-level proof now shows `archival_recovery_partial` surviving daemon and stdio MCP retrieval envelopes only when partial restore state truly shaped the result, while ordinary cold recall remains `cold_consolidated` instead of inventing that marker.**

This gap matters because the docs explicitly promise a stronger model:

- `docs/PLAN.md:1185` — `fastembed is the default local embedding layer`
- `docs/PLAN.md:10340` — embedding model wrapper `Loads model once, shared across threads`
- `docs/PLAN.md:10348-10357` — encode/recall/lifecycle architecture assumes an active bounded runtime, not just durable storage
- `docs/PLAN.md:9759` — doctor output explicitly expects a daemon-dependent embedding runtime state: `Embedding model: NOT LOADED (daemon not running)`
- `AGENTS.md` describes daemon mode as the background local socket service and MCP mode as stdio subprocess integration, implying coherent behavior across modes.

---

## Gap label scoreboard

| label | current | expected | status |
|---|---|---|---|
| `gap.audit_wording_planner_drift` | Normal daemon/MCP success paths are now described as hydrated runtime evidence, with degraded wording scoped to explicit fallback states. | Planner/degraded wording only appears for explicit degraded/no-hydrated-evidence cases. | done |
| `gap.embedder_daemon_authority` | Daemon runtime status/health/doctor now expose machine-readable embedder state plus load/request/cache counters, and regression coverage proves warm reuse on the real daemon recall path. | Health/doctor/runtime posture clearly proves canonical warm embedder lifecycle and reuse. | done |
| `gap.cross_surface_semantic_parity` | Core, daemon, and CLI proof coverage now all show the same realistic semantic query preferring the semantically right record over a lexical distractor while keeping hydrated evidence and non-degraded packaging on normal success paths. | Same durable visibility and explain contract across all surfaces unless policy explicitly differs. | done |
| `gap.cognitive_runtime_vs_db_query` | Runtime now shows bounded retrieval, rerank, maintenance-driven hot/cold lifecycle projection, reconsolidation, deterministic forgetting policy proofs, and hydrated evidence on real daemon and CLI proof surfaces rather than only persistence/query behavior. | Runtime behavior visibly reflects bounded hot/cold memory, rerank, reconsolidation, forgetting, and evidence hydration. | done |
| `gap.daemon_authoritative_runtime_posture` | Runtime status/doctor/health now distinguish `unix_socket_daemon` authority from stdio facade mode, and parity proof covers machine-readable unix-socket status/health/doctor operator artifacts plus stdio degradation semantics. | Operator surfaces truthfully distinguish stdio MCP vs long-lived daemon and expose meaningful warm-runtime authority. | done |
| `gap.restore_archival_recovery_projection` | Shared core freshness/explain packaging emits `archival_recovery_partial` when a partial archival recovery path actually shapes the returned envelope, daemon/runtime records can now persist and replay that projected freshness state, and regression proof covers unix-socket plus stdio MCP transport preserving the marker while ordinary cold recall stays `cold_consolidated`. | Shared surfaces expose archival recovery loss states like `archival_recovery_partial` when they materially affect results. | done |
| `gap.freshness_marker_contract_completion` | Shared recall/why freshness markers now consistently emit the applicable retrieval-time contract (`lifecycle_projection`, `snapshot_scoped`, `as_of_scoped`, `lease_sensitive`, `recheck_required`, `stale_derived`) without over-claiming inspect/restore-only archival recovery states. | Shared freshness markers cover the full applicable documented contract across recall/why wrappers. | done |
| `gap.docs_truth_and_parity_artifacts` | MISSING/docs/tests now align on hydrated success paths, bounded live MCP tools, stdio-vs-daemon runtime authority, semantic parity proof artifacts for the main recall path, and the narrowed remaining archival-recovery parity gap. | MISSING/docs/tests all consistently reflect the real contract with regression proof for unresolved runtime claims. | done |

---

## Audit framing

This audit uses the following standards:

The implementation guidance here may borrow good runtime-memory design patterns from external reference systems, but this document intentionally avoids naming those systems directly. Only the transferable architecture and operational lessons matter.

### What “matches spec” means here

To count as aligned with the design, Membrain must satisfy all of the following in practice, not just in static types or placeholder APIs:

1. **Background runtime is real**
   - daemon can act as the authoritative long-lived process
   - model/state/cache warmup is meaningful over repeated requests
   - doctor/health can truthfully report this runtime posture

2. **Embedding is part of the canonical retrieval path**
   - `fastembed` is not merely compiled in, but actually used by the runtime path expected in docs
   - recall quality and candidate generation materially depend on the embedder where the spec says they should
   - model lifetime and cache semantics reflect the docs

3. **Evidence hydration is complete**
   - recall and explain results are backed by actual hydrated runtime evidence
   - planner/explain envelopes are not returned in place of materialized evidence except in explicitly degraded modes

4. **Surfaces are semantically aligned**
   - CLI, daemon JSON-RPC, and MCP stdio observe the same persisted state and retrieval semantics
   - the user should not have to guess which surface is “more real”

5. **Human-like memory mechanisms are not just aspirational text**
   - background lifecycle behavior, reconsolidation, bounded hot/cold behavior, and retrieval/rerank semantics have real implementation consequences
   - the user experience should reflect more than bare persistence + lookup.

---

## What is already present

### 1. Embedding backend exists in code

Evidence:

- `crates/membrain-core/src/embed.rs:164` defines `FastembedTextEmbedder`
- `crates/membrain-core/src/embed.rs:175-194` constructs the `TextEmbedding` backend
- `crates/membrain-core/src/embed.rs:203-262` exposes single/batch embedding operations

Why this matters:

This is not a fake dependency. There is a legitimate embedding abstraction and a real `fastembed`-backed implementation.

### 2. Docs clearly intend local embedding as default

Evidence:

- `docs/PLAN.md:1185-1199` says `fastembed` is the default local embedding layer and outlines cache/generation behavior
- `docs/PLAN.md:10338-10340` says the embedding model wrapper loads once and is shared across threads

Why this matters:

The design intent is unambiguous: embeddings are not optional decoration; they are central to the canonical architecture.

### 3. Daemon/background concept exists

Evidence:

- `AGENTS.md` documents `membrain daemon` / `membrain-daemon` as a background Unix-socket service
- `docs/PLAN.md:9759` references daemon-running state in doctor output

Why this matters:

The project is explicitly designed around an active runtime, not only per-command subprocesses.

### 4. MCP protocol support now exists

Current fixes already landed recently added:

- standard MCP initialize handshake
- proper notification handling
- usable tool registration/listing
- persisted memory hydration into MCP startup path

Why this matters:

This removes one class of “MCP isn’t even speaking the protocol” blockers. But protocol compliance is not the same thing as architecture completion.

---

## Major gaps

## Gap A — docs/bead state must keep normal recall/explain success wording tied to hydrated runtime evidence

### Evidence

The live daemon code and tests now show hydrated retrieval on normal paths, and the audit/docs wording has been tightened to keep planner/degraded language scoped to explicit fallback states:

- `crates/membrain-daemon/src/daemon.rs:8513` — semantic recall test asserts hydrated `evidence_pack` and `degraded_summary == null`
- `crates/membrain-daemon/src/daemon.rs:8591` — restart hydration test covers inspect/recall/explain over persisted state
- `crates/membrain-daemon/src/daemon.rs:9280` — restart parity test covers recall/inspect/why with persisted runtime evidence
- degraded summaries remain only for explicit no-hydrated-evidence cases

### Why this mattered

This section was primarily a docs-truth problem rather than a missing normal-path hydration implementation.

The spec describes a retrieval pipeline that should actually:

- do tiered retrieval
- rescore candidates
- use engram expansion / deeper tiers as needed
- produce materialized evidence
- make explainability describe a real retrieval outcome

The live daemon path now hydrates evidence on normal success paths, so the key requirement is keeping docs and audit text aligned with that runtime truth. Planner/degraded language should remain reserved for explicit no-hydrated-evidence, fallback, or failure cases.

### What “done” means here

To keep this gap closed:

- recall/explain success paths must continue to describe hydrated evidence on the normal daemon/MCP path
- planner/degraded responses should remain restricted to explicit degraded/failure modes and surfaced as such
- tests and doc examples must keep proving that a persisted memory can be:
  - recalled via CLI
  - recalled via daemon JSON-RPC
  - recalled via MCP stdio
  - explained consistently via `why` / explain surfaces

### Failure modes to guard against

- docs regress and describe normal success as planner-only again
- explain examples drift back toward hypothetical retrieval wording instead of actual evidence
- MCP result shape looks complete but hides empty evidence packs without clearly degraded state
- regression where exact-id works but semantic query still falls back to planner-only behavior

---

## Gap B — daemon embedder authority is now proven on the live runtime path

### Evidence

- `crates/membrain-core/src/embed.rs` contains the real fastembed wrapper
- `docs/PLAN.md:10340` expects “load once, shared across threads” behavior
- `docs/PLAN.md:9759` explicitly models a daemon-dependent loaded/not-loaded state
- `crates/membrain-daemon/src/daemon.rs:421-460` publishes daemon authority mode and `warm_runtime_guarantees`, including `repeated_request_warmth`
- `crates/membrain-daemon/src/daemon.rs:473-525` derives machine-readable embedder state from live load/request/cache counters
- `crates/membrain-daemon/src/daemon.rs:1017-1042` and `crates/membrain-daemon/src/daemon.rs:1369-1386` surface embedder posture and counters in health/doctor feature availability and checks
- `crates/membrain-daemon/tests/runtime_doctor_parity.rs:613-691` warms the embedder via encode/recall before asserting health output
- `crates/membrain-daemon/src/daemon.rs:8954-9011` proves the real daemon recall path leaves the embedder `warm`, records one load, and surfaces cache hit/miss counters plus semantic query counters through status and doctor

### Why this mattered

The repo already had the embedder and operator counters, but the crucial question was whether the live daemon-owned recall path actually exercised them in a way operators could verify.

That proof now exists directly in daemon-path regression coverage rather than only by inference from source code.

### What is now true

- the daemon-owned recall path measurably exercises the canonical embedder
- status/health/doctor can distinguish not-loaded versus warm runtime posture
- repeated retrieval work is visible through load/request/cache counters rather than hidden behind a black box
- the live proof now answers “is fastembed actually running?” with runtime evidence, not just architectural intent

### Remaining related work

This closes the embedder-authority proof row, but it does not close broader product questions about how much of the larger cognitive-runtime thesis is already user-visible.

---

## Gap C — core, daemon, and CLI now prove normal-path semantic parity for the main retrieval contract

### Evidence

Recent debugging exposed a real split and the repo now carries explicit proof that the main path changed:

- `crates/membrain-cli/tests/cli_end_to_end.rs:97-164` proves restart persistence parity for inspect/recall/why on the CLI surface
- `crates/membrain-cli/tests/cli_end_to_end.rs:330-378` proves explain output retains semantic trace fields on the CLI surface
- `crates/membrain-cli/tests/cli_end_to_end.rs` now includes a realistic semantic-recall regression where the semantically right deployment/remediation memory outranks a lexical distractor on the CLI path
- `crates/membrain-daemon/src/daemon.rs:8928-8952` proves the real daemon recall path returns a hydrated semantic winner with `entry_lane=semantic` and shared semantic-executor trace reasons
- `crates/membrain-daemon/src/daemon.rs:9861-9987` proves restart parity for recall/inspect/why on the daemon/MCP side
- `crates/membrain-core/tests/recall_pipeline_integration.rs:830-938` and `crates/membrain-core/tests/retrieval_quality_proof.rs:123-159` prove the shared core retrieval path prefers the semantically right record over the lexical distractor

### Why this mattered

The spec/docs imply that CLI, daemon JSON-RPC, and MCP are multiple access surfaces over one coherent local memory runtime. The user should not have to guess which path is “more real.”

The old persisted-state split made that concern legitimate. The current proof set now demonstrates that the main success-path retrieval contract is shared rather than merely described that way.

### What is now true

- durable memories visible to CLI recall/inspect are also visible through daemon/MCP restart parity coverage unless policy says otherwise
- normal recall success returns hydrated evidence instead of planner-only placeholders on both CLI and daemon/MCP paths
- a realistic semantic query now proves the main retrieval path prefers the semantically right record over a lexical distractor across core, daemon, and CLI proof surfaces
- health/doctor authority wording remains aligned with the transport distinction between stdio MCP and the long-lived daemon

### Remaining related work

The open runtime questions are now narrower than cross-surface semantic parity itself. Remaining gaps live in broader product-thesis rows such as archival recovery projection and the still-partial cognitive-runtime story.

---

## Gap D — runtime now demonstrates bounded cognitive behavior on real surfaces

### Evidence

This row is now closed with real runtime evidence instead of only architectural intent.

What is already user-visible and proven:

- normal recall now returns hydrated evidence rather than planner-only success envelopes
- realistic semantic-recall proof shows the semantically right record beating a lexical distractor across core, daemon, and CLI paths
- health/doctor/runtime status expose warm embedder state, cache/load counters, and daemon-versus-stdio authority posture
- maintenance/lifecycle parity tests now prove both the operator-side maintenance log surface and a real daemon-path lifecycle projection where maintenance moves one retained result onto the cold wrapper path while another remains reconsolidating on recall/why
- forgetting policy proofs remain deterministic and replayable rather than depending on wall-clock timing, so lifecycle consequences stay inspectable instead of magical

### Why this mattered

The earlier complaint was that Membrain still felt like "save to SQLite, then query it back." That is no longer the best description of the current runtime because maintenance, lease policy, reconsolidation, semantic reranking, and hydrated explainability now change what the user actually sees.

### What is now true

- background lifecycle jobs now have real user-visible impact on the daemon recall/why path, not only on operator logs
- recall ranking still uses bounded semantic retrieval and hydrated evidence rather than compact-text scan alone
- hot/cold and reconsolidation state now visibly alter `answered_from`, `entry_lane`, `result_reasons`, and freshness markers on the accepted daemon path
- CLI, daemon, and MCP proof surfaces all show the main retrieval contract with bounded runtime artifacts rather than planner placeholders
- the remaining unresolved retrieval/docs work is narrower and mostly restore-marker parity, not the broader "this is just DB query" complaint

---

## Gap E — daemon is not yet the clearly authoritative always-on operating mode the docs imply

### Evidence

The docs place meaningful weight on daemon mode:

- daemon is the socket server
- doctor output reasons about daemon-dependent embedding state
- background lifecycle work is documented as daemon-owned
- `crates/membrain-daemon/src/daemon.rs:421-450` publishes `authority_mode`, `authoritative_runtime`, and `warm_runtime_guarantees`
- `crates/membrain-daemon/src/daemon.rs:1002-1015` and `crates/membrain-daemon/src/daemon.rs:1332-1345` surface runtime authority in health/doctor operator reports
- `crates/membrain-daemon/src/daemon.rs:2154-2224` explicitly sets stdio MCP to `stdio_facade` and Unix socket runtime to `unix_socket_daemon`
- `docs/MCP_API.md:12-13` and `docs/MCP_API.md:667-669` now state that stdio MCP gets only process-local reuse while daemon owns repeated-request warm-runtime guarantees

The remaining proof gap is now closed: unix-socket resource/status, health, and doctor parity artifacts all assert machine-readable authority fields, while stdio coverage keeps the non-authoritative degradation semantics explicit.

### Why this is a problem

The architecture feels under-committed if:

- daemon exists but is not clearly the source of truth for warm runtime state
- MCP subprocess behaves like a shallower alternate path
- health/doctor/runtime semantics differ depending on how the user entered the system

### What “done” must mean

This gap is now closed:

- daemon is the explicitly authoritative warm runtime, and unix-socket resource/status plus health/doctor surfaces assert that machine-readably
- stdio MCP remains a non-authoritative facade with explicit degraded runtime-authority reporting and best-effort same-process reuse only
- parity artifacts now prove which mode is active and what warm-runtime guarantees are currently in effect

### Failure modes to document in work

- daemon path and MCP path both partially complete in different ways
- docs imply always-on runtime benefits but common user path doesn’t get them
- “daemon not running” becomes a hidden architecture fork instead of an explicit posture

---

## Secondary gaps

## Gap F — tool and docs surface can still mislead agents about what is actually implemented

### Evidence

MCP tooling now advertises useful tools, and the normal recall/why paths are no longer planner-only on success. The remaining mismatch is smaller now:

- `docs/MCP_API.md:8-18` now distinguishes the bounded six callable MCP tools, slash-style MCP discovery methods, placeholder prompt surfaces, stdio direct JSON-RPC compatibility methods, and daemon-only dotted discovery helpers
- `docs/MCP_API.md:667-670` distinguishes stdio MCP process-local reuse from daemon-owned repeated-request warmth
- `docs/CLI.md:167-181` now states the live stdio MCP contract in bounded terms instead of treating every transport helper as part of the callable tool catalog
- transport regression proof now covers both the positive and negative archival-recovery marker cases on daemon and stdio MCP paths

### Why this matters

If tools are described as if they fully implement the intended semantics or as if every mode has the same warm-runtime guarantees, agents will over-trust the system and generate low-quality explanations. The current risk is less about normal-path hydration and more about overstating the MCP tool catalog, health/doctor authority, or stdio-vs-daemon guarantees.

### What “done” must mean

- tool docs/descriptions should reflect current behavior accurately until full implementation lands
- once full implementation lands, descriptions can be tightened around the completed semantics
- avoid claiming stronger retrieval guarantees than the runtime can deliver

---

## Gap G — operational validation story is not yet strong enough

### Why this matters

A system like this needs more than unit tests. It needs proof that the runtime architecture behaves correctly under realistic usage.

### What “done” must mean

Need explicit validation for:

- daemon startup with persisted state
- embedder load/warm state
- repeated recalls showing warm-cache/runtime reuse
- cross-surface parity, including degraded archival-recovery marker parity when partial restore state is surfaced
- degraded-mode visibility
- doctor/health truthfulness
- end-to-end scripts with detailed logs proving the intended runtime model

---

## Concrete missing capabilities checklist

The following are the capabilities still missing or not yet proven strongly enough:

- [x] daemon recall path hydrates evidence on the standard semantic-query path without planner-only fallback
- [x] explain path is backed by actual hydrated evidence rather than route-only scaffolding
- [x] daemon startup loads persisted memories and yields semantic parity with CLI recall/inspect/why
- [x] embedder load/warm state is observable and truthful in doctor/health
- [x] embedder is reused across daemon requests as the spec promises, with remaining work focused on clearer parity proof coverage
- [x] retrieval quality on main paths materially depends on embeddings/reranking rather than persistence-only fallback behavior
- [x] lifecycle/background jobs materially affect user-visible recall behavior through maintenance logs and lifecycle projections, though the broader cognitive-runtime promise is still only partially realized
- [x] daemon/MCP/CLI share one coherent runtime contract on the main hydrated recall/inspect/why path, with remaining transport-proof work narrowed to partial archival-recovery marker parity
- [x] tool descriptions and operational docs reflect what is really implemented today, including hydrated success paths vs explicit degraded fallbacks
- [x] end-to-end tests validate the above in realistic workflows

---

## Transferable runtime-memory patterns to adopt

The implementation guidance below is informed by studying a more mature runtime-memory system, but the patterns are intentionally described generically and without naming that source directly.

### 1. Memory must be a live subsystem, not just a helper library

A strong runtime-memory system separates:

- fast foreground capture
- durable append/persistence
- asynchronous semantic interpretation
- retrieval/materialization APIs
- health/readiness/queue observability

Why this matters:

If capture and semantic reasoning are fused into one synchronous path, the system becomes fragile, slow, and hard to operate. A live memory subsystem lets Membrain act like runtime infrastructure rather than a pile of helper functions.

Implication for Membrain:

- daemon should own warm state, worker lifecycle, and semantic processing
- MCP/CLI should be facades over the same memory substrate, not alternate partial implementations
- health and doctor should report subsystem truth, not static guesses

### 2. Foreground ingest should be cheap, durable, and append-first

A mature design does not require semantic summarization to succeed before data is captured.

Recommended shape:

1. append raw event durably
2. assign correlation/session/task identifiers
3. enqueue semantic work
4. process asynchronously
5. publish status to health/observability surfaces

Why this matters:

This avoids user-facing latency and ensures the system never loses the raw record even when enrichment is delayed or partially degraded.

Implication for Membrain:

- write path should not depend on full semantic completion
- persisted artifacts should survive process restarts cleanly
- degraded mode should preserve raw truth even if enrichment is postponed

### 3. Memory should have staged representations, not one blob

A mature runtime-memory system distinguishes at least these layers:

- raw events / original interactions
- structured observations / extracted facts
- summaries / checkpoints / current state
- embedding/search shards / rankable units
- context-injection renderings / compact operator views

Why this matters:

This is one of the main architectural differences between a memory runtime and a simple note store. Each surface serves a different purpose and should not be collapsed into the same generic stored row.

Implication for Membrain:

- retrieval should not only return compact text rows
- context injection should be a deliberate render target
- summary/checkpoint artifacts should be first-class, not merely derived strings

### 4. Progressive disclosure should be the default retrieval pattern

A strong retrieval pipeline does not jump straight from query to full payload hydration.

Recommended progression:

- lookup/search: compact ranked handles and metadata
- context/timeline: neighborhood around promising items
- hydrate: materialize selected full memories in chosen order
- source: fetch original raw artifacts only when truly needed

Why this matters:

This preserves context budget, prevents over-hydration, and makes agent behavior more disciplined.

Implication for Membrain:

- recall should support cheap ranking and explicit follow-up hydration
- inspect/source/timeline semantics should be distinct and composable
- explainability should be able to refer to both ranked handles and hydrated evidence

### 5. Canonical storage should remain separate from semantic ranking artifacts

A mature design treats relational/durable storage as the source of truth and vector data as a derived search accelerator.

Why this matters:

This keeps migration, filtering, schema evolution, and observability tractable. Embedding/index layers can change without redefining the canonical memory object.

Implication for Membrain:

- hot/cold durable memory should remain canonical
- embedding vectors and search indexes should be rebuildable derived state
- hydrated results should come from canonical store records, not vector-store payload copies

### 6. Embed semantic units, not only giant records

A mature design embeds smaller semantic shards such as:

- facts
- decisions
- rationale
- learned constraints
- next steps
- summary fields

Why this matters:

Embedding one coarse blob per session or memory often produces retrieval that feels like archive lookup instead of semantic recall.

Implication for Membrain:

- consider embedding structured subdocuments rather than only full memory payloads
- preserve backpointers from shards to canonical memory/session ids
- use these shards to improve recall quality and later hydration choices

### 7. Session identity and runtime identity should be distinct

A practical runtime-memory system separates:

- user/session/thread identity
- ingestion stream identity
- semantic worker/runtime identity

Why this matters:

Workers restart, providers rotate, and semantic processing may resume independently from the user-visible session.

Implication for Membrain:

- do not over-collapse all runtime behavior into one session id
- let daemon/worker lifecycle be observable without corrupting canonical session meaning
- support recovery/restart without semantic ambiguity

### 8. Hydration should be explicit, ordered, and rank-preserving

Hydration is not just “query again.” It is a materialization step over chosen handles.

Why this matters:

A robust system must preserve the ordering chosen by ranking while still loading canonical full records.

Implication for Membrain:

- hydration APIs should accept ordered ids/handles
- results should preserve caller rank order
- related expansions (timeline, linked decisions, sources) should be opt-in rather than always loaded

### 9. Cross-surface semantics must come from one substrate

A mature memory system supports many surfaces — hooks, agent tools, APIs, UI/debug views — while preserving one semantic contract.

Why this matters:

This is exactly where Membrain has already shown real regressions: CLI and MCP initially saw different memory visibility. That should be structurally impossible in the final design.

Implication for Membrain:

- CLI, daemon, MCP, and any future UI should share the same ids, ranking contract, hydration rules, and policy semantics
- tool-specific wrappers may differ in formatting, not in truth

### 10. Observability must report daemon/runtime truth, not inferred truth

A mature long-lived memory subsystem reports:

- liveness
- readiness
- worker/backlog state
- model/embedder state
- version compatibility
- last semantic processing errors
- queue health and recent activity

Why this matters:

A system that relies on warm state and background processing cannot be debugged with static storage checks alone.

Implication for Membrain:

- doctor/health must prove whether daemon mode is actually delivering the runtime guarantees promised in docs
- embedder loaded/warm status, queue depth, degraded mode, and parity issues should be explicit

## Study conclusions — what to adopt, adapt, and reject

The local comparative study supports the broad direction already captured above, but sharpens it in a few concrete ways.

### Adopt directly

1. **One authority per truth domain**
   - Runtime mode, warm state, queue state, and semantic readiness need one canonical owner.
   - For Membrain, that means daemon-owned runtime truth rather than inferring readiness from static storage alone.
   - Downstream bead implication: `mb-2lye` defines the authority contract and `mb-2wb3` should only operationalize warm embedder behavior once that authority contract is already truthful.
   - Logging obligation: unit and e2e coverage should emit/assert active mode, readiness class, queue depth/backlog posture, embedder loaded/warm state, and the reason a surface is degraded when runtime truth is absent.
   - Downstream beads: `mb-2lye`, `mb-2wb3`

2. **Materialization must be explicit and rank-preserving**
   - Ranked candidate selection and hydrated evidence should be different stages with explicit boundaries.
   - This directly supports replacing planner-only envelopes with evidence-backed recall/explain paths.
   - Downstream bead implication: `mb-uw37` owns normal-path recall hydration and `mb-1ps7` should explain the actual hydrated result set rather than the route skeleton alone.
   - Logging obligation: traces and tests should record candidate ids before hydration, the final hydrated ids in preserved order, and any omission/defer reason that kept a ranked handle from becoming full evidence.
   - Downstream beads: `mb-uw37`, `mb-1ps7`

3. **Cross-surface wrappers may differ in format, not in truth**
   - CLI, daemon, and MCP should share one substrate and one contract for ids, ranking, hydration, and omission/degraded semantics.
   - Downstream bead implication: `mb-3g6d` is a parity-contract bead, while `mb-2lye` must make clear which runtime mode owns the truth those surfaces are exposing.
   - Logging obligation: parity tests should compare the same request across CLI, daemon, and MCP using stable ids, outcome class, omission/degraded markers, and hydration state rather than transport-specific prose.
   - Downstream beads: `mb-3g6d`, `mb-2lye`

4. **Observability must describe runtime posture, not optimistic assumptions**
   - Health/doctor output needs to expose mode, readiness, queue/embedder posture, degradation, and recent semantic failures.
   - Downstream bead implication: `mb-2lye` owns truthful runtime posture, `mb-2wb3` inherits the warm-embedder reporting contract, and `mb-3drx` should only document guarantees that these checks already prove.
   - Logging obligation: doctor/health scenarios should capture daemon-running, daemon-absent, and stdio/MCP entry cases with explicit evidence for which runtime guarantees are active versus unavailable.
   - Downstream beads: `mb-2lye`, `mb-2wb3`, `mb-3drx`

### Adapt rather than copy

1. **Append-first ingest, but sized for a local-first tool**
   - The pattern is correct: capture should stay cheap and durable, with slower semantic work decoupled behind the runtime.
   - Membrain should adapt this to a local daemon/subprocess footprint rather than introducing unnecessary distributed queue machinery.
   - Downstream bead implication: `mb-2lye` should keep runtime ownership explicit without inventing distributed coordination, and `mb-1jho` should only leverage that background processing where it changes user-visible behavior.
   - Downstream beads: `mb-2lye`, `mb-1jho`

2. **Staged representations, but keep the initial cut minimal**
   - Raw records, structured observations, summaries/checkpoints, and rankable semantic shards are useful distinctions.
   - Membrain should implement only the smallest set of stages needed to make recall, why, and lifecycle behavior visibly better than persistence-plus-query.
   - Downstream bead implication: `mb-uw37` should focus on evidence materialization first, `mb-21vk` can test richer rankable units later, and `mb-1jho` should only count lifecycle stages that surface externally.
   - Downstream beads: `mb-uw37`, `mb-21vk`, `mb-1jho`

3. **Progressive disclosure retrieval, but without forcing every client through a multi-call protocol**
   - Internally, retrieval should still separate ranking, optional expansion, and hydration.
   - Externally, Membrain can present a simple default recall API as long as explainability and logs preserve those stage boundaries.
   - Downstream bead implication: `mb-uw37` should finish one-shot recall hydration, `mb-1ps7` should surface the staged reasoning behind that result, and `mb-3drx` should document the one-call UX without erasing the internal stage boundaries.
   - Validation obligation: the shared retrieval envelope should carry stable `outcome_class`, `evidence_pack`, `omitted_summary`, and `deferred_payloads` semantics even when a client chooses the simplest one-shot surface.
   - Downstream beads: `mb-uw37`, `mb-1ps7`, `mb-3drx`

4. **Semantic shards, but only where they improve retrieval quality measurably**
   - Smaller units such as facts, decisions, rationale, and next steps are likely to outperform single large blobs.
   - Membrain should prove this with comparative tests before broadening storage/index complexity.
   - Downstream bead implication: `mb-21vk` should remain an evidence-gathering/test bead first, not a storage-complexity expansion bead.
   - Validation obligation: tests should compare whole-record ranking against shard-backed ranking and log which stage or candidate family changed the final evidence set.
   - Downstream beads: `mb-21vk`

### Reject or defer for now

1. **Do not introduce a second canonical store for semantic artifacts**
   - Derived embedding/ranking state should remain rebuildable; canonical truth must stay in Membrain's durable memory records.
   - Downstream bead implication: `mb-2wb3` should harden the warm embedder path as derived runtime state, and `mb-uw37` should hydrate evidence from canonical records rather than from secondary vector payload copies.
   - Downstream beads: `mb-2wb3`, `mb-uw37`

2. **Do not over-model worker topology or distributed orchestration**
   - Membrain's current gap is not lack of infrastructure breadth; it is lack of truthful runtime authority and hydration completion.
   - Downstream bead implication: `mb-2lye` should close the authority gap with the smallest truthful runtime model that satisfies health/doctor and warm-state guarantees.
   - Downstream beads: `mb-2lye`

3. **Do not expose planner skeletons as if they were completed retrieval evidence**
   - Route scaffolding is useful internally and in degraded/debug output, but not as the default success result.
   - Downstream bead implication: `mb-uw37` and `mb-1ps7` should retire planner-only normal-path success responses, while `mb-3drx` should document planner skeletons only as explicit degraded/debug behavior.
   - Downstream beads: `mb-uw37`, `mb-1ps7`, `mb-3drx`

## Concrete implementation guidance from the study

1. **Decide runtime authority first**
   - Land `mb-2lye` before broad retrieval/lifecycle work expands further.
   - Without a clear runtime owner, every later parity and health fix stays ambiguous.

2. **Finish hydration before tuning retrieval quality**
   - Land `mb-uw37` and `mb-1ps7` before treating retrieval-quality claims as meaningful.
   - Otherwise tests risk proving ranking behavior on top of incomplete evidence materialization.

3. **Treat parity as a contract, not a patch**
   - `mb-3g6d` should harden one shared cross-surface substrate with identical ids, policy defaults, omission semantics, and degraded signaling.

4. **Use semantic shards only when tests prove they help**
   - `mb-21vk` should compare whole-record ranking against smaller rankable units and log which stage changed the result.

5. **Make lifecycle observable through user-visible deltas**
   - `mb-1jho` should focus on scenarios where a lifecycle action changes later recall/why/health output, not just internal counters.

6. **Keep docs behind runtime truth**
   - `mb-3drx` should not claim unified warm-runtime behavior until `mb-2lye`, `mb-uw37`, and `mb-1ps7` make that statement true.

## Next bead changes implied by the study

The study does not require creating new beads right now. It sharpens the execution order and implementation focus of existing ones:

- `mb-2lye` stays the architectural gate because runtime authority must precede trustworthy health, parity, and lifecycle claims, and its acceptance path should require mode-specific unit/e2e evidence rather than storage-only truth.
- `mb-1ps7` remains the next essential implementation bead, and `mb-3drx` should follow it, because `mb-uw37`'s recall hydration is already implemented while explain/docs truth still need to fully align; those follow-ons should log candidate-order preservation and omission/defer reasons as part of completion.
- `mb-3g6d` should be treated as a contract-enforcement bead, not merely a startup hydration patch, with parity checks centered on one canonical retrieval envelope across CLI, daemon, and MCP.
- `mb-21vk` should explicitly treat semantic shards as a hypothesis to prove, not an assumption to bake in, using comparative tests that show whether shards changed the selected evidence set.
- `mb-1jho` should stay minimal and user-visible: lifecycle work only counts if recall/why/health behavior changes in a logged, testable way.

## Proposed bead decomposition

This section is here to make bead conversion easier. It is not the beads themselves.

1. **Daemon runtime authority and embedder lifecycle**
   - make daemon (or equivalent MCP runtime) the unambiguous source of warm runtime guarantees
   - surface model loaded/warm state truthfully in doctor/health

2. **Persisted memory hydration parity**
   - unify CLI, daemon, and MCP visibility over the same persisted local records
   - remove cross-surface semantic drift

3. **Recall hydration completion**
   - completed in the live daemon/runtime path; normal semantic recall now returns hydrated evidence and reserves degraded summaries for explicit no-hydrated-evidence cases
   - remaining work is truth-alignment in beads/docs and cross-surface follow-through

4. **Explain/why completion**
   - explain actual retrieval outcomes and evidence, not only the route skeleton

5. **Embedding-backed retrieval validation**
   - prove that main retrieval paths use embeddings and/or reranking as intended
   - benchmark warm/cold behavior

6. **Lifecycle/background processing validation**
   - ensure consolidation/reconsolidation/forgetting are not merely present in code but operationally relevant

7. **Docs and MCP tool truthfulness pass**
   - align AGENTS/docs/tool descriptions with implemented semantics and runtime expectations

8. **Comprehensive unit + e2e validation**
   - realistic scenarios, detailed logs, daemon-on and daemon-off cases, persisted-state reload tests

---

## Acceptance bar for closing this audit

We can consider this audit substantially resolved only when all of the following are true:

1. A persisted memory written once can be recalled and explained consistently through CLI, daemon JSON-RPC, and MCP stdio.
2. Standard recall/explain paths continue to present hydrated runtime evidence on normal success cases and reserve degraded summaries for explicit degraded states.
3. Doctor/health can truthfully distinguish:
   - daemon not running
   - daemon running but embedder not loaded
   - embedder loaded and warm
   - degraded/bypassed embedding path
4. Repeated daemon requests demonstrate shared runtime reuse rather than subprocess-cold semantics.
5. User-observable behavior is closer to the docs’ bounded-memory runtime than to a plain SQLite-backed note store.

---

## Why this matters strategically

The project’s ambition is significantly higher than “local notes DB with MCP wrapper.”

If we leave these gaps open, users will reasonably conclude:

- the docs overpromise
- the daemon is optional in a way that undermines the architecture
- embeddings are decorative rather than central
- Membrain is mostly persistence + query + nice explanations

If we close them, Membrain becomes much closer to what the docs actually promise:

- a real local memory runtime
- daemon-owned warm state
- embedding-aware retrieval
- coherent behavior across CLI / daemon / MCP
- truthful explainability and operational health reporting
