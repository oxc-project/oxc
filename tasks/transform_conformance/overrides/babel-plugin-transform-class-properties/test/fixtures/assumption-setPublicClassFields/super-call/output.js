class A {
  foo() {
    return "bar";
  }
}
class B extends A {
  constructor(..._args) {
    super(..._args);
    this.foo = super.foo();
  }
}
