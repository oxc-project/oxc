/**
 * Initialization and deserialization of merged tokens-and-comments array and buffer.
 */

import {
  allCommentsDeserialized,
  cachedComments,
  comments,
  commentsInt32,
  commentsLen,
  deserializeComments,
  getComment,
  initComments,
  initCommentsBuffer,
} from "./comments.ts";
import {
  allTokensDeserialized,
  cachedTokens,
  deserializeTokens,
  initTokens,
  getToken,
  initTokensBuffer,
  tokens,
  tokensLen,
  tokensInt32,
} from "./tokens.ts";
import { COMMENT_SIZE } from "../generated/constants.ts";
import { EMPTY_INT32_ARRAY } from "../utils/typed_arrays.ts";
import { debugAssert, debugAssertIsNonNull } from "../utils/asserts.ts";

import type { Comment } from "./comments.ts";
import type { Token } from "./tokens.ts";

export type TokenOrComment = Token | Comment;

// `tokensAndCommentsInt32` is a buffer containing 16-byte entries,
// representing merged set of all tokens and comments, interleaved in source order.
//
// If they were Rust structs, they would be defined like this:
//
// ```rs
// #[repr(C)]
// struct MergedEntry {
//   /// Start offset of the token/comment in source text
//   start: u32,
//   /// Index of the token/comment within the set of tokens or comments
//   index: u32,
//   /// Is this a token or a comment?
//   type: MergedType,
//   /// (4 bytes padding)
// }
//
// #[repr(u32)]
// enum MergedType {
//   Token = 0,
//   Comment = 1,
// }
// ```
//
// These constants define the shape of the data stored in `tokensAndCommentsInt32` as per the above.
const MERGED_SIZE = 16;
export const MERGED_SIZE32_SHIFT = 2; // 4 x u32s per entry (16 bytes)
export const MERGED_SIZE32 = 1 << MERGED_SIZE32_SHIFT; // 4 x u32s per entry
debugAssert(MERGED_SIZE === MERGED_SIZE32 * 4);
debugAssert(MERGED_SIZE === COMMENT_SIZE, "Size of token, comment, and merged entry must be equal");

export const MERGED_ORIGINAL_INDEX_OFFSET32 = 1; // u32 index of the `original_index` field within an entry
export const MERGED_TYPE_OFFSET32 = 2; // u32 index of the `is_comment` field within an entry

// Type of merged entry.
// "Poor man's enum" which optimizes better than a TS enum.
type MergedType = typeof MERGED_TYPE_TOKEN | typeof MERGED_TYPE_COMMENT;
export const MERGED_TYPE_TOKEN = 0;
const MERGED_TYPE_COMMENT = 1;

// Cached `tokensAndComments` array, returned by `getTokensAndComments`.
// Set to `null` on reset, rebuilt on next access.
let tokensAndComments: TokenOrComment[] | null = null;

// Reusable array for the merged case (when file has both tokens and comments).
// Grows and shrinks as needed. Persists across files to avoid repeated allocation.
let previousTokensAndComments: TokenOrComment[] = [];

// Merged tokens-and-comments buffer (created lazily by `initTokensAndCommentsBuffer`).
// Each entry is 4 x u32s: `{ start, index, is_comment, padding }`.
export let tokensAndCommentsInt32: Int32Array | null = null;

// Number of entries in the tokens-and-comments buffer.
export let tokensAndCommentsLen = 0;

// Backing buffer reused across files.
// Grows when needed (doubled), never shrinks.
// `tokensAndCommentsInt32` is a view over this buffer's prefix.
let tokensAndCommentsBackingInt32 = EMPTY_INT32_ARRAY;

// Minimum capacity (in `u32`s) of `tokensAndCommentsBackingInt32`, when not empty.
// 256 elements = 1 KiB.
const MERGED_BACKING_MIN_CAPACITY = 256;

/**
 * Initialize tokens-and-comments buffer.
 *
 * Creates a buffer containing tokens and comments interleaved in ascending order of `start`.
 *
 * Each token/comment in the input buffers is 16 bytes, with `start` as the first `u32`.
 *
 * `tokensAndCommentsInt32` contains 16-byte entries with the layout:
 * `{ start: u32, index: u32, type: u32, 4 bytes padding }`.
 *
 * `index` is the index of the token/comment within its original buffer (in 16-byte units).
 */
export function initTokensAndCommentsBuffer(): void {
  debugAssert(tokensAndCommentsInt32 === null, "`tokensAndComments` already initialized");

  // Ensure tokens and comments are initialized
  if (tokensInt32 === null) initTokensBuffer();
  debugAssertIsNonNull(tokensInt32);

  if (commentsInt32 === null) initCommentsBuffer();
  debugAssertIsNonNull(commentsInt32);

  tokensAndCommentsLen = tokensLen + commentsLen;

  // Reuse backing buffer across files. Grow if needed, never shrink.
  // After warm-up over first few files, the buffer will be large enough to hold all tokens and comments
  // for all files, so we avoid allocating a large buffer each time.
  // `Int32Array`s can't grow in place, so allocate a new one.
  // First allocation uses minimum capacity. Subsequent growths double, to avoid frequent reallocations.
  // +1 entry for sentinel (see below).
  const requiredLen32 = (tokensAndCommentsLen + 1) << MERGED_SIZE32_SHIFT;
  const backingLen = tokensAndCommentsBackingInt32.length;
  if (backingLen < requiredLen32) {
    tokensAndCommentsBackingInt32 = new Int32Array(
      Math.max(requiredLen32, backingLen === 0 ? MERGED_BACKING_MIN_CAPACITY : backingLen << 1),
    );
  }

  tokensAndCommentsInt32 = tokensAndCommentsBackingInt32;

  // Fast paths for files containing no comments, and no tokens (empty file except for comments)
  if (commentsLen === 0) {
    fillMergedEntries(MERGED_TYPE_TOKEN, tokensInt32, 0, 0, tokensLen);
  } else if (tokensLen === 0) {
    fillMergedEntries(MERGED_TYPE_COMMENT, commentsInt32, 0, 0, commentsLen);
  } else {
    mergeTokensAndComments(tokensInt32, commentsInt32);
  }

  // Write a sentinel `MERGED_TYPE_TOKEN` entry immediately after the last valid entry.
  // This allows `getCommentsAfter`'s forward walk to terminate without an explicit bounds check
  // against `tokensAndCommentsLen` on every iteration - the sentinel acts as a natural stop.
  tokensAndCommentsInt32[(tokensAndCommentsLen << MERGED_SIZE32_SHIFT) + MERGED_TYPE_OFFSET32] =
    MERGED_TYPE_TOKEN;

  debugCheckMergedOrder();
}

/**
 * Merge tokens and comments in ascending order of `start`.
 *
 * Uses two separate inner loops (one for token runs, one for comment runs)
 * so the branch predictor sees a consistent "continue" pattern within each run,
 * only mispredicting once at each run transition.
 */
function mergeTokensAndComments(tokensInt32: Int32Array, commentsInt32: Int32Array): void {
  let tokenIndex = 0,
    commentIndex = 0,
    mergedPos32 = 0;
  let tokenStart = tokensInt32[0],
    commentStart = commentsInt32[0];

  // Push any leading comments
  while (commentStart < tokenStart) {
    writeMergedEntry(MERGED_TYPE_COMMENT, mergedPos32, commentIndex, commentStart);
    mergedPos32 += MERGED_SIZE32;
    if (++commentIndex === commentsLen) {
      fillMergedEntries(MERGED_TYPE_TOKEN, tokensInt32, mergedPos32, tokenIndex, tokensLen);
      return;
    }
    commentStart = commentsInt32[commentIndex << MERGED_SIZE32_SHIFT];
  }

  // Alternate between runs of tokens and runs of comments
  while (true) {
    // Process run of tokens
    do {
      writeMergedEntry(MERGED_TYPE_TOKEN, mergedPos32, tokenIndex, tokenStart);
      mergedPos32 += MERGED_SIZE32;
      if (++tokenIndex === tokensLen) {
        fillMergedEntries(
          MERGED_TYPE_COMMENT,
          commentsInt32,
          mergedPos32,
          commentIndex,
          commentsLen,
        );
        return;
      }
      tokenStart = tokensInt32[tokenIndex << MERGED_SIZE32_SHIFT];
    } while (tokenStart < commentStart);

    // Process run of comments
    do {
      writeMergedEntry(MERGED_TYPE_COMMENT, mergedPos32, commentIndex, commentStart);
      mergedPos32 += MERGED_SIZE32;
      if (++commentIndex === commentsLen) {
        fillMergedEntries(MERGED_TYPE_TOKEN, tokensInt32, mergedPos32, tokenIndex, tokensLen);
        return;
      }
      commentStart = commentsInt32[commentIndex << MERGED_SIZE32_SHIFT];
    } while (commentStart < tokenStart);
  }
}

/**
 * Check that merged entries are in ascending order of `start`.
 *
 * Only runs in debug build (tests). In release build, this function is entirely removed by minifier.
 */
function debugCheckMergedOrder(): void {
  if (!DEBUG) return;

  let lastStart = -1;
  for (let i = 0; i < tokensAndCommentsLen; i++) {
    const start = tokensAndCommentsInt32![i << MERGED_SIZE32_SHIFT];
    if (start <= lastStart) {
      throw new Error(
        `Merged tokens/comments not in order: entry ${i} start ${start} <= previous start ${lastStart}`,
      );
    }
    lastStart = start;
  }
}

/**
 * Write a single entry to the merged buffer.
 */
function writeMergedEntry(
  type: MergedType,
  mergedPos32: number,
  originalIndex: number,
  start: number,
): void {
  tokensAndCommentsInt32![mergedPos32] = start;
  tokensAndCommentsInt32![mergedPos32 + MERGED_ORIGINAL_INDEX_OFFSET32] = originalIndex;
  tokensAndCommentsInt32![mergedPos32 + MERGED_TYPE_OFFSET32] = type;
  // out[outPos + 3] is padding, no need to write it
}

/**
 * Fill output entries from a single source buffer (tokens or comments) sequentially.
 * Used for fast paths and for appending remaining items after the merge loop.
 */
function fillMergedEntries(
  type: MergedType,
  srcInt32: Int32Array,
  mergedPos32: number,
  srcIndex: number,
  srcLen: number,
): void {
  let srcPos32 = srcIndex << MERGED_SIZE32_SHIFT;

  for (; srcIndex < srcLen; srcIndex++) {
    tokensAndCommentsInt32![mergedPos32] = srcInt32[srcPos32];
    tokensAndCommentsInt32![mergedPos32 + MERGED_ORIGINAL_INDEX_OFFSET32] = srcIndex;
    tokensAndCommentsInt32![mergedPos32 + MERGED_TYPE_OFFSET32] = type;
    mergedPos32 += MERGED_SIZE32;
    srcPos32 += MERGED_SIZE32;
  }
}

/**
 * Get token or comment from the merged buffer at `index`.
 * Deserializes the underlying token/comment if needed.
 *
 * @param index - Index in the merged buffer
 * @returns Deserialized token or comment
 */
export function getTokenOrComment(index: number): TokenOrComment {
  const pos32 = index << MERGED_SIZE32_SHIFT;
  const originalIndex = tokensAndCommentsInt32![pos32 + MERGED_ORIGINAL_INDEX_OFFSET32];

  return tokensAndCommentsInt32![pos32 + MERGED_TYPE_OFFSET32] === MERGED_TYPE_TOKEN
    ? getToken(originalIndex)
    : getComment(originalIndex);
}

/**
 * Get the `end` value for an entry in the merged `tokensAndComments` buffer,
 * by looking it up in the original tokens or comments buffer.
 *
 * @param entryIndex - Index in the merged buffer
 * @returns The `end` offset of the token/comment in source text
 */
export function getTokenOrCommentEnd(entryIndex: number): number {
  const pos32 = entryIndex << MERGED_SIZE32_SHIFT;
  const originalIndex = tokensAndCommentsInt32![pos32 + MERGED_ORIGINAL_INDEX_OFFSET32];
  const originalEndPos32 = (originalIndex << MERGED_SIZE32_SHIFT) + 1;

  return tokensAndCommentsInt32![pos32 + MERGED_TYPE_OFFSET32] === MERGED_TYPE_TOKEN
    ? tokensInt32![originalEndPos32]
    : commentsInt32![originalEndPos32];
}

/**
 * Get all tokens and comments in source order.
 *
 * Builds and caches the merged array on first call.
 * Subsequent calls return the same cached array.
 *
 * @returns Array of all tokens and comments, sorted by source position
 */
export function getTokensAndComments(): TokenOrComment[] {
  // If `getTokensAndComments` has already been called, return same array again
  if (tokensAndComments !== null) return tokensAndComments;

  // Init tokens and comments (to get lengths), but don't build `tokens`/`comments` arrays yet
  if (tokensInt32 === null) initTokensBuffer();
  if (commentsInt32 === null) initCommentsBuffer();

  // Fast path: No comments - build `tokens` array and return it directly
  if (commentsLen === 0) {
    if (tokens === null) initTokens();
    debugAssertIsNonNull(tokens);
    return (tokensAndComments = tokens);
  }

  // Fast path: No tokens - build `comments` array and return it directly
  if (tokensLen === 0) {
    if (comments === null) initComments();
    debugAssertIsNonNull(comments);
    return (tokensAndComments = comments);
  }

  // General case: Deserialize all entries into `cachedTokens` / `cachedComments`,
  // but skip building the `tokens` and `comments` arrays (they aren't needed here)
  if (!allTokensDeserialized) deserializeTokens();
  if (!allCommentsDeserialized) deserializeComments();

  // Ensure merged buffer is built
  if (tokensAndCommentsInt32 === null) initTokensAndCommentsBuffer();
  debugAssertIsNonNull(tokensAndCommentsInt32);

  // Since `deserializeTokens` and `deserializeComments` are called first, all entries are already deserialized.
  // We just need to look up entries from `cachedTokens` and `cachedComments` by index.
  //
  // Reuse the array from the previous file to avoid allocation:
  // - If big enough: Truncate to target length (V8 shrinks FixedArray in place, no realloc),
  //   then overwrite existing slots.
  // - If too small: `new Array(n).fill(0)` to pre-allocate exact capacity as a PACKED array,
  //   then overwrite. This avoids copying stale elements (which growing the old array would do).
  if (previousTokensAndComments.length >= tokensAndCommentsLen) {
    tokensAndComments = previousTokensAndComments;
    tokensAndComments.length = tokensAndCommentsLen;
  } else {
    // `new Array(n)` creates a HOLEY array. `.fill(0)` fills all slots, making it PACKED.
    // `0` is used because SMI 0 is all-zero bits, so V8 can use `memset(0)` which is ~10% faster
    // than filling with a non-zero value like `null` (CPUs have dedicated fast-zeroing support).
    // Write `null` into first entry to transition array to PACKED_ELEMENTS.
    // oxlint-disable-next-line unicorn/no-new-array -- `Array.from` is 12x slower (benchmarked)
    tokensAndComments = previousTokensAndComments = new Array(tokensAndCommentsLen).fill(0);
    tokensAndComments[0] = null!;
  }

  for (let i = 0; i < tokensAndCommentsLen; i++) {
    const pos32 = i << MERGED_SIZE32_SHIFT;
    const originalIndex = tokensAndCommentsInt32[pos32 + MERGED_ORIGINAL_INDEX_OFFSET32];
    tokensAndComments[i] =
      tokensAndCommentsInt32[pos32 + MERGED_TYPE_OFFSET32] === MERGED_TYPE_TOKEN
        ? (cachedTokens[originalIndex] as Token)
        : cachedComments[originalIndex];
  }

  return tokensAndComments;
}

/**
 * Reset merged tokens-and-comments array and buffer after file has been linted.
 */
export function resetTokensAndComments() {
  tokensAndComments = null;
  tokensAndCommentsInt32 = null;
  tokensAndCommentsLen = 0;
}
