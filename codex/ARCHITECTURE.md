# ARCHITECTURE

## 1. System view

### Write path
ingest -> normalize -> classify -> score -> route -> persist -> schedule background jobs

### Read path
query context -> retrieval planner -> tier1 scan -> tier2 candidate generation ->
optional tier3 fallback -> graph expansion -> ranking -> packaging -> reinforcement updates

## 2. Major components
- Ingestor
- Normalizer
- Fast encoder
- Salience engine
- Tier router
- Tier1 hot store
- Tier2 warm indexed store
- Tier3 cold archive
- Association graph
- Retriever and ranker
- Consolidator
- Forgetter
- Observability and governance layer

## 3. Design invariants
1. Hot path must stay bounded.
2. Every memory item must have provenance.
3. No hard delete without policy approval or retention expiry.
4. Graph edges must remain repairable from lineage or re-indexing.
5. Tier routing decisions must be traceable.
6. Retrieval ranking must be explainable after the fact.
7. Consolidation must never silently discard authoritative evidence.
8. Contradictions must be represented explicitly, not hidden by overwrite.
