class value {
  static #_x_accessor_storage = 1;
  static get x() {
    return value.#_x_accessor_storage;
  }
  static set x(_value) {
    value.#_x_accessor_storage = _value;
  }
  #_y_accessor_storage = 1;
  get y() {
    return this.#_y_accessor_storage;
  }
  set y(value) {
    this.#_y_accessor_storage = value;
  }
}
