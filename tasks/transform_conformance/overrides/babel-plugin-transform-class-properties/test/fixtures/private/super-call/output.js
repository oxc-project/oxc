class A {
  foo() {
    return "bar";
  }
}
var _foo = new WeakMap();
class B extends A {
  constructor(..._args) {
    super(..._args);
    babelHelpers.classPrivateFieldInitSpec(this, _foo, super.foo());
  }
}
