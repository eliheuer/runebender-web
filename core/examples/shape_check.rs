//! One-off: shape a string through a UFO's own features.
//!
//!     cargo run --example shape_check -- <path-to.ufo> "لا"
use runebender_web::shape::{ShapingFont, ShapingGlyph, ShapingSource};

fn main() {
    let mut args = std::env::args().skip(1);
    let ufo_dir = args.next().expect("usage: shape_check <ufo> <text>");
    let text = args
        .next()
        .unwrap_or_else(|| "\u{0644}\u{0627}".to_string());

    let font = norad::Font::load(&ufo_dir).expect("UFO loads");
    let features =
        std::fs::read_to_string(format!("{ufo_dir}/features.fea")).expect("features.fea");
    let layer = font.layers.default_layer();
    let glyphs: Vec<ShapingGlyph> = std::iter::once(".notdef".to_string())
        .chain(
            layer
                .iter()
                .map(|g| g.name().to_string())
                .filter(|n| n != ".notdef"),
        )
        .map(|name| {
            let glyph = layer.get_glyph(name.as_str());
            ShapingGlyph {
                name: name.clone(),
                advance: glyph.map(|g| g.width).unwrap_or(0.0),
                unicodes: glyph
                    .map(|g| g.codepoints.iter().map(|c| c as u32).collect())
                    .unwrap_or_default(),
            }
        })
        .collect();

    let source = ShapingSource {
        units_per_em: font.font_info.units_per_em.map(|u| *u).unwrap_or(1000.0),
        glyphs,
        features,
    };
    match ShapingFont::build(&source) {
        Ok(font) => {
            let shaped = font.shape(&text, true).expect("shapes");
            let names: Vec<&str> = shaped
                .iter()
                .map(|g| font.glyph_name(g.glyph_id).unwrap_or("?"))
                .collect();
            println!("{text:?} -> {names:?}");
        }
        Err(e) => println!("compile failed: {e}"),
    }
}
