let bound, bound2, bound3;

class C {
  constructor(_bound, x, y) {
    babelHelpers.defineProperty(this, "clash1", bound);
    babelHelpers.defineProperty(this, "clash2", unbound);
    babelHelpers.defineProperty(this, "noClash1", bound2);
    babelHelpers.defineProperty(this, "noClash2", unbound2);
    babelHelpers.defineProperty(this, "noClash3", bound3);
    babelHelpers.defineProperty(this, "noClash4", unbound3);
    babelHelpers.defineProperty(this, "noClash5", (x) => x);
    babelHelpers.defineProperty(this, "noClash6", () => {
      let y;
      return y;
    });
    {
      var _unbound;
      let bound3, unbound3;
    }
    console.log(_bound, _unbound, x, y);
  }
}
