---
id: tier2_index_drift
title: Tier2 Index Drift
source_doc: docs/workflows/tier2_index_drift.md
section: ## Tier2 Index Drift
summary: Contain Tier2 drift by checking authoritative durable records before trusting derived retrieval paths again.
---
# Tier2 Index Drift

## Steps
- Freeze any operator decision that depends on degraded Tier2 retrieval until authoritative state is checked.
- Compare the reported drift symptoms against the durable namespace state and recent repair history.
- Rebuild the affected Tier2 indexes, then verify retrieval parity before restoring the surface to normal service.
