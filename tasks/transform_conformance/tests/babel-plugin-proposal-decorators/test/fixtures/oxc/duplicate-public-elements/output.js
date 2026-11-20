var _toPropertyKey = require("@oxc-project/runtime/helpers/toPropertyKey");
var _runInitializers = require("@oxc-project/runtime/helpers/runInitializers");
var _classPrivateFieldInitSpec = require("@oxc-project/runtime/helpers/classPrivateFieldInitSpec");
var _classPrivateFieldGet = require("@oxc-project/runtime/helpers/classPrivateFieldGet2");
var _classPrivateFieldSet = require("@oxc-project/runtime/helpers/classPrivateFieldSet2");
var _decoratorKey = require("@oxc-project/runtime/helpers/decoratorKey");
var _decorateElement = require("@oxc-project/runtime/helpers/decorateElement");
var _decoratorStart = require("@oxc-project/runtime/helpers/decoratorStart");
var _decoratorMetadata = require("@oxc-project/runtime/helpers/decoratorMetadata");
var _member_dec, _value_dec, _x_dec, _x_dec2;
let _key, _key2, _key3, _key4, _key5, _key6, _key7, _init, _C, _toPropertyKey2, _toPropertyKey3;
function dec() {}
const key = Symbol("method");
let _value_accessor_storage = /* @__PURE__ */ new WeakMap();
let _value_accessor_storage2 = /* @__PURE__ */ new WeakMap();
_member_dec = [[void 0, dec]];
_toPropertyKey2 = _toPropertyKey(key);
_key = _decoratorKey();
_toPropertyKey3 = _toPropertyKey(key);
_key2 = _decoratorKey();
_value_dec = [[void 0, dec]];
_key3 = _decoratorKey();
_key4 = _decoratorKey();
_x_dec = [[void 0, dec]];
_key5 = _decoratorKey();
_key6 = _decoratorKey();
_x_dec2 = [[void 0, dec]];
_key7 = _decoratorKey();
class C {
  constructor() {
    _runInitializers(_init, 5, this);
    _classPrivateFieldInitSpec(this, _value_accessor_storage, _runInitializers(_init, 8, this, 1));
    _runInitializers(_init, 11, this);
    _classPrivateFieldInitSpec(this, _value_accessor_storage2, 2);
  }
  [_key]() {
    return super.value;
  }
  [_key2]() {
    return 2;
  }
  get [_key3]() {
    return _classPrivateFieldGet(_value_accessor_storage, this);
  }
  set [_key3](value) {
    _classPrivateFieldSet(_value_accessor_storage, this, value);
  }
  get [_key4]() {
    return _classPrivateFieldGet(_value_accessor_storage2, this);
  }
  set [_key4](value) {
    _classPrivateFieldSet(_value_accessor_storage2, this, value);
  }
  static get [_key5]() {
    return 1;
  }
  static set [_key6](value) {}
  static get [_key7]() {
    return 2;
  }
}
_C = C;
_init = _decoratorStart(null, C, [
  [
    1,
    _toPropertyKey2,
    _key
  ],
  [
    1,
    _toPropertyKey3,
    _key2
  ],
  [
    4,
    "value",
    _key3
  ],
  [
    4,
    "value",
    _key4
  ],
  [
    10,
    "x",
    _key5
  ],
  [
    11,
    "x",
    _key6
  ],
  [
    10,
    "x",
    _key7
  ]
]);
_decorateElement(_init, 10, "x", _x_dec, C, _init.p[4]);
_decorateElement(_init, 11, "x", [], C, _init.p[5]);
_decorateElement(_init, 10, "x", _x_dec2, C, _init.p[6]);
_decorateElement(_init, 1, _toPropertyKey2, _member_dec, C, _init.p[0]);
_decorateElement(_init, 1, _toPropertyKey3, [], C, _init.p[1]);
_decorateElement(_init, 4, "value", _value_dec, C, _init.p[2]);
_decorateElement(_init, 4, "value", [], C, _init.p[3]);
_decoratorMetadata(_init, C);
_runInitializers(_init, 3, C);
