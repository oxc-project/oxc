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

// Tracks indices of deserialized comments so their `value` can be cleared on reset,
// preventing source text strings from being held alive by stale `value` slices.
//
// Pre-allocated in `initCommentsBuffer` to avoid growth during deserialization.
// `Int32Array` rather than `Array` to avoid GC tracing and write barriers.
//
// `deserializedCommentsLen` is the number of deserialized comments in current file.
// If all comments have been deserialized (`allCommentsDeserialized === true`), `deserializedCommentsLen` is 0,
// and no further indexes are written to `deserializedCommentIndexes`. `resetComments` will reset all comments,
// up to `commentsLen`.
let deserializedCommentIndexes = EMPTY_INT32_ARRAY;
let deserializedCommentsLen = 0;

// Minimum capacity (in `u32`s) of `deserializedCommentIndexes`, when not empty.
// 16 elements = 64 bytes = 1 cache line.
const DESERIALIZED_COMMENT_INDEXES_MIN_CAPACITY = 16;

// Empty comments array.
// Reused for all files which don't have any comments. Frozen to avoid rules mutating it.
const EMPTY_COMMENTS: CommentType[] = Object.freeze([]) as unknown as CommentType[];

const COMMENT_SIZE_SHIFT = 4; // 1 << 4 == 16 bytes, the size of `Comment` in Rust
debugAssert(COMMENT_SIZE === 1 << COMMENT_SIZE_SHIFT);

// Reset `#loc` field on a `Comment` class instance.
let resetCommentLoc: (comment: Comment) => void;

// Get `#loc` field on a `Comment` class instance.
// Only used in debug build (tests).
let getCommentPrivateLoc: (comment: Comment) => Location | null;

/**
 * Comment class.
 *
 * Creates `loc` lazily and caches it in a private field.
 * Using a class with a private `#loc` field avoids hidden class transitions that would occur
 * with `Object.defineProperty` / `delete` on plain objects.
 * All `Comment` instances always have the same V8 hidden class, keeping property access monomorphic.
 */
class Comment implements Span {
  type: CommentType["type"] = null!; // Overwritten later
  value: string = null!; // Overwritten later
  start: number = 0;
  end: number = 0;
  range: [number, number] = [0, 0];

  #loc: Location | null = null;

  get loc(): Location {
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
  }

  // Include `loc` in `JSON.stringify` output.
  // `loc` is a prototype getter, and `JSON.stringify` only serializes own properties,
  // so without this method, `loc` would be excluded.
  toJSON() {
    // oxlint-disable-next-line typescript/no-misused-spread
    return { ...this, loc: this.loc };
  }

  static {
    // Defined in static block to avoid exposing this as a public method
    resetCommentLoc = (comment: Comment) => {
      comment.#loc = null;
    };

    if (DEBUG) getCommentPrivateLoc = (comment: Comment) => comment.#loc;
  }
}

// Make `loc` property enumerable so `for (const key in comment) ...` includes `loc`
Object.defineProperty(Comment.prototype, "loc", { enumerable: true });

/**
 * Deserialize all comments and build the `comments` array.
 * Called by `ast.comments` getter.
 */
export function initComments(): void {
  debugAssert(comments === null, "Comments already deserialized");

  if (!allCommentsDeserialized) deserializeComments();

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
  debugAssert(!allCommentsDeserialized, "Comments already deserialized");

  if (commentsInt32 === null) initCommentsBuffer();

  for (let i = 0; i < commentsLen; i++) {
    deserializeCommentIfNeeded(i);
  }

  allCommentsDeserialized = true;
  // No need to count any more, since all comments have been deserialized
  deserializedCommentsLen = 0;

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

    // Grow `deserializedCommentIndexes` if needed.
    // `Int32Array`s can't grow in place, so allocate a new one.
    // First allocation uses minimum capacity. Subsequent growths double, to avoid frequent reallocations.
    const indexesLen = deserializedCommentIndexes.length;
    if (indexesLen < commentsLen) {
      deserializedCommentIndexes = new Int32Array(
        Math.max(
          commentsLen,
          indexesLen === 0 ? DESERIALIZED_COMMENT_INDEXES_MIN_CAPACITY : indexesLen << 1,
        ),
      );
    }
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
  if (!allCommentsDeserialized) {
    const comment = deserializeCommentIfNeeded(index);

    if (comment !== null) {
      // Comment was newly deserialized.
      // Record the comment so its `value` can be cleared on reset, preventing source text strings
      // from being held alive by stale `value` slices.
      // This is in `getComment` rather than `deserializeCommentIfNeeded` so the bulk path
      // (`deserializeComments`) skips the tracking - it uses `allCommentsDeserialized` instead.
      deserializedCommentIndexes[deserializedCommentsLen++] = index;
      return comment;
    }
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
  const pos = index << COMMENT_SIZE_SHIFT;

  // Fast path: If already deserialized, exit
  const flagPos = pos + DESERIALIZED_FLAG_OFFSET;
  if (commentsUint8![flagPos] !== FLAG_NOT_DESERIALIZED) return null;

  // Mark comment as deserialized, so it won't be deserialized again
  commentsUint8![flagPos] = FLAG_DESERIALIZED;

  // Deserialize comment into a cached `Comment` object
  const comment = cachedComments[index];

  const isBlock = commentsUint8![pos + COMMENT_KIND_OFFSET] !== COMMENT_LINE_KIND;

  const pos32 = pos >> 2,
    start = commentsInt32![pos32],
    end = commentsInt32![pos32 + 1];

  comment.type = isBlock ? "Block" : "Line";
  // Line comments: `// text` -> slice `start + 2..end`
  // Block comments: `/* text */` -> slice `start + 2..end - 2`
  comment.value = sourceText!.slice(start + 2, end - (+isBlock << 1));
  comment.range[0] = comment.start = start;
  comment.range[1] = comment.end = end;

  return comment;
}

/**
 * Check comments buffer has valid ranges and ascending order.
 *
 * Only runs in debug build (tests). In release build, this function is entirely removed by minifier.
 */
function debugCheckValidRanges(): void {
  if (!DEBUG) return;

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

  if (lastEnd > sourceText!.length) {
    throw new Error(`Comments end beyond source text length: ${lastEnd} > ${sourceText!.length}`);
  }
}

/**
 * Check all deserialized comments are in ascending order.
 *
 * Only runs in debug build (tests). In release build, this function is entirely removed by minifier.
 */
function debugCheckDeserializedComments(): void {
  if (!DEBUG) return;

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

  // Clear `value` property of deserialized comments to release source text string slices.
  // Without this, V8's SlicedString optimization keeps the entire source text alive
  // as long as any comment's `value` (which is a slice of it) exists.
  if (allCommentsDeserialized === false) {
    // Only a subset of comments have been deserialized, so clear only those
    for (let i = 0; i < deserializedCommentsLen; i++) {
      cachedComments[deserializedCommentIndexes[i]].value = null!;
    }

    deserializedCommentsLen = 0;
  } else {
    // All comments have been deserialized, so clear them all
    for (let i = 0; i < commentsLen; i++) {
      cachedComments[i].value = null!;
    }

    allCommentsDeserialized = false;

    debugAssert(
      deserializedCommentsLen === 0,
      "Deserialized comments counter should have been reset to 0",
    );
  }

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

  // Check all cached tokens have `value: null` and `#loc: null`
  for (let i = 0; i < cachedComments.length; i++) {
    const comment = cachedComments[i];
    if (comment.value !== null) throw new Error(`Comment ${i} has not had \`value\` cleared`);
    if (getCommentPrivateLoc(comment) !== null) {
      throw new Error(`Comment ${i} has not had \`#loc\` cleared`);
    }
  }
}
