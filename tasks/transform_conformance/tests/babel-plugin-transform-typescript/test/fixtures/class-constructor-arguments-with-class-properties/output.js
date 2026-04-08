class Foo {
  constructor(x) {
    this.x = x;
    babelHelpers.defineProperty(this, "double", (x) => x * 2);
  }
}
class Bar {
  constructor(foo, bar) {
    this.foo = foo;
    this.bar = bar;
    babelHelpers.defineProperty(this, "fn", (foo) => foo.length);
  }
}
