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
const c = new C();
expect([c.field, c.inner, c.caught, c.sibling, c.outer]).toEqual([
  "field",
  "local",
  "outer",
  "sibling",
  undefined,
]);
const d = new D();
expect([d.field, d.caught, d.outer]).toEqual(["field", "local", "parameter"]);
