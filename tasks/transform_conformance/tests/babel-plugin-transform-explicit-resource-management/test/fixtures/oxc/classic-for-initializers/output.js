function run() {
  try {
    var _usingCtx = babelHelpers.usingCtx();
    outer: inner: for (const first = _usingCtx.u(getFirst()), second = _usingCtx.u(getSecond()); keepGoing(); advance()) {
      if (skip()) continue outer;
      if (stop()) break inner;
      use(first, second);
    }
  } catch (_) {
    _usingCtx.e = _;
  } finally {
    _usingCtx.d();
  }
}
async function runAsync() {
  try {
    var _usingCtx2 = babelHelpers.usingCtx();
    for (const first = _usingCtx2.a(getFirst()), second = _usingCtx2.a(getSecond()); keepGoing(); advance()) {
      use(first, second);
    }
  } catch (_) {
    _usingCtx2.e = _;
  } finally {
    await _usingCtx2.d();
  }
}
