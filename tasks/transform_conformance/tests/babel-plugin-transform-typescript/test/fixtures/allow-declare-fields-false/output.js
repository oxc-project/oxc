var _z_dec;
let _init, _Cls;
_z_dec = [[void 0, dce]];
class Cls {
  constructor() {
    babelHelpers.defineProperty(this, "x", void 0);
    babelHelpers.defineProperty(this, "y", 1);
    babelHelpers.defineProperty(this, "z", babelHelpers.runInitializers(_init, 8, this));
    babelHelpers.runInitializers(_init, 11, this);
  }
}
_Cls = Cls;
_init = babelHelpers.decoratorStart(null);
babelHelpers.decorateElement(_init, 5, "z", _z_dec, Cls);
babelHelpers.decoratorMetadata(_init, Cls);
