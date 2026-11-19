var _defineProperty = require("@oxc-project/runtime/helpers/defineProperty");
var _runInitializers = require("@oxc-project/runtime/helpers/runInitializers");
var _decorateElement = require("@oxc-project/runtime/helpers/decorateElement");
var _decoratorStart = require("@oxc-project/runtime/helpers/decoratorStart");
var _decoratorMetadata = require("@oxc-project/runtime/helpers/decoratorMetadata");
var _z_dec;
let _init;
_z_dec = [[void 0, dce]];
class Cls {
  constructor() {
    _defineProperty(this, "y", 1);
    _defineProperty(this, "z", _runInitializers(_init, 8, this));
    _runInitializers(_init, 11, this);
  }
}
_init = _decoratorStart(null);
_decorateElement(_init, 5, "z", _z_dec, Cls);
_decoratorMetadata(_init, Cls);
