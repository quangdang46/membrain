# FAILURE PLAYBOOK

This document enumerates failure modes, signals, immediate mitigations, and longer-term fixes.

## Truth hierarchy during incidents

When incident evidence conflicts, operators resolve truth in this order:

1. authoritative durable records for memory identity, lineage, policy, and contradiction state
2. canonical relation tables and canonical content handles
3. derived summaries, checkpoints, and compaction artifacts
4. indexes, caches, graph materializations, and other acceleration sidecars

Derived artifacts can help diagnose failure, but they must not overrule durable truth.

## 1. Tier1 overflow

### Symptoms
- elevated errors
- degraded recall quality
- latency regression
- missing or duplicated memories

### Immediate response
- isolate namespace or shard if needed
- stop destructive jobs
- preserve forensic logs
- enable degraded mode if available

### Root-cause checklist
- validate lineage
- compare index counts to durable records
- inspect recent deploys
- inspect repair queue growth
- inspect compaction job history

## 2. Tier2 index drift

### Symptoms
- elevated errors
- degraded recall quality
- latency regression
- missing or duplicated memories

### Immediate response
- isolate namespace or shard if needed
- stop destructive jobs
- preserve forensic logs
- enable degraded mode if available

### Root-cause checklist
- validate lineage
- compare index counts to durable records
- inspect recent deploys
- inspect repair queue growth
- inspect compaction job history

## 3. Tier3 segment corruption

### Symptoms
- elevated errors
- degraded recall quality
- latency regression
- missing or duplicated memories

### Immediate response
- isolate namespace or shard if needed
- stop destructive jobs
- preserve forensic logs
- enable degraded mode if available

### Root-cause checklist
- validate lineage
- compare index counts to durable records
- inspect recent deploys
- inspect repair queue growth
- inspect compaction job history

## 4. Contradiction masking

### Symptoms
- elevated errors
- degraded recall quality
- latency regression
- missing or duplicated memories

### Immediate response
- isolate namespace or shard if needed
- stop destructive jobs
- preserve forensic logs
- enable degraded mode if available

### Root-cause checklist
- validate lineage
- compare index counts to durable records
- inspect recent deploys
- inspect repair queue growth
- inspect compaction job history

## 5. False association

### Symptoms
- elevated errors
- degraded recall quality
- latency regression
- missing or duplicated memories

### Immediate response
- isolate namespace or shard if needed
- stop destructive jobs
- preserve forensic logs
- enable degraded mode if available

### Root-cause checklist
- validate lineage
- compare index counts to durable records
- inspect recent deploys
- inspect repair queue growth
- inspect compaction job history

## 6. Duplicate storms

### Symptoms
- elevated errors
- degraded recall quality
- latency regression
- missing or duplicated memories
- candidate pools or duplicate-collapse counters growing beyond declared caps

### Immediate response
- isolate namespace or shard if needed
- stop destructive jobs
- preserve forensic logs
- enable degraded mode if available
- force colder or narrower fallback if duplicate expansion is outrunning bounded request-path budgets

### Root-cause checklist
- validate lineage
- compare index counts to durable records
- inspect recent deploys
- inspect repair queue growth
- inspect compaction job history
- inspect duplicate-collapse counters, candidate-count traces, and cache bypass or stale-warning evidence to confirm containment stayed bounded

## 7. Planner budget blow-up

### Symptoms
- elevated errors
- degraded recall quality
- latency regression
- missing or duplicated memories
- candidate-planning or routing-budget counters exceeding declared caps repeatedly

### Immediate response
- isolate namespace or shard if needed
- stop destructive jobs
- preserve forensic logs
- enable degraded mode if available
- force simpler or narrower bounded routing while planner budgets are being exceeded continuously

### Root-cause checklist
- validate lineage
- compare index counts to durable records
- inspect recent deploys
- inspect repair queue growth
- inspect compaction job history
- inspect planning-budget counters, candidate-count traces, and degraded-mode or cache-bypass evidence to confirm containment remained explicit

## 8. Graph fanout explosion

### Symptoms
- elevated errors
- degraded recall quality
- latency regression
- missing or duplicated memories
- graph-expansion depth or node counters hitting containment caps repeatedly

### Immediate response
- isolate namespace or shard if needed
- stop destructive jobs
- preserve forensic logs
- enable degraded mode if available
- force graph-disabled or shallower bounded fallback if traversal caps are being hit continuously

### Root-cause checklist
- validate lineage
- compare index counts to durable records
- inspect recent deploys
- inspect repair queue growth
- inspect compaction job history
- inspect traversal-cap telemetry, candidate-count traces, and omission reasons to confirm fanout containment stayed explicit

## 9. Repair backlog growth

### Symptoms
- elevated errors
- degraded recall quality
- latency regression
- missing or duplicated memories
- repeated transition-failure artifacts for the same object or edge family
- objects stuck in their prior durable state with repair pending

### Immediate response
- isolate namespace or shard if needed
- stop destructive jobs
- preserve forensic logs
- enable degraded mode if available
- distinguish retryable internal failures from terminal validation or policy failures before replaying work

### Root-cause checklist
- validate lineage
- compare index counts to durable records
- inspect recent deploys
- inspect repair queue growth
- inspect transition-failure artifacts and retry-budget exhaustion
- inspect compaction job history

## 10. Latency regression

### Symptoms
- elevated errors
- degraded recall quality
- latency regression
- missing or duplicated memories

### Immediate response
- isolate namespace or shard if needed
- stop destructive jobs
- preserve forensic logs
- enable degraded mode if available

### Root-cause checklist
- validate lineage
- compare index counts to durable records
- inspect recent deploys
- inspect repair queue growth
- inspect compaction job history

## 11. Cross-namespace leakage

### Symptoms
- elevated errors
- degraded recall quality
- latency regression
- missing or duplicated memories
- policy-denied, redacted, or cache-bypass paths disagreeing across interfaces

### Immediate response
- isolate namespace or shard if needed
- stop destructive jobs
- preserve forensic logs
- enable degraded mode if available
- freeze shared or public widening paths and bypass warm state until namespace filters are revalidated from durable truth

### Root-cause checklist
- validate lineage
- compare index counts to durable records
- inspect recent deploys
- inspect repair queue growth
- inspect compaction job history
- inspect cache-event, degraded-mode, and explain-trace evidence across CLI, daemon, and MCP surfaces to confirm the leak was contained without protected-count disclosure

## 12. Retention-policy bug

### Symptoms
- elevated errors
- degraded recall quality
- latency regression
- missing or duplicated memories
- legal-hold, tombstone, or purge-eligibility markers drifting from durable expectations

### Immediate response
- isolate namespace or shard if needed
- stop destructive jobs
- preserve forensic logs
- enable degraded mode if available
- freeze purge, archive, or payload-drop workflows until retention and legal-hold markers are revalidated from durable truth

### Root-cause checklist
- validate lineage
- compare index counts to durable records
- inspect recent deploys
- inspect repair queue growth
- inspect compaction job history
- inspect retention audits, legal-hold markers, and tombstone evidence to confirm migration or repair did not erase policy-bearing durable state

