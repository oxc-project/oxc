class Singleton {
  static #shared;

  static shared() {
    this.#shared ??= 1;
    return this.#shared;
  }
}

class WithAndOr {
  static #and = 1;
  static #or = 0;

  static run() {
    this.#and &&= 2;
    this.#or ||= 3;
    return [this.#and, this.#or];
  }
}

expect(Singleton.shared()).toBe(1);
expect(WithAndOr.run()).toEqual([2, 3]);
