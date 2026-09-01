#!/usr/bin/env sh
# Bundle root assets into docs/ for a self-contained GitHub Pages site.
set -e
ROOT="$(cd "$(dirname "$0")/.." && pwd)"
DOCS="$ROOT/docs"

cp "$ROOT/presentation.html" "$ROOT/mondo.html" "$ROOT/film.html" "$DOCS/"
cp "$ROOT/SPEC-4.0.md" "$DOCS/"
mkdir -p "$DOCS/hosts/portal-web" "$DOCS/examples/apps"
cp "$ROOT/hosts/portal-web/README.md" "$DOCS/hosts/portal-web/README.md"
cp "$ROOT/examples/apps/README.md" "$DOCS/examples/apps/README.md"

# Point bundled HTML at the docs site layout (book.html = GitBook shell).
for file in presentation.html mondo.html; do
  sed \
    -e 's|href="docs/index.html"|href="book.html"|g' \
    -e 's|href="docs/roadmap.md"|href="book.html?p=roadmap.md"|g' \
    "$DOCS/$file" > "$DOCS/$file.tmp" && mv "$DOCS/$file.tmp" "$DOCS/$file"
done

echo "Docs site ready in docs/ (home: index.html, book: book.html)"
