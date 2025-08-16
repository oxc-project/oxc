export class WorkspaceResolver {
  async invite() {
    try {
      try {
        var _usingCtx = babelHelpers.usingCtx();
        const lock = _usingCtx.a(await acquire(lockFlag));
      } catch (_) {
        _usingCtx.e = _;
      } finally {
        await _usingCtx.d();
      }
    } catch {}
  }
}
