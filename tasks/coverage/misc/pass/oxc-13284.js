// `super()` / `super.prop` / `super.method()` in class constructor
class C extends S {
  constructor(arg = super(), arg2 = super.foo, arg3 = () => super.bar()) {
    super();
    super.foo;
    super.bar();

    () => (arg = super(), arg2 = super.foo, arg3 = () => super.bar()) => {
      if (true) {
        while (false) {
          (arg = super(), arg2 = super.foo, arg3 = () => super.bar()) => {
            super();
            super.foo;
            super.bar();
          };
        }
      }
    }
  }
}

// `super.prop` / `super.method()` in class
class D {
  prop1 = super.foo;
  prop2 = () => super.bar();
  prop3 = () => (arg = super.qux) => {
    if (true) { while (false) { { super.bing; } } }
  };
  prop4 = {
    objMethod(arg = super.foo, arg2 = () => super.bar()) {
      super.foo;
      () => super.bar();
      (arg = super.qux) => {};
      if (true) { while (false) { { super.bing; } } }
    }
  };

  accessor access1 = super.foo;
  accessor access2 = () => super.bar();
  accessor access3 = () => (arg = super.qux) => {
    if (true) { while (false) { { super.bing; } } }
  };
  accessor access4 = {
    objMethod(arg = super.foo, arg2 = () => super.bar()) {
      super.foo;
      () => super.bar();
      (arg = super.qux) => {};
      if (true) { while (false) { { super.bing; } } }
    }
  };

  static prop1 = super.foo;
  static prop2 = () => super.bar();
  static prop3 = () => (arg = super.qux) => {
    if (true) { while (false) { { super.bing; } } }
  };
  static prop4 = {
    objMethod(arg = super.foo, arg2 = () => super.bar()) {
      super.foo;
      () => super.bar();
      (arg = super.qux) => {};
      if (true) { while (false) { { super.bing; } } }
    }
  };

  static accessor access1 = super.foo;
  static accessor access2 = () => super.bar();
  static accessor access3 = () => () => {
    if (true) { while (false) { { super.qux; } } }
  };
  static accessor access4 = {
    objMethod(arg = super.foo, arg2 = () => super.bar()) {
      super.foo;
      () => super.bar();
      (arg = super.qux) => {};
      if (true) { while (false) { { super.bing; } } }
    }
  };

  method(arg = super.foo, arg2 = () => super.bar()) {
    super.foo;
    () => super.bar();
    (arg = super.qux) => {};
    if (true) { while (false) { { super.bing; } } }

    obj = {
      objMethod(arg = super.foo, arg2 = () => super.bar()) {
        super.foo;
        () => super.bar();
        (arg = super.qux) => {};
        if (true) { while (false) { { super.bing; } } }
      }
    };
  }

  static method(arg = super.foo, arg2 = () => super.bar()) {
    super.foo;
    () => super.bar();
    (arg = super.qux) => {};
    if (true) { while (false) { { super.bing; } } }

    obj = {
      objMethod(arg = super.foo, arg2 = () => super.bar()) {
        super.foo;
        () => super.bar();
        (arg = super.qux) => {};
        if (true) { while (false) { { super.bing; } } }
      }
    };
  }

  get x() {
    super.foo;
    () => super.bar();
    (arg = super.qux) => {};
    if (true) { while (false) { { super.bing; } } }

    obj = {
      objMethod(arg = super.foo, arg2 = () => super.bar()) {
        super.foo;
        () => super.bar();
        (arg = super.qux) => {};
        if (true) { while (false) { { super.bing; } } }
      }
    };
  }

  set x(arg = (super.foo, () => super.bar())) {
    super.foo;
    () => super.bar();
    (arg = super.qux) => {};
    if (true) { while (false) { { super.bing; } } }

    obj = {
      objMethod(arg = super.foo, arg2 = () => super.bar()) {
        super.foo;
        () => super.bar();
        (arg = super.qux) => {};
        if (true) { while (false) { { super.bing; } } }
      }
    };
  }

  static {
    super.foo;
    () => super.bar();
    (arg = super.qux) => {};
    if (true) { while (false) { { super.bing; } } }

    obj = {
      objMethod(arg = super.foo, arg2 = () => super.bar()) {
        super.foo;
        () => super.bar();
        (arg = super.qux) => {};
        if (true) { while (false) { { super.bing; } } }
      }
    };
  }
}

// `super()` in nested class
class Outer extends S {
  constructor() {
    class Inner extends S {
      constructor(arg = super()) {
        super();
      }
    }

    // `super` refers to `Outer`'s `super`

    class Inner2 {
      [super()] = 1;
      [super.foo] = 2;
      accessor [super()] = 3;
      accessor [super.foo] = 4;
      [() => 123 + super()]() {}
      [() => 123 + super.bar()]() {}
    }

    class Inner3 extends super() {}
    class Inner4 extends super.foo {}

    @super()
    @super.foo
    class Inner5 {}

    class Inner6 {
      @super()
      @super.foo
      prop = 1;

      @super()
      @super.foo
      static prop = 2;

      @super()
      @super.foo
      accessor access = 3;

      @super()
      @super.foo
      static accessor access = 4;

      @super()
      @super.foo
      method() {}

      @super()
      @super.foo
      static method() {}
    }
  }
}

// `super.prop` / `super.method()` in nested class
class Outer2 {
  // `super` refers to `Outer2`'s `super`

  prop1 = class Inner {
    [super.foo] = 1;
    accessor [super.foo] = 2;
    [(arg = super.foo) => 123 + super.bar()]() {};
    @super.qux prop = 3;
    @super.bing accessor access = 4;
    @super.doom method() {}
  };
  prop2 = class Inner2 extends super.foo {};
  prop3 = 123 + (() => () => 456 + class Inner3 {
    [super.foo] = 1;
    accessor [super.foo] = 2;
    [(arg = super.foo) => 123 + super.bar()]() {};
    @super.qux prop = 3;
    @super.bing accessor access = 4;
    @super.doom method() {}
  });
  prop4 = 123 + (() => () => 456 + class Inner4 extends super.foo {});

  static prop1 = class Inner {
    [super.foo] = 1;
    accessor [super.foo] = 2;
    [(arg = super.foo) => 123 + super.bar()]() {};
    @super.qux prop = 3;
    @super.bing accessor access = 4;
    @super.doom method() {}
  };
  static prop2 = class Inner2 extends super.foo {};
  static prop3 = 123 + (() => () => 456 + class Inner3 {
    [super.foo] = 1;
    accessor [super.foo] = 2;
    [(arg = super.foo) => 123 + super.bar()]() {};
    @super.qux prop = 3;
    @super.bing accessor access = 4;
    @super.doom method() {}
  });
  static prop4 = 123 + (() => () => 456 + class Inner4 extends super.foo {});

  method() {
    class Inner {
      [super.foo] = 1;
      accessor [super.foo] = 2;
      [(arg = super.foo) => 123 + super.bar()]() {};
      @super.qux prop = 3;
      @super.bing accessor access = 4;
      @super.doom method() {}
    }
    class Inner2 extends super.foo {}
    123 + (() => () => 456 + class Inner3 {
      [super.foo] = 1;
      accessor [super.foo] = 2;
      [super.bar()]() {};
      @super.qux prop = 3;
      @super.bing accessor access = 4;
      @super.doom method() {}
    });
    123 + (() => () => 456 + class Inner4 extends super.foo {});
  }

  static method() {
    class Inner {
      [super.foo] = 1;
      accessor [super.foo] = 2;
      [(arg = super.foo) => 123 + super.bar()]() {};
      @super.qux prop = 3;
      @super.bing accessor access = 4;
      @super.doom method() {}
    }
    class Inner2 extends super.foo {}
    123 + (() => () => 456 + class Inner3 {
      [super.foo] = 1;
      accessor [super.foo] = 2;
      [(arg = super.foo) => 123 + super.bar()]() {};
      @super.qux prop = 3;
      @super.bing accessor access = 4;
      @super.doom method() {}
    });
    123 + (() => () => 456 + class Inner4 extends super.foo {});
  }

  get x() {
    class Inner {
      [super.foo] = 1;
      accessor [super.foo] = 2;
      [(arg = super.foo) => 123 + super.bar()]() {};
      @super.qux prop = 3;
      @super.bing accessor access = 4;
      @super.doom method() {}
    }
    class Inner2 extends super.foo {}
    123 + (() => () => 456 + class Inner3 {
      [super.foo] = 1;
      accessor [super.foo] = 2;
      [super.bar()]() {};
      @super.qux prop = 3;
      @super.bing accessor access = 4;
      @super.doom method() {}
    });
    123 + (() => () => 456 + class Inner4 extends super.foo {});
  }

  static {
    class Inner {
      [super.foo] = 1;
      accessor [super.foo] = 2;
      [(arg = super.foo) => 123 + super.bar()]() {};
      @super.qux prop = 3;
      @super.bing accessor access = 4;
      @super.doom method() {}
    }
    class Inner2 extends super.foo {}
    123 + (() => () => 456 + class Inner3 {
      [super.foo] = 1;
      accessor [super.foo] = 2;
      [(arg = super.foo) => 123 + super.bar()]() {};
      @super.qux prop = 3;
      @super.bing accessor access = 4;
      @super.doom method() {}
    });
    123 + (() => () => 456 + class Inner4 extends super.foo {});
  }
}

// `super.prop` / `super.method()` in object method
obj = {
  method(arg = super.foo, arg2 = () => super.bar()) {
    super.foo;
    () => super.bar();
    123 + (() => (arg = super.foo) => 456 + super.qux);
    if (true) { while (false) { { super.bing; } } }
  },
  get x() {
    super.foo;
    () => super.bar();
    123 + (() => (arg = super.foo) => 456 + super.qux);
    if (true) { while (false) { { super.bing; } } }
  },
  set x(arg = (super.foo, () => super.bar())) {
    super.foo;
    () => super.bar();
    123 + (() => (arg = super.foo) => 456 + super.qux);
    if (true) { while (false) { { super.bing; } } }
  },
  outer() {
    return {
      inner(arg = super.foo, arg2 = () => super.bar()) {
        super.foo;
        () => super.bar();
        123 + (() => (arg = super.foo) => 456 + super.qux);
        if (true) { while (false) { { super.bing; } } }
      }
    };
  },
};

// Class nested in object method
obj = {
  method() {
    class Inner {
      prop1 = super.foo;
      prop2 = () => super.bar();
      prop3 = () => (arg = super.qux) => {
        if (true) { while (false) { { super.bing; } } }
      };
      prop4 = {
        objMethod(arg = super.foo, arg2 = () => super.bar()) {
          super.foo;
          () => super.bar();
          (arg = super.qux) => {};
          if (true) { while (false) { { super.bing; } } }
        }
      };

      accessor access1 = super.foo;
      accessor access2 = () => super.bar();
      accessor access3 = () => () => {
        if (true) { while (false) { { super.qux; } } }
      };
      accessor access4 = {
        objMethod(arg = super.foo, arg2 = () => super.bar()) {
          super.foo;
          () => super.bar();
          (arg = super.qux) => {};
          if (true) { while (false) { { super.bing; } } }
        }
      };

      method(arg = super.foo, arg2 = () => super.bar()) {
        super.foo;
        () => super.bar();
        if (true) { while (false) { { super.qux; } } }

        obj = {
          objMethod(arg = super.foo, arg2 = () => super.bar()) {
            super.foo;
            () => super.bar();
            (arg = super.qux) => {};
            if (true) { while (false) { { super.bing; } } }
          }
        };
      }

      get x() {
        super.foo;
        () => super.bar();
        if (true) { while (false) { { super.qux; } } }

        obj = {
          objMethod(arg = super.foo, arg2 = () => super.bar()) {
            super.foo;
            () => super.bar();
            (arg = super.qux) => {};
            if (true) { while (false) { { super.bing; } } }
          }
        };
      }

      set x(arg = (super.foo, () => super.bar())) {
        super.foo;
        () => super.bar();
        if (true) { while (false) { { super.qux; } } }

        obj = {
          objMethod(arg = super.foo, arg2 = () => super.bar()) {
            super.foo;
            () => super.bar();
            (arg = super.qux) => {};
            if (true) { while (false) { { super.bing; } } }
          }
        };
      }

      static {
        super.foo;
        () => super.bar();
        (arg = super.qux) => {};
        if (true) { while (false) { { super.bing; } } }
      }
    }

    // `super` refers to object method's `super`

    class Inner2 {
      [super.foo] = 1;
      accessor [super.foo] = 2;
      [() => 123 + super.bar()]() {}
      @super.qux prop5 = 3;
      @super.bing accessor access = 4;
      @super.doom method() {}
    }

    class Inner3 extends super.foo {}

    @super.foo
    class Inner4 {}

    class Inner5 {
      @super.foo prop = 1;
      @super.foo accessor access = 1;
      @super.foo static prop = 2;
      @super.foo static accessor access = 1;
      @super.foo method() {}
      @super.foo static method() {}
    }
  }
};

// `super()` in deeply nested classes inside class constructor
class Outer3 extends Super {
  constructor() {
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

// `super.foo` in deeply nested classes inside class method
class Outer4 {
  method() {
    class A {
      [
        class B {
          [
            class C {
              [super.foo]() {}
            }
          ]() {}
        }
      ]() {}
    }

    class D extends class E extends class F extends super.foo {} {} {}

    class G {
      @(
        class H {
          @(
            class I {
              @super.foo
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

// `super.foo` in deeply nested classes inside object method
obj = {
  method() {
    class A {
      [
        class B {
          [
            class C {
              [super.foo]() {}
            }
          ]() {}
        }
      ]() {}
    }

    class D extends class E extends class F extends super.foo {} {} {}

    class G {
      @(
        class H {
          @(
            class I {
              @super.foo
              method() {}
            }
          )
          method() {}
        }
      )
      method() {}
    }
  }
};
