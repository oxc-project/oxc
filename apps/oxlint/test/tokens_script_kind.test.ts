/**
 * Tests for ScriptKind detection in token parsing.
 *
 * These tests verify that the correct TypeScript ScriptKind is used based on file extension.
 * Using the wrong ScriptKind (e.g., TSX for a .ts file) causes TypeScript to parse
 * generic arrow functions like `<T>() => {}` incorrectly, creating bogus JsxText tokens
 * that overlap with comments.
 */
import { beforeEach, describe, expect, it } from "vitest";
import {
  getTokens,
  initTokens,
  initTokensAndComments,
  resetTokens,
} from "../src-js/plugins/tokens.ts";
import { setupFileContext, resetFileContext } from "../src-js/plugins/context.ts";
import { buffers } from "../src-js/plugins/lint.ts";
import {
  initSourceText,
  resetSourceAndAst,
  setupSourceForFile,
} from "../src-js/plugins/source_code.ts";
import { parse as parseRaw } from "../src-js/package/parse.ts";
import { debugAssertIsNonNull } from "../src-js/utils/asserts.ts";

import type { Node } from "../src-js/plugins/types.ts";

/**
 * Setup for a test case with a specific file path.
 *
 * @param filePath - File path (extension determines ScriptKind)
 * @param sourceText - Source text to parse
 */
function setup(filePath: string, sourceText: string) {
  // Reset global state
  resetFileContext();
  resetSourceAndAst();
  resetTokens();

  // Set file path
  setupFileContext(filePath);

  // Parse source text into buffer
  parseRaw(filePath, sourceText);

  // Set buffer (`parseRaw` adds buffer containing AST to `buffers` at index 0)
  const buffer = buffers[0];
  debugAssertIsNonNull(buffer);
  setupSourceForFile(buffer, /* hasBOM */ false, /* parserServices */ {});

  // Initialize source text (deserialize from buffer)
  initSourceText();
}

describe("ScriptKind detection", () => {
  beforeEach(() => {
    resetFileContext();
    resetSourceAndAst();
    resetTokens();
  });

  // This source code triggers the bug when parsed as TSX:
  // TypeScript creates bogus JsxText tokens that include the comment,
  // which then overlap with Oxc's comment positions.
  const SOURCE_WITH_GENERIC_ARROW = `
const obj = {
  fn: <T>(arg: T): T => {
    return arg;
  },
};

// A comment after the object
export { obj };
`;

  describe("regression test - demonstrates the bug when using wrong ScriptKind", () => {
    it("should fail with TSX ScriptKind on .ts file content (proving the bug)", async () => {
      // This test demonstrates WHY the fix is needed.
      // When TypeScript parses a .ts file as TSX, generic arrow functions
      // create bogus JsxText tokens that overlap with comments.
      const ts = await import("typescript");

      const tsAst = ts.createSourceFile(
        "test.ts",
        SOURCE_WITH_GENERIC_ARROW,
        { languageVersion: ts.ScriptTarget.Latest },
        true,
        ts.ScriptKind.TSX, // WRONG: using TSX for .ts file
      );

      // Collect tokens
      const tokens: Array<{ kind: string; start: number; end: number }> = [];
      function walk(node: import("typescript").Node): void {
        const { kind } = node;
        if (
          kind >= ts.SyntaxKind.FirstToken &&
          kind <= ts.SyntaxKind.LastToken &&
          kind !== ts.SyntaxKind.EndOfFileToken
        ) {
          const start =
            kind === ts.SyntaxKind.JsxText ? node.getFullStart() : node.getStart(tsAst);
          const end = node.getEnd();
          if (start !== end) {
            tokens.push({ kind: ts.SyntaxKind[kind], start, end });
          }
        } else {
          node.getChildren(tsAst).forEach(walk);
        }
      }
      walk(tsAst);

      // Check for bogus JsxText tokens that overlap with comment position
      const jsxTextTokens = tokens.filter((t) => t.kind === "JsxText");
      // With TSX parsing, there WILL be JsxText tokens (the bug)
      expect(jsxTextTokens.length).toBeGreaterThan(0);

      // The JsxText token spans across the comment, causing overlap
      const commentStart = SOURCE_WITH_GENERIC_ARROW.indexOf("// A comment");
      const overlappingTokens = jsxTextTokens.filter(
        (t) => t.start < commentStart && t.end > commentStart,
      );
      expect(overlappingTokens.length).toBeGreaterThan(0);
    });

    it("should succeed with TS ScriptKind on .ts file content (the fix)", async () => {
      // This test shows that using the correct ScriptKind fixes the issue
      const ts = await import("typescript");

      const tsAst = ts.createSourceFile(
        "test.ts",
        SOURCE_WITH_GENERIC_ARROW,
        { languageVersion: ts.ScriptTarget.Latest },
        true,
        ts.ScriptKind.TS, // CORRECT: using TS for .ts file
      );

      // Collect tokens
      const tokens: Array<{ kind: string; start: number; end: number }> = [];
      function walk(node: import("typescript").Node): void {
        const { kind } = node;
        if (
          kind >= ts.SyntaxKind.FirstToken &&
          kind <= ts.SyntaxKind.LastToken &&
          kind !== ts.SyntaxKind.EndOfFileToken
        ) {
          const start =
            kind === ts.SyntaxKind.JsxText ? node.getFullStart() : node.getStart(tsAst);
          const end = node.getEnd();
          if (start !== end) {
            tokens.push({ kind: ts.SyntaxKind[kind], start, end });
          }
        } else {
          node.getChildren(tsAst).forEach(walk);
        }
      }
      walk(tsAst);

      // With correct TS parsing, there should be NO JsxText tokens
      const jsxTextTokens = tokens.filter((t) => t.kind === "JsxText");
      expect(jsxTextTokens.length).toBe(0);
    });
  });

  describe("generic arrow functions with comments", () => {

    it("should parse .ts file without token/comment overlap", () => {
      setup("test.ts", SOURCE_WITH_GENERIC_ARROW);

      // Initialize tokens - this is where the overlap error would occur
      // if the wrong ScriptKind is used
      expect(() => initTokens()).not.toThrow();

      // Initialize tokens and comments merged - this triggers the full validation
      // that checks tokens and comments don't overlap
      expect(() => initTokensAndComments()).not.toThrow();

      // Verify tokens were generated
      const Program = { range: [0, SOURCE_WITH_GENERIC_ARROW.length] } as Node;
      const tokens = getTokens(Program);
      expect(tokens.length).toBeGreaterThan(0);

      // Verify no JsxText tokens exist (they shouldn't in a .ts file)
      const jsxTextTokens = tokens.filter((t) => t.type === "JSXText");
      expect(jsxTextTokens.length).toBe(0);
    });

    it("should parse .mts file without token/comment overlap", () => {
      setup("test.mts", SOURCE_WITH_GENERIC_ARROW);
      expect(() => initTokens()).not.toThrow();
    });

    it("should parse .cts file without token/comment overlap", () => {
      setup("test.cts", SOURCE_WITH_GENERIC_ARROW);
      expect(() => initTokens()).not.toThrow();
    });

    it("should parse .tsx file with actual JSX", () => {
      // TSX files need actual JSX syntax, not generic arrow functions
      // (which are ambiguous with JSX in TSX mode)
      const tsxSource = `
const Component = () => {
  return <div>Hello</div>;
};

// A comment after the component
export { Component };
`;
      setup("test.tsx", tsxSource);
      expect(() => initTokens()).not.toThrow();
    });

    it("should parse .js file without token/comment overlap", () => {
      // JS version without type annotations
      const jsSource = `
const obj = {
  fn: (arg) => {
    return arg;
  },
};

// A comment after the object
export { obj };
`;
      setup("test.js", jsSource);
      expect(() => initTokens()).not.toThrow();
    });

    it("should parse .jsx file without token/comment overlap", () => {
      const jsxSource = `
const obj = {
  fn: (arg) => {
    return arg;
  },
};

// A comment after the object
export { obj };
`;
      setup("test.jsx", jsxSource);
      expect(() => initTokens()).not.toThrow();
    });
  });

  describe("real-world reproduction case", () => {
    // This is a simplified version of the actual failing code from tracer.ts
    const TRACER_LIKE_SOURCE = `import { foo } from "bar";

function createTracer() {
  return {
    trace: <T>(name: string, fn: () => T): T => {
      return fn();
    },
    getCurrentSpan: () => {
      return null;
    },
  };
}

export const tracer = createTracer();

// For power users
export { foo };
`;

    it("should parse tracer-like .ts file without overlap error", () => {
      setup("tracer.ts", TRACER_LIKE_SOURCE);

      // This is where the original bug manifested:
      // "Overlapping token/comments: last end: X, next start: Y"
      expect(() => initTokens()).not.toThrow();
      expect(() => initTokensAndComments()).not.toThrow();

      const Program = { range: [0, TRACER_LIKE_SOURCE.length] } as Node;
      const tokens = getTokens(Program);

      // Verify we have the expected tokens without any bogus JsxText
      const jsxTextTokens = tokens.filter((t) => t.type === "JSXText");
      expect(jsxTextTokens.length).toBe(0);

      // Verify the generic type parameter `T` is parsed as an Identifier, not JSX
      const identifiers = tokens.filter((t) => t.type === "Identifier" && t.value === "T");
      expect(identifiers.length).toBeGreaterThan(0);
    });
  });

  describe("file extension edge cases", () => {
    const SIMPLE_SOURCE = "const x = 1;";

    it("should handle uppercase extensions", () => {
      setup("test.TS", SIMPLE_SOURCE);
      expect(() => initTokens()).not.toThrow();
    });

    it("should handle mixed case extensions", () => {
      setup("test.Tsx", SIMPLE_SOURCE);
      expect(() => initTokens()).not.toThrow();
    });

    it("should default to TSX for unknown extensions", () => {
      setup("test.unknown", SIMPLE_SOURCE);
      expect(() => initTokens()).not.toThrow();
    });

    it("should handle paths with multiple dots", () => {
      setup("test.spec.ts", SOURCE_WITH_GENERIC_ARROW);
      expect(() => initTokens()).not.toThrow();

      const Program = { range: [0, SOURCE_WITH_GENERIC_ARROW.length] } as Node;
      const tokens = getTokens(Program);
      const jsxTextTokens = tokens.filter((t) => t.type === "JSXText");
      expect(jsxTextTokens.length).toBe(0);
    });

    it("should handle absolute paths", () => {
      setup("/Users/test/project/src/file.ts", SOURCE_WITH_GENERIC_ARROW);
      expect(() => initTokens()).not.toThrow();
    });
  });
});
