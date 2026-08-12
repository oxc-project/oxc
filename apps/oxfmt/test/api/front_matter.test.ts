import { describe, expect, it } from "vitest";
import { format } from "../../dist/index.js";

// The behavior matrix for CSS front matter, pinned end-to-end
// (verification lives here, not in `oxc_formatter_css`,
// mirroring how embedded behavior is validated through oxfmt in general).
// The composed outputs below were verified against bundled Prettier 3.9.
describe("CSS front matter", () => {
  it("formats a Jekyll-style YAML block and the stylesheet body", async () => {
    const source = "---\ntitle:   Home\nlist:\n    -   1\n---\n.a { color: red }\n";
    const result = await format("a.scss", source);
    expect(result.errors).toStrictEqual([]);
    expect(result.code).toBe("---\ntitle: Home\nlist:\n  - 1\n---\n\n.a {\n  color: red;\n}\n");
  });

  it("re-emits an explicit `---yaml` tag", async () => {
    const result = await format("a.css", "---yaml\ntitle:   Home\n---\na {}\n");
    expect(result.errors).toStrictEqual([]);
    expect(result.code).toBe("---yaml\ntitle: Home\n---\n\na {\n}\n");
  });

  it("normalizes the gap after the block to exactly one blank line", async () => {
    const none = await format("a.css", "---\na: 1\n---\nb {}\n");
    const many = await format("a.css", "---\na: 1\n---\n\n\n\nb {}\n");
    expect(none.code).toBe("---\na: 1\n---\n\nb {\n}\n");
    expect(many.code).toBe(none.code);
  });

  it("prints an empty block as the delimiters alone", async () => {
    const result = await format("a.css", "---\n---\nb {}\n");
    expect(result.errors).toStrictEqual([]);
    expect(result.code).toBe("---\n---\n\nb {\n}\n");
  });

  it("re-emits a `...` closing delimiter", async () => {
    const result = await format("a.css", "---\na:  1\n...\nb {}\n");
    expect(result.errors).toStrictEqual([]);
    expect(result.code).toBe("---\na: 1\n...\n\nb {\n}\n");
  });

  it("prints a block without a body alone", async () => {
    const result = await format("a.css", "---\na:  1\n---\n");
    expect(result.errors).toStrictEqual([]);
    expect(result.code).toBe("---\na: 1\n---\n");
  });

  it("keeps non-YAML blocks verbatim", async () => {
    // Custom language: never dispatched (Prettier keeps it raw too).
    const custom = await format("a.css", "---mycustomparser\na:   1\n---\nb {}\n");
    expect(custom.errors).toStrictEqual([]);
    expect(custom.code).toBe("---mycustomparser\na:   1\n---\n\nb {\n}\n");

    // TOML: dispatched but no native formatter, degrades to verbatim.
    const toml = await format("a.css", "+++\na   =   1\n+++\nb {}\n");
    expect(toml.errors).toStrictEqual([]);
    expect(toml.code).toBe("+++\na   =   1\n+++\n\nb {\n}\n");
  });

  it("keeps the block verbatim when the YAML does not parse", async () => {
    const result = await format("a.css", "---\nkey: [unclosed\n---\nb {}\n");
    expect(result.errors).toStrictEqual([]);
    expect(result.code).toBe("---\nkey: [unclosed\n---\n\nb {\n}\n");
  });

  it("keeps the block verbatim under embeddedLanguageFormatting: off", async () => {
    const result = await format("a.css", "---\ntitle:   Home\n---\nb {}\n", {
      embeddedLanguageFormatting: "off",
    });
    expect(result.errors).toStrictEqual([]);
    expect(result.code).toBe("---\ntitle:   Home\n---\n\nb {\n}\n");
  });

  it("keeps a physical BOM at byte 0, before the block", async () => {
    const result = await format("a.css", "\uFEFF---\ntitle:  Home\n---\nb {}\n");
    expect(result.errors).toStrictEqual([]);
    expect(result.code).toBe("\uFEFF---\ntitle: Home\n---\n\nb {\n}\n");
  });

  it("normalizes CRLF input like the rest of the file", async () => {
    const result = await format("a.css", "---\r\ntitle:  Home\r\n---\r\nb {}\r\n");
    expect(result.errors).toStrictEqual([]);
    expect(result.code).toBe("---\ntitle: Home\n---\n\nb {\n}\n");
  });

  it("never treats a css-in-js template starting with `---` as front matter", async () => {
    const source = "const s = css`---\ntitle: x\n---\n.a { color: red }`;\n";
    const result = await format("a.ts", source);
    expect(result.errors).toStrictEqual([]);
    // The fragment is preserved wholesale (no file envelope semantics).
    expect(result.code).toBe(source);
  });

  it("keeps a JSDoc css fence with front matter verbatim", async () => {
    const source = [
      "/**",
      " * ```css",
      " * ---",
      " * title: x",
      " * ---",
      " * .a { color: red }",
      " * ```",
      " */",
      "",
    ].join("\n");
    const result = await format("a.ts", source, { jsdoc: {} });
    expect(result.errors).toStrictEqual([]);
    expect(result.code).toBe(source);
  });
});
