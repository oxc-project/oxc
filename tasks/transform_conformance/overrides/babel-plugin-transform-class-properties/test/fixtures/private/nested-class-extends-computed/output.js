var _foo = /* @__PURE__ */ new WeakMap();
class Foo {
  constructor() {
    babelHelpers.classPrivateFieldInitSpec(this, _foo, 1);
  }
  test() {
    let _this$foo;
    var _foo2 = /* @__PURE__ */ new WeakMap();
    class Nested extends (_this$foo = babelHelpers.classPrivateFieldGet2(_foo, this), class {
      constructor() {
        babelHelpers.defineProperty(this, _this$foo, 2);
      }
    }) {
      constructor(..._args) {
        super(..._args);
        babelHelpers.classPrivateFieldInitSpec(this, _foo2, 3);
      }
    }
  }
}
