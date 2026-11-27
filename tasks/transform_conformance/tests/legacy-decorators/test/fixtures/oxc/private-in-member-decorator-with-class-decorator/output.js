import { dec } from "dec";
let A = class A {
  #a = 1;
  m() {}
  static {
    babelHelpers.decorate([#a in A ? dec : dec], A.prototype, "m", null);
  }
};
A = babelHelpers.decorate([dec], A);
let B = class B {
  #b = 1;
  constructor(p) {}
  m() {}
  static {
    babelHelpers.decorate([#b in B ? dec : dec], B.prototype, "m", null);
  }
};
B = babelHelpers.decorate([dec, babelHelpers.decorateParam(0, dec)], B);
let C = class C {
  #c = 1;
  m() {}
  static {
    babelHelpers.decorate([#c in C ? dec : dec], C.prototype, "m", null);
  }
};
C = babelHelpers.decorate([dec], C);
export { C };
let D = class D {
  #d = 1;
  m() {}
  static {
    babelHelpers.decorate([#d in D ? dec : dec], D.prototype, "m", null);
  }
};
D = babelHelpers.decorate([dec], D);
export default D;
