function disposeSync(make) {
  try {
    try {
      var _usingCtx = babelHelpers.usingCtx();
      const tryResource = _usingCtx.u(make("try"));
      throw new Error("catch");
    } catch (_) {
      _usingCtx.e = _;
    } finally {
      _usingCtx.d();
    }
  } catch (error) {
    try {
      var _usingCtx2 = babelHelpers.usingCtx();
      const catchResource = _usingCtx2.u(make(error.message));
    } catch (_) {
      _usingCtx2.e = _;
    } finally {
      _usingCtx2.d();
    }
  } finally {
    try {
      var _usingCtx3 = babelHelpers.usingCtx();
      const finallyResource = _usingCtx3.u(make("finally"));
    } catch (_) {
      _usingCtx3.e = _;
    } finally {
      _usingCtx3.d();
    }
  }
}
async function disposeAsync(make) {
  try {
    throw new Error("catch");
  } catch {
    try {
      var _usingCtx4 = babelHelpers.usingCtx();
      const catchResource = _usingCtx4.a(await make("catch"));
    } catch (_) {
      _usingCtx4.e = _;
    } finally {
      await _usingCtx4.d();
    }
  } finally {
    try {
      var _usingCtx5 = babelHelpers.usingCtx();
      const finallyResource = _usingCtx5.a(await make("finally"));
    } catch (_) {
      _usingCtx5.e = _;
    } finally {
      await _usingCtx5.d();
    }
  }
}
function disposeDestructured(make) {
  try {
    throw { name: "catch", code: 1 };
  } catch ({ name, code }) {
    try {
      var _usingCtx6 = babelHelpers.usingCtx();
      const resource = _usingCtx6.u(make(name, code));
    } catch (_) {
      _usingCtx6.e = _;
    } finally {
      _usingCtx6.d();
    }
  }
}
