function dec() {}
const key = Symbol("method");
class C {
  @dec [key]() { return super.value; }
  [key]() { return 2; }
  @dec accessor value = 1;
  accessor value = 2;
  @dec static get x() { return 1; }
  static set x(value) {}
  @dec static get x() { return 2; }
}
