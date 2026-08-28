#!/usr/bin/env bash
set -euo pipefail
ROOT="$(cd "$(dirname "$0")/.." && pwd)"
cd "$ROOT"
# shellcheck source=env.sh
source "$(dirname "$0")/env.sh"
PORTAL="${PORTAL:-examples/apps/portal.axl}"
PORT="${PORT:-8080}"
BIND="127.0.0.1:${PORT}"

if ! command -v lsof >/dev/null 2>&1; then
  :
elif lsof -nP -iTCP:"${PORT}" -sTCP:LISTEN >/dev/null 2>&1; then
  echo "error: port ${PORT} is already in use." >&2
  echo "hint: stop the other server (Ctrl+C) or run PORT=8081 $0" >&2
  lsof -nP -iTCP:"${PORT}" -sTCP:LISTEN 2>/dev/null || true
  exit 48
fi

echo "AXL portal: http://${BIND}/  (${PORTAL})" >&2
exec cargo run -p axl-compiler -- serve "$PORTAL" "$BIND"
