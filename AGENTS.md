# Agent notes — runebender-web

A browser font editor: Rust → wasm core (Vello/Kurbo, WebGPU canvas),
Vue 3 UI. This file is for AI agents (codex, Claude Code) working in
this repo or using the editor during type-design sessions.

## Launch

```sh
pnpm install        # first time only; no Rust toolchain required
pnpm dev            # http://localhost:5173 (demo font, no disk access)
```

To let the user view/edit a real font on disk — THE primary agent
workflow — run the workspace server instead and give them the URL:

```sh
pnpm build                                   # if dist/ is stale
node server/serve.mjs <path-to-font> --port 8765
# <path-to-font>: .designspace file, .ufo directory, or a directory
# containing them. Editor at http://localhost:8765.
```

While it runs, any edits you make to the font's files on disk stream
into the open editor live (SSE + file watcher, self-writes suppressed).
The user's Cmd+S saves write through with If-Match content hashes —
your edits can never be silently clobbered by a stale editor (the
editor gets a 409 and keeps the file marked unsaved). When writing
UFOs yourself, write .glif files BEFORE updating contents.plist.

The dev server auto-loads the bundled Virtua Grotesk demo font from
`assets/test-fonts/`. The page needs a WebGPU browser (current
Chrome/Edge, Safari 18+). `pnpm build` produces a static `dist/`;
`RUNEBENDER_BASE=/sub/path/ pnpm build` for subpath hosting.

## Map

- `src/Runebender.vue` — the editor widget. Large single-file
  component; most UI behavior lives here. Panels/toolbars are in
  `src/components/`.
- `src/host/runebenderHost.ts` — the host interface. The widget calls
  its surroundings ONLY through this. `src/hosts/browser/browserHost.ts`
  is the standalone implementation.
- `src/main.ts` — standalone entry: mounts the widget, provides the
  browser host, auto-loads the demo font via `src/devTestFont.ts`.
- `core/` — Rust crate `runebender-web`. Editor model, editing tools,
  Vello renderer, `wasm_api.rs` (wasm-bindgen surface). Path-depends on
  the sibling checkout `../../runebender-core`. The autotracer `img2bez`
  is a git dependency on `github.com/eliheuer/img2bez` (default branch,
  commit pinned in `Cargo.lock`); `cargo update -p img2bez` pulls newer.
- `wasm/` — committed wasm-pack output. NEVER edit by hand.
- `src/gfSidebarData.generated.ts` — generated (see `pnpm gf-sidebar`),
  don't edit by hand.

## Change loops

- **UI/TS change**: edit under `src/`, the dev server hot-reloads.
- **Rust change**: edit under `core/src/`, then `pnpm wasm` (needs
  wasm-pack), commit the refreshed `wasm/` artifacts together with the
  Rust change. `cargo test` runs in `core/`.
- **Demo font bump**: `pnpm demo-font` (copies from a sibling
  `../virtua-grotesk` checkout; `sh scripts/update-demo-font.sh
  --remote` fetches latest main from GitHub instead).

## Known limits (don't chase these as bugs)

- **The browser host (no server) cannot save.** Every write path in
  `browserHost.ts` intentionally returns 501; loading works (drag-drop,
  bundled demo). Real load/save/watch is the workspace server's job
  (`server/serve.mjs` + `src/hosts/local/localHost.ts`). A File System
  Access API host is planned for the hosted editor at runebender.org.
- The ComfyUI integration does not live here — that's
  [runebender-comfy](https://github.com/eliheuer/runebender-comfy),
  which imports this package and provides its own host
  (`hosts/comfy/` + `extension.ts` over there). Keep the
  `RunebenderHost` interface backward-compatible or update comfy in
  lockstep.

## Conventions

- License is GPL-3.0-or-later (user-facing frontend; the shared
  `runebender-core` types crate is permissively licensed instead).
- Vue SFC + TypeScript, no semicolon/style enforcement beyond what's
  already in the file you're editing — match surrounding code.
- Rust follows the Linebender canonical lint set (see
  `core/Cargo.toml` `[lints]`).
