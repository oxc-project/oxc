// Property-mangling *behavioral* conformance lives in the Rust tests at
// `crates/oxc_minifier/tests/property_mangler/` (run via `cargo test -p oxc_minifier`);
// this file covers only the NAPI option/cache/error surface plus a couple of thin
// smoke tests that the option-conversion path actually reaches the engine.
import { describe, expect, it } from "vitest";

import { minifySync } from "../index";

describe("property mangling", () => {
  it("is off by default", () => {
    const r = minifySync("t.js", "globalThis.addEventListener()", { mangle: true });
    expect(r.code).toContain("addEventListener");
  });

  it("`mangleProps` renames a matching property", () => {
    // Smoke test: the `mangleProps` regex reaches the engine and renames a member.
    const r = minifySync("t.js", "x._foo", { mangleProps: "^_", compress: false });
    expect(r.code).not.toContain("_foo");
    expect(r.mangleCache).toEqual({ _foo: "e" });
  });

  it("`mangleQuoted: true` mangles a quoted access", () => {
    // Smoke test: the `mangleQuoted` flag reaches the engine; a quoted index that would be
    // reserved by default becomes a mangle candidate.
    const r = minifySync("t.js", "x['_foo']", {
      mangleProps: "^_",
      mangleQuoted: true,
      compress: false,
    });
    expect(r.code).not.toContain("_foo");
    expect(r.mangleCache).toEqual({ _foo: "e" });
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

  it("rejects a mangleCache value that is not a valid identifier", () => {
    // A cache value is written verbatim into an identifier position, so a non-identifier
    // must be rejected up front: `a-b` would emit `x.a-b` (parses as `x.a - b`), and
    // `""` / `a b` are outright invalid syntax.
    for (const bad of ["a-b", "", "a b"]) {
      const r = minifySync("t.js", "x._foo", {
        mangleProps: "^_",
        mangleCache: { _foo: bad },
      });
      expect(r.errors.length).toBe(1);
    }
  });

  it("warns (does not silently skip) when property mangling bails on the whole file", () => {
    // A direct `eval` disables property mangling program-wide. The N-API must surface a
    // warning so a shared-cache multi-file build does not silently ship mismatched names.
    const r = minifySync("t.js", "eval('x'); o._foo;", { mangleProps: "^_" });
    expect(r.errors.some((e) => e.severity === "Warning")).toBe(true);
    // The property was left unmangled.
    expect(r.code).toContain("_foo");
  });
});
