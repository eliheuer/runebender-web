// Glyph → SVG geometry for the grid thumbnails.
//
// Lives outside `wasm_api` (which is `#[cfg(target_arch = "wasm32")]`)
// so the box math is covered by the normal `cargo test` run rather than
// only being exercised in the browser.

// The only caller is `wasm_api`, which is wasm-only, so a host build sees
// this module as unused. The tests are the point of compiling it there.
#![cfg_attr(not(target_arch = "wasm32"), allow(dead_code))]

use kurbo::{BezPath, Shape};

/// Fraction of the preview box the em occupies.
const EM_FILL: f64 = 0.65;
/// Where the baseline sits, measured down from the top of the box.
const BASELINE_FROM_TOP: f64 = 0.80;

/// A grid-thumbnail SVG: constant vertical extent (so every glyph in the
/// grid renders at one scale, with one baseline) and a per-glyph
/// horizontal extent (so each is centered on its own advance).
///
/// The vertical em window is a *minimum*, not a crop. Arabic descends far
/// below the baseline — hah and jeem, and the final and isolate forms of
/// most of the alphabet — and Latin ascenders and swashes can run past
/// the top. Anything outside the viewBox is clipped by the SVG root,
/// which is worse than the small scale difference from growing the box,
/// so the box grows to hold the ink.
///
/// Returns an empty string for an empty path.
pub(crate) fn grid_thumbnail_svg(bez: &BezPath, upm: f64) -> String {
    if bez.elements().is_empty() {
        return String::new();
    }
    let bbox = bez.bounding_box();
    let upm = if upm > 0.0 { upm } else { 1000.0 };

    let em_height = upm / EM_FILL;
    // In flipped (svg y-down) space the box starts above the baseline and
    // extends downward, leaving the baseline BASELINE_FROM_TOP of the way
    // down.
    let em_min_y = -BASELINE_FROM_TOP * em_height;

    // The path is drawn flipped, so its ink spans -bbox.y1 (top) to
    // -bbox.y0 (bottom).
    let min_y = em_min_y.min(-bbox.y1);
    let max_y = (em_min_y + em_height).max(-bbox.y0);

    format!(
        r#"<svg xmlns="http://www.w3.org/2000/svg" viewBox="{} {} {} {}" preserveAspectRatio="xMidYMid meet"><path d="{}" fill="currentColor" fill-rule="nonzero" transform="scale(1 -1)"/></svg>"#,
        bbox.x0,
        min_y,
        bbox.width(),
        max_y - min_y,
        bez.to_svg(),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Parse `viewBox="x y w h"` out of a generated thumbnail.
    fn view_box(svg: &str) -> (f64, f64, f64, f64) {
        let start = svg.find("viewBox=\"").expect("thumbnail has a viewBox") + 9;
        let rest = &svg[start..];
        let end = rest.find('"').expect("viewBox is quoted");
        let nums: Vec<f64> = rest[..end]
            .split_whitespace()
            .map(|n| n.parse().expect("viewBox numbers parse"))
            .collect();
        (nums[0], nums[1], nums[2], nums[3])
    }

    /// A rectangle from (0, y0) to (600, y1) in font units.
    fn box_path(y0: f64, y1: f64) -> BezPath {
        let mut path = BezPath::new();
        path.move_to((0.0, y0));
        path.line_to((600.0, y0));
        path.line_to((600.0, y1));
        path.line_to((0.0, y1));
        path.close_path();
        path
    }

    /// Ink extent in viewBox (flipped) space: (top, bottom).
    fn ink_span(y0: f64, y1: f64) -> (f64, f64) {
        (-y1, -y0)
    }

    /// Slack for the assertions below. `min_y + height` re-rounds, so an
    /// exact comparison trips on the last bit; a millionth of a font unit
    /// is far below anything that could show on screen.
    const EPS: f64 = 1e-6;

    #[test]
    fn glyphs_that_fit_share_one_em_window() {
        // Two ordinary glyphs of different heights must get the same
        // viewBox, or the grid renders them at different scales and off
        // a common baseline.
        let cap = view_box(&grid_thumbnail_svg(&box_path(0.0, 700.0), 1000.0));
        let xheight = view_box(&grid_thumbnail_svg(&box_path(0.0, 500.0), 1000.0));
        assert_eq!(cap.1, xheight.1);
        assert_eq!(cap.3, xheight.3);
        assert!((cap.3 - 1000.0 / EM_FILL).abs() < 1e-6);
    }

    #[test]
    fn box_grows_to_hold_deep_descenders() {
        let (top, bottom) = ink_span(-900.0, 700.0);
        let (_, min_y, _, height) = view_box(&grid_thumbnail_svg(&box_path(-900.0, 700.0), 1000.0));
        assert!(min_y <= top + EPS, "viewBox top {min_y} clips ink top {top}");
        assert!(
            min_y + height >= bottom - EPS,
            "viewBox bottom {} clips ink bottom {bottom}",
            min_y + height
        );
    }

    #[test]
    fn box_grows_to_hold_tall_ascenders() {
        let (top, bottom) = ink_span(0.0, 1400.0);
        let (_, min_y, _, height) = view_box(&grid_thumbnail_svg(&box_path(0.0, 1400.0), 1000.0));
        assert!(min_y <= top + EPS);
        assert!(min_y + height >= bottom - EPS);
    }

    #[test]
    fn horizontal_extent_is_the_glyphs_own_ink() {
        let (x, _, width, _) = view_box(&grid_thumbnail_svg(&box_path(0.0, 700.0), 1000.0));
        assert_eq!(x, 0.0);
        assert_eq!(width, 600.0);
    }

    #[test]
    fn empty_path_makes_no_svg() {
        assert!(grid_thumbnail_svg(&BezPath::new(), 1000.0).is_empty());
    }

    #[test]
    fn zero_upm_falls_back_to_1000() {
        let fallback = view_box(&grid_thumbnail_svg(&box_path(0.0, 700.0), 0.0));
        let explicit = view_box(&grid_thumbnail_svg(&box_path(0.0, 700.0), 1000.0));
        assert_eq!(fallback, explicit);
    }
}
