var _Outer;

const ident = "A";

class Outer {}

_Outer = Outer;
babelHelpers.defineProperty(Outer, "B", () => {
  // Transform
  babelHelpers.superPropSet(_Outer, "A", 1, _Outer, 1);
  babelHelpers.superPropSet(_Outer, "A", babelHelpers.superPropGet(_Outer, "A", _Outer) + 1, _Outer, 1);
  babelHelpers.superPropSet(_Outer, "A", babelHelpers.superPropGet(_Outer, "A", _Outer) - 1, _Outer, 1);
  babelHelpers.superPropGet(_Outer, "A", _Outer) && babelHelpers.superPropSet(_Outer, "A", 1, _Outer, 1);
  babelHelpers.superPropGet(_Outer, "A", _Outer) || babelHelpers.superPropSet(_Outer, "A", 1, _Outer, 1);

  babelHelpers.superPropSet(_Outer, ident, 1, _Outer, 1);
  babelHelpers.superPropSet(_Outer, ident, babelHelpers.superPropGet(_Outer, ident, _Outer) + 1, _Outer, 1);
  babelHelpers.superPropSet(_Outer, ident, babelHelpers.superPropGet(_Outer, ident, _Outer) - 1, _Outer, 1);
  babelHelpers.superPropGet(_Outer, ident, _Outer) && babelHelpers.superPropSet(_Outer, ident, 1, _Outer, 1);
  babelHelpers.superPropGet(_Outer, ident, _Outer) || babelHelpers.superPropSet(_Outer, ident, 1, _Outer, 1);

  class Inner {
    method() {
      // Don't transform
      super.A = 1;
      super.A += 1;
      super.A -= 1;
      super.A &&= 1;
      super.A ||= 1;

      super[ident] = 1;
      super[ident] += 1;
      super[ident] -= 1;
      super[ident] &&= 1;
      super[ident] ||= 1;
    }

    static staticMethod() {
      // Don't transform
      super.A = 1;
      super.A += 1;
      super.A -= 1;
      super.A &&= 1;
      super.A ||= 1;

      super[ident] = 1;
      super[ident] += 1;
      super[ident] -= 1;
      super[ident] &&= 1;
      super[ident] ||= 1;
    }
  }
});
