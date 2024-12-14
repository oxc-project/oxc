class A extends B {
  constructor() {
    var _super = (..._args) => (
      super(..._args),
      babelHelpers.defineProperty(this, "x", 2),
      this
    );
    x ? _super(a) : _super(b);
  }
}
