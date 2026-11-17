var _readOnlyError = require("@oxc-project/runtime/helpers/readOnlyError");
var _decorateElement = require("@oxc-project/runtime/helpers/decorateElement");
var _decoratorStart = require("@oxc-project/runtime/helpers/decoratorStart");
var _decoratorMetadata = require("@oxc-project/runtime/helpers/decoratorMetadata");
var _runInitializers = require("@oxc-project/runtime/helpers/runInitializers");
var _assign_dec, _compound_dec, _update_dec, _assign_dec2;
let _init, _K2, _init2, _E;
const dec = () => {};
_assign_dec = [[void 0, dec]];
_compound_dec = [[void 0, dec]];
_update_dec = [[void 0, dec]];
let _K = class K {
  static assign() {
    null, _readOnlyError("K");
  }
  static compound() {
    _K + "", _readOnlyError("K");
  }
  static update() {
    +_K, _readOnlyError("K");
  }
  static read() {
    return _K;
  }
};
_K2 = _K;
_init = _decoratorStart(null);
_decorateElement(_init, 9, "assign", _assign_dec, _K);
_decorateElement(_init, 9, "compound", _compound_dec, _K);
_decorateElement(_init, 9, "update", _update_dec, _K);
_decoratorMetadata(_init, _K);
_runInitializers(_init, 3, _K);
let K = _K;
const E = (_assign_dec2 = [[void 0, dec]], _E = class E2 {
  static assign() {
    null, _readOnlyError("E2");
  }
}, _init2 = _decoratorStart(null), _decorateElement(_init2, 9, "assign", _assign_dec2, _E), _decoratorMetadata(_init2, _E), _runInitializers(_init2, 3, _E), _E);
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
