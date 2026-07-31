// Standalone entry — both `pnpm dev` and the production build mount
// the Runebender widget directly into index.html.
//
// Host selection at boot:
//   1. A local workspace server (server/serve.mjs), probed via
//      `/runebender/api/info` (or `?workspace=` for another origin) —
//      load/save/watch wired through it.
//   2. File System Access (desktop Chromium): the fsAccess host opens
//      a UFO/designspace folder straight from disk with the same
//      save/watch contract; a workspace remembered from a previous
//      visit reopens silently when the user granted "Allow on every
//      visit", otherwise the welcome panel offers to reopen it.
//   3. Anything else: the read-only browser host with the bundled
//      demo font.

import { createApp, ref } from "vue";
import { readDevTestFontFiles } from "./devTestFont";
import { runebenderHostKey } from "./host/runebenderHost";
import { browserHost } from "./hosts/browser/browserHost";
import {
  createFsAccessHost,
  isFsAccessSupported,
} from "./hosts/fsaccess/fsAccessHost";
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
    return;
  }

  if (isFsAccessSupported()) {
    const fontPathRef = ref("");
    const host = createFsAccessHost({
      onWorkspaceOpened(slot) {
        fontPathRef.value = slot;
      },
    });
    const stored = await host.primeStoredWorkspace();
    if (stored?.permission === "granted") {
      await host.reopenStoredWorkspaceSilently();
    }
    // A remembered workspace that still needs a permission click gets
    // the welcome panel (with its Reopen button) instead of the demo
    // font auto-load.
    const needsReopen = stored !== null && !fontPathRef.value;
    createApp(Runebender, {
      fontPathRef,
      ...(needsReopen ? {} : { initialFiles: readDevTestFontFiles }),
    })
      .provide(runebenderHostKey, host)
      .mount("#app");
    return;
  }

  createApp(Runebender, {
    initialFiles: readDevTestFontFiles,
  })
    .provide(runebenderHostKey, browserHost)
    .mount("#app");
}

void boot();
