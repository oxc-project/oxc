var _prop2, _shadowed2;

var _prop = new WeakMap();
var _shadowed = new WeakMap();

class Outer {
  constructor() {
    babelHelpers.classPrivateFieldInitSpec(this, _prop, 1);
    babelHelpers.classPrivateFieldInitSpec(this, _shadowed, 2);
  }

  method() {
    class Inner {
      #shadowed() {}

      method() {
        // Refers to `Outer`s private property - transform
        babelHelpers.classPrivateFieldGet2(_prop, this);
        // Refers to `Inner`s private method - don't transform
        this.#shadowed;
      }
    }

    var _innerProp = new WeakMap();
    class InnerWithAdditionalProp {
      constructor() {
        babelHelpers.classPrivateFieldInitSpec(this, _innerProp, 3);
      }

      #shadowed() {}

      method() {
        // Refers to `Outer`s private property - transform
        babelHelpers.classPrivateFieldGet2(_prop, this);
        // Refers to `Inner`s private method - don't transform
        this.#shadowed;
      }
    }
  }
}

let OuterExpr = (
  _prop2 = new WeakMap(),
  _shadowed2 = new WeakMap(),
  class {
    constructor() {
      babelHelpers.classPrivateFieldInitSpec(this, _prop2, 1);
      babelHelpers.classPrivateFieldInitSpec(this, _shadowed2, 2);
    }

    method() {
      var _innerProp2;

      let InnerExpr = class {
        #shadowed() {}

        method() {
          babelHelpers.classPrivateFieldGet2(_prop2, this);
          this.#shadowed;
        }
      };

      let InnerExprWithAdditionalProp = (
        _innerProp2 = new WeakMap(),
        class {
          constructor() {
            babelHelpers.classPrivateFieldInitSpec(this, _innerProp2, 3);
          }

          #shadowed() {}

          method() {
            babelHelpers.classPrivateFieldGet2(_prop2, this);
            this.#shadowed;
          }
        }
      );
    }
  }
);
