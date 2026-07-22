// Curve-smoothness analysis and editing geometry, shared by the Runebender
// curvature HUD (renderer) and the harmonize/balance tools.
//
// The design system's power-of-two discipline is a means (model-friendly
// data), not the goal: curve continuity outranks popcount (see virtua-grotesk
// DESIGN.md, "Curve smoothness comes before popcount"). This module gives the
// editor the tools to see and enforce that — a Speedpunk-style curvature comb,
// per-node continuity (G0/G1/G2/G3), and Curvatura/SuperTool harmonize + Tunni
// balance operations.
//
// Pure design-space geometry (font units), no render deps — unit-tested on
// native `cargo test`. Formulas verified against Simon Cozens' SuperTool and
// Linus Romer's Curvatura.

use kurbo::{Point, Vec2};

use crate::path::Path;

/// A cubic segment of an outline: on-curve `p0`/`p3`, off-curve handles
/// `p1`/`p2`. A straight line is stored as a cubic with handles on the chord
/// (`straight = true`, curvature 0).
#[derive(Clone, Copy, Debug)]
pub struct Cubic {
    pub p0: Point,
    pub p1: Point,
    pub p2: Point,
    pub p3: Point,
    pub straight: bool,
    /// Whether the on-curve point starting this segment (`p0`) is smooth.
    pub start_smooth: bool,
}

/// 2D cross product `u × v = u.x·v.y − u.y·v.x`.
fn cross(u: Vec2, v: Vec2) -> f64 {
    u.x * v.y - u.y * v.x
}

impl Cubic {
    /// First derivative at `t` (power-basis form).
    fn deriv(&self, t: f64) -> Vec2 {
        let a = (self.p3 - self.p0) + (self.p1 - self.p2) * 3.0;
        let b = (self.p2 - self.p1) * 3.0 - (self.p1 - self.p0) * 3.0;
        let c = (self.p1 - self.p0) * 3.0;
        a * (3.0 * t * t) + b * (2.0 * t) + c
    }

    fn deriv2(&self, t: f64) -> Vec2 {
        let a = (self.p3 - self.p0) + (self.p1 - self.p2) * 3.0;
        let b = (self.p2 - self.p1) * 3.0 - (self.p1 - self.p0) * 3.0;
        a * (6.0 * t) + b * 2.0
    }

    /// Point at `t`.
    pub fn eval(&self, t: f64) -> Point {
        let mt = 1.0 - t;
        (self.p0.to_vec2() * (mt * mt * mt)
            + self.p1.to_vec2() * (3.0 * mt * mt * t)
            + self.p2.to_vec2() * (3.0 * mt * t * t)
            + self.p3.to_vec2() * (t * t * t))
        .to_point()
    }

    /// Signed curvature at `t` — κ = (r'×r'') / |r'|³.
    pub fn curvature(&self, t: f64) -> f64 {
        if self.straight {
            return 0.0;
        }
        let d1 = self.deriv(t);
        let d2 = self.deriv2(t);
        let speed = d1.hypot();
        if speed < 1e-9 {
            return 0.0;
        }
        cross(d1, d2) / (speed * speed * speed)
    }

    /// Signed curvature at the start (`t=0`), closed form.
    /// κ(0) = (2/3)·cross(P1−P0, P2−P0) / |P1−P0|³.
    fn curvature_start(&self) -> f64 {
        if self.straight {
            return 0.0;
        }
        let h = self.p1 - self.p0;
        let len = h.hypot();
        if len < 1e-9 {
            return 0.0;
        }
        (2.0 / 3.0) * cross(h, self.p2 - self.p0) / (len * len * len)
    }

    /// Signed curvature at the end (`t=1`), closed form.
    /// κ(1) = (2/3)·cross(P3−P2, P1−P2) / |P3−P2|³.
    fn curvature_end(&self) -> f64 {
        if self.straight {
            return 0.0;
        }
        let h = self.p3 - self.p2;
        let len = h.hypot();
        if len < 1e-9 {
            return 0.0;
        }
        (2.0 / 3.0) * cross(h, self.p1 - self.p2) / (len * len * len)
    }

    /// Incoming tangent direction at the end (`t=1`).
    fn tangent_end(&self) -> Vec2 {
        let t = self.p3 - self.p2;
        if t.hypot() > 1e-9 { t } else { self.p3 - self.p0 }
    }

    /// Outgoing tangent direction at the start (`t=0`).
    fn tangent_start(&self) -> Vec2 {
        let t = self.p1 - self.p0;
        if t.hypot() > 1e-9 { t } else { self.p3 - self.p0 }
    }
}

/// Geometric-continuity level achieved at an on-curve node.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum GLevel {
    /// Intended corner (the node is not marked smooth).
    Corner,
    /// A node marked smooth whose tangents don't line up — a kink (defect).
    Kink,
    /// A line↔curve smooth join: G1 is the best achievable (curvature must
    /// jump 0→κ). Intended and acceptable (e.g. a stem meeting a bowl).
    G1Line,
    /// Tangent-continuous only, curve↔curve — a harmonize candidate.
    G1,
    /// Curvature-continuous.
    G2,
    /// Curvature-derivative-continuous.
    G3,
}

/// Continuity of one on-curve node joining an incoming and outgoing cubic.
#[derive(Clone, Copy, Debug)]
pub struct NodeContinuity {
    pub at: Point,
    pub level: GLevel,
    /// Relative curvature mismatch across the join (0 = perfectly G2).
    pub jump: f64,
}

/// Tangent-collinearity tolerance for G1 (~0.5°), tight enough to catch
/// integer-rounding kinks at points the designer marked smooth.
const G1_ANGLE_TOL: f64 = 0.009; // radians
/// Relative curvature tolerance for G2.
const G2_REL_TOL: f64 = 0.05;
/// Relative dκ tolerance for G3.
const G3_REL_TOL: f64 = 0.08;

/// Classify the continuity of every on-curve node of a glyph's contours.
pub fn node_continuity(paths: &[Path]) -> Vec<NodeContinuity> {
    let mut out = Vec::new();
    for path in paths {
        let segs = contour_cubics(path);
        let n = segs.len();
        if n < 2 {
            continue;
        }
        // Node k joins seg[k-1] (incoming, ends at the node) and seg[k]
        // (outgoing, starts at the node). p0 of seg[k] is the node.
        for k in 0..n {
            let out_seg = &segs[k];
            let in_seg = &segs[(k + n - 1) % n];
            let node = out_seg.p0;
            let level = classify(in_seg, out_seg);
            let (ki, ko) = (in_seg.curvature_end(), out_seg.curvature_start());
            let jump = (ki - ko).abs() / ki.abs().max(ko.abs()).max(1e-6);
            out.push(NodeContinuity {
                at: node,
                level,
                jump,
            });
        }
    }
    out
}

fn classify(in_seg: &Cubic, out_seg: &Cubic) -> GLevel {
    if !out_seg.start_smooth {
        return GLevel::Corner;
    }
    let ti = in_seg.tangent_end();
    let to = out_seg.tangent_start();
    let (li, lo) = (ti.hypot(), to.hypot());
    if li < 1e-9 || lo < 1e-9 {
        return GLevel::Corner;
    }
    let dot = ti.dot(to);
    let angle = cross(ti, to).abs().atan2(dot);
    if dot <= 0.0 || angle > G1_ANGLE_TOL {
        return GLevel::Kink;
    }
    // Two straight segments meeting smoothly are trivially G-continuous.
    if in_seg.straight && out_seg.straight {
        return GLevel::G2;
    }
    // A line meeting a curve can only reach G1 (curvature jumps 0→κ); that is
    // the intended best, not a defect.
    if in_seg.straight || out_seg.straight {
        return GLevel::G1Line;
    }
    let ki = in_seg.curvature_end();
    let ko = out_seg.curvature_start();
    let denom = ki.abs().max(ko.abs()).max(1e-6);
    let rel = (ki - ko).abs() / denom;
    if rel > G2_REL_TOL {
        return GLevel::G1;
    }
    // G3: compare dκ/ds just inside each side.
    let dki = (in_seg.curvature(1.0) - in_seg.curvature(0.97)) / 0.03;
    let dko = (out_seg.curvature(0.03) - out_seg.curvature(0.0)) / 0.03;
    let ddenom = dki.abs().max(dko.abs()).max(1e-6);
    if (dki - dko).abs() / ddenom < G3_REL_TOL {
        GLevel::G3
    } else {
        GLevel::G2
    }
}

/// One rib of the curvature-comb envelope: the point on the curve and the
/// pushed-out point, plus the curvature magnitude for coloring.
#[derive(Clone, Copy, Debug)]
pub struct CombSample {
    pub on: Point,
    pub outer: Point,
    pub kappa: f64,
}

/// Build the curvature comb for a glyph: per curved segment, a strip of
/// samples pushed out along the normal by `gain·|κ|·scale`. `scale` is a
/// design-space factor (so the comb zooms with the outline); `gain` is the
/// user multiplier. Straight segments are skipped (κ = 0). `signed` keeps the
/// curvature sign so the comb flips side at inflections.
pub fn curvature_comb(
    paths: &[Path],
    gain: f64,
    scale: f64,
    signed: bool,
    samples: usize,
) -> Vec<Vec<CombSample>> {
    let mut strips = Vec::new();
    let n = samples.max(2);
    for path in paths {
        for seg in contour_cubics(path) {
            if seg.straight {
                continue;
            }
            let mut strip = Vec::with_capacity(n + 1);
            for i in 0..=n {
                let t = i as f64 / n as f64;
                let d1 = seg.deriv(t);
                let speed = d1.hypot();
                if speed < 1e-9 {
                    continue;
                }
                let k = seg.curvature(t);
                // Unit normal = tangent rotated −90°: (d1.y, −d1.x)/|d1|.
                let normal = Vec2::new(d1.y / speed, -d1.x / speed);
                let mag = if signed { k } else { k.abs() };
                let on = seg.eval(t);
                let outer = on + normal * (mag * gain * scale);
                strip.push(CombSample { on, outer, kappa: k });
            }
            if strip.len() >= 2 {
                strips.push(strip);
            }
        }
    }
    strips
}

/// Peak |κ| across all curved segments — for auto-scaling the comb so the
/// tallest rib is a readable height.
pub fn max_curvature(paths: &[Path]) -> f64 {
    let mut m: f64 = 0.0;
    for path in paths {
        for seg in contour_cubics(path) {
            if seg.straight {
                continue;
            }
            for i in 0..=24 {
                m = m.max(seg.curvature(i as f64 / 24.0).abs());
            }
        }
    }
    m
}

/// Extract the ordered cubic segments of one contour. Consecutive on-curve
/// points with no handles between them become a straight cubic on the chord.
fn contour_cubics(path: &Path) -> Vec<Cubic> {
    use crate::path::PointType;
    let pts = path.points().as_slice();
    let n = pts.len();
    if n < 2 {
        return Vec::new();
    }
    let on: Vec<usize> = (0..n).filter(|&i| pts[i].is_on_curve()).collect();
    if on.len() < 2 {
        return Vec::new();
    }
    let smooth = |i: usize| matches!(pts[i].typ, PointType::OnCurve { smooth: true });
    let mut segs = Vec::with_capacity(on.len());
    for k in 0..on.len() {
        let a = on[k];
        let b = on[(k + 1) % on.len()];
        let mut offs = Vec::new();
        let mut i = (a + 1) % n;
        while i != b {
            offs.push(i);
            i = (i + 1) % n;
        }
        let (p0, p3) = (pts[a].point, pts[b].point);
        let start_smooth = smooth(a);
        match offs.as_slice() {
            [c1, c2] => segs.push(Cubic {
                p0,
                p1: pts[*c1].point,
                p2: pts[*c2].point,
                p3,
                straight: false,
                start_smooth,
            }),
            [c] => {
                // Quadratic → elevate to cubic for uniform handling.
                let q = pts[*c].point.to_vec2();
                let p1 = (p0.to_vec2() + (q - p0.to_vec2()) * (2.0 / 3.0)).to_point();
                let p2 = (p3.to_vec2() + (q - p3.to_vec2()) * (2.0 / 3.0)).to_point();
                segs.push(Cubic {
                    p0,
                    p1,
                    p2,
                    p3,
                    straight: false,
                    start_smooth,
                });
            }
            [] => segs.push(Cubic {
                p0,
                p1: (p0.to_vec2() + (p3.to_vec2() - p0.to_vec2()) / 3.0).to_point(),
                p2: (p0.to_vec2() + (p3.to_vec2() - p0.to_vec2()) * (2.0 / 3.0)).to_point(),
                p3,
                straight: true,
                start_smooth,
            }),
            _ => {}
        }
    }
    segs
}

/// Infinite-line intersection of line (a,b) with line (c,d). `None` if parallel.
fn line_intersect(a: Point, b: Point, c: Point, d: Point) -> Option<Point> {
    let r = b - a;
    let s = d - c;
    let denom = cross(r, s);
    if denom.abs() < 1e-9 {
        return None;
    }
    let t = cross(c - a, s) / denom;
    Some(a + r * t)
}

/// Harmonize a smooth on-curve `node`: given the incoming handles `a1`,`a2`
/// (`a2` adjacent to the node) and outgoing handles `b1`,`b2` (`b1` adjacent),
/// return the new positions of the two adjacent handles that make the join
/// curvature-continuous (G2) while keeping the on-curve point fixed
/// (SuperTool / Curvatura). `None` for degenerate configurations.
pub fn harmonize(a1: Point, a2: Point, node: Point, b1: Point, b2: Point) -> Option<(Point, Point)> {
    let d = line_intersect(a1, a2, b1, b2)?;
    let p0 = (a2 - a1).hypot() / (d - a2).hypot();
    let p1 = (b1 - d).hypot() / (b2 - b1).hypot();
    let r = (p0 * p1).sqrt();
    if !r.is_finite() {
        return None;
    }
    let t = r / (r + 1.0);
    let new_node = a2.lerp(b1, t);
    let fixup = node - new_node;
    Some((a2 + fixup, b1 + fixup))
}

/// Balance a cubic segment's handles (Tunni): move both handles to the same
/// fractional distance toward the Tunni point (handle-line intersection),
/// keeping their directions and the on-curve endpoints. Returns the new
/// `(p1, p2)`. `None` at inflections / degenerate segments.
pub fn balance(p0: Point, p1: Point, p2: Point, p3: Point) -> Option<(Point, Point)> {
    let s = line_intersect(p0, p1, p3, p2)?;
    let sd = (s - p0).hypot();
    let ed = (s - p3).hypot();
    if sd <= 1e-9 || ed <= 1e-9 {
        return None;
    }
    let x = (p1 - p0).hypot() / sd;
    let y = (p2 - p3).hypot() / ed;
    if (x > 1.0 && y > 1.0) || (x < 0.01 && y < 0.01) {
        return None;
    }
    let avg = (x + y) / 2.0;
    Some((p0.lerp(s, avg), p3.lerp(s, avg)))
}

#[cfg(test)]
mod tests {
    use super::*;

    // A cubic approximating a quarter circle of radius r: handle length
    // k·r with k = 4/3·(√2−1) ≈ 0.5523. Curvature ≈ 1/r.
    fn quarter_circle(r: f64) -> Cubic {
        let k = 0.5522847498 * r;
        Cubic {
            p0: Point::new(r, 0.0),
            p1: Point::new(r, k),
            p2: Point::new(k, r),
            p3: Point::new(0.0, r),
            straight: false,
            start_smooth: true,
        }
    }

    #[test]
    fn curvature_of_circle() {
        let c = quarter_circle(100.0);
        // Endpoint curvature magnitude ~ 1/100 (sign depends on winding).
        assert!((c.curvature_start().abs() - 0.01).abs() < 0.001);
        assert!((c.curvature_end().abs() - 0.01).abs() < 0.001);
    }

    #[test]
    fn straight_has_zero_curvature() {
        let s = Cubic {
            p0: Point::new(0.0, 0.0),
            p1: Point::new(33.0, 0.0),
            p2: Point::new(66.0, 0.0),
            p3: Point::new(100.0, 0.0),
            straight: true,
            start_smooth: false,
        };
        assert_eq!(s.curvature(0.5), 0.0);
    }

    #[test]
    fn harmonize_makes_join_g2() {
        // Two cubics meeting smoothly at the origin, shared vertical tangent,
        // both curving the same way (G1 but not G2 — different curvatures).
        let node = Point::new(0.0, 0.0);
        let a2 = Point::new(0.0, -30.0); // incoming handle adjacent to node
        let a1 = Point::new(-70.0, -50.0); // far incoming handle
        let b1 = Point::new(0.0, 50.0); // outgoing handle adjacent to node
        let b2 = Point::new(-80.0, 80.0); // far outgoing handle
        let inc = Cubic { p0: Point::new(-100.0, -90.0), p1: a1, p2: a2, p3: node, straight: false, start_smooth: true };
        let out = Cubic { p0: node, p1: b1, p2: b2, p3: Point::new(-120.0, 120.0), straight: false, start_smooth: true };
        let before = (inc.curvature_end() - out.curvature_start()).abs();
        let (na2, nb1) = harmonize(a1, a2, node, b1, b2).unwrap();
        let inc2 = Cubic { p2: na2, ..inc };
        let out2 = Cubic { p1: nb1, ..out };
        let after = (inc2.curvature_end() - out2.curvature_start()).abs();
        assert!(after < before * 0.3, "harmonize should nearly equalize curvature: {before} -> {after}");
    }

    #[test]
    fn balance_keeps_endpoints() {
        let (p0, p3) = (Point::new(0.0, 0.0), Point::new(100.0, 0.0));
        let (np1, np2) = balance(p0, Point::new(20.0, 50.0), Point::new(90.0, 60.0), p3).unwrap();
        // Handles stay on their original rays from the endpoints.
        assert!((np1 - p0).cross(Point::new(20.0, 50.0) - p0).abs() < 1e-6);
        assert!((np2 - p3).cross(Point::new(90.0, 60.0) - p3).abs() < 1e-6);
    }
}
