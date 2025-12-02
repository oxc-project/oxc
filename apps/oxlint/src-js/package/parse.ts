import {
  getBufferOffset,
  rawTransferSupported as rawTransferSupportedBinding,
  parseRawSync,
} from "../bindings.js";
import { debugAssert, debugAssertIsNonNull } from "../utils/asserts.ts";
import { buffers } from "../plugins/lint.ts";
import { BUFFER_SIZE, BUFFER_ALIGN, DATA_POINTER_POS_32 } from "../generated/constants.ts";

import type { BufferWithArrays } from "../plugins/types.ts";

// Size array buffer for raw transfer
const ARRAY_BUFFER_SIZE = BUFFER_SIZE + BUFFER_ALIGN;

// 1 GiB
const ONE_GIB = 1 << 30;

// Text encoder for encoding source text into buffer
const textEncoder = new TextEncoder();

// Buffer for raw transfer
let buffer: BufferWithArrays | null = null;

// Whether raw transfer is supported
let rawTransferIsSupported: boolean | null = null;

/**
 * Parser source text into buffer.
 * @param path - Path of file to parse
 * @param sourceText - Source text to parse
 * @throws {Error} If raw transfer is not supported on this platform, or parsing failed
 */
export function parse(path: string, sourceText: string) {
  if (!rawTransferSupported()) {
    throw new Error(
      "`RuleTester` is not supported on 32-bit or big-endian systems, versions of NodeJS prior to v22.0.0, " +
        "versions of Deno prior to v2.0.0, or other runtimes",
    );
  }

  // Initialize buffer, if not already
  if (buffer === null) initBuffer();
  debugAssertIsNonNull(buffer);

  // Write source into start of buffer.
  // `TextEncoder` cannot write into a `Uint8Array` larger than 1 GiB,
  // so create a view into buffer of this size to write into.
  const sourceBuffer = new Uint8Array(buffer.buffer, buffer.byteOffset, ONE_GIB);
  const { read, written: sourceByteLen } = textEncoder.encodeInto(sourceText, sourceBuffer);
  if (read !== sourceText.length) throw new Error("Failed to write source text into buffer");

  // Parse into buffer
  parseRawSync(path, buffer, sourceByteLen);

  // Check parsing succeeded.
  // 0 is used as sentinel value to indicate parsing failed.
  // TODO: Get parsing error details from Rust to display nicely.
  const programOffset = buffer.uint32[DATA_POINTER_POS_32];
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
 * Note: On systems with virtual memory, this only consumes 6 GiB of *virtual* memory.
 * It does not consume physical memory until data is actually written to the `Uint8Array`.
 * Physical memory consumed corresponds to the quantity of data actually written.
 */
export function initBuffer() {
  // Create buffer
  const arrayBuffer = new ArrayBuffer(ARRAY_BUFFER_SIZE);
  const offset = getBufferOffset(new Uint8Array(arrayBuffer));
  buffer = new Uint8Array(arrayBuffer, offset, BUFFER_SIZE) as BufferWithArrays;
  buffer.uint32 = new Uint32Array(arrayBuffer, offset, BUFFER_SIZE / 4);
  buffer.float64 = new Float64Array(arrayBuffer, offset, BUFFER_SIZE / 8);

  // Store in `buffers`, at index 0
  debugAssert(buffers.length === 0);
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
