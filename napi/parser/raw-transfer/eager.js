'use strict';

const { parseSyncRawImpl, parseAsyncRawImpl, isJsAst, returnBufferToCache } = require('./common.js');

module.exports = { parseSyncRaw, parseAsyncRaw };

/**
 * Parse JS/TS source synchronously on current thread, using raw transfer to speed up deserialization.
 *
 * @param {string} filename - Filename
 * @param {string} sourceText - Source text of file
 * @param {Object} options - Parsing options
 * @returns {Object} - Object with property getters for `program`, `module`, `comments`, and `errors`
 */
function parseSyncRaw(filename, sourceText, options) {
  let _;
  ({ experimentalRawTransfer: _, ...options } = options);
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
function parseAsyncRaw(filename, sourceText, options) {
  let _;
  ({ experimentalRawTransfer: _, ...options } = options);
  return parseAsyncRawImpl(filename, sourceText, options, deserialize);
}

let deserializeJS = null, deserializeTS = null;

/**
 * Deserialize whole AST from buffer.
 *
 * @param {Uint8Array} buffer - Buffer containing AST in raw form
 * @param {string} sourceText - Source for the file
 * @param {number} sourceByteLen - Length of source text in UTF-8 bytes
 * @returns {Object} - Object with property getters for `program`, `module`, `comments`, and `errors`
 */
function deserialize(buffer, sourceText, sourceByteLen) {
  // Lazy load deserializer, and deserialize buffer to JS objects
  let data;
  if (isJsAst(buffer)) {
    if (deserializeJS === null) deserializeJS = require('../generated/deserialize/js.js');
    data = deserializeJS(buffer, sourceText, sourceByteLen);

    // Add a line comment for hashbang
    const { hashbang } = data.program;
    if (hashbang !== null) {
      data.comments.unshift({ type: 'Line', value: hashbang.value, start: hashbang.start, end: hashbang.end });
    }
  } else {
    if (deserializeTS === null) deserializeTS = require('../generated/deserialize/ts.js');
    data = deserializeTS(buffer, sourceText, sourceByteLen);
    // Note: Do not add line comment for hashbang, to match `@typescript-eslint/parser`.
    // See https://github.com/oxc-project/oxc/blob/ea784f5f082e4c53c98afde9bf983afd0b95e44e/napi/parser/src/lib.rs#L106-L130
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
