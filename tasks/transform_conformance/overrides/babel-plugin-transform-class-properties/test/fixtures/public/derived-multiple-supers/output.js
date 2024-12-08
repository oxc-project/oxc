class Foo extends Bar {
  constructor() {
    var _super = (..._args) => {
      super(..._args);
      babelHelpers.defineProperty(this, "bar", "foo");
      return this;
    };

    if (condition) {
      _super();
    } else {
      _super();
    }
  }
}
