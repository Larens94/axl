#!/usr/bin/env bash
# Local portal dev: prerequisites check + start AXL serve (HTML/API).
set -euo pipefail
ROOT="$(cd "$(dirname "$0")/.." && pwd)"
cd "$ROOT"
# shellcheck source=env.sh
source "$(dirname "$0")/env.sh"

echo "== AXL local dev ==" >&2
echo "repo:  $ROOT" >&2
echo "branch: $(git rev-parse --abbrev-ref HEAD) @ $(git rev-parse --short HEAD)" >&2

if ! command -v node >/dev/null 2>&1; then
  echo "note: node not found — HTML-only mode via demo-portal.sh; install Node for hosts/portal-web" >&2
fi

echo "== build axl-compiler ==" >&2
cargo build -p axl-compiler

echo "" >&2
echo "Starting portal (Ctrl+C to stop)." >&2
echo "  HTML/API:  http://127.0.0.1:${PORT:-8080}/" >&2
echo "  Login:     admin@example.com / admin123" >&2
echo "  Demo gate: http://127.0.0.1:${PORT:-8080}/clienti/demo" >&2
echo "" >&2
echo "Optional React host (second terminal):" >&2
echo "  cd hosts/portal-web && npm install" >&2
echo "  AXL_PROXY_TARGET=http://127.0.0.1:${PORT:-8080} npm run dev" >&2
echo "" >&2

exec "$(dirname "$0")/demo-portal.sh"
