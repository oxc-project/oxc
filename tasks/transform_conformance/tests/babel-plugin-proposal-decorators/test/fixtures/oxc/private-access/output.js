var _C_decorators, _member_dec, _member_dec2, _member_dec3, _member_dec4, _member_dec5, _descriptor;
let _init, _C2, _staticBrand;
let _field = /* @__PURE__ */ new WeakMap();
let _C_brand = /* @__PURE__ */ new WeakSet();
let _staticField = /* @__PURE__ */ new WeakMap();
let _accessor_accessor_storage = /* @__PURE__ */ new WeakMap();
_C_decorators = [[void 0, dec]];
_member_dec = [[void 0, dec]];
_member_dec2 = [[void 0, dec]];
_member_dec3 = [[void 0, dec]];
_member_dec4 = [[void 0, dec]];
_member_dec5 = [[void 0, dec]];
let _C = class C {
  constructor() {
    babelHelpers.classPrivateMethodInitSpec(this, _C_brand);
    babelHelpers.runInitializers(_init, 5, this);
    babelHelpers.classPrivateFieldInitSpec(this, _field, babelHelpers.runInitializers(_init, 16, this, _C));
    babelHelpers.runInitializers(_init, 19, this);
    babelHelpers.classPrivateFieldInitSpec(this, _accessor_accessor_storage, babelHelpers.runInitializers(_init, 8, this, 2));
    babelHelpers.runInitializers(_init, 11, this);
  }
  read() {
    return [
      babelHelpers.assertClassBrand(_C_brand, this, _method).call(this),
      _get_accessor.call(babelHelpers.assertClassBrand(_C_brand, this)),
      babelHelpers.assertClassBrand(_staticBrand, _C, _staticMethod).call(_C)
    ];
  }
};
_C2 = _C;
function _method() {
  return babelHelpers.classPrivateFieldGet2(_field, this);
}
function _staticMethod() {
  return babelHelpers.classPrivateFieldGet2(_staticField, this);
}
function _get_accessor() {
  return babelHelpers.classPrivateFieldGet2(_accessor_accessor_storage, this);
}
function _set_accessor(value) {
  babelHelpers.classPrivateFieldSet2(_accessor_accessor_storage, this, value);
}
_init = babelHelpers.decoratorStart(null);
_staticBrand = /* @__PURE__ */ new WeakSet();
_staticMethod = babelHelpers.decorateElement(_init, 25, "#staticMethod", _member_dec4, _C, [_staticBrand, { value: _staticMethod }]);
_method = babelHelpers.decorateElement(_init, 17, "#method", _member_dec2, _C, [_C_brand, { value: _method }]);
_descriptor = babelHelpers.decorateElement(_init, 20, "#accessor", _member_dec5, _C, [_C_brand, {
  get: _get_accessor,
  set: _set_accessor
}]), _get_accessor = _descriptor.get, _set_accessor = _descriptor.set;
babelHelpers.decorateElement(_init, 29, "#staticField", _member_dec3, _C, [_staticField, {}]);
babelHelpers.decorateElement(_init, 21, "#field", _member_dec, _C, [_field, {}]);
_staticBrand.add(_C);
_C2 = _C = babelHelpers.decorateElement(_init, 0, "C", _C_decorators, _C);
babelHelpers.runInitializers(_init, 3, _C);
babelHelpers.classPrivateFieldInitSpec(_C, _staticField, babelHelpers.runInitializers(_init, 12, _C2, 1));
babelHelpers.runInitializers(_init, 15, _C2);
babelHelpers.runInitializers(_init, 1, _C);
let C = _C;
