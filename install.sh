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
WITH_CODEX_MCP=0
WITH_CLAUDE_MCP=0
AUTO_START_DAEMON=0
MAX_RETRIES=3
DOWNLOAD_TIMEOUT=120
LOCK_DIR="/tmp/${BINARY_NAME}-install.lock.d"
TMP=""

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
  --with-codex-mcp          Register Membrain MCP with Codex
  --with-claude-mcp         Register Membrain MCP with Claude Code (user scope)
  --auto-start-daemon       Configure a user service and start membrain-daemon automatically where supported
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
    --with-codex-mcp) WITH_CODEX_MCP=1; shift ;;
    --with-claude-mcp) WITH_CLAUDE_MCP=1; shift ;;
    --auto-start-daemon) AUTO_START_DAEMON=1; shift ;;
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

codex_mcp_install() {
  command -v codex >/dev/null 2>&1 || { log_warn "codex not found; skipping Codex MCP registration"; return 0; }
  local cmd=(codex mcp add membrain -- "$DEST/$BINARY_NAME" mcp)
  [ -n "$DB_PATH" ] && cmd+=("--db-path" "$DB_PATH")
  if codex mcp get membrain >/dev/null 2>&1; then
    log_warn "Codex MCP server 'membrain' already exists; skipping"
    return 0
  fi
  "${cmd[@]}" >/dev/null 2>&1 || log_warn "Failed to register Membrain MCP with Codex"
}

claude_mcp_install() {
  command -v claude >/dev/null 2>&1 || { log_warn "claude not found; skipping Claude MCP registration"; return 0; }
  local cmd=(claude mcp add --transport stdio --scope user membrain -- "$DEST/$BINARY_NAME" mcp)
  [ -n "$DB_PATH" ] && cmd+=("--db-path" "$DB_PATH")
  if claude mcp get membrain >/dev/null 2>&1; then
    log_warn "Claude MCP server 'membrain' already exists; skipping"
    return 0
  fi
  "${cmd[@]}" >/dev/null 2>&1 || log_warn "Failed to register Membrain MCP with Claude Code"
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
ExecStart=$DEST/$DAEMON_BINARY_NAME$( [ -n "$DB_PATH" ] && printf ' --db-path %q' "$DB_PATH" )
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
      <string>$DEST/$DAEMON_BINARY_NAME</string>
$( [ -n "$DB_PATH" ] && printf '      <string>--db-path</string>\n      <string>%s</string>\n' "$DB_PATH" )
    </array>
    <key>RunAtLoad</key>
    <true/>
    <key>KeepAlive</key>
    <true/>
    <key>StandardOutPath</key>
    <string>$HOME/Library/Logs/membrain-daemon.log</string>
    <key>StandardErrorPath</key>
    <string>$HOME/Library/Logs/membrain-daemon.log</string>
  </dict>
</plist>
EOF
  launchctl unload "$plist" >/dev/null 2>&1 || true
  launchctl load -w "$plist" >/dev/null 2>&1
}

auto_start_daemon_install() {
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
  echo "  Quick start:"
  echo "    $BINARY_NAME --help"
  echo "    $DAEMON_BINARY_NAME --help"
  if [ "$WITH_CODEX_MCP" -eq 1 ]; then
    echo "    Codex MCP registration requested"
  fi
  if [ "$WITH_CLAUDE_MCP" -eq 1 ]; then
    echo "    Claude MCP registration requested"
  fi
  if [ "$AUTO_START_DAEMON" -eq 1 ]; then
    echo "    Daemon auto-start requested"
  fi
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

  [ "$WITH_CODEX_MCP" -eq 1 ] && codex_mcp_install
  [ "$WITH_CLAUDE_MCP" -eq 1 ] && claude_mcp_install
  [ "$AUTO_START_DAEMON" -eq 1 ] && auto_start_daemon_install
  [ "$VERIFY" -eq 1 ] && verify_install

  print_summary
}

if [[ "${BASH_SOURCE[0]:-}" == "${0:-}" ]] || [[ -z "${BASH_SOURCE[0]:-}" ]]; then
  { main "$@"; }
fi
