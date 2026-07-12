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
  DATA_POINTER_POS_32,
  DESERIALIZED_FLAG_OFFSET,
} from "../generated/constants.ts";
import { computeLoc } from "./location.ts";
import { FLAG_NOT_DESERIALIZED, FLAG_DESERIALIZED } from "./tokens.ts";
import { EMPTY_UINT8_ARRAY, EMPTY_INT32_ARRAY } from "../utils/typed_arrays.ts";
import { debugAssert, debugAssertIsNonNull } from "../utils/asserts.ts";

import type { Location, Span } from "./location.ts";

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
const COMMENT_TYPES: CommentType["type"][] = ["Block", "Block", "Block"];
COMMENT_TYPES[COMMENT_LINE_KIND] = "Line";

// Array of numbers to subtract from `end` when slicing source text to get `value` of a comment,
// indexed by `CommentKind` discriminant.
const COMMENT_END_SUBTRACTIONS: number[] = [2, 2, 2];
COMMENT_END_SUBTRACTIONS[COMMENT_LINE_KIND] = 0;

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

// Getter for the `value` property on a `Comment` class instance.
// Copied into a `const` below after being defined in class static block.
let getCommentValueTemp: (this: Comment) => string;

// Getter for the `loc` property on a `Comment` class instance.
// Copied into a `const` below after being defined in class static block.
let getCommentLocTemp: (this: Comment) => Location;

// Setter for `#pos` private property on a `Comment` class instance.
// Copied into a `const` below after being defined in class static block.
let setCommentPosTemp: (comment: Comment, pos: number) => void;

// Reset `#loc` field on a `Comment` class instance.
// Copied into a `const` below after being defined in class static block.
let resetCommentLocTemp: (comment: Comment) => void;

// Get `#loc` field on a `Comment` class instance.
// Only used in debug build (tests).
let getCommentPrivateLoc: (comment: Comment) => Location | null;

/**
 * Comment class.
 *
 * `loc` is defined as an own accessor property via `__defineGetter__` in the constructor,
 * using a shared getter function (`getCommentLoc`). This makes `loc` an own enumerable property,
 * so `{...comment}` spreads it and `JSON.stringify(comment)` serializes it.
 *
 * The computed `Location` value is cached in the private `#loc` field on first access.
 * All instances share the same getter function, keeping the V8 hidden class transition
 * identical across instances. Reset only clears the `#loc` field.
 */
class Comment implements Span {
  type: CommentType["type"] = null!; // Overwritten later
  start: number = 0;
  end: number = 0;
  range: [number, number] = [0, 0];

  declare value: string; // Defined with `__defineGetter__` in constructor
  declare loc: Location; // Defined with `__defineGetter__` in constructor

  #pos: number = 0;
  #loc: Location | null = null;

  constructor() {
    // Define `value` and `loc` as own getter properties (enumerable + configurable by default).
    // This makes `{...comment}` spread `value` and `loc`, and `JSON.stringify(comment)` serialize them.
    // Note: `new Comment()` is 25% faster with `__defineGetter__` vs `Object.defineProperty`.
    // See https://github.com/oxc-project/oxc/pull/22238.
    defineGetter(this, "value", getCommentValue);
    defineGetter(this, "loc", getCommentLoc);
  }

  // Functions requiring access to `#pos` or `#loc` defined in static block to avoid exposing them as public methods
  static {
    getCommentValueTemp = function (this: Comment): string {
      // This assert can fail in real-world plugin code, and is not a bug here, only incorrect usage in plugin.
      // Only make this an explicit error in debug build, because it should be very uncommon.
      // In release build, it will result in an error in `commentsUint8` or `sourceText` accesses below.
      debugAssert(
        commentsUint8 !== null && sourceText !== null,
        "`Comment` object's `value` field accessed after file finished linting",
      );

      const kind = commentsUint8[this.#pos + COMMENT_KIND_OFFSET];

      // Line comments: `// text` -> slice `start + 2..end`
      // Block comments: `/* text */` -> slice `start + 2..end - 2`
      return sourceText.slice(this.start + 2, this.end - COMMENT_END_SUBTRACTIONS[kind]);
    };

    getCommentLocTemp = function (this: Comment): Location {
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

      return (this.#loc = computeLoc(this.start, this.end));
    };

    setCommentPosTemp = function (comment: Comment, pos: number) {
      comment.#pos = pos;
    };

    resetCommentLocTemp = (comment: Comment) => {
      comment.#loc = null;
    };

    if (DEBUG) getCommentPrivateLoc = (comment: Comment) => comment.#loc;
  }
}

// Copied into consts here to avoid checks at call site (`let` binding could be re-assigned)
const getCommentValue = getCommentValueTemp;
const getCommentLoc = getCommentLocTemp;
const setCommentPos = setCommentPosTemp;
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
 *
 * Exception: If the file has a hashbang, eagerly deserializes the first comment and sets its type to `Shebang`.
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

  // If file has a hashbang, eagerly deserialize the first comment, and set its type to `Shebang`.
  // We do this here instead of lazily when comment 0 is deserialized, to remove code
  // from `deserializeCommentIfNeeded`, which can be called many times.
  // Rust side adds hashbang comment to start of comments `Vec` as a `Line` comment.
  // `commentsInt32[0]` is the start of the first comment.
  if (commentsInt32[0] === 0 && sourceText.startsWith("#!")) {
    getComment(0).type = "Shebang";
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

  // Set `#pos` private property, which is used in `value` getter
  setCommentPos(comment, pos);

  const kind = commentsUint8[pos + COMMENT_KIND_OFFSET];
  comment.type = COMMENT_TYPES[kind];

  const pos32 = pos >> 2;
  comment.range[0] = comment.start = commentsInt32[pos32];
  comment.range[1] = comment.end = commentsInt32[pos32 + 1];

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

  // Check all cached comments have `#loc: null`
  for (let i = 0; i < cachedComments.length; i++) {
    const comment = cachedComments[i];
    if (getCommentPrivateLoc(comment) !== null) {
      throw new Error(`Comment ${i} has not had \`#loc\` cleared`);
    }
  }
}
