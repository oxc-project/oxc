/*
 * `SourceCode` methods related to tokens.
 */

import { sourceText, initSourceText } from './source_code.js';

import type { Comment, Node, NodeOrToken, Token } from './types.ts';

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
export function getTokens(
  node: Node,
  countOptions?: CountOptions | number | FilterFn | null | undefined,
  afterCount?: number | null | undefined,
): Token[] {
  throw new Error('`sourceCode.getTokens` not implemented yet'); // TODO
}
/* oxlint-enable no-unused-vars */

/**
 * Get the first token of the given node.
 * @param node - The AST node.
 * @param skipOptions? - Options object. If this is a number then it's `options.skip`.
 *   If this is a function then it's `options.filter`.
 * @returns `Token`, or `null` if all were skipped.
 */
/* oxlint-disable no-unused-vars */
export function getFirstToken(
  node: Node,
  skipOptions?: SkipOptions | number | FilterFn | null | undefined,
): Token | null {
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
export function getFirstTokens(
  node: Node,
  countOptions?: CountOptions | number | FilterFn | null | undefined,
): Token[] {
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
export function getLastToken(
  node: Node,
  skipOptions?: SkipOptions | number | FilterFn | null | undefined,
): Token | null {
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
export function getLastTokens(node: Node, countOptions?: CountOptions | number | FilterFn | null | undefined): Token[] {
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
  skipOptions?: SkipOptions | number | FilterFn | null | undefined,
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
  countOptions?: CountOptions | number | FilterFn | null | undefined,
): Token[] {
  throw new Error('`sourceCode.getTokensBefore` not implemented yet'); // TODO
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
  skipOptions?: SkipOptions | number | FilterFn | null | undefined,
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
  countOptions?: CountOptions | number | FilterFn | null | undefined,
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
  countOptions?: CountOptions | number | FilterFn | null | undefined,
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
  skipOptions?: SkipOptions | null | undefined,
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
  countOptions?: CountOptions | number | FilterFn | null | undefined,
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
  skipOptions?: SkipOptions | null | undefined,
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
  countOptions?: CountOptions | number | FilterFn | null | undefined,
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
export function getTokenByRangeStart(index: number, rangeOptions?: RangeOptions | null | undefined): Token | null {
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
