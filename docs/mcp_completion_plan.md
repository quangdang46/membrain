# Membrain MCP Completion Plan

## Purpose

Close the remaining gap between Membrain's current bounded MCP implementation and the contract that agents, docs, and tests believe they are using.

This is not a plan to implement the full long-term aspirational MCP catalog from `docs/PLAN.md`. It is a plan to make the **current live bounded MCP surface** coherent, well-tested, and truthfully documented so agents can rely on it without guessing.

## Current Truth

The repository already has a real MCP-capable surface:

- `membrain mcp` runs as a stdio transport adapter.
- MCP initialization and transport handlers exist in the daemon runtime.
- `tools/list` and `tools/call` are implemented for a bounded six-tool catalog: `encode`, `recall`, `inspect`, `why`, `health`, `doctor`.
- transport-side discovery and runtime helpers exist: `resources.list` / `resource.read`, `streams.list`, and `shutdown`.
- `prompts/list` and `prompts/get` are also recognized, but today they are effectively placeholder surfaces rather than meaningful product features.
- shared core packaging can already emit `archival_recovery_partial` when partial restore state truly shaped the returned retrieval envelope.

The remaining problem is not "MCP does nothing." The remaining problem is that the live contract is still narrower, more uneven, and less crisply proven than the surrounding docs and plan language suggest.

## Problems To Solve

### 1. Live transport contract is not described crisply enough

The codebase now has a bounded but real MCP transport and tool layer, yet the docs still mix:

- the live bounded runtime surface
- future target-contract tool families
- transport helper methods
- placeholder prompt handlers

That invites agents to over-assume what is truly implemented.

### 2. Degraded retrieval parity is still not fully proven across MCP transport

Normal success-path recall parity is now in good shape, but the repo still tracks one partial gap: transport-level proof that `archival_recovery_partial` survives CLI, daemon, and MCP surfaces unchanged when degraded partial-restore state genuinely shapes the envelope.

### 3. Discovery/catalog semantics are still a little muddy

The runtime recognizes more than just the six callable tools:

- `tools/list`
- `tools/call`
- `resources.list`
- `resource.read`
- `streams.list`
- `shutdown`
- `prompts/list`
- `prompts/get`

The project needs one explicit truth table stating which of these are:

- live and supported
- live but intentionally bounded
- compatibility-only or placeholder
- future-contract only and not yet shipped

### 4. The end-to-end proof story is too thin for the remaining MCP edge cases

The repo already has strong unit and parity coverage, but the last MCP edge cases need explicit regression artifacts so future doc drift is harder.

## Goals

This plan is complete when all of the following are true:

1. The bounded live MCP contract is stated in one consistent way across `AGENTS.md`, `docs/MCP_API.md`, `docs/CLI.md`, `docs/README.md`, and `MISSING.md`.
2. `tools/list` truthfully advertises only the callable tools that are really supported today, with descriptions and schemas aligned to actual runtime behavior.
3. Discovery and helper transport methods are explicitly classified as transport surfaces rather than callable memory tools.
4. Placeholder prompt surfaces are either documented as intentionally empty/minimal or removed from the live advertised contract.
5. The degraded partial-restore marker `archival_recovery_partial` is proven to survive MCP transport unchanged when the shared retrieval envelope surfaces it.
6. A logging-heavy MCP e2e artifact proves initialization, tool discovery, recall/explain parity, degraded marker parity, resource discovery, and shutdown behavior.

## Non-Goals

This plan does not attempt to:

- implement the full aspirational `memory_*` MCP family from `docs/PLAN.md`
- make `membrain mcp` the authoritative warm runtime instead of the daemon
- invent MCP-only semantics that differ from CLI or daemon/JSON-RPC
- ship rich prompt catalogs unless a later plan explicitly chooses to make prompts a real product surface

## Workstreams

## Workstream A — Canonicalize the live MCP contract

Create one canonical "live now" contract table and make every docs surface defer to it.

This table must distinguish:

- callable MCP tools
- transport discovery methods
- runtime resources/streams
- placeholder prompt methods
- future-contract-only tool families

Acceptance criteria:

- no doc claims a broader live MCP surface than the runtime actually exposes
- no doc hides a truly live transport method that agents may rely on
- stdio-versus-daemon runtime authority remains explicit

## Workstream B — Finish degraded retrieval parity through MCP

Add transport-level proof for the already-landed shared-core marker behavior:

- when partial archival recovery actually shapes the retrieval envelope, MCP responses must preserve `archival_recovery_partial`
- when ordinary recall succeeds without degraded partial restore, MCP must not invent that marker

Acceptance criteria:

- daemon-path tests prove the marker is preserved through MCP envelopes
- stdio `membrain mcp` proof covers the same scenario end-to-end
- docs describe the marker as a shared retrieval-envelope fact, not an MCP-local invention

## Workstream C — Tighten tool discovery and callable-tool semantics

Make `tools/list` a trustworthy live source of truth for callable MCP tools.

This includes:

- verifying the six callable tool entries match the real runtime argument names and result shapes
- making sure their descriptions reflect bounded current behavior rather than long-term aspirational behavior
- deciding how prompts should be represented today:
  - either explicitly documented as empty/minimal placeholders
  - or moved out of the advertised live contract if that is cleaner

Acceptance criteria:

- `tools/list` output, docs, and tests all agree on the live callable tool catalog
- prompt behavior is no longer ambiguous
- transport-side discovery methods are not conflated with callable memory tools

## Workstream D — Strengthen the MCP proof harness

Expand the repo's MCP regression story so the remaining edge cases are easy to verify after future changes.

Required proof areas:

- initialize and initialized notification flow
- `tools/list` and `tools/call`
- resource and stream discovery
- happy-path recall / inspect / why parity
- degraded `archival_recovery_partial` parity
- truthful shutdown behavior

Acceptance criteria:

- `crates/membrain-daemon/tests/e2e_mcp.sh` or equivalent scripted proof covers the live bounded MCP contract with readable logs
- targeted Rust tests cover the exact machine-readable fields that must not drift
- MISSING/doc rows can point to concrete proof artifacts instead of broad prose

## Execution Order

1. Canonicalize the live contract vocabulary and truth table.
2. Add or fix the missing degraded-marker parity tests.
3. Align `tools/list`, prompts, and docs to the verified runtime behavior.
4. Expand the e2e MCP proof artifact and then update gap tracking/docs references.

## Proposed Bead Breakdown

1. MCP live-contract truth table and docs alignment
2. MCP degraded archival-recovery parity tests
3. `tools/list` and prompt-surface truthfulness cleanup
4. MCP logging-heavy end-to-end proof artifact expansion

Dependencies:

- workstream B depends on understanding the canonical live contract from workstream A
- workstream C depends on the same contract table from workstream A
- workstream D depends on B and C so the proof harness matches final truth

## Done Definition

This plan is done when a fresh agent can answer all of the following without hedging:

- What does `membrain mcp` really expose today?
- Which parts are callable tools versus transport/discovery helpers?
- Is the bounded MCP surface semantically aligned with CLI and daemon/JSON-RPC?
- Does degraded partial archival recovery survive the MCP envelope correctly?
- Which test or e2e artifact proves each of those claims?
