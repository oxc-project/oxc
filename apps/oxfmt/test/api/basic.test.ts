import { describe, expect, it } from "vitest";
import { format } from "../../dist/index.js";
import type { FormatOptions } from "../../dist/index.js";

describe("Basic", () => {
  it("`format()` function exists", () => {
    expect(typeof format).toBe("function");
  });

  it("dynamic import also works", async () => {
    const { format } = await import("../../dist/index.js");
    const result = await format("a.ts", "const x:number=42");
    expect(result.code).toBe("const x: number = 42;\n");
    expect(result.errors).toStrictEqual([]);
  });

  it("should `format()` multiple times w/o panic", async () => {
    const result1 = await format("a.ts", "const x:number=42");
    expect(result1.code).toBe("const x: number = 42;\n");
    expect(result1.errors).toStrictEqual([]);

    const result2 = await format("a.json", '{"key":           "value"}');
    expect(result2.code).toBe('{ "key": "value" }\n');
    expect(result2.errors).toStrictEqual([]);
  });

  it("should TS types and options work", async () => {
    const options: FormatOptions = {
      quoteProps: "as-needed", // Can be string literal
      printWidth: 120,
      semi: false,
      experimentalSortPackageJson: false,
      experimentalSortImports: {
        // Can be optional object
        partitionByComment: false,
      },
    };

    const result = await format("a.ts", "const x={'y':1}", options);
    expect(result.code).toBe("const x = { y: 1 }\n");
    expect(result.errors).toStrictEqual([]);

    const { errors } = await format("a.ts", "const x={'y':1}", {
      // @ts-expect-error: Test invalid options is validated
      semi: "invalid",
    });
    expect(errors.length).toBe(1);
  });

  it("should format non-js files with options", async () => {
    const jsoncCode = `
{
  // Package name
  "foo": "my",
  // Trailing comma test
  "bar": "1",
}
`.trim();
    const result1 = await format("foo.jsonc", jsoncCode, {
      insertFinalNewline: false,
    });
    expect(result1.code).toBe(`${jsoncCode}`);
    expect(result1.errors).toStrictEqual([]);

    const vueCode = `
<template><div>Vue</div></template>
<style>div{color:red;}</style>
`.trim();
    const result3 = await format("Component.vue", vueCode, {
      vueIndentScriptAndStyle: true,
    });
    expect(result3.code).toBe(
      `
<template><div>Vue</div></template>
<style>
  div {
    color: red;
  }
</style>
`.trimStart(),
    );
    expect(result3.errors).toStrictEqual([]);
  });
});
