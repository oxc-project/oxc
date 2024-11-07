export function named() {
  return _named.apply(this, arguments);
}
function _named() {
  _named = babelHelpers.asyncToGenerator(function* (...args) {
    yield Promise.resolve();
  });
  return _named.apply(this, arguments);
}