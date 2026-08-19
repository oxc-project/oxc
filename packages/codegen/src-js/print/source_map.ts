// Source map generation.
//
// Mapped writes record output offsets and original positions while printing.
// `generateSourceMap` defined here converts them to generated positions
// and encodes a standard Source Map v3 in one pass at the end.

import { debugAssert } from "../asserts.ts";

import type { Options, SourceMap } from "./options.ts";
import type { State } from "../state.ts";

const BASE64_CHARS = "ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
const BASE64_CODES = Uint8Array.from(BASE64_CHARS, (char) => char.charCodeAt(0));
const LINE_SEARCH_ITERATIONS = 16;
const MIN_LF_FAST_PATH_MAPPINGS = 64;
const MAX_LF_FAST_PATH_CHARS_PER_MAPPING = 256;
const MAX_BACKWARD_SOURCE_SCAN = 4096;
const MAX_REPLAYED_SOURCE_SCAN = 16384;
const MAX_BITWISE_VLQ = 0x7fffffff;
const MIN_MAPPING_BUFFER_LENGTH = 64;
// Real-world fixtures use 5.2–5.5 bytes per mapping. Leave some headroom and grow for outliers.
const ESTIMATED_BYTES_PER_MAPPING = 6;
const MAX_MAPPING_SEGMENT_LENGTH = 64;

// Buffer's initial capacity must be greater than or equal to the maximum extra capacity required
// at a `growMappingBuffer` call. See that function for more details.
debugAssert(
  MIN_MAPPING_BUFFER_LENGTH >= MAX_MAPPING_SEGMENT_LENGTH,
  "`MIN_MAPPING_BUFFER_LENGTH` must be >= `MAX_MAPPING_SEGMENT_LENGTH`",
);

// `\r\n` must be one line terminator, rather than two.
const NEXT_LINE_TERMINATOR_REGEX = /\r\n|[\r\n\u2028\u2029]/g;

const ASCII_DECODER = /* @__PURE__ */ new TextDecoder();

/**
 * Convert deferred mapping data into a standard Source Map v3 object.
 */
export function generateSourceMap(state: State, options: Options): SourceMap {
  debugAssert(
    state.mapPositions !== null,
    "Source map positions should exist when sourcemap generation is enabled",
  );

  const { output, mapPositions, mapNames, sourceText } = state;
  const mappingCount = mapPositions.length >> 1;

  debugAssert(sourceText !== null, "`sourceText` should be defined when producing a source map");

  if (mappingCount === 0) {
    return {
      version: 3,
      mappings: "",
      names: [],
      sources: [options.sourceFilename ?? ""],
      sourcesContent: [sourceText],
    };
  }

  let mappingBuffer: Uint8Array = new Uint8Array(
    Math.max(MIN_MAPPING_BUFFER_LENGTH, mappingCount * ESTIMATED_BYTES_PER_MAPPING),
  );
  let mappingLength = 0;
  const names: string[] = [];
  let nameIds: Map<string, number> | undefined;
  let mapNameEntryIndex = 0;
  let nextNamedMappingIndex = (mapNames?.[0] as number | undefined) ?? Infinity;

  let sourceLineStarts: number[] | undefined;
  let sourceScanOffset = 0;
  let sourceLine = 0;
  let sourceLineStart = 0;
  let replayedSourceScanTotal = 0;
  let lineStart = 0;

  // Proving an output contains only `\n` takes a full scan. Amortize it only when enough mappings will benefit.
  // Sparse maps and huge one-line literals stay on the single-pass regexp path.
  const useOutputLineFeedFastPath =
    mappingCount >= MIN_LF_FAST_PATH_MAPPINGS &&
    output.length <= mappingCount * MAX_LF_FAST_PATH_CHARS_PER_MAPPING &&
    !hasUncommonLineTerminator(output);

  // Require mappings to cover a substantial part of the source, so looking for the first line break
  // cannot scan a huge unmapped suffix. Reordered inputs conservatively take the slow path.
  const useSourceLineBoundaryCache =
    mappingCount >= MIN_LF_FAST_PATH_MAPPINGS &&
    sourceText.length <= mappingCount * MAX_LF_FAST_PATH_CHARS_PER_MAPPING &&
    mapPositions[mapPositions.length - 1] * 2 >= sourceText.length;
  const useSourceLineFeedFastPath =
    useSourceLineBoundaryCache && hasOnlyLineFeedsAndCrLf(sourceText);
  let nextLineStart = findNextLineStart(output, 0, useOutputLineFeedFastPath);
  let nextSourceLineStart = useSourceLineBoundaryCache
    ? findNextLineStart(sourceText, 0, useSourceLineFeedFastPath)
    : Infinity;
  let hasSegmentOnLine = false;
  let previousGeneratedColumn = 0;
  let previousOriginalLine = 0;
  let previousOriginalColumn = 0;
  let previousNameId = 0;

  for (let index = 0, positionIndex = 0; index < mappingCount; index++, positionIndex += 2) {
    const offset = mapPositions[positionIndex];
    while (offset >= nextLineStart) {
      lineStart = nextLineStart;
      nextLineStart = findNextLineStart(output, lineStart, useOutputLineFeedFastPath);
      previousGeneratedColumn = 0;
      hasSegmentOnLine = false;
      if (mappingLength === mappingBuffer.length) {
        mappingBuffer = growMappingBuffer(mappingBuffer, mappingLength);
      }
      mappingBuffer[mappingLength++] = 59; // `;`
    }

    const generatedColumn = offset - lineStart;

    // Rust end mappings use `span.end - 1`, which may land inside a multi-byte code point.
    // The JS equivalent can land on a low surrogate - normalize it back to the code point's start.
    let sourceOffset = mapPositions[positionIndex + 1];
    if (sourceOffset > 0) {
      const char = sourceText.charCodeAt(sourceOffset);
      if (
        char >= 0xdc00 &&
        char <= 0xdfff &&
        sourceText.charCodeAt(sourceOffset - 1) >= 0xd800 &&
        sourceText.charCodeAt(sourceOffset - 1) <= 0xdbff
      ) {
        sourceOffset--;
      }
    }

    let originalLine: number;
    let originalColumn: number;
    if (sourceLineStarts === undefined) {
      if (sourceOffset >= sourceScanOffset) {
        // Mappings almost always advance through the source. Scan only the text between this mapping
        // and the last one, instead of building a line table for the whole source.
        if (useSourceLineBoundaryCache) {
          while (sourceOffset >= nextSourceLineStart) {
            sourceLineStart = nextSourceLineStart;
            nextSourceLineStart = findNextLineStart(
              sourceText,
              sourceLineStart,
              useSourceLineFeedFastPath,
            );
            sourceLine++;
          }
          sourceScanOffset = sourceOffset;
        } else {
          while (sourceScanOffset < sourceOffset) {
            const char = sourceText.charCodeAt(sourceScanOffset);
            if (char === 13 && sourceText.charCodeAt(sourceScanOffset + 1) === 10) {
              // An offset on the `\n` of `\r\n` still belongs to the preceding line.
              if (sourceScanOffset + 1 === sourceOffset) break;
              sourceScanOffset += 2;
              sourceLine++;
              sourceLineStart = sourceScanOffset;
            } else if (char === 10 || char === 13 || char === 0x2028 || char === 0x2029) {
              sourceScanOffset++;
              sourceLine++;
              sourceLineStart = sourceScanOffset;
            } else {
              sourceScanOffset++;
            }
          }
        }
      } else if (
        sourceOffset >= sourceLineStart ||
        (sourceScanOffset - sourceOffset <= MAX_BACKWARD_SOURCE_SCAN &&
          replayedSourceScanTotal + sourceScanOffset - sourceOffset <=
            Math.max(MAX_REPLAYED_SOURCE_SCAN, sourceText.length))
      ) {
        // Parent/end mappings and locally reordered nodes can step backwards.
        // These moves are normally within the current line or a nearby one, so a short reverse scan
        // is cheaper than allocating a complete line table.
        //
        // A same-line backtrack needs no scan and leaves the forward cursor where it is.
        // When the mapping crosses a line, count the text a later forward step must replay.
        // Once replaying costs as much as building a line table, the fallback below becomes cheaper.
        if (sourceOffset < sourceLineStart) {
          replayedSourceScanTotal += sourceScanOffset - sourceOffset;

          while (sourceOffset < sourceLineStart) {
            let index = sourceLineStart - 1;
            if (sourceText.charCodeAt(index) === 10 && sourceText.charCodeAt(index - 1) === 13) {
              index -= 2;
            } else {
              index--;
            }

            while (index >= 0) {
              const char = sourceText.charCodeAt(index);
              if (char === 10 || char === 13 || char === 0x2028 || char === 0x2029) break;
              index--;
            }

            sourceLineStart = index + 1;
            sourceLine--;
          }

          sourceScanOffset = sourceOffset;

          if (useSourceLineBoundaryCache) {
            nextSourceLineStart = findNextLineStart(
              sourceText,
              sourceLineStart,
              useSourceLineFeedFastPath,
            );
          }
        }
      } else {
        // A heavily transformed AST can jump arbitrarily far backwards or oscillate repeatedly.
        // Build the table lazily once reverse scanning crosses either limit.
        sourceLineStarts = getLineStarts(sourceText);
        sourceLine = findSourceLine(sourceLineStarts, sourceOffset, previousOriginalLine);
        sourceLineStart = sourceLineStarts[sourceLine];
        sourceScanOffset = sourceOffset;
      }

      originalLine = sourceLine;
      originalColumn = sourceOffset - sourceLineStart;
    } else {
      originalLine = findSourceLine(sourceLineStarts, sourceOffset, previousOriginalLine);
      originalColumn = sourceOffset - sourceLineStarts[originalLine];
    }

    if (mappingLength + MAX_MAPPING_SEGMENT_LENGTH > mappingBuffer.length) {
      mappingBuffer = growMappingBuffer(mappingBuffer, mappingLength);
    }

    if (hasSegmentOnLine) mappingBuffer[mappingLength++] = 44; // `,`

    mappingLength = writeVlq(
      mappingBuffer,
      mappingLength,
      generatedColumn - previousGeneratedColumn,
    );

    // All mappings point into the one source file, so the source-index delta is always zero (`A`)
    mappingBuffer[mappingLength++] = 65;
    mappingLength = writeVlq(mappingBuffer, mappingLength, originalLine - previousOriginalLine);
    mappingLength = writeVlq(mappingBuffer, mappingLength, originalColumn - previousOriginalColumn);

    if (index === nextNamedMappingIndex) {
      nameIds ??= new Map<string, number>();

      const name = mapNames![mapNameEntryIndex + 1] as string;
      let nameId = nameIds.get(name);
      if (nameId === undefined) {
        nameId = names.length;
        names.push(name);
        nameIds.set(name, nameId);
      }

      mappingLength = writeVlq(mappingBuffer, mappingLength, nameId - previousNameId);
      previousNameId = nameId;
      mapNameEntryIndex += 2;
      nextNamedMappingIndex = (mapNames![mapNameEntryIndex] as number | undefined) ?? Infinity;
    }

    hasSegmentOnLine = true;
    previousGeneratedColumn = generatedColumn;
    previousOriginalLine = originalLine;
    previousOriginalColumn = originalColumn;
  }

  return {
    version: 3,
    mappings: ASCII_DECODER.decode(mappingBuffer.subarray(0, mappingLength)),
    names,
    sources: [options.sourceFilename ?? ""],
    sourcesContent: [sourceText],
  };
}

/**
 * Build UTF-16 offsets to the start of every ECMAScript source line.
 */
function getLineStarts(sourceText: string): number[] {
  const lineStarts = [0];
  for (let index = 0; index < sourceText.length; index++) {
    const char = sourceText.charCodeAt(index);
    if (char !== 10 && char !== 13 && char !== 0x2028 && char !== 0x2029) continue;
    if (char === 13 && sourceText.charCodeAt(index + 1) === 10) index++;
    lineStarts.push(index + 1);
  }
  return lineStarts;
}

/**
 * Find the source line containing `sourceOffset`, starting close to the previous result.
 */
function findSourceLine(lineStarts: number[], sourceOffset: number, previousLine: number): number {
  // Source mappings normally advance through the original file. Search the next few lines linearly,
  // as V8 optimizes this small predictable loop well, then fall back for large jumps or transformed ASTs
  // whose source locations move backwards.
  if (sourceOffset >= lineStarts[previousLine]) {
    const end = Math.min(previousLine + LINE_SEARCH_ITERATIONS + 1, lineStarts.length);
    for (let line = previousLine + 1; line < end; line++) {
      if (lineStarts[line] > sourceOffset) return line - 1;
    }
    if (end === lineStarts.length) return lineStarts.length - 1;
  } else {
    const end = Math.max(previousLine - LINE_SEARCH_ITERATIONS, 0);
    for (let line = previousLine - 1; line >= end; line--) {
      if (lineStarts[line] <= sourceOffset) return line;
    }
  }

  let low = 0;
  let high = lineStarts.length;
  while (low < high) {
    const middle = (low + high) >> 1;
    if (lineStarts[middle] <= sourceOffset) low = middle + 1;
    else high = middle;
  }
  return low - 1;
}

/**
 * Write one signed source-map delta as base64 VLQ, returning the next buffer position.
 */
function writeVlq(buffer: Uint8Array, index: number, value: number): number {
  let vlq = value < 0 ? -value * 2 + 1 : value * 2;
  if (vlq <= MAX_BITWISE_VLQ) {
    do {
      let digit = vlq & 31;
      vlq >>>= 5;
      if (vlq > 0) digit |= 32;
      buffer[index++] = BASE64_CODES[digit];
    } while (vlq > 0);

    return index;
  }

  do {
    let digit = vlq % 32;
    vlq = Math.floor(vlq / 32);
    if (vlq > 0) digit += 32;
    buffer[index++] = BASE64_CODES[digit];
  } while (vlq > 0);

  return index;
}

/**
 * Grow the mappings buffer by doubling, preserving its written prefix.
 *
 * The returned buffer is guaranteed to have at least `MAX_MAPPING_SEGMENT_LENGTH` spare capacity.
 */
function growMappingBuffer(buffer: Uint8Array, writtenLength: number): Uint8Array {
  // Buffer starts with at least `MIN_MAPPING_BUFFER_LENGTH` bytes capacity.
  // Maximum extra capacity required is `MAX_MAPPING_SEGMENT_LENGTH`, which is <= `MIN_MAPPING_BUFFER_LENGTH`,
  // so doubling capacity is always enough.
  const newBuffer = new Uint8Array(buffer.length * 2);
  newBuffer.set(buffer.subarray(0, writtenLength));
  return newBuffer;
}

/**
 * Find the UTF-16 offset after the next ECMAScript line terminator.
 */
function findNextLineStart(output: string, from: number, useLineFeedFastPath: boolean): number {
  if (useLineFeedFastPath) {
    const index = output.indexOf("\n", from);
    return index === -1 ? Infinity : index + 1;
  }

  // Let the regexp engine scan long generated lines. A JS `charCodeAt` loop is much slower for large literals,
  // while `lastIndex` avoids allocating a substring just to start the search at `from`.
  NEXT_LINE_TERMINATOR_REGEX.lastIndex = from;
  const match = NEXT_LINE_TERMINATOR_REGEX.exec(output);
  return match === null ? Infinity : match.index + match[0].length;
}

/**
 * Whether output contains a line terminator other than `\n`.
 */
function hasUncommonLineTerminator(output: string): boolean {
  // V8's specialized substring search is substantially faster than a regexp scan here, even when
  // all three searches miss. This also avoids allocating regexp match state for the common path.
  return (
    output.indexOf("\r") !== -1 ||
    output.indexOf("\u2028") !== -1 ||
    output.indexOf("\u2029") !== -1
  );
}

/**
 * Whether every source line terminator can be found by searching for `\n`.
 */
function hasOnlyLineFeedsAndCrLf(sourceText: string): boolean {
  if (sourceText.indexOf("\u2028") !== -1 || sourceText.indexOf("\u2029") !== -1) return false;

  let carriageReturn = sourceText.indexOf("\r");
  while (carriageReturn !== -1) {
    if (sourceText.charCodeAt(carriageReturn + 1) !== 10) return false;
    carriageReturn = sourceText.indexOf("\r", carriageReturn + 1);
  }

  return true;
}
