import { describe, expect, it } from "vitest";
import { format } from "../../dist/index.js";

describe("CSS/SCSS/Less files (oxc_formatter_css)", () => {
  it("should format standalone CSS files in Rust", async () => {
    const source = `a{color:red;background:blue}`;
    const result = await format("styles.css", source);
    expect(result.code).toMatchInlineSnapshot(`
      "a {
        color: red;
        background: blue;
      }
      "
    `);
  });

  it("should format .wxss, .pcss and .postcss extensions as CSS", async () => {
    const source = `a{color:red}`;
    for (const filename of ["app.wxss", "styles.pcss", "styles.postcss"]) {
      // oxlint-disable-next-line no-await-in-loop
      const result = await format(filename, source);
      expect(result.code).toBe("a {\n  color: red;\n}\n");
    }
  });

  it("should format SCSS files in Rust", async () => {
    const source = `$gap:8px;.card{margin:$gap;&:hover{color:blue}}`;
    const result = await format("main.scss", source);
    expect(result.code).toMatchInlineSnapshot(`
      "$gap: 8px;
      .card {
        margin: $gap;
        &:hover {
          color: blue;
        }
      }
      "
    `);
  });

  it("should format Less files in Rust", async () => {
    const source = `@gap:8px;.card{margin:@gap}`;
    const result = await format("theme.less", source);
    expect(result.code).toMatchInlineSnapshot(`
      "@gap: 8px;
      .card {
        margin: @gap;
      }
      "
    `);
  });

  it("should respect singleQuote and useTabs", async () => {
    const source = `a { content: "hi"; font-family: "Arial" }`;
    const result = await format("styles.css", source, {
      singleQuote: true,
      useTabs: true,
    });
    expect(result.code).toMatchInlineSnapshot(`
      "a {
      	content: 'hi';
      	font-family: 'Arial';
      }
      "
    `);
  });

  it("should report a diagnostic for input rejects (no Prettier fallback)", async () => {
    // IE star hack: postcss would tolerate it as a raw declaration,
    // but `oxc_formatter_css` does not, the parse error is surfaced as-is.
    const source = `a { *zoom: 1; color: red }`;
    const result = await format("legacy.css", source);
    expect(result.code).toBe(source);
    expect(result.errors).toHaveLength(1);
    expect(result.errors[0].message).toMatch(/Syntax error/);
  });

  it("should report a diagnostic for genuinely broken input", async () => {
    const source = `a {\n  color: red;\n`;
    const result = await format("broken.css", source);
    expect(result.code).toBe(source);
    expect(result.errors).toHaveLength(1);
    expect(result.errors[0].message).toMatch(/Syntax error/);
  });
});
