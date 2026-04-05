import {
  ACTIVE_SIZE,
  BUFFER_ALIGN,
  BUFFER_SIZE,
  DATA_POINTER_POS_32,
} from "../generated/constants.ts";
import { deserializeProgramOnly, resetBuffer } from "../generated/deserialize.js";
import { getBufferOffset, parseRawSync, rawTransferSupported as rawTransferSupportedBinding } from "../bindings.js";

import type { ParserOptions as RawParserOptions } from "../bindings.js";
import type { BufferWithArrays, Node } from "./types.ts";
import type { Location, LineColumn } from "./location.ts";
import type { Program } from "../generated/types.d.ts";

const ARRAY_BUFFER_SIZE = BUFFER_SIZE + BUFFER_ALIGN;
const ONE_GIB = 1 << 30;
const LINE_BREAK_PATTERN = /\r\n|[\r\n\u2028\u2029]/gu;

const textEncoder = new TextEncoder();

let parserBuffer: BufferWithArrays | null = null;
let rawTransferIsSupported: boolean | null = null;

interface ParseResult {
  buffer: BufferWithArrays;
  sourceStartPos: number;
  sourceByteLen: number;
}

export function parseProgram(
  filename: string,
  sourceText: string,
  options?: Record<string, unknown> | null,
): Program {
  if (!rawTransferSupported()) {
    throw new Error(
      "`context.languageOptions.parser.parse` is not supported on 32-bit or big-endian systems, versions of NodeJS prior to v22.0.0, versions of Deno prior to v2.0.0, or other runtimes",
    );
  }

  const { buffer, sourceStartPos, sourceByteLen } = parseIntoBuffer(filename, sourceText, options);
  const program = deserializeProgramOnly(
    buffer,
    sourceText,
    sourceStartPos,
    sourceByteLen,
    createGetNodeLoc(sourceText),
  );

  // `deserializeProgramOnly` stores references in module-level globals.
  // Clear those eagerly so later internal AST deserialization is fully independent.
  resetBuffer();

  // These getters are tied to Oxlint's main file-level comments/tokens caches.
  // Parsed sub-ASTs should never expose comments/tokens from whatever file is currently being linted.
  Object.defineProperty(program, "comments", {
    value: [],
    writable: false,
    enumerable: true,
    configurable: true,
  });
  Object.defineProperty(program, "tokens", {
    value: [],
    writable: false,
    enumerable: true,
    configurable: true,
  });

  return program;
}

function parseIntoBuffer(
  filename: string,
  sourceText: string,
  options?: Record<string, unknown> | null,
): ParseResult {
  if (parserBuffer === null) initParserBuffer();
  const buffer = parserBuffer!;

  const maxSourceByteLen = sourceText.length * 3;
  if (maxSourceByteLen > ONE_GIB) throw new Error("Source text is too long");
  const sourceStartPos = ACTIVE_SIZE - maxSourceByteLen;

  const sourceBuffer = new Uint8Array(
    buffer.buffer,
    buffer.byteOffset + sourceStartPos,
    maxSourceByteLen,
  );
  const { read, written: sourceByteLen } = textEncoder.encodeInto(sourceText, sourceBuffer);
  if (read !== sourceText.length) throw new Error("Failed to write source text into buffer");

  parseRawSync(filename, buffer, sourceStartPos, sourceByteLen, normalizeParserOptions(options));

  const programOffset = buffer.uint32[DATA_POINTER_POS_32];
  if (programOffset === 0) throw new Error("Parsing failed");

  return { buffer, sourceStartPos, sourceByteLen };
}

function normalizeParserOptions(
  options?: Record<string, unknown> | null,
): RawParserOptions | null {
  if (options == null) return null;

  const parserOptions: RawParserOptions = {};

  if (
    options.lang === "js" ||
    options.lang === "jsx" ||
    options.lang === "ts" ||
    options.lang === "tsx" ||
    options.lang === "dts"
  ) {
    parserOptions.lang = options.lang;
  }

  if (
    options.sourceType === "script" ||
    options.sourceType === "module" ||
    options.sourceType === "commonjs" ||
    options.sourceType === "unambiguous"
  ) {
    parserOptions.sourceType = options.sourceType;
  }

  if (typeof options.ignoreNonFatalErrors === "boolean") {
    parserOptions.ignoreNonFatalErrors = options.ignoreNonFatalErrors;
  }

  return Object.keys(parserOptions).length === 0 ? null : parserOptions;
}

function initParserBuffer(): void {
  const arrayBuffer = new ArrayBuffer(ARRAY_BUFFER_SIZE);
  const offset = getBufferOffset(new Uint8Array(arrayBuffer));
  parserBuffer = new Uint8Array(arrayBuffer, offset, BUFFER_SIZE) as BufferWithArrays;
  parserBuffer.uint32 = new Uint32Array(arrayBuffer, offset, BUFFER_SIZE / 4);
  parserBuffer.float64 = new Float64Array(arrayBuffer, offset, BUFFER_SIZE / 8);
}

function createGetNodeLoc(sourceText: string): (node: Node) => Location {
  let lineStartIndices: number[] | null = null;

  const getLineStarts = () => {
    if (lineStartIndices !== null) return lineStartIndices;

    lineStartIndices = [0];
    LINE_BREAK_PATTERN.lastIndex = 0;
    let match: RegExpExecArray | null;
    while ((match = LINE_BREAK_PATTERN.exec(sourceText)) !== null) {
      lineStartIndices.push(match.index + match[0].length);
    }
    return lineStartIndices;
  };

  const getLineColumn = (offset: number): LineColumn => {
    const starts = getLineStarts();
    let low = 0;
    let high = starts.length;

    while (low < high) {
      const mid = (low + high) >>> 1;
      if (starts[mid] <= offset) {
        low = mid + 1;
      } else {
        high = mid;
      }
    }

    const lineIndex = low - 1;
    return {
      line: lineIndex + 1,
      column: offset - starts[lineIndex],
    };
  };

  return (node: Node): Location => ({
    start: getLineColumn(node.start),
    end: getLineColumn(node.end),
  });
}

function rawTransferSupported(): boolean {
  if (rawTransferIsSupported === null) {
    rawTransferIsSupported = rawTransferRuntimeSupported() && rawTransferSupportedBinding();
  }
  return rawTransferIsSupported;
}

declare global {
  // oxlint-disable-next-line no-var
  var Bun: unknown;
  // oxlint-disable-next-line no-var
  var Deno:
    | {
        version: {
          deno: string;
        };
      }
    | undefined;
}

function rawTransferRuntimeSupported(): boolean {
  let global;
  try {
    global = globalThis;
  } catch {
    return false;
  }

  const processObject = (global as { process?: { versions?: { bun?: string }; release?: { name?: string } } })
    .process;
  const isBun = !!global.Bun || !!processObject?.versions?.bun;
  if (isBun) return false;

  const isDeno = !!global.Deno;
  if (isDeno) {
    const match = Deno!.version?.deno?.match(/^(\d+)\./);
    return !!match && +match[1] >= 2;
  }

  const isNode = processObject?.release?.name === "node";
  if (!isNode) return false;

  const match = process.version?.match(/^v(\d+)\./);
  return !!match && +match[1] >= 22;
}
