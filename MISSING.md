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

However, the current implementation still falls short of the target architecture in several material ways:

1. **Daemon/MCP recall and explain are still planner-oriented rather than fully hydrated runtime retrieval paths.**
2. **The embedding runtime exists in code, but operational proof that the main daemon path loads and reuses it as the canonical retrieval engine is incomplete.**
3. **CLI / daemon / MCP semantics have historically diverged, and although some hydration gaps have been patched, the surfaces are not yet demonstrably unified.**
4. **The system still feels too much like "save records to SQLite and query them back" instead of the designed bounded memory runtime with hot/cold tiers, embedding-backed retrieval, background lifecycle processing, and evidence hydration.**
5. **Operational health/doctor output and runtime behavior do not yet prove the daemon is the authoritative always-on process described by the spec.**

This gap matters because the docs explicitly promise a stronger model:

- `docs/PLAN.md:1185` — `fastembed is the default local embedding layer`
- `docs/PLAN.md:10340` — embedding model wrapper `Loads model once, shared across threads`
- `docs/PLAN.md:10348-10357` — encode/recall/lifecycle architecture assumes an active bounded runtime, not just durable storage
- `docs/PLAN.md:9759` — doctor output explicitly expects a daemon-dependent embedding runtime state: `Embedding model: NOT LOADED (daemon not running)`
- `AGENTS.md` describes daemon mode as the background local socket service and MCP mode as stdio subprocess integration, implying coherent behavior across modes.

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

## Gap A — daemon/MCP recall and explain are still planner-oriented instead of fully hydrated retrieval

### Evidence

The daemon code explicitly says this in multiple places:

- `crates/membrain-daemon/src/daemon.rs:4818` — `planner-only explain envelope; evidence hydration not implemented`
- `crates/membrain-daemon/src/daemon.rs:5609` — `planner-only recall envelope; evidence hydration not implemented`
- similar strings recur in tests and degraded summaries

### Why this is a problem

This is the clearest mismatch with the spec.

The spec describes a retrieval pipeline that should actually:

- do tiered retrieval
- rescore candidates
- use engram expansion / deeper tiers as needed
- produce materialized evidence
- make explainability describe a real retrieval outcome

But a planner-only envelope means the system is still willing to answer with route/explain scaffolding instead of a fully realized runtime retrieval result.

### What “done” must mean

To close this gap:

- recall/explain must hydrate actual evidence on the normal daemon/MCP path
- planner-only responses should be restricted to explicit degraded/failure modes and surfaced as such
- tests must prove that a persisted memory can be:
  - recalled via CLI
  - recalled via daemon JSON-RPC
  - recalled via MCP stdio
  - explained consistently via `why` / explain surfaces

### Failure modes to document in work

- daemon can plan but not hydrate
- explain path describes hypothetical retrieval rather than actual evidence
- MCP result shape looks complete but hides empty evidence packs
- regression where exact-id works but semantic query still falls back to planner-only behavior

---

## Gap B — embedding runtime exists, but operationally authoritative daemon usage is not yet proven

### Evidence

- `crates/membrain-core/src/embed.rs` contains the real fastembed wrapper
- `docs/PLAN.md:10340` expects “load once, shared across threads” behavior
- `docs/PLAN.md:9759` explicitly models a daemon-dependent loaded/not-loaded state

### Why this is a problem

Right now, the repo contains the right parts, but the user experience still suggests uncertainty about whether embeddings are truly active on the main path.

The user explicitly noticed this by asking whether `fastembed` is actually running.

That question should have a crisp answer from health/doctor/runtime behavior, not an inference based on source code plus absence of a separate process.

### What “done” must mean

To close this gap:

- daemon startup must initialize or lazily initialize the canonical embedder in a measurable way
- doctor/health must accurately report whether the embedder is loaded, warm, degraded, unavailable, or bypassed
- repeated retrieval/encode requests through the daemon should demonstrate warm reuse rather than per-request cold behavior
- tests should verify model/cache lifecycle at the daemon level, not just the presence of the wrapper in core code

### Failure modes to document in work

- embedder compiled in but never used by main path
- daemon path silently bypasses embeddings while docs imply otherwise
- health says loaded when runtime never exercised the embedder
- daemon reloads model per request, violating shared-across-threads design intent

---

## Gap C — CLI, daemon, and MCP have not yet fully converged semantically

### Evidence

Recent debugging already exposed a real semantic split:

- CLI could recover persisted user-name memories from `~/.membrain/hot.db`
- MCP originally could not
- a targeted fix was required to hydrate persisted memories into daemon startup state

### Why this is a problem

The spec/docs imply that these are multiple access surfaces over one coherent local memory runtime. The user should not have to know which command path is more “real” or more complete.

If CLI sees memory A and MCP does not, then the system is not yet behaving like a unified memory runtime.

### What “done” must mean

To close this gap:

- all durable memory content visible to CLI recall/inspect must be visible through daemon/MCP unless blocked by explicit policy
- result ranking may differ by output mode, but core memory visibility and explainability semantics must be aligned
- tests must cover cross-surface parity for:
  - encode
  - recall
  - inspect
  - explain/why
  - health/doctor state

### Failure modes to document in work

- CLI sees persisted data but daemon/MCP misses it
- daemon sees only in-session runtime records
- inspect works for exact IDs but recall misses the same record
- one surface defaults to a different namespace/policy/routing behavior than another

---

## Gap D — current system still behaves too much like persistence + query, not bounded cognitive runtime

### Evidence

The user’s complaint is structurally correct: the lived behavior still resembles “save DB rồi query ra” more than a human-inspired memory system.

The docs intend much more:

- `docs/PLAN.md:10348-10357` lays out encode/recall/background lifecycle modules
- hot path, cold path, working memory, consolidation, reconsolidation, forgetting, and reranking are all supposed to matter

### Why this is a problem

Even if the database is correct and query returns results, that alone does not fulfill the product promise.

The human-like analogy only becomes legitimate when the runtime shows consequences such as:

- bounded hot state
- meaningful retrieval/rerank tradeoffs
- background lifecycle processing
- changing recall behavior due to reconsolidation/strength/stability/forgetting
- observability explaining these behaviors

### What “done” must mean

To close this gap:

- background lifecycle jobs must have real operational impact, not just placeholders
- recall ranking must reflect more than simple compact-text matching and persistence order
- doctor/health/why must expose memory-dynamics reasoning in ways users can verify
- benchmark/tests must demonstrate behaviors the docs claim (warm cache effects, bounded hot memory, lifecycle transitions, rerank precision changes, etc.)

### Failure modes to document in work

- all retrieval quality comes from persistence text scan / fallback matching
- lifecycle modules exist but do not affect user-visible recall behavior
- doctor output references architectural concepts that have no runtime effect

---

## Gap E — daemon is not yet the clearly authoritative always-on operating mode the docs imply

### Evidence

The docs place meaningful weight on daemon mode:

- daemon is the socket server
- doctor output reasons about daemon-dependent embedding state
- background lifecycle work is documented as daemon-owned

But current usage still often defaults to MCP subprocess mode, and some important runtime expectations remain incomplete there.

### Why this is a problem

The architecture feels under-committed if:

- daemon exists but is not clearly the source of truth for warm runtime state
- MCP subprocess behaves like a shallower alternate path
- health/doctor/runtime semantics differ depending on how the user entered the system

### What “done” must mean

To close this gap:

- define whether daemon is the authoritative runtime or whether stdio MCP is expected to match it completely
- if daemon is authoritative, make that operationally true and visible
- if stdio MCP must be equivalent, then implement the equivalent runtime guarantees there too
- doctor/health should clearly indicate which mode is active and what guarantees are currently in effect

### Failure modes to document in work

- daemon path and MCP path both partially complete in different ways
- docs imply always-on runtime benefits but common user path doesn’t get them
- “daemon not running” becomes a hidden architecture fork instead of an explicit posture

---

## Secondary gaps

## Gap F — tool and docs surface can still mislead agents about what is actually implemented

### Evidence

MCP tooling now advertises useful tools, but the actual runtime semantics still include planner-only/degraded behavior in important paths.

### Why this matters

If tools are described as if they fully implement the intended semantics while the daemon still returns planner-only envelopes in some modes, agents will over-trust the system and generate low-quality explanations.

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
- cross-surface parity
- degraded-mode visibility
- doctor/health truthfulness
- end-to-end scripts with detailed logs proving the intended runtime model

---

## Concrete missing capabilities checklist

The following are the capabilities still missing or not yet proven strongly enough:

- [ ] daemon recall path hydrates evidence on the standard semantic-query path without planner-only fallback
- [ ] explain path is backed by actual hydrated evidence rather than route-only scaffolding
- [ ] daemon startup loads persisted memories and yields semantic parity with CLI recall/inspect/why
- [ ] embedder load/warm state is observable and truthful in doctor/health
- [ ] embedder is reused across daemon requests as the spec promises
- [ ] retrieval quality on main paths materially depends on embeddings/reranking rather than persistence-only fallback behavior
- [ ] lifecycle/background jobs materially affect user-visible recall behavior
- [ ] daemon/MCP/CLI share one coherent runtime contract
- [ ] tool descriptions and operational docs reflect what is really implemented today
- [ ] end-to-end tests validate the above in realistic workflows

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

## Proposed bead decomposition

This section is here to make bead conversion easier. It is not the beads themselves.

1. **Daemon runtime authority and embedder lifecycle**
   - make daemon (or equivalent MCP runtime) the unambiguous source of warm runtime guarantees
   - surface model loaded/warm state truthfully in doctor/health

2. **Persisted memory hydration parity**
   - unify CLI, daemon, and MCP visibility over the same persisted local records
   - remove cross-surface semantic drift

3. **Recall hydration completion**
   - replace planner-only recall envelope on normal paths with actual evidence hydration
   - preserve planner-only mode only as explicit degraded behavior

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
2. Standard recall/explain paths no longer emit planner-only degraded summaries in normal success cases.
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
