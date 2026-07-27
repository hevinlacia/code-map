#!/usr/bin/env bash
# Thin wrapper around the Python agent CLI.
# Usage:
#   code-map-query.sh <term>              # backward compat: query <term>
#   code-map-query.sh query <term> [--json]
#   code-map-query.sh neighbors <entity> [--json]
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PY="$SCRIPT_DIR/code_map_query.py"

if [ "$#" -gt 0 ] && { [ "$1" = "query" ] || [ "$1" = "neighbors" ]; }; then
  exec python3 "$PY" "$@"
else
  exec python3 "$PY" query "$@"
fi
