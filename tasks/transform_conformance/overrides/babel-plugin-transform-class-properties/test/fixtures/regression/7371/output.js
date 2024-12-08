"use strict";

class C {}

class A extends C {
  constructor() {
    super();
    babelHelpers.defineProperty(this, "field", 1);
    class B extends C {
      constructor() {
        super();
        expect(this.field).toBeUndefined();
      }
    }
    expect(this.field).toBe(1);
    new B();
  }
}
new A();

class Obj {
  constructor() {
    return {};
  }
}

// ensure superClass is still transformed
class SuperClass extends Obj {
  constructor() {
    var _super = (..._args) => (
      super(..._args),
      babelHelpers.defineProperty(this, "field", 1),
      this
    );
    class B extends (_super(), Obj) {
      constructor() {
        super();
        expect(this.field).toBeUndefined();
      }
    }
    expect(this.field).toBe(1);
    new B();
  }
}
new SuperClass();

// ensure ComputedKey Method is still transformed
class ComputedMethod extends Obj {
  constructor() {
    var _super2 = (..._args2) => (
      super(..._args2),
      babelHelpers.defineProperty(this, "field", 1),
      this
    );
    class B extends Obj {
      constructor() {
        super();
        expect(this.field).toBeUndefined();
      }
      [_super2()]() {}
    }
    expect(this.field).toBe(1);
    new B();
  }
}
new ComputedMethod();

// ensure ComputedKey Field is still transformed
class ComputedField extends Obj {
  constructor() {
    let _super4;
    var _super3 = (..._args3) => (
      super(..._args3),
      babelHelpers.defineProperty(this, "field", 1),
      this
    );
    _super4 = _super3();
    class B extends Obj {
      constructor() {
        super();
        babelHelpers.defineProperty(this, _super4, 1);
        expect(this.field).toBeUndefined();
      }
    }
    expect(this.field).toBe(1);
    new B();
  }
}
new ComputedField();
