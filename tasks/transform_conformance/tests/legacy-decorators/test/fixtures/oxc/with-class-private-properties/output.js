let C = class C {
  constructor() {
    babelHelpers.defineProperty(this, "prop", 0);
  }
  meth() {
    return this.prop;
  }
};
C = babelHelpers.decorate([dec], C);

let D = class D {
  constructor() {
    babelHelpers.defineProperty(this, "prop", 0);
  }
  meth() {
    return this.prop;
  }
};
D = babelHelpers.decorate([dec], D);
export { D };

let E = class E {
  constructor() {
    babelHelpers.defineProperty(this, "prop", 0);
  }
  meth() {
    return this.prop;
  }
};
E = babelHelpers.decorate([dec], E);
export default E;