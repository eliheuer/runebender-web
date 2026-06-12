/* tslint:disable */
/* eslint-disable */

export class GlyphEditor {
    private constructor();
    free(): void;
    [Symbol.dispose](): void;
    activateTextSort(index: number): boolean;
    activateTextSortAt(x: number, y: number): boolean;
    activateTextSortAtIndex(x: number, y: number): number;
    /**
     * Activate an inactive text sort hit at screen point and return compact
     * state: `[index, cursor, layout_x, layout_y]`. Empty when no inactive
     * sort was hit, including when the hit sort is already active.
     */
    activateTextSortAtState(x: number, y: number): Float64Array;
    addAnchorAt(x: number, y: number, name: string): boolean;
    /**
     * Advance width of the currently-open glyph (design units).
     * Zero when no glyph is loaded.
     */
    advanceWidth(): number;
    anchorContextAt(x: number, y: number): string;
    clearComponentSelection(): void;
    clearSegmentHover(): boolean;
    clearTextBuffer(): void;
    componentBaseAt(x: number, y: number): string;
    contourContextAt(x: number, y: number): Float64Array;
    /**
     * Number of contours (path elements) in the currently-open
     * glyph. Updates live as the user adds/removes paths.
     */
    contourCount(): number;
    convertHyperToCubic(): boolean;
    copySelection(): boolean;
    /**
     * Serialize the current editable contours back into .glif XML,
     * preserving metadata from `original_bytes` where possible.
     * `mark_color` is the UFO `public.markColor` value; an empty
     * string clears that lib entry.
     */
    currentGlyphGlif(original_bytes: Uint8Array, mark_color: string): Uint8Array;
    /**
     * Move point selection by outline order. `backwards` is Shift-Tab.
     */
    cycleSelectedPoint(backwards: boolean): boolean;
    deleteComponentGlyph(name: string): void;
    deleteSelection(): boolean;
    deleteTextAfterCursor(): boolean;
    deleteTextBeforeCursor(): boolean;
    designToScreen(x: number, y: number): Float64Array;
    duplicateRepeatSelection(): boolean;
    duplicateSelection(): boolean;
    /**
     * Compact glyph metrics state for hot glyph-load paths that already know
     * there is no active selection to preserve.
     *
     * Shape: `[advance_width, contour_count, left_sidebearing, right_sidebearing]`.
     */
    editorMetricsState(): Float64Array;
    /**
     * Compact glyph sidebar + coordinate panel state for hot glyph-load paths.
     *
     * Shape:
     * `[advance_width, contour_count, left_sidebearing, right_sidebearing,
     *   ...selectionState]`.
     */
    editorPanelState(): Float64Array;
    excludeSelection(): boolean;
    finishNudgeSelection(): void;
    /**
     * Auto-zoom and center the loaded glyph for a canvas of the
     * given backing-store size. Called from JS after loading a real
     * glyph so the user doesn't have to hunt for it.
     */
    fitToCanvas(width: number, height: number): void;
    flipSelectionHorizontal(): boolean;
    flipSelectionVertical(): boolean;
    /**
     * Current glyph outline/component bounds as `[x, y, width,
     * height]`, or `[]` when the open glyph has no drawable bounds.
     */
    glyphBounds(): Float64Array;
    insertInactiveTextGlyph(name: string, codepoint: number, advance_width: number): void;
    insertTextCharacter(codepoint: number): boolean;
    insertTextGlyph(name: string, codepoint: number, advance_width: number): void;
    insertTextLineBreak(): void;
    intersectSelection(): boolean;
    leftSidebearing(): number;
    measureInfo(): Float64Array;
    /**
     * Active font vertical metric bounds as `[ascender, descender]`.
     * Empty if fontinfo has not supplied both values.
     */
    metricBounds(): Float64Array;
    moveContour(path_index: number, direction: string): boolean;
    moveSelectionReference(axis: string, value: number): boolean;
    /**
     * Move the coordinate-panel reference point and return the updated
     * compact editor panel state in one JS↔WASM crossing.
     *
     * Shape:
     * `[changed, advance_width, contour_count, left_sidebearing,
     *   right_sidebearing, ...selectionState]`.
     */
    moveSelectionReferenceState(axis: string, value: number): Float64Array;
    moveTextCursorVisualLeft(): void;
    moveTextCursorVisualRight(): void;
    /**
     * Async constructor. Allocates the WebGPU device, attaches to
     * the canvas. Returns a Promise to JS.
     */
    static new(canvas: HTMLCanvasElement, width: number, height: number): Promise<GlyphEditor>;
    nudgeSelection(dx: number, dy: number, shift: boolean, ctrl: boolean, independent: boolean): boolean;
    nudgeSelectionFastState(dx: number, dy: number, shift: boolean, ctrl: boolean, independent: boolean): Float64Array;
    /**
     * Move the current selection and return the updated compact nudge state
     * in the same JS↔WASM crossing.
     *
     * Shape:
     * `[changed, selection_count,
     *   has_bounds, bounds_count, ref_x, ref_y, width, height,
     *   has_anchor, anchor_x, anchor_y]`.
     *
     * Nudging cannot change the selected contour count, so this intentionally
     * skips the contour-membership scan used by the full selection snapshot.
     */
    nudgeSelectionState(dx: number, dy: number, shift: boolean, ctrl: boolean, independent: boolean): Float64Array;
    pasteSelection(): boolean;
    pointerCancel(): boolean;
    pointerDown(x: number, y: number, button: number, mods: number): void;
    pointerMove(x: number, y: number, mods: number): void;
    /**
     * Move the pointer and return compact selection state for hot drag paths
     * that need live coordinate updates without recomputing glyph metrics.
     *
     * Shape:
     * `[0, 0]` when nothing changed; otherwise
     * `[visual_changed, edit_changed, ...selectionState]`.
     */
    pointerMoveSelectionState(x: number, y: number, mods: number): Float64Array;
    /**
     * Move the pointer and report whether anything visible changed.
     *
     * Used by idle hover paths where Vue should not schedule a frame unless
     * the hover/preview state actually changed.
     */
    pointerMoveVisualChanged(x: number, y: number, mods: number): boolean;
    pointerUp(x: number, y: number, button: number, mods: number): boolean;
    redo(): boolean;
    render(): void;
    resize(width: number, height: number): void;
    resizeSelectionReference(axis: string, value: number): boolean;
    /**
     * Resize the selection from the coordinate panel and return the updated
     * compact editor panel state in one JS↔WASM crossing.
     *
     * Shape:
     * `[changed, advance_width, contour_count, left_sidebearing,
     *   right_sidebearing, ...selectionState]`.
     */
    resizeSelectionReferenceState(axis: string, value: number): Float64Array;
    reverseContourAt(x: number, y: number): boolean;
    reverseContours(): boolean;
    rightSidebearing(): number;
    rotateSelectionClockwise(): boolean;
    rotateSelectionCounterClockwise(): boolean;
    roundSelectedCorners(): boolean;
    screenToDesign(x: number, y: number): Float64Array;
    selectAnchorAt(x: number, y: number): boolean;
    selectContourAt(x: number, y: number): boolean;
    selectedAnchorInfo(): string;
    /**
     * Number of contours touched by the current point selection.
     */
    selectedContourCount(): number;
    /**
     * Selected point bounds in design space as
     * `[count, x, y, width, height]`, where x/y are the active
     * coordinate-panel reference point. Empty when there is no
     * selection.
     */
    selectionBounds(): Float64Array;
    /**
     * Number of currently selected entities. Useful for status UI.
     */
    selectionCount(): number;
    /**
     * Compact selection snapshot for hot UI refresh paths.
     *
     * Shape:
     * `[selection_count, selected_contour_count,
     *   has_bounds, bounds_count, ref_x, ref_y, width, height,
     *   has_anchor, anchor_x, anchor_y]`.
     *
     * This intentionally avoids JSON and bundles the common selection
     * panel inputs into one JS↔WASM crossing.
     */
    selectionState(): Float64Array;
    setAdvanceWidth(width: number): boolean;
    setComponentGlyph(name: string, bytes: Uint8Array): void;
    setComponentGlyphs(glyph_xml_by_name: string): void;
    setCoordinateQuadrant(quadrant: string): void;
    setDeviceScale(scale: number): void;
    /**
     * Parse a UFO `fontinfo.plist` and store the vertical metrics
     * (UPM, ascender, descender, x-height, cap-height). The
     * renderer uses these to draw the metric box guidelines.
     */
    setFontInfo(bytes: Uint8Array): void;
    /**
     * Replace the displayed glyph from a UFO `.glif` file's raw
     * bytes. Parses via `norad`, then walks the result into the
     * editor's own contour representation. Clears undo history.
     */
    setGlyphGlif(bytes: Uint8Array): void;
    setGlyphGlifWithCachedComponents(bytes: Uint8Array): void;
    setGlyphGlifWithCachedComponentsPreserveHistory(bytes: Uint8Array): void;
    /**
     * Replace the displayed glyph from a UFO `.glif` file and render
     * resolved component references from a JSON `{ glyphName: glifXml }`
     * map. Component outlines are preview-only for now.
     */
    setGlyphGlifWithComponents(bytes: Uint8Array, glyph_xml_by_name: string): void;
    setGlyphGlifWithComponentsPreserveHistory(bytes: Uint8Array, glyph_xml_by_name: string): void;
    setGlyphNameWithCachedComponents(name: string): boolean;
    setGlyphNameWithCachedComponentsPreserveHistory(name: string): boolean;
    /**
     * Replace the displayed glyph from SVG path data. Each curve
     * segment is decomposed into editable on/off-curve points.
     * Clears undo history (loading a new glyph isn't undoable).
     */
    setGlyphSvg(svg: string): void;
    setKnifeShiftLocked(locked: boolean): boolean;
    setLeftSidebearing(value: number): boolean;
    setOffset(x: number, y: number): void;
    setRightSidebearing(value: number): boolean;
    setShapeShiftLocked(locked: boolean): boolean;
    setShapeTool(shape: string): boolean;
    setStartPointAt(x: number, y: number): boolean;
    setTextDirection(direction: string): void;
    setTextGlyphInventory(json: string): void;
    setTextKerningModel(json: string): void;
    setTheme(theme_json: string): void;
    setTool(tool_id: string): boolean;
    setZoom(zoom: number): void;
    shapeTextBuffer(): boolean;
    subtractSelection(): boolean;
    textBufferLayout(line_height: number): string;
    textBufferPreviewSvg(): string;
    textBufferSnapshot(): string;
    textBufferState(): string;
    textKerningModel(): string;
    textLayoutState(): Float64Array;
    togglePointType(): boolean;
    togglePointTypeAt(x: number, y: number): boolean;
    undo(): boolean;
    unionSelection(): boolean;
    updateSelectedAnchor(name: string, x: number, y: number): boolean;
    updateTextGlyph(index: number, name: string, codepoint: number, advance_width: number): boolean;
    /**
     * Mouse wheel — zoom around the cursor position. `delta_y`
     * follows DOM convention (positive = scroll down = zoom out).
     */
    wheel(x: number, y: number, delta_y: number): void;
    zoom(): number;
}

/**
 * Parse a .glif file's bytes and return an "x-ray" anatomy SVG:
 * outline stroke, control-handle lines, and point markers. Mirrors
 * the xilem anatomy panel closely enough for preview/editing parity.
 */
export function glifAnatomySvg(bytes: Uint8Array): string;

/**
 * Parse a .glif file's bytes and return an anatomy SVG with UFO
 * components resolved against a JSON object of `{ glyphName: glifXml }`.
 */
export function glifAnatomySvgWithComponents(bytes: Uint8Array, glyph_xml_by_name: string): string;

/**
 * Compare an active `.glif` against the same glyph in other masters.
 *
 * `master_glyph_xml_by_name` is JSON shaped as
 * `{ "Bold": "<glyph .../>", "Condensed": null }`; `null` reports a
 * missing glyph for that master. The return value is a JSON array of
 * structured compatibility errors.
 */
export function glifCompatibility(active_bytes: Uint8Array, glyph_name: string, master_glyph_xml_by_name: string): string;

/**
 * Batch-convert every glyph in a master to SVG thumbnails for the
 * grid view. Takes a JSON object `{ glyphName: glifXml }` and returns
 * a JSON object `{ glyphName: svgString }`.
 *
 * Equivalent to calling `glif_to_svg_with_components` once per glyph
 * from JS, but does the work in a single WASM call so we avoid 600+
 * JS↔WASM boundary crossings per master. Profiling showed those
 * crossings, not the actual SVG generation, dominated the edit-to-grid
 * load time (~1.2 s/master in JS, vs ~50 ms in Rust for the same work).
 * Glyphs that fail to parse are silently skipped, mirroring the
 * per-call wrapper's behavior so a single malformed .glif can't sink
 * the whole grid.
 */
export function glifMapToSvgs(glyph_xml_by_name: string, units_per_em: number): string;

/**
 * Parse a .glif file's bytes and return lightweight metadata as
 * JSON. This lets the grid/info sidebar inspect selected glyphs
 * without loading them into the editor or disturbing undo state.
 */
export function glifMetadata(bytes: Uint8Array): string;

/**
 * Parse one .glif file and return a grid-thumbnail SVG with a constant
 * em-based vertical viewBox, resolving components against
 * `{ glyphName: glifXml }`.
 *
 * This is the one-glyph version of `glifMapToSvgs`: edited glyph refreshes
 * should not render every glyph in a master just to update one thumbnail.
 */
export function glifToGridSvgWithComponents(bytes: Uint8Array, glyph_xml_by_name: string, units_per_em: number): string;

/**
 * Parse a .glif file's bytes and return an SVG string fit for an
 * `<img>` or inline render in the glyph grid. Uses the same
 * norad → BezPath path that the live editor uses, then wraps in a
 * viewBox sized to the glyph's own bbox with a Y-flip so UFO's
 * y-up coordinates display correctly.
 */
export function glifToSvg(bytes: Uint8Array): string;

/**
 * Parse a .glif file's bytes and return an SVG with UFO components
 * resolved against a JSON object of `{ glyphName: glifXml }`.
 * This mirrors xilem's grid/preview behavior for composite glyphs.
 */
export function glifToSvgWithComponents(bytes: Uint8Array, glyph_xml_by_name: string): string;

/**
 * Update one UFO kerning group lib entry in a .glif file. `side`
 * accepts `left`/`public.kern1` or `right`/`public.kern2`; an empty
 * group or `-` clears that lib entry, matching xilem's active panel.
 */
export function glifWithKerningGroup(bytes: Uint8Array, side: string, group: string): Uint8Array;

/**
 * Update only the UFO `public.markColor` lib entry in a .glif file.
 * This is used for grid/sidebar mark-color edits that do not load
 * the glyph into the outline editor.
 */
export function glifWithMarkColor(bytes: Uint8Array, mark_color: string): Uint8Array;

/**
 * Update the glyph name in a .glif file while preserving the rest
 * of the glyph data through norad's data model.
 */
export function glifWithName(bytes: Uint8Array, name: string): Uint8Array;

/**
 * Copy only outline data from one `.glif` into another, preserving
 * target glyph identity/metadata. Used by xilem-style grid copy/paste.
 */
export function glifWithOutlinesFrom(source_bytes: Uint8Array, target_bytes: Uint8Array): Uint8Array;

/**
 * Update the first Unicode codepoint in a .glif file. Empty input
 * clears codepoints; otherwise `unicode` accepts `0041`, `U+0041`,
 * or `0x41`.
 */
export function glifWithUnicode(bytes: Uint8Array, unicode: string): Uint8Array;

/**
 * Map a Unicode codepoint to the matching `GlyphCategory`, returned
 * as its `display_name` ("Letter", "Number", …). Uses the same
 * mapping as runebender-xilem (both go through
 * `runebender_core::GlyphCategory`).
 *
 * Returns `"Other"` for codepoints outside the BMP-safe `char`
 * range — the JS side defaults to that anyway for glyphs without
 * a `<unicode>` element.
 */
export function glyphCategoryForCodepoint(cp: number): string;

export function init(): void;

export function traceImageToGlif(image_bytes: Uint8Array, config_json: string): string;

export function traceImageToGlifReport(image_bytes: Uint8Array, config_json: string): string;

export type InitInput = RequestInfo | URL | Response | BufferSource | WebAssembly.Module;

export interface InitOutput {
    readonly memory: WebAssembly.Memory;
    readonly __wbg_glypheditor_free: (a: number, b: number) => void;
    readonly glifAnatomySvg: (a: number, b: number) => [number, number, number, number];
    readonly glifAnatomySvgWithComponents: (a: number, b: number, c: number, d: number) => [number, number, number, number];
    readonly glifCompatibility: (a: number, b: number, c: number, d: number, e: number, f: number) => [number, number, number, number];
    readonly glifMapToSvgs: (a: number, b: number, c: number) => [number, number, number, number];
    readonly glifMetadata: (a: number, b: number) => [number, number, number, number];
    readonly glifToGridSvgWithComponents: (a: number, b: number, c: number, d: number, e: number) => [number, number, number, number];
    readonly glifToSvg: (a: number, b: number) => [number, number, number, number];
    readonly glifToSvgWithComponents: (a: number, b: number, c: number, d: number) => [number, number, number, number];
    readonly glifWithKerningGroup: (a: number, b: number, c: number, d: number, e: number, f: number) => [number, number, number, number];
    readonly glifWithMarkColor: (a: number, b: number, c: number, d: number) => [number, number, number, number];
    readonly glifWithName: (a: number, b: number, c: number, d: number) => [number, number, number, number];
    readonly glifWithOutlinesFrom: (a: number, b: number, c: number, d: number) => [number, number, number, number];
    readonly glifWithUnicode: (a: number, b: number, c: number, d: number) => [number, number, number, number];
    readonly glyphCategoryForCodepoint: (a: number) => [number, number];
    readonly glypheditor_activateTextSort: (a: number, b: number) => number;
    readonly glypheditor_activateTextSortAt: (a: number, b: number, c: number) => number;
    readonly glypheditor_activateTextSortAtIndex: (a: number, b: number, c: number) => number;
    readonly glypheditor_activateTextSortAtState: (a: number, b: number, c: number) => [number, number];
    readonly glypheditor_addAnchorAt: (a: number, b: number, c: number, d: number, e: number) => number;
    readonly glypheditor_advanceWidth: (a: number) => number;
    readonly glypheditor_anchorContextAt: (a: number, b: number, c: number) => [number, number];
    readonly glypheditor_clearComponentSelection: (a: number) => void;
    readonly glypheditor_clearSegmentHover: (a: number) => number;
    readonly glypheditor_clearTextBuffer: (a: number) => void;
    readonly glypheditor_componentBaseAt: (a: number, b: number, c: number) => [number, number];
    readonly glypheditor_contourContextAt: (a: number, b: number, c: number) => [number, number];
    readonly glypheditor_contourCount: (a: number) => number;
    readonly glypheditor_convertHyperToCubic: (a: number) => number;
    readonly glypheditor_copySelection: (a: number) => number;
    readonly glypheditor_currentGlyphGlif: (a: number, b: number, c: number, d: number, e: number) => [number, number, number, number];
    readonly glypheditor_cycleSelectedPoint: (a: number, b: number) => number;
    readonly glypheditor_deleteComponentGlyph: (a: number, b: number, c: number) => void;
    readonly glypheditor_deleteSelection: (a: number) => number;
    readonly glypheditor_deleteTextAfterCursor: (a: number) => number;
    readonly glypheditor_deleteTextBeforeCursor: (a: number) => number;
    readonly glypheditor_designToScreen: (a: number, b: number, c: number) => [number, number];
    readonly glypheditor_duplicateRepeatSelection: (a: number) => number;
    readonly glypheditor_duplicateSelection: (a: number) => number;
    readonly glypheditor_editorMetricsState: (a: number) => [number, number];
    readonly glypheditor_editorPanelState: (a: number) => [number, number];
    readonly glypheditor_excludeSelection: (a: number) => number;
    readonly glypheditor_finishNudgeSelection: (a: number) => void;
    readonly glypheditor_fitToCanvas: (a: number, b: number, c: number) => void;
    readonly glypheditor_flipSelectionHorizontal: (a: number) => number;
    readonly glypheditor_flipSelectionVertical: (a: number) => number;
    readonly glypheditor_glyphBounds: (a: number) => [number, number];
    readonly glypheditor_insertInactiveTextGlyph: (a: number, b: number, c: number, d: number, e: number) => void;
    readonly glypheditor_insertTextCharacter: (a: number, b: number) => number;
    readonly glypheditor_insertTextGlyph: (a: number, b: number, c: number, d: number, e: number) => void;
    readonly glypheditor_insertTextLineBreak: (a: number) => void;
    readonly glypheditor_intersectSelection: (a: number) => number;
    readonly glypheditor_leftSidebearing: (a: number) => number;
    readonly glypheditor_measureInfo: (a: number) => [number, number];
    readonly glypheditor_metricBounds: (a: number) => [number, number];
    readonly glypheditor_moveContour: (a: number, b: number, c: number, d: number) => number;
    readonly glypheditor_moveSelectionReference: (a: number, b: number, c: number, d: number) => number;
    readonly glypheditor_moveSelectionReferenceState: (a: number, b: number, c: number, d: number) => [number, number];
    readonly glypheditor_moveTextCursorVisualLeft: (a: number) => void;
    readonly glypheditor_moveTextCursorVisualRight: (a: number) => void;
    readonly glypheditor_new: (a: any, b: number, c: number) => any;
    readonly glypheditor_nudgeSelection: (a: number, b: number, c: number, d: number, e: number, f: number) => number;
    readonly glypheditor_nudgeSelectionFastState: (a: number, b: number, c: number, d: number, e: number, f: number) => [number, number];
    readonly glypheditor_nudgeSelectionState: (a: number, b: number, c: number, d: number, e: number, f: number) => [number, number];
    readonly glypheditor_pasteSelection: (a: number) => number;
    readonly glypheditor_pointerCancel: (a: number) => number;
    readonly glypheditor_pointerDown: (a: number, b: number, c: number, d: number, e: number) => void;
    readonly glypheditor_pointerMove: (a: number, b: number, c: number, d: number) => void;
    readonly glypheditor_pointerMoveSelectionState: (a: number, b: number, c: number, d: number) => [number, number];
    readonly glypheditor_pointerMoveVisualChanged: (a: number, b: number, c: number, d: number) => number;
    readonly glypheditor_pointerUp: (a: number, b: number, c: number, d: number, e: number) => number;
    readonly glypheditor_redo: (a: number) => number;
    readonly glypheditor_render: (a: number) => [number, number];
    readonly glypheditor_resize: (a: number, b: number, c: number) => void;
    readonly glypheditor_resizeSelectionReference: (a: number, b: number, c: number, d: number) => number;
    readonly glypheditor_resizeSelectionReferenceState: (a: number, b: number, c: number, d: number) => [number, number];
    readonly glypheditor_reverseContourAt: (a: number, b: number, c: number) => number;
    readonly glypheditor_reverseContours: (a: number) => number;
    readonly glypheditor_rightSidebearing: (a: number) => number;
    readonly glypheditor_rotateSelectionClockwise: (a: number) => number;
    readonly glypheditor_rotateSelectionCounterClockwise: (a: number) => number;
    readonly glypheditor_roundSelectedCorners: (a: number) => number;
    readonly glypheditor_screenToDesign: (a: number, b: number, c: number) => [number, number];
    readonly glypheditor_selectAnchorAt: (a: number, b: number, c: number) => number;
    readonly glypheditor_selectContourAt: (a: number, b: number, c: number) => number;
    readonly glypheditor_selectedAnchorInfo: (a: number) => [number, number];
    readonly glypheditor_selectedContourCount: (a: number) => number;
    readonly glypheditor_selectionBounds: (a: number) => [number, number];
    readonly glypheditor_selectionCount: (a: number) => number;
    readonly glypheditor_selectionState: (a: number) => [number, number];
    readonly glypheditor_setAdvanceWidth: (a: number, b: number) => number;
    readonly glypheditor_setComponentGlyph: (a: number, b: number, c: number, d: number, e: number) => [number, number];
    readonly glypheditor_setComponentGlyphs: (a: number, b: number, c: number) => [number, number];
    readonly glypheditor_setCoordinateQuadrant: (a: number, b: number, c: number) => void;
    readonly glypheditor_setDeviceScale: (a: number, b: number) => void;
    readonly glypheditor_setFontInfo: (a: number, b: number, c: number) => [number, number];
    readonly glypheditor_setGlyphGlif: (a: number, b: number, c: number) => [number, number];
    readonly glypheditor_setGlyphGlifWithCachedComponents: (a: number, b: number, c: number) => [number, number];
    readonly glypheditor_setGlyphGlifWithCachedComponentsPreserveHistory: (a: number, b: number, c: number) => [number, number];
    readonly glypheditor_setGlyphGlifWithComponents: (a: number, b: number, c: number, d: number, e: number) => [number, number];
    readonly glypheditor_setGlyphGlifWithComponentsPreserveHistory: (a: number, b: number, c: number, d: number, e: number) => [number, number];
    readonly glypheditor_setGlyphNameWithCachedComponents: (a: number, b: number, c: number) => [number, number, number];
    readonly glypheditor_setGlyphNameWithCachedComponentsPreserveHistory: (a: number, b: number, c: number) => [number, number, number];
    readonly glypheditor_setGlyphSvg: (a: number, b: number, c: number) => [number, number];
    readonly glypheditor_setKnifeShiftLocked: (a: number, b: number) => number;
    readonly glypheditor_setLeftSidebearing: (a: number, b: number) => number;
    readonly glypheditor_setOffset: (a: number, b: number, c: number) => void;
    readonly glypheditor_setRightSidebearing: (a: number, b: number) => number;
    readonly glypheditor_setShapeShiftLocked: (a: number, b: number) => number;
    readonly glypheditor_setShapeTool: (a: number, b: number, c: number) => number;
    readonly glypheditor_setStartPointAt: (a: number, b: number, c: number) => number;
    readonly glypheditor_setTextDirection: (a: number, b: number, c: number) => void;
    readonly glypheditor_setTextGlyphInventory: (a: number, b: number, c: number) => [number, number];
    readonly glypheditor_setTextKerningModel: (a: number, b: number, c: number) => [number, number];
    readonly glypheditor_setTheme: (a: number, b: number, c: number) => [number, number];
    readonly glypheditor_setTool: (a: number, b: number, c: number) => number;
    readonly glypheditor_setZoom: (a: number, b: number) => void;
    readonly glypheditor_shapeTextBuffer: (a: number) => number;
    readonly glypheditor_subtractSelection: (a: number) => number;
    readonly glypheditor_textBufferLayout: (a: number, b: number) => [number, number, number, number];
    readonly glypheditor_textBufferPreviewSvg: (a: number) => [number, number, number, number];
    readonly glypheditor_textBufferSnapshot: (a: number) => [number, number, number, number];
    readonly glypheditor_textBufferState: (a: number) => [number, number, number, number];
    readonly glypheditor_textKerningModel: (a: number) => [number, number, number, number];
    readonly glypheditor_textLayoutState: (a: number) => [number, number];
    readonly glypheditor_togglePointType: (a: number) => number;
    readonly glypheditor_togglePointTypeAt: (a: number, b: number, c: number) => number;
    readonly glypheditor_undo: (a: number) => number;
    readonly glypheditor_unionSelection: (a: number) => number;
    readonly glypheditor_updateSelectedAnchor: (a: number, b: number, c: number, d: number, e: number) => number;
    readonly glypheditor_updateTextGlyph: (a: number, b: number, c: number, d: number, e: number, f: number) => number;
    readonly glypheditor_wheel: (a: number, b: number, c: number, d: number) => void;
    readonly glypheditor_zoom: (a: number) => number;
    readonly traceImageToGlif: (a: number, b: number, c: number, d: number) => [number, number, number, number];
    readonly traceImageToGlifReport: (a: number, b: number, c: number, d: number) => [number, number, number, number];
    readonly init: () => void;
    readonly wasm_bindgen__convert__closures_____invoke__h5aec8594ecc89ac6: (a: number, b: number, c: any) => [number, number];
    readonly wasm_bindgen__convert__closures_____invoke__h34113d0ef8e9a838: (a: number, b: number, c: any, d: any) => void;
    readonly wasm_bindgen__convert__closures_____invoke__h42b8381306a6a6c3: (a: number, b: number, c: any) => void;
    readonly __wbindgen_malloc: (a: number, b: number) => number;
    readonly __wbindgen_realloc: (a: number, b: number, c: number, d: number) => number;
    readonly __wbindgen_exn_store: (a: number) => void;
    readonly __externref_table_alloc: () => number;
    readonly __wbindgen_externrefs: WebAssembly.Table;
    readonly __wbindgen_free: (a: number, b: number, c: number) => void;
    readonly __wbindgen_destroy_closure: (a: number, b: number) => void;
    readonly __externref_table_dealloc: (a: number) => void;
    readonly __wbindgen_start: () => void;
}

export type SyncInitInput = BufferSource | WebAssembly.Module;

/**
 * Instantiates the given `module`, which can either be bytes or
 * a precompiled `WebAssembly.Module`.
 *
 * @param {{ module: SyncInitInput }} module - Passing `SyncInitInput` directly is deprecated.
 *
 * @returns {InitOutput}
 */
export function initSync(module: { module: SyncInitInput } | SyncInitInput): InitOutput;

/**
 * If `module_or_path` is {RequestInfo} or {URL}, makes a request and
 * for everything else, calls `WebAssembly.instantiate` directly.
 *
 * @param {{ module_or_path: InitInput | Promise<InitInput> }} module_or_path - Passing `InitInput` directly is deprecated.
 *
 * @returns {Promise<InitOutput>}
 */
export default function __wbg_init (module_or_path?: { module_or_path: InitInput | Promise<InitInput> } | InitInput | Promise<InitInput>): Promise<InitOutput>;
