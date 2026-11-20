var _runInitializers = require("@oxc-project/runtime/helpers/runInitializers");
var _decorateElement = require("@oxc-project/runtime/helpers/decorateElement");
var _decoratorStart = require("@oxc-project/runtime/helpers/decoratorStart");
var _decoratorMetadata = require("@oxc-project/runtime/helpers/decoratorMetadata");
var _z_dec, _z_dec2;
let _init, _Cls, _init2, _StaticCls;
_z_dec = [[void 0, dce]];
class Cls {
  constructor() {
    this.y = 1;
    this.z = _runInitializers(_init, 8, this);
    _runInitializers(_init, 11, this);
  }
}
_Cls = Cls;
_init = _decoratorStart(null);
_decorateElement(_init, 5, "z", _z_dec, Cls);
_decoratorMetadata(_init, Cls);
_z_dec2 = [[void 0, dce]];
class StaticCls {}
_StaticCls = StaticCls;
_init2 = _decoratorStart(null);
_decorateElement(_init2, 13, "z", _z_dec2, StaticCls);
_decoratorMetadata(_init2, StaticCls);
StaticCls.y = 1;
StaticCls.z = _runInitializers(_init2, 8, _StaticCls);
_runInitializers(_init2, 11, _StaticCls);
