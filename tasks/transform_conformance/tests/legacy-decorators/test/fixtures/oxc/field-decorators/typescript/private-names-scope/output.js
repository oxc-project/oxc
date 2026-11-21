var _A;
// Class and constructor parameter decorators still apply when member decorations move into a
// static block.
let A = class A {
  static {
    _A = this;
  }
  #p = 1;
  static self = _A;
  m() {}
  constructor(x) {}
  static {
    babelHelpers.decorate([dec((a) => a.#p)], A.prototype, "m", null);
  }
};
A = _A = babelHelpers.decorate([cdec, babelHelpers.decorateParam(0, dec)], A);
// A private name of an enclosing class doesn't need the static block, where `this` would differ.
class Outer {
  #x = 1;
  make() {
    class Inner {
      m() {}
    }
    babelHelpers.decorate([dec(this.#x)], Inner.prototype, "m", null);
  }
}

