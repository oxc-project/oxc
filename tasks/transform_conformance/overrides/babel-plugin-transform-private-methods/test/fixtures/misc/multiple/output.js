var _A_brand = new WeakSet();
class A {
  constructor() {
    babelHelpers.classPrivateMethodInitSpec(this, _A_brand);
    babelHelpers.assertClassBrand(_A_brand, this, _method).call(this);
    _get_getter.call(babelHelpers.assertClassBrand(_A_brand, this));
    babelHelpers.toSetter(_set_setter.bind(babelHelpers.assertClassBrand(_A_brand, this)))._ = 1;
    _get_getset.call(babelHelpers.assertClassBrand(_A_brand, this));
    babelHelpers.toSetter(_set_getset.bind(babelHelpers.assertClassBrand(_A_brand, this)))._ = 2;
  }
}
function _method() {}
function _get_getter() {}
function _set_setter(v) {}
function _get_getset() {}
function _set_getset(v) {}
