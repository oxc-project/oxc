var _assign_dec, _compound_dec, _update_dec, _assign_dec2;
let _init, _init2, _E;
const dec = () => {};
_assign_dec = [[void 0, dec]];
_compound_dec = [[void 0, dec]];
_update_dec = [[void 0, dec]];
let _K = class K {
  static assign() {
    null, babelHelpers.readOnlyError("K");
  }
  static compound() {
    _K + "", babelHelpers.readOnlyError("K");
  }
  static update() {
    +_K, babelHelpers.readOnlyError("K");
  }
  static read() {
    return _K;
  }
};
_init = babelHelpers.decoratorStart(null);
babelHelpers.decorateElement(_init, 9, "assign", _assign_dec, _K);
babelHelpers.decorateElement(_init, 9, "compound", _compound_dec, _K);
babelHelpers.decorateElement(_init, 9, "update", _update_dec, _K);
babelHelpers.decoratorMetadata(_init, _K);
babelHelpers.runInitializers(_init, 3, _K);
let K = _K;
const E = (_assign_dec2 = [[void 0, dec]], _E = class E2 {
  static assign() {
    null, babelHelpers.readOnlyError("E2");
  }
}, _init2 = babelHelpers.decoratorStart(null), babelHelpers.decorateElement(_init2, 9, "assign", _assign_dec2, _E), babelHelpers.decoratorMetadata(_init2, _E), babelHelpers.runInitializers(_init2, 3, _E), _E);
for (const fn of [
  K.assign,
  K.compound,
  K.update,
  E.assign
]) {
  expect(fn).toThrow(TypeError);
}
expect(K.read()).toBe(K);
expect(typeof K).toBe("function");
