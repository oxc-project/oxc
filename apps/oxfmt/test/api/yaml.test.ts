import { describe, expect, it } from "vitest";
import { format } from "../../dist/index.js";

describe("YAML files (oxc_formatter_yaml)", () => {
  it("should format standalone YAML files in Rust", async () => {
    const source = `a:   1\nlist:\n  - "x"\n  -   y\nflow: {a: 1, b: [1,2]}\n`;
    const result = await format("config.yaml", source);
    expect(result.code).toMatchInlineSnapshot(`
      "a: 1
      list:
        - "x"
        - y
      flow: { a: 1, b: [1, 2] }
      "
    `);
  });

  it("should format .yml and other extensions and filenames as YAML", async () => {
    const source = `key:   value\n`;
    for (const filename of [
      "config.yml",
      "grammar.sublime-syntax",
      ".clang-format",
      "CITATION.cff",
    ]) {
      // oxlint-disable-next-line no-await-in-loop
      const result = await format(filename, source);
      expect(result.code).toBe("key: value\n");
    }
  });

  it("should respect singleQuote and proseWrap", async () => {
    const quoted = await format("config.yaml", `q: 'single'\n`, {
      singleQuote: true,
    });
    expect(quoted.code).toBe("q: 'single'\n");

    const wrapped = await format("config.yaml", `text: aaa bbb ccc\n`, {
      proseWrap: "always",
      printWidth: 8,
    });
    expect(wrapped.code).toBe("text:\n  aaa\n  bbb\n  ccc\n");
  });

  it("should format rc files as JSON when the whole text parses as JSON", async () => {
    const source = `{\n  "singleQuote":true,   "semi": false\n}\n`;
    const result = await format(".stylelintrc", source);
    expect(result.code).toBe(`{\n  "singleQuote": true,\n  "semi": false\n}\n`);
  });

  it("should format rc files as YAML when the text is not JSON", async () => {
    const source = `singleQuote:   true\nsemi: false\n`;
    const result = await format(".prettierrc", source);
    expect(result.code).toBe("singleQuote: true\nsemi: false\n");
  });

  it("should report a diagnostic for broken input (no Prettier fallback)", async () => {
    // Unclosed flow sequence; the parser is fail-fast, so the error is surfaced as-is.
    // Prettier 3.9 (yaml@2) rejects this input as well.
    const source = `a: [1, 2\n`;
    const result = await format("broken.yaml", source);
    expect(result.code).toBe(source);
    expect(result.errors).toHaveLength(1);
    expect(result.errors[0].message).toMatch(/Syntax error/);
  });
});
