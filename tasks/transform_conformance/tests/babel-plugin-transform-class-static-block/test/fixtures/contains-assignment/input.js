let a, b, d, e;

class C {
  static {
    a = this;
  }
  static {
    [b, c] = this;
  }
  static {
    d ??= this;
  }
  static {
    e.f = this;
  }
  static {
    [g.h, i] = this;
  }
}
