/*
 * Comment class, object pooling, deserialization, and `SourceCode` methods related to comments.
 */

import { ast, buffer, initAst, sourceText } from "./source_code.ts";
import {
  COMMENTS_OFFSET,
  COMMENTS_LEN_OFFSET,
  COMMENT_SIZE,
  COMMENT_KIND_OFFSET,
  COMMENT_LINE_KIND,
  DATA_POINTER_POS_32,
} from "../generated/constants.ts";
import { computeLoc } from "./location.ts";
import { firstTokenAtOrAfter } from "./tokens_methods.ts";
import { debugAssert, debugAssertIsNonNull } from "../utils/asserts.ts";

import type { Node, NodeOrToken } from "./types.ts";
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
 * If the program has a hashbang, prepends a `Shebang` comment.
 */
export function initComments(): void {
  debugAssert(comments === null, "Comments already initialized");

  if (ast === null) initAst();
  debugAssertIsNonNull(ast);
  debugAssertIsNonNull(sourceText);
  debugAssertIsNonNull(buffer);

  const { uint32 } = buffer;
  const programPos32 = uint32[DATA_POINTER_POS_32] >> 2;
  let pos = uint32[programPos32 + (COMMENTS_OFFSET >> 2)];
  const commentsLen = uint32[programPos32 + (COMMENTS_LEN_OFFSET >> 2)];

  // Determine total number of comments (including shebang if present)
  const { hashbang } = ast;
  let index = +(hashbang !== null);
  const totalLen = commentsLen + index;

  // Grow cache if needed (one-time cost as cache warms up)
  while (cachedComments.length < totalLen) {
    cachedComments.push(new Comment());
  }

  // If there's a hashbang, populate slot 0 with `Shebang` comment
  if (index !== 0) {
    debugAssertIsNonNull(hashbang);

    const comment = cachedComments[0];
    comment.type = "Shebang";
    comment.value = hashbang.value;
    comment.range[0] = comment.start = hashbang.start;
    comment.range[1] = comment.end = hashbang.end;
  }

  // Deserialize comments from buffer
  while (index < totalLen) {
    const comment = cachedComments[index++];

    const start = uint32[pos >> 2];
    const end = uint32[(pos + 4) >> 2];
    const isBlock = buffer[pos + COMMENT_KIND_OFFSET] !== COMMENT_LINE_KIND;

    comment.type = isBlock ? "Block" : "Line";
    // Line comments: `// text` -> slice `start + 2..end`
    // Block comments: `/* text */` -> slice `start + 2..end - 2`
    comment.value = sourceText.slice(start + 2, end - (+isBlock << 1));
    comment.range[0] = comment.start = start;
    comment.range[1] = comment.end = end;

    pos += COMMENT_SIZE;
  }

  // Use `slice` rather than copying comments one-by-one into a new array.
  // V8 implements `slice` with a single `memcpy` of the backing store, which is faster
  // than N individual `push` calls with bounds checking and potential resizing.
  //
  // If the comments array from previous file is longer than the current one,
  // reuse it and truncate it to avoid the memcpy entirely.
  if (previousComments.length >= totalLen) {
    previousComments.length = totalLen;
    comments = previousComments;
  } else {
    comments = previousComments = cachedComments.slice(0, totalLen);
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

// Regex that tests if a string is entirely whitespace.
const WHITESPACE_ONLY_REGEXP = /^\s*$/;

/**
 * Retrieve an array containing all comments in the source code.
 * @returns Array of `Comment`s in order they appear in source.
 */
export function getAllComments(): CommentType[] {
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
export function getCommentsBefore(nodeOrToken: NodeOrToken): CommentType[] {
  if (comments === null) initComments();
  debugAssertIsNonNull(comments);
  debugAssertIsNonNull(sourceText);

  let targetStart = nodeOrToken.range[0]; // start

  // Binary search for first comment at or past `nodeOrToken`'s start.
  // Comments before this index are candidates to be included in returned array.
  const sliceEnd = firstTokenAtOrAfter(comments, targetStart, 0);

  let sliceStart = comments.length;
  for (let i = sliceEnd - 1; i >= 0; i--) {
    const comment = comments[i];
    const gap = sourceText.slice(comment.end, targetStart);
    // Ensure that there is nothing except whitespace between the end of the
    // current comment and the start of the next one as we iterate backwards
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
export function getCommentsAfter(nodeOrToken: NodeOrToken): CommentType[] {
  if (comments === null) initComments();
  debugAssertIsNonNull(comments);
  debugAssertIsNonNull(sourceText);

  let targetEnd = nodeOrToken.range[1]; // end

  // Binary search for first comment at or past `nodeOrToken`'s end.
  // Comments from this index onwards are candidates to be included in returned array.
  const sliceStart = firstTokenAtOrAfter(comments, targetEnd, 0);

  const commentsLength = comments.length;
  let sliceEnd = 0;
  for (let i = sliceStart; i < commentsLength; i++) {
    // Ensure that there is nothing except whitespace between the
    // end of the previous comment and the start of the current one
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
export function getCommentsInside(node: Node): CommentType[] {
  if (comments === null) initComments();
  debugAssertIsNonNull(comments);

  const { range } = node,
    rangeStart = range[0],
    rangeEnd = range[1];

  // Binary search for first comment within `node`'s range
  const sliceStart = firstTokenAtOrAfter(comments, rangeStart, 0);
  // Binary search for first comment outside `node`'s range.
  // Its index is used as `sliceEnd`, which is exclusive of the slice.
  const sliceEnd = firstTokenAtOrAfter(comments, rangeEnd, sliceStart);

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
  if (comments === null) initComments();
  debugAssertIsNonNull(comments);

  // Find the first comment after `nodeOrToken1` ends.
  const betweenRangeStart = nodeOrToken1.range[1];
  const firstCommentBetween = firstTokenAtOrAfter(comments, betweenRangeStart, 0);
  // Check if it ends before `nodeOrToken2` starts.
  return (
    firstCommentBetween < comments.length &&
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
export function getJSDocComment(node: Node): CommentType | null {
  throw new Error("`sourceCode.getJSDocComment` is not supported at present (and deprecated)"); // TODO
}
/* oxlint-enable no-unused-vars */
