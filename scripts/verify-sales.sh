#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
cd "$ROOT"
BIN=(cargo run -p axl-compiler --quiet --)

echo "== check sales (json either order) =="
"${BIN[@]}" check examples/apps/sales.axl --json | jq -e '.protocol == "axl-check/1" and .ok == true and .app == "Vendite"'
"${BIN[@]}" check --json examples/apps/sales.axl | jq -e '.ok == true'

echo "== eval CreaCliente =="
"${BIN[@]}" eval examples/apps/sales.axl CreaCliente examples/apps/inputs/sales-cliente.json | jq -e '.ok.nome == "Carla Verdi"'

echo "== eval ElencaClienti (null input) =="
"${BIN[@]}" eval examples/apps/sales.axl ElencaClienti examples/apps/inputs/unit.json | jq -e '.ok | type == "array"'

echo "== eval PaginaClientiDemoUnit (seeded list) =="
"${BIN[@]}" eval examples/apps/sales.axl PaginaClientiDemoUnit examples/apps/inputs/unit.json | jq -e '.ok.total == 2'

echo "== eval InterrogaClienti =="
"${BIN[@]}" eval examples/apps/sales.axl InterrogaClienti examples/apps/inputs/sales-query.json | jq -e '.ok.total == 0 or .ok.total >= 0'

echo "== eval CercaClientiAttiviDemoUnit (filter stato=attivo subset) =="
"${BIN[@]}" eval examples/apps/sales.axl CercaClientiAttiviDemoUnit examples/apps/inputs/sales-query.json | jq -e '.ok.total == 1 and .ok.items[0].stato == "attivo"'

echo "== eval CercaClientiAttivi (filter via JSON input) =="
"${BIN[@]}" eval examples/apps/sales.axl CercaClientiAttivi examples/apps/inputs/sales-query.json | jq -e '.ok.total == 0 or .ok.total >= 0'

echo "== eval CreaProdotto =="
"${BIN[@]}" eval examples/apps/sales.axl CreaProdotto examples/apps/inputs/sales-prodotto.json | jq -e '.ok.nome == "Tastiera meccanica"'

echo "== eval ElencaProdotti (null input) =="
"${BIN[@]}" eval examples/apps/sales.axl ElencaProdotti examples/apps/inputs/unit.json | jq -e '.ok | type == "array"'

echo "== eval PaginaProdottiDemoUnit (seeded list) =="
"${BIN[@]}" eval examples/apps/sales.axl PaginaProdottiDemoUnit examples/apps/inputs/unit.json | jq -e '.ok.total == 2'

echo "== eval InterrogaProdotti =="
"${BIN[@]}" eval examples/apps/sales.axl InterrogaProdotti examples/apps/inputs/sales-prodotto-query.json | jq -e '.ok.total == 0 or .ok.total >= 0'

echo "== eval CercaProdottiPerSkuDemoUnit (filter sku=LP-001 subset) =="
"${BIN[@]}" eval examples/apps/sales.axl CercaProdottiPerSkuDemoUnit examples/apps/inputs/sales-prodotto-query.json | jq -e '.ok.total == 1 and .ok.items[0].sku == "LP-001"'

echo "== eval CercaProdottiPerSku (filter via JSON input) =="
"${BIN[@]}" eval examples/apps/sales.axl CercaProdottiPerSku examples/apps/inputs/sales-prodotto-query.json | jq -e '.ok.total == 0 or .ok.total >= 0'

echo "== eval CercaProdotto =="
"${BIN[@]}" eval examples/apps/sales.axl CercaProdotto examples/apps/inputs/sales-prodotto-id.json | jq -e '.ok.sku == "LP-001" or .error != null'

echo "== eval CreaPreventivo =="
"${BIN[@]}" eval examples/apps/sales.axl CreaPreventivo examples/apps/inputs/sales-preventivo.json | jq -e '.ok.totale == 135880'

echo "== eval CalcolaTotalePreventivo =="
"${BIN[@]}" eval examples/apps/sales.axl CalcolaTotalePreventivo examples/apps/inputs/sales-preventivo.json | jq -e '. == 135880 or .ok == 135880'

echo "== eval ElencaPreventivi (null input) =="
"${BIN[@]}" eval examples/apps/sales.axl ElencaPreventivi examples/apps/inputs/unit.json | jq -e '.ok | type == "array"'

echo "== eval PaginaPreventiviDemoUnit (seeded list) =="
"${BIN[@]}" eval examples/apps/sales.axl PaginaPreventiviDemoUnit examples/apps/inputs/unit.json | jq -e '.ok.total == 1 and .ok.items[0].totale == 268770'

echo "== eval InterrogaPreventivi =="
"${BIN[@]}" eval examples/apps/sales.axl InterrogaPreventivi examples/apps/inputs/sales-preventivo-query.json | jq -e '.ok.total == 0 or .ok.total >= 0'

echo "== eval CercaPreventiviPerStatoDemoUnit (filter stato=bozza subset) =="
"${BIN[@]}" eval examples/apps/sales.axl CercaPreventiviPerStatoDemoUnit examples/apps/inputs/sales-preventivo-filter-query.json | jq -e '.ok.total == 1 and .ok.items[0].stato == "bozza" and .ok.items[0].id == "preventivo-001"'

echo "== eval CercaPreventiviPerStato (filter via JSON input) =="
"${BIN[@]}" eval examples/apps/sales.axl CercaPreventiviPerStato examples/apps/inputs/sales-preventivo-filter-query.json | jq -e '.ok.total == 0 or .ok.total >= 0'

echo "== eval CercaPreventivo =="
"${BIN[@]}" eval examples/apps/sales.axl CercaPreventivo examples/apps/inputs/sales-preventivo-id.json | jq -e '.ok.id == "preventivo-001" or .error != null'

echo "== eval InviaPreventivoDemoUnit (bozza -> inviato) =="
"${BIN[@]}" eval examples/apps/sales.axl InviaPreventivoDemoUnit examples/apps/inputs/unit.json | jq -e '.ok.stato == "inviato" and .ok.id == "preventivo-001"'

echo "== eval ConfermaPreventivoDemoUnit (inviato -> confermato) =="
"${BIN[@]}" eval examples/apps/sales.axl ConfermaPreventivoDemoUnit examples/apps/inputs/unit.json | jq -e '.ok.stato == "confermato" and .ok.totale == 268770'

echo "== eval AnnullaPreventivoDemoUnit (inviato -> bozza) =="
"${BIN[@]}" eval examples/apps/sales.axl AnnullaPreventivoDemoUnit examples/apps/inputs/unit.json | jq -e '.ok.stato == "bozza" and .ok.id == "preventivo-001"'

echo "== eval InviaPreventivoDoppioDemoUnit (stato_non_inviabile) =="
"${BIN[@]}" eval examples/apps/sales.axl InviaPreventivoDoppioDemoUnit examples/apps/inputs/unit.json | jq -e '.error == "stato_non_inviabile"'

echo "== eval DettaglioPreventivoDemoUnit (seeded detail) =="
"${BIN[@]}" eval examples/apps/sales.axl DettaglioPreventivoDemoUnit examples/apps/inputs/unit.json | jq -e '.ok.id == "preventivo-001" and .ok.totale == 268770'

echo "== eval RenderDettaglioPreventivoDemoUnit (seed + detail for templated render) =="
"${BIN[@]}" eval examples/apps/sales.axl RenderDettaglioPreventivoDemoUnit examples/apps/inputs/unit.json | jq -e '.ok.id == "preventivo-001" and .ok.totale == 268770 and .ok.stato == "bozza"'

echo "== eval DettaglioPreventivoLiveEval (live detail after seed) =="
"${BIN[@]}" eval examples/apps/sales.axl DettaglioPreventivoLiveEval examples/apps/inputs/unit.json | jq -e '.ok.id == "preventivo-001" and .ok.totale == 268770'

echo "== eval InviaPreventivoDemoForm (bozza -> inviato via form flow) =="
"${BIN[@]}" eval examples/apps/sales.axl InviaPreventivoDemoForm examples/apps/inputs/sales-workflow-confirm.json | jq -e '.ok.stato == "inviato" and .ok.id == "preventivo-001"'

echo "== eval ConfermaPreventivoDemoForm (bozza -> confermato via form flow) =="
"${BIN[@]}" eval examples/apps/sales.axl ConfermaPreventivoDemoForm examples/apps/inputs/sales-workflow-confirm.json | jq -e '.ok.stato == "confermato" and .ok.totale == 268770'

echo "== render clienti list (seeded demo) =="
"${BIN[@]}" render examples/apps/sales.axl /clienti/demo examples/apps/inputs/unit.json | grep -q 'cliente-001'

echo "== render prodotti list (seeded demo) =="
"${BIN[@]}" render examples/apps/sales.axl /prodotti/demo examples/apps/inputs/unit.json | grep -q 'prodotto-001'

echo "== render preventivi list (seeded demo) =="
"${BIN[@]}" render examples/apps/sales.axl /preventivi/demo examples/apps/inputs/unit.json | grep -q 'preventivo-001'
"${BIN[@]}" render examples/apps/sales.axl /preventivi/demo examples/apps/inputs/unit.json | grep -q '268770'
"${BIN[@]}" render examples/apps/sales.axl /preventivi/demo examples/apps/inputs/unit.json | grep -q 'href="/preventivi/preventivo-001"'

echo "== render preventivo detail (templated path manifest) =="
"${BIN[@]}" ui examples/apps/sales.axl | jq -e '.uis[0].pages[] | select(.path=="/preventivi/{id}") | .template == "/preventivi/{id}" and .input_source == "path" and .input_name == "id"'

echo "== render preventivo detail (templated path /preventivi/preventivo-001) =="
# render CLI uses a fresh store; eval RenderDettaglioPreventivoDemoUnit proves seed+lookup.
# This gate proves path binding and templated action submit on the detail page.
"${BIN[@]}" render examples/apps/sales.axl /preventivi/preventivo-001 null | grep -q 'preventivi/preventivo-001'
"${BIN[@]}" render examples/apps/sales.axl /preventivi/preventivo-001 null | grep -q 'action="/preventivi/preventivo-001/invia"'
"${BIN[@]}" render examples/apps/sales.axl /preventivi/preventivo-001 null | grep -q 'action="/preventivi/preventivo-001/conferma"'
"${BIN[@]}" render examples/apps/sales.axl /preventivi/preventivo-001 null | grep -q 'name="id" value="preventivo-001"'

echo "== render preventivo detail actions (templated workflow) =="
# render uses a fresh runtime (empty store); prove action wiring via manifest + serve below
"${BIN[@]}" ui examples/apps/sales.axl | jq -e '.uis[0].actions[] | select(.path=="/preventivi/invia") | .submit == "/preventivi/{id}/invia"'
"${BIN[@]}" ui examples/apps/sales.axl | jq -e '.uis[0].actions[] | select(.path=="/preventivi/conferma") | .redirect == "/preventivi/{id}"'

echo "== ui manifest (document) pages, forms and actions =="
"${BIN[@]}" ui examples/apps/sales.axl | jq -e '[.uis[].pages[].path] | index("/clienti")'
"${BIN[@]}" ui examples/apps/sales.axl | jq -e '[.uis[].pages[].path] | index("/clienti/demo")'
"${BIN[@]}" ui examples/apps/sales.axl | jq -e '[.uis[].forms[].path] | index("/clienti/new")'
"${BIN[@]}" ui examples/apps/sales.axl | jq -e '[.uis[].pages[].path] | index("/prodotti")'
"${BIN[@]}" ui examples/apps/sales.axl | jq -e '[.uis[].pages[].path] | index("/prodotti/demo")'
"${BIN[@]}" ui examples/apps/sales.axl | jq -e '[.uis[].forms[].path] | index("/prodotti/new")'
"${BIN[@]}" ui examples/apps/sales.axl | jq -e '[.uis[].pages[].path] | index("/preventivi")'
"${BIN[@]}" ui examples/apps/sales.axl | jq -e '[.uis[].pages[].path] | index("/preventivi/demo")'
"${BIN[@]}" ui examples/apps/sales.axl | jq -e '[.uis[].pages[].path] | index("/preventivi/{id}")'
"${BIN[@]}" ui examples/apps/sales.axl | jq -e '[.uis[].forms[].path] | index("/preventivi/new")'
"${BIN[@]}" ui examples/apps/sales.axl | jq -e '[.uis[].actions[].path] | index("/preventivi/invia")'
"${BIN[@]}" ui examples/apps/sales.axl | jq -e '[.uis[].actions[].path] | index("/preventivi/conferma")'
"${BIN[@]}" ui examples/apps/sales.axl | jq -e '.uis[0].forms[] | select(.path=="/clienti/new") | .submit == "/clienti"'
"${BIN[@]}" ui examples/apps/sales.axl | jq -e '.uis[0].forms[] | select(.path=="/prodotti/new") | .submit == "/prodotti"'
"${BIN[@]}" ui examples/apps/sales.axl | jq -e '.uis[0].forms[] | select(.path=="/preventivi/new") | .submit == "/preventivi"'
"${BIN[@]}" ui examples/apps/sales.axl | jq -e '.uis[0].actions[] | select(.path=="/preventivi/invia") | .submit == "/preventivi/{id}/invia"'
"${BIN[@]}" ui examples/apps/sales.axl | jq -e '.uis[0].actions[] | select(.path=="/preventivi/conferma") | .redirect == "/preventivi/{id}"'

echo "== serve GET form + list smoke =="
PORT=18082
(
  "${BIN[@]}" serve examples/apps/sales.axl "127.0.0.1:${PORT}" &
  PID=$!
  cleanup() { kill "$PID" 2>/dev/null; wait "$PID" 2>/dev/null || true; }
  trap cleanup EXIT
  ready=0
  for _ in $(seq 1 50); do
    if curl -sf --max-time 1 "http://127.0.0.1:${PORT}/clienti/new" >/dev/null 2>&1; then
      ready=1
      break
    fi
    sleep 0.2
  done
  test "$ready" -eq 1
  curl -sf --max-time 2 "http://127.0.0.1:${PORT}/clienti/new" | grep -q '<form method="post" action="/clienti">'
  curl -sf --max-time 2 "http://127.0.0.1:${PORT}/clienti/new" | grep -q 'name="stato"'
  curl -sf --max-time 2 "http://127.0.0.1:${PORT}/clienti/demo" | grep -q 'cliente-001'
  curl -sf --max-time 2 "http://127.0.0.1:${PORT}/prodotti/new" | grep -q '<form method="post" action="/prodotti">'
  curl -sf --max-time 2 "http://127.0.0.1:${PORT}/prodotti/demo" | grep -q 'prodotto-001'
  curl -sf --max-time 2 "http://127.0.0.1:${PORT}/preventivi/demo" | grep -q 'preventivo-001'
  curl -sf --max-time 2 -H 'accept: text/html' "http://127.0.0.1:${PORT}/preventivi/preventivo-001" | grep -q 'preventivo-001'
  curl -sf --max-time 2 -H 'accept: text/html' "http://127.0.0.1:${PORT}/preventivi/preventivo-001" | grep -q '268770'
  curl -sf --max-time 2 -H 'accept: text/html' "http://127.0.0.1:${PORT}/preventivi/preventivo-001" | grep -q 'action="/preventivi/preventivo-001/invia"'
  curl -sf --max-time 2 -X POST "http://127.0.0.1:${PORT}/preventivi" \
    -H 'content-type: application/json' \
    -d @examples/apps/inputs/sales-preventivo.json | jq -e '.ok.id == "preventivo-002" and .ok.stato == "bozza"'
  curl -sf --max-time 2 "http://127.0.0.1:${PORT}/preventivi" | grep -q 'href="/preventivi/preventivo-002"'
  curl -sf --max-time 2 -H 'accept: text/html' "http://127.0.0.1:${PORT}/preventivi/preventivo-002" | grep -q 'preventivo-002'
  curl -sf --max-time 2 -H 'accept: text/html' "http://127.0.0.1:${PORT}/preventivi/preventivo-002" | grep -q 'action="/preventivi/preventivo-002/invia"'
  curl -sf --max-time 2 -H 'accept: text/html' "http://127.0.0.1:${PORT}/preventivi/preventivo-002" | grep -q 'action="/preventivi/preventivo-002/conferma"'
  INVIA_HEADERS=$(curl -s -D - -o /dev/null --max-time 2 -X POST "http://127.0.0.1:${PORT}/preventivi/preventivo-002/invia" \
    -H 'content-type: application/x-www-form-urlencoded' \
    -H 'accept: text/html' \
    --data-urlencode 'id=preventivo-002')
  echo "$INVIA_HEADERS" | grep -qi '^HTTP/.* 303'
  echo "$INVIA_HEADERS" | grep -qi '^location: /preventivi/preventivo-002'
  curl -sf --max-time 2 -H 'accept: text/html' "http://127.0.0.1:${PORT}/preventivi/preventivo-002" | grep -q 'inviato'
  CONFERMA_HEADERS=$(curl -s -D - -o /dev/null --max-time 2 -X POST "http://127.0.0.1:${PORT}/preventivi/preventivo-002/conferma" \
    -H 'content-type: application/x-www-form-urlencoded' \
    -H 'accept: text/html' \
    --data-urlencode 'id=preventivo-002')
  echo "$CONFERMA_HEADERS" | grep -qi '^HTTP/.* 303'
  echo "$CONFERMA_HEADERS" | grep -qi '^location: /preventivi/preventivo-002'
  curl -sf --max-time 2 -H 'accept: text/html' "http://127.0.0.1:${PORT}/preventivi/preventivo-002" | grep -q 'confermato'
  UNIQUE_NOME="Form Smoke Client $(date +%s)"
  POST_HEADERS=$(curl -s -D - -o /dev/null --max-time 2 -X POST "http://127.0.0.1:${PORT}/clienti" \
    -H 'content-type: application/x-www-form-urlencoded' \
    -H 'accept: text/html' \
    --data-urlencode 'id=cliente-form-smoke' \
    --data-urlencode "nome=${UNIQUE_NOME}" \
    --data-urlencode 'email=form@example.com' \
    --data-urlencode 'budget=5000' \
    --data-urlencode 'stato=attivo')
  echo "$POST_HEADERS" | grep -qi '^HTTP/.* 303'
  echo "$POST_HEADERS" | grep -qi '^location: /clienti'
  curl -sf --max-time 2 "http://127.0.0.1:${PORT}/clienti" | grep -qF "$UNIQUE_NOME"
  curl -sf --max-time 2 -X POST "http://127.0.0.1:${PORT}/clienti/query" \
    -H 'content-type: application/json' \
    -d '{"order_by":"nome","direction":"asc","limit":10,"offset":0}' \
    | jq -e --arg n "$UNIQUE_NOME" '[.ok.items[].nome] | index($n) != null'
  curl -sf --max-time 2 -X POST "http://127.0.0.1:${PORT}/preventivi" \
    -H 'content-type: application/json' \
    -d @examples/apps/inputs/sales-preventivo.json | jq -e '.ok.id == "preventivo-002" and .ok.stato == "bozza"'
  curl -sf --max-time 2 -X POST "http://127.0.0.1:${PORT}/preventivi/preventivo-002/invia" | jq -e '.ok.stato == "inviato"'
  curl -sf --max-time 2 -X POST "http://127.0.0.1:${PORT}/preventivi/preventivo-002/conferma" | jq -e '.ok.stato == "confermato"'
  curl -sf --max-time 2 -X POST "http://127.0.0.1:${PORT}/preventivi/query" \
    -H 'content-type: application/json' \
    -d '{"order_by":"id","direction":"asc","limit":10,"offset":0}' \
    | jq -e '(.ok.items | map(select(.id=="preventivo-002"))[0].stato) == "confermato"'
  curl -sf --max-time 2 "http://127.0.0.1:${PORT}/preventivi/preventivo-002" | jq -e '.ok.stato == "confermato" and .ok.totale == 135880'
  curl -sf --max-time 2 -X POST "http://127.0.0.1:${PORT}/clienti/search" \
    -H 'content-type: application/json' \
    -d @examples/apps/inputs/sales-query.json | jq -e '.ok.total >= 0'
  curl -sf --max-time 2 -X POST "http://127.0.0.1:${PORT}/prodotti/search" \
    -H 'content-type: application/json' \
    -d @examples/apps/inputs/sales-prodotto-query.json | jq -e '.ok.total >= 0'
  curl -sf --max-time 2 -X POST "http://127.0.0.1:${PORT}/preventivi/search" \
    -H 'content-type: application/json' \
    -d @examples/apps/inputs/sales-preventivo-filter-query.json | jq -e '.ok.total >= 0'
  curl -sf --max-time 2 "http://127.0.0.1:${PORT}/clienti" | grep -q 'href="/preventivi/new"'
)

echo "== durable sqlite cross-process (eval) =="
mkdir -p ./build
rm -f ./build/vendite.db
"${BIN[@]}" eval examples/apps/sales.axl CreaClienteSqlite examples/apps/inputs/sales-cliente.json | jq -e '.ok.nome == "Carla Verdi"'
"${BIN[@]}" eval examples/apps/sales.axl CercaClienteSqlite examples/apps/inputs/sales-cliente-durable-id.json | jq -e '.ok.nome == "Carla Verdi"'

"${BIN[@]}" eval examples/apps/sales.axl CreaPreventivoSqlite examples/apps/inputs/sales-preventivo.json | jq -e '.ok.totale == 135880'
"${BIN[@]}" eval examples/apps/sales.axl CercaPreventivoSqlite examples/apps/inputs/sales-preventivo-durable-id.json | jq -e '.ok.stato == "bozza" and .ok.totale == 135880'
"${BIN[@]}" eval examples/apps/sales.axl InviaPreventivoSqlite examples/apps/inputs/sales-preventivo-durable-id.json | jq -e '.ok.stato == "inviato"'
"${BIN[@]}" eval examples/apps/sales.axl CercaPreventivoSqlite examples/apps/inputs/sales-preventivo-durable-id.json | jq -e '.ok.stato == "inviato"'

echo "== durable sqlite cross-process (HTTP restart) =="
DPORT=18085
(
  rm -f ./build/vendite.db
  "${BIN[@]}" serve examples/apps/sales.axl "127.0.0.1:${DPORT}" &
  DPID=$!
  cleanup() { kill "$DPID" 2>/dev/null; wait "$DPID" 2>/dev/null || true; }
  trap cleanup EXIT
  ready=0
  for _ in $(seq 1 50); do
    if curl -sf --max-time 1 "http://127.0.0.1:${DPORT}/clienti" >/dev/null 2>&1; then
      ready=1
      break
    fi
    sleep 0.2
  done
  test "$ready" -eq 1
  curl -sf --max-time 2 -X POST "http://127.0.0.1:${DPORT}/preventivi/durable" \
    -H 'content-type: application/json' \
    -d @examples/apps/inputs/sales-preventivo.json | jq -e '.ok.id == "preventivo-002"'
  kill "$DPID" 2>/dev/null; wait "$DPID" 2>/dev/null || true
  "${BIN[@]}" serve examples/apps/sales.axl "127.0.0.1:${DPORT}" &
  DPID=$!
  ready=0
  for _ in $(seq 1 50); do
    if curl -sf --max-time 1 "http://127.0.0.1:${DPORT}/preventivi/durable/preventivo-002" >/dev/null 2>&1; then
      ready=1
      break
    fi
    sleep 0.2
  done
  test "$ready" -eq 1
  curl -sf --max-time 2 "http://127.0.0.1:${DPORT}/preventivi/durable/preventivo-002" | jq -e '.ok.stato == "bozza" and .ok.totale == 135880'
  curl -sf --max-time 2 -X POST "http://127.0.0.1:${DPORT}/preventivi/durable/preventivo-002/invia" | jq -e '.ok.stato == "inviato"'
  curl -sf --max-time 2 -X POST "http://127.0.0.1:${DPORT}/preventivi/durable/preventivo-002/conferma" | jq -e '.ok.stato == "confermato"'
  curl -sf --max-time 2 "http://127.0.0.1:${DPORT}/preventivi/durable/preventivo-002" | jq -e '.ok.stato == "confermato"'
)

echo "OK: verify-sales"
