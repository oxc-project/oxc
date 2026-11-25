var _Foo2, _Foo, _default2;
function dec(Class) {
  return class extends Class {};
}
let Foo = _Foo2 = class Foo {
  static getP() {
    return babelHelpers.assertClassBrand(_Foo, this, _p)._;
  }
  static callM() {
    return babelHelpers.assertClassBrand(_Foo, this, _m).call(this);
  }
  static hasP(obj) {
    return babelHelpers.checkInRHS(obj) === _Foo && !!_p;
  }
  static self() {
    return _Foo2;
  }
};
_Foo = Foo;
function _m() {}
var _p = { _: 1 };
Foo = _Foo2 = babelHelpers.decorate([dec], Foo);
let _default = class _default {
  static getP() {
    return babelHelpers.assertClassBrand(_default2, this, _p2)._;
  }
};
_default2 = _default;
var _p2 = { _: 1 };
_default = babelHelpers.decorate([dec], _default);
export default _default;
