'use strict';

const bindings = require('./bindings.js');
const { wrap } = require('./wrap.cjs');
const {
  parseSyncRaw,
  parseAsyncRaw,
  parseSyncLazy,
  parseAsyncLazy,
  rawTransferSupported,
} = require('./raw-transfer/index.js');

module.exports.ParseResult = bindings.ParseResult;
module.exports.ExportExportNameKind = bindings.ExportExportNameKind;
module.exports.ExportImportNameKind = bindings.ExportImportNameKind;
module.exports.ExportLocalNameKind = bindings.ExportLocalNameKind;
module.exports.ImportNameKind = bindings.ImportNameKind;
module.exports.parseWithoutReturn = bindings.parseWithoutReturn;
module.exports.Severity = bindings.Severity;

module.exports.parseAsync = async function parseAsync(filename, sourceText, options) {
  if (options?.experimentalRawTransfer) return await parseAsyncRaw(filename, sourceText, options);
  if (options?.experimentalLazy) return await parseAsyncLazy(filename, sourceText, options);
  return wrap(await bindings.parseAsync(filename, sourceText, options));
};

module.exports.parseSync = function parseSync(filename, sourceText, options) {
  if (options?.experimentalRawTransfer) return parseSyncRaw(filename, sourceText, options);
  if (options?.experimentalLazy) return parseSyncLazy(filename, sourceText, options);
  return wrap(bindings.parseSync(filename, sourceText, options));
};

module.exports.rawTransferSupported = rawTransferSupported;
