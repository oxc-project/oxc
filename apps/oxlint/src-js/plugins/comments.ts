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
import { EMPTY_INT32_ARRAY } from "../utils/typed_arrays.ts";
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
export const BLOCK_COMMENT_KINDS_BITMAP =
  (0b1111 & ~(1 << COMMENT_LINE_KIND) & ~(1 << COMMENT_SHEBANG_KIND)) << 1;

// Mask for bits containing comment kind.
export const COMMENT_KIND_MASK = 3;

// Comments for the current file.
// Created lazily only when needed.
export let comments: CommentType[] | null = null;

// Typed array view over the comments region of the buffer.
// Persists for the lifetime of the file (cleared in `resetComments`).
export let commentsInt32: Int32Array | null = null;

// Number of comments for the current file.
export let commentsLen = 0;

// Cached comment objects, reused across files to reduce GC pressure.
// Comments are mutated in place during deserialization, then `comments` is set to a slice of this array.
export const cachedComments: Comment[] = [];

// Comments array from previous file.
// Reused for next file if next file has fewer comments than the previous file (by truncating to correct length).
let previousComments: Comment[] = [];

// Side tables caching each comment's `range` and `loc`, keyed by comment index (`#pos32 >> COMMENT_SIZE32_SHIFT`).
//
// Each entry stores index into `cachedRanges` / `cachedLocations` pool in bits 0-26 and a gen ID in bits 27-31.
//
// An entry is valid only if its gen ID equals the current gen ID (`commentRangesGenId` / `commentLocsGenId`).
// Gen IDs cycle through 1-31. 0 is reserved - it's the value of a zeroed (or never-written) entry,
// so a zeroed entry can never validate (`resetComments` zeroes all entries).
//
// `range` and `loc` are fully independent (separate gen IDs, and reset), but the two tables always
// have the same length - they're grown together in `initCommentsBuffer`.
let commentRangeIndices = EMPTY_INT32_ARRAY;
let commentRangesGenId = 1;
let commentRangesGenIdShifted = 1 << GEN_ID_SHIFT;
// `true` if any comment `range` was accessed for the current file.
let commentRangesAccessed = false;
// Max `commentsLen` across files in which comment `range`s were accessed, since the table was last reset.
let maxCommentRangesLen = 0;

let commentLocIndices = EMPTY_INT32_ARRAY;
let commentLocsGenId = 1;
let commentLocsGenIdShifted = 1 << GEN_ID_SHIFT;
// `true` if any comment `loc` was accessed for the current file.
let commentLocsAccessed = false;
// Max `commentsLen` across files in which comment `loc`s were accessed, since the table was last reset.
let maxCommentLocsLen = 0;

// Empty comments array.
// Reused for all files which don't have any comments. Frozen to avoid rules mutating it.
const EMPTY_COMMENTS: CommentType[] = Object.freeze([]) as unknown as CommentType[];

const COMMENT_SIZE_SHIFT = 4; // 1 << 4 == 16 bytes, the size of `Comment` in Rust
debugAssert(COMMENT_SIZE === 1 << COMMENT_SIZE_SHIFT);

// Same as `COMMENT_SIZE` / `COMMENT_SIZE_SHIFT`, but in units of `i32`s (for indexing `commentsInt32`).
const COMMENT_SIZE32 = COMMENT_SIZE >> 2;
export const COMMENT_SIZE32_SHIFT = COMMENT_SIZE_SHIFT - 2;
debugAssert(COMMENT_SIZE32 === 1 << COMMENT_SIZE32_SHIFT);

// `kind` is the low byte of the comment's 4th `i32` (byte 12), so a single `commentsInt32` read + a mask
// yields it - no `Uint8Array` view needed. `>> 2` converts the byte offset to an `i32` (word) index.
export const COMMENT_KIND_OFFSET32 = COMMENT_KIND_OFFSET >> 2;
debugAssert(
  (COMMENT_KIND_OFFSET & 3) === 0,
  "`COMMENT_KIND_OFFSET` must be 4-byte aligned to read via `i32`",
);

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

/**
 * Comment class.
 *
 * All properties are defined as own accessor properties via `__defineGetter__` in the constructor,
 * using shared getter functions (e.g. `getCommentType`). This makes them own enumerable
 * properties, so `{...comment}` spreads them and `JSON.stringify(comment)` serializes them.
 *
 * Both `range` and `loc` cache a pool index in a side table (`commentRangeIndices` / `commentLocIndices`),
 * keyed by comment index and validated by gen ID (see the getters), so accessing either twice returns the
 * same object.
 *
 * No value is cached on the instance itself - it holds only `#pos32`.
 *
 * All instances share the same getter functions, keeping the V8 hidden class transition identical across instances.
 * Reset bumps the gen IDs to invalidate both side tables.
 */
class Comment implements Span {
  // All defined with `__defineGetter__` in constructor
  declare type: CommentType["type"];
  declare start: number;
  declare end: number;
  declare range: Range;
  declare loc: Location;
  declare value: string;

  // `#pos32` is the index of the comment's first `i32` in `commentsInt32` (a word index, not a byte offset).
  // Initialized to `0` so V8 keeps it as an SMI. Constructor overwrites it with the real position.
  #pos32: number = 0;

  constructor(pos32: number) {
    this.#pos32 = pos32;

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

  // Functions requiring access to private props defined in static block to avoid exposing them as public methods
  static {
    getCommentTypeTemp = function (this: Comment): CommentType["type"] {
      // This assert can fail in real-world plugin code, and is not a bug here, only incorrect usage in plugin.
      // Only make this an explicit error in debug build, because it should be very uncommon.
      // In release build, it will result in an error in the `commentsInt32` access below.
      debugAssertIsNonNull(
        commentsInt32,
        "`Comment` object's `type` field accessed after file finished linting",
      );

      return COMMENT_TYPES[commentsInt32[this.#pos32 + COMMENT_KIND_OFFSET32] & COMMENT_KIND_MASK];
    };

    getCommentStartTemp = function (this: Comment): number {
      // This assert can fail in real-world plugin code, and is not a bug here, only incorrect usage in plugin.
      // Only make this an explicit error in debug build, because it should be very uncommon.
      // In release build, it will result in an error in the `commentsInt32` access below.
      debugAssertIsNonNull(
        commentsInt32,
        "`Comment` object's `start` field accessed after file finished linting",
      );

      return commentsInt32[this.#pos32];
    };

    getCommentEndTemp = function (this: Comment): number {
      // This assert can fail in real-world plugin code, and is not a bug here, only incorrect usage in plugin.
      // Only make this an explicit error in debug build, because it should be very uncommon.
      // In release build, it will result in an error in the `commentsInt32` access below.
      debugAssertIsNonNull(
        commentsInt32,
        "`Comment` object's `end` field accessed after file finished linting",
      );

      return commentsInt32[this.#pos32 + 1];
    };

    getCommentRangeTemp = function (this: Comment): Range {
      // This assert can fail in real-world plugin code, and is not a bug here, only incorrect usage in plugin.
      // Only make this an explicit error in debug build, because it should be very uncommon.
      // In release build, it will result in an error in the `commentsInt32` access below.
      debugAssertIsNonNull(
        commentsInt32,
        "`Comment` object's `range` field accessed after file finished linting",
      );

      // Check the side table for a `Range` created by an earlier access in this file.
      // Entry layout: `cachedRanges` pool index in bits 0-26, gen ID in bits 27-31.
      // XOR with the current shifted gen ID zeroes the gen bits if (and only if) the entry was written
      // for the current file - what remains is then the pool index itself.
      // Zeroed entries never match - gen IDs cycle through 1-31, and a zeroed entry has gen ID 0.
      const pos32 = this.#pos32;
      const index = pos32 >> COMMENT_SIZE32_SHIFT;
      const rangeIndex = commentRangeIndices[index] ^ commentRangesGenIdShifted;
      if ((rangeIndex & GEN_ID_MASK) === 0) return cachedRanges[rangeIndex];

      // `activeRangesCount` is the index the new `Range` will occupy in `cachedRanges`
      commentRangeIndices[index] = activeRangesCount | commentRangesGenIdShifted;
      commentRangesAccessed = true;
      return createRange(commentsInt32[pos32], commentsInt32[pos32 + 1]);
    };

    getCommentLocTemp = function (this: Comment): Location {
      // This assert can fail in real-world plugin code, and is not a bug here, only incorrect usage in plugin.
      // Only make this an explicit error in debug build, because it should be very uncommon.
      // In release build, it will result in an error in the `commentsInt32` access below.
      debugAssertIsNonNull(
        commentsInt32,
        "`Comment` object's `loc` field accessed after file finished linting",
      );

      // Check the side table for a `Location` created by an earlier access in this file.
      // Entry layout: `cachedLocations` pool index in bits 0-26, gen ID in bits 27-31.
      // XOR with the current shifted gen ID zeroes the gen bits if (and only if) the entry was written
      // for the current file - what remains is then the pool index itself.
      // Zeroed entries never match - gen IDs cycle through 1-31, and a zeroed entry has gen ID 0.
      const pos32 = this.#pos32;
      const index = pos32 >> COMMENT_SIZE32_SHIFT;
      const locIndex = commentLocIndices[index] ^ commentLocsGenIdShifted;
      if ((locIndex & GEN_ID_MASK) === 0) return cachedLocations[locIndex];

      // `activeLocationsCount` is the index the new `Location` will occupy in `cachedLocations`
      commentLocIndices[index] = activeLocationsCount | commentLocsGenIdShifted;
      commentLocsAccessed = true;
      return computeLoc(commentsInt32[pos32], commentsInt32[pos32 + 1]);
    };

    getCommentValueTemp = function (this: Comment): string {
      // This assert can fail in real-world plugin code, and is not a bug here, only incorrect usage in plugin.
      // Only make this an explicit error in debug build, because it should be very uncommon.
      // In release build, it will result in an error in the `commentsInt32` or `sourceText` accesses below.
      debugAssert(
        commentsInt32 !== null && sourceText !== null,
        "`Comment` object's `value` field accessed after file finished linting",
      );

      const pos32 = this.#pos32;

      const kind = commentsInt32[pos32 + COMMENT_KIND_OFFSET32] & COMMENT_KIND_MASK;

      // Line comments: `// text` -> slice `start + 2..end`
      // Block comments: `/* text */` -> slice `start + 2..end - 2`
      // Hashbang: `#! text` -> slice `start + 2..end`
      const start = commentsInt32[pos32],
        end = commentsInt32[pos32 + 1],
        endSubtract = (BLOCK_COMMENT_KINDS_BITMAP >> kind) & 2;
      return sourceText.slice(start + 2, end - endSubtract);
    };
  }
}

// Copied into consts here to avoid checks at call site (`let` binding could be re-assigned)
const getCommentType = getCommentTypeTemp;
const getCommentStart = getCommentStartTemp;
const getCommentEnd = getCommentEndTemp;
const getCommentRange = getCommentRangeTemp;
const getCommentLoc = getCommentLocTemp;
const getCommentValue = getCommentValueTemp;

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
 * Populates `commentsInt32` and `commentsLen`, and grows `cachedComments`
 * and the `commentRangeIndices` / `commentLocIndices` side tables if needed.
 */
export function initCommentsBuffer(): void {
  debugAssert(commentsInt32 === null, "Comments buffer already initialized");

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
    commentsInt32 = EMPTY_INT32_ARRAY;
    return;
  }

  // Create typed array view over the comments region of the buffer.
  // This is a zero-copy view over the same underlying `ArrayBuffer`.
  const arrayBuffer = buffer.buffer,
    absolutePos = buffer.byteOffset + commentsPos;
  commentsInt32 = new Int32Array(arrayBuffer, absolutePos, commentsLen * COMMENT_SIZE32);

  // Grow caches if needed. After first few files, caches should have grown large enough to service all files.
  // Later files will skip this step, and allocations stop.
  if (cachedComments.length < commentsLen) {
    // Loop on a local `pos32` counter rather than calculating `pos32 = cachedComments.length << COMMENT_SIZE32_SHIFT`
    // on each turn of the loop. `Array#push` is not inlined for arrays of objects, so testing `cachedComments.length`
    // in the loop condition would reload `.length` from the heap on every iteration.
    const endPos32 = commentsLen << COMMENT_SIZE32_SHIFT;
    let pos32 = cachedComments.length << COMMENT_SIZE32_SHIFT;
    do {
      cachedComments.push(new Comment(pos32));
      // `| 0` truncates the sum to int32, so V8 drops the SMI overflow check on this add.
      // Buffer is limited to 2 GiB, so any valid `pos32` is a positive int32, so this is safe.
      pos32 = (pos32 + COMMENT_SIZE32) | 0;
    } while (pos32 < endPos32);

    // Grow the `range` and `loc` side tables, so they always have an entry for every comment.
    // The two tables always have the same length - both start as `EMPTY_INT32_ARRAY` and are only ever
    // grown here, together - so a single check and size covers both.
    // `Int32Array`s can't grow in place, so allocate new ones, doubling to amortize growth across files,
    // capped at max `RANGES_AND_LOCS_MAX_COUNT` (the most entries that could ever be required),
    // and minimum `MIN_SIDE_TABLE_SIZE` (to avoid tiny buffers).
    const sideTablesLen = commentRangeIndices.length;
    if (sideTablesLen < commentsLen) {
      const minSize =
        sideTablesLen === 0
          ? MIN_SIDE_TABLE_SIZE
          : Math.min(sideTablesLen * 2, RANGES_AND_LOCS_MAX_COUNT);
      const newSize = Math.max(commentsLen, minSize);
      commentRangeIndices = new Int32Array(newSize);
      commentLocIndices = new Int32Array(newSize);
    }
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
 * Bumps the `range` and `loc` gen IDs to invalidate their side tables, so the getters recompute
 * and draw fresh pool objects when a comment is reused for a different file.
 */
export function resetComments(): void {
  // Early exit if comments were never accessed (e.g. no rules used comments-related methods)
  if (commentsInt32 === null) {
    debugAssertAllCommentsCleared();
    return;
  }

  // If any comment `range`s were accessed for this file, bump the gen ID, which invalidates all entries
  // in `commentRangeIndices` (the `Range`s they point to go back in the pool for reuse by the next file).
  // Files where no comment `range` was accessed leave the gen ID alone - entries can only have been
  // written in files that bumped it, so every live entry's gen ID stays behind the current one.
  // After the gen ID has run through all 31 values, zero the table's used region and start over at 1.
  // `maxCommentRangesLen` bounds the region that can contain entries written since the last zeroing.
  if (commentRangesAccessed === true) {
    if (commentsLen > maxCommentRangesLen) maxCommentRangesLen = commentsLen;

    if (commentRangesGenId === GEN_ID_MAX) {
      commentRangeIndices.fill(0, 0, maxCommentRangesLen);
      commentRangesGenId = 1;
      maxCommentRangesLen = 0;
    } else {
      commentRangesGenId++;
    }

    commentRangesGenIdShifted = commentRangesGenId << GEN_ID_SHIFT;
    commentRangesAccessed = false;
  }

  // Same as the `range` block above, but for `loc`.
  // Bump the independent `loc` gen ID (or wrap and zero the used region at `GEN_ID_MAX`),
  // invalidating all entries in `commentLocIndices`.
  if (commentLocsAccessed === true) {
    if (commentsLen > maxCommentLocsLen) maxCommentLocsLen = commentsLen;

    if (commentLocsGenId === GEN_ID_MAX) {
      commentLocIndices.fill(0, 0, maxCommentLocsLen);
      commentLocsGenId = 1;
      maxCommentLocsLen = 0;
    } else {
      commentLocsGenId++;
    }

    commentLocsGenIdShifted = commentLocsGenId << GEN_ID_SHIFT;
    commentLocsAccessed = false;
  }

  // Reset other state
  comments = null;
  commentsInt32 = null;
  commentsLen = 0;

  debugAssertAllCommentsCleared();
}

/**
 * Check that the comment side tables were invalidated for this file (gen IDs bumped, accessed flags reset).
 *
 * Only runs in debug build (tests). In release build, this function is entirely removed by minifier.
 */
function debugAssertAllCommentsCleared(): void {
  if (!DEBUG) return;

  // Check both comment side tables were invalidated for this file (gen IDs bumped, flags reset)
  if (commentRangesAccessed !== false) {
    throw new Error("`commentRangesAccessed` was not reset");
  }
  if (commentLocsAccessed !== false) {
    throw new Error("`commentLocsAccessed` was not reset");
  }
}
