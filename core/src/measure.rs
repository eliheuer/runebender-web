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

use kurbo::Point;

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

/// A straight edge (a line segment between two consecutive on-curve points),
/// captured for the span pass.
#[derive(Clone, Copy)]
struct Edge {
    /// Position on the measured axis (x for vertical edges, y for horizontal).
    pos: f64,
    /// Extent on the other axis.
    lo: f64,
    hi: f64,
}

/// A segment is treated as "vertical" / "horizontal" when its off-axis drift
/// is within this many units. Virtua's stems are dead-straight, so this stays
/// tight to avoid pulling in slightly-angled strokes.
const AXIS_TOL: f64 = 3.0;
/// Ignore spans and handles shorter than this (noise / coincident points).
const MIN_LEN: f64 = 8.0;
/// Two edges "face" each other only where their perpendicular extents overlap
/// by at least this much, so we don't measure between edges that never meet.
const MIN_OVERLAP: f64 = 16.0;

/// Compute every live measurement for a glyph's contours.
pub fn glyph_measurements(paths: &[Path]) -> Vec<Measurement> {
    let mut out = Vec::new();
    let mut verticals: Vec<Edge> = Vec::new();
    let mut horizontals: Vec<Edge> = Vec::new();

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

            // Straight edges: two consecutive on-curve points with no
            // off-curve between them form a line segment.
            if cur.is_on_curve() && nxt.is_on_curve() {
                let (a, b) = (cur.point, nxt.point);

                // Label the segment's own drawn length (any orientation).
                let seg_len = (b - a).hypot();
                if seg_len >= MIN_LEN {
                    out.push(Measurement {
                        a,
                        b,
                        length: seg_len.round() as i64,
                        kind: MeasureKind::Segment,
                    });
                }

                let dx = (a.x - b.x).abs();
                let dy = (a.y - b.y).abs();
                if dx <= AXIS_TOL && dy > MIN_LEN {
                    verticals.push(Edge {
                        pos: (a.x + b.x) / 2.0,
                        lo: a.y.min(b.y),
                        hi: a.y.max(b.y),
                    });
                } else if dy <= AXIS_TOL && dx > MIN_LEN {
                    horizontals.push(Edge {
                        pos: (a.y + b.y) / 2.0,
                        lo: a.x.min(b.x),
                        hi: a.x.max(b.x),
                    });
                }
            }
        }
    }

    collect_gaps(&mut verticals, MeasureKind::Horizontal, &mut out);
    collect_gaps(&mut horizontals, MeasureKind::Vertical, &mut out);
    out
}

/// From a set of parallel edges, emit a measurement for each adjacent
/// (by position) facing pair that overlaps on the perpendicular axis.
fn collect_gaps(edges: &mut [Edge], kind: MeasureKind, out: &mut Vec<Measurement>) {
    edges.sort_by(|a, b| a.pos.total_cmp(&b.pos));
    for w in edges.windows(2) {
        let (e0, e1) = (w[0], w[1]);
        let gap = e1.pos - e0.pos;
        if gap < MIN_LEN {
            continue;
        }
        let ov_lo = e0.lo.max(e1.lo);
        let ov_hi = e0.hi.min(e1.hi);
        if ov_hi - ov_lo < MIN_OVERLAP {
            continue;
        }
        let mid = (ov_lo + ov_hi) / 2.0;
        let (a, b) = match kind {
            MeasureKind::Horizontal => (Point::new(e0.pos, mid), Point::new(e1.pos, mid)),
            MeasureKind::Vertical => (Point::new(mid, e0.pos), Point::new(mid, e1.pos)),
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
