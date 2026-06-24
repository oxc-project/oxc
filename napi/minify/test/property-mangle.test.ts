import { describe, expect, it } from "vitest";

import { minifySync } from "../index";

describe("property mangling", () => {
  it("is off by default", () => {
    const r = minifySync("t.js", "globalThis.addEventListener()", { mangle: true });
    expect(r.code).toContain("addEventListener");
  });

  it("renames matching properties", () => {
    // base54 names start at `e`, not `a`. `compress: false` so the pure
    // expression is not DCE'd away.
    const r = minifySync("t.js", "x._foo", { mangleProps: "^_", compress: false });
    expect(r.code).toContain("x.e");
    expect(r.code).not.toContain("x._foo");
    expect(r.code).not.toContain("x.a");
  });

  it("reserveProps carves out a subset", () => {
    const r = minifySync("t.js", "x._keep; x._foo;", {
      mangleProps: "^_",
      reserveProps: "_keep$",
      compress: false,
    });
    expect(r.code).toContain("_keep");
    expect(r.code).not.toContain("_foo");
  });

  it("returns a cache and reuses it for stable names", () => {
    const a = minifySync("t.js", "x._foo", { mangleProps: "^_", mangleCache: {}, compress: false });
    expect(a.mangleCache).toBeTruthy();
    const b = minifySync("t.js", "x._foo", {
      mangleProps: "^_",
      mangleCache: a.mangleCache!,
      compress: false,
    });
    expect(b.code).toBe(a.code);
  });

  it("`false` in the cache reserves a property", () => {
    const r = minifySync("t.js", "x._foo", {
      mangleProps: "^_",
      mangleCache: { _foo: false },
      compress: false,
    });
    expect(r.code).toContain("_foo");
  });

  it("reservedProps (literal list) carves out a name", () => {
    // `reservedProps` is the explicit literal list (vs `reserveProps`, a regex).
    const r = minifySync("t.js", "o._keep; o._foo;", {
      mangleProps: "^_",
      reservedProps: ["_keep"],
      compress: false,
    });
    expect(r.code).toContain("_keep");
    expect(r.code).not.toContain("_foo");
  });

  it("a quoted access reserves the unquoted property", () => {
    // `o['_foo']` is a quoted access, which reserves `_foo` program-wide, so the
    // unquoted `o._foo` must survive. `compress: false` isolates the property pass
    // (compress would otherwise un-quote `o['_foo']`).
    const r = minifySync("t.js", "o['_foo']; o._foo;", {
      mangleProps: "^_",
      compress: false,
    });
    expect(r.code).toContain("_foo");
    expect(r.code).not.toContain("o.e");
  });

  it("rejects an invalid regex", () => {
    const r = minifySync("t.js", "x._foo", { mangleProps: "(" });
    expect(r.errors.length).toBe(1);
  });

  it("rejects a `__proto__` cache key", () => {
    // Use a computed key so `__proto__` is a real own property (a plain
    // `{ __proto__: ... }` literal sets the prototype instead of a key).
    const r = minifySync("t.js", "x._foo", {
      mangleProps: "^_",
      mangleCache: { ["__proto__"]: "e" } as any,
    });
    expect(r.errors.length).toBe(1);
  });
});
