const bindings = require('./bindings.js');
const deserializeJS = require('./deserialize-js.js');
const deserializeTS = require('./deserialize-ts.js');

module.exports.ParseResult = bindings.ParseResult;
module.exports.ExportExportNameKind = bindings.ExportExportNameKind;
module.exports.ExportImportNameKind = bindings.ExportImportNameKind;
module.exports.ExportLocalNameKind = bindings.ExportLocalNameKind;
module.exports.ImportNameKind = bindings.ImportNameKind;
module.exports.parseWithoutReturn = bindings.parseWithoutReturn;
module.exports.Severity = bindings.Severity;

function wrap(result) {
  let program, module, comments, errors;
  return {
    get program() {
      if (!program) {
        // Note: This code is repeated in `wasm/parser/update-bindings.mjs` and `crates/oxc-wasm/update-bindings.mjs`.
        // Any changes should be applied in those 2 scripts too.
        program = JSON.parse(result.program, function(key, value) {
          // Set `value` field of `Literal`s for `BigInt`s and `RegExp`s.
          // This is not possible to do on Rust side, as neither can be represented correctly in JSON.
          if (value === null && key === 'value' && Object.hasOwn(this, 'type') && this.type === 'Literal') {
            if (Object.hasOwn(this, 'bigint')) {
              return BigInt(this.bigint);
            }
            if (Object.hasOwn(this, 'regex')) {
              const { regex } = this;
              try {
                return RegExp(regex.pattern, regex.flags);
              } catch (_err) {
                // Invalid regexp, or valid regexp using syntax not supported by this version of NodeJS
              }
            }
          }
          return value;
        });
      }
      return program;
    },
    get module() {
      if (!module) module = result.module;
      return module;
    },
    get comments() {
      if (!comments) comments = result.comments;
      return comments;
    },
    get errors() {
      if (!errors) errors = result.errors;
      return errors;
    },
  };
}

module.exports.parseAsync = async function parseAsync(...args) {
  return wrap(await bindings.parseAsync(...args));
};

module.exports.parseSync = function parseSync(filename, sourceText, options) {
  if (options?.experimentalRawTransfer) {
    return parseSyncRaw(filename, sourceText, options);
  }

  return wrap(bindings.parseSync(filename, sourceText, options));
};

let buffer, encoder;

function parseSyncRaw(filename, sourceText, options) {
  if (!rawTransferSupported()) {
    throw new Error('`experimentalRawTransfer` option is not supported on 32-bit or big-endian systems');
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

  const data = isJsAst
    ? deserializeJS(buffer, sourceText, sourceByteLen)
    : deserializeTS(buffer, sourceText, sourceByteLen);

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
// Raw transfer is only available on 64-bit little-endian systems.
function rawTransferSupported() {
  if (rawTransferIsSupported === null) rawTransferIsSupported = bindings.rawTransferSupported();
  return rawTransferIsSupported;
}

module.exports.rawTransferSupported = rawTransferSupported;
