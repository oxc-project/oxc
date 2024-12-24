var _super$A, _super$A2, _super$A3, _super$A4, _super$A5, _super$A6, _super$bound, _super$bound2, _super$bound3, _super$bound4, _super$bound5, _super$bound6, _unbound, _super$unbound, _super$unbound2, _unbound2, _super$unbound3, _super$unbound4, _unbound3, _super$unbound5, _unbound4, _super$unbound6, _Outer;

let bound = "A";

class Outer {}

_Outer = Outer;
babelHelpers.defineProperty(Outer, "B", () => {
  // Transform
  babelHelpers.superPropSet(_Outer, "A", (_super$A = babelHelpers.superPropGet(_Outer, "A", _Outer), _super$A2 = _super$A++, _super$A), _Outer, 1), _super$A2;
  babelHelpers.superPropSet(_Outer, "A", (_super$A3 = babelHelpers.superPropGet(_Outer, "A", _Outer), _super$A4 = _super$A3--, _super$A3), _Outer, 1), _super$A4;
  babelHelpers.superPropSet(_Outer, "A", (_super$A5 = babelHelpers.superPropGet(_Outer, "A", _Outer), ++_super$A5), _Outer, 1);
  babelHelpers.superPropSet(_Outer, "A", (_super$A6 = babelHelpers.superPropGet(_Outer, "A", _Outer), --_super$A6), _Outer, 1);

  babelHelpers.superPropSet(_Outer, bound, (_super$bound = babelHelpers.superPropGet(_Outer, bound, _Outer), _super$bound2 = _super$bound++, _super$bound), _Outer, 1), _super$bound2;
  babelHelpers.superPropSet(_Outer, bound, (_super$bound3 = babelHelpers.superPropGet(_Outer, bound, _Outer), _super$bound4 = _super$bound3--, _super$bound3), _Outer, 1), _super$bound4;
  babelHelpers.superPropSet(_Outer, bound, (_super$bound5 = babelHelpers.superPropGet(_Outer, bound, _Outer), ++_super$bound5), _Outer, 1);
  babelHelpers.superPropSet(_Outer, bound, (_super$bound6 = babelHelpers.superPropGet(_Outer, bound, _Outer), --_super$bound6), _Outer, 1);

  babelHelpers.superPropSet(_Outer, _unbound = unbound, (_super$unbound = babelHelpers.superPropGet(_Outer, _unbound, _Outer), _super$unbound2 = _super$unbound++, _super$unbound), _Outer, 1), _super$unbound2;
  babelHelpers.superPropSet(_Outer, _unbound2 = unbound, (_super$unbound3 = babelHelpers.superPropGet(_Outer, _unbound2, _Outer), _super$unbound4 = _super$unbound3--, _super$unbound3), _Outer, 1), _super$unbound4;
  babelHelpers.superPropSet(_Outer, _unbound3 = unbound, (_super$unbound5 = babelHelpers.superPropGet(_Outer, _unbound3, _Outer), ++_super$unbound5), _Outer, 1);
  babelHelpers.superPropSet(_Outer, _unbound4 = unbound, (_super$unbound6 = babelHelpers.superPropGet(_Outer, _unbound4, _Outer), --_super$unbound6), _Outer, 1);

  class Inner {
    method() {
      // Don't transform
      super.A++;
      super.A--;
      ++super.A;
      --super.A;

      super[bound]++;
      super[bound]--;
      ++super[bound];
      --super[bound];

      super[unbound]++;
      super[unbound]--;
      ++super[unbound];
      --super[unbound];
    }

    static staticMethod() {
      // Don't transform
      super.A++;
      super.A--;
      ++super.A;
      --super.A;

      super[bound]++;
      super[bound]--;
      ++super[bound];
      --super[bound];

      super[unbound]++;
      super[unbound]--;
      ++super[unbound];
      --super[unbound];
    }
  }
});
