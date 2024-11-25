function foo() {}

export class C {
  // Private properties and methods use up prop names for static block
  #_ = 1;
  static #_2 = 2;
  #_3() {}
  static #_4() {}
  accessor #_5 = 5;
  static accessor #_6 = 6;
  
  // Non-private don't use up prop names
  _7 = 7;
  static _8 = 8;
  _9() {}
  static _10() {}

  static {
    foo();
  }
}
