let _y, _y2, _a, _c;

class Cls {
  y = 1;
  @dce z;
  #x;
  #y = 1;
  @dce #z;
  [x];
  [y] = 1;
  @dce [z];
}

class ClsWithConstructor extends Cls {
  constructor() {
    console.log("before super()");
    super();
    console.log("after super()");
  }
  y = 1;
  @dce z;
  [x];
  [y] = 1;
  @dce [z];
}

class StaticCls {
  static {
    this.y = 1;
  }
  @dce static z;
  static {
    this[_y] = 1;
  }
  @dce static [(x, _y = y, z)];
}

class ClsWithComputedKeyMethods {
  [x()]() {}
  [y()] = 1;
  [z()]() {}
  [a()] = 2;
  [b()];
  [c()] = 3;
  accessor [e()] = 4;
}

class StaticClsWithComputedKeyMethods {
  static [x()]() {}
  static {
    this[_y2] = 1;
  }
  static [(_y2 = y(), z())]() {}
  static {
    this[_a] = 2;
  }
  static {
    this[_c] = 3;
  }
  static accessor [(_a = a(), b(), _c = c(), e())] = 4;
}
