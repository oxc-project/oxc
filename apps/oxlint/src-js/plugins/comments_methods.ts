/*
 * `SourceCode` methods related to comments.
 */

import {
  cachedComments,
  comments,
  commentsUint32,
  commentsLen,
  initComments,
  initCommentsBuffer,
  deserializeCommentIfNeeded,
} from "./comments.ts";
import { sourceText } from "./source_code.ts";
import { firstTokenAtOrAfter } from "./tokens_methods.ts";
import { debugAssertIsNonNull } from "../utils/asserts.ts";

import type { Comment } from "./comments.ts";
import type { Node, NodeOrToken } from "./types.ts";

// Regex that tests if a string is entirely whitespace.
const WHITESPACE_ONLY_REGEXP = /^\s*$/;

/**
 * Retrieve an array containing all comments in the source code.
 * @returns Array of `Comment`s in order they appear in source.
 */
export function getAllComments(): Comment[] {
  if (comments === null) initComments();
  debugAssertIsNonNull(comments);
  return comments;
}

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
  if (commentsUint32 === null) initCommentsBuffer();
  debugAssertIsNonNull(commentsUint32);
  debugAssertIsNonNull(sourceText);

  // Early exit for files with no comments
  if (commentsLen === 0) return [];

  let targetStart = nodeOrToken.range[0]; // start

  // Binary search for first comment at or past `nodeOrToken`'s start.
  // Comments before this index are candidates to be included in returned array.
  const sliceEnd = firstTokenAtOrAfter(commentsUint32, targetStart, 0, commentsLen);

  let sliceStart = commentsLen;
  for (let i = sliceEnd - 1; i >= 0; i--) {
    // Read `end` from buffer: u32 at offset 1 of each 4 x u32 entry
    const commentEnd = commentsUint32[(i << 2) + 1];
    const gap = sourceText.slice(commentEnd, targetStart);
    // Ensure that there is nothing except whitespace between the end of the
    // current comment and the start of the next one as we iterate backwards
    if (WHITESPACE_ONLY_REGEXP.test(gap)) {
      sliceStart = i;
      // Read `start` from buffer
      targetStart = commentsUint32[i << 2];
    } else {
      break;
    }
  }

  // Deserialize only the comments we're returning
  for (let i = sliceStart; i < sliceEnd; i++) {
    deserializeCommentIfNeeded(i);
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
  if (commentsUint32 === null) initCommentsBuffer();
  debugAssertIsNonNull(commentsUint32);
  debugAssertIsNonNull(sourceText);

  // Early exit for files with no comments
  if (commentsLen === 0) return [];

  let targetEnd = nodeOrToken.range[1]; // end

  // Binary search for first comment at or past `nodeOrToken`'s end.
  // Comments from this index onwards are candidates to be included in returned array.
  const sliceStart = firstTokenAtOrAfter(commentsUint32, targetEnd, 0, commentsLen);

  let sliceEnd = 0;
  for (let i = sliceStart; i < commentsLen; i++) {
    // Ensure that there is nothing except whitespace between the
    // end of the previous comment and the start of the current one
    const commentStart = commentsUint32[i << 2];
    const gap = sourceText.slice(targetEnd, commentStart);
    if (WHITESPACE_ONLY_REGEXP.test(gap)) {
      sliceEnd = i + 1;
      // Read `end` from buffer
      targetEnd = commentsUint32[(i << 2) + 1];
    } else {
      break;
    }
  }

  // Deserialize only the comments we're returning
  for (let i = sliceStart; i < sliceEnd; i++) {
    deserializeCommentIfNeeded(i);
  }
  return cachedComments.slice(sliceStart, sliceEnd);
}

/**
 * Get all comment tokens inside the given node.
 * @param node - The AST node to get the comments for.
 * @returns Array of `Comment`s in occurrence order.
 */
export function getCommentsInside(node: Node): Comment[] {
  if (commentsUint32 === null) initCommentsBuffer();
  debugAssertIsNonNull(commentsUint32);

  // Early exit for files with no comments
  if (commentsLen === 0) return [];

  const { range } = node,
    rangeStart = range[0],
    rangeEnd = range[1];

  // Binary search for first comment within `node`'s range
  const sliceStart = firstTokenAtOrAfter(commentsUint32, rangeStart, 0, commentsLen);
  // Binary search for first comment outside `node`'s range.
  // Its index is used as `sliceEnd`, which is exclusive of the slice.
  const sliceEnd = firstTokenAtOrAfter(commentsUint32, rangeEnd, sliceStart, commentsLen);

  // Deserialize only the comments we're returning
  for (let i = sliceStart; i < sliceEnd; i++) {
    deserializeCommentIfNeeded(i);
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
  if (commentsUint32 === null) initCommentsBuffer();
  debugAssertIsNonNull(commentsUint32);

  // Early exit for files with no comments
  if (commentsLen === 0) return false;

  // Find the first comment after `nodeOrToken1` ends.
  const betweenRangeStart = nodeOrToken1.range[1];
  const firstCommentBetween = firstTokenAtOrAfter(
    commentsUint32,
    betweenRangeStart,
    0,
    commentsLen,
  );

  // Check if its end is before `nodeOrToken2` starts.
  // Read `end` from buffer: u32 at offset 1 of the entry.
  return (
    firstCommentBetween < commentsLen &&
    commentsUint32[(firstCommentBetween << 2) + 1] <= nodeOrToken2.range[0]
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
