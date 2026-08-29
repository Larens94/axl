#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
cd "$ROOT"
# shellcheck source=env.sh
source "$(dirname "$0")/env.sh"
BIN=(cargo run -p axl-compiler --quiet --)

echo "== check sql pushdown boundary =="
mkdir -p ./build
rm -f ./build/sql-pushdown-boundary.db
"${BIN[@]}" check examples/apps/sql-pushdown-boundary.axl --json | jq -e '.ok == true'

echo "== seed + query with SQL pushdown =="
"${BIN[@]}" eval examples/apps/sql-pushdown-boundary.axl SqlPushdownSaveDemo examples/apps/inputs/sql-pushdown-income.json
"${BIN[@]}" eval examples/apps/sql-pushdown-boundary.axl SqlPushdownSaveDemo examples/apps/inputs/sql-pushdown-expense.json
"${BIN[@]}" eval examples/apps/sql-pushdown-boundary.axl SqlPushdownQueryDemo examples/apps/inputs/sql-pushdown-query.json \
  | jq -e '.ok.total == 1 and .ok.items[0].kind == "income"'

echo "OK sql pushdown gates"
