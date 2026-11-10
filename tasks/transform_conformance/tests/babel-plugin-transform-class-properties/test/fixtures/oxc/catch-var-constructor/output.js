const x = "field";
class C {
  constructor() {
    babelHelpers.defineProperty(this, "field", x);
    try {
      throw "outer";
    } catch (_x) {
      try {
        throw "inner";
      } catch (_x) {
        var _x = "local";
        this.inner = _x;
      }
      var _x;
      this.caught = _x;
    }
    try {
      throw "sibling";
    } catch (_x) {
      var _x;
      this.sibling = _x;
    }
    this.outer = _x;
  }
}
class D extends C {
  constructor(_x2 = "parameter") {
    super();
    babelHelpers.defineProperty(this, "field", x);
    try {
      throw "caught";
    } catch (_x2) {
      var _x2 = "local";
      this.caught = _x2;
    }
    this.outer = _x2;
  }
}
