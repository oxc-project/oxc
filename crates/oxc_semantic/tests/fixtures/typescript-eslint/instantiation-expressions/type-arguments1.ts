class Foo<T> {
  value: T;
}

class Bar<T> {
  foo = Foo<T>;
}

new Bar();
