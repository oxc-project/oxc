const bindings = require('./bindings.js');

module.exports.moduleLexerAsync = bindings.moduleLexerAsync;
module.exports.moduleLexerSync = bindings.moduleLexerSync;
module.exports.parseWithoutReturn = bindings.parseWithoutReturn;

module.exports.parseAsync = async function parseAsync(...args) {
  const result = await bindings.parseAsync(...args);
  result.program = JSON.parse(result.program);
  return result;
};
module.exports.parseSync = function parseSync(...args) {
  const result = bindings.parseSync(...args);
  result.program = JSON.parse(result.program);
  return result;
};
