var _privateField = new WeakMap();
var _Cl_brand = new WeakSet();
class Cl {
  constructor() {
    babelHelpers.classPrivateMethodInitSpec(this, _Cl_brand);
    babelHelpers.classPrivateFieldInitSpec(this, _privateField, 0);
    this.publicField = (this, babelHelpers.writeOnlyError("#privateFieldValue"));
  }
}
function _set_privateFieldValue(newValue) {
  babelHelpers.classPrivateFieldSet2(_privateField, this, newValue);
}
