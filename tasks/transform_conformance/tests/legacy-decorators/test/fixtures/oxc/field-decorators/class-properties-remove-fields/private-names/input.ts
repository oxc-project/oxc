class A {
  #p = 1;
  @dec(#p in A) a: number;
  @dec((a: A) => a.#p) b = 1;
  @dec((a: A) => a.#p) m() {}
  n(@dec(#p in A) param) {}
}
