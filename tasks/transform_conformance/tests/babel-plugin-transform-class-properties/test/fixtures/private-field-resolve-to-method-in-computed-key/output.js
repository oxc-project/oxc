var _prop2;

var _prop = new WeakMap();

class Outer {
  constructor() {
    babelHelpers.classPrivateFieldInitSpec(this, _prop, 1);
  }

  #shadowed() {
    return 2;
  }

  getInner() {
    let _this$prop, _this$shadowed;

    _this$prop = babelHelpers.classPrivateFieldGet2(_prop, this);
    // This isn't an existing helper. We need to add a helper that throws error.
    // Problem only arises when class properties transform enabled and private methods transform isn't.
    _this$shadowed = babelHelpers.illegalPrivateField("shadowed");

    class Inner {
      constructor() {
        babelHelpers.defineProperty(this, _this$prop, 4);
        babelHelpers.defineProperty(this, _this$shadowed, 5);
      }

      #shadowed() {
        return 3;
      }
    }
    return Inner;
  }

  getInnerWithAdditionalProp() {
    let _this$prop2, _this$shadowed2;

    var _innerProp = new WeakMap();
    _this$prop2 = babelHelpers.classPrivateFieldGet2(_prop, this);
    // This isn't an existing helper. We need to add a helper that throws error.
    // Problem only arises when class properties transform enabled and private methods transform isn't.
    _this$shadowed2 = babelHelpers.illegalPrivateField("shadowed");

    class InnerWithAdditionalProp {
      constructor() {
        babelHelpers.classPrivateFieldInitSpec(this, _innerProp, 7);
        babelHelpers.defineProperty(this, _this$prop2, 8);
        babelHelpers.defineProperty(this, _this$shadowed2, 9);
      }

      #shadowed() {
        return 6;
      }
    }

    return InnerWithAdditionalProp;
  }
}

let OuterExpr = (
  _prop2 = new WeakMap(),
  class {
    constructor() {
      babelHelpers.classPrivateFieldInitSpec(this, _prop2, 1);
    }

    #shadowed() {
      return 2;
    }

    getInner() {
      let _this$prop3, _this$shadowed3;
      return (
        _this$prop3 = babelHelpers.classPrivateFieldGet2(_prop2, this),
        // This isn't an existing helper. We need to add a helper that throws error.
        // Problem only arises when class properties transform enabled and private methods transform isn't.
        _this$shadowed3 = babelHelpers.illegalPrivateField("shadowed"),
        class InnerExpr {
          constructor() {
            babelHelpers.defineProperty(this, _this$prop3, 4);
            babelHelpers.defineProperty(this, _this$shadowed3, 5);
          }

          #shadowed() {
            return 3;
          }
        }
      );
    }

    getInnerWithAdditionalProp() {
      var _innerProp2;
      let _this$prop4, _this$shadowed4;
      return (
        _innerProp2 = new WeakMap(),
        _this$prop4 = babelHelpers.classPrivateFieldGet2(_prop2, this),
        // This isn't an existing helper. We need to add a helper that throws error.
        // Problem only arises when class properties transform enabled and private methods transform isn't.
        _this$shadowed4 = babelHelpers.illegalPrivateField("shadowed"),
        class InnerExprWithAdditionalProp {
          constructor() {
            babelHelpers.classPrivateFieldInitSpec(this, _innerProp2, 7);
            babelHelpers.defineProperty(this, _this$prop4, 8);
            babelHelpers.defineProperty(this, _this$shadowed4, 9);
          }

          #shadowed() {
            return 6;
          }
        }
      );
    }
  }
);
