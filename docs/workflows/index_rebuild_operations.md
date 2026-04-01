---
id: index_rebuild_operations
title: Index Rebuild Operations
source_doc: docs/workflows/index_rebuild_operations.md
section: ## Index Rebuild Operations
summary: Rebuild derived indexes and confirm parity before clearing degraded cache or index alerts.
---
# Index Rebuild Operations

## Steps
- Confirm which index families are degraded or bypassed in the current doctor or health output.
- Run the bounded repair action for the affected index targets and capture the resulting verification artifact.
- Re-run doctor or health checks to verify the degraded reason and recommended runbook have cleared.
