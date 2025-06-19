'use strict';

const bindings = require('./bindings.js');
const { wrap } = require('./wrap.cjs');
const rawTransferSupported = require('./raw-transfer/supported.js');

const { parseSync: parseSyncBinding, parseAsync: parseAsyncBinding } = bindings;

module.exports.ParseResult = bindings.ParseResult;
module.exports.ExportExportNameKind = bindings.ExportExportNameKind;
module.exports.ExportImportNameKind = bindings.ExportImportNameKind;
module.exports.ExportLocalNameKind = bindings.ExportLocalNameKind;
module.exports.ImportNameKind = bindings.ImportNameKind;
module.exports.parseWithoutReturn = bindings.parseWithoutReturn;
module.exports.Severity = bindings.Severity;

module.exports.parseSync = parseSync;
module.exports.parseAsync = parseAsync;
module.exports.rawTransferSupported = rawTransferSupported;

// Lazily loaded as needed
let parseSyncRaw = null,
  parseAsyncRaw,
  parseSyncLazy = null,
  parseAsyncLazy;

function loadRawTransfer() {
  if (parseSyncRaw === null) {
    ({ parseSyncRaw, parseAsyncRaw } = require('./raw-transfer/eager.js'));
  }
}

function loadRawTransferLazy() {
  if (parseSyncLazy === null) {
    ({ parseSyncLazy, parseAsyncLazy } = require('./raw-transfer/lazy.js'));
  }
}

function parseSync(filename, sourceText, options) {
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

async function parseAsync(filename, sourceText, options) {
  if (options?.experimentalRawTransfer) {
    loadRawTransfer();
    return await parseAsyncRaw(filename, sourceText, options);
  }
  if (options?.experimentalLazy) {
    loadRawTransferLazy();
    return await parseAsyncLazy(filename, sourceText, options);
  }
  return wrap(await parseAsyncBinding(filename, sourceText, options));
}
