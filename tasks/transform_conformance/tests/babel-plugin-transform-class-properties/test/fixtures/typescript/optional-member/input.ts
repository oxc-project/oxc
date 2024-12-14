class X {
  static #a = 0;
  method() {
    const o = { X };
    (o.X as typeof X).#a;
    o.X!.#a;
    o.X.#a;
  }
}
