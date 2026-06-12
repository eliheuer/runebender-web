#!/usr/bin/env sh
# Refresh the bundled demo font (assets/test-fonts/) from the latest
# Virtua Grotesk sources. The font is under active development at
# https://github.com/eliheuer/virtua-grotesk, so the demo should be
# easy to bump any time.
#
# Usage:
#   sh scripts/update-demo-font.sh             # copy from ../virtua-grotesk
#   sh scripts/update-demo-font.sh /path/to/virtua-grotesk
#   sh scripts/update-demo-font.sh --remote    # fetch a tarball of main
set -eu

REPO_ROOT="$(cd "$(dirname "$0")/.." && pwd)"
DEST="$REPO_ROOT/assets/test-fonts"
ITEMS="VirtuaGrotesk-Regular.ufo VirtuaGrotesk-Bold.ufo VirtuaGrotesk.designspace"

if [ "${1:-}" = "--remote" ]; then
  TMP="$(mktemp -d)"
  trap 'rm -rf "$TMP"' EXIT
  echo "Fetching latest virtua-grotesk main from GitHub..."
  curl -sL https://github.com/eliheuer/virtua-grotesk/archive/refs/heads/main.tar.gz \
    | tar -xz -C "$TMP"
  SRC="$TMP/virtua-grotesk-main/sources"
else
  SRC="${1:-$REPO_ROOT/../virtua-grotesk}/sources"
fi

if [ ! -d "$SRC" ]; then
  printf 'No virtua-grotesk sources at %s\n' "$SRC" >&2
  printf 'Pass a checkout path, or use --remote to fetch from GitHub.\n' >&2
  exit 1
fi

for item in $ITEMS; do
  if [ ! -e "$SRC/$item" ]; then
    printf 'Missing %s in %s\n' "$item" "$SRC" >&2
    exit 1
  fi
done

for item in $ITEMS; do
  rm -rf "$DEST/$item"
  cp -R "$SRC/$item" "$DEST/$item"
done

echo "Demo font updated from $SRC"
echo "Review with: git status assets/test-fonts"
