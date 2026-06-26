/*
 * `SourceCode` methods related to `LineColumn`.
 * Functions for converting between `LineColumn` and offsets, and splitting source text into lines.
 */

import { ast, initAst, initSourceText, sourceText } from "./source_code.ts";
import visitorKeys from "../generated/keys.ts";
import { debugAssert, debugAssertIsNonNull } from "../utils/asserts.ts";

import type { NodeOrToken, Node } from "./types.ts";
import type { Node as ESTreeNode } from "../generated/types.d.ts";

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
// `lineStartIndices` starts as `[0]`, and `resetLinesAndLocs` doesn't remove that initial element, so it's never empty.
export const lines: string[] = [];
export const lineStartIndices: number[] = [0];

// Pool of `Location` objects, reused across files to reduce GC pressure.
// Each `Location` contains `start` and `end` `LineColumn` sub-objects, which are also reused.
// Never shrunk - `activeLocationsCount` tracks the active count to avoid freeing the backing store.
const cachedLocations: Location[] = [];
let activeLocationsCount = 0;

/**
 * Split source text into lines.
 */
export function initLines(): void {
  if (sourceText === null) initSourceText();
  debugAssertIsNonNull(sourceText);

  // TODO: ESLint freezes `lines`, but doesn't freeze `lineStartIndices`.
  // Should we freeze them? Upside is it would prevent user mutating them, but on downside would prevent us re-using
  // the same arrays for multiple files. Maybe we shouldn't bother, in same way that we don't freeze the AST.
  // Once we introduce lazy deserialization, presumably we'll use proxy arrays (like `NodeArray`), which will make
  // them immutable by user. Maybe we can leave it until then. (@overlookmotel)

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

  // `lineStartIndices` starts as `[0]`, and is reset to length 1 in `resetLinesAndLocs`.
  // Debug check that `lines` and `lineStartIndices` are not already initialized.
  debugAssert(lines.length === 0, "`lines` should be empty at start of `initLines`");
  debugAssert(
    lineStartIndices.length === 1,
    "`lineStartIndices` should have length 1 at start of `initLines`",
  );

  let lastOffset = 0,
    offset,
    match;
  while ((match = LINE_BREAK_PATTERN.exec(sourceText)) !== null) {
    offset = match.index;
    lines.push(sourceText.slice(lastOffset, offset));
    lineStartIndices.push((lastOffset = offset + match[0].length));
  }
  lines.push(sourceText.slice(lastOffset));

  debugAssertLinesIsInitialized();
}

/**
 * Debug assert that `lines` and `lineStartIndices` are initialized.
 * No-op in release build - TSDown will remove this function and all calls to it.
 */
export function debugAssertLinesIsInitialized(): void {
  debugAssert(lines.length > 0, "`lines` should be initialized");
  debugAssert(
    lines.length === lineStartIndices.length,
    "`lines` and `lineStartIndices` should be same length",
  );
}

/**
 * Reset lines after file has been linted, to free memory.
 * Reset `Location` object pool.
 */
export function resetLinesAndLocs(): void {
  lines.length = 0;
  // Leave first entry (0) in place, discard the rest
  lineStartIndices.length = 1;

  activeLocationsCount = 0;
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

  // Build `lines` and `lineStartIndices` tables if they haven't been already.
  // This also decodes `sourceText` if it wasn't already.
  if (lines.length === 0) initLines();
  debugAssertIsNonNull(sourceText);
  debugAssertLinesIsInitialized();

  if (offset > sourceText.length) {
    throw new RangeError(
      `Index out of range (requested index ${offset}, but source text has length ${sourceText.length}).`,
    );
  }

  // Find first line that starts *after* `offset`, via binary search of `lineStartIndices`.
  // `lineStartIndices` is sorted and `lineStartIndices[0]` is always 0.
  //
  // After the loop, `low` is the index of the first line whose start is *past* `offset`.
  // This is also the 1-indexed line number of the line containing `offset`.
  // e.g. if `offset` is on the 3rd line, `low` = 3, and `lineStartIndices[2]` is that line's start.
  // `do...while` is safe because `lineStartIndices` always has at least one entry, so `low < high` at start of loop.
  //
  // Note: Source text is limited to 1 GiB max, so offsets cannot exceed 2^30.
  // This makes it safe to use `>> 1` for division by 2 below (which is faster than `>>> 1`).
  let low = 0,
    high = lineStartIndices.length,
    mid: number;
  do {
    mid = (low + high) >> 1;
    if (offset < lineStartIndices[mid]) {
      high = mid;
    } else {
      low = mid + 1;
    }
  } while (low < high);

  return {
    line: low, // 1-indexed line number
    column: offset - lineStartIndices[low - 1], // Offset from start of the line
  };
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
      // Build `lines` and `lineStartIndices` tables if they haven't been already.
      // This also decodes `sourceText` if it wasn't already.
      if (lines.length === 0) initLines();
      debugAssertIsNonNull(sourceText);
      debugAssertLinesIsInitialized();

      const linesCount = lineStartIndices.length;
      if (line <= 0 || line > linesCount) {
        throw new RangeError(
          `Line number out of range (line ${line} requested). ` +
            `Line numbers should be 1-based, and less than or equal to number of lines in file (${linesCount}).`,
        );
      }
      if (column < 0) throw new RangeError(`Invalid column number (column ${column} requested).`);

      const lineOffset = lineStartIndices[line - 1];
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
        nextLineOffset = lineStartIndices[line];
        if (offset < nextLineOffset) return offset;
      }

      throw new RangeError(
        `Column number out of range (column ${column} requested, ` +
          `but the length of line ${line} is ${nextLineOffset - lineOffset}).`,
      );
    }
  }

  throw new TypeError("Expected `loc` to be an object with integer `line` and `column` properties");
}

/**
 * Get the range of the given node or token.
 * @param nodeOrToken - Node or token to get the range of
 * @returns Range of the node or token
 */
export function getRange(nodeOrToken: NodeOrToken): Range {
  return nodeOrToken.range;
}

/**
 * Get the location of the given node or token.
 * @param nodeOrToken - Node or token to get the location of
 * @returns Location of the node or token
 */
// AST nodes, tokens, and comments handle lazy `loc` computation and caching via their respective getters
// (AST nodes via `NodeProto` prototype getter which caches via `Object.defineProperty`,
// tokens and comments via `Token` / `Comment` class getters which cache in private fields).
// So accessing `.loc` gives the right behavior for all 3, including stable object identity.
export function getLoc(nodeOrToken: NodeOrToken): Location {
  return nodeOrToken.loc;
}

/**
 * Calculate the `Location` for an AST node, and cache it on the node.
 *
 * Used in `loc` getter on AST nodes (not tokens or comments - they use their own caching
 * via `Token` / `Comment` class private fields).
 *
 * Defines a `loc` property on the node with the calculated `Location`, so accessing `loc` twice on same node
 * results in the same object each time.
 *
 * For internal use only.
 *
 * @param node - AST node
 * @returns Location
 */
export function getNodeLoc(node: Node): Location {
  const loc = computeLoc(node.start, node.end);

  // Define `loc` property with the calculated `Location`, so accessing `loc` twice on same node
  // results in the same object each time.
  //
  // We do not make the `loc` property enumerable, because it wasn't present before.
  // It would be weird if `Object.keys(node)` included `loc` if the property had been accessed previously,
  // but not if it hadn't.
  //
  // We also don't make it configurable, because deleting it wouldn't make `node.loc` evaluate to `undefined`,
  // because the access would fall through to the getter on the prototype.
  //
  // Reuse `LOC_DESCRIPTOR` object to avoid unnecessarily creating a temporary object each time.
  LOC_DESCRIPTOR.value = loc;
  Object.defineProperty(node, "loc", LOC_DESCRIPTOR);

  return loc;
}

// Reusable property descriptor for `Object.defineProperty` in `getNodeLoc`.
const LOC_DESCRIPTOR: PropertyDescriptor = {
  value: null,
  writable: true,
  enumerable: false,
  configurable: false,
};

/**
 * Compute a `Location` from `start` and `end` source offsets.
 *
 * Returns a recycled `Location` object from the pool when possible, allocating only during warmup.
 * Initializes `lines` and `lineStartIndices` tables if they haven't been already.
 *
 * @param start - Start offset
 * @param end - End offset
 * @returns Location
 */
export function computeLoc(start: number, end: number): Location {
  // All AST nodes, tokens and comments have `start < end`, with only one exception:
  // `Program` node can have `start === end` if it has no directives or statements - either 0-length file,
  // or purely comments and/or whitespace and/or hashbang. But `start > end` is impossible.
  debugAssert(start <= end, "`start` must be <= `end`");

  if (lines.length === 0) initLines();
  debugAssertLinesIsInitialized();

  // Reuse a cached `Location` object if available, otherwise create a new one.
  // Note: The comparison `activeLocationsCount < cachedLocations.length` must be this way around
  // so that V8 can remove the bounds check on `cachedLocations[activeLocationsCount]`.
  // `cachedLocations.length > activeLocationsCount` would *not* remove the bounds check in Maglev compiler,
  // even though it's semantically equivalent.
  let loc: Location;
  if (activeLocationsCount < cachedLocations.length) {
    loc = cachedLocations[activeLocationsCount];
  } else {
    cachedLocations.push((loc = { start: { line: 0, column: 0 }, end: { line: 0, column: 0 } }));
  }

  activeLocationsCount++;

  const linesLen = lineStartIndices.length;

  // Find first line that starts *after* `start`, via binary search of `lineStartIndices`.
  // `lineStartIndices` is sorted and `lineStartIndices[0]` is always 0.
  //
  // After the loop, `line` is the index of the first line whose start is *past* `start`.
  // This is also the 1-indexed line number of the line containing `start`.
  // e.g. if `start` is on the 3rd line, `line` = 3, and `lineStartIndices[2]` is that line's start.
  // `do...while` is safe because `lineStartIndices` always has at least one entry, so `line < high` at start of loop.
  //
  // Note: Source text is limited to 1 GiB max, so number of lines cannot exceed 2^30.
  // This makes it safe to use `>> 1` for division by 2 below (which is faster than `>>> 1`).
  let line = 0,
    high = linesLen,
    mid: number;
  do {
    mid = (line + high) >> 1;
    if (start < lineStartIndices[mid]) {
      high = mid;
    } else {
      line = mid + 1;
    }
  } while (line < high);

  const lineStart = lineStartIndices[line - 1];

  const locStart = loc.start;
  locStart.line = line;
  locStart.column = start - lineStart;

  // Fast path: If `end` is on the same line as `start`, skip the second binary search.
  // Most tokens (and many small AST nodes) are on a single line, so this is the common case.
  // `line` indexes the *next* line's start in `lineStartIndices`.
  // If we're on the last line, or `end` is before the next line's start, `end` is on the same line as `start`.
  const locEnd = loc.end;
  if (line === linesLen || end < lineStartIndices[line]) {
    locEnd.line = line;
    locEnd.column = end - lineStart;
  } else {
    // `end` is on a later line than `start`.
    //
    // Find first line that starts *after* `end`, via binary search of `lineStartIndices`.
    // Start search from the line after the one containing `start`, to narrow the search range.
    //
    // After the loop, `line` is the index of the first line whose start is *past* `end`.
    // This is also the 1-indexed line number of the line containing `end`.
    line++;
    high = linesLen;
    while (line < high) {
      mid = (line + high) >> 1;
      if (end < lineStartIndices[mid]) {
        high = mid;
      } else {
        line = mid + 1;
      }
    }

    locEnd.line = line;
    locEnd.column = end - lineStartIndices[line - 1];
  }

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

    if (Array.isArray(child)) {
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
