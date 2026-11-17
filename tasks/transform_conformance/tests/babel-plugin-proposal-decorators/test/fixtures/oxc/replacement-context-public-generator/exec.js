{
  let originalB;
  "different super access in a non-decorated static method";

  const createClassWithStaticX = (hint) => {
    return class {
      static accessor ["x" + hint] = hint + "Super" + ".x";
    }
  }

  class C extends createClassWithStaticX("C") { static accessor xC = "C.x"; }

  const dec = (b) => {
    originalB = b;
    return C;
  }

  class A extends createClassWithStaticX("A") {
    static accessor xA = "A.x";
    static *[Symbol.iterator]() {
      @dec
      class B extends createClassWithStaticX("B") {
        static accessor xB = "B.x";
        static *method({ superX = super.xB, x = this.xB, [expect(super.xC).toBe(undefined)]: _ } = {}) {
          expect(superX).toBe(super.xB);
          expect(x).toBe(this.xB);
          yield superX;
          yield x;
        };

        static *[(yield super.xA, yield this.xA, Symbol.iterator)]() {
          yield super.xB;
          yield this.xB;
          yield* originalB.method();
        }
      }
      yield* originalB;
    }
  }
  expect([...A].join()).toBe("ASuper.x,A.x,BSuper.x,B.x,BSuper.x,B.x");
}
