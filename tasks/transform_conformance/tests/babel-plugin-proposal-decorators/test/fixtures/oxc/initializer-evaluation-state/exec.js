// Each evaluation of a decorated class in a parameter or field initializer has its own
// decorator state.
const tag = (value) => () => () => value;

function f(v, C = class { @tag(v) x = 0; }) {
  return C;
}
const A = f(1), B = f(2);
expect([new A().x, new B().x]).toEqual([1, 2]);
expect(A.name).toBe("C");

let n = 0;
class P {
  y = class { @tag(++n) x = 0; };
  z = [class { @tag(n * 10) x = 0; }];
  static s = class { @tag("static") x = 0; };
}
const p1 = new P(), p2 = new P();
expect([new p1.y().x, new p2.y().x]).toEqual([1, 2]);
expect([new p1.z[0]().x, new p2.z[0]().x]).toEqual([10, 20]);
expect(p1.y.name).toBe("y");
expect(new P.s().x).toBe("static");

// `this` and `super` keep their meaning inside the wrapper.
class Base { base() { return "base"; } }
class Derived extends Base {
  value = "derived";
  C = class { @tag(this.value + ":" + super.base()) x = 0; };
}
expect(new new Derived().C().x).toBe("derived:base");
function g(C = class { @tag(arguments.length) x = 0; }) { return C; }
expect(new (g())().x).toBe(0);
