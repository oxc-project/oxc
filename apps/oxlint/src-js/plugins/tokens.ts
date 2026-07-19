/*
 * Token types and tokens initialization / reset.
 */

import { buffer, initSourceText, sourceText } from "./source_code.ts";
import { computeLoc, createRange } from "./location.ts";
import { COMMENT_SIZE, TOKENS_OFFSET_POS_32, TOKENS_LEN_POS_32 } from "../generated/constants.ts";
import { debugAssert, debugAssertIsNonNull } from "../utils/asserts.ts";

import type { Location, Range, Span } from "./location.ts";

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

// Typed array view over the tokens region of the buffer.
// Persists for the lifetime of the file (cleared in `resetTokens`).
export let tokensInt32: Int32Array | null = null;

// Number of tokens for the current file.
export let tokensLen = 0;

// Cached token objects, reused across files to reduce GC pressure.
// Tokens are mutated in place during deserialization, then `tokens` is set to a slice of this array.
export const cachedTokens: Token[] = [];

// Tokens array from previous file.
// Reused for next file if next file has less tokens than the previous file (by truncating it to correct length).
let previousTokens: Token[] = [];

// Tokens whose `range` property has been accessed, and therefore needs clearing on reset.
// Never shrunk - `activeTokensWithRangeCount` tracks the active count to avoid freeing the backing store.
const tokensWithRange: Token[] = [];
let activeTokensWithRangeCount = 0;

// Tokens whose `loc` property has been accessed, and therefore needs clearing on reset.
// Never shrunk - `activeTokensWithLocCount` tracks the active count to avoid freeing the backing store.
const tokensWithLoc: Token[] = [];
let activeTokensWithLocCount = 0;

// Cached `Regex` objects, reused across files.
// `activeTokensWithRegexCount` serves as the index into both `regexObjects` (for the next reusable object)
// and `tokensWithRegex` (below), which grow in lockstep and so always have the same length.
const regexObjects: Regex[] = [];

// Tokens whose `regex` property has been accessed, and therefore needs clearing on reset.
// Regex tokens are rare, so this array is almost always very small.
// Never shrunk - `activeTokensWithRegexCount` tracks the active count to avoid freeing the backing store.
const tokensWithRegex: Token[] = [];
let activeTokensWithRegexCount = 0;

// `defineGetter(obj, prop, getter)` is equivalent to `obj.__defineGetter__(prop, getter)`,
// but without `Object.prototype` lookup at each call site
const defineGetter = Function.prototype.call.bind(
  // @ts-expect-error - `__defineGetter__` is not in `Object.prototype`'s type definition,
  // but it does exist at runtime and is widely supported in JS engines, including V8
  Object.prototype.__defineGetter__,
) as (obj: object, prop: string, getter: () => unknown) => void;

// Getters for `type`, `start`, `end`, `range`, `loc`, `value`, and `regex` properties on a `Token` class instance.
// Copied into `const`s below after being defined in class static block.
let getTokenTypeTemp: (this: Token) => TokenType["type"];
let getTokenStartTemp: (this: Token) => number;
let getTokenEndTemp: (this: Token) => number;
let getTokenRangeTemp: (this: Token) => Range;
let getTokenLocTemp: (this: Token) => Location;
let getTokenValueTemp: (this: Token) => string;
let getTokenRegexTemp: (this: Token) => Regex | undefined;

// Reset `#range` field on a `Token` class instance.
// Copied into a `const` below after being defined in class static block.
let resetRangeTemp: (token: Token) => void;

// Reset `#loc` field on a `Token` class instance.
// Copied into a `const` below after being defined in class static block.
let resetLocTemp: (token: Token) => void;

// Reset `#regex` field on a `Token` class instance.
// Copied into a `const` below after being defined in class static block.
let resetRegexTemp: (token: Token) => void;

// Get `#range` field on a `Token` class instance.
// Only used in debug build (tests).
let getTokenPrivateRange: (token: Token) => Range | null;

// Get `#loc` field on a `Token` class instance.
// Only used in debug build (tests).
let getTokenPrivateLoc: (token: Token) => Location | null;

// Get `#regex` field on a `Token` class instance.
// Only used in debug build (tests).
let getTokenPrivateRegex: (token: Token) => Regex | null;

/**
 * Token implementation with lazy `range`, `loc`, and `regex` caching via private fields.
 *
 * `range`, `loc`, and `regex` are defined as own accessor properties via `__defineGetter__` in the constructor,
 * using shared getter functions (`getTokenRange` / `getTokenLoc` / `getTokenRegex`). This makes them own
 * enumerable properties, so `{...token}` spreads them and `JSON.stringify(token)` serializes them.
 *
 * The computed `range` array, `Location` value, and `Regex` objects are cached in the private
 * `#range` / `#loc` / `#regex` fields on first access, so accessing any of them twice returns the same object.
 *
 * All instances share the same getter functions, keeping the V8 hidden class transition identical across instances.
 * Reset only clears the `#range`, `#loc`, and `#regex` fields.
 */
class Token {
  // All defined with `__defineGetter__` in constructor
  declare type: TokenType["type"];
  declare start: number;
  declare end: number;
  declare range: Range;
  declare loc: Location;
  declare value: string;
  declare regex: Regex | undefined;

  // `#pos32` is the index of the token's first `i32` in `tokensInt32` (a word index, not a byte offset).
  // Initialized to `0` so V8 keeps it as an SMI. Constructor overwrites it with the real position.
  #pos32: number = 0;
  #range: Range | null = null;
  #loc: Location | null = null;
  #regex: Regex | null = null;

  constructor(pos32: number) {
    this.#pos32 = pos32;

    // Define all properties as own getter properties (enumerable + configurable by default).
    // This makes `{...token}` spread them, and `JSON.stringify(token)` serialize them.
    // Note: `new Token()` is 25% faster with `__defineGetter__` vs `Object.defineProperty`.
    // See https://github.com/oxc-project/oxc/pull/22238.
    defineGetter(this, "type", getTokenType);
    defineGetter(this, "start", getTokenStart);
    defineGetter(this, "end", getTokenEnd);
    defineGetter(this, "range", getTokenRange);
    defineGetter(this, "loc", getTokenLoc);
    defineGetter(this, "value", getTokenValue);
    defineGetter(this, "regex", getTokenRegex);
  }

  // Functions requiring access to private props defined in static block to avoid exposing them as public methods
  static {
    getTokenTypeTemp = function (this: Token): TokenType["type"] {
      // This assert can fail in real-world plugin code, and is not a bug here, only incorrect usage in plugin.
      // Only make this an explicit error in debug build, because it should be very uncommon.
      // In release build, it will result in an error in the `tokensInt32` access below.
      debugAssertIsNonNull(
        tokensInt32,
        "`Token` object's `type` field accessed after file finished linting",
      );

      return TOKEN_TYPES[tokensInt32[this.#pos32 + KIND_FLAGS_OFFSET32] & TOKEN_KIND_MASK];
    };

    getTokenStartTemp = function (this: Token): number {
      // This assert can fail in real-world plugin code, and is not a bug here, only incorrect usage in plugin.
      // Only make this an explicit error in debug build, because it should be very uncommon.
      // In release build, it will result in an error in the `tokensInt32` access below.
      debugAssertIsNonNull(
        tokensInt32,
        "`Token` object's `start` field accessed after file finished linting",
      );

      return tokensInt32[this.#pos32];
    };

    getTokenEndTemp = function (this: Token): number {
      // This assert can fail in real-world plugin code, and is not a bug here, only incorrect usage in plugin.
      // Only make this an explicit error in debug build, because it should be very uncommon.
      // In release build, it will result in an error in the `tokensInt32` access below.
      debugAssertIsNonNull(
        tokensInt32,
        "`Token` object's `end` field accessed after file finished linting",
      );

      return tokensInt32[this.#pos32 + 1];
    };

    getTokenRangeTemp = function (this: Token): Range {
      // This assert can fail in real-world plugin code, and is not a bug here, only incorrect usage in plugin.
      // Only make this an explicit error in debug build, because it should be very uncommon.
      // In release build, it will result in an error in the `tokensInt32` access below.
      debugAssertIsNonNull(
        tokensInt32,
        "`Token` object's `range` field accessed after file finished linting",
      );

      const range = this.#range;
      if (range !== null) return range;

      // Store token in `tokensWithRange` array. `resetTokens` will clear the `#range` property.
      // Note: The comparison `activeTokensWithRangeCount < tokensWithRange.length` must be this way around
      // so that V8 can remove the bounds check on `tokensWithRange[activeTokensWithRangeCount]`.
      // `tokensWithRange.length > activeTokensWithRangeCount` would *not* remove the bounds check in Maglev compiler.
      if (activeTokensWithRangeCount < tokensWithRange.length) {
        tokensWithRange[activeTokensWithRangeCount] = this;
      } else {
        tokensWithRange.push(this);
      }
      activeTokensWithRangeCount++;

      const pos32 = this.#pos32;
      return (this.#range = createRange(tokensInt32[pos32], tokensInt32[pos32 + 1]));
    };

    getTokenLocTemp = function (this: Token): Location {
      // This assert can fail in real-world plugin code, and is not a bug here, only incorrect usage in plugin.
      // Only make this an explicit error in debug build, because it should be very uncommon.
      // In release build, it will result in an error in the `tokensInt32` access below.
      debugAssertIsNonNull(
        tokensInt32,
        "`Token` object's `loc` field accessed after file finished linting",
      );

      const loc = this.#loc;
      if (loc !== null) return loc;

      // Store token in `tokensWithLoc` array. `resetTokens` will clear the `#loc` property.
      // Note: The comparison `activeTokensWithLocCount < tokensWithLoc.length` must be this way around
      // so that V8 can remove the bounds check on `tokensWithLoc[activeTokensWithLocCount]`.
      // `tokensWithLoc.length > activeTokensWithLocCount` would *not* remove the bounds check in Maglev compiler.
      if (activeTokensWithLocCount < tokensWithLoc.length) {
        tokensWithLoc[activeTokensWithLocCount] = this;
      } else {
        tokensWithLoc.push(this);
      }
      activeTokensWithLocCount++;

      const pos32 = this.#pos32,
        start = tokensInt32[pos32],
        end = tokensInt32[pos32 + 1];
      return (this.#loc = computeLoc(start, end));
    };

    getTokenValueTemp = function (this: Token): string {
      // This assert can fail in real-world plugin code, and is not a bug here, only incorrect usage in plugin.
      // Only make this an explicit error in debug build, because it should be very uncommon.
      // In release build, it will result in an error in the `tokensInt32` or `sourceText` accesses below.
      debugAssert(
        tokensInt32 !== null && sourceText !== null,
        "`Token` object's `value` field accessed after file finished linting",
      );

      const pos32 = this.#pos32;

      // `kind` (byte 8) and `escaped` (byte 10) share this word - read it once, extract each with a mask.
      const kindAndFlags = tokensInt32[pos32 + KIND_FLAGS_OFFSET32];
      const kind = kindAndFlags & TOKEN_KIND_MASK;

      // Get `value` as slice of source text `start..end`.
      // Slice `start + 1..end` for private identifiers, to strip leading `#`.
      const start = tokensInt32[pos32],
        end = tokensInt32[pos32 + 1];
      let value = sourceText.slice(start + +(kind === PRIVATE_IDENTIFIER_KIND), end);

      // Unescape if `escaped` flag is set
      if (kind <= PRIVATE_IDENTIFIER_KIND && (kindAndFlags & IS_ESCAPED_MASK) !== 0) {
        value = unescapeIdentifier(value);
      }

      return value;
    };

    getTokenRegexTemp = function (this: Token): Regex | undefined {
      // This assert can fail in real-world plugin code, and is not a bug here, only incorrect usage in plugin.
      // Only make this an explicit error in debug build, because it should be very uncommon.
      // In release build, it will result in an error in the `tokensInt32` or `sourceText` accesses below.
      debugAssert(
        tokensInt32 !== null && sourceText !== null,
        "`Token` object's `regex` field accessed after file finished linting",
      );

      // Only `RegularExpression` tokens have `regex` defined. All other tokens have `regex: undefined`.
      const pos32 = this.#pos32;
      if ((tokensInt32[pos32 + KIND_FLAGS_OFFSET32] & TOKEN_KIND_MASK) !== REGEXP_KIND) {
        return undefined;
      }

      const regex = this.#regex;
      if (regex !== null) return regex;

      // First access. Reuse a pooled `Regex` object if available, otherwise create a new one,
      // and store this token in `tokensWithRegex` so `resetTokens` can clear its `#regex`
      // (and the `Regex` object's fields).
      // `regexObjects` and `tokensWithRegex` are the same length, so a single length check covers both.
      // Note: The comparison `activeTokensWithRegexCount < regexObjects.length` must be this way around
      // so that V8 can remove the bounds checks on the array accesses (see `getTokenLoc`).
      let regexObj: Regex;
      if (activeTokensWithRegexCount < regexObjects.length) {
        regexObj = regexObjects[activeTokensWithRegexCount];
        tokensWithRegex[activeTokensWithRegexCount] = this;
      } else {
        regexObjects.push((regexObj = { pattern: null!, flags: null! }));
        tokensWithRegex.push(this);
      }
      activeTokensWithRegexCount++;

      // Parse the regex literal (`/pattern/flags`) from source text.
      // Find the closing `/` by searching back from `end - 1` (the token's last char).
      // `end` (exclusive) would be wrong - `sourceText[end]` is the character *after* the token,
      // which can itself be `/` - e.g. division `/a/g/2` or comment `/a/g//comment`.
      const start = tokensInt32[pos32],
        end = tokensInt32[pos32 + 1];
      const patternEnd = sourceText.lastIndexOf("/", end - 1);
      regexObj.pattern = sourceText.slice(start + 1, patternEnd);
      regexObj.flags = sourceText.slice(patternEnd + 1, end);

      return (this.#regex = regexObj);
    };

    resetRangeTemp = (token: Token) => {
      token.#range = null;
    };

    resetLocTemp = (token: Token) => {
      token.#loc = null;
    };

    resetRegexTemp = (token: Token) => {
      // Clear the `Regex` object's `pattern` and `flags` fields, to release source text string slices
      const regex = token.#regex!;
      regex.pattern = null!;
      regex.flags = null!;
      token.#regex = null;
    };

    if (DEBUG) getTokenPrivateRange = (token: Token) => token.#range;
    if (DEBUG) getTokenPrivateLoc = (token: Token) => token.#loc;
    if (DEBUG) getTokenPrivateRegex = (token: Token) => token.#regex;
  }
}

// Copied into consts here to avoid checks at call site (`let` binding could be re-assigned)
const getTokenType = getTokenTypeTemp;
const getTokenStart = getTokenStartTemp;
const getTokenEnd = getTokenEndTemp;
const getTokenRange = getTokenRangeTemp;
const getTokenLoc = getTokenLocTemp;
const getTokenValue = getTokenValueTemp;
const getTokenRegex = getTokenRegexTemp;

const resetRange = resetRangeTemp;
const resetLoc = resetLocTemp;
const resetRegex = resetRegexTemp;

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

// Mask of bits containing token kind
const TOKEN_KIND_MASK = 15;

// Details of Rust `Token` type
export const TOKEN_SIZE = 16;
debugAssert(TOKEN_SIZE === COMMENT_SIZE, "Size of token, comment, and merged entry must be equal");

const TOKEN_SIZE_SHIFT = 4;
debugAssert(TOKEN_SIZE === 1 << TOKEN_SIZE_SHIFT);

// Same as `TOKEN_SIZE` / `TOKEN_SIZE_SHIFT`, but in units of `i32`s (for indexing `tokensInt32`).
const TOKEN_SIZE32 = TOKEN_SIZE >> 2;
const TOKEN_SIZE32_SHIFT = TOKEN_SIZE_SHIFT - 2;
debugAssert(TOKEN_SIZE32 === 1 << TOKEN_SIZE32_SHIFT);

// `kind` (byte 8) and `escaped` (byte 10) both live in the token's 3rd `i32` (bytes 8-11), so a single
// `tokensInt32` read yields both - no `Uint8Array` view needed. Extract each with a mask.
// See `Token` bit layout in `crates/oxc_parser/src/lexer/token.rs`.
const KIND_FLAGS_OFFSET32 = 2;
const IS_ESCAPED_MASK = 0x1_0000; // `escaped` bool is byte 10 = bit 16 of the word

/**
 * Build the `tokens` array (a slice of `cachedTokens`).
 *
 * Unlike `initTokensArray`, caller does not need to call `initTokensBuffer()` first.
 *
 * This is used by `ast.tokens` getter.
 */
export function initTokens(): void {
  debugAssert(tokens === null, "Tokens already initialized");

  if (tokensInt32 === null) initTokensBuffer();
  initTokensArray();
}

/*
 * Build the `tokens` array (a slice of `cachedTokens`).
 *
 * Caller must ensure `initTokensBuffer()` has been called first
 * (so token buffers and source text are already initialized).
 *
 * Called by `ast.tokens` getter.
 */
export function initTokensArray(): void {
  debugAssert(tokens === null, "Tokens already initialized");
  debugAssert(
    tokensInt32 !== null && sourceText !== null,
    "`initTokensBuffer` must be called before `initTokens`",
  );

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
 * Initialize typed array views over the tokens region of the buffer.
 *
 * Populates `tokensInt32` and `tokensLen`, and grows `cachedTokens` if needed.
 */
export function initTokensBuffer(): void {
  debugAssert(tokensInt32 === null, "Tokens buffer already initialized");

  debugAssertIsNonNull(buffer);

  // Various tokens methods rely on `sourceText` being initialized after `initTokensBuffer`,
  // so we always initialize it here, even if there are no tokens (empty file)
  if (sourceText === null) initSourceText();
  debugAssertIsNonNull(sourceText);

  const { int32 } = buffer;
  const tokensPos = int32[TOKENS_OFFSET_POS_32];
  tokensLen = int32[TOKENS_LEN_POS_32];

  // Create typed array view over just the tokens region of the buffer.
  // This is a zero-copy view over the same underlying `ArrayBuffer`.
  // View persists for the lifetime of the file (cleared in `resetTokens`).
  const arrayBuffer = buffer.buffer,
    absolutePos = buffer.byteOffset + tokensPos;
  tokensInt32 = new Int32Array(arrayBuffer, absolutePos, tokensLen << TOKEN_SIZE32_SHIFT);

  // Grow caches if needed. After first few files, caches should have grown large enough to service all files.
  // Later files will skip this step, and allocations stop.
  if (cachedTokens.length < tokensLen) {
    // Loop on a local `pos32` counter rather than calculating `pos32 = cachedTokens.length << TOKEN_SIZE32_SHIFT`
    // on each turn of the loop. `Array#push` is not inlined for arrays of objects, so testing `cachedTokens.length`
    // in the loop condition would reload `.length` from the heap on every iteration.
    const endPos32 = tokensLen << TOKEN_SIZE32_SHIFT;
    let pos32 = cachedTokens.length << TOKEN_SIZE32_SHIFT;
    do {
      cachedTokens.push(new Token(pos32));
      // `| 0` truncates the sum to int32, so V8 drops the SMI overflow check on this add.
      // Buffer is limited to 2 GiB, so any valid `pos32` is a positive int32, so this is safe.
      pos32 = (pos32 + TOKEN_SIZE32) | 0;
    } while (pos32 < endPos32);
  }

  // Check tokens have valid ranges and are in ascending order
  debugCheckValidRanges();
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
 * Check all tokens have valid ranges, are in ascending order, and are within the source text.
 *
 * Only runs in debug build (tests). In release build, this function is entirely removed by minifier.
 */
function debugCheckValidRanges(): void {
  if (!DEBUG) return;

  debugAssertIsNonNull(sourceText, "`sourceText` should be initialized");

  let lastEnd = 0;
  for (let i = 0; i < tokensLen; i++) {
    const { start, end } = cachedTokens[i];
    if (end <= start) throw new Error(`Invalid token range: ${start}-${end}`);
    if (start < lastEnd) {
      throw new Error(`Overlapping tokens: last end: ${lastEnd}, next start: ${start}`);
    }
    lastEnd = end;
  }

  if (lastEnd > sourceText.length) {
    throw new Error(`Tokens end beyond source text length: ${lastEnd} > ${sourceText.length}`);
  }
}

/**
 * Reset tokens after file has been linted.
 *
 * Clears cached `loc` on tokens that had it accessed, so the getter
 * will recalculate it when the token is reused for a different file.
 */
export function resetTokens() {
  // Early exit if tokens were never accessed (e.g. no rules used tokens-related methods)
  if (tokensInt32 === null) {
    debugAssertAllTokensCleared();
    return;
  }

  // Reset `#range` on tokens where `range` has been accessed
  for (let i = 0; i < activeTokensWithRangeCount; i++) {
    resetRange(tokensWithRange[i]);
  }

  activeTokensWithRangeCount = 0;

  // Reset `#loc` on tokens where `loc` has been accessed
  for (let i = 0; i < activeTokensWithLocCount; i++) {
    resetLoc(tokensWithLoc[i]);
  }

  activeTokensWithLocCount = 0;

  // Reset `#regex` on tokens where `regex` has been accessed
  for (let i = 0; i < activeTokensWithRegexCount; i++) {
    resetRegex(tokensWithRegex[i]);
  }

  activeTokensWithRegexCount = 0;

  // Clear other state
  tokens = null;
  tokensInt32 = null;
  tokensLen = 0;

  debugAssertAllTokensCleared();
}

/**
 * Check that all token and regex objects have been cleared.
 *
 * Only runs in debug build (tests). In release build, this function is entirely removed by minifier.
 */
function debugAssertAllTokensCleared(): void {
  if (!DEBUG) return;

  // Check all cached tokens have `#range: null`, `#loc: null`, and `#regex: null`
  for (let i = 0; i < cachedTokens.length; i++) {
    const token = cachedTokens[i];
    if (getTokenPrivateRange(token) !== null) {
      throw new Error(`Token ${i} has not had \`#range\` cleared`);
    }
    if (getTokenPrivateLoc(token) !== null) {
      throw new Error(`Token ${i} has not had \`#loc\` cleared`);
    }
    if (getTokenPrivateRegex(token) !== null) {
      throw new Error(`Token ${i} has not had \`#regex\` cleared`);
    }
  }

  // Check all regex objects have `pattern: null` and `flags: null`
  for (let i = 0; i < regexObjects.length; i++) {
    const regex = regexObjects[i];
    if (regex.pattern !== null) throw new Error(`Regex ${i} has not had \`pattern\` cleared`);
    if (regex.flags !== null) throw new Error(`Regex ${i} has not had \`flags\` cleared`);
  }
}
