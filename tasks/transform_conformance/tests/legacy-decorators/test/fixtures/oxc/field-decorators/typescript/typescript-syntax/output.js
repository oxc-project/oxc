class A {
  a;
  b = 1;
}
babelHelpers.decorate([dec(x)], A.prototype, "a", void 0);
babelHelpers.decorate([dec((v) => v)], A.prototype, "b", void 0);

