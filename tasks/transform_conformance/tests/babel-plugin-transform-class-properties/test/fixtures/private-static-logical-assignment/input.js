class Foo {
  static #nullish;
  static #and = 1;
  static #or;

  static test() {
    this.#nullish ??= 1;
    this.#and &&= 2;
    this.#or ||= 3;
  }
}
