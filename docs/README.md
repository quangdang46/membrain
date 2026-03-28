# Membrain Docs

Quick pointers:

- [PLAN.md](PLAN.md) — canonical design contract
- [CLI.md](CLI.md) — CLI commands and connection modes
- [MCP_API.md](MCP_API.md) — MCP and daemon transport contract

## Easy start

### Claude Code integration

Membrain supports Claude Code through the local stdio MCP entrypoint `membrain mcp`.

Recommended setup from the Claude Code CLI:

```bash
claude mcp add --transport stdio membrain -- membrain mcp
```

If you want Membrain to use a custom local state root:

```bash
claude mcp add --transport stdio membrain -- membrain mcp --db-path /path/to/state-root
```

If you want the config checked into the repo for the team, use project scope:

```bash
claude mcp add --transport stdio --scope project membrain -- membrain mcp
```

That creates or updates a project-root `.mcp.json` like this:

```json
{
  "mcpServers": {
    "membrain": {
      "command": "membrain",
      "args": ["mcp"]
    }
  }
}
```

Claude Code prompts for approval before using project-scoped servers from `.mcp.json`.

`membrain mcp` uses stdio, so it does **not** listen on a TCP port or Unix socket. Claude Code launches it directly as a subprocess and talks over stdin/stdout.

Current bounded MCP truth:
- `tools/list` advertises six callable tools today: `encode`, `recall`, `inspect`, `why`, `health`, and `doctor`
- slash-style MCP protocol methods such as `initialize`, `tools/list`, `tools/call`, `resources/list`, `resources/read`, `prompts/list`, and `prompts/get` are recognized on the stdio path
- `prompts/list` / `prompts/get` are intentionally placeholder surfaces for now: empty prompt list and `unknown prompt` on named prompt lookup
- the stdio adapter also accepts direct JSON-RPC compatibility methods like `encode`, `recall`, `inspect`, `why`, `health`, `doctor`, and `shutdown`
- long-lived warm-runtime guarantees still belong to `membrain daemon`, not the stdio adapter

For Claude Code MCP details, scopes, and hooks, see the official docs:
- https://code.claude.com/docs/en/mcp
- https://code.claude.com/docs/en/hooks

This repo also ships a project-local Claude hook sink:
- [`.claude/settings.json`](/home/quangdang/projects/tools/membrain/.claude/settings.json)
- [`.claude/hooks/membrain_hook.py`](/home/quangdang/projects/tools/membrain/.claude/hooks/membrain_hook.py)

Current hook posture in this repo:
- configured Claude hook events are persisted into Membrain through `membrain remember`
- the helper is fail-open: if hook parsing or Membrain invocation fails, Claude should keep running
- obvious secret-bearing keys such as tokens, passwords, and authorization headers are redacted before summaries are stored
- the hook stores bounded event summaries, not raw full transcripts or daemon auto-start state

### Codex integration

Codex also supports Membrain through the same stdio MCP entrypoint.

Recommended setup from the Codex CLI:

```bash
codex mcp add membrain -- membrain mcp
```

If you want Membrain to use a custom local state root:

```bash
codex mcp add membrain -- membrain mcp --db-path /path/to/state-root
```

Equivalent Codex config in `~/.codex/config.toml`:

```toml
[mcp_servers.membrain]
command = "membrain"
args = ["mcp"]
```

Codex shares MCP configuration between the CLI and the IDE extension, so you only need to set it up once.

Current Codex note:
- the documented Codex integration path here is MCP configuration
- this repo does not claim a Claude-style Codex hook event system today

### MCP client / subprocess mode

Use this when you want a client to launch Membrain directly without manually connecting to a socket:

```bash
membrain mcp
```

Recommended client setup commands:

```bash
claude mcp add --transport stdio membrain -- membrain mcp
codex mcp add membrain -- membrain mcp
```

### Background local service mode

Use this when you want a long-lived local daemon:

```bash
membrain daemon
# or
membrain-daemon
```

Default daemon socket:

```bash
~/.membrain/membrain.sock
```

Important:
- installing `membrain` does **not** auto-register the MCP server with Claude Code or Codex
- installing `membrain` does **not** auto-start `membrain-daemon`
- `membrain mcp` is enough for Claude Code and Codex MCP integration
- only `membrain daemon` / `membrain-daemon` provides the authoritative warm background runtime
