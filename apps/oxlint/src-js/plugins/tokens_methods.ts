/*
 * `SourceCode` methods related to tokens.
 */

import { cachedTokens, tokensInt32, tokensLen, initTokensBuffer, getToken } from "./tokens.ts";
import {
  tokensAndCommentsInt32,
  tokensAndCommentsLen,
  getTokenOrComment,
  getTokenOrCommentEnd,
  initTokensAndCommentsBuffer,
} from "./tokens_and_comments.ts";
import { debugAssertIsNonNull } from "../utils/asserts.ts";

import type { Node, NodeOrToken } from "./types.ts";
import type { Token } from "./tokens.ts";
import type { TokenOrComment } from "./tokens_and_comments.ts";

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
 * Options for `getTokenByRangeStart`.
 */
export interface RangeOptions {
  /** `true` to include comment tokens in the result */
  includeComments?: boolean;
}

/**
 * Filter function, passed as `filter` property of `SkipOptions` and `CountOptions`.
 */
export type FilterFn = (token: TokenOrComment) => boolean;

/**
 * Whether `Options` may include comment tokens in the result.
 * Resolves to `true` if `Options` has an `includeComments` property whose type includes `true`
 * (i.e. it's `true`, `boolean`, or `boolean | undefined`), and `false` otherwise.
 */
type MayIncludeComments<Options> = Options extends { includeComments: false }
  ? false
  : "includeComments" extends keyof Options
    ? true
    : false;

/**
 * Resolves to `TokenOrComment` if `Options` may include comments, `Token` otherwise.
 */
type TokenResult<Options> = MayIncludeComments<Options> extends true ? TokenOrComment : Token;

// `SkipOptions` object used by `getTokenOrCommentBefore` and `getTokenOrCommentAfter`.
// This object is reused over and over to avoid creating a new options object on each call.
const INCLUDE_COMMENTS_SKIP_OPTIONS: SkipOptions = { includeComments: true, skip: 0 };

/**
 * Get all tokens that are related to the given node.
 * @param node - The AST node.
 * @param countOptions? - Options object. If is a function, equivalent to `{ filter: fn }`.
 * @returns Array of `Token`s, or array of `Token | Comment`s if `includeComments` is `true`.
 */
/**
 * Get all tokens that are related to the given node.
 * @param node - The AST node.
 * @param beforeCount? - The number of tokens before the node to retrieve.
 * @param afterCount? - The number of tokens after the node to retrieve.
 * @returns Array of `Token`s, or array of `Token | Comment`s if `includeComments` is `true`.
 */
export function getTokens<Options extends CountOptions | number | FilterFn | null | undefined>(
  node: Node,
  countOptions?: Options,
  afterCount?: number | null,
): TokenResult<Options>[] {
  // TypeScript cannot verify conditional return types within the function body,
  // so we use `Result` alias + casts on return statements
  type Result = TokenResult<Options>[];

  // Maximum number of tokens to return
  let count = typeof countOptions === "object" && countOptions !== null ? countOptions.count : null;

  // Number of preceding tokens to additionally return
  const beforeCount = typeof countOptions === "number" ? countOptions : 0;

  // Number of following tokens to additionally return
  afterCount =
    (typeof countOptions === "number" || typeof countOptions === "undefined") &&
    typeof afterCount === "number"
      ? afterCount
      : 0;

  // Function to filter tokens
  const filter: FilterFn | null | undefined =
    typeof countOptions === "function"
      ? countOptions
      : typeof countOptions === "object" && countOptions !== null
        ? countOptions.filter
        : null;

  const includeComments = getIncludeComments(countOptions);

  let int32: Int32Array, len: number;
  if (includeComments === false) {
    if (tokensInt32 === null) initTokensBuffer();
    debugAssertIsNonNull(tokensInt32);
    int32 = tokensInt32;
    len = tokensLen;
  } else {
    if (tokensAndCommentsInt32 === null) initTokensAndCommentsBuffer();
    debugAssertIsNonNull(tokensAndCommentsInt32);
    int32 = tokensAndCommentsInt32;
    len = tokensAndCommentsLen;
  }

  const { range } = node,
    rangeStart = range[0],
    rangeEnd = range[1];

  // Binary search for first token past `node`'s start
  let sliceStart = firstTokenAtOrAfter(int32, rangeStart, 0, len);
  // Binary search for first token past `node`'s end
  let sliceEnd = firstTokenAtOrAfter(int32, rangeEnd, sliceStart, len);

  sliceStart = Math.max(0, sliceStart - beforeCount);
  sliceEnd = Math.min(sliceEnd + afterCount, len);

  if (typeof filter !== "function") {
    const end = Math.min(sliceStart + (count ?? sliceEnd), sliceEnd);
    return collectEntries(sliceStart, end, includeComments) as Result;
  }

  const allTokens: TokenOrComment[] = [];

  if (typeof count !== "number") {
    for (let i = sliceStart; i < sliceEnd; i++) {
      const token = getEntry(i, includeComments);
      if (filter(token)) allTokens.push(token);
    }
    return allTokens as Result;
  }

  for (let i = sliceStart; i < sliceEnd && count > 0; i++) {
    const token = getEntry(i, includeComments);
    if (filter(token)) {
      allTokens.push(token);
      count--;
    }
  }

  return allTokens as Result;
}

/**
 * Get the first token of the given node.
 * @param node - The AST node.
 * @param skipOptions? - Options object.
 *   If is a number, equivalent to `{ skip: n }`.
 *   If is a function, equivalent to `{ filter: fn }`.
 * @returns `Token` (or `Token | Comment` if `includeComments` is `true`), or `null` if none found.
 */
export function getFirstToken<Options extends SkipOptions | number | FilterFn | null | undefined>(
  node: Node,
  skipOptions?: Options,
): TokenResult<Options> | null {
  // TypeScript cannot verify conditional return types within the function body,
  // so we use `Result` alias + casts on return statements
  type Result = TokenResult<Options> | null;

  // Number of tokens at the beginning of the given node to skip
  let skip =
    typeof skipOptions === "number"
      ? skipOptions
      : typeof skipOptions === "object" && skipOptions !== null
        ? skipOptions.skip
        : null;

  // Filter function
  const filter: FilterFn | null | undefined =
    typeof skipOptions === "function"
      ? skipOptions
      : typeof skipOptions === "object" && skipOptions !== null
        ? skipOptions.filter
        : null;

  const includeComments = getIncludeComments(skipOptions);

  let int32: Int32Array, len: number;
  if (includeComments === false) {
    if (tokensInt32 === null) initTokensBuffer();
    debugAssertIsNonNull(tokensInt32);
    int32 = tokensInt32;
    len = tokensLen;
  } else {
    if (tokensAndCommentsInt32 === null) initTokensAndCommentsBuffer();
    debugAssertIsNonNull(tokensAndCommentsInt32);
    int32 = tokensAndCommentsInt32;
    len = tokensAndCommentsLen;
  }

  const { range } = node,
    rangeStart = range[0],
    rangeEnd = range[1];

  // Binary search for first token past `node`'s start
  const startIndex = firstTokenAtOrAfter(int32, rangeStart, 0, len);

  if (typeof filter !== "function") {
    const skipTo = startIndex + (skip ?? 0);
    if (skipTo >= len) return null;
    if (entryStart(skipTo, int32) >= rangeEnd) return null;
    return getEntry(skipTo, includeComments) as Result;
  }

  if (typeof skip !== "number") {
    for (let i = startIndex; i < len; i++) {
      if (entryStart(i, int32) >= rangeEnd) return null;
      const token = getEntry(i, includeComments);
      if (filter(token)) return token as Result;
    }
  } else {
    for (let i = startIndex; i < len; i++) {
      if (entryStart(i, int32) >= rangeEnd) return null;
      const token = getEntry(i, includeComments);
      if (filter(token)) {
        if (skip <= 0) return token as Result;
        skip--;
      }
    }
  }

  return null;
}

/**
 * Get the first tokens of the given node.
 * @param node - The AST node.
 * @param countOptions? - Options object.
 *   If is a number, equivalent to `{ count: n }`.
 *   If is a function, equivalent to `{ filter: fn }`.
 * @returns Array of `Token`s, or array of `Token | Comment`s if `includeComments` is `true`.
 */
export function getFirstTokens<Options extends CountOptions | number | FilterFn | null | undefined>(
  node: Node,
  countOptions?: Options,
): TokenResult<Options>[] {
  // TypeScript cannot verify conditional return types within the function body,
  // so we use `Result` alias + casts on return statements
  type Result = TokenResult<Options>[];

  const count =
    typeof countOptions === "number"
      ? countOptions
      : typeof countOptions === "object" && countOptions !== null
        ? countOptions.count
        : null;

  const filter: FilterFn | null | undefined =
    typeof countOptions === "function"
      ? countOptions
      : typeof countOptions === "object" && countOptions !== null
        ? countOptions.filter
        : null;

  const includeComments = getIncludeComments(countOptions);

  let int32: Int32Array, len: number;
  if (includeComments === false) {
    if (tokensInt32 === null) initTokensBuffer();
    debugAssertIsNonNull(tokensInt32);
    int32 = tokensInt32;
    len = tokensLen;
  } else {
    if (tokensAndCommentsInt32 === null) initTokensAndCommentsBuffer();
    debugAssertIsNonNull(tokensAndCommentsInt32);
    int32 = tokensAndCommentsInt32;
    len = tokensAndCommentsLen;
  }

  const { range } = node,
    rangeStart = range[0],
    rangeEnd = range[1];

  // Binary search for first token past `node`'s start
  const sliceStart = firstTokenAtOrAfter(int32, rangeStart, 0, len);
  // Binary search for first token past `node`'s end
  const sliceEnd = firstTokenAtOrAfter(int32, rangeEnd, sliceStart, len);

  if (typeof filter !== "function") {
    if (typeof count !== "number") {
      return collectEntries(sliceStart, sliceEnd, includeComments) as Result;
    }
    return collectEntries(
      sliceStart,
      Math.min(sliceStart + count, sliceEnd),
      includeComments,
    ) as Result;
  }

  const firstTokens: TokenOrComment[] = [];
  if (typeof count !== "number") {
    for (let i = sliceStart; i < sliceEnd; i++) {
      const token = getEntry(i, includeComments);
      if (filter(token)) firstTokens.push(token);
    }
  } else {
    for (let i = sliceStart; i < sliceEnd && firstTokens.length < count; i++) {
      const token = getEntry(i, includeComments);
      if (filter(token)) firstTokens.push(token);
    }
  }
  return firstTokens as Result;
}

/**
 * Get the last token of the given node.
 * @param node - The AST node.
 * @param skipOptions? - Options object.
 *   If is a number, equivalent to `{ skip: n }`.
 *   If is a function, equivalent to `{ filter: fn }`.
 * @returns `Token` (or `Token | Comment` if `includeComments` is `true`), or `null` if none found.
 */
export function getLastToken<Options extends SkipOptions | number | FilterFn | null | undefined>(
  node: Node,
  skipOptions?: Options,
): TokenResult<Options> | null {
  // TypeScript cannot verify conditional return types within the function body,
  // so we use `Result` alias + casts on return statements
  type Result = TokenResult<Options> | null;

  // Number of tokens at the end of the given node to skip
  let skip =
    typeof skipOptions === "number"
      ? skipOptions
      : typeof skipOptions === "object" && skipOptions !== null
        ? skipOptions.skip
        : null;

  const filter: FilterFn | null | undefined =
    typeof skipOptions === "function"
      ? skipOptions
      : typeof skipOptions === "object" && skipOptions !== null
        ? skipOptions.filter
        : null;

  const includeComments = getIncludeComments(skipOptions);

  let int32: Int32Array, len: number;
  if (includeComments === false) {
    if (tokensInt32 === null) initTokensBuffer();
    debugAssertIsNonNull(tokensInt32);
    int32 = tokensInt32;
    len = tokensLen;
  } else {
    if (tokensAndCommentsInt32 === null) initTokensAndCommentsBuffer();
    debugAssertIsNonNull(tokensAndCommentsInt32);
    int32 = tokensAndCommentsInt32;
    len = tokensAndCommentsLen;
  }

  const { range } = node,
    rangeStart = range[0],
    rangeEnd = range[1];

  // Binary search for token immediately before `node`'s end
  const lastTokenIndex = firstTokenAtOrAfter(int32, rangeEnd, 0, len) - 1;

  if (typeof filter !== "function") {
    const skipTo = lastTokenIndex - (skip ?? 0);
    if (skipTo < 0) return null;
    if (entryStart(skipTo, int32) < rangeStart) return null;
    return getEntry(skipTo, includeComments) as Result;
  }

  if (typeof skip !== "number") {
    for (let i = lastTokenIndex; i >= 0; i--) {
      if (entryStart(i, int32) < rangeStart) return null;
      const token = getEntry(i, includeComments);
      if (filter(token)) return token as Result;
    }
  } else {
    for (let i = lastTokenIndex; i >= 0; i--) {
      if (entryStart(i, int32) < rangeStart) return null;
      const token = getEntry(i, includeComments);
      if (filter(token)) {
        if (skip <= 0) return token as Result;
        skip--;
      }
    }
  }

  return null;
}

/**
 * Get the last tokens of the given node.
 * @param node - The AST node.
 * @param countOptions? - Options object.
 *   If is a number, equivalent to `{ count: n }`.
 *   If is a function, equivalent to `{ filter: fn }`.
 * @returns Array of `Token`s, or array of `Token | Comment`s if `includeComments` is `true`.
 */
export function getLastTokens<Options extends CountOptions | number | FilterFn | null | undefined>(
  node: Node,
  countOptions?: Options,
): TokenResult<Options>[] {
  // TypeScript cannot verify conditional return types within the function body,
  // so we use `Result` alias + casts on return statements
  type Result = TokenResult<Options>[];

  // Maximum number of tokens to return
  const count =
    typeof countOptions === "number"
      ? countOptions
      : typeof countOptions === "object" && countOptions !== null
        ? countOptions.count
        : null;

  // Function to filter tokens
  const filter: FilterFn | null | undefined =
    typeof countOptions === "function"
      ? countOptions
      : typeof countOptions === "object" && countOptions !== null
        ? countOptions.filter
        : null;

  const includeComments = getIncludeComments(countOptions);

  let int32: Int32Array, len: number;
  if (includeComments === false) {
    if (tokensInt32 === null) initTokensBuffer();
    debugAssertIsNonNull(tokensInt32);
    int32 = tokensInt32;
    len = tokensLen;
  } else {
    if (tokensAndCommentsInt32 === null) initTokensAndCommentsBuffer();
    debugAssertIsNonNull(tokensAndCommentsInt32);
    int32 = tokensAndCommentsInt32;
    len = tokensAndCommentsLen;
  }

  const { range } = node,
    rangeStart = range[0],
    rangeEnd = range[1];

  // Binary search for first token past `node`'s start
  const sliceStart = firstTokenAtOrAfter(int32, rangeStart, 0, len);
  // Binary search for first token past `node`'s end
  const sliceEnd = firstTokenAtOrAfter(int32, rangeEnd, sliceStart, len);

  if (typeof filter !== "function") {
    if (typeof count !== "number") {
      return collectEntries(sliceStart, sliceEnd, includeComments) as Result;
    }
    return collectEntries(
      Math.max(sliceStart, sliceEnd - count),
      sliceEnd,
      includeComments,
    ) as Result;
  }

  const lastTokens: TokenOrComment[] = [];
  if (typeof count !== "number") {
    for (let i = sliceStart; i < sliceEnd; i++) {
      const token = getEntry(i, includeComments);
      if (filter(token)) lastTokens.push(token);
    }
  } else {
    // `count` is the number of tokens within range from the end, so we iterate in reverse
    for (let i = sliceEnd - 1; i >= sliceStart && lastTokens.length < count; i--) {
      const token = getEntry(i, includeComments);
      if (filter(token)) lastTokens.unshift(token);
    }
  }
  return lastTokens as Result;
}

/**
 * Get the token that precedes a given node or token.
 * @param nodeOrToken - The AST node or token.
 * @param skipOptions? - Options object.
 *   If is a number, equivalent to `{ skip: n }`.
 *   If is a function, equivalent to `{ filter: fn }`.
 * @returns `Token` (or `Token | Comment` if `includeComments` is `true`), or `null` if none found.
 */
export function getTokenBefore<Options extends SkipOptions | number | FilterFn | null | undefined>(
  nodeOrToken: NodeOrToken,
  skipOptions?: Options,
): TokenResult<Options> | null {
  // TypeScript cannot verify conditional return types within the function body,
  // so we use `Result` alias + casts on return statements
  type Result = TokenResult<Options> | null;

  // Number of tokens preceding the given node to skip
  let skip =
    typeof skipOptions === "number"
      ? skipOptions
      : typeof skipOptions === "object" && skipOptions !== null
        ? skipOptions.skip
        : null;

  const filter: FilterFn | null | undefined =
    typeof skipOptions === "function"
      ? skipOptions
      : typeof skipOptions === "object" && skipOptions !== null
        ? skipOptions.filter
        : null;

  const includeComments = getIncludeComments(skipOptions);

  let int32: Int32Array, len: number;
  if (includeComments === false) {
    if (tokensInt32 === null) initTokensBuffer();
    debugAssertIsNonNull(tokensInt32);
    int32 = tokensInt32;
    len = tokensLen;
  } else {
    if (tokensAndCommentsInt32 === null) initTokensAndCommentsBuffer();
    debugAssertIsNonNull(tokensAndCommentsInt32);
    int32 = tokensAndCommentsInt32;
    len = tokensAndCommentsLen;
  }

  const nodeStart = nodeOrToken.range[0];

  // Binary search for token immediately before the given node, token, or comment
  let beforeIndex = firstTokenAtOrAfter(int32, nodeStart, 0, len) - 1;

  if (typeof filter !== "function") {
    const skipTo = beforeIndex - (skip ?? 0);
    if (skipTo < 0) return null;
    return getEntry(skipTo, includeComments) as Result;
  }

  if (typeof skip !== "number") {
    while (beforeIndex >= 0) {
      const token = getEntry(beforeIndex, includeComments);
      if (filter(token)) return token as Result;
      beforeIndex--;
    }
  } else {
    while (beforeIndex >= 0) {
      const token = getEntry(beforeIndex, includeComments);
      if (filter(token)) {
        if (skip <= 0) return token as Result;
        skip--;
      }
      beforeIndex--;
    }
  }

  return null;
}

/**
 * Get the token that precedes a given node or token.
 *
 * @deprecated Use `sourceCode.getTokenBefore` with `includeComments: true` instead.
 *
 * @param nodeOrToken The AST node or token.
 * @param skip - Number of tokens to skip.
 * @returns `TokenOrComment | null`.
 */
export function getTokenOrCommentBefore(
  nodeOrToken: NodeOrToken,
  skip?: number,
): TokenOrComment | null {
  // Equivalent to `return getTokenBefore(nodeOrToken, { includeComments: true, skip });`,
  // but reuse a global object to avoid creating a new object on each call
  INCLUDE_COMMENTS_SKIP_OPTIONS.skip = skip;
  return getTokenBefore(nodeOrToken, INCLUDE_COMMENTS_SKIP_OPTIONS);
}

/**
 * Get the tokens that precede a given node or token.
 * @param nodeOrToken - The AST node or token.
 * @param countOptions? - Options object.
 *   If is a number, equivalent to `{ count: n }`.
 *   If is a function, equivalent to `{ filter: fn }`.
 * @returns Array of `Token`s, or array of `Token | Comment`s if `includeComments` is `true`.
 */
export function getTokensBefore<
  Options extends CountOptions | number | FilterFn | null | undefined,
>(nodeOrToken: NodeOrToken, countOptions?: Options): TokenResult<Options>[] {
  // TypeScript cannot verify conditional return types within the function body,
  // so we use `Result` alias + casts on return statements
  type Result = TokenResult<Options>[];

  // Maximum number of tokens to return
  const count =
    typeof countOptions === "number"
      ? Math.max(0, countOptions)
      : typeof countOptions === "object" && countOptions !== null
        ? countOptions.count
        : null;

  // Function to filter tokens
  const filter: FilterFn | null | undefined =
    typeof countOptions === "function"
      ? countOptions
      : typeof countOptions === "object" && countOptions !== null
        ? countOptions.filter
        : null;

  const includeComments = getIncludeComments(countOptions);

  let int32: Int32Array, len: number;
  if (includeComments === false) {
    if (tokensInt32 === null) initTokensBuffer();
    debugAssertIsNonNull(tokensInt32);
    int32 = tokensInt32;
    len = tokensLen;
  } else {
    if (tokensAndCommentsInt32 === null) initTokensAndCommentsBuffer();
    debugAssertIsNonNull(tokensAndCommentsInt32);
    int32 = tokensAndCommentsInt32;
    len = tokensAndCommentsLen;
  }

  const targetStart = nodeOrToken.range[0];

  // Binary search for first token past `nodeOrToken`'s start
  const sliceEnd = firstTokenAtOrAfter(int32, targetStart, 0, len);

  // Fast path for the common case
  if (typeof filter !== "function") {
    if (typeof count !== "number") return collectEntries(0, sliceEnd, includeComments) as Result;
    return collectEntries(Math.max(0, sliceEnd - count), sliceEnd, includeComments) as Result;
  }

  const tokensBefore: TokenOrComment[] = [];
  if (typeof count !== "number") {
    for (let i = 0; i < sliceEnd; i++) {
      const token = getEntry(i, includeComments);
      if (filter(token)) tokensBefore.push(token);
    }
  } else {
    // Count is the number of preceding tokens, so we iterate in reverse
    for (let i = sliceEnd - 1; i >= 0 && tokensBefore.length < count; i--) {
      const token = getEntry(i, includeComments);
      if (filter(token)) tokensBefore.unshift(token);
    }
  }
  return tokensBefore as Result;
}

/**
 * Get the token that follows a given node or token.
 * @param nodeOrToken - The AST node or token.
 * @param skipOptions? - Options object.
 *   If is a number, equivalent to `{ skip: n }`.
 *   If is a function, equivalent to `{ filter: fn }`.
 * @returns `Token` (or `Token | Comment` if `includeComments` is `true`), or `null` if none found.
 */
export function getTokenAfter<Options extends SkipOptions | number | FilterFn | null | undefined>(
  nodeOrToken: NodeOrToken,
  skipOptions?: Options,
): TokenResult<Options> | null {
  // TypeScript cannot verify conditional return types within the function body,
  // so we use `Result` alias + casts on return statements
  type Result = TokenResult<Options> | null;

  // Number of tokens following the given node to skip
  let skip =
    typeof skipOptions === "number"
      ? skipOptions
      : typeof skipOptions === "object" && skipOptions !== null
        ? skipOptions.skip
        : null;

  const filter: FilterFn | null | undefined =
    typeof skipOptions === "function"
      ? skipOptions
      : typeof skipOptions === "object" && skipOptions !== null
        ? skipOptions.filter
        : null;

  const includeComments = getIncludeComments(skipOptions);

  let int32: Int32Array, len: number;
  if (includeComments === false) {
    if (tokensInt32 === null) initTokensBuffer();
    debugAssertIsNonNull(tokensInt32);
    int32 = tokensInt32;
    len = tokensLen;
  } else {
    if (tokensAndCommentsInt32 === null) initTokensAndCommentsBuffer();
    debugAssertIsNonNull(tokensAndCommentsInt32);
    int32 = tokensAndCommentsInt32;
    len = tokensAndCommentsLen;
  }

  const rangeEnd = nodeOrToken.range[1];

  // Binary search for first token past `nodeOrToken`'s end
  const startIndex = firstTokenAtOrAfter(int32, rangeEnd, 0, len);

  // Fast path for the common case
  if (typeof filter !== "function") {
    const skipTo = startIndex + (skip ?? 0);
    if (skipTo >= len) return null;
    return getEntry(skipTo, includeComments) as Result;
  }

  if (typeof skip !== "number") {
    for (let i = startIndex; i < len; i++) {
      const token = getEntry(i, includeComments);
      if (filter(token)) return token as Result;
    }
  } else {
    for (let i = startIndex; i < len; i++) {
      const token = getEntry(i, includeComments);
      if (filter(token)) {
        if (skip <= 0) return token as Result;
        skip--;
      }
    }
  }

  return null;
}

/**
 * Get the token that follows a given node or token.
 *
 * @deprecated Use `sourceCode.getTokenAfter` with `includeComments: true` instead.
 *
 * @param nodeOrToken The AST node or token.
 * @param skip - Number of tokens to skip.
 * @returns `TokenOrComment | null`.
 */
export function getTokenOrCommentAfter(
  nodeOrToken: NodeOrToken,
  skip?: number,
): TokenOrComment | null {
  // Equivalent to `return getTokenAfter(nodeOrToken, { includeComments: true, skip });`,
  // but reuse a global object to avoid creating a new object on each call
  INCLUDE_COMMENTS_SKIP_OPTIONS.skip = skip;
  return getTokenAfter(nodeOrToken, INCLUDE_COMMENTS_SKIP_OPTIONS);
}

/**
 * Get the tokens that follow a given node or token.
 * @param nodeOrToken - The AST node or token.
 * @param countOptions? - Options object.
 *   If is a number, equivalent to `{ count: n }`.
 *   If is a function, equivalent to `{ filter: fn }`.
 * @returns Array of `Token`s, or array of `Token | Comment`s if `includeComments` is `true`.
 */
export function getTokensAfter<Options extends CountOptions | number | FilterFn | null | undefined>(
  nodeOrToken: NodeOrToken,
  countOptions?: Options,
): TokenResult<Options>[] {
  // TypeScript cannot verify conditional return types within the function body,
  // so we use `Result` alias + casts on return statements
  type Result = TokenResult<Options>[];

  const count =
    typeof countOptions === "number"
      ? countOptions
      : typeof countOptions === "object" && countOptions !== null
        ? countOptions.count
        : null;

  const filter: FilterFn | null | undefined =
    typeof countOptions === "function"
      ? countOptions
      : typeof countOptions === "object" && countOptions !== null
        ? countOptions.filter
        : null;

  const includeComments = getIncludeComments(countOptions);

  let int32: Int32Array, len: number;
  if (includeComments === false) {
    if (tokensInt32 === null) initTokensBuffer();
    debugAssertIsNonNull(tokensInt32);
    int32 = tokensInt32;
    len = tokensLen;
  } else {
    if (tokensAndCommentsInt32 === null) initTokensAndCommentsBuffer();
    debugAssertIsNonNull(tokensAndCommentsInt32);
    int32 = tokensAndCommentsInt32;
    len = tokensAndCommentsLen;
  }

  const rangeEnd = nodeOrToken.range[1];

  // Binary search for first token past `nodeOrToken`'s end
  const sliceStart = firstTokenAtOrAfter(int32, rangeEnd, 0, len);

  // Fast path for the common case
  if (typeof filter !== "function") {
    if (typeof count !== "number") {
      return collectEntries(sliceStart, len, includeComments) as Result;
    }
    return collectEntries(sliceStart, Math.min(sliceStart + count, len), includeComments) as Result;
  }

  const tokenListAfter: TokenOrComment[] = [];
  if (typeof count !== "number") {
    for (let i = sliceStart; i < len; i++) {
      const token = getEntry(i, includeComments);
      if (filter(token)) tokenListAfter.push(token);
    }
  } else {
    for (let i = sliceStart; i < len && tokenListAfter.length < count; i++) {
      const token = getEntry(i, includeComments);
      if (filter(token)) tokenListAfter.push(token);
    }
  }
  return tokenListAfter as Result;
}

/**
 * Get all of the tokens between two non-overlapping nodes.
 * @param left - Node or token before the desired token range.
 * @param right - Node or token after the desired token range.
 * @param countOptions? - Options object. If is a function, equivalent to `{ filter: fn }`.
 * @returns Array of `Token`s, or array of `Token | Comment`s if `includeComments` is `true`.
 */
/**
 * Get all of the tokens between two non-overlapping nodes.
 * @param left - Node or token before the desired token range.
 * @param right - Node or token after the desired token range.
 * @param padding - Number of extra tokens on either side of center.
 * @returns Array of `Token`s, or array of `Token | Comment`s if `includeComments` is `true`.
 */
export function getTokensBetween<
  Options extends CountOptions | number | FilterFn | null | undefined,
>(left: NodeOrToken, right: NodeOrToken, countOptions?: Options): TokenResult<Options>[] {
  // TypeScript cannot verify conditional return types within the function body,
  // so we use `Result` alias + casts on return statements
  type Result = TokenResult<Options>[];

  const count =
    typeof countOptions === "object" && countOptions !== null ? countOptions.count : null;

  const padding = typeof countOptions === "number" ? countOptions : 0;

  const filter: FilterFn | null | undefined =
    typeof countOptions === "function"
      ? countOptions
      : typeof countOptions === "object" && countOptions !== null
        ? countOptions.filter
        : null;

  const includeComments = getIncludeComments(countOptions);

  let int32: Int32Array, len: number;
  if (includeComments === false) {
    if (tokensInt32 === null) initTokensBuffer();
    debugAssertIsNonNull(tokensInt32);
    int32 = tokensInt32;
    len = tokensLen;
  } else {
    if (tokensAndCommentsInt32 === null) initTokensAndCommentsBuffer();
    debugAssertIsNonNull(tokensAndCommentsInt32);
    int32 = tokensAndCommentsInt32;
    len = tokensAndCommentsLen;
  }

  // This range is not invariant over node order.
  // The first argument must be the left node.
  // Same as ESLint's implementation.
  const rangeStart = left.range[1],
    rangeEnd = right.range[0];

  // Binary search for first token past "between" range start
  let sliceStart = firstTokenAtOrAfter(int32, rangeStart, 0, len);
  // Binary search for first token past "between" range end
  let sliceEnd = firstTokenAtOrAfter(int32, rangeEnd, sliceStart, len);

  // Apply padding
  sliceStart = Math.max(0, sliceStart - padding);
  sliceEnd = Math.min(sliceEnd + padding, len);

  if (typeof filter !== "function") {
    if (typeof count !== "number") {
      return collectEntries(sliceStart, sliceEnd, includeComments) as Result;
    }
    return collectEntries(
      sliceStart,
      Math.min(sliceStart + count, sliceEnd),
      includeComments,
    ) as Result;
  }

  const tokensBetween: TokenOrComment[] = [];
  if (typeof count !== "number") {
    for (let i = sliceStart; i < sliceEnd; i++) {
      const token = getEntry(i, includeComments);
      if (filter(token)) tokensBetween.push(token);
    }
  } else {
    for (let i = sliceStart; i < sliceEnd && tokensBetween.length < count; i++) {
      const token = getEntry(i, includeComments);
      if (filter(token)) tokensBetween.push(token);
    }
  }
  return tokensBetween as Result;
}

/**
 * Get the first token between two non-overlapping nodes.
 * @param left - Node or token before the desired token range.
 * @param right - Node or token after the desired token range.
 * @param skipOptions? - Options object.
 *   If is a number, equivalent to `{ skip: n }`.
 *   If is a function, equivalent to `{ filter: fn }`.
 * @returns `Token` (or `Token | Comment` if `includeComments` is `true`), or `null` if none found.
 */
export function getFirstTokenBetween<
  Options extends SkipOptions | number | FilterFn | null | undefined,
>(left: NodeOrToken, right: NodeOrToken, skipOptions?: Options): TokenResult<Options> | null {
  // TypeScript cannot verify conditional return types within the function body,
  // so we use `Result` alias + casts on return statements
  type Result = TokenResult<Options> | null;

  // Number of tokens at the beginning of the "between" range to skip
  let skip =
    typeof skipOptions === "number"
      ? skipOptions
      : typeof skipOptions === "object" && skipOptions !== null
        ? skipOptions.skip
        : null;

  const filter: FilterFn | null | undefined =
    typeof skipOptions === "function"
      ? skipOptions
      : typeof skipOptions === "object" && skipOptions !== null
        ? skipOptions.filter
        : null;

  const includeComments = getIncludeComments(skipOptions);

  let int32: Int32Array, len: number;
  if (includeComments === false) {
    if (tokensInt32 === null) initTokensBuffer();
    debugAssertIsNonNull(tokensInt32);
    int32 = tokensInt32;
    len = tokensLen;
  } else {
    if (tokensAndCommentsInt32 === null) initTokensAndCommentsBuffer();
    debugAssertIsNonNull(tokensAndCommentsInt32);
    int32 = tokensAndCommentsInt32;
    len = tokensAndCommentsLen;
  }

  // This range is not invariant over node order.
  // The first argument must be the left node.
  // Same as ESLint's implementation.
  const rangeStart = left.range[1],
    rangeEnd = right.range[0];

  // Binary search for token immediately following `left`
  const firstTokenIndex = firstTokenAtOrAfter(int32, rangeStart, 0, len);

  if (typeof filter !== "function") {
    const skipTo = firstTokenIndex + (skip ?? 0);
    if (skipTo >= len) return null;
    if (entryStart(skipTo, int32) >= rangeEnd) return null;
    return getEntry(skipTo, includeComments) as Result;
  }

  if (typeof skip !== "number") {
    for (let i = firstTokenIndex; i < len; i++) {
      if (entryStart(i, int32) >= rangeEnd) return null;
      const token = getEntry(i, includeComments);
      if (filter(token)) return token as Result;
    }
  } else {
    for (let i = firstTokenIndex; i < len; i++) {
      if (entryStart(i, int32) >= rangeEnd) return null;
      const token = getEntry(i, includeComments);
      if (filter(token)) {
        if (skip <= 0) return token as Result;
        skip--;
      }
    }
  }

  return null;
}

/**
 * Get the first tokens between two non-overlapping nodes.
 * @param left - Node or token before the desired token range.
 * @param right - Node or token after the desired token range.
 * @param countOptions? - Options object.
 *   If is a number, equivalent to `{ count: n }`.
 *   If is a function, equivalent to `{ filter: fn }`.
 * @returns Array of `Token`s, or array of `Token | Comment`s if `includeComments` is `true`.
 */
export function getFirstTokensBetween<
  Options extends CountOptions | number | FilterFn | null | undefined,
>(left: NodeOrToken, right: NodeOrToken, countOptions?: Options): TokenResult<Options>[] {
  // TypeScript cannot verify conditional return types within the function body,
  // so we use `Result` alias + casts on return statements
  type Result = TokenResult<Options>[];

  const count =
    typeof countOptions === "number"
      ? countOptions
      : typeof countOptions === "object" && countOptions !== null
        ? countOptions.count
        : null;

  const filter: FilterFn | null | undefined =
    typeof countOptions === "function"
      ? countOptions
      : typeof countOptions === "object" && countOptions !== null
        ? countOptions.filter
        : null;

  const includeComments = getIncludeComments(countOptions);

  let int32: Int32Array, len: number;
  if (includeComments === false) {
    if (tokensInt32 === null) initTokensBuffer();
    debugAssertIsNonNull(tokensInt32);
    int32 = tokensInt32;
    len = tokensLen;
  } else {
    if (tokensAndCommentsInt32 === null) initTokensAndCommentsBuffer();
    debugAssertIsNonNull(tokensAndCommentsInt32);
    int32 = tokensAndCommentsInt32;
    len = tokensAndCommentsLen;
  }

  // This range is not invariant over node order.
  // The first argument must be the left node.
  // Same as ESLint's implementation.
  const rangeStart = left.range[1],
    rangeEnd = right.range[0];

  // Find the first token after `left`
  const sliceStart = firstTokenAtOrAfter(int32, rangeStart, 0, len);
  // Find the first token at or after `right`
  const sliceEnd = firstTokenAtOrAfter(int32, rangeEnd, sliceStart, len);

  if (typeof filter !== "function") {
    if (typeof count !== "number") {
      return collectEntries(sliceStart, sliceEnd, includeComments) as Result;
    }
    return collectEntries(
      sliceStart,
      Math.min(sliceStart + count, sliceEnd),
      includeComments,
    ) as Result;
  }

  const firstTokens: TokenOrComment[] = [];
  if (typeof count !== "number") {
    for (let i = sliceStart; i < sliceEnd; i++) {
      const token = getEntry(i, includeComments);
      if (filter(token)) firstTokens.push(token);
    }
  } else {
    for (let i = sliceStart; i < sliceEnd && firstTokens.length < count; i++) {
      const token = getEntry(i, includeComments);
      if (filter(token)) firstTokens.push(token);
    }
  }
  return firstTokens as Result;
}

/**
 * Get the last token between two non-overlapping nodes.
 * @param left - Node or token before the desired token range.
 * @param right - Node or token after the desired token range.
 * @param skipOptions? - Options object.
 *   If is a number, equivalent to `{ skip: n }`.
 *   If is a function, equivalent to `{ filter: fn }`.
 * @returns `Token` (or `Token | Comment` if `includeComments` is `true`), or `null` if none found.
 */
export function getLastTokenBetween<
  Options extends SkipOptions | number | FilterFn | null | undefined,
>(left: NodeOrToken, right: NodeOrToken, skipOptions?: Options): TokenResult<Options> | null {
  // TypeScript cannot verify conditional return types within the function body,
  // so we use `Result` alias + casts on return statements
  type Result = TokenResult<Options> | null;

  // Number of tokens at the end of the "between" range to skip
  let skip =
    typeof skipOptions === "number"
      ? skipOptions
      : typeof skipOptions === "object" && skipOptions !== null
        ? skipOptions.skip
        : null;

  const filter: FilterFn | null | undefined =
    typeof skipOptions === "function"
      ? skipOptions
      : typeof skipOptions === "object" && skipOptions !== null
        ? skipOptions.filter
        : null;

  const includeComments = getIncludeComments(skipOptions);

  let int32: Int32Array, len: number;
  if (includeComments === false) {
    if (tokensInt32 === null) initTokensBuffer();
    debugAssertIsNonNull(tokensInt32);
    int32 = tokensInt32;
    len = tokensLen;
  } else {
    if (tokensAndCommentsInt32 === null) initTokensAndCommentsBuffer();
    debugAssertIsNonNull(tokensAndCommentsInt32);
    int32 = tokensAndCommentsInt32;
    len = tokensAndCommentsLen;
  }

  // This range is not invariant over node order.
  // The first argument must be the left node.
  // Same as ESLint's implementation.
  const rangeStart = left.range[1],
    rangeEnd = right.range[0];

  // Binary search for token immediately preceding `right`.
  // The found token may be within the left node if there are no entries between the nodes.
  const lastTokenIndex = firstTokenAtOrAfter(int32, rangeEnd, 0, len) - 1;

  // Fast path for the common case
  if (typeof filter !== "function") {
    const skipTo = lastTokenIndex - (skip ?? 0);
    if (skipTo < 0) return null;
    if (entryStart(skipTo, int32) < rangeStart) return null;
    return getEntry(skipTo, includeComments) as Result;
  }

  if (typeof skip !== "number") {
    for (let i = lastTokenIndex; i >= 0; i--) {
      if (entryStart(i, int32) < rangeStart) return null;
      const token = getEntry(i, includeComments);
      if (filter(token)) return token as Result;
    }
  } else {
    for (let i = lastTokenIndex; i >= 0; i--) {
      if (entryStart(i, int32) < rangeStart) return null;
      const token = getEntry(i, includeComments);
      if (filter(token)) {
        if (skip <= 0) return token as Result;
        skip--;
      }
    }
  }

  return null;
}

/**
 * Get the last tokens between two non-overlapping nodes.
 * @param left - Node or token before the desired token range.
 * @param right - Node or token after the desired token range.
 * @param countOptions? - Options object.
 *   If is a number, equivalent to `{ count: n }`.
 *   If is a function, equivalent to `{ filter: fn }`.
 * @returns Array of `Token`s, or array of `Token | Comment`s if `includeComments` is `true`.
 */
export function getLastTokensBetween<
  Options extends CountOptions | number | FilterFn | null | undefined,
>(left: NodeOrToken, right: NodeOrToken, countOptions?: Options): TokenResult<Options>[] {
  // TypeScript cannot verify conditional return types within the function body,
  // so we use `Result` alias + casts on return statements
  type Result = TokenResult<Options>[];

  const count =
    typeof countOptions === "number"
      ? countOptions
      : typeof countOptions === "object" && countOptions !== null
        ? countOptions.count
        : null;

  const filter: FilterFn | null | undefined =
    typeof countOptions === "function"
      ? countOptions
      : typeof countOptions === "object" && countOptions !== null
        ? countOptions.filter
        : null;

  const includeComments = getIncludeComments(countOptions);

  let int32: Int32Array, len: number;
  if (includeComments === false) {
    if (tokensInt32 === null) initTokensBuffer();
    debugAssertIsNonNull(tokensInt32);
    int32 = tokensInt32;
    len = tokensLen;
  } else {
    if (tokensAndCommentsInt32 === null) initTokensAndCommentsBuffer();
    debugAssertIsNonNull(tokensAndCommentsInt32);
    int32 = tokensAndCommentsInt32;
    len = tokensAndCommentsLen;
  }

  // This range is not invariant over node order.
  // The first argument must be the left node.
  // Same as ESLint's implementation.
  const rangeStart = left.range[1],
    rangeEnd = right.range[0];

  // Binary search for first token past "between" range start
  const sliceStart = firstTokenAtOrAfter(int32, rangeStart, 0, len);
  // Binary search for first token past "between" range end
  const sliceEnd = firstTokenAtOrAfter(int32, rangeEnd, sliceStart, len);

  // Fast path for the common case
  if (typeof filter !== "function") {
    if (typeof count !== "number") {
      return collectEntries(sliceStart, sliceEnd, includeComments) as Result;
    }
    return collectEntries(
      Math.max(sliceStart, sliceEnd - count),
      sliceEnd,
      includeComments,
    ) as Result;
  }

  const tokensBetween: TokenOrComment[] = [];
  if (typeof count !== "number") {
    for (let i = sliceStart; i < sliceEnd; i++) {
      const token = getEntry(i, includeComments);
      if (filter(token)) tokensBetween.push(token);
    }
  } else {
    // Count is the number of preceding tokens, so we iterate in reverse
    for (let i = sliceEnd - 1; i >= sliceStart && tokensBetween.length < count; i--) {
      const token = getEntry(i, includeComments);
      if (filter(token)) tokensBetween.unshift(token);
    }
  }
  return tokensBetween as Result;
}

/**
 * Get the token starting at the specified index.
 * @param offset - Start offset of the token.
 * @param rangeOptions - Options object.
 * @returns `Token` (or `Token | Comment` if `includeComments` is `true`), or `null` if none found.
 */
export function getTokenByRangeStart<Options extends RangeOptions | null | undefined>(
  offset: number,
  rangeOptions?: Options,
): TokenResult<Options> | null {
  // TypeScript cannot verify conditional return types within the function body,
  // so we use `Result` alias + casts on return statements
  type Result = TokenResult<Options> | null;

  const includeComments =
    typeof rangeOptions === "object" &&
    rangeOptions !== null &&
    "includeComments" in rangeOptions &&
    !!rangeOptions.includeComments;

  let int32: Int32Array, len: number;
  if (includeComments === false) {
    if (tokensInt32 === null) initTokensBuffer();
    debugAssertIsNonNull(tokensInt32);
    int32 = tokensInt32;
    len = tokensLen;
  } else {
    if (tokensAndCommentsInt32 === null) initTokensAndCommentsBuffer();
    debugAssertIsNonNull(tokensAndCommentsInt32);
    int32 = tokensAndCommentsInt32;
    len = tokensAndCommentsLen;
  }

  // Binary search for token starting at the given index.
  //
  // Note: Source text is limited to 1 GiB max, so offsets cannot exceed 2^30.
  // This makes it safe to use `>> 1` for division by 2 (which is faster than `>>> 1`).
  for (let lo = 0, hi = len; lo < hi; ) {
    const mid = (lo + hi) >> 1;
    const tokenStart = int32[mid << 2];
    if (tokenStart < offset) {
      lo = mid + 1;
    } else if (tokenStart > offset) {
      hi = mid;
    } else {
      return getEntry(mid, includeComments) as Result;
    }
  }

  return null;
}

const JSX_WHITESPACE_REGEXP = /\s/u;

/**
 * Determine if two nodes or tokens have at least one whitespace character between them.
 * Order does not matter.
 *
 * Returns `false` if the given nodes or tokens overlap.
 *
 * Checks for whitespace *between tokens*, not including whitespace *inside tokens*.
 * e.g. Returns `false` for `isSpaceBetween(x, y)` in `x+" "+y`.
 *
 * @param first - The first node or token to check between.
 * @param second - The second node or token to check between.
 * @returns `true` if there is a whitespace character between
 *   any of the tokens found between the two given nodes or tokens.
 */
export function isSpaceBetween(first: NodeOrToken, second: NodeOrToken): boolean {
  if (tokensAndCommentsInt32 === null) initTokensAndCommentsBuffer();
  debugAssertIsNonNull(tokensAndCommentsInt32);

  const range1 = first.range,
    range2 = second.range;

  // Find the range between the two nodes/tokens.
  //
  // Unlike other methods which require the user to pass the nodes in order of appearance,
  // `isSpaceBetween()` is invariant over the sequence of the two nodes.
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
  let rangeStart: number = range1[0],
    rangeEnd: number = range2[0];
  if (rangeStart < rangeEnd) {
    rangeStart = range1[1];
  } else {
    rangeEnd = rangeStart;
    rangeStart = range2[1];
  }

  // Binary search for the first token past `rangeStart`.
  // Unless `first` and `second` are adjacent or overlapping,
  // the token will be the first token/comment between the two nodes.
  let index = firstTokenAtOrAfter(tokensAndCommentsInt32, rangeStart, 0, tokensAndCommentsLen);

  for (let lastTokenEnd = rangeStart; index < tokensAndCommentsLen; index++) {
    const tokenStart = tokensAndCommentsInt32[index << 2];
    // The first token of the later node should undergo the check in the second branch
    if (tokenStart > rangeEnd) break;
    if (tokenStart !== lastTokenEnd) return true;
    lastTokenEnd = entryEnd(index, true);
  }

  return false;
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
 * @param first - The first node or token to check between.
 * @param second - The second node or token to check between.
 * @returns `true` if there is a whitespace character between
 *   any of the tokens found between the two given nodes or tokens.
 */
export function isSpaceBetweenTokens(first: NodeOrToken, second: NodeOrToken): boolean {
  if (tokensAndCommentsInt32 === null) initTokensAndCommentsBuffer();
  debugAssertIsNonNull(tokensAndCommentsInt32);

  const range1 = first.range,
    range2 = second.range;

  // Find the range between the two nodes/tokens.
  // Unlike other methods which require the user to pass the nodes in order of appearance,
  // `isSpaceBetweenTokens()` is invariant over the sequence of the two nodes.
  // See comment in `isSpaceBetween` about why this is a single branch.
  let rangeStart: number = range1[0],
    rangeEnd: number = range2[0];
  if (rangeStart < rangeEnd) {
    rangeStart = range1[1];
  } else {
    rangeEnd = rangeStart;
    rangeStart = range2[1];
  }

  // Binary search for the first token past `rangeStart`.
  // Unless `first` and `second` are adjacent or overlapping,
  // the token will be the first token/comment between the two nodes.
  let index = firstTokenAtOrAfter(tokensAndCommentsInt32, rangeStart, 0, tokensAndCommentsLen);

  for (let lastTokenEnd = rangeStart; index < tokensAndCommentsLen; index++) {
    const tokenStart = tokensAndCommentsInt32[index << 2];

    // The first token of the later node should undergo the check in the second branch
    if (tokenStart > rangeEnd) break;

    // Deserialize to check type/value for JSXText whitespace detection
    const token = getTokenOrComment(index);
    if (
      tokenStart !== lastTokenEnd ||
      (tokenStart < rangeEnd && token.type === "JSXText" && JSX_WHITESPACE_REGEXP.test(token.value))
    ) {
      return true;
    }
    lastTokenEnd = token.end;
  }

  return false;
}

/**
 * Extract `includeComments` boolean from options.
 *
 * @param options - Options object, number, function, or nullish
 * @returns `true` if `options` has `includeComments: true`
 */
function getIncludeComments(
  options: SkipOptions | CountOptions | number | FilterFn | null | undefined,
): boolean {
  return (
    typeof options === "object" &&
    options !== null &&
    "includeComments" in options &&
    !!options.includeComments
  );
}

/**
 * Get a token at `index`, deserializing if needed.
 * For `includeComments` mode, gets from the merged buffer instead.
 *
 * @param index - Entry index in the tokens or merged buffer
 * @param includeComments - Whether to use the merged tokens-and-comments buffer
 * @returns Deserialized token or comment
 */
function getEntry(index: number, includeComments: boolean): TokenOrComment {
  return includeComments === true ? getTokenOrComment(index) : getToken(index);
}

/**
 * Get `start` offset of token/comment at `index` from the given buffer.
 *
 * @param index - Entry index
 * @param int32 - The `Int32Array` buffer (tokens or merged)
 * @returns Start offset in source text
 */
function entryStart(index: number, int32: Int32Array): number {
  return int32[index << 2];
}

/**
 * Get `end` offset of token/comment at `index`.
 * For tokens-only, reads from `tokensInt32`. For `includeComments`, looks up from the original buffer.
 *
 * @param index - Entry index in the tokens or merged buffer
 * @param includeComments - Whether to use the merged tokens-and-comments buffer
 * @returns End offset in source text
 */
function entryEnd(index: number, includeComments: boolean): number {
  return includeComments === true ? getTokenOrCommentEnd(index) : tokensInt32![(index << 2) + 1];
}

/**
 * Collect tokens/comments from `startIndex` (inclusive) to `endIndex` (exclusive) into an array.
 * Deserializes each token/comment on demand.
 *
 * For tokens-only mode, batch-deserializes then slices `cachedTokens`.
 * For `includeComments` mode, builds the array entry by entry from the merged buffer.
 *
 * @param startIndex - First entry index (inclusive)
 * @param endIndex - Last entry index (exclusive)
 * @param includeComments - Whether to use the merged tokens-and-comments buffer
 * @returns Array of tokens (and optionally comments)
 */
function collectEntries(
  startIndex: number,
  endIndex: number,
  includeComments: boolean,
): TokenOrComment[] {
  if (includeComments === false) {
    // Batch-deserialize tokens in range, then slice from `cachedTokens` array
    for (let i = startIndex; i < endIndex; i++) {
      getToken(i);
    }
    return cachedTokens!.slice(startIndex, endIndex) as TokenOrComment[];
  }

  const len = endIndex - startIndex;
  if (len === 0) return [];

  // Pre-allocate with correct size. Write `null` into first entry to transition to PACKED_ELEMENTS before the loop.
  // oxlint-disable-next-line unicorn/no-new-array
  const tokensAndCommentsSubset: TokenOrComment[] = new Array(len).fill(0);
  tokensAndCommentsSubset[0] = null!;
  let i = 0;
  do {
    tokensAndCommentsSubset[i] = getTokenOrComment(startIndex + i);
  } while (++i < len);
  return tokensAndCommentsSubset;
}

/**
 * Find the index of the first entry in a `Int32Array` buffer whose `start` is >= `offset`, via binary search.
 *
 * Each entry occupies 4 x u32s (16 bytes), with `start` as the first u32.
 * Searched range starts at `startIndex` and ends at `length`.
 *
 * Returns `length` if all entries have `start` < `offset`.
 *
 * Note: Source text is limited to 1 GiB max, so number of tokens cannot exceed 2^30.
 * This makes it safe to use `>> 1` for division by 2 below (which is faster than `>>> 1`).
 *
 * @param int32 - `Int32Array` buffer (tokens, comments, or tokensAndComments)
 * @param offset - Source offset to search for
 * @param startIndex - Starting entry index for the search
 * @param length - Total number of entries in the buffer
 * @returns Index of first entry with `start >= offset`
 */
export function firstTokenAtOrAfter(
  int32: Int32Array,
  offset: number,
  startIndex: number,
  length: number,
): number {
  for (let endIndex = length; startIndex < endIndex; ) {
    const mid = (startIndex + endIndex) >> 1;
    if (int32[mid << 2] < offset) {
      startIndex = mid + 1;
    } else {
      endIndex = mid;
    }
  }
  return startIndex;
}
