class Foo {
  [a()] = 1;
  @dec declare [b()]: any;
  @dec static declare [c()]: any;
  [g()] = 2;
}

class Bar {
  @dec declare [h()]: any;
}
