// WebGPU renderer for the Runebender canvas, built on Vello.
//
// Gated on wasm32 because Vello's `util::RenderContext::create_surface`
// expects a `wgpu::SurfaceTarget`, and the only `SurfaceTarget` we
// ever hand it is an `HtmlCanvasElement` — that's a browser-only
// path. The path/model/editing modules build on both native and
// wasm32 so unit tests still run on `cargo test`. (Gating lives in lib.rs.)

use kurbo::{Affine, BezPath, Circle, Ellipse, Line, PathEl, Point, Rect, Stroke};
use runebender_core::theme;
use serde::Deserialize;
use std::collections::{HashMap, HashSet, hash_map::DefaultHasher};
use std::hash::{Hash, Hasher};
use std::rc::Rc;
use vello::peniko::{Fill, Mix, color::AlphaColor};
use vello::wgpu;
use vello::wgpu::util::TextureBlitter;
use vello::{AaConfig, Renderer as VelloRenderer, RendererOptions, Scene};
use wasm_bindgen::JsValue;
use web_sys::HtmlCanvasElement;

use crate::editor::{
    DESIGN_GRID_CLOSE_FINE, DESIGN_GRID_CLOSE_MIN_ZOOM, DESIGN_GRID_MID_FINE,
    DESIGN_GRID_MID_MIN_ZOOM, EditorState, KnifePreview, MeasurePreview, PenPreview,
    SegmentHoverPreview, ShapePreview, SidebearingEdge,
};
use crate::measure::{self, MeasureKind};
use crate::model::EntityId;
use crate::path::{Path, PathPoint, PointType};
use crate::text::{TextLayout, TextLayoutItem};

// ============================================================================
// PALETTE
// ============================================================================

type Srgb = AlphaColor<vello::peniko::color::Srgb>;

/// Curvature-comb color ramp: a vibrant cool→warm gradient (teal → indigo →
/// magenta → orange → amber) for low→high curvature. Opaque, tuned to look
/// good on the dark canvas and in screenshots.
fn curve_gradient(t: f64) -> Srgb {
    const STOPS: [[f32; 3]; 5] = [
        [0.16, 0.80, 0.82], // teal
        [0.40, 0.44, 0.95], // indigo
        [0.86, 0.28, 0.72], // magenta
        [1.00, 0.55, 0.24], // orange
        [1.00, 0.84, 0.36], // amber
    ];
    let u = (t.clamp(0.0, 1.0) as f32) * (STOPS.len() as f32 - 1.0);
    let i = (u.floor() as usize).min(STOPS.len() - 2);
    let f = u - i as f32;
    let (a, b) = (STOPS[i], STOPS[i + 1]);
    AlphaColor::new([
        a[0] + (b[0] - a[0]) * f,
        a[1] + (b[1] - a[1]) * f,
        a[2] + (b[2] - a[2]) * f,
        1.0,
    ])
}

const fn srgb(color: theme::ColorRgba) -> Srgb {
    AlphaColor::from_rgba8(color.r, color.g, color.b, color.a)
}

const BG: Srgb = srgb(theme::app::BACKGROUND);
const PATH_STROKE: Srgb = srgb(theme::path::STROKE);
const PREVIEW_FILL: Srgb = srgb(theme::path::PREVIEW_FILL);
const COMPONENT_FILL: Srgb = srgb(theme::component::FILL);
const COMPONENT_SELECTED_FILL: Srgb = srgb(theme::component::SELECTED_FILL);
const HANDLE_LINE: Srgb = srgb(theme::handle::LINE);
const POINT_INNER: Srgb = AlphaColor::from_rgba8(0x18, 0x18, 0x18, 0xff);
const POINT_MARK_RED: Srgb = AlphaColor::from_rgba8(0xff, 0x4a, 0x3d, 0xff);
const POINT_MARK_GREEN: Srgb = AlphaColor::from_rgba8(0x18, 0xb8, 0x6f, 0xff);
/// Anchors, in the palette's pink: green put them in the same family as
/// smooth on-curve points, which is exactly what you do not want when
/// hunting for an anchor in a glyph full of nodes.
const ANCHOR_MARK_PINK: Srgb = AlphaColor::from_rgba8(0xe8, 0x6a, 0xb8, 0xff);
const POINT_MARK_PURPLE: Srgb = AlphaColor::from_rgba8(0x8c, 0x6c, 0xff, 0xff);
const POINT_MARK_YELLOW: Srgb = AlphaColor::from_rgba8(0xff, 0xdc, 0x32, 0xff);
const POINT_MARK_ORANGE: Srgb = AlphaColor::from_rgba8(0xff, 0x98, 0x0f, 0xff);
/// Points of an interpolated instance: uniform grey, no on/off-curve
/// colour coding, because nothing here can be dragged.
/// Anchors are drawn as diamonds; the radius is bumped so a rotated
/// square reads as the same visual size as a point circle.
const ANCHOR_DIAMOND_SCALE: f64 = 1.35;
const POINT_READONLY_INNER: Srgb = POINT_INNER;
const POINT_READONLY_OUTER: Srgb = AlphaColor::from_rgba8(0x8a, 0x8a, 0x8a, 0xff);
const POINT_SMOOTH_INNER: Srgb = POINT_INNER;
const POINT_SMOOTH_OUTER: Srgb = POINT_MARK_GREEN;
const POINT_CORNER_INNER: Srgb = POINT_INNER;
const POINT_CORNER_OUTER: Srgb = POINT_MARK_ORANGE;
const POINT_OFFCURVE_INNER: Srgb = POINT_INNER;
const POINT_OFFCURVE_OUTER: Srgb = POINT_MARK_PURPLE;
const POINT_HYPER_INNER: Srgb = POINT_INNER;
const POINT_HYPER_OUTER: Srgb = POINT_MARK_PURPLE;
const POINT_SELECTED_INNER: Srgb = POINT_MARK_YELLOW;
const POINT_SELECTED_OUTER: Srgb = POINT_MARK_ORANGE;
const START_NODE_OUTER: Srgb = POINT_MARK_ORANGE;
const MARQUEE_FILL: Srgb = srgb(theme::selection::RECT_FILL);
const MARQUEE_STROKE: Srgb = srgb(theme::selection::RECT_STROKE);
const TOOL_PREVIEW: Srgb = srgb(theme::segment::HOVER);
const METRIC_GUIDE: Srgb = srgb(theme::metrics::GUIDE);
const DESIGN_GRID_FINE: Srgb = srgb(theme::design_grid::FINE);
const DESIGN_GRID_COARSE: Srgb = srgb(theme::design_grid::COARSE);
const TEXT_PREVIEW_FILL: Srgb = srgb(theme::grid::GLYPH);
/// Ghost fill under the glyph being edited: the same grey the inactive
/// sorts use, at a tenth strength, so counters read as counters without
/// competing with the outline.
const ACTIVE_GLYPH_FILL_ALPHA: f32 = 0.16;
/// Inactive sorts stay solid while zoomed out, and thin to this once the
/// design grid appears — at that zoom you are drawing, not reading.
const INACTIVE_GLYPH_FILL_ALPHA: f32 = 0.34;
/// The glyph's background layer: a quiet outline behind the drawing,
/// the way Glyphs shows a background.
const BACKGROUND_LAYER_STROKE: Srgb = srgb(theme::base::F);
/// Another glyph shown behind for comparison: a ghost fill, so it never
/// reads as the background layer's outline.
const REFERENCE_GLYPH_FILL: Srgb = srgb(theme::base::C);
const TEXT_CURSOR: Srgb = srgb(theme::selection::RECT_STROKE);
const TEXT_KERN_ACTIVE: Srgb = srgb(theme::kerning::ACTIVE_GLYPH);
const TEXT_KERN_PREVIOUS: Srgb = srgb(theme::kerning::PREVIOUS_GLYPH);

#[derive(Clone)]
struct CanvasTheme {
    bg: Srgb,
    path_stroke: Srgb,
    preview_fill: Srgb,
    component_fill: Srgb,
    component_selected_fill: Srgb,
    handle_line: Srgb,
    point_smooth_inner: Srgb,
    point_smooth_outer: Srgb,
    point_corner_inner: Srgb,
    point_corner_outer: Srgb,
    point_offcurve_inner: Srgb,
    point_offcurve_outer: Srgb,
    point_hyper_inner: Srgb,
    point_hyper_outer: Srgb,
    point_selected_inner: Srgb,
    point_selected_outer: Srgb,
    start_node_outer: Srgb,
    marquee_fill: Srgb,
    marquee_stroke: Srgb,
    tool_preview: Srgb,
    metric_guide: Srgb,
    design_grid_fine: Srgb,
    design_grid_coarse: Srgb,
    text_preview_fill: Srgb,
    text_cursor: Srgb,
    text_kern_active: Srgb,
    text_kern_previous: Srgb,
    /// Casing drawn under points, rings and HUD text so they stay
    /// legible over whatever is behind them. Dark on a dark canvas,
    /// light on a light one — which is why it cannot be a constant.
    halo: Srgb,
    /// Metric box of a sort nobody is editing.
    metric_quiet: Srgb,
    /// The glyph's background layer, and another glyph shown behind it.
    background_layer: Srgb,
    reference_glyph: Srgb,
    /// Continuity rings: G2/G3, G1, line-to-curve, kink.
    continuity_g2: Srgb,
    continuity_g1: Srgb,
    continuity_line: Srgb,
    continuity_kink: Srgb,
    /// Popcount tiers, one power through four or more.
    popcount_1: Srgb,
    popcount_2: Srgb,
    popcount_3: Srgb,
    popcount_4: Srgb,
    /// Points shown read-only: components, interpolated instances.
    point_readonly_outer: Srgb,
}

impl Default for CanvasTheme {
    fn default() -> Self {
        Self {
            bg: BG,
            path_stroke: PATH_STROKE,
            preview_fill: PREVIEW_FILL,
            component_fill: COMPONENT_FILL,
            component_selected_fill: COMPONENT_SELECTED_FILL,
            handle_line: HANDLE_LINE,
            point_smooth_inner: POINT_SMOOTH_INNER,
            point_smooth_outer: POINT_SMOOTH_OUTER,
            point_corner_inner: POINT_CORNER_INNER,
            point_corner_outer: POINT_CORNER_OUTER,
            point_offcurve_inner: POINT_OFFCURVE_INNER,
            point_offcurve_outer: POINT_OFFCURVE_OUTER,
            point_hyper_inner: POINT_HYPER_INNER,
            point_hyper_outer: POINT_HYPER_OUTER,
            point_selected_inner: POINT_SELECTED_INNER,
            point_selected_outer: POINT_SELECTED_OUTER,
            start_node_outer: START_NODE_OUTER,
            marquee_fill: MARQUEE_FILL,
            marquee_stroke: MARQUEE_STROKE,
            tool_preview: TOOL_PREVIEW,
            metric_guide: METRIC_GUIDE,
            design_grid_fine: DESIGN_GRID_FINE,
            design_grid_coarse: DESIGN_GRID_COARSE,
            text_preview_fill: TEXT_PREVIEW_FILL,
            text_cursor: TEXT_CURSOR,
            text_kern_active: TEXT_KERN_ACTIVE,
            text_kern_previous: TEXT_KERN_PREVIOUS,
            halo: HALO_COLOR,
            metric_quiet: TEXT_SORT_METRIC_QUIET,
            background_layer: BACKGROUND_LAYER_STROKE,
            reference_glyph: REFERENCE_GLYPH_FILL,
            continuity_g2: CONTINUITY_G2,
            continuity_g1: CONTINUITY_G1,
            continuity_line: CONTINUITY_LINE,
            continuity_kink: CONTINUITY_KINK,
            popcount_1: POPCOUNT_1,
            popcount_2: POPCOUNT_2,
            popcount_3: POPCOUNT_3,
            popcount_4: POPCOUNT_4,
            point_readonly_outer: POINT_READONLY_OUTER,
        }
    }
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct CanvasThemeInput {
    bg: Option<[u8; 4]>,
    path_stroke: Option<[u8; 4]>,
    preview_fill: Option<[u8; 4]>,
    component_fill: Option<[u8; 4]>,
    component_selected_fill: Option<[u8; 4]>,
    handle_line: Option<[u8; 4]>,
    point_smooth_inner: Option<[u8; 4]>,
    point_smooth_outer: Option<[u8; 4]>,
    point_corner_inner: Option<[u8; 4]>,
    point_corner_outer: Option<[u8; 4]>,
    point_offcurve_inner: Option<[u8; 4]>,
    point_offcurve_outer: Option<[u8; 4]>,
    point_hyper_inner: Option<[u8; 4]>,
    point_hyper_outer: Option<[u8; 4]>,
    point_selected_inner: Option<[u8; 4]>,
    point_selected_outer: Option<[u8; 4]>,
    start_node_outer: Option<[u8; 4]>,
    marquee_fill: Option<[u8; 4]>,
    marquee_stroke: Option<[u8; 4]>,
    tool_preview: Option<[u8; 4]>,
    metric_guide: Option<[u8; 4]>,
    design_grid_fine: Option<[u8; 4]>,
    design_grid_coarse: Option<[u8; 4]>,
    text_preview_fill: Option<[u8; 4]>,
    text_cursor: Option<[u8; 4]>,
    text_kern_active: Option<[u8; 4]>,
    text_kern_previous: Option<[u8; 4]>,
    halo: Option<[u8; 4]>,
    metric_quiet: Option<[u8; 4]>,
    background_layer: Option<[u8; 4]>,
    reference_glyph: Option<[u8; 4]>,
    continuity_g2: Option<[u8; 4]>,
    continuity_g1: Option<[u8; 4]>,
    continuity_line: Option<[u8; 4]>,
    continuity_kink: Option<[u8; 4]>,
    popcount_1: Option<[u8; 4]>,
    popcount_2: Option<[u8; 4]>,
    popcount_3: Option<[u8; 4]>,
    popcount_4: Option<[u8; 4]>,
    point_readonly_outer: Option<[u8; 4]>,
}

impl CanvasTheme {
    fn apply_input(&mut self, input: CanvasThemeInput) {
        macro_rules! apply_color {
            ($field:ident) => {
                if let Some([r, g, b, a]) = input.$field {
                    self.$field = AlphaColor::from_rgba8(r, g, b, a);
                }
            };
        }
        apply_color!(bg);
        apply_color!(path_stroke);
        apply_color!(preview_fill);
        apply_color!(component_fill);
        apply_color!(component_selected_fill);
        apply_color!(handle_line);
        apply_color!(point_smooth_inner);
        apply_color!(point_smooth_outer);
        apply_color!(point_corner_inner);
        apply_color!(point_corner_outer);
        apply_color!(point_offcurve_inner);
        apply_color!(point_offcurve_outer);
        apply_color!(point_hyper_inner);
        apply_color!(point_hyper_outer);
        apply_color!(point_selected_inner);
        apply_color!(point_selected_outer);
        apply_color!(start_node_outer);
        apply_color!(marquee_fill);
        apply_color!(marquee_stroke);
        apply_color!(tool_preview);
        apply_color!(metric_guide);
        apply_color!(design_grid_fine);
        apply_color!(design_grid_coarse);
        apply_color!(text_preview_fill);
        apply_color!(text_cursor);
        apply_color!(text_kern_active);
        apply_color!(text_kern_previous);
        apply_color!(halo);
        apply_color!(metric_quiet);
        apply_color!(background_layer);
        apply_color!(reference_glyph);
        apply_color!(continuity_g2);
        apply_color!(continuity_g1);
        apply_color!(continuity_line);
        apply_color!(continuity_kink);
        apply_color!(popcount_1);
        apply_color!(popcount_2);
        apply_color!(popcount_3);
        apply_color!(popcount_4);
        apply_color!(point_readonly_outer);
    }
}

// --- Sizes (xilem size::*; STROKE_SCALE = 1.5) ---
const STROKE_SCALE: f64 = 1.5;
const SMOOTH_POINT_RADIUS_PX: f64 = 4.5;
const SMOOTH_POINT_SELECTED_RADIUS_PX: f64 = 5.5;
const CORNER_POINT_HALF_PX: f64 = 3.5;
const CORNER_POINT_SELECTED_HALF_PX: f64 = 4.5;
const OFFCURVE_POINT_RADIUS_PX: f64 = SMOOTH_POINT_RADIUS_PX;
const OFFCURVE_POINT_SELECTED_RADIUS_PX: f64 = SMOOTH_POINT_SELECTED_RADIUS_PX;
const HYPER_POINT_RADIUS_PX: f64 = 4.0;
const HYPER_POINT_SELECTED_RADIUS_PX: f64 = 5.0;
const START_NODE_HALF_PX: f64 = 5.5;
const START_NODE_SELECTED_HALF_PX: f64 = 6.5;
const START_NODE_OFFSET_PX: f64 = 8.0;
/// One weight for every editor line: contour, handles, point rings and
/// the comb's fins.
const LINE_PX: f64 = 1.0 * STROKE_SCALE;
const POINT_OUTLINE_PX: f64 = LINE_PX;
/// Dark casing drawn under the outline, handle lines and points, so
/// they stay readable on top of the curvature comb. Two pixels a side,
/// which matches the weight of the comb's own black rib lines.
const HALO_PX: f64 = 4.0;
const HALO_COLOR: Srgb = AlphaColor::from_rgba8(0x0c, 0x0c, 0x0c, 0xd8);
const PATH_STROKE_PX: f64 = LINE_PX;
const COMB_FIN_PX: f64 = LINE_PX;
const COMPONENT_SELECTION_STROKE_PX: f64 = 2.0;
const HANDLE_LINE_PX: f64 = LINE_PX;
const MARQUEE_STROKE_PX: f64 = 1.0 * STROKE_SCALE;
const METRIC_LINE_PX: f64 = 1.0 * STROKE_SCALE;
const TOOL_PREVIEW_LINE_PX: f64 = 1.0 * STROKE_SCALE;
const SEGMENT_HOVER_LINE_PX: f64 = 3.0;
const TOOL_PREVIEW_DOT_RADIUS_PX: f64 = 3.0;
// Dash pattern shared by every in-progress tool preview (pen rubber-band,
// knife line) so lines that have not "landed" yet read consistently as
// provisional. Mirrors runebender-xilem `theme::tool_preview::LINE_DASH`.
const TOOL_PREVIEW_DASH: [f64; 2] = [4.0, 4.0];
const TEXT_CURSOR_LINE_PX: f64 = 1.5;
const TEXT_CURSOR_LINE_MAX_PX: f64 = 4.0;
const TEXT_CURSOR_TRIANGLE_WIDTH_PX: f64 = 24.0;
const TEXT_CURSOR_TRIANGLE_HEIGHT_PX: f64 = 16.0;
/// Caret triangle width as a share of the sort's on-screen height, and
/// the range it is allowed to take.
const TEXT_CURSOR_MARKER_FRACTION: f64 = 0.09;
const TEXT_CURSOR_MARKER_MIN_PX: f64 = 4.0;
const TEXT_CURSOR_MARKER_MAX_PX: f64 = 34.0;
const TEXT_SORT_MARK_SIZE: f64 = 24.0;
const TEXT_SORT_MARK_MIN_SIZE: f64 = 1.5;
/// A mark is this fraction of the sort's on-screen height, so it shrinks
/// with the text instead of staying a fixed size while the glyphs get
/// smaller — which turned a page of text into a mesh of green marks.
const TEXT_SORT_MARK_FRACTION: f64 = 0.05;
/// Below this on-screen size the corner marks and their metric boxes
/// switch off outright. A fade just turns them into grit — either they
/// are worth drawing at full strength or they are in the way.
const TEXT_SORT_MARK_HIDE_BELOW_PX: f64 = 3.0;
/// Metric lines for the sorts that are not being edited: just lighter
/// than the canvas, so the boxes read as structure behind the marks
/// without competing with the glyphs. They switch off with the marks —
/// at a zoom where the marks are gone the boxes are noise too.
const TEXT_SORT_METRIC_QUIET: Srgb = AlphaColor::from_rgba8(0x24, 0x24, 0x24, 0xff);
/// Continuity rings and popcount tiers. Defaults only: the host sends
/// the theme's own, so they follow a theme switch like everything else.
const CONTINUITY_G2: Srgb = AlphaColor::new([0.13, 0.83, 0.65, 1.0]);
const CONTINUITY_G1: Srgb = AlphaColor::new([1.0, 0.82, 0.25, 1.0]);
const CONTINUITY_LINE: Srgb = AlphaColor::new([0.50, 0.55, 0.64, 0.9]);
const CONTINUITY_KINK: Srgb = AlphaColor::new([1.0, 0.27, 0.23, 1.0]);
const POPCOUNT_1: Srgb = AlphaColor::new([0.09, 0.72, 0.44, 1.0]);
const POPCOUNT_2: Srgb = AlphaColor::new([1.0, 0.86, 0.2, 1.0]);
const POPCOUNT_3: Srgb = AlphaColor::new([1.0, 0.6, 0.06, 1.0]);
const POPCOUNT_4: Srgb = AlphaColor::new([1.0, 0.29, 0.24, 1.0]);
// Mid zoom shows the machine lattice plainly: a line every 8 units,
// uniform — no darker every-32 accent (Eli). The close level keeps its
// 2-unit fine grid with the 8-unit accent.
const DESIGN_GRID_MID_COARSE_N: u32 = 0;
const DESIGN_GRID_CLOSE_COARSE_N: u32 = 4;
const DESIGN_GRID_FINE_LINE_PX: f64 = 0.5;
const DESIGN_GRID_COARSE_LINE_PX: f64 = 1.0;

// ============================================================================
// RENDERER
// ============================================================================

/// Which layers of the grid-measurement HUD are on. All-false is the plain
/// editor: nothing extra drawn. Driven from the select-mode side panel.
#[derive(Clone, Copy)]
pub struct MeasureOptions {
    /// Tint the outline segments, curves, and handle lines by popcount.
    pub colorize: bool,
    /// Label Bézier handle lengths.
    pub handles: bool,
    /// Label straight outline segment lengths.
    pub segments: bool,
    /// Draw + label stem/counter/height spans (dimension lines).
    pub spans: bool,
    /// Draw + label left/right side bearings and mark the extreme columns.
    pub sidebearings: bool,
    /// Spell lengths out as sums of powers of two — `96 = 64+32` — rather
    /// than as the bare number.
    pub popcount: bool,
}

impl Default for MeasureOptions {
    fn default() -> Self {
        // Every layer off, but a label that does appear reads as a sum:
        // that is the point of the measurement HUD.
        Self {
            colorize: false,
            handles: false,
            segments: false,
            spans: false,
            sidebearings: false,
            popcount: true,
        }
    }
}

impl MeasureOptions {
    fn any(&self) -> bool {
        self.colorize || self.handles || self.segments || self.spans || self.sidebearings
    }

    /// How a length is written on the canvas.
    fn label(&self, value: i64) -> String {
        if self.popcount {
            measure::label(value)
        } else {
            value.to_string()
        }
    }
}

/// Which curve-smoothness layers are on. All-false = plain editor.
#[derive(Clone, Copy, Default)]
pub struct CurveOptions {
    /// Speedpunk-style curvature comb.
    pub comb: bool,
    /// Continuity markers per smooth node (G0/G1/G2/G3 dots).
    pub continuity: bool,
}

pub struct Renderer {
    // Hand-rolled wgpu setup (instead of vello::util::RenderContext) so
    // we can request the adapter's full max_texture_dimension_2d. Vello
    // 0.8's RenderContext hardcodes Limits::default(), which caps
    // textures at 8192 — too small for full-DPR rendering on Retina/5K
    // displays.
    device: wgpu::Device,
    queue: wgpu::Queue,
    surface: wgpu::Surface<'static>,
    surface_config: wgpu::SurfaceConfiguration,
    target_texture: wgpu::Texture,
    target_view: wgpu::TextureView,
    blitter: TextureBlitter,
    vello: VelloRenderer,
    scene: Scene,
    theme: CanvasTheme,
    editable_outline_cache: Option<EditableOutlineCacheEntry>,
    path_outline_cache: HashMap<EntityId, PathOutlineCacheEntry>,
    edit_controls_cache: HashMap<EntityId, EditControlsCacheEntry>,
    design_grid_cache: Vec<DesignGridCacheEntry>,
    grid_overlay: Option<GridOverlay>,
    text_outline_cache: HashMap<String, TextOutlineCacheEntry>,
    hud_text: crate::hud_text::HudText,
    measure_options: MeasureOptions,
    /// Draw points in the read-only grey style (interpolated instance).
    readonly_points: bool,
    curve_options: CurveOptions,
    device_scale: f64,
    width: u32,
    height: u32,
}

#[derive(Clone)]
struct EditableOutlineCacheEntry {
    edit_revision: u64,
    path: Rc<BezPath>,
}

#[derive(Clone)]
struct PathOutlineCacheEntry {
    signature: u64,
    path: Rc<BezPath>,
}

#[derive(Clone)]
struct EditControlsCacheEntry {
    key: EditControlsCacheKey,
    geometry: Rc<EditControlsGeometry>,
}

#[derive(Clone, Copy, PartialEq)]
struct EditControlsCacheKey {
    path_signature: u64,
    selection_signature: u64,
    view_coeffs: [u64; 6],
    point_scale_bits: u64,
}

impl EditControlsCacheKey {
    fn new(
        path: &Path,
        selection: &crate::editing::Selection,
        view: Affine,
        point_scale: f64,
    ) -> Self {
        Self {
            path_signature: path_outline_signature(path),
            selection_signature: path_selection_signature(path, selection),
            view_coeffs: view.as_coeffs().map(f64::to_bits),
            point_scale_bits: point_scale.to_bits(),
        }
    }
}

#[derive(Clone, Default)]
struct EditControlsGeometry {
    outline: BezPath,
    handle_lines: BezPath,
    smooth_circles: BezPath,
    corner_squares: BezPath,
    offcurve_circles: BezPath,
    hyper_circles: BezPath,
    selected_circles: BezPath,
    selected_squares: BezPath,
    start_arrow: Option<StartArrowGeometry>,
}

impl EditControlsGeometry {
    fn with_capacity(capacity: EditControlsGeometryCapacity) -> Self {
        Self {
            outline: BezPath::with_capacity(capacity.outline),
            handle_lines: BezPath::with_capacity(capacity.handle_lines),
            smooth_circles: BezPath::with_capacity(capacity.smooth_circles),
            corner_squares: BezPath::with_capacity(capacity.corner_squares),
            offcurve_circles: BezPath::with_capacity(capacity.offcurve_circles),
            hyper_circles: BezPath::with_capacity(capacity.hyper_circles),
            selected_circles: BezPath::with_capacity(capacity.selected_circles),
            selected_squares: BezPath::with_capacity(capacity.selected_squares),
            start_arrow: None,
        }
    }

    fn capacity(&self) -> EditControlsGeometryCapacity {
        EditControlsGeometryCapacity {
            outline: self.outline.elements().len(),
            handle_lines: self.handle_lines.elements().len(),
            smooth_circles: self.smooth_circles.elements().len(),
            corner_squares: self.corner_squares.elements().len(),
            offcurve_circles: self.offcurve_circles.elements().len(),
            hyper_circles: self.hyper_circles.elements().len(),
            selected_circles: self.selected_circles.elements().len(),
            selected_squares: self.selected_squares.elements().len(),
        }
    }

    fn append(&mut self, other: &Self) {
        append_bezpath(&mut self.outline, &other.outline);
        append_bezpath(&mut self.handle_lines, &other.handle_lines);
        append_bezpath(&mut self.smooth_circles, &other.smooth_circles);
        append_bezpath(&mut self.corner_squares, &other.corner_squares);
        append_bezpath(&mut self.offcurve_circles, &other.offcurve_circles);
        append_bezpath(&mut self.hyper_circles, &other.hyper_circles);
        append_bezpath(&mut self.selected_circles, &other.selected_circles);
        append_bezpath(&mut self.selected_squares, &other.selected_squares);
    }
}

#[derive(Clone, Copy, Default)]
struct EditControlsGeometryCapacity {
    outline: usize,
    handle_lines: usize,
    smooth_circles: usize,
    corner_squares: usize,
    offcurve_circles: usize,
    hyper_circles: usize,
    selected_circles: usize,
    selected_squares: usize,
}

impl EditControlsGeometryCapacity {
    fn add(&mut self, other: Self) {
        self.outline += other.outline;
        self.handle_lines += other.handle_lines;
        self.smooth_circles += other.smooth_circles;
        self.corner_squares += other.corner_squares;
        self.offcurve_circles += other.offcurve_circles;
        self.hyper_circles += other.hyper_circles;
        self.selected_circles += other.selected_circles;
        self.selected_squares += other.selected_squares;
    }
}

#[derive(Clone, Copy)]
struct StartArrowGeometry {
    center: Point,
    next: Point,
    selected: bool,
}

/// The grid as drawn this frame, kept so point windows can re-stroke it
/// clipped to their interiors (points are windows onto the grid).
#[derive(Clone)]
struct GridOverlay {
    accent: Rc<BezPath>,
    accent_alpha: f32,
    fine: Option<(Rc<BezPath>, f32)>,
}

#[derive(Clone)]
struct DesignGridCacheEntry {
    key: DesignGridCacheKey,
    fine_path: Rc<BezPath>,
    coarse_path: Rc<BezPath>,
}

#[derive(Clone, Copy, PartialEq)]
struct DesignGridCacheKey {
    spacing_bits: u64,
    coarse_n: u32,
    width: u32,
    height: u32,
    view_coeffs: [u64; 6],
    bounds: [u64; 4],
    origin: [u64; 2],
}

impl DesignGridCacheKey {
    #[allow(clippy::too_many_arguments)]
    fn new(
        spacing: f64,
        coarse_n: u32,
        width: u32,
        height: u32,
        view: Affine,
        min_x: f64,
        max_x: f64,
        min_y: f64,
        max_y: f64,
        origin_x: f64,
        origin_y: f64,
    ) -> Self {
        Self {
            spacing_bits: spacing.to_bits(),
            coarse_n,
            width,
            height,
            view_coeffs: view.as_coeffs().map(f64::to_bits),
            bounds: [
                min_x.to_bits(),
                max_x.to_bits(),
                min_y.to_bits(),
                max_y.to_bits(),
            ],
            origin: [origin_x.to_bits(), origin_y.to_bits()],
        }
    }
}

#[derive(Clone)]
struct TextOutlineCacheEntry {
    /// The edit revision this path was resolved at. Every edit bumps it, so a
    /// stale composite cannot survive a change to the anchors placing it.
    revision: u64,
    path: Rc<BezPath>,
}

impl Renderer {
    pub async fn new(canvas: HtmlCanvasElement, width: u32, height: u32) -> Result<Self, JsValue> {
        let instance = wgpu::Instance::new(&wgpu::InstanceDescriptor::default());

        let surface = instance
            .create_surface(wgpu::SurfaceTarget::Canvas(canvas))
            .map_err(|e| JsValue::from_str(&format!("create_surface: {e:?}")))?;

        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::HighPerformance,
                force_fallback_adapter: false,
                compatible_surface: Some(&surface),
            })
            .await
            .map_err(|e| JsValue::from_str(&format!("request_adapter: {e:?}")))?;

        let adapter_limits = adapter.limits();
        let mut limits = wgpu::Limits::default();
        limits.max_texture_dimension_2d = adapter_limits.max_texture_dimension_2d;

        let optional_features = wgpu::Features::CLEAR_TEXTURE | wgpu::Features::PIPELINE_CACHE;
        let required_features = adapter.features() & optional_features;

        let (device, queue) = adapter
            .request_device(&wgpu::DeviceDescriptor {
                label: Some("runebender device"),
                required_features,
                required_limits: limits,
                ..Default::default()
            })
            .await
            .map_err(|e| JsValue::from_str(&format!("request_device: {e:?}")))?;

        let capabilities = surface.get_capabilities(&adapter);
        let surface_format = capabilities
            .formats
            .into_iter()
            .find(|fmt| {
                matches!(
                    fmt,
                    wgpu::TextureFormat::Rgba8Unorm | wgpu::TextureFormat::Bgra8Unorm
                )
            })
            .ok_or_else(|| JsValue::from_str("no compatible surface format"))?;

        let surface_config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: surface_format,
            width,
            height,
            present_mode: wgpu::PresentMode::AutoVsync,
            desired_maximum_frame_latency: 1,
            alpha_mode: wgpu::CompositeAlphaMode::Auto,
            view_formats: vec![],
        };
        surface.configure(&device, &surface_config);

        let (target_texture, target_view) = create_intermediate_target(width, height, &device);
        let blitter = TextureBlitter::new(&device, surface_format);

        let vello = VelloRenderer::new(
            &device,
            RendererOptions {
                use_cpu: false,
                antialiasing_support: vello::AaSupport::area_only(),
                num_init_threads: None,
                pipeline_cache: None,
            },
        )
        .map_err(|e| JsValue::from_str(&format!("Renderer::new: {e:?}")))?;

        Ok(Self {
            device,
            queue,
            surface,
            surface_config,
            target_texture,
            target_view,
            blitter,
            vello,
            scene: Scene::new(),
            theme: CanvasTheme::default(),
            editable_outline_cache: None,
            path_outline_cache: HashMap::new(),
            edit_controls_cache: HashMap::new(),
            design_grid_cache: Vec::new(),
            grid_overlay: None,
            text_outline_cache: HashMap::new(),
            hud_text: crate::hud_text::HudText::new(),
            measure_options: MeasureOptions::default(),
            readonly_points: false,
            curve_options: CurveOptions::default(),
            device_scale: 1.0,
            width,
            height,
        })
    }

    pub fn set_theme_json(&mut self, theme_json: &str) -> Result<(), JsValue> {
        let input: CanvasThemeInput = serde_json::from_str(theme_json)
            .map_err(|e| JsValue::from_str(&format!("parse canvas theme: {e}")))?;
        self.theme.apply_input(input);
        self.design_grid_cache.clear();
        Ok(())
    }

    pub fn clear_glyph_geometry_caches(&mut self) {
        self.editable_outline_cache = None;
        self.path_outline_cache.clear();
        self.edit_controls_cache.clear();
        self.text_outline_cache.clear();
    }

    /// Forget the text-preview outlines. They are built from the parsed
    /// glyph set, which lives outside this struct and is replaced a glyph at
    /// a time as edits land — and the revision the cache keys on only tracks
    /// the glyph on the canvas, so a change to that set has to say so here.
    /// A composite draws whatever its base draws, so one glyph changing can
    /// change any of them: the whole cache goes.
    pub fn invalidate_text_outlines(&mut self) {
        self.text_outline_cache.clear();
    }

    pub fn resize(&mut self, width: u32, height: u32) {
        if width == 0 || height == 0 {
            return;
        }
        self.surface_config.width = width;
        self.surface_config.height = height;
        self.surface.configure(&self.device, &self.surface_config);
        let (target_texture, target_view) = create_intermediate_target(width, height, &self.device);
        self.target_texture = target_texture;
        self.target_view = target_view;
        self.width = width;
        self.height = height;
        self.design_grid_cache.clear();
    }

    pub fn set_device_scale(&mut self, scale: f64) {
        let next = scale.clamp(1.0, 8.0);
        if (self.device_scale - next).abs() > f64::EPSILON {
            self.device_scale = next;
            self.design_grid_cache.clear();
        }
    }

    pub fn set_readonly_points(&mut self, on: bool) {
        self.readonly_points = on;
    }

    pub fn set_measure_options(
        &mut self,
        colorize: bool,
        handles: bool,
        segments: bool,
        spans: bool,
        sidebearings: bool,
        popcount: bool,
    ) {
        self.measure_options = MeasureOptions {
            colorize,
            handles,
            segments,
            spans,
            sidebearings,
            popcount,
        };
    }

    pub fn set_curve_options(&mut self, comb: bool, continuity: bool) {
        self.curve_options = CurveOptions { comb, continuity };
    }

    fn px(&self, value: f64) -> f64 {
        value * self.device_scale
    }

    fn point_scale(&self, zoom: f64) -> f64 {
        // Keep points readable at close zoom without letting them dominate
        // the outline at wide zoom. Viewport zoom is measured in backing
        // pixels, so compute the scale curve in CSS/logical pixels.
        const MIN_ZOOM_SCALE: f64 = 0.72;
        const BASE_ZOOM_SCALE: f64 = 1.0;
        const FINE_GRID_ZOOM_SCALE: f64 = 1.6;
        const MAX_ZOOM_SCALE: f64 = 2.4;
        let logical_zoom = zoom / self.device_scale.max(1.0);
        let fine_grid_zoom = DESIGN_GRID_CLOSE_MIN_ZOOM / self.device_scale.max(1.0);
        let wide_t = (logical_zoom / DESIGN_GRID_MID_MIN_ZOOM).clamp(0.0, 1.0);
        let mid_t = ((logical_zoom - DESIGN_GRID_MID_MIN_ZOOM)
            / (fine_grid_zoom - DESIGN_GRID_MID_MIN_ZOOM).max(1e-6))
        .clamp(0.0, 1.0);
        let close_t =
            ((logical_zoom - fine_grid_zoom) / (fine_grid_zoom * 2.5).max(1e-6)).clamp(0.0, 1.0);
        let zoom_scale = if logical_zoom <= DESIGN_GRID_MID_MIN_ZOOM {
            lerp(MIN_ZOOM_SCALE, BASE_ZOOM_SCALE, smoothstep(wide_t))
        } else if logical_zoom <= fine_grid_zoom {
            lerp(BASE_ZOOM_SCALE, FINE_GRID_ZOOM_SCALE, smoothstep(mid_t))
        } else {
            lerp(FINE_GRID_ZOOM_SCALE, MAX_ZOOM_SCALE, smoothstep(close_t))
        };
        self.device_scale * zoom_scale
    }

    fn text_overlay_zoom_t(&self, zoom: f64) -> f64 {
        let logical_zoom = zoom / self.device_scale.max(1.0);
        let fine_grid_zoom = DESIGN_GRID_CLOSE_MIN_ZOOM / self.device_scale.max(1.0);
        (logical_zoom / (fine_grid_zoom * 1.5).max(1e-6)).clamp(0.0, 1.0)
    }

    /// Fade for the live measurement HUD: 0 until roughly the zoom where the
    /// fine 2-unit grid appears ("zoomed in close enough"), ramping to 1 as
    /// you push in further.
    fn measure_overlay_t(&self, zoom: f64) -> f64 {
        let logical_zoom = zoom / self.device_scale.max(1.0);
        // Visible across most working zooms, not just extreme close-ups: fade
        // in while the glyph still only fills part of the view, and reach full
        // opacity well before the fine grid appears.
        const START: f64 = 0.30;
        const FULL: f64 = 0.70;
        ((logical_zoom - START) / (FULL - START)).clamp(0.0, 1.0)
    }

    fn text_cursor_line_px(&self, zoom: f64) -> f64 {
        lerp(
            TEXT_CURSOR_LINE_PX,
            TEXT_CURSOR_LINE_MAX_PX,
            smoothstep(self.text_overlay_zoom_t(zoom)),
        )
    }

    /// Caret triangles sized off the sort's on-screen height, the way the
    /// corner marks are: a fixed screen size made the caret taller than
    /// the whole line once you zoomed out.
    fn text_cursor_marker_scale(&self, zoom: f64, sort_height: f64) -> f64 {
        let box_px = (sort_height.max(1.0) * zoom).max(1.0);
        let width = (box_px * TEXT_CURSOR_MARKER_FRACTION).clamp(
            self.px(TEXT_CURSOR_MARKER_MIN_PX),
            self.px(TEXT_CURSOR_MARKER_MAX_PX),
        );
        width / self.px(TEXT_CURSOR_TRIANGLE_WIDTH_PX)
    }

    /// Size of a sort's corner marks, in device pixels, from the height
    /// one sort takes up on screen. Zooming out shrinks the marks with
    /// the text; zooming in grows them to the old fixed size and stops.
    fn text_sort_mark_size(&self, zoom: f64, sort_height: f64) -> f64 {
        let box_px = (sort_height.max(1.0) * zoom).max(1.0);
        (box_px * TEXT_SORT_MARK_FRACTION).clamp(
            self.px(TEXT_SORT_MARK_MIN_SIZE),
            self.px(TEXT_SORT_MARK_SIZE),
        )
    }

    /// Whether the corner marks and quiet metric boxes are drawn at all.
    fn text_metric_marks_visible(&self, mark_size: f64) -> bool {
        mark_size >= self.px(TEXT_SORT_MARK_HIDE_BELOW_PX)
    }

    /// Paint one frame against the given editor state.
    pub fn render(
        &mut self,
        state: &EditorState,
        glyphs: &std::collections::HashMap<String, norad::Glyph>,
        preview_mode: bool,
        text_mode_active: bool,
    ) -> Result<(), JsValue> {
        self.scene.reset();
        self.draw_state(state, glyphs, preview_mode, text_mode_active, None);
        self.present()
    }

    /// Paint one frame while a keyboard nudge burst is active.
    ///
    /// The editor already knows which contours are being translated
    /// during a nudge burst, so reuse cached geometry for all other
    /// contours and only rebuild the paths that actually changed.
    pub fn render_changed_paths(
        &mut self,
        state: &EditorState,
        glyphs: &std::collections::HashMap<String, norad::Glyph>,
        changed_path_indices: &[usize],
        preview_mode: bool,
        text_mode_active: bool,
    ) -> Result<(), JsValue> {
        self.scene.reset();
        let changed_paths = changed_path_indices.iter().copied().collect::<HashSet<_>>();
        self.draw_state(state, glyphs, preview_mode, text_mode_active, Some(&changed_paths));
        self.present()
    }

    fn draw_state(
        &mut self,
        state: &EditorState,
        glyphs: &std::collections::HashMap<String, norad::Glyph>,
        preview_mode: bool,
        text_mode_active: bool,
        changed_path_indices: Option<&HashSet<usize>>,
    ) {
        self.grid_overlay = None;
        let view = state.viewport.affine();
        let has_text_session = state.has_text_session;
        let text_layout =
            has_text_session.then(|| state.text_buffer.layout(state.text_line_height()));
        let active_sort_origin = text_layout
            .as_ref()
            .and_then(|layout| {
                let active_index = state.text_buffer.active_sort()?;
                layout
                    .items
                    .iter()
                    .find(|item| item.index == active_index)
                    .map(|item| (item.x, item.y))
            })
            .unwrap_or((0.0, 0.0));
        let glyph_view = view * Affine::translate(active_sort_origin);

        if !preview_mode {
            self.draw_design_grid(state, view, active_sort_origin.0, active_sort_origin.1);

            // Metric guides go in first so the glyph fill paints on top.
            if !has_text_session {
                self.draw_metric_guides(state, glyph_view);
            }
        }

        // Reference art goes under everything the tools draw.
        if !preview_mode {
            self.draw_underlays(state, glyph_view);
        }

        if has_text_session {
            self.draw_text_buffer(
                state,
                glyphs,
                view,
                preview_mode,
                text_mode_active,
                text_layout.as_ref(),
            );
            // Only draw the single-glyph editor's outline + handles when a
            // sort is actually active — that's the glyph being edited in
            // context, drawn at the active sort's origin. With no active
            // sort (e.g. right after typing a run), glyph_view falls back
            // to the run origin and the editor would render whatever glyph
            // was last open as a ghost over the start of the text.
            if !preview_mode && !text_mode_active && state.text_buffer.active_sort().is_some() {
                // Ghost fill for the sort being edited, lighter than the
                // neighbours around it.
                let outline = self.editable_outline_path(state);
                if !outline.elements().is_empty() {
                    self.scene.fill(
                        Fill::NonZero,
                        glyph_view,
                        self.theme
                            .text_preview_fill
                            .with_alpha(ACTIVE_GLYPH_FILL_ALPHA),
                        None,
                        outline.as_ref(),
                    );
                }
                self.draw_edit_controls(state, glyph_view, changed_path_indices);
            }
            return;
        }

        // Glyph fill (in design space — viewport applies the Y-flip).
        // Combine every contour into ONE BezPath before filling so the
        // NonZero winding rule treats opposite-wound inner contours as
        // holes (UFO/PostScript convention). Filling each contour
        // separately would paint counters solid.
        let outline = changed_path_indices
            .filter(|indices| !indices.is_empty() && !preview_mode)
            .map(|indices| self.editable_outline_path_for_changed_paths(state, indices))
            .unwrap_or_else(|| self.editable_outline_path(state));
        if preview_mode {
            let mut combined = outline.as_ref().clone();
            for component in &state.component_previews {
                for el in component.transformed_path.elements() {
                    combined.push(*el);
                }
            }
            if !combined.elements().is_empty() {
                self.scene.fill(
                    Fill::NonZero,
                    glyph_view,
                    self.theme.preview_fill,
                    None,
                    &combined,
                );
            }
            self.draw_text_buffer(state, glyphs, view, true, text_mode_active, None);
            return;
        }
        for component in &state.component_previews {
            if component.transformed_path.elements().is_empty() {
                continue;
            }
            let fill = if state.selected_component == Some(component.id) {
                self.theme.component_selected_fill
            } else {
                self.theme.component_fill
            };
            self.scene.fill(
                Fill::NonZero,
                glyph_view,
                fill,
                None,
                component.transformed_path.as_ref(),
            );
            if state.selected_component == Some(component.id) {
                let screen_path = glyph_view * component.transformed_path.as_ref();
                self.scene.stroke(
                    &Stroke::new(self.px(COMPONENT_SELECTION_STROKE_PX)),
                    Affine::IDENTITY,
                    self.theme.text_cursor,
                    None,
                    &screen_path,
                );
            }
        }
        if !outline.elements().is_empty() {
            self.scene.fill(
                Fill::NonZero,
                glyph_view,
                self.theme
                    .text_preview_fill
                    .with_alpha(ACTIVE_GLYPH_FILL_ALPHA),
                None,
                outline.as_ref(),
            );
        }
        self.draw_edit_controls(state, glyph_view, changed_path_indices);
    }

    /// Grey markers on every point of a screen-space path: on-curve
    /// points as circles, off-curve control points slightly smaller.
    /// Used for the sorts around the one being edited, which have no
    /// editable point structure of their own (they come from the text
    /// inventory as outlines).
    fn draw_readonly_points(&mut self, screen_path: &BezPath, zoom: f64) {
        let scale = self.point_scale(zoom);
        let on_radius = SMOOTH_POINT_RADIUS_PX * scale * 0.85;
        let off_radius = OFFCURVE_POINT_RADIUS_PX * scale * 0.6;
        let mut on_curve = BezPath::new();
        let mut off_curve = BezPath::new();
        // Handle lines join each control point to the on-curve point it
        // belongs to, the same pairing the editable glyph shows.
        let mut handles = BezPath::new();
        let mut current = Point::ZERO;
        let mut subpath_start = Point::ZERO;
        for element in screen_path.elements() {
            match *element {
                PathEl::MoveTo(p) => {
                    append_circle_path(&mut on_curve, p, on_radius);
                    current = p;
                    subpath_start = p;
                }
                PathEl::LineTo(p) => {
                    append_circle_path(&mut on_curve, p, on_radius);
                    current = p;
                }
                PathEl::QuadTo(c, p) => {
                    append_circle_path(&mut off_curve, c, off_radius);
                    append_circle_path(&mut on_curve, p, on_radius);
                    handles.move_to(current);
                    handles.line_to(c);
                    handles.move_to(c);
                    handles.line_to(p);
                    current = p;
                }
                PathEl::CurveTo(c1, c2, p) => {
                    append_circle_path(&mut off_curve, c1, off_radius);
                    append_circle_path(&mut off_curve, c2, off_radius);
                    append_circle_path(&mut on_curve, p, on_radius);
                    handles.move_to(current);
                    handles.line_to(c1);
                    handles.move_to(c2);
                    handles.line_to(p);
                    current = p;
                }
                PathEl::ClosePath => {
                    current = subpath_start;
                }
            }
        }
        let stroke = Stroke::new(LINE_PX * scale);
        if !handles.elements().is_empty() {
            self.scene.stroke(
                &stroke,
                Affine::IDENTITY,
                POINT_READONLY_OUTER,
                None,
                &handles,
            );
        }
        for path in [&off_curve, &on_curve] {
            if path.elements().is_empty() {
                continue;
            }
            self.scene.fill(
                Fill::NonZero,
                Affine::IDENTITY,
                POINT_READONLY_INNER,
                None,
                path,
            );
            self.scene
                .stroke(&stroke, Affine::IDENTITY, POINT_READONLY_OUTER, None, path);
        }
    }

    /// True once the design grid has faded in — the zoom where the user
    /// is drawing rather than reading.
    fn design_grid_visible(&self, state: &EditorState) -> bool {
        state.viewport.zoom > DESIGN_GRID_MID_MIN_ZOOM
    }

    fn draw_edit_controls(
        &mut self,
        state: &EditorState,
        glyph_view: Affine,
        changed_path_indices: Option<&HashSet<usize>>,
    ) {
        // Curvature comb first, so the outline, handle lines, and points draw
        // on top of it and stay grabbable.
        self.draw_curvature_comb(state, glyph_view);
        // Handle lines and points are drawn in screen space so they
        // stay at constant pixel size regardless of zoom.
        let point_scale = self.point_scale(state.viewport.zoom);
        let mut current_path_ids = HashSet::with_capacity(state.paths.len());
        let mut controls_by_path = Vec::with_capacity(state.paths.len());
        let mut combined_capacity = EditControlsGeometryCapacity::default();
        let mut start_arrows = Vec::new();
        for (index, path) in state.paths.iter().enumerate() {
            let id = path_id(path);
            current_path_ids.insert(id);
            let controls = self.edit_controls_for_path(
                index,
                path,
                glyph_view,
                &state.selection,
                point_scale,
                changed_path_indices,
            );
            combined_capacity.add(controls.capacity());
            if let Some(start_arrow) = controls.start_arrow {
                start_arrows.push(start_arrow);
            }
            controls_by_path.push(controls);
        }
        self.edit_controls_cache
            .retain(|id, _| current_path_ids.contains(id));
        let mut combined = EditControlsGeometry::with_capacity(combined_capacity);
        for controls in &controls_by_path {
            combined.append(controls);
        }
        self.draw_edit_controls_geometry(&combined, &start_arrows, point_scale);

        self.draw_propagated_anchors(state, glyph_view);
        self.draw_anchors(state, glyph_view);

        if let Some(preview) = state.segment_hover {
            self.draw_segment_hover(preview);
        }
        if let Some(rect) = state.marquee {
            self.draw_marquee(rect);
        }
        if let Some(preview) = state.shape_preview {
            self.draw_shape_preview(preview);
        }
        if let Some(preview) = state.pen_preview {
            self.draw_pen_preview(preview);
        }
        if let Some(preview) = state.measure_preview.as_ref() {
            self.draw_measure_preview(preview);
        }
        if let Some(preview) = state.knife_preview.as_ref() {
            self.draw_knife_preview(preview, state.viewport.zoom);
        }

        self.draw_measurements(state, glyph_view);
        self.draw_continuity_markers(state, glyph_view);
    }

    /// The curve-smoothness HUD: a Speedpunk-style curvature comb and/or
    /// per-node continuity markers (G0/G1/G2/G3), toggled from the curve panel.
    /// Curvature comb — drawn BEHIND the outline/handles/points so they stay
    /// selectable and movable over it.
    fn draw_curvature_comb(&mut self, state: &EditorState, glyph_view: Affine) {
        if self.curve_options.comb {
            let maxk = crate::curve::max_curvature(&state.paths);
            if maxk > 1e-12 {
                // Shorter than Speedpunk's default so ribs don't collide
                // across tight counters; scales with the em.
                let scale = 74.0 / maxk;
                // Fewer samples => distinct, readable ribs.
                let strips = crate::curve::curvature_comb(&state.paths, 1.0, scale, false, 16);
                // Every rib edge (outer, base, and the fins between ribs) is
                // stroked in the background colour at the contour's weight, so
                // the ribbon reads as separated fins that blend into the page.
                let fin = Stroke::new(self.px(COMB_FIN_PX));
                let bg = self.theme.bg;
                for strip in &strips {
                    // Fills first, then the separators once each. Stroking
                    // every quad (as this used to) painted each shared edge
                    // twice and painted over the contour itself, which is
                    // why the ribs read heavier than every other line.
                    for w in strip.windows(2) {
                        let (s0, s1) = (w[0], w[1]);
                        let mut quad = BezPath::new();
                        quad.move_to(glyph_view * s0.on);
                        quad.line_to(glyph_view * s1.on);
                        quad.line_to(glyph_view * s1.outer);
                        quad.line_to(glyph_view * s0.outer);
                        quad.close_path();
                        let k = (s0.kappa.abs() + s1.kappa.abs()) * 0.5 / maxk;
                        self.scene.fill(
                            Fill::NonZero,
                            Affine::IDENTITY,
                            curve_gradient(k),
                            None,
                            &quad,
                        );
                    }

                    let mut edges = BezPath::new();
                    for sample in strip.iter() {
                        edges.move_to(glyph_view * sample.on);
                        edges.line_to(glyph_view * sample.outer);
                    }
                    if let Some(first) = strip.first() {
                        edges.move_to(glyph_view * first.outer);
                        for sample in &strip[1..] {
                            edges.line_to(glyph_view * sample.outer);
                        }
                    }
                    self.scene.stroke(&fin, Affine::IDENTITY, bg, None, &edges);
                }
            }
        }
    }

    /// Continuity markers — a hollow ring around each smooth node so the point
    /// marker shows through inside. Colors are chosen distinct from the
    /// editor's point palette (see the legend in CurvePanel). Drawn on top.
    fn draw_continuity_markers(&mut self, state: &EditorState, glyph_view: Affine) {
        if self.curve_options.continuity {
            let g2 = self.theme.continuity_g2;
            let g1 = self.theme.continuity_g1;
            let line = self.theme.continuity_line;
            let kink = self.theme.continuity_kink;
            // Ring geometry follows the same zoom curve the points do,
            // so the gap between a ring and the point inside it stays
            // constant however far you zoom.
            let scale = self.point_scale(state.viewport.zoom);
            let r = SMOOTH_POINT_RADIUS_PX * scale * 1.9;
            let ring = Stroke::new(LINE_PX * scale);
            for nc in crate::curve::node_continuity(&state.paths) {
                use crate::curve::GLevel;
                let color = match nc.level {
                    GLevel::Corner => continue,
                    GLevel::G2 | GLevel::G3 => g2,
                    GLevel::G1 => g1,
                    GLevel::G1Line => line,
                    GLevel::Kink => kink,
                };
                let c = glyph_view * nc.at;
                let circle = Circle::new(c, r);
                // Same dark casing the points and handles get, so the
                // ring stays legible over the curvature comb.
                self.scene.stroke(
                    &Stroke::new(ring.width + HALO_PX),
                    Affine::IDENTITY,
                    self.theme.halo,
                    None,
                    &circle,
                );
                self.scene
                    .stroke(&ring, Affine::IDENTITY, color, None, &circle);
            }
        }
    }

    /// The live grid-measurement HUD. Layers are toggled by MeasureOptions:
    /// colorized outline/curves/handles, and popcount-tiered `value = sum`
    /// labels for handle lengths, straight segment lengths, and scan-line
    /// stem/counter/thickness spans (the spans get gapped arrow dimension
    /// lines). Set in Virtua itself; fades in with zoom.
    fn draw_measurements(&mut self, state: &EditorState, glyph_view: Affine) {
        let opts = self.measure_options;
        if !opts.any() {
            return;
        }
        let t = self.measure_overlay_t(state.viewport.zoom);
        if t <= 0.0 {
            return;
        }
        let t32 = t as f32;

        // Popcount tiers on the glyph grid's own mark colours: 1 power
        // is structural (green), 2 an elegant sum (yellow), 3 acceptable
        // (orange), 4+ a flagged correction (red). The host sends the
        // theme's versions of those four.
        let green = self.theme.popcount_1;
        let yellow = self.theme.popcount_2;
        let orange = self.theme.popcount_3;
        let red = self.theme.popcount_4;
        let tier = |pc: u32| match pc {
            0 | 1 => green,
            2 => yellow,
            3 => orange,
            _ => red,
        };

        // Colorize the outline: redraw each segment/curve/handle in its tier
        // color (the plain gray versions were skipped in draw_edit_controls).
        if opts.colorize {
            for cs in measure::colored_strokes(&state.paths) {
                let mut screen = cs.path;
                screen.apply_affine(glyph_view);
                let width = if cs.wide {
                    PATH_STROKE_PX
                } else {
                    HANDLE_LINE_PX
                };
                self.scene.stroke(
                    &Stroke::new(self.px(width)),
                    Affine::IDENTITY,
                    tier(cs.popcount).multiply_alpha(t32),
                    None,
                    &screen,
                );
            }
        }

        let dim_stroke = Stroke::new(self.px(1.25));
        let label_px = self.px(15.0) as f32;
        // Screen-space rects of labels already placed this frame, so new
        // labels can dodge them.
        let mut placed: Vec<Rect> = Vec::new();

        // Side bearings: an arrow line from each advance margin to the glyph's
        // extreme point, labeled + popcount-colored.
        if opts.sidebearings && state.advance_width > 0.0 {
            if let Some(sb) = measure::side_bearings(&state.paths, state.advance_width) {
                for (is_left, x, y, val) in [
                    (true, sb.min_x, sb.y_left, sb.lsb),
                    (false, sb.max_x, sb.y_right, sb.rsb),
                ] {
                    let color = tier(measure::popcount(val)).multiply_alpha(t32 * 0.9);
                    let margin_x = if is_left { 0.0 } else { sb.advance };
                    let a = glyph_view * Point::new(margin_x, y);
                    let b = glyph_view * Point::new(x, y);
                    self.draw_dimension_line(a, b, color, &dim_stroke);
                    self.place_label(a, b, &opts.label(val), color, label_px, &mut placed);
                }
            }
        }

        if !(opts.handles || opts.segments || opts.spans) {
            return;
        }

        let measurements = measure::glyph_measurements(&state.paths);

        for m in &measurements {
            let show = match m.kind {
                MeasureKind::Handle => opts.handles,
                MeasureKind::Segment => opts.segments,
                MeasureKind::Horizontal | MeasureKind::Vertical => opts.spans,
            };
            if !show {
                continue;
            }

            let a = glyph_view * m.a;
            let b = glyph_view * m.b;
            let color = tier(measure::popcount(m.length)).multiply_alpha(t32);

            // Spans get a gapped dimension line with outward arrowheads;
            // handles/segments annotate existing outline/handle strokes.
            if matches!(m.kind, MeasureKind::Horizontal | MeasureKind::Vertical) {
                self.draw_dimension_line(a, b, color, &dim_stroke);
            }

            self.place_label(a, b, &opts.label(m.length), color, label_px, &mut placed);
        }
    }

    /// Place a measurement label with basic spatial awareness: anchor it just
    /// off the line by orientation (beside a vertical line, above a horizontal
    /// one, rather than centered on top of it), then step it further out — and
    /// to the other side if needed — until it clears every label already
    /// placed this frame.
    fn place_label(
        &mut self,
        a: Point,
        b: Point,
        text: &str,
        color: Srgb,
        label_px: f32,
        placed: &mut Vec<Rect>,
    ) {
        let (dx, dy) = (b.x - a.x, b.y - a.y);
        let len = dx.hypot(dy).max(1e-6);
        // Perpendicular to the line, pointed to the preferred side: up for a
        // horizontalish line, right for a verticalish one.
        let (mut nx, mut ny) = (-dy / len, dx / len);
        let horizontalish = dx.abs() >= dy.abs();
        if (horizontalish && ny > 0.0) || (!horizontalish && nx < 0.0) {
            nx = -nx;
            ny = -ny;
        }

        let w = text.chars().count() as f64 * label_px as f64 * 0.55;
        let h = label_px as f64;
        let mid = a.midpoint(b);
        let base = self.px(6.0);
        let step = h + self.px(4.0);
        let pad = self.px(2.0);

        let top_left = |dirx: f64, diry: f64, dist: f64| {
            let cx = mid.x + dirx * dist;
            let cy = mid.y + diry * dist;
            // Anchor the label edge nearest the line, so it never crosses it.
            let x = if dirx > 0.3 {
                cx
            } else if dirx < -0.3 {
                cx - w
            } else {
                cx - w / 2.0
            };
            let y = if diry > 0.3 {
                cy
            } else if diry < -0.3 {
                cy - h
            } else {
                cy - h / 2.0
            };
            Point::new(x, y)
        };

        let mut chosen = top_left(nx, ny, base);
        'search: for &sign in &[1.0_f64, -1.0] {
            let (dirx, diry) = (nx * sign, ny * sign);
            for k in 0..6 {
                let cand = top_left(dirx, diry, base + k as f64 * step);
                let rect = Rect::new(
                    cand.x - pad,
                    cand.y - pad,
                    cand.x + w + pad,
                    cand.y + h + pad,
                );
                let clear = !placed
                    .iter()
                    .any(|r| r.x0 < rect.x1 && rect.x0 < r.x1 && r.y0 < rect.y1 && rect.y0 < r.y1);
                if clear {
                    chosen = cand;
                    break 'search;
                }
            }
        }

        placed.push(Rect::new(chosen.x, chosen.y, chosen.x + w, chosen.y + h));
        self.hud_text
            .draw_line(&mut self.scene, text, chosen, label_px, color, HALO_COLOR);
    }

    /// A dimension line for a span: a shaft that stops short of both endpoints
    /// (so it doesn't touch the contour) with an outward-pointing arrowhead at
    /// each end coming close to the edge.
    fn draw_dimension_line(&mut self, a: Point, b: Point, color: Srgb, stroke: &Stroke) {
        let (dx, dy) = (b.x - a.x, b.y - a.y);
        let len = dx.hypot(dy);
        if len < 1e-3 {
            return;
        }
        let (ux, uy) = (dx / len, dy / len); // unit a -> b
        let (px, py) = (-uy, ux); // perpendicular
        let end_gap = self.px(3.0);
        let head = self.px(7.0);
        let wing = self.px(4.0);
        let a2 = Point::new(a.x + ux * end_gap, a.y + uy * end_gap);
        let b2 = Point::new(b.x - ux * end_gap, b.y - uy * end_gap);

        let mut path = BezPath::new();
        path.move_to(a2);
        path.line_to(b2);
        // Arrowhead at a2, tip pointing outward (toward a); wings open inward.
        path.move_to(a2);
        path.line_to(Point::new(
            a2.x + ux * head + px * wing,
            a2.y + uy * head + py * wing,
        ));
        path.move_to(a2);
        path.line_to(Point::new(
            a2.x + ux * head - px * wing,
            a2.y + uy * head - py * wing,
        ));
        // Arrowhead at b2, tip pointing outward (toward b).
        path.move_to(b2);
        path.line_to(Point::new(
            b2.x - ux * head + px * wing,
            b2.y - uy * head + py * wing,
        ));
        path.move_to(b2);
        path.line_to(Point::new(
            b2.x - ux * head - px * wing,
            b2.y - uy * head - py * wing,
        ));
        self.scene
            .stroke(stroke, Affine::IDENTITY, color, None, &path);
    }

    fn draw_anchors(&mut self, state: &EditorState, view: Affine) {
        let scale = self.point_scale(state.viewport.zoom);
        let outline_stroke = Stroke::new(POINT_OUTLINE_PX * scale);
        for anchor in &state.anchors {
            let center = view * anchor.point;
            let selected = state.is_anchor_selected(anchor.id);
            let radius = (if selected {
                SMOOTH_POINT_SELECTED_RADIUS_PX
            } else {
                SMOOTH_POINT_RADIUS_PX
            }) * scale;
            // Diamonds, not circles: anchors read as their own kind of
            // thing next to on-curve and off-curve points.
            let diamond = diamond_path(center, radius * ANCHOR_DIAMOND_SCALE);
            let (inner, outer) = if selected {
                (
                    self.theme.point_selected_inner,
                    self.theme.point_selected_outer,
                )
            } else {
                (POINT_INNER, ANCHOR_MARK_PINK)
            };
            self.scene
                .fill(Fill::NonZero, Affine::IDENTITY, inner, None, &diamond);
            self.scene
                .stroke(&outline_stroke, Affine::IDENTITY, outer, None, &diamond);
        }
    }

    fn draw_propagated_anchors(&mut self, state: &EditorState, view: Affine) {
        let scale = self.point_scale(state.viewport.zoom);
        let radius = SMOOTH_POINT_RADIUS_PX * scale;
        let outline_stroke = Stroke::new(POINT_OUTLINE_PX * scale);
        for anchor in &state.propagated_anchors {
            // Same diamond as a real anchor, outline only: these come
            // from the base glyph and are not editable here.
            let diamond = diamond_path(view * anchor.point, radius * ANCHOR_DIAMOND_SCALE);
            self.scene.stroke(
                &outline_stroke,
                Affine::IDENTITY,
                ANCHOR_MARK_PINK,
                None,
                &diamond,
            );
        }
    }

    /// Is this sort anywhere near the canvas? A page of text is mostly
    /// off screen, and an off-screen sort costs as much to build as a
    /// visible one — path, transform, fill and all.
    fn text_item_visible(
        &self,
        item: &TextLayoutItem,
        sort_top: f64,
        sort_bottom: f64,
        view: Affine,
    ) -> bool {
        // A sort's ink can reach past its metric box, so keep a margin of
        // roughly one line either side rather than culling to the pixel.
        let slack = (sort_top - sort_bottom).abs().max(1.0);
        let box_rect = Rect::new(
            item.x - slack,
            item.y + sort_bottom - slack,
            item.x + item.advance_width + slack,
            item.y + sort_top + slack,
        );
        let screen = view.transform_rect_bbox(box_rect);
        screen.x1 >= 0.0
            && screen.y1 >= 0.0
            && screen.x0 <= self.width as f64
            && screen.y0 <= self.height as f64
    }

    fn draw_text_buffer(
        &mut self,
        state: &EditorState,
        glyphs: &std::collections::HashMap<String, norad::Glyph>,
        view: Affine,
        preview_mode: bool,
        text_mode_active: bool,
        frame_layout: Option<&TextLayout>,
    ) {
        let (ascender, descender) = state.text_metric_bounds();
        let (sort_top, sort_bottom) = state.text_sort_metric_bounds();
        let mark_size = self.text_sort_mark_size(state.viewport.zoom, sort_top - sort_bottom);
        let line_height = state.text_line_height();
        let layout_storage;
        let layout = if let Some(layout) = frame_layout {
            layout
        } else {
            layout_storage = state.text_buffer.layout(line_height);
            &layout_storage
        };
        let kern_sort_index = state.text_buffer.manual_kerning_sort();

        if !preview_mode {
            let mut active_metric_path = BezPath::new();
            let mut previous_metric_path = BezPath::new();
            let mut guide_metric_path = BezPath::new();
            let mut cursor_metric_path = BezPath::new();
            // Full metric boxes for the sorts nobody is editing, drawn
            // first so the marks sit on top of them.
            let mut quiet_metric_path = BezPath::new();
            let mut active_sort_items: Vec<TextLayoutItem> = Vec::new();
            for item in &layout.items {
                let sort_active = state
                    .text_buffer
                    .sort(item.index)
                    .map(|sort| sort.active)
                    .unwrap_or(false);
                if !sort_active && !self.text_item_visible(item, sort_top, sort_bottom, view) {
                    continue;
                }
                if !text_mode_active && sort_active {
                    // Drawn after the quiet boxes below: sorts share edges,
                    // so a neighbour's grey box would otherwise paint over
                    // the green metrics of the sort being edited.
                    active_sort_items.push(*item);
                    continue;
                }
                let metric_color = if text_mode_active {
                    match kern_sort_index {
                        Some(index) if index == item.index => self.theme.text_kern_active,
                        Some(index) if index == item.index + 1 => self.theme.text_kern_previous,
                        _ => self.theme.metric_guide,
                    }
                } else if sort_active {
                    self.theme.text_cursor
                } else {
                    self.theme.metric_guide
                };
                let metric_path = if metric_color == self.theme.text_kern_active {
                    &mut active_metric_path
                } else if metric_color == self.theme.text_kern_previous {
                    &mut previous_metric_path
                } else if metric_color == self.theme.text_cursor {
                    &mut cursor_metric_path
                } else {
                    &mut guide_metric_path
                };
                // Only the sorts nobody is editing: the active one draws
                // its own metrics, and a grey box on top of those is what
                // made them look wrong.
                if !sort_active {
                    append_text_sort_metric_box(
                        &mut quiet_metric_path,
                        item.x,
                        item.y,
                        item.advance_width,
                        state,
                        sort_top,
                        sort_bottom,
                        view,
                    );
                }
                append_text_sort_corner_marks(
                    metric_path,
                    item.x,
                    item.y,
                    item.advance_width,
                    ascender,
                    descender,
                    sort_top,
                    sort_bottom,
                    view,
                    mark_size,
                );
            }
            let stroke = Stroke::new(self.px(METRIC_LINE_PX));
            if self.text_metric_marks_visible(mark_size) {
                self.stroke_metric_batch(&quiet_metric_path, self.theme.metric_quiet, &stroke);
                self.stroke_metric_batch(&guide_metric_path, self.theme.metric_guide, &stroke);
                self.stroke_metric_batch(&active_metric_path, self.theme.text_kern_active, &stroke);
                self.stroke_metric_batch(
                    &previous_metric_path,
                    self.theme.text_kern_previous,
                    &stroke,
                );
                self.stroke_metric_batch(&cursor_metric_path, self.theme.text_cursor, &stroke);
            }
            for item in &active_sort_items {
                self.draw_text_sort_metrics(state, item.x, item.y, item.advance_width, view);
            }
        }

        for item in &layout.items {
            let Some(sort) = state.text_buffer.sort(item.index) else {
                continue;
            };
            if !sort.active && !self.text_item_visible(item, sort_top, sort_bottom, view) {
                continue;
            }
            let render_active_editable = !preview_mode && sort.active && !text_mode_active;
            if render_active_editable {
                for component in &state.component_previews {
                    if component.transformed_path.elements().is_empty() {
                        continue;
                    }
                    let component_fill = if state.selected_component == Some(component.id) {
                        self.theme.component_selected_fill
                    } else {
                        self.theme.component_fill
                    };
                    self.scene.fill(
                        Fill::NonZero,
                        view * Affine::translate((item.x, item.y)),
                        component_fill,
                        None,
                        component.transformed_path.as_ref(),
                    );
                    if state.selected_component == Some(component.id) {
                        let screen_path = (view * Affine::translate((item.x, item.y)))
                            * component.transformed_path.as_ref();
                        self.scene.stroke(
                            &Stroke::new(self.px(COMPONENT_SELECTION_STROKE_PX)),
                            Affine::IDENTITY,
                            self.theme.text_cursor,
                            None,
                            &screen_path,
                        );
                    }
                }
            } else {
                let Some(glyph_name) = sort.glyph_name() else {
                    continue;
                };
                // Straight from the parsed glyphs. These used to come from an
                // SVG string in the text buffer, regenerated and re-sent on
                // every edit, behind a cache keyed by the string's address —
                // so a composite could keep drawing a shape its anchors no
                // longer described. There is nothing left to invalidate.
                let Some(path) = self.text_preview_path(glyph_name, state, glyphs) else {
                    continue;
                };
                if path.elements().is_empty() {
                    continue;
                }
                let transform = view * Affine::translate((item.x, item.y));
                let zoomed_in = !preview_mode && self.design_grid_visible(state);
                let fill = if zoomed_in {
                    self.theme
                        .text_preview_fill
                        .with_alpha(INACTIVE_GLYPH_FILL_ALPHA)
                } else {
                    self.theme.text_preview_fill
                };
                self.scene
                    .fill(Fill::NonZero, transform, fill, None, path.as_ref());
                if zoomed_in {
                    // Same grey, drawn as an outline so the neighbours
                    // read as structure next to the glyph being edited,
                    // with their points shown greyed out — the same
                    // read-only treatment an interpolated instance gets.
                    let screen_path = transform * path.as_ref();
                    self.scene.stroke(
                        &Stroke::new(self.px(LINE_PX)),
                        Affine::IDENTITY,
                        self.theme.text_preview_fill,
                        None,
                        &screen_path,
                    );
                    self.draw_readonly_points(&screen_path, state.viewport.zoom);
                }
            }
        }

        if !preview_mode && text_mode_active {
            self.draw_text_cursor(
                layout.cursor_x,
                layout.cursor_y,
                sort_top,
                sort_bottom,
                view,
                state.viewport.zoom,
            );
        }
    }

    fn editable_outline_path(&mut self, state: &EditorState) -> Rc<BezPath> {
        let edit_revision = state.edit_revision();
        if let Some(entry) = &self.editable_outline_cache {
            if entry.edit_revision == edit_revision {
                return Rc::clone(&entry.path);
            }
        }

        let path = self.build_editable_outline_path_with_cache(state);
        let path = Rc::new(path);
        self.editable_outline_cache = Some(EditableOutlineCacheEntry {
            edit_revision,
            path: Rc::clone(&path),
        });
        path
    }

    fn editable_outline_path_for_changed_paths(
        &mut self,
        state: &EditorState,
        changed_path_indices: &HashSet<usize>,
    ) -> Rc<BezPath> {
        let mut current_path_ids = HashSet::with_capacity(state.paths.len());
        let mut path_outlines = Vec::with_capacity(state.paths.len());
        let mut combined_capacity = 0usize;

        for (index, path) in state.paths.iter().enumerate() {
            let id = path_id(path);
            current_path_ids.insert(id);
            let path_changed = changed_path_indices.contains(&index);
            let cached = (!path_changed)
                .then(|| {
                    self.path_outline_cache
                        .get(&id)
                        .map(|entry| Rc::clone(&entry.path))
                })
                .flatten();
            let path_outline = if let Some(cached) = cached {
                cached
            } else {
                let signature = path_outline_signature(path);
                let cached = self
                    .path_outline_cache
                    .get(&id)
                    .filter(|entry| entry.signature == signature)
                    .map(|entry| Rc::clone(&entry.path));
                if let Some(cached) = cached {
                    cached
                } else {
                    let mut outline = BezPath::new();
                    path.append_to_bezpath(&mut outline);
                    let outline = Rc::new(outline);
                    self.path_outline_cache.insert(
                        id,
                        PathOutlineCacheEntry {
                            signature,
                            path: Rc::clone(&outline),
                        },
                    );
                    outline
                }
            };
            combined_capacity += path_outline.elements().len();
            path_outlines.push(path_outline);
        }

        self.path_outline_cache
            .retain(|id, _| current_path_ids.contains(id));

        let mut combined = BezPath::with_capacity(combined_capacity);
        for path_outline in &path_outlines {
            append_bezpath(&mut combined, path_outline);
        }

        let path = Rc::new(combined);
        self.editable_outline_cache = Some(EditableOutlineCacheEntry {
            edit_revision: state.edit_revision(),
            path: Rc::clone(&path),
        });
        path
    }

    fn build_editable_outline_path_with_cache(&mut self, state: &EditorState) -> BezPath {
        let mut current_path_ids = HashSet::with_capacity(state.paths.len());
        let mut path_outlines = Vec::with_capacity(state.paths.len());
        let mut combined_capacity = 0usize;

        for path in &state.paths {
            let id = path_id(path);
            let signature = path_outline_signature(path);
            current_path_ids.insert(id);

            let cached = self
                .path_outline_cache
                .get(&id)
                .filter(|entry| entry.signature == signature)
                .map(|entry| Rc::clone(&entry.path));
            let path_outline = if let Some(cached) = cached {
                cached
            } else {
                let mut outline = BezPath::new();
                path.append_to_bezpath(&mut outline);
                let outline = Rc::new(outline);
                self.path_outline_cache.insert(
                    id,
                    PathOutlineCacheEntry {
                        signature,
                        path: Rc::clone(&outline),
                    },
                );
                outline
            };
            combined_capacity += path_outline.elements().len();
            path_outlines.push(path_outline);
        }

        self.path_outline_cache
            .retain(|id, _| current_path_ids.contains(id));

        let mut combined = BezPath::with_capacity(combined_capacity);
        for path_outline in &path_outlines {
            append_bezpath(&mut combined, path_outline);
        }

        combined
    }

    /// The background layer and the reference glyph, under the glyph
    /// being edited. The background is an outline and the reference is a
    /// fill, so at a glance the two are never confused with each other
    /// or with what you are drawing.
    fn draw_underlays(&mut self, state: &EditorState, glyph_view: Affine) {
        if let Some(path) = state.reference_outline.as_ref() {
            self.scene.fill(
                Fill::NonZero,
                glyph_view,
                self.theme.reference_glyph,
                None,
                path,
            );
        }
        if let Some(path) = state.background_outline.as_ref() {
            let screen_path = glyph_view * path;
            self.scene.stroke(
                &Stroke::new(self.px(LINE_PX)),
                Affine::IDENTITY,
                self.theme.background_layer,
                None,
                &screen_path,
            );
        }
    }

    /// The outline drawn for a glyph beside the one being edited, resolved
    /// from the parsed glyphs and memoised for the frame.
    ///
    /// The cache is keyed by the glyph revision, which every edit bumps, so a
    /// composite cannot outlive a change to the anchors that place it. The
    /// version this replaced keyed on the address of an SVG string, and a
    /// regenerated string of the same length landing in the same allocation
    /// returned the old path.
    fn text_preview_path(
        &mut self,
        glyph_name: &str,
        state: &EditorState,
        glyphs: &std::collections::HashMap<String, norad::Glyph>,
    ) -> Option<Rc<BezPath>> {
        let revision = state.edit_revision();
        if let Some(entry) = self.text_outline_cache.get(glyph_name) {
            if entry.revision == revision {
                return Some(Rc::clone(&entry.path));
            }
        }
        let path = Rc::new(crate::editor::resolve_glyph_bezpath(glyph_name, glyphs)?);
        self.text_outline_cache.insert(
            glyph_name.to_string(),
            TextOutlineCacheEntry {
                revision,
                path: Rc::clone(&path),
            },
        );
        Some(path)
    }

    fn draw_text_cursor(
        &mut self,
        cursor_x: f64,
        baseline_y: f64,
        ascender: f64,
        descender: f64,
        view: Affine,
        zoom: f64,
    ) {
        let top = view * Point::new(cursor_x, baseline_y + ascender);
        let bottom = view * Point::new(cursor_x, baseline_y + descender);
        let line_width = self.px(self.text_cursor_line_px(zoom));
        let marker_scale = self.text_cursor_marker_scale(zoom, ascender - descender);
        let triangle_width = self.px(TEXT_CURSOR_TRIANGLE_WIDTH_PX * marker_scale);
        let triangle_height = self.px(TEXT_CURSOR_TRIANGLE_HEIGHT_PX * marker_scale);

        self.scene.stroke(
            &Stroke::new(line_width),
            Affine::IDENTITY,
            self.theme.text_cursor,
            None,
            &Line::new(top, bottom),
        );

        let mut top_triangle = BezPath::new();
        top_triangle.move_to((top.x - triangle_width / 2.0, top.y));
        top_triangle.line_to((top.x + triangle_width / 2.0, top.y));
        top_triangle.line_to((top.x, top.y + triangle_height));
        top_triangle.close_path();
        self.scene.fill(
            Fill::NonZero,
            Affine::IDENTITY,
            self.theme.text_cursor,
            None,
            &top_triangle,
        );

        let mut bottom_triangle = BezPath::new();
        bottom_triangle.move_to((bottom.x - triangle_width / 2.0, bottom.y));
        bottom_triangle.line_to((bottom.x + triangle_width / 2.0, bottom.y));
        bottom_triangle.line_to((bottom.x, bottom.y - triangle_height));
        bottom_triangle.close_path();
        self.scene.fill(
            Fill::NonZero,
            Affine::IDENTITY,
            self.theme.text_cursor,
            None,
            &bottom_triangle,
        );
    }

    fn stroke_metric_batch(&mut self, path: &BezPath, color: Srgb, stroke: &Stroke) {
        if !path.elements().is_empty() {
            self.scene
                .stroke(stroke, Affine::IDENTITY, color, None, path);
        }
    }

    fn draw_text_sort_metrics(
        &mut self,
        state: &EditorState,
        x: f64,
        baseline_y: f64,
        advance_width: f64,
        view: Affine,
    ) {
        let (ascender, descender) = state.text_metric_bounds();
        let (box_top, box_bottom) = state.text_sort_metric_bounds();
        let mut metric_path = BezPath::new();
        if box_top > box_bottom {
            append_screen_line(
                &mut metric_path,
                view,
                Point::new(x, baseline_y + box_bottom),
                Point::new(x, baseline_y + box_top),
            );
            append_screen_line(
                &mut metric_path,
                view,
                Point::new(x + advance_width, baseline_y + box_bottom),
                Point::new(x + advance_width, baseline_y + box_top),
            );
        }

        let mut ys = vec![0.0, ascender, descender, box_top, box_bottom];
        if let Some(metrics) = state.metrics.as_ref() {
            ys.extend(
                [metrics.units_per_em, metrics.x_height, metrics.cap_height]
                    .into_iter()
                    .flatten(),
            );
        }
        ys.retain(|y| y.is_finite());
        ys.sort_by(|a, b| a.total_cmp(b));
        ys.dedup_by(|a, b| (*a - *b).abs() < 0.001);
        for y in ys {
            append_screen_line(
                &mut metric_path,
                view,
                Point::new(x, baseline_y + y),
                Point::new(x + advance_width, baseline_y + y),
            );
        }
        self.stroke_metric_batch(
            &metric_path,
            self.theme.metric_guide,
            &Stroke::new(self.px(METRIC_LINE_PX)),
        );
        self.draw_sidebearing_hover(state, x, baseline_y, advance_width, view);
    }

    /// Light up the metric edge under the pointer (or being dragged):
    /// the same vertical line at the same weight as the rest of the
    /// metric box, only recoloured, so the grabbable thing announces
    /// itself without shouting.
    fn draw_sidebearing_hover(
        &mut self,
        state: &EditorState,
        x: f64,
        baseline_y: f64,
        advance_width: f64,
        view: Affine,
    ) {
        let Some(edge) = state.sidebearing_hover else {
            return;
        };
        let (box_top, box_bottom) = state.text_sort_metric_bounds();
        if box_top <= box_bottom {
            return;
        }
        let edge_x = match edge {
            SidebearingEdge::Left => x,
            SidebearingEdge::Right => x + advance_width,
        };
        let mut path = BezPath::new();
        append_screen_line(
            &mut path,
            view,
            Point::new(edge_x, baseline_y + box_bottom),
            Point::new(edge_x, baseline_y + box_top),
        );
        self.scene.stroke(
            &Stroke::new(self.px(METRIC_LINE_PX)),
            Affine::IDENTITY,
            self.theme.text_cursor,
            None,
            &path,
        );
    }

    fn edit_controls_for_path(
        &mut self,
        path_index: usize,
        path: &Path,
        view: Affine,
        selection: &crate::editing::Selection,
        point_scale: f64,
        changed_path_indices: Option<&HashSet<usize>>,
    ) -> Rc<EditControlsGeometry> {
        let id = path_id(path);
        let path_changed = changed_path_indices
            .map(|indices| indices.contains(&path_index))
            .unwrap_or(true);
        if !path_changed {
            let view_coeffs = view.as_coeffs().map(f64::to_bits);
            let point_scale_bits = point_scale.to_bits();
            if let Some(entry) = self.edit_controls_cache.get(&id)
                && entry.key.view_coeffs == view_coeffs
                && entry.key.point_scale_bits == point_scale_bits
            {
                return Rc::clone(&entry.geometry);
            }
        }
        let key = EditControlsCacheKey::new(path, selection, view, point_scale);
        if let Some(entry) = self.edit_controls_cache.get(&id)
            && entry.key == key
        {
            return Rc::clone(&entry.geometry);
        }
        let geometry = Rc::new(Self::build_edit_controls_geometry(
            path,
            view,
            selection,
            point_scale,
        ));
        self.edit_controls_cache.insert(
            id,
            EditControlsCacheEntry {
                key,
                geometry: Rc::clone(&geometry),
            },
        );
        geometry
    }

    fn draw_edit_controls_geometry(
        &mut self,
        controls: &EditControlsGeometry,
        start_arrows: &[StartArrowGeometry],
        point_scale: f64,
    ) {
        // When colorize is on, the measurement pass redraws the outline and
        // handle lines tinted by popcount, so skip the plain gray ones here.
        if !self.measure_options.colorize {
            if !controls.handle_lines.elements().is_empty() {
                self.scene.stroke(
                    &Stroke::new(self.px(HANDLE_LINE_PX) + HALO_PX),
                    Affine::IDENTITY,
                    self.theme.halo,
                    None,
                    &controls.handle_lines,
                );
                self.scene.stroke(
                    &Stroke::new(self.px(HANDLE_LINE_PX)),
                    Affine::IDENTITY,
                    self.theme.handle_line,
                    None,
                    &controls.handle_lines,
                );
            }
            if !controls.outline.elements().is_empty() {
                self.scene.stroke(
                    &Stroke::new(self.px(PATH_STROKE_PX) + HALO_PX),
                    Affine::IDENTITY,
                    self.theme.halo,
                    None,
                    &controls.outline,
                );
                self.scene.stroke(
                    &Stroke::new(self.px(PATH_STROKE_PX)),
                    Affine::IDENTITY,
                    self.theme.path_stroke,
                    None,
                    &controls.outline,
                );
            }
        }
        let outline_stroke = Stroke::new(POINT_OUTLINE_PX * point_scale);
        // An interpolated instance shows its structure but nothing is
        // editable, so every point drops to the same grey.
        let (smooth_inner, smooth_outer) = if self.readonly_points {
            (POINT_READONLY_INNER, POINT_READONLY_OUTER)
        } else {
            (self.theme.point_smooth_inner, self.theme.point_smooth_outer)
        };
        let (corner_inner, corner_outer) = if self.readonly_points {
            (POINT_READONLY_INNER, POINT_READONLY_OUTER)
        } else {
            (self.theme.point_corner_inner, self.theme.point_corner_outer)
        };
        let (offcurve_inner, offcurve_outer) = if self.readonly_points {
            (POINT_READONLY_INNER, POINT_READONLY_OUTER)
        } else {
            (
                self.theme.point_offcurve_inner,
                self.theme.point_offcurve_outer,
            )
        };
        let (hyper_inner, hyper_outer) = if self.readonly_points {
            (POINT_READONLY_INNER, POINT_READONLY_OUTER)
        } else {
            (self.theme.point_hyper_inner, self.theme.point_hyper_outer)
        };
        self.draw_point_batch(
            &controls.smooth_circles,
            smooth_inner,
            smooth_outer,
            &outline_stroke,
        );
        self.draw_point_batch(
            &controls.corner_squares,
            corner_inner,
            corner_outer,
            &outline_stroke,
        );
        self.draw_point_batch(
            &controls.offcurve_circles,
            offcurve_inner,
            offcurve_outer,
            &outline_stroke,
        );
        self.draw_point_batch(
            &controls.hyper_circles,
            hyper_inner,
            hyper_outer,
            &outline_stroke,
        );
        self.draw_point_batch_tinted(
            &controls.selected_circles,
            self.theme.point_selected_inner,
            self.theme.point_selected_outer,
            &outline_stroke,
            Some(self.theme.point_selected_outer),
        );
        self.draw_point_batch_tinted(
            &controls.selected_squares,
            self.theme.point_selected_inner,
            self.theme.point_selected_outer,
            &outline_stroke,
            Some(self.theme.point_selected_outer),
        );
        for start_arrow in start_arrows {
            self.draw_start_arrow(
                start_arrow.center,
                start_arrow.next,
                start_arrow.selected,
                point_scale,
            );
        }
    }

    fn build_edit_controls_geometry(
        path: &Path,
        view: Affine,
        selection: &crate::editing::Selection,
        point_scale: f64,
    ) -> EditControlsGeometry {
        let mut geometry = Self::build_point_geometry(path, view, selection, point_scale);
        geometry.outline = Self::build_outline(path, view);
        geometry.handle_lines = Self::build_handle_lines(path, view);
        geometry
    }

    fn build_outline(path: &Path, view: Affine) -> BezPath {
        let mut outline = BezPath::new();
        path.append_to_bezpath(&mut outline);
        outline.apply_affine(view);
        outline
    }

    fn build_handle_lines(path: &Path, view: Affine) -> BezPath {
        let points = path.points().as_slice();
        if points.len() < 2 {
            return BezPath::new();
        }
        let closed = path_is_closed(path);
        let mut lines = BezPath::new();
        let n = points.len();
        for (i, pt) in points.iter().enumerate() {
            if !pt.is_on_curve() {
                continue;
            }
            let on = view * pt.point;

            // Forward neighbour.
            let next_i = if i + 1 < n {
                Some(i + 1)
            } else if closed {
                Some(0)
            } else {
                None
            };
            if let Some(ni) = next_i
                && points[ni].is_off_curve()
            {
                let off = view * points[ni].point;
                lines.move_to(on);
                lines.line_to(off);
            }

            // Backward neighbour.
            let prev_i = if i > 0 {
                Some(i - 1)
            } else if closed {
                Some(n - 1)
            } else {
                None
            };
            if let Some(pi) = prev_i
                && points[pi].is_off_curve()
            {
                let off = view * points[pi].point;
                lines.move_to(on);
                lines.line_to(off);
            }
        }
        lines
    }

    fn build_point_geometry(
        path: &Path,
        view: Affine,
        selection: &crate::editing::Selection,
        point_scale: f64,
    ) -> EditControlsGeometry {
        let points = path.points().as_slice();
        let closed = path_is_closed(path);
        let start_index = closed
            .then(|| points.iter().position(PathPoint::is_on_curve))
            .flatten();
        let mut smooth_circles = BezPath::new();
        let mut corner_squares = BezPath::new();
        let mut offcurve_circles = BezPath::new();
        let mut hyper_circles = BezPath::new();
        let mut selected_circles = BezPath::new();
        let mut selected_squares = BezPath::new();
        let mut start_arrow = None;
        for (index, pt) in points.iter().enumerate() {
            let center = view * pt.point;
            let selected = selection.contains(&pt.id);

            if matches!(path, Path::Hyper(_)) && pt.is_on_curve() {
                let radius = (if selected {
                    HYPER_POINT_SELECTED_RADIUS_PX
                } else {
                    HYPER_POINT_RADIUS_PX
                }) * point_scale;
                if selected {
                    append_circle_path(&mut selected_circles, center, radius);
                } else {
                    append_circle_path(&mut hyper_circles, center, radius);
                }
            } else {
                match pt.typ {
                    PointType::OnCurve { smooth: true } => {
                        let radius = (if selected {
                            SMOOTH_POINT_SELECTED_RADIUS_PX
                        } else {
                            SMOOTH_POINT_RADIUS_PX
                        }) * point_scale;
                        if selected {
                            append_circle_path(&mut selected_circles, center, radius);
                        } else {
                            append_circle_path(&mut smooth_circles, center, radius);
                        }
                    }
                    PointType::OnCurve { smooth: false } => {
                        let half = (if selected {
                            CORNER_POINT_SELECTED_HALF_PX
                        } else {
                            CORNER_POINT_HALF_PX
                        }) * point_scale;
                        let target = if selected {
                            &mut selected_squares
                        } else {
                            &mut corner_squares
                        };
                        append_rect_path(
                            target,
                            Rect::new(
                                center.x - half,
                                center.y - half,
                                center.x + half,
                                center.y + half,
                            ),
                        );
                    }
                    PointType::OffCurve { .. } => {
                        let radius = (if selected {
                            OFFCURVE_POINT_SELECTED_RADIUS_PX
                        } else {
                            OFFCURVE_POINT_RADIUS_PX
                        }) * point_scale;
                        if selected {
                            append_circle_path(&mut selected_circles, center, radius);
                        } else {
                            append_circle_path(&mut offcurve_circles, center, radius);
                        }
                    }
                }
            }
            if start_index == Some(index) {
                let next = next_point_pos(&points, index, closed);
                start_arrow = Some(StartArrowGeometry {
                    center,
                    next: view * next,
                    selected,
                });
            }
        }
        EditControlsGeometry {
            outline: BezPath::new(),
            handle_lines: BezPath::new(),
            smooth_circles,
            corner_squares,
            offcurve_circles,
            hyper_circles,
            selected_circles,
            selected_squares,
            start_arrow,
        }
    }

    /// The grid inside a point window is tinted to match that point's
    /// own ring, so each point reads as one object.
    fn draw_point_batch(&mut self, path: &BezPath, inner: Srgb, outer: Srgb, stroke: &Stroke) {
        self.draw_point_batch_tinted(path, inner, outer, stroke, Some(outer));
    }

    /// `grid_tint` paints the grid inside the point window in that
    /// colour instead of the canvas grid greys.
    fn draw_point_batch_tinted(
        &mut self,
        path: &BezPath,
        inner: Srgb,
        outer: Srgb,
        stroke: &Stroke,
        grid_tint: Option<Srgb>,
    ) {
        if path.elements().is_empty() {
            return;
        }
        // Dark casing first, so a point on top of the comb keeps its
        // edge.
        self.scene.stroke(
            &Stroke::new(stroke.width + HALO_PX),
            Affine::IDENTITY,
            self.theme.halo,
            None,
            path,
        );
        if self.readonly_points {
            // Fill and stroke one point at a time so overlapping points
            // stack, each masking the one under it, instead of showing
            // both outlines through each other.
            for point in split_subpaths(path) {
                self.scene
                    .fill(Fill::NonZero, Affine::IDENTITY, inner, None, &point);
                self.scene
                    .stroke(stroke, Affine::IDENTITY, outer, None, &point);
            }
            return;
        }
        // The point is a WINDOW: the inner fill masks the outline and
        // handle lines, then the grid is re-stroked clipped to the point
        // interiors, so only the grid shows through (Eli's design).
        self.scene
            .fill(Fill::NonZero, Affine::IDENTITY, inner, None, path);
        // Read-only instance points are plain discs: no grid window,
        // because nothing here is being placed on the grid.
        if let Some(overlay) = self.grid_overlay.clone().filter(|_| !self.readonly_points) {
            self.scene
                .push_layer(Fill::NonZero, Mix::Normal, 1.0, Affine::IDENTITY, path);
            // inside the window the grid must be READABLE, not ambient:
            // full-strength, wider strokes than the canvas grid
            let coarse = grid_tint.unwrap_or(self.theme.design_grid_coarse);
            self.scene.stroke(
                &Stroke::new(1.9),
                Affine::IDENTITY,
                coarse.with_alpha(overlay.accent_alpha),
                None,
                overlay.accent.as_ref(),
            );
            if let Some((fine, fine_alpha)) = &overlay.fine {
                let fine_color = grid_tint.unwrap_or(self.theme.design_grid_fine);
                self.scene.stroke(
                    &Stroke::new(1.3),
                    Affine::IDENTITY,
                    fine_color.with_alpha(*fine_alpha),
                    None,
                    fine.as_ref(),
                );
            }
            self.scene.pop_layer();
        }
        self.scene
            .stroke(stroke, Affine::IDENTITY, outer, None, path);
    }

    fn draw_start_arrow(
        &mut self,
        screen_pos: Point,
        next_screen: Point,
        selected: bool,
        scale: f64,
    ) {
        let arrow_size = (if selected {
            START_NODE_SELECTED_HALF_PX
        } else {
            START_NODE_HALF_PX
        }) * scale;
        let direction = next_screen - screen_pos;
        let len = direction.hypot();
        if len < 0.001 {
            return;
        }
        let forward = direction / len;
        let perpendicular = kurbo::Vec2::new(-forward.y, forward.x);
        let center = screen_pos + perpendicular * (START_NODE_OFFSET_PX * scale);
        let tip = center + forward * arrow_size;
        let base_center = center - forward * (arrow_size * 0.5);
        let base_left = base_center + perpendicular * (arrow_size * 0.5);
        let base_right = base_center - perpendicular * (arrow_size * 0.5);
        let mut arrow = BezPath::new();
        arrow.move_to(tip);
        arrow.line_to(base_left);
        arrow.line_to(base_right);
        arrow.close_path();
        let fill = if selected {
            self.theme.point_selected_outer
        } else {
            self.theme.start_node_outer
        };
        self.scene
            .fill(Fill::NonZero, Affine::IDENTITY, fill, None, &arrow);
    }

    /// Draw the font's metric box: vertical lines at x=0 and
    /// x=advance_width, horizontal lines at each defined metric Y.
    /// Bounded to the glyph's advance-width rectangle so it reads as
    /// "the glyph's space," matching runebender-xilem's
    /// `draw_metrics_guides`.
    fn draw_metric_guides(&mut self, state: &EditorState, view: Affine) {
        let Some(metrics) = state.metrics.as_ref() else {
            return;
        };
        if state.advance_width <= 0.0 {
            return;
        }

        let width = state.advance_width;
        let Some((box_top, box_bottom)) = state.glyph_metric_bounds() else {
            return;
        };
        let mut guide_path = BezPath::new();
        if box_top > box_bottom {
            append_screen_line(
                &mut guide_path,
                view,
                Point::new(0.0, box_bottom),
                Point::new(0.0, box_top),
            );
            append_screen_line(
                &mut guide_path,
                view,
                Point::new(width, box_bottom),
                Point::new(width, box_top),
            );
        }

        // Horizontal metric lines. Baseline is always drawn (y=0);
        // others appear only when defined in fontinfo.
        let mut ys: Vec<f64> = vec![0.0, box_top, box_bottom];
        for opt in [
            metrics.units_per_em,
            metrics.ascender,
            metrics.descender,
            metrics.x_height,
            metrics.cap_height,
        ] {
            if let Some(y) = opt {
                ys.push(y);
            }
        }
        ys.retain(|y| y.is_finite());
        ys.sort_by(|a, b| a.total_cmp(b));
        ys.dedup_by(|a, b| (*a - *b).abs() < 0.001);
        for y in ys {
            append_screen_line(
                &mut guide_path,
                view,
                Point::new(0.0, y),
                Point::new(width, y),
            );
        }
        if !guide_path.elements().is_empty() {
            self.scene.stroke(
                &Stroke::new(self.px(METRIC_LINE_PX)),
                Affine::IDENTITY,
                self.theme.metric_guide,
                None,
                &guide_path,
            );
        }
        self.draw_sidebearing_hover(state, 0.0, 0.0, width, view);
    }

    /// Draw the zoom-dependent design-space grid behind the glyph.
    ///
    /// The mid level shows 8-unit spacing with 32-unit coarse lines;
    /// the close level adds a 2-unit grid with 8-unit coarse lines.
    /// Match xilem's calibration so the coarser grid remains visible
    /// near the default editing zoom, while the dense 2-unit grid waits
    /// until the user is very close in. Anchor both axes to the active
    /// sort origin so the primary horizontal gridline lands on the
    /// font baseline in text mode.
    fn draw_design_grid(
        &mut self,
        state: &EditorState,
        view: Affine,
        origin_x: f64,
        origin_y: f64,
    ) {
        let zoom = state.viewport.zoom;
        // Each level fades in over a zoom octave instead of popping, so
        // the two levels read as one continuous grid: mid (8u) ramps
        // 0.8x->1.6x, close (2u) ramps 8x->16x.
        let mid_alpha = smoothstep(
            ((zoom - DESIGN_GRID_MID_MIN_ZOOM) / DESIGN_GRID_MID_MIN_ZOOM).clamp(0.0, 1.0),
        );
        if mid_alpha <= 0.0 {
            return;
        }

        let top_left = state.viewport.screen_to_design(Point::ZERO);
        let bottom_right = state
            .viewport
            .screen_to_design(Point::new(self.width as f64, self.height as f64));
        let min_x = top_left.x.min(bottom_right.x);
        let max_x = top_left.x.max(bottom_right.x);
        let min_y = top_left.y.min(bottom_right.y);
        let max_y = top_left.y.max(bottom_right.y);

        // The 8-unit lines are ONE grid at every zoom: the mid level draws
        // them in the same style as the close level's 8-unit accent, and
        // the close level only adds the 2s underneath.
        let accent = self.draw_grid_level(
            view,
            DESIGN_GRID_MID_FINE,
            DESIGN_GRID_MID_COARSE_N,
            mid_alpha,
            true,
            true,
            min_x,
            max_x,
            min_y,
            max_y,
            origin_x,
            origin_y,
        );
        self.grid_overlay = Some(GridOverlay {
            accent,
            accent_alpha: mid_alpha as f32,
            fine: None,
        });

        let close_alpha = smoothstep(
            ((zoom - DESIGN_GRID_CLOSE_MIN_ZOOM) / DESIGN_GRID_CLOSE_MIN_ZOOM).clamp(0.0, 1.0),
        );
        if close_alpha > 0.0 {
            let fine = self.draw_grid_level(
                view,
                DESIGN_GRID_CLOSE_FINE,
                DESIGN_GRID_CLOSE_COARSE_N,
                close_alpha,
                false,
                // the 8s already come from the mid pass — do not restroke
                false,
                min_x,
                max_x,
                min_y,
                max_y,
                origin_x,
                origin_y,
            );
            if let Some(overlay) = self.grid_overlay.as_mut() {
                overlay.fine = Some((fine, close_alpha as f32));
            }
        }
    }

    #[allow(clippy::too_many_arguments)]
    #[allow(clippy::too_many_arguments)]
    fn draw_grid_level(
        &mut self,
        view: Affine,
        spacing: f64,
        coarse_n: u32,
        alpha: f64,
        fine_as_accent: bool,
        stroke_coarse: bool,
        min_x: f64,
        max_x: f64,
        min_y: f64,
        max_y: f64,
        origin_x: f64,
        origin_y: f64,
    ) -> Rc<BezPath> {
        let fine_stroke = if fine_as_accent {
            Stroke::new(DESIGN_GRID_COARSE_LINE_PX)
        } else {
            Stroke::new(DESIGN_GRID_FINE_LINE_PX)
        };
        let fine_color = if fine_as_accent {
            self.theme.design_grid_coarse
        } else {
            self.theme.design_grid_fine
        };
        let coarse_stroke = Stroke::new(DESIGN_GRID_COARSE_LINE_PX);
        let (fine_path, coarse_path) = self.design_grid_paths(
            view, spacing, coarse_n, min_x, max_x, min_y, max_y, origin_x, origin_y,
        );

        if !fine_path.elements().is_empty() {
            self.scene.stroke(
                &fine_stroke,
                Affine::IDENTITY,
                fine_color.multiply_alpha(alpha as f32),
                None,
                fine_path.as_ref(),
            );
        }
        if stroke_coarse && !coarse_path.elements().is_empty() {
            self.scene.stroke(
                &coarse_stroke,
                Affine::IDENTITY,
                self.theme.design_grid_coarse.multiply_alpha(alpha as f32),
                None,
                coarse_path.as_ref(),
            );
        }
        fine_path
    }

    #[allow(clippy::too_many_arguments)]
    fn design_grid_paths(
        &mut self,
        view: Affine,
        spacing: f64,
        coarse_n: u32,
        min_x: f64,
        max_x: f64,
        min_y: f64,
        max_y: f64,
        origin_x: f64,
        origin_y: f64,
    ) -> (Rc<BezPath>, Rc<BezPath>) {
        let key = DesignGridCacheKey::new(
            spacing,
            coarse_n,
            self.width,
            self.height,
            view,
            min_x,
            max_x,
            min_y,
            max_y,
            origin_x,
            origin_y,
        );
        if let Some(entry) = self.design_grid_cache.iter().find(|entry| entry.key == key) {
            return (Rc::clone(&entry.fine_path), Rc::clone(&entry.coarse_path));
        }

        let (fine_path, coarse_path) = build_grid_level_paths(
            view, spacing, coarse_n, min_x, max_x, min_y, max_y, origin_x, origin_y,
        );
        let fine_path = Rc::new(fine_path);
        let coarse_path = Rc::new(coarse_path);
        self.design_grid_cache.push(DesignGridCacheEntry {
            key,
            fine_path: Rc::clone(&fine_path),
            coarse_path: Rc::clone(&coarse_path),
        });
        if self.design_grid_cache.len() > 4 {
            self.design_grid_cache.remove(0);
        }
        (fine_path, coarse_path)
    }

    fn draw_marquee(&mut self, rect: kurbo::Rect) {
        // Marquee is already in screen space; draw with identity.
        self.scene.fill(
            Fill::NonZero,
            Affine::IDENTITY,
            self.theme.marquee_fill,
            None,
            &rect,
        );
        self.scene.stroke(
            &Stroke::new(self.px(MARQUEE_STROKE_PX)).with_dashes(0.0, [self.px(4.0), self.px(4.0)]),
            Affine::IDENTITY,
            self.theme.marquee_stroke,
            None,
            &rect,
        );
    }

    fn draw_shape_preview(&mut self, preview: ShapePreview) {
        let stroke = Stroke::new(TOOL_PREVIEW_LINE_PX);
        let rect = match preview {
            ShapePreview::Rectangle(rect) => {
                self.scene.stroke(
                    &stroke,
                    Affine::IDENTITY,
                    self.theme.tool_preview,
                    None,
                    &rect,
                );
                rect
            }
            ShapePreview::Ellipse(rect) => {
                let ellipse = Ellipse::from_rect(rect);
                self.scene.stroke(
                    &stroke,
                    Affine::IDENTITY,
                    self.theme.tool_preview,
                    None,
                    &ellipse,
                );
                rect
            }
        };

        for point in [rect.origin(), rect.origin() + rect.size().to_vec2()] {
            let dot = Circle::new(point, TOOL_PREVIEW_DOT_RADIUS_PX);
            self.scene.fill(
                Fill::NonZero,
                Affine::IDENTITY,
                self.theme.tool_preview,
                None,
                &dot,
            );
        }
    }

    fn draw_segment_hover(&mut self, preview: SegmentHoverPreview) {
        let stroke = Stroke::new(SEGMENT_HOVER_LINE_PX);
        let mut path = BezPath::new();
        match preview {
            SegmentHoverPreview::Line(line) => {
                path.move_to(line.p0);
                path.line_to(line.p1);
            }
            SegmentHoverPreview::Cubic(cubic) => {
                path.move_to(cubic.p0);
                path.curve_to(cubic.p1, cubic.p2, cubic.p3);
            }
            SegmentHoverPreview::Quadratic(quad) => {
                path.move_to(quad.p0);
                path.quad_to(quad.p1, quad.p2);
            }
        }
        self.scene.stroke(
            &stroke,
            Affine::IDENTITY,
            self.theme.tool_preview,
            None,
            &path,
        );
    }

    fn draw_pen_preview(&mut self, preview: PenPreview) {
        // Dashed + device-scaled, matching the knife preview, so the
        // not-yet-committed rubber-band reads as provisional and is no
        // longer the faint hairline it was when it skipped `px()`.
        let stroke = Stroke::new(self.px(TOOL_PREVIEW_LINE_PX))
            .with_dashes(0.0, TOOL_PREVIEW_DASH.map(|dash| self.px(dash)).to_vec());
        let dot_r = self.px(TOOL_PREVIEW_DOT_RADIUS_PX);

        if let Some(start) = preview.line_start {
            let target = preview
                .close_target
                .or(preview.snap_target)
                .unwrap_or(preview.cursor);
            self.scene.stroke(
                &stroke,
                Affine::IDENTITY,
                self.theme.tool_preview,
                None,
                &Line::new(start, target),
            );
        }

        let dot = Circle::new(preview.cursor, dot_r);
        self.scene.fill(
            Fill::NonZero,
            Affine::IDENTITY,
            self.theme.tool_preview,
            None,
            &dot,
        );

        if let Some(close_target) = preview.close_target {
            let close_zone = Circle::new(close_target, dot_r * 2.0);
            self.scene.stroke(
                &stroke,
                Affine::IDENTITY,
                self.theme.tool_preview,
                None,
                &close_zone,
            );
        }
        if let Some(snap_target) = preview.snap_target {
            let snap_zone = Circle::new(snap_target, dot_r * 2.5);
            self.scene.stroke(
                &stroke,
                Affine::IDENTITY,
                self.theme.tool_preview,
                None,
                &snap_zone,
            );
        }
    }

    fn draw_measure_preview(&mut self, preview: &MeasurePreview) {
        // Dashed, device-scaled, tool-preview color — consistent with the
        // pen and knife previews (and the xilem measure tool). Previously
        // this was a solid, un-`px()`'d (half-weight on HiDPI) blue line.
        let stroke = Stroke::new(self.px(TOOL_PREVIEW_LINE_PX))
            .with_dashes(0.0, TOOL_PREVIEW_DASH.map(|dash| self.px(dash)).to_vec());
        self.scene.stroke(
            &stroke,
            Affine::IDENTITY,
            self.theme.tool_preview,
            None,
            &preview.line,
        );

        let dot_r = self.px(TOOL_PREVIEW_DOT_RADIUS_PX);
        for point in [preview.line.p0, preview.line.p1] {
            let dot = Circle::new(point, dot_r);
            self.scene.fill(
                Fill::NonZero,
                Affine::IDENTITY,
                self.theme.tool_preview,
                None,
                &dot,
            );
        }
        for point in &preview.intersections {
            let dot = Circle::new(*point, dot_r * 1.4);
            self.scene.fill(
                Fill::NonZero,
                Affine::IDENTITY,
                self.theme.tool_preview,
                None,
                &dot,
            );
        }
    }

    fn draw_knife_preview(&mut self, preview: &KnifePreview, zoom: f64) {
        let stroke = Stroke::new(self.px(TOOL_PREVIEW_LINE_PX))
            .with_dashes(0.0, TOOL_PREVIEW_DASH.map(|dash| self.px(dash)).to_vec());
        self.scene.stroke(
            &stroke,
            Affine::IDENTITY,
            self.theme.tool_preview,
            None,
            &preview.line,
        );

        let marker_radius = SMOOTH_POINT_RADIUS_PX * self.point_scale(zoom);
        for point in [preview.line.p0, preview.line.p1] {
            let dot = Circle::new(point, marker_radius);
            self.scene.fill(
                Fill::NonZero,
                Affine::IDENTITY,
                POINT_MARK_ORANGE,
                None,
                &dot,
            );
        }

        for point in &preview.intersections {
            let dot = Circle::new(*point, marker_radius);
            self.scene
                .fill(Fill::NonZero, Affine::IDENTITY, POINT_MARK_RED, None, &dot);
        }
    }

    fn present(&mut self) -> Result<(), JsValue> {
        let surface_texture = self
            .surface
            .get_current_texture()
            .map_err(|e| JsValue::from_str(&format!("get_current_texture: {e:?}")))?;

        self.vello
            .render_to_texture(
                &self.device,
                &self.queue,
                &self.scene,
                &self.target_view,
                &vello::RenderParams {
                    base_color: self.theme.bg.into(),
                    width: self.width,
                    height: self.height,
                    antialiasing_method: AaConfig::Area,
                },
            )
            .map_err(|e| JsValue::from_str(&format!("render_to_texture: {e:?}")))?;

        // Vello can't bind the surface as a compute output directly,
        // so it renders into the intermediate `target_texture` and we
        // blit from there to the actual surface.
        let surface_view = surface_texture
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());

        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("runebender blit"),
            });
        self.blitter
            .copy(&self.device, &mut encoder, &self.target_view, &surface_view);
        self.queue.submit([encoder.finish()]);

        surface_texture.present();
        Ok(())
    }
}

fn text_sort_minimal_metric_ys(
    baseline_y: f64,
    ascender: f64,
    descender: f64,
    box_top: f64,
    box_bottom: f64,
) -> Vec<f64> {
    let mut ys = vec![
        baseline_y + box_bottom,
        baseline_y + descender,
        baseline_y,
        baseline_y + ascender,
        baseline_y + box_top,
    ];
    ys.retain(|y| y.is_finite());
    ys.sort_by(|a, b| a.total_cmp(b));
    ys.dedup_by(|a, b| (*a - *b).abs() < 0.001);
    ys
}

/// The full metric box for one sort: the vertical edges at its origin and
/// advance, and a horizontal line at every metric height.
#[allow(clippy::too_many_arguments)]
fn append_text_sort_metric_box(
    path: &mut BezPath,
    x: f64,
    baseline_y: f64,
    advance_width: f64,
    state: &EditorState,
    box_top: f64,
    box_bottom: f64,
    view: Affine,
) {
    if box_top > box_bottom {
        for edge_x in [x, x + advance_width] {
            append_screen_line(
                path,
                view,
                Point::new(edge_x, baseline_y + box_bottom),
                Point::new(edge_x, baseline_y + box_top),
            );
        }
    }
    for y in text_sort_metric_box_ys(state, box_top, box_bottom) {
        append_screen_line(
            path,
            view,
            Point::new(x, baseline_y + y),
            Point::new(x + advance_width, baseline_y + y),
        );
    }
}

/// Metric heights worth a line: baseline, the font's own heights, and the
/// sort box edges.
fn text_sort_metric_box_ys(state: &EditorState, box_top: f64, box_bottom: f64) -> Vec<f64> {
    let (ascender, descender) = state.text_metric_bounds();
    let mut ys = vec![0.0, ascender, descender, box_top, box_bottom];
    if let Some(metrics) = state.metrics.as_ref() {
        ys.extend(
            [metrics.units_per_em, metrics.x_height, metrics.cap_height]
                .into_iter()
                .flatten(),
        );
    }
    ys.retain(|y| y.is_finite());
    ys.sort_by(|a, b| a.total_cmp(b));
    ys.dedup_by(|a, b| (*a - *b).abs() < 0.001);
    ys
}

/// Corner marks: a tick at each metric height on each edge of the sort,
/// clipped to the sort's own box. Arms point inward only — a full cross
/// spilled past the box on every side and read as clutter rather than as
/// the corners of something.
#[allow(clippy::too_many_arguments)]
fn append_text_sort_corner_marks(
    path: &mut BezPath,
    x: f64,
    baseline_y: f64,
    advance_width: f64,
    ascender: f64,
    descender: f64,
    box_top: f64,
    box_bottom: f64,
    view: Affine,
    size: f64,
) {
    let metric_ys =
        text_sort_minimal_metric_ys(baseline_y, ascender, descender, box_top, box_bottom);

    // The box in screen space. The view flips y, so sort the edges rather
    // than assuming which way round they land.
    let corner_a = view * Point::new(x, baseline_y + box_bottom);
    let corner_b = view * Point::new(x + advance_width, baseline_y + box_top);
    let (left, right) = (corner_a.x.min(corner_b.x), corner_a.x.max(corner_b.x));
    let (top, bottom) = (corner_a.y.min(corner_b.y), corner_a.y.max(corner_b.y));

    for edge_x in [x, x + advance_width] {
        for y in metric_ys.iter().copied() {
            let center = view * Point::new(edge_x, y);
            let x0 = (center.x - size).max(left);
            let x1 = (center.x + size).min(right);
            if x1 > x0 {
                path.move_to((x0, center.y));
                path.line_to((x1, center.y));
            }
            let y0 = (center.y - size).max(top);
            let y1 = (center.y + size).min(bottom);
            if y1 > y0 {
                path.move_to((center.x, y0));
                path.line_to((center.x, y1));
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::text_sort_minimal_metric_ys;

    #[test]
    fn minimal_text_metrics_include_upm_top_cross_when_above_ascender() {
        assert_eq!(
            text_sort_minimal_metric_ys(0.0, 700.0, -300.0, 1000.0, -300.0),
            vec![-300.0, 0.0, 700.0, 1000.0]
        );
    }

    #[test]
    fn minimal_text_metrics_deduplicate_upm_top_when_equal_to_ascender() {
        assert_eq!(
            text_sort_minimal_metric_ys(0.0, 800.0, -200.0, 800.0, -200.0),
            vec![-200.0, 0.0, 800.0]
        );
    }
}

fn create_intermediate_target(
    width: u32,
    height: u32,
    device: &wgpu::Device,
) -> (wgpu::Texture, wgpu::TextureView) {
    let target_texture = device.create_texture(&wgpu::TextureDescriptor {
        label: Some("runebender intermediate target"),
        size: wgpu::Extent3d {
            width,
            height,
            depth_or_array_layers: 1,
        },
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        usage: wgpu::TextureUsages::STORAGE_BINDING | wgpu::TextureUsages::TEXTURE_BINDING,
        format: wgpu::TextureFormat::Rgba8Unorm,
        view_formats: &[],
    });
    let target_view = target_texture.create_view(&wgpu::TextureViewDescriptor::default());
    (target_texture, target_view)
}

/// Whether the path is a closed contour (so handle/point wrap-around
/// is allowed). All three Path variants expose a `closed: bool`.
fn path_is_closed(path: &Path) -> bool {
    match path {
        Path::Cubic(c) => c.closed,
        Path::Quadratic(q) => q.closed,
        Path::Hyper(h) => h.closed,
    }
}

fn smoothstep(t: f64) -> f64 {
    t * t * (3.0 - 2.0 * t)
}

fn lerp(a: f64, b: f64, t: f64) -> f64 {
    a + (b - a) * t
}

fn next_point_pos(points: &[PathPoint], index: usize, closed: bool) -> Point {
    if index + 1 < points.len() {
        points[index + 1].point
    } else if closed && !points.is_empty() {
        points[0].point
    } else {
        points[index].point + kurbo::Vec2::new(1.0, 0.0)
    }
}

fn hash_outline_part(mut hash: u64, value: u64) -> u64 {
    hash ^= value;
    hash.wrapping_mul(0x100000001b3)
}

fn path_id(path: &Path) -> EntityId {
    match path {
        Path::Cubic(path) => path.id,
        Path::Quadratic(path) => path.id,
        Path::Hyper(path) => path.id,
    }
}

fn path_outline_signature(path: &Path) -> u64 {
    let (kind, closed) = match path {
        Path::Cubic(path) => (1, path.closed),
        Path::Quadratic(path) => (2, path.closed),
        Path::Hyper(path) => (3, path.closed),
    };
    let points = path.points().as_slice();
    let mut hash = hash_outline_part(0xcbf29ce484222325u64, kind);
    hash = hash_outline_part(hash, points.len() as u64);
    hash = hash_outline_part(hash, u64::from(closed));
    for point in points {
        hash = hash_outline_part(hash, hash_entity_id(point.id));
        hash = hash_outline_part(hash, point.point.x.to_bits());
        hash = hash_outline_part(hash, point.point.y.to_bits());
        hash = hash_outline_part(hash, point_type_signature(point.typ));
    }
    hash
}

fn path_selection_signature(path: &Path, selection: &crate::editing::Selection) -> u64 {
    let points = path.points().as_slice();
    let mut hash = hash_outline_part(0xcbf29ce484222325u64, points.len() as u64);
    for point in points {
        if selection.contains(&point.id) {
            hash = hash_outline_part(hash, hash_entity_id(point.id));
        }
    }
    hash
}

fn hash_entity_id(id: EntityId) -> u64 {
    let mut hasher = DefaultHasher::new();
    id.hash(&mut hasher);
    hasher.finish()
}

fn point_type_signature(point_type: PointType) -> u64 {
    match point_type {
        PointType::OffCurve { auto } => u64::from(auto),
        PointType::OnCurve { smooth } => 0x100 | u64::from(smooth),
    }
}

#[allow(clippy::too_many_arguments)]
fn build_grid_level_paths(
    view: Affine,
    spacing: f64,
    coarse_n: u32,
    min_x: f64,
    max_x: f64,
    min_y: f64,
    max_y: f64,
    origin_x: f64,
    origin_y: f64,
) -> (BezPath, BezPath) {
    let mut fine_path = BezPath::new();
    let mut coarse_path = BezPath::new();
    let start_x = ((min_x - origin_x) / spacing).floor() as i64;
    let end_x = ((max_x - origin_x) / spacing).ceil() as i64;
    let start_y = ((min_y - origin_y) / spacing).floor() as i64;
    let end_y = ((max_y - origin_y) / spacing).ceil() as i64;

    for ix in start_x..=end_x {
        let x = origin_x + ix as f64 * spacing;
        let is_coarse = coarse_n > 0 && (ix.unsigned_abs() % coarse_n as u64 == 0);
        let path = if is_coarse {
            &mut coarse_path
        } else {
            &mut fine_path
        };
        let p0 = view * Point::new(x, min_y);
        let p1 = view * Point::new(x, max_y);
        path.move_to(p0);
        path.line_to(p1);
    }

    for iy in start_y..=end_y {
        let y = origin_y + iy as f64 * spacing;
        let is_coarse = coarse_n > 0 && (iy.unsigned_abs() % coarse_n as u64 == 0);
        let path = if is_coarse {
            &mut coarse_path
        } else {
            &mut fine_path
        };
        let p0 = view * Point::new(min_x, y);
        let p1 = view * Point::new(max_x, y);
        path.move_to(p0);
        path.line_to(p1);
    }

    (fine_path, coarse_path)
}

fn append_rect_path(path: &mut BezPath, rect: Rect) {
    path.move_to((rect.x0, rect.y0));
    path.line_to((rect.x1, rect.y0));
    path.line_to((rect.x1, rect.y1));
    path.line_to((rect.x0, rect.y1));
    path.close_path();
}

fn append_screen_line(path: &mut BezPath, view: Affine, p0: Point, p1: Point) {
    let p0 = view * p0;
    let p1 = view * p1;
    path.move_to(p0);
    path.line_to(p1);
}

fn append_circle_path(path: &mut BezPath, center: Point, radius: f64) {
    const KAPPA: f64 = 0.552_284_749_830_793_6;
    let control = radius * KAPPA;
    path.move_to((center.x + radius, center.y));
    path.curve_to(
        (center.x + radius, center.y + control),
        (center.x + control, center.y + radius),
        (center.x, center.y + radius),
    );
    path.curve_to(
        (center.x - control, center.y + radius),
        (center.x - radius, center.y + control),
        (center.x - radius, center.y),
    );
    path.curve_to(
        (center.x - radius, center.y - control),
        (center.x - control, center.y - radius),
        (center.x, center.y - radius),
    );
    path.curve_to(
        (center.x + control, center.y - radius),
        (center.x + radius, center.y - control),
        (center.x + radius, center.y),
    );
    path.close_path();
}

fn append_bezpath(target: &mut BezPath, source: &BezPath) {
    for element in source.elements() {
        target.push(*element);
    }
}

/// Split a batched path into one `BezPath` per point, so each can be
/// painted separately.
fn split_subpaths(path: &BezPath) -> Vec<BezPath> {
    let mut out: Vec<BezPath> = Vec::new();
    for element in path.elements() {
        if matches!(element, PathEl::MoveTo(_)) {
            out.push(BezPath::new());
        }
        if let Some(current) = out.last_mut() {
            current.push(*element);
        }
    }
    out
}

/// A diamond (rotated square) centred on `center`, `radius` from centre
/// to each tip.
fn diamond_path(center: Point, radius: f64) -> BezPath {
    let mut path = BezPath::new();
    path.move_to(Point::new(center.x, center.y - radius));
    path.line_to(Point::new(center.x + radius, center.y));
    path.line_to(Point::new(center.x, center.y + radius));
    path.line_to(Point::new(center.x - radius, center.y));
    path.close_path();
    path
}
