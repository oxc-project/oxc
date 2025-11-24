import { describe, it, vi, expect, beforeEach } from "vitest";
import { isSpaceBetween } from "../src-js/plugins/tokens.js";
import { resetSourceAndAst } from "../src-js/plugins/source_code";
import { parse } from "@typescript-eslint/typescript-estree";
import type { Node } from "../src-js/plugins/types.js";

let sourceText!: string;

vi.mock("../src-js/plugins/source_code.ts", async (importOriginal) => {
  const original: any = await importOriginal();
  return {
    ...original,
    get sourceText() {
      return sourceText;
    },
  };
});

beforeEach(() => {
  resetSourceAndAst();
  // @ts-expect-error
  sourceText = null;
});

// https://github.com/eslint/eslint/blob/df5566f826d9f5740546e473aa6876b1f7d2f12c/tests/lib/languages/js/source-code/source-code.js#L721-L745
describe("should return true when there is at least one whitespace character between two nodes", () => {
  for (const [code, expected] of [
    ["let foo = bar;let baz = qux;", false],
    ["let foo = bar;/**/let baz = qux;", false],
    ["let foo = bar;/* */let baz = qux;", false],
    ["let foo = bar; let baz = qux;", true],
    ["let foo = bar; /**/let baz = qux;", true],
    ["let foo = bar; /* */let baz = qux;", true],
    ["let foo = bar;/**/ let baz = qux;", true],
    ["let foo = bar;/* */ let baz = qux;", true],
    ["let foo = bar; /**/ let baz = qux;", true],
    ["let foo = bar; /* */ let baz = qux;", true],
    ["let foo = bar;\tlet baz = qux;", true],
    ["let foo = bar;\t/**/let baz = qux;", true],
    ["let foo = bar;\t/* */let baz = qux;", true],
    ["let foo = bar;/**/\tlet baz = qux;", true],
    ["let foo = bar;/* */\tlet baz = qux;", true],
    ["let foo = bar;\t/**/\tlet baz = qux;", true],
    ["let foo = bar;\t/* */\tlet baz = qux;", true],
    ["let foo = bar;\nlet baz = qux;", true],
    ["let foo = bar;\n/**/let baz = qux;", true],
    ["let foo = bar;\n/* */let baz = qux;", true],
    ["let foo = bar;/**/\nlet baz = qux;", true],
    ["let foo = bar;/* */\nlet baz = qux;", true],
    ["let foo = bar;\n/**/\nlet baz = qux;", true],
    ["let foo = bar;\n/* */\nlet baz = qux;", true],
    ["let foo = 1;let foo2 = 2; let foo3 = 3;", true],
  ] satisfies [string, boolean][]) {
    it(`should return ${expected} for ${code}`, () => {
      const { body } = parse(code, { range: true, sourceType: "module" });
      sourceText = code;
      expect(isSpaceBetween(body[0] as any as Node, body.at(-1) as any as Node)).toBe(expected);
    });
  }
});
