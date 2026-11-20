class A {
  p;
  q;
  a;
  b = 1;
  c;
  constructor(p, q = 2) {
    this.p = p;
    this.q = q;
  }
}
babelHelpers.decorate([dec], A.prototype, "a", void 0);
babelHelpers.decorate([dec], A.prototype, "b", void 0);
class B extends Base {
  p;
  a;
  constructor(p) {
    super();
    this.p = p;
  }
}
babelHelpers.decorate([dec], B.prototype, "a", void 0);

