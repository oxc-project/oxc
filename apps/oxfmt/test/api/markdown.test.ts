import { describe, expect, it } from "vitest";
import { format } from "../../dist/index.js";

// Markdown is formatted by `oxc_formatter_markdown`;
// its front matter and fenced code blocks dispatch through the session like any other host.
// The outputs below were verified against bundled Prettier 3.9.
// JS / TS code blocks are covered by `js-in-markdown.test.ts`.
describe("Markdown", () => {
  it("formats every Markdown file name", async () => {
    for (const filename of ["a.md", "a.markdown", "README", "contents.lr"]) {
      // oxlint-disable-next-line no-await-in-loop
      const result = await format(filename, "#  Title\n");
      expect(result.errors).toStrictEqual([]);
      expect(result.code).toBe("# Title\n");
    }
  });

  it("applies proseWrap and singleQuote", async () => {
    const source = 'a b c d e f\n\n[x](u "it\'s")\n';
    const result = await format("a.md", source, { printWidth: 5, proseWrap: "always" });
    expect(result.errors).toStrictEqual([]);
    expect(result.code).toMatchInlineSnapshot(`
      "a b c
      d e f

      [x](u "it's")
      "
    `);
    const single = await format("a.md", '[x](u "t")\n', { singleQuote: true });
    expect(single.code).toBe("[x](u 't')\n");
  });

  it("formats YAML front matter", async () => {
    const result = await format("a.md", "---\ntitle:   Home\n---\n#  Hi\n");
    expect(result.errors).toStrictEqual([]);
    expect(result.code).toMatchInlineSnapshot(`
      "---
      title: Home
      ---

      # Hi
      "
    `);
  });

  it("formats fenced code blocks, keeping their column", async () => {
    const source =
      "- a\n\n  ```js\n  const  s = `x\n  y`\n  ```\n\n> ```css\n> a{color:red}\n> ```\n";
    const result = await format("a.md", source);
    expect(result.errors).toStrictEqual([]);
    expect(result.code).toMatchInlineSnapshot(`
      "- a

        \`\`\`js
        const s = \`x
        y\`;
        \`\`\`

      > \`\`\`css
      > a {
      >   color: red;
      > }
      > \`\`\`
      "
    `);
  });

  it("keeps container columns as spaces under useTabs", async () => {
    // A tab would shift the content column the next parse strips (the code and the quote change)
    const source =
      "- item\n\n  ```js\n  new Miniflare({\n  httpsKey: 1,\n  });\n  ```\n\n  > quoted\n  > more\n";
    const result = await format("a.md", source, { useTabs: true });
    expect(result.errors).toStrictEqual([]);
    expect(result.code).toBe(
      "- item\n\n  ```js\n  new Miniflare({\n  \thttpsKey: 1,\n  });\n  ```\n\n  > quoted\n  > more\n",
    );
  });

  it("keeps unknown or unparsable code blocks verbatim", async () => {
    const source = "```unknown\na   b\n```\n\n```js\nconst =\n```\n";
    const result = await format("a.md", source);
    expect(result.errors).toStrictEqual([]);
    expect(result.code).toBe(source);
  });

  it("keeps front matter and code blocks verbatim under embeddedLanguageFormatting: off", async () => {
    const source = "---\ntitle:   Home\n---\n\n```css\na{color:red}\n```\n";
    const result = await format("a.md", source, { embeddedLanguageFormatting: "off" });
    expect(result.errors).toStrictEqual([]);
    expect(result.code).toBe(source);
  });
});
