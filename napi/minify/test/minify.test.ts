import { Worker } from "node:worker_threads";
import { describe, expect, it } from "vitest";

import { minify, minifySync } from "../index";

describe("simple", () => {
  const code = "/*! legal comment */\nfunction foo() { var bar; bar(undefined) } foo();";

  it("matches output", () => {
    const ret = minifySync("test.js", code, { sourcemap: true });
    expect(ret.code).toEqual("function foo(){var e;e(void 0)}foo();");
    expect(ret.errors.length).toBe(0);
    expect(ret.map).toMatchObject({
      names: ["bar"],
      sources: ["test.js"],
      sourcesContent: [code],
      version: 3,
    });
  });

  it("can turn off everything", () => {
    const ret = minifySync("test.js", code, {
      compress: false,
      mangle: false,
      codegen: { removeWhitespace: false },
    });
    expect(ret.code).toBe("function foo() {\n\tvar bar;\n\tbar(undefined);\n}\nfoo();\n");
  });

  it("defaults to esnext", () => {
    const code = "try { foo } catch (e) {}";
    const ret = minifySync("test.js", code);
    expect(ret.code).toBe("try{foo}catch{}");
  });

  it("returns parser error", () => {
    const code = "const";
    const ret = minifySync("test.js", code);
    expect(ret.code).toBe("");
    expect(ret.errors.length).toBe(1);
  });

  it("supports drop_labels option", () => {
    const code = "PURE: { foo(); bar(); } OTHER: { baz(); }";
    const ret = minifySync("test.js", code, {
      compress: { dropLabels: ["PURE"] },
    });
    expect(ret.code).toBe("OTHER:baz();");
    expect(ret.errors.length).toBe(0);
  });
});

describe("treeshake options", () => {
  it("respects annotations by default", () => {
    const code = "/* @__PURE__ */ foo(); bar();";
    const ret = minifySync("test.js", code, {
      compress: {},
    });
    // The @__PURE__ annotated call should be removed
    expect(ret.code).toBe("bar();");
    expect(ret.errors.length).toBe(0);
  });

  it("can disable annotations", () => {
    const code = "/* @__PURE__ */ foo(); bar();";
    const ret = minifySync("test.js", code, {
      compress: {
        treeshake: {
          annotations: false,
        },
      },
    });
    // With annotations disabled, @__PURE__ is not respected
    expect(ret.code).toBe("foo(),bar();");
    expect(ret.errors.length).toBe(0);
  });

  it("supports manual pure functions", () => {
    const code = "foo(); bar(); baz();";
    const ret = minifySync("test.js", code, {
      compress: {
        treeshake: {
          manualPureFunctions: ["foo", "baz"],
        },
      },
    });
    // foo and baz should be removed as they're marked as pure, bar should remain
    expect(ret.code).toBe("bar();");
    expect(ret.errors.length).toBe(0);
  });

  it("supports propertyReadSideEffects as boolean", () => {
    const code = "const x = obj.prop; foo();";
    const ret = minifySync("test.js", code, {
      compress: {
        treeshake: {
          propertyReadSideEffects: false,
        },
      },
    });
    expect(ret.errors.length).toBe(0);
  });

  it('supports propertyReadSideEffects as "always"', () => {
    const code = "const x = obj.prop; foo();";
    const ret = minifySync("test.js", code, {
      compress: {
        treeshake: {
          propertyReadSideEffects: "always",
        },
      },
    });
    expect(ret.errors.length).toBe(0);
  });

  it("rejects invalid propertyReadSideEffects string value", () => {
    const code = "const x = obj.prop; foo();";
    const ret = minifySync("test.js", code, {
      compress: {
        treeshake: {
          propertyReadSideEffects: "invalid" as any,
        },
      },
    });
    expect(ret.errors.length).toBe(1);
    expect(ret.errors[0].message).toContain("Invalid propertyReadSideEffects value");
  });
});

describe("async minify", () => {
  const code = "/*! legal comment */\nfunction foo() { var bar; bar(undefined) } foo();";

  it("produces same result as sync", async () => {
    const syncResult = minifySync("test.js", code, { sourcemap: true });
    const asyncResult = await minify("test.js", code, { sourcemap: true });

    expect(asyncResult.code).toEqual(syncResult.code);
    expect(asyncResult.errors.length).toBe(syncResult.errors.length);
    expect(asyncResult.map).toMatchObject(syncResult.map!);
  });
});

describe("property mangling", () => {
  it("is independent from identifier mangling and returns a reusable cache", () => {
    const ret = minifySync("test.js", "let local; obj._often; obj._rare; obj._often;", {
      compress: false,
      mangle: false,
      mangleProps: { include: /^_/ },
    });

    expect(ret.errors).toHaveLength(0);
    expect(ret.code).toBe("let local;obj.e;obj.t;obj.e;");
    expect(ret.mangleCache).toEqual({ _often: "e", _rare: "t" });
  });

  it("keeps quoted occurrences unless quoted is enabled", () => {
    const source = "obj._field; obj['_field'];";
    expect(
      minifySync("test.js", source, {
        compress: false,
        mangle: false,
        mangleProps: { include: /^_/ },
      }).code,
    ).toBe("obj.e;obj[`_field`];");
    expect(
      minifySync("test.js", source, {
        compress: false,
        mangle: false,
        mangleProps: { include: /^_/, quoted: true },
      }).code,
    ).toBe("obj.e;obj.e;");
  });

  it("honors false entries and duplicate custom targets", () => {
    const ret = minifySync("test.js", "obj._first; obj._second; obj._keep;", {
      compress: false,
      mangle: false,
      mangleProps: {
        include: /^_/,
        cache: { _first: "A", _second: "A", _keep: false },
      },
    });

    expect(ret.code).toBe("obj.A;obj.A;obj._keep;");
    expect(ret.mangleCache).toEqual({ _first: "A", _second: "A", _keep: false });
  });

  it("keeps cache keys out of automatic targets across round trips", () => {
    const first = minifySync("test.js", "obj.foo;", {
      compress: false,
      mangle: false,
      mangleProps: { include: /./, cache: { e: false } },
    });
    expect(first.code).toBe("obj.t;");
    expect(first.mangleCache).toEqual({ e: false, foo: "t" });

    const second = minifySync("test.js", "obj.e; obj.foo;", {
      compress: false,
      mangle: false,
      mangleProps: { include: /./, cache: first.mangleCache },
    });
    expect(second.code).toBe("obj.e;obj.t;");
    expect(second.mangleCache).toEqual(first.mangleCache);
  });

  it("keeps every property readable in debug mode", () => {
    const ret = minifySync("test.js", "obj._alpha; obj._beta; obj._gamma;", {
      compress: false,
      mangle: false,
      mangleProps: { include: /^_/, debug: true },
    });
    expect(ret.code).toBe("obj._$_alpha$_;obj._$_beta$_;obj._$_gamma$_;");
  });

  it("honors exclude and reserved names", () => {
    const ret = minifySync("test.js", "obj._skipOne; obj._reserved; obj._mangle;", {
      compress: false,
      mangle: false,
      mangleProps: {
        include: /^_/,
        exclude: /^_skip(?:One|Two)$/,
        reserved: ["_reserved"],
      },
    });
    expect(ret.errors).toHaveLength(0);
    expect(ret.code).toBe("obj._skipOne;obj._reserved;obj.e;");
  });

  it("preserves the case-insensitive RegExp flag", () => {
    const ret = minifySync("test.js", "obj._FIELD; obj._other;", {
      compress: false,
      mangle: false,
      mangleProps: { include: /^_field$/i },
    });
    expect(ret.errors).toHaveLength(0);
    expect(ret.code).toBe("obj.e;obj._other;");
  });

  it("works through the async API", async () => {
    const ret = await minify("test.js", "obj._field; obj._field;", {
      compress: false,
      mangle: false,
      mangleProps: { include: /^_/ },
    });
    expect(ret.errors).toHaveLength(0);
    expect(ret.code).toBe("obj.e;obj.e;");
    expect(ret.mangleCache).toEqual({ _field: "e" });
  });

  it("does not mangle properties when parsing has errors", () => {
    const ret = minifySync("test.js", "const invalid; obj._field;", {
      compress: false,
      mangle: false,
      mangleProps: { include: /^_/ },
    });
    expect(ret.errors).toHaveLength(1);
    expect(ret.code).toBe("const invalid;obj._field;");
    expect(ret.mangleCache).toBeUndefined();
  });

  it("returns cache entries in stable lexical order", () => {
    const ret = minifySync(
      "test.js",
      "obj._zeta; obj._alpha; obj._middle; obj._beta; obj._omega;",
      {
        compress: false,
        mangle: false,
        mangleProps: { include: /^_/ },
      },
    );
    expect(Object.keys(ret.mangleCache!)).toEqual([
      "_alpha",
      "_beta",
      "_middle",
      "_omega",
      "_zeta",
    ]);
  });

  it("requires RegExp filters", () => {
    expect(() =>
      minifySync("test.js", "obj._field;", {
        mangleProps: { include: "^_" as any },
      }),
    ).toThrow(/RegExp/);
  });

  it.each([
    [{ include: /^(?!_keep)/ }, "Invalid mangleProps.include regex"],
    [{ include: /^_/g }, "Invalid mangleProps.include regex"],
    [{ include: /^_/, cache: { _field: true as any } }, "expected a string or false"],
    [
      { include: /^_/, cache: { ["__proto__"]: false } },
      "this original name cannot be used as a cache key",
    ],
    [{ include: /^_/, cache: { _field: "not-valid" } }, "must be an IdentifierName"],
    [{ include: /^_/, cache: { _field: "__proto__" } }, "must be an IdentifierName"],
    [
      { include: /^_/, cache: { _field: "constructor" } },
      "other than '__proto__', 'constructor', or 'prototype'",
    ],
    [
      { include: /^_/, cache: { _field: "prototype" } },
      "other than '__proto__', 'constructor', or 'prototype'",
    ],
  ])("rejects invalid options", (mangleProps, message) => {
    const ret = minifySync("test.js", "obj._field;", { mangleProps: mangleProps as any });
    expect(ret.errors).toHaveLength(1);
    expect(ret.errors[0].message).toContain(message);
    expect(ret.mangleCache).toBeUndefined();
  });
});

describe("worker", () => {
  it("should run", async () => {
    const code = await new Promise((resolve, reject) => {
      const worker = new Worker("./test/worker.mjs");
      worker.on("error", (err) => {
        reject(err);
      });
      worker.on("exit", (code) => {
        resolve(code);
      });
    });
    expect(code).toBe(0);
  });
});
