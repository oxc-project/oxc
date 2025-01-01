class Cl {
  static publicGetPrivateField() {
    return _get_privateFieldValue.call(Cl);
  }
  static publicSetPrivateField(newValue) {
    babelHelpers.toSetter(_set_privateFieldValue.bind(Cl))._ = newValue;
  }
  static get publicFieldValue() {
    return Cl.publicField;
  }
  static set publicFieldValue(newValue) {
    Cl.publicField = newValue;
  }
  static testUpdates() {
    var _Cl$privateFieldValue, _Cl$privateFieldValue2, _Cl$privateFieldValue3;
    _privateField._ = 0;
    Cl.publicField = 0;
    babelHelpers.toSetter(_set_privateFieldValue.bind(Cl))._ = (_set_privateFieldValue.call(babelHelpers.assertClassBrand(Cl, Cl), (_Cl$privateFieldValue = _get_privateFieldValue.call(babelHelpers.assertClassBrand(Cl, Cl)), _Cl$privateFieldValue2 = _Cl$privateFieldValue++, _Cl$privateFieldValue)), _Cl$privateFieldValue2);
    Cl.publicFieldValue = Cl.publicFieldValue++;
    _set_privateFieldValue.call(babelHelpers.assertClassBrand(Cl, Cl), (_Cl$privateFieldValue3 = _get_privateFieldValue.call(babelHelpers.assertClassBrand(Cl, Cl)), ++_Cl$privateFieldValue3));
    ++Cl.publicFieldValue;
    _set_privateFieldValue.call(babelHelpers.assertClassBrand(Cl, Cl), _get_privateFieldValue.call(babelHelpers.assertClassBrand(Cl, Cl)) + 1);
    Cl.publicFieldValue += 1;
    babelHelpers.toSetter(_set_privateFieldValue.bind(Cl))._ = -(_get_privateFieldValue.call(Cl) ** _get_privateFieldValue.call(Cl));
    Cl.publicFieldValue = -(Cl.publicFieldValue ** Cl.publicFieldValue);
  }
}
function _get_privateFieldValue() {
  return _privateField._;
}
function _set_privateFieldValue(newValue) {
  _privateField._ = newValue;
}
var _privateField = { _: "top secret string" };
babelHelpers.defineProperty(Cl, "publicField", "not secret string");
