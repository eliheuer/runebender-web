//! One-off: replay a boolean path operation on a .glif exactly the way
//! the editor runs it, outside the browser, so a refused or garbage
//! result can be dissected.
//!
//!     cargo run --example bool_replay -- <path-to.glif> [union|difference|intersection]
use runebender_web::editor::{EditorState, norad_glyph_to_bezpath};

fn main() {
    let mut args = std::env::args().skip(1);
    let glif_path = args.next().expect("usage: bool_replay <glif> [op]");
    let op = match args.next().as_deref() {
        None | Some("union") => linesweeper::BinaryOp::Union,
        Some("difference") => linesweeper::BinaryOp::Difference,
        Some("intersection") => linesweeper::BinaryOp::Intersection,
        Some(other) => panic!("unknown op {other:?}"),
    };

    let glyph = norad::Glyph::load(&glif_path).expect("glif loads");
    let bez = norad_glyph_to_bezpath(&glyph);

    let mut state = EditorState::default();
    state.set_glyph_from_bezpath(&bez);
    state.advance_width = glyph.width;

    println!("input: {} contours", state.paths.len());
    for (index, path) in state.paths.iter().enumerate() {
        let points = path.points();
        let (min, max) = points.iter().fold(
            ((f64::MAX, f64::MAX), (f64::MIN, f64::MIN)),
            |((min_x, min_y), (max_x, max_y)), point| {
                (
                    (min_x.min(point.point.x), min_y.min(point.point.y)),
                    (max_x.max(point.point.x), max_y.max(point.point.y)),
                )
            },
        );
        println!(
            "  contour {index}: {} points, bbox ({:.1},{:.1})..({:.1},{:.1})",
            points.len(),
            min.0,
            min.1,
            max.0,
            max.1
        );
    }

    let changed = state.boolean_selection(op);
    println!("boolean_selection({op:?}) -> changed = {changed}");
    if changed {
        println!("output: {} contours", state.paths.len());
        let mut worst: f64 = 0.0;
        for path in &state.paths {
            for point in path.points().iter() {
                worst = worst.max(point.point.x.abs()).max(point.point.y.abs());
            }
        }
        println!("largest |coordinate| in output: {worst}");
        if let Some(bbox) = state.glyph_bbox() {
            println!("output bbox: {bbox:?}");
        }
    }
}
