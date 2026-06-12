# runebender-web

A font editor that runs in the browser. Rust (Vello + Kurbo, compiled
to WebAssembly with a WebGPU render path) drives the canvas; a Vue 3
app provides the surrounding UI. Ported from
[runebender-xilem](https://github.com/eliheuer/runebender-xilem) and
extracted from
[runebender-comfy](https://github.com/eliheuer/runebender-comfy), which
now imports this package for its ComfyUI widget.

The editor opens UFO/designspace sources and ships with
[Virtua Grotesk](https://github.com/eliheuer/virtua-grotesk) as a demo
font that loads automatically.

## Run it

```sh
pnpm install
pnpm dev        # editor at http://localhost:5173 with the demo font
```

No Rust toolchain needed — the compiled WebAssembly module in `wasm/`
is committed. You need a browser with WebGPU (current Chrome, Edge, or
Safari 18+).

To open your own font, drag a `.ufo` directory (or a `.designspace`
with its UFOs) onto the editor.

## Build a static site

```sh
pnpm build                                  # dist/, served from /
RUNEBENDER_BASE=/cloud/editor/ pnpm build   # served from a subpath
```

The output is fully static — host it anywhere.

## Repo layout

```
core/      Rust crate (runebender-web) — editor model, tools, Vello
           renderer, wasm-bindgen API. Builds to wasm/ via `pnpm wasm`.
src/       Vue app. Runebender.vue is the editor widget;
           host/runebenderHost.ts is the host-integration interface;
           hosts/browser/ is the standalone implementation.
wasm/      Committed wasm-pack output (runebender_web_bg.wasm + shim).
assets/    Bundled demo font (refresh with `pnpm demo-font`).
```

Rust development: `core/` depends on sibling checkouts of
[runebender-core](https://github.com/eliheuer/runebender-core) and
[img2bez](https://github.com/eliheuer/img2bez) (`../../…` path deps),
matching the convention across the Runebender repos. After changing
Rust code, run `pnpm wasm` and commit the refreshed `wasm/` output.

## Embedding

The editor widget is host-agnostic: it talks to its surroundings only
through the `RunebenderHost` interface (`src/host/runebenderHost.ts`).
runebender-comfy embeds it inside ComfyUI by providing a host backed by
the ComfyUI workspace API; the standalone build provides
`src/hosts/browser/`. To embed it elsewhere, depend on this package,
mount `src/Runebender.vue`, and provide your own host.

## License

GPL-3.0-or-later.
