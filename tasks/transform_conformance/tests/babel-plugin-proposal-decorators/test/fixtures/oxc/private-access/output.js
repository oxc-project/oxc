var _classPrivateMethodInitSpec = require("@oxc-project/runtime/helpers/classPrivateMethodInitSpec");
var _runInitializers = require("@oxc-project/runtime/helpers/runInitializers");
var _classPrivateFieldInitSpec = require("@oxc-project/runtime/helpers/classPrivateFieldInitSpec");
var _classPrivateFieldGet = require("@oxc-project/runtime/helpers/classPrivateFieldGet2");
var _classPrivateFieldSet = require("@oxc-project/runtime/helpers/classPrivateFieldSet2");
var _assertClassBrand = require("@oxc-project/runtime/helpers/assertClassBrand");
var _decorateElement = require("@oxc-project/runtime/helpers/decorateElement");
var _decoratorStart = require("@oxc-project/runtime/helpers/decoratorStart");
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
    _classPrivateMethodInitSpec(this, _C_brand);
    _runInitializers(_init, 5, this);
    _classPrivateFieldInitSpec(this, _field, _runInitializers(_init, 16, this, _C));
    _runInitializers(_init, 19, this);
    _classPrivateFieldInitSpec(this, _accessor_accessor_storage, _runInitializers(_init, 8, this, 2));
    _runInitializers(_init, 11, this);
  }
  read() {
    return [
      _assertClassBrand(_C_brand, this, _method).call(this),
      _get_accessor.call(_assertClassBrand(_C_brand, this)),
      _assertClassBrand(_staticBrand, _C, _staticMethod).call(_C)
    ];
  }
};
_C2 = _C;
function _method() {
  return _classPrivateFieldGet(_field, this);
}
function _staticMethod() {
  return _classPrivateFieldGet(_staticField, this);
}
function _get_accessor() {
  return _classPrivateFieldGet(_accessor_accessor_storage, this);
}
function _set_accessor(value) {
  _classPrivateFieldSet(_accessor_accessor_storage, this, value);
}
_init = _decoratorStart(null);
_staticBrand = /* @__PURE__ */ new WeakSet();
_staticBrand.add(_C);
_staticMethod = _decorateElement(_init, 25, "#staticMethod", _member_dec4, _C, [_staticBrand, { value: _staticMethod }]);
_method = _decorateElement(_init, 17, "#method", _member_dec2, _C, [_C_brand, { value: _method }]);
_descriptor = _decorateElement(_init, 20, "#accessor", _member_dec5, _C, [_C_brand, {
  get: _get_accessor,
  set: _set_accessor
}]), _get_accessor = _descriptor.get, _set_accessor = _descriptor.set;
_decorateElement(_init, 29, "#staticField", _member_dec3, _C, [_staticField, {}]);
_decorateElement(_init, 21, "#field", _member_dec, _C, [_field, {}]);
_C2 = _C = _decorateElement(_init, 0, "C", _C_decorators, _C);
_runInitializers(_init, 3, _C);
_classPrivateFieldInitSpec(_C, _staticField, _runInitializers(_init, 12, _C2, 1));
_runInitializers(_init, 15, _C2);
_runInitializers(_init, 1, _C);
let C = _C;
