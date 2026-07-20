// Aligned buffer pool for raw transfer back.
//
// Copy of the buffer machinery in `napi/parser/src-js/raw-transfer/common.js`, with
// `getBufferOffset` provided by this package's own binding (precedent for a per-package copy:
// `apps/oxlint/src-js/package/parse.ts`). Sharing between napi packages is done by copying /
// code generation, never runtime imports.

import { getBufferOffset } from "../bindings.js";

// Buffer geometry. Must be identical to the constants generated into
// `napi/parser/src-js/generated/constants.js` by `oxc_ast_tools`.
// The `RawTransferBackGenerator` will emit this package's own generated copy;
// until then these are maintained by hand.
export const BLOCK_SIZE = 2147483632;
export const BLOCK_ALIGN = 4294967296;
export const BUFFER_SIZE = 2147483576;
export const ACTIVE_SIZE = 2147483560;

const ARRAY_BUFFER_SIZE = BLOCK_SIZE + BLOCK_ALIGN;
const ONE_GIB = 1 << 30;

// 2-tier buffer cache (tier 1 strong references, tier 2 `WeakRef`s), identical strategy to
// `napi/parser/src-js/raw-transfer/common.js` - see the explanation there.
const CLEAR_BUFFERS_TIMEOUT = 10_000; // 10 seconds
const buffers = [],
  oldBuffers = [];
let clearBuffersTimeout = null;

const textEncoder = new TextEncoder();

/**
 * Get a buffer (from cache if possible), and copy source text into it.
 *
 * @param {string|null|undefined} sourceText - Original source text, or `null` / `undefined`
 *   when encoding without source (comments unavailable, spans best-effort)
 * @returns {Object} - Object of form `{ buffer, sourceStartPos, sourceByteLen }`.
 *   - `buffer`: `Uint8Array` for the AST to be written into.
 *   - `sourceStartPos`: Position of first byte of source text in buffer (`ACTIVE_SIZE` if none).
 *   - `sourceByteLen`: Length of source text in UTF-8 bytes (`0` if none).
 */
export function acquireBuffer(sourceText) {
  // Cancel timeout for clearing buffers
  if (clearBuffersTimeout !== null) {
    clearTimeout(clearBuffersTimeout);
    clearBuffersTimeout = null;
  }

  // Revive any discarded buffers which have not yet been garbage collected
  if (oldBuffers.length > 0) {
    const revivedBuffers = [];
    for (let oldBuffer of oldBuffers) {
      oldBuffer = oldBuffer.deref();
      if (oldBuffer !== undefined) revivedBuffers.push(oldBuffer);
    }
    oldBuffers.length = 0;
    if (revivedBuffers.length > 0) buffers.unshift(...revivedBuffers);
  }

  // Reuse existing buffer, or create a new one
  const buffer = buffers.length > 0 ? buffers.pop() : createBuffer();

  if (sourceText == null || sourceText === "") {
    return { buffer, sourceStartPos: ACTIVE_SIZE, sourceByteLen: 0 };
  }

  // Write source into end of buffer.
  // Maximum size of a string encoded in UTF-8 is 3 x the length of the string in UTF-16 characters.
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

  return { buffer, sourceStartPos, sourceByteLen };
}

/**
 * Return buffer to cache, to be reused.
 * Set a timer to clear buffers.
 *
 * @param {Uint8Array} buffer - Buffer
 * @returns {undefined}
 */
export function returnBufferToCache(buffer) {
  buffers.push(buffer);

  if (clearBuffersTimeout !== null) clearTimeout(clearBuffersTimeout);
  clearBuffersTimeout = setTimeout(clearBuffersCache, CLEAR_BUFFERS_TIMEOUT);
  clearBuffersTimeout.unref();
}

/**
 * Downgrade buffers in tier 1 cache (`buffers`) to tier 2 (`oldBuffers`)
 * so they can be garbage collected.
 *
 * @returns {undefined}
 */
function clearBuffersCache() {
  clearBuffersTimeout = null;

  for (const buffer of buffers) {
    oldBuffers.push(new WeakRef(buffer));
  }
  buffers.length = 0;
}

/**
 * Create a `Uint8Array` which is 2 GiB in size, with its start aligned on 4 GiB.
 *
 * Same construction as `createBuffer` in `napi/parser/src-js/raw-transfer/common.js` -
 * see the explanation there. `buffer` / `int32` / `float64` views are `BUFFER_SIZE` bytes
 * (excluding allocator metadata + `ChunkFooter` so JS cannot clobber them); `block` is
 * `BLOCK_SIZE` bytes and is what gets passed to Rust.
 *
 * @returns {Uint8Array} - Buffer
 */
function createBuffer() {
  const arrayBuffer = new ArrayBuffer(ARRAY_BUFFER_SIZE);
  const offset = getBufferOffset(new Uint8Array(arrayBuffer));
  const buffer = new Uint8Array(arrayBuffer, offset, BUFFER_SIZE);
  buffer.int32 = new Int32Array(arrayBuffer, offset, BUFFER_SIZE / 4);
  buffer.float64 = new Float64Array(arrayBuffer, offset, BUFFER_SIZE / 8);
  buffer.block = new Uint8Array(arrayBuffer, offset, BLOCK_SIZE);
  return buffer;
}
