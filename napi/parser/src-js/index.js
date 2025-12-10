import { createRequire } from "node:module";
import { parse as parseBinding, parseSync as parseSyncBinding } from "./bindings.js";
import { wrap } from "./wrap.js";

export { default as visitorKeys } from "../generated/visit/keys.js";
export { Visitor } from "./visit/index.js";

export {
  ExportExportNameKind,
  ExportImportNameKind,
  ExportLocalNameKind,
  ImportNameKind,
  ParseResult,
  Severity,
} from "./bindings.js";
export { rawTransferSupported } from "./raw-transfer/supported.js";

const require = createRequire(import.meta.url);

// Lazily loaded as needed
let parseSyncRaw = null,
  parseRaw,
  parseSyncLazy = null,
  parseLazy,
  LazyVisitor;

/**
 * Lazy-load code related to raw transfer.
 * @returns {undefined}
 */
function loadRawTransfer() {
  if (parseSyncRaw === null) {
    ({ parseSyncRaw, parse: parseRaw } = require("./raw-transfer/eager.js"));
  }
}

/**
 * Lazy-load code related to raw transfer lazy deserialization.
 * @returns {undefined}
 */
function loadRawTransferLazy() {
  if (parseSyncLazy === null) {
    ({ parseSyncLazy, parse: parseLazy, Visitor: LazyVisitor } = require("./raw-transfer/lazy.js"));
  }
}

/**
 * Parse JS/TS source synchronously on current thread.
 *
 * @param {string} filename - Filename
 * @param {string} sourceText - Source text of file
 * @param {Object|undefined} options - Parsing options
 * @returns {Object} - Object with property getters for `program`, `module`, `comments`, and `errors`
 * @throws {Error} - If `experimentalRawTransfer` or `experimentalLazy` option is enabled,
 *   and raw transfer is not supported on this platform
 */
export function parseSync(filename, sourceText, options) {
  if (options?.experimentalRawTransfer) {
    loadRawTransfer();
    return parseSyncRaw(filename, sourceText, options);
  }
  if (options?.experimentalLazy) {
    loadRawTransferLazy();
    return parseSyncLazy(filename, sourceText, options);
  }
  return wrap(parseSyncBinding(filename, sourceText, options));
}

/**
 * Parse JS/TS source asynchronously on a separate thread.
 *
 * Note that not all of the workload can happen on a separate thread.
 * Parsing on Rust side does happen in a separate thread, but deserialization of the AST to JS objects
 * has to happen on current thread. This synchronous deserialization work typically outweighs
 * the asynchronous parsing by a factor of between 3 and 20.
 *
 * i.e. the majority of the workload cannot be parallelized by using this method.
 *
 * Generally `parseSync` is preferable to use as it does not have the overhead of spawning a thread.
 * If you need to parallelize parsing multiple files, it is recommended to use worker threads.
 *
 * @param {string} filename - Filename
 * @param {string} sourceText - Source text of file
 * @param {Object|undefined} options - Parsing options
 * @returns {Object} - Object with property getters for `program`, `module`, `comments`, and `errors`
 * @throws {Error} - If `experimentalRawTransfer` or `experimentalLazy` option is enabled,
 *   and raw transfer is not supported on this platform
 */
export async function parse(filename, sourceText, options) {
  if (options?.experimentalRawTransfer) {
    loadRawTransfer();
    return await parseRaw(filename, sourceText, options);
  }
  if (options?.experimentalLazy) {
    loadRawTransferLazy();
    return await parseLazy(filename, sourceText, options);
  }
  return wrap(await parseBinding(filename, sourceText, options));
}

/**
 * Get `Visitor` class to construct visitors with.
 * @returns {function} - `Visitor` class
 */
export function experimentalGetLazyVisitor() {
  loadRawTransferLazy();
  return LazyVisitor;
}
