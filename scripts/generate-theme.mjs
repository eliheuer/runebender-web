#!/usr/bin/env node
// themes/runebender.theme.json → src/themeTokens.generated.ts
//
// The authoring file holds OKLCH; the app wants hex. Everything the
// editor paints with comes out of here: the CSS custom properties on the
// host element, the colours handed to the wasm renderer, and the mark
// colours written into UFO lib keys.
//
// Run with: pnpm run theme

import { readFileSync, writeFileSync } from "node:fs";
import { dirname, resolve } from "node:path";
import { fileURLToPath } from "node:url";
import { oklchToHex, oklchToUfoRgba } from "./oklch.mjs";

const root = resolve(dirname(fileURLToPath(import.meta.url)), "..");
const source = resolve(root, "themes/runebender.theme.json");
const target = resolve(root, "src/themeTokens.generated.ts");

const theme = JSON.parse(readFileSync(source, "utf8"));

/** Every hue at every step, plus the neutral ramp. */
function buildPalette() {
  const palette = {};
  for (const [hueName, hue] of Object.entries(theme.hues)) {
    const lift = theme.lift[hueName] ?? 0;
    for (const [stepName, step] of Object.entries(theme.steps)) {
      palette[`${hueName}.${stepName}`] = oklchToHex(
        step.lightness + lift,
        step.chroma,
        hue,
      );
    }
  }
  for (const step of theme.neutral.steps) {
    palette[`neutral.${step}`] = oklchToHex(
      step / 100,
      theme.neutral.chroma,
      theme.neutral.hue,
    );
  }
  return palette;
}

const palette = buildPalette();

function colorFor(reference) {
  const hex = palette[reference];
  if (!hex) throw new Error(`theme references unknown colour "${reference}"`);
  return hex;
}

/** kebab-case CSS custom property name for a token. */
function cssName(group, key) {
  const kebab = key.replace(/[A-Z]/g, (c) => `-${c.toLowerCase()}`);
  return group ? `--rb-${group}-${kebab}` : `--rb-${kebab}`;
}

/**
 * One theme as CSS custom properties. Names match what the components
 * already ask for, so a theme switch is a change of values, not markup.
 */
function tokensFor(definition) {
  const surfaces = definition.surfaces;
  const text = definition.text;
  const roles = definition.roles;
  const t = {};

  t["--rb-app-background"] = colorFor(surfaces.app);
  t["--rb-panel-background"] = colorFor(surfaces.panel);
  t["--rb-grid-cell-background"] = colorFor(surfaces.panel);
  t["--rb-grid-cell-hover-background"] = colorFor(surfaces.buttonHover);
  t["--rb-button-background"] = colorFor(surfaces.button);
  t["--rb-button-hover"] = colorFor(surfaces.buttonHover);
  t["--rb-control-background"] = colorFor(surfaces.control);
  t["--rb-field-background"] = colorFor(surfaces.field);
  t["--rb-panel-outline"] = colorFor(surfaces.outline);
  t["--rb-panel-divider"] = colorFor(surfaces.divider);
  // Two aliases the panels grew on their own. Kept so their styles do
  // not have to change, defined here so they stop falling back to hexes.
  t["--rb-panel-bg"] = colorFor(surfaces.panel);
  t["--rb-panel-border"] = colorFor(surfaces.outline);

  t["--rb-primary-text"] = colorFor(text.primary);
  t["--rb-secondary-text"] = colorFor(text.secondary);
  t["--rb-muted-text"] = colorFor(text.muted);
  t["--rb-subdued-text"] = colorFor(text.subdued);
  t["--rb-overlay-text"] = colorFor(text.overlay);
  t["--rb-glyph-preview"] = colorFor(text.glyph);

  t["--rb-accent"] = colorFor(roles.accent);
  t["--rb-warning"] = colorFor(roles.warning);
  t["--rb-danger"] = colorFor(roles.danger);
  t["--rb-danger-text"] = colorFor(roles.danger);
  t["--rb-grid-selected"] = colorFor(roles.accent);
  t["--rb-mark-selected-ring"] = colorFor(roles.accent);
  t["--rb-background-image-selection"] = colorFor(roles.accent);

  // Canvas bridge: Vue resolves these and hands the values to the wasm
  // renderer, so the WebGPU scene and the chrome stay one theme.
  t["--rb-canvas-background"] = colorFor(surfaces.canvas);
  t["--rb-canvas-path-stroke"] = colorFor(roles.pathStroke);
  t["--rb-canvas-selection"] = colorFor(roles.selection);
  t["--rb-canvas-component"] = colorFor(roles.component);
  t["--rb-canvas-component-selected"] = colorFor(roles.componentSelected);
  t["--rb-canvas-point-smooth-inner"] = colorFor(roles.pointInner);
  t["--rb-canvas-point-smooth-outer"] = colorFor(roles.pointSmooth);
  t["--rb-canvas-point-corner-inner"] = colorFor(roles.pointInner);
  t["--rb-canvas-point-corner-outer"] = colorFor(roles.pointCorner);
  t["--rb-canvas-point-offcurve-inner"] = colorFor(roles.pointInner);
  t["--rb-canvas-point-offcurve-outer"] = colorFor(roles.pointOffcurve);
  t["--rb-canvas-point-hyper-inner"] = colorFor(roles.pointInner);
  t["--rb-canvas-point-hyper-outer"] = colorFor(roles.pointHyper);
  t["--rb-canvas-point-selected"] = colorFor(roles.pointSelected);
  t["--rb-canvas-start-node"] = colorFor(roles.startNode);
  t["--rb-canvas-text-cursor"] = colorFor(roles.textCursor);
  t["--rb-canvas-kern-active"] = colorFor(roles.kernActive);
  t["--rb-canvas-kern-previous"] = colorFor(roles.kernPrevious);
  t["--rb-canvas-text-preview-fill"] = colorFor(roles.previewFill);
  t["--rb-canvas-background-layer"] = colorFor(roles.background);
  t["--rb-canvas-reference-glyph"] = colorFor(roles.reference);

  for (const [name, hex] of Object.entries(palette)) {
    const [hue, step] = name.split(".");
    t[cssName("color", `${hue}-${step}`)] = hex;
  }
  return t;
}

const themes = Object.fromEntries(
  Object.entries(theme.themes).map(([id, definition]) => [
    id,
    { id, name: definition.name, tokens: tokensFor(definition) },
  ]),
);

const markColors = theme.markColors.map(({ name, step }) => {
  const [hue, stepName] = step.split(".");
  const lift = theme.lift[hue] ?? 0;
  const { lightness, chroma } = theme.steps[stepName];
  return {
    name,
    color: colorFor(step),
    ufoRgba: oklchToUfoRgba(lightness + lift, chroma, theme.hues[hue]),
  };
});

const banner = `// GENERATED by scripts/generate-theme.mjs — do not edit.
// Colour is authored in OKLCH in themes/runebender.theme.json; run
// \`pnpm run theme\` after changing it.
`;

const body = `${banner}
/** Every colour in the system, as \`hue.step\` (e.g. "green.base"). */
export const PALETTE = ${JSON.stringify(palette, null, 2)} as const;

export type ThemeId = ${Object.keys(themes)
  .map((id) => JSON.stringify(id))
  .join(" | ")};

export type Theme = {
  id: ThemeId;
  name: string;
  /** CSS custom properties to set on the editor's host element. */
  tokens: Record<string, string>;
};

export const THEMES: Record<ThemeId, Theme> = ${JSON.stringify(themes, null, 2)};

export const THEME_IDS = Object.keys(THEMES) as ThemeId[];

export const DEFAULT_THEME_ID: ThemeId = ${JSON.stringify(Object.keys(themes)[0])};

/** Glyph mark colours, in palette order. \`ufoRgba\` is what goes in the UFO. */
export const THEME_MARK_COLORS = ${JSON.stringify(markColors, null, 2)} as const;
`;

writeFileSync(target, body);
console.log(`theme: ${Object.keys(themes).length} themes, ${Object.keys(palette).length} colours → ${target.replace(`${root}/`, "")}`);
for (const [name, hex] of Object.entries(palette)) {
  if (!name.startsWith("neutral")) console.log(`  ${name.padEnd(16)} ${hex}`);
}
