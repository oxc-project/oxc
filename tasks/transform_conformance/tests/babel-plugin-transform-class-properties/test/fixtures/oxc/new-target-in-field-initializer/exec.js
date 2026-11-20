// `new.target` is `undefined` in field initializers, including in nested arrow functions.
class A {
  direct = new.target;
  arrow = (() => new.target)();
  fn = function () { return new.target; };
  nested = class { [String(new.target)] = 1; };
}
const a = new A();
expect(a.direct).toBeUndefined();
expect(a.arrow).toBeUndefined();
expect(new a.fn()).not.toBeUndefined();
expect(Object.keys(new a.nested())).toEqual(["undefined"]);

class B extends A {
  constructor() {
    super();
    this.target = new.target;
  }
}
expect(new B().target).toBe(B);
expect(new B().direct).toBeUndefined();
