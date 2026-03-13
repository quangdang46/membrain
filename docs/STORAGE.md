# STORAGE

## Tier definitions
### Tier1
In-process hot memory with bounded size.

### Tier2
Warm indexed store supporting exact and bounded hybrid retrieval.

### Tier3
Cold durable archive supporting cheap storage and reconstructable recall.

## Principles
- every tier has a different cost profile
- tier transitions must be explicit
- payloads and summaries should be separated
- rebuild must be possible from durable evidence
- indexes must be repairable
- tier1 should avoid large payload ownership
