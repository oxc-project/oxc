@dec
class C {
  @dec #field = C;
  @dec #method() { return this.#field; }
  @dec static #staticField = 1;
  @dec static #staticMethod() { return this.#staticField; }
  @dec accessor #accessor = 2;
  read() { return [this.#method(), this.#accessor, C.#staticMethod()]; }
}
