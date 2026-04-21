// `super.foo` not in a class or object method
super.foo;
() => (arg = super.foo) => 123 + super.foo;
if (true) { while (false) { { super.foo; } } }
() => () => {
  super.foo;
  () => (arg = super.foo) => 123 + super.foo;
  if (true) { while (false) { { super.foo; } } }
};

// `super.foo` in a function
function f(arg = super.foo) {
  super.foo;
  () => (arg = super.foo) => 123 + super.foo;
  if (true) { while (false) { { super.foo; } } }
}
f = function(arg = super.foo) {
  super.foo;
  () => (arg = super.foo) => 123 + super.foo;
  if (true) { while (false) { { super.foo; } } }
};

// `super.foo` in function inside class constructor
class A extends Super {
  constructor() {
    function inner(arg = super.foo) {
      super.foo;
      () => (arg = super.foo) => 123 + super.foo;
      if (true) { while (false) { { super.foo; } } }
    }
    f = () => function(arg = super.foo) {
      super.foo;
      () => (arg = super.foo) => 123 + super.foo;
      if (true) { while (false) { { super.foo; } } }
    };
  }
}

// `super.foo` in function inside class method
class B extends Super {
  method() {
    function inner(arg = super.foo) {
      super.foo;
      () => (arg = super.foo) => 123 + super.foo;
      if (true) { while (false) { { super.foo; } } }
    }
    f = () => function(arg = super.foo) {
      super.foo;
      () => (arg = super.foo) => 123 + super.foo;
      if (true) { while (false) { { super.foo; } } }
    };
  }
}

// `super.foo` in function inside class property
class C extends Super {
  prop = () => {
    function inner(arg = super.foo) {
      super.foo;
      () => (arg = super.foo) => 123 + super.foo;
      if (true) { while (false) { { super.foo; } } }
    }
    f = () => function(arg = super.foo) {
      super.foo;
      () => (arg = super.foo) => 123 + super.foo;
      if (true) { while (false) { { super.foo; } } }
    };
  };
}

// `super.foo` in class computed keys
class D {
  [super.foo] = 1;
  static [() => super.foo] = 2;
  accessor [123 + super.foo] = 3;
  static accessor [() => () => 123 + super.foo] = 4;
  [super.foo]() {};
  static [() => super.foo]() {};
  get [super.foo]() {};
  static get [() => super.foo]() {};
  set [super.foo](v) {};
  static set [() => () => 123 + super.foo](v) {};
}

// `super.foo` in class extends
class E extends super.foo {}
class F extends (() => (arg = super.foo) => 123 + super.foo) {}

// `super.foo` in class decorators
@super.foo
class G extends Super {
  @super.foo prop = 1;
  @super.foo static prop = 2;
  @super.foo accessor access = 3;
  @super.foo static accessor access = 4;
  @super.foo method() {}
}

// `super.foo` in computed keys in class inside function
function g() {
  class Inner {
    [super.foo] = 1;
    static [(arg = super.foo) => super.foo] = 2;
    accessor [123 + super.foo] = 3;
    static accessor [() => (arg = super.foo) => 123 + super.foo] = 4;
    [super.foo]() {};
    static [() => super.foo]() {};
    get [super.foo]() {};
    static get [() => super.foo]() {};
    set [super.foo](v) {};
    static set [() => () => 123 + super.foo](v) {};
  }
}

// `super.foo` in extends clause of class inside function
function h() {
  class E extends super.foo {}
  class F extends (() => (arg = super.foo) => 123 + super.foo) {}
}

// `super.foo` in decorators in class inside function
function i() {
  @super.foo
  class Inner {
    @super.foo prop = 1;
    @super.foo static prop = 2;
    @super.foo accessor access = 3;
    @super.foo static accessor access = 4;
    @super.foo method() {}
  }
}

// `super.foo` in object properties
obj = {
  prop: (arg = super.foo) => {
    super.foo;
    () => (arg = super.foo) => 123 + super.foo;
    if (true) { while (false) { { super.foo; } } }
  },
  ['x']: (arg = super.foo) => {
    super.foo;
    () => (arg = super.foo) => 123 + super.foo;
    if (true) { while (false) { { super.foo; } } }
  },
};
