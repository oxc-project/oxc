
class A {
  prop = 0;
  constructor() {}
}

class B {
  prop = 0;
  constructor() {}
}
babelHelpers.decorate([dec, babelHelpers.decorateMetadata("design:type", Object)], B.prototype, "prop", void 0);

let C = class C {
  prop = 0;
  constructor() {}
};
C = babelHelpers.decorate([dec, babelHelpers.decorateMetadata("design:paramtypes", [])], C);
