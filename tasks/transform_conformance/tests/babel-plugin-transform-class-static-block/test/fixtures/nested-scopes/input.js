let x, y;

class C {
  static {
    x = (() => this)();
  }

  static {
    if (true) {
      y = this;
      z = this;
    }
  }
}
