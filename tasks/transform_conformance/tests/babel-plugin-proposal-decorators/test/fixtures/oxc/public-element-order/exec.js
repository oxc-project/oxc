// Captured public elements keep the class's property order.
const dec = (value) => value;
const k = "n";
class A {
  m() {}
  accessor x;
  @dec [k]() {}
}
expect(Object.getOwnPropertyNames(A.prototype)).toEqual(["constructor", "m", "x", "n"]);

class B {
  @dec [k]() {}
  m() {}
  accessor x;
  get g() { return 1; }
  @dec set g(value) {}
  static s() {}
  static accessor y;
  @dec static [k]() {}
}
expect(Object.getOwnPropertyNames(B.prototype)).toEqual(["constructor", "n", "m", "x", "g"]);
expect(Object.getOwnPropertyNames(B).filter((name) => /^[sny]$/.test(name))).toEqual(["s", "y", "n"]);
expect(Object.getOwnPropertyDescriptor(B.prototype, "g").get()).toBe(1);
