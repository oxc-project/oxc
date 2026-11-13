class A {
  #_x_accessor_storage = 1;
  get x() { return this.#_x_accessor_storage; }
  set x(value) { this.#_x_accessor_storage = value; }
  #y = 2;
  z = 3;
  static s = 4;
  getY() { return this.#y; }
}
