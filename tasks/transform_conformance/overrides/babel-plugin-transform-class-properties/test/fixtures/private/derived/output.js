var _prop = new WeakMap();
class Foo {
  constructor() {
    babelHelpers.classPrivateFieldInitSpec(this, _prop, "foo");
  }
}
var _prop2 = new WeakMap();
class Bar extends Foo {
  constructor(..._args) {
    super(..._args);
    babelHelpers.classPrivateFieldInitSpec(this, _prop2, "bar");
  }
}
