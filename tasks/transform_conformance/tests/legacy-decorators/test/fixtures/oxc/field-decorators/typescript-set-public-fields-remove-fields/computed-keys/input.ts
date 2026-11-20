class A {
  [key("a")]: number;
  @dec [key("b")]: number;
  @dec [key("c")] = 1;
  [key("m")]() {}
  @dec static [key("d")]: number;
  @dec static [key("e")] = 2;
  [key("f")] = 3;
  @dec ["literal"]: number;
}
