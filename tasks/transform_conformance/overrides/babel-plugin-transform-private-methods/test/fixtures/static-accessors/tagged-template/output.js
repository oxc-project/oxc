class Foo {
  static test() {
    const receiver = _get_tag.call(babelHelpers.assertClassBrand(Foo, this)).bind(this)``;
    expect(receiver).toBe(this);
  }
}
function _get_tag() {
  return function() {
    return this;
  };
}
