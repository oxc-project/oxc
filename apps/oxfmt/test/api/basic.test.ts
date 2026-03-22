import { describe, expect, it } from "vitest";
import { format, defineConfig } from "../../dist/index.js";
import type { FormatOptions } from "../../dist/index.js";

describe("defineConfig() API", () => {
  it("`defineConfig()` function exists", () => {
    expect(typeof defineConfig).toBe("function");
  });

  it("`defineConfig()` returns the same object", () => {
    const config = defineConfig({ semi: true, tabWidth: 4, ignorePatterns: ["*.min.js"] });
    expect(config).toStrictEqual({ semi: true, tabWidth: 4, ignorePatterns: ["*.min.js"] });
  });
});

describe("format() API", () => {
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

  it("should normalize JSON-family formats by filepath", async () => {
    const jsonResult = await format("foo.json", "{foo:'bar',}");
    expect(jsonResult.code).toBe('{ "foo": "bar" }\n');
    expect(jsonResult.errors).toStrictEqual([]);

    const json5Result = await format("foo.json5", '{"foo":"bar"}');
    expect(json5Result.code).toBe('{ foo: "bar" }\n');
    expect(json5Result.errors).toStrictEqual([]);

    const packageResult = await format("package.json", "{name:'fixture',version:'1.0.0'}");
    expect(packageResult.code).toBe(
      `{
  "name": "fixture",
  "version": "1.0.0"
}
`,
    );
    expect(packageResult.errors).toStrictEqual([]);
  });

  it("should allow overriding JSON-family parser like Prettier", async () => {
    const jsonAsJson5 = await format("foo.json", '{"foo":"bar"}', {
      parser: "json5",
    });
    expect(jsonAsJson5.code).toBe('{ foo: "bar" }\n');
    expect(jsonAsJson5.errors).toStrictEqual([]);

    const json5AsJson = await format("foo.json5", "{foo:'bar',}", {
      parser: "json",
    });
    expect(json5AsJson.code).toBe('{ "foo": "bar" }\n');
    expect(json5AsJson.errors).toStrictEqual([]);
  });
});
