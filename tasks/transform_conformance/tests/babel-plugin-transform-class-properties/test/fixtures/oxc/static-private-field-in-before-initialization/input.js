class Foo {
  static {
    values.push(#b in this);
  }
  static #b = 0;
  static {
    values.push(#b in this);
  }
}

class Bar {
  static {
    values.push(#b in (calls++, this));
  }
  static #b;
  static {
    values.push(#b in this);
  }
}

class Throws {
  static {
    #b in null;
  }
  static #b;
}
