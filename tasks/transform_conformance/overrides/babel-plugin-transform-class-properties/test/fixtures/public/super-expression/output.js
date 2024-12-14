class Foo extends Bar {
  constructor() {
    var _super = (..._args) => (
      super(..._args),
      babelHelpers.defineProperty(this, "bar", "foo"),
      this
    );
    foo(_super());
  }
}
