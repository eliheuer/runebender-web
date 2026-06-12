import { defineConfig } from "vite";
import vue from "@vitejs/plugin-vue";

// Standalone Runebender web editor.
//
// This is a normal Vite HTML app build: index.html + src/main.ts mount
// the editor with the browser host and auto-load the bundled demo font
// (assets/test-fonts/, refresh with scripts/update-demo-font.sh).
//
// The wasm-pack `--target web` shim in wasm/ loads its binary via
//   new URL('runebender_web_bg.wasm', import.meta.url)
// which Vite resolves natively in app mode: the wasm is emitted to
// assets/ and the URL is rewritten with `base` applied.
//
// `RUNEBENDER_BASE` sets the subpath the site is served from, e.g.
//   RUNEBENDER_BASE=/cloud/editor/ pnpm build
// for embedding under runebender.org/cloud/editor/. Default is root.
//
// Downstream hosts (runebender-comfy's ComfyUI extension) do NOT use
// this config — they import src/Runebender.vue and the host interface
// directly and run their own build.

export default defineConfig({
  base: process.env.RUNEBENDER_BASE ?? "/",
  plugins: [vue()],
  define: {
    "process.env.NODE_ENV": JSON.stringify("production"),
  },
  build: {
    outDir: "dist",
    emptyOutDir: true,
    target: "esnext",
  },
});
