var _Class;
const foo = { [Symbol.toPrimitive]: () => "foo" };
expect((_Class = class {}, babelHelpers.defineProperty(_Class, foo, 0), _Class).foo).toBe(0);
expect(class {
  static [foo]() {
    return 0;
  }
}.foo()).toBe(0);
expect(class {
  static get [foo]() {
    return 0;
  }
}.foo).toBe(0);
expect(class {
  static set [foo](v) {
    return v;
  }
}.foo = 0).toBe(0);
expect(new class {
  constructor() {
    babelHelpers.defineProperty(this, foo, 0);
  }
}().foo).toBe(0);
const arrayLike = { [Symbol.toPrimitive]: () => [] };
expect(() => {
  var _Class2;
  return _Class2 = class {}, babelHelpers.defineProperty(_Class2, arrayLike, 0), _Class2;
}).toThrow("@@toPrimitive must return a primitive value.");
expect(() => class {
  static [arrayLike]() {
    return 0;
  }
}).toThrow("@@toPrimitive must return a primitive value.");
expect(() => class {
  static get [arrayLike]() {
    return 0;
  }
}).toThrow("@@toPrimitive must return a primitive value.");
expect(() => class {
  static set [arrayLike](v) {
    return v;
  }
}).toThrow("@@toPrimitive must return a primitive value.");
expect(() => new class {
  constructor() {
    babelHelpers.defineProperty(this, arrayLike, 0);
  }
}()).toThrow("@@toPrimitive must return a primitive value.");
