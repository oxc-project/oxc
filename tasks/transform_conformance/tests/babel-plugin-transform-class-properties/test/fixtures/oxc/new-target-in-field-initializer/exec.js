// `new.target` is `undefined` in field initializers and static blocks, including in nested arrow functions.
class A {
  direct = new.target;
  spaced = new /* comment */ . target;
  arrow = (() => new.target)();
  fn = function () { return new.target; };
  nested = class { [String(new.target)] = 1; };
}
const a = new A();
expect(a.direct).toBeUndefined();
expect(a.spaced).toBeUndefined();
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

// Static initializers and static blocks run outside the class after lowering.
function outer() {
  class C {
    static field = new.target;
    static arrow = (() => new.target)();
    static block;
    static {
      C.block = new.target;
    }
  }
  return C;
}
const C = new outer();
expect(C.field).toBeUndefined();
expect(C.arrow).toBeUndefined();
expect(C.block).toBeUndefined();
