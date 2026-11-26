class A {
  accessor x = 1;
  #y = 2;
  z = 3;
  static s = 4;
  getY() { return this.#y; }
}
