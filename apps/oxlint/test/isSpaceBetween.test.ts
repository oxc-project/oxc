import assert from "node:assert";
import { describe, it, expect, beforeEach } from "vitest";
import { parse as parseRaw } from "../src-js/package/parse.ts";
import { setupFileContext, resetFileContext } from "../src-js/plugins/context.ts";
import { buffers } from "../src-js/plugins/lint.ts";
import {
  ast,
  initAst,
  resetSourceAndAst,
  setupSourceForFile,
} from "../src-js/plugins/source_code.ts";
import { isSpaceBetween, isSpaceBetweenTokens } from "../src-js/plugins/tokens.ts";
import { debugAssertIsNonNull } from "../src-js/utils/asserts.ts";

import type { ParseOptions } from "../src-js/package/parse.ts";
import type { Program } from "../src-js/generated/types.d.ts";

/**
 * Parse source text into AST using Oxc parser.
 * Set up global state, as if was linting the provided file.
 * @param path - File path
 * @param sourceText - Source text
 * @param options - Parse options
 * @returns AST
 */
function parse(path: string, sourceText: string, options?: ParseOptions): Program {
  // Set file path
  setupFileContext(path);

  // Parse source, writing source text and AST into buffer
  parseRaw(path, sourceText, options);

  // Set buffer (`parseRaw` adds buffer containing AST to `buffers` at index 0)
  const buffer = buffers[0];
  debugAssertIsNonNull(buffer);
  setupSourceForFile(buffer, /* hasBOM */ false, /* parserServices */ {});

  // Deserialize AST from buffer
  initAst();
  debugAssertIsNonNull(ast);

  // Return AST
  return ast;
}

beforeEach(() => {
  resetFileContext();
  resetSourceAndAst();
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
        const ast = parse("dummy.js", code);

        const firstStmt = ast.body[0];
        const lastStmt = ast.body.at(-1);
        assert(firstStmt != null);
        assert(lastStmt != null);
        assert(firstStmt !== lastStmt);

        expect(isSpaceBetween(firstStmt, lastStmt)).toBe(expected);
        // Reversed order
        expect(isSpaceBetween(lastStmt, firstStmt)).toBe(expected);
      });
    }
  });
});

describe("isSpaceBetweenTokens()", () => {
  // https://github.com/eslint/eslint/blob/v9.39.1/tests/lib/languages/js/source-code/source-code.js#L2166-L2206
  it("JSXText tokens that contain only whitespaces should be handled as space", () => {
    const ast = parse("dummy.jsx", "let jsx = <div>\n   {content}\n</div>");

    const stmt = ast.body[0];
    assert.strictEqual(stmt.type, "VariableDeclaration");
    const jsx = stmt.declarations[0].init!;
    assert.strictEqual(jsx.type, "JSXElement");
    const { openingElement, closingElement } = jsx;
    assert(closingElement !== null);
    const interpolation = jsx.children[1];
    assert(interpolation != null);

    expect(isSpaceBetweenTokens(openingElement, interpolation)).toBe(true);
    expect(isSpaceBetweenTokens(interpolation, closingElement)).toBe(true);
    // Reversed order
    expect(isSpaceBetweenTokens(interpolation, openingElement)).toBe(true);
    expect(isSpaceBetweenTokens(closingElement, interpolation)).toBe(true);
  });

  // https://github.com/eslint/eslint/blob/v9.39.1/tests/lib/languages/js/source-code/source-code.js#L2208-L2233
  it("JSXText tokens that contain both letters and whitespaces should be handled as space", () => {
    const ast = parse("dummy.jsx", "let jsx = <div>\n   Hello\n</div>");

    const stmt = ast.body[0];
    assert.strictEqual(stmt.type, "VariableDeclaration");
    const jsx = stmt.declarations[0].init!;
    assert.strictEqual(jsx.type, "JSXElement");
    const { openingElement, closingElement } = jsx;
    assert(closingElement !== null);

    expect(isSpaceBetweenTokens(openingElement, closingElement)).toBe(true);
    // Reversed order
    expect(isSpaceBetweenTokens(closingElement, openingElement)).toBe(true);
  });

  // https://github.com/eslint/eslint/blob/v9.39.1/tests/lib/languages/js/source-code/source-code.js#L2235-L2261
  it("JSXText tokens that contain only letters should NOT be handled as space", () => {
    const ast = parse("dummy.jsx", "let jsx = <div>Hello</div>");

    const stmt = ast.body[0];
    assert.strictEqual(stmt.type, "VariableDeclaration");
    const jsx = stmt.declarations[0].init!;
    assert.strictEqual(jsx.type, "JSXElement");
    const { openingElement, closingElement } = jsx;
    assert(closingElement !== null);

    expect(isSpaceBetweenTokens(openingElement, closingElement)).toBe(false);
    // Reversed order
    expect(isSpaceBetweenTokens(closingElement, openingElement)).toBe(false);
  });

  // https://github.com/eslint/eslint/blob/v9.39.1/tests/lib/languages/js/source-code/source-code.js#L2263-L2300
  it("should return false if either of the arguments' location is inside the other one", () => {
    const ast = parse("dummy.js", "let foo = bar;");

    const stmt = ast.body[0];
    assert(stmt != null);

    const firstToken = ast.tokens[0];
    const lastToken = ast.tokens.at(-1);
    assert(firstToken != null);
    assert(lastToken != null);
    assert(firstToken !== lastToken);

    expect(isSpaceBetweenTokens(firstToken, stmt)).toBe(false);
    expect(isSpaceBetweenTokens(lastToken, stmt)).toBe(false);
    // Reversed order
    expect(isSpaceBetweenTokens(stmt, firstToken)).toBe(false);
    expect(isSpaceBetweenTokens(stmt, lastToken)).toBe(false);
  });
});
