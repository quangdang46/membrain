# Contributor Execution and Handoff Document

This document captures the golden path for agents and human contributors, defining precisely how work should be selected, executed, validated, and handed off. This ensures any contributor can safely resume work without re-deriving the state of the task or reverse-engineering which validation has been run.

## 1. Contributor Execution Golden Path

Every work session in this repository must follow this sequence to maintain a healthy shared backlog and prevent conflicting work.

### 1.1 Orient on the Canon
- Do not start by guessing what needs to be built.
- Read `docs/PLAN.md` for product behavior.
- Use `docs/INDEX.md` and `docs/CONTRIBUTING.md` for document hierarchy and evidence rules.
- Review `AGENTS.md` for execution rules.

### 1.2 Pick Ready Work
- Do not invent work that is not in the `.beads/` backlog.
- Use the `bv` (graph-aware triage) or `br ready --json` (beads tracker) to pick the highest priority, unblocked issue.

### 1.3 Claim the Bead and Reserve Scope
- Move the selected bead to `in_progress` (`br update <id> --status=in_progress`) immediately to signal to other agents that it is claimed.
- If using an MCP workflow, reserve the narrowest practical file/glob surface using `file_reservation_paths` with the Bead ID so others don't concurrently rewrite your files.

### 1.4 Announce Start
- Post a message in the Agent Mail thread (the thread ID should match the Bead ID, e.g., `mb-dve.6.3`) explaining what scope is being worked on and what validation you'll eventually run.

### 1.5 Implement with Discipline
- Observe canonical architecture boundaries (e.g., policy goes in `membrain-core/policy`, not in the CLI adapter).
- Never overwrite or stash concurrent edits from other contributors.
- If a document conflicts with `PLAN.md` or a rule is ambiguous, pause and create a follow-up bead instead of making a silent guess.

---

## 2. Doc-Propagation Contract

When a bead modifies architecture, scope, or product behavior, the code is only half the work. The contract must be propagated accurately up the documentation hierarchy.

1. **Rule of Precedence**: `docs/PLAN.md` is the canonical contract. If you discover a gap that changes core product thesis, that must be reflected (via a follow-up or explicit patch) before downstream docs are updated.
2. **Architecture Impact**: If your change shifts a module boundary or changes where data is stored, update `docs/ARCHITECTURE.md`.
3. **Data Shape Impact**: If your change introduces a new core column, metadata field, or governance hook, update `docs/DATA_SCHEMAS.md` and/or `docs/SECURITY_GOVERNANCE.md`.
4. **Testing Impact**: If you change how performance or confidence is measured, update `docs/TEST_STRATEGY.md` and the evidence checklist in `docs/CONTRIBUTING.md`.
5. **Workflow Rules**: Only update `AGENTS.md` or this document if the mechanics of coordination and handoff themselves need changing.

---

## 3. The Handoff Contract (Minimum Completion Payload)

Before you end a session or hand off a bead to another contributor, you must provide the **Minimum Completion Payload**. This guarantees the next agent (or human) does not have to rebuild your mental context.

Your final session artifact or mail message MUST include:

1. **Bead Status**: Which Bead IDs were touched, and what is their absolute final status (`completed`, still `in_progress`, `blocked`, or intentionally deferred)?
2. **Changed Surfaces**: A clear list of files, documents, and interfaces that were modified.
3. **Validation Provenance**: Exactly what validation was run (e.g., `cargo check --all-targets`, specific deterministic tests, or `bench` scripts). Explicitly list what passed, what failed, and what was intentionally deferred.
4. **Open Risks and Questions**: Document any unresolved ambiguity, partial implementations, or failed design assumptions that the next contributor must address immediately rather than discovering silently later.
5. **Next Recommended Step**: Provide clear, deterministic commands or commands for the next contributor (`br ready`, `bv --robot-next`, or specific follow-up beads they should start on next).
6. **Bead Synchronization**: The session MUST sync the beads to disk via `br sync --flush-only` prior to handoff.

If any of these 6 elements are missing, the handoff is incomplete and the reviewer (or next agent) should reject it.
