#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
cd "$ROOT"
# shellcheck source=env.sh
source "$(dirname "$0")/env.sh"
BIN=(cargo run -p axl-compiler --quiet --)

echo "== check form validation boundary =="
"${BIN[@]}" check examples/apps/form-validation-boundary.axl --json | jq -e '.ok == true'

echo "== portal still checks with clienti drawer =="
"${BIN[@]}" check examples/apps/portal.axl --json | jq -e '.ok == true'
"${BIN[@]}" ui examples/apps/portal.axl | jq -e '
  any(.uis[].drawers[]?; .path == "/clienti/{id}")
'

echo "== HTTP validation HTML unit =="
cargo test -p axl-compiler form_post_renders_validation_html --quiet

echo "OK form validation gates"
