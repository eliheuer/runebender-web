#!/usr/bin/env node
// Write `com.runebender.markLabel` into every marked glyph of a UFO.
//
// UFO stores a mark as a colour, so a palette change orphans the file:
// the glyph still says 0.09,0.72,0.44 long after green stopped meaning
// that. Runebender writes the label alongside the colour from now on,
// and falls back to snapping an unlabelled colour to the nearest hue —
// but snapping is a guess, and this is how you replace it with a fact.
//
// It is deliberately a two-step tool. A dry run prints every distinct
// colour in the source, how many glyphs carry it, and the label it
// would get; you check that against what those marks mean to you, and
// only then write. Two colours proposing the same label is reported as
// a collision, because that is exactly where a guess would merge groups
// you keep apart.
//
// Usage:
//   node scripts/stamp-mark-labels.mjs <path/to.ufo | path/to/sources>
//   node scripts/stamp-mark-labels.mjs <path> --map 0,0.67,0.91,1=teal
//   node scripts/stamp-mark-labels.mjs <path> --map ... --write

import { readFileSync, writeFileSync, readdirSync, statSync } from "node:fs";
import { join } from "node:path";
import { pathToFileURL } from "node:url";

const MARK_LABEL_KEY = "com.runebender.markLabel";

const args = process.argv.slice(2);
const write = args.includes("--write");
const roots = args.filter((a) => !a.startsWith("--"));
const overrides = new Map();
for (let i = 0; i < args.length; i += 1) {
  if (args[i] !== "--map") continue;
  const [rgba, label] = (args[i + 1] ?? "").split("=");
  if (rgba && label) overrides.set(rgba.trim(), label.trim());
}

if (roots.length === 0) {
  console.error("usage: stamp-mark-labels.mjs <ufo or directory> [--map rgba=label] [--write]");
  process.exit(1);
}

const { labelForRgba } = await import(
  pathToFileURL(new URL("../src/components/markColors.ts", import.meta.url).pathname).href
).catch(() => ({ labelForRgba: null }));

// markColors.ts is TypeScript, so when it cannot be imported directly
// the same hue maths lives here. Keep the two in step.
const fallbackLabelForRgba = (() => {
  const hues = { red: 27, orange: 46, yellow: 96, green: 152, teal: 186, blue: 248, purple: 292, pink: 350 };
  const toLinear = (v) => (v <= 0.04045 ? v / 12.92 : ((v + 0.055) / 1.055) ** 2.4);
  return (rgba) => {
    const [r0, g0, b0] = rgba.split(",").map(Number);
    if ([r0, g0, b0].some((n) => !Number.isFinite(n))) return null;
    const r = toLinear(r0), g = toLinear(g0), b = toLinear(b0);
    const l = Math.cbrt(0.4122214708 * r + 0.5363325363 * g + 0.0514459929 * b);
    const m = Math.cbrt(0.2119034982 * r + 0.6806995451 * g + 0.1073969566 * b);
    const s = Math.cbrt(0.0883024619 * r + 0.2817188376 * g + 0.6299787005 * b);
    const a = 1.9779984951 * l - 2.428592205 * m + 0.4505937099 * s;
    const bb = 0.0259040371 * l + 0.7827717662 * m - 0.808675766 * s;
    if (Math.hypot(a, bb) < 0.03) return null;
    let hue = (Math.atan2(bb, a) * 180) / Math.PI;
    if (hue < 0) hue += 360;
    let best = null;
    let bestDistance = Infinity;
    for (const [name, paletteHue] of Object.entries(hues)) {
      const raw = Math.abs(hue - paletteHue);
      const distance = Math.min(raw, 360 - raw);
      if (distance < bestDistance) { bestDistance = distance; best = name; }
    }
    return bestDistance <= 30 ? best : null;
  };
})();

const proposeLabel = labelForRgba ?? fallbackLabelForRgba;

/** Every .glif under a path, following .ufo directories. */
function glifsUnder(path) {
  const found = [];
  const walk = (dir) => {
    for (const entry of readdirSync(dir)) {
      const full = join(dir, entry);
      if (statSync(full).isDirectory()) walk(full);
      else if (entry.endsWith(".glif")) found.push(full);
    }
  };
  walk(path);
  return found;
}

const MARK_RE = /<key>\s*public\.markColor\s*<\/key>\s*<string>\s*([0-9.,\s]+)\s*<\/string>/;
const LABEL_RE = new RegExp(`<key>\\s*${MARK_LABEL_KEY.replace(/\./g, "\\.")}\\s*</key>\\s*<string>\\s*([^<]*)</string>`);

const files = roots.flatMap(glifsUnder);
const byColor = new Map();
for (const file of files) {
  const xml = readFileSync(file, "utf8");
  const rgba = MARK_RE.exec(xml)?.[1]?.replace(/\s+/g, "");
  if (!rgba) continue;
  const entry = byColor.get(rgba) ?? { files: [], labelled: 0 };
  entry.files.push(file);
  if (LABEL_RE.test(xml)) entry.labelled += 1;
  byColor.set(rgba, entry);
}

if (byColor.size === 0) {
  console.log(`no marked glyphs under ${roots.join(", ")}`);
  process.exit(0);
}

console.log(`${files.length} glyphs, ${[...byColor.values()].reduce((n, e) => n + e.files.length, 0)} marked\n`);
const plan = [];
for (const [rgba, entry] of [...byColor].sort((a, b) => b[1].files.length - a[1].files.length)) {
  const label = overrides.get(rgba) ?? proposeLabel(rgba);
  const source = overrides.has(rgba) ? "given" : label ? "snapped" : "no match";
  plan.push({ rgba, label, entry });
  console.log(
    `  ${rgba.padEnd(22)} ${String(entry.files.length).padStart(4)} glyphs  →  ` +
      `${(label ?? "?").padEnd(8)} (${source})` +
      (entry.labelled ? `  [${entry.labelled} already labelled]` : ""),
  );
}

const collisions = new Map();
for (const { rgba, label } of plan) {
  if (!label) continue;
  collisions.set(label, [...(collisions.get(label) ?? []), rgba]);
}
let blocked = false;
for (const [label, colors] of collisions) {
  if (colors.length > 1) {
    blocked = true;
    console.log(
      `\ncollision: ${colors.join("  and  ")} would both become "${label}".` +
        `\n  If those mean different things, pass --map ${colors[1]}=<other label>.`,
    );
  }
}
const unmatched = plan.filter((p) => !p.label);
for (const { rgba, entry } of unmatched) {
  blocked = true;
  console.log(`\nno palette hue near ${rgba} (${entry.files.length} glyphs) — pass --map ${rgba}=<label>.`);
}

if (!write) {
  console.log(`\nDry run. Re-run with --write to stamp ${MARK_LABEL_KEY} into these files.`);
  process.exit(0);
}
if (blocked) {
  console.log("\nNothing written: resolve the above with --map first.");
  process.exit(1);
}

let written = 0;
for (const { label, entry } of plan) {
  for (const file of entry.files) {
    const xml = readFileSync(file, "utf8");
    if (LABEL_RE.test(xml)) {
      const next = xml.replace(LABEL_RE, `<key>${MARK_LABEL_KEY}</key>\n\t\t\t<string>${label}</string>`);
      if (next !== xml) { writeFileSync(file, next); written += 1; }
      continue;
    }
    // Straight after the colour it explains, at the same indent.
    const next = xml.replace(MARK_RE, (match) => {
      const indent = /\n(\s*)<key>/.exec(`\n${match}`)?.[1] ?? "\t\t\t";
      return `${match}\n${indent}<key>${MARK_LABEL_KEY}</key>\n${indent}<string>${label}</string>`;
    });
    if (next !== xml) { writeFileSync(file, next); written += 1; }
  }
}
console.log(`\nStamped ${MARK_LABEL_KEY} into ${written} glyphs.`);
