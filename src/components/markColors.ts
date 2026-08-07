// Mark colours: what a glyph is tagged with, and how that is stored.
//
// UFO's `public.markColor` is a colour, not a label — "r,g,b,a" and
// nothing else. So a palette change orphans every file written before
// it: the glyph still says 0.09,0.72,0.44 long after green stopped
// meaning that. Worse for anything reasoning in names, which is what a
// designer and a training pipeline both do.
//
// So we write the name alongside the colour, under
// `com.runebender.markLabel` (MARK_LABEL_KEY in wasm_api.rs), and read
// the name first. The colour is what Glyphs, RoboFont and fontmake
// need; the name is what the mark means.
//
// Two rules keep files stable:
//
//   - the colour written for a label is fixed, not the active theme's.
//     Otherwise switching to Light and saving would rewrite the mark of
//     every glyph in the font.
//   - a file with no label is read by snapping its colour to the
//     nearest hue, for display only. That path never writes.

import { PALETTE, THEME_MARK_COLORS } from "../themeTokens.generated";

export type MarkColor = {
  /** UFO `public.markColor` value: "r,g,b,a" with 0–1 floats. */
  rgba: string;
  /** The label stored beside it, and the palette hue it names. */
  name: string;
};

export const MARK_COLORS: MarkColor[] = THEME_MARK_COLORS.map((color) => ({
  rgba: color.ufoRgba,
  name: color.name,
}));

const MARK_BY_NAME = new Map(MARK_COLORS.map((mark) => [mark.name, mark]));

/** The UFO colour to write for a label. Fixed, so files do not churn. */
export function rgbaForLabel(label: string): string {
  return MARK_BY_NAME.get(label)?.rgba ?? "";
}

/**
 * How a label is drawn: the theme's colour for that hue. A CSS variable
 * rather than a value, so switching themes restyles every mark in the
 * font without touching a single glyph.
 */
export function cssForLabel(label: string): string {
  return MARK_BY_NAME.has(label) ? `var(--rb-mark-${label})` : "";
}

/// Convert "r,g,b,a" (0–1 floats) to a CSS rgba(...) string.
export function rgbaToCss(s: string, alphaOverride?: number): string {
  const [r, g, b, a] = s.split(",").map(Number);
  if ([r, g, b, a].some((n) => !Number.isFinite(n))) return "transparent";
  const aa = alphaOverride ?? a;
  return `rgba(${Math.round(r * 255)}, ${Math.round(g * 255)}, ${Math.round(b * 255)}, ${aa})`;
}

function srgbToLinear(value: number): number {
  return value <= 0.04045 ? value / 12.92 : ((value + 0.055) / 1.055) ** 2.4;
}

/** Hue angle in OKLCH, or null for something too grey to have one. */
function hueOf(r255: number, g255: number, b255: number): number | null {
  const r = srgbToLinear(r255 / 255);
  const g = srgbToLinear(g255 / 255);
  const b = srgbToLinear(b255 / 255);
  const l = Math.cbrt(0.4122214708 * r + 0.5363325363 * g + 0.0514459929 * b);
  const m = Math.cbrt(0.2119034982 * r + 0.6806995451 * g + 0.1073969566 * b);
  const s = Math.cbrt(0.0883024619 * r + 0.2817188376 * g + 0.6299787005 * b);
  const a = 1.9779984951 * l - 2.428592205 * m + 0.4505937099 * s;
  const bb = 0.0259040371 * l + 0.7827717662 * m - 0.808675766 * s;
  if (Math.hypot(a, bb) < 0.03) return null;
  const hue = (Math.atan2(bb, a) * 180) / Math.PI;
  return hue < 0 ? hue + 360 : hue;
}

const PALETTE_HUES: Array<[string, number]> = MARK_COLORS.map((mark) => {
  const hex = PALETTE[`${mark.name}.base` as keyof typeof PALETTE] as string;
  const channels = [1, 3, 5].map((i) => parseInt(hex.slice(i, i + 2), 16));
  return [mark.name, hueOf(channels[0], channels[1], channels[2]) ?? 0];
});

/**
 * The label an unlabelled colour is nearest to, by hue alone — a mark
 * from an older palette is the same idea in a different shade, and hue
 * is what survives that. Display only: guessing is never written back.
 *
 * Returns null for a colour with no hue to speak of (a grey mark), and
 * for anything the palette has no hue near.
 */
export function labelForRgba(rgba: string): string | null {
  const [r, g, b] = rgba.split(",").map(Number);
  if ([r, g, b].some((n) => !Number.isFinite(n))) return null;
  const hue = hueOf(r * 255, g * 255, b * 255);
  if (hue === null) return null;
  let best: string | null = null;
  let bestDistance = Infinity;
  for (const [name, paletteHue] of PALETTE_HUES) {
    const raw = Math.abs(hue - paletteHue);
    const distance = Math.min(raw, 360 - raw);
    if (distance < bestDistance) {
      bestDistance = distance;
      best = name;
    }
  }
  // Two neighbouring hues in the palette are about 40° apart, so
  // anything further than half that from every one of them is not a
  // near miss — it is a colour we do not have a name for.
  return bestDistance <= 30 ? best : null;
}

export { THEME_MARK_COLORS };
