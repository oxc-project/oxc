'use strict';

const { parseSyncRawImpl, parseAsyncRawImpl, returnBufferToCache } = require('./index.js');

module.exports = { parseSyncLazy, parseAsyncLazy };

function parseSyncLazy(filename, sourceText, options) {
  return parseSyncRawImpl(filename, sourceText, options, construct);
}

function parseAsyncLazy(filename, sourceText, options) {
  return parseAsyncRawImpl(filename, sourceText, options, construct);
}

// Registry for buffers which are held by lazily-deserialized ASTs.
// Returns buffer to cache when the `ast` wrapper is garbage collected.
//
// Check for existence of `FinalizationRegistry`, to avoid errors on old versions of NodeJS
// which don't support it. e.g. Prettier supports NodeJS v14.
// Raw transfer is disabled on NodeJS before v22, so it doesn't matter if this is `null` on old NodeJS
// - it'll never be accessed in that case.
const bufferRecycleRegistry = typeof FinalizationRegistry === 'undefined'
  ? null
  : new FinalizationRegistry(returnBufferToCache);

let constructLazyData = null, TOKEN;

// Get an object with getters which lazy deserialize AST from buffer
function construct(buffer, sourceText, sourceLen) {
  // Lazy load deserializer, and get `TOKEN` to store in `ast` objects
  if (constructLazyData === null) {
    ({ construct: constructLazyData, TOKEN } = require('../generated/deserialize/lazy.js'));
  }

  // Create AST object
  const sourceIsAscii = sourceText.length === sourceLen;
  const ast = { buffer, sourceText, sourceLen, sourceIsAscii, nodes: new Map(), token: TOKEN };

  // Register `ast` with the recycle registry so buffer is returned to cache
  // when `ast` is garbage collected
  bufferRecycleRegistry.register(ast, buffer, ast);

  // Get root data class instance
  const data = constructLazyData(ast);

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
    dispose: dispose.bind(null, ast),
  };
}

// Dispose of this AST.
//
// After calling this method, trying to read any nodes from this AST may cause an error.
//
// Buffer is returned to the cache to be reused.
//
// The buffer would be returned to the cache anyway, once all nodes of the AST are garbage collected,
// but calling `dispose` is preferable, as it will happen immediately.
// Otherwise, garbage collector may take time to collect the `ast` object, and new buffers may be created
// in the meantime, when we could have reused this one.
function dispose(ast) {
  // Return buffer to cache to be reused
  returnBufferToCache(ast.buffer);

  // Remove connection between `ast` and the buffer
  ast.buffer = null;

  // Clear other contents of `ast`, so they can be garbage collected
  ast.sourceText = null;
  ast.nodes = null;

  // Remove `ast` from recycling register.
  // When `ast` is garbage collected, there's no longer any action to be taken.
  bufferRecycleRegistry.unregister(ast);
}
