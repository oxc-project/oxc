const x = "field";
class C {
  field = x;
  constructor() {
    try {
      throw "outer";
    } catch (x) {
      try {
        throw "inner";
      } catch (x) {
        var x = "local";
        this.inner = x;
      }
      var x;
      this.caught = x;
    }
    try {
      throw "sibling";
    } catch (x) {
      var x;
      this.sibling = x;
    }
    this.outer = x;
  }
}
class D extends C {
  field = x;
  constructor(x = "parameter") {
    super();
    try {
      throw "caught";
    } catch (x) {
      var x = "local";
      this.caught = x;
    }
    this.outer = x;
  }
}
