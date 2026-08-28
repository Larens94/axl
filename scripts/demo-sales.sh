#!/usr/bin/env bash
set -euo pipefail
ROOT="$(cd "$(dirname "$0")/.." && pwd)"
cd "$ROOT"
# shellcheck source=env.sh
source "$(dirname "$0")/env.sh"
exec cargo run -p axl-compiler -- serve examples/apps/sales.axl 127.0.0.1:8080
