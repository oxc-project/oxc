import { DATA_POINTER_POS_32, SOURCE_LEN_OFFSET } from '../generated/constants.js';

// We use the deserializer which removes `ParenthesizedExpression`s from AST,
// and with `range`, `loc`, and `parent` properties on AST nodes, to match ESLint
// @ts-expect-error we need to generate `.d.ts` file for this module
import { deserializeProgramOnly, resetBuffer } from '../../dist/generated/deserialize.js';

import visitorKeys from '../generated/keys.js';
import {
  getLineColumnFromOffset,
  getNodeLoc,
  getOffsetFromLineColumn,
  initLines,
  lines,
  resetLines,
} from './location.js';

import type { Program } from '../generated/types.d.ts';
import type { Scope, ScopeManager, Variable } from './scope.ts';
import type { BufferWithArrays, Comment, Node, NodeOrToken, Ranged, Token } from './types.ts';

const { max } = Math;

const WHITESPACE_ONLY_REGEXP = /^\s*$/;

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
   * Retrieve an array containing all comments in the source code.
   * @returns Array of `Comment`s in occurrence order.
   */
  getAllComments(): Comment[] {
    if (ast === null) initAst();
    // `comments` property is a getter. Comments are deserialized lazily.
    return ast.comments;
  },

  /**
   * Get all comment tokens directly before the given node or token.
   * @param nodeOrToken - The AST node or token to check for adjacent comment tokens.
   * @returns Array of `Comment`s in occurrence order.
   */
  getCommentsBefore(nodeOrToken: NodeOrToken): Comment[] {
    if (ast === null) initAst();

    const { comments } = ast,
      commentsLength = comments.length;

    let targetStart = nodeOrToken.range[0]; // start

    let sliceStart = commentsLength;
    let sliceEnd = 0;

    // Reverse iteration isn't ideal, but this entire implementation may need to be rewritten
    // with token-based APIs to match eslint.
    for (let i = commentsLength - 1; i >= 0; i--) {
      const comment = comments[i];
      const commentEnd = comment.end;

      if (commentEnd < targetStart) {
        const gap = sourceText.slice(commentEnd, targetStart);
        if (WHITESPACE_ONLY_REGEXP.test(gap)) {
          // Nothing except whitespace between end of comment and start of `nodeOrToken`
          sliceStart = sliceEnd = i + 1;
          targetStart = comment.start;
        }
        break;
      }
    }

    for (let i = sliceEnd - 1; i >= 0; i--) {
      const comment = comments[i];
      const gap = sourceText.slice(comment.end, targetStart);
      if (WHITESPACE_ONLY_REGEXP.test(gap)) {
        // Nothing except whitespace between end of comment and start of `nodeOrToken`
        sliceStart = i;
        targetStart = comment.start;
      } else {
        break;
      }
    }

    return comments.slice(sliceStart, sliceEnd);
  },

  /**
   * Get all comment tokens directly after the given node or token.
   * @param nodeOrToken - The AST node or token to check for adjacent comment tokens.
   * @returns Array of `Comment`s in occurrence order.
   */
  getCommentsAfter(nodeOrToken: NodeOrToken): Comment[] {
    if (ast === null) initAst();

    const { comments } = ast,
      commentsLength = comments.length;

    let targetEnd = nodeOrToken.range[1]; // end

    const commentsAfter: Comment[] = [];
    for (let i = 0; i < commentsLength; i++) {
      const comment = comments[i],
        commentStart = comment.start;

      if (commentStart < targetEnd) {
        continue;
      }
      const gap = sourceText.slice(targetEnd, commentStart);
      if (WHITESPACE_ONLY_REGEXP.test(gap)) {
        // Nothing except whitespace between end of `nodeOrToken` and start of comment
        commentsAfter.push(comment);
        targetEnd = comment.end;
      } else {
        break;
      }
    }

    return commentsAfter;
  },

  /**
   * Get all comment tokens inside the given node.
   * @param node - The AST node to get the comments for.
   * @returns Array of `Comment`s in occurrence order.
   */
  getCommentsInside(node: Node): Comment[] {
    if (ast === null) initAst();

    const { comments } = ast,
      commentsLength = comments.length;

    let sliceStart = commentsLength;
    let sliceEnd: number | undefined = undefined;

    const { range } = node,
      rangeStart = range[0],
      rangeEnd = range[1];

    // Linear search for first comment within `node`'s range.
    // TODO: Use binary search.
    for (let i = 0; i < commentsLength; i++) {
      const comment = comments[i];
      if (comment.start >= rangeStart) {
        sliceStart = i;
        break;
      }
    }

    // Continued linear search for first comment outside `node`'s range.
    // Its index is used as `sliceEnd`, which is exclusive of the slice.
    for (let i = sliceStart; i < commentsLength; i++) {
      const comment = comments[i];
      if (comment.start > rangeEnd) {
        sliceEnd = i;
        break;
      }
    }

    return comments.slice(sliceStart, sliceEnd);
  },

  /**
   * Check whether any comments exist or not between the given 2 nodes.
   * @param nodeOrToken1 - The node to check.
   * @param nodeOrToken2 - The node to check.
   * @returns `true` if one or more comments exist.
   */
  commentsExistBetween(nodeOrToken1: NodeOrToken, nodeOrToken2: NodeOrToken): boolean {
    if (ast === null) initAst();

    // Find the first comment after `nodeOrToken1` ends.
    // Check if it ends before `nodeOrToken2` starts.
    const { comments } = ast,
      commentsLength = comments.length;
    const betweenRangeStart = nodeOrToken1.range[1]; // end
    for (let i = 0; i < commentsLength; i++) {
      const comment = comments[i];
      if (comment.start >= betweenRangeStart) {
        return comment.end <= nodeOrToken2.range[0]; // start
      }
    }
    return false;
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

  getLocFromIndex: getLineColumnFromOffset,
  getIndexFromLoc: getOffsetFromLineColumn,

  getAncestors,

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

/**
 * Get all the ancestors of a given node.
 * @param node - AST node
 * @returns All the ancestor nodes in the AST, not including the provided node,
 *   starting from the root node at index 0 and going inwards to the parent node.
 */
function getAncestors(node: Node): Node[] {
  const ancestors = [];

  while (true) {
    // @ts-expect-error `parent` property should be present on `Node` type
    node = node.parent;
    if (node === null) break;
    ancestors.push(node);
  }

  return ancestors.reverse();
}

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
