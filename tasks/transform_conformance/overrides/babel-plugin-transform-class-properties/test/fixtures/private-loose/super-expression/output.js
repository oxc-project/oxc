var _bar = /*#__PURE__*/babelHelpers.classPrivateFieldLooseKey("bar");
class Foo extends Bar {
  constructor() {
    var _super = (..._args) => (
      super(..._args),
      Object.defineProperty(this, _bar, {
        writable: true,
        value: "foo"
      }),
      this
    );
    foo(_super());
  }
}
