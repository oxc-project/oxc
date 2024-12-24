class C {
  static {
    this.a();
  }
}

class C2 {
  static {
    C2.b();
  }
}

class C3 {
  static {
    this.c();
    C3.d();
  }
}

let C4 = class C {
  static {
    this.e();
  }
};

let C5 = class C {
  static {
    this.f();
    C5.g();
  }
  static {
    this.h();
  }
};

class Nested {
  static {
    this.i = () => this.j();
    function inner() {
      return [this, Nested];
    }
    otherIdent;
  }
}
