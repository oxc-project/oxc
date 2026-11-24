let _y, _k, _k2, _k3, _k4, _k5;
const bound = "b";
class C {
  constructor() {
    this["x"] = 1;
    this[1] = 2;
    this[1n] = 3;
    this[_y] = 4;
    this[bound] = 5;
    this.z = 6;
  }
  "x";
  1;
  1n;
  [_y = y];
  [bound];
  z;
}
class D {
  constructor() {
    this[_k] = 1;
    this[_k4] = 4;
  }
  static {
    _k5 = k("h");
  }
  [_k = k("a")];
  [k("b")];
  static {
    this[_k2] = 2;
  }
  [(_k2 = k("c"), k("d"))];
  [k("e")]() {}
  static {
    this[_k3] = 3;
  }
  [(_k3 = k("f"), _k4 = k("g"))];
  static {
    this[_k5] = 5;
  }
}
