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

// ASCII-only mode comprehensive test
// See: https://github.com/oxc-project/oxc/issues/17068
describe("asciiOnly option", () => {
  it("escapes all non-ASCII variants", () => {
    const testCases: Array<{ input: string; expected: string; description: string }> = [
      // BMP characters in identifiers (\uXXXX)
      { input: "let π = 1", expected: "let \\u03C0=1;", description: "BMP identifier" },
      {
        input: "let _π = 1",
        expected: "let _\\u03C0=1;",
        description: "BMP identifier with underscore prefix",
      },
      {
        input: "let π_ = 1",
        expected: "let \\u03C0_=1;",
        description: "BMP identifier with underscore suffix",
      },

      // BMP characters in strings (\uXXXX)
      { input: "'π'", expected: "`\\u03C0`;", description: "BMP string" },
      { input: "'中文'", expected: "`\\u4E2D\\u6587`;", description: "CJK string" },

      // Non-BMP characters in strings (\u{XXXXXX})
      { input: "'🐈'", expected: "`\\u{1F408}`;", description: "Non-BMP emoji string" },
      { input: "'𐀀'", expected: "`\\u{10000}`;", description: "Non-BMP string" },

      // Non-BMP identifiers (\u{XXXXXX})
      { input: "var 𐀀", expected: "var \\u{10000};", description: "Non-BMP identifier" },

      // Non-BMP property access -> computed syntax
      { input: "x.𐀀", expected: 'x["\\u{10000}"];', description: "Non-BMP property access" },

      // Non-BMP object keys -> string keys
      { input: "({𐀀: 1})", expected: '({"\\u{10000}":1});', description: "Non-BMP object key" },

      // Always escaped characters (regardless of asciiOnly)
      { input: "'\uFEFF'", expected: "`\\uFEFF`;", description: "BOM always escaped" },
      { input: "'\u2028'", expected: "`\\u2028`;", description: "Line separator always escaped" },
      {
        input: "'\u2029'",
        expected: "`\\u2029`;",
        description: "Paragraph separator always escaped",
      },
    ];

    for (const { input, expected, description } of testCases) {
      const ret = minifySync("test.js", input, {
        mangle: false,
        codegen: { asciiOnly: true },
      });
      expect(ret.code, description).toBe(expected);
      expect(ret.errors.length, `${description} should have no errors`).toBe(0);
    }
  });
});
