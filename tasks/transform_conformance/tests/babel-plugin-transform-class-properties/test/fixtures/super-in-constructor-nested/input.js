class C extends S {
  prop = 1;
  constructor() {
    // Insert prop initializer after `super()`
    super();
    this.self = this;
  }
}

class C2 extends S {
  prop = 1;
  constructor() {
    class Inner extends S {
      constructor() {
        // Don't transform - different `super`
        super();
      }
    }

    // Insert prop initializer after `super()`
    super();
    this.self = this;
  }
}

class C3 extends S {
  prop = 1;
  constructor() {
    if (condition) {
      // Transform to `_super()`
      super(1, 2);
      return this;
    }

    // Transform to `_super()`
    super(3, 4);

    if (condition2) {
      // Don't transform
      super(5, 6);
    }

    // Don't transform
    super(7, 8);
  }
}
