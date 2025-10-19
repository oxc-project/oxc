import { DATA_POINTER_POS_32, SOURCE_LEN_OFFSET } from '../generated/constants.js';

// We use the deserializer which removes `ParenthesizedExpression`s from AST,
// and with `range`, `loc`, and `parent` properties on AST nodes, to match ESLint
// @ts-expect-error we need to generate `.d.ts` file for this module
import { deserializeProgramOnly, resetBuffer } from '../../dist/generated/deserialize.js';

import visitorKeys from '../generated/keys.js';
import * as commentMethods from './comments.js';
import {
  getLineColumnFromOffset,
  getNodeLoc,
  getOffsetFromLineColumn,
  initLines,
  lines,
  resetLines,
} from './location.js';
import * as scopeMethods from './scope.js';
import * as tokenMethods from './tokens.js';

import type { Program } from '../generated/types.d.ts';
import type { ScopeManager } from './scope.ts';
import type { BufferWithArrays, Node, NodeOrToken, Ranged } from './types.ts';

const { max } = Math;

// Text decoder, for decoding source text from buffer
const textDecoder = new TextDecoder('utf-8', { ignoreBOM: true });

// Buffer containing AST. Set before linting a file by `setupSourceForFile`.
let buffer: BufferWithArrays | null = null;

// Indicates if the original source text has a BOM. Set before linting a file by `setupSourceForFile`.
let hasBOM = false;

// Lazily populated when `SOURCE_CODE.text` or `SOURCE_CODE.ast` is accessed,
// or `initAst()` is called before the AST is walked.
export let sourceText: string | null = null;
let sourceByteLen: number = 0;
export let ast: Program | null = null;

/**
 * Set up source for the file about to be linted.
 * @param bufferInput - Buffer containing AST
 * @param hasBOMInput - `true` if file's original source text has Unicode BOM
 */
export function setupSourceForFile(bufferInput: BufferWithArrays, hasBOMInput: boolean): void {
  buffer = bufferInput;
  hasBOM = hasBOMInput;
}

/**
 * Decode source text from buffer.
 */
export function initSourceText(): void {
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
  ast = deserializeProgramOnly(buffer, sourceText, sourceByteLen, getNodeLoc);
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
  resetBuffer();
  resetLines();
}

// `SourceCode` object.
//
// Only one file is linted at a time, so we can reuse a single object for all files.
//
// This has advantages:
// 1. Property accesses don't need to go up prototype chain, as they would for instances of a class.
// 2. No need for private properties, which are somewhat expensive to access - use top-level variables instead.
//
// Freeze the object to prevent user mutating it.
export const SOURCE_CODE = Object.freeze({
  // Get source text.
  get text(): string {
    if (sourceText === null) initSourceText();
    return sourceText;
  },

  // `true` if source text has Unicode BOM.
  get hasBOM(): boolean {
    return hasBOM;
  },

  // Get AST of the file.
  get ast(): Program {
    if (ast === null) initAst();
    return ast;
  },

  // Get `ScopeManager` for the file.
  get scopeManager(): ScopeManager {
    throw new Error('`sourceCode.scopeManager` not implemented yet'); // TODO
  },

  // Get visitor keys to traverse this AST.
  get visitorKeys(): { [key: string]: string[] } {
    return visitorKeys;
  },

  // Get parser services for the file.
  get parserServices(): { [key: string]: unknown } {
    throw new Error('`sourceCode.parserServices` not implemented yet'); // TODO
  },

  // Get source text as array of lines, split according to specification's definition of line breaks.
  get lines(): string[] {
    if (lines.length === 0) initLines();
    return lines;
  },

  /**
   * Get the source code for the given node.
   * @param node? - The AST node to get the text for.
   * @param beforeCount? - The number of characters before the node to retrieve.
   * @param afterCount? - The number of characters after the node to retrieve.
   * @returns Source text representing the AST node.
   */
  getText(
    node?: Ranged | null | undefined,
    beforeCount?: number | null | undefined,
    afterCount?: number | null | undefined,
  ): string {
    if (sourceText === null) initSourceText();

    // ESLint treats all falsy values for `node` as undefined
    if (!node) return sourceText;

    // ESLint ignores falsy values for `beforeCount` and `afterCount`
    const { range } = node;
    let start = range[0], end = range[1];
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

  /**
   * Determine if two nodes or tokens have at least one whitespace character between them.
   * Order does not matter. Returns `false` if the given nodes or tokens overlap.
   * @param nodeOrToken1 - The first node or token to check between.
   * @param nodeOrToken2 - The second node or token to check between.
   * @returns `true` if there is a whitespace character between
   *   any of the tokens found between the two given nodes or tokens.
   */
  // oxlint-disable-next-line no-unused-vars
  isSpaceBetween(nodeOrToken1: NodeOrToken, nodeOrToken2: NodeOrToken): boolean {
    throw new Error('`sourceCode.isSpaceBetween` not implemented yet'); // TODO
  },

  /**
   * Get the deepest node containing a range index.
   * @param index Range index of the desired node.
   * @returns The node if found, or `null` if not found.
   */
  // oxlint-disable-next-line no-unused-vars
  getNodeByRangeIndex(index: number): Node | null {
    throw new Error('`sourceCode.getNodeByRangeIndex` not implemented yet'); // TODO
  },

  // Location methods
  getLocFromIndex: getLineColumnFromOffset,
  getIndexFromLoc: getOffsetFromLineColumn,

  // Comment methods
  getAllComments: commentMethods.getAllComments,
  getCommentsBefore: commentMethods.getCommentsBefore,
  getCommentsAfter: commentMethods.getCommentsAfter,
  getCommentsInside: commentMethods.getCommentsInside,
  commentsExistBetween: commentMethods.commentsExistBetween,

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
  getTokensBefore: tokenMethods.getTokensBefore,
  getTokenAfter: tokenMethods.getTokenAfter,
  getTokensAfter: tokenMethods.getTokensAfter,
  getTokensBetween: tokenMethods.getTokensBetween,
  getFirstTokenBetween: tokenMethods.getFirstTokenBetween,
  getFirstTokensBetween: tokenMethods.getFirstTokensBetween,
  getLastTokenBetween: tokenMethods.getLastTokenBetween,
  getLastTokensBetween: tokenMethods.getLastTokensBetween,
  getTokenByRangeStart: tokenMethods.getTokenByRangeStart,
});

export type SourceCode = typeof SOURCE_CODE;
