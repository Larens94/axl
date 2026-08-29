#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
cd "$ROOT"
# shellcheck source=env.sh
source "$(dirname "$0")/env.sh"
BIN=(cargo run -p axl-compiler --quiet --)
TMP=./build/verify-ui-modal.html
mkdir -p ./build

echo "== check modal boundary =="
"${BIN[@]}" check examples/apps/modal-boundary.axl --json | jq -e '.ok == true'

echo "== ui manifest lists modals =="
"${BIN[@]}" ui examples/apps/modal-boundary.axl | jq -e '
  .protocol == "axl-ui/1"
  and (.uis[0].modals | length) == 1
  and .uis[0].modals[0].path == "/clienti/{id}/confirm"
  and .uis[0].modals[0].on == "/clienti"
'

echo "== render modal overlay =="
"${BIN[@]}" render examples/apps/modal-boundary.axl /clienti/cliente-modal-1/confirm null >"$TMP"
grep -q 'class="modal-panel"' "$TMP"
grep -q 'Modal Demo' "$TMP"
grep -q 'role="dialog"' "$TMP"

echo "== list page still renders =="
"${BIN[@]}" render examples/apps/modal-boundary.axl /clienti null >"$TMP"
grep -q 'Modal Demo' "$TMP"
if grep -q 'class="modal-panel"' "$TMP"; then
  echo "list page must not render modal overlay" >&2
  exit 1
fi

echo "== invalid modal path diagnostic =="
set +o pipefail
"${BIN[@]}" check examples/invalid/ui-modal.axl --json \
  | jq -e '.ok == false and any(.diagnostics[]; .code == "AXL-U902")'
set -o pipefail

echo "OK modal gates"
