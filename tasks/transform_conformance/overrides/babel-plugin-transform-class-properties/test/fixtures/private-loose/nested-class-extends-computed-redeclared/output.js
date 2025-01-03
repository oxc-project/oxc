var _foo = babelHelpers.classPrivateFieldLooseKey("foo");
class Foo {
  constructor() {
    Object.defineProperty(this, _foo, {
      writable: true,
      value: 1
    });
  }
  test() {
    var _foo2;
    let _this$foo;
    var _foo3 = babelHelpers.classPrivateFieldLooseKey("foo");
    class Nested extends (_foo2 = babelHelpers.classPrivateFieldLooseKey("foo"), _this$foo = babelHelpers.classPrivateFieldLooseBase(this, _foo2)[_foo2], class {
      constructor() {
        Object.defineProperty(this, _foo2, {
          writable: true,
          value: 2
        });
        this[_this$foo] = 2;
      }
    }) {
      constructor(..._args) {
        super(..._args);
        Object.defineProperty(this, _foo3, {
          writable: true,
          value: 3
        });
      }
    }
  }
}
