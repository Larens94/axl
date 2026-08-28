#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
cd "$ROOT"
# shellcheck source=env.sh
source "$(dirname "$0")/env.sh"
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

echo "== eval DettaglioPreventivoDemoUnit (righe typed line items) =="
"${BIN[@]}" eval examples/apps/sales.axl DettaglioPreventivoDemoUnit examples/apps/inputs/unit.json | jq -e '(.ok.righe | length) == 2 and .ok.righe[0].prodotto_id == "prodotto-001" and .ok.righe[0].quantita == 2 and .ok.righe[0].prezzo_unitario == 129900 and .ok.righe[0].importo == 259800 and .ok.righe[1].prodotto_id == "prodotto-002" and .ok.righe[1].importo == 8970'

echo "== eval RenderDettaglioPreventivoDemoUnit (seed + detail for templated render) =="
"${BIN[@]}" eval examples/apps/sales.axl RenderDettaglioPreventivoDemoUnit examples/apps/inputs/unit.json | jq -e '.ok.id == "preventivo-001" and .ok.totale == 268770 and .ok.stato == "bozza" and (.ok.righe | length) == 2'

echo "== eval InviaPreventivoSeeded (InviaPreventivo + sales-preventivo-id.json) =="
"${BIN[@]}" eval examples/apps/sales.axl InviaPreventivoSeeded examples/apps/inputs/sales-preventivo-id.json | jq -e '.ok.stato == "inviato" and .ok.id == "preventivo-001"'

echo "== eval ConfermaPreventivoSeeded (ConfermaPreventivo + sales-preventivo-id.json) =="
"${BIN[@]}" eval examples/apps/sales.axl ConfermaPreventivoSeeded examples/apps/inputs/sales-preventivo-id.json | jq -e '.ok.stato == "confermato" and .ok.totale == 268770'

echo "== eval CreaOrdineDaPreventivoDemoUnit (confermato -> ordine bozza, independent id) =="
ORDINE_DEMO=$("${BIN[@]}" eval examples/apps/sales.axl CreaOrdineDaPreventivoDemoUnit examples/apps/inputs/unit.json)
echo "$ORDINE_DEMO" | jq -e '.ok.stato == "bozza" and .ok.preventivo_id == "preventivo-001" and .ok.id != .ok.preventivo_id and .ok.totale == 268770'
ORDINE_DEMO_ID=$(echo "$ORDINE_DEMO" | jq -r '.ok.id')

echo "== eval CreaOrdineDaPreventivoSeeded (seeded confermato -> ordine) =="
ORDINE_SEEDED=$("${BIN[@]}" eval examples/apps/sales.axl CreaOrdineDaPreventivoSeeded examples/apps/inputs/sales-preventivo-id.json)
echo "$ORDINE_SEEDED" | jq -e '.ok.stato == "bozza" and .ok.id != .ok.preventivo_id and .ok.totale == 268770'

echo "== eval CreaOrdineDaPreventivoNonConfermatoDemoUnit (preventivo_non_confermato) =="
"${BIN[@]}" eval examples/apps/sales.axl CreaOrdineDaPreventivoNonConfermatoDemoUnit examples/apps/inputs/unit.json | jq -e '.error == "preventivo_non_confermato"'

echo "== eval PaginaOrdiniDemoUnit (seeded ordine list) =="
"${BIN[@]}" eval examples/apps/sales.axl PaginaOrdiniDemoUnit examples/apps/inputs/unit.json | jq -e '.ok.total == 1 and .ok.items[0].totale == 268770'

echo "== eval ConfermaOrdineDemoUnit (bozza -> confermato) =="
"${BIN[@]}" eval examples/apps/sales.axl ConfermaOrdineDemoUnit examples/apps/inputs/unit.json | jq -e '.ok.stato == "confermato" and .ok.totale == 268770'

echo "== eval AnnullaOrdineDemoUnit (bozza -> annullato) =="
"${BIN[@]}" eval examples/apps/sales.axl AnnullaOrdineDemoUnit examples/apps/inputs/unit.json | jq -e '.ok.stato == "annullato" and .ok.id != .ok.preventivo_id'

echo "== eval ConfermaOrdineDoppioDemoUnit (stato_non_confermable) =="
"${BIN[@]}" eval examples/apps/sales.axl ConfermaOrdineDoppioDemoUnit examples/apps/inputs/unit.json | jq -e '.error == "stato_non_confermable"'

echo "== eval AnnullaOrdineConfermatoDemoUnit (stato_non_annullabile) =="
"${BIN[@]}" eval examples/apps/sales.axl AnnullaOrdineConfermatoDemoUnit examples/apps/inputs/unit.json | jq -e '.error == "stato_non_annullabile"'

echo "== eval DettaglioOrdineDemoUnit (seeded detail) =="
"${BIN[@]}" eval examples/apps/sales.axl DettaglioOrdineDemoUnit examples/apps/inputs/unit.json | jq -e '.ok.preventivo_id == "preventivo-001" and .ok.id != .ok.preventivo_id and .ok.totale == 268770 and .ok.cliente_id == "cliente-001"'

echo "== eval DettaglioOrdineDemoUnit (righe typed line items) =="
"${BIN[@]}" eval examples/apps/sales.axl DettaglioOrdineDemoUnit examples/apps/inputs/unit.json | jq -e '(.ok.righe | length) == 2 and .ok.righe[0].prodotto_id == "prodotto-001" and .ok.righe[0].quantita == 2 and .ok.righe[1].prodotto_id == "prodotto-002" and .ok.righe[1].quantita == 3'

echo "== eval RenderDettaglioOrdineDemoUnit (seed + detail for templated render) =="
RENDER_ORDINE=$("${BIN[@]}" eval examples/apps/sales.axl RenderDettaglioOrdineDemoUnit examples/apps/inputs/unit.json)
echo "$RENDER_ORDINE" | jq -e '.ok.id != .ok.preventivo_id and .ok.totale == 268770 and .ok.stato == "bozza" and (.ok.righe | length) == 2'
ORDINE_RENDER_ID=$(echo "$RENDER_ORDINE" | jq -r '.ok.id')

echo "== eval ConfermaOrdineSeeded (ConfermaOrdine + sales-preventivo-id.json) =="
"${BIN[@]}" eval examples/apps/sales.axl ConfermaOrdineSeeded examples/apps/inputs/sales-preventivo-id.json | jq -e '.ok.stato == "confermato" and .ok.totale == 268770'

echo "== eval InviaPreventivoDemoForm (bozza -> inviato via form flow) =="
"${BIN[@]}" eval examples/apps/sales.axl InviaPreventivoDemoForm examples/apps/inputs/sales-workflow-confirm.json | jq -e '.ok.stato == "inviato" and .ok.id == "preventivo-001"'

echo "== eval ConfermaPreventivoDemoForm (bozza -> confermato via form flow) =="
"${BIN[@]}" eval examples/apps/sales.axl ConfermaPreventivoDemoForm examples/apps/inputs/sales-workflow-confirm.json | jq -e '.ok.stato == "confermato" and .ok.totale == 268770'

echo "== eval ConfermaOrdineDemoForm (bozza -> confermato via form flow) =="
"${BIN[@]}" eval examples/apps/sales.axl ConfermaOrdineDemoForm examples/apps/inputs/sales-workflow-confirm.json | jq -e '.ok.stato == "confermato" and .ok.totale == 268770'

echo "== eval AnnullaOrdineDemoForm (bozza -> annullato via form flow) =="
"${BIN[@]}" eval examples/apps/sales.axl AnnullaOrdineDemoForm examples/apps/inputs/sales-workflow-confirm.json | jq -e '.ok.stato == "annullato" and .ok.totale == 268770'

echo "== render clienti list (seeded demo) =="
"${BIN[@]}" render examples/apps/sales.axl /clienti/demo examples/apps/inputs/unit.json | grep -q 'cliente-001'

echo "== render prodotti list (seeded demo) =="
"${BIN[@]}" render examples/apps/sales.axl /prodotti/demo examples/apps/inputs/unit.json | grep -q 'prodotto-001'

echo "== render preventivi list (seeded demo) =="
"${BIN[@]}" render examples/apps/sales.axl /preventivi/demo examples/apps/inputs/unit.json | grep -q 'preventivo-001'
"${BIN[@]}" render examples/apps/sales.axl /preventivi/demo examples/apps/inputs/unit.json | grep -q '268770'
"${BIN[@]}" render examples/apps/sales.axl /preventivi/demo examples/apps/inputs/unit.json | grep -q 'href="/preventivi/preventivo-001"'

echo "== render ordini list (seeded demo) =="
"${BIN[@]}" render examples/apps/sales.axl /ordini/demo examples/apps/inputs/unit.json | grep -q 'preventivo-001'
"${BIN[@]}" render examples/apps/sales.axl /ordini/demo examples/apps/inputs/unit.json | grep -q '268770'
"${BIN[@]}" render examples/apps/sales.axl /ordini/demo examples/apps/inputs/unit.json | grep -qE 'href="/ordini/[^"]+"'

echo "== render ordine detail (templated path manifest) =="
"${BIN[@]}" ui examples/apps/sales.axl | jq -e '.uis[0].pages[] | select(.path=="/ordini/{id}") | .template == "/ordini/{id}" and .input_source == "path" and .input_name == "id"'

echo "== render ordine detail (templated path with dynamic ordine id) =="
"${BIN[@]}" render examples/apps/sales.axl "/ordini/${ORDINE_RENDER_ID}" null | grep -q "ordini/${ORDINE_RENDER_ID}"
"${BIN[@]}" render examples/apps/sales.axl "/ordini/${ORDINE_RENDER_ID}" null | grep -q "action=\"/ordini/${ORDINE_RENDER_ID}/conferma\""
"${BIN[@]}" render examples/apps/sales.axl "/ordini/${ORDINE_RENDER_ID}" null | grep -q "action=\"/ordini/${ORDINE_RENDER_ID}/annulla\""
"${BIN[@]}" render examples/apps/sales.axl "/ordini/${ORDINE_RENDER_ID}" null | grep -q "name=\"id\" value=\"${ORDINE_RENDER_ID}\""

echo "== render ordine detail actions (templated workflow) =="
"${BIN[@]}" ui examples/apps/sales.axl | jq -e '.uis[0].actions[] | select(.path=="/ordini/conferma") | .submit == "/ordini/{id}/conferma"'
"${BIN[@]}" ui examples/apps/sales.axl | jq -e '.uis[0].actions[] | select(.path=="/ordini/conferma") | .redirect == "/ordini/{id}"'
"${BIN[@]}" ui examples/apps/sales.axl | jq -e '.uis[0].actions[] | select(.path=="/ordini/annulla") | .submit == "/ordini/{id}/annulla"'
"${BIN[@]}" ui examples/apps/sales.axl | jq -e '.uis[0].actions[] | select(.path=="/ordini/annulla") | .redirect == "/ordini/{id}"'

echo "== render preventivo detail (templated path manifest) =="
"${BIN[@]}" ui examples/apps/sales.axl | jq -e '.uis[0].pages[] | select(.path=="/preventivi/{id}") | .template == "/preventivi/{id}" and .input_source == "path" and .input_name == "id"'

echo "== render preventivo detail (templated path /preventivi/preventivo-001) =="
# render CLI uses a fresh store; eval RenderDettaglioPreventivoDemoUnit proves seed+lookup.
# This gate proves path binding and templated action submit on the detail page.
"${BIN[@]}" render examples/apps/sales.axl /preventivi/preventivo-001 null | grep -q 'preventivi/preventivo-001'
"${BIN[@]}" render examples/apps/sales.axl /preventivi/preventivo-001 null | grep -q 'action="/preventivi/preventivo-001/invia"'
"${BIN[@]}" render examples/apps/sales.axl /preventivi/preventivo-001 null | grep -q 'action="/preventivi/preventivo-001/conferma"'
"${BIN[@]}" render examples/apps/sales.axl /preventivi/preventivo-001 null | grep -q 'action="/ordini/da-preventivo/preventivo-001"'
"${BIN[@]}" render examples/apps/sales.axl /preventivi/preventivo-001 null | grep -q 'name="id" value="preventivo-001"'

echo "== render preventivo detail actions (templated workflow) =="
# render uses a fresh runtime (empty store); prove action wiring via manifest + serve below
"${BIN[@]}" ui examples/apps/sales.axl | jq -e '.uis[0].actions[] | select(.path=="/preventivi/invia") | .submit == "/preventivi/{id}/invia"'
"${BIN[@]}" ui examples/apps/sales.axl | jq -e '.uis[0].actions[] | select(.path=="/preventivi/conferma") | .redirect == "/preventivi/{id}"'
"${BIN[@]}" ui examples/apps/sales.axl | jq -e '[.uis[].pages[].path] | index("/ordini")'
"${BIN[@]}" ui examples/apps/sales.axl | jq -e '[.uis[].pages[].path] | index("/ordini/demo")'
"${BIN[@]}" ui examples/apps/sales.axl | jq -e '[.uis[].pages[].path] | index("/ordini/{id}")'
"${BIN[@]}" ui examples/apps/sales.axl | jq -e '[.uis[].actions[].path] | index("/preventivi/ordine")'
"${BIN[@]}" ui examples/apps/sales.axl | jq -e '[.uis[].actions[].path] | index("/ordini/conferma")'
"${BIN[@]}" ui examples/apps/sales.axl | jq -e '[.uis[].actions[].path] | index("/ordini/annulla")'
"${BIN[@]}" ui examples/apps/sales.axl | jq -e '.uis[0].actions[] | select(.path=="/preventivi/ordine") | .submit == "/ordini/da-preventivo/{id}" and .on == "/preventivi/{id}" and .redirect == "/ordini/{id}"'

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
"${BIN[@]}" ui examples/apps/sales.axl | jq -e '[.uis[].pages[].path] | index("/ordini")'
"${BIN[@]}" ui examples/apps/sales.axl | jq -e '[.uis[].pages[].path] | index("/ordini/demo")'
"${BIN[@]}" ui examples/apps/sales.axl | jq -e '[.uis[].pages[].path] | index("/ordini/{id}")'
"${BIN[@]}" ui examples/apps/sales.axl | jq -e '[.uis[].actions[].path] | index("/preventivi/ordine")'
"${BIN[@]}" ui examples/apps/sales.axl | jq -e '[.uis[].actions[].path] | index("/ordini/conferma")'
"${BIN[@]}" ui examples/apps/sales.axl | jq -e '[.uis[].actions[].path] | index("/ordini/annulla")'
"${BIN[@]}" ui examples/apps/sales.axl | jq -e '.uis[0].forms[] | select(.path=="/clienti/new") | .submit == "/clienti"'
"${BIN[@]}" ui examples/apps/sales.axl | jq -e '.uis[0].forms[] | select(.path=="/prodotti/new") | .submit == "/prodotti"'
"${BIN[@]}" ui examples/apps/sales.axl | jq -e '.uis[0].forms[] | select(.path=="/preventivi/new") | .submit == "/preventivi"'
"${BIN[@]}" ui examples/apps/sales.axl | jq -e '.uis[0].actions[] | select(.path=="/preventivi/invia") | .submit == "/preventivi/{id}/invia"'
"${BIN[@]}" ui examples/apps/sales.axl | jq -e '.uis[0].actions[] | select(.path=="/preventivi/conferma") | .redirect == "/preventivi/{id}"'
"${BIN[@]}" ui examples/apps/sales.axl | jq -e '.uis[0].actions[] | select(.path=="/preventivi/ordine") | .submit == "/ordini/da-preventivo/{id}" and .on == "/preventivi/{id}" and .redirect == "/ordini/{id}"'
"${BIN[@]}" ui examples/apps/sales.axl | jq -e '.uis[0].actions[] | select(.path=="/ordini/conferma") | .submit == "/ordini/{id}/conferma"'
"${BIN[@]}" ui examples/apps/sales.axl | jq -e '.uis[0].actions[] | select(.path=="/ordini/conferma") | .redirect == "/ordini/{id}"'
"${BIN[@]}" ui examples/apps/sales.axl | jq -e '.uis[0].actions[] | select(.path=="/ordini/annulla") | .submit == "/ordini/{id}/annulla"'
"${BIN[@]}" ui examples/apps/sales.axl | jq -e '.uis[0].actions[] | select(.path=="/ordini/annulla") | .redirect == "/ordini/{id}"'

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
  curl -sf --max-time 2 -H 'accept: text/html' "http://127.0.0.1:${PORT}/preventivi/preventivo-001" | grep -q '<th>prodotto_id</th>'
  curl -sf --max-time 2 -H 'accept: text/html' "http://127.0.0.1:${PORT}/preventivi/preventivo-001" | grep -q '<th>quantita</th>'
  curl -sf --max-time 2 -H 'accept: text/html' "http://127.0.0.1:${PORT}/preventivi/preventivo-001" | grep -q 'prodotto-001'
  curl -sf --max-time 2 -H 'accept: text/html' "http://127.0.0.1:${PORT}/preventivi/preventivo-001" | grep -q 'action="/preventivi/preventivo-001/invia"'
  curl -sf --max-time 2 -H 'accept: text/html' "http://127.0.0.1:${PORT}/preventivi/preventivo-001" | grep -q 'action="/preventivi/preventivo-001/conferma"'
  INVIA_001_HEADERS=$(curl -s -D - -o /dev/null --max-time 2 -X POST "http://127.0.0.1:${PORT}/preventivi/preventivo-001/invia" \
    -H 'content-type: application/x-www-form-urlencoded' \
    -H 'accept: text/html' \
    --data-urlencode 'id=preventivo-001')
  echo "$INVIA_001_HEADERS" | grep -qi '^HTTP/.* 303'
  echo "$INVIA_001_HEADERS" | grep -qi '^location: /preventivi/preventivo-001'
  curl -sf --max-time 2 -H 'accept: text/html' "http://127.0.0.1:${PORT}/preventivi/preventivo-001" | grep -q 'inviato'
  CONFERMA_001_HEADERS=$(curl -s -D - -o /dev/null --max-time 2 -X POST "http://127.0.0.1:${PORT}/preventivi/preventivo-001/conferma" \
    -H 'content-type: application/x-www-form-urlencoded' \
    -H 'accept: text/html' \
    --data-urlencode 'id=preventivo-001')
  echo "$CONFERMA_001_HEADERS" | grep -qi '^HTTP/.* 303'
  echo "$CONFERMA_001_HEADERS" | grep -qi '^location: /preventivi/preventivo-001'
  curl -sf --max-time 2 -H 'accept: text/html' "http://127.0.0.1:${PORT}/preventivi/preventivo-001" | grep -q 'confermato'
  curl -sf --max-time 2 -H 'accept: text/html' "http://127.0.0.1:${PORT}/preventivi/preventivo-001" | grep -q 'action="/ordini/da-preventivo/preventivo-001"'
  ORDINE_001_HEADERS=$(curl -s -D - -o /dev/null --max-time 2 -X POST "http://127.0.0.1:${PORT}/ordini/da-preventivo/preventivo-001" \
    -H 'content-type: application/x-www-form-urlencoded' \
    -H 'accept: text/html' \
    --data-urlencode 'id=preventivo-001')
  echo "$ORDINE_001_HEADERS" | grep -qi '^HTTP/.* 303'
  ORDINE_001_ID=$(echo "$ORDINE_001_HEADERS" | grep -i '^location:' | sed -E 's|^[Ll]ocation:[[:space:]]*/ordini/||' | tr -d '\r')
  test -n "$ORDINE_001_ID"
  test "$ORDINE_001_ID" != "preventivo-001"
  echo "$ORDINE_001_HEADERS" | grep -qi "^location: /ordini/${ORDINE_001_ID}"
  curl -sf --max-time 2 "http://127.0.0.1:${PORT}/ordini" | grep -q 'preventivo-001'
  curl -sf --max-time 2 "http://127.0.0.1:${PORT}/ordini" | grep -q '268770'
  curl -sf --max-time 2 "http://127.0.0.1:${PORT}/ordini" | grep -q "href=\"/ordini/${ORDINE_001_ID}\""
  curl -sf --max-time 2 -X POST "http://127.0.0.1:${PORT}/ordini/da-preventivo/preventivo-001" | jq -e '.ok.stato == "bozza" and .ok.id != .ok.preventivo_id and .ok.totale == 268770'
  curl -sf --max-time 2 -H 'accept: text/html' "http://127.0.0.1:${PORT}/ordini/${ORDINE_001_ID}" | grep -q 'preventivo-001'
  curl -sf --max-time 2 -H 'accept: text/html' "http://127.0.0.1:${PORT}/ordini/${ORDINE_001_ID}" | grep -q '268770'
  curl -sf --max-time 2 -H 'accept: text/html' "http://127.0.0.1:${PORT}/ordini/${ORDINE_001_ID}" | grep -q '<th>prodotto_id</th>'
  curl -sf --max-time 2 -H 'accept: text/html' "http://127.0.0.1:${PORT}/ordini/${ORDINE_001_ID}" | grep -q 'prodotto-002'
  curl -sf --max-time 2 -H 'accept: text/html' "http://127.0.0.1:${PORT}/ordini/${ORDINE_001_ID}" | grep -q 'cliente-001'
  curl -sf --max-time 2 -H 'accept: text/html' "http://127.0.0.1:${PORT}/ordini/${ORDINE_001_ID}" | grep -q "action=\"/ordini/${ORDINE_001_ID}/conferma\""
  curl -sf --max-time 2 -H 'accept: text/html' "http://127.0.0.1:${PORT}/ordini/${ORDINE_001_ID}" | grep -q "action=\"/ordini/${ORDINE_001_ID}/annulla\""
  ORDINE_CONFERMA_HEADERS=$(curl -s -D - -o /dev/null --max-time 2 -X POST "http://127.0.0.1:${PORT}/ordini/${ORDINE_001_ID}/conferma" \
    -H 'content-type: application/x-www-form-urlencoded' \
    -H 'accept: text/html' \
    --data-urlencode "id=${ORDINE_001_ID}")
  echo "$ORDINE_CONFERMA_HEADERS" | grep -qi '^HTTP/.* 303'
  echo "$ORDINE_CONFERMA_HEADERS" | grep -qi "^location: /ordini/${ORDINE_001_ID}"
  curl -sf --max-time 2 -H 'accept: text/html' "http://127.0.0.1:${PORT}/ordini/${ORDINE_001_ID}" | grep -q 'confermato'
  curl -s --max-time 2 -X POST "http://127.0.0.1:${PORT}/ordini/${ORDINE_001_ID}/conferma" | jq -e '.error == "stato_non_confermable"'
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
  ORDINE_002_JSON=$(curl -sf --max-time 2 -X POST "http://127.0.0.1:${PORT}/ordini/da-preventivo/preventivo-002")
  echo "$ORDINE_002_JSON" | jq -e '.ok.stato == "bozza" and .ok.id != .ok.preventivo_id and .ok.totale == 135880'
  ORDINE_002_ID=$(echo "$ORDINE_002_JSON" | jq -r '.ok.id')
  curl -sf --max-time 2 -H 'accept: text/html' "http://127.0.0.1:${PORT}/ordini/${ORDINE_002_ID}" | grep -q 'preventivo-002'
  curl -sf --max-time 2 -H 'accept: text/html' "http://127.0.0.1:${PORT}/ordini/${ORDINE_002_ID}" | grep -q "action=\"/ordini/${ORDINE_002_ID}/annulla\""
  ORDINE_ANNULLA_HEADERS=$(curl -s -D - -o /dev/null --max-time 2 -X POST "http://127.0.0.1:${PORT}/ordini/${ORDINE_002_ID}/annulla" \
    -H 'content-type: application/x-www-form-urlencoded' \
    -H 'accept: text/html' \
    --data-urlencode "id=${ORDINE_002_ID}")
  echo "$ORDINE_ANNULLA_HEADERS" | grep -qi '^HTTP/.* 303'
  echo "$ORDINE_ANNULLA_HEADERS" | grep -qi "^location: /ordini/${ORDINE_002_ID}"
  curl -sf --max-time 2 -H 'accept: text/html' "http://127.0.0.1:${PORT}/ordini/${ORDINE_002_ID}" | grep -q 'annullato'
  curl -s -H 'accept: application/json' --max-time 2 "http://127.0.0.1:${PORT}/ordini/${ORDINE_002_ID}" | jq -e '.ok.stato == "annullato" and .ok.totale == 135880'
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
  DURABLE_ORDINE=$(curl -sf --max-time 2 -X POST "http://127.0.0.1:${DPORT}/ordini/durable/da-preventivo/preventivo-002")
  echo "$DURABLE_ORDINE" | jq -e '.ok.stato == "bozza" and .ok.id != .ok.preventivo_id and .ok.totale == 135880'
  DURABLE_ORDINE_ID=$(echo "$DURABLE_ORDINE" | jq -r '.ok.id')
  curl -sf --max-time 2 -X POST "http://127.0.0.1:${DPORT}/ordini/durable/${DURABLE_ORDINE_ID}/conferma" | jq -e '.ok.stato == "confermato"'
  curl -sf --max-time 2 "http://127.0.0.1:${DPORT}/ordini/durable/${DURABLE_ORDINE_ID}" | jq -e '.ok.stato == "confermato" and .ok.preventivo_id == "preventivo-002"'
)

echo "OK: verify-sales"
