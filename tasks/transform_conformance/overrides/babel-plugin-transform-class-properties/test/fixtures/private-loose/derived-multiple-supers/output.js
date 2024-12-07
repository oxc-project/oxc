var _bar = babelHelpers.classPrivateFieldLooseKey("bar");
class Foo extends Bar {
  constructor() {
    var _super = (..._args) => (super(..._args), Object.defineProperty(this, _bar, {
      writable: true,
      value: "foo"
    }), this);
    if (condition) {
      _super();
    } else {
      _super();
    }
  }
}
