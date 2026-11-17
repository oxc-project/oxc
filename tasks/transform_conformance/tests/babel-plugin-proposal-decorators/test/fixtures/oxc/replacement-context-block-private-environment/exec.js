{
  "different private/super access in a static block";

  const dummy = () => {}

  let b, originalB;

  const idFactory = (hint) => {
    return class { static ["id" + hint](v) { return v } }
  }

  class C extends idFactory("C") { #x = "C#x"; }

  const dec = (b) => {
    originalB = b;
    return C;
  }

  class A extends idFactory("A") {
    #x = "A#x";
    static *[Symbol.iterator]() {
      @dec
      class B extends idFactory("B") {
        #x = "B#x";
        static {
          this.value = super.idB(o => o.#x);
        }

        static *iter() {
          yield super.idB(C.value);
        }
      }
      b = new originalB();
      yield* originalB.iter();
    }
  }
  const readers = [...A];
  expect(readers.map(fn => fn(b)).join()).toBe("B#x");
  for (const read of readers) {
    expect(() => read(new A())).toThrow(TypeError);
    expect(() => read(new C())).toThrow(TypeError);
  }
}
