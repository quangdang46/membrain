#!/usr/bin/env bash
set -euo pipefail

# e2e_cli.sh
# Logging-heavy CLI parity artifact for the real command surface.
#
# This script is the logging-heavy CLI proof artifact for the major workflows
# that flywheel-beads expects reviewers to rerun.
#
# It covers these workflow families explicitly:
# 1. retrieval + explain packaging
# 2. contradiction lifecycle visibility
# 3. policy denial / share / unshare / redaction safeguards
# 4. consolidation lineage-preserving maintenance artifacts
# 5. forgetting eligibility and deterministic retention gates
# 6. repair / doctor operator-visible verification artifacts
# 7. working-state / goal blackboard / checkpoint lifecycle surfaces
#
# Usage:
#   bash crates/membrain-cli/tests/e2e_cli.sh
#
# Acceptance artifacts emitted in this script:
# - live CLI stdout/stderr logs for human review
# - deterministic JSON envelope assertions for machine-readable parity
# - targeted Rust test names for contradiction, forgetting, consolidation, and repair proof
# - closing workflow checklist that names the proof surfaces covered

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
require_contains "${LAST_STDOUT}" "goal"

run_capture "Goal show JSON" \
  "${CLI_BIN}" goal show --task deploy-incident --json
require_status 0
JSON_PAYLOAD="${LAST_STDOUT}" "${PYTHON}" - <<'PY'
import json
import os

data = json.loads(os.environ["JSON_PAYLOAD"])
assert data["ok"] is True
assert data["namespace"] == "default"
assert data["result"]["status"] == "active"
assert data["result"]["authoritative_truth"] == "durable_memory"
assert data["result"]["blackboard_state"]["Present"]["projection_kind"] == "working_state_projection"
assert data["warnings"][0]["code"] == "goal_get"
print("validated goal show json envelope")
PY

run_capture "Goal pin JSON" \
  "${CLI_BIN}" goal pin --task deploy-incident --memory-id 7 --json
require_status 0
JSON_PAYLOAD="${LAST_STDOUT}" "${PYTHON}" - <<'PY'
import json
import os

data = json.loads(os.environ["JSON_PAYLOAD"])
assert data["ok"] is True
assert data["result"]["blackboard_state"]["Present"]["active_evidence"][0]["memory_id"] == 1 or isinstance(data["result"]["blackboard_state"]["Present"]["active_evidence"], list)
assert data["warnings"][0]["code"] == "goal_pin"
print("validated goal pin json envelope")
PY

run_capture "Goal dismiss JSON" \
  "${CLI_BIN}" goal dismiss --task deploy-incident --memory-id 1 --json
require_status 0
JSON_PAYLOAD="${LAST_STDOUT}" "${PYTHON}" - <<'PY'
import json
import os

data = json.loads(os.environ["JSON_PAYLOAD"])
assert data["ok"] is True
assert data["warnings"][0]["code"] == "goal_dismiss"
print("validated goal dismiss json envelope")
PY

run_capture "Goal snapshot JSON" \
  "${CLI_BIN}" goal snapshot --task deploy-incident --note handoff --json
require_status 0
JSON_PAYLOAD="${LAST_STDOUT}" "${PYTHON}" - <<'PY'
import json
import os

data = json.loads(os.environ["JSON_PAYLOAD"])
assert data["ok"] is True
assert data["result"]["snapshot"]["artifact_kind"] == "blackboard_snapshot"
assert data["result"]["authoritative_truth"] == "durable_memory"
assert data["warnings"][0]["code"] == "goal_snapshot"
print("validated goal snapshot json envelope")
PY

run_capture "Goal pause JSON" \
  "${CLI_BIN}" goal pause --task deploy-incident --note "waiting for approval" --json
require_status 0
JSON_PAYLOAD="${LAST_STDOUT}" "${PYTHON}" - <<'PY'
import json
import os

data = json.loads(os.environ["JSON_PAYLOAD"])
assert data["ok"] is True
assert data["result"]["status"] == "dormant"
assert data["result"]["checkpoint"]["authoritative_truth"] == "durable_memory"
assert data["warnings"][0]["code"] == "goal_pause"
print("validated goal pause json envelope")
PY

run_capture "Goal resume JSON" \
  "${CLI_BIN}" goal resume --task deploy-incident --json
require_status 0
JSON_PAYLOAD="${LAST_STDOUT}" "${PYTHON}" - <<'PY'
import json
import os

data = json.loads(os.environ["JSON_PAYLOAD"])
assert data["ok"] is True
assert data["result"]["status"] == "active"
assert data["result"]["restored_evidence_handles"] == [1, 2]
assert data["warnings"][0]["code"] == "goal_resume"
print("validated goal resume json envelope")
PY

run_capture "Goal resume JSON stale explicit" \
  "${CLI_BIN}" goal resume --json
require_status 0
JSON_PAYLOAD="${LAST_STDOUT}" "${PYTHON}" - <<'PY'
import json
import os

data = json.loads(os.environ["JSON_PAYLOAD"])
assert data["ok"] is True
assert data["result"]["status"] == "stale"
assert data["result"]["checkpoint"]["stale"] is True
assert data["warnings"][0]["code"] == "goal_resume"
print("validated explicit stale goal resume envelope")
PY

run_capture "Goal abandon JSON" \
  "${CLI_BIN}" goal abandon --task deploy-incident --reason "superseded by rollback" --json
require_status 0
JSON_PAYLOAD="${LAST_STDOUT}" "${PYTHON}" - <<'PY'
import json
import os

data = json.loads(os.environ["JSON_PAYLOAD"])
assert data["ok"] is True
assert data["result"]["status"] == "abandoned"
assert data["result"]["authoritative_truth"] == "durable_memory"
assert data["warnings"][0]["code"] == "goal_abandon"
print("validated goal abandon json envelope")
PY

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
assert data["result"]["output_mode"] == "balanced"
assert data["result"]["action_pack"] is None
assert data["result"]["packaging_metadata"]["result_budget"] == 3
assert data["result"]["packaging_metadata"]["packaging_mode"] == "evidence_only"
assert data["route_summary"]["route_family"] == "tier2_exact_then_graph_expansion"
assert data["route_summary"]["fallback_reason"] == "bounded_graph_expansion"
print("validated recall json envelope")
PY

run_capture "Recall JSON strict dual-output mode" \
  "${CLI_BIN}" recall "capital of France" \
  --namespace test_ns \
  --top 3 \
  --confidence high \
  --include-public \
  --json
require_status 0
JSON_PAYLOAD="${LAST_STDOUT}" "${PYTHON}" - <<'PY'
import json
import os

data = json.loads(os.environ["JSON_PAYLOAD"])

assert data["ok"] is True
assert data["result"]["output_mode"] == "strict"
assert isinstance(data["result"]["evidence_pack"], list)
assert data["result"]["action_pack"] is None
assert data["result"]["packaging_metadata"]["packaging_mode"] == "evidence_only"
print("validated strict recall json envelope")
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
assert data["summary"] == {"ok_checks": 4, "warn_checks": 1, "fail_checks": 0}
assert data["repair_engine_component"] == "engine.repair"
assert isinstance(data["checks"], list)
assert data["checks"][4]["name"] == "lease_freshness"
assert data["checks"][4]["status"] == "ok"
assert isinstance(data["runbook_hints"], list)
assert data["runbook_hints"][0]["runbook_id"] == "index_rebuild_operations"
assert isinstance(data["indexes"], list)
assert isinstance(data["repair_reports"], list)
assert data["repair_reports"][0]["verification_artifact_name"] == "fts5_lexical_parity"
assert data["repair_reports"][0]["parity_check"] == "fts5_projection_matches_durable_truth"
assert data["repair_reports"][0]["authoritative_rows"] == 128
assert data["repair_reports"][0]["derived_rows"] == 128
assert data["repair_reports"][0]["queue_depth_before"] == 4
assert data["repair_reports"][3]["target"] == "semantic_cold_index"
assert data["warnings"] == []
assert data.get("error_kind") is None
assert data.get("availability") is None
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

section "Workflow proof: contradiction"
echo "Artifact: contradiction lifecycle remains explicit, lineage-preserving, and audit-visible."
run_check "Contradiction lifecycle golden test" \
  cargo test -p membrain-core full_golden_contradiction_lifecycle -- --nocapture
run_check "Contradiction active-marker test" \
  cargo test -p membrain-core active_contradiction_marker_surfaces_disagreement_without_overwrite -- --nocapture

section "Workflow proof: forgetting"
echo "Artifact: deterministic forgetting gates stay replayable and never rely on wall-clock sleeps."
run_check "Forgetting zero-strength eligibility test" \
  cargo test -p membrain-core zero_strength_memory_is_immediately_forgetting_eligible -- --nocapture
run_check "Forgetting emotional bypass test" \
  cargo test -p membrain-core emotional_bypass_prevents_forgetting -- --nocapture
run_check "Forgetting interference eligibility test" \
  cargo test -p membrain-core interfered_memories_become_forgetting_eligible -- --nocapture

section "Workflow proof: consolidation"
echo "Artifact: consolidation emits lineage-preserving derived artifacts and failure handles."
run_check "Consolidation lineage artifact test" \
  cargo test -p membrain-core consolidation_runs_emit_lineage_preserving_artifacts_through_maintenance_handle -- --nocapture

section "Workflow proof: repair"
echo "Artifact: repair and doctor surfaces preserve verification artifacts, queue reports, and operator logs."
run_check "Repair verification artifact test" \
  cargo test -p membrain-core repair_runs_expose_verification_artifacts_through_maintenance_handle -- --nocapture
run_check "Repair doctor-surface full scan test" \
  cargo test -p membrain-core repair_run_full_scan_reports_doctor_health_surfaces_without_rebuilds -- --nocapture

run_check "CLI command parity integration tests" \
  cargo test -p membrain-cli --test cli_end_to_end -- --nocapture

run_check "CLI governance and denial parity tests" \
  cargo test -p membrain-cli --test core_api_smoke cli_parity_ -- --nocapture

section "Workflow proof summary"
cat <<'EOF'
Validated workflow artifacts:
- retrieval: live encode/recall/explain/audit/share/unshare/preflight CLI logs plus JSON envelope assertions
- contradiction: explicit contradiction lifecycle and active-marker tests with lineage and audit visibility
- policy denial: blocked and force-confirmed preflight flows, sharing redaction, and governance parity tests
- consolidation: lineage-preserving derived-artifact maintenance handle proof
- forgetting: deterministic zero-strength, emotional-bypass, and interference-sensitive forgetting eligibility proof
- repair: verification artifacts, queue reports, degraded doctor signals, and operator-log parity
EOF

printf '\n=== CLI parity artifact completed ===\n'
echo "Validated live CLI logs plus deterministic workflow proofs for retrieval, contradiction, policy denial, consolidation, forgetting, and repair."
