// File → New Font. Builds a blank UFO (or a two-master designspace) in
// memory, set up the way Google Fonts expects, and hands it to the same
// loader that reads a font off disk.
//
// Everything is a synthetic `File` carrying a `webkitRelativePath`, so
// the app cannot tell a new project from an opened folder. Nothing is
// written anywhere until the user picks Save As.

import { GF_LATIN_CORE, type TemplateGlyph } from "./newFontTemplate.generated";

/** Vertical metrics, in font units. */
const UPM = 1000;
const ASCENDER = 800;
const DESCENDER = -200;
const CAP_HEIGHT = 700;
const X_HEIGHT = 500;
// GF wants the win metrics to clear the ink; typo + hhea agree, and
// useTypoMetrics (OS/2 fsSelection bit 7) makes typo the ones that count.
const WIN_ASCENT = 1000;
const WIN_DESCENT = 300;

/** Placeholder advance widths — a starting point, not a design. */
const DEFAULT_WIDTH = 600;
const SPACE_WIDTH = 260;

export type NewProjectKind = "ufo" | "designspace";

export type NewProjectMaster = {
  styleName: string;
  /** OS/2 usWeightClass, and the master's position on the wght axis. */
  weight: number;
};

/** The masters a new designspace starts with. */
const DESIGNSPACE_MASTERS: NewProjectMaster[] = [
  { styleName: "Regular", weight: 400 },
  { styleName: "Bold", weight: 700 },
];

export type NewProject = {
  files: File[];
  /** Path of the .designspace within the project, if there is one. */
  designspacePath: string | null;
  /** What the top bar should show. */
  label: string;
};

function fileAt(path: string, text: string): File {
  const file = new File([text], path.slice(path.lastIndexOf("/") + 1), {
    type: "text/plain",
  });
  // The loader keys everything off webkitRelativePath, which is read-only
  // on a constructed File — define it so a synthetic project walks the
  // same path as a picked folder.
  Object.defineProperty(file, "webkitRelativePath", {
    value: path,
    enumerable: true,
  });
  return file;
}

/** UFO folder name for a style, e.g. "Sample Regular" → "Sample-Regular.ufo". */
function ufoDirName(familyName: string, styleName: string): string {
  const family = familyName.replace(/\s+/g, "");
  const style = styleName.replace(/\s+/g, "");
  return `${family}-${style}.ufo`;
}

function plist(body: string): string {
  return `<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
${body}
</plist>
`;
}

function escapeXml(value: string): string {
  return value
    .replace(/&/g, "&amp;")
    .replace(/</g, "&lt;")
    .replace(/>/g, "&gt;")
    .replace(/"/g, "&quot;");
}

function metainfoPlist(): string {
  return plist(`<dict>
  <key>creator</key>
  <string>org.runebender</string>
  <key>formatVersion</key>
  <integer>3</integer>
</dict>`);
}

function layercontentsPlist(): string {
  return plist(`<array>
  <array>
    <string>public.default</string>
    <string>glyphs</string>
  </array>
</array>`);
}

function fontinfoPlist(
  familyName: string,
  styleName: string,
  weight: number,
): string {
  const year = new Date().getFullYear();
  return plist(`<dict>
  <key>familyName</key>
  <string>${escapeXml(familyName)}</string>
  <key>styleName</key>
  <string>${escapeXml(styleName)}</string>
  <key>styleMapFamilyName</key>
  <string>${escapeXml(familyName)}</string>
  <key>styleMapStyleName</key>
  <string>${weight >= 700 ? "bold" : "regular"}</string>
  <key>versionMajor</key>
  <integer>1</integer>
  <key>versionMinor</key>
  <integer>0</integer>
  <key>copyright</key>
  <string>Copyright ${year} The ${escapeXml(familyName)} Project Authors</string>
  <key>openTypeNameLicense</key>
  <string>This Font Software is licensed under the SIL Open Font License, Version 1.1.</string>
  <key>openTypeNameLicenseURL</key>
  <string>https://openfontlicense.org</string>
  <key>unitsPerEm</key>
  <integer>${UPM}</integer>
  <key>ascender</key>
  <integer>${ASCENDER}</integer>
  <key>descender</key>
  <integer>${DESCENDER}</integer>
  <key>capHeight</key>
  <integer>${CAP_HEIGHT}</integer>
  <key>xHeight</key>
  <integer>${X_HEIGHT}</integer>
  <key>italicAngle</key>
  <integer>0</integer>
  <key>openTypeOS2WeightClass</key>
  <integer>${weight}</integer>
  <key>openTypeOS2TypoAscender</key>
  <integer>${ASCENDER}</integer>
  <key>openTypeOS2TypoDescender</key>
  <integer>${DESCENDER}</integer>
  <key>openTypeOS2TypoLineGap</key>
  <integer>0</integer>
  <key>openTypeHheaAscender</key>
  <integer>${ASCENDER}</integer>
  <key>openTypeHheaDescender</key>
  <integer>${DESCENDER}</integer>
  <key>openTypeHheaLineGap</key>
  <integer>0</integer>
  <key>openTypeOS2WinAscent</key>
  <integer>${WIN_ASCENT}</integer>
  <key>openTypeOS2WinDescent</key>
  <integer>${WIN_DESCENT}</integer>
  <key>openTypeOS2Selection</key>
  <array>
    <integer>7</integer>
  </array>
  <key>postscriptUnderlinePosition</key>
  <integer>-75</integer>
  <key>postscriptUnderlineThickness</key>
  <integer>50</integer>
</dict>`);
}

function libPlist(glyphs: TemplateGlyph[]): string {
  const order = glyphs
    .map((g) => `    <string>${escapeXml(g.name)}</string>`)
    .join("\n");
  return plist(`<dict>
  <key>public.glyphOrder</key>
  <array>
${order}
  </array>
</dict>`);
}

/**
 * UFO glyph filename for a glyph name, following the format's rule that
 * an uppercase letter is suffixed with an underscore so the name survives
 * case-insensitive filesystems (A → A_.glif).
 */
export function glifFileName(name: string): string {
  let out = "";
  for (const ch of name) {
    if (ch >= "A" && ch <= "Z") out += `${ch}_`;
    else if (ch === ".") out += "_";
    else out += ch;
  }
  return `${out}.glif`;
}

function advanceWidthFor(name: string): number {
  if (name === "space" || name === "nbspace" || name === "uni00A0") {
    return SPACE_WIDTH;
  }
  // Combining marks carry no advance of their own.
  if (name.endsWith("comb")) return 0;
  return DEFAULT_WIDTH;
}

function glif(glyph: TemplateGlyph): string {
  const unicode = glyph.unicode
    ? `\n  <unicode hex="${glyph.unicode}"/>`
    : "";
  return `<?xml version="1.0" encoding="UTF-8"?>
<glyph name="${escapeXml(glyph.name)}" format="2">
  <advance width="${advanceWidthFor(glyph.name)}"/>${unicode}
  <outline>
  </outline>
</glyph>
`;
}

function contentsPlist(glyphs: TemplateGlyph[]): string {
  const entries = glyphs
    .map(
      (g) =>
        `  <key>${escapeXml(g.name)}</key>\n  <string>${escapeXml(glifFileName(g.name))}</string>`,
    )
    .join("\n");
  return plist(`<dict>
${entries}
</dict>`);
}

/** Every file of one blank UFO, rooted at `dir`. */
function ufoFiles(
  dir: string,
  familyName: string,
  styleName: string,
  weight: number,
  glyphs: TemplateGlyph[],
): File[] {
  const files = [
    fileAt(`${dir}/metainfo.plist`, metainfoPlist()),
    fileAt(`${dir}/fontinfo.plist`, fontinfoPlist(familyName, styleName, weight)),
    fileAt(`${dir}/lib.plist`, libPlist(glyphs)),
    fileAt(`${dir}/layercontents.plist`, layercontentsPlist()),
    fileAt(`${dir}/glyphs/contents.plist`, contentsPlist(glyphs)),
  ];
  for (const glyph of glyphs) {
    files.push(fileAt(`${dir}/glyphs/${glifFileName(glyph.name)}`, glif(glyph)));
  }
  return files;
}

function designspaceXml(
  familyName: string,
  masters: NewProjectMaster[],
): string {
  const [light, heavy] = [masters[0], masters[masters.length - 1]];
  const sources = masters
    .map(
      (m) => `    <source filename="${escapeXml(ufoDirName(familyName, m.styleName))}" name="${escapeXml(`${familyName} ${m.styleName}`)}" familyname="${escapeXml(familyName)}" stylename="${escapeXml(m.styleName)}">
      <location>
        <dimension name="Weight" xvalue="${m.weight}"/>
      </location>
    </source>`,
    )
    .join("\n");
  const instances = masters
    .map(
      (m) => `    <instance name="${escapeXml(`${familyName} ${m.styleName}`)}" familyname="${escapeXml(familyName)}" stylename="${escapeXml(m.styleName)}">
      <location>
        <dimension name="Weight" xvalue="${m.weight}"/>
      </location>
    </instance>`,
    )
    .join("\n");

  return `<?xml version="1.0" encoding="UTF-8"?>
<designspace format="4.1">
  <axes>
    <axis tag="wght" name="Weight" minimum="${light.weight}" maximum="${heavy.weight}" default="${light.weight}"/>
  </axes>
  <sources>
${sources}
  </sources>
  <instances>
${instances}
  </instances>
</designspace>
`;
}

/**
 * A new blank project: one UFO, or a designspace with a Regular and a
 * Bold master on a wght axis. Glyph set is GF Latin Core — the minimum
 * for onboarding into Google Fonts — with correct names, codepoints and
 * glyph order, and no outlines.
 */
export function buildNewProject(
  kind: NewProjectKind,
  familyName: string,
): NewProject {
  const family = familyName.trim() || "New Font";
  const glyphs = GF_LATIN_CORE;
  const root = family.replace(/\s+/g, "");

  if (kind === "ufo") {
    const dir = `${root}/${ufoDirName(family, "Regular")}`;
    return {
      files: ufoFiles(dir, family, "Regular", 400, glyphs),
      designspacePath: null,
      label: ufoDirName(family, "Regular"),
    };
  }

  const files: File[] = [];
  for (const master of DESIGNSPACE_MASTERS) {
    files.push(
      ...ufoFiles(
        `${root}/${ufoDirName(family, master.styleName)}`,
        family,
        master.styleName,
        master.weight,
        glyphs,
      ),
    );
  }
  const dsPath = `${root}/${root}.designspace`;
  files.push(fileAt(dsPath, designspaceXml(family, DESIGNSPACE_MASTERS)));
  return { files, designspacePath: dsPath, label: `${root}.designspace` };
}
