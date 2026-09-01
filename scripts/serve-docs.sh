#!/usr/bin/env sh
set -e
cd "$(dirname "$0")/.."
sh scripts/prepare-docs-site.sh
echo "Home:  http://127.0.0.1:4000/"
echo "Book:  http://127.0.0.1:4000/book.html"
exec npx --yes serve docs -l 4000
