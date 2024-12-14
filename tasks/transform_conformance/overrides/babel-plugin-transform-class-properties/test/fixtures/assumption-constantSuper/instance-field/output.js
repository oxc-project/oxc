class A extends B {
  constructor(..._args) {
    super(..._args);
    babelHelpers.defineProperty(this, "foo", super.bar);
  }
}
