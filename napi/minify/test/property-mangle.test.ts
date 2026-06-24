// Property-mangling *behavioral* conformance lives in `esbuild-mangle-props.test.ts`;
// this file covers the NAPI option/cache/error surface.
import { describe, expect, it } from "vitest";

import { minifySync } from "../index";

describe("property mangling", () => {
  it("is off by default", () => {
    const r = minifySync("t.js", "globalThis.addEventListener()", { mangle: true });
    expect(r.code).toContain("addEventListener");
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
