var _foo = /*#__PURE__*/babelHelpers.classPrivateFieldLooseKey("foo");
class Foo {
  constructor() {
    Object.defineProperty(this, _foo, {
      writable: true,
      value: 1
    });
  }
  test() {
    var _foo3;
    let _this$foo;
    var _foo2 = /*#__PURE__*/babelHelpers.classPrivateFieldLooseKey("foo");
    class Nested extends (_foo3 = /*#__PURE__*/babelHelpers.classPrivateFieldLooseKey("foo"), _this$foo = babelHelpers.classPrivateFieldLooseBase(this, _foo3)[_foo3], class {
      constructor() {
        Object.defineProperty(this, _foo3, {
          writable: true,
          value: 2
        });
        this[_this$foo] = 2;
      }
    }) {
      constructor(...args) {
        super(...args);
        Object.defineProperty(this, _foo2, {
          writable: true,
          value: 3
        });
      }
    }
  }
}
