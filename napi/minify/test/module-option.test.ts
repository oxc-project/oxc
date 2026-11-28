import { runInNewContext } from "node:vm";
import { describe, expect, it } from "vitest";

import { minifySync } from "../index";

// Regression tests for https://github.com/oxc-project/oxc/issues/30440
// `module: false` must parse the source as a classic script, so block-level
// function declarations keep Annex B hoisting (the mangler must not reuse an
// outer `var`'s name for one) and a leading `"use strict"` is preserved,
// even when the file name is a label without a JS extension.

const annexB = `(function () {
  var log = [], bank = { kind: "bank" }, click;
  if (bank) { function draw() { log.push("draw " + bank.kind); } click = function () { draw(); }; }
  var imgs = { kind: "imgs" };
  if (imgs) { function set(v) { log.push("set " + v); } set(imgs.kind); }
  click();
  result = log.join(", ");
})();`;

function evaluate(source: string) {
  const context: { result?: unknown } = { result: undefined };
  runInNewContext(source, context, { timeout: 1000 });
  return context.result;
}

describe("module: false with file names without a JS extension", () => {
  it("preserves Annex B function hoisting behavior", () => {
    const expected = evaluate(annexB);
    expect(expected).toBe("set imgs, draw bank");
    for (const filename of ["inline", "page.html inline <script>", "inline.js"]) {
      const ret = minifySync(filename, annexB, { module: false });
      expect(ret.errors).toEqual([]);
      expect(evaluate(ret.code)).toBe(expected);
    }
  });

  it('preserves a leading "use strict"', () => {
    const source = `"use strict";\ntry { undeclared = 1; result = "assigned"; } catch { result = "threw"; }`;
    const expected = evaluate(source);
    expect(expected).toBe("threw");
    const ret = minifySync("inline", source, { module: false });
    expect(ret.errors).toEqual([]);
    expect(evaluate(ret.code)).toBe(expected);
  });
});
