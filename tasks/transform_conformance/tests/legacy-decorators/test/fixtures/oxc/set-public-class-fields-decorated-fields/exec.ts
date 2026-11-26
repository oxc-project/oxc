const log: string[] = [];
function key(name: string) {
  log.push(`key:${name}`);
  return name;
}
function dec(target: any, prop: string) {
  if (typeof target === "function") {
    log.push(`static:${prop}=${target[prop]}`);
  } else {
    log.push(`proto:${prop}`);
  }
}

class A {
  @dec static y = 1;
  @dec z = 3;
  static [key("a")] = 4;
  @dec static [key("b")] = 5;
  @dec [key("c")] = 6;
  @dec static w = this.y + 10;
  @dec static v: any;
  @dec u: any;
  [key("e")] = 7;
  @dec static [key("f")]: any;
  @dec "q" = 8;
  @dec static 9 = 10;
}

class Base {}
class B extends Base {
  @dec static x = 1;
  @dec y = 2;
  constructor() {
    super();
  }
}

const a = new A() as any;
const b = new B();
expect([A.y, (A as any).a, (A as any).b, A.w]).toEqual([1, 4, 5, 11]);
expect([a.z, a.c, a.e, a.q, (A as any)[9]]).toEqual([3, 6, 7, 8, 10]);
expect([B.x, b.y]).toEqual([1, 2]);
expect(log).toEqual([
  "key:a",
  "key:b",
  "key:c",
  "key:e",
  "key:f",
  "proto:z",
  "proto:c",
  "proto:u",
  "proto:q",
  "static:y=1",
  "static:b=5",
  "static:w=11",
  "static:v=undefined",
  "static:f=undefined",
  "static:9=10",
  "proto:y",
  "static:x=1",
]);

// A decorated static field is assigned, not defined, like an undecorated one: it calls an
// inherited setter and can't overwrite the class's read-only `name`.
const assigned: number[] = [];
class SetterBase {
  static set s(value: number) {
    assigned.push(value);
  }
  static set t(value: number) {
    assigned.push(value);
  }
}
class SetterChild extends SetterBase {
  @dec static s = 1;
  @dec static [key("t")] = 2;
}
expect(assigned).toEqual([1, 2]);
expect([Object.hasOwn(SetterChild, "s"), Object.hasOwn(SetterChild, "t")]).toEqual([false, false]);
expect(() => {
  class Named {
    @dec static name = "custom";
  }
  return Named;
}).toThrow(TypeError);

// A decorator that returns a descriptor replaces the static field defined before it, and a class
// decorator runs after the member decorators.
const seen: string[] = [];
function accessor(target: any, prop: string): any {
  seen.push(`accessor:${prop}=${target[prop]}`);
  return { get: () => `get:${prop}`, configurable: true, enumerable: true };
}
function named(target: any) {
  seen.push(`class:${target.name}`);
}
function tKey() {
  return "t";
}

@named
class C {
  @accessor static s = 1;
  @accessor static [tKey()] = 2;
  static r = C.name;
}

expect([C.s, (C as any).t, C.r]).toEqual(["get:s", "get:t", "C"]);
expect(seen).toEqual(["accessor:s=1", "accessor:t=2", "class:C"]);
