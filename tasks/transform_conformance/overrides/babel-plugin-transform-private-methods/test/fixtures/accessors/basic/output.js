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
  return babelHelpers.classPrivateFieldGet2(_privateField, this);
}
function _set_privateFieldValue(newValue) {
  babelHelpers.classPrivateFieldSet2(_privateField, this, newValue);
}
