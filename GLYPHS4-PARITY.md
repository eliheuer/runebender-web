# Glyphs 4 parity

What Glyphs 4 does that Runebender web does not, as of August 2026.

Glyphs 4.0 shipped on 27 July 2026. The goal here is not a pixel copy —
it is that a designer moving between the two apps is never stopped by a
missing capability. This list is the working backlog for that.

Status marks:

- **none** — nothing in Runebender does this
- **partial** — something works, but not enough to do the job
- **have** — close enough to Glyphs for real work

---

## 1. Export and compilation — **none**

Runebender cannot produce a font binary. There is no TTF, OTF, WOFF2 or
variable-font export, no instance generation, and no hinting. A designer
can draw here but has to leave to ship anything.

| Feature | Status | Notes |
| --- | --- | --- |
| Static TTF/OTF export | none | |
| Variable font export | none | Glyphs 4 also added CFF2 variable export |
| Instances / exports list | none | No instance definitions in the UI at all |
| TrueType and PostScript hinting, autohint | none | |
| WOFF2, web font packaging | none | |
| Icon set export (`.glyphsicons`, SF Symbols) | none | Glyphs 4 feature, low priority for us |

Everything else on this list is polish next to this one. The font
compiler is already in-tree in a sense — fea-rs and write-fonts are
linked for shaping — so a first TTF export is less far off than it looks.

## 2. Font-level data — mostly **none**

| Feature | Status | Notes |
| --- | --- | --- |
| Font Info window (names, vertical metrics, version, licence) | none | Read from the UFO, never editable |
| Masters: add, remove, reorder, duplicate | none | Masters are whatever the designspace declares |
| Axes: add, edit, rename, axis mappings (`avar`) | partial | Sliders work; the axes themselves are read-only |
| Instances and their custom parameters | none | |
| Custom parameters at any level | none | |
| Glyph order and sorting rules | partial | Grid follows `public.glyphOrder`; not editable |
| Glyph info database (category, script, production name) | partial | We categorise by codepoint; no editable per-glyph info |
| Create glyphs from recipes (`aacute = a + acutecomb`) | none | Glyphs builds composites from a recipe language |

## 3. Drawing tools

| Feature | Status | Notes |
| --- | --- | --- |
| Select, Pen, Knife, Measure, Shapes, Hand, Text | have | |
| Hyper-pen / spline pen | have | Ours goes further than Glyphs here |
| Sketch → outline tracing | have | img2bez; Glyphs has no equivalent |
| **Star nodes** (non-destructive G2) | none | Headline Glyphs 4 feature. We have harmonize, but it is a one-shot edit, not a live constraint |
| **Pen points** (offset/width along a path) | none | Headline Glyphs 4 feature: stroked paths with ease-in/out |
| Node-based stroke settings | none | |
| Rotate from the bounding box, 45° steps, slant with Option | partial | We have rotate buttons, no bounding-box rotation handles |
| Scale/skew from the bounding box | partial | Transform panel only |
| Corner and cap components | none | |
| Remove overlap | have | Boolean union |
| Offset curve | none | |
| Round corners | none | |
| Slanter filter (with corrections) | none | New in Glyphs 4 |
| Simplify filter | none | New in Glyphs 4; would pair well with the sketch tool |
| Hatch, Decompose, Tidy Up Paths filters | none | |
| Filter API / scripted filters | none | |

## 4. Spacing and kerning

| Feature | Status | Notes |
| --- | --- | --- |
| Sidebearings, width editing | have | |
| Kerning groups on a glyph | partial | Fields exist; no group manager |
| **Visual kerning groups** (drag glyphs onto a shelf) | none | Headline Glyphs 4 feature |
| Kerning panel: list, search, clean up, compress | none | |
| Manual kerning by dragging in the text view | have | Shift-drag |
| Context kerning | none | New in Glyphs 4 |
| Metrics keys (`=H`, `=|n`, formulas) | none | Big daily-workflow gap |
| Auto-spacing | none | |

## 5. Interpolation and variable fonts

| Feature | Status | Notes |
| --- | --- | --- |
| Axis sliders with live interpolated preview | have | |
| Compatibility checking between masters | have | |
| Master switching in the editor | have | |
| Intermediate / brace layers | none | Glyphs 4 unified smart layers and brace layers |
| Smart components (glyph-level axes) | none | |
| Editing an interpolated instance | partial | We snap to the nearest master; Fontra-style "create source here" is not built |
| Axis particles (auto-generate master/instance grids) | none | New in Glyphs 4 |
| `avar` 2 support | none | |
| Move on Axis panel | none | |

## 6. Layers, backgrounds, guides

| Feature | Status | Notes |
| --- | --- | --- |
| Per-glyph background layer | none | Nothing to draw against or toggle |
| Layers panel (masters, backups, brace layers) | none | |
| Global and local guides | none | Only metric lines |
| Measurement guides | partial | Measure tool, no persistent guides |
| Background image per glyph | partial | Sketch/trace only; no placed-image workflow |
| Grid and subdivision settings | partial | Design grid is fixed |

## 7. Anchors and marks

| Feature | Status | Notes |
| --- | --- | --- |
| Place, move, multi-select anchors | have | |
| Anchors shown on composites (propagated) | have | Read-only, which matches Glyphs |
| Mark cloud (see every mark that attaches here) | none | Glyphs 4 improved this |
| `@metrics`-relative anchor positions | none | New in Glyphs 4 |
| Anchor name validation | none | |
| Automatic anchor placement | none | |

## 8. OpenType features

| Feature | Status | Notes |
| --- | --- | --- |
| Compile `features.fea` for the editor's own shaping | have | fea-rs + harfrust, added Aug 2026 |
| Feature editor (edit, reorder, auto-generate) | none | We read the file; we never write it |
| Automatic feature generation (`ccmp`, `locl`, `init/medi/fina`) | none | Glyphs writes these for you |
| Feature variations | none | |
| Classes and prefixes UI | none | |

## 9. Colour fonts — **none**

| Feature | Status | Notes |
| --- | --- | --- |
| Colour layers, palettes (CPAL) | none | |
| COLRv1 export, gradients, blend modes | none | Headline Glyphs 4 feature |
| SVG colour fonts | none | |

## 10. Proofing and preview

| Feature | Status | Notes |
| --- | --- | --- |
| Text preview strip | have | |
| Waterfall preview | partial | Sizes cascade in the glyph tile; not a real proof sheet |
| Multi-line text editing with real shaping | have | Per-line bidi, harfrust shaping |
| Print proof / PDF specimen | none | |
| Image export of the canvas | none | |
| Variable preview panel (test an instance) | partial | Sliders drive the canvas, no separate panel |

## 11. Scripting and extensibility — **none**

| Feature | Status | Notes |
| --- | --- | --- |
| Macro window / Python console | none | |
| Plugin API (filters, reporters, palettes) | none | |
| Scripted batch operations across glyphs | none | |

## 12. Where we are ahead

Worth keeping in view so parity work does not trade these away.

- Runs in a browser with no install, on a real font on disk.
- Live reload and ETag-guarded saves, so an agent or another tool can
  edit the same source while the editor is open.
- The measurement HUD: popcount sums, colourised outlines, stem and
  counter spans. Nothing in Glyphs does this.
- Sketch-to-outline tracing.
- The hyper-pen.

---

## Suggested order

1. **Export.** Nothing else matters if the work cannot ship. A static
   TTF first, then variable.
2. **Font Info and masters/axes/instances editing.** Needed before export
   is genuinely useful, and it unblocks intermediate layers.
3. **Metrics keys and the kerning panel.** The two spacing gaps a
   working designer hits every day.
4. **Background layer and guides.** Cheap, and missed constantly.
5. **Intermediate (brace) layers.** The big interpolation gap.
6. **Feature editor.** We already compile features; writing them is the
   other half.
7. **Filters** (simplify, round corners, offset, slant) — each is small
   and self-contained, good work to slot between larger pieces.
8. **Colour fonts** and **scripting** — large, and neither blocks a
   text-face workflow.

Sources: [Glyphs 4 release notes](https://updates.glyphsapp.com/Glyphs4.0-4000.html),
[Glyphs 4 announcement](https://glyphsapp.com/news/glyphs-4-create-love-the-process).
