class A {
  foo() {
    return "bar";
  }
}
class B extends A {
  constructor(..._args) {
    super(..._args);
    babelHelpers.defineProperty(this, "foo", super.foo());
  }
}
