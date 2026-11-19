
let _y;
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
  static {
    _y = y;
  }
  "x";
  1;
  1n;
  [_y];
  [bound];
  z;
}

