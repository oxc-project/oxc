let receiver;
var _Accessor_brand = /* @__PURE__ */ new WeakSet();
class Accessor {
  constructor() {
    babelHelpers.classPrivateMethodInitSpec(this, _Accessor_brand);
  }
  run(rhs) {
    var _receiver;
    return _get_value.call(babelHelpers.assertClassBrand(_Accessor_brand, _receiver = receiver)) && (babelHelpers.toSetter(_set_value.bind(babelHelpers.assertClassBrand(_Accessor_brand, _receiver)))._ = rhs());
  }
  parameter(result = [(() => {
    var _receiver2;
    return _get_value.call(babelHelpers.assertClassBrand(_Accessor_brand, _receiver2 = receiver)) && (babelHelpers.toSetter(_set_value.bind(babelHelpers.assertClassBrand(_Accessor_brand, _receiver2)))._ = rhs());
  })()]) {
    return result;
  }
}
function _get_value() {
  receiver = {};
  return 1;
}
function _set_value(value) {}
