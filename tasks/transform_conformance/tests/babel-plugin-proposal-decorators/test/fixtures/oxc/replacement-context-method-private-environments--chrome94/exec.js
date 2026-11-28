{
  "different private/super access in a static private method";

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
        @(yield super.idA(o => o.#x), dummy)
        static *#iter() {
          yield super.idB(o => o.#x);
        }
        static bad() { return C.#iter(); }
        static *iter() {
          yield* super.idB(originalB.#iter());
        }
      }
      b = new originalB();
      yield* originalB.iter();
    }
  }
  const readers = [...A];
  expect(() => originalB.bad()).toThrow(TypeError);
  expect(readers.map(fn => fn(b)).join()).toBe("B#x,B#x");
  for (const read of readers) {
    expect(() => read(new A())).toThrow(TypeError);
    expect(() => read(new C())).toThrow(TypeError);
  }
}

{
  "different private/super access in a non-decorated static method";

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
        static [(yield super.idA(o => o.#x), Symbol.iterator)] = function* () {
          yield (o => o.#x);
        }
      }
      b = new originalB();
      yield* B;
    }
  }
  const readers = [...A];
  expect(readers.map(fn => fn(b)).join()).toBe("B#x,B#x");
  for (const read of readers) {
    expect(() => read(new A())).toThrow(TypeError);
    expect(() => read(new C())).toThrow(TypeError);
  }
}

{
  "different private/super access in a decorated static method";

  const dummy = () => {};

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
        @(yield super.idA(o => o.#x), dummy)
        static [(yield super.idA(o => o.#x), Symbol.iterator)] = function* () {
          yield (o => o.#x);
        }
      }
      b = new originalB();
      yield* B;
    }
  }
  const readers = [...A];
  expect(readers.map(fn => fn(b)).join()).toBe("B#x,B#x,B#x");
  for (const read of readers) {
    expect(() => read(new A())).toThrow(TypeError);
    expect(() => read(new C())).toThrow(TypeError);
  }
}
