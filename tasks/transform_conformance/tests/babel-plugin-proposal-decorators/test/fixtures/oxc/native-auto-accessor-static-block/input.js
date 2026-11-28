function keep(value) { return value; }

@keep
class Decorated {
  static #value = 1;
  static { this.value = this.#value; }
}

class Plain {
  static #value = 2;
  static { this.value = this.#value; }
}

class Accessor {
  accessor value = 3;
  static #value = 4;
  static { this.other = this.#value; }
}
