let results = [];
var _Foo_brand = new WeakSet();
class Foo {
  constructor() {
    babelHelpers.classPrivateMethodInitSpec(this, _Foo_brand);
    this.self, results.push(2), babelHelpers.readOnlyError("#privateFieldValue");
  }
  get self() {
    results.push(1);
    return this;
  }
}
function _get_privateFieldValue() {
  return 42;
}
