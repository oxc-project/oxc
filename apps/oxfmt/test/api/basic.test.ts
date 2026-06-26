import { describe, expect, it } from "vitest";
import { format } from "../../dist/index.js";

describe("Format non-js", () => {
  it("should format json with options", async () => {
    const jsoncCode = `
{
  // Package name
  "foo": "my",
  // Trailing comma test
  "bar": "1",
}
`.trim();
    const result = await format("foo.jsonc", jsoncCode, {
      insertFinalNewline: false,
    });
    expect(result.code).toBe(`${jsoncCode}`);
    expect(result.errors).toStrictEqual([]);
  });

  it("should format vue with options", async () => {
    const vueCode = `
<template><div>Vue</div></template>
<style>div{color:red;}</style>
`.trim();
    const result = await format("Component.vue", vueCode, {
      vueIndentScriptAndStyle: true,
    });
    expect(result.code).toBe(
      `
<template><div>Vue</div></template>
<style>
  div {
    color: red;
  }
</style>
`.trimStart(),
    );
    expect(result.errors).toStrictEqual([]);
  });

  it("should surface Prettier parse errors as-is", async () => {
    const brokenVue = `<template><div></template>`;
    const result = await format("broken.vue", brokenVue, {});

    expect(result.code).toBe(brokenVue);
    expect(result.errors[0]?.message).toMatch(/Unexpected closing tag/);
  });
});

describe("Format empty", () => {
  it("should format empty string", async () => {
    let result = await format("empty.js", "", {});
    expect(result.code).toBe("");
    expect(result.errors).toStrictEqual([]);

    result = await format("empty.toml", "  ", {});
    expect(result.code).toBe("");
    expect(result.errors).toStrictEqual([]);

    result = await format("empty.json", "\n\n", {});
    expect(result.code).toBe("");
    expect(result.errors).toStrictEqual([]);

    result = await format("empty.md", " \n ", {});
    expect(result.code).toBe("");
    expect(result.errors).toStrictEqual([]);
  });
});
