import { format } from "../dist/index.js";
import { describe, expect, test } from "vitest";
import type { FormatOptions } from "../dist/index.js";

describe("API Tests", () => {
  test("`format()` function exists", () => {
    expect(typeof format).toBe("function");
  });

  test("should `format()` multiple times", async () => {
    const result1 = await format("a.ts", "const x:number=42");
    expect(result1.code).toBe("const x: number = 42;\n");
    expect(result1.errors).toStrictEqual([]);

    const result2 = await format("a.json", '{"key":           "value"}');
    expect(result2.code).toBe('{ "key": "value" }\n');
    expect(result2.errors).toStrictEqual([]);
  });

  test("should TS types and options work", async () => {
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

  test("should `format()` with options work", async () => {
    const pkgJSON = JSON.stringify({
      version: "1.0.0",
      name: "my-package",
    });
    const result1 = await format("package.json", pkgJSON);
    expect(result1.code).toBe(`{\n  "name": "my-package",\n  "version": "1.0.0"\n}\n`);
    expect(result1.errors).toStrictEqual([]);

    const result2 = await format("package.json", pkgJSON, { experimentalSortPackageJson: false });
    expect(result2.code).toBe(`{\n  "version": "1.0.0",\n  "name": "my-package"\n}\n`);
    expect(result2.errors).toStrictEqual([]);

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

  // Regression test for https://github.com/oxc-project/oxc/issues/17604
  test("should format Vue file with interpolations idempotently", async () => {
    const vueCode = `<template><div>{{ msg }}</div></template>`;
    const result1 = await format("test.vue", vueCode);
    const result2 = await format("test.vue", result1.code);
    const result3 = await format("test.vue", result2.code);
    expect(result1.code).toBe(result2.code);
    expect(result2.code).toBe(result3.code);
    expect(result1.errors).toStrictEqual([]);
  });
});
