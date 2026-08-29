#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
cd "$ROOT"
# shellcheck source=env.sh
source "$(dirname "$0")/env.sh"
BIN=(cargo run -p axl-compiler --quiet --)
TMP=./build/verify-ui-drawer.html
mkdir -p ./build

echo "== check drawer boundary =="
"${BIN[@]}" check examples/apps/drawer-boundary.axl --json | jq -e '.ok == true'

echo "== ui manifest lists drawers =="
"${BIN[@]}" ui examples/apps/drawer-boundary.axl | jq -e '
  .protocol == "axl-ui/1"
  and (.uis[0].drawers | length) == 1
  and .uis[0].drawers[0].path == "/clienti/{id}"
  and .uis[0].drawers[0].on == "/clienti"
'

echo "== render drawer overlay =="
"${BIN[@]}" render examples/apps/drawer-boundary.axl /clienti/cliente-drawer-1 null >"$TMP"
grep -q 'class="drawer-panel"' "$TMP"
grep -q 'Drawer Demo' "$TMP"
grep -q 'role="dialog"' "$TMP"

echo "== list page still renders =="
"${BIN[@]}" render examples/apps/drawer-boundary.axl /clienti null >"$TMP"
grep -q 'Drawer Demo' "$TMP"
if grep -q 'role="dialog"' "$TMP"; then
  echo "list page must not render drawer overlay" >&2
  exit 1
fi

echo "== invalid drawer path diagnostic =="
set +o pipefail
"${BIN[@]}" check examples/invalid/ui-drawer.axl --json \
  | jq -e '.ok == false and any(.diagnostics[]; .code == "AXL-U902")'
set -o pipefail

echo "OK drawer gates"
