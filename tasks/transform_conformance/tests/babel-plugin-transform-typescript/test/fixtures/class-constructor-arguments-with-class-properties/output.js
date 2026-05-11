let x = 10;
class Foo {
  constructor(_x) {
    this.x = _x;
    babelHelpers.defineProperty(this, "field", x);
  }
}
