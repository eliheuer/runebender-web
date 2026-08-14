export type KerningSide = "left" | "right";

export type GlyphKerningGroups = {
  left?: string;
  right?: string;
};

export const KERNING_GROUP_PREFIX = {
  // UFO names groups by their position in a pair, not by the edge of
  // the glyph they describe. The first glyph contributes its right
  // edge (kern1); the second contributes its left edge (kern2).
  left: "public.kern2.",
  right: "public.kern1.",
} as const;

export function defaultKerningEntryKeys(
  leftGlyph: string,
  rightGlyph: string,
  leftGlyphGroups?: GlyphKerningGroups,
  rightGlyphGroups?: GlyphKerningGroups,
): [string, string] {
  return [
    leftGlyphGroups?.right || leftGlyph,
    rightGlyphGroups?.left || rightGlyph,
  ];
}

export function serializeKerningPlist(
  kerning: Map<string, Map<string, number>>,
): string {
  const lines = [
    '<?xml version="1.0" encoding="UTF-8"?>',
    '<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">',
    '<plist version="1.0">',
    '<dict>',
  ];

  for (const first of Array.from(kerning.keys()).sort()) {
    const pairs = kerning.get(first);
    if (!pairs || pairs.size === 0) continue;
    lines.push(`  <key>${escapeXml(first)}</key>`);
    lines.push("  <dict>");
    for (const second of Array.from(pairs.keys()).sort()) {
      const value = pairs.get(second);
      if (value === undefined) continue;
      lines.push(`    <key>${escapeXml(second)}</key>`);
      lines.push(`    <real>${formatPlistNumber(value)}</real>`);
    }
    lines.push("  </dict>");
  }

  lines.push("</dict>", "</plist>", "");
  return lines.join("\n");
}

function escapeXml(value: string): string {
  return value
    .replace(/&/g, "&amp;")
    .replace(/</g, "&lt;")
    .replace(/>/g, "&gt;");
}

function formatPlistNumber(value: number): string {
  return Number.isInteger(value) ? String(value) : String(Number(value.toFixed(6)));
}
