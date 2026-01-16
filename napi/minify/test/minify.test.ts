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

describe("ascii_only option", () => {
  it("escapes non-ASCII characters in strings", () => {
    const code = 'let x = "中文"';
    const ret = minifySync("test.js", code, {
      codegen: { asciiOnly: true },
    });
    // Minifier converts strings to template literals
    expect(ret.code).toBe("let x=`\\u4E2D\\u6587`;");
    expect(ret.errors.length).toBe(0);
  });

  it("escapes non-BMP characters with unicode escapes", () => {
    const code = 'let x = "𐀀"';
    const ret = minifySync("test.js", code, {
      codegen: { asciiOnly: true },
    });
    // Minifier converts strings to template literals
    expect(ret.code).toBe("let x=`\\u{10000}`;");
    expect(ret.errors.length).toBe(0);
  });

  it("escapes non-ASCII identifiers", () => {
    const code = "let 中文 = 1";
    const ret = minifySync("test.js", code, {
      mangle: false,
      codegen: { asciiOnly: true },
    });
    expect(ret.code).toBe("let \\u4E2D\\u6587=1;");
    expect(ret.errors.length).toBe(0);
  });

  it("converts non-BMP property access to computed", () => {
    const code = "x.𐀀";
    const ret = minifySync("test.js", code, {
      codegen: { asciiOnly: true },
    });
    expect(ret.code).toBe('x["\\u{10000}"];');
    expect(ret.errors.length).toBe(0);
  });

  it("converts non-BMP object keys to strings", () => {
    const code = "let obj = {𐀀: 1}";
    const ret = minifySync("test.js", code, {
      codegen: { asciiOnly: true },
    });
    expect(ret.code).toBe('let obj={"\\u{10000}":1};');
    expect(ret.errors.length).toBe(0);
  });

  it("does not escape when ascii_only is false", () => {
    const code = 'let x = "中文"';
    const ret = minifySync("test.js", code, {
      codegen: { asciiOnly: false },
    });
    // Minifier converts strings to template literals
    expect(ret.code).toBe("let x=`中文`;");
    expect(ret.errors.length).toBe(0);
  });

  it("always escapes BOM character", () => {
    const code = 'let x = "\uFEFF"';
    const ret = minifySync("test.js", code, {
      codegen: { asciiOnly: false },
    });
    // Minifier converts strings to template literals, BOM is always escaped
    expect(ret.code).toBe("let x=`\\uFEFF`;");
    expect(ret.errors.length).toBe(0);
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
