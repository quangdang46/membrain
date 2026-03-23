#!/usr/bin/env bash
set -euo pipefail

# e2e_mcp.sh
# Logging-heavy daemon/MCP support artifact for the surfaces that actually exist
# today. This does not pretend there is already a full request-driving MCP
# harness in this shell file; instead it verifies the real daemon binary help
# and captures the runtime flags that are currently exposed.
#
# Usage:
#   bash crates/membrain-daemon/tests/e2e_mcp.sh

DAEMON_CMD=(cargo run -p membrain-daemon --)

echo "=== Membrain daemon / MCP support artifact ==="
echo "Binary: membrain-daemon"
echo

echo "=== daemon help ==="
if ! help_output="$(${DAEMON_CMD[@]} --help 2>&1)"; then
  echo "$help_output"
  echo
  echo "Daemon build/help is currently blocked before daemon runtime checks can run."
  echo "Known blocker path: crates/membrain-core/src/engine/reconsolidation.rs"
  exit 1
fi
echo "$help_output"
echo

for expected in \
  --socket-path \
  --request-concurrency \
  --max-queue-depth \
  --maintenance-interval-secs \
  --maintenance-poll-budget \
  --maintenance-step-delay-ms
 do
  if ! grep -Fq -- "$expected" <<<"$help_output"; then
    echo "missing expected daemon flag: $expected" >&2
    exit 1
  fi
done

echo "=== parity note ==="
echo "This shell artifact currently validates only the real daemon runtime surface."
echo "It does not simulate fake MCP encode/recall/preflight payloads anymore."
echo "For request/response parity, use Rust tests and MCP/JSON-RPC harnesses tied to the actual exposed implementation."
echo

echo "=== daemon / MCP support artifact completed ==="
