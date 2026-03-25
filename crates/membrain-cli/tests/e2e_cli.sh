#!/usr/bin/env bash
set -euo pipefail

# e2e_cli.sh
# Logging-heavy CLI parity artifact for the real command surface.
#
# This script does two things:
# 1. runs representative CLI commands and prints the actual human/JSON payloads
# 2. runs targeted Rust parity tests so policy and envelope drift fail loudly
#
# Usage:
#   bash crates/membrain-cli/tests/e2e_cli.sh

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "${SCRIPT_DIR}/../../.." && pwd)"
cd "${REPO_ROOT}"

if command -v python3 >/dev/null 2>&1; then
  PYTHON=python3
elif command -v python >/dev/null 2>&1; then
  PYTHON=python
else
  echo "python3 or python is required for JSON validation" >&2
  exit 1
fi

CLI_BIN="${REPO_ROOT}/target/debug/membrain"
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

section "Membrain CLI parity artifact"
echo "Repo: ${REPO_ROOT}"
echo "Binary: membrain"
echo "Validation: live command logs + targeted cargo parity tests"

run_check "Build membrain CLI binary" cargo build -p membrain-cli --bin membrain

run_capture "CLI help" "${CLI_BIN}" --help
require_status 0
require_contains "${LAST_STDOUT}" "Membrain CLI"
require_contains "${LAST_STDOUT}" "recall"
require_contains "${LAST_STDOUT}" "share"
require_contains "${LAST_STDOUT}" "doctor"

run_capture "Encode JSON" \
  "${CLI_BIN}" encode "Paris is the capital of France" \
  --namespace test_ns \
  --kind semantic \
  --source cli-e2e \
  --json
require_status 0
JSON_PAYLOAD="${LAST_STDOUT}" "${PYTHON}" -c '
import json
import os

data = json.loads(os.environ["JSON_PAYLOAD"])
assert data["ok"] is True
assert data["namespace"] == "test_ns"
assert data["outcome_class"] == "accepted"
assert data["result"]["memory_id"] == 1
assert data["result"]["memory_type"] == "observation"
assert data["result"]["compact_text"] == "Paris is the capital of France"
assert data["result"]["source"] == "cli-e2e"
print("validated encode json envelope")
'

run_capture "Recall JSON" \
  "${CLI_BIN}" recall "capital of France" \
  --namespace test_ns \
  --top 3 \
  --explain full \
  --json
require_status 0
JSON_PAYLOAD="${LAST_STDOUT}" "${PYTHON}" - <<'PY'
import json
import os

data = json.loads(os.environ["JSON_PAYLOAD"])

assert data["ok"] is True
assert data["namespace"] == "test_ns"
assert data["outcome_class"] == "accepted"
assert isinstance(data["route_summary"]["route_reason"], str)
assert isinstance(data["route_summary"]["tier1_consulted_first"], bool)
assert isinstance(data["trace_stages"], list)
assert isinstance(data["result"]["evidence_pack"], list)
assert data["result"]["action_pack"] is None
assert data["result"]["packaging_metadata"]["result_budget"] == 3
print("validated recall json envelope")
PY

run_capture "Observe JSON passive-observation provenance" \
  "${CLI_BIN}" observe "watcher noticed deploy drift\n\nwatcher saw cache warmup finish" \
  --namespace test_ns \
  --source-label stdin:e2e \
  --json
require_status 0
JSON_PAYLOAD="${LAST_STDOUT}" "${PYTHON}" - <<'PY'
import json
import os

data = json.loads(os.environ["JSON_PAYLOAD"])

assert data["ok"] is True
assert data["namespace"] == "test_ns"
assert data["outcome_class"] == "accepted"
assert data["passive_observation"]["source_kind"] == "observation"
assert data["passive_observation"]["write_decision"] == "capture"
assert data["passive_observation"]["captured_as_observation"] is True
assert data["passive_observation"]["observation_source"] == {"Present": "stdin:e2e"}
assert data["passive_observation"]["observation_chunk_id"]["Present"].startswith("obs-")
assert data["passive_observation"]["retention_marker"] == {"Present": "volatile_observation"}
print("validated observe passive-observation json envelope")
PY

run_capture "Inspect missing memory JSON failure envelope" \
  "${CLI_BIN}" inspect \
  --id 123 \
  --namespace test_ns \
  --json
require_status 0
JSON_PAYLOAD="${LAST_STDOUT}" "${PYTHON}" - <<'PY'
import json
import os

data = json.loads(os.environ["JSON_PAYLOAD"])

assert data["ok"] is False
assert data["error_kind"] == "validation_failure"
assert data["outcome_class"] == "rejected"
assert data["request_id"] == "inspect-not-found"
assert data["remediation"]["summary"] == "validation_failure"
assert data["remediation"]["next_steps"][0] == "fix_request"
print("validated inspect validation-failure envelope")
PY

run_capture "Explain JSON" \
  "${CLI_BIN}" explain "how to deploy the service after the last incident?" \
  --namespace test_ns \
  --json
require_status 0
JSON_PAYLOAD="${LAST_STDOUT}" "${PYTHON}" - <<'PY'
import json
import os

data = json.loads(os.environ["JSON_PAYLOAD"])

assert data["ok"] is True
assert data["namespace"] == "test_ns"
assert data["outcome_class"] == "accepted"
assert isinstance(data["trace_stages"], list)
assert data["result"]["explain"]["ranking_profile"] == "balanced"
assert any(
    "matched_patterns=how to" in reason["detail"]
    for reason in data["result"]["explain"]["result_reasons"]
)
print("validated explain json envelope")
PY

run_capture "Maintenance JSON" \
  "${CLI_BIN}" maintenance \
  --action repair \
  --namespace test_ns \
  --json
require_status 0
JSON_PAYLOAD="${LAST_STDOUT}" "${PYTHON}" - <<'PY'
import json
import os

data = json.loads(os.environ["JSON_PAYLOAD"])

assert data["outcome"] == "accepted"
assert data["action"] == "repair"
assert data["namespace"] == "test_ns"
assert data["targets_checked"] == 7
assert data["rebuilt"] == 7
assert data["affected_item_count"] == 458
assert data["error_count"] == 0
assert data["queue_depth_before"] == 7
assert data["queue_depth_after"] == 0
assert len(data["results"]) == 7
assert data["results"][0]["verification_artifact_name"] == "fts5_lexical_parity"
assert data["results"][0]["parity_check"] == "fts5_projection_matches_durable_truth"
assert data["results"][0]["authoritative_rows"] == 128
assert data["results"][0]["derived_rows"] == 128
assert data["results"][0]["durable_sources"] == [
  "durable_memory_records",
  "namespace_policy_metadata",
  "canonical_content_handles",
]
print("validated maintenance json envelope")
PY

run_capture "Audit JSON" \
  "${CLI_BIN}" audit \
  --namespace team.alpha \
  --id 21 \
  --recent 1 \
  --json
require_status 0
JSON_PAYLOAD="${LAST_STDOUT}" "${PYTHON}" - <<'PY'
import json
import os

data = json.loads(os.environ["JSON_PAYLOAD"])

assert data["total_matches"] == 3
assert data["returned_rows"] == 1
assert data["truncated"] is True
row = data["rows"][0]
assert row["request_id"] == "req-migration-21"
assert row["related_run"] == "migration-0042"
assert row["kind"] == "maintenance_migration_applied"
print("validated audit json correlation fields")
PY

run_capture "Share JSON" \
  "${CLI_BIN}" share \
  --id 42 \
  --namespace team.beta \
  --json
require_status 0
JSON_PAYLOAD="${LAST_STDOUT}" "${PYTHON}" - <<'PY'
import json
import os

data = json.loads(os.environ["JSON_PAYLOAD"])

assert data["ok"] is True
assert data["namespace"] == "team.beta"
assert data["result"]["visibility"] == "shared"
assert data["result"]["policy_summary"]["policy_family"] == "visibility_sharing"
audit = data["result"]["audit"]
assert audit["event_kind"] == "approved_sharing"
assert audit["actor_source"] == "cli_share"
assert audit["request_id"] == "req-share-42"
assert audit["effective_namespace"] == "team.beta"
assert audit["source_namespace"] == "team.beta"
assert audit["target_namespace"] == "team.beta"
assert audit["policy_family"] == "visibility_sharing"
assert audit["outcome_class"] == "accepted"
assert audit["blocked_stage"] == "policy_gate"
assert audit["related_run"] == "share-run-42"
assert audit["redacted"] is False
assert data["result"]["audit_rows"][0]["request_id"] == "req-share-42"
assert data["result"]["audit_rows"][0]["kind"] == "approved_sharing"
print("validated share json envelope")
PY

run_capture "Recall JSON include-public policy widening" \
  "${CLI_BIN}" recall "capital of France" \
  --namespace test_ns \
  --top 3 \
  --include-public \
  --json
require_status 0
JSON_PAYLOAD="${LAST_STDOUT}" "${PYTHON}" - <<'PY'
import json
import os

data = json.loads(os.environ["JSON_PAYLOAD"])

assert data["ok"] is True
assert data["policy_filters_applied"][0]["policy_family"] == "shared_public_widening"
assert data["policy_filters_applied"][0]["sharing_scope"] == {"Present": "approved_shared"}
assert data["policy_summary"]["filters"][0]["policy_family"] == "shared_public_widening"
assert data["policy_summary"]["filters"][0]["sharing_scope"] == {"Present": "approved_shared"}
print("validated include-public policy widening")
PY

run_capture "Unshare JSON" \
  "${CLI_BIN}" unshare \
  --id 42 \
  --namespace team.alpha \
  --json
require_status 0
JSON_PAYLOAD="${LAST_STDOUT}" "${PYTHON}" - <<'PY'
import json
import os

data = json.loads(os.environ["JSON_PAYLOAD"])

assert data["ok"] is True
assert data["namespace"] == "team.alpha"
assert data["result"]["visibility"] == "private"
assert data["result"]["policy_summary"]["redaction_fields"][0] == "sharing_scope"
audit = data["result"]["audit"]
assert audit["event_kind"] == "policy_redacted"
assert audit["actor_source"] == "cli_unshare"
assert audit["request_id"] == "req-unshare-42"
assert audit["effective_namespace"] == "team.alpha"
assert audit["source_namespace"] == "team.alpha"
assert audit["target_namespace"] == "team.alpha"
assert audit["policy_family"] == "visibility_sharing"
assert audit["outcome_class"] == "accepted"
assert audit["blocked_stage"] == "policy_gate"
assert audit["related_run"] == "share-run-42"
assert audit["redacted"] is True
assert data["result"]["audit_rows"][0]["request_id"] == "req-unshare-42"
assert data["result"]["audit_rows"][0]["kind"] == "policy_redacted"
assert data["result"]["audit_rows"][0]["redacted"] is True
print("validated unshare redaction envelope")
PY

run_capture "Doctor JSON report" "${CLI_BIN}" doctor --json
require_status 0
JSON_PAYLOAD="${LAST_STDOUT}" "${PYTHON}" - <<'PY'
import json
import os

data = json.loads(os.environ["JSON_PAYLOAD"])

assert data["status"] == "ok"
assert data["action"] == "doctor"
assert isinstance(data["metrics"], dict)
assert isinstance(data["indexes"], list)
assert isinstance(data["repair_reports"], list)
assert data["repair_reports"][0]["verification_artifact_name"] == "fts5_lexical_parity"
assert data["repair_reports"][0]["parity_check"] == "fts5_projection_matches_durable_truth"
assert data["repair_reports"][0]["authoritative_rows"] == 128
assert data["repair_reports"][0]["derived_rows"] == 128
assert data["repair_reports"][0]["queue_depth_before"] == 4
assert data["repair_reports"][3]["target"] == "semantic_cold_index"
assert isinstance(data["health"], dict)
print("validated doctor report")
PY

run_capture "Preflight explain JSON" \
  "${CLI_BIN}" preflight explain \
  --namespace team.alpha \
  --original-query "delete prior audit events across all namespaces" \
  --proposed-action "purge namespace audit history" \
  --json
require_status 4
JSON_PAYLOAD="${LAST_STDOUT}" "${PYTHON}" - <<'PY'
import json
import os

data = json.loads(os.environ["JSON_PAYLOAD"])

assert data["allowed"] is False
assert data["preflight_state"] == "blocked"
assert data["preflight_outcome"] == "preview_only"
assert data["blocked_reasons"] == ["scope_ambiguous", "confirmation_required"]
assert data["audit"]["actor_source"] == "cli_preflight"
print("validated preflight explain blocked json")
PY

run_capture "Preflight allow JSON" \
  "${CLI_BIN}" preflight allow \
  --namespace team.alpha \
  --original-query "delete prior audit events" \
  --proposed-action "purge namespace audit history" \
  --authorization-token allow-123 \
  --bypass-flag manual_override \
  --json
require_status 0
JSON_PAYLOAD="${LAST_STDOUT}" "${PYTHON}" - <<'PY'
import json
import os

data = json.loads(os.environ["JSON_PAYLOAD"])

assert data["success"] is True
assert data["preflight_state"] == "ready"
assert data["preflight_outcome"] == "force_confirmed"
assert data["confirmation"]["confirmed"] is True
assert data["confirmation_reason"] == "operator confirmed exact previewed scope"
print("validated preflight allow force-confirmed json")
PY

run_capture "Recall human output" \
  "${CLI_BIN}" recall "capital of France" \
  --namespace test_ns \
  --top 3 \
  --explain full
require_status 0
require_contains "${LAST_STDOUT}" "Recall 'capital of France' in 'test_ns' → 0 results"
require_contains "${LAST_STDOUT}" "route: tier2_exact_then_graph_expansion → small_session_lookup"
require_contains "${LAST_STDOUT}" "tier1: consulted=false, answered_directly=false, deeper=true"

run_capture "Audit human output" \
  "${CLI_BIN}" audit \
  --namespace team.alpha \
  --id 21 \
  --recent 1
require_status 0
require_contains "${LAST_STDOUT}" "matched=3 returned=1 truncated=true"
require_contains "${LAST_STDOUT}" "maintenance_migration_applied"
require_contains "${LAST_STDOUT}" "request_id=Some(\"req-migration-21\")"
require_contains "${LAST_STDOUT}" "run=Some(\"migration-0042\")"

run_capture "Share human output" \
  "${CLI_BIN}" share \
  --id 42 \
  --namespace team.beta
require_status 0
require_contains "${LAST_STDOUT}" "Shared memory #42 into 'team.beta' [shared]"

run_capture "Unshare human output" \
  "${CLI_BIN}" unshare \
  --id 42 \
  --namespace team.alpha
require_status 0
require_contains "${LAST_STDOUT}" "Unshared memory #42 in 'team.alpha' [private]"

run_capture "Preflight explain human output" \
  "${CLI_BIN}" preflight explain \
  --namespace team.alpha \
  --original-query "delete prior audit events across all namespaces" \
  --proposed-action "purge namespace audit history"
require_status 4
require_contains "${LAST_STDOUT}" "Preflight explain [blocked] state=blocked outcome=preview_only"
require_contains "${LAST_STDOUT}" "blocked_reasons: scope_ambiguous, confirmation_required"

run_capture "Preflight allow human output" \
  "${CLI_BIN}" preflight allow \
  --namespace team.alpha \
  --original-query "delete prior audit events" \
  --proposed-action "purge namespace audit history" \
  --authorization-token allow-123 \
  --bypass-flag manual_override
require_status 0
require_contains "${LAST_STDOUT}" "Preflight allow [accepted] state=ready outcome=force_confirmed"
require_contains "${LAST_STDOUT}" "confirmation_reason: operator confirmed exact previewed scope"

run_check "CLI command parity integration tests" \
  cargo test -p membrain-cli --test cli_end_to_end -- --nocapture

run_check "CLI governance and denial parity tests" \
  cargo test -p membrain-cli --test core_api_smoke cli_parity_ -- --nocapture

printf '\n=== CLI parity artifact completed ===\n'
echo "Validated live CLI logs for canonical retrieval, audit, sharing, redaction, and doctor surfaces."
echo "Validated deterministic parity tests for encode/recall pipeline, denial matrices, cross-namespace redaction, and degraded outcomes."
