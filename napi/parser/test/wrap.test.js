import { describe, expect, it } from "vitest";

import { parseSync as parseSyncBinding } from "../src-js/bindings.js";
import { wrap } from "../src-js/wrap.js";

// Written in JS, not TS, because `bindings.js` has no type declarations.
describe("wrap", () => {
  it("takes every field out of the native `ParseResult` eagerly", () => {
    const code = "import x from 'y'; /* c */ let a; let a;";
    const native = parseSyncBinding("test.js", code, { showSemanticErrors: true });
    const result = wrap(native);

    // Native getters `mem::take` their field, so the native side is now empty,
    // even though no field of `result` has been read yet
    expect(native.program).toBe("");
    expect(native.module.staticImports).toEqual([]);
    expect(native.comments).toEqual([]);
    expect(native.errors).toEqual([]);

    expect(result.program.body).toHaveLength(3);
    expect(result.module.staticImports).toHaveLength(1);
    expect(result.comments).toHaveLength(1);
    expect(result.errors).toHaveLength(1);
  });

  it("returns the same `program` object on every access", () => {
    const result = wrap(parseSyncBinding("test.js", "let a;", {}));
    expect(result.program).toBe(result.program);
  });
});
