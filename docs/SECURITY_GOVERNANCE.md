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

## Early enforcement points by operation surface

Every major surface must perform governance checks before it starts the expensive, durable, or externally visible part of the flow. The denial or redaction artifact must stay parity-consistent across CLI, daemon, IPC, and MCP wrappers even when those wrappers format the outcome differently.

| Surface | Earliest required enforcement point | Work that must not start before the check passes | Required denial or redaction artifact |
|---|---|---|---|
| Encode / `memory_put` / observe-style writes | After actor binding and effective namespace resolution, before duplicate-family candidate generation, storage fanout, enrichment, or payload persistence | Similarity search, duplicate shortlist generation, payload writes, graph/index updates, deferred-job enqueue | Validation failure or policy denial that names the blocked policy family without exposing protected candidate counts, neighboring record existence, or hidden namespace details |
| Exact read / `memory_get` | Before canonical handle lookup is allowed to dereference durable storage or payload state | Payload fetch, cold-tier hydrate, lineage expansion, redacted-field materialization | Policy denial or redacted success that keeps exact lookup from becoming an authorization bypass |
| Search / `memory_search` | Before index fanout, candidate generation, workspace/session expansion, or ranking work | Index scans, vector or lexical fanout, candidate counting, ranking, payload fetch | Validation failure or policy denial with no protected candidate counts, record existence, workspace handles, or session identifiers |
| Recall / `memory_recall` / ask-style retrieval | Before candidate generation, graph expansion, cold-tier planning, or packaging work | Candidate generation, graph traversal, cold payload fetch, prompt packaging, route fallback that would widen scope | Explicit denial, redacted success, or validation failure that preserves inspectable policy filters and never recodes a denial as low confidence, empty evidence, or ordinary miss |
| Share / unshare / visibility widening | Before any visibility mutation, cross-namespace serialization, recipient packaging, or durable share-record creation | Share-record writes, recipient-visible packaging, external serialization, background fanout | Policy denial or redacted success that preserves source namespace, target scope, and governing visibility policy in inspect/audit surfaces without leaking unauthorized payload fields |
| Forget / delete / archive transitions | Before tombstone planning, payload destruction, retention demotion, or deletion job scheduling | Payload removal, archive compaction, tombstone mutation, hard-delete execution, retention-tier rewrite | Explicit policy denial for pinned or hold-protected records; allowed paths must remain distinguishable as archival, redacted success, or hard deletion |
| Restore / repair / rebuild | Before a job dereferences archived payloads, rebuild inputs, or multi-namespace repair plans | Archived payload hydrate, index rebuild, graph repair, restore writes, cross-namespace repair fanout | Validation failure, scoped configuration error, or policy denial that preserves hold state and namespace scope rather than silently skipping into an unscoped rebuild |
| Mutation flows such as link, merge, fork, pin, or metadata updates | Before existing-state fetches that would inspect protected neighbors or before durable mutation begins | Protected-neighbor inspection, relation expansion, mutation writes, downstream maintenance enqueue | Policy denial or redacted success that keeps protected relation endpoints, prior state, and widening intent hidden unless policy allows them |

Surface-specific rules:

- CLI, daemon, IPC, and MCP wrappers may differ in presentation, but they must preserve the same effective namespace, denial class, redaction markers, and audit-visible policy family.
- Background jobs and scheduler-owned actions must bind persisted namespace, actor, and policy context before prefetch, repair, restore, deletion, or mutation stages begin; ambient process state is never sufficient.
- Approved widening flows may expose only the explicitly shareable slice. They must not fetch or serialize broader payloads first and redact them afterward.
- When a flow is denied before expensive work, inspect, explain, and audit surfaces must still preserve machine-readable evidence that the early gate fired, including whether the blocked step was candidate generation, payload fetch, background mutation, or external serialization.

## Governance entities and relationship model

This section freezes the core governance vocabulary so later storage, CLI, MCP, and sharing work can reuse one model instead of inventing local policy terms.

### Namespace

- A namespace is the canonical ownership and isolation boundary for a memory's identity, existence, retrieval, mutation, repair, and audit trail.
- A memory belongs to exactly one authoritative namespace for its full lifetime unless an explicit fork/merge or other policy-approved migration contract says otherwise.
- Shared visibility can widen who may read a slice of namespace-owned content, but it does not mint a second canonical namespace or split durable ownership.

### Agent identity

- `agent_id` names the actor that created, derived, shared, repaired, or otherwise touched a record inside its authoritative namespace.
- Agent identity is part of provenance and ACL evaluation, but it is not an authorization shortcut that can replace namespace binding, workspace ACL, or session visibility.
- Agent-scoped private memories may be readable only by the owning or policy-approved agent even when they live inside a larger shared namespace.

### Visibility classes

- `private` visibility means the memory stays bound to its authoritative namespace and the narrower workspace, agent, and session rules that apply there.
- `shared` visibility means the memory remains owned by one authoritative namespace but may be surfaced through an explicit policy-approved widening path to another allowed namespace, workspace, or agent scope.
- `public` visibility means the memory may be recalled through approved public/shared paths, but it still retains one authoritative namespace, lineage chain, and audit trail.
- Visibility changes widen or tighten readable scope; they do not change canonical identity, provenance, or the namespace that owns retention and audit obligations.

### Ownership and widening rules

- Namespace ownership answers where a memory canonically lives; visibility answers which additional callers may see an approved slice of it.
- A later `share`, `unshare`, `include_public`, `fork`, or `merge` flow must start from this distinction instead of treating visibility as a second identity system.
- Redaction, denial, and audit requirements continue to apply after widening; visibility never turns protected data into ungoverned global data.

## Effective namespace contract

Every operation executes in exactly one effective namespace, even when additional shared or public surfaces are allowed.

### Resolution rules

- An explicit `namespace` parameter wins when it is well-formed and policy-allowed.
- If `namespace` is omitted, a wrapper may bind a caller-scoped default only when one deterministic default exists from authenticated context, session binding, scheduler ownership, or equivalent durable configuration.
- If no deterministic default exists, the request fails as a validation error before touching retrieval or storage engines.
- Malformed or unknown namespace values are validation failures, not policy denials.
- Cross-namespace reads or writes are denied unless they use an explicit approved sharing path such as visibility-scoped sharing, fork or merge surfaces, or a policy-approved cross-namespace relation flow.
- Query-intent classification and override controls such as `ask` / `override-intent` are retrieval-strategy hints, not scope selectors. They may only tune bounded retrieval or packaging behavior inside the already-authorized effective namespace unless the request also carries a separate explicit widening control that policy allows.
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
- Query-intent routing must not silently recode a policy denial as low confidence, empty evidence, or a harmless route fallback. If an `ask` request is denied for namespace or sharing reasons, the denial stays explicit even when intent classification itself succeeded.
- Cross-namespace denials must behave the same across CLI, daemon, MCP, IPC, repair, and background-triggered flows.

### Redacted success

Use redacted success when the caller may know an operation succeeded but is not entitled to every returned field.

- Redacted fields must stay distinguishable from absent, capped, or unknown fields.
- Redaction must preserve enough machine-readable markers for inspect, explain, and audit to say that policy hid a field intentionally.
- A redacted success must not silently widen into a policy denial later in the same flow; the boundary between allow, redact, and deny must remain inspectable.

## Retention, legal-hold, deletion, and forgetting outcomes

Retention and legal-hold rules constrain deletion, forgetting, archival, restore, and any other lifecycle action that could remove or narrow authoritative evidence.

### Retention classes and default outcomes

- `volatile`, `normal`, `durable`, and `pinned` express intended retention behavior; ranking pressure, cache eviction, or storage-tier movement must not silently rewrite that intent.
- Retention class influences whether a memory may be archived, how aggressively forgetting may demote it from ordinary recall, and whether an explicit deletion request is eligible for policy review.
- Utility-driven forgetting may archive or demote eligible memories, but it must not masquerade as legal, compliance, or operator-approved hard deletion.
- `pinned` or otherwise policy-protected memories stay ineligible for utility-driven archival or deletion until the protecting policy marker is lifted through an explicit authorized path.

### Legal-hold and policy-hold behavior

- Legal hold, compliance lock, deletion hold, and equivalent `policy_flags` override convenience deletion, forgetting, archive compaction, and payload-destruction paths.
- A held memory may still be denied or redacted to an unentitled caller, but the hold must preserve the authoritative evidence and audit trail required for later inspection or policy review.
- Restore, repair, migration, rollback, and rebuild flows must preserve hold state exactly; they must not clear or reinterpret a hold just because a record changes tier, schema version, or payload layout.
- Background maintenance must fail closed when a hold-bearing record reaches a destructive or semi-destructive step without an explicit policy-approved continuation path.

### User-visible outcome matrix

| Situation | Required outcome |
|---|---|
| Caller lacks entitlement to know whether a protected item exists | Return policy denial without exposing existence, counts, or handles. |
| Caller may know the operation succeeded but not see protected fields | Return redacted success with explicit redaction markers. |
| Caller requests forgetting or deletion for a hold-protected or pinned item | Return explicit policy denial naming the governing policy family without clearing the hold or deleting evidence. |
| Caller requests deletion for an eligible item under an approved policy path | Allow only the policy-approved deletion scope, preserve tombstone or audit evidence when required, and keep the result distinguishable from archival. |
| Utility-driven forgetting runs on an eligible item | Archive or demote according to lifecycle policy; do not present the result as hard deletion. |
| Retention expiry or compliance workflow removes payload recoverability | Preserve the required tombstone, audit record, and policy markers instead of silently dropping the item from inspectable history. |

### Denial, redaction, and deletion distinctions

- Denial means the caller is not entitled to the requested action or scope; it does not imply whether the protected item exists or is held.
- Redaction means the caller may receive a bounded success response while protected fields, handles, payloads, or linkage remain hidden behind explicit policy markers.
- Archival means the memory leaves ordinary default recall but remains durable and inspectable through authorized restore or audit paths.
- Hard deletion is a separate policy-governed outcome reserved for explicit compliance, retention-expiry, or operator-authorized flows; it must preserve whatever tombstone or audit artifact the governing policy requires.
- Forgetting, demotion, or archive compaction must never silently become hard deletion because a lower-fidelity representation already exists.

### Evidence and parity requirements

Any surface that exposes governance-sensitive outcomes must keep the following distinctions machine-readable and parity-checked across CLI, daemon, IPC, MCP, inspect, explain, repair, and audit flows:

- validation failure versus policy denial
- policy denial versus redacted success
- archival versus hard deletion
- ordinary retention pressure versus legal-hold or compliance-lock override
- payload removal versus durable tombstone or audit-evidence preservation

Every governance-sensitive outcome must also leave behind inspectable explain and audit evidence instead of transport-specific prose only.

#### Required explain fields

When a request is denied, redacted, narrowed by retention or hold policy, or widened through approved sharing, the explain surface for that outcome must preserve these machine-readable fields directly or through a stable referenced handle:

- `policy_summary.effective_namespace` — the single namespace that governed evaluation
- `policy_summary.policy_family` — the governing family such as namespace isolation, workspace ACL, agent ACL, session visibility, retention, legal hold, or visibility sharing
- `policy_summary.outcome_class` — validation failure, policy denial, redacted success, archival, hard deletion, or allowed-with-sharing
- `policy_summary.blocked_stage` — the earliest blocked step such as candidate generation, payload fetch, external serialization, background mutation, or destructive lifecycle execution
- `policy_summary.redaction_fields` — which fields were intentionally hidden when redacted success is allowed
- `policy_summary.retention_state` — the retention class and any legal-hold, compliance-lock, deletion-hold, or equivalent policy marker that materially shaped the outcome
- `policy_summary.sharing_scope` — source and target scope when approved widening or cross-namespace sharing materially shaped visibility

These fields must stay inspectable across every public interface. A wrapper may rename labels for presentation, but it must not drop, merge, or silently infer away any of these distinctions.

#### Required audit artifacts

Governance-sensitive execution must emit audit artifacts that let later operators prove what policy ran and what it prevented or allowed. At minimum, the audit trail must preserve:

- `event_family` for denial, redaction, retention override, hold-protected destructive request, approved sharing, namespace-isolation incident, or degraded governance enforcement
- `request_id` or equivalent correlation handle that joins explain, audit, and transport-visible outcomes
- `effective_namespace` plus any explicit source and target namespaces involved in approved widening or multi-namespace maintenance
- `policy_family`, `outcome_class`, and `blocked_stage` using the same semantics as explain surfaces
- `retention_state` and hold markers when lifecycle policy shaped the result
- `redaction_summary` when protected fields, actors, reasons, or payload slices were intentionally hidden from the caller
- `related_run`, `related_snapshot`, or equivalent maintenance correlation when repair, rebuild, restore, compaction, or incident response materially shaped the outcome

Audit artifacts must make denial, redaction, retention, and namespace-isolation outcomes inspectable even when the caller-facing transport returns only a bounded error or redacted success.

#### Parity-test obligations

Every future governance-sensitive change must ship parity evidence that proves the same scenario produces the same machine-readable governance outcome across CLI, daemon/JSON-RPC, IPC, MCP, inspect, explain, repair, and audit surfaces where that scenario is exposed.

The minimum parity matrix must cover:

- malformed or missing namespace as validation failure
- same-scope allow path with explicit effective namespace recording
- cross-namespace denial without protected existence or count leakage
- approved sharing or public-widening path with inspectable source and target scope
- redacted success that preserves redaction markers instead of collapsing to absence
- retention or legal-hold denial for forget, delete, archive, restore, repair, or rebuild flows
- archival versus hard deletion with distinct tombstone or audit preservation evidence
- namespace-isolation enforcement during degraded, cache-bypassed, repair, or background-job execution

Silent cross-surface divergence is explicitly forbidden. If one interface cannot yet preserve the canonical explain fields, audit artifacts, or outcome distinctions, the change is incomplete rather than allowed to ship with wrapper-specific behavior.

## Approved sharing and cross-namespace flows

- `visibility`, `namespace_id`, `include_public`, `share`, `fork`, and `merge` are explicit widening mechanisms; they do not weaken namespace isolation for ordinary requests.
- Later-stage resumable-goal stacks, blackboard checkpoints, and pause/resume state remain subject to the same effective-namespace, workspace, agent, and session visibility rules as the memories and task context they summarize. A checkpoint may aid resume or inspection, but it must not become a hidden cross-namespace handoff channel or a policy bypass around the underlying evidence.
- `share` and `unshare` mutate visibility scope, not canonical ownership: the memory keeps one authoritative namespace, identity, lineage, and audit trail even when widened access is allowed or later tightened again.
- `include_public` widens recall only to policy-approved shared or public surfaces and never to private namespaces.
- Approved shared/public access remains subordinate to workspace ACL, agent ACL, and session visibility; shared visibility is not a blanket bypass.
- Cross-namespace relations or links require explicit policy support and must preserve both endpoint namespaces in durable records.
- Repair, cache warmup, and background maintenance may operate across multiple namespaces only when the plan enumerates those namespaces explicitly and audit surfaces preserve the touched scope.
- Explain, audit, and cache-related metadata must preserve when approved widening materially shaped the visible result set instead of collapsing it into an ordinary same-namespace allow path.

## Preflight governance and blocked-action semantics

Risky actions and high-stakes action-oriented guidance must preserve the same governance outcome distinctions inside the preflight layer that later transports expose to users.

### Governance rules for preflight

- Policy, namespace, sharing, retention, and legal-hold checks run before preflight can report `ready` for a risky action.
- `preflight.allow`, `--force`, or any equivalent confirmation path may satisfy only the local confirmation step for the exact previewed scope and generation.
- Confirmation does not bypass namespace isolation, policy denial, retention rules, legal hold, or any other governance check.
- Missing confirmation, stale preview, stale generation, missing snapshot, blocked dependency, or confidence shortfall remain blocked-action states. They are not recoded as successful apply and should not be inflated into true rejection unless the request is malformed, impossible, or policy-denied.
- Public transports may format preflight differently, but they must preserve the same machine-readable `preflight_state`, `blocked_reasons`, per-check results, and audit correlation fields across CLI, daemon, IPC, and MCP.

### Required governance distinctions inside preflight

Any surface that exposes preview, preflight, or force-confirm behavior must keep these distinctions inspectable:

- missing confirmation versus policy denial
- stale preview or generation mismatch versus malformed request
- blocked readiness versus degraded execution
- local force-confirmation versus policy-approved authorization
- preview-only outcome versus destructive apply outcome

Silent divergence across transports is a contract violation. If one wrapper cannot preserve the canonical blocked-action semantics, the change is incomplete.

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
