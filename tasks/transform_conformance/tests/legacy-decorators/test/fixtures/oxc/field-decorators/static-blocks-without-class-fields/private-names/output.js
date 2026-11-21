class A {
  #p = 1;
  m() {}
  static {
    babelHelpers.decorate([dec(#p in A)], A.prototype, "m", null);
  }
}

