#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
cd "$ROOT"
# shellcheck source=env.sh
source "$(dirname "$0")/env.sh"
BIN=(cargo run -p axl-compiler --quiet --)
TMP=./build/verify-ui-kpi.html
mkdir -p ./build

echo "== check kpi registry boundary =="
"${BIN[@]}" check examples/apps/kpi-registry-boundary.axl --json | jq -e '.ok == true'

echo "== ui manifest lists kpis and slots =="
"${BIN[@]}" ui examples/apps/kpi-registry-boundary.axl | jq -e '
  .protocol == "axl-ui/1"
  and (.kit.slots | length) >= 9
  and (.uis[0].slots | length) == 9
  and (.uis[0].pages[0].kpis | length) == 2
  and .uis[0].pages[0].kpis[0].field == "totale_utenti"
'

echo "== render kpi dashboard =="
"${BIN[@]}" render examples/apps/kpi-registry-boundary.axl / null >"$TMP"
grep -q 'data-slot="kpi.card"' "$TMP"
grep -q 'Utenti registrati' "$TMP"
grep -q 'Ordini aperti' "$TMP"
grep -q '>12<' "$TMP"
grep -q '>3<' "$TMP"

echo "== invalid kpi field diagnostic =="
set +o pipefail
"${BIN[@]}" check examples/invalid/ui-kpi.axl --json \
  | jq -e '.ok == false and any(.diagnostics[]; .code == "AXL-U922")'
set -o pipefail

echo "OK kpi/registry gates"
