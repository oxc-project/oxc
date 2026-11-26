import { k } from "./k";
let _b, _c, _d, _h, _i, _n;
let n = 0;
class Foo {
  [a()] = 1;
  [(_b = b(), _c = c(), _d = d, g())] = 2;
}
babelHelpers.decorate([dec], Foo.prototype, _b, void 0);
babelHelpers.decorate([dec], Foo.prototype, _d, void 0);
babelHelpers.decorate([dec], Foo.prototype, k, void 0);
babelHelpers.decorate([dec], Foo.prototype, "e", void 0);
babelHelpers.decorate([dec], Foo, _c, void 0);
class Bar {
  static {
    _h = h(), _i = i(), _n = n++;
  }
}
babelHelpers.decorate([dec], Bar.prototype, _h, void 0);
babelHelpers.decorate([dec], Bar.prototype, _n, void 0);
babelHelpers.decorate([dec], Bar, _i, void 0);
