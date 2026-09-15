import { describe, expect, test } from "vitest";

import { parseSync } from "./parser.ts";
import type { ExpressionStatement, ParserOptions } from "./parser.ts";

const modes: [string, ParserOptions][] = [["JSON", {}]];
if (process.env.RUN_RAW_TESTS === "true") {
  modes.push(["raw", { experimentalRawTransfer: true }]);
}
if (process.env.RUN_LAZY_TESTS === "true") {
  modes.push(["lazy", { experimentalLazy: true }]);
}

describe.each(modes)("JavaScript strings (%s)", (_mode, options) => {
  test.each([String.raw`"\u0077ith"`, String.raw`"\uD800"`])(
    "recovers from an invalid import-type option key: %s",
    (key) => {
      const result = parseSync(
        "test.ts",
        `type T = import("x", { ${key}: { type: "json" } });`,
        options,
      );
      expect(result.errors).toHaveLength(1);
      expect(result.errors[0].message).toBe("Expected 'with' in import type options");
      expect(result.program.body).toHaveLength(1);
    },
  );

  describe.each(["js", "ts"])("%s", (extension) => {
    test("preserves UTF-16 code units and literal replacement characters", () => {
      const source = String.raw`["\uD800", "\uDC00", "\uD800\uDC00", "\uFFFDd800", "\uFFFD", "\uD800\0\n\"\\", "漢😎"]`;
      const result = parseSync(`test.${extension}`, source, options);
      expect(result.errors).toEqual([]);
      expect((result.program.body[0] as ExpressionStatement).expression).toMatchObject({
        type: "ArrayExpression",
        elements: [
          "\uD800",
          "\uDC00",
          "\uD800\uDC00",
          "\uFFFDd800",
          "\uFFFD",
          '\uD800\0\n"\\',
          "漢😎",
        ].map((value) => ({ value })),
      });
    });

    test("decodes long mixed strings containing lone surrogates", () => {
      const value = "aé漢😎\uD800\uFFFDd800\uDC00".repeat(1000);
      const result = parseSync(`test.${extension}`, `[${JSON.stringify(value)}]`, options);
      expect(result.errors).toEqual([]);
      expect((result.program.body[0] as ExpressionStatement).expression).toMatchObject({
        elements: [{ value }],
      });
    });

    test("distinguishes invalid cooked values, empty strings, and lone surrogates", () => {
      const source = "tag`\\u{${x}${y}\\uD800`;";
      const result = parseSync(`test.${extension}`, source, options);
      expect(result.errors).toEqual([]);
      expect((result.program.body[0] as ExpressionStatement).expression).toMatchObject({
        type: "TaggedTemplateExpression",
        quasi: {
          quasis: [
            { value: { raw: "\\u{", cooked: null } },
            { value: { raw: "", cooked: "" } },
            { value: { raw: "\\uD800", cooked: "\uD800" } },
          ],
        },
      });
    });

    test("preserves module request values in the AST and module record", () => {
      const source = String.raw`import "\uD800"; import "\uFFFDd800"; export * from "\uDC00";`;
      const result = parseSync(`test.${extension}`, source, options);
      expect(result.errors).toEqual([]);
      expect(result.program.body).toMatchObject([
        { source: { value: "\uD800" } },
        { source: { value: "\uFFFDd800" } },
        { source: { value: "\uDC00" } },
      ]);
      expect(result.module.staticImports.map((entry) => entry.moduleRequest.value)).toEqual([
        "\uD800",
        "\uFFFDd800",
      ]);
      expect(result.module.staticExports[0].entries[0].moduleRequest?.value).toBe("\uDC00");
    });

    test("reports invalid module export names without corrupting recovery", () => {
      const result = parseSync(
        `test.${extension}`,
        String.raw`export { value as "\uD800" };`,
        options,
      );
      expect(result.errors.length).toBeGreaterThan(0);
      expect(result.program.body).toMatchObject([
        { specifiers: [{ exported: { value: "\uD800" } }] },
      ]);
    });
  });
});
