class X {
  static #m = () => {};
  method() {
    const o = { X };
    (o.X as typeof X).#m();
    o.X!.#m();
    o.X.#m();
  }
}
