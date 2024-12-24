class C extends S {
  constructor() {
    var _super = (..._args) => (super(..._args), babelHelpers.defineProperty(this, "prop", 1), this);
    return {};
  }
}
