// Property-mangling *behavioral* conformance lives in the Rust tests at
// `crates/oxc_minifier/tests/mangler/property/` (run via `cargo test -p oxc_minifier`);
// this file covers only the NAPI option/error surface plus a couple of thin
// smoke tests that the option-conversion path actually reaches the engine.
import { describe, expect, it } from "vitest";

import { minifySync } from "../index";

describe("property mangling", () => {
  it("is off by default", () => {
    const r = minifySync("t.js", "globalThis.addEventListener()", { mangle: true });
    expect(r.code).toContain("addEventListener");
  });

  it("`mangleProps.regex` renames a matching property", () => {
    // Smoke test: the `mangleProps` object reaches the engine and renames a member.
    const r = minifySync("t.js", "x._foo", { mangleProps: { regex: "^_" }, compress: false });
    expect(r.code).not.toContain("_foo");
  });

  it("`mangleQuoted: true` mangles a quoted access", () => {
    // Smoke test: the `mangleQuoted` flag reaches the engine; a quoted index that would be
    // reserved by default becomes a mangle candidate.
    const r = minifySync("t.js", "x['_foo']", {
      mangleProps: { regex: "^_", mangleQuoted: true },
      compress: false,
    });
    expect(r.code).not.toContain("_foo");
  });

  it("`exclude` (regex) carves names out of `regex`", () => {
    const r = minifySync("t.js", "o._public; o._foo;", {
      mangleProps: { regex: "^_", exclude: "^_public" },
      compress: false,
    });
    expect(r.code).toContain("_public");
    expect(r.code).not.toContain("_foo");
  });

  it("`reserved` (literal list) carves out a name", () => {
    // `reserved` is the explicit literal list (vs `exclude`, a regex).
    const r = minifySync("t.js", "o._keep; o._foo;", {
      mangleProps: { regex: "^_", reserved: ["_keep"] },
      compress: false,
    });
    expect(r.code).toContain("_keep");
    expect(r.code).not.toContain("_foo");
  });

  it("rejects an invalid `regex`", () => {
    const r = minifySync("t.js", "x._foo", { mangleProps: { regex: "(" } });
    expect(r.errors.length).toBe(1);
  });

  it("rejects an invalid `exclude` regex", () => {
    // Both regexes are validated whenever the `mangleProps` object is present.
    const r = minifySync("t.js", "x._foo", { mangleProps: { regex: "^_", exclude: "(" } });
    expect(r.errors.length).toBe(1);
  });

  it("warns (does not silently skip) when property mangling bails on the whole file", () => {
    // A direct `eval` disables property mangling program-wide. The N-API must surface a
    // warning so a shared-cache multi-file build does not silently ship mismatched names.
    const r = minifySync("t.js", "eval('x'); o._foo;", { mangleProps: { regex: "^_" } });
    expect(r.errors.some((e) => e.severity === "Warning")).toBe(true);
    // The property was left unmangled.
    expect(r.code).toContain("_foo");
  });
});
