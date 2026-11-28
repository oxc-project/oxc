function dec(method) {
  seen.push(method.call({}).next?.().value ?? method.call({}));
  return function () { return "decorated"; };
}
let seen = [];

class A {
  @dec m() { return "m"; }
  plain() { return "plain"; }
  *[Symbol.iterator]() { yield "iterated"; }
}
expect(seen).toEqual(["m"]);
expect(Object.getOwnPropertyNames(A.prototype)).toEqual(["constructor", "m", "plain"]);
expect([new A().m(), new A().plain(), ...new A()]).toEqual(["decorated", "plain", "iterated"]);

seen = [];
class B {
  @dec *[Symbol.iterator]() { yield "first"; }
  *[Symbol.iterator]() { yield "second"; }
}
expect(seen).toEqual(["first"]);
expect([...new B()]).toEqual(["second"]);

seen = [];
{
  const Symbol = { iterator: "m" };
  class C {
    @dec m() { return "first"; }
    [Symbol.iterator]() { return "second"; }
  }
  expect(seen).toEqual(["first"]);
  expect(new C().m()).toBe("second");
}

seen = [];
class D {
  @dec *[Symbol.iterator]() { yield "iterated"; }
  m() { return "m"; }
}
expect(seen).toEqual(["iterated"]);
expect(Object.getOwnPropertyNames(D.prototype)).toEqual(["constructor", "m"]);
expect([new D().m(), new D()[Symbol.iterator]()]).toEqual(["m", "decorated"]);

seen = [];
Symbol.foo = "m";
class E {
  @dec m() { return "first"; }
  [Symbol.foo]() { return "second"; }
}
delete Symbol.foo;
expect(seen).toEqual(["first"]);
expect(new E().m()).toBe("second");
