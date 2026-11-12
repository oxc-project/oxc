var _foo$bar, _a, _foo$bar2;
class C {
  #_a_accessor_storage;
  get a() {
    return this.#_a_accessor_storage;
  }
  set a(value) {
    this.#_a_accessor_storage = value;
  }
  static #_b_accessor_storage;
  static get b() {
    return C.#_b_accessor_storage;
  }
  static set b(value) {
    C.#_b_accessor_storage = value;
  }
  #_c_accessor_storage = "hello";
  get c() {
    return this.#_c_accessor_storage;
  }
  set c(value) {
    this.#_c_accessor_storage = value;
  }
  #_0_accessor_storage;
  get [_a = a]() {
    return this.#_0_accessor_storage;
  }
  set [a](value) {
    this.#_0_accessor_storage = value;
  }
  #_1_accessor_storage;
  get [_foo$bar2 = _foo$bar = foo.bar]() {
    return this.#_1_accessor_storage;
  }
  set [_foo$bar](value) {
    this.#_1_accessor_storage = value;
  }
  #_d_accessor_storage;
  get d() {
    return this.#_d_accessor_storage;
  }
  set d(value) {
    this.#_d_accessor_storage = value;
  }
  #_e_accessor_storage;
  get #e() {
    return this.#_e_accessor_storage;
  }
  set #e(value) {
    this.#_e_accessor_storage = value;
  }
}
babelHelpers.decorate([dec], C.prototype, "a", null);
babelHelpers.decorate([dec], C, "b", null);
babelHelpers.decorate([dec], C.prototype, "c", null);
babelHelpers.decorate([dec], C.prototype, _a, null);
babelHelpers.decorate([dec], C.prototype, _foo$bar2, null);
