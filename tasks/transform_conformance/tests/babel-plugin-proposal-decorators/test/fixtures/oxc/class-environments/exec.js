const classes = [];
for (let i = 0; i < 2; i++) {
  classes.push(class {
    @(() => () => i) field;
    @((_, { addInitializer }) => addInitializer(function () { this.extra = i; })) method() {}
  });
}
expect(classes.map(C => { const c = new C(); return [c.field, c.extra]; })).toEqual([[0, 0], [1, 1]]);

const captures = [];
const errors = [];
const capture = fn => {
  captures.push(fn);
  try { fn(); } catch (error) { errors.push(error.constructor); }
  return () => {};
};
class C {
  @capture(() => C) [capture(() => C)];
}
expect(errors).toEqual([ReferenceError, ReferenceError]);
const original = C;
C = null;
expect(captures.map(fn => fn())).toEqual([original, original]);

const privateClasses = [];
for (let i = 0; i < 2; i++) {
  privateClasses.push(class {
    @(() => {}) #field = i;
    @(() => {}) static #staticField = i;
    read(other) { return other.#field; }
    static read() { return this.#staticField; }
  });
}
const first = new privateClasses[0]();
const second = new privateClasses[1]();
expect(first.read(first)).toBe(0);
expect(second.read(second)).toBe(1);
expect(() => first.read(second)).toThrow(TypeError);
expect(privateClasses.map(C => C.read())).toEqual([0, 1]);

const privateMethods = [];
for (let i = 0; i < 2; i++) {
  class C {
    @(() => () => i) #method() {}
    @(() => () => i) static #staticMethod() {}
    call(other) { return other.#method(); }
    static call(other) { return other.#staticMethod(); }
  }
  privateMethods.push(C);
}
const methodFirst = new privateMethods[0]();
const methodSecond = new privateMethods[1]();
expect(methodFirst.call(methodFirst)).toBe(0);
expect(methodSecond.call(methodSecond)).toBe(1);
expect(() => methodFirst.call(methodSecond)).toThrow(TypeError);
expect(privateMethods.map(C => C.call(C))).toEqual([0, 1]);
expect(() => privateMethods[0].call(privateMethods[1])).toThrow(TypeError);

const unbraced = [];
for (let i = 0; i < 2; i++)
  unbraced.push(new (@(() => {}) class {
    #field = i;
    read() { return this.#field; }
  }));
expect(unbraced.map(value => value.read())).toEqual([0, 1]);
