import assert from "node:assert/strict";
import test from "node:test";

import {
  defaultKerningEntryKeys,
  KERNING_GROUP_PREFIX,
  serializeKerningPlist,
} from "./kerning.ts";

test("UFO group prefixes describe the glyph edge they kern", () => {
  assert.equal(KERNING_GROUP_PREFIX.right, "public.kern1.");
  assert.equal(KERNING_GROUP_PREFIX.left, "public.kern2.");
});

test("a new group pair is serialized in UFO pair order", () => {
  const [first, second] = defaultKerningEntryKeys(
    "a",
    "quoteright",
    { left: "public.kern2.a", right: "public.kern1.a" },
    {
      left: "public.kern2.quoteright",
      right: "public.kern1.quoteright",
    },
  );
  assert.deepEqual([first, second], ["public.kern1.a", "public.kern2.quoteright"]);

  const xml = serializeKerningPlist(new Map([[first, new Map([[second, -96]])]]));
  assert.match(
    xml,
    /<key>public\.kern1\.a<\/key>\s*<dict>\s*<key>public\.kern2\.quoteright<\/key>\s*<real>-96<\/real>/,
  );
  assert.doesNotMatch(xml, /<key>public\.kern2\.a<\/key>/);
});
