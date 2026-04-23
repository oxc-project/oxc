/*
 * `SourceCode` methods related to comments.
 */

import {
  cachedComments,
  comments,
  commentsInt32,
  commentsLen,
  getComment,
  initComments,
  initCommentsBuffer,
} from "./comments.ts";
import {
  initTokensAndCommentsBuffer,
  tokensAndCommentsInt32,
  tokensAndCommentsLen,
  MERGED_SIZE32,
  MERGED_SIZE32_SHIFT,
  MERGED_ORIGINAL_INDEX_OFFSET32,
  MERGED_TYPE_OFFSET32,
  MERGED_TYPE_TOKEN,
} from "./tokens_and_comments.ts";
import { firstTokenAtOrAfter } from "./tokens_methods.ts";
import { debugAssert, debugAssertIsNonNull } from "../utils/asserts.ts";

import type { Comment } from "./comments.ts";
import type { Node, NodeOrToken } from "./types.ts";

/**
 * Retrieve an array containing all comments in the source code.
 * @returns Array of `Comment`s in order they appear in source.
 */
export function getAllComments(): Comment[] {
  if (comments === null) initComments();
  debugAssertIsNonNull(comments);
  return comments;
}

debugAssert(MERGED_TYPE_OFFSET32 > 0, "`getCommentsBefore` relies on this");

/**
 * Get all comments directly before the given node or token.
 *
 * "Directly before" means only comments before this node, and after the preceding token.
 *
 * ```js
 * // Define `x`
 * const x = 1;
 * // Define `y`
 * const y = 2;
 * ```
 *
 * `sourceCode.getCommentsBefore(varDeclY)` will only return "Define `y`" comment, not also "Define `x`".
 *
 * @param nodeOrToken - The AST node or token to check for adjacent comment tokens.
 * @returns Array of `Comment`s in occurrence order.
 */
export function getCommentsBefore(nodeOrToken: NodeOrToken): Comment[] {
  if (tokensAndCommentsInt32 === null) initTokensAndCommentsBuffer();
  debugAssertIsNonNull(tokensAndCommentsInt32);

  // Early exit for files with no comments
  if (commentsLen === 0) return [];

  const targetStart = nodeOrToken.range[0];

  // Binary search merged buffer for first entry at or after target's start
  const searchIndex = firstTokenAtOrAfter(
    tokensAndCommentsInt32,
    targetStart,
    0,
    tokensAndCommentsLen,
  );

  // Walk backwards over consecutive comments.
  // Operate in pos32 space: `typePos32` points directly at the type field, decrementing by 4 each step,
  // instead of recomputing `(i << 2) + 2` per iteration.
  const startTypePos32 =
    (searchIndex << MERGED_SIZE32_SHIFT) - (MERGED_SIZE32 - MERGED_TYPE_OFFSET32);
  let typePos32 = startTypePos32;
  // `MERGED_TYPE_OFFSET32` is greater than 0 (checked by debug assert above), so `typePos32 > 0` is right check.
  // If `MERGED_TYPE_OFFSET32` was zero, it'd be `typePos32 >= 0`.
  while (typePos32 > 0 && tokensAndCommentsInt32[typePos32] !== MERGED_TYPE_TOKEN) {
    typePos32 -= MERGED_SIZE32;
  }

  const count32 = startTypePos32 - typePos32;
  if (count32 === 0) return [];

  // Read `originalIndex` of earliest comment, calculate slice end from how far we walked.
  // `typePos32` is at the entry before the first comment.
  const sliceStart =
    tokensAndCommentsInt32[
      typePos32 + (MERGED_SIZE32 - MERGED_TYPE_OFFSET32 + MERGED_ORIGINAL_INDEX_OFFSET32)
    ];
  const sliceEnd = sliceStart + (count32 >> MERGED_SIZE32_SHIFT);

  for (let i = sliceStart; i < sliceEnd; i++) {
    getComment(i);
  }
  return cachedComments.slice(sliceStart, sliceEnd);
}

/**
 * Get all comment tokens directly after the given node or token.
 *
 * "Directly after" means only comments between end of this node, and the next token following it.
 *
 * ```js
 * const x = 1;
 * // Define `y`
 * const y = 2;
 * // Define `z`
 * const z = 3;
 * ```
 *
 * `sourceCode.getCommentsAfter(varDeclX)` will only return "Define `y`" comment, not also "Define `z`".
 *
 * @param nodeOrToken - The AST node or token to check for adjacent comment tokens.
 * @returns Array of `Comment`s in occurrence order.
 */
export function getCommentsAfter(nodeOrToken: NodeOrToken): Comment[] {
  if (tokensAndCommentsInt32 === null) initTokensAndCommentsBuffer();
  debugAssertIsNonNull(tokensAndCommentsInt32);

  // Early exit for files with no comments
  if (commentsLen === 0) return [];

  const targetEnd = nodeOrToken.range[1];

  // Binary search merged buffer for first entry at or after target's end.
  const searchIndex = firstTokenAtOrAfter(
    tokensAndCommentsInt32,
    targetEnd,
    0,
    tokensAndCommentsLen,
  );

  // Walk forwards over consecutive comments.
  // Operate in pos32 space: `typePos32` points directly at the type field, incrementing by 4 each step,
  // instead of recomputing `(i << 2) + 2` per iteration.
  // No explicit bounds check is needed - a sentinel `MERGED_TYPE_TOKEN` entry is written after the last
  // valid entry in `initTokensAndCommentsBuffer`, so the loop terminates naturally.
  const startTypePos32 = (searchIndex << MERGED_SIZE32_SHIFT) + MERGED_TYPE_OFFSET32;
  let typePos32 = startTypePos32;
  while (tokensAndCommentsInt32[typePos32] !== MERGED_TYPE_TOKEN) {
    typePos32 += MERGED_SIZE32;
  }

  const count32 = typePos32 - startTypePos32;
  if (count32 === 0) return [];

  // Read `originalIndex` of earliest comment, calculate slice end from how far we walked
  const sliceStart =
    tokensAndCommentsInt32[
      startTypePos32 - (MERGED_TYPE_OFFSET32 - MERGED_ORIGINAL_INDEX_OFFSET32)
    ];
  const sliceEnd = sliceStart + (count32 >> MERGED_SIZE32_SHIFT);

  for (let i = sliceStart; i < sliceEnd; i++) {
    getComment(i);
  }
  return cachedComments.slice(sliceStart, sliceEnd);
}

/**
 * Get all comment tokens inside the given node.
 * @param node - The AST node to get the comments for.
 * @returns Array of `Comment`s in occurrence order.
 */
export function getCommentsInside(node: Node): Comment[] {
  if (commentsInt32 === null) initCommentsBuffer();
  debugAssertIsNonNull(commentsInt32);

  // Early exit for files with no comments
  if (commentsLen === 0) return [];

  const { range } = node,
    rangeStart = range[0],
    rangeEnd = range[1];

  // Binary search for first comment within `node`'s range
  const sliceStart = firstTokenAtOrAfter(commentsInt32, rangeStart, 0, commentsLen);
  // Binary search for first comment outside `node`'s range.
  // Its index is used as `sliceEnd`, which is exclusive of the slice.
  const sliceEnd = firstTokenAtOrAfter(commentsInt32, rangeEnd, sliceStart, commentsLen);

  // Deserialize only the comments we're returning
  for (let i = sliceStart; i < sliceEnd; i++) {
    getComment(i);
  }
  return cachedComments.slice(sliceStart, sliceEnd);
}

/**
 * Check whether any comments exist or not between the given 2 nodes.
 * @param nodeOrToken1 - Start node/token.
 * @param nodeOrToken2 - End node/token.
 * @returns `true` if one or more comments exist between the two.
 */
export function commentsExistBetween(
  nodeOrToken1: NodeOrToken,
  nodeOrToken2: NodeOrToken,
): boolean {
  if (commentsInt32 === null) initCommentsBuffer();
  debugAssertIsNonNull(commentsInt32);

  // Early exit for files with no comments
  if (commentsLen === 0) return false;

  // Find the first comment after `nodeOrToken1` ends.
  const betweenRangeStart = nodeOrToken1.range[1];
  const firstCommentBetween = firstTokenAtOrAfter(commentsInt32, betweenRangeStart, 0, commentsLen);

  // Check if its end is before `nodeOrToken2` starts.
  // Read `end` from buffer: u32 at offset 1 of the entry.
  return (
    firstCommentBetween < commentsLen &&
    commentsInt32[(firstCommentBetween << 2) + 1] <= nodeOrToken2.range[0]
  );
}

/**
 * Retrieve the JSDoc comment for a given node.
 *
 * @deprecated
 *
 * @param node - The AST node to get the comment for.
 * @returns The JSDoc comment for the given node, or `null` if not found.
 */
/* oxlint-disable no-unused-vars */
export function getJSDocComment(node: Node): Comment | null {
  throw new Error("`sourceCode.getJSDocComment` is not supported at present (and deprecated)"); // TODO
}
/* oxlint-enable no-unused-vars */
