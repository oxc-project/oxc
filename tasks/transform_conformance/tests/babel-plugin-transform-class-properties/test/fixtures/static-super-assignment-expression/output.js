var _unbound, _unbound2, _unbound3, _unbound4, _Outer;

let bound = "A";

class Outer {}

_Outer = Outer;
babelHelpers.defineProperty(Outer, "B", () => {
  // Transform
  babelHelpers.superPropSet(_Outer, "A", 1, _Outer, 1);
  babelHelpers.superPropSet(_Outer, "A", babelHelpers.superPropGet(_Outer, "A", _Outer) + 1, _Outer, 1);
  babelHelpers.superPropSet(_Outer, "A", babelHelpers.superPropGet(_Outer, "A", _Outer) - 1, _Outer, 1);
  babelHelpers.superPropGet(_Outer, "A", _Outer) && babelHelpers.superPropSet(_Outer, "A", 1, _Outer, 1);
  babelHelpers.superPropGet(_Outer, "A", _Outer) || babelHelpers.superPropSet(_Outer, "A", 1, _Outer, 1);

  babelHelpers.superPropSet(_Outer, bound, 1, _Outer, 1);
  babelHelpers.superPropSet(_Outer, bound, babelHelpers.superPropGet(_Outer, bound, _Outer) + 1, _Outer, 1);
  babelHelpers.superPropSet(_Outer, bound, babelHelpers.superPropGet(_Outer, bound, _Outer) - 1, _Outer, 1);
  babelHelpers.superPropGet(_Outer, bound, _Outer) && babelHelpers.superPropSet(_Outer, bound, 1, _Outer, 1);
  babelHelpers.superPropGet(_Outer, bound, _Outer) || babelHelpers.superPropSet(_Outer, bound, 1, _Outer, 1);

  babelHelpers.superPropSet(_Outer, unbound, 1, _Outer, 1);
  babelHelpers.superPropSet(_Outer, _unbound = unbound, babelHelpers.superPropGet(_Outer, _unbound, _Outer) + 1, _Outer, 1);
  babelHelpers.superPropSet(_Outer, _unbound2 = unbound, babelHelpers.superPropGet(_Outer, _unbound2, _Outer) - 1, _Outer, 1);
  babelHelpers.superPropGet(_Outer, _unbound3 = unbound, _Outer) && babelHelpers.superPropSet(_Outer, _unbound3, 1, _Outer, 1);
  babelHelpers.superPropGet(_Outer, _unbound4 = unbound, _Outer) || babelHelpers.superPropSet(_Outer, _unbound4, 1, _Outer, 1);

  class Inner {
    method() {
      // Don't transform
      super.A = 1;
      super.A += 1;
      super.A -= 1;
      super.A &&= 1;
      super.A ||= 1;

      super[bound] = 1;
      super[bound] += 1;
      super[bound] -= 1;
      super[bound] &&= 1;
      super[bound] ||= 1;

      super[unbound] = 1;
      super[unbound] += 1;
      super[unbound] -= 1;
      super[unbound] &&= 1;
      super[unbound] ||= 1;
    }

    static staticMethod() {
      // Don't transform
      super.A = 1;
      super.A += 1;
      super.A -= 1;
      super.A &&= 1;
      super.A ||= 1;

      super[bound] = 1;
      super[bound] += 1;
      super[bound] -= 1;
      super[bound] &&= 1;
      super[bound] ||= 1;

      super[unbound] = 1;
      super[unbound] += 1;
      super[unbound] -= 1;
      super[unbound] &&= 1;
      super[unbound] ||= 1;
    }
  }
});
