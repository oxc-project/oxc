import { dec } from "dec";

@dec
class A {
  #a = 1;
  @(#a in A ? dec : dec) m() {}
}

@dec
class B {
  #b = 1;
  constructor(@dec p: any) {}
  @(#b in B ? dec : dec) m() {}
}

@dec
export class C {
  #c = 1;
  @(#c in C ? dec : dec) m() {}
}

@dec
export default class D {
  #d = 1;
  @(#d in D ? dec : dec) m() {}
}
