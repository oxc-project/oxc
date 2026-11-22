class Cls {
  y = 1;
  @dce
  z;
}

class StaticCls {
  static {
    this.y = 1;
  }
  @dce
  static z;
}
