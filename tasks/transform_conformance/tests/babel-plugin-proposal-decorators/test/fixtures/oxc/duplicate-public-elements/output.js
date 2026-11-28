var _member_dec, _value_dec, _x_dec, _x_dec2;
let _key, _key2, _key3, _key4, _key5, _key6, _key7, _init, _C, _babelHelpers$toPrope, _babelHelpers$toPrope2;
function dec() {}
const key = Symbol("method");
let _value_accessor_storage = /* @__PURE__ */ new WeakMap();
let _value_accessor_storage2 = /* @__PURE__ */ new WeakMap();
_member_dec = [[void 0, dec]];
_babelHelpers$toPrope = babelHelpers.toPropertyKey(key);
_key = babelHelpers.decoratorKey();
_babelHelpers$toPrope2 = babelHelpers.toPropertyKey(key);
_key2 = babelHelpers.decoratorKey();
_value_dec = [[void 0, dec]];
_key3 = babelHelpers.decoratorKey();
_key4 = babelHelpers.decoratorKey();
_x_dec = [[void 0, dec]];
_key5 = babelHelpers.decoratorKey();
_key6 = babelHelpers.decoratorKey();
_x_dec2 = [[void 0, dec]];
_key7 = babelHelpers.decoratorKey();
class C {
  constructor() {
    babelHelpers.runInitializers(_init, 5, this);
    babelHelpers.classPrivateFieldInitSpec(this, _value_accessor_storage, babelHelpers.runInitializers(_init, 8, this, 1));
    babelHelpers.runInitializers(_init, 11, this);
    babelHelpers.classPrivateFieldInitSpec(this, _value_accessor_storage2, 2);
  }
  [_key]() {
    return super.value;
  }
  [_key2]() {
    return 2;
  }
  get [_key3]() {
    return babelHelpers.classPrivateFieldGet2(_value_accessor_storage, this);
  }
  set [_key3](value) {
    babelHelpers.classPrivateFieldSet2(_value_accessor_storage, this, value);
  }
  get [_key4]() {
    return babelHelpers.classPrivateFieldGet2(_value_accessor_storage2, this);
  }
  set [_key4](value) {
    babelHelpers.classPrivateFieldSet2(_value_accessor_storage2, this, value);
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
_init = babelHelpers.decoratorStart(null, C, [
  [
    1,
    _babelHelpers$toPrope,
    _key
  ],
  [
    1,
    _babelHelpers$toPrope2,
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
babelHelpers.decorateElement(_init, 10, "x", _x_dec, C, _init.p[4]);
babelHelpers.decorateElement(_init, 11, "x", [], C, _init.p[5]);
babelHelpers.decorateElement(_init, 10, "x", _x_dec2, C, _init.p[6]);
babelHelpers.decorateElement(_init, 1, _babelHelpers$toPrope, _member_dec, C, _init.p[0]);
babelHelpers.decorateElement(_init, 1, _babelHelpers$toPrope2, [], C, _init.p[1]);
babelHelpers.decorateElement(_init, 4, "value", _value_dec, C, _init.p[2]);
babelHelpers.decorateElement(_init, 4, "value", [], C, _init.p[3]);
babelHelpers.decoratorMetadata(_init, C);
babelHelpers.runInitializers(_init, 3, C);
