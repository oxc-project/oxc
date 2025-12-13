import { DATA_POINTER_POS_32, PROGRAM_OFFSET } from "../../generated/constants.js";
import { RawTransferData } from "../../generated/lazy/constructors.js";
import { walkProgram } from "../../generated/lazy/walk.js";
import { parseAsyncRawImpl, parseSyncRawImpl, returnBufferToCache } from "./common.js";
import { TOKEN } from "./lazy-common.js";
import { getVisitorsArr } from "./visitor.js";
export { Visitor } from "./visitor.js";

/**
 * Parse JS/TS source synchronously on current thread.
 *
 * The data in buffer is not deserialized. Is deserialized to JS objects lazily, when accessing the
 * properties of objects.
 *
 * e.g. `program` in returned object is an instance of `Program` class, with getters for `start`, `end`,
 * `body` etc.
 *
 * Returned object contains a `visit` function which can be used to visit the AST with a `Visitor`
 * (`Visitor` class can be obtained by calling `experimentalGetLazyVisitor()`).
 *
 * Returned object contains a `dispose` method. When finished with this AST, it's advisable to call
 * `dispose`, to return the buffer to the cache, so it can be reused.
 * Garbage collector should do this anyway at some point, but on an unpredictable schedule,
 * so it's preferable to call `dispose` manually, to ensure the buffer can be reused immediately.
 *
 * @param {string} filename - Filename
 * @param {string} sourceText - Source text of file
 * @param {Object} options - Parsing options
 * @returns {Object} - Object with property getters for `program`, `module`, `comments`, and `errors`,
 *   and `dispose` and `visit` methods
 */
export function parseSyncLazy(filename, sourceText, options) {
  return parseSyncRawImpl(filename, sourceText, options, construct);
}

/**
 * Parse JS/TS source asynchronously on a separate thread.
 *
 * The data in buffer is not deserialized. Is deserialized to JS objects lazily, when accessing the
 * properties of objects.
 *
 * e.g. `program` in returned object is an instance of `Program` class, with getters for `start`, `end`,
 * `body` etc.
 *
 * Because this function does not deserialize the AST, unlike `parse`, very little work happens
 * on current thread in this function. Deserialization work only occurs when properties of the objects
 * are accessed.
 *
 * Returned object contains a `visit` function which can be used to visit the AST with a `Visitor`
 * (`Visitor` class can be obtained by calling `experimentalGetLazyVisitor()`).
 *
 * Returned object contains a `dispose` method. When finished with this AST, it's advisable to call
 * `dispose`, to return the buffer to the cache, so it can be reused.
 * Garbage collector should do this anyway at some point, but on an unpredictable schedule,
 * so it's preferable to call `dispose` manually, to ensure the buffer can be reused immediately.
 *
 * @param {string} filename - Filename
 * @param {string} sourceText - Source text of file
 * @param {Object} options - Parsing options
 * @returns {Object} - Object with property getters for `program`, `module`, `comments`, and `errors`,
 *   and `dispose` and `visit` methods
 */
export function parse(filename, sourceText, options) {
  return parseAsyncRawImpl(filename, sourceText, options, construct);
}

// Registry for buffers which are held by lazily-deserialized ASTs.
// Returns buffer to cache when the `ast` wrapper is garbage collected.
//
// Check for existence of `FinalizationRegistry`, to avoid errors on old versions of NodeJS
// which don't support it. e.g. Prettier supports NodeJS v14.
// Raw transfer is disabled on NodeJS before v22, so it doesn't matter if this is `null` on old NodeJS
// - it'll never be accessed in that case.
const bufferRecycleRegistry =
  typeof FinalizationRegistry === "undefined"
    ? null
    : new FinalizationRegistry(returnBufferToCache);

/**
 * Get an object with getters which lazy deserialize AST and other data from buffer.
 *
 * Object also includes `dispose` and `visit` functions.
 *
 * @param {Uint8Array} buffer - Buffer containing AST in raw form
 * @param {string} sourceText - Source for the file
 * @param {number} sourceByteLen - Length of source text in UTF-8 bytes
 * @param {Object} _options - Parsing options
 * @returns {Object} - Object with property getters for `program`, `module`, `comments`, and `errors`,
 *   and `dispose` and `visit` methods
 */
function construct(buffer, sourceText, sourceByteLen, _options) {
  // Create AST object
  const sourceIsAscii = sourceText.length === sourceByteLen;
  const ast = { buffer, sourceText, sourceByteLen, sourceIsAscii, nodes: new Map(), token: TOKEN };

  // Register `ast` with the recycle registry so buffer is returned to cache
  // when `ast` is garbage collected
  bufferRecycleRegistry.register(ast, buffer, ast);

  // Get root data class instance
  const rawDataPos = buffer.uint32[DATA_POINTER_POS_32];
  const data = new RawTransferData(rawDataPos, ast);

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
    visit(visitor) {
      walkProgram(rawDataPos + PROGRAM_OFFSET, ast, getVisitorsArr(visitor));
    },
  };
}

/**
 * Dispose of this AST.
 *
 * After calling this method, trying to read any nodes from this AST may cause an error.
 *
 * Buffer is returned to the cache to be reused.
 *
 * The buffer would be returned to the cache anyway, once all nodes of the AST are garbage collected,
 * but calling `dispose` is preferable, as it will happen immediately.
 * Otherwise, garbage collector may take time to collect the `ast` object, and new buffers may be created
 * in the meantime, when we could have reused this one.
 *
 * @param {Object} ast - AST object containing buffer etc
 * @returns {undefined}
 */
function dispose(ast) {
  // Return buffer to cache, to be reused
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
