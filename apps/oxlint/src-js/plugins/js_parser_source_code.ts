/*
 * ESLint-compatible `SourceCode` implementation over an AST produced by a custom (JS) parser.
 *
 * The standard `SourceCode` (`source_code.ts`) reads tokens, comments, and AST lazily from
 * the buffer sent from Rust. For files parsed by a custom parser there is no buffer -
 * everything comes from the parser's `parseForESLint` / `parse` output. This module provides
 * a `SourceCode` object with the same public API surface as `SOURCE_CODE`, backed by plain
 * JS arrays and objects.
 *
 * Only one file is linted at a time, so a single frozen singleton object `JS_PARSER_SOURCE_CODE`
 * is reused for all files, with state stored in module-level variables
 * (same pattern as `source_code.ts`).
 *
 * The token and comment methods are ported from ESLint's `TokenStore`:
 * ESLint code: https://github.com/eslint/eslint/blob/main/lib/languages/js/source-code/token-store
 * ESLint license (MIT): https://github.com/eslint/eslint/blob/main/LICENSE
 * `isSpaceBetween` is ported from ESLint's `SourceCode`:
 * https://github.com/eslint/eslint/blob/main/lib/languages/js/source-code/source-code.js
 */

import { analyze } from "@typescript-eslint/scope-manager";
import defaultVisitorKeys from "../generated/keys.ts";
import { getFallbackKeys } from "./js_ast_walk.ts";
import {
  getLineColumnFromOffset,
  getOffsetFromLineColumn,
  initLines,
  lines,
  lineStartIndices,
} from "./location.ts";
import { addGlobalsToScopeManager } from "./scope.ts";
import { debugAssertIsNonNull } from "../utils/asserts.ts";

import type {
  JsParserNode,
  JsParserParseResult,
  JsParserProgram,
  JsParserScopeManager,
  JsParserToken,
} from "./parsers.ts";
import type { Location, Range, Ranged } from "./location.ts";
import type { Scope, Variable } from "./scope.ts";

// A node, token, or comment - anything with a `range`
type RangedNodeOrToken = Ranged & { type?: string };

// Type of `filter` functions in token method options
type FilterFn = (tokenOrComment: JsParserToken) => boolean;

// Options for `getFirstToken` etc.
interface SkipOptions {
  includeComments?: boolean;
  filter?: FilterFn | null;
  skip?: number;
}

// Options for `getFirstTokens` etc.
interface CountOptions {
  includeComments?: boolean;
  filter?: FilterFn | null;
  count?: number;
}

type CursorWithSkipOptions = number | FilterFn | SkipOptions | null | undefined;
type CursorWithCountOptions = number | FilterFn | CountOptions | null | undefined;

// ---------------------------------------------------------------------------
// Module state. Set before linting a file by `setupJsParserSourceCode`.
// ---------------------------------------------------------------------------

// Source text of file
let text: string | null = null;
// AST produced by the parser
let program: JsParserProgram | null = null;
// Tokens and comments from the parser output, each sorted by range
let tokens: JsParserToken[] = [];
let comments: JsParserToken[] = [];
// Visitor keys: parser-provided keys merged over default keys. Set by `setupJsParserSourceCode`.
let mergedVisitorKeys: Record<string, readonly string[]> | null = null;
// Parser services from parser output
const EMPTY_PARSER_SERVICES: Readonly<Record<string, unknown>> = Object.freeze({});
let parserServices: Record<string, unknown> = EMPTY_PARSER_SERVICES as Record<string, unknown>;
// Scope manager provided by parser, or `null` if parser didn't provide one
let parserProvidedScopeManager: JsParserScopeManager | null = null;
// Resolved scope manager (parser-provided, or created by analyzing the AST), with configured
// globals added. Created lazily on first access.
let scopeManager: JsParserScopeManager | null = null;

// Lazily populated caches. `lines` / `lineStartIndices` line tables (and their `initLines`
// builder and `getLineColumnFromOffset` / `getOffsetFromLineColumn` accessors) are shared with
// the buffer-based path via `location.ts` - it reads the same source text (set by
// `setSourceTextForJsParser`) and is reset through `resetFile` -> `resetLinesAndLocs`.
let indexMap: Record<number, number> | null = null;
let tokensAndComments: JsParserToken[] | null = null;

/**
 * Set up source code for a file parsed by a custom (JS) parser.
 *
 * @param parseResult - Result of parser's `parseForESLint` method
 *   (for `parse`-only parsers, wrap the AST as `{ ast }`)
 * @param sourceText - Source text of file
 */
export function setupJsParserSourceCode(
  parseResult: JsParserParseResult,
  sourceText: string,
): void {
  text = sourceText;
  program = parseResult.ast;
  tokens = program.tokens ?? [];
  comments = program.comments ?? [];
  mergedVisitorKeys = mergeVisitorKeys(parseResult.visitorKeys);
  parserServices = parseResult.services ?? (EMPTY_PARSER_SERVICES as Record<string, unknown>);
  parserProvidedScopeManager = parseResult.scopeManager ?? null;
}

/**
 * Reset all state after file has been linted, to free memory.
 */
export function resetJsParserSourceCode(): void {
  text = null;
  program = null;
  tokens = [];
  comments = [];
  mergedVisitorKeys = null;
  parserServices = EMPTY_PARSER_SERVICES as Record<string, unknown>;
  parserProvidedScopeManager = null;
  scopeManager = null;
  indexMap = null;
  tokensAndComments = null;
  // `lines` / `lineStartIndices` are owned by `location.ts` and reset via `resetFile`
  // (`resetSourceAndAst` -> `resetLinesAndLocs`), so nothing to reset here.
}

/**
 * Get visitor keys for the current file (parser-provided keys merged over default keys).
 * Used by `lint_js_parser.ts` to walk the AST.
 * @returns Visitor keys
 */
export function getJsParserVisitorKeys(): Record<string, readonly string[]> {
  debugAssertIsNonNull(mergedVisitorKeys);
  return mergedVisitorKeys;
}

/**
 * Merge parser-provided visitor keys over the default visitor keys.
 *
 * For node types present in both, the keys are the union (default keys first),
 * like ESLint's `vk.unionWith`.
 *
 * @param parserVisitorKeys - Visitor keys provided by parser, or `null` / `undefined`
 * @returns Merged visitor keys
 */
function mergeVisitorKeys(
  parserVisitorKeys: Record<string, readonly string[]> | null | undefined,
): Record<string, readonly string[]> {
  if (parserVisitorKeys == null) {
    return defaultVisitorKeys as Record<string, readonly string[]>;
  }

  const merged: Record<string, readonly string[]> = {
    ...(defaultVisitorKeys as Record<string, readonly string[]>),
  };
  for (const type of Object.keys(parserVisitorKeys)) {
    const parserKeys = parserVisitorKeys[type];
    if (!Array.isArray(parserKeys)) continue;
    // `Object.hasOwn` check so a type named e.g. `constructor` doesn't pick up
    // `Object.prototype` members (same guard as ESLint's `vk.unionWith`)
    const existingKeys = Object.hasOwn(merged, type) ? merged[type] : undefined;
    merged[type] =
      existingKeys === undefined ? parserKeys : [...new Set([...existingKeys, ...parserKeys])];
  }
  return merged;
}

// ---------------------------------------------------------------------------
// Scopes
// ---------------------------------------------------------------------------

/**
 * Get scope manager for the file, creating it if necessary.
 *
 * If the parser provided a scope manager, that one is used.
 * Otherwise, the AST is analyzed with `@typescript-eslint/scope-manager`
 * (like `scope.ts` does for the buffer-based path).
 *
 * Configured globals are added to the scope manager's global scope (once).
 *
 * @returns Scope manager
 */
function getScopeManager(): JsParserScopeManager {
  if (scopeManager !== null) return scopeManager;

  debugAssertIsNonNull(program);
  debugAssertIsNonNull(mergedVisitorKeys);

  let manager = parserProvidedScopeManager;
  if (manager === null) {
    const { sourceType } = program;
    manager = analyze(program as never, {
      childVisitorKeys: mergedVisitorKeys as Record<string, string[]>,
      globalReturn: sourceType === "commonjs",
      impliedStrict: sourceType === "module",
      jsxPragma: "React",
      jsxFragmentName: null,
      lib: [],
      sourceType: sourceType ?? "script",
      emitDecoratorMetadata: false,
    }) as unknown as JsParserScopeManager;
  }

  // Add globals from configuration and resolve references
  addGlobalsToScopeManager(manager);

  scopeManager = manager;
  return manager;
}

/**
 * Get the scope for the given node.
 * Same algorithm as `scope.ts` `getScope`.
 * @param node - The node to get the scope of
 * @returns The scope information for this node
 */
function getScope(node: JsParserNode): Scope {
  if (!node) throw new TypeError("Missing required argument: `node`");

  const manager = getScopeManager();

  const inner = node.type !== "Program";

  // Traverse up the AST to find a node whose scope can be acquired
  let currentNode: JsParserNode | null | undefined = node;
  do {
    const scope = manager.acquire(currentNode, inner);
    if (scope !== null) {
      return scope.type === "function-expression-name" ? scope.childScopes[0] : scope;
    }

    currentNode = currentNode.parent;
  } while (currentNode != null);

  return manager.scopes[0];
}

/**
 * Get the variables that `node` defines.
 * @param node - The node for which the variables are obtained
 * @returns An array of variable objects representing the variables that `node` defines
 */
function getDeclaredVariables(node: JsParserNode): Variable[] {
  return getScopeManager().getDeclaredVariables(node);
}

/**
 * Determine whether the given identifier node is a reference to a global variable.
 * Same algorithm as `scope.ts` `isGlobalReference`.
 * @param node - `Identifier` node to check
 * @returns `true` if the identifier is a reference to a global variable
 */
function isGlobalReference(node: JsParserNode): boolean {
  if (!node) throw new TypeError("Missing required argument: `node`");
  if (node.type !== "Identifier") return false;

  const { scopes } = getScopeManager();
  if (scopes.length === 0) return false;
  const globalScope = scopes[0];

  // If the identifier is a reference to a global variable, the global scope should have a variable with the name
  const variable = globalScope.set.get(node.name as string);

  // Global variables are not defined by any node, so they should have no definitions
  if (variable === undefined || variable.defs.length > 0) return false;

  // If there is a variable by the same name exists in the global scope,
  // we need to check our node is one of its references
  const { references } = variable;
  for (let i = 0, len = references.length; i < len; i++) {
    if ((references[i].identifier as unknown) === node) return true;
  }

  return false;
}

/**
 * Marks as used a variable with the given name in a scope indicated by the given reference node.
 * Same algorithm as `scope.ts` `markVariableAsUsed`.
 * @param name - Variable name
 * @param refNode - Reference node. Defaults to `Program` node if not provided.
 * @returns `true` if a variable with the given name was found and marked as used, otherwise `false`
 */
function markVariableAsUsed(name: string, refNode?: JsParserNode): boolean {
  debugAssertIsNonNull(program);
  if (refNode === undefined) refNode = program;

  let currentScope = getScope(refNode);

  // When in the global scope, check if there's a module/function child scope whose `block`
  // is the `Program` node. See `scope.ts` `markVariableAsUsed` for explanation.
  if (currentScope.type === "global") {
    const { childScopes } = currentScope;
    if (childScopes.length !== 0) {
      // Top-level scopes refer to a `Program` node
      const firstChild = childScopes[0];
      if ((firstChild.block as unknown) === program) currentScope = firstChild;
    }
  }

  for (let scope: Scope | null = currentScope; scope !== null; scope = scope.upper) {
    const { variables } = scope;
    for (let i = 0, len = variables.length; i < len; i++) {
      const variable = variables[i];
      if (variable.name === name) {
        // @ts-expect-error - `eslintUsed` is a dynamic property not in the types
        variable.eslintUsed = true;
        return true;
      }
    }
  }

  return false;
}

// ---------------------------------------------------------------------------
// Token store.
//
// Ported from ESLint's `TokenStore`:
// https://github.com/eslint/eslint/blob/main/lib/languages/js/source-code/token-store
// ESLint license (MIT): https://github.com/eslint/eslint/blob/main/LICENSE
// ---------------------------------------------------------------------------

/**
 * Check if a token is a comment token.
 * Same as `@eslint-community/eslint-utils` `isCommentToken`.
 * @param token - Token to check
 * @returns `true` if `token` is a comment token
 */
function isCommentToken(token: JsParserToken): boolean {
  return token.type === "Line" || token.type === "Block" || token.type === "Shebang";
}

/**
 * Create the map from locations to indices in `tokens`.
 *
 * The first/last location of tokens is mapped to the index of the token.
 * The first/last location of comments is mapped to the index of the next token of each comment.
 *
 * @returns The map from locations to indices in `tokens`
 */
function getIndexMap(): Record<number, number> {
  if (indexMap !== null) return indexMap;

  const map: Record<number, number> = Object.create(null);
  let tokenIndex = 0;
  let commentIndex = 0;
  let nextStart;
  let range;

  while (tokenIndex < tokens.length || commentIndex < comments.length) {
    nextStart =
      commentIndex < comments.length ? comments[commentIndex].range[0] : Number.MAX_SAFE_INTEGER;
    while (tokenIndex < tokens.length && (range = tokens[tokenIndex].range)[0] < nextStart) {
      map[range[0]] = tokenIndex;
      map[range[1] - 1] = tokenIndex;
      tokenIndex += 1;
    }

    nextStart = tokenIndex < tokens.length ? tokens[tokenIndex].range[0] : Number.MAX_SAFE_INTEGER;
    while (
      commentIndex < comments.length &&
      (range = comments[commentIndex].range)[0] < nextStart
    ) {
      map[range[0]] = tokenIndex;
      map[range[1] - 1] = tokenIndex;
      commentIndex += 1;
    }
  }

  indexMap = map;
  return map;
}

/**
 * Find the index of the first token which is after the given location.
 * If it was not found, this returns `tokens.length`.
 * @param tokensOrComments - It searches the token in this list
 * @param location - The location to search
 * @returns The found index or `tokensOrComments.length`
 */
function search(tokensOrComments: JsParserToken[], location: number): number {
  for (let minIndex = 0, maxIndex = tokensOrComments.length - 1; minIndex <= maxIndex; ) {
    const index = ((minIndex + maxIndex) / 2) | 0;
    const token = tokensOrComments[index];
    const tokenStartLocation = token.range[0];

    if (location <= tokenStartLocation) {
      if (index === minIndex) return index;
      maxIndex = index;
    } else {
      minIndex = index + 1;
    }
  }
  return tokensOrComments.length;
}

/**
 * Get the index of the `startLoc` in `tokens`.
 * `startLoc` can be the value of `node.range[1]`, so this checks about `startLoc - 1` as well.
 * @param startLoc - The location to get an index
 * @returns The index
 */
function getFirstIndex(startLoc: number): number {
  const map = getIndexMap();
  if (startLoc === -1) return 0;
  if (startLoc in map) return map[startLoc];
  if (startLoc - 1 in map) {
    const index = map[startLoc - 1];
    const token = tokens[index];

    // If the mapped index is out of bounds, the returned cursor index will point after the end
    // of the tokens array
    if (!token) return tokens.length;

    // For the map of "comment's location -> token's index", it points the next token of a comment.
    // In that case, +1 is unnecessary.
    if (token.range[0] >= startLoc) return index;
    return index + 1;
  }

  // Program node that doesn't start/end with a token or comment
  if (startLoc === 0) return 0;
  return tokens.length;
}

/**
 * Get the index of the `endLoc` in `tokens`.
 * The information of end locations are recorded at `endLoc - 1` in the index map,
 * so this checks about `endLoc - 1` as well.
 * @param endLoc - The location to get an index
 * @returns The index
 */
function getLastIndex(endLoc: number): number {
  const map = getIndexMap();
  if (endLoc === -1) return tokens.length - 1;
  if (endLoc in map) return map[endLoc] - 1;
  if (endLoc - 1 in map) {
    const index = map[endLoc - 1];
    const token = tokens[index];

    // If the mapped index is out of bounds, the returned cursor index will point before the end
    // of the tokens array
    if (!token) return tokens.length - 1;

    // For the map of "comment's location -> token's index", it points the next token of a comment.
    // In that case, -1 is necessary.
    if (token.range[1] > endLoc) return index - 1;
    return index;
  }

  // Program node that doesn't start/end with a token or comment
  if (endLoc === 0) return -1;
  return tokens.length - 1;
}

/**
 * The abstract class about cursors which iterate tokens.
 * See ESLint's `token-store/cursor.js`.
 */
abstract class Cursor {
  current: JsParserToken | null = null;

  /**
   * Get the first token. This consumes this cursor.
   * @returns The first token or `null`
   */
  getOneToken(): JsParserToken | null {
    return this.moveNext() ? this.current : null;
  }

  /**
   * Get all tokens. This consumes this cursor.
   * @returns All tokens
   */
  getAllTokens(): JsParserToken[] {
    const result: JsParserToken[] = [];
    while (this.moveNext()) result.push(this.current!);
    return result;
  }

  /**
   * Move this cursor to the next token.
   * @returns `true` if the next token exists
   */
  abstract moveNext(): boolean;
}

/**
 * The cursor which iterates tokens only.
 */
class ForwardTokenCursor extends Cursor {
  index: number;
  indexEnd: number;

  constructor(startLoc: number, endLoc: number) {
    super();
    this.index = getFirstIndex(startLoc);
    this.indexEnd = getLastIndex(endLoc);
  }

  moveNext(): boolean {
    if (this.index <= this.indexEnd) {
      this.current = tokens[this.index];
      this.index += 1;
      return true;
    }
    return false;
  }

  // Shorthands for performance (same as ESLint)

  override getOneToken(): JsParserToken | null {
    return this.index <= this.indexEnd ? tokens[this.index] : null;
  }

  override getAllTokens(): JsParserToken[] {
    return tokens.slice(this.index, this.indexEnd + 1);
  }
}

/**
 * The cursor which iterates tokens only in reverse.
 */
class BackwardTokenCursor extends Cursor {
  index: number;
  indexEnd: number;

  constructor(startLoc: number, endLoc: number) {
    super();
    this.index = getLastIndex(endLoc);
    this.indexEnd = getFirstIndex(startLoc);
  }

  moveNext(): boolean {
    if (this.index >= this.indexEnd) {
      this.current = tokens[this.index];
      this.index -= 1;
      return true;
    }
    return false;
  }

  override getOneToken(): JsParserToken | null {
    return this.index >= this.indexEnd ? tokens[this.index] : null;
  }
}

/**
 * The cursor which iterates tokens and comments.
 */
class ForwardTokenCommentCursor extends Cursor {
  tokenIndex: number;
  commentIndex: number;
  border: number;

  constructor(startLoc: number, endLoc: number) {
    super();
    this.tokenIndex = getFirstIndex(startLoc);
    this.commentIndex = search(comments, startLoc);
    this.border = endLoc;
  }

  moveNext(): boolean {
    const token = this.tokenIndex < tokens.length ? tokens[this.tokenIndex] : null;
    const comment = this.commentIndex < comments.length ? comments[this.commentIndex] : null;

    if (token && (!comment || token.range[0] < comment.range[0])) {
      this.current = token;
      this.tokenIndex += 1;
    } else if (comment) {
      this.current = comment;
      this.commentIndex += 1;
    } else {
      this.current = null;
    }

    return this.current !== null && (this.border === -1 || this.current.range[1] <= this.border);
  }
}

/**
 * The cursor which iterates tokens and comments in reverse.
 */
class BackwardTokenCommentCursor extends Cursor {
  tokenIndex: number;
  commentIndex: number;
  border: number;

  constructor(startLoc: number, endLoc: number) {
    super();
    this.tokenIndex = getLastIndex(endLoc);
    this.commentIndex = search(comments, endLoc) - 1;
    this.border = startLoc;
  }

  moveNext(): boolean {
    const token = this.tokenIndex >= 0 ? tokens[this.tokenIndex] : null;
    const comment = this.commentIndex >= 0 ? comments[this.commentIndex] : null;

    if (token && (!comment || token.range[1] > comment.range[1])) {
      this.current = token;
      this.tokenIndex -= 1;
    } else if (comment) {
      this.current = comment;
      this.commentIndex -= 1;
    } else {
      this.current = null;
    }

    return this.current !== null && (this.border === -1 || this.current.range[0] >= this.border);
  }
}

/**
 * The abstract class about cursors which manipulate another cursor.
 */
abstract class DecorativeCursor extends Cursor {
  cursor: Cursor;

  constructor(cursor: Cursor) {
    super();
    this.cursor = cursor;
  }

  moveNext(): boolean {
    const retv = this.cursor.moveNext();
    this.current = this.cursor.current;
    return retv;
  }
}

/**
 * The decorative cursor which ignores specified tokens.
 */
class FilterCursor extends DecorativeCursor {
  predicate: FilterFn;

  constructor(cursor: Cursor, predicate: FilterFn) {
    super(cursor);
    this.predicate = predicate;
  }

  override moveNext(): boolean {
    const { predicate } = this;
    while (super.moveNext()) {
      if (predicate(this.current!)) return true;
    }
    return false;
  }
}

/**
 * The decorative cursor which ignores the first few tokens.
 */
class SkipCursor extends DecorativeCursor {
  count: number;

  constructor(cursor: Cursor, count: number) {
    super(cursor);
    this.count = count;
  }

  override moveNext(): boolean {
    while (this.count > 0) {
      this.count -= 1;
      if (!super.moveNext()) return false;
    }
    return super.moveNext();
  }
}

/**
 * The decorative cursor which limits the number of tokens.
 */
class LimitCursor extends DecorativeCursor {
  count: number;

  constructor(cursor: Cursor, count: number) {
    super(cursor);
    this.count = count;
  }

  override moveNext(): boolean {
    if (this.count > 0) {
      this.count -= 1;
      return super.moveNext();
    }
    return false;
  }
}

/**
 * The cursor which iterates tokens only, with inflated range.
 * This is for the backward compatibility of padding options.
 */
class PaddedTokenCursor extends ForwardTokenCursor {
  constructor(startLoc: number, endLoc: number, beforeCount: number, afterCount: number) {
    super(startLoc, endLoc);
    this.index = Math.max(0, this.index - beforeCount);
    this.indexEnd = Math.min(tokens.length - 1, this.indexEnd + afterCount);
  }
}

/**
 * Create a base cursor (tokens only, or tokens + comments), forward or backward.
 * Equivalent of ESLint's `CursorFactory#createBaseCursor`.
 *
 * @param forward - `true` to iterate forwards
 * @param startLoc - The start location of the iteration range
 * @param endLoc - The end location of the iteration range
 * @param includeComments - The flag to iterate comments as well
 * @returns The created cursor
 */
function createBaseCursor(
  forward: boolean,
  startLoc: number,
  endLoc: number,
  includeComments: boolean,
): Cursor {
  if (forward) {
    return includeComments
      ? new ForwardTokenCommentCursor(startLoc, endLoc)
      : new ForwardTokenCursor(startLoc, endLoc);
  }
  return includeComments
    ? new BackwardTokenCommentCursor(startLoc, endLoc)
    : new BackwardTokenCursor(startLoc, endLoc);
}

/**
 * Create a cursor that iterates tokens with normalized options.
 * Equivalent of ESLint's `CursorFactory#createCursor`.
 *
 * @param forward - `true` to iterate forwards
 * @param startLoc - The start location of the iteration range
 * @param endLoc - The end location of the iteration range
 * @param includeComments - The flag to iterate comments as well
 * @param filter - The predicate function to choose tokens
 * @param skip - The count of tokens the cursor skips
 * @param count - The maximum count of tokens the cursor iterates. `-1` is no limit.
 * @returns The created cursor
 */
function createCursor(
  forward: boolean,
  startLoc: number,
  endLoc: number,
  includeComments: boolean,
  filter: FilterFn | null,
  skip: number,
  count: number,
): Cursor {
  let cursor = createBaseCursor(forward, startLoc, endLoc, includeComments);
  if (filter) cursor = new FilterCursor(cursor, filter);
  if (skip >= 1) cursor = new SkipCursor(cursor, skip);
  if (count >= 0) cursor = new LimitCursor(cursor, count);
  return cursor;
}

/**
 * Create a cursor from options in "skip" form (`getFirstToken` etc.).
 * If options is a number then it's `options.skip`. If a function then it's `options.filter`.
 *
 * @param forward - `true` to iterate forwards
 * @param startLoc - The start location of the iteration range
 * @param endLoc - The end location of the iteration range
 * @param opts - Options
 * @returns The created cursor
 */
function createCursorWithSkip(
  forward: boolean,
  startLoc: number,
  endLoc: number,
  opts: CursorWithSkipOptions,
): Cursor {
  let includeComments = false;
  let skip = 0;
  let filter: FilterFn | null = null;

  if (typeof opts === "number") {
    skip = opts | 0;
  } else if (typeof opts === "function") {
    filter = opts;
  } else if (opts) {
    includeComments = !!opts.includeComments;
    skip = opts.skip! | 0;
    filter = opts.filter || null;
  }
  if (skip < 0) throw new Error("options.skip should be zero or a positive integer.");
  if (filter && typeof filter !== "function") {
    throw new Error("options.filter should be a function.");
  }

  return createCursor(forward, startLoc, endLoc, includeComments, filter, skip, -1);
}

/**
 * Create a cursor from options in "count" form (`getFirstTokens` etc.).
 * If options is a number then it's `options.count`. If a function then it's `options.filter`.
 *
 * @param forward - `true` to iterate forwards
 * @param startLoc - The start location of the iteration range
 * @param endLoc - The end location of the iteration range
 * @param opts - Options
 * @returns The created cursor
 */
function createCursorWithCount(
  forward: boolean,
  startLoc: number,
  endLoc: number,
  opts: CursorWithCountOptions,
): Cursor {
  let includeComments = false;
  let count = 0;
  let countExists = false;
  let filter: FilterFn | null = null;

  if (typeof opts === "number") {
    count = opts | 0;
    countExists = true;
  } else if (typeof opts === "function") {
    filter = opts;
  } else if (opts) {
    includeComments = !!opts.includeComments;
    count = opts.count! | 0;
    countExists = typeof opts.count === "number";
    filter = opts.filter || null;
  }
  if (count < 0) throw new Error("options.count should be zero or a positive integer.");
  if (filter && typeof filter !== "function") {
    throw new Error("options.filter should be a function.");
  }

  return createCursor(
    forward,
    startLoc,
    endLoc,
    includeComments,
    filter,
    0,
    countExists ? count : -1,
  );
}

/**
 * Create a cursor from options in "padding" form (`getTokens`, `getTokensBetween`).
 *
 * @param startLoc - The start location of the iteration range
 * @param endLoc - The end location of the iteration range
 * @param beforeCount - The number of tokens before the node to retrieve, or options object
 * @param afterCount - The number of tokens after the node to retrieve
 * @returns The created cursor
 */
function createCursorWithPadding(
  startLoc: number,
  endLoc: number,
  beforeCount: CursorWithCountOptions,
  afterCount: number | undefined,
): Cursor {
  if (typeof beforeCount === "undefined" && typeof afterCount === "undefined") {
    return new ForwardTokenCursor(startLoc, endLoc);
  }
  if (typeof beforeCount === "number" || typeof beforeCount === "undefined") {
    return new PaddedTokenCursor(startLoc, endLoc, beforeCount! | 0, afterCount! | 0);
  }
  return createCursorWithCount(true, startLoc, endLoc, beforeCount);
}

/**
 * Get comment tokens that are adjacent to the current cursor position.
 * @param cursor - A cursor instance
 * @returns An array of comment tokens adjacent to the current cursor position
 */
function getAdjacentCommentTokensFromCursor(cursor: Cursor): JsParserToken[] {
  const result: JsParserToken[] = [];
  let currentToken = cursor.getOneToken();

  while (currentToken && isCommentToken(currentToken)) {
    result.push(currentToken);
    currentToken = cursor.getOneToken();
  }

  return result;
}

// ---------------------------------------------------------------------------
// `isSpaceBetween`
// ---------------------------------------------------------------------------

const JSX_WHITESPACE_REGEXP = /\s/u;

/**
 * Check if two nodes or tokens overlap.
 * @param first - The first node or token to check
 * @param second - The second node or token to check
 * @returns `true` if the two nodes or tokens overlap
 */
function nodesOrTokensOverlap(first: RangedNodeOrToken, second: RangedNodeOrToken): boolean {
  return (
    (first.range[0] <= second.range[0] && second.range[0] < first.range[1]) ||
    (second.range[0] <= first.range[0] && first.range[0] < second.range[1])
  );
}

/**
 * Determine if two nodes or tokens have at least one whitespace character between them.
 * Order does not matter. Returns `false` if the given nodes or tokens overlap.
 *
 * Ported from ESLint's `SourceCode#isSpaceBetween` (ESLint 9 version, which also supported
 * checking for whitespace inside `JSXText` tokens for the deprecated `isSpaceBetweenTokens`).
 *
 * @param first - The first node or token to check between
 * @param second - The second node or token to check between
 * @param checkInsideOfJSXText - Whether to check whitespace inside `JSXText` tokens
 * @returns `true` if there is a whitespace character between the two nodes or tokens
 */
function isSpaceBetweenImpl(
  first: RangedNodeOrToken,
  second: RangedNodeOrToken,
  checkInsideOfJSXText: boolean,
): boolean {
  if (nodesOrTokensOverlap(first, second)) return false;

  const [startingNodeOrToken, endingNodeOrToken] =
    first.range[1] <= second.range[0] ? [first, second] : [second, first];
  const firstToken =
    getLastTokenImpl(startingNodeOrToken) ?? (startingNodeOrToken as JsParserToken);
  const finalToken = getFirstTokenImpl(endingNodeOrToken) ?? (endingNodeOrToken as JsParserToken);
  let currentToken: JsParserToken = firstToken;

  while (currentToken !== finalToken) {
    const nextToken = createCursorWithSkip(true, currentToken.range[1], -1, {
      includeComments: true,
    }).getOneToken();
    // Token store exhausted before reaching `finalToken` - possible when `finalToken` is
    // a node containing no tokens (custom parsers can produce such nodes; ESLint would
    // throw here). Fall back to comparing ranges directly, ESLint's "gap = space" rule.
    if (nextToken === null) return currentToken.range[1] !== finalToken.range[0];

    if (
      currentToken.range[1] !== nextToken.range[0] ||
      // For backward compatibility, check whitespace inside `JSXText` tokens
      (checkInsideOfJSXText &&
        nextToken !== finalToken &&
        nextToken.type === "JSXText" &&
        JSX_WHITESPACE_REGEXP.test(nextToken.value))
    ) {
      return true;
    }

    currentToken = nextToken;
  }

  return false;
}

/**
 * Get the first token of a node (no options).
 * @param node - Node or token
 * @returns First token, or `null`
 */
function getFirstTokenImpl(node: RangedNodeOrToken): JsParserToken | null {
  return createCursorWithSkip(true, node.range[0], node.range[1], undefined).getOneToken();
}

/**
 * Get the last token of a node (no options).
 * @param node - Node or token
 * @returns Last token, or `null`
 */
function getLastTokenImpl(node: RangedNodeOrToken): JsParserToken | null {
  return createCursorWithSkip(false, node.range[0], node.range[1], undefined).getOneToken();
}

// ---------------------------------------------------------------------------
// `getNodeByRangeIndex`
// ---------------------------------------------------------------------------

/**
 * Find deepest node containing `index`.
 * `node` must contain `index` itself. This function finds a deeper node if one exists.
 *
 * @param node - Node to start traversal from
 * @param index - Range index of the desired node
 * @returns Deepest node containing `index`
 */
function findNodeAt(node: JsParserNode, index: number): JsParserNode {
  debugAssertIsNonNull(mergedVisitorKeys);
  const keys = mergedVisitorKeys[node.type] ?? getFallbackKeys(node);

  for (let keyIndex = 0, keysLen = keys.length; keyIndex < keysLen; keyIndex++) {
    const child = node[keys[keyIndex]];

    if (Array.isArray(child)) {
      for (let arrIndex = 0, arrLen = child.length; arrIndex < arrLen; arrIndex++) {
        const element: unknown = child[arrIndex];
        if (isNodeWithRange(element) && element.range[0] <= index && index < element.range[1]) {
          return findNodeAt(element, index);
        }
      }
    } else if (isNodeWithRange(child) && child.range[0] <= index && index < child.range[1]) {
      return findNodeAt(child, index);
    }
  }

  // Index is not within any child node, so this is the deepest node containing the index
  return node;
}

/**
 * Check if a value is an AST node with a valid `range`.
 * @param value - Value to check
 * @returns `true` if `value` is an AST node with a `range`
 */
function isNodeWithRange(value: unknown): value is JsParserNode {
  return (
    value !== null &&
    typeof value === "object" &&
    typeof (value as { type?: unknown }).type === "string" &&
    Array.isArray((value as { range?: unknown }).range)
  );
}

// ---------------------------------------------------------------------------
// `SourceCode` object
// ---------------------------------------------------------------------------

// `SourceCode` object for files parsed by a custom (JS) parser.
//
// Mirrors the public API surface of `SOURCE_CODE` in `source_code.ts` exactly.
// Only one file is linted at a time, so a single frozen object is reused for all files.
export const JS_PARSER_SOURCE_CODE = Object.freeze({
  /**
   * Source text.
   */
  get text(): string {
    debugAssertIsNonNull(text);
    return text;
  },

  /**
   * `true` if file has Unicode BOM.
   *
   * Always `false` for files parsed by a custom parser - Rust strips the BOM from the source text
   * before sending it to JS.
   */
  get hasBOM(): boolean {
    return false;
  },

  /**
   * AST of the file, as produced by the custom parser.
   */
  get ast(): JsParserProgram {
    debugAssertIsNonNull(program);
    return program;
  },

  /**
   * `true` if the AST is in ESTree format.
   */
  isESTree: true,

  /**
   * `ScopeManager` for the file.
   * Provided by the parser if it returned one from `parseForESLint`,
   * otherwise created by analyzing the AST.
   */
  get scopeManager(): JsParserScopeManager {
    return getScopeManager();
  },

  /**
   * Visitor keys to traverse this AST (parser-provided keys merged over default keys).
   */
  get visitorKeys(): Readonly<Record<string, readonly string[]>> {
    debugAssertIsNonNull(mergedVisitorKeys);
    return mergedVisitorKeys;
  },

  /**
   * Parser services for the file, as provided by the parser.
   */
  get parserServices(): Record<string, unknown> {
    return parserServices;
  },

  /**
   * Source text as array of lines, split according to specification's definition of line breaks.
   */
  get lines(): string[] {
    if (lines.length === 0) initLines();
    return lines;
  },

  /**
   * Character offset of the first character of each line in source text,
   * split according to specification's definition of line breaks.
   */
  get lineStartIndices(): number[] {
    if (lines.length === 0) initLines();
    return lineStartIndices;
  },

  /**
   * Array of all tokens and comments in the file, in source order.
   */
  get tokensAndComments(): JsParserToken[] {
    if (tokensAndComments === null) {
      // `tokens` and `comments` are each already sorted by start offset (they never
      // overlap), so merge them linearly rather than concatenating and re-sorting.
      const merged: JsParserToken[] = [];
      const tokensLen = tokens.length,
        commentsLen = comments.length;
      let tokenIndex = 0,
        commentIndex = 0;
      while (tokenIndex < tokensLen && commentIndex < commentsLen) {
        if (tokens[tokenIndex].range[0] <= comments[commentIndex].range[0]) {
          merged.push(tokens[tokenIndex++]);
        } else {
          merged.push(comments[commentIndex++]);
        }
      }
      while (tokenIndex < tokensLen) merged.push(tokens[tokenIndex++]);
      while (commentIndex < commentsLen) merged.push(comments[commentIndex++]);
      tokensAndComments = merged;
    }
    return tokensAndComments;
  },

  /**
   * Get the source code for the given node.
   * @param node? - The AST node to get the text for.
   * @param beforeCount? - The number of characters before the node to retrieve.
   * @param afterCount? - The number of characters after the node to retrieve.
   * @returns Source text representing the AST node.
   */
  getText(node?: Ranged | null, beforeCount?: number | null, afterCount?: number | null): string {
    debugAssertIsNonNull(text);

    // ESLint treats all falsy values for `node` as undefined
    if (!node) return text;

    // ESLint ignores falsy values for `beforeCount` and `afterCount`
    const { range } = node;
    let start = range[0],
      end = range[1];
    if (beforeCount) start = Math.max(start - beforeCount, 0);
    if (afterCount) end += afterCount;
    return text.slice(start, end);
  },

  /**
   * Get all the ancestors of a given node.
   * @param node - AST node
   * @returns All the ancestor nodes in the AST, not including the provided node,
   *   starting from the root node at index 0 and going inwards to the parent node.
   */
  getAncestors(node: JsParserNode): JsParserNode[] {
    const ancestors = [];

    let current: JsParserNode | null | undefined = node;
    while (true) {
      current = current.parent;
      if (current == null) break;
      ancestors.push(current);
    }

    return ancestors.reverse();
  },

  /**
   * Get source text as array of lines, split according to specification's definition of line breaks.
   */
  getLines(): string[] {
    if (lines.length === 0) initLines();
    return lines;
  },

  // Location methods

  /**
   * Get the range of the given node or token.
   * @param nodeOrToken - Node or token to get the range of
   * @returns Range of the node or token
   */
  getRange(nodeOrToken: Ranged): Range {
    return nodeOrToken.range;
  },

  /**
   * Get the location of the given node or token.
   * @param nodeOrToken - Node or token to get the location of
   * @returns Location of the node or token
   */
  getLoc(nodeOrToken: JsParserNode | JsParserToken): Location {
    // Parsers are called with `loc: true`, so nodes and tokens should have `loc` properties.
    // Compute from `range` as a fallback.
    const { loc } = nodeOrToken;
    if (loc != null) return loc;
    return {
      start: getLineColumnFromOffset(nodeOrToken.range[0]),
      end: getLineColumnFromOffset(nodeOrToken.range[1]),
    };
  },

  /**
   * Get the deepest node containing a range index.
   * @param index - Range index of the desired node
   * @returns The node if found, or `null` if not found
   */
  getNodeByRangeIndex(index: number): JsParserNode | null {
    debugAssertIsNonNull(program);

    // If index is outside of `Program`, return `null`
    if (index < program.range[0] || index >= program.range[1]) return null;

    return findNodeAt(program, index);
  },

  // ESLint's `SourceCode` exposes these under different names than `location.ts` uses
  getLocFromIndex: getLineColumnFromOffset,
  getIndexFromLoc: getOffsetFromLineColumn,

  // Comment methods

  /**
   * Get all comments in the file.
   * @returns Array of comments, in source order
   */
  getAllComments(): JsParserToken[] {
    return comments;
  },

  /**
   * Get all comment tokens directly before the given node or token.
   * @param nodeOrToken - The AST node or token to check for adjacent comment tokens
   * @returns An array of comments in occurrence order
   */
  getCommentsBefore(nodeOrToken: RangedNodeOrToken): JsParserToken[] {
    const cursor = createCursorWithCount(false, -1, nodeOrToken.range[0], {
      includeComments: true,
    });
    return getAdjacentCommentTokensFromCursor(cursor).reverse();
  },

  /**
   * Get all comment tokens directly after the given node or token.
   * @param nodeOrToken - The AST node or token to check for adjacent comment tokens
   * @returns An array of comments in occurrence order
   */
  getCommentsAfter(nodeOrToken: RangedNodeOrToken): JsParserToken[] {
    const cursor = createCursorWithCount(true, nodeOrToken.range[1], -1, {
      includeComments: true,
    });
    return getAdjacentCommentTokensFromCursor(cursor);
  },

  /**
   * Get all comment tokens inside the given node.
   * @param node - The AST node to get the comments for
   * @returns An array of comments in occurrence order
   */
  getCommentsInside(node: RangedNodeOrToken): JsParserToken[] {
    return createCursorWithPadding(
      node.range[0],
      node.range[1],
      {
        includeComments: true,
        filter: isCommentToken,
      },
      undefined,
    ).getAllTokens();
  },

  /**
   * Check whether any comments exist between the given 2 nodes.
   * @param left - The node to check
   * @param right - The node to check
   * @returns `true` if one or more comments exist
   */
  commentsExistBetween(left: RangedNodeOrToken, right: RangedNodeOrToken): boolean {
    const index = search(comments, left.range[1]);
    return index < comments.length && comments[index].range[1] <= right.range[0];
  },

  /**
   * Retrieve the JSDoc comment for a given node.
   * @deprecated
   * @param node - The AST node to get the comment for
   * @returns The JSDoc comment for the given node, or `null` if not found
   */
  // oxlint-disable-next-line no-unused-vars
  getJSDocComment(node: JsParserNode): JsParserToken | null {
    throw new Error("`sourceCode.getJSDocComment` is not supported at present (and deprecated)");
  },

  // Scope methods
  isGlobalReference,
  getDeclaredVariables,
  getScope,
  markVariableAsUsed,

  // Token methods

  /**
   * Get all tokens that are related to the given node.
   * @param node - The AST node
   * @param beforeCount? - Options object, or the number of tokens before the node to retrieve
   * @param afterCount? - The number of tokens after the node to retrieve
   * @returns Array of tokens
   */
  getTokens(
    node: RangedNodeOrToken,
    beforeCount?: CursorWithCountOptions,
    afterCount?: number,
  ): JsParserToken[] {
    return createCursorWithPadding(
      node.range[0],
      node.range[1],
      beforeCount,
      afterCount,
    ).getAllTokens();
  },

  /**
   * Get the first token of the given node.
   * @param node - The AST node
   * @param options? - Options object, `skip` count, or `filter` function
   * @returns Token, or `null` if no matching token
   */
  getFirstToken(node: RangedNodeOrToken, options?: CursorWithSkipOptions): JsParserToken | null {
    return createCursorWithSkip(true, node.range[0], node.range[1], options).getOneToken();
  },

  /**
   * Get the first `count` tokens of the given node.
   * @param node - The AST node
   * @param options? - Options object, `count`, or `filter` function
   * @returns Array of tokens
   */
  getFirstTokens(node: RangedNodeOrToken, options?: CursorWithCountOptions): JsParserToken[] {
    return createCursorWithCount(true, node.range[0], node.range[1], options).getAllTokens();
  },

  /**
   * Get the last token of the given node.
   * @param node - The AST node
   * @param options? - Options object, `skip` count, or `filter` function
   * @returns Token, or `null` if no matching token
   */
  getLastToken(node: RangedNodeOrToken, options?: CursorWithSkipOptions): JsParserToken | null {
    return createCursorWithSkip(false, node.range[0], node.range[1], options).getOneToken();
  },

  /**
   * Get the last `count` tokens of the given node.
   * @param node - The AST node
   * @param options? - Options object, `count`, or `filter` function
   * @returns Array of tokens
   */
  getLastTokens(node: RangedNodeOrToken, options?: CursorWithCountOptions): JsParserToken[] {
    return createCursorWithCount(false, node.range[0], node.range[1], options)
      .getAllTokens()
      .reverse();
  },

  /**
   * Get the token that precedes a given node or token.
   * @param node - The AST node or token
   * @param options? - Options object, `skip` count, or `filter` function
   * @returns Token, or `null` if no matching token
   */
  getTokenBefore(node: RangedNodeOrToken, options?: CursorWithSkipOptions): JsParserToken | null {
    return createCursorWithSkip(false, -1, node.range[0], options).getOneToken();
  },

  /**
   * Get the token or comment that precedes a given node or token.
   * @deprecated Use `getTokenBefore` with `{ includeComments: true }` option instead.
   * @param node - The AST node or token
   * @param skip? - The count of tokens the cursor skips
   * @returns Token or comment, or `null` if no matching token
   */
  getTokenOrCommentBefore(node: RangedNodeOrToken, skip?: number): JsParserToken | null {
    return createCursorWithSkip(false, -1, node.range[0], {
      includeComments: true,
      skip,
    }).getOneToken();
  },

  /**
   * Get the `count` tokens that precede a given node or token.
   * @param node - The AST node or token
   * @param options? - Options object, `count`, or `filter` function
   * @returns Array of tokens
   */
  getTokensBefore(node: RangedNodeOrToken, options?: CursorWithCountOptions): JsParserToken[] {
    return createCursorWithCount(false, -1, node.range[0], options).getAllTokens().reverse();
  },

  /**
   * Get the token that follows a given node or token.
   * @param node - The AST node or token
   * @param options? - Options object, `skip` count, or `filter` function
   * @returns Token, or `null` if no matching token
   */
  getTokenAfter(node: RangedNodeOrToken, options?: CursorWithSkipOptions): JsParserToken | null {
    return createCursorWithSkip(true, node.range[1], -1, options).getOneToken();
  },

  /**
   * Get the token or comment that follows a given node or token.
   * @deprecated Use `getTokenAfter` with `{ includeComments: true }` option instead.
   * @param node - The AST node or token
   * @param skip? - The count of tokens the cursor skips
   * @returns Token or comment, or `null` if no matching token
   */
  getTokenOrCommentAfter(node: RangedNodeOrToken, skip?: number): JsParserToken | null {
    return createCursorWithSkip(true, node.range[1], -1, {
      includeComments: true,
      skip,
    }).getOneToken();
  },

  /**
   * Get the `count` tokens that follow a given node or token.
   * @param node - The AST node or token
   * @param options? - Options object, `count`, or `filter` function
   * @returns Array of tokens
   */
  getTokensAfter(node: RangedNodeOrToken, options?: CursorWithCountOptions): JsParserToken[] {
    return createCursorWithCount(true, node.range[1], -1, options).getAllTokens();
  },

  /**
   * Get all of the tokens between two non-overlapping nodes.
   * @param left - Node before the desired token range
   * @param right - Node after the desired token range
   * @param padding? - Options object, or number of extra tokens on either side of center
   * @returns Tokens between left and right
   */
  getTokensBetween(
    left: RangedNodeOrToken,
    right: RangedNodeOrToken,
    padding?: CursorWithCountOptions,
  ): JsParserToken[] {
    return createCursorWithPadding(
      left.range[1],
      right.range[0],
      padding,
      typeof padding === "number" ? padding : undefined,
    ).getAllTokens();
  },

  /**
   * Get the first token between two non-overlapping nodes.
   * @param left - Node before the desired token range
   * @param right - Node after the desired token range
   * @param options? - Options object, `skip` count, or `filter` function
   * @returns Token, or `null` if no matching token
   */
  getFirstTokenBetween(
    left: RangedNodeOrToken,
    right: RangedNodeOrToken,
    options?: CursorWithSkipOptions,
  ): JsParserToken | null {
    return createCursorWithSkip(true, left.range[1], right.range[0], options).getOneToken();
  },

  /**
   * Get the first `count` tokens between two non-overlapping nodes.
   * @param left - Node before the desired token range
   * @param right - Node after the desired token range
   * @param options? - Options object, `count`, or `filter` function
   * @returns Array of tokens between left and right
   */
  getFirstTokensBetween(
    left: RangedNodeOrToken,
    right: RangedNodeOrToken,
    options?: CursorWithCountOptions,
  ): JsParserToken[] {
    return createCursorWithCount(true, left.range[1], right.range[0], options).getAllTokens();
  },

  /**
   * Get the last token between two non-overlapping nodes.
   * @param left - Node before the desired token range
   * @param right - Node after the desired token range
   * @param options? - Options object, `skip` count, or `filter` function
   * @returns Token, or `null` if no matching token
   */
  getLastTokenBetween(
    left: RangedNodeOrToken,
    right: RangedNodeOrToken,
    options?: CursorWithSkipOptions,
  ): JsParserToken | null {
    return createCursorWithSkip(false, left.range[1], right.range[0], options).getOneToken();
  },

  /**
   * Get the last `count` tokens between two non-overlapping nodes.
   * @param left - Node before the desired token range
   * @param right - Node after the desired token range
   * @param options? - Options object, `count`, or `filter` function
   * @returns Array of tokens between left and right
   */
  getLastTokensBetween(
    left: RangedNodeOrToken,
    right: RangedNodeOrToken,
    options?: CursorWithCountOptions,
  ): JsParserToken[] {
    return createCursorWithCount(false, left.range[1], right.range[0], options)
      .getAllTokens()
      .reverse();
  },

  /**
   * Get the token starting at the specified index.
   * @param offset - Index of the start of the token's range
   * @param options? - Options object
   * @returns The token starting at index, or `null` if no such token
   */
  getTokenByRangeStart(
    offset: number,
    options?: { includeComments?: boolean } | null,
  ): JsParserToken | null {
    const includeComments = !!(options && options.includeComments);
    const token = createBaseCursor(true, offset, -1, includeComments).getOneToken();

    if (token && token.range[0] === offset) return token;
    return null;
  },

  /**
   * Determine if two nodes or tokens have at least one whitespace character between them.
   * Order does not matter. Returns `false` if the given nodes or tokens overlap.
   * @param first - The first node or token to check between
   * @param second - The second node or token to check between
   * @returns `true` if there is a whitespace character between
   *   any of the tokens found between the two given nodes or tokens
   */
  isSpaceBetween(first: RangedNodeOrToken, second: RangedNodeOrToken): boolean {
    return isSpaceBetweenImpl(first, second, false);
  },

  /**
   * Determine if two nodes or tokens have at least one whitespace character between them.
   * Order does not matter. Returns `false` if the given nodes or tokens overlap.
   *
   * Unlike `isSpaceBetween`, also returns `true` if there is a `JSXText` token containing
   * whitespace between the two input tokens.
   *
   * @deprecated Use `sourceCode.isSpaceBetween` instead.
   * @param first - The first node or token to check between
   * @param second - The second node or token to check between
   * @returns `true` if there is a whitespace character between
   *   any of the tokens found between the two given nodes or tokens
   */
  isSpaceBetweenTokens(first: RangedNodeOrToken, second: RangedNodeOrToken): boolean {
    return isSpaceBetweenImpl(first, second, true);
  },

  // Directive methods

  /**
   * Get disable directives in the file.
   *
   * For files parsed by a custom parser, disable directives are processed on Rust side
   * (from the comments returned by `lintFileWithJsParser`), so this returns an empty result.
   *
   * @returns Empty problems and directives arrays
   */
  getDisableDirectives(): { problems: never[]; directives: never[] } {
    return { problems: [], directives: [] };
  },
});

/**
 * `SourceCode` object type for files parsed by a custom (JS) parser.
 */
export type JsParserSourceCode = typeof JS_PARSER_SOURCE_CODE;
