var _A_brand = new WeakSet();
class A {
  constructor() {
    babelHelpers.classPrivateMethodInitSpec(this, _A_brand);
  }
  m() {
    [babelHelpers.toSetter(_set_setter.bind(babelHelpers.assertClassBrand(_A_brand, this)))._] = [1];
    [(this, babelHelpers.readOnlyError("#getter"))._] = [1];
  }
}
function _set_setter(v) {}
function _get_getter() {}
