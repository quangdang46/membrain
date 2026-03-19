## linehash — Stable Line-Anchored Editing

`linehash` is installed in this environment and must always be used for file-targeted reads and edits when a shell-based edit workflow is appropriate.

### Why to Prefer It

- Uses content-hashed line anchors like `12:ab` instead of fragile exact-text matching
- Rejects stale or ambiguous edits instead of guessing
- Works well for agent-driven file editing and concurrent-change detection

### Preferred Workflow

1. Read with anchors:
   ```bash
   linehash read <file>
   ```
2. Apply targeted edits by anchor:
   ```bash
   linehash edit <file> <line:hash> <new-content>
   linehash edit <file> <start:hash>..<end:hash> <new-content>
   linehash insert <file> <line:hash> <new-content>
   linehash delete <file> <line:hash>
   ```
3. If an anchor is stale or ambiguous, re-run `linehash read <file>` and retry using the new anchors.

### Rules

- Prefer `linehash` over ad-hoc text replacement when editing specific file lines
- Use `linehash read` to refresh anchors before editing files that may have changed
- Treat stale-anchor failures as a signal to re-read, not to force the edit

---

## Flywheel Swarm Required Operating Rules

These rules are mandatory for every swarm agent working in this repository.

1. **Rule 0 — Override:** Human instructions override `AGENTS.md`, backlog guidance, prior agent plans, and default workflow suggestions.
2. **Rule 1 — No deletion:** Never delete files or directories without explicit human permission.
3. **No destructive git:** Never run destructive git commands such as `git reset --hard`, `git clean -fd`, `git checkout -- <path>`, `git restore <path>`, `git push --force`, or equivalent history-rewriting shortcuts.
4. **Branch policy:** Work on `main`; never target or create `master` for this repository.
5. **No script edits:** Make code and documentation changes manually; do not mass-edit repository files with scripts unless the human explicitly asks for it.
6. **No file proliferation:** Do not create duplicate variants like `mainV2.rs`, `main_improved.rs`, `foo_new.py`, or parallel doc copies; edit the canonical file instead.
7. **Validation after changes:** Run validation appropriate to the changed surface before handoff. If you change Rust code, run `cargo check --all-targets` and then any bead-specific checks.
8. **Multi-agent awareness:** Expect concurrent edits from other agents. Never stash, revert, overwrite, or otherwise disturb changes just because you did not author them.
9. **Post-compaction reset:** After any compaction or major context loss, reread `AGENTS.md` before continuing.

### Swarm Tool Blur Review

- **`br`:** Canonical task state, dependencies, and status transitions.
- **`bv`:** Graph-aware routing and prioritization; use only `--robot-*` flags in agent workflows.
- **Agent Mail:** Agent identity, inbox/outbox threads, file reservations, and handoff coordination.
- **`cass` / `cm`:** Reuse prior session knowledge and procedural memory; avoid interactive modes and prefer automation-safe commands like `cass ... --robot|--json` and `cm context ... --json`.
- **`ubs`:** Bug-scan the changed surface before commit; fix real findings or document a justified deferral.
- **`dcg`:** Destructive Command Guard; treat blocked commands as safety signals and find a safer path instead of bypassing the guard.

---

## Documentation Precedence and Conflict Resolution

Use this order whenever docs disagree or when backlog work needs a canonical interpretation:

1. `docs/PLAN.md` is the canonical design contract.
2. Subsystem docs under `docs/` elaborate `PLAN.md`; they do not override it.
3. `docs/ARCHITECTURE.md` freezes workspace shape, module ownership seams, and read/write-path boundaries derived from `PLAN.md`.
4. `docs/INDEX.md` and `docs/CONTRIBUTING.md` define doc hierarchy, contributor evidence rules, and PR discipline that must stay aligned with `PLAN.md`.
5. `AGENTS.md` translates repository workflow, coordination, and execution discipline for active contributors; it should clarify how to operate within the canon, not invent competing product behavior, module ownership, or evidence thresholds.
6. Historical artifacts, legacy command references, logs, scratch plans, and older snapshot prose are informative only; they do not override the canonical contract.

### Conflict-Resolution Procedure

- If a subsystem doc disagrees with `PLAN.md`, `PLAN.md` wins until the conflict is resolved explicitly.
- If two non-canonical docs disagree, do not average them together or guess based on convenience; trace both back to `PLAN.md` and the active bead.
- Resolve ambiguity locally only when the canonical interpretation is directly supported by `PLAN.md` plus the established hierarchy in `docs/INDEX.md` and `docs/CONTRIBUTING.md`, and the fix does not invent new product behavior.
- Pause implementation on that point when the canonical interpretation is still ambiguous, when multiple reasonable readings would change product or workflow behavior, or when fixing the conflict would require creating new contract language rather than applying existing guidance.
- If implementation pauses, capture the conflict in the active bead; if no bead cleanly owns it, create a focused follow-up bead for the doc or workflow gap, note which files or sections conflict, and state the blocked decision that should not be guessed through.
- Patch documentation directly only when the change is a faithful clarification of already-canonical behavior; if the update changes scope, introduces new rules, or resolves a true contract gap, update or create the corresponding bead first so the backlog remains the audit trail.
- Notify other agents in the matching Agent Mail thread when you open or escalate the conflict so parallel contributors do not silently diverge.
- When `PLAN.md` contains older merged-snapshot text that conflicts with its explicit canonical corrections, thesis, invariants, or restrictions, the explicit canonical overlays win.

---

## MCP Agent Mail — Multi-Agent Coordination

A mail-like layer that lets coding agents coordinate asynchronously via MCP tools and resources. Provides identities, inbox/outbox, searchable threads, and advisory file reservations with human-auditable artifacts in Git.

### Why It's Useful

- **Prevents conflicts:** Explicit file reservations (leases) for files/globs
- **Token-efficient:** Messages stored in per-project archive, not in context
- **Quick reads:** `resource://inbox/...`, `resource://thread/...`

### Same Repository Workflow

1. **Register identity at session start:**
   ```
   ensure_project(project_key=<abs-path>)
   register_agent(project_key, program, model)
   ```
   Do this before sending mail, reserving files, or claiming active collaboration state.

2. **Reserve the smallest practical edit surface before editing:**
   ```
   file_reservation_paths(project_key, agent_name, ["src/**"], ttl_seconds=3600, exclusive=true)
   ```
   Prefer narrow file or glob reservations over broad repo-wide claims. Use exclusive reservations for active edits and shared reservations only when observing or coordinating without mutating the same surface.

3. **Coordinate in issue-linked threads:**
   ```
   send_message(..., thread_id="FEAT-123")
   fetch_inbox(project_key, agent_name)
   acknowledge_message(project_key, agent_name, message_id)
   ```
   Use the Bead ID as the thread ID when possible. Post a start message when claiming work, progress replies when plans or scope change materially, a completion message when done, and a handoff message when leaving unfinished or partially validated work.

4. **Acknowledge and maintain reservations:**
   Poll your inbox often enough to catch coordination changes, acknowledge messages that request acknowledgement, renew reservations if work is still active near TTL expiry, and release reservations promptly when the edit surface is no longer needed. If work is being handed off directly, transfer reservations explicitly in-thread by naming the recipient, reserved paths, and remaining TTL; the handoff is not complete until the receiving agent acknowledges takeover.

### Reservation Hygiene

- Reserve the smallest practical set of files or globs that covers the edit you are actually making; narrow existing reservations instead of broadening them reflexively.
- Use **exclusive** reservations when you expect to modify the surface; use **shared** reservations only for read-heavy collaboration or investigation that should not block active editors.
- Prefer adding another small reservation for a newly discovered file over grabbing an entire directory preemptively.
- If a reservation conflicts, first narrow your pattern, wait for expiry when the work is truly overlapping, or coordinate in-thread before escalating scope.
- Renew TTL only while the work is actively in progress; release reservations as soon as the protected surface is no longer being edited.
- If work is handed off mid-stream, prefer an explicit transfer message in the existing Agent Mail thread over silent reservation expiry; the receiving agent should acknowledge before the original holder treats the surface as handed off.
- Do not treat reservations as ownership of a whole feature area; they are short-lived collision-avoidance hints for specific edit surfaces.

5. **Quick reads:**
   ```
   resource://inbox/{Agent}?project=<abs-path>&limit=20
   resource://thread/{id}?project=<abs-path>&include_bodies=true
   ```

6. **Keep Agent Mail distinct from Beads:**
   Use Agent Mail for coordination, reservations, and handoff narrative; use Beads for task status, dependencies, and backlog truth.

### Macros vs Granular Tools

- **Prefer macros for speed:** `macro_start_session`, `macro_prepare_thread`, `macro_file_reservation_cycle`, `macro_contact_handshake`
- **Use granular tools for control:** `register_agent`, `file_reservation_paths`, `send_message`, `fetch_inbox`, `acknowledge_message`

### Common Pitfalls

- `"from_agent not registered"`: Always `register_agent` in the correct `project_key` first
- `"FILE_RESERVATION_CONFLICT"`: Adjust patterns, wait for expiry, or use non-exclusive reservation
- **Auth errors:** If JWT+JWKS enabled, include bearer token with matching `kid`

---

## Beads (br) — Dependency-Aware Issue Tracking

Beads provides a lightweight, dependency-aware issue database and CLI (`br` - beads_rust) for selecting "ready work," setting priorities, and tracking status. It complements MCP Agent Mail's messaging and file reservations.

**Important:** `br` is non-invasive—it NEVER runs git commands automatically. You must manually commit changes after `br sync --flush-only`.

**Workspace contract for this repo:** `.beads/` is already initialized here. Preserve the live workspace and the issue IDs already present in `.beads/issues.jsonl` instead of re-running init or changing prefixes to match generic examples or older defaults. Before assuming the workspace is missing or unhealthy, validate it with `br info` and `br doctor`.

### Conventions

- **Single source of truth:** Beads for task status/priority/dependencies; Agent Mail for conversation and audit
- **Existing workspace first:** Build on the current backlog state and preserve this repo’s existing issue-ID family (currently `mb-...`) rather than inventing a new prefix or reinitializing Beads
- **Shared identifiers:** Use the current Beads issue ID verbatim (e.g., `mb-1ga.5`) as Mail `thread_id` and prefix subjects with `[mb-1ga.5]`
- **Reservations:** When starting a task, call `file_reservation_paths()` with the issue ID in `reason`

### Typical Agent Flow

1. **Validate workspace health when needed:**
   ```bash
   br info
   br doctor
   ```

2. **Pick ready work (Beads):**
   ```bash
   br ready --json  # Choose highest priority, no blockers
   ```

3. **Reserve edit surface (Mail):**
   ```
   file_reservation_paths(project_key, agent_name, ["src/**"], ttl_seconds=3600, exclusive=true, reason="mb-1ga.5")
   ```

4. **Announce start (Mail):**
   ```
   send_message(..., thread_id="mb-1ga.5", subject="[mb-1ga.5] Start: <title>", ack_required=true)
   ```

5. **Work and update:** Reply in-thread with progress

6. **Complete and release:**
   ```bash
   br close mb-1ga.5 --reason "Completed"
   br sync --flush-only  # Export to JSONL (no git operations)
   ```
   ```
   release_file_reservations(project_key, agent_name, paths=["src/**"])
   ```
   Final Mail reply: `[mb-1ga.5] Completed` with summary

### Mapping Cheat Sheet

| Concept | Value |
|---------|-------|
| Mail `thread_id` | Current Beads issue ID (e.g., `mb-1ga.5`) |
| Mail subject | `[<issue-id>] ...` |
| File reservation `reason` | `<issue-id>` |
| Commit messages | Include `<issue-id>` for traceability |

---

## bv — Graph-Aware Triage Engine

bv is a graph-aware triage engine for Beads projects (`.beads/beads.jsonl`). It computes PageRank, betweenness, critical path, cycles, HITS, eigenvector, and k-core metrics deterministically.

**Scope boundary:** bv handles *what to work on* (triage, priority, planning). For agent-to-agent coordination (messaging, work claiming, file reservations), use MCP Agent Mail.

**CRITICAL: Use ONLY `--robot-*` flags. Bare `bv` launches an interactive TUI that blocks your session.**

### The Workflow: Start With Triage

**`bv --robot-triage` is your single entry point.** It returns:
- `quick_ref`: at-a-glance counts + top 3 picks
- `recommendations`: ranked actionable items with scores, reasons, unblock info
- `quick_wins`: low-effort high-impact items
- `blockers_to_clear`: items that unblock the most downstream work
- `project_health`: status/type/priority distributions, graph metrics
- `commands`: copy-paste shell commands for next steps

```bash
bv --robot-triage        # THE MEGA-COMMAND: start here
bv --robot-next          # Minimal: just the single top pick + claim command
```

### Command Reference

**Planning:**
| Command | Returns |
|---------|---------|
| `--robot-plan` | Parallel execution tracks with `unblocks` lists |
| `--robot-priority` | Priority misalignment detection with confidence |

**Graph Analysis:**
| Command | Returns |
|---------|---------|
| `--robot-insights` | Full metrics: PageRank, betweenness, HITS, eigenvector, critical path, cycles, k-core, articulation points, slack |
| `--robot-label-health` | Per-label health: `health_level`, `velocity_score`, `staleness`, `blocked_count` |
| `--robot-label-flow` | Cross-label dependency: `flow_matrix`, `dependencies`, `bottleneck_labels` |
| `--robot-label-attention [--attention-limit=N]` | Attention-ranked labels |

**History & Change Tracking:**
| Command | Returns |
|---------|---------|
| `--robot-history` | Bead-to-commit correlations |
| `--robot-diff --diff-since <ref>` | Changes since ref: new/closed/modified issues, cycles |

**Other:**
| Command | Returns |
|---------|---------|
| `--robot-burndown <sprint>` | Sprint burndown, scope changes, at-risk items |
| `--robot-forecast <id\|all>` | ETA predictions with dependency-aware scheduling |
| `--robot-alerts` | Stale issues, blocking cascades, priority mismatches |
| `--robot-suggest` | Hygiene: duplicates, missing deps, label suggestions |
| `--robot-graph [--graph-format=json\|dot\|mermaid]` | Dependency graph export |
| `--export-graph <file.html>` | Interactive HTML visualization |

### Scoping & Filtering

```bash
bv --robot-plan --label backend              # Scope to label's subgraph
bv --robot-insights --as-of HEAD~30          # Historical point-in-time
bv --recipe actionable --robot-plan          # Pre-filter: ready to work
bv --recipe high-impact --robot-triage       # Pre-filter: top PageRank
bv --robot-triage --robot-triage-by-track    # Group by parallel work streams
bv --robot-triage --robot-triage-by-label    # Group by domain
```

### Understanding Robot Output

**All robot JSON includes:**
- `data_hash` — Fingerprint of source beads.jsonl
- `status` — Per-metric state: `computed|approx|timeout|skipped` + elapsed ms
- `as_of` / `as_of_commit` — Present when using `--as-of`

**Two-phase analysis:**
- **Phase 1 (instant):** degree, topo sort, density
- **Phase 2 (async, 500ms timeout):** PageRank, betweenness, HITS, eigenvector, cycles

### jq Quick Reference

```bash
bv --robot-triage | jq '.quick_ref'                        # At-a-glance summary
bv --robot-triage | jq '.recommendations[0]'               # Top recommendation
bv --robot-plan | jq '.plan.summary.highest_impact'        # Best unblock target
bv --robot-insights | jq '.status'                         # Check metric readiness
bv --robot-insights | jq '.Cycles'                         # Circular deps (must fix!)
```

---
## Beads Workflow Integration

This project uses [beads_rust](https://github.com/Dicklesworthstone/beads_rust) (`br`) for issue tracking. Issues are stored in `.beads/` and tracked in git.

**Important:** `br` is non-invasive—it NEVER executes git commands. After `br sync --flush-only`, you must manually run `git add .beads/ && git commit`.

### Essential Commands

```bash
# View issues (launches TUI - avoid in automated sessions)
bv

# CLI commands for agents (use these instead)
br ready              # Show issues ready to work (no blockers)
br list --status=open # All open issues
br show <id>          # Full issue details with dependencies
br create --title="..." --type=task --priority=2
br update <id> --status=in_progress
br close <id> --reason "Completed"
br close <id1> <id2>  # Close multiple issues at once
br sync --flush-only  # Export to JSONL (NO git operations)
```

### Workflow Pattern

1. **Start**: Run `br ready` to find actionable work
2. **Claim**: Use `br update <id> --status=in_progress`
3. **Work**: Implement the task
4. **Complete**: Use `br close <id>`
5. **Sync**: Run `br sync --flush-only` then manually commit

### Key Concepts

- **Dependencies**: Issues can block other issues. `br ready` shows only unblocked work.
- **Priority**: P0=critical, P1=high, P2=medium, P3=low, P4=backlog (use numbers, not words)
- **Types**: task, bug, feature, epic, question, docs
- **Blocking**: `br dep add <issue> <depends-on>` to add dependencies

### Session Protocol

**Before ending any session, run this checklist:**

```bash
git status              # Check what changed
git add <files>         # Stage code changes
br sync --flush-only    # Export beads to JSONL
git add .beads/         # Stage beads changes
git commit -m "..."     # Commit everything together
git push                # Push to remote
```

### Best Practices

- Check `br ready` at session start to find available work
- Update status as you work (in_progress → closed)
- Create new issues with `br create` when you discover tasks
- Use descriptive titles and set appropriate priority/type
- Always `br sync --flush-only && git add .beads/` before ending session

<!-- end-bv-agent-instructions -->

## Contributor Golden Path and Troubleshooting

Use this as the safe default path for both humans and agents working in this repository.

### Golden Path

1. **Orient on the canon first** — read `docs/PLAN.md` for product contract questions, `docs/INDEX.md` and `docs/CONTRIBUTING.md` for doc hierarchy and review/evidence rules, and `AGENTS.md` for execution, coordination, and handoff workflow.
2. **Pick work from ready backlog** — use `bv --robot-triage` or `br ready --json` to choose ready work instead of guessing or starting from blocked items.
3. **Claim the bead explicitly** — move the selected Bead to `in_progress` before editing so the shared backlog reflects reality.
4. **Reserve only the edit surface you need** — use Agent Mail reservations for the smallest practical file or glob set, and keep Beads state separate from coordination narrative.
5. **Announce start in the bead thread** — send a start message with the Bead ID, planned surface, and any expected validation so other contributors can deconflict early.
6. **Implement with the repo’s workflow rules** — use `linehash` for shell-based targeted edits when appropriate, refresh stale anchors instead of forcing them, and keep changes bounded to the bead’s actual scope.
7. **Validate the changed surface** — run the quality gates that fit the surface you changed, including targeted checks and any required benchmark, lint, or bug-scan steps, and note anything intentionally deferred.
8. **Sync shared state before handoff** — update Bead status, run `br sync --flush-only`, release or transfer reservations, and send a completion or handoff message with what changed, what was validated, and what remains.

### Troubleshooting Guideposts

- **Stale `linehash` anchors:** re-run `linehash read <file>` and retry with the refreshed anchors instead of forcing an edit.
- **Doc conflicts or ambiguity:** trace the question back through `docs/PLAN.md`, then `docs/INDEX.md` / `docs/CONTRIBUTING.md`; if the canon is still ambiguous, pause that point and capture the conflict in the active bead instead of guessing.
- **Reservation conflicts:** narrow the reservation, wait if the overlap is real, or coordinate in-thread before broadening scope.
- **Unexpected concurrent diffs:** treat them as normal shared work, not cleanup targets; never stash, revert, overwrite, or delete them just because you did not author them.
- **Workflow uncertainty:** prefer `bv --robot-triage`, `br ready --json`, and the existing bead thread over ad-hoc assumptions about what to work on next.
- **Blocked validation or partial completion:** say so explicitly in the handoff, keep the bead state accurate, and create follow-up beads for newly discovered or deferred work rather than hiding TODOs in prose.

## Landing the Plane (Session Completion)

**When ending a work session**, you MUST complete ALL steps below.

**MANDATORY WORKFLOW:**

1. **File issues for remaining work** - Create follow-up beads for anything unfinished, newly discovered, or intentionally deferred instead of leaving implicit TODOs.
2. **Run quality gates** - Run the tests, linters, builds, benchmarks, or targeted checks appropriate to the changed surface, and note anything intentionally deferred.
3. **Update issue status** - Close finished work, keep partial work accurately open or in progress, and preserve blocker or dependency state needed for the next contributor.
4. **Sync beads** - `br sync --flush-only` to export JSONL after status updates.
5. **Hand off** - Leave enough context for the next contributor to resume safely without re-deriving what changed, what was validated, what remains, and which reservations, mail threads, or status transitions still matter.

**Session-close rules:**

- Unexpected worktree edits are normal concurrent work from other agents and are **not** cleanup targets.
- Never stash, revert, overwrite, delete, or otherwise disturb concurrent edits just because you did not author them.
- If the user explicitly asks for the built-in TODO functionality, comply; otherwise keep Beads as the canonical shared-work tracker.
- If validation is blocked or deferred, say so explicitly in the handoff instead of implying completion.

**Minimum handoff payload:**

- Bead IDs touched, with final status for each (`completed`, still `in_progress`, blocked, or intentionally deferred).
- Files, docs, commands, or surfaces changed so the next contributor knows where to look first.
- Validation already run, including what passed, what failed, and what was intentionally not run.
- Open risks, unresolved questions, or decisions that should not be resumed blindly.
- The next recommended step, including whether a follow-up bead already exists or still needs to be created.
- Any active reservations, relevant Agent Mail thread IDs, or coordination state that still matters, including whether reservations were released or explicitly transferred to another agent.

---

Note for Codex/GPT-5.2:

You constantly bother me and stop working with concerned questions that look similar to this:

```
Unexpected changes (need guidance)

- Working tree still shows edits I did not make in Cargo.toml, Cargo.lock, src/cli/commands/upgrade.rs, src/storage/sqlite.rs, tests/conformance.rs, tests/storage_deps.rs. Please advise whether to keep/commit/revert these before any further work. I did not touch them.

Next steps (pick one)

1. Decide how to handle the unrelated modified files above so we can resume cleanly.
2. Triage beads_rust-orko (clippy/cargo warnings) and beads_rust-ydqr (rustfmt failures).
3. If you want a full suite run later, fix conformance/clippy blockers and re‑run cargo test --all.
```

NEVER EVER DO THAT AGAIN. The answer is literally ALWAYS the same: those are changes created by the potentially dozen of other agents working on the project at the same time. This is not only a common occurence, it happens multiple times PER MINUTE. The way to deal with it is simple: you NEVER, under ANY CIRCUMSTANCE, stash, revert, overwrite, or otherwise disturb in ANY way the work of other agents. Just treat those changes identically to changes that you yourself made. Just fool yourself into thinking YOU made the changes and simply don't recall it for some reason.

## Legacy `bd` Workflow (Deprecated)

Historical docs may still mention `beads_viewer`/`bd` commands. For this repository, that workflow is deprecated.

Canonical issue workflow is:
- `br` for task state and dependency management
- `br sync --flush-only` for JSONL export (no git automation)
- `bv --robot-*` for triage/planning (never bare `bv`)

Do not run `bd`/`bd sync` for normal work. Only use legacy command names when reading old artifacts or translating historical instructions.

Quick translation from legacy docs:

| Legacy | Canonical |
|--------|-----------|
| `bd ready` | `br ready` |
| `bd list --status=open` | `br list --status open` |
| `bd show <id>` | `br show <id>` |
| `bd update <id> --status=in_progress` | `br update <id> --status in_progress` |
| `bd close <id>` | `br close <id> --reason "Completed"` |
| `bd sync` | `br sync --flush-only` + manual git add/commit/push |

---

## UBS Quick Reference for AI Agents

UBS stands for "Ultimate Bug Scanner": **The AI Coding Agent's Secret Weapon: Flagging Likely Bugs for Fixing Early On**

**Install:** `curl -sSL https://raw.githubusercontent.com/Dicklesworthstone/ultimate_bug_scanner/master/install.sh | bash`

**Golden Rule:** `ubs <changed-files>` before every commit. Exit 0 = safe. Exit >0 = fix & re-run.
**Workflow Contract:**
- Prefer explicit changed files or staged files, not whole-repo scans, unless the change truly spans the whole project
- Treat `ubs` as a required bug-finding pass for the surface you changed, not as optional cleanup
- Investigate every finding, fix the real issue when valid, and re-run `ubs` on the changed surface until it passes or you explicitly document a justified deferral in the handoff

**Commands:**
```bash
ubs file.ts file2.py                    # Specific files (< 1s) — USE THIS
ubs $(git diff --name-only --cached)    # Staged files — before commit
ubs --only=js,python src/               # Language filter (3-5x faster)
ubs --ci --fail-on-warning .            # CI mode — before PR
ubs --help                              # Full command reference
ubs sessions --entries 1                # Tail the latest install session log
ubs .                                   # Whole project (ignores things like .venv and node_modules automatically)
```

**Output Format:**
```
⚠️  Category (N errors)
    file.ts:42:5 – Issue description
    💡 Suggested fix
Exit code: 1
```
Parse: `file:line:col` → location | 💡 → how to fix | Exit 0/1 → pass/fail

**Fix Workflow:**
1. Read finding → category + fix suggestion
2. Navigate `file:line:col` → view context
3. Verify real issue (not false positive)
4. Fix root cause (not symptom)
5. Re-run `ubs <file>` → exit 0
6. Commit

**Speed Critical:** Scope to changed files. `ubs src/file.ts` (< 1s) vs `ubs .` (30s). Never full scan for small edits.

**Bug Severity:**
- **Critical** (always fix): Null safety, XSS/injection, async/await, memory leaks
- **Important** (production): Type narrowing, division-by-zero, resource leaks
- **Contextual** (judgment): TODO/FIXME, console logs

**Anti-Patterns:**
- ❌ Ignore findings → ✅ Investigate each
- ❌ Full scan per edit → ✅ Scope to file
- ❌ Fix symptom (`if (x) { x.y }`) → ✅ Root cause (`x?.y`)

---

## cass — Cross-Agent Session Search

`cass` indexes prior agent conversations (Claude Code, Codex, Cursor, Gemini, ChatGPT, etc.) so we can reuse solved problems.

**Rules:** Never run bare `cass` (TUI). Always use `--robot` or `--json`.

**Workflow Contract:**
- Use `cass` before re-investigating a bug, failure mode, design question, or workflow problem that may already have been solved in earlier sessions
- Prefer narrow, automation-safe queries first (`search`, `view`, `expand`, `capabilities`, `robot-docs`) and keep output lean with flags like `--fields minimal`, `--agent`, and `--days N`
- Treat `cass` as a reuse and orientation tool: extract prior solutions or context, then verify them against the current repository state before acting
- Never use interactive cass modes in agent workflows

### Examples

```bash
cass health
cass search "async runtime" --robot --limit 5
cass view /path/to/session.jsonl -n 42 --json
cass expand /path/to/session.jsonl -n 42 -C 3 --json
cass capabilities --json
cass robot-docs guide
```

### Tips

- Use `--fields minimal` for lean output
- Filter by agent with `--agent`
- Use `--days N` to limit to recent history

stdout is data-only, stderr is diagnostics; exit code 0 means success.

Treat cass as a way to avoid re-solving problems other agents already handled.

---

Note for Codex/GPT-5.2:

You constantly bother me and stop working with concerned questions that look similar to this:

```
Unexpected changes (need guidance)

- Working tree still shows edits I did not make in Cargo.toml, Cargo.lock, src/cli/commands/upgrade.rs, src/storage/sqlite.rs, tests/conformance.rs, tests/storage_deps.rs. Please advise whether to keep/commit/revert these before any further work. I did not touch them.

Next steps (pick one)

1. Decide how to handle the unrelated modified files above so we can resume cleanly.
2. Triage beads_rust-orko (clippy/cargo warnings) and beads_rust-ydqr (rustfmt failures).
3. If you want a full suite run later, fix conformance/clippy blockers and re-run cargo test --all.
```

NEVER EVER DO THAT AGAIN. The answer is literally ALWAYS the same: those are changes created by the potentially dozen of other agents working on the project at the same time. This is not only a common occurrence, it happens multiple times PER MINUTE. The way to deal with it is simple: you NEVER, under ANY CIRCUMSTANCE, stash, revert, overwrite, or otherwise disturb in ANY way the work of other agents. Just treat those changes identically to changes that you yourself made. Just fool yourself into thinking YOU made the changes and simply don't recall it for some reason.

---



## Note on Built-in TODO Functionality

Also, if I ask you to explicitly use your built-in TODO functionality, don't complain about this and say you need to use beads. You can use built-in TODOs if I tell you specifically to do so. Always comply with such orders.
