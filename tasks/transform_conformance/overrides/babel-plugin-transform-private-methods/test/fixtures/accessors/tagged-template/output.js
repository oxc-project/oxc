var _Foo_brand = new WeakSet();
class Foo {
  constructor() {
    babelHelpers.classPrivateMethodInitSpec(this, _Foo_brand);
    _get_tag.call(babelHelpers.assertClassBrand(_Foo_brand, this)).bind(this)``;
  }
}
function _get_tag() {
  return () => this;
}
new Foo();
