import { k } from "./k";
let n = 0;

class Foo {
  [a()] = 1;
  @dec declare [b()]: any;
  @dec static declare [c()]: any;
  @dec declare [d]: any;
  @dec declare [k]: any;
  @dec declare "e": any;
  [g()] = 2;
}

class Bar {
  @dec declare [h()]: any;
  @dec static declare [i()]: any;
  @dec declare [n++]: any;
}
