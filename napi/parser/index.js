const bindings = require('./bindings.js');

module.exports.ParseResult = bindings.ParseResult;
module.exports.ExportExportNameKind = bindings.ExportExportNameKind;
module.exports.ExportImportNameKind = bindings.ExportImportNameKind;
module.exports.ExportLocalNameKind = bindings.ExportLocalNameKind;
module.exports.ImportNameKind = bindings.ImportNameKind;
module.exports.parseWithoutReturn = bindings.parseWithoutReturn;
module.exports.Severity = bindings.Severity;

function wrap(result) {
  let program, module, comments, errors, magicString;
  return {
    get program() {
      if (!program) program = JSON.parse(result.program);
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
    get magicString() {
      if (!magicString) magicString = result.magicString;
      magicString.generateMap = function generateMap(options) {
        return {
          toString: () => magicString.toSourcemapString(options),
          toUrl: () => magicString.toSourcemapUrl(options),
          toMap: () => magicString.toSourcemapObject(options),
        };
      };
      return magicString;
    },
  };
}

module.exports.parseAsync = async function parseAsync(...args) {
  return wrap(await bindings.parseAsync(...args));
};

module.exports.parseSync = function parseSync(...args) {
  return wrap(bindings.parseSync(...args));
};
