// Contract tests for `print`.
//
// The `raw_transfer_back` encoder is not implemented yet, so the behavioral tests are marked
// `it.fails`: they run, fail with the seam's "not implemented" error, and the suite stays green.
// The moment the implementation makes one pass, `it.fails` flips it to a hard failure - remove
// the `.fails` modifier as the implementation lands.
//
// Tests NOT marked `.fails` assert behavior which is real today (input validation, the stub
// error surface) and must keep passing.

import { describe, expect, it } from "vitest";
import { parseSync } from "oxc-parser";

import { print } from "../src-js/index.js";

function parse(sourceText: string, filename = "test.js") {
  const ret = parseSync(filename, sourceText);
  expect(ret.errors).toEqual([]);
  return ret;
}

describe("input validation (real behavior, must pass)", () => {
  it("throws TypeError on null", () => {
    // @ts-expect-error deliberately wrong input
    expect(() => print(null)).toThrow(TypeError);
  });

  it("throws TypeError on undefined", () => {
    // @ts-expect-error deliberately wrong input
    expect(() => print(undefined)).toThrow(TypeError);
  });

  it("throws TypeError on a non-Program node", () => {
    // @ts-expect-error deliberately wrong input
    expect(() => print({ type: "Identifier", name: "x", start: 0, end: 1 })).toThrow(TypeError);
  });

  it("throws TypeError on a string", () => {
    // @ts-expect-error deliberately wrong input
    expect(() => print("const x = 1;")).toThrow(TypeError);
  });
});

describe("implementation seam (current stub behavior, remove when encoder lands)", () => {
  it("print throws the not-implemented error for a valid Program", () => {
    const { program } = parse("const x = 1 + 2;");
    expect(() => print(program)).toThrow(/raw_transfer_back encoder is not implemented/);
  });

  it("printRawSync binding returns the not-implemented error", async () => {
    const { printRawSync } = await import("../src-js/bindings.js");
    const result = printRawSync(new Uint8Array(0), 0, 0, 0, null);
    expect(result.code).toBe("");
    expect(result.errors).toHaveLength(1);
    expect(result.errors[0].message).toMatch(/raw_transfer_back is not implemented/);
  });
});

describe("print contract (fails until raw_transfer_back lands)", () => {
  it.fails("prints a simple program", () => {
    const { program } = parse("const x = 1 + 2;");
    expect(print(program).code).toBe("const x = 1 + 2;\n");
  });

  it.fails("prints statements and expressions", () => {
    const source = [
      "function add(a, b) {",
      "\treturn a + b;",
      "}",
      "const result = add(1, 2);",
      "if (result > 2) {",
      "\tconsole.log(`result: ${result}`);",
      "}",
      "",
    ].join("\n");
    const { program } = parse(source);
    expect(print(program).code).toBe(source);
  });

  it.fails("singleQuote", () => {
    const { program } = parse('const s = "hello";');
    expect(print(program, { singleQuote: true }).code).toBe("const s = 'hello';\n");
  });

  it.fails("minify", () => {
    const { program } = parse("const x = 1 + 2;\nconst y = 3;");
    expect(print(program, { minify: true }).code).toBe("const x=1+2;const y=3;");
  });

  it.fails("indentChar and indentWidth", () => {
    const { program } = parse("if (a) {\n\tb();\n}");
    expect(print(program, { indentChar: "space", indentWidth: 2 }).code).toBe(
      "if (a) {\n  b();\n}\n",
    );
  });

  it.fails("comments with sourceText", () => {
    const sourceText = "// leading comment\nconst x = 1;\n";
    const { program } = parse(sourceText);
    expect(print(program, { sourceText }).code).toBe(sourceText);
  });

  it.fails("comments are omitted without sourceText", () => {
    const { program } = parse("// leading comment\nconst x = 1;\n");
    expect(print(program).code).toBe("const x = 1;\n");
  });

  it.fails("sourcemap", () => {
    const sourceText = "const x = 1 + 2;";
    const { program } = parse(sourceText);
    const result = print(program, { sourceText, filename: "test.js", sourcemap: true });
    expect(result.map).toBeDefined();
    expect(result.map!.sources).toEqual(["test.js"]);
    expect(result.map!.mappings.length).toBeGreaterThan(0);
  });

  it.fails("print is a fixed point on its own output", () => {
    const { program } = parse("const x = { a: [1, 2], b() { return this.a; } };");
    const once = print(program).code;
    const twice = print(parse(once).program).code;
    expect(twice).toBe(once);
  });

  it.fails("TypeScript AST", () => {
    const sourceText = "type A = string | number;\nconst x: A = 1;\n";
    const { program } = parse(sourceText, "test.ts");
    expect(print(program).code).toBe(sourceText);
  });
});
