class A {
  static y = 1;
  z = 3;
}
babelHelpers.decorate([dec], A.prototype, "z", void 0);
babelHelpers.decorate([dec], A, "y", void 0);
