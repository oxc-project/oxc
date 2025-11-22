/*
 * `SourceCode` methods related to tokens.
 */

import { parse } from '@typescript-eslint/typescript-estree';
import { sourceText, initSourceText } from './source_code.js';
import { debugAssertIsNonNull } from '../utils/asserts.js';

import type { Comment, Node, NodeOrToken } from './types.ts';
import type { Span } from './location.ts';

const { max } = Math;

/**
 * Options for various `SourceCode` methods e.g. `getFirstToken`.
 */
export interface SkipOptions {
  /** Number of skipping tokens */
  skip?: number;
  /** `true` to include comment tokens in the result */
  includeComments?: boolean;
  /** Function to filter tokens */
  filter?: FilterFn | null;
}

/**
 * Options for various `SourceCode` methods e.g. `getFirstTokens`.
 */
export interface CountOptions {
  /** Maximum number of tokens to return */
  count?: number;
  /** `true` to include comment tokens in the result */
  includeComments?: boolean;
  /** Function to filter tokens */
  filter?: FilterFn | null;
}

/**
 * Options for various `SourceCode` methods e.g. `getTokenByRangeStart`.
 */
export interface RangeOptions {
  /** `true` to include comment tokens in the result */
  includeComments?: boolean;
}

/**
 * Filter function, passed as `filter` property of `SkipOptions` and `CountOptions`.
 */
export type FilterFn = (token: Token) => boolean;

/**
 * AST token type.
 */
export type Token =
  | BooleanToken
  | CommentToken
  | IdentifierToken
  | JSXIdentifierToken
  | JSXTextToken
  | KeywordToken
  | NullToken
  | NumericToken
  | PrivateIdentifierToken
  | PunctuatorToken
  | RegularExpressionToken
  | StringToken
  | TemplateToken;

interface BaseToken extends Omit<Span, 'start' | 'end'> {
  type: Token['type'];
  value: string;
}

export interface BooleanToken extends BaseToken {
  type: 'Boolean';
}

export type CommentToken = BlockCommentToken | LineCommentToken;

export interface BlockCommentToken extends BaseToken {
  type: 'Block';
}

export interface LineCommentToken extends BaseToken {
  type: 'Line';
}

export interface IdentifierToken extends BaseToken {
  type: 'Identifier';
}

export interface JSXIdentifierToken extends BaseToken {
  type: 'JSXIdentifier';
}

export interface JSXTextToken extends BaseToken {
  type: 'JSXText';
}

export interface KeywordToken extends BaseToken {
  type: 'Keyword';
}

export interface NullToken extends BaseToken {
  type: 'Null';
}

export interface NumericToken extends BaseToken {
  type: 'Numeric';
}

export interface PrivateIdentifierToken extends BaseToken {
  type: 'PrivateIdentifier';
}

export interface PunctuatorToken extends BaseToken {
  type: 'Punctuator';
}

export interface RegularExpressionToken extends BaseToken {
  type: 'RegularExpression';
  regex: {
    flags: string;
    pattern: string;
  };
}

export interface StringToken extends BaseToken {
  type: 'String';
}

export interface TemplateToken extends BaseToken {
  type: 'Template';
}

// Tokens for the current file parsed by TS-ESLint.
// Created lazily only when needed.
let tokens: Token[] | null = null;
let comments: CommentToken[] | null = null;
let tokensWithComments: Token[] | null = null;

/**
 * Initialize TS-ESLint tokens for current file.
 */
function initTokens() {
  debugAssertIsNonNull(sourceText);
  ({ tokens, comments } = parse(sourceText, {
    sourceType: 'module',
    tokens: true,
    comment: true,
    // TODO: Enable JSX only when needed
    jsx: true,
  }));
}

/**
 * Discard TS-ESLint tokens to free memory.
 */
export function resetTokens() {
  tokens = null;
  comments = null;
  tokensWithComments = null;
}

/**
 * Get all tokens that are related to the given node.
 * @param node - The AST node.
 * @param countOptions? - Options object. If this is a function then it's `countOptions.filter`.
 * @returns Array of `Token`s.
 */
/**
 * Get all tokens that are related to the given node.
 * @param node - The AST node.
 * @param beforeCount? - The number of tokens before the node to retrieve.
 * @param afterCount? - The number of tokens after the node to retrieve.
 * @returns Array of `Token`s.
 */
export function getTokens(
  node: Node,
  countOptions?: CountOptions | number | FilterFn | null,
  afterCount?: number | null,
): Token[] {
  if (tokens === null) initTokens();
  debugAssertIsNonNull(tokens);
  debugAssertIsNonNull(comments);

  // Maximum number of tokens to return
  const count = typeof countOptions === 'object' && countOptions !== null ? countOptions.count : null;

  // Number of preceding tokens to additionally return
  const beforeCount = typeof countOptions === 'number' ? countOptions : 0;

  // Number of following tokens to additionally return
  afterCount =
    (typeof countOptions === 'number' || typeof countOptions === 'undefined') && typeof afterCount === 'number'
      ? afterCount
      : 0;

  // Function to filter tokens
  const filter =
    typeof countOptions === 'function'
      ? countOptions
      : typeof countOptions === 'object' && countOptions !== null
        ? countOptions.filter
        : null;

  // Whether to return comment tokens
  const includeComments =
    typeof countOptions === 'object' &&
    countOptions !== null &&
    'includeComments' in countOptions &&
    countOptions.includeComments;

  // Source array of tokens to search in
  let nodeTokens: Token[] | null = null;
  if (includeComments) {
    if (tokensWithComments === null) {
      // TODO: `tokens` and `comments` are already sorted, so there's a more efficient algorithm to merge them.
      // That'd certainly be faster in Rust, but maybe here it's faster to leave it to JS engine to sort them?
      // TODO: Once we have our own tokens which have `start` and `end` properties, we can use them instead of `range`.
      tokensWithComments = [...tokens, ...comments].sort((a, b) => a.range[0] - b.range[0]);
    }
    nodeTokens = tokensWithComments;
  } else {
    nodeTokens = tokens;
  }

  const { range } = node,
    rangeStart = range[0],
    rangeEnd = range[1];

  // Binary search for first token within `node`'s range
  const tokensLength = nodeTokens.length;
  let sliceStart = tokensLength;
  for (let lo = 0; lo < sliceStart; ) {
    const mid = (lo + sliceStart) >> 1;
    if (nodeTokens[mid].range[0] < rangeStart) {
      lo = mid + 1;
    } else {
      sliceStart = mid;
    }
  }

  // Binary search for the first token outside `node`'s range
  let sliceEnd = tokensLength;
  for (let lo = sliceStart; lo < sliceEnd; ) {
    const mid = (lo + sliceEnd) >> 1;
    if (nodeTokens[mid].range[0] < rangeEnd) {
      lo = mid + 1;
    } else {
      sliceEnd = mid;
    }
  }

  sliceStart = max(0, sliceStart - beforeCount);
  sliceEnd += afterCount;

  nodeTokens = nodeTokens.slice(sliceStart, sliceEnd);

  // Filter before limiting by `count`
  if (filter) nodeTokens = nodeTokens.filter(filter);
  if (typeof count === 'number' && count < nodeTokens.length) nodeTokens = nodeTokens.slice(0, count);

  return nodeTokens;
}

/**
 * Get the first token of the given node.
 * @param node - The AST node.
 * @param skipOptions? - Options object. If this is a number then it's `options.skip`.
 *   If this is a function then it's `options.filter`.
 * @returns `Token`, or `null` if all were skipped.
 */
/* oxlint-disable no-unused-vars */
export function getFirstToken(node: Node, skipOptions?: SkipOptions | number | FilterFn | null): Token | null {
  throw new Error('`sourceCode.getFirstToken` not implemented yet'); // TODO
}
/* oxlint-enable no-unused-vars */

/**
 * Get the first tokens of the given node.
 * @param node - The AST node.
 * @param countOptions? - Options object. If this is a number then it's `options.count`.
 *   If this is a function then it's `options.filter`.
 * @returns Array of `Token`s.
 */
/* oxlint-disable no-unused-vars */
export function getFirstTokens(node: Node, countOptions?: CountOptions | number | FilterFn | null): Token[] {
  throw new Error('`sourceCode.getFirstTokens` not implemented yet'); // TODO
}
/* oxlint-enable no-unused-vars */

/**
 * Get the last token of the given node.
 * @param node - The AST node.
 * @param skipOptions? - Options object. Same options as `getFirstToken()`.
 * @returns `Token`, or `null` if all were skipped.
 */
/* oxlint-disable no-unused-vars */
export function getLastToken(node: Node, skipOptions?: SkipOptions | number | FilterFn | null): Token | null {
  throw new Error('`sourceCode.getLastToken` not implemented yet'); // TODO
}
/* oxlint-enable no-unused-vars */

/**
 * Get the last tokens of the given node.
 * @param node - The AST node.
 * @param countOptions? - Options object. Same options as `getFirstTokens()`.
 * @returns Array of `Token`s.
 */
// oxlint-disable-next-line no-unused-vars
export function getLastTokens(node: Node, countOptions?: CountOptions | number | FilterFn | null): Token[] {
  throw new Error('`sourceCode.getLastTokens` not implemented yet'); // TODO
}

/**
 * Get the token that precedes a given node or token.
 * @param nodeOrToken - The AST node or token.
 * @param skipOptions? - Options object. Same options as `getFirstToken()`.
 * @returns `Token`, or `null` if all were skipped.
 */
/* oxlint-disable no-unused-vars */
export function getTokenBefore(
  nodeOrToken: NodeOrToken | Comment,
  skipOptions?: SkipOptions | number | FilterFn | null,
): Token | null {
  throw new Error('`sourceCode.getTokenBefore` not implemented yet'); // TODO
}
/* oxlint-enable no-unused-vars */

/**
 * Get the token that precedes a given node or token.
 *
 * @deprecated Use `sourceCode.getTokenBefore` with `includeComments: true` instead.
 *
 * @param nodeOrToken The AST node or token.
 * @param skip - Number of tokens to skip.
 * @returns `Token`, or `null` if all were skipped.
 */
/* oxlint-disable no-unused-vars */
export function getTokenOrCommentBefore(nodeOrToken: NodeOrToken | Comment, skip?: number): Token | null {
  // TODO: Implement equivalent of:
  // `return getTokenBefore(nodeOrToken, { includeComments: true, skip });`
  // But could use a const object at top level for options object, to avoid creating temporary object on each call.
  throw new Error('`sourceCode.getTokenOrCommentBefore` not implemented yet');
}
/* oxlint-enable no-unused-vars */

/**
 * Get the tokens that precede a given node or token.
 * @param nodeOrToken - The AST node or token.
 * @param countOptions? - Options object. Same options as `getFirstTokens()`.
 * @returns Array of `Token`s.
 */
/* oxlint-disable no-unused-vars */
export function getTokensBefore(
  nodeOrToken: NodeOrToken | Comment,
  countOptions?: CountOptions | number | FilterFn | null,
): Token[] {
  if (tokens === null) initTokens();
  debugAssertIsNonNull(tokens);
  debugAssertIsNonNull(comments);

  // Maximum number of tokens to return
  const count =
    typeof countOptions === 'number'
      ? max(0, countOptions)
      : typeof countOptions === 'object' && countOptions !== null
        ? countOptions.count
        : null;

  // Function to filter tokens
  const filter =
    typeof countOptions === 'function'
      ? countOptions
      : typeof countOptions === 'object' && countOptions !== null
        ? countOptions.filter
        : null;

  // Whether to return comment tokens
  const includeComments =
    typeof countOptions === 'object' &&
    countOptions !== null &&
    'includeComments' in countOptions &&
    countOptions.includeComments;

  // Source array of tokens to search in
  let nodeTokens: Token[] | null = null;
  if (includeComments) {
    if (tokensWithComments === null) {
      tokensWithComments = [...tokens, ...comments].sort((a, b) => a.range[0] - b.range[0]);
    }
    nodeTokens = tokensWithComments;
  } else {
    nodeTokens = tokens;
  }

  const targetStart = nodeOrToken.range[0];

  let sliceEnd = 0;
  let hi = nodeTokens.length;
  while (sliceEnd < hi) {
    const mid = (sliceEnd + hi) >> 1;
    if (nodeTokens[mid].range[0] < targetStart) {
      sliceEnd = mid + 1;
    } else {
      hi = mid;
    }
  }

  let tokensBefore: Token[];
  // Fast path for the common case
  if (typeof filter !== 'function') {
    if (typeof count !== 'number') {
      tokensBefore = nodeTokens.slice(0, sliceEnd);
    } else {
      tokensBefore = nodeTokens.slice(sliceEnd - count, sliceEnd);
    }
  } else {
    if (typeof count !== 'number') {
      tokensBefore = [];
      for (let i = 0; i < sliceEnd; i++) {
        const token = nodeTokens[i];
        if (filter(token)) {
          tokensBefore.push(token);
        }
      }
    } else {
      tokensBefore = [];
      // Count is the number of preceding tokens so we iterate in reverse
      for (let i = sliceEnd - 1; i >= 0; i--) {
        const token = nodeTokens[i];
        if (filter(token)) {
          tokensBefore.unshift(token);
        }
        if (tokensBefore.length === count) {
          break;
        }
      }
    }
  }

  return tokensBefore;
}
/* oxlint-enable no-unused-vars */

/**
 * Get the token that follows a given node or token.
 * @param nodeOrToken - The AST node or token.
 * @param skipOptions? - Options object. Same options as `getFirstToken()`.
 * @returns `Token`, or `null` if all were skipped.
 */
/* oxlint-disable no-unused-vars */
export function getTokenAfter(
  nodeOrToken: NodeOrToken | Comment,
  skipOptions?: SkipOptions | number | FilterFn | null,
): Token | null {
  throw new Error('`sourceCode.getTokenAfter` not implemented yet'); // TODO
}
/* oxlint-enable no-unused-vars */

/**
 * Get the token that follows a given node or token.
 *
 * @deprecated Use `sourceCode.getTokenAfter` with `includeComments: true` instead.
 *
 * @param nodeOrToken The AST node or token.
 * @param skip - Number of tokens to skip.
 * @returns `Token`, or `null` if all were skipped.
 */
/* oxlint-disable no-unused-vars */
export function getTokenOrCommentAfter(nodeOrToken: NodeOrToken | Comment, skip?: number): Token | null {
  // TODO: Implement equivalent of:
  // `return getTokenAfter(nodeOrToken, { includeComments: true, skip });`
  // But could use a const object at top level for options object, to avoid creating temporary object on each call.
  throw new Error('`sourceCode.getTokenOrCommentAfter` not implemented yet');
}
/* oxlint-enable no-unused-vars */

/**
 * Get the tokens that follow a given node or token.
 * @param nodeOrToken - The AST node or token.
 * @param countOptions? - Options object. Same options as `getFirstTokens()`.
 * @returns Array of `Token`s.
 */
/* oxlint-disable no-unused-vars */
export function getTokensAfter(
  nodeOrToken: NodeOrToken | Comment,
  countOptions?: CountOptions | number | FilterFn | null,
): Token[] {
  throw new Error('`sourceCode.getTokensAfter` not implemented yet'); // TODO
}
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
export function getTokensBetween(
  nodeOrToken1: NodeOrToken | Comment,
  nodeOrToken2: NodeOrToken | Comment,
  countOptions?: CountOptions | number | FilterFn | null,
): Token[] {
  throw new Error('`sourceCode.getTokensBetween` not implemented yet'); // TODO
}
/* oxlint-enable no-unused-vars */

/**
 * Get the first token between two non-overlapping nodes.
 * @param nodeOrToken1 - Node before the desired token range.
 * @param nodeOrToken2 - Node after the desired token range.
 * @param skipOptions? - Options object. Same options as `getFirstToken()`.
 * @returns `Token`, or `null` if all were skipped.
 */
/* oxlint-disable no-unused-vars */
export function getFirstTokenBetween(
  nodeOrToken1: NodeOrToken | Comment,
  nodeOrToken2: NodeOrToken | Comment,
  skipOptions?: SkipOptions | null,
): Token | null {
  throw new Error('`sourceCode.getFirstTokenBetween` not implemented yet'); // TODO
}
/* oxlint-enable no-unused-vars */

/**
 * Get the first tokens between two non-overlapping nodes.
 * @param nodeOrToken1 - Node before the desired token range.
 * @param nodeOrToken2 - Node after the desired token range.
 * @param countOptions? - Options object. Same options as `getFirstTokens()`.
 * @returns Array of `Token`s between `nodeOrToken1` and `nodeOrToken2`.
 */
/* oxlint-disable no-unused-vars */
export function getFirstTokensBetween(
  nodeOrToken1: NodeOrToken | Comment,
  nodeOrToken2: NodeOrToken | Comment,
  countOptions?: CountOptions | number | FilterFn | null,
): Token[] {
  throw new Error('`sourceCode.getFirstTokensBetween` not implemented yet'); // TODO
}
/* oxlint-enable no-unused-vars */

/**
 * Get the last token between two non-overlapping nodes.
 * @param nodeOrToken1 - Node before the desired token range.
 * @param nodeOrToken2 - Node after the desired token range.
 * @param skipOptions? - Options object. Same options as `getFirstToken()`.
 * @returns `Token`, or `null` if all were skipped.
 */
/* oxlint-disable no-unused-vars */
export function getLastTokenBetween(
  nodeOrToken1: NodeOrToken | Comment,
  nodeOrToken2: NodeOrToken | Comment,
  skipOptions?: SkipOptions | null,
): Token | null {
  throw new Error('`sourceCode.getLastTokenBetween` not implemented yet'); // TODO
}
/* oxlint-enable no-unused-vars */

/**
 * Get the last tokens between two non-overlapping nodes.
 * @param nodeOrToken1 - Node before the desired token range.
 * @param nodeOrToken2 - Node after the desired token range.
 * @param countOptions? - Options object. Same options as `getFirstTokens()`.
 * @returns Array of `Token`s between `nodeOrToken1` and `nodeOrToken2`.
 */
/* oxlint-disable no-unused-vars */
export function getLastTokensBetween(
  nodeOrToken1: NodeOrToken | Comment,
  nodeOrToken2: NodeOrToken | Comment,
  countOptions?: CountOptions | number | FilterFn | null,
): Token[] {
  throw new Error('`sourceCode.getLastTokensBetween` not implemented yet'); // TODO
}
/* oxlint-enable no-unused-vars */

/**
 * Get the token starting at the specified index.
 * @param index - Index of the start of the token's range.
 * @param rangeOptions - Options object.
 * @returns The token starting at index, or `null` if no such token.
 */
// oxlint-disable-next-line no-unused-vars
export function getTokenByRangeStart(index: number, rangeOptions?: RangeOptions | null): Token | null {
  throw new Error('`sourceCode.getTokenByRangeStart` not implemented yet'); // TODO
}

// Regex that tests for whitespace.
// TODO: Is this too liberal? Should it be a more constrained set of whitespace characters?
const WHITESPACE_REGEXP = /\s/;

/**
 * Determine if two nodes or tokens have at least one whitespace character between them.
 * Order does not matter.
 *
 * Returns `false` if the given nodes or tokens overlap.
 *
 * Checks for whitespace *between tokens*, not including whitespace *inside tokens*.
 * e.g. Returns `false` for `isSpaceBetween(x, y)` in `x+" "+y`.
 *
 * TODO: Implementation is not quite right at present.
 * We don't use tokens, so return `true` for `isSpaceBetween(x, y)` in `x+" "+y`, but should return `false`.
 * Note: `checkInsideOfJSXText === false` in ESLint's implementation of `sourceCode.isSpaceBetween`.
 * https://github.com/eslint/eslint/blob/523c076866400670fb2192a3f55dbf7ad3469247/lib/languages/js/source-code/source-code.js#L182-L230
 *
 * @param nodeOrToken1 - The first node or token to check between.
 * @param nodeOrToken2 - The second node or token to check between.
 * @returns `true` if there is a whitespace character between
 *   any of the tokens found between the two given nodes or tokens.
 */
export function isSpaceBetween(nodeOrToken1: NodeOrToken, nodeOrToken2: NodeOrToken): boolean {
  const range1 = nodeOrToken1.range,
    range2 = nodeOrToken2.range,
    start1 = range1[0],
    start2 = range2[0];

  // Find the gap between the two nodes/tokens.
  //
  // 1 node/token can completely enclose another, but they can't *partially* overlap.
  // ```
  // Possible:
  // |------------|
  //    |------|
  //
  // Impossible:
  // |------------|
  //       |------------|
  // ```
  // We use that invariant to reduce this to a single branch.
  let gapStart, gapEnd;
  if (start1 < start2) {
    gapStart = range1[1]; // end1
    gapEnd = start2;
  } else {
    gapStart = range2[1]; // end2;
    gapEnd = start1;
  }

  // If `gapStart >= gapEnd`, one node encloses the other, or the two are directly adjacent
  if (gapStart >= gapEnd) return false;

  // Check if there's any whitespace in the gap
  if (sourceText === null) initSourceText();
  debugAssertIsNonNull(sourceText);

  return WHITESPACE_REGEXP.test(sourceText.slice(gapStart, gapEnd));
}

/**
 * Determine if two nodes or tokens have at least one whitespace character between them.
 * Order does not matter.
 *
 * Returns `false` if the given nodes or tokens overlap.
 *
 * Checks for whitespace *between tokens*, not including whitespace *inside tokens*.
 * e.g. Returns `false` for `isSpaceBetween(x, y)` in `x+" "+y`.
 *
 * Unlike `SourceCode#isSpaceBetween`, this function does return `true` if there is a `JSText` token between the two
 * input tokens, and it contains whitespace.
 * e.g. Returns `true` for `isSpaceBetweenTokens(x, slash)` in `<X>a b</X>`.
 *
 * @deprecated Use `sourceCode.isSpaceBetween` instead.
 *
 * TODO: Implementation is not quite right at present, for same reasons as `SourceCode#isSpaceBetween`.
 *
 * @param nodeOrToken1 - The first node or token to check between.
 * @param nodeOrToken2 - The second node or token to check between.
 * @returns `true` if there is a whitespace character between
 *   any of the tokens found between the two given nodes or tokens.
 */
export function isSpaceBetweenTokens(token1: NodeOrToken, token2: NodeOrToken): boolean {
  return isSpaceBetween(token1, token2);
}
