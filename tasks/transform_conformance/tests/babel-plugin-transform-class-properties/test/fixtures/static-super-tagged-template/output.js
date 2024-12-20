var _C;

class S {
  static method() {
    return this;
  }
}

class C extends S {}
_C = C;
babelHelpers.defineProperty(C, "prop", babelHelpers.superPropGet(_C, "method", _C).bind(_C)`xyz`);

expect(C.prop).toBe(C);
