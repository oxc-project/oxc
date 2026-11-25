let value;
class C {
  static #m() { return 42; }
  static has(C = this) { return #m in C; }
  static read(C = this) { return C.#m(); }
  static { value = [this.has(), this.read(), this.has({})]; }
}
