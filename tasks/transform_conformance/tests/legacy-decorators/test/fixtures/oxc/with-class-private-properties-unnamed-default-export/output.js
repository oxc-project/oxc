let _default = class _default {
  constructor() {
    babelHelpers.defineProperty(this, "prop", 0);
  }
  meth() {
    return this.prop;
  }
};
_default = babelHelpers.decorate([dec], _default);
export default _default;
