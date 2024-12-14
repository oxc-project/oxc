class Foo extends Bar {
  constructor(..._args) {
    super(..._args);
    babelHelpers.defineProperty(this, "bar", "foo");
  }
}
