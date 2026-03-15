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
