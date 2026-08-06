// Dev-only convenience: auto-load a UFO sitting at
// `web/assets/test-fonts/<MyFont>.ufo/` so reloading the page
// doesn't mean re-dragging the font in every time.
//
// Imported behind the dev-mode auto-load path, and by standalone
// website builds that explicitly opt in with
// VITE_RUNEBENDER_LOAD_TEST_FONT=1. ComfyUI extension builds keep the
// module out of the production bundle.

const FILES = import.meta.glob(
  // features.fea rides along: shaping compiles it for harfrust, which is
  // where ligatures like lam-alef come from.
  "../assets/test-fonts/**/*.{glif,plist,designspace,fea}",
  { eager: true, query: "?url", import: "default" },
) as Record<string, string>;

/**
 * Fetch every file in `web/assets/test-fonts/` and return them as
 * `File` objects with `webkitRelativePath` set, in the same shape
 * `loadGlifFiles` already accepts from drag-drop + directory picker.
 *
 * Returns an empty array if the test-fonts directory is empty.
 */
export async function readDevTestFontFiles(): Promise<File[]> {
  const entries = Object.entries(FILES);
  if (entries.length === 0) return [];

  // ~1000 small fetches in one burst: a single transient failure must
  // not take down the whole demo load (it used to — the editor fell
  // back to the welcome screen). Retry each file once, then skip it.
  const fetchBlob = async (url: string) => {
    try {
      const res = await fetch(url);
      if (res.ok) return res.blob();
    } catch {}
    const retry = await fetch(url);
    if (!retry.ok) throw new Error(`fetch failed: ${url}`);
    return retry.blob();
  };

  const settled = await Promise.allSettled(
    entries.map(async ([sourcePath, url]) => {
      const blob = await fetchBlob(url);
      // The source path looks like one of:
      //   "../assets/test-fonts/VirtuaGrotesk-Regular.ufo/glyphs/A_.glif"
      //   "../assets/test-fonts/VirtuaGrotesk.designspace"
      // For UFO files we want the relative path starting at the .ufo
      // segment; for the .designspace we keep just its filename. Either
      // way, the existing /glyphs/ filter and the new .designspace
      // detection in loadGlifFiles see the same shape as a real drop.
      const ufoMatch = sourcePath.match(/([^/]+\.ufo\/.*)$/);
      const rel = ufoMatch ? ufoMatch[1] : sourcePath.split("/").pop()!;
      const fileName = rel.split("/").pop()!;
      const file = new File([blob], fileName);
      try {
        Object.defineProperty(file, "webkitRelativePath", {
          value: rel,
          configurable: true,
        });
      } catch {
        // Some browsers refuse to override the prop; the
        // "any .glif fallback" filter still catches files.
      }
      return file;
    }),
  );
  const files = settled
    .filter((r): r is PromiseFulfilledResult<File> => r.status === "fulfilled")
    .map((r) => r.value);
  const failed = settled.length - files.length;
  if (failed > 0) {
    console.warn(`[runebender] demo font: ${failed} file(s) failed to load`);
  }
  return files;
}
