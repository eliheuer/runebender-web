// RunebenderHost backed by the local workspace server (server/serve.mjs).
//
// The server is dumb file storage over a font workspace on disk; all
// UFO semantics stay in the editor. This host adds the safety layer
// for agent-concurrent editing:
//
//   - Every file read records the server's content-hash ETag; every
//     write sends it back as If-Match, so a stale editor gets a 409
//     instead of silently clobbering what an agent just wrote.
//   - watchWorkspaceChanges subscribes to the server's SSE stream and
//     hands externally changed file contents to the editor. The server
//     already suppresses this editor's own writes (mtime + hash), so
//     everything arriving here is genuinely external.

import type {
  RunebenderHost,
  WorkspaceExternalChange,
  WorkspaceSlotPayload,
} from "../../host/runebenderHost";

export type LocalServerInfo = {
  server: string;
  root: string;
  rootPath?: string;
  entry: string | null;
  entryPath?: string | null;
  readOnly: boolean;
};

// The editor's workspace payloads are text files; UFO binaries
// (images/, data/) are left alone on disk.
const TEXT_EXTENSIONS = [
  ".glif",
  ".plist",
  ".designspace",
  ".fea",
  ".xml",
  ".json",
  ".txt",
];

const isTextPath = (p: string) =>
  TEXT_EXTENSIONS.some((ext) => p.endsWith(ext));

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

const unavailable = () =>
  new Response(
    JSON.stringify({ error: "Not supported by the local workspace server." }),
    {
      status: 501,
      statusText: "Not Implemented",
      headers: { "content-type": "application/json" },
    },
  );

export function createLocalHost(
  info: LocalServerInfo,
  base = "",
): RunebenderHost {
  const api = (p: string) => `${base}/runebender/api/${p}`;
  const fileUrl = (rel: string) =>
    api(`file/${rel.split("/").map(encodeURIComponent).join("/")}`);

  // The editor prefixes workspace paths with the slot name; the server
  // speaks paths relative to its root. The slot IS the root's label.
  const slot = info.root;
  const stripSlot = (p: string) =>
    p.startsWith(`${slot}/`) ? p.slice(slot.length + 1) : p;

  // server-relative path -> last seen content ETag
  const etags = new Map<string, string>();

  // `record: false` fetches without committing the ETag — used by the
  // watch path, where the ETag must only advance if the editor actually
  // applies the change (a held-back change keeps the stale ETag so the
  // user's next save 409s instead of clobbering the external edit).
  async function fetchFile(
    rel: string,
    { record = true } = {},
  ): Promise<{ text: string; etag: string | null } | null> {
    const res = await fetch(fileUrl(rel));
    if (!res.ok) return null;
    const etag = res.headers.get("etag")?.replaceAll('"', "") ?? null;
    if (record && etag) etags.set(rel, etag);
    return { text: await res.text(), etag };
  }

  return {
    log(level, message) {
      if (level === "error") console.error(message);
      else console.info(message);
    },

    async publishState() {
      // No graph node to mirror state into.
    },

    async loadWorkspaceSlot(): Promise<WorkspaceSlotPayload | null> {
      const listRes = await fetch(api("files"));
      if (!listRes.ok) return null;
      const { files } = (await listRes.json()) as {
        files: { path: string }[];
      };
      const wanted = files.filter((f) => isTextPath(f.path));
      const entries = await mapConcurrent(
        wanted,
        32,
        async (f) => {
          const got = await fetchFile(f.path);
          return got === null ? null : { path: f.path, text: got.text };
        },
      );
      return {
        slot,
        files: entries.filter(
          (e): e is { path: string; text: string } => e !== null,
        ),
        linked_source: true,
        origin_root: info.root,
        origin_source: info.entry ?? info.root,
        display_root: info.rootPath ?? info.root,
        display_source: info.entryPath ?? info.entry ?? info.rootPath ?? info.root,
      };
    },

    async listWorkspaceSlots() {
      return [{ slot, label: info.entry ?? slot }];
    },

    async clearWorkspaceSlots() {
      return { deleted: [] };
    },

    workspacePreviewUrl() {
      return "";
    },

    async drawBotPresetSource() {
      return null;
    },

    async writeWorkspaceFile(path, text) {
      const rel = stripSlot(path);
      const known = etags.get(rel);
      const res = await fetch(fileUrl(rel), {
        method: "PUT",
        headers: { "if-match": known ? `"${known}"` : "*" },
        body: text,
      });
      if (res.ok) {
        const data = (await res
          .clone()
          .json()
          .catch(() => null)) as { etag?: string } | null;
        if (data?.etag) etags.set(rel, data.etag);
      }
      return res;
    },

    async chooseSource() {
      return { cancelled: true };
    },

    async linkSource() {
      return {
        response: unavailable(),
        data: { error: "Not supported by the local workspace server." },
      };
    },

    async saveWorkspaceAs() {
      return {
        response: unavailable(),
        data: { error: "Not supported by the local workspace server." },
      };
    },

    async traceBackgroundGlyph() {
      return {
        response: unavailable(),
        data: { error: "Background tracing requires the ComfyUI host." },
      };
    },

    async traceBackgroundCandidate() {
      return {
        response: unavailable(),
        data: { error: "Background tracing requires the ComfyUI host." },
      };
    },

    async invalidateWorkspacePath() {
      // The local server keeps no compiled cache.
    },

    watchWorkspaceChanges(handler) {
      const source = new EventSource(api("events"));
      source.onmessage = async (ev) => {
        let data: { type?: string; path?: string };
        try {
          data = JSON.parse(ev.data);
        } catch {
          return;
        }
        if (!data.path || !isTextPath(data.path)) return;
        const rel = data.path;
        const prefixed = `${slot}/${rel}`;
        if (data.type === "change") {
          const got = await fetchFile(rel, { record: false });
          if (!got) return;
          const applied = await handler([
            { type: "change", path: prefixed, text: got.text },
          ]);
          // Commit the ETag only for changes the editor applied; a
          // held-back change keeps the stale one (see fetchFile).
          if (Array.isArray(applied) && applied.includes(prefixed) && got.etag) {
            etags.set(rel, got.etag);
          }
        } else if (data.type === "delete") {
          etags.delete(rel);
          void handler([{ type: "delete", path: prefixed }]);
        }
      };
      source.onerror = () => {
        // EventSource auto-reconnects; nothing to do.
      };
    },
  };
}
