var _key, _key2, _key3, _key4;
let _key5, _key6, _key7;
class A {
  constructor() {
    this[_key5] = 1;
    this[_key7] = 3;
  }
  static {
    _key3 = key("d"), _key6 = _key4 = key("e"), _key7 = key("f");
  }
  [(key("a"), _key = key("b"), _key5 = _key2 = key("c"), key("m"))]() {}
  static {
    this[_key6] = 2;
  }
}
babelHelpers.decorate([dec], A.prototype, _key, void 0);
babelHelpers.decorate([dec], A.prototype, _key2, void 0);
babelHelpers.decorate([dec], A.prototype, "literal", void 0);
babelHelpers.decorate([dec], A, _key3, void 0);
babelHelpers.decorate([dec], A, _key4, void 0);

