function f(a = () => {}, b = () => {}) {
  try {
    var _usingCtx = babelHelpers.usingCtx();
    const x = _usingCtx.u(a()), y = _usingCtx.u(b());
    doSomethingWith(x, y, () => {});
  } catch (_) {
    _usingCtx.e = _;
  } finally {
    _usingCtx.d();
  }
}
