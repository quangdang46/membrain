# Membrain Docs

Quick pointers:

- [PLAN.md](PLAN.md) — canonical design contract
- [CLI.md](CLI.md) — CLI commands and connection modes
- [MCP_API.md](MCP_API.md) — MCP and daemon transport contract

## Easy start

### Claude Code integration

Membrain now supports an easy Claude Code subprocess integration path through `membrain mcp`.

To wire it into Claude Code, add this under `mcpServers` in your Claude Code MCP configuration:

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

If you want Membrain to use a custom local state root:

```json
{
  "mcpServers": {
    "membrain": {
      "command": "membrain",
      "args": ["mcp", "--db-path", "/path/to/state-root"]
    }
  }
}
```

`membrain mcp` uses stdio, so it does **not** listen on a TCP port or Unix socket. Claude Code launches it directly as a subprocess and talks over stdin/stdout.

For Claude Code hooks guidance, see the official hooks docs:
- https://code.claude.com/docs/en/hooks

A practical project-level `.claude/settings.json` example that enables Membrain MCP plus startup/prompt reminders looks like this:

```json
{
  "mcpServers": {
    "membrain": {
      "command": "membrain",
      "args": ["mcp"]
    }
  },
  "hooks": {
    "SessionStart": [
      {
        "matcher": "startup|resume",
        "hooks": [
          {
            "type": "command",
            "command": "bash -lc 'pgrep -f \"membrain-daemon\" >/dev/null || nohup membrain-daemon >/tmp/membrain-daemon.log 2>&1 &'"
          },
          {
            "type": "command",
            "command": "echo 'Membrain is available in this project. Prefer using Membrain MCP or CLI recall/inspect/why before guessing prior context. Local state lives under ~/.membrain by default.'"
          }
        ]
      }
    ],
    "UserPromptSubmit": [
      {
        "hooks": [
          {
            "type": "command",
            "command": "echo 'Membrain reminder: use memory tools for prior context, incidents, and reusable facts. Prefer `membrain recall`, `membrain inspect`, `membrain why`, or the Membrain MCP server when context may already exist.'"
          }
        ]
      }
    ]
  }
}
```

### MCP client / subprocess mode

Use this when you want a client to launch Membrain directly without manually connecting to a socket:

```bash
membrain mcp
```

For Claude Code, add Membrain in `mcpServers` like this:

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
