// `super()` not in a class or object method
super();
() => () => 123 + super();
if (true) { while (false) { { super(); } } }
() => (arg = super()) => {
  super();
  () => () => 123 + super();
  if (true) { while (false) { { super(); } } }
};

// `super()` in a function
function f(arg = super()) {
  super();
  () => () => 123 + super();
  if (true) { while (false) { { super(); } } }
}
f = function(arg = super()) {
  super();
  () => () => 123 + super();
  if (true) { while (false) { { super(); } } }
};

// `super()` in class constructor of class without super class
class A {
  constructor(arg = super()) {
    super();
    () => (arg = super()) => 123 + super();
    if (true) { while (false) { { super(); } } }
  }
}

// `super()` in class properties
class B extends Super {
  prop = super();
  static prop = () => (arg = super()) => 123 + super();

  accessor access = super();
  static accessor access = () => () => 123 + super();
}

// `super()` in class methods / getters / setters
class C extends Super {
  method(arg = super()) {
    super();
    () => (arg = super()) => 123 + super();
    if (true) { while (false) { { super(); } } }
  }

  static method(arg = super()) {
    super();
    () => (arg = super()) => 123 + super();
    if (true) { while (false) { { super(); } } }
  }

  ['x'](arg = super()) {
    super();
    () => (arg = super()) => 123 + super();
    if (true) { while (false) { { super(); } } }
  }

  static ['x'](arg = super()) {
    super();
    () => (arg = super()) => 123 + super();
    if (true) { while (false) { { super(); } } }
  }

  get y() {
    super();
    () => (arg = super()) => 123 + super();
    if (true) { while (false) { { super(); } } }
  }

  set y(arg = super()) {
    super();
    () => (arg = super()) => 123 + super();
    if (true) { while (false) { { super(); } } }
  }

  static get y() {
    super();
    () => (arg = super()) => 123 + super();
    if (true) { while (false) { { super(); } } }
  }

  static set y(arg = super()) {
    super();
    () => (arg = super()) => 123 + super();
    if (true) { while (false) { { super(); } } }
  }
}

// `super()` in class static block
class D extends Super {
  static {
    super();
    () => (arg = super()) => 123 + super();
    if (true) { while (false) { { super(); } } }
  }
}

// `super()` in function or object method inside class constructor
class E extends Super {
  constructor() {
    function inner(arg = super()) {
      super();
      () => (arg = super()) => 123 + super();
      if (true) { while (false) { { super(); } } }
    }
    f = () => function(arg = super()) {
      super();
      () => (arg = super()) => 123 + super();
      if (true) { while (false) { { super(); } } }
    };
    obj = {
      method(arg = super()) {
        super();
        () => (arg = super()) => 123 + super();
        if (true) { while (false) { { super(); } } }
      },
      set x(arg = super()) {
        super();
        () => (arg = super()) => 123 + super();
        if (true) { while (false) { { super(); } } }
      },
    };
  }
}

// `super()` in class computed keys
class F extends Super {
  [super()] = 1;
  static [(arg = super()) => super()] = 2;
  accessor [123 + super()] = 3;
  static accessor [() => (arg = super()) => 123 + super()] = 4;
  [super()]() {};
  static [() => super()]() {};
  get [super()]() {};
  static get [() => super()]() {};
  set [super()](v) {};
  static set [() => () => 123 + super()](v) {};
}

// `super()` in class extends
class G extends super() {}
class H extends (() => () => 123 + super()) {}

// `super()` in class decorators
@super()
class I extends Super {
  @super() prop = 1;
  @super() static prop = 2;
  @super() accessor access = 3;
  @super() static accessor access = 4;
  @super() method() {}
}

// `super()` in properties / methods of class inside class constructor
class J extends Super {
  constructor() {
    class Inner {
      prop = super();
      static prop = (arg = super()) => 123 + super();
      accessor access = super();
      static accessor access = (arg = super()) => 123 + super();
      method() { super(); }
      static method() { (arg = super()) => 123 + super(); }
      get x() { super(); }
      static get y() { (arg = super()) => 123 + super(); }
      set x(v) { super(); }
      static set y(v) { (arg = super()) => 123 + super(); }
      static { super(); }
    }
  }
}

// `super()` in computed keys of class inside class property
class K extends Super {
  prop = class Inner {
    [super()] = 1;
    static [(arg = super()) => super()] = 2;
    accessor [123 + super()] = 3;
    static accessor [() => (arg = super()) => 123 + super()] = 4;
    [super()]() {};
    static [() => super()]() {};
    get [super()]() {};
    static get [() => super()]() {};
    set [super()](v) {};
    static set [() => () => 123 + super()](v) {};
  };
}

// `super()` in computed keys of class inside class method
class L extends Super {
  method() {
    class Inner {
      [super()] = 1;
      static [(arg = super()) => super()] = 2;
      accessor [123 + super()] = 3;
      static accessor [() => (arg = super()) => 123 + super()] = 4;
      [super()]() {};
      static [() => super()]() {};
      get [super()]() {};
      static get [() => super()]() {};
      set [super()](v) {};
      static set [() => () => 123 + super()](v) {};
    }
  }
}

// `super()` in extends clause of class inside class property
class M extends Super {
  prop1 = class Inner extends super() {};
  prop2 = class Inner extends (() => (arg = super()) => 123 + super()) {};
}

// `super()` in extends clause of class inside class method
class N extends Super {
  method() {
    class Inner1 extends super() {};
    class Inner2 extends (() => (arg = super()) => 123 + super()) {};
  }
}

// `super()` in decorators of class inside class property
class O extends Super {
  prop = @super() class Inner extends Super {
    @super() prop = 1;
    @super() static prop = 2;
    @super() accessor access = 3;
    @super() static accessor access = 4;
    @super() method() {}
  };
}

// `super()` in decorators of class inside class method
class P extends Super {
  method() {
    @super()
    class Inner extends Super {
      @super() prop = 1;
      @super() static prop = 2;
      @super() accessor access = 3;
      @super() static accessor access = 4;
      @super() method() {}
    }
  }
}

// `super()` in deeply nested classes inside class method
class Q extends Super {
  method() {
    class A {
      [
        class B {
          [
            class C {
              [super()]() {}
            }
          ]() {}
        }
      ]() {}
    }

    class D extends class E extends class F extends super() {} {} {}

    class G {
      @(
        class H {
          @(
            class I {
              @super()
              method() {}
            }
          )
          method() {}
        }
      )
      method() {}
    }
  }
}

// `super()` in object methods
obj = {
  method() {
    super();
    () => (arg = super()) => 123 + super();
    if (true) { while (false) { { super(); } } }
  },
  ['x']() {
    super();
    () => (arg = super()) => 123 + super();
    if (true) { while (false) { { super(); } } }
  },
  get x() {
    super();
    () => (arg = super()) => 123 + super();
    if (true) { while (false) { { super(); } } }
  },
  set x(v) {
    super();
    () => (arg = super()) => 123 + super();
    if (true) { while (false) { { super(); } } }
  },
};
