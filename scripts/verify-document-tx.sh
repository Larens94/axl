#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
cd "$ROOT"
# shellcheck source=env.sh
source "$(dirname "$0")/env.sh"
BIN=(cargo run -p axl-compiler --quiet --)

echo "== check document tx boundary =="
mkdir -p ./build
rm -f ./build/document-tx-boundary.json ./build/document-tx-boundary.json.migrations.json
"${BIN[@]}" check examples/apps/document-tx-boundary.axl --json | jq -e '.protocol == "axl-check/1" and .ok == true'

echo "== eval DocumentTxCommitDemo (persisted) =="
"${BIN[@]}" eval examples/apps/document-tx-boundary.axl DocumentTxCommitDemo examples/apps/inputs/document-tx-pair.json \
  | jq -e '.ok.id == "doc-tx-c2"'
"${BIN[@]}" eval examples/apps/document-tx-boundary.axl DocumentFindDemo examples/apps/inputs/document-tx-c1-id.json \
  | jq -e '.ok.value == "one"'

echo "== eval DocumentTxRollbackDemo (writes hidden) =="
"${BIN[@]}" eval examples/apps/document-tx-boundary.axl DocumentTxRollbackDemo examples/apps/inputs/document-tx-rollback-pair.json \
  | jq -e '.ok == null'
"${BIN[@]}" eval examples/apps/document-tx-boundary.axl DocumentFindDemo examples/apps/inputs/document-tx-r1-id.json \
  | jq -e '.error == "not_found"'

echo "== eval DocumentMigrateUpDemo + status sidecar =="
"${BIN[@]}" eval examples/apps/document-tx-boundary.axl DocumentMigrateUpDemo examples/apps/inputs/document-migrate-v1.json \
  | jq -e '.ok == "doc-migrate-v1"'
"${BIN[@]}" eval examples/apps/document-tx-boundary.axl DocumentMigrateStatusDemo examples/apps/inputs/unit.json \
  | jq -e '.ok == "doc-migrate-v1"'
test -f ./build/document-tx-boundary.json.migrations.json

echo "OK document tx/migrate gates"
