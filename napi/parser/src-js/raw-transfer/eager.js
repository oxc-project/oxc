import { createRequire } from "node:module";
import { isJsAst, parseAsyncRawImpl, parseSyncRawImpl, returnBufferToCache } from "./common.js";

const require = createRequire(import.meta.url);

/**
 * Parse JS/TS source synchronously on current thread, using raw transfer to speed up deserialization.
 *
 * @param {string} filename - Filename
 * @param {string} sourceText - Source text of file
 * @param {Object} options - Parsing options
 * @returns {Object} - Object with property getters for `program`, `module`, `comments`, and `errors`
 */
export function parseSyncRaw(filename, sourceText, options) {
  return parseSyncRawImpl(filename, sourceText, options, deserialize);
}

/**
 * Parse JS/TS source asynchronously, using raw transfer to speed up deserialization.
 *
 * Note that not all of the workload can happen on a separate thread.
 * Parsing on Rust side does happen in a separate thread, but deserialization of the AST to JS objects
 * has to happen on current thread. This synchronous deserialization work typically outweighs
 * the asynchronous parsing by a factor of around 3.
 *
 * i.e. the majority of the workload cannot be parallelized by using this method.
 *
 * Generally `parseSyncRaw` is preferable to use as it does not have the overhead of spawning a thread.
 * If you need to parallelize parsing multiple files, it is recommended to use worker threads.
 *
 * @param {string} filename - Filename
 * @param {string} sourceText - Source text of file
 * @param {Object} options - Parsing options
 * @returns {Object} - Object with property getters for `program`, `module`, `comments`, and `errors`
 */
export function parse(filename, sourceText, options) {
  return parseAsyncRawImpl(filename, sourceText, options, deserialize);
}

// Deserializers are large files, so lazy-loaded.
// `deserialize` functions are stored in this array once loaded.
// Index into these arrays is `isJs * 1 + range * 2 + experimentalParent * 4`.
const deserializers = [null, null, null, null, null, null, null, null];
const deserializerNames = [
  "ts",
  "js",
  "ts_range",
  "js_range",
  "ts_parent",
  "js_parent",
  "ts_range_parent",
  "js_range_parent",
];

/**
 * Deserialize whole AST from buffer.
 *
 * @param {Uint8Array} buffer - Buffer containing AST in raw form
 * @param {string} sourceText - Source for the file
 * @param {number} sourceByteLen - Length of source text in UTF-8 bytes
 * @param {Object} options - Parsing options
 * @returns {Object} - Object with property getters for `program`, `module`, `comments`, and `errors`
 */
function deserialize(buffer, sourceText, sourceByteLen, options) {
  const isJs = isJsAst(buffer),
    range = !!options.range,
    parent = !!options.experimentalParent;

  // Lazy load deserializer, and deserialize buffer to JS objects
  const deserializerIndex = +isJs | (+range << 1) | (+parent << 2);
  let deserializeThis = deserializers[deserializerIndex];
  if (deserializeThis === null) {
    deserializeThis = deserializers[deserializerIndex] = require(
      `../../generated/deserialize/${deserializerNames[deserializerIndex]}.js`,
    ).deserialize;
  }

  const data = deserializeThis(buffer, sourceText, sourceByteLen);

  // Add a line comment for hashbang if JS.
  // Do not add comment if TS, to match `@typescript-eslint/parser`.
  // See https://github.com/oxc-project/oxc/blob/ea784f5f082e4c53c98afde9bf983afd0b95e44e/napi/parser/src/lib.rs#L106-L130
  if (isJs) {
    const { hashbang } = data.program;
    if (hashbang !== null) {
      data.comments.unshift(
        range
          ? {
              type: "Line",
              value: hashbang.value,
              start: hashbang.start,
              end: hashbang.end,
              range: hashbang.range,
            }
          : { type: "Line", value: hashbang.value, start: hashbang.start, end: hashbang.end },
      );
    }
  }

  // Return buffer to cache, to be reused
  returnBufferToCache(buffer);

  // We cannot lazily deserialize in the getters, because the buffer might be re-used to parse
  // another file before the getter is called
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
