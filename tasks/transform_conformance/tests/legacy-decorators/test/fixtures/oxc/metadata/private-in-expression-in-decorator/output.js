
import { dec } from "dec";
let Cls = class Cls {
  #zoo = 0;
  foo() {}
  static {
    babelHelpers.decorate([
      dec(#zoo in Cls),
      babelHelpers.decorateMetadata("design:type", Function),
      babelHelpers.decorateMetadata("design:paramtypes", []),
      babelHelpers.decorateMetadata("design:returntype", void 0)
    ], Cls.prototype, "foo", null);
  }
};
Cls = babelHelpers.decorate([dec], Cls);
export { Cls };
let Cls2 = class Cls2 {
  #zoo = 0;
  foo(param) {}
  static {
    babelHelpers.decorate([
      babelHelpers.decorateParam(0, dec(#zoo in Cls2)),
      babelHelpers.decorateMetadata("design:type", Function),
      babelHelpers.decorateMetadata("design:paramtypes", [Number]),
      babelHelpers.decorateMetadata("design:returntype", void 0)
    ], Cls2.prototype, "foo", null);
  }
};
Cls2 = babelHelpers.decorate([dec], Cls2);
export { Cls2 };