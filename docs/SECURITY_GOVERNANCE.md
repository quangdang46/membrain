# membrain — Security and Governance Contract

> Canonical source: `PLAN.md` Sections 12.3, 13, 23, 25, 26, 34, and Feature 9.

This document freezes the governance contract for namespace isolation, access-control layering, redaction behavior, and incident handling. If this document diverges from `PLAN.md`, the plan wins.

## Core governance invariants

1. Namespace isolation is checked before expensive retrieval, storage fanout, graph expansion, or background mutation work.
2. Workspace ACL, agent ACL, and session visibility apply equally to writes, reads, repair, maintenance, and background execution.
3. Shared or public visibility is explicit, not accidental; no request widens itself into cross-namespace scope by default.
4. Policy denials and redactions remain explainable without leaking protected existence, counts, handles, or payload details.
5. Isolation violations are incident-grade events, not routine validation noise.

## Policy evaluation order

Every request-path and background action must apply governance in this order:

1. bind authenticated actor or scheduled-job identity
2. resolve one effective namespace
3. validate workspace ACL, agent ACL, and session visibility
4. validate visibility or sharing scope for any requested cross-namespace surface
5. apply retention, legal-hold, deletion, and redaction policy
6. only then proceed to storage, index, graph, or retrieval work

If any earlier step fails, later expensive work must not run speculatively.

## Effective namespace contract

Every operation executes in exactly one effective namespace, even when additional shared or public surfaces are allowed.

### Resolution rules

- An explicit `namespace` parameter wins when it is well-formed and policy-allowed.
- If `namespace` is omitted, a wrapper may bind a caller-scoped default only when one deterministic default exists from authenticated context, session binding, scheduler ownership, or equivalent durable configuration.
- If no deterministic default exists, the request fails as a validation error before touching retrieval or storage engines.
- Malformed or unknown namespace values are validation failures, not policy denials.
- Cross-namespace reads or writes are denied unless they use an explicit approved sharing path such as visibility-scoped sharing, fork or merge surfaces, or a policy-approved cross-namespace relation flow.
- Background jobs must inherit namespace scope from the scheduled record or repair plan; they must never infer it from ambient process state.

### Request-behavior matrix

| Request shape | Required behavior |
|---|---|
| Explicit namespace, same-scope request | Execute only after ACL and policy checks pass. |
| Omitted namespace with one deterministic default | Bind the effective namespace, record it in traces/audit, and continue normally. |
| Omitted namespace with no deterministic default | Return validation failure before retrieval, indexing, or writes begin. |
| Malformed or unknown namespace | Return validation failure without probing protected data. |
| Cross-namespace request without explicit sharing semantics | Return policy denial without revealing whether protected data exists in the target namespace. |
| Cross-namespace request with explicit sharing semantics | Allow only the explicitly shareable slice permitted by policy; keep source and target namespaces inspectable in audit/explain surfaces. |
| Background or repair job without persisted namespace binding | Refuse execution or leave the job queued as a scoped configuration error. |

## ACL and visibility surfaces

### Namespace isolation

- Namespace is the primary isolation boundary for memory existence, retrieval, mutation, and maintenance.
- The system must not do best-effort fallback from a denied namespace into a broader or global search.
- Shared or public knowledge must live behind explicit visibility metadata or approved mapping tables, not behind accidental omission of namespace filters.

### Workspace ACL

- Workspace ACL gates access to workspace-bound records inside a namespace.
- A namespace-allowed caller still may be denied workspace-scoped data when the workspace binding is outside the caller's allowed surface.
- Workspace denial may still permit redacted higher-level summaries when policy explicitly allows them; otherwise the response is a full denial.

### Agent ACL

- Agent ACL controls private and collaboration-sensitive memories within a namespace.
- Agent identity must be evaluated before a wrapper broadens to shared/public surfaces.
- Agent ACL applies equally to reads, writes, background jobs, and repair operations that might otherwise surface protected content.

### Session visibility

- Session visibility controls whether session-scoped memories and raw session identifiers may be surfaced.
- A request may be allowed to read a memory while still receiving redacted session detail.
- Session visibility must not be bypassed by inspect, explain, repair, or audit helpers.

### Cross-tenant protection

- Cross-tenant or cross-user isolation failures are security incidents.
- Derived artifacts such as caches, graph projections, summaries, and explain traces must preserve the same namespace and visibility boundaries as durable truth.
- Namespace-aware keys and filters are required for caches, repair, and prefetch layers so warm state cannot leak across tenants.
- When cache, repair, or degraded-serving paths are bypassed, stale, or rebuilding, explain and audit surfaces must still preserve machine-readable evidence that the namespace boundary was enforced rather than silently collapsing the event into an ordinary miss or generic internal error.

## Denial, redaction, and explain contract

### Validation failure

Use validation failure when the request is malformed, missing required scope, references an unknown namespace, or combines parameters illegally.

- Validation failures must happen before candidate generation or storage mutation.
- The response may explain what is structurally wrong, but it must not imply that a protected namespace or record exists.

### Policy denial

Use policy denial when the request is structurally valid but the caller is not entitled to the requested scope.

- Policy denial must not reveal protected counts, memory IDs, candidate rankings, workspace handles, or whether a denied target currently contains matching data.
- Explain surfaces may name the denial class or violated policy family, but not the protected contents of the denied surface.
- Cross-namespace denials must behave the same across CLI, daemon, MCP, IPC, repair, and background-triggered flows.

### Redacted success

Use redacted success when the caller may know an operation succeeded but is not entitled to every returned field.

- Redacted fields must stay distinguishable from absent, capped, or unknown fields.
- Redaction must preserve enough machine-readable markers for inspect, explain, and audit to say that policy hid a field intentionally.
- A redacted success must not silently widen into a policy denial later in the same flow; the boundary between allow, redact, and deny must remain inspectable.

## Approved sharing and cross-namespace flows

- `visibility`, `namespace_id`, `include_public`, `share`, `fork`, and `merge` are explicit widening mechanisms; they do not weaken namespace isolation for ordinary requests.
- `include_public` widens recall only to policy-approved shared or public surfaces and never to private namespaces.
- Cross-namespace relations or links require explicit policy support and must preserve both endpoint namespaces in durable records.
- Repair, cache warmup, and background maintenance may operate across multiple namespaces only when the plan enumerates those namespaces explicitly and audit surfaces preserve the touched scope.

## Incident and audit requirements

- Any confirmed or suspected namespace-isolation breach must emit an auditable incident-grade event.
- Incident handling should isolate the affected namespace or shard, preserve recent audit evidence, and move affected shared surfaces into degraded, read-only, or offline service until validation passes.
- Isolation incidents must be visible to operators as security events, not ordinary validation counters.
- Post-incident validation must prove that namespace filters, visibility rules, and redaction behavior are restored across durable truth and derived surfaces.

## Policy precedence

When policies overlap, precedence must be deterministic and auditable:

1. hard legal/compliance constraints
2. namespace isolation and tenant boundary checks
3. workspace/agent/session visibility rules
4. sharing and visibility expansion rules
5. redaction rules for partially visible content
6. convenience or ranking behavior

Ranking, retrieval quality, cache warmth, and repair convenience never outrank namespace or governance constraints.

## Evidence minimum for namespace-contract changes

Any change that alters namespace, ACL, sharing, denial, or redaction semantics must leave behind evidence for:

- explicit same-namespace allow path
- deterministic default-namespace binding
- missing-namespace rejection when no default exists
- malformed or unknown namespace validation failure
- cross-namespace denial without leakage of protected existence or counts
- approved shared/public access path
- background-job and repair-path preservation of namespace scope
- parity across CLI, daemon, IPC, and MCP surfaces
