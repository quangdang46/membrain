# membrain

> Brain-inspired, local-first memory runtime for AI agents.

membrain is a Rust workspace for building **persistent, explainable, policy-aware memory** for serious agents, tools, and long-running workflows.

Most so-called agent memory systems are really just a thin layer over embeddings and retrieval. They can store things, but they are weak at the parts that actually matter when an agent runs for a long time: contradiction, provenance, explainability, governance, lifecycle, and repair.

membrain is an attempt to build that missing layer.

It is designed for people who want memory that can be trusted, inspected, debugged, repaired, and evolved without turning into black-box infrastructure.

If that direction resonates, this repo is worth watching.

---

## The short version

membrain is built around a stricter definition of memory:

- memory should stay useful beyond one prompt window
- retrieval should return the **smallest evidence set** that helps the task
- the system should preserve **where information came from**
- conflicts should be represented explicitly, not overwritten silently
- the hot path should stay **bounded and measurable**
- governance should apply **before** expensive work happens
- caches, indexes, and summaries should be **repairable derived state**, not hidden truth

That combination is what makes membrain interesting.

This is not just “AI memory.”
It is memory with systems discipline.

---

## Detailed runtime workflow

![membrain workflow](assets/workflow.png)

This single workflow shows how membrain routes every request through one canonical core: encode persists authoritative records first, recall assembles the smallest explainable evidence set under hard bounds, and maintenance runs in background lanes to enrich or repair derived state without silently redefining the truth.

---

## Why this project exists

Current agent memory stacks usually fail in one of two ways.

The first failure mode is being too shallow:

- save text
- compute embeddings
- query later
- call that “memory”

The second failure mode is being too magical:

- lots of clever behavior
- weak guarantees
- poor observability
- no clear distinction between truth, cache, inference, and guesswork

membrain goes in the opposite direction.

The project thesis is that a real memory runtime for agents should behave like a serious system component:

- **measurable** under load
- **inspectable** after the fact
- **provenance-preserving** by default
- **contradiction-aware** instead of overwrite-happy
- **policy-aware** instead of scope-sloppy
- **repairable** when derived layers drift
- **local-first** instead of assuming a hosted memory backend

That is the core pitch.

---

## Why now?

Agent systems are moving out of the toy stage.

More builders are now trying to make agents:

- run for longer than a single prompt
- recover context across sessions
- collaborate through tools and protocols
- make decisions that need auditability
- operate inside teams, projects, and policy boundaries
- become dependable enough for real workflows instead of demos

That shift changes what “memory” needs to mean.

A thin retrieval layer may be enough for a prototype.
It stops being enough when an agent has to:

- remember what mattered last week
- explain why it chose one piece of evidence over another
- preserve contradictory information instead of flattening it
- operate inside namespace or visibility boundaries
- survive stale caches, index drift, or partial repair states
- support real operator debugging instead of vibes-based trust

This is exactly the moment where memory infrastructure starts to matter as a real systems layer.

That is why projects like membrain matter now.
Not because memory is trendy, but because agent workflows are becoming serious enough that weak memory abstractions are starting to hurt.

---

## Why this could matter for all agent builders

Even if someone never adopts membrain directly, the ideas in this repo push on problems that almost every serious agent builder will hit sooner or later.

Those problems include:

- how to keep memory useful without flooding context
- how to separate hot serving state from durable truth
- how to preserve provenance when memories are summarized or transformed
- how to represent uncertainty and contradiction honestly
- how to make retrieval explainable enough for debugging and trust
- how to enforce policy and scope before expensive work
- how to keep derived memory layers repairable instead of magical

These are not niche concerns.
They are core concerns for the next generation of agent systems.

As agents become longer-running, more autonomous, more collaborative, and more embedded in real workflows, memory stops being a convenience feature and becomes one of the central architecture problems.

That is why membrain could matter beyond this repository itself.

It is exploring an idea that is likely to become broadly important:

> agent memory should be treated like real infrastructure, with semantics, budgets, governance, explainability, and repairability built in from the start.

If that idea spreads, a lot of future agent tooling will likely look more like membrain than like today’s thin memory wrappers.

---

## What makes membrain different

### It is brain-inspired, but not hand-wavy

The repo takes ideas from real memory systems in the brain — working memory, salience, consolidation, reconsolidation, forgetting, associative recall, interference — but only keeps them when they remain practical.

In membrain, “brain-inspired” does **not** mean vague biomimicry.
It means useful abstractions subjected to engineering rules:

- stay bounded
- stay explainable
- stay benchmarkable
- stay repairable

### It treats explainability as a product feature

If a result appears, the system should be able to say why.
If a result was filtered, narrowed, deferred, redacted, or conflict-marked, the system should be able to say that too.

membrain’s contract is built around machine-readable explanation surfaces such as:

- route summaries
- candidate budgets
- omission reasons
- policy summaries
- provenance summaries
- freshness markers
- conflict markers
- trace stages

This makes the system more useful for operators, contributors, and tool builders who need to debug behavior instead of merely hoping retrieval “felt good.”

### It treats contradiction as a first-class part of memory

Real memory systems have to deal with:

- outdated beliefs
- superseded facts
- partially conflicting evidence
- policy overrides
- competing interpretations

membrain is designed to **preserve that state explicitly**.

It does not want the system to quietly replace the old answer and pretend disagreement never happened.

### It puts governance before convenience

Namespace isolation, approved sharing, retention, redaction, auditability, and policy enforcement are not bolt-ons here.
They are part of the core contract.

That means wrappers and convenience surfaces are not allowed to invent weaker semantics than the system underneath them.

### It values repairability more than convenience

Many systems work beautifully until an index drifts, a cache becomes stale, a graph gets inconsistent, or a summary layer stops matching the durable source of truth.

membrain is explicit about the hierarchy:

- durable records are authoritative
- caches are accelerators
- warmed state is not truth
- graph/index/summary layers should be rebuildable
- degraded mode should be explicit when repair is needed

This mindset makes the project much more interesting than a typical memory wrapper.

---

## What membrain is

membrain is a **memory runtime**, not just a database and not just a plugin.

It is intended to help agent systems:

- remember useful information over time
- retrieve the right evidence without flooding downstream context
- preserve source and lineage information
- express uncertainty and contradiction explicitly
- enforce scope and policy safely
- expose enough internal structure that humans can trust what happened

That places it in a different category from:

- simple vector stores
- context-window stuffing utilities
- generic “RAG memory” helpers
- note stores with no lifecycle or governance model

The long-term goal is a **brain-inspired cognitive runtime with production discipline**.

---

## What membrain is not

membrain is **not** trying to be:

- just another vector database with a memory-themed README
- a black-box memory service that cannot explain itself
- a remote-LLM-dependent hot-path system
- a premature distributed architecture that scales before it can prove local correctness
- a repo where docs are decorative and semantics are improvised in code

That last point matters.

In this project, docs are part of the contract.
They define invariants, interface semantics, evidence requirements, and review standards.

---

## Current status

membrain is under active development and is intentionally **docs-first / contract-driven**.

That means two things are true at the same time:

First, this is already a real Rust codebase with tests, binaries, and working command surfaces.

Second, the design contract is broader than what is fully landed today.

That is deliberate.

The repo prefers to make the architecture explicit before over-claiming implementation maturity.

### What is already real today

The workspace already includes:

- `membrain-core` for the canonical engine and shared semantics
- `membrain` as a working CLI binary
- `membrain-daemon` as a working daemon binary
- tested command flows for encode, recall, explain, inspect, audit, maintenance, share, and unshare behavior
- machine-readable JSON output on important user-facing paths
- daemon / JSON-RPC-oriented runtime surfaces
- canonical docs for architecture, retrieval, CLI, MCP, operations, and roadmap

### What is still broader than current implementation

The contract also covers a larger future surface around:

- richer contradiction handling
- graph-assisted retrieval under hard budgets
- consolidation and forgetting pipelines
- deeper operator tooling
- later-stage cognitive extensions
- evidence-gated scale-out

So the honest positioning is this:

> membrain is already a real system, but it is also an ambitious architecture being landed carefully instead of hand-waved into existence.

That honesty is part of the appeal.

---

## Who should care about this repo

membrain is especially relevant for a few kinds of builders.

### AI agent builders
If you need memory that lasts beyond a session and does more than naive retrieval, membrain is aimed directly at that problem.

### Local-first developers
If you want memory infrastructure that can live close to your toolchain and does not assume a hosted memory service, membrain fits that worldview.

### Tooling and infra teams
If you build CLI agents, daemon-backed tools, MCP-connected systems, or operator-heavy automation, the repo’s attention to contracts and machine-readable output becomes very useful.

### Research-minded product engineers
If you are interested in long-horizon memory, salience, forgetting, consolidation, or conflict-aware recall — but still want systems rigor — this repo is unusually rich.

### Contributors who like hard problems
If you enjoy:

- Rust
- retrieval systems
- observability
- policy boundaries
- interface parity
- lifecycle design
- repair and degraded-mode thinking

there is a lot here to work on.

---

## The architecture, in one glance

### Write path

```text
ingest -> normalize -> classify -> score -> route -> persist -> schedule background jobs
```

### Read path

```text
query context -> retrieval planner -> tier1 scan -> tier2 candidate generation
-> optional tier3 fallback -> graph expansion -> ranking -> packaging -> reinforcement updates
```

### The system thesis

membrain’s production contract is a brain-inspired cognitive runtime built around a few hard rules:

- foreground work stays bounded and measurable
- provenance and lineage are first-class
- routing and retrieval stay explainable
- contradictions are represented, not erased
- governance applies before expensive work
- derived state remains repairable
- advanced mechanisms must earn their place with evidence

### The restrictions are part of the product

Across CLI, daemon, MCP, tests, and docs, membrain repeatedly enforces constraints like these:

- no full-store scans on request paths
- no uncapped graph traversal on request paths
- no remote or LLM work on core encode/recall hot paths
- no cold payload fetch before the final candidate cut
- no policy bypass in wrapper surfaces
- no silent contradiction overwrite
- no maintenance work quietly leaking into request latency

This is a big part of what gives the repo its character.

---

## The three-tier memory model

membrain centers on a three-tier architecture.

### Tier1

This is the hot, immediate reuse layer.
It exists for fast bounded access and is optimized for hot-path serving.
It is not the only truth of the system.

### Tier2

This is the warm durable indexed store.
It is the main searchable memory layer for most practical retrieval work.

### Tier3

This is the colder, archival, longer-horizon durable layer.
It exists for depth and durability, but the architecture is careful not to let Tier3 behavior silently dominate request-path cost.

### Why the distinction matters

One of the most important design choices in membrain is the separation between:

- controller-like working state
- hot serving state
- durable memory ownership
- background lifecycle work

That separation prevents caches and warmed state from pretending to be authoritative memory.

### Performance posture

The operational contract sets strong latency targets such as:

- Tier1 lookup under 0.1ms
- Tier2 retrieval under 5ms
- Tier3 retrieval under 50ms
- encode under 10ms

Those are not random marketing numbers.
They are part of the discipline the repo uses to evaluate changes.

---

## The core crates

The workspace is built around three main crates.

### `crates/membrain-core`

This is where the real product semantics live.
It owns the canonical domain model and the logic that should remain consistent regardless of interface.

That includes work around:

- types and config
- namespace and policy enforcement
- store and index contracts
- encode and recall logic
- ranking and result packaging
- lifecycle and maintenance engines
- observability and audit vocabulary

### `crates/membrain-cli`

This crate owns the end-user CLI surface.

It is responsible for:

- command structure
- argument parsing
- text rendering
- machine-readable JSON output

It should not invent a separate product meaning from `membrain-core`.

### `crates/membrain-daemon`

This crate owns runtime lifecycle and local transport behavior.

It covers things like:

- daemon execution
- Unix socket and JSON-RPC-oriented runtime behavior
- runtime status surfaces
- MCP-oriented integration layers

This separation makes the project easier to extend without turning everything into one big interface-driven blob.

---

## The stack

The current docs and manifests point to a stack optimized for locality, systems control, and inspectability.

Core ingredients include:

- Rust for the implementation language
- Tokio for async runtime behavior
- SQLite with WAL and FTS5 for durable local storage and lexical retrieval support
- USearch / HNSW-style indexing for semantic candidate generation
- fastembed for local embeddings
- petgraph for graph and association modeling
- Unix sockets with JSON-RPC-oriented transport for the daemon surface
- MCP-oriented integration layers for tool ecosystems

The stack tells you a lot about the repo’s intent.

This is not trying to be “memory as a cloud product first.”
It is trying to be **memory as serious local infrastructure**.

---

## The memory model is richer than chunks + embeddings

membrain’s memory model is one of the most interesting parts of the project.

The docs define a broad taxonomy that includes things like:

- events
- episodes
- facts
- relations
- summaries
- goals
- skills
- constraints
- hypotheses
- conflict records
- policy artifacts
- observations
- tool outcomes
- user preferences
- session markers

Orthogonal to that, the brain-inspired encoding model also reasons about kinds such as:

- episodic
- semantic
- procedural
- schema

That separation matters.

It lets the system distinguish between:

- what a memory *is*
- how it *behaves*
- how it *should be stored or recalled*
- how it *should interact with lifecycle and ranking*

The model also takes identity and provenance seriously.

A memory is not just some text plus a vector.
It can carry:

- stable identity
- namespace scope
- source kind and source reference
- lineage to parent memories
- policy flags
- retention state
- salience and confidence
- contradiction markers
- uncertainty surfaces
- sharing visibility

That richer structure is what enables conflict-aware, explainable, and auditable memory behavior.

---

## Retrieval is a product surface, not just a query

The retrieval objective in membrain is sharp:

> return the smallest evidence set that maximizes downstream task success.

That is a better objective than “retrieve as much vaguely relevant stuff as possible.”

### Retrieval modes

The documented retrieval model includes multiple modes such as:

- exact retrieval
- recent retrieval
- semantic retrieval
- associative retrieval
- constraint retrieval
- reconstruction retrieval

### Retrieval is staged and budgeted

Candidate generation is designed as a bounded flow:

- direct hints first
- Tier1 scan for hot reuse
- Tier2 exact search
- Tier2 semantic candidate generation
- bounded graph expansion
- optional Tier3 fallback
- dedup and diversify
- rank
- package

### Packaging is first-class

A lot of systems stop after ranking.
membrain does not.

The final packaged result is expected to preserve more than “top N results.”
It aims to preserve concepts like:

- outcome class
- evidence pack
- optional action pack
- omission summary
- policy summary
- provenance summary
- freshness markers
- conflict markers
- deferred payloads
- packaging metadata
- explanation handle or embedded trace detail

That packaging layer is a huge part of the project’s value.
It turns retrieval into something the rest of an agent stack can trust and reason about.

---

## Explainability, governance, and operations are not side quests

membrain is unusual because it treats these areas as core architecture, not later clean-up.

### Explainability

The system should be able to say:

- why a route was chosen
- why evidence was included
- why evidence was omitted
- whether policy narrowed the answer
- whether payload hydration was deferred
- whether freshness or conflict conditions apply

### Governance

The system should preserve meaningful distinctions like:

- validation failure vs policy denial
- same-namespace access vs approved widening
- archive vs destructive delete
- absent data vs redacted data
- degraded result vs complete result

### Operations

The system should also behave honestly when things are imperfect.

That means explicit handling for situations like:

- degraded mode
- read-only posture
- repair in flight
- maintenance windows
- preview-only / blocked / accepted / rejected action paths
- rollback-aware operations

This makes membrain much more attractive for serious tools than a retrieval layer that only looks good when everything is healthy.

---

## What you can already do today

The current CLI surface already includes commands for:

- remembering / encoding memory
- recalling memory
- explaining ranking and routing
- inspecting a memory by id
- auditing recent history
- running maintenance actions
- checking system health with doctor/benchmark-style commands
- sharing and unsharing memory under policy-aware scope
- running the local daemon

Many of these surfaces already support `--json`, which makes them usable in scripts and automation.

The current tests also exercise realistic machine-readable outputs for important flows.

So while the full vision is still landing, this is not just a documentation mockup.

---

## Quickstart

Build the workspace:

```bash
cargo build --workspace
```

Run the tests:

```bash
cargo test --workspace
```

Check the help output:

```bash
cargo run --bin membrain -- --help
cargo run --bin membrain-daemon -- --help
```

Install the binaries locally:

```bash
cargo install --path crates/membrain-cli
cargo install --path crates/membrain-daemon
```

Install from the published release with the curl-pipe installer:

```bash
curl -fsSL "https://raw.githubusercontent.com/quangdang46/membrain/main/install.sh?$(date +%s)" | bash
```

Useful installer flags:

```bash
curl -fsSL "https://raw.githubusercontent.com/quangdang46/membrain/main/install.sh?$(date +%s)" | bash -s -- --verify --easy-mode
curl -fsSL "https://raw.githubusercontent.com/quangdang46/membrain/main/install.sh?$(date +%s)" | bash -s -- --db-path /path/to/state-root
```

What the installer now does by default:

- install both `membrain` and `membrain-daemon`
- configure Membrain MCP for Claude Code in `~/.claude/settings.json`
- configure Membrain MCP for Codex in `~/.codex/config.toml`
- install or update Membrain-owned Claude and Codex hook entries without replacing unrelated existing hook config
- enable Codex hooks through `[features].codex_hooks = true`
- configure a user-level daemon auto-start service on supported platforms
- create a real-time daemon log file so users can tail daemon activity directly
- optionally verify the installed binaries

Default installer paths:

- Claude settings: `~/.claude/settings.json`
- Codex config: `~/.codex/config.toml`
- Codex hooks: `~/.codex/hooks.json`
- installed Membrain hook helper: `~/.local/share/membrain/hooks/membrain_agent_hook.py`
- daemon log: `~/.local/state/membrain/membrain-daemon.log`

After install, follow the daemon log with:

```bash
tail -f ~/.local/state/membrain/membrain-daemon.log
```

---

## A simple first workflow

Store a memory:

```bash
cargo run --bin membrain -- remember "Paris is the capital of France" \
  --namespace demo \
  --kind semantic \
  --source cli
```

Recall it later:

```bash
cargo run --bin membrain -- recall "capital of France" \
  --namespace demo \
  --top 3 \
  --explain full \
  --json
```

Ask for explanation:

```bash
cargo run --bin membrain -- why "how do I deploy after the last incident?" \
  --namespace demo \
  --json
```

Inspect a specific memory:

```bash
cargo run --bin membrain -- inspect --id 1 --namespace demo --json
```

Inspect audit history:

```bash
cargo run --bin membrain -- audit --namespace demo --recent 10 --json
```

Run a maintenance action:

```bash
cargo run --bin membrain -- maintenance \
  --action repair \
  --namespace demo \
  --json
```

Start the local daemon:

```bash
cargo run --bin membrain-daemon -- --socket-path /tmp/membrain.sock
```

That flow captures the repo’s character well.
It is not only about storing and recalling data.
It is also about explanation, audit, and operational behavior.

---

## Roadmap direction

The roadmap is phased in a way that says a lot about the project.

The ordering principle is simple:

- measurable before clever
- explainable before highly optimized
- repairable before operationally large

The rough progression is:

- establish contracts, schema semantics, and measurable foundations
- land fast encode and indexed retrieval
- make results explainable and contradiction-aware
- add lifecycle systems like consolidation, forgetting, and repair
- only then push deeper into operations maturity and evidence-gated scale-out

This matters because it prevents the project from turning into a hype-driven architecture that tries to do everything at once.

### A note on sharding and distribution

The repo has explicit docs about future sharding/distribution, but they are intentionally framed as **later-stage** work.

Scale-out is not treated as default ambition.
It is gated behind actual workload evidence and benchmark pressure.

That is a sign of maturity, not lack of ambition.

---

## How to read the repo

If you want to understand membrain properly, start with these docs.

Read [`docs/PLAN.md`](docs/PLAN.md) for the canonical long-form design contract.

Then read [`docs/ARCHITECTURE.md`](docs/ARCHITECTURE.md) to understand ownership seams and boundaries.

Then read [`docs/MEMORY_MODEL.md`](docs/MEMORY_MODEL.md) and [`docs/RETRIEVAL.md`](docs/RETRIEVAL.md) to understand the semantics that make the system different.

Then read [`docs/CONTRIBUTING.md`](docs/CONTRIBUTING.md) to see how this repo expects changes to be justified.

Other important references:

- [`docs/CLI.md`](docs/CLI.md) for the CLI contract
- [`docs/MCP_API.md`](docs/MCP_API.md) for MCP and daemon/JSON-RPC semantics
- [`docs/OPERATIONS.md`](docs/OPERATIONS.md) for runbooks and safeguard semantics
- [`docs/ROADMAP.md`](docs/ROADMAP.md) for the phase view
- [`docs/SHARDING_AND_DISTRIBUTION.md`](docs/SHARDING_AND_DISTRIBUTION.md) for later-stage scale-out framing
- [`AGENTS.md`](AGENTS.md) for repository workflow guidance

---

## Contributing

Contributions are welcome, especially from people who care about memory systems that are both ambitious and disciplined.

Good contribution areas include:

- core Rust systems work
- retrieval and packaging behavior
- contradiction and uncertainty handling
- lifecycle engines
- observability and audit surfaces
- daemon/runtime ergonomics
- CLI polish
- docs and contract tightening

A practical way to get started is:

```bash
cargo build --workspace
cargo test --workspace
cargo run --bin membrain -- --help
cargo run --bin membrain-daemon -- --help
```

Then read:

- [`docs/PLAN.md`](docs/PLAN.md)
- [`docs/ARCHITECTURE.md`](docs/ARCHITECTURE.md)
- [`docs/CONTRIBUTING.md`](docs/CONTRIBUTING.md)

### What this repo values

This repository strongly prefers **evidence-backed changes**.

Depending on the kind of change, that can mean:

- benchmark evidence for hot-path work
- migration notes for schema changes
- rollback notes for behavior changes
- governance analysis for deletion/retention semantics
- observability hooks for performance-sensitive work
- parity validation across CLI, daemon, and MCP-facing surfaces

That may be stricter than many repos, but it is also what makes the architecture coherent.

---

## Why this repo is worth starring

A star-worthy project is not just one with a cool idea.
It is one where the idea is sharp, the direction is credible, and the implementation has enough seriousness that you can imagine it becoming important.

membrain has that combination.

It has:

- a strong product thesis
- a real codebase, not just an idea
- unusually explicit design contracts
- meaningful systems constraints
- a local-first angle that matters
- enough ambition to be exciting
- enough discipline to be believable

If you care about the future of agent memory, there is a good chance this repo will contain ideas worth following even before the full vision is complete.

---

## Final note

membrain is trying to answer a hard question:

**What would agent memory look like if we treated it as real infrastructure instead of a retrieval gimmick?**

That is the project.

If that direction resonates with you:

- star the repo
- follow the roadmap
- read the canonical docs
- open an issue
- send a focused PR

There is a lot of serious work left to do, and that is exactly what makes the project interesting.
