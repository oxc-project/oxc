/*
 * Token types and tokens initialization / reset.
 */

import { buffer, initSourceText, sourceText } from "./source_code.ts";
import { computeLoc } from "./location.ts";
import {
  COMMENT_SIZE,
  DESERIALIZED_FLAG_OFFSET,
  TOKENS_OFFSET_POS_32,
  TOKENS_LEN_POS_32,
} from "../generated/constants.ts";
import { debugAssert, debugAssertIsNonNull } from "../utils/asserts.ts";

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

type Regex = RegularExpressionToken["regex"];

// Tokens for the current file.
// Created lazily only when needed.
export let tokens: TokenType[] | null = null;

// Typed array views over the tokens region of the buffer.
// These persist for the lifetime of the file (cleared in `resetTokens`).
let tokensUint8: Uint8Array | null = null;
export let tokensUint32: Uint32Array | null = null;

// Number of tokens for the current file.
export let tokensLen = 0;

// Whether all tokens have been deserialized into `cachedTokens`.
export let allTokensDeserialized = false;

// Cached token objects, reused across files to reduce GC pressure.
// Tokens are mutated in place during deserialization, then `tokens` is set to a slice of this array.
export const cachedTokens: Token[] = [];

// Tokens array from previous file.
// Reused for next file if next file has less tokens than the previous file (by truncating it to correct length).
let previousTokens: Token[] = [];

// Tokens whose `loc` property has been accessed, and therefore needs clearing on reset
const tokensWithLoc: Token[] = [];

// Cached regex descriptor objects, reused across files
const regexObjects: Regex[] = [];

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
  regex: Regex | undefined;
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
export const TOKEN_SIZE = 16;
debugAssert(TOKEN_SIZE === COMMENT_SIZE, "Size of token, comment, and merged entry must be equal");

const TOKEN_SIZE_SHIFT = 4;
debugAssert(TOKEN_SIZE === 1 << TOKEN_SIZE_SHIFT);

const KIND_FIELD_OFFSET = 8;
const IS_ESCAPED_FIELD_OFFSET = 10;

// Values for the "deserialized" flag byte in buffer.
// * `FLAG_DESERIALIZED` indicates the token/comment is already deserialized.
// * `FLAG_NOT_DESERIALIZED` indicates the token/comment is not yet deserialized.
//   `Token` / `Comment` object may be uninitialized, or contain stale data.
export const FLAG_NOT_DESERIALIZED = 0;
export const FLAG_DESERIALIZED = 1;

/**
 * Deserialize all tokens and build the `tokens` array.
 * Called by `ast.tokens` getter.
 */
export function initTokens(): void {
  debugAssert(tokens === null, "Tokens already initialized");

  if (!allTokensDeserialized) deserializeTokens();

  // Create `tokens` array as a slice of `cachedTokens` array.
  //
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
}

/**
 * Deserialize all tokens into `cachedTokens`.
 * Does NOT build the `tokens` array - use `initTokens` for that.
 */
export function deserializeTokens(): void {
  debugAssert(!allTokensDeserialized, "Tokens already deserialized");

  if (tokensUint32 === null) initTokensBuffer();

  for (let i = 0; i < tokensLen; i++) {
    deserializeTokenIfNeeded(i);
  }

  allTokensDeserialized = true;

  debugCheckDeserializedTokens();
}

/**
 * Initialize typed array views over the tokens region of the buffer.
 *
 * Populates `tokensUint8`, `tokensUint32`, and `tokensLen`, and grows `cachedTokens` if needed.
 * Does NOT deserialize tokens - they are deserialized lazily via `deserializeTokenIfNeeded`.
 */
export function initTokensBuffer(): void {
  debugAssert(tokensUint8 === null && tokensUint32 === null, "Tokens buffer already initialized");

  debugAssertIsNonNull(buffer);

  // Various tokens methods rely on `sourceText` being initialized after `initTokensBuffer`,
  // so we always initialize it here, even if there are no tokens (empty file)
  if (sourceText === null) initSourceText();
  debugAssertIsNonNull(sourceText);

  const { uint32 } = buffer;
  const tokensPos = uint32[TOKENS_OFFSET_POS_32];
  tokensLen = uint32[TOKENS_LEN_POS_32];

  // Create typed array views over just the tokens region of the buffer.
  // These are zero-copy views over the same underlying `ArrayBuffer`.
  // Views persist for the lifetime of the file (cleared in `resetTokens`).
  const arrayBuffer = buffer.buffer,
    absolutePos = buffer.byteOffset + tokensPos;
  tokensUint8 = new Uint8Array(arrayBuffer, absolutePos, tokensLen << TOKEN_SIZE_SHIFT);
  tokensUint32 = new Uint32Array(arrayBuffer, absolutePos, tokensLen << (TOKEN_SIZE_SHIFT - 2));

  // Grow cache if needed (one-time cost as cache warms up)
  while (cachedTokens.length < tokensLen) {
    cachedTokens.push(new Token());
  }

  // Check buffer data has valid ranges and ascending order
  debugCheckValidRanges();
}

/**
 * Get token at `index`, deserializing if needed.
 *
 * Caller must ensure `initTokensBuffer()` has been called before calling this function.
 *
 * @param index - Token index in the tokens buffer
 * @returns Deserialized token
 */
export function getToken(index: number): TokenType {
  const token = deserializeTokenIfNeeded(index);
  return (token === null ? cachedTokens[index] : token) as TokenType;
}

/**
 * Deserialize token at `index` if not already deserialized.
 *
 * Caller must ensure `initTokensBuffer()` has been called before calling this function.
 *
 * @param index - Token index in the tokens buffer
 * @returns `Token` object if newly deserialized, or `null` if already deserialized
 */
export function deserializeTokenIfNeeded(index: number): Token | null {
  const pos = index << TOKEN_SIZE_SHIFT;

  // Fast path: If already deserialized, exit
  const flagPos = pos + DESERIALIZED_FLAG_OFFSET;
  if (tokensUint8![flagPos] !== FLAG_NOT_DESERIALIZED) return null;

  // Mark token as deserialized, so it won't be deserialized again
  tokensUint8![flagPos] = FLAG_DESERIALIZED;

  // Deserialize token into a cached `Token` object
  const token = cachedTokens[index];

  const kind = tokensUint8![pos + KIND_FIELD_OFFSET];

  const pos32 = pos >> 2,
    start = tokensUint32![pos32],
    end = tokensUint32![pos32 + 1];

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
    let regex: Regex;
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

  return token;
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
 * Check tokens buffer has valid ranges and ascending order.
 *
 * Only runs in debug build (tests). In release build, this function is entirely removed by minifier.
 */
function debugCheckValidRanges(): void {
  if (!DEBUG) return;

  let lastEnd = 0;
  for (let i = 0; i < tokensLen; i++) {
    const pos32 = i << 2;
    const start = tokensUint32![pos32];
    const end = tokensUint32![pos32 + 1];
    if (end <= start) throw new Error(`Invalid token range: ${start}-${end}`);
    if (start < lastEnd) {
      throw new Error(`Overlapping tokens: last end: ${lastEnd}, next start: ${start}`);
    }
    lastEnd = end;
  }
}

/**
 * Check all deserialized tokens are in ascending order.
 *
 * Only runs in debug build (tests). In release build, this function is entirely removed by minifier.
 */
function debugCheckDeserializedTokens(): void {
  if (!DEBUG) return;

  let lastEnd = 0;
  for (let i = 0; i < tokensLen; i++) {
    const flagPos = (i << TOKEN_SIZE_SHIFT) + DESERIALIZED_FLAG_OFFSET;
    if (tokensUint8![flagPos] !== FLAG_DESERIALIZED) {
      throw new Error(`Token ${i} not marked as deserialized after \`deserializeTokens()\` call`);
    }

    const { start, end } = cachedTokens[i];
    if (end <= start) throw new Error(`Invalid deserialized token range: ${start}-${end}`);
    if (start < lastEnd) {
      throw new Error(
        `Deserialized tokens not in order: last end: ${lastEnd}, next start: ${start}`,
      );
    }
    lastEnd = end;
  }
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
  tokensUint8 = null;
  tokensUint32 = null;
  tokensLen = 0;
  allTokensDeserialized = false;
}
