var _a;
let _a2;
var _a_accessor_storage = /* @__PURE__ */ new WeakMap();
var _c_accessor_storage = /* @__PURE__ */ new WeakMap();
var _a_computed_accessor_storage = /* @__PURE__ */ new WeakMap();
_a2 = _a = a;
class C {
  constructor() {
    babelHelpers.classPrivateFieldInitSpec(this, _a_accessor_storage, void 0);
    babelHelpers.classPrivateFieldInitSpec(this, _c_accessor_storage, "hello");
    babelHelpers.classPrivateFieldInitSpec(this, _a_computed_accessor_storage, void 0);
  }
  get a() {
    return babelHelpers.classPrivateFieldGet2(_a_accessor_storage, this);
  }
  set a(value) {
    babelHelpers.classPrivateFieldSet2(_a_accessor_storage, this, value);
  }
  static get b() {
    return _b_accessor_storage._;
  }
  static set b(value) {
    _b_accessor_storage._ = value;
  }
  get c() {
    return babelHelpers.classPrivateFieldGet2(_c_accessor_storage, this);
  }
  set c(value) {
    babelHelpers.classPrivateFieldSet2(_c_accessor_storage, this, value);
  }
  get [_a2]() {
    return babelHelpers.classPrivateFieldGet2(_a_computed_accessor_storage, this);
  }
  set [a](value) {
    babelHelpers.classPrivateFieldSet2(_a_computed_accessor_storage, this, value);
  }
}
var _b_accessor_storage = { _: void 0 };
babelHelpers.decorate([dec], C.prototype, "a", null);
babelHelpers.decorate([dec], C, "b", null);
babelHelpers.decorate([dec], C.prototype, "c", null);
babelHelpers.decorate([dec], C.prototype, _a, null);
