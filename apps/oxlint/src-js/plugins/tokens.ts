/*
 * Token types and tokens initialization / reset.
 */

import { ast, buffer, initAst, initSourceText, sourceText } from "./source_code.ts";
import { computeLoc } from "./location.ts";
import { TOKENS_OFFSET_POS_32, TOKENS_LEN_POS_32 } from "../generated/constants.ts";
import { debugAssert, debugAssertIsNonNull } from "../utils/asserts.ts";

import type { Comment } from "./types.ts";
import type { Location, Span } from "./location.ts";

/**
 * AST token type.
 */
type TokenType =
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

// Export type as `Token` for external consumers
export type { TokenType as Token };

interface BaseToken extends Span {
  value: string;
  regex: undefined;
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

export interface RegularExpressionToken extends Span {
  type: "RegularExpression";
  value: string;
  regex: {
    pattern: string;
    flags: string;
  };
}

export interface StringToken extends BaseToken {
  type: "String";
}

export interface TemplateToken extends BaseToken {
  type: "Template";
}

export type TokenOrComment = TokenType | Comment;

// Tokens for the current file.
// Created lazily only when needed.
export let tokens: TokenType[] | null = null;
let comments: Comment[] | null = null;
export let tokensAndComments: TokenOrComment[] | null = null;

// Cached token objects, reused across files to reduce GC pressure.
// Tokens are mutated in place during deserialization, then `tokens` is set to a slice of this array.
const cachedTokens: Token[] = [];

// Tokens array from previous file.
// Reused for next file if next file has less tokens than the previous file (by truncating it to correct length).
let previousTokens: Token[] = [];

// Tokens whose `loc` property has been accessed, and therefore needs clearing on reset
const tokensWithLoc: Token[] = [];

// Cached regex descriptor objects, reused across files
const regexObjects: RegularExpressionToken["regex"][] = [];

// Tokens whose `regex` property was set, and therefore needs clearing on reset.
// Regex tokens are rare, so this array is almost always very small.
// `tokensWithRegex.length` also serves as the index into `regexObjects`
// for the next regex descriptor object which can be reused.
const tokensWithRegex: Token[] = [];

// Reset `#loc` field on a `Token` class instance
let resetLoc: (token: Token) => void;

/**
 * Token implementation with lazy `loc` caching via private field.
 *
 * Using a class with a private `#loc` field avoids hidden class transitions that would occur
 * with `Object.defineProperty` / `delete` on plain objects.
 * All `Token` instances always have the same V8 hidden class, keeping property access monomorphic.
 */
class Token {
  type: TokenType["type"] = null!; // Overwritten later
  value: string = null!; // Overwritten later
  regex: RegularExpressionToken["regex"] | undefined;
  start: number = 0;
  end: number = 0;
  range: [number, number] = [0, 0];

  #loc: Location | null = null;

  get loc(): Location {
    const loc = this.#loc;
    if (loc !== null) return loc;

    tokensWithLoc.push(this);
    return (this.#loc = computeLoc(this.start, this.end));
  }

  static {
    // Defined in static block to avoid exposing this as a public method
    resetLoc = (token: Token) => {
      token.#loc = null;
    };
  }
}

// Make `loc` property enumerable so that `for (const key in token) ...` includes `loc` in the keys it iterates over
Object.defineProperty(Token.prototype, "loc", { enumerable: true });

// Typed array views over the tokens region of the buffer
let tokensUint8: Uint8Array | null = null;
let tokensUint32: Uint32Array | null = null;

// `ESTreeKind` discriminants (set by Rust side)
const PRIVATE_IDENTIFIER_KIND = 2;
const REGEXP_KIND = 8;

// Indexed by `ESTreeKind` discriminant (matches `ESTreeKind` enum in `estree_kind.rs`)
const TOKEN_TYPES: TokenType["type"][] = [
  "Identifier",
  "Keyword",
  "PrivateIdentifier",
  "Punctuator",
  "Numeric",
  "String",
  "Boolean",
  "Null",
  "RegularExpression",
  "Template",
  "JSXText",
  "JSXIdentifier",
];

// Details of Rust `Token` type
const TOKEN_SIZE_SHIFT = 4; // 1 << 4 == 16 bytes, the size of `Token` in Rust
const KIND_FIELD_OFFSET = 8;
const IS_ESCAPED_FIELD_OFFSET = 10;

/**
 * Initialize tokens for current file.
 */
export function initTokens() {
  debugAssert(tokens === null, "Tokens already initialized");

  // Deserialize tokens from buffer
  if (sourceText === null) initSourceText();
  debugAssertIsNonNull(sourceText);

  debugAssertIsNonNull(buffer);

  const { uint32 } = buffer;
  const tokensPos = uint32[TOKENS_OFFSET_POS_32];
  const tokensLen = uint32[TOKENS_LEN_POS_32];

  // Create typed array views over just the tokens region of the buffer.
  // These are zero-copy views over the same underlying `ArrayBuffer`.
  const arrayBuffer = buffer.buffer,
    absolutePos = buffer.byteOffset + tokensPos;
  tokensUint8 = new Uint8Array(arrayBuffer, absolutePos, tokensLen << TOKEN_SIZE_SHIFT);
  tokensUint32 = new Uint32Array(arrayBuffer, absolutePos, tokensLen << (TOKEN_SIZE_SHIFT - 2));

  // Grow cache if needed (one-time cost as cache warms up)
  while (cachedTokens.length < tokensLen) {
    cachedTokens.push(new Token());
  }

  // Deserialize into cached token objects
  for (let i = 0; i < tokensLen; i++) {
    deserializeTokenInto(cachedTokens[i], i);
  }

  tokensUint8 = null;
  tokensUint32 = null;

  // Use `slice` rather than copying tokens one-by-one into a new array.
  // V8 implements `slice` with a single `memcpy` of the backing store, which is faster
  // than N individual `push` calls with bounds checking and potential resizing.
  //
  // If the tokens array from previous file is longer than the current one,
  // reuse it and truncate it to avoid the memcpy entirely.
  // Assuming random distribution of file sizes, this cheaper branch should be hit on 50% of files.
  if (previousTokens.length >= tokensLen) {
    previousTokens.length = tokensLen;
    tokens = previousTokens as TokenType[];
  } else {
    tokens = (previousTokens = cachedTokens.slice(0, tokensLen)) as TokenType[];
  }

  // Check `tokens` have valid ranges and are in ascending order
  debugCheckValidRanges(tokens, "token");
}

/**
 * Deserialize token `i` from buffer into an existing token object.
 * @param token - Token object to mutate
 * @param index - Token index
 */
function deserializeTokenInto(token: Token, index: number): void {
  const pos32 = index << 2;
  const start = tokensUint32![pos32],
    end = tokensUint32![pos32 + 1];

  const pos = pos32 << (TOKEN_SIZE_SHIFT - 2);
  const kind = tokensUint8![pos + KIND_FIELD_OFFSET];

  // Get `value` as slice of source text `start..end`.
  // Slice `start + 1..end` for private identifiers, to strip leading `#`.
  let value = sourceText!.slice(start + +(kind === PRIVATE_IDENTIFIER_KIND), end);

  if (kind <= PRIVATE_IDENTIFIER_KIND) {
    // Unescape if `escaped` flag is set
    if (tokensUint8![pos + IS_ESCAPED_FIELD_OFFSET] === 1) {
      value = unescapeIdentifier(value);
    }
  } else if (kind === REGEXP_KIND) {
    // Reuse cached regex descriptor object if available, otherwise create a new one.
    // The array access is inside the `regexObjects.length > regexIndex` branch so V8 can elide the bounds check.
    let regex: RegularExpressionToken["regex"];
    const regexIndex = tokensWithRegex.length;
    if (regexObjects.length > regexIndex) {
      regex = regexObjects[regexIndex];
    } else {
      regexObjects.push((regex = { pattern: "", flags: "" }));
    }
    token.regex = regex;

    const patternEnd = value.lastIndexOf("/");
    regex.pattern = value.slice(1, patternEnd);
    regex.flags = value.slice(patternEnd + 1);

    tokensWithRegex.push(token);
  }

  token.type = TOKEN_TYPES[kind];
  token.value = value;
  token.range[0] = token.start = start;
  token.range[1] = token.end = end;
}

/**
 * Unescape an identifier.
 *
 * We do this on JS side, because escaped identifiers are so extremely rare that this function
 * is never called in practice anyway.
 *
 * @param {string} name - Identifier name to unescape
 * @returns {string} - Unescaped identifier name
 */
function unescapeIdentifier(name: string): string {
  return name.replace(/\\u(?:\{([0-9a-fA-F]+)\}|([0-9a-fA-F]{4}))/g, (_, hex1, hex2) =>
    String.fromCodePoint(parseInt(hex1 ?? hex2, 16)),
  );
}

/**
 * Check `tokens` have valid ranges and are in ascending order.
 *
 * Only runs in debug build (tests). In release build, this function is entirely removed by minifier.
 */
function debugCheckValidRanges(tokens: TokenOrComment[], description: string): void {
  if (!DEBUG) return;

  let lastEnd = 0;
  for (const token of tokens) {
    const { start, end } = token;
    if (end <= start) throw new Error(`Invalid ${description} range: ${start}-${end}`);
    if (start < lastEnd) {
      throw new Error(`Overlapping ${description}s: last end: ${lastEnd}, next start: ${start}`);
    }
    lastEnd = end;
  }
}

/**
 * Initialize `tokensAndComments`.
 *
 * Caller must ensure `tokens` is initialized before calling this function,
 * by calling `initTokens()` if `tokens === null`.
 */
export function initTokensAndComments() {
  debugAssertIsNonNull(tokens);

  // Get comments from AST
  if (comments === null) {
    if (ast === null) initAst();
    debugAssertIsNonNull(ast);
    comments = ast.comments;

    debugCheckValidRanges(comments, "comment");
  }

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
    tokenStart = token.start,
    commentStart = comment.start;

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
    commentStart = comment.start;
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
      tokenStart = token.start;
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
      commentStart = comment.start;
    } while (commentStart < tokenStart);
  }

  debugAssert(false, "End of `initTokensAndComments` should be unreachable");
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
  expected.sort((a, b) => a.start - b.start);

  if (tokensAndComments.length !== expected.length) {
    throw new Error("`tokensAndComments` has wrong length");
  }

  for (let i = 0; i < tokensAndComments.length; i++) {
    if (tokensAndComments[i] !== expected[i]) {
      throw new Error("`tokensAndComments` is not correctly ordered");
    }
  }

  debugCheckValidRanges(tokensAndComments, "token/comment");
}

/**
 * Reset tokens after file has been linted.
 *
 * Clears cached `loc` on tokens that had it accessed, so the getter
 * will recalculate it when the token is reused for a different file.
 */
export function resetTokens() {
  for (let i = 0, len = tokensWithLoc.length; i < len; i++) {
    resetLoc(tokensWithLoc[i]);
  }
  tokensWithLoc.length = 0;

  for (let i = 0, len = tokensWithRegex.length; i < len; i++) {
    tokensWithRegex[i].regex = undefined;
  }
  tokensWithRegex.length = 0;

  tokens = null;
  comments = null;
  tokensAndComments = null;
}
