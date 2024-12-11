class A {
  constructor() {
    babelHelpers.defineProperty(this, "b", 44);
  }
  method() {
    var _ref;
    (_ref = (undefined, this)) === null || _ref === void 0 ? void 0 : babelHelpers.assertClassBrand(A, _ref, _a)._;
    (undefined, this)?.b;
  }
}
var _a = {
  _: 33
};