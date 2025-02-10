class Cl {
  static getValue() {
    return _get_privateStaticFieldValue.call(Cl);
  }
  static setValue() {
    babelHelpers.toSetter(_set_privateStaticFieldValue.bind(Cl))._ = "dank";
  }
}
function _get_privateStaticFieldValue() {
  return _PRIVATE_STATIC_FIELD._;
}
function _set_privateStaticFieldValue(newValue) {
  _PRIVATE_STATIC_FIELD._ = `Updated: ${newValue}`;
}
var _PRIVATE_STATIC_FIELD = { _: "top secret string" };
