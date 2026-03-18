# ARCHITECTURE

## 1. System view

### Write path
ingest -> normalize -> classify -> score -> route -> persist -> schedule background jobs

### Read path
query context -> retrieval planner -> tier1 scan -> tier2 candidate generation ->
optional tier3 fallback -> graph expansion -> ranking -> packaging -> reinforcement updates

## 1.1 Write-side routing and tier contract
- Working memory is a bounded pre-persistence controller surface. It stages attended input but is not itself a persisted tier or authoritative memory store.
- Admission into canonical memory storage happens only after bounded encode-side gating says the candidate still deserves persistence. Inputs that never clear that gate may die in controller state without becoming durable records.
- Once intake is accepted for persistence, the first authoritative durable write lands in the hot durable path. Tier1 seeding is a derived acceleration step layered on top of that hot durable record.
- Ordinary online encode does not mint new canonical memories directly into Tier3; cold durable ownership is reached through consolidation, archive, repair, import, or migration flows.
- Successful encode and successful recall may refresh Tier1 or other hot serving mirrors for bounded immediate reuse, but eviction from working memory or Tier1 is not archival, deletion, or contradiction resolution.
- Durable promotion or demotion between hot and cold ownership remains an explicit auditable controller action rather than an implicit side effect of one cache event.

### Tier1 encode fast-path contract
- The synchronous pre-persistence fast path is limited to four ordered steps: normalize raw intake into the canonical envelope, derive a stable fingerprint from the normalized form, run shallow classification on bounded features, and compute provisional salience from the normalized form plus that shallow class.
- These steps exist to freeze the first durable write shape and routing inputs. They must remain narrow, deterministic, and cheap enough to stay inside the encode fast-path budget.
- Normalization must preserve raw evidence, bind provenance and scope, and produce the canonical object shape needed for downstream persistence and explainability. It must not perform heavyweight enrichment, summarization, graph work, or remote calls.
- Fingerprinting is a duplicate-family and collision-detection hint derived from normalized content. It must be stable for semantically identical normalized input under the same normalization generation, and it must not become an authorization token or replace canonical durable identity.
- Shallow classification must run only on bounded local features available on the fast path. It may select the first persisted memory kind, route family, or controller lane, but it must not depend on deep model inference, corpus scans, or deferred context that is unavailable synchronously.
- Provisional salience is a first-pass scalar used to support bounded routing and early retention decisions before later enrichment. It is provisional by design and may be refined later, but the synchronous value must remain inspectable as the input to the initial route.
- The fast path ends at the first authoritative durable write plus any bounded Tier1 seed or refresh. Enrichment, duplicate resolution beyond the bounded hint path, graph formation, large-payload handling, and other heavyweight work remain deferred.
- The contract requires unit coverage for normalization and fingerprint stability, plus structured traces or logs that expose the ordered fast-path stages, provisional route inputs, bounded candidate counts when duplicate hints are consulted, and whether the path stayed inside the declared latency budget.

## 2. Major components
- Ingestor
- Normalizer
- Fast encoder
- Salience engine
- Tier router
- Tier1 hot cache
- Tier2 warm indexed store
- Tier3 cold archive
- Association graph
- Retriever and ranker
- Consolidator
- Forgetter
- Observability and governance layer

## 3. Canonical thesis
The production contract is a brain-inspired cognitive runtime, not a claim of literal biological equivalence.

1. Foreground work stays bounded and measurable.
2. Provenance and lineage are first-class.
3. Explainability is required for routing, retrieval, ranking, and filtering.
4. Repairability outranks convenience for derived state.
5. Contradictions are represented, not erased.
6. Governance applies before expensive work.
7. Brain-inspired mechanisms become canonical only when they remain bounded, explainable, and benchmarked.

## 4. Design invariants
1. Hot path must stay bounded.
2. Every memory item must have provenance.
3. No hard delete without policy approval or retention expiry.
4. Graph edges must remain repairable from lineage or re-indexing.
5. Tier routing decisions must be traceable.
6. Retrieval ranking must be explainable after the fact.
7. Consolidation must never silently discard authoritative evidence.
8. Contradictions must be represented explicitly, not hidden by overwrite.

## 5. Restriction contract

These are architectural guardrails, not tuning suggestions. They apply across standalone CLI, daemon, MCP, IPC, tests, and any future wrapper surface.

### Request-path and foreground restrictions
- No LLM or remote API calls in encode, recall, `on_recall`, reconsolidation-apply, or forgetting-eligibility paths.
- No full-store `O(n)` scans on any request path; every retrieval lane must run within an explicit candidate budget.
- No cold-payload decompression or large payload fetch before the final candidate cut.
- No graph expansion without hard depth and node caps.
- No policy or namespace bypass in wrapper surfaces; governance checks happen before expensive retrieval work, not after.

### Storage and lifecycle restrictions
- Tier1 stores handles and hot metadata, not giant payload ownership.
- Tier2 keeps metadata and filtering state separate from large content so prefilters stay bounded.
- Tier3 and graph/index acceleration state remain rebuildable from durable records.
- Archive and forgetting flows stay reversible by default unless explicit policy requires irreversible deletion.

### Research and benchmark restrictions
- No benchmark claim without dataset cardinality, machine profile, build mode, and warm/cold declaration.
- Do not present p95 or p99 claims from microbench-sized samples as production facts; label them exploratory instead.
- Brain-inspired mechanisms remain optional research behavior unless benchmark and ablation evidence justify promotion to the core contract.

## 6. Rust workspace skeleton and module boundaries

### 6.1 Minimum workspace shape
| Surface | Owns | Must not own |
|---|---|---|
| `membrain-core` | Canonical domain model, storage contracts, retrieval and ranking engines, policy enforcement, graph, maintenance, migrations, and observability types | CLI presentation, transport-specific envelopes, or wrapper-only semantics |
| `membrain-cli` | CLI commands, local process entrypoints, and rendering of human or machine-readable output | Product semantics, policy shortcuts, or independent routing and ranking logic |
| optional daemon or service crate | Runtime lifecycle, background scheduler wiring, JSON-RPC or MCP transport adapters, and process supervision | Divergent semantics from `membrain-core` or wrapper-specific policy behavior |
| benchmark targets or crates | Reproducible latency and throughput harnesses plus evidence artifacts | Production routing behavior or hidden feature flags that change the canonical contract |
| integration-test support modules | Shared fixtures, parity harnesses, and failure-injection helpers | New product behavior that bypasses the public core APIs |

### 6.2 `membrain-core` ownership seams
- `types`, `constants`, and `config` freeze shared data shapes, budgets, generations, and configuration parsing defaults.
- `policy` owns effective-namespace resolution, ACL or visibility decisions, retention or hold gates, redaction markers, and machine-readable policy summaries before expensive work starts.
- `brain_store` or the equivalent top-level facade composes policy, stores, indexes, graph, and engine modules into stable core APIs instead of letting wrappers orchestrate semantics ad hoc.
- `store::hot`, `store::warm` or `store::tier2`, and `store::cold` or `archive` own durable layout, bounded retrieval primitives, and rebuildable persistence surfaces; they do not invent policy outcomes or widen scope.
- `embed` and `index` own bounded embedding and candidate-generation mechanics plus index maintenance generations; they do not package payloads or hide fallback semantics.
- `engine::encode`, `engine::recall`, and `engine::ranking` own request-path orchestration, bounded planning, score fusion, and packaging inputs for the canonical recall surface.
- `engine::consolidation`, `engine::forgetting`, and `engine::repair` stay separate from request-path engines so maintenance can be bounded, cancellable, audited, and tested independently.
- `graph` owns lineage-aware relation traversal and neighborhood expansion under explicit depth and node caps; recall plans may call it only through declared budgets.
- `migrate` owns schema-version transitions, backfills, and compatibility plumbing rather than ordinary request-path repair.
- `observability` owns shared metrics, trace fields, explain payload fragments, and audit-friendly event vocabularies so wrappers preserve one contract.

### 6.3 Boundary rules for downstream implementation beads
- Future code beads should land work in the narrowest owning surface above instead of growing catch-all `cli`, `store`, or `engine` modules.
- Interface adapters call stable `membrain-core` APIs and translate envelopes or presentation only; they must not reimplement policy checks, retrieval plans, ranking formulas, or maintenance semantics.
- Policy evaluation stays centralized and inspectable. Storage, graph, ranking, and interface modules consume policy-shaped inputs or outputs rather than inventing silent denials, redactions, or widening behavior.
- Store modules expose namespace-aware primitives and durable state transitions, but they do not decide product-facing route selection, visibility widening, or deletion semantics on their own.
- Maintenance engines keep consolidation, forgetting, repair, restore, and migration flows isolated from foreground recall and encode code so cancellation, degraded mode, and failure-injection tests can target them directly.
- Shared observability fields come from the core observability surface; wrappers may format them differently, but they must not silently rename away outcome classes or policy markers.
- Benchmarks and integration harnesses may depend inward on core modules, but production crates should not depend outward on benchmark-only or fixture-only code.
