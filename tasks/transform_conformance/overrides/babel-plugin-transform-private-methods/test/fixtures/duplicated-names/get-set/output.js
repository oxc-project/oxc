var _privateField = new WeakMap();
var _Cl_brand = new WeakSet();
class Cl {
  constructor() {
    babelHelpers.classPrivateMethodInitSpec(this, _Cl_brand);
    babelHelpers.classPrivateFieldInitSpec(this, _privateField, 0);
  }
}
function _get_getSet() {
  return babelHelpers.classPrivateFieldGet2(_privateField, this);
}
function _set_getSet(newValue) {
  babelHelpers.classPrivateFieldSet2(_privateField, this, newValue);
}
