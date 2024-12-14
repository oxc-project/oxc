class A {
  static #a = 33;
  b = 44;
  method() {
    (undefined, this)?.#a;
    (undefined, this)?.b;
  }
}
