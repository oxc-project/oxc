// Fields without initializers are removed, parameter property fields are kept.
class Plain {
  x: number;
  constructor(public p: number) {}
}

@dec
class Decorated {
  x: number;
  constructor(public p: number) {}
}

// The parameter property still gets a field when a removed field has its name.
class SameName {
  p: number;
  constructor(public p: number) {}
}

class Outer {
  accessor a = 1;
  constructor(public p: number) {
    class Inner {
      constructor(private q: number) {}
    }
  }
}
