const classes = [];
for (let i = 0; i < 2; i++) {
  class C {
    static #m() { return 42; }
    static has(C = this) { return #m in C; }
    static read(C = this) { return C.#m(); }
    static {
      expect(this.has()).toBe(true);
      expect(this.read()).toBe(42);
      expect(this.has({})).toBe(false);
      expect(() => this.read({})).toThrow(TypeError);
    }
  }
  classes.push(C);
}
for (const C of classes) {
  expect(C.has()).toBe(true);
  expect(C.read()).toBe(42);
}
expect(classes[0].has(classes[1])).toBe(false);
expect(() => classes[0].read(classes[1])).toThrow(TypeError);

class Outer {
  static {
    class C {
      static #m() {}
      static has(C = this) { return #m in C; }
      static { expect(this.has()).toBe(true); }
    }
  }
}
