let _k;
let A = class A {
  static {
    _k = k;
  }
  constructor(p, q) {
    this.p = p;
    this.b = "";
  }
};
babelHelpers.decorate([dec, babelHelpers.decorateMetadata("design:type", Number)], A.prototype, "a", void 0);
babelHelpers.decorate([dec, babelHelpers.decorateMetadata("design:type", String)], A.prototype, "b", void 0);
babelHelpers.decorate([dec, babelHelpers.decorateMetadata("design:type", Boolean)], A.prototype, _k, void 0);
babelHelpers.decorate([dec, babelHelpers.decorateMetadata("design:type", typeof A === "undefined" ? Object : A)], A, "c", void 0);
A = babelHelpers.decorate([babelHelpers.decorateParam(1, dec), babelHelpers.decorateMetadata("design:paramtypes", [Number, String])], A);

