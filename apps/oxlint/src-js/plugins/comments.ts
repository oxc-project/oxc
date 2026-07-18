/*
 * Comment class, object pooling, and deserialization.
 */

import { buffer, initSourceText, sourceText } from "./source_code.ts";
import {
  COMMENTS_OFFSET,
  COMMENTS_LEN_OFFSET,
  COMMENT_SIZE,
  COMMENT_KIND_OFFSET,
  COMMENT_LINE_KIND,
  COMMENT_SHEBANG_KIND,
  DATA_POINTER_POS_32,
  DESERIALIZED_FLAG_OFFSET,
} from "../generated/constants.ts";
import { computeLoc } from "./location.ts";
import { FLAG_NOT_DESERIALIZED, FLAG_DESERIALIZED } from "./tokens.ts";
import { EMPTY_UINT8_ARRAY, EMPTY_INT32_ARRAY } from "../utils/typed_arrays.ts";
import { debugAssert, debugAssertIsNonNull } from "../utils/asserts.ts";

import type { Location, Range, Span } from "./location.ts";

/**
 * Comment.
 */
interface CommentType extends Span {
  type: "Line" | "Block" | "Shebang";
  value: string;
}

// Export type as `Comment` for external consumers
export type { CommentType as Comment };

// Array of comment `type`s, indexed by `CommentKind` discriminant.
//
// Slot 3 (`COMMENT_SHEBANG_KIND`) holds `"Shebang"`. Rust side writes that synthetic kind to the hashbang
// comment's `kind` byte in the buffer (it's not a real `CommentKind`), so we read it here as a `Shebang` comment.
const COMMENT_TYPES: CommentType["type"][] = ["Block", "Block", "Block", "Block"];
COMMENT_TYPES[COMMENT_LINE_KIND] = "Line";
COMMENT_TYPES[COMMENT_SHEBANG_KIND] = "Shebang";

// Array of numbers to subtract from `end` when slicing source text to get `value` of a comment,
// indexed by `CommentKind` discriminant.
const COMMENT_END_SUBTRACTIONS: number[] = [2, 2, 2, 2];
COMMENT_END_SUBTRACTIONS[COMMENT_LINE_KIND] = 0;
COMMENT_END_SUBTRACTIONS[COMMENT_SHEBANG_KIND] = 0;

// Comments for the current file.
// Created lazily only when needed.
export let comments: CommentType[] | null = null;

// Typed array views over the comments region of the buffer.
// These persist for the lifetime of the file (cleared in `resetComments`).
let commentsUint8: Uint8Array | null = null;
export let commentsInt32: Int32Array | null = null;

// Number of comments for the current file.
export let commentsLen = 0;

// Whether all comments have been deserialized into `cachedComments`.
export let allCommentsDeserialized = false;

// Cached comment objects, reused across files to reduce GC pressure.
// Comments are mutated in place during deserialization, then `comments` is set to a slice of this array.
export const cachedComments: Comment[] = [];

// Comments array from previous file.
// Reused for next file if next file has fewer comments than the previous file (by truncating to correct length).
let previousComments: Comment[] = [];

// Comments whose `range` property has been accessed, and therefore needs clearing on reset.
// Never shrunk - `activeCommentsWithRangeCount` tracks the active count to avoid freeing the backing store.
const commentsWithRange: Comment[] = [];
let activeCommentsWithRangeCount = 0;

// Comments whose `loc` property has been accessed, and therefore needs clearing on reset.
// Never shrunk - `activeCommentsWithLocCount` tracks the active count to avoid freeing the backing store.
const commentsWithLoc: Comment[] = [];
let activeCommentsWithLocCount = 0;

// Empty comments array.
// Reused for all files which don't have any comments. Frozen to avoid rules mutating it.
const EMPTY_COMMENTS: CommentType[] = Object.freeze([]) as unknown as CommentType[];

const COMMENT_SIZE_SHIFT = 4; // 1 << 4 == 16 bytes, the size of `Comment` in Rust
debugAssert(COMMENT_SIZE === 1 << COMMENT_SIZE_SHIFT);

// `defineGetter(obj, prop, getter)` is equivalent to `obj.__defineGetter__(prop, getter)`,
// but without `Object.prototype` lookup at each call site
const defineGetter = Function.prototype.call.bind(
  // @ts-expect-error - `__defineGetter__` is not in `Object.prototype`'s type definition,
  // but it does exist at runtime and is widely supported in JS engines, including V8
  Object.prototype.__defineGetter__,
) as (obj: object, prop: string, getter: () => unknown) => void;

// Getters for `type`, `start`, `end`, `range`, `loc`, and `value` properties on a `Comment` class instance.
// Copied into `const`s below after being defined in class static block.
let getCommentTypeTemp: (this: Comment) => CommentType["type"];
let getCommentStartTemp: (this: Comment) => number;
let getCommentEndTemp: (this: Comment) => number;
let getCommentRangeTemp: (this: Comment) => Range;
let getCommentLocTemp: (this: Comment) => Location;
let getCommentValueTemp: (this: Comment) => string;

// Setter for `#pos` private property on a `Comment` class instance.
// Copied into a `const` below after being defined in class static block.
let setCommentPosTemp: (comment: Comment, pos: number) => void;

// Reset `#range` field on a `Comment` class instance.
// Copied into a `const` below after being defined in class static block.
let resetCommentRangeTemp: (comment: Comment) => void;

// Reset `#loc` field on a `Comment` class instance.
// Copied into a `const` below after being defined in class static block.
let resetCommentLocTemp: (comment: Comment) => void;

// Get `#range` field on a `Comment` class instance.
// Only used in debug build (tests).
let getCommentPrivateRange: (comment: Comment) => Range | null;

// Get `#loc` field on a `Comment` class instance.
// Only used in debug build (tests).
let getCommentPrivateLoc: (comment: Comment) => Location | null;

/**
 * Comment class.
 *
 * `range` and `loc` are defined as own accessor properties via `__defineGetter__` in the constructor,
 * using shared getter functions (`getCommentRange` / `getCommentLoc`). This makes them own enumerable
 * properties, so `{...comment}` spreads them and `JSON.stringify(comment)` serializes them.
 *
 * The computed `range` array and `Location` value are cached in the private `#range` / `#loc` fields
 * on first access, so accessing either twice returns the same object. All instances share the same
 * getter functions, keeping the V8 hidden class transition identical across instances.
 * Reset only clears the `#range` and `#loc` fields.
 */
class Comment implements Span {
  // All defined with `__defineGetter__` in constructor
  declare type: CommentType["type"];
  declare start: number;
  declare end: number;
  declare range: Range;
  declare loc: Location;
  declare value: string;

  #pos: number = 0;
  #range: Range | null = null;
  #loc: Location | null = null;

  constructor() {
    // Define all properties as own getter properties (enumerable + configurable by default).
    // This makes `{...comment}` spread them, and `JSON.stringify(comment)` serialize them.
    // Note: `new Comment()` is 25% faster with `__defineGetter__` vs `Object.defineProperty`.
    // See https://github.com/oxc-project/oxc/pull/22238.
    defineGetter(this, "type", getCommentType);
    defineGetter(this, "start", getCommentStart);
    defineGetter(this, "end", getCommentEnd);
    defineGetter(this, "range", getCommentRange);
    defineGetter(this, "loc", getCommentLoc);
    defineGetter(this, "value", getCommentValue);
  }

  // Functions requiring access to `#pos` or `#loc` defined in static block to avoid exposing them as public methods
  static {
    getCommentTypeTemp = function (this: Comment): CommentType["type"] {
      // This assert can fail in real-world plugin code, and is not a bug here, only incorrect usage in plugin.
      // Only make this an explicit error in debug build, because it should be very uncommon.
      // In release build, it will result in an error in the `commentsUint8` access below.
      debugAssertIsNonNull(
        commentsUint8,
        "`Comment` object's `type` field accessed after file finished linting",
      );

      return COMMENT_TYPES[commentsUint8[this.#pos + COMMENT_KIND_OFFSET]];
    };

    getCommentStartTemp = function (this: Comment): number {
      // This assert can fail in real-world plugin code, and is not a bug here, only incorrect usage in plugin.
      // Only make this an explicit error in debug build, because it should be very uncommon.
      // In release build, it will result in an error in the `commentsInt32` access below.
      debugAssertIsNonNull(
        commentsInt32,
        "`Comment` object's `start` field accessed after file finished linting",
      );

      return commentsInt32[this.#pos >> 2];
    };

    getCommentEndTemp = function (this: Comment): number {
      // This assert can fail in real-world plugin code, and is not a bug here, only incorrect usage in plugin.
      // Only make this an explicit error in debug build, because it should be very uncommon.
      // In release build, it will result in an error in the `commentsInt32` access below.
      debugAssertIsNonNull(
        commentsInt32,
        "`Comment` object's `end` field accessed after file finished linting",
      );

      return commentsInt32[(this.#pos >> 2) + 1];
    };

    getCommentRangeTemp = function (this: Comment): Range {
      // This assert can fail in real-world plugin code, and is not a bug here, only incorrect usage in plugin.
      // Only make this an explicit error in debug build, because it should be very uncommon.
      // In release build, it will result in an error in the `commentsInt32` access below.
      debugAssertIsNonNull(
        commentsInt32,
        "`Comment` object's `range` field accessed after file finished linting",
      );

      const range = this.#range;
      if (range !== null) return range;

      // Store comment in `commentsWithRange` array. `resetComments` will clear the `#range` property.
      // Note: The comparison `activeCommentsWithRangeCount < commentsWithRange.length` must be this way around
      // so that V8 can remove the bounds check on `commentsWithRange[activeCommentsWithRangeCount]`.
      // `commentsWithRange.length > activeCommentsWithRangeCount` would *not* remove the bounds check in Maglev compiler.
      if (activeCommentsWithRangeCount < commentsWithRange.length) {
        commentsWithRange[activeCommentsWithRangeCount] = this;
      } else {
        commentsWithRange.push(this);
      }
      activeCommentsWithRangeCount++;

      const pos32 = this.#pos >> 2;
      return (this.#range = [commentsInt32[pos32], commentsInt32[pos32 + 1]]);
    };

    getCommentLocTemp = function (this: Comment): Location {
      // This assert can fail in real-world plugin code, and is not a bug here, only incorrect usage in plugin.
      // Only make this an explicit error in debug build, because it should be very uncommon.
      // In release build, it will result in an error in the `commentsInt32` access below.
      debugAssertIsNonNull(
        commentsInt32,
        "`Comment` object's `loc` field accessed after file finished linting",
      );

      const loc = this.#loc;
      if (loc !== null) return loc;

      // Store comment in `commentsWithLoc` array. `resetComments` will clear the `#loc` property.
      // Note: The comparison `activeCommentsWithLocCount < commentsWithLoc.length` must be this way around
      // so that V8 can remove the bounds check on `commentsWithLoc[activeCommentsWithLocCount]`.
      // `commentsWithLoc.length > activeCommentsWithLocCount` would *not* remove the bounds check in Maglev compiler.
      if (activeCommentsWithLocCount < commentsWithLoc.length) {
        commentsWithLoc[activeCommentsWithLocCount] = this;
      } else {
        commentsWithLoc.push(this);
      }
      activeCommentsWithLocCount++;

      const pos32 = this.#pos >> 2,
        start = commentsInt32[pos32],
        end = commentsInt32[pos32 + 1];
      return (this.#loc = computeLoc(start, end));
    };

    getCommentValueTemp = function (this: Comment): string {
      // This assert can fail in real-world plugin code, and is not a bug here, only incorrect usage in plugin.
      // Only make this an explicit error in debug build, because it should be very uncommon.
      // In release build, it will result in an error in `commentsUint8`, `commentsInt32`, or `sourceText`
      // accesses below.
      debugAssert(
        commentsUint8 !== null && commentsInt32 !== null && sourceText !== null,
        "`Comment` object's `value` field accessed after file finished linting",
      );

      const pos = this.#pos;

      const kind = commentsUint8[pos + COMMENT_KIND_OFFSET];

      // Line comments: `// text` -> slice `start + 2..end`
      // Block comments: `/* text */` -> slice `start + 2..end - 2`
      // Hashbang: `#! text` -> slice `start + 2..end`
      const pos32 = pos >> 2,
        start = commentsInt32[pos32],
        end = commentsInt32[pos32 + 1];
      return sourceText.slice(start + 2, end - COMMENT_END_SUBTRACTIONS[kind]);
    };

    setCommentPosTemp = function (comment: Comment, pos: number) {
      comment.#pos = pos;
    };

    resetCommentRangeTemp = (comment: Comment) => {
      comment.#range = null;
    };

    resetCommentLocTemp = (comment: Comment) => {
      comment.#loc = null;
    };

    if (DEBUG) getCommentPrivateRange = (comment: Comment) => comment.#range;
    if (DEBUG) getCommentPrivateLoc = (comment: Comment) => comment.#loc;
  }
}

// Copied into consts here to avoid checks at call site (`let` binding could be re-assigned)
const getCommentType = getCommentTypeTemp;
const getCommentStart = getCommentStartTemp;
const getCommentEnd = getCommentEndTemp;
const getCommentRange = getCommentRangeTemp;
const getCommentLoc = getCommentLocTemp;
const getCommentValue = getCommentValueTemp;

const setCommentPos = setCommentPosTemp;
const resetCommentRange = resetCommentRangeTemp;
const resetCommentLoc = resetCommentLocTemp;

/**
 * Deserialize all comments and build the `comments` array.
 * Called by `ast.comments` getter.
 */
export function initComments(): void {
  debugAssert(comments === null, "Comments already deserialized");

  if (allCommentsDeserialized === false) deserializeComments();

  // `initCommentsBuffer` (called by `deserializeComments`) sets `comments` for zero-comment files
  if (comments !== null) return;

  // Create `comments` array as a slice of `cachedComments` array.
  //
  // Use `slice` rather than copying comments one-by-one into a new array.
  // V8 implements `slice` with a single `memcpy` of the backing store, which is faster
  // than N individual `push` calls with bounds checking and potential resizing.
  //
  // If the comments array from previous file is longer than the current one,
  // reuse it and truncate it to avoid the memcpy entirely.
  // Assuming random distribution of number of comments in files, this cheaper branch should be hit on 50% of files.
  if (previousComments.length >= commentsLen) {
    previousComments.length = commentsLen;
    comments = previousComments;
  } else {
    comments = previousComments = cachedComments.slice(0, commentsLen);
  }
}

/**
 * Deserialize all comments into `cachedComments`.
 * Does NOT build the `comments` array - use `initComments` for that.
 */
export function deserializeComments(): void {
  debugAssert(allCommentsDeserialized === false, "Comments already deserialized");

  if (commentsInt32 === null) initCommentsBuffer();

  for (let i = 0; i < commentsLen; i++) {
    deserializeCommentIfNeeded(i);
  }

  allCommentsDeserialized = true;

  debugCheckDeserializedComments();
}

/**
 * Initialize typed array views over the comments region of the buffer.
 *
 * Populates `commentsUint8`, `commentsInt32`, and `commentsLen`, and grows `cachedComments` if needed.
 * Does NOT deserialize comments - they are deserialized lazily via `deserializeCommentIfNeeded`.
 */
export function initCommentsBuffer(): void {
  debugAssert(
    commentsUint8 === null && commentsInt32 === null,
    "Comments buffer already initialized",
  );

  debugAssert(
    allCommentsDeserialized === false,
    "`allCommentsDeserialized` flag should have been reset at end of last file",
  );

  debugAssertIsNonNull(buffer);

  // We don't need to deserialize source text if there are no comments, so we could move this call to after
  // the `commentsLen === 0` check. However, various comments methods rely on that if `initComments` has been called,
  // then `sourceText` is initialized. Doing it eagerly here avoids having to check if `sourceText` is `null`
  // in all those methods, which can be called quite frequently.
  if (sourceText === null) initSourceText();
  debugAssertIsNonNull(sourceText);

  const { int32 } = buffer;
  const programPos32 = int32[DATA_POINTER_POS_32] >> 2;
  const commentsPos = int32[programPos32 + (COMMENTS_OFFSET >> 2)];
  commentsLen = int32[programPos32 + (COMMENTS_LEN_OFFSET >> 2)];

  // Fast path for files with no comments
  if (commentsLen === 0) {
    comments = EMPTY_COMMENTS;
    commentsUint8 = EMPTY_UINT8_ARRAY;
    commentsInt32 = EMPTY_INT32_ARRAY;
    allCommentsDeserialized = true;
    return;
  }

  // Create typed array views over the comments region of the buffer.
  // These are zero-copy views over the same underlying `ArrayBuffer`.
  const arrayBuffer = buffer.buffer,
    absolutePos = buffer.byteOffset + commentsPos;
  commentsUint8 = new Uint8Array(arrayBuffer, absolutePos, commentsLen * COMMENT_SIZE);
  commentsInt32 = new Int32Array(arrayBuffer, absolutePos, commentsLen * (COMMENT_SIZE >> 2));

  // Grow caches if needed. After first few files, caches should have grown large enough to service all files.
  // Later files will skip this step, and allocations stop.
  if (cachedComments.length < commentsLen) {
    do {
      cachedComments.push(new Comment());
    } while (cachedComments.length < commentsLen);
  }

  // Check buffer data has valid ranges and ascending order
  debugCheckValidRanges();
}

/**
 * Get comment at `index`, deserializing if needed.
 *
 * Caller must ensure `initCommentsBuffer()` has been called before calling this function.
 *
 * @param index - Comment index in the comments buffer
 * @returns Deserialized comment
 */
export function getComment(index: number): CommentType {
  // Skip all other checks if all comments have been deserialized
  if (allCommentsDeserialized === false) {
    const comment = deserializeCommentIfNeeded(index);
    if (comment !== null) return comment;
  }

  // Comment was already deserialized
  return cachedComments[index];
}

/**
 * Deserialize comment at `index` if not already deserialized.
 *
 * Caller must ensure `initCommentsBuffer()` has been called before calling this function.
 *
 * @param index - Comment index in the comments buffer
 * @returns `Comment` object if newly deserialized, or `null` if already deserialized
 */
function deserializeCommentIfNeeded(index: number): Comment | null {
  debugAssertIsNonNull(commentsUint8, "Comment buffers should be initialized");
  debugAssertIsNonNull(commentsInt32, "Comment buffers should be initialized");
  debugAssertIsNonNull(sourceText, "Source text should be initialized");

  const pos = index << COMMENT_SIZE_SHIFT;

  // Fast path: If already deserialized, exit
  const flagPos = pos + DESERIALIZED_FLAG_OFFSET;
  if (commentsUint8[flagPos] !== FLAG_NOT_DESERIALIZED) return null;

  // Mark comment as deserialized, so it won't be deserialized again
  commentsUint8[flagPos] = FLAG_DESERIALIZED;

  // Deserialize comment into a cached `Comment` object
  const comment = cachedComments[index];

  // Set `#pos` private property, which is used in getters
  setCommentPos(comment, pos);

  return comment;
}

/**
 * Check all comments in buffer have valid ranges, are in ascending order, and are within the source text.
 *
 * Only runs in debug build (tests). In release build, this function is entirely removed by minifier.
 */
function debugCheckValidRanges(): void {
  if (!DEBUG) return;

  debugAssertIsNonNull(sourceText, "`sourceText` should be initialized");

  let lastEnd = 0;
  for (let i = 0; i < commentsLen; i++) {
    const pos32 = i << 2;
    const start = commentsInt32![pos32];
    const end = commentsInt32![pos32 + 1];
    if (end <= start) throw new Error(`Invalid comment range: ${start}-${end}`);
    if (start < lastEnd) {
      throw new Error(`Overlapping comments: last end: ${lastEnd}, next start: ${start}`);
    }
    lastEnd = end;
  }

  if (lastEnd > sourceText.length) {
    throw new Error(`Comments end beyond source text length: ${lastEnd} > ${sourceText.length}`);
  }
}

/**
 * Check all deserialized comments have valid ranges, are in ascending order, and are within the source text.
 *
 * Only runs in debug build (tests). In release build, this function is entirely removed by minifier.
 */
function debugCheckDeserializedComments(): void {
  if (!DEBUG) return;

  debugAssertIsNonNull(sourceText, "`sourceText` should be initialized");

  let lastEnd = 0;
  for (let i = 0; i < commentsLen; i++) {
    const flagPos = (i << COMMENT_SIZE_SHIFT) + DESERIALIZED_FLAG_OFFSET;
    if (commentsUint8![flagPos] !== FLAG_DESERIALIZED) {
      throw new Error(
        `Comment ${i} not marked as deserialized after \`deserializeComments()\` call`,
      );
    }

    const { start, end } = cachedComments[i];
    if (end <= start) throw new Error(`Invalid deserialized comment range: ${start}-${end}`);
    if (start < lastEnd) {
      throw new Error(
        `Deserialized comments not in order: last end: ${lastEnd}, next start: ${start}`,
      );
    }
    lastEnd = end;
  }

  if (lastEnd > sourceText.length) {
    throw new Error(`Comments end beyond source text length: ${lastEnd} > ${sourceText.length}`);
  }
}

/**
 * Reset comments after file has been linted.
 *
 * Clears cached `loc` on comments that had it accessed, so the getter
 * will recalculate it when the comment is reused for a different file.
 */
export function resetComments(): void {
  // Early exit if comments were never accessed (e.g. no rules used comments-related methods)
  if (commentsInt32 === null) {
    debugAssertAllCommentsCleared();
    return;
  }

  // Reset flag for all comments having been deserialized
  allCommentsDeserialized = false;

  // Reset `#range` on comments where `range` has been accessed
  for (let i = 0; i < activeCommentsWithRangeCount; i++) {
    resetCommentRange(commentsWithRange[i]);
  }

  activeCommentsWithRangeCount = 0;

  // Reset `#loc` on comments where `loc` has been accessed
  for (let i = 0; i < activeCommentsWithLocCount; i++) {
    resetCommentLoc(commentsWithLoc[i]);
  }

  activeCommentsWithLocCount = 0;

  // Reset other state
  comments = null;
  commentsUint8 = null;
  commentsInt32 = null;
  commentsLen = 0;

  debugAssertAllCommentsCleared();
}

/**
 * Check that all comment objects have been cleared.
 *
 * Only runs in debug build (tests). In release build, this function is entirely removed by minifier.
 */
function debugAssertAllCommentsCleared(): void {
  if (!DEBUG) return;

  // Check all cached comments have `#range: null` and `#loc: null`
  for (let i = 0; i < cachedComments.length; i++) {
    const comment = cachedComments[i];
    if (getCommentPrivateRange(comment) !== null) {
      throw new Error(`Comment ${i} has not had \`#range\` cleared`);
    }
    if (getCommentPrivateLoc(comment) !== null) {
      throw new Error(`Comment ${i} has not had \`#loc\` cleared`);
    }
  }
}
