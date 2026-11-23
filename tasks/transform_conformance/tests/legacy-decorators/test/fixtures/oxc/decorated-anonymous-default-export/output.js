function dec(Class) {
  return Class;
}
let _default = class _default {
  constructor() {
    babelHelpers.defineProperty(this, "field", 1);
  }
};
babelHelpers.defineProperty(_default, "value", 42);
_default = babelHelpers.decorate([dec], _default);
export default _default;
