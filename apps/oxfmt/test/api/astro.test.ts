import { describe, expect, it } from "vitest";
import { format } from "../../dist/index.js";

const INPUT = `---
import b from "b"
import a from "a"
const   items=[1,2,3]
if (!items) return Astro.redirect('/404')
---
<div class="p-4 flex bg-red-500"><Foo   title={ "x" } />
{items.map((i)=><p>{i}</p>)}
</div>
<script>
const x   = 1
</script>
<style>
div{color:red}
</style>
`;

describe("Astro support", () => {
  describe("Basic", () => {
    it("should format `.astro` with `astro: {}` (defaults)", async () => {
      const result = await format("App.astro", INPUT, { astro: {} });
      expect(result.errors).toStrictEqual([]);
      expect(result.code).toMatchSnapshot();
    });

    it("should format `.astro` with `astro: true` (defaults, equivalent to `{}`)", async () => {
      const trueResult = await format("App.astro", INPUT, { astro: true });
      const objectResult = await format("App.astro", INPUT, { astro: {} });
      expect(trueResult.errors).toStrictEqual([]);
      expect(trueResult.code).toBe(objectResult.code);
    });

    it("should respect `astro.skipFrontmatter`", async () => {
      const result = await format("App.astro", INPUT, { astro: { skipFrontmatter: true } });
      expect(result.errors).toStrictEqual([]);
      expect(result.code).toContain("const   items=[1,2,3]");
    });

    it("should accept all `astro.*` options together", async () => {
      const result = await format("App.astro", INPUT, {
        astro: { allowShorthand: true, skipFrontmatter: false, compressHTML: "html" },
      });
      expect(result.errors).toStrictEqual([]);
      expect(result.code).toMatchSnapshot();
    });
  });

  describe("Gating", () => {
    it("should error on `.astro` without `astro` option", async () => {
      const result = await format("App.astro", INPUT);
      expect(result.code).toBe(INPUT);
      expect(result.errors.length).toBe(1);
      expect(result.errors[0].message).toMatch(/Cannot format `\.astro`/);
    });

    it("should error on `.astro` with `astro: false` (explicitly disabled)", async () => {
      const result = await format("App.astro", INPUT, { astro: false });
      expect(result.code).toBe(INPUT);
      expect(result.errors.length).toBe(1);
    });
  });

  it("should format frontmatter by Oxfmt (`sortImports`)", async () => {
    const result = await format("App.astro", INPUT, { astro: {}, sortImports: {} });
    expect(result.errors).toStrictEqual([]);
    expect(result.code.indexOf('from "a"')).toBeLessThan(result.code.indexOf('from "b"'));
  });

  it("should sort Tailwind classes in template attributes", async () => {
    const result = await format("App.astro", INPUT, { astro: {}, sortTailwindcss: {} });
    expect(result.errors).toStrictEqual([]);
    expect(result.code).toContain('class="flex bg-red-500 p-4"');
  });

  it('should keep `<style lang="sass">` as-is', async () => {
    const sass = `<style lang="sass">\n.a\n\t\tcolor: red\n</style>\n`;
    const result = await format("App.astro", `<p   class="x" />\n${sass}`, { astro: {} });
    expect(result.errors).toStrictEqual([]);
    expect(result.code).toBe(`<p class="x" />\n${sass}`);
  });

  it("should keep a `<style>` that fails to format as-is, without affecting later files", async () => {
    const broken = "<p />\n<style>\n.a { color: red\n</style>\n";
    const brokenResult = await format("Broken.astro", broken, { astro: {} });
    expect(brokenResult.errors).toStrictEqual([]);
    expect(brokenResult.code).toBe(broken);
    expect(process.env.PRETTIER_DEBUG).toBeUndefined();

    const sass = `<style lang="sass">\n.a\n  color: red\n</style>\n`;
    const result = await format("App.astro", sass, { astro: {} });
    expect(result.errors).toStrictEqual([]);
    expect(result.code).toBe(sass);
  });

  it("should format ```astro code blocks in Markdown", async () => {
    const input = '# T\n\n```astro\n<p   class="x">{ a }</p>\n```\n';
    const result = await format("README.md", input, { astro: {} });
    expect(result.errors).toStrictEqual([]);
    expect(result.code).toContain('<p class="x">{a}</p>');
  });
});
