// When a constructor parameter property name clashes with a name used in a class
// field initializer, the `this.X` assignment must use the original source name.
class Foo {
  double = (x: number) => x * 2;
  constructor(public x: number) {}
}

class Bar {
  fn = (foo: string) => foo.length;
  constructor(public foo: string, private bar: number) {}
}
