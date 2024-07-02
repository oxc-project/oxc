function decorator() {}
@decorator
class Foo {
  bar(baz: typeof this) {}
}
