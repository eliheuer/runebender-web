#!/usr/bin/env bash
# Launch Runebender's workspace server and open the editor in a
# chromeless browser window (Chrome/Brave "app mode" — no tabs, no
# address bar, just the editor).
#
#   ./run-app.sh <path-to-font> [--port N]
#
#   <path-to-font>  a .ufo dir, a .designspace file, or a directory
#                   containing them
#   --port N        listen port (default 8765)
#
# The server stays in the foreground; Ctrl-C stops it and the app
# window closes with it.

set -euo pipefail

HERE="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

port=8765
target=""
while [[ $# -gt 0 ]]; do
  case "$1" in
    --port) port="$2"; shift 2 ;;
    -h|--help)
      echo "usage: ./run-app.sh <path-to-font> [--port N]"; exit 0 ;;
    *)
      if [[ -z "$target" ]]; then target="$1"; shift
      else echo "error: unexpected argument: $1" >&2; exit 1; fi ;;
  esac
done

if [[ -z "$target" ]]; then
  echo "error: missing font path" >&2
  echo "usage: ./run-app.sh <path-to-font> [--port N]" >&2
  exit 1
fi

# Find a Chromium-based browser that supports app mode + WebGPU.
browser=""
for candidate in \
  "/Applications/Google Chrome.app/Contents/MacOS/Google Chrome" \
  "/Applications/Brave Browser.app/Contents/MacOS/Brave Browser" \
  "/Applications/Microsoft Edge.app/Contents/MacOS/Microsoft Edge" \
  "/Applications/Chromium.app/Contents/MacOS/Chromium"; do
  if [[ -x "$candidate" ]]; then browser="$candidate"; break; fi
done

if [[ -z "$browser" ]]; then
  echo "error: no Chromium-based browser found (need Chrome/Brave/Edge for app mode + WebGPU)" >&2
  exit 1
fi

url="http://localhost:${port}/"

# img2bez (the autotracer) is a git dependency compiled INTO the wasm;
# Cargo.lock pins the exact commit. Rebuild the wasm whenever that
# pinned commit changes — otherwise we'd serve a stale autotracer.
# To pull a newer img2bez: run `cargo update -p img2bez` (in core/),
# then launch — this guard picks up the new commit and rebuilds.
wasm_artifact="$HERE/wasm/runebender_web_bg.wasm"
rev_stamp="$HERE/wasm/.img2bez-rev"
cur_rev="$(grep -A2 'name = "img2bez"' "$HERE/core/Cargo.lock" 2>/dev/null \
  | grep -oE '[0-9a-f]{40}' | head -1)"

needs_wasm=0
if [[ ! -f "$wasm_artifact" ]]; then
  needs_wasm=1
elif [[ -n "$cur_rev" && "$(cat "$rev_stamp" 2>/dev/null)" != "$cur_rev" ]]; then
  needs_wasm=1
fi

if [[ "$needs_wasm" == 1 ]]; then
  echo "rebuilding wasm against img2bez ${cur_rev:-(locked rev)}…"
  (cd "$HERE" && pnpm wasm)
  [[ -n "$cur_rev" ]] && printf '%s\n' "$cur_rev" > "$rev_stamp"
  rm -rf "$HERE/dist"   # force dist rebuild below to pick up the new wasm
fi

# Build dist/ if it's missing (or was just invalidated above).
if [[ ! -d "$HERE/dist" ]]; then
  echo "building dist/…"
  (cd "$HERE" && pnpm build)
fi

# Start the workspace server in the background.
node "$HERE/server/serve.mjs" "$target" --port "$port" &
server_pid=$!

# Make sure the server dies when this script does.
cleanup() { kill "$server_pid" 2>/dev/null || true; }
trap cleanup EXIT INT TERM

# Wait for the server to start accepting connections.
for _ in $(seq 1 50); do
  if curl -sf -o /dev/null "$url"; then break; fi
  sleep 0.1
done

echo "opening chromeless window: $url"
"$browser" \
  --app="$url" \
  --user-data-dir="$HOME/.runebender-chrome" \
  --window-size=1400,900 \
  >/dev/null 2>&1 &

# Keep the server in the foreground.
wait "$server_pid"
