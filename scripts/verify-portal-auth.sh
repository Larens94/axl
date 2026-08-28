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

echo "OK auth gates"
