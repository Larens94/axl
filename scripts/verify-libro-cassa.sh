#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
cd "$ROOT"
BIN=(cargo run -p axl-compiler --quiet --)

echo "== check ledger (json either order) =="
"${BIN[@]}" check examples/apps/ledger.axl --json | jq -e '.protocol == "axl-check/1" and .ok == true and .app == "LibroCassa"'
"${BIN[@]}" check --json examples/apps/ledger.axl | jq -e '.ok == true'

echo "== eval saldo =="
SALDO="$("${BIN[@]}" eval examples/apps/ledger.axl CalcolaSaldo examples/apps/inputs/ledger-saldo.json)"
test "$SALDO" = "108000"

echo "== eval query memory =="
"${BIN[@]}" eval examples/apps/ledger.axl InterrogaVoci examples/apps/inputs/ledger-query.json | jq -e '.ok.total == 0 or .ok.total >= 0'

echo "== render saldo =="
"${BIN[@]}" render examples/apps/ledger.axl /saldo examples/apps/inputs/ledger-saldo.json | grep -q '108000'

echo "== render voci (ui page, seeded demo) =="
"${BIN[@]}" render examples/apps/ledger.axl /voci examples/apps/inputs/ledger-voci-demo.json | grep -q 'voce-001'

echo "== eval PaginaVociDemoUnit (unit input, inline make) =="
"${BIN[@]}" eval examples/apps/ledger.axl PaginaVociDemoUnit examples/apps/inputs/unit.json | jq -e '.ok.total == 2'

echo "== render voci/demo (unit input, no JSON pair) =="
"${BIN[@]}" render examples/apps/ledger.axl /voci/demo examples/apps/inputs/unit.json | grep -q 'voce-001'

echo "== ui manifest pages =="
"${BIN[@]}" ui examples/apps/ledger.axl | jq -e '[.uis[].pages[].path] | index("/voci") and index("/saldo") and index("/voci/demo")'

echo "OK: verify-libro-cassa"
