var _a = /*#__PURE__*/ new WeakMap();
class A {
  constructor() {
    babelHelpers.classPrivateFieldInitSpec(this, _a, {});
  }
  method() {
    babelHelpers.classPrivateFieldGet2(_a, this).get(message.id)?.(message);
  }
}
