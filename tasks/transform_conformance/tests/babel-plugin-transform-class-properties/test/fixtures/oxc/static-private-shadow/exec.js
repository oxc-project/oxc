class A {
  static #cache = "cached";
  static get() {
    if (this.#cache !== null) return this.#cache;
    const A = 'local';
    return A;
  }
}

expect(A.get()).toBe("cached");
