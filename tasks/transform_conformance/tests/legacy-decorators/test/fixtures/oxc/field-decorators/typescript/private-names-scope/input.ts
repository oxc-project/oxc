// Class and constructor parameter decorators still apply when member decorations move into a
// static block.
@cdec
class A {
  #p = 1;
  static self = A;
  @dec((a: A) => a.#p) m() {}
  constructor(@dec x) {}
}

// A private name of an enclosing class doesn't need the static block, where `this` would differ.
class Outer {
  #x = 1;
  make() {
    class Inner {
      @dec(this.#x) m() {}
    }
  }
}
