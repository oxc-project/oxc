// Fields without initializers are removed, parameter property fields are kept.
class Plain {
  p;
  constructor(p) {
    this.p = p;
  }
}
@dec class Decorated {
  p;
  constructor(p) {
    this.p = p;
  }
}
// The parameter property still gets a field when a removed field has its name.
class SameName {
  p;
  constructor(p) {
    this.p = p;
  }
}
class Outer {
  p;
  accessor a = 1;
  constructor(p) {
    this.p = p;
    class Inner {
      q;
      constructor(q) {
        this.q = q;
      }
    }
  }
}

