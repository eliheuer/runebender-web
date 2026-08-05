// On-canvas shaped text for HUD overlays (measurement readouts today, a
// foundation for on-canvas proofing tomorrow). Built on the Linebender
// text stack — parley (layout) + harfrust (shaping) + fontique (font
// management) — and drawn through vello's `Scene::draw_glyphs`, the same
// GPU path the glyph outlines use.
//
// A single font (Virtua Grotesk Regular) is embedded and registered so
// labels render regardless of which glyph or font is open in the editor,
// and so a glyph drawn before its own digits exist still gets labels. The
// font is a frozen compiled snapshot, independent of the live UFO source.

use std::sync::Arc;

use kurbo::{Affine, Stroke};
use parley::{
    FontContext, FontFamily, FontFamilyName, LayoutContext, PositionedLayoutItem, StyleProperty,
};
use vello::Glyph;
use vello::peniko::{Blob, Fill, color::AlphaColor};

type Srgb = AlphaColor<vello::peniko::color::Srgb>;

/// The embedded HUD typeface. Compiled Virtua Grotesk Regular; measurement
/// numerals are therefore set in the very typeface being measured.
const HUD_FONT: &[u8] = include_bytes!("../assets/hud/VirtuaGrotesk-Regular.ttf");

/// Shaped-text renderer for HUD labels. Holds the parley contexts and the
/// registered family name; reused across frames so the font is parsed once.
pub(crate) struct HudText {
    font_cx: FontContext,
    layout_cx: LayoutContext<()>,
    family: String,
}

impl HudText {
    pub(crate) fn new() -> Self {
        let mut font_cx = FontContext::new();
        let blob = Blob::new(Arc::new(HUD_FONT) as Arc<dyn AsRef<[u8]> + Send + Sync>);
        let registered = font_cx.collection.register_fonts(blob, None);
        let family = registered
            .first()
            .and_then(|(id, _)| font_cx.collection.family_name(*id))
            .unwrap_or("sans-serif")
            .to_string();
        Self {
            font_cx,
            layout_cx: LayoutContext::new(),
            family,
        }
    }

    /// Draw a single line of text into `scene`. `top_left` is the label's
    /// top-left corner in device (screen) pixels; `px` is the font size in
    /// device pixels; `color` fills every glyph. Shaping (kerning, and any
    /// script joining the string needs) is applied by harfrust.
    ///
    /// Each glyph is stroked in `halo` before it is filled, the same casing
    /// the outline and handles get: a label lands on top of whatever the
    /// canvas is already showing — a lit curvature comb, a filled counter —
    /// and without it the numerals disappear into the busy parts.
    pub(crate) fn draw_line(
        &mut self,
        scene: &mut vello::Scene,
        text: &str,
        top_left: kurbo::Point,
        px: f32,
        color: Srgb,
        halo: Srgb,
    ) {
        // Clone the family name so the immutable borrow of `self.family`
        // doesn't collide with the mutable borrows of the parley contexts.
        let family = self.family.clone();
        let mut builder = self
            .layout_cx
            .ranged_builder(&mut self.font_cx, text, 1.0, true);
        builder.push_default(StyleProperty::FontFamily(FontFamily::Single(
            FontFamilyName::named(family.as_str()),
        )));
        builder.push_default(StyleProperty::FontSize(px));
        let mut layout: parley::Layout<()> = builder.build(text);
        layout.break_all_lines(None);

        // Scaled off the type size so the casing reads the same whether the
        // label is small or large; the stroke is centered, so half of this
        // width sits outside the glyph.
        let halo_width = (px as f64 * 0.28).clamp(2.0, 5.0);

        let transform = Affine::translate((top_left.x, top_left.y));
        for line in layout.lines() {
            for item in line.items() {
                let PositionedLayoutItem::GlyphRun(glyph_run) = item else {
                    continue;
                };
                let run = glyph_run.run();
                let font = run.font();
                let font_size = run.font_size();
                let coords = run.normalized_coords();
                scene
                    .draw_glyphs(font)
                    .font_size(font_size)
                    .brush(halo)
                    .transform(transform)
                    .normalized_coords(coords)
                    .draw(
                        &Stroke::new(halo_width),
                        glyph_run.positioned_glyphs().map(|g| Glyph {
                            id: g.id,
                            x: g.x,
                            y: g.y,
                        }),
                    );
                scene
                    .draw_glyphs(font)
                    .font_size(font_size)
                    .brush(color)
                    .transform(transform)
                    .normalized_coords(coords)
                    .draw(
                        Fill::NonZero,
                        glyph_run.positioned_glyphs().map(|g| Glyph {
                            id: g.id,
                            x: g.x,
                            y: g.y,
                        }),
                    );
            }
        }
    }
}
