#!/usr/bin/env bash
# Copy Markdown sources into dist/ so that *.md URLs serve raw Markdown.
# Run after `zola build` and before deploying dist/.
set -euo pipefail

CONTENT_DIR="content/docs"
DIST_DIR="dist/docs"

find "$CONTENT_DIR" -name '*.md' | while read -r src; do
  rel="${src#$CONTENT_DIR/}"
  base="$(basename "$rel")"
  dir="$(dirname "$rel")"

  # _index.md handling
  if [ "$base" = "_index.md" ]; then
    # Skip redirect-only _index.md files
    if grep -q 'redirect_to' "$src" && ! awk 'BEGIN{s=0} /^[+]{3}$/{s++;next} s>=2{found=1; exit} END{exit !found}' "$src"; then
      continue
    fi
    # _index.md with content → parent dir name .md  (directives/_index.md → directives.md)
    if [ "$dir" = "." ]; then
      continue  # top-level _index.md — skip
    fi
    dest="$DIST_DIR/${dir}.md"
  else
    dest="$DIST_DIR/$rel"
  fi

  mkdir -p "$(dirname "$dest")"

  # Strip TOML front matter (+++…+++) and transform links
  awk 'BEGIN{s=0} /^[+]{3}$/{s++;next} s>=2' "$src" \
    | sed -E 's|@/docs/([^)]+)/_index\.md|/docs/\1.md|g' \
    | sed -E 's|@(/docs/[^)]+)|\1|g' \
    | sed -E 's|\]\(/docs/([^)#]+)/#|\](/docs/\1.md#|g' \
    > "$dest"
done

echo "Markdown sources copied to $DIST_DIR"
