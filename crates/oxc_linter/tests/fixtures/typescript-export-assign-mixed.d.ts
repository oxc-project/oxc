export = foobar;

declare function foobar(): void;
declare namespace foobar {
  type MyType = string
  enum MyEnum {
    Foo,
    Bar,
    Baz
  }
}
