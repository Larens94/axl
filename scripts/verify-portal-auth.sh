#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
cd "$ROOT"
# shellcheck source=env.sh
source "$(dirname "$0")/env.sh"
BIN=(cargo run -p axl-compiler --quiet --)
PORTAL="${PORTAL:-examples/apps/portal.axl}"

echo "== check auth domain =="
"${BIN[@]}" check examples/apps/auth.axl --json | jq -e '.protocol == "axl-check/1" and .ok == true'

echo "== check portal (auth + vendite) =="
"${BIN[@]}" check "$PORTAL" --json | jq -e '.protocol == "axl-check/1" and .ok == true and .app == "Portal"'

echo "== ui manifest modular portal (AuthUi + VenditeUi + VenditeDemoUi) =="
"${BIN[@]}" ui "$PORTAL" | jq -e '.app == "Portal" and (.uis | length) == 3'
"${BIN[@]}" ui "$PORTAL" | jq -e '[.uis[].name] | index("AuthUi") != null and index("VenditeUi") != null and index("VenditeDemoUi") != null'

echo "== eval RegistraUtente =="
"${BIN[@]}" eval "$PORTAL" RegistraUtente examples/apps/inputs/auth-register.json | jq -e '.ok.email == "nuovo@example.com"'

echo "== eval InizializzaAuthDemo =="
"${BIN[@]}" eval "$PORTAL" InizializzaAuthDemo examples/apps/inputs/unit.json | jq -e '.ok.total >= 2'

echo "== eval LoginAdminDemoUnit =="
"${BIN[@]}" eval "$PORTAL" LoginAdminDemoUnit examples/apps/inputs/unit.json | jq -e '.ok.utente_id == "utente-admin" and (.ok.token | type == "string")'

echo "== eval UtenteHaPermessoDemoUnit (RBAC seed) =="
"${BIN[@]}" eval "$PORTAL" UtenteHaPermessoDemoUnit examples/apps/inputs/unit.json | jq -e '.ok == true'

echo "== eval CreaRuoloDinamicoDemoUnit (dynamic role + permission) =="
"${BIN[@]}" eval "$PORTAL" CreaRuoloDinamicoDemoUnit examples/apps/inputs/unit.json | jq -e '.ok.nome == "supporto" and .ok.permessi[0] == "vendite.clienti.read"'

echo "== eval PaginaHome =="
"${BIN[@]}" eval "$PORTAL" PaginaHome examples/apps/inputs/unit.json | jq -e '.ok.titolo == "AXL Portal"'

echo "== eval ResetPasswordDemoUnit (richiesta + reset + login) =="
"${BIN[@]}" eval "$PORTAL" ResetPasswordDemoUnit examples/apps/inputs/unit.json | jq -e '.ok.email == "admin@example.com"'

echo "== eval PaginaClientiRbacDemoUnit (RBAC + vendite list) =="
"${BIN[@]}" eval "$PORTAL" PaginaClientiRbacDemoUnit examples/apps/inputs/unit.json | jq -e '.ok.total >= 0'

echo "== route guards in HTTP manifest (session + can on VenditeApi POST /clienti) =="
"${BIN[@]}" blocks "$PORTAL" >/dev/null
"${BIN[@]}" ir "$PORTAL" | jq -e '[.nodes[] | select(.kind=="route_guard")] | length > 0'
"${BIN[@]}" ir "$PORTAL" | jq -e '[.nodes[] | select(.kind=="route_guard" and .metadata.kind=="session")] | length > 0'
"${BIN[@]}" ir "$PORTAL" | jq -e '[.nodes[] | select(.kind=="route_guard" and .metadata.kind=="can")] | length > 0'

echo "== Gate 8 secret refs (no plaintext pepper/jwt in IR) =="
"${BIN[@]}" ir "$PORTAL" | jq -e '[.nodes[] | select(.kind=="config" and .metadata.secret_ref != null)] | length >= 3'
"${BIN[@]}" ir "$PORTAL" | jq -e '
  [.nodes[]
    | select(.kind=="config")
    | select(.metadata.secret_ref == null)
    | .metadata.value
    | tostring
    | select(test("axl-auth-demo-pepper|axl-auth-demo-jwt|axl-vendite-demo"))]
  | length == 0
'

echo "== React codegen from axl-ui/1 (experiment targets) =="
TMP_REACT=$(mktemp -d)
"${BIN[@]}" experiment "$PORTAL" "$TMP_REACT" >/dev/null
test -f "$TMP_REACT/targets/react/axl_routes.tsx"
test -f "$TMP_REACT/targets/react/axl_layouts.tsx"
test -f "$TMP_REACT/targets/react/axl_registry.ts"
grep -q 'axl-ui/1' "$TMP_REACT/targets/react/axl_routes.tsx"
grep -q 'GuestLayout' "$TMP_REACT/targets/react/axl_layouts.tsx"
grep -q 'axlProductionRoutes' "$TMP_REACT/targets/react/axl_routes.tsx"
rm -rf "$TMP_REACT"

echo "== HTTP route guard smoke (401 without session, 200 with session) =="
GPORT=18088
(
  "${BIN[@]}" serve "$PORTAL" "127.0.0.1:${GPORT}" &
  GPID=$!
  cleanup() { kill "$GPID" 2>/dev/null; wait "$GPID" 2>/dev/null || true; }
  trap cleanup EXIT
  ready=0
  for _ in $(seq 1 50); do
    if curl -sf --max-time 1 "http://127.0.0.1:${GPORT}/" >/dev/null 2>&1; then
      ready=1
      break
    fi
    sleep 0.2
  done
  test "$ready" -eq 1
  curl -s --max-time 2 -X POST "http://127.0.0.1:${GPORT}/clienti" \
    -H 'content-type: application/json' \
    -d '{"id":"cliente-guard","nome":"Guard","email":"g@example.com","budget":1,"stato":"attivo"}' \
    | jq -e '.error == "session_required"'
  curl -sf --max-time 2 -X POST "http://127.0.0.1:${GPORT}/auth/init-demo" \
    -H 'content-type: application/json' -d 'null' >/dev/null
  SID=$(curl -sf --max-time 2 -X POST "http://127.0.0.1:${GPORT}/auth/login" \
    -H 'content-type: application/json' \
    -d '{"email":"admin@example.com","password":"admin123"}' | jq -r '.ok.session_id')
  test -n "$SID"
  curl -sf --max-time 2 -H "Cookie: sid=${SID}" -X POST "http://127.0.0.1:${GPORT}/clienti" \
    -H 'content-type: application/json' \
    -d '{"id":"cliente-guard","nome":"Guard","email":"g@example.com","budget":1,"stato":"attivo"}' \
    | jq -e '.ok.id == "cliente-guard"'
  curl -s --max-time 2 -H "Cookie: sid=${SID}" "http://127.0.0.1:${GPORT}/auth/admin/utenti" \
    | jq -e '.ok.total >= 2'
)

echo "OK auth gates"
