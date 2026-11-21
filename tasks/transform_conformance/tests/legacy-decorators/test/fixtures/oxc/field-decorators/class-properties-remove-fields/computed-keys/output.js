let _a, _c;
// Decorated fields without an initializer are removed, but their keys are still evaluated in
// order: in the next computed key, or in a static block when no computed key follows.
class A {
  static {
    _c = c();
  }
  [(_a = a(), b())]() {}
}
babelHelpers.decorate([dec], A.prototype, _a, void 0);
babelHelpers.decorate([dec], A.prototype, _c, void 0);

