var _prop = /* @__PURE__ */ babelHelpers.classPrivateFieldLooseKey("prop");
class Foo {
  constructor() {
    Object.defineProperty(this, _prop, {
      writable: true,
      value: "foo"
    });
  }
}
var _prop2 = /* @__PURE__ */ babelHelpers.classPrivateFieldLooseKey("prop");
class Bar extends Foo {
  constructor(..._args) {
    super(..._args);
    Object.defineProperty(this, _prop2, {
      writable: true,
      value: "bar"
    });
  }
}
