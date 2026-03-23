#!/usr/bin/env bash
set -euo pipefail

# e2e_cli.sh
# Logging-heavy CLI parity artifact for the currently exposed CLI surface.
#
# This script is intentionally honest about the current repo state:
# - it uses the real clap spellings exposed by crates/membrain-cli/src/main.rs
# - it does not invent unsupported flags like --content / --query / --limit
# - if the CLI binary does not compile, it reports that blocker explicitly
#
# Usage:
#   bash crates/membrain-cli/tests/e2e_cli.sh

CLI_CMD=(cargo run -p membrain-cli --)

run_json() {
  local label="$1"
  shift
  echo "=== ${label} ==="
  if ! output="$(${CLI_CMD[@]} "$@" 2>&1)"; then
    echo "$output"
    return 1
  fi
  echo "$output"
  echo
}

echo "=== Membrain CLI parity artifact ==="
echo "Binary: membrain"
echo

if ! help_output="$(${CLI_CMD[@]} --help 2>&1)"; then
  echo "$help_output"
  echo
  echo "CLI build/help is currently blocked before end-to-end command checks can run."
  echo "Known blocker path: crates/membrain-core/src/engine/reconsolidation.rs"
  exit 1
fi

echo "$help_output"
echo

run_json "encode (json)" \
  encode "Paris is the capital of France" \
  --namespace test_ns \
  --kind semantic \
  --source cli-e2e \
  --json

run_json "recall (json)" \
  recall "capital of France" \
  --namespace test_ns \
  --top 3 \
  --explain full \
  --json

run_json "inspect missing memory (json failure envelope)" \
  inspect --id 123 --namespace test_ns --json

run_json "explain (json)" \
  explain "how to deploy the service after the last incident?" \
  --namespace test_ns \
  --json

run_json "maintenance repair (json)" \
  maintenance --action repair --namespace test_ns --json

run_json "audit slice (json)" \
  audit --namespace team.alpha --id 21 --recent 1 --json

run_json "doctor (json)" \
  doctor

echo "=== CLI parity artifact completed ==="
