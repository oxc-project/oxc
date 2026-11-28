{
  let originalB;
  "different super access in a static private method";

  const dummy = () => {}

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
        @(yield super.xA, yield this.xA, dummy)
        static *#method({ superX = super.xB, x = this.xC, [expect(super.xC).toBeUndefined()]: _ } = {}) {
          expect(superX).toBe("BSuper.x");
          expect(x).toBe(this.xC);
          yield superX;
          yield x;
        };

        static *iter() {
          yield super.xB;
          yield this.xB;
          yield* C.#method();
        }

        static *read(receiver) {
          yield* originalB.#method.call(receiver);
        }
      }
      yield* originalB.iter();
    }
  }
  const iterator = A[Symbol.iterator]();
  expect(iterator.next()).toEqual({ value: "ASuper.x", done: false });
  expect(iterator.next()).toEqual({ value: "A.x", done: false });
  expect(iterator.next()).toEqual({ value: "BSuper.x", done: false });
  expect(iterator.next()).toEqual({ value: "B.x", done: false });
  // Static private methods remain installed on the original constructor.
  expect(() => iterator.next()).toThrow(TypeError);
  // Reach the generator body with a branded lookup and distinct call receivers.
  expect([...originalB.read(C)]).toEqual(["BSuper.x", "C.x"]);
  expect([...originalB.read(originalB)]).toEqual(["BSuper.x", undefined]);
}
