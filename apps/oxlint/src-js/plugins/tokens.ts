/*
 * `SourceCode` methods related to tokens.
 */

import type { Comment, Node, NodeOrToken, Token } from './types.ts';

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
