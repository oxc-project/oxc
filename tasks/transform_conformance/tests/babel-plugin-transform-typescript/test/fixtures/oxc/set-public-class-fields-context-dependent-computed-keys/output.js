var _computedKey = Symbol();
let _k2;
function* yieldKey() {
  let _yield$a;
  return class {
    static {
      delete this[_computedKey];
    }
    static {
      this[_yield$a] = 1;
    }
    static [(_yield$a = yield "a", _computedKey)]() {}
  };
}
async function awaitKey(k) {
  let _await$k;
  return class {
    static {
      delete this[_computedKey];
    }
    static {
      this[_await$k] = 1;
    }
    static [(_await$k = await k, _computedKey)]() {}
  };
}
function thisKey() {
  let _this$k;
  return class {
    static {
      delete this[_computedKey];
    }
    static {
      this[_this$k] = 1;
    }
    static [(_this$k = this.k, _computedKey)]() {}
  };
}
function argumentsKey(_k) {
  let _arguments$;
  return class {
    constructor() {
      this[_arguments$] = 1;
    }
    static {
      delete this[_computedKey];
    }
    static [(_arguments$ = arguments[0], _computedKey)]() {}
  };
}
function* afterLastComputedKey() {
  let _yield$a2, _yield$b;
  return class {
    constructor() {
      this[_yield$a2] = 1;
    }
    static {
      delete this[_computedKey];
    }
    [(_yield$a2 = yield "a", k())]() {}
    static {
      this[_yield$b] = 2;
    }
    m() {}
    static [(_yield$b = yield "b", _computedKey)]() {}
  };
}
function* afterHeritage() {
  let _yield$a3;
  return class extends base() {
    static {
      delete this[_computedKey];
    }
    static {
      this[_yield$a3] = 1;
    }
    static [(_yield$a3 = yield "a", _computedKey)]() {}
  };
}
function* beforeDeclaration() {
  let _yield$a4;
  class C {
    static {
      delete this[_computedKey];
    }
    static {
      this[_yield$a4] = 1;
    }
    static [(_yield$a4 = yield "a", _computedKey)]() {}
  }
  return C;
}
function* beforeNamedExpression() {
  let _yield$a5;
  const C = class {
    static {
      delete this[_computedKey];
    }
    static {
      this[_yield$a5] = 1;
    }
    static [(_yield$a5 = yield "a", _computedKey)]() {}
  };
  return C;
}
function* uninitializedKey() {
  return class {
    static {
      delete this[_computedKey];
    }
    static [(yield "a", yield "b", _computedKey)]() {}
  };
}
class StaticBlock {
  static {
    _k2 = k();
  }
  static {
    this[_k2] = 1;
  }
}
