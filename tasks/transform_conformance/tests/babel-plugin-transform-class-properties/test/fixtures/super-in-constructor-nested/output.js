class C extends S {
  constructor() {
    // Insert prop initializer after `super()`
    super();
    babelHelpers.defineProperty(this, "prop", 1);
    this.self = this;
  }
}

class C2 extends S {
  constructor() {
    class Inner extends S {
      constructor() {
        // Don't transform - different `super`
        super();
      }
    }

    // Insert prop initializer after `super()`
    super();
    babelHelpers.defineProperty(this, "prop", 1);
    this.self = this;
  }
}

class C3 extends S {
  constructor() {
    var _super = (..._args) => (super(..._args), babelHelpers.defineProperty(this, "prop", 1), this);

    if (condition) {
      // Transform to `_super()`
      _super(1, 2);
      return this;
    }

    // Transform to `_super()`
    _super(3, 4);

    if (condition2) {
      // Don't transform
      super(5, 6);
    }

    // Don't transform
    super(7, 8);
  }
}
