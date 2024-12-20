class Outer {
  #prop = 1;
  #shadowed = 2;

  method() {
    class Inner {
      #shadowed() {}

      method() {
        // Refers to `Outer`s private property - transform
        this.#prop;
        // Refers to `Inner`s private method - don't transform
        this.#shadowed;
      }
    }

    class InnerWithAdditionalProp {
      #shadowed() {}

      #innerProp = 3;

      method() {
        // Refers to `Outer`s private property - transform
        this.#prop;
        // Refers to `Inner`s private method - don't transform
        this.#shadowed;
      }
    }
  }
}

let OuterExpr = class {
  #prop = 1;
  #shadowed = 2;

  method() {
    let InnerExpr = class {
      #shadowed() {}

      method() {
        // Refers to `Outer`s private property - transform
        this.#prop;
        // Refers to `Inner`s private method - don't transform
        this.#shadowed;
      }
    };

    let InnerExprWithAdditionalProp = class {
      #shadowed() {}

      #innerProp = 3;

      method() {
        // Refers to `Outer`s private property - transform
        this.#prop;
        // Refers to `Inner`s private method - don't transform
        this.#shadowed;
      }
    };
  }
};
