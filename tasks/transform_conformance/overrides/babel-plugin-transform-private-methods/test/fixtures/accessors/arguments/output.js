var _privateField = new WeakMap();
var _Cl_brand = new WeakSet();
class Cl {
  constructor() {
    babelHelpers.classPrivateMethodInitSpec(this, _Cl_brand);
    babelHelpers.classPrivateFieldInitSpec(this, _privateField, "top secret string");
    this.publicField = "not secret string";
  }
  publicGetPrivateField() {
    return _get_privateFieldValue.call(babelHelpers.assertClassBrand(_Cl_brand, this));
  }
  publicSetPrivateField(newValue) {
    babelHelpers.toSetter(_set_privateFieldValue.bind(babelHelpers.assertClassBrand(_Cl_brand, this)))._ = newValue;
  }
}
function _get_privateFieldValue() {
  expect(arguments.length).toBe(0);
  return babelHelpers.classPrivateFieldGet2(_privateField, this);
}
function _set_privateFieldValue(newValue) {
  expect(arguments.length).toBe(1);
  babelHelpers.classPrivateFieldSet2(_privateField, this, newValue);
}
