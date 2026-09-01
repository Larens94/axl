#!/usr/bin/env sh
# Serve the repo root so docs/index.html can fetch ../SPEC-4.0.md, presentation.html, etc.
set -e
cd "$(dirname "$0")/.."
echo "Docs: http://127.0.0.1:4000/docs/index.html"
exec npx --yes serve . -l 4000
