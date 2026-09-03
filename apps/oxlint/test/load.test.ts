import { describe, expect, it } from "vitest";
import { getPluginLoadErrorHint } from "../src-js/plugins/load.ts";

describe("getPluginLoadErrorHint", () => {
  it("hints at the ES module requirement for an `import` statement in CommonJS", () => {
    const err = new SyntaxError("Cannot use import statement outside a module");
    expect(getPluginLoadErrorHint(err)).toMatchInlineSnapshot(`
      "

      Plugins must be ES modules. Add \`"type": "module"\` to the nearest \`package.json\`, or give the plugin file an \`.mjs\` or \`.mts\` extension."
    `);
  });

  it("hints at the ES module requirement for an `export` statement in CommonJS", () => {
    const err = new SyntaxError("Unexpected token 'export'");
    expect(getPluginLoadErrorHint(err)).toContain('"type": "module"');
  });

  it("returns empty string for an unrelated `SyntaxError`", () => {
    expect(getPluginLoadErrorHint(new SyntaxError("Unexpected end of input"))).toBe("");
  });

  it("returns empty string for a non-`SyntaxError`", () => {
    expect(getPluginLoadErrorHint(new Error("Cannot use import statement outside a module"))).toBe(
      "",
    );
  });

  it("returns empty string for a non-error", () => {
    expect(getPluginLoadErrorHint(null)).toBe("");
    expect(getPluginLoadErrorHint("Cannot use import statement outside a module")).toBe("");
  });
});
