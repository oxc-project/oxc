var _bar = /*#__PURE__*/new WeakMap();
class Foo extends Bar {
  constructor() {
    var _super = (..._args) => (
      super(..._args),
      babelHelpers.classPrivateFieldInitSpec(this, _bar, "foo"),
      this
    );

    if (condition) {
      _super();
    } else {
      _super();
    }
  }
}
