import { dec } from "dec";

@dec
export class Cls {
  #zoo = 0;
  @dec(#zoo in Cls)
  foo() {}
}

@dec
export class Cls2 {
  #zoo = 0;
  foo(@dec(#zoo in Cls2) param: number) {}
}
