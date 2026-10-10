class A {
  static #cache = null;
  static get() {
    if (this.#cache !== null) return this.#cache;
    const A = 'local';
    return A;
  }
}
