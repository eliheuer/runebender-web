import type { Category } from "./components/CategorySidebar.vue";
import { GF_GLYPHSETS } from "./gfSidebarData.generated";

export type GlyphSidebarFilter =
  | { kind: "all" }
  | { kind: "category"; category: Category; subcategory?: string }
  | { kind: "languageGroup"; id: string }
  | { kind: "language"; id: string }
  | { kind: "gfGlyphset"; id: string }
  | { kind: "builtin"; id: string };

export type SidebarSearchMode = "all" | "name" | "unicode";
export type GlyphSortMode = "name" | "unicode";

export type SidebarCategorySubfilter = {
  id: string;
  label: string;
};

export type SidebarCategoryGroup = {
  category: Category;
  icon: string;
  subfilters?: SidebarCategorySubfilter[];
};

export type SidebarLanguageGroup = {
  id: string;
  label: string;
  icon: string;
  filters: SidebarCharacterFilter[];
};

export type SidebarGlyphTarget = {
  name: string;
  unicode: number;
  label?: string;
};

export type SidebarCharacterFilter = {
  id: string;
  label: string;
  source: "google-fonts-lang" | "google-fonts-glyphsets" | "google-fonts-subset" | "unicode-range";
  glyphNames?: readonly string[];
  targets?: readonly SidebarGlyphTarget[];
  chars?: string;
  ranges?: Array<[number, number]>;
  expectedCount?: number;
};

export type SidebarBuiltinFilter = {
  id: string;
  label: string;
  source: "runebender" | "google-fonts-glyphsets";
  glyphNames?: readonly string[];
  chars?: string;
  ranges?: Array<[number, number]>;
  expectedCount?: number;
};

export const CATEGORY_GROUPS: SidebarCategoryGroup[] = [
  {
    category: "Letter",
    icon: "A",
    subfilters: [
      { id: "uppercase", label: "Uppercase" },
      { id: "lowercase", label: "Lowercase" },
      { id: "ligature", label: "Ligature" },
    ],
  },
  {
    category: "Number",
    icon: "9",
    subfilters: [
      { id: "decimal", label: "Decimal" },
      { id: "fraction", label: "Fraction" },
      { id: "superior-inferior", label: "Superior/Inferior" },
    ],
  },
  {
    category: "Separator",
    icon: "··",
    subfilters: [
      { id: "space", label: "Space" },
      { id: "line", label: "Line" },
    ],
  },
  {
    category: "Punctuation",
    icon: "?",
    subfilters: [
      { id: "quote", label: "Quote" },
      { id: "dash", label: "Dash" },
      { id: "paren", label: "Parenthesis" },
    ],
  },
  {
    category: "Symbol",
    icon: "¶",
    subfilters: [
      { id: "currency", label: "Currency" },
      { id: "math", label: "Math" },
      { id: "arrow", label: "Arrow" },
    ],
  },
  {
    category: "Mark",
    icon: "◌",
    subfilters: [
      { id: "nonspacing", label: "Nonspacing" },
      { id: "spacing", label: "Spacing" },
    ],
  },
  { category: "Other", icon: "•" },
];

const SCRIPT_META: Record<string, { id: string; label: string; icon: string }> = {
  Arabic: { id: "Arab", label: "Arabic", icon: "ض" },
  Cyrillic: { id: "Cyrl", label: "Cyrillic", icon: "Я" },
  Greek: { id: "Grek", label: "Greek", icon: "Ω" },
  Hebrew: { id: "Hebr", label: "Hebrew", icon: "א" },
  Latin: { id: "Latn", label: "Latin", icon: "G" },
  Phonetics: { id: "Phon", label: "Phonetics", icon: "ə" },
  TransLatin: { id: "Tran", label: "TransLatin", icon: "Ǧ" },
};

const GF_HEBREW_SUBSET_RANGES: Array<[number, number]> = [
  [0x0000, 0x0000],
  [0x000d, 0x000d],
  [0x0020, 0x0020],
  [0x002d, 0x002d],
  [0x00a0, 0x00a0],
  [0x0307, 0x0308],
  [0x0591, 0x05c7],
  [0x05d0, 0x05ea],
  [0x05ef, 0x05f4],
  [0x200b, 0x200f],
  [0x2010, 0x2010],
  [0x20aa, 0x20aa],
  [0x25cc, 0x25cc],
  [0xfb1d, 0xfb36],
  [0xfb38, 0xfb3c],
  [0xfb3e, 0xfb3e],
  [0xfb40, 0xfb41],
  [0xfb43, 0xfb44],
  [0xfb46, 0xfb4f],
];

const GF_HEBREW_PRESENTATION_FORM_RANGES: Array<[number, number]> = [
  [0xfb1d, 0xfb36],
  [0xfb38, 0xfb3c],
  [0xfb3e, 0xfb3e],
  [0xfb40, 0xfb41],
  [0xfb43, 0xfb44],
  [0xfb46, 0xfb4f],
];

const HEBREW_LETTER_NAMES = [
  [0x05d0, "alef-hb"],
  [0x05d1, "bet-hb"],
  [0x05d2, "gimel-hb"],
  [0x05d3, "dalet-hb"],
  [0x05d4, "he-hb"],
  [0x05d5, "vav-hb"],
  [0x05d6, "zayin-hb"],
  [0x05d7, "het-hb"],
  [0x05d8, "tet-hb"],
  [0x05d9, "yod-hb"],
  [0x05da, "finalkaf-hb"],
  [0x05db, "kaf-hb"],
  [0x05dc, "lamed-hb"],
  [0x05dd, "finalmem-hb"],
  [0x05de, "mem-hb"],
  [0x05df, "finalnun-hb"],
  [0x05e0, "nun-hb"],
  [0x05e1, "samekh-hb"],
  [0x05e2, "ayin-hb"],
  [0x05e3, "finalpe-hb"],
  [0x05e4, "pe-hb"],
  [0x05e5, "finaltsadi-hb"],
  [0x05e6, "tsadi-hb"],
  [0x05e7, "qof-hb"],
  [0x05e8, "resh-hb"],
  [0x05e9, "shin-hb"],
  [0x05ea, "tav-hb"],
  [0x05ef, "yodtriangle-hb"],
  [0x05f0, "vavvav-hb"],
  [0x05f1, "vavyod-hb"],
  [0x05f2, "yodyod-hb"],
  [0x05f3, "geresh-hb"],
  [0x05f4, "gershayim-hb"],
] as const;

function uniTarget(codepoint: number, name = `uni${codepoint.toString(16).toUpperCase().padStart(4, "0")}`): SidebarGlyphTarget {
  return { name, unicode: codepoint };
}

function rangeTargets(start: number, end: number): SidebarGlyphTarget[] {
  const targets: SidebarGlyphTarget[] = [];
  for (let codepoint = start; codepoint <= end; codepoint++) {
    targets.push(uniTarget(codepoint));
  }
  return targets;
}

const HEBREW_LETTER_TARGETS: SidebarGlyphTarget[] = HEBREW_LETTER_NAMES.map(
  ([unicode, name]) => ({ unicode, name }),
);

const HEBREW_POINTS_AND_MARKS_TARGETS: SidebarGlyphTarget[] = [
  ...rangeTargets(0x0591, 0x05c7),
];

const HEBREW_PRESENTATION_FORM_TARGETS: SidebarGlyphTarget[] = [
  ...rangeTargets(0xfb1d, 0xfb36),
  ...rangeTargets(0xfb38, 0xfb3c),
  uniTarget(0xfb3e),
  ...rangeTargets(0xfb40, 0xfb41),
  ...rangeTargets(0xfb43, 0xfb44),
  ...rangeTargets(0xfb46, 0xfb4f),
];

const GF_HEBREW_SUBSET_TARGETS: SidebarGlyphTarget[] = [
  uniTarget(0x0020, "space"),
  uniTarget(0x002d, "hyphen"),
  uniTarget(0x00a0, "nbspace"),
  uniTarget(0x0307, "dotaccentcomb"),
  uniTarget(0x0308, "dieresiscomb"),
  ...HEBREW_POINTS_AND_MARKS_TARGETS,
  ...HEBREW_LETTER_TARGETS,
  ...rangeTargets(0x200b, 0x200f),
  uniTarget(0x2010),
  uniTarget(0x20aa),
  uniTarget(0x25cc, "dottedCircle"),
  ...HEBREW_PRESENTATION_FORM_TARGETS,
];

function glyphsetFiltersForScript(script: string): SidebarCharacterFilter[] {
  return GF_GLYPHSETS.filter((glyphset) => glyphset.script === script).map((glyphset) => ({
    id: glyphset.id,
    label: glyphset.label,
    source: "google-fonts-glyphsets",
    glyphNames: glyphset.glyphNames,
    expectedCount: glyphset.expectedCount,
  }));
}

export const SIDEBAR_LANGUAGE_GROUPS: SidebarLanguageGroup[] = [
  {
    ...SCRIPT_META.Arabic,
    filters: glyphsetFiltersForScript("Arabic"),
  },
  {
    id: "Hans",
    label: "Chinese",
    icon: "字",
    filters: [
      { id: "Hans", label: "Chinese Han", source: "unicode-range", ranges: [[0x4e00, 0x9fff]] },
    ],
  },
  {
    id: "Cyrl",
    label: "Cyrillic",
    icon: "Я",
    filters: glyphsetFiltersForScript("Cyrillic"),
  },
  {
    id: "Deva",
    label: "Devanagari",
    icon: "दे",
    filters: [
      { id: "Deva", label: "Devanagari", source: "unicode-range", ranges: [[0x0900, 0x097f]] },
    ],
  },
  {
    id: "Grek",
    label: "Greek",
    icon: "Ω",
    filters: glyphsetFiltersForScript("Greek"),
  },
  {
    ...SCRIPT_META.Hebrew,
    filters: [
      {
        id: "GF_Hebrew_Subset",
        label: "Google Fonts Hebrew",
        source: "google-fonts-subset",
        targets: GF_HEBREW_SUBSET_TARGETS,
        ranges: GF_HEBREW_SUBSET_RANGES,
        expectedCount: GF_HEBREW_SUBSET_TARGETS.length,
      },
      {
        id: "Hebrew_Letters",
        label: "Hebrew letters",
        source: "unicode-range",
        targets: HEBREW_LETTER_TARGETS,
        ranges: [[0x05d0, 0x05ea], [0x05ef, 0x05f4]],
        expectedCount: HEBREW_LETTER_TARGETS.length,
      },
      {
        id: "Hebrew_Points_Marks",
        label: "Hebrew points and marks",
        source: "unicode-range",
        targets: HEBREW_POINTS_AND_MARKS_TARGETS,
        ranges: [[0x0591, 0x05c7]],
        expectedCount: HEBREW_POINTS_AND_MARKS_TARGETS.length,
      },
      {
        id: "Hebrew_Presentation_Forms",
        label: "Hebrew presentation forms",
        source: "unicode-range",
        targets: HEBREW_PRESENTATION_FORM_TARGETS,
        ranges: GF_HEBREW_PRESENTATION_FORM_RANGES,
        expectedCount: HEBREW_PRESENTATION_FORM_TARGETS.length,
      },
    ],
  },
  {
    id: "Jpan",
    label: "Japanese",
    icon: "あ",
    filters: [
      { id: "Jpan", label: "Kana + Han", source: "unicode-range", ranges: [[0x3040, 0x30ff], [0x4e00, 0x9fff]] },
    ],
  },
  {
    id: "Kore",
    label: "Korean",
    icon: "한",
    filters: [
      { id: "Kore", label: "Hangul", source: "unicode-range", ranges: [[0x1100, 0x11ff], [0xac00, 0xd7af]] },
    ],
  },
  {
    id: "Latn",
    label: "Latin",
    icon: "G",
    filters: glyphsetFiltersForScript("Latin"),
  },
  {
    id: "Thai",
    label: "Thai",
    icon: "ก",
    filters: [
      { id: "Thai", label: "Thai", source: "unicode-range", ranges: [[0x0e00, 0x0e7f]] },
    ],
  },
  ...Object.values(SCRIPT_META)
    .filter((meta) => !["Arab", "Cyrl", "Grek", "Hebr", "Latn"].includes(meta.id))
    .map((meta) => ({
      ...meta,
      filters: glyphsetFiltersForScript(meta.label),
    }))
    .filter((group) => group.filters.length > 0),
];

export const SIDEBAR_FILTERS: SidebarBuiltinFilter[] = [
  { id: "exporting", label: "Exporting glyphs", source: "runebender" },
  { id: "incompatible", label: "Incompatible masters", source: "runebender" },
  ...GF_GLYPHSETS.filter((glyphset) =>
    ["GF_Latin_Core", "GF_Latin_Plus", "GF_Arabic_Core", "GF_Cyrillic_Core", "GF_Greek_Core"].includes(
      glyphset.id,
    ),
  ).map((glyphset) => ({
    id: glyphset.id,
    label: glyphset.label,
    source: "google-fonts-glyphsets" as const,
    glyphNames: glyphset.glyphNames,
    expectedCount: glyphset.expectedCount,
  })),
];
