// Copyright 2026 the Runebender Authors
// SPDX-License-Identifier: Apache-2.0

// Curve-smoothness analysis and editing geometry. The pure geometry
// (Cubic, continuity classification, curvature comb, harmonize /
// balance / optimize) lives in `runebender_core::curve`, shared by
// all Runebender editors; this module keeps the web editor's
// `Path`-based API by converting contours to cubic segments here.


pub use runebender_core::curve::{
    balance, curvature_end, curvature_start, harmonize, max_curvature as max_curvature_cubics,
    node_continuity as node_continuity_cubics, optimize_contour, popcount, CombSample, Cubic,
    GLevel, NodeContinuity, OptPoint,
};

use crate::path::Path;

/// Classify the continuity of every on-curve node of a glyph's contours.
pub fn node_continuity(paths: &[Path]) -> Vec<NodeContinuity> {
    node_continuity_cubics(&all_cubics(paths))
}

/// Build the curvature comb for a glyph (see `runebender_core::curve`).
pub fn curvature_comb(
    paths: &[Path],
    gain: f64,
    scale: f64,
    signed: bool,
    samples: usize,
) -> Vec<Vec<CombSample>> {
    runebender_core::curve::curvature_comb(&all_cubics(paths), gain, scale, signed, samples)
}

/// Peak |κ| across all curved segments.
pub fn max_curvature(paths: &[Path]) -> f64 {
    max_curvature_cubics(&all_cubics(paths))
}

fn all_cubics(paths: &[Path]) -> Vec<Vec<Cubic>> {
    paths.iter().map(contour_cubics).collect()
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

