#!/usr/bin/env bash
set -euo pipefail

site_dir="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)"
cd -- "$site_dir"

if ! command -v pandoc >/dev/null 2>&1; then
  echo "Pandoc is required: https://pandoc.org/installing.html" >&2
  exit 1
fi

rm -rf -- dist
mkdir -- dist
pandoc intro.md \
  --from=gfm \
  --to=html5 \
  --standalone \
  --template=page.html \
  --metadata=pagetitle:"Tiger Code Sha1" \
  --output=dist/index.html
cp -- style.css dist/style.css
cp -R -- assets dist/assets

printf 'Built %s/dist/index.html\n' "$site_dir"
