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
  get publicFieldValue() {
    return this.publicField;
  }
  set publicFieldValue(newValue) {
    this.publicField = newValue;
  }
  testUpdates() {
    var _this$privateFieldVal, _this$privateFieldVal2, _this$privateFieldVal3;
    babelHelpers.classPrivateFieldSet2(_privateField, this, 0);
    this.publicField = 0;
    babelHelpers.toSetter(_set_privateFieldValue.bind(babelHelpers.assertClassBrand(_Cl_brand, this)))._ = (_set_privateFieldValue.call(babelHelpers.assertClassBrand(_Cl_brand, this), (_this$privateFieldVal = _get_privateFieldValue.call(babelHelpers.assertClassBrand(_Cl_brand, this)), _this$privateFieldVal2 = _this$privateFieldVal++, _this$privateFieldVal)), _this$privateFieldVal2);
    this.publicFieldValue = this.publicFieldValue++;
    _set_privateFieldValue.call(babelHelpers.assertClassBrand(_Cl_brand, this), (_this$privateFieldVal3 = _get_privateFieldValue.call(babelHelpers.assertClassBrand(_Cl_brand, this)), ++_this$privateFieldVal3));
    ++this.publicFieldValue;
    _set_privateFieldValue.call(babelHelpers.assertClassBrand(_Cl_brand, this), _get_privateFieldValue.call(babelHelpers.assertClassBrand(_Cl_brand, this)) + 1);
    this.publicFieldValue += 1;
    babelHelpers.toSetter(_set_privateFieldValue.bind(babelHelpers.assertClassBrand(_Cl_brand, this)))._ = -(_get_privateFieldValue.call(babelHelpers.assertClassBrand(_Cl_brand, this)) ** _get_privateFieldValue.call(babelHelpers.assertClassBrand(_Cl_brand, this)));
    this.publicFieldValue = -(this.publicFieldValue ** this.publicFieldValue);
  }
}
function _get_privateFieldValue() {
  return babelHelpers.classPrivateFieldGet2(_privateField, this);
}
function _set_privateFieldValue(newValue) {
  babelHelpers.classPrivateFieldSet2(_privateField, this, newValue);
}
