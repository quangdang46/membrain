#!/usr/bin/env bash
set -euo pipefail

# e2e_mcp.sh
# Logging-heavy daemon / JSON-RPC / MCP parity artifact for the exposed runtime.
#
# This script validates three things:
# 1. real daemon CLI/runtime flags
# 2. daemon JSON-RPC parity tests for doctor, preflight, and share/unshare flows
# 3. MCP retrieval-envelope shape tests for the shared retrieval payload contract
#
# Usage:
#   bash crates/membrain-daemon/tests/e2e_mcp.sh

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "${SCRIPT_DIR}/../../.." && pwd)"
cd "${REPO_ROOT}"

DAEMON_BIN="${REPO_ROOT}/target/debug/membrain-daemon"
LAST_STDOUT=""
LAST_STDERR=""
LAST_STATUS=0

section() {
  printf '\n=== %s ===\n' "$1"
}

run_capture() {
  local label="$1"
  shift

  local stdout_file
  local stderr_file
  stdout_file="$(mktemp)"
  stderr_file="$(mktemp)"

  section "$label"
  printf '+'
  printf ' %q' "$@"
  printf '\n'

  if "$@" >"${stdout_file}" 2>"${stderr_file}"; then
    LAST_STATUS=0
  else
    LAST_STATUS=$?
  fi

  LAST_STDOUT="$(<"${stdout_file}")"
  LAST_STDERR="$(<"${stderr_file}")"

  if [[ -n "${LAST_STDOUT}" ]]; then
    printf '%s\n' "${LAST_STDOUT}"
  fi
  if [[ -n "${LAST_STDERR}" ]]; then
    printf '%s\n' '--- stderr ---'
    printf '%s\n' "${LAST_STDERR}"
  fi
  printf '%s\n' "--- exit: ${LAST_STATUS} ---"

  rm -f "${stdout_file}" "${stderr_file}"
}

run_check() {
  local label="$1"
  shift

  section "$label"
  printf '+'
  printf ' %q' "$@"
  printf '\n'
  "$@"
}

require_status() {
  local expected="$1"
  if [[ "${LAST_STATUS}" -ne "${expected}" ]]; then
    echo "expected exit ${expected}, got ${LAST_STATUS}" >&2
    exit 1
  fi
}

require_contains() {
  local haystack="$1"
  local needle="$2"
  if [[ "${haystack}" != *"${needle}"* ]]; then
    echo "missing expected text: ${needle}" >&2
    exit 1
  fi
}

section "Membrain daemon / MCP parity artifact"
echo "Repo: ${REPO_ROOT}"
echo "Binary: membrain-daemon"
echo "Validation: daemon help surface + targeted JSON-RPC and MCP parity tests"

run_check "Build daemon binary" cargo build -p membrain-daemon --bin membrain-daemon

run_capture "Daemon help" "${DAEMON_BIN}" --help
require_status 0
require_contains "${LAST_STDOUT}" "Membrain local daemon"
require_contains "${LAST_STDOUT}" "--socket-path"
require_contains "${LAST_STDOUT}" "--request-concurrency"
require_contains "${LAST_STDOUT}" "--max-queue-depth"
require_contains "${LAST_STDOUT}" "--maintenance-interval-secs"
require_contains "${LAST_STDOUT}" "--maintenance-poll-budget"
require_contains "${LAST_STDOUT}" "--maintenance-step-delay-ms"

run_capture "Daemon rejects zero maintenance interval" \
  "${DAEMON_BIN}" \
  --maintenance-interval-secs 0
if [[ "${LAST_STATUS}" -eq 0 ]]; then
  echo "expected clap validation failure for zero maintenance interval" >&2
  exit 1
fi
require_contains "${LAST_STDERR}" "value must be at least 1"

run_capture "Daemon rejects non-numeric queue depth" \
  "${DAEMON_BIN}" \
  --max-queue-depth abc
if [[ "${LAST_STATUS}" -eq 0 ]]; then
  echo "expected clap validation failure for non-numeric queue depth" >&2
  exit 1
fi
require_contains "${LAST_STDERR}" "invalid integer value: abc"

run_check "Runtime doctor parity tests" \
  cargo test -p membrain-daemon --test runtime_doctor_parity -- --nocapture

run_check "Preflight runtime parity tests" \
  cargo test -p membrain-daemon --test preflight_runtime_parity -- --nocapture

run_check "Share/unshare runtime parity tests" \
  cargo test -p membrain-daemon --test runtime_share_unshare_parity -- --nocapture

run_check "Forget runtime parity tests" \
  cargo test -p membrain-daemon runtime_forget_distinguishes_archive_restore_and_policy_delete -- --nocapture

run_check "MCP retrieval envelope tests" \
  cargo test -p membrain-daemon mcp_response -- --nocapture

run_check "MCP observe payload tests" \
  cargo test -p membrain-daemon observe_request_round_trips -- --nocapture

run_check "MCP inspect payload tests" \
  cargo test -p membrain-daemon mcp_inspect_payload -- --nocapture

run_check "Daemon observe/runtime inspect tests" \
  cargo test -p membrain-daemon runtime_inspect_returns_typed_mcp_inspect_payload -- --nocapture

section "Reference parity notes"
cat <<'EOF'
Validated surfaces:
- daemon CLI flag parity for runtime and maintenance knobs
- JSON-RPC error-model parity for doctor/resources/streams validation
- JSON-RPC runtime doctor payload shape and runtime resource listings
- preflight blocked, degraded, and force-confirmed policy/governance flows
- preflight MCP request-envelope parity for run/explain/allow wrappers
- encode/share/unshare visibility handling, redaction, and audit-field parity
- passive observation request parsing plus runtime observe/inspect provenance parity
- forget archive/restore/delete parity including reason_code, disposition, reversibility, audit_kind, and operator-review markers
- MCP retrieval and inspect payload families preserving canonical envelope structure
EOF

printf '\n=== daemon / MCP parity artifact completed ===\n'
