var _prop = /* @__PURE__ */ new WeakMap();
class Foo {
  constructor() {
    babelHelpers.classPrivateFieldInitSpec(this, _prop, "foo");
  }
}
var _prop2 = /* @__PURE__ */ new WeakMap();
class Bar extends Foo {
  constructor(..._args) {
    super(..._args);
    babelHelpers.classPrivateFieldInitSpec(this, _prop2, "bar");
  }
}
