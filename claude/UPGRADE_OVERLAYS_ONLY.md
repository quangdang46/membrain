# UPGRADE OVERLAYS

## 1. Research framing

The original plan is strongest when interpreted as a **functional translation** of neuroscience into systems design.
Keep this framing explicit:

- hippocampus ↔ hot episodic index
- neocortex ↔ deep semantic cold store
- amygdala ↔ emotional salience and retention bias
- prefrontal cortex ↔ working-memory and executive control
- engrams ↔ clustered associative recall units
- reconsolidation ↔ update-on-recall mutation window
- sleep / homeostasis ↔ background consolidation and pruning jobs

### Hard rule
Do not claim biological identity.
Claim **engineering correspondence** and **behavioral analogy** unless the mechanism is directly justified by measurement.

### Research claim classes
Every neuroscience-inspired claim should be tagged as one of:
- `analogy`
- `evidence-backed inspiration`
- `engineering hypothesis`
- `production assumption`
- `benchmark target`

### Recommended upgrade to wording
Replace “port the full set of human memory mechanisms” with:
“map the key functional mechanisms of human memory into a production-grade memory operating system for AI agents.”

That keeps your ambition while reducing overclaim risk.

## 2. Core design invariants

1. Foreground recall must remain bounded even if total memory grows by 100x.
2. Encode fast path must never depend on remote APIs.
3. No background job may block foreground retrieval beyond latency budget.
4. Every memory mutation must preserve provenance or emit an explicit loss event.
5. Contradictions must be represented, not silently overwritten.
6. Tier transitions must be auditable.
7. Cold payload fetch must occur only after candidate trimming.
8. Graph expansion must obey hard caps.
9. Standalone mode and daemon mode must preserve semantic equivalence for core APIs.
10. Benchmarks must be reproducible in release mode on declared hardware.

## 3. Non-negotiable restrictions

### Foreground path restrictions
- No LLM calls in encode, recall, on_recall, reconsolidation apply path, or forgetting eligibility path.
- No full-store O(n) scan in any request path.
- No decompression of cold payload before final candidate cut.
- No graph BFS without hard depth and node caps.
- No policy bypass in CLI, daemon, MCP, or IPC wrappers.

### Storage restrictions
- Tier1 must not own giant payloads.
- Tier2 must separate metadata from large content.
- Tier3 must remain recoverable after rebuild from durable records.
- Archive must be reversible by default.

### Research restrictions
- No benchmark claim without dataset cardinality, machine profile, and warm/cold declaration.
- No p95 claim from microbench-size sample counts unless labeled as exploratory.

## 4. Performance budget decomposition

### Encode fast path
Budget targets:
- cache lookup and hash: microseconds
- embedding cache hit: near zero
- cache miss embedding: bounded under target hardware profile
- novelty search: bounded by top-1 or top-k small search
- DB insert + HNSW add: bounded
- total p95 fast path: <10ms

### Tier1
Budget targets:
- exact lookup + score + return
- p95 <0.1ms
- p99 must remain close enough that tail does not invalidate fast-path narrative

### Tier2
Budget targets:
- metadata prefilter
- HNSW search
- float32 rescore
- optional engram expansion within hard budget
- p95 <5ms at declared hot cardinality

### Tier3
Budget targets:
- mmap probe
- sparse metadata fetch
- float32 rescore
- cold payload only for final selection
- p95 <50ms at declared cold cardinality

## 5. Benchmark contracts by stage

### Stage 1 — Foundation + Lazy Decay
Must pass:
- encode→recall roundtrip
- WAL verified
- effective_strength formula stable
- embedding cache measurable benefit
- hot prefilter touches metadata table only

### Stage 2 — Full Encode Pipeline
Must pass:
- attention gating
- novelty and duplicate routing
- emotional bypass rules
- working-memory deterministic eviction
- bounded interference updates

### Stage 3 — on_recall / LTP-LTD
Must pass:
- idempotent request-bounded on_recall
- stability growth monotonic
- recall overhead bounded
- labile transition durable through restart

### Stage 4 — 3-tier retrieval
Must pass:
- tier escalation determinism
- adaptive ef bounded
- context rerank measurable
- engram expansion within budget
- partial / tip-of-tongue path does not leak full payload incorrectly

### Stage 5 — Reconsolidation
Must pass:
- valid labile-window enforcement
- re-embed and reindex coherence
- cache invalidation correctness
- crash-safe update application

### Stage 6 — Consolidation
Must pass:
- NREM migration retrievable after move
- REM-like cross-linking auditable
- homeostasis never prunes pinned or authoritative evidence
- background work does not break foreground SLOs

### Stage 7 — Engram maturity
Must pass:
- centroid stability
- split and sibling creation rules
- BFS caps
- restart serialization integrity

### Stage 8 — Forgetting engine
Must pass:
- prune eligibility is policy-safe
- archive restore roundtrip
- overload convergence
- recall quality improves or remains stable after pruning

### Stage 9 — Daemon + IPC + MCP
Must pass:
- semantic parity with standalone mode
- socket lifecycle robustness
- concurrency safety
- IPC overhead bounded

### Stage 10 — Production readiness
Must pass:
- export/import roundtrip
- corruption detection via doctor
- reproducible benchmark suite
- rollback notes and repair playbooks

## 6. Go / no-go redesign triggers

Redesign instead of patching if any remain unresolved after one bounded redesign cycle:
- Tier2 p95 cannot remain under target at declared hot set size
- engram expansion tail latency cannot be capped
- reconsolidation leaves cache/index divergence
- forgetting still removes high-utility memories under realistic load
- daemon mode introduces correctness divergence not present in standalone mode

## 7. Suggested additions to the original milestone structure

For every milestone, add five explicit sections:
- Restrictions
- Benchmarks
- Regression budget
- Kill criteria
- Exit artifacts

### Exit artifacts example
For each stage completion:
- benchmark report
- failure matrix
- design note
- migration note if schema changed
- rollback note if behavior changed
- ops note if background jobs changed

## 8. Suggested benchmark tables to add directly into the plan

### Retrieval benchmark template
| Scenario | Corpus size | Warm/Cold | Concurrency | p50 | p95 | p99 | Pass? |
|---|---:|---|---:|---:|---:|---:|---|

### Encode benchmark template
| Scenario | Cache hit rate | Avg payload size | p50 | p95 | p99 | Pass? |
|---|---:|---:|---:|---:|---:|---|

### Consolidation benchmark template
| Job | Items moved | Foreground load | p95 foreground delta | Duration | Pass? |
|---|---:|---:|---:|---:|---|

### Forgetting benchmark template
| Prune class | Eligible set | False prune rate | Restore success | Recall quality delta | Pass? |
|---|---:|---:|---:|---:|---|

## 9. Quality gates

### Correctness
- no silent contradiction overwrite
- no lost committed memories
- no orphaned engram edges after mutation
- no stale cache after accepted update

### Utility
- retrieval precision remains acceptable on canonical corpora
- context reranking improves same-context retrieval
- forgetting reduces noise without destroying key facts
- consolidation improves utility, not just storage efficiency

### Operability
- doctor can detect seeded failure cases
- export/import works
- repair paths documented
- dashboard metrics exist for tier hit rates, cache hit rates, p95/p99

## 10. Recommended structure for the true mega-plan

Keep your current core sections intact, then append these new parts:

12. Stage Gates
13. Stage Restrictions
14. Benchmark Protocol
15. Performance Budgets
16. Go / No-Go Decision Rules
17. Quality Gates
18. Risk Register
19. Repair & Operations Acceptance
20. Research Notes / Falsifiable Claims

That gives you a mega-plan that still feels like your own plan, not a replacement.
