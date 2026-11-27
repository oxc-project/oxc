declare let a: any, factory: any, f: any;

@a class A1 {}
@a.b.c class A2 {}
@a.b() class A3 {}
@(factory().decorate) class A4 {}
@(factory().decorate()) class A5 {}
@(factory()()) class A6 {}
@(this.x) class A7 {}
@(a!.b) class A8 {}
@(a["b"]) class A9 {}
@(a?.b) class A10 {}

class B {
  static #x: any;
  @B.#x m1() {}
  @B.#x.y() m2() {}
  @(a.#x.y) m3() {}
  @(f().#x) m4() {}
}

class C extends B {
  m() {
    class D {
      @(super.x) m() {}
    }
  }
}
