#!/usr/bin/env node
// Local workspace server for the Runebender web editor.
//
//   node server/serve.mjs <font-or-dir> [--port N] [--open]
//
// Serves the built editor (dist/) plus a small file API over a font
// workspace on disk, so the browser editor can load and save real UFO
// sources while other programs — AI coding agents, mainly — edit the
// same files. Designed for the "Claude edits the font on disk while
// the editor is open" loop:
//
//   - Every file read returns an ETag (sha-256 of content); writes
//     require If-Match so a stale editor can never silently clobber a
//     file an agent just changed (409 instead).
//   - A recursive file watcher pushes change events to the editor over
//     SSE, with self-write suppression (mtime + content hash recorded
//     at write time) so the editor's own saves don't echo back as
//     external changes. The suppression scheme follows Fontra's
//     FileWatcher.ignoreNextChange.
//
// The server is deliberately dumb storage: all UFO semantics live in
// the editor's wasm core. Zero npm dependencies.

import { createServer } from "node:http";
import { createHash } from "node:crypto";
import { spawn } from "node:child_process";
import fs from "node:fs";
import fsp from "node:fs/promises";
import path from "node:path";
import { fileURLToPath } from "node:url";

const HERE = path.dirname(fileURLToPath(import.meta.url));
const DIST = path.resolve(HERE, "..", "dist");

const MIME = {
  ".html": "text/html; charset=utf-8",
  ".js": "text/javascript",
  ".mjs": "text/javascript",
  ".css": "text/css",
  ".json": "application/json",
  ".wasm": "application/wasm",
  ".png": "image/png",
  ".svg": "image/svg+xml",
  ".woff2": "font/woff2",
  ".glif": "application/xml",
  ".plist": "application/xml",
  ".designspace": "application/xml",
  ".fea": "text/plain",
};

// ---------------------------------------------------------------- CLI

function usage(msg) {
  if (msg) console.error(`error: ${msg}\n`);
  console.error(
    "usage: runebender-serve <font-or-dir> [--port N] [--open]\n\n" +
      "  <font-or-dir>  a .ufo directory, a .designspace file, or a\n" +
      "                 directory containing them\n" +
      "  --port N       listen port (default 8765)\n" +
      "  --open         open the editor in the default browser",
  );
  process.exit(msg ? 1 : 0);
}

const args = process.argv.slice(2);
let target = null;
let port = 8765;
let openBrowser = false;
for (let i = 0; i < args.length; i++) {
  const a = args[i];
  if (a === "--port") port = Number(args[++i]);
  else if (a === "--open") openBrowser = true;
  else if (a === "--help" || a === "-h") usage();
  else if (!target) target = a;
  else usage(`unexpected argument: ${a}`);
}
if (!target) usage("missing font path");
if (!Number.isInteger(port) || port <= 0) usage("bad --port");

// Resolve the workspace root and the entry source within it. A .ufo or
// .designspace target serves its PARENT directory as the root, so a
// designspace's sibling UFOs are reachable and agents' edits anywhere
// in the project are visible.
const targetAbs = path.resolve(target);
let stat;
try {
  stat = fs.statSync(targetAbs);
} catch {
  usage(`no such path: ${target}`);
}
let root, entry;
if (targetAbs.endsWith(".ufo") && stat.isDirectory()) {
  root = path.dirname(targetAbs);
  entry = path.basename(targetAbs);
} else if (targetAbs.endsWith(".designspace") && stat.isFile()) {
  root = path.dirname(targetAbs);
  entry = path.basename(targetAbs);
} else if (stat.isDirectory()) {
  root = targetAbs;
  const names = fs.readdirSync(targetAbs);
  entry =
    names.find((n) => n.endsWith(".designspace")) ??
    names.find((n) => n.endsWith(".ufo")) ??
    null;
  if (!entry) usage(`no .designspace or .ufo found in ${target}`);
} else {
  usage(`not a .ufo, .designspace, or directory: ${target}`);
}

// ------------------------------------------------------------- helpers

const sha256 = (buf) => createHash("sha256").update(buf).digest("hex");

/** Resolve a relative API path inside root, rejecting escapes. */
function safeJoin(rel) {
  const abs = path.resolve(root, rel);
  if (abs !== root && !abs.startsWith(root + path.sep)) return null;
  return abs;
}

const SKIP_DIRS = new Set(["node_modules", ".git", "__pycache__"]);

async function* walk(dir) {
  let entries;
  try {
    entries = await fsp.readdir(dir, { withFileTypes: true });
  } catch {
    return;
  }
  for (const e of entries) {
    if (e.name.startsWith(".") || SKIP_DIRS.has(e.name)) continue;
    const abs = path.join(dir, e.name);
    if (e.isDirectory()) yield* walk(abs);
    else if (e.isFile()) yield abs;
  }
}

function send(res, status, body, headers = {}) {
  res.writeHead(status, headers);
  res.end(body);
}

function sendJson(res, status, obj, headers = {}) {
  send(res, status, JSON.stringify(obj), {
    "content-type": "application/json",
    ...CORS,
    ...headers,
  });
}

// The editor may run on a different origin during development
// (vite :5173) or, later, from runebender.org against a local server —
// so the API is permissive-CORS. It binds to 127.0.0.1 only.
const CORS = {
  "access-control-allow-origin": "*",
  "access-control-allow-methods": "GET, PUT, DELETE, OPTIONS",
  "access-control-allow-headers": "content-type, if-match",
  "access-control-expose-headers": "etag, last-modified",
};

// ------------------------------------------- self-write suppression

// path -> { hash, mtimeMs, at }  recorded when WE write, so the file
// watcher can tell our writes from external ones. Entries are pruned
// after a generous window; matching is by content hash first (a no-op
// rewrite by an agent also produces zero churn, like Fontra).
const selfWrites = new Map();
const SELF_WRITE_TTL_MS = 10_000;

function recordSelfWrite(abs, buf, mtimeMs) {
  selfWrites.set(abs, { hash: sha256(buf), mtimeMs, at: Date.now() });
}

function isSelfWrite(abs, buf, mtimeMs) {
  const rec = selfWrites.get(abs);
  if (!rec) return false;
  if (Date.now() - rec.at > SELF_WRITE_TTL_MS) {
    selfWrites.delete(abs);
    return false;
  }
  if (buf !== null && sha256(buf) === rec.hash) return true;
  if (mtimeMs !== null && Math.abs(mtimeMs - rec.mtimeMs) < 1) return true;
  return false;
}

// ------------------------------------------------------ SSE + watcher

const sseClients = new Set();

function broadcast(event) {
  const line = `data: ${JSON.stringify(event)}\n\n`;
  for (const res of sseClients) res.write(line);
}

// Debounce-and-settle per path: agents often write bursts (a .glif
// then contents.plist), and editors write-then-rename. Collect events
// for a short quiet period before classifying, which also dodges the
// half-written-file race Fontra papers over with sleep(0.15).
const pending = new Map(); // relPath -> timer
const DEBOUNCE_MS = 200;

function onFsEvent(relPath) {
  if (!relPath) return;
  const rel = relPath.split(path.sep).join("/");
  const parts = rel.split("/");
  if (parts.some((p) => p.startsWith(".") || SKIP_DIRS.has(p))) return;
  clearTimeout(pending.get(rel));
  pending.set(
    rel,
    setTimeout(() => {
      pending.delete(rel);
      void classifyAndBroadcast(rel);
    }, DEBOUNCE_MS),
  );
}

async function classifyAndBroadcast(rel) {
  const abs = safeJoin(rel);
  if (!abs) return;
  let buf = null;
  let mtimeMs = null;
  try {
    const st = await fsp.stat(abs);
    if (st.isDirectory()) return; // directory-level events are noise
    mtimeMs = st.mtimeMs;
    buf = await fsp.readFile(abs);
  } catch {
    broadcast({ type: "delete", path: rel });
    return;
  }
  if (isSelfWrite(abs, buf, mtimeMs)) return;
  broadcast({ type: "change", path: rel, etag: sha256(buf) });
}

try {
  fs.watch(root, { recursive: true }, (_eventType, filename) => {
    onFsEvent(filename);
  });
} catch (err) {
  console.warn(
    `file watching unavailable (${err.message}) — external changes ` +
      "won't live-reload",
  );
}

// --------------------------------------------------------------- API

async function handleApi(req, res, url) {
  if (req.method === "OPTIONS") return send(res, 204, null, CORS);

  if (url.pathname === "/runebender/api/info") {
    return sendJson(res, 200, {
      server: "runebender-serve",
      root: path.basename(root),
      rootPath: root,
      entry,
      entryPath: entry ? path.join(root, entry) : root,
      readOnly: false,
    });
  }

  if (url.pathname === "/runebender/api/files") {
    const prefix = url.searchParams.get("prefix") ?? "";
    const files = [];
    for await (const abs of walk(root)) {
      const rel = path.relative(root, abs).split(path.sep).join("/");
      if (!rel.startsWith(prefix)) continue;
      const st = await fsp.stat(abs);
      files.push({ path: rel, size: st.size, mtimeMs: st.mtimeMs });
    }
    return sendJson(res, 200, { files });
  }

  if (url.pathname === "/runebender/api/events") {
    res.writeHead(200, {
      "content-type": "text/event-stream",
      "cache-control": "no-cache",
      connection: "keep-alive",
      ...CORS,
    });
    res.write(`data: ${JSON.stringify({ type: "hello" })}\n\n`);
    sseClients.add(res);
    const ping = setInterval(() => res.write(": ping\n\n"), 30_000);
    req.on("close", () => {
      clearInterval(ping);
      sseClients.delete(res);
    });
    return;
  }

  const filePrefix = "/runebender/api/file/";
  if (url.pathname.startsWith(filePrefix)) {
    const rel = decodeURIComponent(url.pathname.slice(filePrefix.length));
    const abs = safeJoin(rel);
    if (!abs) return sendJson(res, 400, { error: "path escapes root" });

    if (req.method === "GET") {
      let buf;
      try {
        buf = await fsp.readFile(abs);
      } catch {
        return sendJson(res, 404, { error: "not found" });
      }
      const ext = path.extname(abs).toLowerCase();
      return send(res, 200, buf, {
        "content-type": MIME[ext] ?? "application/octet-stream",
        etag: `"${sha256(buf)}"`,
        ...CORS,
      });
    }

    if (req.method === "PUT") {
      const ifMatch = req.headers["if-match"];
      if (!ifMatch) {
        return sendJson(res, 428, {
          error: "If-Match required (use * to create)",
        });
      }
      let current = null;
      try {
        current = await fsp.readFile(abs);
      } catch {
        /* new file */
      }
      const currentTag = current ? sha256(current) : null;
      const wanted = ifMatch.replaceAll('"', "").trim();
      // "*" means CREATE — it must never overwrite an existing file. A
      // client that wants to overwrite must present the current ETag,
      // i.e. prove it has read what it is replacing.
      const matches =
        wanted === "*"
          ? current === null
          : current !== null && wanted === currentTag;
      if (!matches) {
        return sendJson(
          res,
          409,
          {
            error:
              wanted === "*"
                ? "exists; If-Match:* only creates — read the file first"
                : current === null
                  ? "file does not exist"
                  : "file changed on disk",
            etag: currentTag,
          },
          currentTag ? { etag: `"${currentTag}"` } : {},
        );
      }
      const chunks = [];
      for await (const c of req) chunks.push(c);
      const body = Buffer.concat(chunks);
      // Vandalism guard: replacing a glyph's non-empty outline with an
      // empty one is almost always a client bug (stale or never-loaded
      // model), not a design decision. Deliberate clears must say so.
      if (
        abs.endsWith(".glif") &&
        current &&
        current.includes("<outline>") &&
        !body.includes("<outline>") &&
        req.headers["x-allow-outline-clear"] !== "1"
      ) {
        return sendJson(res, 409, {
          error:
            "refusing to clear a non-empty outline (send x-allow-outline-clear: 1 if deliberate)",
          etag: currentTag,
        });
      }
      await fsp.mkdir(path.dirname(abs), { recursive: true });
      await fsp.writeFile(abs, body);
      const st = await fsp.stat(abs);
      recordSelfWrite(abs, body, st.mtimeMs);
      return sendJson(res, 200, { etag: sha256(body) });
    }

    if (req.method === "DELETE") {
      try {
        await fsp.unlink(abs);
        selfWrites.set(abs, {
          hash: null,
          mtimeMs: null,
          at: Date.now(),
        });
      } catch {
        return sendJson(res, 404, { error: "not found" });
      }
      return sendJson(res, 200, { ok: true });
    }
  }

  return sendJson(res, 404, { error: "unknown API route" });
}

// ------------------------------------------------------ static editor

async function handleStatic(req, res, url) {
  let rel = decodeURIComponent(url.pathname);
  if (rel.endsWith("/")) rel += "index.html";
  const abs = path.resolve(DIST, "." + rel);
  if (!abs.startsWith(DIST)) return send(res, 400, "bad path");
  let buf;
  try {
    buf = await fsp.readFile(abs);
  } catch {
    return send(res, 404, "not found (is dist/ built? run: pnpm build)");
  }
  const ext = path.extname(abs).toLowerCase();
  send(res, 200, buf, {
    "content-type": MIME[ext] ?? "application/octet-stream",
  });
}

// --------------------------------------------------------------- main

const server = createServer((req, res) => {
  const url = new URL(req.url, `http://127.0.0.1:${port}`);
  const handler = url.pathname.startsWith("/runebender/api/")
    ? handleApi
    : handleStatic;
  handler(req, res, url).catch((err) => {
    console.error(err);
    if (!res.headersSent) sendJson(res, 500, { error: String(err) });
    else res.end();
  });
});

server.listen(port, "127.0.0.1", () => {
  const url = `http://localhost:${port}/`;
  console.log(`runebender-serve`);
  console.log(`  workspace  ${root}`);
  console.log(`  entry      ${entry}`);
  console.log(`  editor     ${url}`);
  if (openBrowser && process.platform === "darwin") {
    spawn("open", [url], { stdio: "ignore", detached: true }).unref();
  }
});
