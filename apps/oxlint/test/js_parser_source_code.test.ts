// Tests for `JS_PARSER_SOURCE_CODE`, the `SourceCode` implementation for files parsed by
// custom (JS) parsers.
//
// Token / comment / location methods are compared against ESLint's own `SourceCode`
// (constructed over the same AST), to ensure behavior matches ESLint exactly.

import { parseForESLint } from "@typescript-eslint/parser";
import { SourceCode as ESLintSourceCode } from "eslint";
import { beforeAll, describe, expect, it } from "vitest";
import { compileJsVisitors, walkParserAst } from "../src-js/plugins/js_ast_walk.ts";
import {
  getJsParserVisitorKeys,
  JS_PARSER_SOURCE_CODE,
  resetJsParserSourceCode,
  setupJsParserSourceCode,
} from "../src-js/plugins/js_parser_source_code.ts";
import { setGlobalsForFile, resetGlobals } from "../src-js/plugins/globals.ts";
import { resetSourceAndAst, setSourceTextForJsParser } from "../src-js/plugins/source_code.ts";

import type { JsParserNode, JsParserParseResult } from "../src-js/plugins/parsers.ts";

const CODE = [
  "// leading comment",
  "const a = 1; /* mid */ const b = a + 2;",
  "",
  "/* block",
  "   comment */",
  "function foo(x, y) {",
  "  // inner comment",
  "  return x + y; // trailing",
  "}",
  "foo(a, b);",
  "",
].join("\n");

let eslintSourceCode: ESLintSourceCode;
let ast: JsParserParseResult["ast"];
let parseResult: JsParserParseResult;

beforeAll(() => {
  parseResult = parseForESLint(CODE, {
    range: true,
    loc: true,
    tokens: true,
    comment: true,
  }) as unknown as JsParserParseResult;
  ast = parseResult.ast;

  eslintSourceCode = new ESLintSourceCode({
    text: CODE,
    ast: ast as unknown as ConstructorParameters<typeof ESLintSourceCode>[0]["ast"],
  });

  // `lint_js_parser.ts` sets the source text on `source_code.ts` before setup; the shared
  // `location.ts` line tables (used by `lines` / `getLocFromIndex` / `getIndexFromLoc`) read
  // from there.
  setSourceTextForJsParser(CODE);
  setupJsParserSourceCode(parseResult, CODE, false);
  setGlobalsForFile(JSON.stringify({ globals: {}, envs: {} }));

  // Walk the AST to set `parent` pointers on all nodes, as `lint_js_parser.ts` does
  // before rules run. `getScope` and `getAncestors` rely on `parent` pointers.
  walkParserAst(ast, getJsParserVisitorKeys(), compileJsVisitors([{ Program() {} }]));
});

// Sample nodes for token method calls
function getNodes() {
  const body = ast.body as JsParserNode[];
  const varDeclA = body[0];
  const varDeclB = body[1];
  const fnDecl = body[2];
  const callStmt = body[3];
  return { program: ast, varDeclA, varDeclB, fnDecl, callStmt };
}

describe("JS_PARSER_SOURCE_CODE", () => {
  it("exposes text, ast, lines, and lineStartIndices", () => {
    expect(JS_PARSER_SOURCE_CODE.text).toBe(CODE);
    expect(JS_PARSER_SOURCE_CODE.ast).toBe(ast);
    expect(JS_PARSER_SOURCE_CODE.hasBOM).toBe(false);
    expect(JS_PARSER_SOURCE_CODE.isESTree).toBe(true);
    expect(JS_PARSER_SOURCE_CODE.lines).toEqual(eslintSourceCode.lines);
    expect(JS_PARSER_SOURCE_CODE.lineStartIndices).toEqual(
      (eslintSourceCode as unknown as { lineStartIndices: number[] }).lineStartIndices,
    );
  });

  it("hasBOM reflects the flag passed to setupJsParserSourceCode", () => {
    // Rust strips the BOM before sending source text, but reports whether the original file
    // had one. Re-setup with the flag set, then restore shared state for the other tests.
    setupJsParserSourceCode(parseResult, CODE, true);
    expect(JS_PARSER_SOURCE_CODE.hasBOM).toBe(true);
    setupJsParserSourceCode(parseResult, CODE, false);
    expect(JS_PARSER_SOURCE_CODE.hasBOM).toBe(false);
  });

  it("getLocFromIndex / getIndexFromLoc match ESLint", () => {
    for (const index of [0, 1, 5, 19, 20, CODE.length - 1, CODE.length]) {
      expect(JS_PARSER_SOURCE_CODE.getLocFromIndex(index)).toEqual(
        eslintSourceCode.getLocFromIndex(index),
      );
    }
    for (const loc of [
      { line: 1, column: 0 },
      { line: 2, column: 5 },
      { line: 6, column: 9 },
    ]) {
      expect(JS_PARSER_SOURCE_CODE.getIndexFromLoc(loc)).toBe(
        eslintSourceCode.getIndexFromLoc(loc),
      );
    }
    expect(() => JS_PARSER_SOURCE_CODE.getLocFromIndex(CODE.length + 1)).toThrow(RangeError);
    expect(() => JS_PARSER_SOURCE_CODE.getIndexFromLoc({ line: 0, column: 0 })).toThrow(RangeError);
  });

  it("getText matches ESLint", () => {
    const { varDeclA, fnDecl } = getNodes();
    expect(JS_PARSER_SOURCE_CODE.getText()).toBe(eslintSourceCode.getText());
    for (const node of [varDeclA, fnDecl]) {
      const eslintNode = node as unknown as Parameters<typeof eslintSourceCode.getText>[0];
      expect(JS_PARSER_SOURCE_CODE.getText(node)).toBe(eslintSourceCode.getText(eslintNode));
      expect(JS_PARSER_SOURCE_CODE.getText(node, 5, 3)).toBe(
        eslintSourceCode.getText(eslintNode, 5, 3),
      );
    }
  });

  it("token methods match ESLint", () => {
    const { program, varDeclA, varDeclB, fnDecl, callStmt } = getNodes();
    const e = eslintSourceCode as unknown as Record<string, (...args: unknown[]) => unknown>;
    const o = JS_PARSER_SOURCE_CODE as unknown as Record<string, (...args: unknown[]) => unknown>;

    const isPunctuator = (token: { type: string }) => token.type === "Punctuator";

    // [methodName, ...args]
    const singleNodeCalls: [string, ...unknown[]][] = [];
    for (const node of [program, varDeclA, varDeclB, fnDecl, callStmt]) {
      for (const options of [
        undefined,
        1,
        2,
        { skip: 1 },
        { includeComments: true },
        { includeComments: true, skip: 2 },
        { filter: isPunctuator },
        isPunctuator,
      ]) {
        singleNodeCalls.push(["getFirstToken", node, options]);
        singleNodeCalls.push(["getLastToken", node, options]);
        singleNodeCalls.push(["getTokenBefore", node, options]);
        singleNodeCalls.push(["getTokenAfter", node, options]);
      }
      for (const options of [
        undefined,
        2,
        { count: 3 },
        { includeComments: true },
        { includeComments: true, count: 4 },
        { filter: isPunctuator },
        { filter: isPunctuator, count: 2 },
      ]) {
        singleNodeCalls.push(["getFirstTokens", node, options]);
        singleNodeCalls.push(["getLastTokens", node, options]);
        singleNodeCalls.push(["getTokensBefore", node, options]);
        singleNodeCalls.push(["getTokensAfter", node, options]);
      }
      singleNodeCalls.push(["getTokens", node]);
      singleNodeCalls.push(["getTokens", node, 2, 3]);
      singleNodeCalls.push(["getTokens", node, { includeComments: true }]);
      singleNodeCalls.push(["getCommentsBefore", node]);
      singleNodeCalls.push(["getCommentsAfter", node]);
      singleNodeCalls.push(["getCommentsInside", node]);
    }

    for (const [method, ...args] of singleNodeCalls) {
      expect(o[method](...args)).toEqual(e[method](...args));
    }

    // Between methods
    const betweenCalls: [string, ...unknown[]][] = [];
    for (const [left, right] of [
      [varDeclA, varDeclB],
      [varDeclA, fnDecl],
      [varDeclB, callStmt],
    ]) {
      for (const options of [undefined, 1, { includeComments: true }, { skip: 1 }]) {
        betweenCalls.push(["getFirstTokenBetween", left, right, options]);
        betweenCalls.push(["getLastTokenBetween", left, right, options]);
      }
      for (const options of [undefined, 2, { includeComments: true }, { count: 2 }]) {
        betweenCalls.push(["getFirstTokensBetween", left, right, options]);
        betweenCalls.push(["getLastTokensBetween", left, right, options]);
      }
      betweenCalls.push(["getTokensBetween", left, right]);
      betweenCalls.push(["getTokensBetween", left, right, 2]);
      betweenCalls.push(["commentsExistBetween", left, right]);
    }

    for (const [method, ...args] of betweenCalls) {
      expect(o[method](...args)).toEqual(e[method](...args));
    }
  });

  it("getTokenByRangeStart matches ESLint", () => {
    const { varDeclA } = getNodes();
    const commentStart = (ast.comments![0] as { range: [number, number] }).range[0];

    for (const [offset, options] of [
      [varDeclA.range[0], undefined],
      [varDeclA.range[0] + 1, undefined],
      [commentStart, undefined],
      [commentStart, { includeComments: true }],
    ] as [number, { includeComments?: boolean } | undefined][]) {
      expect(JS_PARSER_SOURCE_CODE.getTokenByRangeStart(offset, options)).toEqual(
        (
          eslintSourceCode.getTokenByRangeStart as (
            offset: number,
            options?: { includeComments?: boolean },
          ) => unknown
        )(offset, options),
      );
    }
  });

  it("isSpaceBetween matches ESLint", () => {
    const { varDeclA, varDeclB, fnDecl, callStmt } = getNodes();
    const tokens = ast.tokens!;

    const pairs: [unknown, unknown][] = [
      [varDeclA, varDeclB],
      [varDeclB, varDeclA],
      [fnDecl, callStmt],
      [tokens[0], tokens[1]],
      [tokens[1], tokens[2]],
      [varDeclA, varDeclA],
    ];
    for (const [first, second] of pairs) {
      expect(
        JS_PARSER_SOURCE_CODE.isSpaceBetween(
          first as Parameters<typeof JS_PARSER_SOURCE_CODE.isSpaceBetween>[0],
          second as Parameters<typeof JS_PARSER_SOURCE_CODE.isSpaceBetween>[1],
        ),
      ).toBe(
        eslintSourceCode.isSpaceBetween(
          first as Parameters<typeof eslintSourceCode.isSpaceBetween>[0],
          second as Parameters<typeof eslintSourceCode.isSpaceBetween>[1],
        ),
      );
    }
  });

  it("tokensAndComments and getAllComments match ESLint", () => {
    expect(JS_PARSER_SOURCE_CODE.tokensAndComments).toEqual(
      (eslintSourceCode as unknown as { tokensAndComments: unknown[] }).tokensAndComments,
    );
    expect(JS_PARSER_SOURCE_CODE.getAllComments()).toEqual(eslintSourceCode.getAllComments());
  });

  it("getNodeByRangeIndex matches ESLint", () => {
    for (const index of [0, 5, 25, 30, 60, 100, CODE.length - 2]) {
      const ours = JS_PARSER_SOURCE_CODE.getNodeByRangeIndex(index);
      const theirs = eslintSourceCode.getNodeByRangeIndex(index);
      expect(ours, `index ${index}`).toBe(theirs as unknown as JsParserNode | null);
    }
    expect(JS_PARSER_SOURCE_CODE.getNodeByRangeIndex(CODE.length + 10)).toBe(null);
  });

  it("getAncestors returns ancestors via parent chain", () => {
    const { fnDecl } = getNodes();
    // `parent` pointers were set by the walk in `beforeAll`
    const fnBody = fnDecl.body as JsParserNode;
    expect(JS_PARSER_SOURCE_CODE.getAncestors(fnBody)).toEqual([ast, fnDecl]);
  });

  it("scope methods work (parser-provided scope manager)", () => {
    const { program, fnDecl } = getNodes();

    const programScope = JS_PARSER_SOURCE_CODE.getScope(program);
    expect(programScope.type).toBe("global");

    const fnScope = JS_PARSER_SOURCE_CODE.getScope(fnDecl.body as JsParserNode);
    expect(fnScope.type).toBe("function");
    expect(fnScope.variables.map((v) => v.name)).toContain("x");

    const declared = JS_PARSER_SOURCE_CODE.getDeclaredVariables(fnDecl);
    expect(declared.map((v) => v.name)).toEqual(["foo", "x", "y"]);

    expect(JS_PARSER_SOURCE_CODE.markVariableAsUsed("a")).toBe(true);
    expect(JS_PARSER_SOURCE_CODE.markVariableAsUsed("nonexistent")).toBe(false);
  });

  it("getDisableDirectives returns empty result", () => {
    expect(JS_PARSER_SOURCE_CODE.getDisableDirectives()).toEqual({ problems: [], directives: [] });
  });

  it("getJSDocComment throws (not supported, deprecated)", () => {
    const { fnDecl } = getNodes();
    expect(() => JS_PARSER_SOURCE_CODE.getJSDocComment(fnDecl)).toThrow(
      "`sourceCode.getJSDocComment` is not supported at present (and deprecated)",
    );
  });
});

describe("reset", () => {
  it("resetJsParserSourceCode clears state", () => {
    resetJsParserSourceCode();
    // Production resets the shared `source_code.ts` / `location.ts` state via `resetFile`
    resetSourceAndAst();
    resetGlobals();
    // Tests run in debug build, where accessing `text` after reset fails a debug assertion
    expect(() => JS_PARSER_SOURCE_CODE.text).toThrow("Expected non-null value");
  });
});
