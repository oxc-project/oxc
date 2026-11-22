const classes = [];
for (let i = 0; i < 2; i++) {
  class C {
    static #m() { return 42; }
    static has(C = this) { return #m in C; }
    static read(C = this) { return C.#m(); }
  }
  classes.push(C);
}

for (const C of classes) {
  expect(C.has()).toBe(true);
  expect(C.read()).toBe(42);
  expect(C.name).toBe("C");
}
expect(classes[0].has(classes[1])).toBe(false);
expect(classes[1].has(classes[0])).toBe(false);
expect(() => classes[0].read(classes[1])).toThrow(TypeError);
expect(() => classes[1].read(classes[0])).toThrow(TypeError);

// Static initializers may call a method that uses the lexical alias.
// A nested class with the same name must still use the outer class's brand.
const nestedClasses = [];
for (let i = 0; i < 2; i++) {
  class C {
    static #m() { return 42; }
    static value = this.read();
    static read(C = this) { return C.#m(); }
    static nested() {
      class C {
        static has(value) { return #m in value; }
        static read(value) { return value.#m(); }
      }
      return C;
    }
  }
  nestedClasses.push([C, C.nested()]);
}
for (const [C, Nested] of nestedClasses) {
  expect(C.value).toBe(42);
  expect(C.read()).toBe(42);
  expect(Nested.has(C)).toBe(true);
  expect(Nested.read(C)).toBe(42);
}
expect(nestedClasses[0][1].has(nestedClasses[1][0])).toBe(false);
expect(() => nestedClasses[0][1].read(nestedClasses[1][0])).toThrow(TypeError);
