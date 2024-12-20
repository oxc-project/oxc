class Outer {
  #prop = 1;

  #shadowed() {
    return 2;
  }

  getInner() {
    class Inner {
      #shadowed() {
        return 3;
      }

      // Refers to `Outer`s private property - transform
      [this.#prop] = 4;
      // Refers to `Inner`s private method - don't transform
      [this.#shadowed] = 5;
    }
    return Inner;
  }

  getInnerWithAdditionalProp() {
    class InnerWithAdditionalProp {
      #shadowed() {
        return 6;
      }

      #innerProp = 7;

      // Refers to `Outer`s private property - transform
      [this.#prop] = 8;
      // Refers to `Inner`s private method - don't transform
      [this.#shadowed] = 9;
    }

    return InnerWithAdditionalProp;
  }
}

let OuterExpr = class {
  #prop = 1;

  #shadowed() {
    return 2;
  }

  getInner() {
    return class InnerExpr {
      #shadowed() {
        return 3;
      }

      // Refers to `Outer`s private property - transform
      [this.#prop] = 4;
      // Refers to `Inner`s private method - don't transform
      [this.#shadowed] = 5;
    };
  }

  getInnerWithAdditionalProp() {
    return class InnerExprWithAdditionalProp {
      #shadowed() {
        return 6;
      }

      #innerProp = 7;

      // Refers to `Outer`s private property - transform
      [this.#prop] = 8;
      // Refers to `Inner`s private method - don't transform
      [this.#shadowed] = 9;
    };
  }
};

const outer = new Outer();
expect(() => outer.getInner()).toThrowError("Receiver must be an instance of class Inner");
expect(() => outer.getInnerWithAdditionalProp()).toThrowError(
  "Receiver must be an instance of class InnerWithAdditionalProp"
);

const outerExpr = new OuterExpr();
expect(() => outerExpr.getInner()).toThrowError("Receiver must be an instance of class InnerExpr");
expect(() => outerExpr.getInnerWithAdditionalProp()).toThrowError(
  "Receiver must be an instance of class InnerExprWithAdditionalProp"
);
