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
} from "../generated/constants.ts";
import { computeLoc } from "./location.ts";
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

// Bitmask with a bit set for each `CommentKind` discriminant which is a block comment, shifted left 1 bit.
// `(BLOCK_COMMENT_KINDS_BITMAP >> kind) & 2` then yields the number of characters to subtract from `end`
// when slicing source text to get `value` of a comment - 2 for block comments, 0 for line and hashbang comments.
// Cheaper than a lookup table, which would involve a bounds check and load every time.
const BLOCK_COMMENT_KINDS_BITMAP =
  (0b1111 & ~(1 << COMMENT_LINE_KIND) & ~(1 << COMMENT_SHEBANG_KIND)) << 1;

// Comments for the current file.
// Created lazily only when needed.
export let comments: CommentType[] | null = null;

// Typed array views over the comments region of the buffer.
// These persist for the lifetime of the file (cleared in `resetComments`).
let commentsUint8: Uint8Array | null = null;
export let commentsInt32: Int32Array | null = null;

// Number of comments for the current file.
export let commentsLen = 0;

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

  // `#pos` initialized to `0` so V8 keeps it as an SMI. Constructor overwrites it with the real buffer position.
  #pos: number = 0;
  #range: Range | null = null;
  #loc: Location | null = null;

  constructor(pos: number) {
    this.#pos = pos;

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
        end = commentsInt32[pos32 + 1],
        endSubtract = (BLOCK_COMMENT_KINDS_BITMAP >> kind) & 2;
      return sourceText.slice(start + 2, end - endSubtract);
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

const resetCommentRange = resetCommentRangeTemp;
const resetCommentLoc = resetCommentLocTemp;

/**
 * Build the `comments` array (a slice of `cachedComments`).
 *
 * Unlike `initCommentsArray`, caller does not need to call `initCommentsBuffer()` first.
 *
 * This is used by `ast.comments` getter and `getAllComments` method.
 */
export function initComments(): void {
  debugAssert(comments === null, "Comments already deserialized");

  if (commentsInt32 === null) initCommentsBuffer();
  initCommentsArray();
}

/*
 * Build the `comments` array (a slice of `cachedComments`).
 *
 * Caller must ensure `initCommentsBuffer()` has been called first
 * (so comment buffers and source text are already initialized).
 *
 * Called by `ast.comments` getter.
 */
export function initCommentsArray(): void {
  debugAssert(comments === null, "Comments already deserialized");
  debugAssert(
    commentsInt32 !== null && sourceText !== null,
    "`initCommentsBuffer` must be called before `initComments`",
  );

  // Create `comments` array as a slice of `cachedComments` array.
  //
  // Use `slice` rather than copying comments one-by-one into a new array.
  // V8 implements `slice` with a single `memcpy` of the backing store, which is faster
  // than N individual `push` calls with bounds checking and potential resizing.
  //
  // If the comments array from previous file is longer than the current one,
  // reuse it and truncate it to avoid the memcpy entirely.
  // Assuming random distribution of number of comments in files, this cheaper branch should be hit on 50% of files.
  //
  // Don't touch `previousComments` for files with no comments.
  if (previousComments.length < commentsLen) {
    comments = previousComments = cachedComments.slice(0, commentsLen);
  } else if (commentsLen === 0) {
    comments = EMPTY_COMMENTS;
  } else {
    previousComments.length = commentsLen;
    comments = previousComments;
  }
}

/**
 * Initialize typed array views over the comments region of the buffer.
 *
 * Populates `commentsUint8`, `commentsInt32`, and `commentsLen`, and grows `cachedComments` if needed.
 */
export function initCommentsBuffer(): void {
  debugAssert(
    commentsUint8 === null && commentsInt32 === null,
    "Comments buffer already initialized",
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
    commentsUint8 = EMPTY_UINT8_ARRAY;
    commentsInt32 = EMPTY_INT32_ARRAY;
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
    // Loop on a local `pos` counter rather than calculating `pos = cachedComments.length << COMMENT_SIZE_SHIFT`
    // on each turn of the loop. `Array#push` is not inlined for arrays of objects, so testing `cachedComments.length`
    // in the loop condition would reload `.length` from the heap on every iteration.
    const endPos = commentsLen << COMMENT_SIZE_SHIFT;
    let pos = cachedComments.length << COMMENT_SIZE_SHIFT;
    do {
      cachedComments.push(new Comment(pos));
      // `| 0` truncates the sum to int32, so V8 drops the SMI overflow check on this add.
      // Buffer is limited to 2 GiB, so any valid `pos` is a positive int32, so this is safe.
      pos = (pos + COMMENT_SIZE) | 0;
    } while (pos < endPos);
  }

  // Check comments have valid ranges and are in ascending order
  debugCheckValidRanges();
}

/**
 * Check all comments have valid ranges, are in ascending order, and are within the source text.
 *
 * Only runs in debug build (tests). In release build, this function is entirely removed by minifier.
 */
function debugCheckValidRanges(): void {
  if (!DEBUG) return;

  debugAssertIsNonNull(sourceText, "`sourceText` should be initialized");

  let lastEnd = 0;
  for (let i = 0; i < commentsLen; i++) {
    const { start, end } = cachedComments[i];
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
