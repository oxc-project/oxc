class A {
  constructor() {
    babelHelpers.defineProperty(this, "a", void 0);
    babelHelpers.defineProperty(this, "b", 1);
    babelHelpers.defineProperty(this, "f", 3);
  }
  m() {}
}
babelHelpers.defineProperty(A, "c", void 0);
babelHelpers.defineProperty(A, "d", 2);
babelHelpers.decorate([dec], A.prototype, "a", void 0);
babelHelpers.decorate([dec], A.prototype, "b", void 0);
babelHelpers.decorate([dec], A.prototype, "m", null);
babelHelpers.decorate([dec], A, "c", void 0);
babelHelpers.decorate([dec], A, "d", void 0);

