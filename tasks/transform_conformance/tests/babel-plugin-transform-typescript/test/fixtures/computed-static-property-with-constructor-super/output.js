let _x, _y;
class Sub extends Base {
  static {
    _x = x(), _y = y();
  }
  static {
    this[_x] = 1;
  }
  static {
    this[_y] = 2;
  }
  constructor(a) {
    super(a);
  }
}
