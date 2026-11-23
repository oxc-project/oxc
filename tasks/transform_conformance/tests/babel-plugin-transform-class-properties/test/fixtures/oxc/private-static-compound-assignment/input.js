class Foo {
  static #prop = 1;

  static test() {
    this.#prop += 1;
    getObj().#prop -= 2;
  }
}
