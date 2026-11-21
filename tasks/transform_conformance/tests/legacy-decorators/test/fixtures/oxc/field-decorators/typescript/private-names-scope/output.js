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
// A static block nested in a decorator doesn't end the class's own static block.
let B = class B {
  #p = 1;
  m() {}
  static {
    babelHelpers.decorate([dec(class {
      static {}
    }, B, #p in Object)], B.prototype, "m", null);
  }
};
B = babelHelpers.decorate([cdec], B);
// The decorators and heritage of a nested class can refer to the outer private name.
class C {
  #x = 1;
  m() {}
  static {
    babelHelpers.decorate([dec(class extends (#x in Object ? A : B) {
      #x = 2;
    })], C.prototype, "m", null);
  }
}
let _default = class _default {
  #p = 1;
  m() {}
  static {
    babelHelpers.decorate([dec(#p in Object)], _default.prototype, "m", null);
  }
};
_default = babelHelpers.decorate([cdec], _default);
export default _default;

