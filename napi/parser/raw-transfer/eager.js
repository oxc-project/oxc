'use strict';

const { parseSyncRawImpl, parseAsyncRawImpl, isJsAst, returnBufferToCache } = require('./index.js');

module.exports = { parseSyncRaw, parseAsyncRaw };

function parseSyncRaw(filename, sourceText, options) {
  return parseSyncRawImpl(filename, sourceText, options, deserialize);
}

function parseAsyncRaw(filename, sourceText, options) {
  return parseAsyncRawImpl(filename, sourceText, options, deserialize);
}

let deserializeJS = null, deserializeTS = null;

// Deserialize whole AST from buffer eagerly
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
