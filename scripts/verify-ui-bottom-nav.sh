#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
cd "$ROOT"
# shellcheck source=env.sh
source "$(dirname "$0")/env.sh"
BIN=(cargo run -p axl-compiler --quiet --)
TMP=./build/verify-ui-bottom-nav.html
mkdir -p ./build

echo "== check bottom-nav boundary =="
"${BIN[@]}" check examples/apps/bottom-nav-boundary.axl --json | jq -e '.ok == true'

echo "== ui manifest declares shell =="
"${BIN[@]}" ui examples/apps/bottom-nav-boundary.axl | jq -e '
  .protocol == "axl-ui/1"
  and .shell.desktop == "sidebar"
  and .shell.mobile == "bottom-nav"
'

echo "== render emits bottom-nav =="
"${BIN[@]}" render examples/apps/bottom-nav-boundary.axl /clienti null >"$TMP"
grep -q 'class="bottom-nav"' "$TMP"
grep -q 'aria-label="Navigazione principale"' "$TMP"
grep -q 'href="/ordini"' "$TMP"
grep -q 'class="bottom-nav-link active"' "$TMP"
grep -q 'class="sidebar"' "$TMP"

echo "OK bottom-nav gates"
