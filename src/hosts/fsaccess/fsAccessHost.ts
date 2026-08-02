// RunebenderHost backed by the File System Access API — the hosted
// editor (runebender.org) editing a real UFO/designspace directory on
// the user's disk, desktop Chromium only.
//
// This is the same save contract as the local workspace server, with
// the browser holding the directory handle instead of a server holding
// the files:
//
//   - Every file read records a content hash; every write re-reads the
//     file on disk first and refuses with a 409 when the disk content
//     no longer matches the last-seen hash — a stale editor can't
//     clobber what an agent or another tool just wrote. (Unlike the
//     server's If-Match this check-then-write isn't atomic, but the
//     window is a single task tick against human/agent-speed writers.)
//   - watchWorkspaceChanges uses FileSystemObserver (Chrome 133+,
//     recursive) with a lastModified-polling fallback, and hands
//     externally changed file contents to the editor. Self-writes are
//     suppressed by hash equality. Held-back changes (unsaved local
//     edits) keep the stale hash so the user's next save surfaces the
//     conflict instead of overwriting the external edit.

import type {
  RunebenderHost,
  WorkspaceExternalChange,
  WorkspaceSlotPayload,
} from "../../host/runebenderHost";
import {
  addRecentWorkspace,
  clearWorkspaceHandle,
  loadRecentWorkspaces,
  loadWorkspaceHandle,
  saveWorkspaceHandle,
  type RecentEntry,
} from "./handleStore";

const TEXT_EXTENSIONS = [
  ".glif",
  ".plist",
  ".designspace",
  ".fea",
  ".xml",
  ".json",
  ".txt",
  // Glyphs sources load via the editor's in-memory conversion
  // (read-only), so a picked folder holding only a .glyphs works.
  ".glyphs",
];

const isTextPath = (p: string) =>
  TEXT_EXTENSIONS.some((ext) => p.endsWith(ext));

// Directories that never hold font-source text files; skipping them
// keeps tree walks cheap in folders that also contain a repo checkout.
const SKIP_DIRS = new Set(["node_modules", "__pycache__", "venv", ".venv"]);
const shouldSkipDir = (name: string) =>
  name.startsWith(".") || SKIP_DIRS.has(name);

const POLL_INTERVAL_MS = 3000;

type IterableDirectoryHandle = FileSystemDirectoryHandle & {
  entries(): AsyncIterableIterator<
    [string, FileSystemFileHandle | FileSystemDirectoryHandle]
  >;
};

type PermissionCapableHandle = FileSystemDirectoryHandle & {
  queryPermission?(desc: { mode: "read" | "readwrite" }): Promise<PermissionState>;
  requestPermission?(desc: { mode: "read" | "readwrite" }): Promise<PermissionState>;
};

type DirectoryPickerWindow = Window & {
  showDirectoryPicker?: (options?: {
    id?: string;
    mode?: "read" | "readwrite";
    startIn?: FileSystemHandle | string;
  }) => Promise<FileSystemDirectoryHandle>;
  showOpenFilePicker?: (options?: {
    id?: string;
    types?: { description: string; accept: Record<string, string[]> }[];
    excludeAcceptAllOption?: boolean;
  }) => Promise<FileSystemFileHandle[]>;
};

type ObserverRecord = {
  type: string;
  relativePathComponents: (string | number)[];
  relativePathMovedFrom?: (string | number)[] | null;
};

type ObserverCtor = new (
  callback: (records: ObserverRecord[]) => void,
) => {
  observe(
    handle: FileSystemDirectoryHandle,
    options?: { recursive?: boolean },
  ): Promise<void>;
  disconnect(): void;
};

export function isFsAccessSupported(): boolean {
  return typeof (window as DirectoryPickerWindow).showDirectoryPicker ===
    "function";
}

async function sha256Hex(text: string): Promise<string> {
  const digest = await crypto.subtle.digest(
    "SHA-256",
    new TextEncoder().encode(text),
  );
  return Array.from(new Uint8Array(digest), (b) =>
    b.toString(16).padStart(2, "0"),
  ).join("");
}

async function mapConcurrent<T, R>(
  items: T[],
  limit: number,
  fn: (item: T) => Promise<R>,
): Promise<R[]> {
  const results = new Array<R>(items.length);
  let next = 0;
  const workers = Array.from(
    { length: Math.min(limit, items.length) },
    async () => {
      while (next < items.length) {
        const index = next++;
        results[index] = await fn(items[index]);
      }
    },
  );
  await Promise.all(workers);
  return results;
}

const jsonResponse = (status: number, body: Record<string, unknown>) =>
  new Response(JSON.stringify(body), {
    status,
    headers: { "content-type": "application/json" },
  });

const unavailable = () =>
  jsonResponse(501, { error: "Not supported by the browser file host." });

export type FsAccessHost = RunebenderHost & {
  // The workspace stored from a previous visit, if any — primed by
  // primeStoredWorkspace() at boot; drives the "Reopen <name>" UI.
  storedWorkspaceName(): string | null;
  primeStoredWorkspace(): Promise<{
    name: string;
    permission: PermissionState;
  } | null>;
  // Auto-open without a gesture; only succeeds when the user chose
  // "Allow on every visit" on an earlier visit (permission already
  // granted for the stored handle).
  reopenStoredWorkspaceSilently(): Promise<string | null>;
};

export function createFsAccessHost(options: {
  // Called with the slot name once a directory is open; main.ts points
  // the editor's fontPathRef at it, which triggers the workspace load.
  onWorkspaceOpened: (slot: string) => void;
}): FsAccessHost {
  let root: FileSystemDirectoryHandle | null = null;
  let slot = "";
  // root-relative path -> last-seen state (the ETag equivalent)
  const files = new Map<string, { hash: string; mtime: number }>();

  let stored: { handle: FileSystemDirectoryHandle; name: string } | null =
    null;
  // A .designspace picked via pickSourceFile, waiting for its folder
  // grant (grantSourceFolder starts the directory picker here).
  let pendingSourceFile: FileSystemFileHandle | null = null;
  // Recents as last listed — openRecentWorkspace indexes into this.
  let recents: RecentEntry[] = [];
  let watchHandler:
    | ((
        changes: WorkspaceExternalChange[],
      ) => void | string[] | Promise<void | string[]>)
    | null = null;
  let watching = false;
  let pollTimer: ReturnType<typeof setInterval> | null = null;
  let observer: InstanceType<ObserverCtor> | null = null;

  const stripSlot = (p: string) =>
    p.startsWith(`${slot}/`) ? p.slice(slot.length + 1) : p;

  async function getDirectory(
    rel: string[],
    create: boolean,
  ): Promise<FileSystemDirectoryHandle | null> {
    if (!root) return null;
    let dir: FileSystemDirectoryHandle = root;
    for (const part of rel) {
      try {
        dir = await dir.getDirectoryHandle(part, { create });
      } catch {
        return null;
      }
    }
    return dir;
  }

  async function getFileHandle(
    rel: string,
    { create = false } = {},
  ): Promise<FileSystemFileHandle | null> {
    const parts = rel.split("/");
    const name = parts.pop();
    if (!name) return null;
    const dir = await getDirectory(parts, create);
    if (!dir) return null;
    try {
      return await dir.getFileHandle(name, { create });
    } catch {
      return null;
    }
  }

  async function readFile(
    rel: string,
  ): Promise<{ text: string; hash: string; mtime: number } | null> {
    const handle = await getFileHandle(rel);
    if (!handle) return null;
    try {
      const file = await handle.getFile();
      const text = await file.text();
      return { text, hash: await sha256Hex(text), mtime: file.lastModified };
    } catch {
      return null;
    }
  }

  async function walkTextFiles(
    dir: FileSystemDirectoryHandle,
    prefix: string,
    out: { rel: string; handle: FileSystemFileHandle }[],
  ): Promise<void> {
    for await (const [name, entry] of (dir as IterableDirectoryHandle).entries()) {
      const rel = prefix ? `${prefix}/${name}` : name;
      if (entry.kind === "file") {
        if (isTextPath(name)) {
          out.push({ rel, handle: entry as FileSystemFileHandle });
        }
      } else if (!shouldSkipDir(name)) {
        await walkTextFiles(entry as FileSystemDirectoryHandle, rel, out);
      }
    }
  }

  // External changes are processed one at a time: an agent rewriting
  // fifty glifs lands as fifty ordered handler calls, never interleaved
  // reads of a half-applied batch.
  let changeQueue: Promise<void> = Promise.resolve();
  function enqueueChange(rel: string, kind: "change" | "delete") {
    changeQueue = changeQueue.then(() => processChange(rel, kind)).catch((e) => {
      console.warn(`external change for ${rel} failed:`, e);
    });
  }

  async function processChange(rel: string, kind: "change" | "delete") {
    if (!watchHandler || !isTextPath(rel)) return;
    const prefixed = `${slot}/${rel}`;
    if (kind === "delete") {
      // Deletion events also fire mid-way through atomic replaces
      // (write temp + swap); only report files that are really gone.
      if (await getFileHandle(rel)) return;
      files.delete(rel);
      await watchHandler([{ type: "delete", path: prefixed }]);
      return;
    }
    const got = await readFile(rel);
    if (!got) return;
    const known = files.get(rel);
    if (known?.hash === got.hash) {
      // Our own write echoing back, or a touch without content change.
      files.set(rel, { hash: got.hash, mtime: got.mtime });
      return;
    }
    const applied = await watchHandler([
      { type: "change", path: prefixed, text: got.text },
    ]);
    // Commit the hash only for changes the editor applied; a held-back
    // change keeps the stale hash so the next save 409s.
    if (Array.isArray(applied) && applied.includes(prefixed)) {
      files.set(rel, { hash: got.hash, mtime: got.mtime });
    }
  }

  async function startWatching() {
    if (watching || !root || !watchHandler) return;
    watching = true;
    const Ctor = (globalThis as { FileSystemObserver?: ObserverCtor })
      .FileSystemObserver;
    if (Ctor) {
      observer = new Ctor((records) => {
        for (const record of records) {
          const rel = record.relativePathComponents.join("/");
          if (record.type === "appeared" || record.type === "modified") {
            enqueueChange(rel, "change");
          } else if (record.type === "disappeared") {
            enqueueChange(rel, "delete");
          } else if (record.type === "moved") {
            const from = record.relativePathMovedFrom?.join("/");
            if (from) enqueueChange(from, "delete");
            enqueueChange(rel, "change");
          } else {
            // "unknown"/"errored": the observer lost track; re-scan.
            void pollOnce();
          }
        }
      });
      try {
        await observer.observe(root, { recursive: true });
        return;
      } catch (e) {
        console.warn("FileSystemObserver failed, falling back to polling:", e);
        observer = null;
      }
    }
    pollTimer = setInterval(() => void pollOnce(), POLL_INTERVAL_MS);
  }

  let polling = false;
  async function pollOnce() {
    if (polling || !root) return;
    polling = true;
    try {
      const found: { rel: string; handle: FileSystemFileHandle }[] = [];
      await walkTextFiles(root, "", found);
      const seen = new Set<string>();
      for (const { rel, handle } of found) {
        seen.add(rel);
        const known = files.get(rel);
        try {
          const file = await handle.getFile();
          if (!known || file.lastModified !== known.mtime) {
            enqueueChange(rel, "change");
          }
        } catch {
          // Transient read failure; the next poll retries.
        }
      }
      for (const rel of files.keys()) {
        if (!seen.has(rel)) enqueueChange(rel, "delete");
      }
    } finally {
      polling = false;
    }
  }

  async function adoptDirectory(
    handle: FileSystemDirectoryHandle,
    { persist = true } = {},
  ): Promise<string> {
    stopWatching();
    root = handle;
    slot = handle.name;
    files.clear();
    if (persist) {
      try {
        await saveWorkspaceHandle(handle);
        stored = { handle, name: handle.name };
      } catch (e) {
        console.warn("could not persist workspace handle:", e);
      }
    }
    void addRecentWorkspace(handle, "folder");
    options.onWorkspaceOpened(slot);
    return slot;
  }

  function stopWatching() {
    watching = false;
    observer?.disconnect();
    observer = null;
    if (pollTimer !== null) {
      clearInterval(pollTimer);
      pollTimer = null;
    }
  }

  return {
    log(level, message) {
      if (level === "error") console.error(message);
      else console.info(message);
    },

    async publishState() {
      // No graph node to mirror state into.
    },

    storedWorkspaceName() {
      return stored?.name ?? null;
    },

    async primeStoredWorkspace() {
      stored = await loadWorkspaceHandle();
      if (!stored) return null;
      const handle = stored.handle as PermissionCapableHandle;
      const permission =
        (await handle.queryPermission?.({ mode: "readwrite" })) ?? "prompt";
      return { name: stored.name, permission };
    },

    async reopenStoredWorkspaceSilently() {
      if (!stored) return null;
      const handle = stored.handle as PermissionCapableHandle;
      const state = await handle.queryPermission?.({ mode: "readwrite" });
      if (state !== "granted") return null;
      return adoptDirectory(stored.handle, { persist: false });
    },

    async openWorkspaceFolder() {
      const picker = (window as DirectoryPickerWindow).showDirectoryPicker;
      if (!picker) return { error: "File System Access is not supported." };
      try {
        const handle = await picker({
          id: "runebender-font",
          mode: "readwrite",
        });
        return { slot: await adoptDirectory(handle) };
      } catch (e) {
        if ((e as DOMException).name === "AbortError") {
          return { cancelled: true };
        }
        return { error: String(e) };
      }
    },

    async pickSourceFile() {
      const picker = (window as DirectoryPickerWindow).showOpenFilePicker;
      if (!picker) return { error: "File System Access is not supported." };
      try {
        const [handle] = await picker({
          id: "runebender-font",
          types: [
            {
              description: "Font source (.designspace, .glyphs)",
              accept: {
                "text/plain": [".designspace", ".glyphs"],
              },
            },
          ],
          excludeAcceptAllOption: false,
        });
        const name = handle.name.toLowerCase();
        if (name.endsWith(".glyphs")) {
          pendingSourceFile = null;
          void addRecentWorkspace(handle, "file");
          return { kind: "glyphs" as const, file: await handle.getFile() };
        }
        pendingSourceFile = handle;
        return { kind: "designspace" as const, name: handle.name };
      } catch (e) {
        if ((e as DOMException).name === "AbortError") {
          return { cancelled: true };
        }
        return { error: String(e) };
      }
    },

    async grantSourceFolder() {
      const picker = (window as DirectoryPickerWindow).showDirectoryPicker;
      if (!picker) return { error: "File System Access is not supported." };
      const source = pendingSourceFile;
      if (!source) return { error: "No source file is pending." };
      try {
        // startIn: the picked file's handle — the dialog opens in the
        // directory that CONTAINS it, so Select is one click.
        const handle = await picker({
          id: "runebender-font",
          mode: "readwrite",
          startIn: source,
        });
        // Guard against picking some unrelated folder. The file may
        // sit deeper than the granted root (picking a parent folder
        // is fine), so search the tree instead of only direct children.
        const found: { rel: string; handle: FileSystemFileHandle }[] = [];
        await walkTextFiles(handle, "", found);
        if (!found.some(({ rel }) => rel.split("/").pop() === source.name)) {
          return {
            error: `${handle.name} does not contain ${source.name} — pick that file's folder (or a folder above it)`,
          };
        }
        pendingSourceFile = null;
        return { slot: await adoptDirectory(handle) };
      } catch (e) {
        if ((e as DOMException).name === "AbortError") {
          return { cancelled: true };
        }
        return { error: String(e) };
      }
    },

    async listRecentWorkspaces() {
      recents = await loadRecentWorkspaces();
      return recents.map((entry, index) => ({
        index,
        name: entry.name,
        kind: entry.kind,
      }));
    },

    async openRecentWorkspace(index: number) {
      const entry = recents[index];
      if (!entry) return { error: "That recent entry is gone." };
      const handle = entry.handle as PermissionCapableHandle;
      const mode = entry.kind === "folder" ? "readwrite" : "read";
      try {
        const state =
          (await handle.requestPermission?.({ mode: mode as "readwrite" })) ??
          "denied";
        if (state !== "granted") return { cancelled: true };
        if (entry.kind === "file") {
          void addRecentWorkspace(entry.handle, "file");
          return {
            file: await (entry.handle as FileSystemFileHandle).getFile(),
          };
        }
        return {
          slot: await adoptDirectory(
            entry.handle as FileSystemDirectoryHandle,
          ),
        };
      } catch (e) {
        return { error: String(e) };
      }
    },

    async reopenStoredWorkspace() {
      if (!stored) return { error: "No stored workspace." };
      const handle = stored.handle as PermissionCapableHandle;
      // Inside a user gesture this shows Chrome's three-way prompt,
      // including "Allow on every visit" — the persistent grant that
      // makes the next visit reopen silently.
      const state =
        (await handle.requestPermission?.({ mode: "readwrite" })) ?? "denied";
      if (state !== "granted") return { cancelled: true };
      return { slot: await adoptDirectory(stored.handle, { persist: false }) };
    },

    async loadWorkspaceSlot(): Promise<WorkspaceSlotPayload | null> {
      if (!root) return null;
      const found: { rel: string; handle: FileSystemFileHandle }[] = [];
      await walkTextFiles(root, "", found);
      files.clear();
      const entries = await mapConcurrent(found, 32, async ({ rel, handle }) => {
        try {
          const file = await handle.getFile();
          const text = await file.text();
          files.set(rel, {
            hash: await sha256Hex(text),
            mtime: file.lastModified,
          });
          return { path: rel, text };
        } catch {
          return null;
        }
      });
      void startWatching();
      // Surface WHAT is being edited, not just the folder: the
      // shallowest designspace (folders like archive/ hold retired
      // ones), else the first .ufo directory.
      const paths = found.map(({ rel }) => rel);
      const byDepth = (a: string, b: string) =>
        a.split("/").length - b.split("/").length || a.localeCompare(b);
      const entry =
        paths.filter((p) => p.endsWith(".designspace")).sort(byDepth)[0] ??
        paths
          .filter((p) => p.includes(".ufo/"))
          .map((p) => p.slice(0, p.indexOf(".ufo/") + 4))
          .sort(byDepth)[0] ??
        null;
      const displaySource = entry ? `${slot}/${entry}` : slot;
      return {
        slot,
        files: entries.filter(
          (e): e is { path: string; text: string } => e !== null,
        ),
        linked_source: true,
        origin_root: slot,
        origin_source: entry ?? slot,
        display_root: slot,
        display_source: displaySource,
      };
    },

    async listWorkspaceSlots() {
      return root ? [{ slot, label: slot }] : [];
    },

    async clearWorkspaceSlots() {
      await clearWorkspaceHandle();
      stored = null;
      return { deleted: [] };
    },

    workspacePreviewUrl() {
      return "";
    },

    async drawBotPresetSource() {
      return null;
    },

    async writeWorkspaceFile(path, text) {
      if (!root) return unavailable();
      const rel = stripSlot(path);
      const known = files.get(rel);
      const onDisk = await readFile(rel);

      if (!known) {
        if (onDisk) {
          if (onDisk.text === text) {
            // No-op write of identical content: adopt the state quietly.
            files.set(rel, { hash: onDisk.hash, mtime: onDisk.mtime });
            return jsonResponse(200, { etag: onDisk.hash });
          }
          // Never read this file, so we have no right to overwrite it.
          console.error(
            `refusing to overwrite never-read file: ${rel} (reload the workspace)`,
          );
          return jsonResponse(409, {
            error: "refusing to overwrite never-read file",
          });
        }
      } else if (onDisk && onDisk.hash !== known.hash) {
        // Changed on disk since we last read it (an agent or another
        // tool wrote it) — same conflict the server signals via If-Match.
        return jsonResponse(409, { error: "file changed on disk" });
      } else if (!onDisk) {
        // We had read it but it vanished; recreate rather than fail the
        // user's save.
      }

      const handle = await getFileHandle(rel, { create: true });
      if (!handle) return jsonResponse(500, { error: `cannot open ${rel}` });
      try {
        const writable = await handle.createWritable();
        await writable.write(text);
        await writable.close();
      } catch (e) {
        return jsonResponse(500, { error: String(e) });
      }
      const file = await handle.getFile();
      const hash = await sha256Hex(text);
      files.set(rel, { hash, mtime: file.lastModified });
      return jsonResponse(200, { etag: hash });
    },

    async chooseSource() {
      return { cancelled: true };
    },

    async linkSource() {
      return {
        response: unavailable(),
        data: { error: "Not supported by the browser file host." },
      };
    },

    async saveWorkspaceAs() {
      return {
        response: unavailable(),
        data: { error: "Not supported by the browser file host." },
      };
    },

    async traceBackgroundGlyph() {
      return {
        response: unavailable(),
        data: { error: "Background tracing requires the local workspace server." },
      };
    },

    async traceBackgroundCandidate() {
      return {
        response: unavailable(),
        data: { error: "Background tracing requires the local workspace server." },
      };
    },

    async invalidateWorkspacePath() {
      // No compiled cache exists in the browser file host.
    },

    watchWorkspaceChanges(handler) {
      watchHandler = handler;
      void startWatching();
    },
  };
}
