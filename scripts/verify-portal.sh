#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
cd "$ROOT"
SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
export PORTAL="${PORTAL:-examples/apps/portal.axl}"

"$SCRIPT_DIR/verify-portal-auth.sh"
"$SCRIPT_DIR/verify-portal-sales.sh"

echo "OK portal"
