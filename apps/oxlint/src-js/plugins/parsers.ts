/*
 * Loading and registry of custom (JS) parsers.
 *
 * Custom parsers are configured via `languageOptions.parser` in a config override
 * (ESLint compatibility). Files matched by such an override are parsed on JS side
 * by the custom parser, and linted by JS plugin rules (see `lint_js_parser.ts`).
 */

import { getErrorMessage } from "../utils/utils.ts";

import type { Location } from "./location.ts";
import type { Scope, Variable } from "./scope.ts";

/**
 * AST node produced by a custom parser.
 *
 * Node types are not restricted to standard ESTree types - custom parsers can produce
 * arbitrary node types (e.g. Ember's `GlimmerTemplate`).
 */
export interface JsParserNode {
  type: string;
  range: [number, number];
  loc?: Location;
  parent?: JsParserNode | null;
  [key: string]: unknown;
}

/**
 * Token produced by a custom parser (in `ast.tokens` / `ast.comments`).
 */
export interface JsParserToken {
  type: string;
  value: string;
  range: [number, number];
  loc?: Location;
}

/**
 * `Program` AST node produced by a custom parser.
 *
 * The parser is called with `tokens: true, comment: true, range: true, loc: true`,
 * so ESLint-compatible parsers attach `tokens` and `comments` arrays to the `Program` node.
 */
export interface JsParserProgram extends JsParserNode {
  sourceType?: "script" | "module" | "commonjs";
  tokens?: JsParserToken[];
  comments?: JsParserToken[];
}

/**
 * Scope manager provided by a custom parser (or created by analyzing the parser's AST).
 *
 * This is the minimal interface required by `SourceCode`'s scope methods.
 * Both `eslint-scope` and `@typescript-eslint/scope-manager` scope managers conform to it.
 */
export interface JsParserScopeManager {
  scopes: Scope[];
  globalScope: Scope | null;
  acquire(node: JsParserNode, inner?: boolean): Scope | null;
  getDeclaredVariables(node: JsParserNode): Variable[];
}

/**
 * Result of a parser's `parseForESLint` method.
 */
export interface JsParserParseResult {
  ast: JsParserProgram;
  scopeManager?: JsParserScopeManager | null;
  visitorKeys?: Record<string, readonly string[]> | null;
  services?: Record<string, unknown> | null;
}

/**
 * Custom parser (ESLint `languageOptions.parser` compatible).
 *
 * Must have either a `parseForESLint` method, or a `parse` method.
 * If both are present, `parseForESLint` takes priority (same as ESLint).
 */
export interface Parser {
  parseForESLint?: (code: string, options?: Record<string, unknown>) => JsParserParseResult;
  parse?: (code: string, options?: Record<string, unknown>) => JsParserProgram;
}

// Parser objects for loaded parsers.
// Indexed by `parserId`, which is passed to `lintFileWithJsParser`.
// Rust side asserts that `parserId` returned by `loadParser` equals the number of parsers
// registered so far, so parsers are only ever appended to this array.
export const registeredParsers: Parser[] = [];

/**
 * Load a custom parser.
 *
 * Mirrors the structure of `loadPlugin` in `load.ts`.
 *
 * @param url - Absolute path of parser file as a `file://...` URL
 * @returns Parser ID or error serialized to JSON string
 */
export async function loadParser(url: string): Promise<string> {
  try {
    const parser = resolveParser(await import(url));
    registeredParsers.push(parser);
    return JSON.stringify({ Success: { parserId: registeredParsers.length - 1 } });
  } catch (err) {
    return JSON.stringify({ Failure: getErrorMessage(err) });
  }
}

/**
 * Resolve parser object from an imported module.
 *
 * The parser is either the module's default export, or the module namespace itself
 * (for CommonJS parsers with named `parseForESLint` / `parse` exports, like `@typescript-eslint/parser`).
 *
 * @param mod - Imported module namespace object
 * @returns Parser object
 * @throws {Error} If neither the default export nor the module itself is a valid parser
 */
function resolveParser(mod: unknown): Parser {
  const candidates = [(mod as { default?: unknown }).default, mod];
  for (const candidate of candidates) {
    if (candidate === null || (typeof candidate !== "object" && typeof candidate !== "function")) {
      continue;
    }
    const parser = candidate as Parser;
    if (typeof parser.parseForESLint === "function" || typeof parser.parse === "function") {
      return parser;
    }
  }

  throw new Error("Parser must have a `parseForESLint` or `parse` method");
}
