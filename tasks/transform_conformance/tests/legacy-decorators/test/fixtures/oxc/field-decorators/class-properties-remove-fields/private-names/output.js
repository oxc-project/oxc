var _p = /* @__PURE__ */ new WeakMap();
class A {
  constructor() {
    babelHelpers.classPrivateFieldInitSpec(this, _p, 1);
    babelHelpers.defineProperty(this, "b", 1);
  }
  m() {}
  n(param) {}
  static {
    babelHelpers.decorate([dec(_p.has(babelHelpers.checkInRHS(A)))], A.prototype, "a", void 0);
    babelHelpers.decorate([dec((a) => babelHelpers.classPrivateFieldGet2(_p, a))], A.prototype, "b", void 0);
    babelHelpers.decorate([dec((a) => babelHelpers.classPrivateFieldGet2(_p, a))], A.prototype, "m", null);
    babelHelpers.decorate([babelHelpers.decorateParam(0, dec(_p.has(babelHelpers.checkInRHS(A))))], A.prototype, "n", null);
  }
}

