export { C };
try {
  var _usingCtx = babelHelpers.usingCtx();
  var x = _usingCtx.u(foo());
  var C = class C {
    static getSelf() {
      return C;
    }
  };
  var K = C;
  C = 123;
  assert(K.getSelf() === K);
} catch (_) {
  _usingCtx.e = _;
} finally {
  _usingCtx.d();
}
