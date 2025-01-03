var _C;
class C {}
_C = C;
babelHelpers.defineProperty(C, "prop", () => {
  // Transform
  babelHelpers.superPropGet(_C, "prop", _C);
  babelHelpers.superPropGet(_C, prop, _C);
  babelHelpers.superPropGet(_C, "prop", _C, 2)([]);
  babelHelpers.superPropGet(_C, prop, _C, 2)([]);

  const obj = {
    method() {
      // Don't transform
      super.prop;
      super[prop];
      super.prop();
      super[prop]();
    }
  };

  class Inner {
    method() {
      // Don't transform
      super.prop;
      super[prop];
      super.prop();
      super[prop]();
    }

    static staticMethod() {
      // Don't transform
      super.prop;
      super[prop];
      super.prop();
      super[prop]();
    }
  }
});
