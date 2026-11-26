{
  "different private/super access in a static class property";

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
        static value = super.idB(o => o.#x);

        static *iter() {
          yield super.idB(C.value);
        }
      }
      b = new originalB();
      yield* originalB.iter();
    }
  }
  const readers = [...A];
  expect(readers.map(fn => fn(b)).join()).toBe("B#x,B#x");
  for (const read of readers) {
    expect(() => read(new A())).toThrow(TypeError);
    expect(() => read(new C())).toThrow(TypeError);
  }
  expect(C.hasOwnProperty("value")).toBe(true);
  expect(originalB.hasOwnProperty("value")).toBe(false);
}

{
  "different private/super access in a static private class property";

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
        static #value = super.idB(o => o.#x);

        static hasValue(receiver) { return #value in receiver; }
        static *iter() {
          yield super.idB(C.#value);
        }
      }
      b = new originalB();
      yield* originalB.iter();
    }
  }
  const readers = [...A];
  expect(readers.map(fn => fn(b)).join()).toBe("B#x,B#x");
  for (const read of readers) {
    expect(() => read(new A())).toThrow(TypeError);
    expect(() => read(new C())).toThrow(TypeError);
  }
  expect(originalB.hasValue(C)).toBe(true);
  expect(originalB.hasValue(originalB)).toBe(false);
}

{
  "different private/super access in a non-decorated static class property";

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
        static [(yield super.idA(o => o.#x), Symbol.iterator)] = super.idB(function* () {
          yield (o => o.#x);
        });
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
  expect(C.hasOwnProperty(Symbol.iterator)).toBe(true);
  expect(originalB.hasOwnProperty(Symbol.iterator)).toBe(false);
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
        static [(yield super.idA(o => o.#x), Symbol.iterator)] = super.idB(function* () {
          yield (o => o.#x);
        });
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
  expect(C.hasOwnProperty(Symbol.iterator)).toBe(true);
  expect(originalB.hasOwnProperty(Symbol.iterator)).toBe(false);
}
