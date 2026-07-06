// Copyright 2026 the Runebender Authors
// SPDX-License-Identifier: GPL-3.0-or-later

//! Local browser/WASM image tracing adapter.
//!
//! Keep `img2bez` details isolated here so updating the sibling tracer crate
//! does not require changes throughout the Vue host.
//!
//! This wraps the modern img2bez API (`trace_glyph` + `TraceOptions` +
//! `FontMetrics`), the same path img2bez's own wasm bindings use, so a trace
//! here is byte-identical to the CLI and the blog demo.

use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct TraceImageConfig {
    glyph: String,
    unicode: Option<String>,
    width: Option<f64>,
    target_height: Option<f64>,
    x_offset: Option<f64>,
    y_offset: Option<f64>,
    grid: Option<i32>,
    accuracy: Option<f64>,
    invert: Option<bool>,
    threshold: Option<u8>,
    /// Source-class preset, mirroring the CLI `--profile`: `wild`/`auto`
    /// (default, auto-detect), `clean` (crisp render), or `photo` (soft scan of
    /// printed type — forces the de-texture pre-blur). Unknown falls back to
    /// wild.
    profile: Option<String>,
    /// Image pre-blur sigma (px). Omitted = let the profile/auto-detect decide.
    pre_blur: Option<f64>,
    /// Pre-fit smoothing multiplier (1.0 = default).
    smoothing: Option<f64>,
    /// Corner turn-angle threshold in degrees (default 12).
    corner_threshold: Option<f64>,
    /// Output-shape mode: `default`, `smooth` (all-curves), or `line`.
    mode: Option<String>,
    /// Drawing style: `basic` (default), `grotesk`, `old-style`, `geometric`,
    /// `brush`, `nib`, `qalam`. Layers design-specific tuning on the base.
    style: Option<String>,
}

pub fn trace_image_to_glif(image_bytes: &[u8], config_json: &str) -> Result<String, String> {
    trace_image(image_bytes, config_json).map(|trace| trace.glif)
}

pub fn trace_image_to_glif_report(image_bytes: &[u8], config_json: &str) -> Result<String, String> {
    let trace = trace_image(image_bytes, config_json)?;
    serde_json::to_string(&trace).map_err(|e| format!("serialize trace report: {e}"))
}

fn trace_image(image_bytes: &[u8], config_json: &str) -> Result<TraceImageReport, String> {
    if image_bytes.is_empty() {
        return Err("image bytes are empty".to_string());
    }

    let config: TraceImageConfig =
        serde_json::from_str(config_json).map_err(|e| format!("parse trace config: {e}"))?;
    if config.glyph.trim().is_empty() {
        return Err("glyph name is required".to_string());
    }

    // Build (TraceOptions, FontMetrics) from the library defaults — the same
    // source of truth the CLI and img2bez's wasm bindings use — overriding only
    // the fields the host actually provided. Profile `wild` (the img2bez
    // default) supplies the looser fit accuracy that keeps unknown rasters from
    // over-segmenting; the host can still override via `accuracy`.
    let profile = config
        .profile
        .as_deref()
        .map(img2bez::Profile::from_name_lossy)
        .unwrap_or(img2bez::Profile::Wild);
    let mut opts = img2bez::TraceOptions::for_profile(profile);
    opts.verbose = false;
    if let Some(accuracy) = config.accuracy {
        opts.fit_accuracy = accuracy.max(0.1);
    }
    if let Some(h) = config.target_height {
        opts.em_height = h.max(1.0);
    }
    if let Some(g) = config.grid {
        opts.grid = g.max(0);
    }
    if let Some(inv) = config.invert {
        opts.invert = inv;
    }
    if let Some(t) = config.threshold {
        opts.threshold = img2bez::ThresholdMethod::Fixed(t);
    }
    // Input-adaptive levers + output mode (mirror the CLI). An explicit pre-blur
    // wins over the profile/auto-detection; omitted leaves the profile to decide.
    if let Some(b) = config.pre_blur {
        opts.pre_blur = b.max(0.0);
    }
    if let Some(s) = config.smoothing {
        opts.smoothing = s.max(0.0);
    }
    if let Some(c) = config.corner_threshold {
        opts.corner_threshold_deg = c;
    }
    if let Some(m) = config.mode.as_deref() {
        opts.mode = match m {
            "smooth" => img2bez::TraceMode::Smooth,
            "line" => img2bez::TraceMode::LineOnly,
            _ => img2bez::TraceMode::Default,
        };
    }
    // Layer the drawing-style tuning on top (mirrors the CLI --style).
    img2bez::Style::from_name_lossy(config.style.as_deref().unwrap_or("basic")).apply(&mut opts);

    // targetHeight is ascender − descender; yOffset is the descender. Place the
    // traced outline into that band via the font metrics.
    let target_height = config.target_height.map(|h| h.max(1.0)).unwrap_or(1088.0);
    let y_offset = config.y_offset.unwrap_or(-256.0);
    let mut metrics = img2bez::FontMetrics::from_target_height(target_height, y_offset);
    metrics.advance_width = Some(config.width.unwrap_or(600.0).max(1.0));
    // The host snaps the left ink edge to `xOffset` and re-aligns the
    // background image to the same x afterwards (alignBackgroundImageToTrace),
    // so the trace must land its leftmost ink exactly there — otherwise the
    // outline is offset from the image it was traced from.
    metrics.lsb = config.x_offset.unwrap_or(64.0);

    let codepoints = parse_codepoints(config.unicode.as_deref())?;

    // Effective profile (forced via `profile`, or auto-detected from the image),
    // so the UI can show what "Auto" resolved to.
    let effective_profile = match img2bez::measure_image(image_bytes) {
        Ok(stats) => {
            let detected = stats.classify();
            if opts.profile.applies_pre_blur() {
                opts.profile
            } else if opts.auto_pre_blur && detected.applies_pre_blur() {
                detected
            } else {
                opts.profile
            }
        }
        Err(_) => opts.profile,
    };

    let glyph = img2bez::trace_glyph(image_bytes, config.glyph.as_str(), &codepoints, &opts, &metrics)
        .map_err(|e| format!("img2bez trace failed: {e}"))?;

    let glif = glyph.to_glif();
    let paths = glyph.outline.to_bezpaths();
    let (curves, lines, on_curves, off_curves) = count_path_geometry(&paths);

    Ok(TraceImageReport {
        glif,
        contours: paths.len(),
        curves,
        lines,
        on_curves,
        off_curves,
        advance_width: glyph.advance.width,
        profile: profile_name(effective_profile).to_string(),
        // The placement model frames the glyph straight into the em via the
        // font metrics, so there is no separate reposition shift to report.
        reposition_shift_x: 0.0,
        reposition_shift_y: 0.0,
    })
}

/// Lowercase name of a profile for the trace report.
fn profile_name(p: img2bez::Profile) -> &'static str {
    match p {
        img2bez::Profile::Wild => "wild",
        img2bez::Profile::Clean => "clean",
        img2bez::Profile::Photo => "photo",
        // Profile is #[non_exhaustive] as of the 0.1.0 API freeze.
        _ => "unknown",
    }
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct TraceImageReport {
    glif: String,
    contours: usize,
    curves: usize,
    lines: usize,
    on_curves: usize,
    off_curves: usize,
    advance_width: f64,
    /// Effective trace profile (`wild`/`clean`/`photo`) — forced or auto-detected.
    profile: String,
    reposition_shift_x: f64,
    reposition_shift_y: f64,
}

fn count_path_geometry(paths: &[img2bez::kurbo::BezPath]) -> (usize, usize, usize, usize) {
    let mut curves = 0usize;
    let mut lines = 0usize;
    let mut on_curves = 0usize;
    let mut off_curves = 0usize;
    for path in paths {
        for element in path.elements() {
            match element {
                img2bez::kurbo::PathEl::MoveTo(_) => {
                    on_curves += 1;
                }
                img2bez::kurbo::PathEl::LineTo(_) => {
                    lines += 1;
                    on_curves += 1;
                }
                img2bez::kurbo::PathEl::CurveTo(..) => {
                    curves += 1;
                    on_curves += 1;
                    off_curves += 2;
                }
                img2bez::kurbo::PathEl::QuadTo(..) => {
                    on_curves += 1;
                    off_curves += 1;
                }
                img2bez::kurbo::PathEl::ClosePath => {}
            }
        }
    }
    (curves, lines, on_curves, off_curves)
}

fn parse_codepoints(value: Option<&str>) -> Result<Vec<char>, String> {
    let Some(value) = value else {
        return Ok(Vec::new());
    };
    let value = value.trim();
    if value.is_empty() {
        return Ok(Vec::new());
    }

    let mut codepoints = Vec::new();
    for raw in value.split(|c: char| c == ',' || c.is_whitespace()) {
        let raw = raw.trim();
        if raw.is_empty() {
            continue;
        }
        let hex = raw
            .strip_prefix("U+")
            .or_else(|| raw.strip_prefix("u+"))
            .unwrap_or(raw);
        let codepoint =
            u32::from_str_radix(hex, 16).map_err(|_| format!("invalid unicode hex: {raw}"))?;
        let ch = char::from_u32(codepoint)
            .ok_or_else(|| format!("invalid unicode scalar value: {raw}"))?;
        codepoints.push(ch);
    }
    Ok(codepoints)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_common_unicode_forms() {
        assert_eq!(parse_codepoints(None).expect("none parses"), Vec::<char>::new());
        assert_eq!(
            parse_codepoints(Some("0023, U+0026")).expect("hex parses"),
            vec!['#', '&']
        );
    }

    #[test]
    fn rejects_invalid_unicode() {
        let err = parse_codepoints(Some("not-hex")).expect_err("invalid hex rejected");
        assert!(err.contains("invalid unicode hex"));
    }

    #[test]
    fn trace_report_rejects_empty_image_bytes() {
        let err = trace_image_to_glif_report(&[], r#"{"glyph":"a"}"#)
            .expect_err("empty image bytes rejected");
        assert!(err.contains("image bytes are empty"));
    }
}
