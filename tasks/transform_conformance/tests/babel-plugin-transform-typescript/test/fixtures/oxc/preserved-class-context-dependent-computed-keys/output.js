var _computedKey = Symbol();
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
      delete this[_computedKey];
    }
    static {
      this[_yield$a2] = 1;
    }
    static [(_yield$a2 = yield "a", _computedKey)]() {}
  };
}
function* decoratedStaticMethod() {
  let _yield$a3;
  return class {
    static {
      delete this[_computedKey];
    }
    @dec static m() {}
    static {
      this[_yield$a3] = 1;
    }
    static [(_yield$a3 = yield "a", _computedKey)]() {}
  };
}
