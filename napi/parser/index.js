const { availableParallelism } = require('node:os');
const bindings = require('./bindings.js');
const { wrap } = require('./wrap.cjs');

module.exports.ParseResult = bindings.ParseResult;
module.exports.ExportExportNameKind = bindings.ExportExportNameKind;
module.exports.ExportImportNameKind = bindings.ExportImportNameKind;
module.exports.ExportLocalNameKind = bindings.ExportLocalNameKind;
module.exports.ImportNameKind = bindings.ImportNameKind;
module.exports.parseWithoutReturn = bindings.parseWithoutReturn;
module.exports.Severity = bindings.Severity;

module.exports.parseAsync = async function parseAsync(filename, sourceText, options) {
  if (options?.experimentalRawTransfer) return await parseAsyncRaw(filename, sourceText, options);
  return wrap(await bindings.parseAsync(filename, sourceText, options));
};

module.exports.parseSync = function parseSync(filename, sourceText, options) {
  if (options?.experimentalRawTransfer) return parseSyncRaw(filename, sourceText, options);
  return wrap(bindings.parseSync(filename, sourceText, options));
};

function parseSyncRaw(filename, sourceText, options) {
  const { buffer, sourceByteLen, options: optionsAmended } = prepareRaw(sourceText, options);
  bindings.parseSyncRaw(filename, buffer, sourceByteLen, optionsAmended);
  return deserialize(buffer, sourceText, sourceByteLen);
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
let availableCores = availableParallelism();
const queue = [];

async function parseAsyncRaw(filename, sourceText, options) {
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
  const { buffer, sourceByteLen, options: optionsAmended } = prepareRaw(sourceText, options);
  await bindings.parseAsyncRaw(filename, buffer, sourceByteLen, optionsAmended);
  const ret = deserialize(buffer, sourceText, sourceByteLen);

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

  return ret;
}

const ONE_GIB = 1 << 30,
  TWO_GIB = ONE_GIB * 2,
  SIX_GIB = ONE_GIB * 6;

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
const buffers = [], oldBuffers = [];

let encoder = null, deserializeJS = null, deserializeTS = null, clearBuffersTimeout = null;

// Get a buffer (from cache if possible), copy source text into it, and amend options object
function prepareRaw(sourceText, options) {
  if (!rawTransferSupported()) {
    throw new Error(
      '`experimentalRawTransfer` option is not supported on 32-bit or big-endian systems, ' +
        'versions of NodeJS prior to v22.0.0, versions of Deno prior to v2.0.0, and other runtimes',
    );
  }

  // Delete `experimentalRawTransfer` option
  let _;
  ({ experimentalRawTransfer: _, ...options } = options);

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

  // Get/create `TextEncoder`
  if (encoder === null) encoder = new TextEncoder();

  // Write source into start of buffer.
  // `TextEncoder` cannot write into a `Uint8Array` larger than 1 GiB,
  // so create a view into buffer of this size to write into.
  const sourceBuffer = new Uint8Array(buffer.buffer, buffer.byteOffset, ONE_GIB);
  const { read, written: sourceByteLen } = encoder.encodeInto(sourceText, sourceBuffer);
  if (read !== sourceText.length) throw new Error('Failed to write source text into buffer');

  return { buffer, sourceByteLen, options };
}

// Deserialize AST from buffer
function deserialize(buffer, sourceText, sourceByteLen) {
  // 2147483636 = (2 * 1024 * 1024 * 1024) - 12
  // i.e. 12 bytes from end of 2 GiB buffer
  const isJsAst = buffer[2147483636] === 0;

  // Lazy load deserializer, and deserialize buffer to JS objects
  let data;
  if (isJsAst) {
    if (deserializeJS === null) deserializeJS = require('./generated/deserialize/js.js');
    data = deserializeJS(buffer, sourceText, sourceByteLen);

    // Add a line comment for hashbang
    const { hashbang } = data.program;
    if (hashbang !== null) {
      data.comments.unshift({ type: 'Line', value: hashbang.value, start: hashbang.start, end: hashbang.end });
    }
  } else {
    if (deserializeTS === null) deserializeTS = require('./generated/deserialize/ts.js');
    data = deserializeTS(buffer, sourceText, sourceByteLen);
    // Note: Do not add line comment for hashbang, to match `@typescript-eslint/parser`.
    // See https://github.com/oxc-project/oxc/blob/ea784f5f082e4c53c98afde9bf983afd0b95e44e/napi/parser/src/lib.rs#L106-L130
  }

  // Return buffer to cache, to be reused
  buffers.push(buffer);

  // Set timer to clear buffers
  if (clearBuffersTimeout !== null) clearTimeout(clearBuffersTimeout);
  clearBuffersTimeout = setTimeout(clearBuffersCache, CLEAR_BUFFERS_TIMEOUT);
  clearBuffersTimeout.unref();

  // We cannot lazily deserialize in the getters, because the buffer might be re-used to parse
  // another file before the getter is called.
  return {
    get program() {
      return data.program;
    },
    get module() {
      return data.module;
    },
    get comments() {
      return data.comments;
    },
    get errors() {
      return data.errors;
    },
  };
}

// Downgrade buffers in tier 1 cache (`buffers`) to tier 2 (`oldBuffers`),
// so they can be garbage collected
function clearBuffersCache() {
  clearBuffersTimeout = null;

  for (const buffer of buffers) {
    oldBuffers.push(new WeakRef(buffer));
  }
  buffers.length = 0;
}

// Create a `Uint8Array` which is 2 GiB in size, with its start aligned on 4 GiB.
//
// Achieve this by creating a 6 GiB `ArrayBuffer`, getting the offset within it that's aligned to 4 GiB,
// chopping off that number of bytes from the start, and shortening to 2 GiB.
//
// It's always possible to obtain a 2 GiB slice aligned on 4 GiB within a 6 GiB buffer,
// no matter how the 6 GiB buffer is aligned.
//
// Note: On systems with virtual memory, this only consumes 6 GiB of *virtual* memory.
// It does not consume physical memory until data is actually written to the `Uint8Array`.
// Physical memory consumed corresponds to the quantity of data actually written.
function createBuffer() {
  const arrayBuffer = new ArrayBuffer(SIX_GIB);
  const offset = bindings.getBufferOffset(new Uint8Array(arrayBuffer));
  return new Uint8Array(arrayBuffer, offset, TWO_GIB);
}

let rawTransferIsSupported = null;

// Returns `true` if `experimentalRawTransfer` is option is supported.
//
// Raw transfer is only supported on 64-bit little-endian systems,
// and NodeJS >= v22.0.0 or Deno >= v2.0.0.
//
// Versions of NodeJS prior to v22.0.0 do not support creating an `ArrayBuffer` larger than 4 GiB.
// Bun (as at v1.2.4) also does not support creating an `ArrayBuffer` larger than 4 GiB.
// Support on Deno v1 is unknown and it's EOL, so treating Deno before v2.0.0 as unsupported.
function rawTransferSupported() {
  if (rawTransferIsSupported === null) {
    rawTransferIsSupported = rawTransferRuntimeSupported() && bindings.rawTransferSupported();
  }
  return rawTransferIsSupported;
}

module.exports.rawTransferSupported = rawTransferSupported;

// Checks copied from:
// https://github.com/unjs/std-env/blob/ab15595debec9e9115a9c1d31bc7597a8e71dbfd/src/runtimes.ts
// MIT license: https://github.com/unjs/std-env/blob/ab15595debec9e9115a9c1d31bc7597a8e71dbfd/LICENCE
function rawTransferRuntimeSupported() {
  let global;
  try {
    global = globalThis;
  } catch (e) {
    return false;
  }

  const isBun = !!global.Bun || !!global.process?.versions?.bun;
  if (isBun) return false;

  const isDeno = !!global.Deno;
  if (isDeno) {
    const match = Deno.version?.deno?.match(/^(\d+)\./);
    return !!match && match[1] * 1 >= 2;
  }

  const isNode = global.process?.release?.name === 'node';
  if (!isNode) return false;

  const match = process.version?.match(/^v(\d+)\./);
  return !!match && match[1] * 1 >= 22;
}
