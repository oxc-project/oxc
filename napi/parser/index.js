const bindings = require('./bindings.js');
const { wrap } = require('./wrap.cjs');

module.exports.ParseResult = bindings.ParseResult;
module.exports.ExportExportNameKind = bindings.ExportExportNameKind;
module.exports.ExportImportNameKind = bindings.ExportImportNameKind;
module.exports.ExportLocalNameKind = bindings.ExportLocalNameKind;
module.exports.ImportNameKind = bindings.ImportNameKind;
module.exports.parseWithoutReturn = bindings.parseWithoutReturn;
module.exports.Severity = bindings.Severity;

module.exports.parseAsync = async function parseAsync(...args) {
  return wrap(await bindings.parseAsync(...args));
};

module.exports.parseSync = function parseSync(filename, sourceText, options) {
  if (options?.experimentalRawTransfer) {
    return parseSyncRaw(filename, sourceText, options);
  }
  return wrap(bindings.parseSync(filename, sourceText, options));
};

let buffer, encoder, deserializeJS, deserializeTS;

function parseSyncRaw(filename, sourceText, options) {
  if (!rawTransferSupported()) {
    throw new Error(
      '`experimentalRawTransfer` option is not supported on 32-bit or big-endian systems, ' +
        'versions of NodeJS prior to v22.0.0, versions of Deno prior to v2.0.0, and other runtimes',
    );
  }

  // Delete `experimentalRawTransfer` option
  let experimentalRawTransfer;
  ({ experimentalRawTransfer, ...options } = options);

  // Create buffer and `TextEncoder`
  if (!buffer) {
    buffer = createBuffer();
    encoder = new TextEncoder();
  }

  // Write source into start of buffer.
  // `TextEncoder` cannot write into a `Uint8Array` larger than 1 GiB,
  // so create a view into buffer of this size to write into.
  const sourceBuffer = new Uint8Array(buffer.buffer, buffer.byteOffset, ONE_GIB);
  const { read, written: sourceByteLen } = encoder.encodeInto(sourceText, sourceBuffer);
  if (read !== sourceText.length) {
    throw new Error('Failed to write source text into buffer');
  }

  // Parse
  bindings.parseSyncRaw(filename, buffer, sourceByteLen, options);

  // Deserialize.
  // We cannot lazily deserialize in the getters, because the buffer might be re-used to parse
  // another file before the getter is called.

  // (2 * 1024 * 1024 * 1024 - 12)
  const astTypeFlagPos = 2147483636;
  let isJsAst = buffer[astTypeFlagPos] === 0;

  // Lazy load deserializer, and deserialize buffer to JS objects
  let data;
  if (isJsAst) {
    if (!deserializeJS) deserializeJS = require('./generated/deserialize/js.js');
    data = deserializeJS(buffer, sourceText, sourceByteLen);

    // Add a line comment for hashbang
    const { hashbang } = data.program;
    if (hashbang !== null) {
      data.comments.unshift({ type: 'Line', value: hashbang.value, start: hashbang.start, end: hashbang.end });
    }
  } else {
    if (!deserializeTS) deserializeTS = require('./generated/deserialize/ts.js');
    data = deserializeTS(buffer, sourceText, sourceByteLen);
    // Note: Do not add line comment for hashbang, to match `@typescript-eslint/parser`.
    // See https://github.com/oxc-project/oxc/blob/ea784f5f082e4c53c98afde9bf983afd0b95e44e/napi/parser/src/lib.rs#L106-L130
  }

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

const ONE_GIB = 1 << 30,
  TWO_GIB = ONE_GIB * 2,
  SIX_GIB = ONE_GIB * 6;

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
