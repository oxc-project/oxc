import { describe, it, vi, expect, beforeEach } from "vitest";
import { isSpaceBetween, isSpaceBetweenTokens } from "../src-js/plugins/tokens.ts";
import { resetSourceAndAst } from "../src-js/plugins/source_code.ts";
import { parse } from "@typescript-eslint/typescript-estree";

import type { Node } from "../src-js/plugins/types.ts";

let sourceText: string | null = null;

vi.mock("../src-js/plugins/source_code.ts", async (importOriginal) => {
  const original: Record<string, unknown> = await importOriginal();
  return {
    ...original,
    get sourceText(): string {
      if (sourceText === null) {
        throw new Error("Must set `sourceText` before calling token methods");
      }
      return sourceText;
    },
  };
});

beforeEach(() => {
  resetSourceAndAst();
  sourceText = null;
});

describe("isSpaceBetween()", () => {
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
        sourceText = code;
        const ast = parse(sourceText, { range: true, sourceType: "module" }),
          body = ast.body as unknown as Node[];
        expect(isSpaceBetween(body[0]!, body.at(-1)!)).toBe(expected);
      });
    }
  });
});

describe("isSpaceBetweenTokens()", () => {
  // https://github.com/eslint/eslint/blob/v9.39.1/tests/lib/languages/js/source-code/source-code.js#L2166-L2206
  it("JSXText tokens that contain only whitespaces should be handled as space", () => {
    sourceText = "let jsx = <div>\n   {content}\n</div>";
    const ast = parse(sourceText, {
      range: true,
      sourceType: "module",
      jsx: true,
    });
    // @ts-expect-error
    const jsx = ast.body[0].declarations[0].init;
    const interpolation = jsx.children[1];

    expect(isSpaceBetweenTokens(jsx.openingElement, interpolation)).toBe(true);

    expect(isSpaceBetweenTokens(interpolation, jsx.closingElement)).toBe(true);

    // Reversed order
    expect(isSpaceBetweenTokens(interpolation, jsx.openingElement)).toBe(true);

    expect(isSpaceBetweenTokens(jsx.closingElement, interpolation)).toBe(true);
  });

  // https://github.com/eslint/eslint/blob/v9.39.1/tests/lib/languages/js/source-code/source-code.js#L2208-L2233
  it("JSXText tokens that contain both letters and whitespaces should be handled as space", () => {
    sourceText = "let jsx = <div>\n   Hello\n</div>";
    const ast = parse(sourceText, {
      range: true,
      sourceType: "module",
      jsx: true,
    });
    // @ts-expect-error
    const jsx = ast.body[0].declarations[0].init;

    expect(isSpaceBetweenTokens(jsx.openingElement, jsx.closingElement)).toBe(true);

    // Reversed order
    expect(isSpaceBetweenTokens(jsx.closingElement, jsx.openingElement)).toBe(true);
  });

  // https://github.com/eslint/eslint/blob/v9.39.1/tests/lib/languages/js/source-code/source-code.js#L2235-L2261
  it("JSXText tokens that contain only letters should NOT be handled as space", () => {
    sourceText = "let jsx = <div>Hello</div>";
    const ast = parse(sourceText, {
      range: true,
      sourceType: "module",
      jsx: true,
    });
    // @ts-expect-error
    const jsx = ast.body[0].declarations[0].init;

    expect(isSpaceBetweenTokens(jsx.openingElement, jsx.closingElement)).toBe(false);

    // Reversed order
    expect(isSpaceBetweenTokens(jsx.closingElement, jsx.openingElement)).toBe(false);
  });

  // https://github.com/eslint/eslint/blob/v9.39.1/tests/lib/languages/js/source-code/source-code.js#L2263-L2300
  it("should return false either of the arguments' location is inside the other one", () => {
    sourceText = "let foo = bar;";
    const ast = parse(sourceText, {
        range: true,
        tokens: true,
        sourceType: "module",
        jsx: true,
      }),
      body = ast.body as unknown as Node[],
      { tokens } = ast;

    expect(isSpaceBetweenTokens(tokens[0], body[0])).toBe(false);

    expect(isSpaceBetweenTokens(tokens.at(-1)!, body[0])).toBe(false);

    expect(isSpaceBetweenTokens(body[0], tokens[0])).toBe(false);

    expect(isSpaceBetweenTokens(body[0], tokens.at(-1)!)).toBe(false);
  });
});
