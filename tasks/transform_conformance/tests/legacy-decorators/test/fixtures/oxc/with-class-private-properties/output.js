let C = class C {
  #prop = 0;
  meth() {
    return this.#prop;
  }
};
C = babelHelpers.decorate([dec], C);

let D = class D {
  #prop = 0;
  meth() {
    return this.#prop;
  }
};
D = babelHelpers.decorate([dec], D);

export { D };
let E = class E {
  #prop = 0;
  meth() {
    return this.#prop;
  }
};
E = babelHelpers.decorate([dec], E);
export default E;
