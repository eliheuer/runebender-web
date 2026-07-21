// Live grid measurements for the on-canvas HUD: handle lengths, and the
// horizontal/vertical spans between facing straight edges. The span pass
// yields stem widths AND counters from the same logic — a counter is just
// the gap between two facing near-vertical inner edges — so the designer
// sees every measurement that matters while drawing, on Virtua Grotesk's
// power-of-two grid.
//
// Everything here is design-space geometry (font units). The renderer maps
// it to the screen and draws ticks + labels; popcount/tier styling reads
// off the length. Kept ungated and free of render deps so it unit-tests on
// native `cargo test`.

use kurbo::{BezPath, Line, ParamCurve, Point, Shape};

use crate::path::Path;

/// What a single measurement describes.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum MeasureKind {
    /// On-curve point to its off-curve control (a Bézier handle).
    Handle,
    /// A straight outline segment: two consecutive on-curve points, its own
    /// drawn length (any orientation, including diagonals).
    Segment,
    /// Horizontal span between two facing near-vertical edges (stem/counter).
    Horizontal,
    /// Vertical span between two facing near-horizontal edges (bar/height).
    Vertical,
}

/// One measurement in design space. `a`/`b` are the endpoints of the span
/// (for a Handle, `a` is the on-curve anchor); `length` is the rounded
/// design-unit distance to label.
#[derive(Clone, Copy, Debug)]
pub struct Measurement {
    pub a: Point,
    pub b: Point,
    pub length: i64,
    pub kind: MeasureKind,
}

/// Ignore spans, segments, and handles shorter than this (noise / coincident
/// points / near-tangent scan crossings).
const MIN_LEN: f64 = 8.0;

/// Compute every live measurement for a glyph's contours: handle lengths,
/// straight segment lengths, and — via a horizontal and vertical scan line
/// through the glyph center — the stroke thicknesses and counter/bowl spans
/// between facing outline crossings. The scan-line pass works on curves and
/// straights alike, so an all-curve `o` measures the same as a straight `n`.
pub fn glyph_measurements(paths: &[Path]) -> Vec<Measurement> {
    let mut out = Vec::new();

    for path in paths {
        let pts = path.points().as_slice();
        let n = pts.len();
        if n < 2 {
            continue;
        }
        for i in 0..n {
            let cur = &pts[i];
            let nxt = &pts[(i + 1) % n];

            // Handles: an off-curve point paired with its adjacent on-curve
            // anchor. Each off-curve has exactly one on-curve neighbor.
            if !cur.is_on_curve() {
                let prev = &pts[(i + n - 1) % n];
                let anchor = if prev.is_on_curve() {
                    Some(prev)
                } else if nxt.is_on_curve() {
                    Some(nxt)
                } else {
                    None
                };
                if let Some(anchor) = anchor {
                    let len = (cur.point - anchor.point).hypot();
                    if len >= MIN_LEN {
                        out.push(Measurement {
                            a: anchor.point,
                            b: cur.point,
                            length: len.round() as i64,
                            kind: MeasureKind::Handle,
                        });
                    }
                }
            }

            // Straight segment: two consecutive on-curve points, its own length.
            if cur.is_on_curve() && nxt.is_on_curve() {
                let (a, b) = (cur.point, nxt.point);
                let seg_len = (b - a).hypot();
                if seg_len >= MIN_LEN {
                    out.push(Measurement {
                        a,
                        b,
                        length: seg_len.round() as i64,
                        kind: MeasureKind::Segment,
                    });
                }
            }
        }
    }

    scan_spans(paths, &mut out);
    out
}

/// Cast a horizontal and a vertical line through the glyph's center and emit a
/// span for each gap between consecutive outline crossings — the stroke
/// thicknesses and the counter/bowl, for curved and straight glyphs alike.
fn scan_spans(paths: &[Path], out: &mut Vec<Measurement>) {
    let mut bez = BezPath::new();
    for p in paths {
        p.append_to_bezpath(&mut bez);
    }
    if bez.elements().is_empty() {
        return;
    }
    let bbox = bez.bounding_box();
    if bbox.width() < MIN_LEN || bbox.height() < MIN_LEN {
        return;
    }
    let cx = (bbox.x0 + bbox.x1) / 2.0;
    let cy = (bbox.y0 + bbox.y1) / 2.0;

    let hline = Line::new((bbox.x0 - 1.0, cy), (bbox.x1 + 1.0, cy));
    let mut xs: Vec<f64> = bez
        .segments()
        .flat_map(|seg| {
            seg.intersect_line(hline)
                .into_iter()
                .map(move |hit| hline.eval(hit.line_t).x)
        })
        .collect();
    emit_scan(&mut xs, cy, MeasureKind::Horizontal, out);

    let vline = Line::new((cx, bbox.y0 - 1.0), (cx, bbox.y1 + 1.0));
    let mut ys: Vec<f64> = bez
        .segments()
        .flat_map(|seg| {
            seg.intersect_line(vline)
                .into_iter()
                .map(move |hit| vline.eval(hit.line_t).y)
        })
        .collect();
    emit_scan(&mut ys, cx, MeasureKind::Vertical, out);
}

/// Sort crossings along a scan line, drop near-duplicates, and emit a span for
/// each consecutive gap. `fixed` is the scan line's constant coordinate.
fn emit_scan(coords: &mut [f64], fixed: f64, kind: MeasureKind, out: &mut Vec<Measurement>) {
    coords.sort_by(|a, b| a.total_cmp(b));
    let mut prev = f64::NEG_INFINITY;
    let mut kept: Vec<f64> = Vec::with_capacity(coords.len());
    for &c in coords.iter() {
        if c - prev >= 2.0 {
            kept.push(c);
            prev = c;
        }
    }
    for w in kept.windows(2) {
        let gap = w[1] - w[0];
        if gap < MIN_LEN {
            continue;
        }
        let (a, b) = match kind {
            MeasureKind::Horizontal => (Point::new(w[0], fixed), Point::new(w[1], fixed)),
            MeasureKind::Vertical => (Point::new(fixed, w[0]), Point::new(fixed, w[1])),
            _ => unreachable!(),
        };
        out.push(Measurement {
            a,
            b,
            length: gap.round() as i64,
            kind,
        });
    }
}

/// A drawn outline piece (straight segment, curve, or handle line) tagged
/// with the popcount that colors it, so the outline itself can echo the
/// label colors and link each number to its geometry.
#[derive(Clone)]
pub struct ColoredStroke {
    /// The piece in design space.
    pub path: BezPath,
    /// Popcount driving the tier color.
    pub popcount: u32,
    /// True for outline pieces (segments/curves, drawn at the path width),
    /// false for handle lines (drawn thinner).
    pub wide: bool,
}

/// Break each contour into colorable pieces: straight segments and curves at
/// the outline width, plus their handle lines thinner. A segment is colored by
/// its own length; a curve by the worse popcount of its two handles.
pub fn colored_strokes(paths: &[Path]) -> Vec<ColoredStroke> {
    let mut out = Vec::new();
    for path in paths {
        let pts = path.points().as_slice();
        let n = pts.len();
        if n < 2 {
            continue;
        }
        let on: Vec<usize> = (0..n).filter(|&i| pts[i].is_on_curve()).collect();
        if on.len() < 2 {
            continue;
        }
        for k in 0..on.len() {
            let a = on[k];
            let b = on[(k + 1) % on.len()];
            // Off-curve points strictly between the two on-curve anchors.
            let mut offs = Vec::new();
            let mut i = (a + 1) % n;
            while i != b {
                offs.push(i);
                i = (i + 1) % n;
            }
            let (pa, pb) = (pts[a].point, pts[b].point);

            match offs.as_slice() {
                [] => push_line(&mut out, pa, pb, true),
                [c] => {
                    let cp = pts[*c].point;
                    let mut bp = BezPath::new();
                    bp.move_to(pa);
                    bp.quad_to(cp, pb);
                    let pc = popcount((cp - pa).hypot().round() as i64)
                        .max(popcount((cp - pb).hypot().round() as i64));
                    out.push(ColoredStroke {
                        path: bp,
                        popcount: pc,
                        wide: true,
                    });
                    push_line(&mut out, pa, cp, false);
                    push_line(&mut out, pb, cp, false);
                }
                [c1, c2] => {
                    let (cp1, cp2) = (pts[*c1].point, pts[*c2].point);
                    let mut bp = BezPath::new();
                    bp.move_to(pa);
                    bp.curve_to(cp1, cp2, pb);
                    let pc = popcount((cp1 - pa).hypot().round() as i64)
                        .max(popcount((cp2 - pb).hypot().round() as i64));
                    out.push(ColoredStroke {
                        path: bp,
                        popcount: pc,
                        wide: true,
                    });
                    push_line(&mut out, pa, cp1, false);
                    push_line(&mut out, pb, cp2, false);
                }
                _ => {}
            }
        }
    }
    out
}

/// Side-bearing geometry: the horizontal gaps between the advance margins
/// (x=0 and x=advance) and the glyph's leftmost/rightmost points, plus where
/// those extreme points and the glyph's vertical extent are, for drawing.
#[derive(Clone, Copy)]
pub struct SideBearings {
    pub advance: f64,
    pub lsb: i64,
    pub rsb: i64,
    /// Extreme x positions of the outline (furthest-left / furthest-right).
    pub min_x: f64,
    pub max_x: f64,
    /// y of the leftmost / rightmost point (so the SB line points at it).
    pub y_left: f64,
    pub y_right: f64,
}

/// Left/right side bearings and the extreme-point positions. `None` for an
/// empty glyph. LSB = leftmost x (from origin); RSB = advance − rightmost x.
pub fn side_bearings(paths: &[Path], advance: f64) -> Option<SideBearings> {
    let mut bez = BezPath::new();
    for p in paths {
        p.append_to_bezpath(&mut bez);
    }
    if bez.elements().is_empty() {
        return None;
    }
    let bbox = bez.bounding_box();
    let (min_x, max_x) = (bbox.x0, bbox.x1);

    // y of the extreme on-curve points, so each SB line ends at the point.
    let mid_y = (bbox.y0 + bbox.y1) / 2.0;
    let (mut y_left, mut y_right) = (mid_y, mid_y);
    let (mut best_l, mut best_r) = (f64::MAX, f64::MAX);
    for p in paths {
        for pt in p.points().as_slice() {
            if !pt.is_on_curve() {
                continue;
            }
            let dl = (pt.point.x - min_x).abs();
            if dl < best_l {
                best_l = dl;
                y_left = pt.point.y;
            }
            let dr = (pt.point.x - max_x).abs();
            if dr < best_r {
                best_r = dr;
                y_right = pt.point.y;
            }
        }
    }

    Some(SideBearings {
        advance,
        lsb: min_x.round() as i64,
        rsb: (advance - max_x).round() as i64,
        min_x,
        max_x,
        y_left,
        y_right,
    })
}

/// Append a colored line piece if it is long enough to be meaningful.
fn push_line(out: &mut Vec<ColoredStroke>, p0: Point, p1: Point, wide: bool) {
    let len = (p1 - p0).hypot();
    if len < MIN_LEN {
        return;
    }
    let mut bp = BezPath::new();
    bp.move_to(p0);
    bp.line_to(p1);
    out.push(ColoredStroke {
        path: bp,
        popcount: popcount(len.round() as i64),
        wide,
    });
}

/// Popcount (Hamming weight) of a length: the number of powers of two it is
/// the sum of. Lower = more structural.
pub fn popcount(value: i64) -> u32 {
    (value.max(0) as u64).count_ones()
}

/// The label for a length: `"96 = 64+32"`, `"256 = 2^8"`. Pure powers get an
/// exponent; sums list their powers high-to-low. Caret notation is used
/// rather than Unicode superscripts because the embedded HUD font has no
/// superscript glyphs (they render as tofu).
pub fn label(value: i64) -> String {
    if value <= 0 {
        return value.to_string();
    }
    let v = value as u64;
    if v.is_power_of_two() {
        return format!("{value} = 2^{}", v.trailing_zeros());
    }
    let mut parts = Vec::new();
    for bit in (0..64).rev() {
        if v & (1u64 << bit) != 0 {
            parts.push((1u64 << bit).to_string());
        }
    }
    format!("{value} = {}", parts.join("+"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn labels_and_popcount() {
        assert_eq!(label(256), "256 = 2^8");
        assert_eq!(label(96), "96 = 64+32");
        assert_eq!(label(272), "272 = 256+16");
        assert_eq!(popcount(256), 1);
        assert_eq!(popcount(96), 2);
        assert_eq!(popcount(116), 4); // 64+32+16+4
    }
}
