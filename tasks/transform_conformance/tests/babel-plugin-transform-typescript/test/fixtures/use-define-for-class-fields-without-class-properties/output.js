
let _y, _y2, _y3, _y4, _a, _c, _y5, _a2, _c2;

class Cls {
  constructor() {
    this.y = 1;
    this[_y] = 1;
  }
  static {
    x, _y = y, z;
  }
  #x;
  #y = 1;
  @dce #z;
}

class ClsWithConstructor extends Cls {
  static {
    x, _y2 = y, z;
  }
  constructor() {
    console.log("before super()");
    super();
    this.y = 1;
    this[_y2] = 1;
    console.log("after super()");
  }
}

class StaticCls {
  static {
    x, _y3 = y, z;
  }
  static {
    this.y = 1;
  }
  static {
    this[_y3] = 1;
  }
}

class ClsWithComputedKeyMethods {
  constructor() {
    this[_y4] = 1;
    this[_a] = 2;
    this[_c] = 3;
  }
  [x()]() { }
  [(_y4 = y(), z())]() { }
  accessor [(_a = a(), b(), _c = c(), e())] = 4;
}

class StaticClsWithComputedKeyMethods {
  static [x()]() { }
  static {
    this[_y5] = 1;
  }
  static [(_y5 = y(), z())]() { }
  static {
    this[_a2] = 2;
  }
  static {
    this[_c2] = 3;
  }
  static accessor [(_a2 = a(), b(), _c2 = c(), e())] = 4;
}
