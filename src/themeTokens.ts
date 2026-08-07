// Mark colours, for the swatch rows and for what goes into a UFO's
// `public.markColor`. They come from the generated theme so there is one
// palette, not two: edit themes/runebender.theme.json and run
// `pnpm run theme`.
//
// A glyph already marked in an older file keeps the exact rgba stored in
// its UFO — the grid draws what the file says, not what the palette
// says. Only newly applied marks use these.

export { THEME_MARK_COLORS } from "./themeTokens.generated";
