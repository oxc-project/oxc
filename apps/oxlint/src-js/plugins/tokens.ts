/*
 * Token types and tokens initialization / reset.
 */

import { buffer, initSourceText, sourceText } from "./source_code.ts";
import {
  activeLocationsCount,
  activeRangesCount,
  cachedLocations,
  cachedRanges,
  computeLoc,
  createRange,
  GEN_ID_MASK,
  GEN_ID_MAX,
  GEN_ID_SHIFT,
  MIN_SIDE_TABLE_SIZE,
  RANGES_AND_LOCS_MAX_COUNT,
} from "./location.ts";
import { COMMENT_SIZE, TOKENS_OFFSET_POS_32, TOKENS_LEN_POS_32 } from "../generated/constants.ts";
import { EMPTY_INT32_ARRAY } from "../utils/typed_arrays.ts";
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

// Side tables caching each token's `range` and `loc`, keyed by token index (`#pos32 >> TOKEN_SIZE32_SHIFT`).
//
// Each entry stores index into `cachedRanges` / `cachedLocations` pool in bits 0-26 and a gen ID in bits 27-31.
//
// An entry is valid only if its gen ID equals the current gen ID (`tokenRangesGenId` / `tokenLocsGenId`).
// Gen IDs cycle through 1-31. 0 is reserved - it's the value of a zeroed (or never-written) entry,
// so a zeroed entry can never validate (`resetTokens` zeroes all entries).
//
// `range` and `loc` are fully independent (separate gen IDs, and reset), but the two tables always
// have the same length - they're grown together in `initTokensBuffer`.
let tokenRangeIndices = EMPTY_INT32_ARRAY;
let tokenRangesGenId = 1;
let tokenRangesGenIdShifted = 1 << GEN_ID_SHIFT;
// `true` if any token `range` was accessed for the current file.
let tokenRangesAccessed = false;
// Max `tokensLen` across files in which token `range`s were accessed, since the table was last reset.
let maxTokenRangesLen = 0;

let tokenLocIndices = EMPTY_INT32_ARRAY;
let tokenLocsGenId = 1;
let tokenLocsGenIdShifted = 1 << GEN_ID_SHIFT;
// `true` if any token `loc` was accessed for the current file.
let tokenLocsAccessed = false;
// Max `tokensLen` across files in which token `loc`s were accessed, since the table was last reset.
let maxTokenLocsLen = 0;

// Cached `Regex` objects, reused across files.
// `regexIndices.size` is the index of the next `Regex` object available for reuse (not already in use in this file).
const regexObjects: Regex[] = [];

// Map from a regex token's `pos32` to the index of the `Regex` object in `regexObjects`.
// Keys and values are both integers (SMIs), so the map's backing store contains no heap pointers -
// `Map#set` incurs no write barriers, and GC never has to mark through it.
// Regex tokens are rare, so this map is almost always empty. Cleared in `resetTokens`.
const regexIndices = new Map<number, number>();

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

/**
 * Token class.
 *
 * All properties are defined as own accessor properties via `__defineGetter__` in the constructor,
 * using shared getter functions (e.g. `getTokenType`). This makes them own enumerable
 * properties, so `{...token}` spreads them and `JSON.stringify(token)` serializes them.
 *
 * Both `range` and `loc` cache a pool index in a side table (`tokenRangeIndices` / `tokenLocIndices`),
 * keyed by token index and validated by gen ID (see the getters), so accessing either twice returns the
 * same object.
 *
 * `Regex` objects are cached in the `regexIndices` map, keyed by `#pos32`.
 *
 * No value is cached on the instance itself - it holds only `#pos32`.
 *
 * All instances share the same getter functions, keeping the V8 hidden class transition identical across instances.
 * Reset bumps the gen IDs to invalidate both side tables, and clears the `regexIndices` map.
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

      // Check the side table for a `Range` created by an earlier access in this file.
      // Entry layout: `cachedRanges` pool index in bits 0-26, gen ID in bits 27-31.
      // XOR with the current shifted gen ID zeroes the gen bits if (and only if) the entry was written
      // for the current file - what remains is then the pool index itself.
      // Zeroed entries never match - gen IDs cycle through 1-31, and a zeroed entry has gen ID 0.
      const pos32 = this.#pos32;
      const index = pos32 >> TOKEN_SIZE32_SHIFT;
      const rangeIndex = tokenRangeIndices[index] ^ tokenRangesGenIdShifted;
      if ((rangeIndex & GEN_ID_MASK) === 0) return cachedRanges[rangeIndex];

      // `activeRangesCount` is the index the new `Range` will occupy in `cachedRanges`
      tokenRangeIndices[index] = activeRangesCount | tokenRangesGenIdShifted;
      tokenRangesAccessed = true;
      return createRange(tokensInt32[pos32], tokensInt32[pos32 + 1]);
    };

    getTokenLocTemp = function (this: Token): Location {
      // This assert can fail in real-world plugin code, and is not a bug here, only incorrect usage in plugin.
      // Only make this an explicit error in debug build, because it should be very uncommon.
      // In release build, it will result in an error in the `tokensInt32` access below.
      debugAssertIsNonNull(
        tokensInt32,
        "`Token` object's `loc` field accessed after file finished linting",
      );

      // Check the side table for a `Location` created by an earlier access in this file.
      // Entry layout: `cachedLocations` pool index in bits 0-26, gen ID in bits 27-31.
      // XOR with the current shifted gen ID zeroes the gen bits if (and only if) the entry was written
      // for the current file - what remains is then the pool index itself.
      // Zeroed entries never match - gen IDs cycle through 1-31, and a zeroed entry has gen ID 0.
      const pos32 = this.#pos32;
      const index = pos32 >> TOKEN_SIZE32_SHIFT;
      const locIndex = tokenLocIndices[index] ^ tokenLocsGenIdShifted;
      if ((locIndex & GEN_ID_MASK) === 0) return cachedLocations[locIndex];

      // `activeLocationsCount` is the index the new `Location` will occupy in `cachedLocations`
      tokenLocIndices[index] = activeLocationsCount | tokenLocsGenIdShifted;
      tokenLocsAccessed = true;
      return computeLoc(tokensInt32[pos32], tokensInt32[pos32 + 1]);
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

      // Return the `Regex` object created by a previous access, if any
      let index = regexIndices.get(pos32);
      if (index !== undefined) return regexObjects[index];

      // First access. Reuse a pooled `Regex` object if available, otherwise create a new one,
      // and record its index in `regexIndices`, so later accesses return the same object.
      // `regexIndices.size` is the number of objects used so far, so also the index of the next reusable one.
      // Note: The comparison `index < regexObjects.length` must be this way around
      // so that V8 can remove the bounds check on `regexObjects[index]` (see `getTokenLoc`).
      index = regexIndices.size;
      let regexObj: Regex;
      if (index < regexObjects.length) {
        regexObj = regexObjects[index];
      } else {
        regexObjects.push((regexObj = { pattern: null!, flags: null! }));
      }
      regexIndices.set(pos32, index);

      // Parse the regex literal (`/pattern/flags`) from source text.
      // Find the closing `/` by searching back from `end - 1` (the token's last char).
      // `end` (exclusive) would be wrong - `sourceText[end]` is the character *after* the token,
      // which can itself be `/` - e.g. division `/a/g/2` or comment `/a/g//comment`.
      const start = tokensInt32[pos32],
        end = tokensInt32[pos32 + 1];
      const patternEnd = sourceText.lastIndexOf("/", end - 1);
      regexObj.pattern = sourceText.slice(start + 1, patternEnd);
      regexObj.flags = sourceText.slice(patternEnd + 1, end);

      return regexObj;
    };
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
 * Populates `tokensInt32` and `tokensLen`, and grows `cachedTokens` and the
 * `tokenRangeIndices` / `tokenLocIndices` side tables if needed.
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

    // Grow the `range` and `loc` side tables, so they always have an entry for every token.
    // The two tables always have the same length - both start as `EMPTY_INT32_ARRAY` and are only ever
    // grown here, together - so a single check and size covers both.
    // `Int32Array`s can't grow in place, so allocate new ones, doubling to amortize growth across files,
    // capped at max `RANGES_AND_LOCS_MAX_COUNT` (the most entries that could ever be required),
    // and minimum `MIN_SIDE_TABLE_SIZE` (to avoid tiny buffers).
    const sideTablesLen = tokenRangeIndices.length;
    if (sideTablesLen < tokensLen) {
      const minSize =
        sideTablesLen === 0
          ? MIN_SIDE_TABLE_SIZE
          : Math.min(sideTablesLen * 2, RANGES_AND_LOCS_MAX_COUNT);
      const newSize = Math.max(tokensLen, minSize);
      tokenRangeIndices = new Int32Array(newSize);
      tokenLocIndices = new Int32Array(newSize);
    }
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
 * Bumps the `range` and `loc` gen IDs to invalidate their side tables, so the getters recompute
 * and draw fresh pool objects when a token is reused for a different file.
 */
export function resetTokens() {
  // Early exit if tokens were never accessed (e.g. no rules used tokens-related methods)
  if (tokensInt32 === null) {
    debugAssertAllTokensCleared();
    return;
  }

  // If any token `range`s were accessed for this file, bump the gen ID, which invalidates all entries
  // in `tokenRangeIndices` (the `Range`s they point to go back in the pool for reuse by the next file).
  // Files where no token `range` was accessed leave the gen ID alone - entries can only have been
  // written in files that bumped it, so every live entry's gen ID stays behind the current one.
  // After the gen ID has run through all 31 values, zero the table's used region and start over at 1.
  // `maxTokenRangesLen` bounds the region that can contain entries written since the last zeroing.
  if (tokenRangesAccessed === true) {
    if (tokensLen > maxTokenRangesLen) maxTokenRangesLen = tokensLen;

    if (tokenRangesGenId === GEN_ID_MAX) {
      tokenRangeIndices.fill(0, 0, maxTokenRangesLen);
      tokenRangesGenId = 1;
      maxTokenRangesLen = 0;
    } else {
      tokenRangesGenId++;
    }

    tokenRangesGenIdShifted = tokenRangesGenId << GEN_ID_SHIFT;
    tokenRangesAccessed = false;
  }

  // Same as the `range` block above, but for `loc`.
  // Bump the independent `loc` gen ID (or wrap and zero the used region at `GEN_ID_MAX`),
  // invalidating all entries in `tokenLocIndices`.
  if (tokenLocsAccessed === true) {
    if (tokensLen > maxTokenLocsLen) maxTokenLocsLen = tokensLen;

    if (tokenLocsGenId === GEN_ID_MAX) {
      tokenLocIndices.fill(0, 0, maxTokenLocsLen);
      tokenLocsGenId = 1;
      maxTokenLocsLen = 0;
    } else {
      tokenLocsGenId++;
    }

    tokenLocsGenIdShifted = tokenLocsGenId << GEN_ID_SHIFT;
    tokenLocsAccessed = false;
  }

  // Clear `pattern` and `flags` on the `Regex` objects used for this file, to release source text string slices,
  // and clear the map from tokens to objects.
  // Skip if no regexes were accessed, so `Map#clear` doesn't allocate a new backing table for the map.
  const regexCount = regexIndices.size;
  if (regexCount !== 0) {
    for (let i = 0; i < regexCount; i++) {
      const regexObj = regexObjects[i];
      regexObj.pattern = null!;
      regexObj.flags = null!;
    }

    regexIndices.clear();
  }

  // Clear other state
  tokens = null;
  tokensInt32 = null;
  tokensLen = 0;

  debugAssertAllTokensCleared();
}

/**
 * Check that the token side tables were invalidated for this file (gen IDs bumped, accessed flags reset),
 * and all regex objects have been cleared.
 *
 * Only runs in debug build (tests). In release build, this function is entirely removed by minifier.
 */
function debugAssertAllTokensCleared(): void {
  if (!DEBUG) return;

  // Check both token side tables were invalidated for this file (gen IDs bumped, flags reset)
  if (tokenRangesAccessed !== false) {
    throw new Error("`tokenRangesAccessed` was not reset");
  }
  if (tokenLocsAccessed !== false) {
    throw new Error("`tokenLocsAccessed` was not reset");
  }

  // Check the map from tokens to `Regex` objects has been cleared
  if (regexIndices.size !== 0) throw new Error("`regexIndices` has not been cleared");

  // Check all regex objects have `pattern: null` and `flags: null`
  for (let i = 0; i < regexObjects.length; i++) {
    const regex = regexObjects[i];
    if (regex.pattern !== null) throw new Error(`Regex ${i} has not had \`pattern\` cleared`);
    if (regex.flags !== null) throw new Error(`Regex ${i} has not had \`flags\` cleared`);
  }
}
