var _z_dec, _z_dec2;
let _init, _Cls, _init2, _StaticCls;
_z_dec = [[void 0, dce]];
class Cls {
  constructor() {
    this.y = 1;
    this.z = babelHelpers.runInitializers(_init, 8, this);
    babelHelpers.runInitializers(_init, 11, this);
  }
}
_Cls = Cls;
_init = babelHelpers.decoratorStart(null);
babelHelpers.decorateElement(_init, 5, "z", _z_dec, Cls);
babelHelpers.decoratorMetadata(_init, Cls);
_z_dec2 = [[void 0, dce]];
class StaticCls {}
_StaticCls = StaticCls;
_init2 = babelHelpers.decoratorStart(null);
babelHelpers.decorateElement(_init2, 13, "z", _z_dec2, StaticCls);
babelHelpers.decoratorMetadata(_init2, StaticCls);
StaticCls.y = 1;
StaticCls.z = babelHelpers.runInitializers(_init2, 8, _StaticCls);
babelHelpers.runInitializers(_init2, 11, _StaticCls);
