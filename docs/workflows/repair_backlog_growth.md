---
id: repair_backlog_growth
title: Repair Backlog Growth
source_doc: docs/workflows/repair_backlog_growth.md
section: ## Repair Backlog Growth
summary: Drain queued repair work and verify the queue stays empty before declaring the runtime recovered.
---
# Repair Backlog Growth

## Steps
- Inspect the current repair queue depth and identify whether work is merely in flight or repeatedly rolling back.
- Resolve the highest-risk repair target first, then re-check the queue to confirm backlog pressure is falling.
- Once the queue is empty, run the relevant parity or verification check so recovery is backed by evidence instead of queue state alone.
