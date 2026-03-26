#!/usr/bin/env bash
set -euo pipefail

# e2e_mcp.sh
# Logging-heavy daemon / JSON-RPC / MCP parity artifact for the exposed runtime.
#
# This script is the logging-heavy daemon / JSON-RPC / MCP proof artifact for
# the major workflow families exposed through the runtime surface.
#
# It covers these workflow families explicitly:
# 1. retrieval-envelope parity
# 2. contradiction-adjacent safeguard classification / blocked-envelope behavior
# 3. policy denial / preflight safeguard behavior
# 4. share / unshare visibility + redaction parity
# 5. consolidation request-envelope parity
# 6. forgetting archive / restore / policy-delete runtime parity
# 7. schema compression inspectability and lineage parity
# 8. repair / doctor operator-visible runtime parity
# 9. observe / inspect provenance payload parity
#
# Usage:
#   bash crates/membrain-daemon/tests/e2e_mcp.sh
#
# Acceptance artifacts emitted in this script:
# - live daemon CLI stdout/stderr logs for human review
# - targeted JSON-RPC and MCP parity tests with named workflow sections
# - closing workflow checklist that maps runtime proof to the bead contract

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

section "Workflow proof: repair / doctor"
run_check "Runtime doctor JSON-RPC parity test" \
  cargo test -p membrain-daemon --test runtime_doctor_parity runtime_doctor_jsonrpc_response_matches_shared_doctor_contract_fields -- --nocapture
run_check "Runtime doctor resource parity test" \
  cargo test -p membrain-daemon --test runtime_doctor_parity runtime_doctor_resource_read_matches_runtime_doctor_payload_shape -- --nocapture
run_check "Runtime health JSON-RPC parity test" \
  cargo test -p membrain-daemon --test runtime_doctor_parity runtime_health_jsonrpc_response_matches_shared_health_contract_fields -- --nocapture
run_check "Runtime health resource parity test" \
  cargo test -p membrain-daemon --test runtime_doctor_parity runtime_health_resource_read_matches_runtime_health_payload_shape -- --nocapture

section "Workflow proof: contradiction / safeguards"
run_check "Preflight blocked-envelope serialization tests" \
  cargo test -p membrain-daemon force_confirm_blockers_preserve_blocked_serialization_contract -- --nocapture
run_check "Preflight runtime parity tests" \
  cargo test -p membrain-daemon --test preflight_runtime_parity -- --nocapture

section "Workflow proof: share / unshare redaction"
run_check "Share/unshare runtime parity tests" \
  cargo test -p membrain-daemon --test runtime_share_unshare_parity -- --nocapture

section "Workflow proof: consolidation"
run_check "Runtime consolidate request parsing test" \
  cargo test -p membrain-daemon parse_method_consolidate_accepts_canonical_fields -- --nocapture
run_check "MCP consolidate envelope round-trip test" \
  cargo test -p membrain-daemon mutation_method_envelopes_round_trip_via_serde -- --nocapture

section "Workflow proof: forgetting"
run_check "Forget runtime parity tests" \
  cargo test -p membrain-daemon runtime_forget_distinguishes_archive_restore_and_policy_delete -- --nocapture

section "Workflow proof: schema compression"
run_check "Runtime compress parity test" \
  cargo test -p membrain-daemon runtime_compress_applies_schema_compression_and_returns_log_history -- --nocapture
run_check "Runtime schemas parity test" \
  cargo test -p membrain-daemon runtime_schemas_lists_eligible_schema_previews -- --nocapture
run_check "JSON-RPC schemas request parsing test" \
  cargo test -p membrain-daemon parse_method_accepts_schemas_params -- --nocapture
run_check "MCP schemas envelope round-trip test" \
  cargo test -p membrain-daemon mcp_request_envelopes_round_trip_via_serde -- --nocapture

section "Workflow proof: retrieval / inspect envelopes"
run_check "MCP retrieval envelope tests" \
  cargo test -p membrain-daemon mcp_response -- --nocapture
run_check "MCP inspect payload tests" \
  cargo test -p membrain-daemon mcp_inspect_payload -- --nocapture

section "Workflow proof: observe provenance"
run_check "MCP observe payload tests" \
  cargo test -p membrain-daemon observe_request_round_trips -- --nocapture
run_check "Daemon observe/runtime inspect tests" \
  cargo test -p membrain-daemon runtime_inspect_returns_typed_mcp_inspect_payload -- --nocapture

section "Workflow proof: working-state blackboard and checkpoints"
run_check "MCP goal request round-trip tests" \
  cargo test -p membrain-daemon mcp_goal_requests_round_trip -- --nocapture
run_check "Runtime goal blackboard projection parity test" \
  cargo test -p membrain-daemon --test runtime_doctor_parity runtime_goal_working_state_methods_surface_blackboard_projection_parity -- --nocapture
run_check "Runtime goal pause/resume checkpoint parity test" \
  cargo test -p membrain-daemon runtime_goal_pause_and_resume_surface_checkpointed_working_state -- --nocapture
run_check "Runtime goal stale resume parity test" \
  cargo test -p membrain-daemon runtime_goal_resume_without_checkpoint_reports_stale_explicitly -- --nocapture

section "Workflow proof summary"
cat <<'EOF'
Validated runtime surfaces:
- daemon CLI flag parity for runtime and maintenance knobs
- retrieval: MCP retrieval and inspect payload families preserving canonical envelope structure, explicit output_mode, and evidence/action separation
- contradiction: preflight blocked-envelope serialization and runtime safeguard parity for contradiction-adjacent destructive requests
- policy denial: preflight blocked, degraded, and force-confirmed safeguard flows
- share/unshare: visibility handling, redaction semantics, and audit-field parity
- consolidation: runtime request parsing plus MCP consolidate envelope round-trip parity for scriptable maintenance entrypoints
- forgetting: archive/restore/delete parity including reason_code, disposition, reversibility, audit_kind, and operator-review markers
- schema compression: runtime `compress`/`schemas` parity including inspectable schema previews, lineage-bearing compression artifacts, verification state, and compression-log history
- repair: runtime doctor payload shape, health JSON-RPC/resource parity, repair-aware degraded markers, and operator-visible remediation
- observe provenance: passive observation request parsing plus runtime observe/inspect provenance parity
- working-state: MCP goal request coverage includes get/pin/dismiss/snapshot/resume/abandon request surfaces, while runtime proof currently covers blackboard projection plus snapshot, pause, resume, and stale-resume checkpoint lifecycle behavior
EOF

printf '\n=== daemon / MCP parity artifact completed ===\n'
