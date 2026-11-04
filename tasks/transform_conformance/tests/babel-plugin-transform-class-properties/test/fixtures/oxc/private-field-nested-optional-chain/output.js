var _items = /*#__PURE__*/ new WeakMap();
class Cache {
  constructor() {
    babelHelpers.classPrivateFieldInitSpec(this, _items, new Map());
  }
  runNext(key) {
    return (babelHelpers.classPrivateFieldGet2(_items, this).get(key)?.find(x => x.paused))?.continue() ?? Promise.resolve();
  }
  callNext(key) {
    return (babelHelpers.classPrivateFieldGet2(_items, this).get(key)?.find(x => x.paused))?.();
  }
}
