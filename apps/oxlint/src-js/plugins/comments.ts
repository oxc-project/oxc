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
export let commentsUint32: Uint32Array | null = null;

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
const commentsWithLoc: Comment[] = [];

// Empty comments array.
// Reused for all files which don't have any comments. Frozen to avoid rules mutating it.
const EMPTY_COMMENTS: CommentType[] = Object.freeze([]) as unknown as CommentType[];

// Empty typed arrays, reused for files with no comments.
const EMPTY_UINT8_ARRAY = new Uint8Array(0);
const EMPTY_UINT32_ARRAY = new Uint32Array(0);

const COMMENT_SIZE_SHIFT = 4; // 1 << 4 == 16 bytes, the size of `Comment` in Rust
debugAssert(COMMENT_SIZE === 1 << COMMENT_SIZE_SHIFT);

// Reset `#loc` field on a `Comment` class instance.
let resetCommentLoc: (comment: Comment) => void;

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

    commentsWithLoc.push(this);
    return (this.#loc = computeLoc(this.start, this.end));
  }

  static {
    // Defined in static block to avoid exposing this as a public method
    resetCommentLoc = (comment: Comment) => {
      comment.#loc = null;
    };
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

  if (commentsUint32 === null) initCommentsBuffer();

  for (let i = 0; i < commentsLen; i++) {
    deserializeCommentIfNeeded(i);
  }

  allCommentsDeserialized = true;

  debugCheckDeserializedComments();
}

/**
 * Initialize typed array views over the comments region of the buffer.
 *
 * Populates `commentsUint8`, `commentsUint32`, and `commentsLen`, and grows `cachedComments` if needed.
 * Does NOT deserialize comments - they are deserialized lazily via `deserializeCommentIfNeeded`.
 *
 * Exception: If the file has a hashbang, eagerly deserializes the first comment and sets its type to `Shebang`.
 */
export function initCommentsBuffer(): void {
  debugAssert(
    commentsUint8 === null && commentsUint32 === null,
    "Comments buffer already initialized",
  );

  debugAssertIsNonNull(buffer);

  // We don't need to deserialize source text if there are no comments, so we could move this call to after
  // the `commentsLen === 0` check. However, various comments methods rely on that if `initComments` has been called,
  // then `sourceText` is initialized. Doing it eagerly here avoids having to check if `sourceText` is `null`
  // in all those methods, which can be called quite frequently.
  if (sourceText === null) initSourceText();
  debugAssertIsNonNull(sourceText);

  const { uint32 } = buffer;
  const programPos32 = uint32[DATA_POINTER_POS_32] >> 2;
  const commentsPos = uint32[programPos32 + (COMMENTS_OFFSET >> 2)];
  commentsLen = uint32[programPos32 + (COMMENTS_LEN_OFFSET >> 2)];

  // Fast path for files with no comments
  if (commentsLen === 0) {
    comments = EMPTY_COMMENTS;
    commentsUint8 = EMPTY_UINT8_ARRAY;
    commentsUint32 = EMPTY_UINT32_ARRAY;
    allCommentsDeserialized = true;
    return;
  }

  // Create typed array views over the comments region of the buffer.
  // These are zero-copy views over the same underlying `ArrayBuffer`.
  const arrayBuffer = buffer.buffer,
    absolutePos = buffer.byteOffset + commentsPos;
  commentsUint8 = new Uint8Array(arrayBuffer, absolutePos, commentsLen * COMMENT_SIZE);
  commentsUint32 = new Uint32Array(arrayBuffer, absolutePos, commentsLen * (COMMENT_SIZE >> 2));

  // Grow cache if needed (one-time cost as cache warms up)
  while (cachedComments.length < commentsLen) {
    cachedComments.push(new Comment());
  }

  // If file has a hashbang, eagerly deserialize the first comment, and set its type to `Shebang`.
  // We do this here instead of lazily when comment 0 is deserialized, to remove code
  // from `deserializeCommentIfNeeded`, which can be called many times.
  // Rust side adds hashbang comment to start of comments `Vec` as a `Line` comment.
  // `commentsUint32[0]` is the start of the first comment.
  if (commentsUint32[0] === 0 && sourceText.startsWith("#!")) {
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
  const comment = deserializeCommentIfNeeded(index);
  return comment === null ? cachedComments[index] : comment;
}

/**
 * Deserialize comment at `index` if not already deserialized.
 *
 * Caller must ensure `initCommentsBuffer()` has been called before calling this function.
 *
 * @param index - Comment index in the comments buffer
 * @returns `Comment` object if newly deserialized, or `null` if already deserialized
 */
export function deserializeCommentIfNeeded(index: number): Comment | null {
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
    start = commentsUint32![pos32],
    end = commentsUint32![pos32 + 1];

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
    const start = commentsUint32![pos32];
    const end = commentsUint32![pos32 + 1];
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
  for (let i = 0, len = commentsWithLoc.length; i < len; i++) {
    resetCommentLoc(commentsWithLoc[i]);
  }
  commentsWithLoc.length = 0;

  comments = null;
  commentsUint8 = null;
  commentsUint32 = null;
  commentsLen = 0;
  allCommentsDeserialized = false;
}
