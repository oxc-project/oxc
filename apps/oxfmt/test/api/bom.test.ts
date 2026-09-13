import { describe, expect, it } from "vitest";
import { format } from "../../dist/index.js";

// Pins the physical-file BOM contract (`oxc_formatter_core::spec::split_bom`):
// each Rust formatter strips the whole leading BOM run before parsing and
// re-emits a single BOM at byte 0 of the output.
describe("BOM handling", () => {
  const cases: [string, string, string][] = [
    ["a.js", "\uFEFFlet a = 1", "\uFEFFlet a = 1;\n"],
    ["a.css", "\uFEFFa { color: red }", "\uFEFFa {\n  color: red;\n}\n"],
    ["a.yaml", "\uFEFFkey:   value", "\uFEFFkey: value\n"],
    ["a.graphql", "\uFEFF{ a }", "\uFEFF{\n  a\n}\n"],
    ["a.json", '\uFEFF{"a":1}', '\uFEFF{ "a": 1 }\n'],
  ];

  it.each(cases)("preserves a leading BOM in %s", async (filename, source, expected) => {
    const result = await format(filename, source);
    expect(result.errors).toStrictEqual([]);
    expect(result.code).toBe(expected);
  });

  it.each(cases)("swallows a doubled BOM in %s down to one", async (filename, source, expected) => {
    const result = await format(filename, "\uFEFF" + source);
    expect(result.errors).toStrictEqual([]);
    expect(result.code).toBe(expected);
  });

  it("keeps a BOM-only file as-is", async () => {
    const result = await format("a.js", "\uFEFF");
    expect(result.errors).toStrictEqual([]);
    expect(result.code).toBe("\uFEFF");
  });

  // In an embedded position U+FEFF is content, not a BOM:
  // `FormatSession::dispatch` answers `PreserveOriginal`,
  // so the part stays verbatim instead of losing the character.
  it("preserves a BOM-headed embedded part verbatim (css-in-js)", async () => {
    const source = "const s = css`\uFEFFa { color :  red }`;\n";
    const result = await format("a.js", source);
    expect(result.errors).toStrictEqual([]);
    expect(result.code).toBe(source);
  });

  it("preserves a BOM-headed front matter verbatim while the css body still formats", async () => {
    const result = await format("a.css", "---\n\uFEFFkey:   value\n---\na { color :  red }\n");
    expect(result.errors).toStrictEqual([]);
    expect(result.code).toBe("---\n\uFEFFkey:   value\n---\n\na {\n  color: red;\n}\n");
  });
});
