/*
 * `SourceCode` methods related to comments.
 */

import { ast, initAst, sourceText } from "./source_code.ts";
import { debugAssertIsNonNull } from "../utils/asserts.ts";

import type { Comment, Node, NodeOrToken } from "./types.ts";

// Regex that tests if a string is entirely whitespace.
const WHITESPACE_ONLY_REGEXP = /^\s*$/;

/**
 * Retrieve an array containing all comments in the source code.
 * @returns Array of `Comment`s in order they appear in source.
 */
export function getAllComments(): Comment[] {
  if (ast === null) initAst();
  debugAssertIsNonNull(ast);

  // `comments` property is a getter. Comments are deserialized lazily.
  return ast.comments;
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
  if (ast === null) initAst();
  debugAssertIsNonNull(ast);
  debugAssertIsNonNull(sourceText);

  const { comments } = ast,
    commentsLength = comments.length;

  let targetStart = nodeOrToken.range[0]; // start

  let sliceStart = commentsLength;
  let sliceEnd = 0;

  // Binary search for the comment immediately before `nodeOrToken`.
  for (let lo = 0, hi = commentsLength; lo < hi; ) {
    const mid = (lo + hi) >> 1;
    if (comments[mid].end <= targetStart) {
      sliceEnd = lo = mid + 1;
    } else {
      hi = mid;
    }
  }

  for (let i = sliceEnd - 1; i >= 0; i--) {
    const comment = comments[i];
    const gap = sourceText.slice(comment.end, targetStart);
    // Ensure that there is nothing except whitespace between the end of the
    // current comment and the start of the next as we iterate backwards.
    if (WHITESPACE_ONLY_REGEXP.test(gap)) {
      sliceStart = i;
      targetStart = comment.start;
    } else {
      break;
    }
  }

  return comments.slice(sliceStart, sliceEnd);
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
  if (ast === null) initAst();
  debugAssertIsNonNull(ast);
  debugAssertIsNonNull(sourceText);

  const { comments } = ast,
    commentsLength = comments.length;

  let targetEnd = nodeOrToken.range[1]; // end

  let sliceStart = commentsLength;
  let sliceEnd = 0;

  // Binary search for the comment immediately after `nodeOrToken`.
  for (let lo = 0, hi = commentsLength; lo < hi; ) {
    const mid = (lo + hi) >> 1;
    if (comments[mid].start < targetEnd) {
      lo = mid + 1;
    } else {
      sliceStart = hi = mid;
    }
  }

  for (let i = sliceStart; i < commentsLength; i++) {
    // Ensure that there is nothing except whitespace between the
    // end of the previous comment and the start of the current.
    const comment = comments[i];
    const gap = sourceText.slice(targetEnd, comment.start);
    if (WHITESPACE_ONLY_REGEXP.test(gap)) {
      sliceEnd = i + 1;
      targetEnd = comment.end;
    } else {
      break;
    }
  }

  return comments.slice(sliceStart, sliceEnd);
}

/**
 * Get all comment tokens inside the given node.
 * @param node - The AST node to get the comments for.
 * @returns Array of `Comment`s in occurrence order.
 */
export function getCommentsInside(node: Node): Comment[] {
  if (ast === null) initAst();
  debugAssertIsNonNull(ast);

  const { comments } = ast,
    commentsLength = comments.length;

  let sliceStart = commentsLength;
  let sliceEnd: number | undefined = undefined;

  const { range } = node,
    rangeStart = range[0],
    rangeEnd = range[1];

  // Binary search for first comment within `node`'s range.
  for (let lo = 0, hi = commentsLength; lo < hi; ) {
    const mid = (lo + hi) >> 1;
    if (comments[mid].start < rangeStart) {
      lo = mid + 1;
    } else {
      sliceStart = hi = mid;
    }
  }

  // Binary search for first comment outside `node`'s range.
  // Its index is used as `sliceEnd`, which is exclusive of the slice.
  for (let lo = sliceStart, hi = commentsLength; lo < hi; ) {
    const mid = (lo + hi) >> 1;
    if (comments[mid].start < rangeEnd) {
      lo = mid + 1;
    } else {
      sliceEnd = hi = mid;
    }
  }

  return comments.slice(sliceStart, sliceEnd);
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
  if (ast === null) initAst();
  debugAssertIsNonNull(ast);

  // Find the first comment after `nodeOrToken1` ends.
  const { comments } = ast,
    commentsLength = comments.length,
    betweenRangeStart = nodeOrToken1.range[1];
  let firstCommentBetween = -1;

  for (let lo = 0, hi = commentsLength; lo < hi; ) {
    const mid = (lo + hi) >> 1;
    if (comments[mid].start < betweenRangeStart) {
      lo = mid + 1;
    } else {
      firstCommentBetween = hi = mid;
    }
  }
  // Check if it ends before `nodeOrToken2` starts.
  return (
    0 <= firstCommentBetween &&
    firstCommentBetween < commentsLength &&
    comments[firstCommentBetween].end <= nodeOrToken2.range[0]
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
