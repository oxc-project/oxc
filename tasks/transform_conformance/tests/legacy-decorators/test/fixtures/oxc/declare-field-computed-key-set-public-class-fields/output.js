let _a, _b, _c, _g, _h;
class Foo {
  constructor() {
    this[_a] = 1;
    this[_g] = 2;
  }
  [_a = a()];
  [(_b = b(), _c = c(), _g = g())];
}
babelHelpers.decorate([dec], Foo.prototype, _b, void 0);
babelHelpers.decorate([dec], Foo, _c, void 0);
class Bar {
  static {
    _h = h();
  }
}
babelHelpers.decorate([dec], Bar.prototype, _h, void 0);
