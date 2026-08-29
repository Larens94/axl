#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
cd "$ROOT"
# shellcheck source=env.sh
source "$(dirname "$0")/env.sh"
BIN=(cargo run -p axl-compiler --quiet --)
TMP=./build/verify-ui-chart.html
mkdir -p ./build

echo "== check chart boundary =="
"${BIN[@]}" check examples/apps/chart-boundary.axl --json | jq -e '.ok == true'

echo "== ui manifest lists charts =="
"${BIN[@]}" ui examples/apps/chart-boundary.axl | jq -e '
  .protocol == "axl-ui/1"
  and (.kit.slots | index("chart.bar") != null)
  and (.uis[0].pages[0].charts | length) == 1
  and .uis[0].pages[0].charts[0].field == "serie"
'

echo "== render bar chart =="
"${BIN[@]}" render examples/apps/chart-boundary.axl / null >"$TMP"
grep -q 'data-slot="chart.bar"' "$TMP"
grep -q 'Andamento mensile' "$TMP"
grep -q 'class="chart-bar"' "$TMP"
grep -q 'Gen' "$TMP"
grep -q 'Feb' "$TMP"

echo "== invalid chart field diagnostic =="
set +o pipefail
"${BIN[@]}" check examples/invalid/ui-chart.axl --json \
  | jq -e '.ok == false and any(.diagnostics[]; .code == "AXL-U925")'
set -o pipefail

echo "OK chart gates"
