class Hello {
  constructor() {
    this.input = { foo };
    this.#_util_accessor_storage = this.input.foo();
  }
  #_util_accessor_storage;
  get util() {
    return this.#_util_accessor_storage;
  }
  set util(value) {
    this.#_util_accessor_storage = value;
  }
}
class AccessorOnly {
  #_y_accessor_storage = 2;
  get y() {
    return this.#_y_accessor_storage;
  }
  set y(value) {
    this.#_y_accessor_storage = value;
  }
}
