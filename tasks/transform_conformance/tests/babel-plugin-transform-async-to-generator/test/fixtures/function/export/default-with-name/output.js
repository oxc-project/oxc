export default function D(_x) {
  return _D.apply(this, arguments);
}
function _D() {
  _D = babelHelpers.asyncToGenerator(function* (a, b = 0) {
    yield Promise.resolve();
  });
  return _D.apply(this, arguments);
}