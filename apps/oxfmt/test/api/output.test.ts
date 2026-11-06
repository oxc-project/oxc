import { mkdtemp, rm, writeFile } from "node:fs/promises";
import { tmpdir } from "node:os";
import { join, relative } from "node:path";
import { describe, expect, it, vi } from "vitest";
import { format } from "../../dist/index.js";

describe("format results", () => {
  it.each([
    ["broken.css", "} a { color: red }"],
    ["App.svelte", "<p>{count}</p>\n"],
  ])("should leave %s as-is on an error", async (filename, source) => {
    const result = await format(filename, source);
    expect(result.errors).toHaveLength(1);
    expect(result.code).toBe(source);
  });

  it.each([
    ["App.astro", "astro", "<div   class='a'  />\n"],
    ["App.svelte", "svelte", "<script>let   count=$state(0);</script>\n<p>{count}</p>\n"],
  ])("should format %s with `%s: true` as with `{}`", async (filename, key, input) => {
    const trueResult = await format(filename, input, { [key]: true });
    const objectResult = await format(filename, input, { [key]: {} });
    expect(trueResult.errors).toStrictEqual([]);
    expect(trueResult.code).toBe(objectResult.code);
  });

  // `prettier-plugin-svelte` does not handle this (https://github.com/sveltejs/prettier-plugin-svelte/issues/456),
  // and dumps the offending AST node via `console.error` before throwing
  // (https://github.com/sveltejs/prettier-plugin-svelte/blob/c4b7844961d0b937ab1eab2c8323d40a9422ac30/src/print/index.ts#L725)
  it("should currently error on Svelte with `embeddedLanguageFormatting: 'off'` (known limitation)", async () => {
    const errSpy = vi.spyOn(console, "error").mockImplementation(() => {});
    try {
      const input = "<script>\nconst x={a:1,b:2};\n</script>\n<p>{x.a}</p>\n";
      const result = await format("App.svelte", input, {
        svelte: {},
        embeddedLanguageFormatting: "off",
      });
      expect(result.errors[0]?.message).toMatch(/unknown node type: Script/);
    } finally {
      errSpy.mockRestore();
    }
  });

  it("should format JSDoc in a .vue `<script>` the same as in plain JS", async () => {
    const jsdoc = `/**
 * @param {string}   name   -  The   name
 * @returns  {boolean}   Whether   it   is   valid
 */
function validate(name) {}`;
    const jsResult = await format("a.js", jsdoc, { jsdoc: {} });
    const vueResult = await format("a.vue", `<script>\n${jsdoc}\n</script>\n`, { jsdoc: {} });
    expect(jsResult.errors).toStrictEqual([]);
    expect(vueResult.errors).toStrictEqual([]);
    expect(vueResult.code).toContain(jsResult.code.trim());
  });

  it("should resolve a relative Tailwind `config` path from the cwd", async () => {
    const dir = await mkdtemp(join(tmpdir(), "oxfmt-tailwind-"));
    try {
      await writeFile(
        join(dir, "tailwind.config.js"),
        "module.exports = { content: [], theme: { extend: {} }, plugins: [] };",
      );
      const result = await format(
        "src/test.tsx",
        `const A = <div className="p-4 flex">Hello</div>;`,
        {
          sortTailwindcss: { config: join(relative(process.cwd(), dir), "tailwind.config.js") },
        },
      );
      expect(result.errors).toStrictEqual([]);
      expect(result.code).toContain('className="flex p-4"');
    } finally {
      await rm(dir, { recursive: true, force: true });
    }
  });
});
