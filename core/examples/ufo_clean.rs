// Copyright 2026 the Runebender Authors
// SPDX-License-Identifier: GPL-3.0-or-later

//! Rewrite a UFO through norad, dropping every `com.schriftgestaltung.*`
//! lib key (font-level and per-glyph).
//!
//! Glyphs' own keys shadow the designspace when Glyphs re-opens a UFO,
//! and they carry stale interpolation values. Stripping them leaves a
//! plain UFO that any tool reads the same way.
//!
//!     cargo run --example ufo_clean -- <in.ufo> <out.ufo>

fn main() {
    let mut args = std::env::args().skip(1);
    let (Some(input), Some(output)) = (args.next(), args.next()) else {
        eprintln!("usage: ufo_clean <in.ufo> <out.ufo>");
        std::process::exit(2);
    };

    let mut font = match norad::Font::load(&input) {
        Ok(font) => font,
        Err(e) => {
            eprintln!("load {input}: {e}");
            std::process::exit(1);
        }
    };

    let mut font_keys = 0;
    let mut glyph_keys = 0;
    let mut touched_glyphs = 0;

    font_keys += strip(&mut font.lib);
    for layer in font.layers.iter_mut() {
        for glyph in layer.iter_mut() {
            let dropped = strip(&mut glyph.lib);
            if dropped > 0 {
                glyph_keys += dropped;
                touched_glyphs += 1;
            }
        }
    }

    if let Err(e) = font.save(&output) {
        eprintln!("save {output}: {e}");
        std::process::exit(1);
    }
    println!(
        "{input} -> {output}: dropped {font_keys} font lib keys, {glyph_keys} keys across {touched_glyphs} glyphs"
    );
}

fn strip(lib: &mut plist::Dictionary) -> usize {
    let doomed: Vec<String> = lib
        .keys()
        .filter(|key| key.starts_with("com.schriftgestaltung"))
        .map(ToString::to_string)
        .collect();
    for key in &doomed {
        lib.remove(key);
    }
    doomed.len()
}
