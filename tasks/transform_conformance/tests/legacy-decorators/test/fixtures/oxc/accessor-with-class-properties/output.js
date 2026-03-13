var _a_accessor_storage = /* @__PURE__ */ new WeakMap();
var _c_accessor_storage = /* @__PURE__ */ new WeakMap();
class C {
  constructor() {
    babelHelpers.classPrivateFieldInitSpec(this, _a_accessor_storage, void 0);
    babelHelpers.classPrivateFieldInitSpec(this, _c_accessor_storage, "hello");
  }
  get a() {
    return babelHelpers.classPrivateFieldGet2(_a_accessor_storage, this);
  }
  set a(value) {
    babelHelpers.classPrivateFieldSet2(_a_accessor_storage, this, value);
  }
  static get b() {
    return babelHelpers.assertClassBrand(C, this, _b_accessor_storage)._;
  }
  static set b(value) {
    _b_accessor_storage._ = babelHelpers.assertClassBrand(C, this, value);
  }
  get c() {
    return babelHelpers.classPrivateFieldGet2(_c_accessor_storage, this);
  }
  set c(value) {
    babelHelpers.classPrivateFieldSet2(_c_accessor_storage, this, value);
  }
}
var _b_accessor_storage = { _: void 0 };
babelHelpers.decorate([dec], C.prototype, "a", null);
babelHelpers.decorate([dec], C, "b", null);
babelHelpers.decorate([dec], C.prototype, "c", null);
