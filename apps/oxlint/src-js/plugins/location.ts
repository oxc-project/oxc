/*
 * `SourceCode` methods related to `LineColumn`.
 * Functions for converting between `LineColumn` and offsets, and splitting source text into lines.
 */

import { ast, initAst, initSourceText, sourceText } from "./source_code.ts";
import visitorKeys from "../generated/keys.ts";
import { debugAssertIsNonNull } from "../utils/asserts.ts";

import type { Node } from "./types.ts";
import type { Node as ESTreeNode } from "../generated/types.d.ts";

const { defineProperty } = Object,
  { isArray } = Array;

/**
 * Range of source offsets.
 */
export type Range = [number, number];

/**
 * Interface for any type which has `range` field.
 */
export interface Ranged {
  range: Range;
}

/**
 * Interface for any type which has location properties.
 */
export interface Span extends Ranged {
  start: number;
  end: number;
  loc: Location;
}

/**
 * Source code location.
 */
export interface Location {
  start: LineColumn;
  end: LineColumn;
}

/**
 * Line number + column number pair.
 * `line` is 1-indexed, `column` is 0-indexed.
 */
export interface LineColumn {
  line: number;
  column: number;
}

// Pattern for splitting source text into lines
const LINE_BREAK_PATTERN = /\r\n|[\r\n\u2028\u2029]/gu;

// Lazily populated when `SOURCE_CODE.lines` is accessed.
// `lineStartOffsets` starts as `[0]`, and `resetLines` doesn't remove that initial element, so it's never empty.
export const lines: string[] = [];
const lineStartOffsets: number[] = [0];

/**
 * Split source text into lines.
 */
export function initLines(): void {
  if (sourceText === null) initSourceText();
  debugAssertIsNonNull(sourceText);

  // This implementation is based on the one in ESLint.
  // TODO: Investigate if using `String.prototype.matchAll` is faster.
  // This comment is above ESLint's implementation:
  /*
   * Previously, this was implemented using a regex that
   * matched a sequence of non-linebreak characters followed by a
   * linebreak, then adding the lengths of the matches. However,
   * this caused a catastrophic backtracking issue when the end
   * of a file contained a large number of non-newline characters.
   * To avoid this, the current implementation just matches newlines
   * and uses match.index to get the correct line start indices.
   */

  // Note: `lineStartOffsets` starts as `[0]`
  let lastOffset = 0,
    offset,
    match;
  while ((match = LINE_BREAK_PATTERN.exec(sourceText)) !== null) {
    offset = match.index;
    lines.push(sourceText.slice(lastOffset, offset));
    lineStartOffsets.push((lastOffset = offset + match[0].length));
  }
  lines.push(sourceText.slice(lastOffset));
}

/**
 * Reset lines after file has been linted, to free memory.
 */
export function resetLines(): void {
  lines.length = 0;
  // Leave first entry (0) in place, discard the rest
  lineStartOffsets.length = 1;
}

/**
 * Convert a source text index into a (line, column) pair.
 * @param offset - The index of a character in a file.
 * @returns `{line, column}` location object with 1-indexed line and 0-indexed column.
 * @throws {TypeError|RangeError} If non-numeric `offset`, or `offset` out of range.
 */
export function getLineColumnFromOffset(offset: number): LineColumn {
  if (typeof offset !== "number" || offset < 0 || (offset | 0) !== offset) {
    throw new TypeError("Expected `offset` to be a non-negative integer.");
  }

  // Build `lines` and `lineStartOffsets` tables if they haven't been already.
  // This also decodes `sourceText` if it wasn't already.
  if (lines.length === 0) initLines();
  debugAssertIsNonNull(sourceText);

  if (offset > sourceText.length) {
    throw new RangeError(
      `Index out of range (requested index ${offset}, but source text has length ${sourceText.length}).`,
    );
  }

  return getLineColumnFromOffsetUnchecked(offset);
}

/**
 * Convert a source text index into a (line, column) pair without:
 * 1. Checking type of `offset`, or that it's in range.
 * 2. Initializing `lineStartOffsets`. Caller must do that before calling this method.
 *
 * @param offset - The index of a character in a file.
 * @returns `{line, column}` location object with 1-indexed line and 0-indexed column.
 */
function getLineColumnFromOffsetUnchecked(offset: number): LineColumn {
  // Binary search `lineStartOffsets` for the line containing `offset`
  let low = 0,
    high = lineStartOffsets.length,
    mid: number;
  do {
    mid = ((low + high) / 2) | 0; // Use bitwise OR to floor the division
    if (offset < lineStartOffsets[mid]) {
      high = mid;
    } else {
      low = mid + 1;
    }
  } while (low < high);

  return { line: low, column: offset - lineStartOffsets[low - 1] };
}

/**
 * Convert a `{ line, column }` pair into a range index.
 * @param loc - A line/column location.
 * @returns The character index of the location in the file.
 * @throws {TypeError|RangeError} If `loc` is not an object with a numeric `line` and `column`,
 *   or if the `line` is less than or equal to zero, or the line or column is out of the expected range.
 */
export function getOffsetFromLineColumn(loc: LineColumn): number {
  if (loc !== null && typeof loc === "object") {
    const { line, column } = loc;
    if (
      typeof line === "number" &&
      typeof column === "number" &&
      (line | 0) === line &&
      (column | 0) === column
    ) {
      // Build `lines` and `lineStartOffsets` tables if they haven't been already.
      // This also decodes `sourceText` if it wasn't already.
      if (lines.length === 0) initLines();
      debugAssertIsNonNull(sourceText);

      const linesCount = lineStartOffsets.length;
      if (line <= 0 || line > linesCount) {
        throw new RangeError(
          `Line number out of range (line ${line} requested). ` +
            `Line numbers should be 1-based, and less than or equal to number of lines in file (${linesCount}).`,
        );
      }
      if (column < 0) throw new RangeError(`Invalid column number (column ${column} requested).`);

      const lineOffset = lineStartOffsets[line - 1];
      const offset = lineOffset + column;

      // Comment from ESLint implementation:
      /*
       * By design, `getIndexFromLoc({ line: lineNum, column: 0 })` should return the start index of
       * the given line, provided that the line number is valid element of `lines`. Since the
       * last element of `lines` is an empty string for files with trailing newlines, add a
       * special case where getting the index for the first location after the end of the file
       * will return the length of the file, rather than throwing an error. This allows rules to
       * use `getIndexFromLoc` consistently without worrying about edge cases at the end of a file.
       */

      let nextLineOffset;
      if (line === linesCount) {
        nextLineOffset = sourceText.length;
        if (offset <= nextLineOffset) return offset;
      } else {
        nextLineOffset = lineStartOffsets[line];
        if (offset < nextLineOffset) return offset;
      }

      throw new RangeError(
        `Column number out of range (column ${column} requested, ` +
          `but the length of line ${line} is ${nextLineOffset - lineOffset}).`,
      );
    }
  }

  throw new TypeError(
    "Expected `loc` to be an object with integer `line` and `column` properties.",
  );
}

/**
 * Get the `Location` for an AST node. Used in `loc` getters on AST nodes.
 *
 * Overwrites the `loc` getter with the calculated `Location`, so accessing `loc` twice on same node
 * results in the same object each time.
 *
 * For internal use only.
 *
 * @param node - AST node object
 * @returns Location
 */
export function getNodeLoc(node: Node): Location {
  // Build `lines` and `lineStartOffsets` tables if they haven't been already.
  // This also decodes `sourceText` if it wasn't already.
  if (lines.length === 0) initLines();

  const loc = {
    start: getLineColumnFromOffsetUnchecked(node.start),
    end: getLineColumnFromOffsetUnchecked(node.end),
  };

  // Replace `loc` getter with the calculated value
  defineProperty(node, "loc", { value: loc, writable: true });

  return loc;
}

/**
 * Get the deepest node containing a range index.
 * @param offset - Range index of the desired node
 * @returns The node if found, or `null` if not found
 */
export function getNodeByRangeIndex(offset: number): ESTreeNode | null {
  if (ast === null) initAst();
  debugAssertIsNonNull(ast);

  // If index is outside of `Program`, return `null`
  // TODO: Once `Program`'s span covers the entire file (as per ESLint v10), `index < ast.start` check can be removed
  // (or changed to `index < 0` if we want to check for negative indices)
  if (offset < ast.start || offset >= ast.end) return null;

  // Search for the node containing the index
  index = offset;
  return traverse(ast);
}

let index: number = 0;

/**
 * Find deepest node containing `index`.
 * `node` must contain `index` itself. This function finds a deeper node if one exists.
 *
 * @param node - Node to start traversal from
 * @returns Deepest node containing `index`
 */
function traverse(node: ESTreeNode): ESTreeNode {
  // TODO: Handle decorators on exports e.g. `@dec export class C {}`.
  // Decorators in that position have spans outside of the `export` node's span.
  // ESLint doesn't handle this case correctly, so not a big deal that we don't at present either.

  const keys = (visitorKeys as Record<string, readonly string[]>)[node.type];

  // All nodes' properties are in source order, so we could use binary search here.
  // But the max number of visitable properties is 5, so linear search is fine. Possibly linear is faster anyway.
  for (let keyIndex = 0, keysLen = keys.length; keyIndex < keysLen; keyIndex++) {
    const child = (node as unknown as Record<string, ESTreeNode | ESTreeNode[]>)[keys[keyIndex]];

    if (isArray(child)) {
      // TODO: Binary search would be faster, especially for arrays of statements, which can be large
      for (let arrIndex = 0, arrLen = child.length; arrIndex < arrLen; arrIndex++) {
        const entry = child[arrIndex];
        if (entry !== null) {
          // Array entries are in source order, so if this node is after the index,
          // all remaining nodes in the array are after the index too. So we can skip checking the rest of them.
          // We cannot skip all the rest of the outer loop, because in `TemplateLiteral`,
          // the 2 arrays `quasis` and `expressions` are interleaved. Ditto `TSTemplateLiteralType`.
          if (entry.start > index) break;
          // This node starts on or before the index. If it ends after the index, index is within this node.
          // Traverse into this node to find a deeper node if there is one.
          if (entry.end > index) return traverse(entry);
        }
      }
    } else if (child !== null) {
      // Node properties are in source order, so if this node is after the index,
      // all other properties are too. So we can skip checking the rest of them.
      if (child.start > index) break;
      // This node starts on or before the index. If it ends after the index, index is within this node.
      // Traverse into this node to find a deeper node if there is one.
      if (child.end > index) return traverse(child);
    }
  }

  // Index is not within any child node, so this is the deepest node containing the index
  return node;
}
