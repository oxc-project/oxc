class C {
  [a ?? b] = c ?? d;
  static [e ?? f] = g ?? h;
  static {
    i ?? j;
  }
}

class C2 extends S {
  prop = k ?? l;
  constructor() {
    if (true) {
      super();
    }
  }
}
