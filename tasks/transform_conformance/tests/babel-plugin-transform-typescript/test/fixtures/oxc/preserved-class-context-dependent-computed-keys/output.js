function* autoAccessor() {
  let _yield$a, _arguments$;
  return class {
    constructor() {
      this[_arguments$] = 2;
    }
    static {
      this[_yield$a] = 1;
    }
    accessor x = 0;
    [(_yield$a = yield "a", _arguments$ = arguments[0])];
  };
}
function* decorated() {
  let _yield$a2;
  return @dec class {
    static {
      this[_yield$a2] = 1;
    }
    @(_yield$a2 = yield "a", () => {}) static #_computedKey() {}
  };
}
function* decoratedStaticMethod() {
  let _yield$a3;
  return class {
    @dec static m() {}
    static {
      this[_yield$a3] = 1;
    }
    @(_yield$a3 = yield "a", () => {}) static #_computedKey() {}
  };
}
