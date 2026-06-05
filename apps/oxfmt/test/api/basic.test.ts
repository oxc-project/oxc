import { describe, expect, it } from "vitest";
import { format } from "../../dist/index.js";
import { formatFileSafe } from "../../src-js/libs/apis";

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
  it("should return Prettier formatFile errors as data", async () => {
    const result = await formatFileSafe({
      code: `{`,
      options: { parser: "json5", filepath: "broken.json5" },
    });
    expect(result.ok).toBe(false);
    if (!result.ok) {
      expect(result.error).not.toBe("File formatting failed");
      expect(result.error).toMatch(/Unexpected|JSON|end/i);
    }
  });

  it("should surface Prettier parse errors from format API", async () => {
    const result = await format("broken.json5", `{`, {});

    expect(result.code).toBe(`{`);
    expect(result.errors).toHaveLength(1);
    expect(result.errors[0]?.message).not.toBe("File formatting failed");
    expect(result.errors[0]?.message).toMatch(/Unexpected|JSON|end/i);
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
