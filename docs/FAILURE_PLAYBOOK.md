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

## 7. Planner budget blow-up

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

## 8. Graph fanout explosion

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

## 12. Retention-policy bug

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

