class Methods {
  #method() {}
  static #staticMethod() {}

  run(object, rhs) {
    object.#method ||= rhs();
    object.#method ??= rhs();
    object.#method &&= rhs();
    object.#method += rhs();
    ++object.#method;
    object.#method--;
  }

  static run(object, rhs) {
    object.#staticMethod ||= rhs();
    object.#staticMethod ??= rhs();
    object.#staticMethod &&= rhs();
    object.#staticMethod += rhs();
    ++object.#staticMethod;
    object.#staticMethod--;
  }
}
