// OKLCH → sRGB, with gamut mapping.
//
// Colours are authored in OKLCH because it is perceptually even: two
// hues at the same lightness and chroma look equally bright and equally
// colourful, which is what makes a set of labels read as one family.
// sRGB hex is only the output format.
//
// Matrices from Björn Ottosson's Oklab reference.

/** OKLCH (L 0..1, C 0..~0.4, H degrees) → linear sRGB, unclamped. */
export function oklchToLinearSrgb(L, C, H) {
  const hRad = (H * Math.PI) / 180;
  const a = C * Math.cos(hRad);
  const b = C * Math.sin(hRad);

  const l_ = L + 0.3963377774 * a + 0.2158037573 * b;
  const m_ = L - 0.1055613458 * a - 0.0638541728 * b;
  const s_ = L - 0.0894841775 * a - 1.291485548 * b;

  const l = l_ * l_ * l_;
  const m = m_ * m_ * m_;
  const s = s_ * s_ * s_;

  return [
    +4.0767416621 * l - 3.3077115913 * m + 0.2309699292 * s,
    -1.2684380046 * l + 2.6097574011 * m - 0.3413193965 * s,
    -0.0041960863 * l - 0.7034186147 * m + 1.707614701 * s,
  ];
}

function linearToSrgb(value) {
  return value <= 0.0031308
    ? 12.92 * value
    : 1.055 * Math.pow(value, 1 / 2.4) - 0.055;
}

function inGamut([r, g, b]) {
  const epsilon = 1e-6;
  return [r, g, b].every((v) => v >= -epsilon && v <= 1 + epsilon);
}

/**
 * OKLCH → sRGB channels in 0..255.
 *
 * A colour outside sRGB keeps its lightness and hue and loses chroma
 * until it fits — clipping the channels instead would shift the hue,
 * which is exactly what a perceptual space is meant to avoid.
 */
export function oklchToRgb(L, C, H) {
  let low = 0;
  let high = C;
  let chroma = C;
  if (!inGamut(oklchToLinearSrgb(L, C, H))) {
    for (let i = 0; i < 24; i += 1) {
      chroma = (low + high) / 2;
      if (inGamut(oklchToLinearSrgb(L, chroma, H))) {
        low = chroma;
      } else {
        high = chroma;
      }
    }
    chroma = low;
  }
  return oklchToLinearSrgb(L, chroma, H)
    .map(linearToSrgb)
    .map((v) => Math.round(Math.min(1, Math.max(0, v)) * 255));
}

export function oklchToHex(L, C, H) {
  return `#${oklchToRgb(L, C, H)
    .map((v) => v.toString(16).padStart(2, "0"))
    .join("")}`;
}

/** Hex with an alpha byte appended, e.g. #18b86f80. */
export function oklchToHexAlpha(L, C, H, alpha) {
  const a = Math.round(Math.min(1, Math.max(0, alpha)) * 255);
  return `${oklchToHex(L, C, H)}${a.toString(16).padStart(2, "0")}`;
}

/** UFO mark colours are "r,g,b,a" with each channel 0..1. */
export function oklchToUfoRgba(L, C, H, alpha = 1) {
  const channels = oklchToRgb(L, C, H).map((v) => Number((v / 255).toFixed(2)));
  return `${channels.join(",")},${alpha}`;
}
