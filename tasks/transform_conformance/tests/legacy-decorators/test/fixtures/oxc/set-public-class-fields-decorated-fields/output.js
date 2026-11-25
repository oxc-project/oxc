var _ref, _f;
let _a, _b, _c, _e;
class A {
  constructor() {
    this.z = 3;
    this[_c] = 6;
    this[_e] = 7;
    this["q"] = 8;
  }
  static {
    this.y = 1;
  }
  z;
  static {
    this[_a] = 4;
  }
  static {
    this[_b] = 5;
  }
  [_ref = (_a = a(), _b = b(), _c = c())];
  static {
    this.w = this.y;
  }
  static v;
  u;
  [_e = e()];
  static [_f = f()];
  "q";
  static {
    this[9] = 10;
  }
}
babelHelpers.decorate([dec], A.prototype, "z", void 0);
babelHelpers.decorate([dec], A.prototype, _ref, void 0);
babelHelpers.decorate([dec], A.prototype, "u", void 0);
babelHelpers.decorate([dec], A.prototype, "q", void 0);
babelHelpers.decorate([dec], A, "y", void 0);
babelHelpers.decorate([dec], A, _b, void 0);
babelHelpers.decorate([dec], A, "w", void 0);
babelHelpers.decorate([dec], A, "v", void 0);
babelHelpers.decorate([dec], A, _f, void 0);
babelHelpers.decorate([dec], A, 9, void 0);
class B extends Base {
  static {
    this.x = 1;
  }
  y;
  constructor() {
    super();
    this.y = 2;
  }
}
babelHelpers.decorate([dec], B.prototype, "y", void 0);
babelHelpers.decorate([dec], B, "x", void 0);
