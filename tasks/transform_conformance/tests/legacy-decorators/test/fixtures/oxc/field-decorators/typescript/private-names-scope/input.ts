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

// A static block nested in a decorator doesn't end the class's own static block.
@cdec
class B {
  #p = 1;
  @dec(class { static {} }, B, #p in Object) m() {}
}

// The decorators and heritage of a nested class can refer to the outer private name.
class C {
  #x = 1;
  @dec(class extends (#x in Object ? A : B) { #x = 2 }) m() {}
}

// An anonymous class gets an inner name for its static block to refer to.
export default @cdec class {
  #p = 1;
  @dec(#p in Object) m() {}
}
