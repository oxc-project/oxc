import os from "node:os";
import { BUFFER_ALIGN, BUFFER_SIZE, IS_TS_FLAG_POS } from "../../generated/constants.js";
import {
  getBufferOffset,
  parseRaw as parseRawBinding,
  parseRawSync as parseRawSyncBinding,
} from "../bindings.js";
import { rawTransferSupported } from "./supported.js";

// Throw an error if running on a platform which raw transfer doesn't support.
//
// Note: This module is lazy-loaded only when user calls `parseSync` or `parseAsync` with
// `experimentalRawTransfer` or `experimentalLazy` options, or calls `experimentalGetLazyVisitor`.
if (!rawTransferSupported()) {
  throw new Error(
    "`experimentalRawTransfer` and `experimentalLazy` options are not supported " +
      "on 32-bit or big-endian systems, versions of NodeJS prior to v22.0.0, " +
      "versions of Deno prior to v2.0.0, or other runtimes",
  );
}

/**
 * Parse JS/TS source synchronously on current thread using raw transfer.
 *
 * Convert the buffer returned by Rust to a JS object with provided `convert` function.
 *
 * This function contains logic shared by both `parseSyncRaw` and `parseSyncLazy`.
 *
 * @param {string} filename - Filename
 * @param {string} sourceText - Source text of file
 * @param {Object} options - Parsing options
 * @param {function} convert - Function to convert the buffer returned from Rust into a JS object
 * @returns {Object} - The return value of `convert`
 */
export function parseSyncRawImpl(filename, sourceText, options, convert) {
  const { buffer, sourceByteLen } = prepareRaw(sourceText);
  parseRawSyncBinding(filename, buffer, sourceByteLen, options);
  return convert(buffer, sourceText, sourceByteLen, options);
}

// User should not schedule more async tasks than there are available CPUs, as it hurts performance,
// but it's a common mistake in async JS code to do exactly that.
//
// That anti-pattern looks like this when applied to Oxc:
//
// ```js
// const asts = await Promise.all(
//   files.map(
//     async (filename) => {
//       const sourceText = await fs.readFile(filename, 'utf8');
//       const ast = await oxc.parseAsync(filename, sourceText);
//       return ast;
//     }
//   )
// );
// ```
//
// In most cases, that'd just result in a bit of degraded performance, and higher memory use because
// of loading sources into memory prematurely.
//
// However, raw transfer uses a 6 GiB buffer for each parsing operation.
// Most of the memory pages in those buffers are never touched, so this does not consume a huge amount
// of physical memory, but it does still consume virtual memory.
//
// If we allowed creating a large number of 6 GiB buffers simultaneously, it would quickly consume
// virtual memory space and risk memory exhaustion. The code above would exhaust all of bottom half
// (heap) of 48-bit virtual memory space if `files.length >= 21_845`. This is not a number which
// is unrealistic in real world code.
//
// To guard against this possibility, we implement a simple queue.
// No more than `os.availableParallelism()` files can be parsed simultaneously, and any further calls to
// `parseAsyncRaw` will be put in a queue, to execute once other tasks complete.
//
// Fallback to `os.cpus().length` on versions of NodeJS prior to v18.14.0, which do not support
// `os.availableParallelism`.
let availableCores = os.availableParallelism ? os.availableParallelism() : os.cpus().length;
const queue = [];

/**
 * Parse JS/TS source asynchronously using raw transfer.
 *
 * Convert the buffer returned by Rust to a JS object with provided `convert` function.
 *
 * Queues up parsing operations if more calls than number of CPU cores (see above).
 *
 * This function contains logic shared by both `parseAsyncRaw` and `parseAsyncLazy`.
 *
 * @param {string} filename - Filename
 * @param {string} sourceText - Source text of file
 * @param {Object} options - Parsing options
 * @param {function} convert - Function to convert the buffer returned from Rust into a JS object
 * @returns {Object} - The return value of `convert`
 */
export async function parseAsyncRawImpl(filename, sourceText, options, convert) {
  // Wait for a free CPU core if all CPUs are currently busy.
  //
  // Note: `availableCores` is NOT decremented if have to wait in the queue first,
  // and NOT incremented when parsing completes and it runs next task in the queue.
  //
  // This is to avoid a race condition if `parseAsyncRaw` is called during the microtick in between
  // `resolve` being called below, and the promise resolving here. In that case the new task could
  // start running, and then the promise resolves, and the queued task also starts running.
  // We'd then have `availableParallelism() + 1` tasks running simultaneously. Potentially, this could
  // happen repeatedly, with the number of tasks running simultaneously ever-increasing.
  if (availableCores === 0) {
    // All CPU cores are busy. Put this task in queue and wait for capacity to become available.
    await new Promise((resolve, _) => {
      queue.push(resolve);
    });
  } else {
    // A CPU core is available. Mark core as busy, and run parsing now.
    availableCores--;
  }

  // Parse
  const { buffer, sourceByteLen } = prepareRaw(sourceText);
  await parseRawBinding(filename, buffer, sourceByteLen, options);
  const data = convert(buffer, sourceText, sourceByteLen, options);

  // Free the CPU core
  if (queue.length > 0) {
    // Some further tasks waiting in queue. Run the next one.
    // Do not increment `availableCores` (see above).
    const resolve = queue.shift();
    resolve();
  } else {
    // No tasks waiting in queue. This CPU is now free.
    availableCores++;
  }

  return data;
}

const ARRAY_BUFFER_SIZE = BUFFER_SIZE + BUFFER_ALIGN;
const ONE_GIB = 1 << 30;

// We keep a cache of buffers for raw transfer, so we can reuse them as much as possible.
//
// When processing multiple files, it's ideal if can reuse an existing buffer, as it's more likely to
// be warm in CPU cache, it avoids allocations, and it saves work for the garbage collector.
//
// However, we also don't want to keep a load of large buffers around indefinitely using up memory,
// if they're not going to be used again.
//
// We have no knowledge of what pattern over time user may process files in (could be lots in quick
// succession, or more occasionally in a long-running process). So we try to use flexible caching
// strategy which is adaptable to many usage patterns.
//
// We use a 2-tier cache.
// Tier 1 uses strong references, tier 2 uses weak references.
//
// When parsing is complete and the buffer is no longer in use, push it to `buffers` (tier 1 cache).
// Set a timer to clear the cache when no activity for 10 seconds.
//
// When the timer expires, move all the buffers from tier 1 cache into `oldBuffers` (tier 2).
// They are stored there as `WeakRef`s, so the garbage collector is free to reclaim them.
//
// On the next call to `parseSync` or `parseAsync`, promote any buffers in tier 2 cache which were not
// already garbage collected back into tier 1 cache. This is on assumption that parsing one file
// indicates parsing as a whole is an ongoing process, and there will likely be further calls to
// `parseSync` / `parseAsync` in future.
//
// The weak tier 2 cache is because V8 does not necessarily free memory as soon as it's able to be
// freed. We don't want to block it from freeing memory, but if it's not done that yet, there's no
// point creating a new buffer, when one already exists.
const CLEAR_BUFFERS_TIMEOUT = 10_000; // 10 seconds
const buffers = [],
  oldBuffers = [];
let clearBuffersTimeout = null;

const textEncoder = new TextEncoder();

/**
 * Get a buffer (from cache if possible), and copy source text into it.
 *
 * @param {string} sourceText - Source text of file
 * @returns {Object} - Object of form `{ buffer, sourceByteLen }`.
 *   - `buffer`: `Uint8Array` containing the AST in raw form.
 *   - `sourceByteLen`: Length of source text in UTF-8 bytes
 *     (which may not be equal to `sourceText.length` if source contains non-ASCII characters).
 */
export function prepareRaw(sourceText) {
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

  // Write source into start of buffer.
  // `TextEncoder` cannot write into a `Uint8Array` larger than 1 GiB,
  // so create a view into buffer of this size to write into.
  const sourceBuffer = new Uint8Array(buffer.buffer, buffer.byteOffset, ONE_GIB);
  const { read, written: sourceByteLen } = textEncoder.encodeInto(sourceText, sourceBuffer);
  if (read !== sourceText.length) throw new Error("Failed to write source text into buffer");

  return { buffer, sourceByteLen };
}

/**
 * Get if AST should be parsed as JS or TS.
 * Rust side sets a `bool` in this position in buffer which is `true` if TS.
 *
 * @param {Uint8Array} buffer - Buffer containing AST in raw form
 * @returns {boolean} - `true` if AST is JS, `false` if TS
 */
export function isJsAst(buffer) {
  return buffer[IS_TS_FLAG_POS] === 0;
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
 * Achieve this by creating a 6 GiB `ArrayBuffer`, getting the offset within it that's aligned to 4 GiB,
 * chopping off that number of bytes from the start, and shortening to 2 GiB.
 *
 * It's always possible to obtain a 2 GiB slice aligned on 4 GiB within a 6 GiB buffer,
 * no matter how the 6 GiB buffer is aligned.
 *
 * Note: On systems with virtual memory, this only consumes 6 GiB of *virtual* memory.
 * It does not consume physical memory until data is actually written to the `Uint8Array`.
 * Physical memory consumed corresponds to the quantity of data actually written.
 *
 * @returns {Uint8Array} - Buffer
 */
function createBuffer() {
  const arrayBuffer = new ArrayBuffer(ARRAY_BUFFER_SIZE);
  const offset = getBufferOffset(new Uint8Array(arrayBuffer));
  const buffer = new Uint8Array(arrayBuffer, offset, BUFFER_SIZE);
  buffer.uint32 = new Uint32Array(arrayBuffer, offset, BUFFER_SIZE / 4);
  buffer.float64 = new Float64Array(arrayBuffer, offset, BUFFER_SIZE / 8);
  return buffer;
}
