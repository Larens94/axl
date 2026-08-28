#!/usr/bin/env bash
set -euo pipefail
exec cargo run -p axl-compiler -- serve examples/apps/sales.axl 127.0.0.1:8080
