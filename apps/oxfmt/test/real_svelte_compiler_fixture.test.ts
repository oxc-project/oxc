import { createRequire } from "node:module";
import { join } from "node:path";
import { describe, expect, it } from "vitest";

const fixturesDir = join(import.meta.dirname, "cli", "plugin_languages_real_package", "fixtures");
const compilerPath = join(fixturesDir, "node_modules", "svelte", "compiler.js");
const snipTagContentPath = join(
  fixturesDir,
  "node_modules",
  "prettier-plugin-svelte",
  "src",
  "lib",
  "snipTagContent.js",
);
const require = createRequire(import.meta.url);

const { parse } = require(compilerPath) as {
  parse: (source: string) => {
    type: string;
    start: number;
    end: number;
    html: {
      type: string;
      start: number;
      end: number;
      children: Array<{ type: string; name?: string }>;
    };
    instance: { type: string; attributes: Array<{ name: string }> } | null;
    css: { type: string; attributes: Array<{ name: string }> } | null;
  };
};
const { snipScriptAndStyleTagContent, snippedTagContentAttribute } = require(snipTagContentPath) as {
  snipScriptAndStyleTagContent: (source: string) => { text: string; isTypescript: boolean };
  snippedTagContentAttribute: string;
};

const input = `<style>h1{color:red}</style>
<h1>Hello {name}</h1>
<script>export let name = "world";</script>
`;

describe("real prettier-plugin-svelte compiler fixture", () => {
  it("returns source spans and preserves snipped top-level tag markers", () => {
    const { text, isTypescript } = snipScriptAndStyleTagContent(input);
    const ast = parse(text);

    expect(isTypescript).toBe(false);
    expect(ast).toMatchObject({
      type: "Root",
      start: 0,
      end: text.length,
      html: {
        type: "Fragment",
        start: 0,
        end: text.length,
      },
      instance: {
        type: "Script",
      },
      css: {
        type: "Style",
      },
    });
    expect(ast.instance?.attributes).toEqual(
      expect.arrayContaining([expect.objectContaining({ name: snippedTagContentAttribute })]),
    );
    expect(ast.css?.attributes).toEqual(
      expect.arrayContaining([expect.objectContaining({ name: snippedTagContentAttribute })]),
    );
    expect(ast.html.children).toEqual([expect.objectContaining({ type: "Element", name: "h1" })]);
  });
});
