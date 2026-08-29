#!/usr/bin/env bash
# Sync axl-ui/1 React codegen from the Portal AXL graph into src/generated/.
set -euo pipefail
ROOT="$(cd "$(dirname "$0")/../../.." && pwd)"
HOST="$(cd "$(dirname "$0")/.." && pwd)"
PORTAL="${PORTAL:-examples/apps/portal.axl}"
OUT="$(mktemp -d)"
cleanup() { rm -rf "$OUT"; }
trap cleanup EXIT

cd "$ROOT"
# shellcheck source=/dev/null
source "$ROOT/scripts/env.sh"
cargo run -p axl-compiler --quiet -- experiment "$PORTAL" "$OUT" >/dev/null
mkdir -p "$HOST/src/generated"
cp "$OUT/targets/react/axl_routes.tsx" \
  "$OUT/targets/react/axl_layouts.tsx" \
  "$OUT/targets/react/axl_registry.ts" \
  "$OUT/targets/react/axl_slots.ts" \
  "$HOST/src/generated/"
echo "synced axl-ui/1 → hosts/portal-web/src/generated/"
