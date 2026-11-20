let _key, _key2, _key3;
class A {
  constructor() {
    this[_key] = 1;
    this[_key3] = 3;
  }
  static {
    key("d"), _key2 = key("e"), _key3 = key("f");
  }
  [(key("a"), key("b"), _key = key("c"), key("m"))]() {}
  static {
    this[_key2] = 2;
  }
}
babelHelpers.decorate([dec], A.prototype, null, void 0);
babelHelpers.decorate([dec], A.prototype, null, void 0);
babelHelpers.decorate([dec], A.prototype, "literal", void 0);
babelHelpers.decorate([dec], A, null, void 0);

