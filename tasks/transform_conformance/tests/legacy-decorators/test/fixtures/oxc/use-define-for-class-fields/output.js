class Cls {
  constructor() {
    this.y = 1;
  }
}
babelHelpers.decorate([dce], Cls.prototype, "z", void 0);

class StaticCls { }
babelHelpers.decorate([dce], StaticCls, "z", void 0);
StaticCls.y = 1;
