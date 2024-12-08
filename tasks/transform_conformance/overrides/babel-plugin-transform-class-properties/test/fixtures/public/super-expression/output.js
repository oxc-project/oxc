class Foo extends Bar {
  constructor() {
    var _super = (..._args) => {
      super(..._args);
      babelHelpers.defineProperty(this, "bar", "foo");
      return this;
    };
    foo(_super());
  }
}
