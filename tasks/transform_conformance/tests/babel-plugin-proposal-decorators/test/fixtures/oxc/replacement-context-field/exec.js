{
  let originalB;
  "different super access in a static class property";

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
        @(yield super.xA, expect(this.xA).toBe("A.x"), dummy)
        static [(yield this.xA, expect(super.xA).toBe("ASuper.x"), "value")] = [super.xC, this.xC];

        static *iter() {
          yield super.xB;
          yield this.xB;
          yield* C.value;
        }
      }
      yield* originalB.iter();
    }
  }
  // The initializer keeps B's home object, while its receiver is the replacement C.
  expect([...A]).toEqual(["ASuper.x", "A.x", "BSuper.x", "B.x", undefined, "C.x"]);
}
