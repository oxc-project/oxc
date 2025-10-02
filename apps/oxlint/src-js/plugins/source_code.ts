import { createRequire } from 'node:module';
import {
  DATA_POINTER_POS_32,
  SOURCE_LEN_OFFSET,
  // TODO(camc314): we need to generate `.d.ts` file for this module.
  // @ts-expect-error
} from '../generated/constants.js';
// @ts-expect-error we need to generate `.d.ts` file for this module
import { deserializeProgramOnly } from '../../dist/generated/deserialize/ts.js';

import type { Program } from '@oxc-project/types';
import type { Scope, ScopeManager, Variable } from './scope.ts';
import type { BufferWithArrays, Comment, LineColumn, Node, NodeOrToken, Token } from './types.ts';

const require = createRequire(import.meta.url);

const { max } = Math;

// Pattern for splitting source text into lines
const LINE_BREAK_PATTERN = /\r\n|[\r\n\u2028\u2029]/gu;

// Text decoder, for decoding source text from buffer
const textDecoder = new TextDecoder('utf-8', { ignoreBOM: true });

// Buffer containing AST. Set before linting a file by `setupSourceForFile`.
let buffer: BufferWithArrays | null = null;

// Indicates if the original source text has a BOM. Set before linting a file by `setupSourceForFile`.
let hasBOM = false;

// Lazily populated when `SOURCE_CODE.text` or `SOURCE_CODE.ast` is accessed,
// or `getAst()` is called before the AST is walked.
let sourceText: string | null = null;
let sourceByteLen: number = 0;
let ast: Program | null = null;

// Lazily populated when `SOURCE_CODE.lines` is accessed.
const lines: string[] = [],
  lineStartOffsets: number[] = [];

// Lazily populated when `SOURCE_CODE.visitorKeys` is accessed.
let visitorKeys: { [key: string]: string[] } | null = null;

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
function initSourceText(): void {
  const { uint32 } = buffer,
    programPos = uint32[DATA_POINTER_POS_32];
  sourceByteLen = uint32[(programPos + SOURCE_LEN_OFFSET) >> 2];
  sourceText = textDecoder.decode(buffer.subarray(0, sourceByteLen));
}

/**
 * Deserialize AST from buffer.
 */
function initAst(): void {
  if (sourceText === null) initSourceText();

  // `preserveParens` argument is `false`, to match ESLint.
  // ESLint does not include `ParenthesizedExpression` nodes in its AST.
  ast = deserializeProgramOnly(buffer, sourceText, sourceByteLen, false);
}

/**
 * Get AST of the file being linted.
 * If AST has not already been deserialized, do it now.
 * @returns AST of the file being linted.
 */
export function getAst(): Program {
  if (ast === null) initAst();
  return ast;
}

/**
 * Split source text into lines.
 */
function initLines(): void {
  if (lines.length !== 0) return;

  if (sourceText === null) initSourceText();

  // This implementation is based on the one in ESLint.
  // TODO: Investigate if using `String.prototype.matchAll` is faster.
  // This comment is above ESLint's implementation:
  /*
   * Previously, this was implemented using a regex that
   * matched a sequence of non-linebreak characters followed by a
   * linebreak, then adding the lengths of the matches. However,
   * this caused a catastrophic backtracking issue when the end
   * of a file contained a large number of non-newline characters.
   * To avoid this, the current implementation just matches newlines
   * and uses match.index to get the correct line start indices.
   */

  lineStartOffsets.push(0);
  let lastOffset = 0, offset, match;
  while ((match = LINE_BREAK_PATTERN.exec(sourceText))) {
    offset = match.index;
    lines.push(sourceText.slice(lastOffset, offset));
    lineStartOffsets.push(lastOffset = offset + match[0].length);
  }
  lines.push(sourceText.slice(lastOffset));
}

/**
 * Reset source after file has been linted, to free memory.
 *
 * Setting `buffer` to `null` also prevents AST being deserialized after linting,
 * at which point the buffer may be being reused for another file.
 * The buffer might contain a half-constructed AST (if parsing is currently in progress in Rust),
 * which would cause deserialization to malfunction.
 * With `buffer` set to `null`, accessing `SOURCE_CODE.ast` will still throw, but the error message will be clearer,
 * and no danger of an infinite loop due to a circular AST (unlikely but possible).
 */
export function resetSource(): void {
  buffer = null;
  sourceText = null;
  ast = null;
  lines.length = 0;
  lineStartOffsets.length = 0;
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
    return getAst();
  },

  // Get `ScopeManager` for the file.
  get scopeManager(): ScopeManager {
    throw new Error('`sourceCode.scopeManager` not implemented yet'); // TODO
  },

  // Get visitor keys to traverse this AST.
  get visitorKeys(): { [key: string]: string[] } {
    // This is the path relative to `plugins.js` file in `dist` directory
    if (visitorKeys === null) visitorKeys = require('./generated/visit/keys.js').default;
    return visitorKeys;
  },

  // Get parser services for the file.
  get parserServices(): { [key: string]: unknown } {
    throw new Error('`sourceCode.parserServices` not implemented yet'); // TODO
  },

  // Get source text as array of lines, split according to specification's definition of line breaks.
  get lines(): string[] {
    initLines();
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
    node?: Node | null | undefined,
    beforeCount?: number | null | undefined,
    afterCount?: number | null | undefined,
  ): string {
    if (sourceText === null) initSourceText();

    // ESLint treats all falsy values for `node` as undefined
    if (!node) return sourceText;

    // ESLint ignores falsy values for `beforeCount` and `afterCount`
    let { start, end } = node;
    if (beforeCount) start = max(start - beforeCount, 0);
    if (afterCount) end += afterCount;
    return sourceText.slice(start, end);
  },

  /**
   * Retrieve an array containing all comments in the source code.
   * @returns Array of `Comment`s in occurrence order.
   */
  getAllComments(): Comment[] {
    throw new Error('`sourceCode.getAllComments` not implemented yet'); // TODO
  },

  /**
   * Get all comment tokens directly before the given node or token.
   * @param nodeOrToken - The AST node or token to check for adjacent comment tokens.
   * @returns Array of `Comment`s in occurrence order.
   */
  // oxlint-disable-next-line no-unused-vars
  getCommentsBefore(nodeOrToken: NodeOrToken): Comment[] {
    throw new Error('`sourceCode.getCommentsBefore` not implemented yet'); // TODO
  },

  /**
   * Get all comment tokens directly after the given node or token.
   * @param nodeOrToken - The AST node or token to check for adjacent comment tokens.
   * @returns Array of `Comment`s in occurrence order.
   */
  // oxlint-disable-next-line no-unused-vars
  getCommentsAfter(nodeOrToken: NodeOrToken): Comment[] {
    throw new Error('`sourceCode.getCommentsAfter` not implemented yet'); // TODO
  },

  /**
   * Get all comment tokens inside the given node.
   * @param node - The AST node to get the comments for.
   * @returns Array of `Comment`s in occurrence order.
   */
  // oxlint-disable-next-line no-unused-vars
  getCommentsInside(node: Node): Comment[] {
    throw new Error('`sourceCode.getCommentsInside` not implemented yet'); // TODO
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
   * Determine whether the given identifier node is a reference to a global variable.
   * @param node - `Identifier` node to check.
   * @returns `true` if the identifier is a reference to a global variable.
   */
  // oxlint-disable-next-line no-unused-vars
  isGlobalReference(node: Node): boolean {
    throw new Error('`sourceCode.isGlobalReference` not implemented yet'); // TODO
  },

  /**
   * Get all tokens that are related to the given node.
   * @param node - The AST node.
   * @param countOptions? - Options object. If this is a function then it's `options.filter`.
   * @returns Array of `Token`s.
   */
  /**
   * Get all tokens that are related to the given node.
   * @param node - The AST node.
   * @param beforeCount? - The number of tokens before the node to retrieve.
   * @param afterCount? - The number of tokens after the node to retrieve.
   * @returns Array of `Token`s.
   */
  /* oxlint-disable no-unused-vars */
  getTokens(
    node: Node,
    countOptions?: CountOptions | number | FilterFn | null | undefined,
    afterCount?: number | null | undefined,
  ): Token[] {
    throw new Error('`sourceCode.getTokens` not implemented yet'); // TODO
  },
  /* oxlint-enable no-unused-vars */

  /**
   * Get the first token of the given node.
   * @param node - The AST node.
   * @param skipOptions? - Options object. If this is a number then it's `options.skip`.
   *   If this is a function then it's `options.filter`.
   * @returns `Token`, or `null` if all were skipped.
   */
  // oxlint-disable-next-line no-unused-vars
  getFirstToken(node: Node, skipOptions?: SkipOptions | number | FilterFn | null | undefined): Token | null {
    throw new Error('`sourceCode.getFirstToken` not implemented yet'); // TODO
  },

  /**
   * Get the first tokens of the given node.
   * @param node - The AST node.
   * @param countOptions? - Options object. If this is a number then it's `options.count`.
   *   If this is a function then it's `options.filter`.
   * @returns Array of `Token`s.
   */
  // oxlint-disable-next-line no-unused-vars
  getFirstTokens(node: Node, countOptions?: CountOptions | number | FilterFn | null | undefined): Token[] {
    throw new Error('`sourceCode.getFirstTokens` not implemented yet'); // TODO
  },

  /**
   * Get the last token of the given node.
   * @param node - The AST node.
   * @param skipOptions? - Options object. Same options as `getFirstToken()`.
   * @returns `Token`, or `null` if all were skipped.
   */
  // oxlint-disable-next-line no-unused-vars
  getLastToken(node: Node, skipOptions?: SkipOptions | number | FilterFn | null | undefined): Token | null {
    throw new Error('`sourceCode.getLastToken` not implemented yet'); // TODO
  },

  /**
   * Get the last tokens of the given node.
   * @param node - The AST node.
   * @param countOptions? - Options object. Same options as `getFirstTokens()`.
   * @returns Array of `Token`s.
   */
  // oxlint-disable-next-line no-unused-vars
  getLastTokens(node: Node, countOptions?: CountOptions | number | FilterFn | null | undefined): Token[] {
    throw new Error('`sourceCode.getLastTokens` not implemented yet'); // TODO
  },

  /**
   * Get the token that precedes a given node or token.
   * @param nodeOrToken - The AST node or token.
   * @param skipOptions? - Options object. Same options as `getFirstToken()`.
   * @returns `Token`, or `null` if all were skipped.
   */
  /* oxlint-disable no-unused-vars */
  getTokenBefore(
    nodeOrToken: NodeOrToken | Comment,
    skipOptions?: SkipOptions | number | FilterFn | null | undefined,
  ): Token | null {
    throw new Error('`sourceCode.getTokenBefore` not implemented yet'); // TODO
  },
  /* oxlint-enable no-unused-vars */

  /**
   * Get the tokens that precedes a given node or token.
   * @param nodeOrToken - The AST node or token.
   * @param countOptions? - Options object. Same options as `getFirstTokens()`.
   * @returns Array of `Token`s.
   */
  /* oxlint-disable no-unused-vars */
  getTokensBefore(
    nodeOrToken: NodeOrToken | Comment,
    countOptions?: CountOptions | number | FilterFn | null | undefined,
  ): Token[] {
    throw new Error('`sourceCode.getTokensBefore` not implemented yet'); // TODO
  },
  /* oxlint-enable no-unused-vars */

  /**
   * Get the token that follows a given node or token.
   * @param nodeOrToken - The AST node or token.
   * @param skipOptions? - Options object. Same options as `getFirstToken()`.
   * @returns `Token`, or `null` if all were skipped.
   */
  /* oxlint-disable no-unused-vars */
  getTokenAfter(
    nodeOrToken: NodeOrToken | Comment,
    skipOptions?: SkipOptions | number | FilterFn | null | undefined,
  ): Token | null {
    throw new Error('`sourceCode.getTokenAfter` not implemented yet'); // TODO
  },
  /* oxlint-enable no-unused-vars */

  /**
   * Get the tokens that follow a given node or token.
   * @param nodeOrToken - The AST node or token.
   * @param countOptions? - Options object. Same options as `getFirstTokens()`.
   * @returns Array of `Token`s.
   */
  /* oxlint-disable no-unused-vars */
  getTokensAfter(
    nodeOrToken: NodeOrToken | Comment,
    countOptions?: CountOptions | number | FilterFn | null | undefined,
  ): Token[] {
    throw new Error('`sourceCode.getTokensAfter` not implemented yet'); // TODO
  },
  /* oxlint-enable no-unused-vars */

  /**
   * Get all of the tokens between two non-overlapping nodes.
   * @param nodeOrToken1 - Node before the desired token range.
   * @param nodeOrToken2 - Node after the desired token range.
   * @param countOptions? - Options object. If this is a function then it's `options.filter`.
   * @returns Array of `Token`s between `nodeOrToken1` and `nodeOrToken2`.
   */
  /**
   * Get all of the tokens between two non-overlapping nodes.
   * @param nodeOrToken1 - Node before the desired token range.
   * @param nodeOrToken2 - Node after the desired token range.
   * @param padding - Number of extra tokens on either side of center.
   * @returns Array of `Token`s between `nodeOrToken1` and `nodeOrToken2`.
   */
  /* oxlint-disable no-unused-vars */
  getTokensBetween(
    nodeOrToken1: NodeOrToken | Comment,
    nodeOrToken2: NodeOrToken | Comment,
    countOptions?: CountOptions | number | FilterFn | null | undefined,
  ): Token[] {
    throw new Error('`sourceCode.getTokensBetween` not implemented yet'); // TODO
  },
  /* oxlint-enable no-unused-vars */

  /**
   * Get the first token between two non-overlapping nodes.
   * @param nodeOrToken1 - Node before the desired token range.
   * @param nodeOrToken2 - Node after the desired token range.
   * @param countOptions? - Options object. Same options as `getFirstToken()`.
   * @returns `Token`, or `null` if all were skipped.
   */
  /* oxlint-disable no-unused-vars */
  getFirstTokenBetween(
    nodeOrToken1: NodeOrToken | Comment,
    nodeOrToken2: NodeOrToken | Comment,
    skipOptions?: SkipOptions | null | undefined,
  ): Token | null {
    throw new Error('`sourceCode.getFirstTokenBetween` not implemented yet'); // TODO
  },
  /* oxlint-enable no-unused-vars */

  /**
   * Get the first tokens between two non-overlapping nodes.
   * @param nodeOrToken1 - Node before the desired token range.
   * @param nodeOrToken2 - Node after the desired token range.
   * @param countOptions? - Options object. Same options as `getFirstTokens()`.
   * @returns Array of `Token`s between `nodeOrToken1` and `nodeOrToken2`.
   */
  /* oxlint-disable no-unused-vars */
  getFirstTokensBetween(
    nodeOrToken1: NodeOrToken | Comment,
    nodeOrToken2: NodeOrToken | Comment,
    countOptions?: CountOptions | number | FilterFn | null | undefined,
  ): Token[] {
    throw new Error('`sourceCode.getFirstTokensBetween` not implemented yet'); // TODO
  },
  /* oxlint-enable no-unused-vars */

  /**
   * Get the last token between two non-overlapping nodes.
   * @param nodeOrToken1 - Node before the desired token range.
   * @param nodeOrToken2 - Node after the desired token range.
   * @param skipOptions? - Options object. Same options as `getFirstToken()`.
   * @returns `Token`, or `null` if all were skipped.
   */
  /* oxlint-disable no-unused-vars */
  getLastTokenBetween(
    nodeOrToken1: NodeOrToken | Comment,
    nodeOrToken2: NodeOrToken | Comment,
    skipOptions?: SkipOptions | null | undefined,
  ): Token | null {
    throw new Error('`sourceCode.getLastTokenBetween` not implemented yet'); // TODO
  },
  /* oxlint-enable no-unused-vars */

  /**
   * Get the last tokens between two non-overlapping nodes.
   * @param nodeOrToken1 - Node before the desired token range.
   * @param nodeOrToken2 - Node after the desired token range.
   * @param countOptions? - Options object. Same options as `getFirstTokens()`.
   * @returns Array of `Token`s between `nodeOrToken1` and `nodeOrToken2`.
   */
  /* oxlint-disable no-unused-vars */
  getLastTokensBetween(
    nodeOrToken1: NodeOrToken | Comment,
    nodeOrToken2: NodeOrToken | Comment,
    countOptions?: CountOptions | number | FilterFn | null | undefined,
  ): Token[] {
    throw new Error('`sourceCode.getLastTokensBetween` not implemented yet'); // TODO
  },
  /* oxlint-enable no-unused-vars */

  /**
   * Get the token starting at the specified index.
   * @param index - Index of the start of the token's range.
   * @param options - Options object.
   * @returns The token starting at index, or `null` if no such token.
   */
  // oxlint-disable-next-line no-unused-vars
  getTokenByRangeStart(index: number, rangeOptions?: RangeOptions | null | undefined): Token | null {
    throw new Error('`sourceCode.getTokenByRangeStart` not implemented yet'); // TODO
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

  /**
   * Convert a source text index into a (line, column) pair.
   * @param index The index of a character in a file.
   * @returns `{line, column}` location object with 1-indexed line and 0-indexed column.
   * @throws {TypeError|RangeError} If non-numeric `index`, or `index` out of range.
   */
  // oxlint-disable-next-line no-unused-vars
  getLocFromIndex(index: number): LineColumn {
    throw new Error('`sourceCode.getLocFromIndex` not implemented yet'); // TODO
  },

  /**
   * Convert a `{ line, column }` pair into a range index.
   * @param loc - A line/column location.
   * @returns The range index of the location in the file.
   * @throws {TypeError|RangeError} If `loc` is not an object with a numeric `line` and `column`,
   *   or if the `line` is less than or equal to zero, or the line or column is out of the expected range.
   */
  // oxlint-disable-next-line no-unused-vars
  getIndexFromLoc(loc: LineColumn): number {
    if (loc !== null && typeof loc === 'object') {
      const { line, column } = loc;
      if (typeof line === 'number' && typeof column === 'number') {
        initLines();

        const linesCount = lineStartOffsets.length;
        if (line <= 0 || line > linesCount) {
          throw new RangeError(
            `Line number out of range (line ${line} requested). ` +
              `Line numbers should be 1-based, and less than or equal to number of lines in file (${linesCount}).`,
          );
        }
        if (column < 0) throw new RangeError(`Invalid column number (column ${column} requested).`);

        const lineStartOffset = lineStartOffsets[line - 1];
        const offset = lineStartOffset + column;

        // Comment from ESLint implementation:
        /*
         * By design, `getIndexFromLoc({ line: lineNum, column: 0 })` should return the start index of
         * the given line, provided that the line number is valid element of `lines`. Since the
         * last element of `lines` is an empty string for files with trailing newlines, add a
         * special case where getting the index for the first location after the end of the file
         * will return the length of the file, rather than throwing an error. This allows rules to
         * use `getIndexFromLoc` consistently without worrying about edge cases at the end of a file.
         */

        let lineEndOffset;
        if (line === linesCount) {
          lineEndOffset = sourceText.length;
          if (offset <= lineEndOffset) return offset;
        } else {
          lineEndOffset = lineStartOffsets[line];
          if (offset < lineEndOffset) return offset;
        }

        throw new RangeError(
          `Column number out of range (column ${column} requested, ` +
            `but the length of line ${line} is ${lineEndOffset - lineStartOffset}).`,
        );
      }
    }

    throw new TypeError('Expected `loc` to be an object with numeric `line` and `column` properties.');
  },

  /**
   * Check whether any comments exist or not between the given 2 nodes.
   * @param nodeOrToken1 - The node to check.
   * @param nodeOrToken2 - The node to check.
   * @returns `true` if one or more comments exist.
   */
  // oxlint-disable-next-line no-unused-vars
  commentsExistBetween(nodeOrToken1: NodeOrToken, nodeOrToken2: NodeOrToken): boolean {
    throw new Error('`sourceCode.commentsExistBetween` not implemented yet'); // TODO
  },

  /**
   * Get all the ancestors of a given node.
   * @param node - AST node
   * @returns All the ancestor nodes in the AST, not including the provided node,
   *   starting from the root node at index 0 and going inwards to the parent node.
   */
  // oxlint-disable-next-line no-unused-vars
  getAncestors(node: Node): Node[] {
    throw new Error('`sourceCode.getAncestors` not implemented yet'); // TODO
  },

  /**
   * Get the variables that `node` defines.
   * This is a convenience method that passes through to the same method on the `scopeManager`.
   * @param node - The node for which the variables are obtained.
   * @returns An array of variable nodes representing the variables that `node` defines.
   */
  // oxlint-disable-next-line no-unused-vars
  getDeclaredVariables(node: Node): Variable[] {
    throw new Error('`sourceCode.getDeclaredVariables` not implemented yet'); // TODO
  },

  /**
   * Get the scope for the given node
   * @param node - The node to get the scope of.
   * @returns The scope information for this node.
   */
  // oxlint-disable-next-line no-unused-vars
  getScope(node: Node): Scope {
    throw new Error('`sourceCode.getScope` not implemented yet'); // TODO
  },

  /**
   * Mark a variable as used in the current scope
   * @param name - The name of the variable to mark as used.
   * @param refNode? - The closest node to the variable reference.
   * @returns `true` if the variable was found and marked as used, `false` if not.
   */
  // oxlint-disable-next-line no-unused-vars
  markVariableAsUsed(name: string, refNode: Node): boolean {
    throw new Error('`sourceCode.markVariableAsUsed` not implemented yet'); // TODO
  },
});

export type SourceCode = typeof SOURCE_CODE;

// Options for various `SourceCode` methods e.g. `getFirstToken`.
export interface SkipOptions {
  // Number of skipping tokens
  skip?: number;
  // `true` to include comment tokens in the result
  includeComments?: boolean;
  // Function to filter tokens
  filter?: FilterFn | null;
}

// Options for various `SourceCode` methods e.g. `getFirstTokens`.
export interface CountOptions {
  // Maximum number of tokens to return
  count?: number;
  // `true` to include comment tokens in the result
  includeComments?: boolean;
  // Function to filter tokens
  filter?: FilterFn | null;
}

// Options for various `SourceCode` methods e.g. `getTokenByRangeStart`.
export interface RangeOptions {
  // `true` to include comment tokens in the result
  includeComments?: boolean;
}

// Filter function, passed as `filter` property of `SkipOptions` and `CountOptions`.
export type FilterFn = (token: Token) => boolean;
