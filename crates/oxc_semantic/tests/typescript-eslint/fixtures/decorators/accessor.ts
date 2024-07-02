function decorator() {}
class Foo {
  @decorator
  get foo() {
    return 1;
  }
  @decorator
  set bar(value) {}
}
