#!/usr/bin/env bash
set -euo pipefail
umask 022

BINARY_NAME="membrain"
DAEMON_BINARY_NAME="membrain-daemon"
OWNER="quangdang46"
REPO="membrain"
DEST="${DEST:-$HOME/.local/bin}"
VERSION="${VERSION:-}"
DB_PATH="${DB_PATH:-}"
QUIET=0
EASY=0
VERIFY=0
FROM_SOURCE=0
UNINSTALL=0
MAX_RETRIES=3
DOWNLOAD_TIMEOUT=120
LOCK_DIR="/tmp/${BINARY_NAME}-install.lock.d"
TMP=""
DATA_HOME="${XDG_DATA_HOME:-$HOME/.local/share}"
STATE_HOME="${XDG_STATE_HOME:-$HOME/.local/state}"
SUPPORT_DIR="$DATA_HOME/$BINARY_NAME"
HOOKS_DIR="$SUPPORT_DIR/hooks"
HOOK_SCRIPT_PATH="$HOOKS_DIR/membrain_agent_hook.py"
DAEMON_RUNNER_PATH="$SUPPORT_DIR/${DAEMON_BINARY_NAME}-run.sh"
DAEMON_LOG_DIR="$STATE_HOME/$BINARY_NAME"
DAEMON_LOG_PATH="$DAEMON_LOG_DIR/${DAEMON_BINARY_NAME}.log"
CLAUDE_SETTINGS_PATH="$HOME/.claude/settings.json"
CODEX_CONFIG_PATH="$HOME/.codex/config.toml"
CODEX_HOOKS_PATH="$HOME/.codex/hooks.json"

log_info() { [ "$QUIET" -eq 1 ] && return; echo "[${BINARY_NAME}] $*" >&2; }
log_warn() { echo "[${BINARY_NAME}] WARN: $*" >&2; }
log_success() { [ "$QUIET" -eq 1 ] && return; echo "✓ $*" >&2; }
die() { echo "ERROR: $*" >&2; exit 1; }

usage() {
  cat <<'EOF'
Usage: install.sh [OPTIONS]

Options:
  --dest <dir>              Install binaries into this directory
  --dest=<dir>
  --version <tag>           Install a specific release tag such as v0.1.0
  --version=<tag>
  --db-path <path>          Configure MCP registration / daemon service to use a custom Membrain state root
  --db-path=<path>
  --system                  Install into /usr/local/bin
  --easy-mode               Add install dir to PATH in writable shell rc files
  --verify                  Run post-install verification commands
  --from-source             Build from source instead of downloading a release archive
  --uninstall               Remove installed binaries and local service artifacts
  --quiet, -q              Reduce installer output
  -h, --help               Show this help
EOF
  exit 0
}

cleanup() { rm -rf "$TMP" "$LOCK_DIR" 2>/dev/null || true; }
trap cleanup EXIT

acquire_lock() {
  mkdir "$LOCK_DIR" 2>/dev/null || die "Another install is running. If stuck: rm -rf $LOCK_DIR"
  echo $$ > "$LOCK_DIR/pid"
}

while [ $# -gt 0 ]; do
  case "$1" in
    --dest) DEST="$2"; shift 2 ;;
    --dest=*) DEST="${1#*=}"; shift ;;
    --version) VERSION="$2"; shift 2 ;;
    --version=*) VERSION="${1#*=}"; shift ;;
    --db-path) DB_PATH="$2"; shift 2 ;;
    --db-path=*) DB_PATH="${1#*=}"; shift ;;
    --system) DEST="/usr/local/bin"; shift ;;
    --easy-mode) EASY=1; shift ;;
    --verify) VERIFY=1; shift ;;
    --from-source) FROM_SOURCE=1; shift ;;
    --quiet|-q) QUIET=1; shift ;;
    --uninstall) UNINSTALL=1; shift ;;
    -h|--help) usage ;;
    *) die "Unknown option: $1" ;;
  esac
done

detect_platform() {
  local os arch
  case "$(uname -s)" in
    Linux*) os="linux" ;;
    Darwin*) os="macos" ;;
    MINGW*|MSYS*|CYGWIN*) os="windows" ;;
    *) die "Unsupported OS: $(uname -s)" ;;
  esac
  case "$(uname -m)" in
    x86_64|amd64) arch="x86_64" ;;
    aarch64|arm64) arch="aarch64" ;;
    *) die "Unsupported arch: $(uname -m)" ;;
  esac
  echo "${os}_${arch}"
}

resolve_version() {
  [ -n "$VERSION" ] && return 0
  VERSION=$(
    curl -fsSL \
      --connect-timeout 10 \
      --max-time 30 \
      -H "Accept: application/vnd.github.v3+json" \
      "https://api.github.com/repos/${OWNER}/${REPO}/releases/latest" \
      2>/dev/null | grep '"tag_name":' | sed -E 's/.*"([^"]+)".*/\1/'
  ) || true
  if [ -z "$VERSION" ]; then
    VERSION=$(
      curl -fsSL -o /dev/null -w '%{url_effective}' \
        "https://github.com/${OWNER}/${REPO}/releases/latest" \
        2>/dev/null | sed -E 's|.*/tag/||'
    ) || true
  fi
  [[ "$VERSION" =~ ^v[0-9] ]] || die "Could not resolve version"
}

download_file() {
  local url="$1" dest="$2" partial="${2}.part" attempt=0
  while [ $attempt -lt $MAX_RETRIES ]; do
    attempt=$((attempt + 1))
    curl -fL \
      --connect-timeout 30 \
      --max-time "$DOWNLOAD_TIMEOUT" \
      --retry 2 \
      $( [ -s "$partial" ] && echo "--continue-at -" ) \
      $( [ "$QUIET" -eq 0 ] && [ -t 2 ] && echo "--progress-bar" || echo "-sS" ) \
      -o "$partial" "$url" && mv -f "$partial" "$dest" && return 0
    [ $attempt -lt $MAX_RETRIES ] && { log_warn "Retrying in 3s..."; sleep 3; }
  done
  return 1
}

sha256_file() {
  if command -v sha256sum >/dev/null 2>&1; then
    sha256sum "$1" | awk '{print $1}'
  else
    shasum -a 256 "$1" | awk '{print $1}'
  fi
}

install_binary_atomic() {
  local src="$1" dest="$2" tmp="${dest}.tmp.$$"
  install -m 0755 "$src" "$tmp"
  mv -f "$tmp" "$dest" || { rm -f "$tmp"; die "Failed to install binary: $dest"; }
}

maybe_add_path() {
  case ":$PATH:" in
    *":$DEST:"*) return 0 ;;
  esac
  if [ "$EASY" -eq 1 ]; then
    for rc in "$HOME/.zshrc" "$HOME/.bashrc"; do
      [ -f "$rc" ] && [ -w "$rc" ] || continue
      grep -qF "$DEST" "$rc" && continue
      printf '\nexport PATH="%s:$PATH"  # %s installer\n' "$DEST" "$BINARY_NAME" >> "$rc"
    done
    log_warn "PATH updated. Restart shell or run: export PATH=\"$DEST:\$PATH\""
  else
    log_warn "Add to PATH: export PATH=\"$DEST:\$PATH\""
  fi
}

release_archive_name() {
  local platform="$1"
  case "$platform" in
    linux_x86_64) echo "membrain-${VERSION}-linux-x86_64.tar.gz" ;;
    linux_aarch64) echo "membrain-${VERSION}-linux-aarch64.tar.gz" ;;
    macos_x86_64) echo "membrain-${VERSION}-macos-x86_64.tar.gz" ;;
    macos_aarch64) echo "membrain-${VERSION}-macos-aarch64.tar.gz" ;;
    windows_x86_64) echo "membrain-${VERSION}-windows-x86_64.zip" ;;
    *) die "Unsupported packaged platform: $platform" ;;
  esac
}

build_from_source() {
  command -v cargo >/dev/null || die "Rust/cargo not found. Install Rust from https://rustup.rs"
  command -v git >/dev/null || die "git not found"
  git clone --depth 1 "https://github.com/${OWNER}/${REPO}.git" "$TMP/src"
  (
    cd "$TMP/src"
    CARGO_TARGET_DIR="$TMP/target" cargo build --release --bin membrain --bin membrain-daemon
  )
  install_binary_atomic "$TMP/target/release/$BINARY_NAME" "$DEST/$BINARY_NAME"
  install_binary_atomic "$TMP/target/release/$DAEMON_BINARY_NAME" "$DEST/$DAEMON_BINARY_NAME"
}

install_from_archive() {
  local platform="$1" archive
  archive="$(release_archive_name "$platform")"
  local url="https://github.com/${OWNER}/${REPO}/releases/download/${VERSION}/${archive}"

  download_file "$url" "$TMP/$archive" || return 1
  if download_file "${url}.sha256" "$TMP/checksum.sha256" 2>/dev/null; then
    local expected actual
    expected="$(awk '{print $1}' "$TMP/checksum.sha256")"
    actual="$(sha256_file "$TMP/$archive")"
    [ "$expected" = "$actual" ] || die "Checksum mismatch for $archive"
    log_info "Checksum verified for $archive"
  fi

  case "$archive" in
    *.tar.gz) tar -xzf "$TMP/$archive" -C "$TMP" ;;
    *.zip)
      command -v unzip >/dev/null || die "unzip not found"
      unzip -q "$TMP/$archive" -d "$TMP"
      ;;
  esac

  local main_bin daemon_bin
  main_bin="$(find "$TMP" -name "$BINARY_NAME" -type f -perm -111 2>/dev/null | head -1)"
  daemon_bin="$(find "$TMP" -name "$DAEMON_BINARY_NAME" -type f -perm -111 2>/dev/null | head -1)"
  [ -n "$main_bin" ] || die "Binary not found in archive: $BINARY_NAME"
  [ -n "$daemon_bin" ] || die "Binary not found in archive: $DAEMON_BINARY_NAME"
  install_binary_atomic "$main_bin" "$DEST/$BINARY_NAME"
  install_binary_atomic "$daemon_bin" "$DEST/$DAEMON_BINARY_NAME"
}

ensure_python3() {
  command -v python3 >/dev/null 2>&1 || { log_warn "python3 not found; skipping hook/config merge helpers"; return 1; }
}

write_hook_script() {
  ensure_python3 || return 1
  mkdir -p "$HOOKS_DIR"
  cat > "$HOOK_SCRIPT_PATH" <<'PY'
#!/usr/bin/env python3
"""Persist Claude Code and Codex hook events into Membrain without breaking the session."""

from __future__ import annotations

import argparse
import json
import os
import subprocess
import sys
from typing import Any


SENSITIVE_KEY_PARTS = (
    "token",
    "secret",
    "password",
    "passwd",
    "authorization",
    "api_key",
    "apikey",
    "cookie",
    "session_token",
    "bearer",
    "private_key",
)

MAX_VALUE_LEN = 240
MAX_CONTENT_LEN = 900


def truncate(value: str, limit: int = MAX_VALUE_LEN) -> str:
    compact = " ".join(value.split())
    if len(compact) <= limit:
        return compact
    return compact[: limit - 3] + "..."


def is_sensitive_key(key: str) -> bool:
    lowered = key.lower()
    return any(part in lowered for part in SENSITIVE_KEY_PARTS)


def sanitize(value: Any, depth: int = 0) -> Any:
    if depth > 3:
        return "<depth-limited>"
    if isinstance(value, dict):
        sanitized: dict[str, Any] = {}
        for key, nested in value.items():
            sanitized[key] = "<redacted>" if is_sensitive_key(key) else sanitize(nested, depth + 1)
        return sanitized
    if isinstance(value, list):
        return [sanitize(item, depth + 1) for item in value[:8]]
    if isinstance(value, str):
        return truncate(value)
    return value


def payload_text(payload: dict[str, Any], *keys: str) -> str | None:
    for key in keys:
        value = payload.get(key)
        if isinstance(value, str) and value.strip():
            return truncate(value)
    return None


def detect_event(provider: str, payload: dict[str, Any], cli_event: str | None) -> str:
    if cli_event:
        return cli_event
    if provider == "codex":
        return payload_text(payload, "hook_event_name", "hookEventName") or "Unknown"
    return "Unknown"


def summarize_payload(provider: str, event: str, payload: dict[str, Any]) -> tuple[str, str]:
    session_id = payload_text(payload, "session_id", "sessionId")
    transcript_path = payload_text(payload, "transcript_path", "transcriptPath")
    cwd = payload_text(payload, "cwd")
    tool_name = payload_text(payload, "tool_name", "toolName")
    reason = payload_text(payload, "reason", "stop_hook_active", "stopHookActive")
    matcher = payload_text(payload, "matcher")
    prompt = payload_text(payload, "prompt", "user_prompt", "userPrompt")
    notification = payload_text(payload, "message", "notification")
    provider_tag = f"{provider}_hook"

    sanitized_input = sanitize(payload.get("tool_input"))
    sanitized_response = sanitize(payload.get("tool_response"))
    sanitized_payload = sanitize(payload)

    parts = [f"{provider_tag} event={event}"]
    context_parts = [f"{provider_tag} event={event}"]

    if tool_name:
        parts.append(f"tool={tool_name}")
        context_parts.append(f"tool={tool_name}")
    if reason:
        parts.append(f"reason={reason}")
        context_parts.append(f"reason={reason}")
    if matcher:
        parts.append(f"matcher={matcher}")
    if session_id:
        parts.append(f"session_id={session_id}")
    if cwd:
        parts.append(f"cwd={cwd}")
        context_parts.append(f"cwd={cwd}")
    if transcript_path:
        parts.append(f"transcript_path={transcript_path}")

    if prompt:
        parts.append(f'prompt="{prompt}"')
    elif notification:
        parts.append(f'notification="{notification}"')

    if sanitized_input not in (None, {}, []):
        parts.append("tool_input=" + truncate(json.dumps(sanitized_input, sort_keys=True), 280))
    if sanitized_response not in (None, {}, []):
        parts.append("tool_response=" + truncate(json.dumps(sanitized_response, sort_keys=True), 280))

    if len(parts) <= 2:
        parts.append("payload=" + truncate(json.dumps(sanitized_payload, sort_keys=True), 320))

    content = truncate("; ".join(parts), MAX_CONTENT_LEN)
    context = truncate(" ".join(context_parts), 240)
    return content, context


def attention_for_event(event: str) -> str:
    return {
        "UserPromptSubmit": "0.95",
        "PostToolUseFailure": "0.85",
        "StopFailure": "0.85",
        "PermissionRequest": "0.75",
        "PostToolUse": "0.55",
        "PreToolUse": "0.35",
        "SessionStart": "0.4",
        "SessionEnd": "0.45",
        "Stop": "0.45",
        "SubagentStop": "0.5",
        "TaskCompleted": "0.6",
    }.get(event, "0.45")


def remember_event(
    membrain_bin: str,
    namespace: str,
    db_path: str | None,
    provider: str,
    event: str,
    content: str,
    context: str,
) -> None:
    command = [
        membrain_bin,
        "remember",
        "--namespace",
        namespace,
        "--source",
        f"{provider}_hook",
        "--attention",
        attention_for_event(event),
        "--context",
        context,
        "--quiet",
        content,
    ]
    if db_path:
        command[2:2] = ["--db-path", db_path]
    subprocess.run(
        command,
        check=False,
        stdout=subprocess.DEVNULL,
        stderr=subprocess.DEVNULL,
        timeout=20,
    )


def maybe_emit_hook_output(provider: str, event: str) -> None:
    # Codex Stop hooks expect JSON stdout on success.
    if provider == "codex" and event == "Stop":
        print("{}")


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--provider", required=True, choices=("claude", "codex"))
    parser.add_argument("--event")
    parser.add_argument("--namespace", default=os.environ.get("MEMBRAIN_NAMESPACE", "default"))
    parser.add_argument("--db-path", default=os.environ.get("MEMBRAIN_DB_PATH"))
    parser.add_argument("--membrain-bin", default=os.environ.get("MEMBRAIN_BIN", "membrain"))
    args = parser.parse_args()

    try:
        raw = sys.stdin.read()
        payload = json.loads(raw) if raw.strip() else {}
        if not isinstance(payload, dict):
            payload = {"payload": sanitize(payload)}
    except Exception:
        payload = {}

    event = detect_event(args.provider, payload, args.event)

    try:
        content, context = summarize_payload(args.provider, event, payload)
        remember_event(
            membrain_bin=args.membrain_bin,
            namespace=args.namespace,
            db_path=args.db_path,
            provider=args.provider,
            event=event,
            content=content,
            context=context,
        )
        maybe_emit_hook_output(args.provider, event)
    except Exception:
        if args.provider == "codex" and event == "Stop":
            print("{}")
        return 0

    return 0


if __name__ == "__main__":
    raise SystemExit(main())
PY
  chmod 0755 "$HOOK_SCRIPT_PATH"
}

write_daemon_runner() {
  mkdir -p "$SUPPORT_DIR" "$DAEMON_LOG_DIR"
  cat > "$DAEMON_RUNNER_PATH" <<EOF
#!/usr/bin/env bash
set -euo pipefail
mkdir -p "$(dirname "$DAEMON_LOG_PATH")"
exec "$DEST/$DAEMON_BINARY_NAME"$( [ -n "$DB_PATH" ] && printf ' --db-path %q' "$DB_PATH" ) >>"$DAEMON_LOG_PATH" 2>&1
EOF
  chmod 0755 "$DAEMON_RUNNER_PATH"
}

claude_integration_install() {
  ensure_python3 || return 0
  write_hook_script || return 0
  mkdir -p "$(dirname "$CLAUDE_SETTINGS_PATH")"
  python3 - "$CLAUDE_SETTINGS_PATH" "$DEST/$BINARY_NAME" "$HOOK_SCRIPT_PATH" "$DB_PATH" <<'PY'
import json
import pathlib
import sys

settings_path = pathlib.Path(sys.argv[1]).expanduser()
binary_path = sys.argv[2]
hook_script = sys.argv[3]
db_path = sys.argv[4]

if settings_path.exists():
    try:
        data = json.loads(settings_path.read_text(encoding="utf-8"))
    except Exception:
        data = {}
else:
    data = {}

if not isinstance(data, dict):
    data = {}

args = ["mcp"]
if db_path:
    args += ["--db-path", db_path]

data.setdefault("mcpServers", {})
data["mcpServers"]["membrain"] = {
    "command": binary_path,
    "args": args,
}

hook_events = {
    "SessionStart": "startup|resume|clear|compact",
    "UserPromptSubmit": None,
    "PreToolUse": "Read|Grep|Glob|WebFetch|WebSearch|Bash|Edit|Write|MultiEdit|Task",
    "PostToolUse": "Read|Grep|Glob|Edit|Write|MultiEdit|Bash|Task|WebFetch|WebSearch",
    "PostToolUseFailure": "Bash|Read|Edit|Write|MultiEdit|Task|WebFetch|WebSearch|Grep|Glob|mcp__.*",
    "Stop": None,
    "SubagentStart": None,
    "SubagentStop": None,
    "TaskCompleted": None,
    "TeammateIdle": None,
    "InstructionsLoaded": "session_start|nested_traversal|path_glob_match|include|compact",
    "ConfigChange": None,
    "CwdChanged": None,
    "FileChanged": "CLAUDE.md|AGENTS.md|README.md|settings.json|settings.local.json",
    "PreCompact": None,
    "PostCompact": None,
    "SessionEnd": None,
    "Notification": None,
    "PermissionRequest": "Bash|Write|Edit|MultiEdit|Task|mcp__.*",
    "StopFailure": None,
    "Elicitation": None,
    "ElicitationResult": None,
    "WorktreeCreate": None,
    "WorktreeRemove": None,
}

hooks = data.setdefault("hooks", {})

def membrain_group(event_name: str, matcher: str | None):
    command = f'python3 "{hook_script}" --provider claude --event {event_name} --membrain-bin "{binary_path}"'
    if db_path:
        command += f' --db-path "{db_path}"'
    group = {
        "hooks": [
            {
                "type": "command",
                "command": command,
            }
        ]
    }
    if matcher:
        group["matcher"] = matcher
    return group

for event_name, matcher in hook_events.items():
    groups = hooks.get(event_name, [])
    if not isinstance(groups, list):
        groups = []
    cleaned = []
    for group in groups:
        if not isinstance(group, dict):
            cleaned.append(group)
            continue
        hook_defs = group.get("hooks")
        if isinstance(hook_defs, list) and any(
            isinstance(hook, dict)
            and hook.get("type") == "command"
            and hook_script in str(hook.get("command", ""))
            and "--provider claude" in str(hook.get("command", ""))
            for hook in hook_defs
        ):
            continue
        cleaned.append(group)
    cleaned.append(membrain_group(event_name, matcher))
    hooks[event_name] = cleaned

settings_path.write_text(json.dumps(data, indent=2) + "\n", encoding="utf-8")
PY
}

codex_hooks_install() {
  ensure_python3 || return 0
  write_hook_script || return 0
  mkdir -p "$(dirname "$CODEX_HOOKS_PATH")"
  python3 - "$CODEX_HOOKS_PATH" "$DEST/$BINARY_NAME" "$HOOK_SCRIPT_PATH" "$DB_PATH" <<'PY'
import json
import pathlib
import sys

hooks_path = pathlib.Path(sys.argv[1]).expanduser()
binary_path = sys.argv[2]
hook_script = sys.argv[3]
db_path = sys.argv[4]

if hooks_path.exists():
    try:
        data = json.loads(hooks_path.read_text(encoding="utf-8"))
    except Exception:
        data = {}
else:
    data = {}

if not isinstance(data, dict):
    data = {}

hook_events = {
    "SessionStart": "startup|resume",
    "UserPromptSubmit": None,
    "PreToolUse": "Bash",
    "PostToolUse": "Bash",
    "Stop": None,
}

hooks = data.setdefault("hooks", {})

def membrain_group(event_name: str, matcher: str | None):
    command = f'python3 "{hook_script}" --provider codex --event {event_name} --membrain-bin "{binary_path}"'
    if db_path:
        command += f' --db-path "{db_path}"'
    group = {
        "hooks": [
            {
                "type": "command",
                "command": command,
            }
        ]
    }
    if matcher:
        group["matcher"] = matcher
    return group

for event_name, matcher in hook_events.items():
    groups = hooks.get(event_name, [])
    if not isinstance(groups, list):
        groups = []
    cleaned = []
    for group in groups:
        if not isinstance(group, dict):
            cleaned.append(group)
            continue
        hook_defs = group.get("hooks")
        if isinstance(hook_defs, list) and any(
            isinstance(hook, dict)
            and hook.get("type") == "command"
            and hook_script in str(hook.get("command", ""))
            and "--provider codex" in str(hook.get("command", ""))
            for hook in hook_defs
        ):
            continue
        cleaned.append(group)
    cleaned.append(membrain_group(event_name, matcher))
    hooks[event_name] = cleaned

hooks_path.write_text(json.dumps(data, indent=2) + "\n", encoding="utf-8")
PY
}

codex_config_install() {
  ensure_python3 || return 0
  mkdir -p "$(dirname "$CODEX_CONFIG_PATH")"
  python3 - "$CODEX_CONFIG_PATH" "$DEST/$BINARY_NAME" "$DB_PATH" <<'PY'
import json
import pathlib
import re
import sys

config_path = pathlib.Path(sys.argv[1]).expanduser()
binary_path = sys.argv[2]
db_path = sys.argv[3]

text = config_path.read_text(encoding="utf-8") if config_path.exists() else ""

def strip_table(text: str, table: str) -> str:
    pattern = re.compile(
        rf'(?ms)^\[{re.escape(table)}\]\n(?:^(?!\[).*(?:\n|$))*'
    )
    return re.sub(pattern, "", text).rstrip()

def upsert_key_in_table(text: str, table: str, key: str, value_line: str) -> str:
    header = f"[{table}]"
    lines = text.splitlines()
    start = None
    end = None
    for idx, line in enumerate(lines):
        if line.strip() == header:
            start = idx
            end = len(lines)
            for j in range(idx + 1, len(lines)):
                if lines[j].startswith("[") and lines[j].endswith("]"):
                    end = j
                    break
            break
    if start is None:
        text = text.rstrip()
        if text:
            text += "\n\n"
        return text + f"{header}\n{value_line}\n"
    key_pattern = re.compile(rf"^\s*{re.escape(key)}\s*=")
    for idx in range(start + 1, end):
        if key_pattern.match(lines[idx]):
            lines[idx] = value_line
            return "\n".join(lines).rstrip() + "\n"
    lines.insert(end, value_line)
    return "\n".join(lines).rstrip() + "\n"

args = ["mcp"]
if db_path:
    args += ["--db-path", db_path]

text = strip_table(text, "mcp_servers.membrain")
text = upsert_key_in_table(text, "features", "codex_hooks", "codex_hooks = true")
text = text.rstrip()
if text:
    text += "\n\n"
text += "[mcp_servers.membrain]\n"
text += f"command = {json.dumps(binary_path)}\n"
text += f"args = {json.dumps(args)}\n"

config_path.write_text(text.rstrip() + "\n", encoding="utf-8")
PY
}

install_support_files() {
  write_hook_script || true
  write_daemon_runner
}

install_editor_integrations() {
  install_support_files
  claude_integration_install
  codex_config_install
  codex_hooks_install
}

configure_systemd_user_service() {
  command -v systemctl >/dev/null 2>&1 || return 1
  mkdir -p "$HOME/.config/systemd/user"
  local unit="$HOME/.config/systemd/user/membrain-daemon.service"
  cat > "$unit" <<EOF
[Unit]
Description=Membrain user daemon
After=default.target

[Service]
ExecStart=$DAEMON_RUNNER_PATH
Restart=on-failure
RestartSec=3

[Install]
WantedBy=default.target
EOF
  systemctl --user daemon-reload >/dev/null 2>&1 || true
  systemctl --user enable --now membrain-daemon.service >/dev/null 2>&1
}

configure_launch_agent() {
  command -v launchctl >/dev/null 2>&1 || return 1
  mkdir -p "$HOME/Library/LaunchAgents"
  local plist="$HOME/Library/LaunchAgents/com.${OWNER}.${REPO}.daemon.plist"
  cat > "$plist" <<EOF
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
  <dict>
    <key>Label</key>
    <string>com.${OWNER}.${REPO}.daemon</string>
    <key>ProgramArguments</key>
    <array>
      <string>$DAEMON_RUNNER_PATH</string>
    </array>
    <key>RunAtLoad</key>
    <true/>
    <key>KeepAlive</key>
    <true/>
    <key>StandardOutPath</key>
    <string>$DAEMON_LOG_PATH</string>
    <key>StandardErrorPath</key>
    <string>$DAEMON_LOG_PATH</string>
  </dict>
</plist>
EOF
  launchctl unload "$plist" >/dev/null 2>&1 || true
  launchctl load -w "$plist" >/dev/null 2>&1
}

auto_start_daemon_install() {
  install_support_files
  case "$(uname -s)" in
    Linux*)
      if configure_systemd_user_service; then
        return 0
      fi
      ;;
    Darwin*)
      if configure_launch_agent; then
        return 0
      fi
      ;;
  esac
  log_warn "Automatic daemon startup is not supported on this platform or service manager; starting a persistent user service was skipped"
}

remove_path_lines() {
  for rc in "$HOME/.bashrc" "$HOME/.zshrc"; do
    [ -f "$rc" ] || continue
    python3 - "$rc" "$BINARY_NAME" <<'PY' 2>/dev/null || true
import sys
path, needle = sys.argv[1], sys.argv[2]
with open(path, "r", encoding="utf-8") as fh:
    lines = fh.readlines()
with open(path, "w", encoding="utf-8") as fh:
    for line in lines:
        if f"# {needle} installer" not in line:
            fh.write(line)
PY
  done
}

claude_integration_uninstall() {
  ensure_python3 || return 0
  [ -f "$CLAUDE_SETTINGS_PATH" ] || return 0
  python3 - "$CLAUDE_SETTINGS_PATH" "$HOOK_SCRIPT_PATH" <<'PY'
import json
import pathlib
import sys

settings_path = pathlib.Path(sys.argv[1]).expanduser()
hook_script = sys.argv[2]

try:
    data = json.loads(settings_path.read_text(encoding="utf-8"))
except Exception:
    sys.exit(0)

if not isinstance(data, dict):
    sys.exit(0)

mcp_servers = data.get("mcpServers")
if isinstance(mcp_servers, dict):
    mcp_servers.pop("membrain", None)
    if not mcp_servers:
        data.pop("mcpServers", None)

hooks = data.get("hooks")
if isinstance(hooks, dict):
    for event_name in list(hooks.keys()):
        groups = hooks.get(event_name)
        if not isinstance(groups, list):
            continue
        cleaned = []
        for group in groups:
            hook_defs = group.get("hooks") if isinstance(group, dict) else None
            if isinstance(hook_defs, list) and any(
                isinstance(hook, dict)
                and hook.get("type") == "command"
                and hook_script in str(hook.get("command", ""))
                and "--provider claude" in str(hook.get("command", ""))
                for hook in hook_defs
            ):
                continue
            cleaned.append(group)
        if cleaned:
            hooks[event_name] = cleaned
        else:
            hooks.pop(event_name, None)
    if not hooks:
        data.pop("hooks", None)

settings_path.write_text(json.dumps(data, indent=2) + "\n", encoding="utf-8")
PY
}

codex_hooks_uninstall() {
  ensure_python3 || return 0
  [ -f "$CODEX_HOOKS_PATH" ] || return 0
  python3 - "$CODEX_HOOKS_PATH" "$HOOK_SCRIPT_PATH" <<'PY'
import json
import pathlib
import sys

hooks_path = pathlib.Path(sys.argv[1]).expanduser()
hook_script = sys.argv[2]

try:
    data = json.loads(hooks_path.read_text(encoding="utf-8"))
except Exception:
    sys.exit(0)

if not isinstance(data, dict):
    sys.exit(0)

hooks = data.get("hooks")
if isinstance(hooks, dict):
    for event_name in list(hooks.keys()):
        groups = hooks.get(event_name)
        if not isinstance(groups, list):
            continue
        cleaned = []
        for group in groups:
            hook_defs = group.get("hooks") if isinstance(group, dict) else None
            if isinstance(hook_defs, list) and any(
                isinstance(hook, dict)
                and hook.get("type") == "command"
                and hook_script in str(hook.get("command", ""))
                and "--provider codex" in str(hook.get("command", ""))
                for hook in hook_defs
            ):
                continue
            cleaned.append(group)
        if cleaned:
            hooks[event_name] = cleaned
        else:
            hooks.pop(event_name, None)
    if not hooks:
        hooks_path.unlink(missing_ok=True)
        sys.exit(0)

hooks_path.write_text(json.dumps(data, indent=2) + "\n", encoding="utf-8")
PY
}

codex_config_uninstall() {
  ensure_python3 || return 0
  [ -f "$CODEX_CONFIG_PATH" ] || return 0
  python3 - "$CODEX_CONFIG_PATH" <<'PY'
import pathlib
import re
import sys

config_path = pathlib.Path(sys.argv[1]).expanduser()
text = config_path.read_text(encoding="utf-8")

text = re.sub(
    r'(?ms)^\[mcp_servers\.membrain\]\n(?:^(?!\[).*(?:\n|$))*',
    "",
    text,
).rstrip()

lines = text.splitlines()
for idx, line in enumerate(lines):
    if line.strip() == "[features]":
        end = len(lines)
        for j in range(idx + 1, len(lines)):
            if lines[j].startswith("[") and lines[j].endswith("]"):
                end = j
                break
        lines = [
            line
            for offset, line in enumerate(lines)
            if not (idx < offset < end and re.match(r"^\s*codex_hooks\s*=", line))
        ]
        break

config_path.write_text("\n".join(lines).rstrip() + "\n", encoding="utf-8")
PY
}

remove_support_files() {
  rm -f "$HOOK_SCRIPT_PATH" "$DAEMON_RUNNER_PATH"
  rmdir "$HOOKS_DIR" 2>/dev/null || true
  rmdir "$SUPPORT_DIR" 2>/dev/null || true
}

do_uninstall() {
  rm -f "$DEST/$BINARY_NAME" "$DEST/$DAEMON_BINARY_NAME"
  rm -f "$HOME/.config/systemd/user/membrain-daemon.service"
  if command -v systemctl >/dev/null 2>&1; then
    systemctl --user disable --now membrain-daemon.service >/dev/null 2>&1 || true
    systemctl --user daemon-reload >/dev/null 2>&1 || true
  fi
  local plist="$HOME/Library/LaunchAgents/com.${OWNER}.${REPO}.daemon.plist"
  if [ -f "$plist" ]; then
    command -v launchctl >/dev/null 2>&1 && launchctl unload "$plist" >/dev/null 2>&1 || true
    rm -f "$plist"
  fi
  claude_integration_uninstall
  codex_hooks_uninstall
  codex_config_uninstall
  remove_support_files
  remove_path_lines
  log_success "Uninstalled $BINARY_NAME and $DAEMON_BINARY_NAME"
  exit 0
}

verify_install() {
  "$DEST/$BINARY_NAME" --version >/dev/null
  "$DEST/$DAEMON_BINARY_NAME" --version >/dev/null
}

print_summary() {
  echo ""
  echo "✓ $BINARY_NAME installed → $DEST/$BINARY_NAME"
  echo "✓ $DAEMON_BINARY_NAME installed → $DEST/$DAEMON_BINARY_NAME"
  echo "  Version: $("$DEST/$BINARY_NAME" --version 2>/dev/null || echo 'unknown')"
  echo ""
  echo "  Installed defaults:"
  echo "    Claude MCP + hook config updated at $CLAUDE_SETTINGS_PATH"
  echo "    Codex MCP config + hooks updated at $CODEX_CONFIG_PATH and $CODEX_HOOKS_PATH"
  echo "    Daemon auto-start attempted where supported"
  echo "    Real-time daemon log: $DAEMON_LOG_PATH"
  echo ""
  echo "  Quick start:"
  echo "    $BINARY_NAME --help"
  echo "    $DAEMON_BINARY_NAME --help"
  echo "    tail -f \"$DAEMON_LOG_PATH\""
}

main() {
  acquire_lock
  TMP="$(mktemp -d)"
  mkdir -p "$DEST"

  [ "$UNINSTALL" -eq 1 ] && do_uninstall

  local platform
  platform="$(detect_platform)"
  log_info "Platform: $platform | Dest: $DEST"

  if [ "$FROM_SOURCE" -eq 0 ]; then
    resolve_version
    if ! install_from_archive "$platform"; then
      log_warn "Binary download failed; falling back to source build"
      build_from_source
    fi
  else
    build_from_source
  fi

  maybe_add_path
  install_editor_integrations
  auto_start_daemon_install
  [ "$VERIFY" -eq 1 ] && verify_install

  print_summary
}

if [[ "${BASH_SOURCE[0]:-}" == "${0:-}" ]] || [[ -z "${BASH_SOURCE[0]:-}" ]]; then
  { main "$@"; }
fi
