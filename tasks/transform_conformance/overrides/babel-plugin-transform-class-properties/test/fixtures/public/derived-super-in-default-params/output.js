let _super = function() {
  babelHelpers.defineProperty(this, "bar", "foo");
  return this;
};
class Foo extends Bar {
  constructor(x = _super.call(super())) {}
}
