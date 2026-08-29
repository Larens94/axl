#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
cd "$ROOT"
# shellcheck source=env.sh
source "$(dirname "$0")/env.sh"
BIN=(cargo run -p axl-compiler --quiet --)

if [[ -z "${AXL_POSTGRES_URL:-}" ]]; then
  echo "SKIP postgres verify: AXL_POSTGRES_URL not set"
  exit 0
fi

echo "== check postgres boundary =="
"${BIN[@]}" check examples/apps/postgres-boundary.axl --json | jq -e '.protocol == "axl-check/1" and .ok == true'

echo "== eval PostgresSaveDemo + PostgresFindDemo + PostgresQueryDemo =="
"${BIN[@]}" eval examples/apps/postgres-boundary.axl PostgresSaveDemo examples/apps/inputs/postgres-record.json \
  | jq -e '.ok.id == "pg-demo-1"'
"${BIN[@]}" eval examples/apps/postgres-boundary.axl PostgresFindDemo examples/apps/inputs/postgres-record-id.json \
  | jq -e '.ok.id == "pg-demo-1"'
"${BIN[@]}" eval examples/apps/postgres-boundary.axl PostgresQueryDemo examples/apps/inputs/postgres-query.json \
  | jq -e '.ok.total >= 1 and .ok.items[0].id == "pg-demo-1"'

echo "OK postgres gates"
