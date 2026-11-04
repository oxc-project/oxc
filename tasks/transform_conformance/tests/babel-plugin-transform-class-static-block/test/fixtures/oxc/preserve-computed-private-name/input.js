const key = Symbol("method");
class A {
  static {}
  static [key] = function () {};
  static {}
  static #f = () => {};
  static getName() { return this.#f.name; }
}
