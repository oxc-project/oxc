const bound = "b";
class C {
  "x" = 1;
  1 = 2;
  1n = 3;
  [y] = 4;
  [bound] = 5;
  z = 6;
}

class D {
  [k("a")] = 1;
  [k("b")]: number;
  static [k("c")] = 2;
  [k("d")]: number;
  [k("e")]() {}
  static [k("f")] = 3;
  [k("g")] = 4;
  static [k("h")] = 5;
}
