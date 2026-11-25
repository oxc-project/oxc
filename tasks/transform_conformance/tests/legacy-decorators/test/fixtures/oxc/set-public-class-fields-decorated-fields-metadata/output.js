let _k;
class A {
  constructor() {
    this.s = "s";
  }
  static {
    _k = k();
  }
  static {
    this.n = 1;
  }
  static {
    this[_k] = true;
  }
}
babelHelpers.decorate([dec, babelHelpers.decorateMetadata("design:type", String)], A.prototype, "s", void 0);
babelHelpers.decorate([dec, babelHelpers.decorateMetadata("design:type", Number)], A, "n", void 0);
babelHelpers.decorate([dec, babelHelpers.decorateMetadata("design:type", Boolean)], A, _k, void 0);
babelHelpers.decorate([dec, babelHelpers.decorateMetadata("design:type", Number)], A, "u", void 0);
