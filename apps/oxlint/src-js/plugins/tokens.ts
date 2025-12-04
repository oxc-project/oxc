/*
 * `SourceCode` methods related to tokens.
 */

import { createRequire } from "node:module";
import { sourceText } from "./source_code.ts";
import { debugAssert, debugAssertIsNonNull } from "../utils/asserts.ts";

import type { Comment, Node, NodeOrToken } from "./types.ts";
import type { Span } from "./location.ts";

const { max, min } = Math;

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
 * AST token type.
 */
export type Token =
  | BooleanToken
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

interface BaseToken extends Omit<Span, "start" | "end"> {
  value: string;
}

export interface BooleanToken extends BaseToken {
  type: "Boolean";
}

export interface IdentifierToken extends BaseToken {
  type: "Identifier";
}

export interface JSXIdentifierToken extends BaseToken {
  type: "JSXIdentifier";
}

export interface JSXTextToken extends BaseToken {
  type: "JSXText";
}

export interface KeywordToken extends BaseToken {
  type: "Keyword";
}

export interface NullToken extends BaseToken {
  type: "Null";
}

export interface NumericToken extends BaseToken {
  type: "Numeric";
}

export interface PrivateIdentifierToken extends BaseToken {
  type: "PrivateIdentifier";
}

export interface PunctuatorToken extends BaseToken {
  type: "Punctuator";
}

export interface RegularExpressionToken extends BaseToken {
  type: "RegularExpression";
  regex: {
    flags: string;
    pattern: string;
  };
}

export interface StringToken extends BaseToken {
  type: "String";
}

export interface TemplateToken extends BaseToken {
  type: "Template";
}

export interface CommentToken extends BaseToken {
  type: "Line" | "Block";
}

type TokenOrComment = Token | CommentToken;

// `SkipOptions` object used by `getTokenOrCommentBefore` and `getTokenOrCommentAfter`.
// This object is reused over and over to avoid creating a new options object on each call.
const INCLUDE_COMMENTS_SKIP_OPTIONS: SkipOptions = { includeComments: true, skip: 0 };

// Tokens for the current file parsed by TS-ESLint.
// Created lazily only when needed.
export let tokens: Token[] | null = null;
let comments: CommentToken[] | null = null;
export let tokensAndComments: (Token | CommentToken)[] | null = null;

// TS-ESLint `parse` method.
// Lazy-loaded only when needed, as it's a lot of code.
// Bundle contains both `@typescript-eslint/typescript-estree` and `typescript`.
let tsEslintParse: typeof import("@typescript-eslint/typescript-estree").parse | null = null;

/**
 * Initialize TS-ESLint tokens for current file.
 *
 * Caller must ensure `sourceText` is initialized before calling this function.
 */
export function initTokens() {
  debugAssertIsNonNull(sourceText);

  // Lazy-load TS-ESLint.
  // `./ts_eslint.cjs` is path to the bundle in `dist` directory, as well as relative path in `src-js`,
  // so is valid both in bundled `dist` output, and in unit tests.
  if (tsEslintParse === null) {
    const require = createRequire(import.meta.url);
    tsEslintParse = (
      require("./ts_eslint.cjs") as typeof import("@typescript-eslint/typescript-estree")
    ).parse;
  }

  // Our types differ a little from TS-ESLint's
  type ParseRet = { tokens: Token[]; comments: CommentToken[] };

  ({ tokens, comments } = tsEslintParse(sourceText, {
    sourceType: "module",
    tokens: true,
    comment: true,
    // TODO: Set this option dependent on source type
    jsx: true,
  }) as ParseRet);
}

/**
 * Initialize `tokensAndComments`.
 *
 * Caller must ensure `tokens` and `comments` are initialized before calling this function,
 * by calling `initTokens()` if `tokens === null`.
 */
export function initTokensAndComments() {
  debugAssertIsNonNull(tokens);
  debugAssertIsNonNull(comments);

  // TODO: Replace `range[0]` with `start` throughout this function
  // once we have our own tokens which have `start` property

  // Fast paths for file with no comments, or file which is only comments
  const commentsLength = comments.length;
  if (commentsLength === 0) {
    tokensAndComments = tokens;
    return;
  }

  const tokensLength = tokens.length;
  if (tokensLength === 0) {
    tokensAndComments = comments;
    return;
  }

  // File contains both tokens and comments.
  // Fill `tokensAndComments` with the 2 arrays interleaved in source order.
  tokensAndComments = [];

  let tokenIndex = 0,
    commentIndex = 0,
    token = tokens[0],
    comment = comments[0],
    tokenStart = token.range[0],
    commentStart = comment.range[0];

  // Push any leading comments
  while (commentStart < tokenStart) {
    // Push current comment
    tokensAndComments.push(comment);

    // If that was last comment, push all remaining tokens, and exit
    if (++commentIndex === commentsLength) {
      tokensAndComments.push(...tokens.slice(tokenIndex));
      debugCheckTokensAndComments();
      return;
    }

    // Get next comment
    comment = comments[commentIndex];
    commentStart = comment.range[0];
  }

  // Push a run of tokens, then a run of comments, and so on, until all tokens and comments are exhausted
  while (true) {
    // There's at least 1 token and 1 comment remaining, and token is first.
    // Push tokens until we reach the next comment or the end.
    do {
      // Push current token
      tokensAndComments.push(token);

      // If that was last token, push all remaining comments, and exit
      if (++tokenIndex === tokensLength) {
        tokensAndComments.push(...comments.slice(commentIndex));
        debugCheckTokensAndComments();
        return;
      }

      // Get next token
      token = tokens[tokenIndex];
      tokenStart = token.range[0];
    } while (tokenStart < commentStart);

    // There's at least 1 token and 1 comment remaining, and comment is first.
    // Push comments until we reach the next token or the end.
    do {
      // Push current comment
      tokensAndComments.push(comment);

      // If that was last comment, push all remaining tokens, and exit
      if (++commentIndex === commentsLength) {
        tokensAndComments.push(...tokens.slice(tokenIndex));
        debugCheckTokensAndComments();
        return;
      }

      // Get next comment
      comment = comments[commentIndex];
      commentStart = comment.range[0];
    } while (commentStart < tokenStart);
  }

  debugAssert(false, "Unreachable");
}

/**
 * Check `tokensAndComments` contains all tokens and comments, in ascending order.
 *
 * Only runs in debug build (tests). In release build, this function is entirely removed by minifier.
 */
function debugCheckTokensAndComments() {
  if (!DEBUG) return;

  debugAssertIsNonNull(tokens);
  debugAssertIsNonNull(comments);
  debugAssertIsNonNull(tokensAndComments);

  const expected = [...tokens, ...comments];
  expected.sort((a, b) => {
    if (a.range[0] === b.range[0]) throw new Error("2 tokens/comments have the same start");
    return a.range[0] - b.range[0];
  });

  if (tokensAndComments.length !== expected.length) {
    throw new Error("`tokensAndComments` has wrong length");
  }

  for (let i = 0; i < tokensAndComments.length; i++) {
    if (tokensAndComments[i] !== expected[i]) {
      throw new Error("`tokensAndComments` is not correctly ordered");
    }
  }
}

/**
 * Discard TS-ESLint tokens to free memory.
 */
export function resetTokens() {
  tokens = null;
  comments = null;
  tokensAndComments = null;
}

/**
 * Get all tokens that are related to the given node.
 * @param node - The AST node.
 * @param countOptions? - Options object. If is a function, equivalent to `{ filter: fn }`.
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
): TokenOrComment[] {
  if (tokens === null) initTokens();
  debugAssertIsNonNull(tokens);
  debugAssertIsNonNull(comments);

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
  const filter =
    typeof countOptions === "function"
      ? countOptions
      : typeof countOptions === "object" && countOptions !== null
        ? countOptions.filter
        : null;

  // Whether to return comment tokens
  const includeComments =
    typeof countOptions === "object" &&
    countOptions !== null &&
    "includeComments" in countOptions &&
    countOptions.includeComments;

  // Source array of tokens to search in
  let tokenList: TokenOrComment[];
  if (includeComments) {
    if (tokensAndComments === null) initTokensAndComments();
    debugAssertIsNonNull(tokensAndComments);
    tokenList = tokensAndComments;
  } else {
    tokenList = tokens;
  }

  const { range } = node,
    rangeStart = range[0],
    rangeEnd = range[1];

  // Binary search for first token past `node`'s start
  const tokensLength = tokenList.length;
  let sliceStart = tokensLength;
  for (let lo = 0; lo < sliceStart; ) {
    const mid = (lo + sliceStart) >> 1;
    if (tokenList[mid].range[0] < rangeStart) {
      lo = mid + 1;
    } else {
      sliceStart = mid;
    }
  }

  // Binary search for first token past `node`'s end
  let sliceEnd = tokensLength;
  for (let lo = sliceStart; lo < sliceEnd; ) {
    const mid = (lo + sliceEnd) >> 1;
    if (tokenList[mid].range[0] < rangeEnd) {
      lo = mid + 1;
    } else {
      sliceEnd = mid;
    }
  }

  sliceStart = max(0, sliceStart - beforeCount);
  sliceEnd = min(sliceEnd + afterCount, tokensLength);

  if (typeof filter !== "function") {
    return tokenList.slice(sliceStart, min(sliceStart + (count ?? sliceEnd), sliceEnd));
  }

  const allTokens: TokenOrComment[] = [];

  if (typeof count !== "number") {
    for (let i = sliceStart; i < sliceEnd; i++) {
      const token = tokenList[i];
      if (filter(token)) allTokens.push(token);
    }
    return allTokens;
  }

  for (let i = sliceStart; i < sliceEnd && count > 0; i++) {
    const token = tokenList[i];
    if (filter(token)) {
      allTokens.push(token);
      count--;
    }
  }

  return allTokens;
}

/**
 * Get the first token of the given node.
 * @param node - The AST node.
 * @param skipOptions? - Options object.
 *   If is a number, equivalent to `{ skip: n }`.
 *   If is a function, equivalent to `{ filter: fn }`.
 * @returns `Token`, or `null` if all were skipped.
 */
export function getFirstToken(
  node: Node,
  skipOptions?: SkipOptions | number | FilterFn | null,
): TokenOrComment | null {
  if (tokens === null) initTokens();
  debugAssertIsNonNull(tokens);
  debugAssertIsNonNull(comments);

  // Number of tokens at the beginning of the given node to skip
  let skip =
    typeof skipOptions === "number"
      ? skipOptions
      : typeof skipOptions === "object" && skipOptions !== null
        ? skipOptions.skip
        : null;

  // Filter function
  const filter =
    typeof skipOptions === "function"
      ? skipOptions
      : typeof skipOptions === "object" && skipOptions !== null
        ? skipOptions.filter
        : null;

  // Whether to include comments
  const includeComments =
    typeof skipOptions === "object" &&
    skipOptions !== null &&
    "includeComments" in skipOptions &&
    skipOptions.includeComments;

  // Source array of tokens
  let tokenList: TokenOrComment[];
  if (includeComments) {
    if (tokensAndComments === null) initTokensAndComments();
    debugAssertIsNonNull(tokensAndComments);
    tokenList = tokensAndComments;
  } else {
    tokenList = tokens;
  }

  const { range } = node,
    rangeStart = range[0],
    rangeEnd = range[1];

  // Binary search for first token past `node`'s start
  const tokensLength = tokenList.length;
  let startIndex = tokensLength;
  for (let lo = 0; lo < startIndex; ) {
    const mid = (lo + startIndex) >> 1;
    if (tokenList[mid].range[0] < rangeStart) {
      lo = mid + 1;
    } else {
      startIndex = mid;
    }
  }

  if (typeof filter !== "function") {
    const skipTo = startIndex + (skip ?? 0);
    // Avoid indexing out of bounds
    if (skipTo >= tokensLength) return null;

    const token = tokenList[skipTo];
    if (token.range[0] >= rangeEnd) return null;
    return token;
  }

  if (typeof skip !== "number") {
    for (let i = startIndex; i < tokensLength; i++) {
      const token = tokenList[i];
      if (token.range[0] >= rangeEnd) return null; // Token is outside the node
      if (filter(token)) return token;
    }
  } else {
    for (let i = startIndex; i < tokensLength; i++) {
      const token = tokenList[i];
      if (token.range[0] >= rangeEnd) return null; // Token is outside the node
      if (filter(token)) {
        if (skip <= 0) return token;
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
 * @returns Array of `Token`s.
 */
export function getFirstTokens(
  node: Node,
  countOptions?: CountOptions | number | FilterFn | null,
): TokenOrComment[] {
  if (tokens === null) initTokens();
  debugAssertIsNonNull(tokens);
  debugAssertIsNonNull(comments);

  const count =
    typeof countOptions === "number"
      ? countOptions
      : typeof countOptions === "object" && countOptions !== null
        ? countOptions.count
        : null;

  const filter =
    typeof countOptions === "function"
      ? countOptions
      : typeof countOptions === "object" && countOptions !== null
        ? countOptions.filter
        : null;

  const includeComments =
    typeof countOptions === "object" &&
    countOptions !== null &&
    "includeComments" in countOptions &&
    countOptions.includeComments;

  let tokenList: TokenOrComment[];
  if (includeComments) {
    if (tokensAndComments === null) initTokensAndComments();
    debugAssertIsNonNull(tokensAndComments);
    tokenList = tokensAndComments;
  } else {
    tokenList = tokens;
  }

  const { range } = node,
    rangeStart = range[0],
    rangeEnd = range[1];

  // Binary search for first token past `node`'s start
  const tokensLength = tokenList.length;
  let sliceStart = tokensLength;
  for (let lo = 0; lo < sliceStart; ) {
    const mid = (lo + sliceStart) >> 1;
    if (tokenList[mid].range[0] < rangeStart) {
      lo = mid + 1;
    } else {
      sliceStart = mid;
    }
  }

  // Binary search for first token past `node`'s end
  let sliceEnd = tokensLength;
  for (let lo = sliceStart; lo < sliceEnd; ) {
    const mid = (lo + sliceEnd) >> 1;
    if (tokenList[mid].range[0] < rangeEnd) {
      lo = mid + 1;
    } else {
      sliceEnd = mid;
    }
  }

  if (typeof filter !== "function") {
    if (typeof count !== "number") return tokenList.slice(sliceStart, sliceEnd);
    return tokenList.slice(sliceStart, min(sliceStart + count, sliceEnd));
  }

  const firstTokens: TokenOrComment[] = [];
  if (typeof count !== "number") {
    for (let i = sliceStart; i < sliceEnd; i++) {
      const token = tokenList[i];
      if (filter(token)) firstTokens.push(token);
    }
  } else {
    for (let i = sliceStart; i < sliceEnd && firstTokens.length < count; i++) {
      const token = tokenList[i];
      if (filter(token)) firstTokens.push(token);
    }
  }
  return firstTokens;
}

/**
 * Get the last token of the given node.
 * @param node - The AST node.
 * @param skipOptions? - Options object.
 *   If is a number, equivalent to `{ skip: n }`.
 *   If is a function, equivalent to `{ filter: fn }`.
 * @returns `Token`, or `null` if all were skipped.
 */
export function getLastToken(
  node: Node,
  skipOptions?: SkipOptions | number | FilterFn | null,
): TokenOrComment | null {
  if (tokens === null) initTokens();
  debugAssertIsNonNull(tokens);
  debugAssertIsNonNull(comments);

  // Number of tokens at the end of the given node to skip
  let skip =
    typeof skipOptions === "number"
      ? skipOptions
      : typeof skipOptions === "object" && skipOptions !== null
        ? skipOptions.skip
        : null;

  const filter =
    typeof skipOptions === "function"
      ? skipOptions
      : typeof skipOptions === "object" && skipOptions !== null
        ? skipOptions.filter
        : null;

  // Whether to return comment tokens
  const includeComments =
    typeof skipOptions === "object" &&
    skipOptions !== null &&
    "includeComments" in skipOptions &&
    skipOptions.includeComments;

  // Source array of tokens to search in
  let tokenList: TokenOrComment[];
  if (includeComments) {
    if (tokensAndComments === null) initTokensAndComments();
    debugAssertIsNonNull(tokensAndComments);
    tokenList = tokensAndComments;
  } else {
    tokenList = tokens;
  }

  const { range } = node,
    rangeStart = range[0],
    rangeEnd = range[1];

  // Binary search for token immediately before `node`'s end
  const tokensLength = tokenList.length;
  let lastTokenIndex = 0;
  for (let hi = tokensLength; lastTokenIndex < hi; ) {
    const mid = (lastTokenIndex + hi) >> 1;
    if (tokenList[mid].range[0] < rangeEnd) {
      lastTokenIndex = mid + 1;
    } else {
      hi = mid;
    }
  }

  lastTokenIndex--;

  if (typeof filter !== "function") {
    const skipTo = lastTokenIndex - (skip ?? 0);
    // Avoid indexing out of bounds
    if (skipTo < 0) return null;
    const token = tokenList[skipTo];
    if (token.range[0] < rangeStart) return null;
    return token;
  }

  if (typeof skip !== "number") {
    for (let i = lastTokenIndex; i >= 0; i--) {
      const token = tokenList[i];
      if (token.range[0] < rangeStart) return null;
      if (filter(token)) return token;
    }
  } else {
    for (let i = lastTokenIndex; i >= 0; i--) {
      const token = tokenList[i];
      if (token.range[0] < rangeStart) return null;
      if (filter(token)) {
        if (skip <= 0) return token;
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
 * @returns Array of `Token`s.
 */
export function getLastTokens(
  node: Node,
  countOptions?: CountOptions | number | FilterFn | null,
): TokenOrComment[] {
  if (tokens === null) initTokens();
  debugAssertIsNonNull(tokens);
  debugAssertIsNonNull(comments);

  // Maximum number of tokens to return
  const count =
    typeof countOptions === "number"
      ? countOptions
      : typeof countOptions === "object" && countOptions !== null
        ? countOptions.count
        : null;

  // Function to filter tokens
  const filter =
    typeof countOptions === "function"
      ? countOptions
      : typeof countOptions === "object" && countOptions !== null
        ? countOptions.filter
        : null;

  // Whether to return comment tokens
  const includeComments =
    typeof countOptions === "object" &&
    countOptions !== null &&
    "includeComments" in countOptions &&
    countOptions.includeComments;

  // Source array of tokens to search in
  let tokenList: TokenOrComment[];
  if (includeComments) {
    if (tokensAndComments === null) initTokensAndComments();
    debugAssertIsNonNull(tokensAndComments);
    tokenList = tokensAndComments;
  } else {
    tokenList = tokens;
  }

  const { range } = node,
    rangeStart = range[0],
    rangeEnd = range[1];

  // Binary search for first token past `node`'s start
  const tokensLength = tokenList.length;
  let sliceStart = tokensLength;
  for (let lo = 0; lo < sliceStart; ) {
    const mid = (lo + sliceStart) >> 1;
    if (tokenList[mid].range[0] < rangeStart) {
      lo = mid + 1;
    } else {
      sliceStart = mid;
    }
  }

  // Binary search for first token past `node`'s end
  let sliceEnd = tokensLength;
  for (let lo = sliceStart; lo < sliceEnd; ) {
    const mid = (lo + sliceEnd) >> 1;
    if (tokenList[mid].range[0] < rangeEnd) {
      lo = mid + 1;
    } else {
      sliceEnd = mid;
    }
  }

  if (typeof filter !== "function") {
    if (typeof count !== "number") return tokenList.slice(sliceStart, sliceEnd);
    return tokenList.slice(max(sliceStart, sliceEnd - count), sliceEnd);
  }

  const lastTokens: TokenOrComment[] = [];
  if (typeof count !== "number") {
    for (let i = sliceStart; i < sliceEnd; i++) {
      const token = tokenList[i];
      if (filter(token)) lastTokens.push(token);
    }
  } else {
    // `count` is the number of tokens within range from the end, so we iterate in reverse
    for (let i = sliceEnd - 1; i >= sliceStart && lastTokens.length < count; i--) {
      const token = tokenList[i];
      if (filter(token)) lastTokens.unshift(token);
    }
  }
  return lastTokens;
}

/**
 * Get the token that precedes a given node or token.
 * @param nodeOrToken - The AST node or token.
 * @param skipOptions? - Options object.
 *   If is a number, equivalent to `{ skip: n }`.
 *   If is a function, equivalent to `{ filter: fn }`.
 * @returns `Token`, or `null` if all were skipped.
 */
export function getTokenBefore(
  nodeOrToken: NodeOrToken | Comment,
  skipOptions?: SkipOptions | number | FilterFn | null,
): TokenOrComment | null {
  if (tokens === null) initTokens();
  debugAssertIsNonNull(tokens);
  debugAssertIsNonNull(comments);

  // Number of tokens preceding the given node to skip
  let skip =
    typeof skipOptions === "number"
      ? skipOptions
      : typeof skipOptions === "object" && skipOptions !== null
        ? skipOptions.skip
        : null;

  const filter =
    typeof skipOptions === "function"
      ? skipOptions
      : typeof skipOptions === "object" && skipOptions !== null
        ? skipOptions.filter
        : null;

  // Whether to return comment tokens
  const includeComments =
    typeof skipOptions === "object" &&
    skipOptions !== null &&
    "includeComments" in skipOptions &&
    skipOptions.includeComments;

  // Source array of tokens to search in
  let tokenList: TokenOrComment[];
  if (includeComments) {
    if (tokensAndComments === null) initTokensAndComments();
    debugAssertIsNonNull(tokensAndComments);
    tokenList = tokensAndComments;
  } else {
    tokenList = tokens;
  }

  const nodeStart = nodeOrToken.range[0];

  // Index of the token immediately before the given node, token, or comment.
  let beforeIndex = 0;
  let hi = tokenList.length;

  while (beforeIndex < hi) {
    const mid = (beforeIndex + hi) >> 1;
    if (tokenList[mid].range[0] < nodeStart) {
      beforeIndex = mid + 1;
    } else {
      hi = mid;
    }
  }

  beforeIndex--;

  if (typeof filter !== "function") {
    const skipTo = beforeIndex - (skip ?? 0);
    // Avoid indexing out of bounds
    if (skipTo < 0) return null;
    return tokenList[skipTo];
  }

  if (typeof skip !== "number") {
    while (beforeIndex >= 0) {
      const token = tokenList[beforeIndex];
      if (filter(token)) return token;
      beforeIndex--;
    }
  } else {
    while (beforeIndex >= 0) {
      const token = tokenList[beforeIndex];
      if (filter(token)) {
        if (skip <= 0) return token;
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
 * @returns `Token`, or `null` if all were skipped.
 */
export function getTokenOrCommentBefore(
  nodeOrToken: NodeOrToken | Comment,
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
 * @returns Array of `Token`s.
 */
export function getTokensBefore(
  nodeOrToken: NodeOrToken | Comment,
  countOptions?: CountOptions | number | FilterFn | null,
): TokenOrComment[] {
  if (tokens === null) initTokens();
  debugAssertIsNonNull(tokens);
  debugAssertIsNonNull(comments);

  // Maximum number of tokens to return
  const count =
    typeof countOptions === "number"
      ? max(0, countOptions)
      : typeof countOptions === "object" && countOptions !== null
        ? countOptions.count
        : null;

  // Function to filter tokens
  const filter =
    typeof countOptions === "function"
      ? countOptions
      : typeof countOptions === "object" && countOptions !== null
        ? countOptions.filter
        : null;

  // Whether to return comment tokens
  const includeComments =
    typeof countOptions === "object" &&
    countOptions !== null &&
    "includeComments" in countOptions &&
    countOptions.includeComments;

  // Source array of tokens to search in
  let tokenList: TokenOrComment[];
  if (includeComments) {
    if (tokensAndComments === null) initTokensAndComments();
    debugAssertIsNonNull(tokensAndComments);
    tokenList = tokensAndComments;
  } else {
    tokenList = tokens;
  }

  const targetStart = nodeOrToken.range[0];

  let sliceEnd = 0;
  let hi = tokenList.length;
  while (sliceEnd < hi) {
    const mid = (sliceEnd + hi) >> 1;
    if (tokenList[mid].range[0] < targetStart) {
      sliceEnd = mid + 1;
    } else {
      hi = mid;
    }
  }

  // Fast path for the common case
  if (typeof filter !== "function") {
    if (typeof count !== "number") return tokenList.slice(0, sliceEnd);
    return tokenList.slice(sliceEnd - count, sliceEnd);
  }

  const tokensBefore: TokenOrComment[] = [];
  if (typeof count !== "number") {
    for (let i = 0; i < sliceEnd; i++) {
      const token = tokenList[i];
      if (filter(token)) tokensBefore.push(token);
    }
  } else {
    // Count is the number of preceding tokens, so we iterate in reverse
    for (let i = sliceEnd - 1; i >= 0 && tokensBefore.length < count; i--) {
      const token = tokenList[i];
      if (filter(token)) tokensBefore.unshift(token);
    }
  }
  return tokensBefore;
}

/**
 * Get the token that follows a given node or token.
 * @param nodeOrToken - The AST node or token.
 * @param skipOptions? - Options object.
 *   If is a number, equivalent to `{ skip: n }`.
 *   If is a function, equivalent to `{ filter: fn }`.
 * @returns `Token`, or `null` if all were skipped.
 */
export function getTokenAfter(
  nodeOrToken: NodeOrToken | Comment,
  skipOptions?: SkipOptions | number | FilterFn | null,
): TokenOrComment | null {
  if (tokens === null) initTokens();
  debugAssertIsNonNull(tokens);
  debugAssertIsNonNull(comments);

  // Number of tokens following the given node to skip
  let skip =
    typeof skipOptions === "number"
      ? skipOptions
      : typeof skipOptions === "object" && skipOptions !== null
        ? skipOptions.skip
        : null;

  const filter =
    typeof skipOptions === "function"
      ? skipOptions
      : typeof skipOptions === "object" && skipOptions !== null
        ? skipOptions.filter
        : null;

  const includeComments =
    typeof skipOptions === "object" &&
    skipOptions !== null &&
    "includeComments" in skipOptions &&
    skipOptions.includeComments;

  // Source array of tokens to search in
  let tokenList: TokenOrComment[];
  if (includeComments) {
    if (tokensAndComments === null) initTokensAndComments();
    debugAssertIsNonNull(tokensAndComments);
    tokenList = tokensAndComments;
  } else {
    tokenList = tokens;
  }

  const { range } = nodeOrToken,
    rangeEnd = range[1];

  // Binary search for first token past `nodeOrToken`'s end
  const tokensLength = tokenList.length;
  let startIndex = tokensLength;
  for (let lo = 0; lo < startIndex; ) {
    const mid = (lo + startIndex) >> 1;
    if (tokenList[mid].range[0] < rangeEnd) {
      lo = mid + 1;
    } else {
      startIndex = mid;
    }
  }

  // Fast path for the common case
  if (typeof filter !== "function") {
    const skipTo = startIndex + (skip ?? 0);
    // Avoid indexing out of bounds
    if (skipTo >= tokensLength) return null;
    return tokenList[skipTo];
  }

  if (typeof skip !== "number") {
    for (let i = startIndex; i < tokensLength; i++) {
      const token = tokenList[i];
      if (filter(token)) return token;
    }
  } else {
    for (let i = startIndex; i < tokensLength; i++) {
      const token = tokenList[i];
      if (filter(token)) {
        if (skip <= 0) return token;
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
 * @returns `Token`, or `null` if all were skipped.
 */
export function getTokenOrCommentAfter(
  nodeOrToken: NodeOrToken | Comment,
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
 * @returns Array of `Token`s.
 */
export function getTokensAfter(
  nodeOrToken: NodeOrToken | Comment,
  countOptions?: CountOptions | number | FilterFn | null,
): TokenOrComment[] {
  if (tokens === null) initTokens();
  debugAssertIsNonNull(tokens);
  debugAssertIsNonNull(comments);

  const count =
    typeof countOptions === "number"
      ? countOptions
      : typeof countOptions === "object" && countOptions !== null
        ? countOptions.count
        : null;

  const filter =
    typeof countOptions === "function"
      ? countOptions
      : typeof countOptions === "object" && countOptions !== null
        ? countOptions.filter
        : null;

  const includeComments =
    typeof countOptions === "object" &&
    countOptions !== null &&
    "includeComments" in countOptions &&
    countOptions.includeComments;

  let tokenList: TokenOrComment[];
  if (includeComments) {
    if (tokensAndComments === null) initTokensAndComments();
    debugAssertIsNonNull(tokensAndComments);
    tokenList = tokensAndComments;
  } else {
    tokenList = tokens;
  }

  const rangeEnd = nodeOrToken.range[1];

  let sliceStart = tokenList.length;
  for (let lo = 0; lo < sliceStart; ) {
    const mid = (lo + sliceStart) >> 1;
    if (tokenList[mid].range[0] < rangeEnd) {
      lo = mid + 1;
    } else {
      sliceStart = mid;
    }
  }

  // Fast path for the common case
  if (typeof filter !== "function") {
    if (typeof count !== "number") return tokenList.slice(sliceStart);
    return tokenList.slice(sliceStart, sliceStart + count);
  }

  const tokenListAfter: TokenOrComment[] = [];
  if (typeof count !== "number") {
    for (let i = sliceStart; i < tokenList.length; i++) {
      const token = tokenList[i];
      if (filter(token)) tokenListAfter.push(token);
    }
  } else {
    for (let i = sliceStart; i < tokenList.length && tokenListAfter.length < count; i++) {
      const token = tokenList[i];
      if (filter(token)) tokenListAfter.push(token);
    }
  }
  return tokenListAfter;
}

/**
 * Get all of the tokens between two non-overlapping nodes.
 * @param left - Node or token before the desired token range.
 * @param right - Node or token after the desired token range.
 * @param countOptions? - Options object. If is a function, equivalent to `{ filter: fn }`.
 * @returns Array of `Token`s between `left` and `right`.
 */
/**
 * Get all of the tokens between two non-overlapping nodes.
 * @param left - Node or token before the desired token range.
 * @param right - Node or token after the desired token range.
 * @param padding - Number of extra tokens on either side of center.
 * @returns Array of `Token`s between `left` and `right`.
 */
export function getTokensBetween(
  left: NodeOrToken | Comment,
  right: NodeOrToken | Comment,
  countOptions?: CountOptions | number | FilterFn | null,
): TokenOrComment[] {
  if (tokens === null) initTokens();
  debugAssertIsNonNull(tokens);
  debugAssertIsNonNull(comments);

  const count =
    typeof countOptions === "object" && countOptions !== null ? countOptions.count : null;

  const padding = typeof countOptions === "number" ? countOptions : 0;

  const filter =
    typeof countOptions === "function"
      ? countOptions
      : typeof countOptions === "object" && countOptions !== null
        ? countOptions.filter
        : null;

  const includeComments =
    typeof countOptions === "object" &&
    countOptions !== null &&
    "includeComments" in countOptions &&
    countOptions.includeComments;

  let tokenList: TokenOrComment[];
  if (includeComments) {
    if (tokensAndComments === null) initTokensAndComments();
    debugAssertIsNonNull(tokensAndComments);
    tokenList = tokensAndComments;
  } else {
    tokenList = tokens;
  }

  // This range is not invariant over node order.
  // The first argument must be the left node.
  // Same as ESLint's implementation.
  const rangeStart = left.range[1],
    rangeEnd = right.range[0],
    tokensLength = tokenList.length;

  // Binary search for first token past "between" range start
  let sliceStart = tokensLength;
  for (let lo = 0; lo < sliceStart; ) {
    const mid = (lo + sliceStart) >> 1;
    if (tokenList[mid].range[0] < rangeStart) {
      lo = mid + 1;
    } else {
      sliceStart = mid;
    }
  }

  // Binary search for first token past "between" range end
  let sliceEnd = tokensLength;
  for (let lo = sliceStart; lo < sliceEnd; ) {
    const mid = (lo + sliceEnd) >> 1;
    if (tokenList[mid].range[0] < rangeEnd) {
      lo = mid + 1;
    } else {
      sliceEnd = mid;
    }
  }

  // Apply padding
  sliceStart = max(0, sliceStart - padding);
  sliceEnd += padding;

  if (typeof filter !== "function") {
    if (typeof count !== "number") return tokenList.slice(sliceStart, sliceEnd);
    return tokenList.slice(sliceStart, min(sliceStart + count, sliceEnd));
  }

  const tokensBetween: TokenOrComment[] = [];
  if (typeof count !== "number") {
    for (let i = sliceStart; i < sliceEnd; i++) {
      const token = tokenList[i];
      if (filter(token)) tokensBetween.push(token);
    }
  } else {
    for (let i = sliceStart; i < sliceEnd && tokensBetween.length < count; i++) {
      const token = tokenList[i];
      if (filter(token)) tokensBetween.push(token);
    }
  }
  return tokensBetween;
}

/**
 * Get the first token between two non-overlapping nodes.
 * @param left - Node or token before the desired token range.
 * @param right - Node or token after the desired token range.
 * @param skipOptions? - Options object.
 *   If is a number, equivalent to `{ skip: n }`.
 *   If is a function, equivalent to `{ filter: fn }`.
 * @returns `Token`, or `null` if all were skipped.
 */
export function getFirstTokenBetween(
  left: NodeOrToken | Comment,
  right: NodeOrToken | Comment,
  skipOptions?: SkipOptions | number | FilterFn | null,
): TokenOrComment | null {
  if (tokens === null) initTokens();
  debugAssertIsNonNull(tokens);
  debugAssertIsNonNull(comments);

  // Number of tokens at the beginning of the "between" range to skip
  let skip =
    typeof skipOptions === "number"
      ? skipOptions
      : typeof skipOptions === "object" && skipOptions !== null
        ? skipOptions.skip
        : null;

  const filter =
    typeof skipOptions === "function"
      ? skipOptions
      : typeof skipOptions === "object" && skipOptions !== null
        ? skipOptions.filter
        : null;

  const includeComments =
    typeof skipOptions === "object" &&
    skipOptions !== null &&
    "includeComments" in skipOptions &&
    skipOptions.includeComments;

  let tokenList: TokenOrComment[];
  if (includeComments) {
    if (tokensAndComments === null) initTokensAndComments();
    debugAssertIsNonNull(tokensAndComments);
    tokenList = tokensAndComments;
  } else {
    tokenList = tokens;
  }

  // This range is not invariant over node order.
  // The first argument must be the left node.
  // Same as ESLint's implementation.
  const rangeStart = left.range[1],
    rangeEnd = right.range[0];

  const tokensLength = tokenList.length;

  // Binary search for token immediately following `left`
  let firstTokenIndex = tokensLength;
  for (let lo = 0; lo < firstTokenIndex; ) {
    const mid = (lo + firstTokenIndex) >> 1;
    if (tokenList[mid].range[0] < rangeStart) {
      lo = mid + 1;
    } else {
      firstTokenIndex = mid;
    }
  }

  if (typeof filter !== "function") {
    const skipTo = firstTokenIndex + (skip ?? 0);
    // Avoid indexing out of bounds
    if (skipTo >= tokensLength) return null;
    const token = tokenList[skipTo];
    if (token.range[0] >= rangeEnd) return null;
    return token;
  }

  if (typeof skip !== "number") {
    for (let i = firstTokenIndex; i < tokensLength; i++) {
      const token = tokenList[i];
      if (token.range[0] >= rangeEnd) return null;
      if (filter(token)) return token;
    }
  } else {
    for (let i = firstTokenIndex; i < tokensLength; i++) {
      const token = tokenList[i];
      if (token.range[0] >= rangeEnd) return null;
      if (filter(token)) {
        if (skip <= 0) return token;
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
 * @returns Array of `Token`s between `left` and `right`.
 */
export function getFirstTokensBetween(
  left: NodeOrToken | Comment,
  right: NodeOrToken | Comment,
  countOptions?: CountOptions | number | FilterFn | null,
): TokenOrComment[] {
  if (tokens === null) initTokens();
  debugAssertIsNonNull(tokens);
  debugAssertIsNonNull(comments);

  const count =
    typeof countOptions === "number"
      ? countOptions
      : typeof countOptions === "object" && countOptions !== null
        ? countOptions.count
        : null;

  const filter =
    typeof countOptions === "function"
      ? countOptions
      : typeof countOptions === "object" && countOptions !== null
        ? countOptions.filter
        : null;

  const includeComments =
    typeof countOptions === "object" &&
    countOptions !== null &&
    "includeComments" in countOptions &&
    countOptions.includeComments;

  let tokenList: TokenOrComment[];
  if (includeComments) {
    if (tokensAndComments === null) initTokensAndComments();
    debugAssertIsNonNull(tokensAndComments);
    tokenList = tokensAndComments;
  } else {
    tokenList = tokens;
  }

  // This range is not invariant over node order.
  // The first argument must be the left node.
  // Same as ESLint's implementation.
  const rangeStart = left.range[1],
    rangeEnd = right.range[0];

  const tokensLength = tokenList.length;

  // Find the first token after `left`
  let sliceStart = tokensLength;
  for (let lo = 0; lo < sliceStart; ) {
    const mid = (lo + sliceStart) >> 1;
    if (tokenList[mid].range[0] < rangeStart) {
      lo = mid + 1;
    } else {
      sliceStart = mid;
    }
  }

  // Find the first token at or after `right`
  let sliceEnd = tokensLength;
  for (let lo = sliceStart; lo < sliceEnd; ) {
    const mid = (lo + sliceEnd) >> 1;
    if (tokenList[mid].range[0] < rangeEnd) {
      lo = mid + 1;
    } else {
      sliceEnd = mid;
    }
  }

  if (typeof filter !== "function") {
    if (typeof count !== "number") return tokenList.slice(sliceStart, sliceEnd);
    return tokenList.slice(sliceStart, min(sliceStart + count, sliceEnd));
  }

  const firstTokens: TokenOrComment[] = [];
  if (typeof count !== "number") {
    for (let i = sliceStart; i < sliceEnd; i++) {
      const token = tokenList[i];
      if (filter(token)) firstTokens.push(token);
    }
  } else {
    for (let i = sliceStart; i < sliceEnd && firstTokens.length < count; i++) {
      const token = tokenList[i];
      if (filter(token)) firstTokens.push(token);
    }
  }
  return firstTokens;
}

/**
 * Get the last token between two non-overlapping nodes.
 * @param left - Node or token before the desired token range.
 * @param right - Node or token after the desired token range.
 * @param skipOptions? - Options object.
 *   If is a number, equivalent to `{ skip: n }`.
 *   If is a function, equivalent to `{ filter: fn }`.
 * @returns `Token`, or `null` if all were skipped.
 */
export function getLastTokenBetween(
  left: NodeOrToken | Comment,
  right: NodeOrToken | Comment,
  skipOptions?: SkipOptions | number | FilterFn | null,
): TokenOrComment | null {
  if (tokens === null) initTokens();
  debugAssertIsNonNull(tokens);
  debugAssertIsNonNull(comments);

  // Number of tokens at the end of the "between" range to skip
  let skip =
    typeof skipOptions === "number"
      ? skipOptions
      : typeof skipOptions === "object" && skipOptions !== null
        ? skipOptions.skip
        : null;

  const filter =
    typeof skipOptions === "function"
      ? skipOptions
      : typeof skipOptions === "object" && skipOptions !== null
        ? skipOptions.filter
        : null;

  const includeComments =
    typeof skipOptions === "object" &&
    skipOptions !== null &&
    "includeComments" in skipOptions &&
    skipOptions.includeComments;

  let tokenList: TokenOrComment[];
  if (includeComments) {
    if (tokensAndComments === null) initTokensAndComments();
    debugAssertIsNonNull(tokensAndComments);
    tokenList = tokensAndComments;
  } else {
    tokenList = tokens;
  }

  // This range is not invariant over node order.
  // The first argument must be the left node.
  // Same as ESLint's implementation.
  const rangeStart = left.range[1],
    rangeEnd = right.range[0];

  // Binary search for token immediately preceding `right`
  // The found token may be within the left node if there are no tokens between the nodes.
  let lastTokenIndex = -1;
  for (let lo = 0, hi = tokenList.length - 1; lo <= hi; ) {
    const mid = (lo + hi) >> 1;
    if (tokenList[mid].range[0] < rangeEnd) {
      lastTokenIndex = mid;
      lo = mid + 1;
    } else {
      hi = mid - 1;
    }
  }

  // Fast path for the common case
  if (typeof filter !== "function") {
    const skipTo = lastTokenIndex - (skip ?? 0);
    // Avoid indexing out of bounds
    if (skipTo < 0) return null;
    const token = tokenList[skipTo];
    if (token.range[0] < rangeStart) return null;
    return token;
  }

  if (typeof skip !== "number") {
    for (let i = lastTokenIndex; i >= 0; i--) {
      const token = tokenList[i];
      if (token.range[0] < rangeStart) return null;
      if (filter(token)) return token;
    }
  } else {
    for (let i = lastTokenIndex; i >= 0; i--) {
      const token = tokenList[i];
      if (token.range[0] < rangeStart) return null;
      if (filter(token)) {
        if (skip <= 0) return token;
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
 * @returns Array of `Token`s between `left` and `right`.
 */
export function getLastTokensBetween(
  left: NodeOrToken | Comment,
  right: NodeOrToken | Comment,
  countOptions?: CountOptions | number | FilterFn | null,
): TokenOrComment[] {
  if (tokens === null) initTokens();
  debugAssertIsNonNull(tokens);
  debugAssertIsNonNull(comments);

  const count =
    typeof countOptions === "number"
      ? countOptions
      : typeof countOptions === "object" && countOptions !== null
        ? countOptions.count
        : null;

  const filter =
    typeof countOptions === "function"
      ? countOptions
      : typeof countOptions === "object" && countOptions !== null
        ? countOptions.filter
        : null;

  const includeComments =
    typeof countOptions === "object" &&
    countOptions !== null &&
    "includeComments" in countOptions &&
    countOptions.includeComments;

  let tokenList: TokenOrComment[];
  if (includeComments) {
    if (tokensAndComments === null) initTokensAndComments();
    debugAssertIsNonNull(tokensAndComments);
    tokenList = tokensAndComments;
  } else {
    tokenList = tokens;
  }

  // This range is not invariant over node order.
  // The first argument must be the left node.
  // Same as ESLint's implementation.
  const rangeStart = left.range[1],
    rangeEnd = right.range[0],
    tokensLength = tokenList.length;

  // Binary search for first token past "between" range start
  let sliceStart = tokensLength;
  for (let lo = 0; lo < sliceStart; ) {
    const mid = (lo + sliceStart) >> 1;
    if (tokenList[mid].range[0] < rangeStart) {
      lo = mid + 1;
    } else {
      sliceStart = mid;
    }
  }

  // Binary search for first token past "between" range end
  let sliceEnd = tokensLength;
  for (let lo = sliceStart; lo < sliceEnd; ) {
    const mid = (lo + sliceEnd) >> 1;
    if (tokenList[mid].range[0] < rangeEnd) {
      lo = mid + 1;
    } else {
      sliceEnd = mid;
    }
  }

  // Fast path for the common case
  if (typeof filter !== "function") {
    if (typeof count !== "number") return tokenList.slice(sliceStart, sliceEnd);
    return tokenList.slice(max(sliceStart, sliceEnd - count), sliceEnd);
  }

  const tokensBetween: TokenOrComment[] = [];
  if (typeof count !== "number") {
    for (let i = sliceStart; i < sliceEnd; i++) {
      const token = tokenList[i];
      if (filter(token)) tokensBetween.push(token);
    }
  } else {
    // Count is the number of preceding tokens, so we iterate in reverse
    for (let i = sliceEnd - 1; i >= sliceStart && tokensBetween.length < count; i--) {
      const token = tokenList[i];
      if (filter(token)) tokensBetween.unshift(token);
    }
  }
  return tokensBetween;
}

/**
 * Get the token starting at the specified index.
 * @param index - Index of the start of the token's range.
 * @param rangeOptions - Options object.
 * @returns The token starting at index, or `null` if no such token.
 */
export function getTokenByRangeStart(
  index: number,
  rangeOptions?: RangeOptions | null,
): TokenOrComment | null {
  if (tokens === null) initTokens();
  debugAssertIsNonNull(tokens);
  debugAssertIsNonNull(comments);

  const includeComments =
    typeof rangeOptions === "object" &&
    rangeOptions !== null &&
    "includeComments" in rangeOptions &&
    rangeOptions.includeComments;

  let tokenList: TokenOrComment[];
  if (includeComments) {
    if (tokensAndComments === null) initTokensAndComments();
    debugAssertIsNonNull(tokensAndComments);
    tokenList = tokensAndComments;
  } else {
    tokenList = tokens;
  }

  // Binary search for token starting at the given index
  for (let lo = 0, hi = tokenList.length; lo < hi; ) {
    const mid = (lo + hi) >> 1;
    const tokenStart = tokenList[mid].range[0];
    if (tokenStart === index) {
      return tokenList[mid];
    } else if (tokenStart < index) {
      lo = mid + 1;
    } else {
      hi = mid;
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
  if (tokensAndComments === null) {
    if (tokens === null) initTokens();
    initTokensAndComments();
  }
  debugAssertIsNonNull(tokensAndComments);

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
  // the token will be the first token between the two nodes.
  const tokensAndCommentsLength = tokensAndComments.length;
  let tokenBetweenIndex = tokensAndCommentsLength;
  for (let lo = 0; lo < tokenBetweenIndex; ) {
    const mid = (lo + tokenBetweenIndex) >> 1;
    if (tokensAndComments[mid].range[0] < rangeStart) {
      lo = mid + 1;
    } else {
      tokenBetweenIndex = mid;
    }
  }

  for (
    let lastTokenEnd = rangeStart;
    tokenBetweenIndex < tokensAndCommentsLength;
    tokenBetweenIndex++
  ) {
    const { range } = tokensAndComments[tokenBetweenIndex];
    const tokenStart = range[0];
    // The first token of the later node should undergo the check in the second branch
    if (tokenStart > rangeEnd) break;
    if (tokenStart !== lastTokenEnd) return true;
    lastTokenEnd = range[1];
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
  if (tokensAndComments === null) {
    if (tokens === null) initTokens();
    initTokensAndComments();
  }
  debugAssertIsNonNull(tokensAndComments);

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
  // the token will be the first token between the two nodes.
  const tokensAndCommentsLength = tokensAndComments.length;
  let tokenBetweenIndex = tokensAndCommentsLength;
  for (let lo = 0; lo < tokenBetweenIndex; ) {
    const mid = (lo + tokenBetweenIndex) >> 1;
    if (tokensAndComments[mid].range[0] < rangeStart) {
      lo = mid + 1;
    } else {
      tokenBetweenIndex = mid;
    }
  }

  for (
    let lastTokenEnd = rangeStart;
    tokenBetweenIndex < tokensAndCommentsLength;
    tokenBetweenIndex++
  ) {
    const token = tokensAndComments[tokenBetweenIndex],
      { range } = token,
      tokenStart = range[0];

    // The first token of the later node should undergo the check in the second branch
    if (tokenStart > rangeEnd) break;
    if (
      tokenStart !== lastTokenEnd ||
      (token.type === "JSXText" && JSX_WHITESPACE_REGEXP.test(token.value))
    ) {
      return true;
    }
    lastTokenEnd = range[1];
  }

  return false;
}
