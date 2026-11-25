import { dec } from "dec";
var _A, _B, _C, _D;
var _a = /* @__PURE__ */ new WeakMap();
let A = class A {
  constructor() {
    babelHelpers.classPrivateFieldInitSpec(this, _a, 1);
  }
  m() {}
};
_A = A;
babelHelpers.decorate([_a.has(babelHelpers.checkInRHS(_A)) ? dec : dec], _A.prototype, "m", null);
A = babelHelpers.decorate([dec], A);
var _b = /* @__PURE__ */ new WeakMap();
let B = class B {
  constructor(p) {
    babelHelpers.classPrivateFieldInitSpec(this, _b, 1);
  }
  m() {}
};
_B = B;
babelHelpers.decorate([_b.has(babelHelpers.checkInRHS(_B)) ? dec : dec], _B.prototype, "m", null);
B = babelHelpers.decorate([dec, babelHelpers.decorateParam(0, dec)], B);
var _c = /* @__PURE__ */ new WeakMap();
let C = class C {
  constructor() {
    babelHelpers.classPrivateFieldInitSpec(this, _c, 1);
  }
  m() {}
};
_C = C;
babelHelpers.decorate([_c.has(babelHelpers.checkInRHS(_C)) ? dec : dec], _C.prototype, "m", null);
C = babelHelpers.decorate([dec], C);
export { C };
var _d = /* @__PURE__ */ new WeakMap();
let D = class D {
  constructor() {
    babelHelpers.classPrivateFieldInitSpec(this, _d, 1);
  }
  m() {}
};
_D = D;
babelHelpers.decorate([_d.has(babelHelpers.checkInRHS(_D)) ? dec : dec], _D.prototype, "m", null);
D = babelHelpers.decorate([dec], D);
export default D;
