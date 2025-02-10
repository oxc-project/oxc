var _F_brand = new WeakSet();
var _x = new WeakMap();
var _y = new WeakMap();
class F {
  constructor() {
    babelHelpers.classPrivateMethodInitSpec(this, _F_brand);
    babelHelpers.classPrivateFieldInitSpec(this, _x, 0);
    babelHelpers.classPrivateFieldInitSpec(this, _y, (() => {
      throw "error";
    })());
  }
  m() {
    _F_brand.has(babelHelpers.checkInRHS(this));
    _x.has(babelHelpers.checkInRHS(this));
    _y.has(babelHelpers.checkInRHS(this));
    _F_brand.has(babelHelpers.checkInRHS(this));
  }
}
function _get_w() {}
function _z() {}
