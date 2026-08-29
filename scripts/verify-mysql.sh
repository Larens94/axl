#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
cd "$ROOT"
# shellcheck source=env.sh
source "$(dirname "$0")/env.sh"
BIN=(cargo run -p axl-compiler --quiet --)

if [[ -z "${AXL_MYSQL_URL:-}" ]]; then
  echo "SKIP mysql verify: AXL_MYSQL_URL not set"
  exit 0
fi

echo "== check mysql boundary =="
"${BIN[@]}" check examples/apps/mysql-boundary.axl --json | jq -e '.protocol == "axl-check/1" and .ok == true'

echo "== eval MysqlSaveDemo + MysqlQueryDemo =="
"${BIN[@]}" eval examples/apps/mysql-boundary.axl MysqlSaveDemo examples/apps/inputs/postgres-record.json \
  | jq -e '.ok.id == "pg-demo-1"'
"${BIN[@]}" eval examples/apps/mysql-boundary.axl MysqlQueryDemo examples/apps/inputs/postgres-query.json \
  | jq -e '.ok.total >= 1'

echo "== eval MysqlTxCommitDemo + MysqlMigrateUpDemo =="
"${BIN[@]}" eval examples/apps/mysql-boundary.axl MysqlTxCommitDemo examples/apps/inputs/postgres-tx-pair.json \
  | jq -e '.ok.id == "pg-boundary-tx-c2"'
"${BIN[@]}" eval examples/apps/mysql-boundary.axl MysqlMigrateUpDemo examples/apps/inputs/mysql-migrate-v1.json \
  | jq -e '.ok == "mysql-boundary-migrate-v1"'

echo "OK mysql gates"
