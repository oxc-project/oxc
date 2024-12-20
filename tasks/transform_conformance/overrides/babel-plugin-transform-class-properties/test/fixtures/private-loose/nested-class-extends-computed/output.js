var _foo = babelHelpers.classPrivateFieldLooseKey("foo");
class Foo {
  constructor() {
    Object.defineProperty(this, _foo, {
      writable: true,
      value: 1
    });
  }
  test() {
    let _this$foo;
    var _foo2 = babelHelpers.classPrivateFieldLooseKey("foo");
    class Nested extends (_this$foo = babelHelpers.classPrivateFieldLooseBase(this, _foo)[_foo], class {
      constructor() {
        this[_this$foo] = 2;
      }
    }) {
      constructor(..._args) {
        super(..._args);
        Object.defineProperty(this, _foo2, {
          writable: true,
          value: 3
        });
      }
    }
  }
}
