# FAILURE PLAYBOOK

This document enumerates failure modes, signals, immediate mitigations, and longer-term fixes.

## Truth hierarchy during incidents

When incident evidence conflicts, operators resolve truth in this order:

1. authoritative durable records for memory identity, lineage, policy, and contradiction state
2. canonical relation tables and canonical content handles
3. derived summaries, checkpoints, and compaction artifacts
4. indexes, caches, graph materializations, and other acceleration sidecars

Derived artifacts can help diagnose failure, but they must not overrule durable truth.

## Canonical service-response matrix

This matrix freezes the operator-facing promise for each major incident class so later CLI, daemon, JSON-RPC, and MCP surfaces can communicate degraded state consistently instead of inventing ad hoc behavior.

| Failure mode | Safe serving mode | User-visible contract | Rollback eligibility | Post-incident proof |
|---|---|---|---|---|
| Tier1 overflow | reduced fidelity: bypass or shed hot-tier reuse, continue colder bounded reads | Explain or warnings surface hot-path degradation, colder fallback, and any capped recall quality; writes remain allowed if policy and durability are intact | Roll back only if a recent rollout changed hot-tier sizing, routing, or eviction semantics and colder serving does not restore budgets | Tier1 pressure returns below alert threshold; colder fallback no longer needed; candidate parity and durability remain intact |
| Tier2 index drift | reduced fidelity: durable-truth reads or exact lookup remain available while ranked/indexed recall runs colder, narrower, or temporarily skips affected indexes | Returned results or warnings state that indexed recall is degraded or bypassed; do not claim full-ranked coverage while index truth is in doubt | Roll back if drift started immediately after index/schema/routing rollout and rebuild plus colder fallback does not restore correctness | Rebuilt counts match durable truth; ranked recall and latency return to budget; explain traces stop surfacing index-bypass warnings |
| Tier3 segment corruption | fail closed for affected cold segments; serve only tiers whose authoritative inputs are verified | Users see affected archive or reconstruction surfaces as unavailable or partial rather than silently incomplete | Roll back if recent storage/layout change caused segment corruption and restoring prior segment layout is safer than live mutation | Segment integrity passes verification; affected archives reconstruct or remain explicitly unavailable; no silent payload gaps |
| contradiction masking | reduced fidelity for normal recall, but fail closed on any path that would present one hidden winner as consensus | Responses must surface conflict/open-override state or explicitly refuse simplified packaging while contradiction metadata is suspect | Roll back if a recent ranking, packaging, or conflict-resolution change introduced masking and durable conflict metadata remains intact | Conflicting siblings are again inspectable; packaging and explain surfaces preserve omission or override markers |
| false association | reduced fidelity: disable or narrow graph/association expansion while exact and entity-safe paths continue | Explain surfaces must say association expansion was narrowed or bypassed; no fabricated cross-links | Roll back if a recent graph/ranking release introduced the false-link behavior and disabling the new logic restores safe bounded recall | Graph-assisted paths no longer emit the bad linkage; exact and entity-centric parity against durable relation truth is restored |
| duplicate storms | reduced fidelity: enforce stricter dedup, narrower candidates, or colder fallback | Users may see capped or deduplicated results with explicit omission reasons; never silently loop or over-expand | Roll back if a recent duplicate-collapse, novelty, or candidate-generation change caused runaway duplication and bounded fallback does not stabilize | Candidate counts are back within caps; duplicate-family counters normalize; degraded duplication safeguards can be removed |
| planner budget blow-up | reduced fidelity: simpler planner, smaller budgets, or exact-first fallback | Responses warn that simpler bounded routing is active; latency protection wins over breadth | Roll back if a recent planner or routing rollout caused repeated budget blow-ups and simplified routing does not restore safe behavior | Planner counters stay within caps; normal route selection resumes without repeated degraded-mode entry |
| graph fanout explosion | reduced fidelity: graph-disabled or shallower bounded fallback | Responses state graph expansion was capped or disabled; omit unreachable graph-only context rather than inventing it | Roll back if a recent graph traversal or edge-generation change triggered the fanout explosion and safe caps do not contain it | Traversal counters remain within limits; graph-enabled recall resumes with explicit bounded traces |
| repair backlog growth | reduced fidelity while prior durable state remains authoritative; fail closed on write or transition classes whose correctness cannot be guaranteed | Users may see slower reads, stale-derived warnings, or temporary refusal of affected transitions; validation and policy failures must stay explicit instead of auto-retrying forever. Doctor surfaces should point operators at the repair-backlog runbook instead of leaving degraded state implicit. | Roll back if a recent rollout created systemic retryable internal failures and reverting is safer than replaying more broken work | Backlog drains, retry-budget exhaustion stops growing, and affected objects either finish repair or remain explicitly escalated |
| latency regression | reduced fidelity only if colder or narrower serving keeps correctness; fail closed if latency is caused by unsafe policy or namespace behavior | Users should see degraded-performance or narrowed-serving warnings, not semantically different results disguised as success | Roll back if a recent binary/config change caused regression and reverting is lower risk than extended degraded mode | Latency returns within budget and degraded-performance indicators clear |
| cross-namespace leakage | fail closed for the affected widening or namespace scope | Refuse or redact affected requests; never return protected counts, handles, or payload hints while containment is active | Roll back if a recent auth/policy/cache/routing change introduced leakage and reverting is the fastest safe containment | Isolation tests and live audit traces confirm namespace boundaries again; denied paths reveal no protected existence data |
| retention-policy bug | fail closed for purge/archive/payload-drop actions; safe reads may continue if policy-bearing durable records are intact | Users and operators are told retention-altering actions are frozen pending validation; no destructive maintenance proceeds on ambiguous policy state | Roll back if a recent forgetting, retention, migration, or governance rollout caused policy-marker drift and revert is safer than live mutation | Legal-hold, tombstone, retention, and purge-eligibility markers match durable expectations again; frozen destructive actions can resume |

## Rollback decision rules

- Rollback is appropriate only when a recent binary, config, schema, policy, or routing change is the likely cause and reverting is lower risk than continued degraded serving or live mutation.
- Prefer containment plus repair when the failure is confined to derived state and durable truth remains trustworthy.
- Prefer fail-closed containment over rollback when rollback could re-expose namespace leakage, policy drift, or destructive retention behavior.
- Never roll back by trusting stale derived artifacts over authoritative durable records.

## Post-incident verification contract

An incident is not resolved just because errors drop. Before clearing incident state, operators should prove:

1. the affected namespace, shard, or surface has returned to its intended serving mode,
2. durable truth, policy markers, and lineage remain authoritative and consistent,
3. degraded-mode, bypass, stale-warning, or fail-closed behavior has either been removed safely or remains explicitly declared,
4. explain, inspect, audit, and health degraded-status surfaces tell the same story across CLI, daemon, and MCP, and
5. any remaining follow-up repair or rollback work is explicitly tracked rather than implied.

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

