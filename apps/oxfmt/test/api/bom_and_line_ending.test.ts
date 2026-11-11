import { describe, expect, it } from "vitest";
import { format } from "../../dist/index.js";

// `conformance:fixture` checks every fixture's BOM / CRLF / CR variants, these pin the rest:
// a doubled BOM, a BOM-headed embedded part (invisible in a snapshot) and error label offsets.

// Entries parse a BOM-stripped or newline-normalized copy, but error labels must point into the original source
const shift = (labels: { start: number; end: number }[], by: number) =>
  labels.map((l) => ({ ...l, start: l.start + by, end: l.end + by }));

// Pins the physical-file BOM contract (`oxc_formatter_core::spec::split_bom`):
// each Rust formatter strips the whole leading BOM run before parsing and
// re-emits a single BOM at byte 0 of the output.
describe("BOM handling", () => {
  it.each([
    ["a.js", "let a = 1", "let a = 1;\n"],
    ["a.css", "a { color: red }", "a {\n  color: red;\n}\n"],
    ["a.yaml", "key:   value", "key: value\n"],
    ["a.graphql", "{ a }", "{\n  a\n}\n"],
    ["a.json", '{"a":1}', '{ "a": 1 }\n'],
    ["a.toml", "a   =   1", "a = 1\n"],
  ])("should swallow a doubled BOM in %s down to one", async (filename, source, expected) => {
    const result = await format(filename, "\uFEFF\uFEFF" + source);
    expect(result.errors).toStrictEqual([]);
    expect(result.code).toBe("\uFEFF" + expected);
  });

  it.each([
    ["a.js", "let = ;"],
    ["a.css", "a {"],
    ["a.yaml", "a: ["],
    ["a.graphql", "{ a"],
    ["a.json", '{"a": }'],
    ["a.json", '{"a": 1 + 2}'],
    ["a.toml", "a = "],
  ])("should shift error labels in %s past the BOM", async (filename, source) => {
    const plain = await format(filename, source);
    const withBom = await format(filename, "\uFEFF" + source);
    // Labels are UTF-8 byte offsets, a BOM is 3 bytes
    expect(withBom.errors[0].labels).toStrictEqual(shift(plain.errors[0].labels, 3));
  });

  // In an embedded position U+FEFF is content, not a BOM:
  // `FormatSession::dispatch` answers `PreserveOriginal`,
  // so the part stays verbatim instead of losing the character.
  it("should keep a BOM-headed css-in-js template as-is", async () => {
    const source = "const s = css`\uFEFFa { color :  red }`;\n";
    const result = await format("a.js", source);
    expect(result.errors).toStrictEqual([]);
    expect(result.code).toBe(source);
  });

  it("should keep a BOM-headed front matter as-is and format the CSS body", async () => {
    const result = await format("a.css", "---\n\uFEFFkey:   value\n---\na { color :  red }\n");
    expect(result.errors).toStrictEqual([]);
    expect(result.code).toBe("---\n\uFEFFkey:   value\n---\n\na {\n  color: red;\n}\n");
  });
});

describe("Line ending handling", () => {
  it.each([
    ["a.css", "a {}\nb {"],
    ["a.css", "\uFEFFa {}\nb {"],
    ["a.yaml", "a: 1\nb: ["],
  ])("should report error labels in %s against the CRLF source", async (filename, source) => {
    const lf = await format(filename, source);
    const crlf = await format(filename, source.replaceAll("\n", "\r\n"));
    // Each input has 1 line break before the error
    expect(crlf.errors[0].labels).toStrictEqual(shift(lf.errors[0].labels, 1));
  });
});
