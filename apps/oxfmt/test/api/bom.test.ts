import { describe, expect, it } from "vitest";
import { format } from "../../dist/index.js";

// Pins the physical-file BOM contract (`oxc_formatter_core::spec::split_bom`):
// each Rust formatter strips the BOM before parsing and re-emits it exactly once at byte 0 of the output.
describe("BOM handling", () => {
  it("preserves a leading BOM across Tier 1 languages", async () => {
    const cases: [string, string, string][] = [
      ["a.js", "\uFEFFlet a = 1", "\uFEFFlet a = 1;\n"],
      ["a.css", "\uFEFFa { color: red }", "\uFEFFa {\n  color: red;\n}\n"],
      ["a.yaml", "\uFEFFkey:   value", "\uFEFFkey: value\n"],
      ["a.graphql", "\uFEFF{ a }", "\uFEFF{\n  a\n}\n"],
      ["a.json", '\uFEFF{"a":1}', '\uFEFF{ "a": 1 }\n'],
    ];
    for (const [filename, source, expected] of cases) {
      // oxlint-disable-next-line no-await-in-loop
      const result = await format(filename, source);
      expect(result.errors).toStrictEqual([]);
      expect(result.code).toBe(expected);
    }
  });

  it("keeps a BOM-only file as-is", async () => {
    const result = await format("a.js", "\uFEFF");
    expect(result.errors).toStrictEqual([]);
    expect(result.code).toBe("\uFEFF");
  });

  it("pins the doubled-BOM behavior (only the first is the BOM; the second reaches the parser)", async () => {
    // Both parsers treat the second U+FEFF as ordinary whitespace,
    // so it is swallowed and a single BOM comes back out.
    const js = await format("a.js", "\uFEFF\uFEFFlet a = 1");
    expect(js.errors).toStrictEqual([]);
    expect(js.code).toBe("\uFEFFlet a = 1;\n");

    const yaml = await format("a.yaml", "\uFEFF\uFEFFkey: value");
    expect(yaml.errors).toStrictEqual([]);
    expect(yaml.code).toBe("\uFEFFkey: value\n");
  });
});
