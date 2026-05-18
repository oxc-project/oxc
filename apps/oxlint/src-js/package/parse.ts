import {
  getBufferOffset,
  rawTransferSupported as rawTransferSupportedBinding,
  parseRawSync,
} from "../bindings.js";
import { debugAssert, debugAssertIsNonNull } from "../utils/asserts.ts";
import { buffers } from "../plugins/lint.ts";
import {
  BLOCK_SIZE,
  BLOCK_ALIGN,
  BUFFER_SIZE,
  DATA_POINTER_POS_32,
  ACTIVE_SIZE,
} from "../generated/constants.ts";

import type { BufferWithArrays } from "../plugins/types.ts";
import type { ParserOptions as ParseOptions } from "../bindings.js";

export type { ParseOptions };

// Size array buffer for raw transfer
const ARRAY_BUFFER_SIZE = BLOCK_SIZE + BLOCK_ALIGN;

// 1 GiB
const ONE_GIB = 1 << 30;

// Text encoder for encoding source text into buffer
const textEncoder = new TextEncoder();

// Buffers for raw transfer.
// Both are views of the same memory, but `blockBuffer` is slightly larger, and is what we pass to Rust.
let buffer: BufferWithArrays | null = null;
let blockBuffer: Uint8Array | null = null;

// Whether raw transfer is supported
let rawTransferIsSupported: boolean | null = null;

/**
 * Parser source text into buffer.
 * @param path - Path of file to parse
 * @param sourceText - Source text to parse
 * @param options - Parsing options
 * @throws {Error} If raw transfer is not supported on this platform, or parsing failed
 */
export function parse(path: string, sourceText: string, options?: ParseOptions) {
  if (!rawTransferSupported()) {
    throw new Error(
      "`RuleTester` is not supported on 32-bit or big-endian systems, versions of NodeJS prior to v22.0.0, " +
        "versions of Deno prior to v2.0.0, or other runtimes",
    );
  }

  // Initialize buffer, if not already
  if (buffer === null) initBuffer();
  debugAssertIsNonNull(buffer);
  debugAssertIsNonNull(blockBuffer);

  // Write source into end of buffer.
  // Maximum size of a string encoded in UTF-8 is 3 x the length of the string in UTF-16 characters
  // (a source which consists entirely of 3-byte UTF-8 characters).
  // We can't predict how many bytes will be needed exactly in advance of encoding, so we reserve
  // the maximum theoretically possible number of bytes required.
  // `TextEncoder` cannot write into a `Uint8Array` larger than 1 GiB, so size is capped at 1 GiB.
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
  debugAssert(sourceByteLen <= maxSourceByteLen);

  // Parse into buffer
  parseRawSync(path, blockBuffer, sourceStartPos, sourceByteLen, options);

  // Check parsing succeeded.
  // 0 is used as sentinel value to indicate parsing failed.
  // TODO: Get parsing error details from Rust to display nicely.
  const programOffset = buffer.int32[DATA_POINTER_POS_32];
  if (programOffset === 0) throw new Error("Parsing failed");
}

/**
 * Create a `Uint8Array` which is 2 GiB in size, with its start aligned on 4 GiB.
 *
 * Store it in `buffer`, and also in `buffers` array, so it's accessible to `lintFileImpl` by passing `0`as `bufferId`.
 *
 * Achieve this by creating a 6 GiB `ArrayBuffer`, getting the offset within it that's aligned to 4 GiB,
 * chopping off that number of bytes from the start, and shortening to 2 GiB.
 *
 * It's always possible to obtain a 2 GiB slice aligned on 4 GiB within a 6 GiB buffer,
 * no matter how the 6 GiB buffer is aligned.
 *
 * `buffer` itself, and `int32` and `float64` views of `buffer`, are `BUFFER_SIZE` bytes,
 * which excludes `FixedSizeAllocatorMetadata` and `ChunkFooter`.
 * This ensures this critical data cannot be accidentally overwritten on JS side.
 * `blockBuffer` is `BLOCK_SIZE` bytes, which includes `FixedSizeAllocatorMetadata` and `ChunkFooter`.
 * `blockBuffer` is what we pass to Rust, which needs to write them.
 *
 * Note: On systems with virtual memory, this only consumes 6 GiB of *virtual* memory.
 * It does not consume physical memory until data is actually written to the `Uint8Array`.
 * Physical memory consumed corresponds to the quantity of data actually written.
 */
export function initBuffer() {
  // Create buffer
  const arrayBuffer = new ArrayBuffer(ARRAY_BUFFER_SIZE);
  const offset = getBufferOffset(new Uint8Array(arrayBuffer));
  buffer = new Uint8Array(arrayBuffer, offset, BUFFER_SIZE) as BufferWithArrays;
  buffer.int32 = new Int32Array(arrayBuffer, offset, BUFFER_SIZE / 4);
  buffer.float64 = new Float64Array(arrayBuffer, offset, BUFFER_SIZE / 8);

  blockBuffer = new Uint8Array(arrayBuffer, offset, BLOCK_SIZE);

  // Store in `buffers`, at index 0
  debugAssert(buffers.length === 0, "`buffers` array should be empty");
  buffers.push(buffer);
}

/**
 * Returns `true` if raw transfer is supported.
 *
 * Raw transfer is only supported on 64-bit little-endian systems,
 * and NodeJS >= v22.0.0 or Deno >= v2.0.0.
 *
 * Versions of NodeJS prior to v22.0.0 do not support creating an `ArrayBuffer` larger than 4 GiB.
 * Bun (as at v1.2.4) also does not support creating an `ArrayBuffer` larger than 4 GiB.
 * Support on Deno v1 is unknown and it's EOL, so treating Deno before v2.0.0 as unsupported.
 *
 * No easy way to determining pointer width (64 bit or 32 bit) in JS,
 * so call a function on Rust side to find out.
 *
 * @returns {boolean} - `true` if raw transfer is supported on this platform
 */
function rawTransferSupported() {
  if (rawTransferIsSupported === null) {
    rawTransferIsSupported = rawTransferRuntimeSupported() && rawTransferSupportedBinding();
  }
  return rawTransferIsSupported;
}

declare global {
  var Bun: unknown;
  var Deno:
    | {
        version: {
          deno: string;
        };
      }
    | undefined;
}

// Checks copied from:
// https://github.com/unjs/std-env/blob/ab15595debec9e9115a9c1d31bc7597a8e71dbfd/src/runtimes.ts
// MIT license: https://github.com/unjs/std-env/blob/ab15595debec9e9115a9c1d31bc7597a8e71dbfd/LICENCE
function rawTransferRuntimeSupported() {
  let global;
  try {
    global = globalThis;
  } catch {
    return false;
  }

  const isBun = !!global.Bun || !!global.process?.versions?.bun;
  if (isBun) return false;

  const isDeno = !!global.Deno;
  if (isDeno) {
    const match = Deno!.version?.deno?.match(/^(\d+)\./);
    return !!match && +match[1] >= 2;
  }

  const isNode = global.process?.release?.name === "node";
  if (!isNode) return false;

  const match = process.version?.match(/^v(\d+)\./);
  return !!match && +match[1] >= 22;
}
