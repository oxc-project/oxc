var _foo = new WeakMap();
class Foo {
  constructor() {
    babelHelpers.classPrivateFieldInitSpec(this, _foo, 1);
  }
  test() {
    var _foo2;
    let _this$foo;
    var _foo3 = new WeakMap();
    class Nested extends (_foo2 = new WeakMap(), _this$foo = babelHelpers.classPrivateFieldGet2(_foo2, this), class {
      constructor() {
        babelHelpers.classPrivateFieldInitSpec(this, _foo2, 2);
        babelHelpers.defineProperty(this, _this$foo, 2);
      }
    }) {
      constructor(..._args) {
        super(..._args);
        babelHelpers.classPrivateFieldInitSpec(this, _foo3, 3);
      }
    }
  }
}
