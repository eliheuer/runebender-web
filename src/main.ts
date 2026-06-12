// Standalone entry — both `pnpm dev` and the production build mount
// the Runebender widget directly into index.html.
//
// At boot we probe for the local workspace server (server/serve.mjs).
// If present, the editor opens the served font with load/save/watch
// wired through it; otherwise it falls back to the browser host and
// the bundled demo font.
//
// `?workspace=http://localhost:8765` points the editor at a server on
// another origin — useful when developing the editor under `pnpm dev`
// while a workspace server runs separately.

import { createApp } from "vue";
import { readDevTestFontFiles } from "./devTestFont";
import { runebenderHostKey } from "./host/runebenderHost";
import { browserHost } from "./hosts/browser/browserHost";
import { createLocalHost, type LocalServerInfo } from "./hosts/local/localHost";
import Runebender from "./Runebender.vue";

async function detectWorkspaceServer(): Promise<{
  info: LocalServerInfo;
  base: string;
} | null> {
  const base =
    new URLSearchParams(window.location.search).get("workspace") ?? "";
  try {
    const res = await fetch(`${base}/runebender/api/info`);
    if (!res.ok) return null;
    const info = (await res.json()) as LocalServerInfo;
    if (info?.server !== "runebender-serve") return null;
    return { info, base };
  } catch {
    return null;
  }
}

async function boot() {
  const server = await detectWorkspaceServer();
  if (server) {
    createApp(Runebender, {
      fontPathRef: { value: server.info.root },
    })
      .provide(runebenderHostKey, createLocalHost(server.info, server.base))
      .mount("#app");
  } else {
    createApp(Runebender, {
      initialFiles: readDevTestFontFiles,
    })
      .provide(runebenderHostKey, browserHost)
      .mount("#app");
  }
}

void boot();
