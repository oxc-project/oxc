import { dec } from "dec";
var _Cls, _Cls2;
let Cls = class Cls {
  static {
    _Cls = this;
  }
  #zoo = 0;
  foo() {}
  static {
    babelHelpers.decorate([
      dec(#zoo in _Cls),
      babelHelpers.decorateMetadata("design:type", Function),
      babelHelpers.decorateMetadata("design:paramtypes", []),
      babelHelpers.decorateMetadata("design:returntype", void 0)
    ], _Cls.prototype, "foo", null);
  }
};
Cls = _Cls = babelHelpers.decorate([dec], Cls);
export { Cls };
let Cls2 = class Cls2 {
  static {
    _Cls2 = this;
  }
  #zoo = 0;
  foo(param) {}
  static {
    babelHelpers.decorate([
      babelHelpers.decorateParam(0, dec(#zoo in _Cls2)),
      babelHelpers.decorateMetadata("design:type", Function),
      babelHelpers.decorateMetadata("design:paramtypes", [Number]),
      babelHelpers.decorateMetadata("design:returntype", void 0)
    ], _Cls2.prototype, "foo", null);
  }
};
Cls2 = _Cls2 = babelHelpers.decorate([dec], Cls2);
export { Cls2 };
