import { DATA_POINTER_POS_32, SOURCE_LEN_OFFSET } from "../generated/constants.ts";

// We use the deserializer which removes `ParenthesizedExpression`s from AST,
// and with `range`, `loc`, and `parent` properties on AST nodes, to match ESLint
// @ts-expect-error we need to generate `.d.ts` file for this module
import { deserializeProgramOnly, resetBuffer } from "../generated/deserialize.js";

import visitorKeys from "../generated/keys.ts";
import * as commentMethods from "./comments.ts";
import {
  getLineColumnFromOffset,
  getNodeByRangeIndex,
  getNodeLoc,
  getOffsetFromLineColumn,
  initLines,
  lines,
  lineStartIndices,
  resetLines,
} from "./location.ts";
import { resetScopeManager, SCOPE_MANAGER } from "./scope.ts";
import * as scopeMethods from "./scope.ts";
import { resetTokens } from "./tokens.ts";
import { tokens, tokensAndComments, initTokens, initTokensAndComments } from "./tokens.ts";
import * as tokenMethods from "./tokens.ts";
import { debugAssertIsNonNull } from "../utils/asserts.ts";

import type { Program } from "../generated/types.d.ts";
import type { Ranged } from "./location.ts";
import type { Token, CommentToken } from "./tokens.ts";
import type { BufferWithArrays, Node } from "./types.ts";
import type { ScopeManager } from "./scope.ts";

const { max } = Math;

// Text decoder, for decoding source text from buffer
const textDecoder = new TextDecoder("utf-8", { ignoreBOM: true });

// Buffer containing AST. Set before linting a file by `setupSourceForFile`.
let buffer: BufferWithArrays | null = null;

// Indicates if the original source text has a BOM. Set before linting a file by `setupSourceForFile`.
let hasBOM = false;

// Lazily populated when `SOURCE_CODE.text` or `SOURCE_CODE.ast` is accessed,
// or `initAst()` is called before the AST is walked.
export let sourceText: string | null = null;
let sourceByteLen: number = 0;
export let ast: Program | null = null;

// Parser services object. Set before linting a file by `setupSourceForFile`.
let parserServices: Record<string, unknown> | null = null;

/**
 * Set up source for the file about to be linted.
 * @param bufferInput - Buffer containing AST
 * @param hasBOMInput - `true` if file's original source text has Unicode BOM
 * @param parserServicesInput - Parser services object for the file
 */
export function setupSourceForFile(
  bufferInput: BufferWithArrays,
  hasBOMInput: boolean,
  parserServicesInput: Record<string, unknown>,
): void {
  buffer = bufferInput;
  hasBOM = hasBOMInput;
  parserServices = parserServicesInput;
}

/**
 * Decode source text from buffer.
 */
export function initSourceText(): void {
  debugAssertIsNonNull(buffer);
  const { uint32 } = buffer,
    programPos = uint32[DATA_POINTER_POS_32];
  sourceByteLen = uint32[(programPos + SOURCE_LEN_OFFSET) >> 2];
  sourceText = textDecoder.decode(buffer.subarray(0, sourceByteLen));
}

/**
 * Deserialize AST from buffer.
 */
export function initAst(): void {
  if (sourceText === null) initSourceText();
  debugAssertIsNonNull(sourceText);

  ast = deserializeProgramOnly(buffer, sourceText, sourceByteLen, getNodeLoc);
  debugAssertIsNonNull(ast);
}

/**
 * Reset source and AST after file has been linted, to free memory.
 *
 * Setting `buffer` to `null` also prevents AST being deserialized after linting,
 * at which point the buffer may be being reused for another file.
 * The buffer might contain a half-constructed AST (if parsing is currently in progress in Rust),
 * which would cause deserialization to malfunction.
 * With `buffer` set to `null`, accessing `SOURCE_CODE.ast` will still throw, but the error message will be clearer,
 * and no danger of an infinite loop due to a circular AST (unlikely but possible).
 */
export function resetSourceAndAst(): void {
  buffer = null;
  sourceText = null;
  ast = null;
  parserServices = null;
  resetBuffer();
  resetLines();
  resetScopeManager();
  resetTokens();
}

// `SourceCode` object.
//
// Only one file is linted at a time, so we can reuse a single object for all files.
//
// This has advantages:
// 1. Reduce object creation.
// 2. Property accesses don't need to go up prototype chain, as they would for instances of a class.
// 3. No need for private properties, which are somewhat expensive to access - use top-level variables instead.
//
// Freeze the object to prevent user mutating it.
export const SOURCE_CODE = Object.freeze({
  /**
   * Source text.
   */
  get text(): string {
    if (sourceText === null) initSourceText();
    debugAssertIsNonNull(sourceText);
    return sourceText;
  },

  /**
   * `true` if file has Unicode BOM.
   */
  get hasBOM(): boolean {
    return hasBOM;
  },

  /**
   * AST of the file.
   */
  get ast(): Program {
    if (ast === null) initAst();
    debugAssertIsNonNull(ast);
    return ast;
  },

  /**
   * `true` if the AST is in ESTree format.
   */
  // This property is present in ESLint's `SourceCode`, but is undocumented
  isESTree: true,

  /**
   * `ScopeManager` for the file.
   */
  get scopeManager(): ScopeManager {
    return SCOPE_MANAGER;
  },

  /**
   * Visitor keys to traverse this AST.
   */
  get visitorKeys(): Readonly<Record<string, readonly string[]>> {
    return visitorKeys;
  },

  /**
   * Parser services for the file.
   */
  get parserServices(): Record<string, unknown> {
    debugAssertIsNonNull(parserServices);
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
  // This property is present in ESLint's `SourceCode`, but is undocumented
  get tokensAndComments(): (Token | CommentToken)[] {
    if (tokensAndComments === null) {
      if (tokens === null) {
        if (sourceText === null) initSourceText();
        initTokens();
      }
      initTokensAndComments();
    }
    debugAssertIsNonNull(tokensAndComments);
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
    if (sourceText === null) initSourceText();
    debugAssertIsNonNull(sourceText);

    // ESLint treats all falsy values for `node` as undefined
    if (!node) return sourceText;

    // ESLint ignores falsy values for `beforeCount` and `afterCount`
    const { range } = node;
    let start = range[0],
      end = range[1];
    if (beforeCount) start = max(start - beforeCount, 0);
    if (afterCount) end += afterCount;
    return sourceText.slice(start, end);
  },

  /**
   * Get all the ancestors of a given node.
   * @param node - AST node
   * @returns All the ancestor nodes in the AST, not including the provided node,
   *   starting from the root node at index 0 and going inwards to the parent node.
   */
  getAncestors(node: Node): Node[] {
    const ancestors = [];

    while (true) {
      // @ts-expect-error `parent` property should be present on `Node` type
      node = node.parent;
      if (node === null) break;
      ancestors.push(node);
    }

    return ancestors.reverse();
  },

  // Location methods
  getNodeByRangeIndex,
  getLocFromIndex: getLineColumnFromOffset,
  getIndexFromLoc: getOffsetFromLineColumn,

  // Comment methods
  getAllComments: commentMethods.getAllComments,
  getCommentsBefore: commentMethods.getCommentsBefore,
  getCommentsAfter: commentMethods.getCommentsAfter,
  getCommentsInside: commentMethods.getCommentsInside,
  commentsExistBetween: commentMethods.commentsExistBetween,
  getJSDocComment: commentMethods.getJSDocComment,

  // Scope methods
  isGlobalReference: scopeMethods.isGlobalReference,
  getDeclaredVariables: scopeMethods.getDeclaredVariables,
  getScope: scopeMethods.getScope,
  markVariableAsUsed: scopeMethods.markVariableAsUsed,

  // Token methods
  getTokens: tokenMethods.getTokens,
  getFirstToken: tokenMethods.getFirstToken,
  getFirstTokens: tokenMethods.getFirstTokens,
  getLastToken: tokenMethods.getLastToken,
  getLastTokens: tokenMethods.getLastTokens,
  getTokenBefore: tokenMethods.getTokenBefore,
  getTokenOrCommentBefore: tokenMethods.getTokenOrCommentBefore,
  getTokensBefore: tokenMethods.getTokensBefore,
  getTokenAfter: tokenMethods.getTokenAfter,
  getTokenOrCommentAfter: tokenMethods.getTokenOrCommentAfter,
  getTokensAfter: tokenMethods.getTokensAfter,
  getTokensBetween: tokenMethods.getTokensBetween,
  getFirstTokenBetween: tokenMethods.getFirstTokenBetween,
  getFirstTokensBetween: tokenMethods.getFirstTokensBetween,
  getLastTokenBetween: tokenMethods.getLastTokenBetween,
  getLastTokensBetween: tokenMethods.getLastTokensBetween,
  getTokenByRangeStart: tokenMethods.getTokenByRangeStart,
  isSpaceBetween: tokenMethods.isSpaceBetween,
  isSpaceBetweenTokens: tokenMethods.isSpaceBetweenTokens,
});

export type SourceCode = typeof SOURCE_CODE;
