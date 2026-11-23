function dec(Class: any) {
  return class extends Class {};
}

@dec
class Foo {
  static #p = 1;
  static #m() {}
  static getP() {
    return this.#p;
  }
  static callM() {
    return this.#m();
  }
  static hasP(obj: any) {
    return #p in obj;
  }
  static self() {
    return Foo;
  }
}

@dec
export default class {
  static #p = 1;
  static getP() {
    return this.#p;
  }
}
