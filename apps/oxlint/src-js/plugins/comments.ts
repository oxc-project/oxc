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
} from "../generated/constants.ts";
import { computeLoc } from "./location.ts";
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

// Cached comment objects, reused across files to reduce GC pressure.
// Comments are mutated in place during deserialization, then `comments` is set to a slice of this array.
const cachedComments: Comment[] = [];

// Comments array from previous file.
// Reused for next file if next file has fewer comments than the previous file (by truncating to correct length).
let previousComments: Comment[] = [];

// Comments whose `loc` property has been accessed, and therefore needs clearing on reset.
const commentsWithLoc: Comment[] = [];

// Empty comments array.
// Reused for all files which don't have any comments. Frozen to avoid rules mutating it.
const emptyComments: CommentType[] = Object.freeze([]) as unknown as CommentType[];

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
 * Initialize comments for current file.
 *
 * Deserializes comments from the buffer using object pooling.
 * If the program has a hashbang, sets first comment to a `Shebang` comment.
 */
export function initComments(): void {
  debugAssert(comments === null, "Comments already initialized");

  // We don't need to deserialize source text if there are no comments, so we could move this call to after
  // the `commentsLen === 0` check. However, various comments methods rely on that if `initComments` has been called,
  // then `sourceText` is initialized. Doing it eagerly here avoids having to check if `sourceText` is `null`
  // in all those methods, which can be called quite frequently.
  if (sourceText === null) initSourceText();
  debugAssertIsNonNull(sourceText);
  debugAssertIsNonNull(buffer);

  const { uint32 } = buffer;
  const programPos32 = uint32[DATA_POINTER_POS_32] >> 2;
  const commentsPos = uint32[programPos32 + (COMMENTS_OFFSET >> 2)];
  const commentsLen = uint32[programPos32 + (COMMENTS_LEN_OFFSET >> 2)];

  // Fast path for files with no comments
  if (commentsLen === 0) {
    comments = emptyComments;
    return;
  }

  // Grow cache if needed (one-time cost as cache warms up)
  while (cachedComments.length < commentsLen) {
    cachedComments.push(new Comment());
  }

  // Deserialize comments from buffer
  for (let i = 0; i < commentsLen; i++) {
    const comment = cachedComments[i];

    const pos = commentsPos + i * COMMENT_SIZE,
      pos32 = pos >> 2;

    const start = uint32[pos32];
    const end = uint32[pos32 + 1];
    const isBlock = buffer[pos + COMMENT_KIND_OFFSET] !== COMMENT_LINE_KIND;

    comment.type = isBlock ? "Block" : "Line";
    // Line comments: `// text` -> slice `start + 2..end`
    // Block comments: `/* text */` -> slice `start + 2..end - 2`
    comment.value = sourceText.slice(start + 2, end - (+isBlock << 1));
    comment.range[0] = comment.start = start;
    comment.range[1] = comment.end = end;
  }

  // Set first comment as `Shebang` if file has hashbang.
  // Rust side adds hashbang comment to start of comments `Vec` as a `Line` comment.
  // `uint32[commentsPos >> 2]` is the start of the first comment.
  if (uint32[commentsPos >> 2] === 0 && sourceText.startsWith("#!")) {
    cachedComments[0].type = "Shebang";
  }

  // Use `slice` rather than copying comments one-by-one into a new array.
  // V8 implements `slice` with a single `memcpy` of the backing store, which is faster
  // than N individual `push` calls with bounds checking and potential resizing.
  //
  // If the comments array from previous file is longer than the current one,
  // reuse it and truncate it to avoid the memcpy entirely.
  if (previousComments.length >= commentsLen) {
    previousComments.length = commentsLen;
    comments = previousComments;
  } else {
    comments = previousComments = cachedComments.slice(0, commentsLen);
  }

  // Check `comments` have valid ranges and are in ascending order
  debugCheckValidRanges(comments);
}

/**
 * Check comments have valid ranges and are in ascending order.
 *
 * Only runs in debug build (tests). In release build, this function is entirely removed by minifier.
 */
function debugCheckValidRanges(commentsArr: CommentType[]): void {
  if (!DEBUG) return;

  let lastEnd = 0;
  for (const comment of commentsArr) {
    const { start, end } = comment;
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
}
